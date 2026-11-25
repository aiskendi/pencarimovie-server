//! Mint a fresh Bot API `file_id` by RE-SENDING the document to a relay account.
//!
//! A `file_reference` lives about an hour, and only the bot that owns the file can
//! replace it: the Bot API's `sendDocument(chat_id, document)` is
//! `messages.sendMedia` carrying an `inputMediaDocument` built from the SAME
//! `(id, access_hash, file_reference)`, and the reply's document object holds the
//! new reference. `--mint-probe` proved that round-trip by hand; this module is the
//! same round-trip wired into the resolve path, so a cold Play starts with a
//! reference that is already alive instead of discovering `FILE_REFERENCE_EXPIRED`
//! and paying for a re-resolve the player does not wait for.
//!
//! It is an OPTIMISATION, never a dependency. The origin stays the source of truth
//! for the id: [`adopt`] hands back the caller's own body untouched whenever the
//! mint cannot run, and one log line names the reason. A mint failure must never
//! turn a playable file into an error.
//!
//! The send goes to a relay ACCOUNT, not a bot: Telegram refuses bot Ã¢â€ â€™ bot
//! outright (`400 USER_BOT_TO_BOT_DISABLED`, measured 2026-11-07 Ã¢â‚¬â€ see
//! `backfill::REQUEST_CAPTION`), so the peer list is the origin's userbot pool.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use grammers_client::media::Media;
use grammers_client::message::InputMessage;
use grammers_client::tl;
use serde_json::Value;

use crate::fileid;
use crate::log::log;
use crate::telegram;

/// How long a minted id is reused before the code is minted again.
///
/// UNTIL MEASURED. A minted reference is the same ~1 h `file_reference` as any
/// other, so the ceiling is the resolve cache's own 50 min, not this. 300 s is
/// deliberately short: it collapses the burst a cold start produces (a player
/// opens several Range requests, and a list re-open resolves the same code twice)
/// without ever serving a reference old enough to be dead on arrival. Raise it only
/// with a measurement of how long a re-sent reference actually lives.
const MINT_TTL_MS: i64 = 300_000;

/// Ceiling for ONE mint, so a slow Telegram round-trip cannot spend the request.
///
/// The resolve ladder is bounded by `resolve::RESOLVE_BUDGET` (20 s) and the
/// router kills a request at 30 s (Heroku `H12`), so this has to fit in the gap:
/// 6 s leaves 4 s of headroom even on the worst ladder. A measured mint is ~0.3 s.
const MINT_TIMEOUT: Duration = Duration::from_secs(6);

/// How long the peer list from `GET /userbots` is reused. The pool is the
/// operator's config, not per-request state, and a fetch is a signed origin call.
const PEERS_TTL_SECS: i64 = 3600;

/// Retry window when the fetch failed or answered no accounts. A full hour of
/// "empty" would disable the mint for the rest of the process after one blip.
const PEERS_RETRY_SECS: i64 = 60;

fn minted() -> &'static Mutex<HashMap<String, (String, i64)>> {
    static C: std::sync::OnceLock<Mutex<HashMap<String, (String, i64)>>> = std::sync::OnceLock::new();
    C.get_or_init(|| Mutex::new(HashMap::new()))
}

/// `(fetched_at, accounts, status, detail)` for the last `GET /userbots`.
///
/// The STATUS travels with the list because an empty list has two completely
/// different causes â€” the origin refused the lease (401) or it has no relay
/// accounts (200 + `accounts: []`) â€” and the caller is the one that has to say
/// which. `detail` is the sentence to report when the list is empty.
fn peer_cache() -> &'static Mutex<Option<(i64, Vec<String>, u16, String)>> {
    static P: std::sync::OnceLock<Mutex<Option<(i64, Vec<String>, u16, String)>>> =
        std::sync::OnceLock::new();
    P.get_or_init(|| Mutex::new(None))
}

/// Peers are tried round-robin so one relay account does not carry every send.
static NEXT_PEER: AtomicUsize = AtomicUsize::new(0);

/// Unix seconds before which NO mint is attempted (`0` = clear).
///
/// A `FLOOD_WAIT` is about the sending account, not the file, so retrying it for
/// the next code would just stack more waits onto the same bot. The resolve path
/// calls this module once per origin resolve, i.e. on the interactive path, where
/// an unbounded retry loop is exactly what must not happen.
static SUPPRESS_UNTIL: AtomicI64 = AtomicI64::new(0);

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// A freshly minted id and the relay account it was sent through.
#[derive(Debug, Clone)]
pub struct Minted {
    pub file_id: String,
    pub peer: String,
}

