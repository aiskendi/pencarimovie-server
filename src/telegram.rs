//! Telegram transport on `grammers`.
//!
//! Live `Client`s are kept per bot and reused for every Range request — there is
//! no out-of-process worker and no IPC layer, because grammers is a library.
//! What matters here:
//!
//! * a session that survives restarts (a JSON file implementing `Session`),
//! * a login path of its own (`auth.ImportBotAuthorization` via
//!   `Client::bot_sign_in`, never a copied session),
//! * a download primitive built on `iter_download()` + `chunk_size` /
//!   `skip_chunks` (the per-connection Telegram primitive),
//! * `FILE_REFERENCE_EXPIRED` surfaced as a typed error so the caller can
//!   re-resolve once.
//!
//! ## One connection per DC per Client
//! grammers multiplexes every request for a DC over that DC's connection. To
//! get Telegram's recommended "separate connections" model, `--workers N`
//! creates N independent `Client`s (each with its own `SenderPool`, therefore
//! its own connection set), and requests pick one round-robin. That is the
//! "N Clients" option the task allows; run N *processes* instead if you want
//! hard isolation (measured in README.md).

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use grammers_client::media::{Downloadable, InputAudio, Media};
use grammers_client::message::InputMessage;
use grammers_client::{Client, InvocationError, SenderPool};
use grammers_session::types::{
    ChannelState, DcOption, PeerId, PeerInfo, PeerRef, UpdateState, UpdatesState,
};
use grammers_session::{BoxFuture as SessFuture, Session, SessionData};
use grammers_tl_types as tl;
use serde::{Deserialize, Serialize};

use crate::credentials::{fetch_credentials_from_wordpress, load_cached_api_credentials};
use crate::fileid::{DecodedFileId, is_document_file_id, location_kind};
use crate::log::log;
use crate::storage::storage_path;

/// The session file, kept per bot in `storage/sessions/<bot_id>/` with a name
/// that cannot be mistaken for a session written by the old PHP server.
pub const SESSION_FILE: &str = "session.rust.json";

pub fn session_dir(bot_id: &str) -> PathBuf {
    let safe: String = bot_id
        .trim()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    storage_path(&format!("sessions/{}", safe))
}

pub fn has_session_file(bot_id: &str) -> bool {
    if bot_id.trim().is_empty() {
        return false;
    }
    session_dir(bot_id).join(SESSION_FILE).exists()
}

/* ── JSON session storage ────────────────────────────────────────────────── */

#[derive(Debug)]
pub struct SessionErr(String);

impl std::fmt::Display for SessionErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SessionErr {}

#[derive(Serialize, Deserialize, Default)]
struct DiskSession {
    #[serde(default)]
    home_dc: Option<i32>,
    #[serde(default)]
    dc_options: Vec<DcOption>,
    #[serde(default)]
    peer_infos: Vec<(PeerId, PeerInfo)>,
    #[serde(default)]
    updates_state: UpdatesState,
}

/// The ports MadelineProto tries against one DC, in order.
///
/// [`vendor/danog/madelineproto/src/DataCenter.php`](vendor/danog/madelineproto/src/DataCenter.php:345)
/// (`generateContexts`) walks `array_unique([$port, 443, 80, 88, 5222])` and
/// reconnects with the next one when the previous address does not answer — the
/// "Telegram Desktop server retry" that keeps the PHP engine alive on networks
/// where 443 to one DC is filtered. grammers has no such matrix (its sender pool
/// is hard-wired to `transport::Full`), but it re-reads `Session::dc_option()`
/// on EVERY connect, so rotating the port here is the same retry at the only
/// seam this stack exposes.
pub const DC_PORTS: [u16; 4] = [443, 80, 88, 5222];

/// The addresses `*.web.telegram.org` actually resolve to, measured 2026-11-03:
/// `pluto`+`aurora` → `149.154.175.209`, `venus`+`vesta`+`flower` →
/// `149.154.167.99`, `flora` → `149.154.170.96`.
///
/// They are NOT the MTProto endpoints grammers ships (`149.154.167.51`,
/// `149.154.175.50`, …), and they answer TLS/HTTP (`403`/`302`) on 443 — but
/// Telegram has always served these frontends as MTProto endpoints on the same
/// port too (they are the classic `.99` DC addresses Telegram Desktop still
/// carries), which is why `DataCenter::generateContexts()` treats them as extra
/// addresses for a DC rather than giving up. The five subdomains collapse to
/// three distinct IPs, so there is nothing to gain from keeping the per-DC
/// names: what matters on the wire is the address.
///
/// Tried ONLY after the stored address and every [`DC_PORTS`] entry has been
/// exhausted, never for a DC that already holds an auth key.
pub const DC_FALLBACK_IPV4: [std::net::Ipv4Addr; 3] = [
    std::net::Ipv4Addr::new(149, 154, 167, 99),
    std::net::Ipv4Addr::new(149, 154, 175, 209),
    std::net::Ipv4Addr::new(149, 154, 170, 96),
];

/// The `step`-th address to try for a DC: its own address across every
/// [`DC_PORTS`] port first, then the web-frontend addresses on 443.
fn dc_candidate(ipv4: std::net::Ipv4Addr, step: usize) -> (std::net::Ipv4Addr, u16) {
    if step < DC_PORTS.len() {
        (ipv4, DC_PORTS[step])
    } else {
        (
            DC_FALLBACK_IPV4[(step - DC_PORTS.len()) % DC_FALLBACK_IPV4.len()],
            443,
        )
    }
}

/// A `Session` backed by one JSON file. Every mutation is persisted at once
/// (write to a temp file then rename) so a restart reuses the auth keys.
pub struct JsonSession {
    data: Mutex<SessionData>,
    path: PathBuf,
    /// Per-DC cursor into [`DC_PORTS`]. In memory only: it exists to spread
    /// successive connect attempts over the port list, and a restart starting
    /// again at 443 is exactly what we want.
    dc_cursor: Mutex<HashMap<i32, usize>>,
}

impl JsonSession {
    pub fn load(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let mut data = SessionData::default();
        if let Ok(text) = fs::read_to_string(&path) {
            if let Ok(d) = serde_json::from_str::<DiskSession>(&text) {
                if let Some(h) = d.home_dc {
                    data.home_dc = h;
                }
                for o in d.dc_options {
                    data.dc_options.insert(o.id, o);
                }
                for (k, v) in d.peer_infos {
                    data.peer_infos.insert(k, v);
                }
                data.updates_state = d.updates_state;
            }
        }
        Self {
            data: Mutex::new(data),
            path,
            dc_cursor: Mutex::new(HashMap::new()),
        }
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, SessionData>, SessionErr> {
        self.data
            .lock()
            .map_err(|_| SessionErr("session lock poisoned".to_string()))
    }

    fn persist(&self, data: &SessionData) {
        let disk = DiskSession {
            home_dc: Some(data.home_dc),
            dc_options: data.dc_options.values().cloned().collect(),
            peer_infos: data.peer_infos.iter().map(|(k, v)| (*k, v.clone())).collect(),
            updates_state: data.updates_state.clone(),
        };
        if let Ok(text) = serde_json::to_string(&disk) {
            let tmp = self.path.with_extension("json.tmp");
            if fs::write(&tmp, text).is_ok() {
                let _ = fs::rename(&tmp, &self.path);
            }
        }
    }
}

impl Session for JsonSession {
    type Error = SessionErr;

    fn home_dc_id(&self) -> Result<i32, Self::Error> {
        Ok(self.lock()?.home_dc)
    }

    fn set_home_dc_id(&self, dc_id: i32) -> SessFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            let mut d = self.lock()?;
            d.home_dc = dc_id;
            self.persist(&d);
            Ok(())
        })
    }

    /// The address grammers dials for this DC.
    ///
    /// A DC that already holds an auth key is returned untouched — that key was
    /// negotiated with that address, and re-pointing a live session would break
    /// a working install. Any DC *without* a key is a login in progress, and
    /// that is the case a filtered network breaks: answer with the next port in
    /// [`DC_PORTS`] each time the sender re-connects, so a blocked 443 is
    /// followed by 80, then 88, then 5222 instead of the same dead socket.
    /// `set_dc_option()` persists whichever one finally answered, so the
    /// rotation heals itself.
    fn dc_option(&self, dc_id: i32) -> Result<Option<DcOption>, Self::Error> {
        let d = self.lock()?;
        let Some(opt) = d.dc_options.get(&dc_id).cloned() else {
            return Ok(None);
        };
        if opt.auth_key.is_some() {
            return Ok(Some(opt));
        }
        let (ip, port) = {
            let mut cursors = self
                .dc_cursor
                .lock()
                .map_err(|_| SessionErr("dc cursor lock poisoned".to_string()))?;
            let step = {
                let cursor = cursors.entry(dc_id).or_insert(0);
                let step = *cursor;
                *cursor = cursor.wrapping_add(1);
                step
            };
            // The address rotates too, not just the port: once this DC's own
            // address has been tried on every port, the web-frontend addresses
            // are next (see `DC_FALLBACK_IPV4`).
            let pick = dc_candidate(*opt.ipv4.ip(), step);
            // Which address this connect is about to dial. One line per connect at
            // debug level, because the only way to read a "Telegram unreachable"
            // report later is to know which candidate was in play.
            log(
                "dc candidate",
                &serde_json::json!({
                    "dc": dc_id,
                    "step": step,
                    "addr": format!("{}:{}", pick.0, pick.1),
                }),
            );
            pick
        };
        Ok(Some(DcOption {
            ipv4: std::net::SocketAddrV4::new(ip, port),
            ipv6: std::net::SocketAddrV6::new(*opt.ipv6.ip(), port, opt.ipv6.flowinfo(), opt.ipv6.scope_id()),
            ..opt
        }))
    }

    fn set_dc_option(&self, dc_option: &DcOption) -> SessFuture<'_, Result<(), Self::Error>> {
        let opt = dc_option.clone();
        Box::pin(async move {
            let mut d = self.lock()?;
            d.dc_options.insert(opt.id, opt.clone());
            self.persist(&d);
            Ok(())
        })
    }

    fn peer(&self, peer: PeerId) -> SessFuture<'_, Result<Option<PeerInfo>, Self::Error>> {
        Box::pin(async move { Ok(self.lock()?.peer_infos.get(&peer).cloned()) })
    }

    fn cache_peer(&self, peer: PeerInfo) -> SessFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            let mut d = self.lock()?;
            let id = peer.id();
            match d.peer_infos.get_mut(&id) {
                Some(existing) => {
                    existing.extend_info(&peer);
                }
                None => {
                    d.peer_infos.insert(id, peer.clone());
                }
            }
            self.persist(&d);
            Ok(())
        })
    }

    fn updates_state(&self) -> SessFuture<'_, Result<UpdatesState, Self::Error>> {
        Box::pin(async move { Ok(self.lock()?.updates_state.clone()) })
    }

    fn set_update_state(&self, update: UpdateState) -> SessFuture<'_, Result<(), Self::Error>> {
        Box::pin(async move {
            let mut d = self.lock()?;
            match update {
                UpdateState::All(state) => d.updates_state = state,
                UpdateState::Primary { pts, date, seq } => {
                    d.updates_state.pts = pts;
                    d.updates_state.date = date;
                    d.updates_state.seq = seq;
                }
                UpdateState::Secondary { qts } => d.updates_state.qts = qts,
                UpdateState::Channel { id, pts } => {
                    d.updates_state.channels.retain(|c| c.id != id);
                    d.updates_state.channels.push(ChannelState { id, pts });
                }
            }
            self.persist(&d);
            Ok(())
        })
    }
}

/* ── live clients ────────────────────────────────────────────────────────── */

#[derive(Clone, Default, Debug)]
pub struct BotIdentity {
    pub id: String,
    pub username: String,
    pub first_name: String,
}

pub struct BotEntry {
    pub identity: BotIdentity,
    clients: Vec<Client>,
    /// One gate per `Client`. grammers multiplexes every request for a DC over
    /// that DC's single connection, and the DC resets it once more than a
    /// handful of `upload.getFile` parts are outstanding on it (measured:
    /// 914 x "Connection reset by peer (os error 104)" with 200 concurrent
    /// downloads). Each gate therefore bounds in-flight parts per CONNECTION,
    /// so `--workers 8 --inflight 8` is 64 in flight across 8 connections,
    /// which is the shape Telegram documents.
    gates: Vec<Arc<tokio::sync::Semaphore>>,
    next: AtomicUsize,
}

impl BotEntry {
    fn pick_index(&self) -> usize {
        self.next.fetch_add(1, Ordering::Relaxed) % self.clients.len()
    }
    pub fn pick(&self) -> Client {
        self.clients[self.pick_index()].clone()
    }
    /// The `Client` and the in-flight gate that belongs to it. Both must be
    /// taken in one call so a request cannot end up holding another
    /// connection's permit.
    pub fn pick_slot(&self) -> (Client, Arc<tokio::sync::Semaphore>) {
        let i = self.pick_index();
        (self.clients[i].clone(), self.gates[i].clone())
    }
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }
}

fn live_tasks() -> &'static Mutex<Vec<tokio::task::JoinHandle<()>>> {
    static T: std::sync::OnceLock<Mutex<Vec<tokio::task::JoinHandle<()>>>> = std::sync::OnceLock::new();
    T.get_or_init(|| Mutex::new(Vec::new()))
}

fn live_clients() -> &'static Mutex<Vec<Client>> {
    static C: std::sync::OnceLock<Mutex<Vec<Client>>> = std::sync::OnceLock::new();
    C.get_or_init(|| Mutex::new(Vec::new()))
}

pub fn disconnect_all() {
    if let Ok(mut tasks) = live_tasks().lock() {
        for t in tasks.drain(..) {
            t.abort();
        }
    }
    if let Ok(mut clients) = live_clients().lock() {
        clients.clear();
    }
}

pub struct TelegramPool {
    workers: usize,
    inflight: usize,
    bots: tokio::sync::Mutex<HashMap<String, Arc<BotEntry>>>,
    boot_lock: tokio::sync::Mutex<()>,
}

impl TelegramPool {
    pub fn new(workers: usize, inflight: usize) -> Self {
        Self {
            workers: workers.max(1),
            inflight: inflight.max(1),
            bots: tokio::sync::Mutex::new(HashMap::new()),
            boot_lock: tokio::sync::Mutex::new(()),
        }
    }

    pub fn workers(&self) -> usize {
        self.workers
    }

    pub fn inflight(&self) -> usize {
        self.inflight
    }

    pub fn cached(&self) -> Option<Arc<BotEntry>> {
        None
    }

    pub async fn get(&self, bot_id: &str) -> Option<Arc<BotEntry>> {
        self.bots.lock().await.get(bot_id.trim()).cloned()
    }

    /// Ceiling for any single Telegram round-trip inside [`Self::boot`].
    ///
    /// grammers' runner RETRIES a datacentre connection forever when it cannot
    /// reach Telegram at all (blocked egress, or an IP range Telegram
    /// black-holes), so `is_authorized` / `bot_sign_in` / `invoke` never resolve:
    /// no error, no log line, no timeout. Measured on a Render free instance
    /// (2026-11-03): `/api/provision` stayed open past 100 s with an empty
    /// debug.log, the boot top-up never logged `guest bot ready`, and because
    /// `boot()` holds `boot_lock` every later login queued behind the same dead
    /// await — and the provision lock was never dropped, so the whole install
    /// could not provision at all. A stated failure beats an invisible one.
    const BOOT_RPC_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(20);