/// Why a mint did not happen, in a shape a log line and a diagnostic row can carry.
///
/// `reason` is one of `no_pool`, `bot_boot_failed`, `no_peers`, `peer_unresolved`,
/// `decode_failed`, `not_a_document`, `USER_BOT_TO_BOT_DISABLED`, `chat_not_found`,
/// `flood_wait`, `transport`, `reference_rejected`, `send_failed`,
/// `encode_failed`; `detail` is the origin/Telegram sentence behind it.
#[derive(Debug, Clone)]
pub struct MintError {
    pub reason: String,
    pub detail: String,
}

impl MintError {
    fn new(reason: &str, detail: impl Into<String>) -> Self {
        Self {
            reason: reason.to_string(),
            detail: detail.into(),
        }
    }
}

impl std::fmt::Display for MintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.reason, self.detail)
    }
}

fn mint_key(short_code: &str, bot_id: &str) -> String {
    format!("{}:{}", short_code, bot_id)
}

/* Ã¢â€â‚¬Ã¢â€â‚¬ the relay peer list Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬ */

/// The relay accounts from `GET {wp_api_base}/userbots`, cached in-process.
///
/// Authenticated by [`crate::httpc::http_json_h_status`] â€” the engine's own helper â€”
/// rather than by assembling the ticket and PM-Auth headers here. That is what gives
/// this request the 401 recovery every other origin call already had (clear the
/// refused lease, re-enrol, retry once): the hand-rolled version answered 401 until
/// the process restarted, and every mint in that window failed for the wrong reason.
pub async fn peers() -> Vec<String> {
    peers_with_detail().await.0
}

/// The accounts PLUS the sentence to report when there are none.
async fn peers_with_detail() -> (Vec<String>, String) {
    let now = crate::storage::now();
    if let Ok(g) = peer_cache().lock() {
        if let Some((at, list, _status, detail)) = &*g {
            let ttl = if list.is_empty() {
                PEERS_RETRY_SECS
            } else {
                PEERS_TTL_SECS
            };
            if now - *at < ttl {
                return (list.clone(), detail.clone());
            }
        }
    }
    let (list, status, detail) = fetch_peers().await;
    if let Ok(mut g) = peer_cache().lock() {
        *g = Some((now, list.clone(), status, detail.clone()));
    }
    (list, detail)
}

/// Pin the peer list for this process (`--mint-batch` with explicit peers, and
/// nothing else). Writing the cache is the whole point: every mint reads it.
pub fn seed_peers(list: Vec<String>) {
    if let Ok(mut g) = peer_cache().lock() {
        *g = Some((crate::storage::now(), list, 200, String::new()));
    }
}

/// `GET /userbots`, as `(accounts, http_status, empty_detail)`.
async fn fetch_peers() -> (Vec<String>, u16, String) {
    let url = format!(
        "{}/bots",
        crate::config::wp_api_base().trim_end_matches('/')
    );
    // The origin's protected route: `http_json_h_status` attaches the ticket and the
    // PM-Auth proof, and on 401 it clears the refused lease, re-enrols and retries
    // once â€” none of which the hand-rolled request did. `0` means it never completed.
    let (body, status) = crate::httpc::http_json_h_status(&url, &[], "GET", 12, &[]).await;
    let names = if (200..300).contains(&status) {
        accounts_of(&body)
    } else {
        Vec::new()
    };
    if names.is_empty() {
        // Always with the status: an empty pool and a refused lease must never read
        // the same in the log. The 401 recovery has already run inside
        // `http_json_h_status`, so a 401 here means the lease could not be replaced.
        if (200..300).contains(&status) {
            // Say WHAT arrived, not just that it was empty: a 200 with no `bots[]`
            // and a 200 whose `bots[]` is empty are different faults, and three
            // rounds of fixes went by without either being visible in a device log.
            let keys: Vec<&str> = body
                .as_object()
                .map(|o| o.keys().map(|k| k.as_str()).collect())
                .unwrap_or_default();
            log(
                "userbots answered no accounts",
                &serde_json::json!({
                    "url": url,
                    "status": status,
                    "keys": keys.join(","),
                    "bytes": body.to_string().len(),
                    "is_array": body.is_array(),
                }),
            );
        } else {
            log(
                "userbots fetch rejected",
                &serde_json::json!({"url": url, "status": status}),
            );
        }
    }
    let detail = if names.is_empty() {
        userbots_detail(status)
    } else {
        String::new()
    };
    (names, status, detail)
}