    /// The error a bounded round-trip answers with, naming the fault precisely:
    /// the origin answered, the session file is fine, Telegram is unreachable.
    fn telegram_unreachable(what: &str) -> String {
        // ALWAYS-ON (`stderr_log`), not `log()`: this is the one line that explains
        // a wedged install on a fresh container, and it must reach the platform log
        // without anyone enabling debug first. Measured on the Render instance: the
        // whole failure produced one `reclaimed ... lock` line and nothing else,
        // which is why the hang stayed invisible for minutes.
        crate::log::stderr_log(
            "telegram unreachable",
            Some(&serde_json::json!({
                "step": what,
                "timeout_secs": Self::BOOT_RPC_TIMEOUT.as_secs(),
            })),
        );
        format!(
            "Telegram did not answer {} within {}s: this host cannot reach a Telegram \
             datacentre (blocked egress or a blackholed IP range), so no guest bot can be \
             signed in from here.",
            what,
            Self::BOOT_RPC_TIMEOUT.as_secs()
        )
    }

    /// Boot (or reuse) the clients for this bot.
    ///
    /// With a token the call performs its own login
    /// (`auth.ImportBotAuthorization`) and persists the session; without one it
    /// resumes the stored session.
    pub async fn boot(&self, bot_id: &str, token: Option<&str>) -> Result<Arc<BotEntry>, String> {
        let key = bot_id.trim().to_string();
        if key.is_empty() {
            return Err("bot_id is required".to_string());
        }
        if let Some(entry) = self.bots.lock().await.get(&key).cloned() {
            if token.is_none() {
                return Ok(entry);
            }
            let client = entry.pick();
            let authorized = tokio::time::timeout(Self::BOOT_RPC_TIMEOUT, client.is_authorized())
                .await
                .map(|r| r.unwrap_or(false))
                .unwrap_or(false);
            if authorized {
                return Ok(entry);
            }
        }

        // Breadcrumb BEFORE the lock: a boot that blocks on `boot_lock` — another
        // login stuck inside a datacentre connect — then shows up as a second
        // `telegram boot` line for the same bot instead of as silence.
        log(
            "telegram boot",
            &serde_json::json!({
                "bot_id": key,
                "login": token.is_some(),
                "workers": self.workers,
            }),
        );
        let _guard = self.boot_lock.lock().await;
        if let Some(entry) = self.bots.lock().await.get(&key).cloned() {
            return Ok(entry);
        }

        let mut creds = load_cached_api_credentials();
        if creds.is_none() {
            if let Some(t) = token {
                creds = fetch_credentials_from_wordpress(t).await;
            }
        }
        let (api_id, api_hash) = creds.ok_or_else(|| {
            "API credentials are missing: storage/api_credentials.json not found and WordPress returned none."
                .to_string()
        })?;

        let dir = session_dir(&key);
        fs::create_dir_all(&dir)
            .map_err(|e| format!("could not create session dir {}: {}", dir.display(), e))?;
        let base = dir.join(SESSION_FILE);

        let (client, task) = make_client(&base, api_id).await?;
        // Bounded: a fresh `Client` connects lazily on its first request, and the
        // runner retries that connection forever instead of failing.
        let authorized = tokio::time::timeout(Self::BOOT_RPC_TIMEOUT, client.is_authorized())
            .await
            .map_err(|_| Self::telegram_unreachable("the session check"))?
            .map_err(|e| e.to_string())?;
        if !authorized {
            let tok = token.ok_or_else(|| {
                "No valid Telegram session. Provisioning a bot is required.".to_string()
            })?;
            // A login IS `auth.ImportBotAuthorization`, the exact call Telegram
            // answers with FLOOD_WAIT. While a wait is recorded the attempt cannot
            // succeed AND pushes the wait further out (`note_flood` never shortens
            // it), so the retry has to wait for the WAIT to expire — not for the
            // next request. This is the one choke point every login goes through
            // (download candidate loop, BOT_TOKENS provisioning, /api/botlogin), so
            // the guard belongs here rather than in each caller.
            // The wait is PER TOKEN (measured: 4 of 6 bots waiting, 2 signing in
            // fine), so the guard and the record both key on THIS bot. A global
            // flag here refused logins that would have worked.
            let flood = crate::session::flood_remaining_for(&key);
            if flood > 0 {
                return Err(format!(
                    "bot sign-in paused: Telegram rate limit for this bot for another {}s (FLOOD_WAIT).",
                    flood
                ));
            }
            // auth.importBotAuthorization, exactly what grammers sends.
            // Bounded, and a timeout must NOT arm the flood gate: grammers never
            // reached Telegram, so there is no FLOOD_WAIT to honour.
            tokio::time::timeout(Self::BOOT_RPC_TIMEOUT, client.bot_sign_in(tok, &api_hash))
                .await
                .map_err(|_| Self::telegram_unreachable("auth.ImportBotAuthorization"))?
                .map_err(|e| {
                    let msg = format!("bot sign-in failed: {}", e);
                    crate::session::note_flood_for(&key, &msg);
                    msg
                })?;
        }

        let users = tokio::time::timeout(
            Self::BOOT_RPC_TIMEOUT,
            client.invoke(&tl::functions::users::GetUsers {
                id: vec![tl::enums::InputUser::UserSelf],
            }),
        )
        .await
        .map_err(|_| Self::telegram_unreachable("users.GetUsers(self)"))?
        .map_err(|e| format!("getSelf failed: {}", e))?;
        let (id, username, first_name) = match users.first() {
            Some(tl::enums::User::User(u)) => (
                u.id,
                u.username.clone().unwrap_or_default(),
                u.first_name.clone().unwrap_or_default(),
            ),
            _ => (0, String::new(), String::new()),
        };
        let identity = BotIdentity {
            id: if id == 0 { key.clone() } else { id.to_string() },
            username,
            first_name,
        };

        let mut clients = vec![client];
        let mut tasks = vec![task];
        // Extra clients: copies of the SAME session file, one connection set
        // each. Copying (rather than logging in N times) avoids N flood-waits
        // and matches what a multi-connection client does with one auth key.
        for i in 1..self.workers {
            let extra = dir.join(format!("session.rust.w{}.json", i));
            if base.exists() {
                let _ = fs::copy(&base, &extra);
            }
            match make_client(&extra, api_id).await {
                Ok((c, t)) => {
                    clients.push(c);
                    tasks.push(t);
                }
                Err(e) => {
                    log("extra worker client failed", &serde_json::json!({"error": e, "worker": i}));
                }
            }
        }

        {
            let mut live = live_clients().lock().map_err(|_| "lock".to_string())?;
            live.extend(clients.iter().cloned());
        }
        if let Ok(mut lt) = live_tasks().lock() {
            lt.extend(tasks);
        }

        let gates: Vec<Arc<tokio::sync::Semaphore>> = clients
            .iter()
            .map(|_| Arc::new(tokio::sync::Semaphore::new(self.inflight)))
            .collect();
        let entry = Arc::new(BotEntry {
            identity: identity.clone(),
            clients,
            gates,
            next: AtomicUsize::new(0),
        });
        self.bots.lock().await.insert(key.clone(), entry.clone());
        if !identity.id.is_empty() && identity.id != key {
            self.bots.lock().await.insert(identity.id.clone(), entry.clone());
            let real_dir = session_dir(&identity.id);
            if !real_dir.exists() && dir.exists() {
                let _ = fs::create_dir_all(&real_dir);
                if let Ok(entries) = fs::read_dir(&dir) {
                    for e in entries.flatten() {
                        let _ = fs::copy(e.path(), real_dir.join(e.file_name()));
                    }
                }
            }
        }
        log(
            "telegram session resumed",
            &serde_json::json!({
                "bot_id": key,
                "username": identity.username,
                "workers": entry.client_count(),
                "inflight_per_connection": self.inflight,
                "inflight_total": entry.client_count() * self.inflight
            }),
        );
        Ok(entry)
    }

    pub async fn boot_with_token(&self, token: &str, hint: &str) -> Result<Arc<BotEntry>, String> {
        self.boot(hint, Some(token)).await
    }
}

async fn make_client(path: &Path, api_id: i32) -> Result<(Client, tokio::task::JoinHandle<()>), String> {
    let session = Arc::new(JsonSession::load(path));
    let SenderPool { runner, handle, .. } = SenderPool::new(session, api_id);
    let client = Client::new(handle);
    let task = tokio::spawn(runner.run());
    Ok((client, task))
}

/* ── download primitive ──────────────────────────────────────────────────── */

#[derive(Clone)]
pub struct DocTarget {
    pub dc_id: i32,
    pub size: u64,
    pub location: tl::enums::InputFileLocation,
}

impl DocTarget {
    /// The `(id, access_hash, file_reference)` triple this target addresses, for
    /// persisting a file WE sent: the ledger keeps it so a later request can be
    /// answered without resolving anything through the origin.
    fn doc_location(&self) -> Option<&tl::types::InputDocumentFileLocation> {
        match &self.location {
            tl::enums::InputFileLocation::InputDocumentFileLocation(l) => Some(l),
            _ => None,
        }
    }

    pub fn doc_id(&self) -> i64 {
        self.doc_location().map(|l| l.id).unwrap_or(0)
    }

    pub fn access_hash(&self) -> i64 {
        self.doc_location().map(|l| l.access_hash).unwrap_or(0)
    }

    pub fn file_ref(&self) -> Vec<u8> {
        self.doc_location()
            .map(|l| l.file_reference.clone())
            .unwrap_or_default()
    }
}

impl Downloadable for DocTarget {
    fn to_raw_input_location(&self) -> Option<tl::enums::InputFileLocation> {
        Some(self.location.clone())
    }
    fn size(&self) -> Option<usize> {
        if self.size == 0 {
            None
        } else {
            Some(self.size as usize)
        }
    }
    fn dc_id(&self) -> Option<i32> {
        Some(self.dc_id)
    }
}

/* ── upload + send (the backfill primitive) ──────────────────────────────── */

/// One file we sent ourselves, described the way the byte path already wants it.
///
/// `DocTarget` is the ONLY thing `fetch_range` accepts, and a send result carries
/// exactly what it needs: `document.dc_id`, `document.size`, and the
/// `(id, access_hash, file_reference)` triple that becomes an
/// `InputDocumentFileLocation`. That is why a backfilled file is playable with no
/// resolve, no origin round-trip and no second streaming path — the handle from
/// the send IS the handle for the download.
pub struct SentDoc {
    pub target: DocTarget,
    pub message_id: i32,
    /// `id_access_dc` — the identity of THIS upload, for a persisted record.
    pub key: String,
}

/// Resolve `@username` (or a bare name) to the peer to send to.
///
/// The peer is a relay ACCOUNT, not a bot: the origin relays through one of these
/// because Telegram does not deliver bot-to-bot messages, and that account has an
/// open chat with every bot in the pool. `tgfile_userbot_accounts()` on the origin
/// is the same list, exposed by `GET /userbots`.
pub async fn resolve_peer(client: &Client, username: &str) -> Result<PeerRef, GramErr> {
    let name = username.trim().trim_start_matches('@');
    if name.is_empty() {
        return Err(GramErr {
            message: "resolve_peer: empty address".to_string(),
            ..Default::default()
        });
    }
    // A bare numeric id is a USER with no username — which is what the relay pool
    // actually publishes (measured: four accounts, `"username": ""` on all of
    // them). There is no `contacts.resolveUsername` for that, so the dialog list
    // is the only source of an `access_hash`.
    if let Ok(uid) = name.parse::<i64>() {
        return resolve_peer_by_id(client, uid).await;
    }
    let peer = match client.resolve_username(name).await {
        Ok(Some(p)) => p,
        Ok(None) => {
            return Err(GramErr {
                message: format!("resolve_peer: @{name} not found"),
                ..Default::default()
            })
        }
        Err(e) => {
            return Err(GramErr {
                message: format!("resolve_peer @{name}: {e}"),
                ..Default::default()
            })
        }
    };
    // `PeerRef` is what every send takes; the cache also makes the second call
    // free, which matters because a backfill resolves the same relay account
    // once per upload.
    match peer.to_ref().await {
        Ok(Some(r)) => Ok(r),
        Ok(None) => Err(GramErr {
            message: format!("resolve_peer @{name}: no reference (not in session cache)"),
            ..Default::default()
        }),
        Err(e) => Err(GramErr {
            message: format!("resolve_peer @{name}: {e}"),
            ..Default::default()
        }),
    }
}

/// How many dialogs to walk before giving up on finding a relay account. The
/// pool accounts are contacted constantly, so they are always recent; the cap
/// only bounds a pathological bot account.
const DIALOG_SCAN_LIMIT: usize = 400;

/// Resolve a numeric user id to a sendable peer via the bot's OWN dialog list.
///
/// The relay accounts carry no `@username` (the pool manifest publishes ids
/// only), and Telegram needs an `access_hash` to address a user. The dialog list
/// holds one for every chat the bot already has, which is the same precondition
/// the PHP relay depends on: the account has an open chat with the bot. So a
/// miss here means exactly that — no chat — and it says so instead of sending
/// into the void.
pub async fn resolve_peer_by_id(client: &Client, user_id: i64) -> Result<PeerRef, GramErr> {
    let mut dialogs = client.iter_dialogs();
    let mut seen = 0usize;
    while seen < DIALOG_SCAN_LIMIT {
        let next = dialogs.next().await.map_err(|e| GramErr {
            message: format!("iter_dialogs: {e}"),
            ..Default::default()
        })?;
        let Some(dialog) = next else { break };
        seen += 1;
        let peer = dialog.peer();
        if peer.id().bare_id() != Some(user_id) {
            continue;
        }
        return match peer.to_ref().await {
            Ok(Some(r)) => Ok(r),
            Ok(None) => Err(GramErr {
                message: format!("relay {user_id}: dialog present but no reference"),
                ..Default::default()
            }),
            Err(e) => Err(GramErr {
                message: format!("relay {user_id}: {e}"),
                ..Default::default()
            }),
        };
    }
    Err(GramErr {
        message: format!(
            "relay {user_id} is not in this bot's dialogs ({seen} scanned) — the account must have an open chat with the bot"
        ),
        ..Default::default()
    })
}