/// What to report when `GET /userbots` produced no relay accounts.
///
/// The status is the point. The old sentence â€” "GET /userbots answered no usable
/// accounts" â€” read as "the operator's pool is empty" when the truth was a refused
/// lease (`401`), which is exactly the log line this change exists to kill.
fn userbots_detail(status: u16) -> String {
    if (200..300).contains(&status) {
        format!("GET /bots answered {status} with no peers in accounts[], bots[] or data")
    } else {
        format!("GET /bots answered {status}")
    }
}

/// `accounts[]`, plus the shapes the same route has been seen behind.
///
/// The REST envelope is the only documented one; `data.accounts` and a bare array
/// cost nothing and keep a wrapper change on the origin from silently emptying the
/// peer list (an empty list means the mint never runs at all).
fn accounts_of(body: &Value) -> Vec<String> {
    // FIRST NON-EMPTY LIST WINS. The origin answers both keys at once â€” `bots[]`
    // with the live bots and `"accounts": []` kept for shape compatibility â€” so
    // taking the first *present* key stopped on the empty one and reported "no
    // accounts" on a 200 that carried 50 bots (measured 2026-11-25: bytes 5007,
    // keys accounts,bots,build,mode,note,ok,relay_bot_id).
    let candidates = [
        body.get("accounts").and_then(|v| v.as_array()),
        body.get("bots").and_then(|v| v.as_array()),
        body.get("data")
            .and_then(|d| d.get("accounts"))
            .and_then(|v| v.as_array()),
        body.get("data").and_then(|d| d.get("bots")).and_then(|v| v.as_array()),
        body.as_array(),
    ];
    let rows = candidates
        .into_iter()
        .flatten()
        .find(|a| !a.is_empty());
    let Some(rows) = rows else {
        return Vec::new();
    };
    let mut names: Vec<String> = Vec::new();
    let mut ids: Vec<String> = Vec::new();
    for a in rows {
        let name = a
            .get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .trim_start_matches('@')
            .to_string();
        if !name.is_empty() {
            if !names.iter().any(|s| *s == name) {
                names.push(name);
            }
            continue;
        }
        // A relay account without a username is still addressable when the bot
        // already has a chat with it (`telegram::resolve_peer` walks the dialog list
        // for a numeric id), and the pool manifest has published ids only.
        let id = match a
            .get("id")
            .or_else(|| a.get("user_id"))
            .or_else(|| a.get("bot_id"))
        {
            Some(Value::Number(n)) => n.to_string(),
            Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
            _ => continue,
        };
        if !ids.iter().any(|s| *s == id) {
            ids.push(id);
        }
    }
    // Usernames first: a username resolves without a dialog scan.
    names.extend(ids);
    names
}

/* Ã¢â€â‚¬Ã¢â€â‚¬ the mint Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬ */

fn send_reason(e: &telegram::GramErr) -> MintError {
    let m = &e.message;
    let reason = if m.contains("USER_BOT_TO_BOT_DISABLED") {
        "USER_BOT_TO_BOT_DISABLED"
    } else if m.contains("CHAT_NOT_FOUND")
        || m.contains("PEER_ID_INVALID")
        || m.contains("USER_ID_INVALID")
        || m.contains("CHANNEL_INVALID")
        || m.contains("CHANNEL_PRIVATE")
    {
        "chat_not_found"
    } else if e.flood_wait.is_some() || m.contains("FLOOD_WAIT") {
        "flood_wait"
    } else if e.transport {
        "transport"
    } else if e.file_reference {
        "reference_rejected"
    } else {
        "send_failed"
    };
    MintError::new(reason, m.clone())
}

/// Re-send `source_file_id` to one relay account and return the NEW Bot API id.
///
/// The id comes from the SEND RESULT, never from a second resolve: the reply's
/// document object is the only thing carrying the fresh `file_reference`. The
/// message is deleted afterwards (best effort) so the relay chat does not fill up;
/// a chat that forbids deletion must not cost the id just minted.
pub async fn mint(bot_id: &str, source_file_id: &str) -> Result<Minted, MintError> {
    let now = crate::storage::now();
    let until = SUPPRESS_UNTIL.load(Ordering::Relaxed);
    if now < until {
        return Err(MintError::new(
            "flood_wait",
            format!("mint suppressed for another {}s", until - now),
        ));
    }
    let decoded = fileid::decode_file_id(source_file_id)
        .map_err(|e| MintError::new("decode_failed", e))?;
    // Only a document type has an `inputDocument` to resend: a photo id sent
    // through `inputMediaDocument` is addressed as a document and fails on the
    // server with an error that names the wrong object.
    if !fileid::is_document_file_id(&decoded) {
        return Err(MintError::new(
            "not_a_document",
            format!("file_id type {} has no document to resend", decoded.type_name),
        ));
    }
    let Some(pool) = telegram::shared_pool() else {
        return Err(MintError::new(
            "no_pool",
            "this process has published no Telegram pool yet",
        ));
    };
    // Prefer the connection the engine already holds for this bot: booting a second
    // client for the same session is a second login for no benefit.
    let entry = match pool.get(bot_id).await {
        Some(e) => e,
        None => pool
            .boot(bot_id, None)
            .await
            .map_err(|e| MintError::new("bot_boot_failed", e))?,
    };
    let client = entry.pick();

    // The detail carries the STATUS of the fetch, so this rejection can never read
    // "no usable accounts" when the origin actually refused the lease.
    let (names, no_peers) = peers_with_detail().await;
    if names.is_empty() {
        return Err(MintError::new("no_peers", no_peers));
    }
    let start = NEXT_PEER.fetch_add(1, Ordering::Relaxed) % names.len();
    let mut tried = String::new();
    let mut resolved: Option<(grammers_session::types::PeerRef, String)> = None;
    for k in 0..names.len() {
        let name = &names[(start + k) % names.len()];
        match telegram::resolve_peer(&client, name).await {
            Ok(p) => {
                resolved = Some((p, name.clone()));
                break;
            }
            Err(e) => tried.push_str(&format!("{name}: {}; ", e.message)),
        }
    }
    let Some((peer_ref, peer_name)) = resolved else {
        // `None` from `to_ref()` lands here too: the peer exists but this session
        // has no `access_hash` for it (a bot cannot enumerate dialogs to find one).
        return Err(MintError::new("peer_unresolved", tried));
    };

    // No upload: the document already lives on Telegram, so this is a pure
    // `messages.sendMedia`. `video_cover`/`video_timestamp` stay null because this
    // is not editing a video's attributes.
    let media = tl::types::InputMediaDocument {
        spoiler: false,
        id: tl::enums::InputDocument::Document(tl::types::InputDocument {
            id: decoded.id,
            access_hash: decoded.access_hash,
            file_reference: decoded.file_reference.clone().unwrap_or_default(),
        }),
        video_cover: None,
        video_timestamp: None,
        ttl_seconds: None,
        query: None,
    };
    let message = match client
        .send_message(peer_ref, InputMessage::new().media(media))
        .await
    {
        Ok(m) => m,
        Err(e) => {
            let err = telegram::map_invocation_error(e);
            if let Some(wait) = err.flood_wait {
                // `wait` counts from Telegram's own answer; a missing value still
                // deserves a short pause rather than an immediate retry storm.
                SUPPRESS_UNTIL.store(
                    crate::storage::now() + wait.max(5) as i64,
                    Ordering::Relaxed,
                );
            }
            return Err(send_reason(&err));
        }
    };
    let _kept_message_id = message.id();

    // Read the reply, then delete the message in BOTH cases: a send whose answer is
    // the wrong shape still put a document in the relay chat, and leaving it there
    // would fill the chat on every failed mint.
    let replied = match message.media() {
        Some(Media::Document(m)) => match m.raw.document {
            Some(tl::enums::Document::Document(d)) => Ok(d),
            _ => Err(MintError::new(
                "send_failed",
                "the sent media carries no document",
            )),
        },
        _ => Err(MintError::new("send_failed", "the sent media is not a document")),
    };
    // The message is KEPT. It is what holds the document in a chat, so deleting it
    // can shorten the life of the very reference we just minted Ã¢â‚¬â€ and the origin
    // leaves its relay messages in place for the same reason. It also costs an API
    // call on the play path.
    let doc = replied?;

    // Version 4 / sub 0 is the wire shape this account's Bot API ids use (the live
    // fixture in `fileid` is 4/0), and the reply carries no encoded version of its
    // own. `file_type` is kept from the input: the send does not change what the
    // document IS, only its reference.
    let minted = fileid::DecodedFileId {
        id: doc.id,
        access_hash: doc.access_hash,
        file_reference: Some(doc.file_reference.clone()),
        dc_id: doc.dc_id,
        file_type: decoded.file_type,
        type_name: decoded.type_name.clone(),
        version: 4,
        sub_version: 0,
        web_url: None,
        size: doc.size.max(0) as u64,
        raw: String::new(),
    };
    let file_id = fileid::encode_file_id(&minted);
    if file_id.is_empty() {
        return Err(MintError::new(
            "encode_failed",
            "encode_file_id produced an empty id",
        ));
    }
    Ok(Minted {
        file_id,
        peer: peer_name,
    })
}