/// Stream `source` into Telegram and send it as an audio document, returning the
/// [`DocTarget`] that makes it immediately playable.
///
/// `upload_stream`, not `upload_file`: the source is a remote HTTP body (the
/// Qobuz CDN response), so there is no file on disk to read and no temp file to
/// clean up. Above `BIG_FILE_SIZE` (10 MB) grammers switches to
/// `upload::SaveBigFilePart` over four workers, which a 24.8 MB FLAC hits.
///
/// `size` must be the exact byte count of what the reader will yield: it is sent
/// to Telegram as the document size and used to plan every `upload.getFile`
/// part on playback.
/// How long a master upload may stay silent before it is declared hung.
///
/// MEASURED 2026-11-10 on the deployed fleet: an upload inside `upload_stream`
/// never returned. There is no timeout on that future, so the task lived forever
/// — and with it the process-wide backfill gate (`backfill::FILL_OWNER_SINCE`),
/// which is why every later query on that dyno answered `a fill is already
/// running in this process - deferred` for over an hour while nothing streamed.
///
/// A floor is required because the first byte of a 50 MB FLAC can take minutes on
/// a phone link, and a ceiling because a wedged connection must not outlive the
/// turn. The allowance is deliberately generous (256 KB/s, floor of ten minutes):
/// this is a liveness backstop against a hang, NOT a bandwidth policy — killing a
/// slow but working upload would trade a stuck fill for a failed one.
const UPLOAD_FLOOR_SECS: u64 = 600;
const UPLOAD_CEILING_SECS: u64 = 1500;
const UPLOAD_ASSUMED_BYTES_PER_SEC: u64 = 256 * 1024;

/// The wall-clock budget for uploading `size` bytes. Pure, so the shape of the
/// rule is testable without a Telegram connection.
pub(crate) fn upload_deadline_secs(size: u64) -> u64 {
    let allowance = size / UPLOAD_ASSUMED_BYTES_PER_SEC;
    (UPLOAD_FLOOR_SECS + allowance).min(UPLOAD_CEILING_SECS)
}

pub async fn upload_and_send_audio<S>(
    client: &Client,
    peer: PeerRef,
    source: &mut S,
    size: u64,
    file_name: &str,
    mime: &str,
    title: &str,
    performer: &str,
    duration_secs: i64,
    // A cover image to attach as the document's thumbnail, already fetched and
    // already validated as a small JPEG (see `backfill::fetch_thumb`). Without
    // one the file lands with NO thumb, which is why a filled track showed a
    // blank square in the player and in `/thumb/{code}`: that route reads the
    // document's own thumbs from Telegram, so if the send carried none there is
    // nothing to read.
    thumb: Option<Vec<u8>>,
    caption: &str,
) -> Result<SentDoc, GramErr>
where
    S: tokio::io::AsyncRead + Unpin,
{
    if size == 0 {
        return Err(GramErr {
            message: "upload_and_send_audio: zero size".to_string(),
            ..Default::default()
        });
    }

    // BOUNDED, and the bound is the point: `upload_stream` has no timeout of its
    // own, so a send that stops making progress parks this task — and the caller's
    // fill gate — for the life of the process. On the deadline the future is
    // dropped and the caller gets an error, which releases its lease and lets the
    // NEXT query try again.
    let deadline = upload_deadline_secs(size);
    let uploaded = match tokio::time::timeout(
        std::time::Duration::from_secs(deadline),
        client.upload_stream(source, size as usize, file_name.to_string()),
    )
    .await
    {
        Ok(result) => result.map_err(|e| GramErr {
            message: format!("upload_stream {file_name}: {e}"),
            transport: true,
            ..Default::default()
        })?,
        Err(_) => {
            log(
                "backfill upload deadline exceeded",
                &serde_json::json!({"file_name": file_name, "bytes": size, "secs": deadline}),
            );
            return Err(GramErr {
                message: format!(
                    "upload_stream {file_name}: no completion in {deadline}s — upload abandoned as hung"
                ),
                transport: true,
                ..Default::default()
            });
        }
    };

    let mut media = InputAudio::new(uploaded).mime_type(mime.to_string());
    // Upload the cover as a document thumb. Failure is not fatal: a track with no
    // artwork still plays, and losing the whole fill over a 404 on a JPEG would be
    // the wrong trade.
    if let Some(bytes) = thumb {
        if !bytes.is_empty() {
            let mut cursor = &bytes[..];
            // A thumbnail is a few hundred KB; 60 s is already absurdly generous
            // and a hang here must cost the cover, not the whole fill.
            let thumb_upload = tokio::time::timeout(
                std::time::Duration::from_secs(60),
                client.upload_stream(&mut cursor, bytes.len(), "thumb.jpg".to_string()),
            )
            .await;
            match thumb_upload {
                Ok(Ok(up)) => media = media.thumb(Some(up)),
                Ok(Err(e)) => log(
                    "backfill thumb upload failed",
                    &serde_json::json!({"error": e.to_string(), "bytes": bytes.len()}),
                ),
                Err(_) => log(
                    "backfill thumb upload failed",
                    &serde_json::json!({"error": "no completion in 60s", "bytes": bytes.len()}),
                ),
            }
        }
    }
    if duration_secs > 0 {
        media = media.duration(std::time::Duration::from_secs(duration_secs as u64));
    }
    if !title.is_empty() {
        media = media.title(Some(title.to_string()));
    }
    if !performer.is_empty() {
        media = media.performer(Some(performer.to_string()));
    }

    // A caption with media is how the origin's userbot pool is asked to relay a
    // file to a bot — see `backfill::REQUEST_CAPTION`. Empty means "no caption",
    // which is the previous behaviour exactly.
    let mut outgoing = InputMessage::new().media(media);
    if !caption.is_empty() {
        outgoing = outgoing.text(caption.to_string());
    }

    // The send itself is bounded too: a media message is a single round trip once
    // the file is uploaded, so two minutes is a hang, not a slow network.
    let msg = match tokio::time::timeout(
        std::time::Duration::from_secs(120),
        client.send_message(peer, outgoing),
    )
    .await
    {
        Ok(result) => result.map_err(|e| GramErr {
            message: format!("send_audio {file_name}: {e}"),
            ..Default::default()
        })?,
        Err(_) => {
            return Err(GramErr {
                message: format!("send_audio {file_name}: no completion in 120s — send abandoned as hung"),
                ..Default::default()
            })
        }
    };
    let message_id = msg.id();

    // The document triple the byte path needs. Taken from the SEND RESULT, never
    // guessed and never re-resolved: `file_reference` is only valid for the bot
    // that received it, so this is also the only bot that can play it back.
    let doc = match msg.media() {
        Some(Media::Document(m)) => match m.raw.document {
            Some(tl::enums::Document::Document(d)) => d,
            _ => {
                return Err(GramErr {
                    message: format!("send_audio {file_name}: document empty"),
                    ..Default::default()
                })
            }
        },
        _ => {
            return Err(GramErr {
                message: format!("send_audio {file_name}: sent media is not a document"),
                ..Default::default()
            })
        }
    };

    let key = format!("{}_{}_{}", doc.id, doc.access_hash, doc.dc_id);
    log(
        "backfill sent",
        &serde_json::json!({
            "file_name": file_name,
            "bytes": doc.size,
            "message_id": message_id,
            "dc_id": doc.dc_id,
        }),
    );

    Ok(SentDoc {
        target: DocTarget {
            dc_id: doc.dc_id,
            size: doc.size.max(0) as u64,
            location: tl::enums::InputFileLocation::InputDocumentFileLocation(
                tl::types::InputDocumentFileLocation {
                    id: doc.id,
                    access_hash: doc.access_hash,
                    file_reference: doc.file_reference,
                    thumb_size: String::new(),
                },
            ),
        },
        message_id,
        key,
    })
}

#[derive(Debug, Clone)]
pub struct GramErr {
    pub message: String,
    /// `FILE_REFERENCE_EXPIRED` / `FILE_MIGRATE_*`: the caller re-resolves once.
    pub file_reference: bool,
    /// `FLOOD_WAIT_<n>` seconds. Surfaced as HTTP 429, never as a 5xx: a client
    /// reads 5xx/404 as fatal and never retries (root AGENTS.md: "A rate limit
    /// must answer 429, not 404").
    pub flood_wait: Option<u64>,
    /// A connection-level failure (reset / 0 bytes / broken pipe) that is worth
    /// retrying on the same part.
    pub transport: bool,
}

impl Default for GramErr {
    fn default() -> Self {
        Self {
            message: String::new(),
            file_reference: false,
            flood_wait: None,
            transport: false,
        }
    }
}

impl std::fmt::Display for GramErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Does this message look like the DC dropping the connection rather than a
/// logical failure? Matched on the text because grammers reports the same
/// condition as `Io`, `Transport` or `Deserialize` depending on where it broke.
pub fn is_transport_reset(message: &str) -> bool {
    let m = message.to_ascii_lowercase();
    const NEEDLES: [&str; 10] = [
        "connection reset",
        "reset by peer",
        "read 0 bytes",
        "broken pipe",
        "os error 104",
        "os error 32",
        "unexpected eof",
        "connection closed",
        "connection aborted",
        "early eof",
    ];
    NEEDLES.iter().any(|n| m.contains(n))
}

pub fn map_invocation_error(e: InvocationError) -> GramErr {
    match &e {
        InvocationError::Rpc(rpc) => {
            // grammers strips the digits out of the name into `value`, so
            // `FLOOD_WAIT_420` arrives as name "FLOOD_WAIT" + value Some(420).
            let flood_wait = if rpc.is("FLOOD_WAIT")
                || rpc.is("FLOOD_WAIT_*")
                || rpc.is("FLOOD_PREMIUM_WAIT_*")
                || rpc.code == 420
            {
                Some(rpc.value.map(u64::from).unwrap_or(0))
            } else {
                None
            };
            GramErr {
                message: format!("{} ({})", rpc.name, rpc.code),
                // `AUTH_BYTES_INVALID` belongs here too. Telegram documents it as
                // "The provided authorization is invalid" — the file_id carries
                // auth bytes bound to the session/bot that MINTED it, so an id
                // resolved for bot A and fetched by bot B always fails this way,
                // and so does a raw file_id pasted from another client. It is the
                // same recoverable condition as a dead reference: re-resolve once
                // against the bot that will actually download. Without it the
                // error fell through to a fatal `500 DOWNLOAD_FAILED`, which is
                // the top playback incident in the origin table (137 of the newest
                // 500 rows, e.g. `short_code=SmEdhz`, bot 8938418955 — while the
                // same short code streamed fine locally through bot 7791686977).
                file_reference: rpc.name.contains("FILE_REFERENCE")
                    || rpc.name.contains("FILE_MIGRATE")
                    || rpc.name.contains("AUTH_BYTES"),
                flood_wait,
                // `-503 Timeout` is Telegram answering "ask again", not a dead
                // file — and it must be retried by the LIVE BODY, which only
                // retries what is marked transient.
                //
                // MEASURED 2026-11-09, BitChord on `R1kYa6` (TUIDE · SUN KISS):
                //
                //   download served by bot            {"range":"bytes=1048576-"}
                //   download stream aborted mid-body  {"bytes_sent":3145728,
                //                                      "error":"Timeout (-503)"}
                //
                // — and NO `download stream retrying a window` line in between,
                // because `is_transport_reset()` looks for TCP phrasings
                // (`reset by peer`, `broken pipe`) and this message is a plain
                // RPC name. So the body died at 3 MB and the player restarted the
                // whole track, which is what the user saw as it looping. It is a
                // transient answer on `upload.getFile` and is classified as one
                // here so every existing retry path (body window, pre-flight
                // rotation) handles it.
                transport: rpc.code == -503,
            }
        }
        other => {
            let message = other.to_string();
            GramErr {
                transport: is_transport_reset(&message),
                message,
                file_reference: false,
                flood_wait: None,
            }
        }
    }
}

/// Reject the shapes a media stream cannot use (`prepareFileTarget`).
pub fn prepare_file_target(file_id_mt: &str, size: u64) -> Result<DocTarget, String> {
    let mut decoded: DecodedFileId = crate::fileid::decode_file_id(file_id_mt)?;
    decoded.size = size;
    if !is_document_file_id(&decoded) {
        return Err(format!(
            "file_id type \"{}\" is not a downloadable document",
            decoded.type_name
        ));
    }
    if location_kind(&decoded) != "document" {
        return Err("unsupported file location".to_string());
    }
    Ok(DocTarget {
        dc_id: decoded.dc_id,
        size,
        location: tl::enums::InputFileLocation::InputDocumentFileLocation(
            tl::types::InputDocumentFileLocation {
                id: decoded.id,
                access_hash: decoded.access_hash,
                file_reference: decoded.file_reference.unwrap_or_default(),
                thumb_size: String::new(),
            },
        ),
    })
}

/// One of the document's THUMBNAILS, addressed exactly like the media itself.
///
/// A thumbnail is not a separate file on Telegram: it is the same
/// `id`/`access_hash`/`file_reference`/`dc_id` with a non-empty `thumb_size`
/// (`s`/`m`/`x`/`y`/`w`, largest last) in `inputDocumentFileLocation`. So a
/// `file_id_mt` the engine can already stream carries its own posters — no
/// resolve, no second id, and nothing the WordPress thumbnail endpoint used to
/// mint has to exist.
///
/// `size` is deliberately `0`: the thumb's length is unknown until Telegram
/// answers, and `Downloadable::size() -> None` is what lets `iter_download`
/// stream to the end of the object instead of planning a fixed-length read.
pub fn prepare_thumb_target(file_id_mt: &str, thumb_size: &str) -> Result<DocTarget, String> {
    let decoded: DecodedFileId = crate::fileid::decode_file_id(file_id_mt)?;
    // A stored `thumb_file_id` decodes as type 0 (`thumbnail`), NOT as one of the
    // media document types — and Telegram still serves it from
    // `inputDocumentFileLocation` + `thumb_size` (measured on a thumb id minted by
    // the downloading bot: `s` answered an 18628-byte JPEG; every
    // `inputPhotoFileLocation` size answered FILE_REFERENCE_EXPIRED, so the photo
    // location is NOT the right one for these ids). Media streaming stays strict:
    // `prepare_file_target()` is what enforces document-only for real playback.
    if !is_document_file_id(&decoded) && decoded.file_type != 0 {
        return Err(format!(
            "file_id type \"{}\" carries no document thumbs",
            decoded.type_name
        ));
    }
    Ok(DocTarget {
        dc_id: decoded.dc_id,
        size: 0,
        location: tl::enums::InputFileLocation::InputDocumentFileLocation(
            tl::types::InputDocumentFileLocation {
                id: decoded.id,
                access_hash: decoded.access_hash,
                file_reference: decoded.file_reference.unwrap_or_default(),
                thumb_size: thumb_size.to_string(),
            },
        ),
    })
}

pub const MIN_CHUNK: u64 = 4096;
pub const MAX_CHUNK: u64 = 512 * 1024;
pub const MB: u64 = 1024 * 1024;

/// Largest `4096 * 2^k` (k = 0..7) that is <= `want`, <= 512 KB, divides
/// `limit_pos` and does not cross the next 1 MB boundary. Telegram's
/// alignment rules: offset/limit divisible by 4096, a part must not cross a
/// 1 MB boundary, `1048576 % limit == 0`, `limit <= 1 MB`.
pub fn plan_limit(pos: u64, want: u64) -> u64 {
    let boundary = MB - (pos % MB);
    let cap = want.min(boundary).min(MAX_CHUNK);
    let mut best = 0u64;
    let mut v = MIN_CHUNK;
    while v <= cap {
        if pos % v == 0 {
            best = v;
        }
        v *= 2;
    }
    if best == 0 {
        MIN_CHUNK
    } else {
        best
    }
}