/* Ã¢â€â‚¬Ã¢â€â‚¬ the resolve-path seam Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬ */

fn cached(short_code: &str, bot_id: &str) -> Option<String> {
    let g = minted().lock().ok()?;
    let (id, at) = g.get(&mint_key(short_code, bot_id))?;
    if now_ms() - *at < MINT_TTL_MS {
        Some(id.clone())
    } else {
        None
    }
}

fn remember(short_code: &str, bot_id: &str, file_id: &str) {
    let Ok(mut g) = minted().lock() else {
        return;
    };
    // Bounded like `resolve::batch_lock`: expired rows are the only ones dropped.
    if g.len() > 1024 {
        let now = now_ms();
        g.retain(|_, (_, at)| now - *at < MINT_TTL_MS);
    }
    g.insert(
        mint_key(short_code, bot_id),
        (file_id.to_string(), now_ms()),
    );
}

/* Ã¢â€â‚¬Ã¢â€â‚¬ the local re-send (CHANGE 1) Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬Ã¢â€â‚¬ */

/// The id THIS process minted for the pair, while it is still inside
/// [`MINT_TTL_MS`] and therefore still live.
///
/// It is the one source a local re-send can always address: the bot that sends is
/// the bot the id was minted for (`mint` encodes the send's reply). An id the
/// CALLER holds is not that guarantee Ã¢â‚¬â€ it may have been minted by a different bot,
/// and `sendMedia` answers `MEDIA_EMPTY` for it. That is why `resolve` tries this
/// rung first and the caller's id only after the origin's `fresh=1` mint has
/// answered nothing usable.
pub fn live(short_code: &str, bot_id: &str) -> Option<String> {
    cached(short_code, bot_id)
}

/// Re-send `source` and return the NEW `file_id`, or `None` if the send did not
/// produce one.
///
/// `messages.sendMedia` carrying the SAME document returns a document object with
/// a brand-new `file_reference` (~100-390 ms measured, and a minted id can seed
/// the next mint), so a live id is refreshed locally instead of paying the
/// origin's `force=1&nocache=1` re-mint.
///
/// Exactly ONE send is attempted. A refused send is Telegram saying the id is
/// DEAD (`reference_rejected`) or not this bot's (`MEDIA_EMPTY`), and retrying it
/// would spend another round-trip on the path that is already late Ã¢â‚¬â€ so the answer
/// is `None` and the caller tries its next rung, which is exactly the behaviour
/// before this function existed. A mint failure can never change a request's
/// outcome.
///
/// `source_label` is `minted_id` / `caller_id` and exists only so the log line
/// says WHICH rung's id was refused; the two are otherwise the same operation.
pub async fn resend(
    short_code: &str,
    bot_id: &str,
    source: &str,
    source_label: &'static str,
) -> Option<String> {
    if short_code.trim().is_empty() || bot_id.trim().is_empty() {
        return None;
    }
    let source = source.trim();
    if source.is_empty() {
        return None;
    }
    match tokio::time::timeout(MINT_TIMEOUT, mint(bot_id, source)).await {
        Ok(Ok(minted)) => {
            remember(short_code, bot_id, &minted.file_id);
            report(short_code, bot_id, &minted.file_id);
            log(
                "refreshed the file reference locally - no origin re-mint",
                &serde_json::json!({
                    "short_code": short_code,
                    "bot_id": bot_id,
                    "peer": minted.peer,
                    "source": source_label,
                    "same_id": minted.file_id == source,
                }),
            );
            Some(minted.file_id)
        }
        Ok(Err(e)) => {
            // `reference_rejected` here is the proof the id is dead; every other
            // reason is the same story for the caller: try the next rung.
            log(
                "local refresh failed - falling back",
                &serde_json::json!({
                    "short_code": short_code,
                    "bot_id": bot_id,
                    "reason": e.reason,
                    "detail": e.detail,
                    "source": source_label,
                }),
            );
            None
        }
        Err(_) => {
            log(
                "local refresh timed out - falling back",
                &serde_json::json!({
                    "short_code": short_code,
                    "bot_id": bot_id,
                    "timeout_secs": MINT_TIMEOUT.as_secs(),
                    "source": source_label,
                }),
            );
            None
        }
    }
}

/// Report a minted id to the origin so the next caller for this `(file, bot)` finds
/// it in step 3 instead of paying another relay send. Fire-and-forget: spawned,
/// logs at most one line, and can never fail a mint or a play.
///
/// The origin's own `fresh=1` mint reaches it too, so the next resolve for that
/// pair is a step-3 hit rather than a second relay send.
pub(crate) fn report(short_code: &str, bot_id: &str, file_id: &str) {
    let sc = short_code.to_string();
    let bid = bot_id.to_string();
    let fid = file_id.to_string();
    tokio::spawn(async move {
        let url = format!(
            "{}/file-id",
            crate::config::wp_api_base().trim_end_matches('/')
        );
        let body = serde_json::json!({"short_code": sc, "bot_id": bid, "file_id": fid}).to_string();
        let mut req = crate::httpc::client()
            .post(&url)
            .timeout(std::time::Duration::from_secs(8))
            .header("Content-Type", "application/json")
            .body(body);
        for (k, v) in crate::credentials::ticket_headers()
            .into_iter()
            .chain(crate::httpc::pm_auth_headers("POST", &url))
        {
            req = req.header(k.as_str(), v.as_str());
        }
        match req.send().await {
            Ok(r) if r.status().is_success() => log(
                "reported the minted file id to the origin",
                &serde_json::json!({"short_code": sc}),
            ),
            Ok(r) => log(
                "origin refused the minted file id",
                &serde_json::json!({"short_code": sc, "status": r.status().as_u16()}),
            ),
            Err(e) => log(
                "reporting the minted file id failed",
                &serde_json::json!({"short_code": sc, "error": e.to_string()}),
            ),
        }
    });
}

/// Test-only: seed the mint cache as if this process had just minted `file_id`.
///
/// `remember` stays private â€” a caller that could cache an id it did not mint would
/// defeat the one guarantee this cache makes (the id belongs to the bot that will
/// re-send it). The resolve path's rung order is the thing the test needs to see.
#[cfg(test)]
pub(crate) fn seed_minted(short_code: &str, bot_id: &str, file_id: &str) {
    remember(short_code, bot_id, file_id);
}

/// The id the origin answered, in the order the rest of the engine reads it.
fn source_id(v: &Value) -> Option<String> {
    for key in ["file_id_mt", "file_id_tmp", "file_id"] {
        let id = v
            .get(key)
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if !id.is_empty() {
            return Some(id);
        }
    }
    None
}

/// An id the origin minted for THIS bot, via the handover (`fresh=1` skips the
/// stored row and lets inputfilebot send the document to the requesting bot, so the
/// id in the response belongs to the bot that will re-send it). `None` means the
/// origin did not answer with anything usable, and the caller keeps what it had.
async fn fresh_seed(short_code: &str, bot_id: &str) -> Option<String> {
    let url = format!(
        "{}/resolve-file",
        crate::config::wp_api_base().trim_end_matches('/')
    );
    let params: Vec<(String, String)> = vec![
        ("short_code".to_string(), short_code.to_string()),
        ("bot_id".to_string(), bot_id.to_string()),
        ("fresh".to_string(), "1".to_string()),
    ];
    let (body, status) = crate::httpc::http_json_h_status(&url, &params, "GET", 8, &[]).await;
    if !(200..300).contains(&status) {
        return None;
    }
    source_id(&body)
}