/// Attempts for one part: the first try plus two retries.
const PART_MAX_ATTEMPTS: usize = 3;
/// 100 / 300 / 700 ms between transport retries.
const PART_BACKOFF_MS: [u64; 3] = [100, 300, 700];
/// A `FLOOD_WAIT` at or below this is slept and retried once (inside the same
/// permit); above it the caller answers 429 so the client retries on its own
/// schedule instead of holding a connection for minutes.
pub const FLOOD_WAIT_RETRY_MAX_SECS: u64 = 30;

/// Fetch `want` bytes starting at an already 4096-aligned `pos`, ungated.
pub async fn fetch_range(client: &Client, target: &DocTarget, pos: u64, want: u64) -> Result<Vec<u8>, GramErr> {
    fetch_range_core(client, None, target, pos, want).await
}

/// Same, but at most `gate` parts run at once on this `Client`'s connection.
/// Requests QUEUE for a permit — they never fail because the gate is full.
pub async fn fetch_range_gated(
    client: &Client,
    gate: &tokio::sync::Semaphore,
    target: &DocTarget,
    pos: u64,
    want: u64,
) -> Result<Vec<u8>, GramErr> {
    fetch_range_core(client, Some(gate), target, pos, want).await
}

async fn fetch_range_core(
    client: &Client,
    gate: Option<&tokio::sync::Semaphore>,
    target: &DocTarget,
    pos: u64,
    want: u64,
) -> Result<Vec<u8>, GramErr> {
    if want == 0 {
        return Ok(Vec::new());
    }
    let limit = plan_limit(pos, want);
    let skip = pos / limit;
    if skip > i32::MAX as u64 {
        return Err(GramErr {
            message: format!("offset {} beyond supported chunk index", pos),
            ..Default::default()
        });
    }

    // The permit bounds how many parts THIS CONNECTION has outstanding. It is
    // held across every retry below, so a retrying part cannot multiply the
    // connection's load.
    let _permit = match gate {
        Some(g) => Some(g.acquire().await.map_err(|_| GramErr {
            message: "download gate closed".to_string(),
            ..Default::default()
        })?),
        None => None,
    };

    let mut transport_attempts = 0usize;
    let mut flood_retried = false;
    loop {
        match fetch_range_once(client, target, limit, skip, want).await {
            Ok(v) => return Ok(v),
            Err(e) => {
                if let Some(secs) = e.flood_wait {
                    if !flood_retried && secs <= FLOOD_WAIT_RETRY_MAX_SECS {
                        flood_retried = true;
                        log(
                            "upload.getFile rate limited, sleeping then retrying part",
                            &serde_json::json!({"flood_wait_seconds": secs, "pos": pos}),
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(secs + 1)).await;
                        continue;
                    }
                    return Err(e);
                }
                // FILE_REFERENCE_EXPIRED keeps its single re-resolve path in the
                // caller — it is never a transport problem.
                if e.file_reference || !e.transport {
                    return Err(e);
                }
                transport_attempts += 1;
                if transport_attempts >= PART_MAX_ATTEMPTS {
                    return Err(e);
                }
                let delay = PART_BACKOFF_MS[(transport_attempts - 1).min(PART_BACKOFF_MS.len() - 1)];
                log(
                    "upload.getFile transport reset, retrying part",
                    &serde_json::json!({
                        "attempt": transport_attempts,
                        "backoff_ms": delay,
                        "error": e.message,
                        "pos": pos,
                    }),
                );
                tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            }
        }
    }
}

async fn fetch_range_once(
    client: &Client,
    target: &DocTarget,
    limit: u64,
    skip: u64,
    want: u64,
) -> Result<Vec<u8>, GramErr> {
    let mut iter = client
        .iter_download(target)
        .chunk_size(limit as i32)
        .skip_chunks(skip as i32);
    let mut out: Vec<u8> = Vec::with_capacity(want as usize);
    while (out.len() as u64) < want {
        match iter.next().await {
            Ok(Some(chunk)) => {
                if chunk.is_empty() {
                    break;
                }
                out.extend_from_slice(&chunk);
            }
            Ok(None) => break,
            Err(e) => return Err(map_invocation_error(e)),
        }
    }
    out.truncate(want as usize);
    Ok(out)
}

/// A single aligned chunk, for the concurrency benchmark (one range request per
/// task, no streaming).
pub async fn fetch_one_chunk(client: &Client, target: &DocTarget, pos: u64, want: u64) -> Result<usize, GramErr> {
    let data = fetch_range(client, target, pos, want).await?;
    Ok(data.len())
}

#[cfg(test)]
impl BotEntry {
    pub fn new_test_entry(bot_id: &str, workers: usize, inflight: usize) -> Self {
        let dummy_path = std::env::temp_dir().join(format!("pm_test_sess_{}.json", bot_id));
        let session = Arc::new(JsonSession::load(&dummy_path));
        let mut clients = Vec::new();
        for _ in 0..workers {
            let SenderPool { handle, .. } = SenderPool::new(session.clone(), 12345);
            clients.push(Client::new(handle));
        }
        let gates = (0..workers)
            .map(|_| Arc::new(tokio::sync::Semaphore::new(inflight)))
            .collect();
        Self {
            identity: BotIdentity {
                id: bot_id.to_string(),
                username: format!("{}_bot", bot_id),
                first_name: format!("Bot {}", bot_id),
            },
            clients,
            gates,
            next: AtomicUsize::new(0),
        }
    }

    pub fn available_permits_per_worker(&self) -> Vec<usize> {
        self.gates.iter().map(|g| g.available_permits()).collect()
    }

    pub fn total_available_permits(&self) -> usize {
        self.gates.iter().map(|g| g.available_permits()).sum()
    }
}