/// A minted id for this `(short_code, bot_id)`, minting one if the cache is cold.
///
/// `None` means "keep the origin's id": the caller must not distinguish a missing
/// pool from a refused send, because neither is worth failing a request over.
pub async fn ensure(short_code: &str, bot_id: &str, resolved: &Value) -> Option<String> {
    if short_code.trim().is_empty() || bot_id.trim().is_empty() {
        return None;
    }
    if let Some(id) = cached(short_code, bot_id) {
        return Some(id);
    }
    let source = source_id(resolved)?;
    match tokio::time::timeout(MINT_TIMEOUT, mint(bot_id, &source)).await {
        Ok(Ok(minted)) => {
            remember(short_code, bot_id, &minted.file_id);
            report(short_code, bot_id, &minted.file_id);
            log(
                "minted a fresh file id",
                &serde_json::json!({
                    "short_code": short_code,
                    "bot_id": bot_id,
                    "peer": minted.peer,
                    "minted_differs": minted.file_id != source,
                }),
            );
            Some(minted.file_id)
        }
        Ok(Err(e)) => {
            // A REFUSED SEND IS USUALLY THE WRONG OWNER, NOT A BAD FILE.
            // The origin's answer can carry an id minted for another bot, and
            // `messages.sendMedia` then answers MEDIA_EMPTY in ~125 ms — instant,
            // before any network round trip. Measured repeatedly on 2026-11-25:
            // the same code mints fine from the id `fresh=1` returns for THIS bot
            // and fails from the one a plain resolve returns. So ask the origin for
            // a bot-owned id once and retry, before giving up on the mint.
            if let Some(fresh) = fresh_seed(short_code, bot_id).await {
                if fresh != source {
                    match tokio::time::timeout(MINT_TIMEOUT, mint(bot_id, &fresh)).await {
                        Ok(Ok(minted)) => {
                            remember(short_code, bot_id, &minted.file_id);
                            report(short_code, bot_id, &minted.file_id);
                            log(
                                "minted a fresh file id",
                                &serde_json::json!({
                                    "short_code": short_code,
                                    "bot_id": bot_id,
                                    "peer": minted.peer,
                                    "minted_differs": minted.file_id != fresh,
                                    "seed": "fresh_handover",
                                }),
                            );
                            return Some(minted.file_id);
                        }
                        Ok(Err(e2)) => log(
                            "mint skipped - the bot-owned id was refused too",
                            &serde_json::json!({
                                "short_code": short_code,
                                "bot_id": bot_id,
                                "reason": e2.reason,
                                "detail": e2.detail,
                                "first": e.detail,
                            }),
                        ),
                        Err(_) => log(
                            "mint timed out on the bot-owned id",
                            &serde_json::json!({"short_code": short_code, "bot_id": bot_id}),
                        ),
                    }
                }
            }
            log(
                "mint skipped - keeping the origin's file id",
                &serde_json::json!({
                    "short_code": short_code,
                    "bot_id": bot_id,
                    "reason": e.reason,
                    "detail": e.detail,
                }),
            );
            None
        }
        Err(_) => {
            log(
                "mint timed out - keeping the origin's file id",
                &serde_json::json!({
                    "short_code": short_code,
                    "bot_id": bot_id,
                    "timeout_secs": MINT_TIMEOUT.as_secs(),
                }),
            );
            None
        }
    }
}