#[cfg(test)]
impl TelegramPool {
    pub async fn insert_test_entry(&self, bot_id: &str, entry: Arc<BotEntry>) {
        self.bots.lock().await.insert(bot_id.trim().to_string(), entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The MadelineProto retry, at the seam grammers actually re-reads.
    ///
    /// A DC with no auth key is a login in progress — the state a filtered
    /// network breaks — so consecutive connects must land on 443, then 80, 88,
    /// 5222 and wrap, instead of dialling the same dead socket forever.
    #[test]
    fn a_fresh_dc_rotates_ports_like_madelineproto() {
        let dir = std::env::temp_dir().join(format!("pm-dc-rotate-{}", std::process::id()));
        let _ = fs::create_dir_all(&dir);
        let file = dir.join("session.rust.json");
        let _ = fs::remove_file(&file);
        // No file on disk: grammers' default session, which is 5 DCs with no keys.
        let session = JsonSession::load(&file);
        let ports: Vec<u16> = (0..5)
            .map(|_| session.dc_option(2).unwrap().expect("DC 2 is known").ipv4.port())
            .collect();
        assert_eq!(ports, DC_PORTS.iter().copied().chain([DC_PORTS[0]]).collect::<Vec<u16>>());
        let _ = fs::remove_file(&file);
    }

    /// After the DC's own address has been tried on every port, the addresses
    /// `*.web.telegram.org` resolve to are dialled on 443 — the address axis of
    /// the retry, not just the port one.
    #[test]
    fn the_web_frontend_addresses_are_tried_after_every_port() {
        let own = std::net::Ipv4Addr::new(149, 154, 167, 51);
        for (step, port) in DC_PORTS.iter().copied().enumerate() {
            assert_eq!(dc_candidate(own, step), (own, port));
        }
        let base = DC_PORTS.len();
        for (n, expected) in DC_FALLBACK_IPV4.iter().copied().enumerate() {
            assert_eq!(dc_candidate(own, base + n), (expected, 443));
        }
        // …and it wraps rather than stopping, so a long-lived process keeps
        // retrying the whole set.
        assert_eq!(
            dc_candidate(own, base + DC_FALLBACK_IPV4.len()),
            (DC_FALLBACK_IPV4[0], 443)
        );
        // A signed-in DC is never rotated: that is the caller's guard, pinned here
        // so nobody "simplifies" it back into the candidate scan.
        assert!(!DC_FALLBACK_IPV4.contains(&own));
    }

    /// The upload deadline is a LIVENESS backstop, not a bandwidth policy: a
    /// small file gets the floor (a phone link is slow to start), a large one
    /// gets a larger allowance, and nothing can exceed the ceiling — a wedged
    /// connection must not outlive the process like the 2026-11-10 one did.
    #[test]
    fn an_upload_deadline_has_a_floor_a_slope_and_a_ceiling() {
        assert_eq!(upload_deadline_secs(0), UPLOAD_FLOOR_SECS);
        assert_eq!(upload_deadline_secs(24_805_746), UPLOAD_FLOOR_SECS + 94);
        assert_eq!(upload_deadline_secs(53_975_130), UPLOAD_FLOOR_SECS + 205);
        // Bigger never means shorter.
        let mut last = 0;
        for mb in [0u64, 1, 10, 25, 50, 100, 400, 4096] {
            let d = upload_deadline_secs(mb * 1024 * 1024);
            assert!(d >= last, "{mb} MB got {d}s after {last}s");
            assert!(d <= UPLOAD_CEILING_SECS, "{mb} MB exceeded the ceiling");
            last = d;
        }
        assert_eq!(upload_deadline_secs(4096 * 1024 * 1024), UPLOAD_CEILING_SECS);
    }

    #[test]
    fn workers_and_inflight_allocated_independently_per_bot() {
        let workers = 4;
        let inflight = 8;
        let pool = TelegramPool::new(workers, inflight);
        assert_eq!(pool.workers(), 4);
        assert_eq!(pool.inflight(), 8);

        let bot_a = Arc::new(BotEntry::new_test_entry("bot_111", workers, inflight));
        let bot_b = Arc::new(BotEntry::new_test_entry("bot_222", workers, inflight));

        assert_eq!(bot_a.client_count(), 4);
        assert_eq!(bot_b.client_count(), 4);

        // Each bot has 4 worker connections, each with 8 inflight permits (total 32 each).
        assert_eq!(bot_a.available_permits_per_worker(), vec![8, 8, 8, 8]);
        assert_eq!(bot_b.available_permits_per_worker(), vec![8, 8, 8, 8]);
        assert_eq!(bot_a.total_available_permits(), 32);
        assert_eq!(bot_b.total_available_permits(), 32);
    }

    #[test]
    fn pick_slot_cycles_workers_independently() {
        let workers = 3;
        let inflight = 5;
        let bot_a = BotEntry::new_test_entry("bot_a", workers, inflight);
        let bot_b = BotEntry::new_test_entry("bot_b", workers, inflight);

        // Advance Bot A two times (slots 0, 1)
        let (_, gate_a0) = bot_a.pick_slot();
        let (_, gate_a1) = bot_a.pick_slot();
        assert_eq!(Arc::strong_count(&gate_a0), 2); // 1 in bot_a.gates + 1 returned
        assert_eq!(Arc::strong_count(&gate_a1), 2);

        // Bot B starts at slot 0 regardless of Bot A's actions
        assert_eq!(bot_b.pick_index(), 0);
        assert_eq!(bot_b.pick_index(), 1);

        // Bot A's next slot is 2, then wraps to 0
        assert_eq!(bot_a.pick_index(), 2);
        assert_eq!(bot_a.pick_index(), 0);
    }

    #[tokio::test]
    async fn bot_a_saturation_does_not_affect_bot_b_inflight() {
        let workers = 4;
        let inflight = 8;
        let pool = TelegramPool::new(workers, inflight);

        let bot_a = Arc::new(BotEntry::new_test_entry("bot_a", workers, inflight));
        let bot_b = Arc::new(BotEntry::new_test_entry("bot_b", workers, inflight));

        pool.insert_test_entry("bot_a", bot_a.clone()).await;
        pool.insert_test_entry("bot_b", bot_b.clone()).await;

        // Saturate Bot A completely by acquiring all 32 permits (4 workers * 8 inflight)
        let mut held_permits_a = Vec::new();
        for _ in 0..(workers * inflight) {
            let (_, gate) = bot_a.pick_slot();
            let permit = gate.try_acquire_owned().expect("Bot A permit should be available");
            held_permits_a.push(permit);
        }

        // Verify Bot A is 100% saturated
        assert_eq!(bot_a.total_available_permits(), 0);
        assert_eq!(bot_a.available_permits_per_worker(), vec![0, 0, 0, 0]);

        // Attempting to acquire another permit on Bot A must fail immediately
        let (_, gate_a) = bot_a.pick_slot();
        assert!(gate_a.try_acquire().is_err());

        // Bot B must be completely unaffected: full 32 permits (8 per worker) remain available!
        assert_eq!(bot_b.total_available_permits(), 32);
        assert_eq!(bot_b.available_permits_per_worker(), vec![8, 8, 8, 8]);

        // Bot B can acquire all its permits concurrently
        let mut held_permits_b = Vec::new();
        for _ in 0..(workers * inflight) {
            let (_, gate) = bot_b.pick_slot();
            let permit = gate.try_acquire_owned().expect("Bot B permit must be independent and available");
            held_permits_b.push(permit);
        }
        assert_eq!(bot_b.total_available_permits(), 0);

        // Release Bot A's permits
        drop(held_permits_a);
        assert_eq!(bot_a.total_available_permits(), 32);

        // Bot B remains saturated until its own permits are dropped
        assert_eq!(bot_b.total_available_permits(), 0);
        drop(held_permits_b);
        assert_eq!(bot_b.total_available_permits(), 32);
    }

    #[tokio::test]
    async fn concurrent_tasks_across_bots_run_independently() {
        let workers = 4;
        let inflight = 8;
        let bot_a = Arc::new(BotEntry::new_test_entry("bot_1", workers, inflight));
        let bot_b = Arc::new(BotEntry::new_test_entry("bot_2", workers, inflight));

        let barrier = Arc::new(tokio::sync::Barrier::new(65)); // 32 from bot_a + 32 from bot_b + main coordinator

        let mut handles = Vec::new();

        // Spawn 32 concurrent workers on Bot A
        for _ in 0..32 {
            let bot = bot_a.clone();
            let b = barrier.clone();
            handles.push(tokio::spawn(async move {
                let (_, gate) = bot.pick_slot();
                let _permit = gate.acquire().await.unwrap();
                b.wait().await;
            }));
        }

        // Spawn 32 concurrent workers on Bot B
        for _ in 0..32 {
            let bot = bot_b.clone();
            let b = barrier.clone();
            handles.push(tokio::spawn(async move {
                let (_, gate) = bot.pick_slot();
                let _permit = gate.acquire().await.unwrap();
                b.wait().await;
            }));
        }

        // The barrier will only be satisfied if ALL 64 tasks across BOTH bots
        // acquired their permits concurrently! If inflight was shared across bots,
        // it would cap at 32 and deadlock here.
        tokio::time::timeout(std::time::Duration::from_secs(3), barrier.wait())
            .await
            .expect("All 64 tasks across both bots must acquire permits concurrently without deadlock");

        for h in handles {
            h.await.unwrap();
        }

        assert_eq!(bot_a.total_available_permits(), 32);
        assert_eq!(bot_b.total_available_permits(), 32);
    }
}