/// Replace a resolve body's id with a minted one, or hand it back unchanged.
///
/// The CALLER persists the result (`resolve.rs` already calls
/// `save_resolve_cache` on every branch where an id exists), and that save stamps
/// `resolved_at` afresh Ã¢â‚¬â€ which is load-bearing: `download.rs` trusts an unverified
/// id for 60 s from that stamp, and the origin's stamp would make a brand-new
/// reference look stale. Both id fields are rewritten so nothing downstream can
/// pick the pre-mint reference out of `file_id`.
pub async fn adopt(short_code: &str, bot_id: &str, resolved: Value) -> Value {
    if source_id(&resolved).is_none() {
        return resolved;
    }
    // CACHE ONLY. Resolving must never send: the origin's id is used as-is, and a
    // mint happens only after Telegram actually refuses the reference
    // (`resolve::refresh_before_origin`).
    // Minting here cost one relay send per cold (code, bot) even when the stored id
    // was perfectly good, which is the duplicate send this whole change removes.
    let Some(id) = cached(short_code, bot_id) else {
        return resolved;
    };
    if resolved.get("file_id_mt").and_then(|v| v.as_str()) == Some(id.as_str()) {
        return resolved;
    }
    let mut out = resolved;
    if let Some(obj) = out.as_object_mut() {
        obj.insert("file_id_mt".to_string(), serde_json::json!(id));
        obj.insert("file_id".to_string(), serde_json::json!(id));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accounts_are_read_from_the_rest_envelope() {
        let body = serde_json::json!({
            "ok": 1,
            "accounts": [
                {"username": "@BannedMany"},
                {"username": ""},
                {"username": ""},
                {"username": "Matalipmy"},
            ]
        });
        assert_eq!(accounts_of(&body), vec!["BannedMany", "Matalipmy"]);
    }

    /// A pool that publishes ids only (which is what the manifest did) must still
    /// produce candidates: `telegram::resolve_peer` handles a numeric id through
    /// the bot's dialog list. Usernames still come FIRST.
    #[test]
    fn a_nameless_account_falls_back_to_its_id() {
        let body = serde_json::json!({
            "accounts": [
                {"username": "", "id": 7000000001i64},
                {"username": "relay", "id": 7000000002i64},
            ]
        });
        assert_eq!(accounts_of(&body), vec!["relay", "7000000001"]);
    }

    #[test]
    fn an_answer_without_accounts_is_empty_not_a_panic() {
        assert!(accounts_of(&serde_json::json!({"ok": 0})).is_empty());
        assert!(accounts_of(&serde_json::json!("nope")).is_empty());
    }

    /// The fallback sentence a mint reports must name the HTTP status.
    ///
    /// The bug it closes: a refused lease (`401`) and an empty relay pool (`200`)
    /// both produced "GET /userbots answered no usable accounts", so a device log
    /// pointed at the operator's pool while the truth was PM-Auth.
    #[test]
    fn an_empty_peer_list_names_the_http_status() {
        let refused = userbots_detail(401);
        assert_eq!(refused, "GET /bots answered 401");
        assert!(refused.contains("401"));
        assert!(!refused.contains("no usable accounts"));

        // A call that never completed has no status; `0` is the engine's own word
        // for that everywhere else, and it must not read as an empty pool either.
        assert_eq!(userbots_detail(0), "GET /bots answered 0");

        // A genuine 200 with no accounts says so, and still names the status.
        let empty = userbots_detail(200);
        assert!(empty.contains("200"));
        assert!(!empty.contains("no usable accounts"));
    }

    /// The three id keys, in the order the engine reads them.
    #[test]
    fn source_id_prefers_file_id_mt_then_tmp_then_plain() {
        let all = serde_json::json!({"file_id_mt": "MT", "file_id_tmp": "TMP", "file_id": "PLAIN"});
        assert_eq!(source_id(&all).as_deref(), Some("MT"));
        let tmp = serde_json::json!({"file_id_tmp": "TMP", "file_id": "PLAIN"});
        assert_eq!(source_id(&tmp).as_deref(), Some("TMP"));
        assert!(source_id(&serde_json::json!({"ok": 0})).is_none());
    }

    #[test]
    fn a_minted_id_is_remembered_per_code_and_bot() {
        remember("codeA", "botA", "minted-1");
        assert_eq!(cached("codeA", "botA").as_deref(), Some("minted-1"));
        assert!(cached("codeA", "botB").is_none());
        assert!(cached("codeB", "botA").is_none());
    }

    /// CHANGE 1's first rung: only an id THIS process minted is offered as the one
    /// a local re-send can always address. It is keyed like every other mint
    /// (`code:bot`), so another pair's mint is never handed to this one.
    #[test]
    fn a_live_mint_is_offered_only_for_its_own_pair() {
        remember("liveA", "botA", "minted-live");
        assert_eq!(live("liveA", "botA").as_deref(), Some("minted-live"));
        assert!(live("liveA", "botB").is_none(), "a file id is bot-specific");
        assert!(live("liveB", "botA").is_none(), "and it is code-specific");
    }

    /// `resend` with nothing to send must not reach for a pool, a peer or a
    /// network: it answers `None` so the caller keeps its ladder.
    #[tokio::test]
    async fn a_resend_with_nothing_to_send_does_no_work() {
        assert!(resend("", "botC", "held", "caller_id").await.is_none());
        assert!(resend("codeC", "", "held", "caller_id").await.is_none());
        assert!(resend("codeC", "botC", "   ", "caller_id").await.is_none());
    }

    /// `adopt` without a pool must not touch the body: that is the whole fallback
    /// contract (`no_pool` is the reason on this test process).
    #[tokio::test]
    async fn adopt_keeps_the_origin_id_when_the_mint_cannot_run() {
        let body = serde_json::json!({"file_id_mt": "ORIGIN", "file_id": "ORIGIN", "title": "t", "file_size": 1});
        let out = adopt("codeZ", "botZ", body.clone()).await;
        assert_eq!(out, body);
    }
}
