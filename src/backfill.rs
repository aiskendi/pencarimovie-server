//! Backfill: put a recording Telegram does not hold into Telegram, then play it
//! from OUR OWN send result.
//!
//! The order matters and each gate is cheaper than the next step:
//!
//! 1. the local ledger — did *this engine* already backfill this track?
//! 2. the origin's `search-music` — is a row of exactly this byte size already
//!    indexed? (best effort: if the origin refuses, we continue and rely on 1)
//! 3. fetch the master as a STREAM, never a buffer: `bytes_stream()` feeds
//!    `upload_stream()` directly, so a 24.8 MB FLAC costs no temp file and no
//!    full-size allocation.
//! 4. send it to a relay account (a USERBOT, not a bot — Telegram does not
//!    deliver bot-to-bot messages) and keep the `DocTarget` out of the send
//!    result. That handle is what makes the file playable immediately, without
//!    waiting for the origin to index it.
//!
//! The origin's webhook writes `tg_file_new` afterwards, which is what makes the
//! track show up in search for every client; step 4 is what makes it play now.

use futures_util::TryStreamExt;
use grammers_client::Client;
use serde_json::{json, Value};
use tokio_util::io::StreamReader;

use crate::log::log;

/// Where a hi-res/lossless master can be fetched from — the ONE usable source.
///
/// There is no fallback list any more, and the removed candidates are named here
/// so nobody re-adds one on a hunch. Measured 2026-11-07 → 2026-11-09:
///
/// | Candidate                                        | Answer                                                                                     |
/// | ------------------------------------------------ | ------------------------------------------------------------------------------------------ |
/// | `heisenbergbackaaahh.de.deplexo.com/a`           | the only one that serves masters (`/search`, `/stream/{id}`, `fmt=6` FLAC)                  |
/// | `bitchord-hires-addon.de.deplexo.com/a`          | `{"error":"bitchord_auth","code":"missing"}` on every path — it wants the official app's credential, not a bearer |
/// | `bitchord-monochrome-addon.onrender.com`         | live manifest, `{"tracks":[]}` for every query                                              |
/// | `bit-chord-addon.vercel.app/a`, `/b`, `/e`       | `{"error":"upstream service unavailable"}` / `{"error":"request failed"}`                   |
///
/// A fallback that can only ever refuse does real damage: it spends a request on
/// every failure AND makes the log name a second cause that is not why anything
/// broke (that is how `bitchord… -> 401` once hid `heisenberg… -> 503`).
///
/// THERE IS NO COMPILED-IN SOURCE ANY MORE. A fill is turned on by a source the
/// operator configured — never by a host baked into the binary.
///
/// A default host in the binary is a default somebody else's server pays for: a
/// fresh clone backfills against an address nobody chose, and when that address
/// starts refusing, every single search pays a request for it and the log fills
/// with `source refused {"status":503}`. MEASURED 2026-11-10: that ran all morning
/// against the old default, on an install that had never configured a source.
///
/// It is a LIST — comma, semicolon or whitespace separated — because a single host
/// is not a reliable thing in this ecosystem: in one minute,
/// `bit-chord-addon.vercel.app/a|/b|/d|/e|/f` answered 502/500/502/400/502 while
/// `/` and `/c` answered 200 with full-length FLAC masters. Each entry is tried in
/// order and the first answer wins, so one dead host is not a dead feature.
///
/// TWO LAYERS, and the precedence is deliberate:
///
/// 1. the dashboard's `music_sources` — ENABLED entries only, in the order the
///    operator listed them. These were validated on entry by
///    [`crate::music_upstreams::probe`], so they are the ones that actually work;
/// 2. `PM_HIRES_BASE`, appended AFTER them, deduped.
///
/// So the env var stays supported exactly as it was, as the BASE LAYER and the
/// fallback: an install that saved no source (or disabled them all) behaves as
/// before, and a source added in the dashboard is tried FIRST because it is the
/// one the operator can see and just tested. A URL present in both is one entry,
/// not two — a doubled entry would spend the same request twice per fill.
fn hires_bases() -> Vec<String> {
    let settings = crate::routes::load_catalog_settings_merged();
    let mut out = crate::music_upstreams::enabled_bases(&settings);
    if let Ok(raw) = std::env::var("PM_HIRES_BASE") {
        for base in parse_bases(&raw) {
            if !out.iter().any(|seen| seen == &base) {
                out.push(base);
            }
        }
    }
    out
}

/// The list inside `PM_HIRES_BASE`, split and normalised. Pure, so it is testable
/// without mutating the process environment.
fn parse_bases(raw: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for part in raw.split(|c: char| c == ',' || c == ';' || c.is_whitespace()) {
        let base = part.trim().trim_end_matches('/');
        // A bare host is a configuration mistake worth skipping rather than
        // probing: `http_json_h_status` would report a transport error and the
        // reason would read as "the source is down".
        if base.is_empty()
            || !(base.starts_with("http://") || base.starts_with("https://"))
        {
            continue;
        }
        if !out.iter().any(|seen| seen == base) {
            out.push(base.to_string());
        }
    }
    out
}

/// The caption put on the send: a bare mention of the indexer bot.
///
/// MEASURED 2026-11-07 — this is the only way this engine can get a file INDEXED:
///
/// - the catalogue's rows come from `saving_do_action()`, and
///   `parse2.php:55` calls it for ONE bot id, `1422930632`;
/// - our sender is a bot, and Telegram refuses bot → bot outright:
///   `400 USER_BOT_TO_BOT_DISABLED caused by messages.sendMedia` (sent one
///   throwaway 15 MB track to prove it — the upload completed, the send did not);
/// - a bot → USER send works, and `tgfile_userbot_file_request_fallback()` shows
///   the origin's own convention: media handed to a relay account with a caption
///   naming a bot, which that account's userbot automation then delivers to the
///   bot. That is a user → bot hop, which Telegram allows.
///
/// So the file goes to the relay account with this mention and the origin's
/// userbot does the hop. Verified end-to-end by watching for the row, not assumed.
pub const REQUEST_CAPTION: &str = "File Request By @inputfilebot";

/// How many seconds this process waits between two fills it STARTS.
///
/// MEASURED 2026-11-10 on the deployed fleet: four fills on one dyno inside
/// twenty seconds (four different searches, four different users) were enough for
/// the CDN to answer every subsequent `size probe` with `429 Too Many Requests`,
/// and from then on nothing filled at all — `on-demand fill skipped {"reason":
/// "size probe: HTTP 429"}` ten times in one window. A fill costs several requests
/// to that host, so the budget is fills-per-minute, not requests-per-fill.
///
/// Every SEARCH that finds no hi-res row triggers a fill, which is right for one
/// listener and is a hammer for a busy fleet. This paces it.
const FILL_GAP_SECS: i64 = 75;

/// Seconds to stop asking after the source said 429.
const FILL_COOL_OFF_SECS: i64 = 120;

static NEXT_FILL_AT: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);
static COOL_UNTIL: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

/// Whether a fill may start now, and if not, why — in words a log line can carry.
fn fill_gate() -> Result<(), String> {
    use std::sync::atomic::Ordering::SeqCst;
    let now = crate::storage::now();
    let cool = COOL_UNTIL.load(SeqCst);
    if now < cool {
        return Err(format!(
            "source cooling down after a rate limit ({}s left)",
            cool - now
        ));
    }
    let next = NEXT_FILL_AT.load(SeqCst);
    if now < next {
        return Err(format!("fill pacing: next one in {}s", next - now));
    }
    NEXT_FILL_AT.store(now + FILL_GAP_SECS, SeqCst);
    Ok(())
}

/// Stop asking for a while. Called on 429, never on a plain refusal.
fn source_cool_off(secs: i64) {
    use std::sync::atomic::Ordering::SeqCst;
    let until = crate::storage::now() + secs.clamp(30, 600);
    let prev = COOL_UNTIL.fetch_max(until, SeqCst);
    if prev < until {
        log(
            "backfill: source asked us to slow down",
            &json!({"cool_off_s": secs.clamp(30, 600)}),
        );
    }
}

/// GET `path` from the first source that answers it, in configured order.
///
/// The failure string stays deterministic — `tools/probe_lease_release.mjs` asserts
/// that two consecutive failed attempts fail IDENTICALLY, and that equality is the
/// proof that a lease was released rather than held. So the list is joined in
/// CONFIGURED order, not in the order the requests happened to fail: two runs with
/// the same dead hosts produce the same string.
async fn source_get(path: &str, timeout_secs: u64) -> Result<Value, String> {
    let bases = hires_bases();
    if bases.is_empty() {
        return Err(
            "backfill is disabled (no music upstream: add one in the dashboard, or set PM_HIRES_BASE)"
                .to_string(),
        );
    }
    let mut tried: Vec<String> = Vec::new();
    for base in &bases {
        let url = format!("{base}{path}");
        // BOUNDED PER BASE, from the outside.
        //
        // `timeout_secs` is passed to the helper, but a server that accepts the
        // connection and then dribbles (or never answers the body) is not a connect
        // timeout — MEASURED 2026-11-10: a fill logged the trigger, booted the bot
        // and then sat silent for minutes with NO `source refused` line, i.e. the
        // first base never produced a status at all. With a list this matters twice
        // over: N bases must cost at most N x this, never forever.
        let (body, status) = match tokio::time::timeout(
            std::time::Duration::from_secs(timeout_secs),
            crate::httpc::http_json_h_status(&url, &[], "GET", timeout_secs, &[]),
        )
        .await
        {
            Ok(pair) => pair,
            Err(_) => {
                log(
                    "backfill: source timed out",
                    &json!({"url": url, "secs": timeout_secs}),
                );
                tried.push(format!("{url} -> timeout after {timeout_secs}s"));
                continue;
            }
        };
        if status == 200 {
            if tried.len() > 0 {
                log(
                    "backfill: source answered after a fallback",
                    &json!({"url": url, "skipped": tried.len()}),
                );
            }
            return Ok(body);
        }
        // A source that names its own refusal is worth the log line: the `code`
        // field is the difference between "down right now" and "will never answer
        // us".
        let why = body.get("code").and_then(|v| v.as_str()).unwrap_or("");
        log(
            "backfill: source refused",
            &json!({"url": url, "status": status, "why": why}),
        );
        // 429 is the one status that is about US rather than about the track, so
        // it cools the whole source off instead of falling through to the next
        // base as if the track were missing.
        if status == 429 {
            source_cool_off(FILL_COOL_OFF_SECS);
            return Err(format!("{url} -> 429 (rate limited, cooling off)"));
        }
        tried.push(format!("{url} -> {status}"));
    }
    Err(tried.join(" | "))
}

/// A file we sent and can serve, described the way the byte path wants it.
pub struct Backfilled {
    pub target: crate::telegram::DocTarget,
    pub message_id: i32,
    pub size: u64,
    pub file_name: String,
}

/* ── small helpers ───────────────────────────────────────────────────────── */

fn jstr(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// RFC 3986 percent-encoding for one query value. Two lines of real work, so it
/// does not justify a dependency for the single call site.
fn enc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(*b as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// Delete apostrophes instead of leaving them for the origin to space out.
///
/// MEASURED 2026-11-08, and it cost seven duplicate uploads: the origin's
/// `filter_filename()` replaces every character outside `[letter, digit, &, space]`
/// with a SPACE, so the file we named `What I've Done` is stored as
/// `What I ve Done` and its search tokens become `what i ve done`. Every client
/// types `what ive done` — one token — so Manticore can never match the row and
/// the song looks unindexed forever. Deleting the apostrophe here leaves
/// `What Ive Done` → tokens `what ive done`, which is what people search, and the
/// origin's filter then has no punctuation left to break.
///
/// Curly forms included: a phone keyboard sends `’`, not `'`.
fn drop_apostrophes(s: &str) -> String {
    s.chars()
        .filter(|c| !matches!(c, '\'' | '\u{2019}' | '\u{2018}' | '`' | '\u{00B4}'))
        .collect()
}

/// A file name Telegram will accept: no path separators, no characters that
/// break a caption.
fn safe_name(artist: &str, title: &str, ext: &str) -> String {
    let raw = if artist.is_empty() {
        title.to_string()
    } else {
        format!("{artist} - {title}")
    };
    let raw = drop_apostrophes(&raw);
    let cleaned: String = raw
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\n' | '\r' => ' ',
            c => c,
        })
        .collect();
    let cleaned = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if cleaned.is_empty() {
        format!("track.{ext}")
    } else {
        format!("{cleaned}.{ext}")
    }
}

/* ── ledger (the reliable duplicate gate) ────────────────────────────────── */

fn ledger_path() -> std::path::PathBuf {
    crate::storage::storage_path("backfill.json")
}

fn ledger_load() -> Value {
    std::fs::read_to_string(ledger_path())
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| json!({}))
}

fn ledger_get(key: &str) -> Option<Value> {
    ledger_load().get(key).cloned()
}

fn ledger_put(key: &str, entry: Value) {
    let mut doc = ledger_load();
    if let Some(obj) = doc.as_object_mut() {
        obj.insert(key.to_string(), entry);
        if let Ok(text) = serde_json::to_string(&doc) {
            let _ = std::fs::write(ledger_path(), text);
        }
    }
}

fn ledger_del(key: &str) {
    let mut doc = ledger_load();
    if let Some(obj) = doc.as_object_mut() {
        if obj.remove(key).is_some() {
            if let Ok(text) = serde_json::to_string(&doc) {
                let _ = std::fs::write(ledger_path(), text);
            }
        }
    }
}

/* ── the origin's own duplicate check, best effort ───────────────────────── */

/// `Some(short_code)` when the catalogue already holds a row of EXACTLY this
/// byte size for this query.
///
/// Byte size is the key on purpose: a lossy export of the same song never lands
/// on the same size as the lossless master, and the lossless master is one
/// byte-for-byte file no matter which addon handed it over. `None` also means
/// "the origin refused to answer" — which must not block a backfill, so the
/// caller still has the local ledger as the real gate.
async fn already_indexed(title: &str, artist: &str, size: u64) -> Option<String> {
    // Two spellings, because the catalogue holds both: rows sent before this fix
    // carry the origin's spaced form (`What I ve Done`), rows sent after carry the
    // deleted form (`What Ive Done`). Searching only one of them is exactly how
    // seven copies of the same 45.9 MB master got uploaded.
    for q in search_spellings(title, artist) {
        if let Some(code) = already_indexed_once(&q, size).await {
            return Some(code);
        }
    }
    None
}

/// Every query the duplicate gate should try, deduped, empty ones dropped.
fn search_spellings(title: &str, artist: &str) -> Vec<String> {
    let raw = format!("{title} {artist}");
    let joined = drop_apostrophes(&raw)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let spaced = raw
        .replace(['\'', '\u{2019}', '\u{2018}', '`', '\u{00B4}'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let mut out: Vec<String> = Vec::new();
    for q in [joined, spaced] {
        if !q.is_empty() && !out.contains(&q) {
            out.push(q);
        }
    }
    out
}

async fn already_indexed_once(q: &str, size: u64) -> Option<String> {
    let url = format!(
        "{}/search-music?search={}&limit=25",
        crate::config::wp_api_base().trim_end_matches('/'),
        enc(&q)
    );
    let (body, status) = crate::httpc::http_json_h_status(&url, &[], "GET", 15, &[]).await;
    if status != 200 {
        log(
            "backfill: origin duplicate check unavailable",
            &json!({"q": q, "status": status}),
        );
        return None;
    }
    let rows = body
        .get("tracks")
        .or_else(|| body.get("items"))
        .or_else(|| body.get("files"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    for row in rows {
        if row.get("file_size").and_then(|v| v.as_u64()) == Some(size) {
            return Some(jstr(&row, "short_code"));
        }
    }
    None
}

/* ── relay peer ─────────────────────────────────────────────────────────── */

/// The relay accounts a backfill sends through.
///
/// The file goes to one of these, the account auto-forwards it to `@inputfilebot`
/// (1422930632), and that bot's webhook is the one wired to index inbound media.
///
/// By USERNAME, not id: `messages.sendMedia` needs a peer reference carrying an
/// `access_hash`, and the pool manifest publishes ids only — a bot cannot call
/// `messages.getDialogs` (BOT_METHOD_INVALID) to find one, so an id is not an
/// address at all.
const RELAY_USERNAMES: [&str; 2] = ["BannedMany", "Matalipmy"];

/// The first relay account this bot can actually reach.
///
/// Resolving is cheap; sending 25 MB is not. So every candidate is resolved
/// first and the upload happens once, to whichever one answered.
async fn resolve_relay(
    client: &Client,
    explicit: &str,
) -> Result<(grammers_session::types::PeerRef, String), String> {
    let mut tried = String::new();
    let mut candidates: Vec<&str> = Vec::new();
    if !explicit.trim().is_empty() {
        candidates.push(explicit.trim());
    }
    candidates.extend(RELAY_USERNAMES);
    for name in candidates {
        match crate::telegram::resolve_peer(client, name).await {
            Ok(peer) => return Ok((peer, name.to_string())),
            Err(e) => tried.push_str(&format!(" {name}: {e};")),
        }
    }
    Err(format!("no reachable relay account —{tried}"))
}

/* ── the on-demand trigger ──────────────────────────────────────────────── */

/// At most ONE fill runs in a process at a time.
///
/// MEASURED 2026-11-08 19:14–19:15 UTC — the first window after the source
/// started serving real masters again: five fills landed inside 25 s
/// (17–45 MB each) and the fleet logged `Error R14 (Memory quota exceeded)`
/// **eleven times in forty seconds on all five dynos**, while the router's 5xx
/// count went from 0 to 19. A fill streams a whole master through
/// `upload_stream` while up to eight user streams are live in the same 512 MB
/// process, so two of them cannot be allowed to overlap.
///
/// The gate is per process, not per fleet: each dyno decides for itself, which
/// is the right scope because the memory it protects is per dyno.
///
/// IT IS A LEASE, NOT A SEMAPHORE, and that is the whole fix for 2026-11-10.
///
/// The semaphore version held its permit inside the spawned task, so the permit
/// was released when the task ENDED — which is exactly what a hung upload never
/// does. Live evidence from the five-dyno fleet, 17:15 UTC, for over an hour on
/// three unrelated tracks:
///
/// ```text
/// backfill: catalogue has no hi-res lossless {"q":"geisha reverb slowed","rows":3}
/// backfill: a fill is already running in this process - deferred {"q":"geisha reverb slowed"}
/// ```
///
/// …repeated for every query on that dyno, while `source master sized` /
/// `streaming the master` stayed at 0: one hung `upload_stream` owned the gate
/// forever and only a restart cleared it. A timestamp lease cannot be held by a
/// corpse, and it is the same TTL rule [`reserve()`] already applies to the
/// ledger (300 s there, because that lease covers one short API round trip; this
/// one covers a whole upload, so it is longer).
///
/// [`FILL_LEASE_SECS`] must therefore exceed the slowest legitimate fill, or a
/// live fill would be stolen and the memory this gate protects would be doubled.
const FILL_LEASE_SECS: i64 = 1800;

/// When the fill that currently owns the gate started, in unix seconds. `0` =
/// free. Compare-and-swap on this one value is what makes takeover safe: two
/// tasks deciding at the same instant both CAS against the value they read, and
/// only one wins.
static FILL_OWNER_SINCE: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

/// What the gate says about the lease on record. Pure, so the rules below are
/// testable without spawning a fill or sleeping for five minutes.
#[derive(Debug, PartialEq, Eq)]
enum FillGateState {
    /// Nobody holds it.
    Free,
    /// Held, but for longer than any fill can legitimately run — the owner is
    /// dead and the next fill may take it over.
    Expired { held_s: i64 },
    /// Held by a live fill; come back later.
    Busy { left_s: i64 },
}

fn fill_gate_state(owner: i64, now: i64) -> FillGateState {
    if owner <= 0 {
        return FillGateState::Free;
    }
    let held_s = now.saturating_sub(owner);
    if held_s >= FILL_LEASE_SECS {
        FillGateState::Expired { held_s }
    } else {
        FillGateState::Busy {
            left_s: FILL_LEASE_SECS - held_s,
        }
    }
}

/// The lease itself: releasing it is dropping it, so every early return, panic
/// and cancelled task frees the gate without a `release()` anyone can forget.
struct FillLease(i64);

impl Drop for FillLease {
    fn drop(&mut self) {
        // Only if the lease is still OURS: a fill that was taken over as expired
        // must never clear the gate the new owner now holds.
        let _ = FILL_OWNER_SINCE.compare_exchange(
            self.0,
            0,
            std::sync::atomic::Ordering::SeqCst,
            std::sync::atomic::Ordering::SeqCst,
        );
    }
}

/// Take the fill gate, or say who holds it and for how much longer.
fn acquire_fill_lease() -> Result<FillLease, String> {
    use std::sync::atomic::Ordering::SeqCst;
    let stamp = crate::storage::now();
    loop {
        let owner = FILL_OWNER_SINCE.load(SeqCst);
        match fill_gate_state(owner, stamp) {
            FillGateState::Free => {
                if FILL_OWNER_SINCE.compare_exchange(0, stamp, SeqCst, SeqCst).is_ok() {
                    return Ok(FillLease(stamp));
                }
            }
            FillGateState::Expired { held_s } => {
                if FILL_OWNER_SINCE
                    .compare_exchange(owner, stamp, SeqCst, SeqCst)
                    .is_ok()
                {
                    log(
                        "backfill: fill lease expired",
                        &json!({"held_s": held_s, "lease_s": FILL_LEASE_SECS}),
                    );
                    return Ok(FillLease(stamp));
                }
            }
            FillGateState::Busy { left_s } => {
                return Err(format!("its lease has {left_s}s left"));
            }
        }
        // Lost the race against another task; re-read and decide again.
    }
}

/// The whole point of the backfill: a client asked for LOSSLESS and the
/// catalogue had nothing lossless for that query.
///
/// Fire-and-forget on purpose. The search answer is served from what the
/// catalogue already holds — a user staring at a spinner is worse than a lossy
/// row — and the lossless copy lands for the next request. Measured need, from
/// the live log: `paraluman adie`, `mahika`, `DAISIES`, `Done For Me (feat.
/// Kehlani)` were all searched at `quality:"LOSSLESS"` with 1–7 rows and no
/// improvement, because nothing was listening for that.
///
/// One attempt per query per 10 minutes: a browse burst must not queue ten
/// uploads of the same song, and the ledger stops the same recording twice.
pub fn spawn_for_query(q: &str, client: grammers_client::Client) {
    let key = q.trim().to_ascii_lowercase();
    if key.is_empty() {
        return;
    }
    if crate::cache::get("backfill_q", &key).is_some() {
        return;
    }
    crate::cache::put(
        "backfill_q",
        &key,
        &json!({"at": crate::storage::now()}),
        600,
        0,
    );
    let query = q.to_string();
    let throttle_key = key.clone();
    tokio::spawn(async move {
        // The lease lives until this task ends, however it ends: `_lease` is
        // dropped on the early returns below, on a panic, and after a successful
        // fill. A fill that never returns cannot hold the gate past
        // [`FILL_LEASE_SECS`], which is the bug this replaced.
        let _lease = match acquire_fill_lease() {
            Ok(lease) => lease,
            Err(why) => {
                log(
                    "backfill: a fill is already running in this process - deferred",
                    &json!({"q": query, "reason": why}),
                );
                // The 10-minute per-query throttle was written before this task
                // started; drop it so the next browse of this query can try again
                // instead of the track staying unfilled for ten minutes.
                crate::cache::invalidate("backfill_q", &throttle_key);
                return;
            }
        };
        match backfill_for_query(&client, &query).await {
            Ok(()) => log("backfill: on-demand fill done", &json!({"q": query})),
            Err(e) => log(
                "backfill: on-demand fill skipped",
                &json!({"q": query, "reason": e}),
            ),
        }
    });
}

/// Words that mean "this is not the recording that was asked for".
///
/// MEASURED 2026-11-07 from live BitChord traffic: `q=Billie Eilish & Khalid
/// lovely` had no real result on the source at all — its top bit-exact row was
/// `DJ Moodz - Lovely - Tribute to Billie Eilish and Khalid`, next a remix, an
/// instrumental and piano covers — and we uploaded the tribute. It cost 16 MB and
/// put a cover in the archive under a title nobody asked for.
///
/// That was survivable while the trigger required `quality=lossless` from the
/// client. It is not survivable now that any song without a lossless row triggers
/// a fill: a query the source cannot actually satisfy must end in "no", not in a
/// stand-in. A row that says what it is gets believed.
const NOT_THE_RECORDING: [&str; 19] = [
    "tribute",
    "originally performed",
    "made famous",
    "in the style of",
    "cover version",
    "karaoke",
    "instrumental",
    "lullaby version",
    "piano version",
    "8-bit",
    "8 bit",
    "string quartet",
    "backing track",
    // A remix is a DIFFERENT recording, so it is not the song that was asked for
    // either. Without these, the `lovely` case above would have fallen straight
    // through the tribute to `Samy Jebari - Lovely (Samy Jebari Remix)`.
    "remix",
    "rework",
    "sped up",
    "slowed",
    "nightcore",
    "acoustic version",
];

fn is_a_stand_in(title: &str, artist: &str, album: &str) -> bool {
    let hay = format!("{title} {artist} {album}").to_lowercase();
    NOT_THE_RECORDING.iter().any(|marker| hay.contains(marker))
}

/// Strip the parts of a client query that steer the source away from the real
/// recording.
///
/// MEASURED 2026-11-07 from live traffic: `LSD Genius (feat. Sia, Diplo &
/// Labrinth)` answers with only `album: "LABRINTH, SIA, DIPLO PRESENT... LSD
/// (REMIXES)"` rows (176 s and 194 s), while `LSD Genius` answers with the 213 s
/// 24-bit original. The stand-in guard refused the remixes — correctly — and the
/// fill then gave up, so a song the source HAD was reported as missing.
///
/// `None` when there is nothing to strip.
fn reduced_query(q: &str) -> Option<String> {
    let mut out = String::with_capacity(q.len());
    let mut depth = 0usize;
    for c in q.chars() {
        match c {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    // `to_ascii_lowercase` keeps byte offsets identical, so a `find` on the
    // lowercased copy is a safe index into the original.
    let lower = out.to_ascii_lowercase();
    let mut cut = out.len();
    for marker in [" feat. ", " feat ", " ft. ", " ft ", " featuring "] {
        if let Some(i) = lower.find(marker) {
            cut = cut.min(i);
        }
    }
    let trimmed = out[..cut]
        .trim()
        .trim_end_matches(['-', ',', '&', ':'])
        .trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(q.trim()) {
        return None;
    }
    Some(trimmed.to_string())
}

/// The source's rows for one query.
async fn source_rows(q: &str) -> Result<Vec<Value>, String> {
    let body = source_get(&format!("/search?q={}&limit=5", enc(q)), 10).await?;
    Ok(body
        .get("tracks")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default())
}

/// Whether a source row advertises a lossless rendition, in either vocabulary.
///
/// The gate used to be `audioQuality.contains("bit")`, which is Qobuz's wording
/// (`"16-bit / 44.1 kHz"`). A BitChord-protocol addon answers
/// `{"format":"flac","audioQuality":"LOSSLESS"}` — no "bit" anywhere — so EVERY
/// row from one was refused and the fill never started. MEASURED 2026-11-10 on
/// `bit-chord-addon.vercel.app/c`: its search returns full-length FLAC rows and
/// its `/stream` hands back a 40 647 395-byte master with `accept-ranges: bytes`,
/// and we called it "source had no lossless row for this query".
///
/// A row is only a HINT. `may_hold_a_master()` on the signed answer and
/// `master_size()` on the actual bytes are what gate an upload, so accepting a
/// wider vocabulary here cannot let a 30-second preview through.
pub(crate) fn row_says_lossless(row: &Value) -> bool {
    const LOSSLESS_CONTAINERS: [&str; 8] =
        ["flac", "alac", "wav", "aiff", "ape", "wv", "dsf", "dff"];
    let quality = jstr(row, "audioQuality").to_ascii_lowercase();
    // `format` is a bare token (`flac`, `mp3`), so it is matched EXACTLY: a
    // substring test would read "mixtape" as APE and "wave" as WAV.
    let format = jstr(row, "format").to_ascii_lowercase();
    let format_is_lossless = LOSSLESS_CONTAINERS.contains(&format.as_str());
    // `codec` / `container` are the OTHER vocabulary. An addon built on a
    // verified-probe shape names these and sends no `format` at all, so a row
    // that is plainly a FLAC used to be filtered out before the fill ever saw it.
    // Same EXACT-token rule as `format`, because they name a bare container too.
    let codec = jstr(row, "codec").to_ascii_lowercase();
    let container = jstr(row, "container").to_ascii_lowercase();
    let named_is_lossless = LOSSLESS_CONTAINERS.contains(&codec.as_str())
        || LOSSLESS_CONTAINERS.contains(&container.as_str());
    // `quality` is free text on the Qobuz-style answer and may name the container
    // inside a sentence, so it is matched as a substring there.
    let label = jstr(row, "quality").to_ascii_lowercase();
    let label_is_lossless = LOSSLESS_CONTAINERS.iter().any(|codec| label.contains(codec));
    format_is_lossless
        || named_is_lossless
        || label_is_lossless
        || quality.contains("bit")
        || quality.contains("lossless")
        || quality.contains("hi-res")
        || quality.contains("hires")
}

/// Upload the first acceptable row, or say nothing was usable.
///
/// `Ok(false)` is not an error: it means every row was rejected (no lossless
/// claim, or a stand-in), which is the caller's cue to try a reduced query.
async fn fill_from_rows(client: &Client, q: &str, rows: &[Value]) -> Result<bool, String> {
    for row in rows {
        let id = jstr(row, "id");
        if id.is_empty() || !row_says_lossless(row) {
            continue;
        }
        let title = jstr(row, "title");
        let artist = jstr(row, "artist");
        let album = jstr(row, "album");
        if is_a_stand_in(&title, &artist, &album) {
            log(
                "backfill: skipped a stand-in (cover/tribute/karaoke)",
                &json!({"q": q, "title": title, "artist": artist}),
            );
            continue;
        }
        let duration = row.get("duration").and_then(|v| v.as_i64()).unwrap_or(0);
        // `artworkURL` is the row's own cover — that is what gives the indexed
        // row a thumbnail, so it travels with the fill instead of being guessed
        // from the title later.
        backfill(
            client,
            &id,
            &title,
            &artist,
            duration,
            "",
            &jstr(row, "artworkURL"),
        )
        .await?;
        return Ok(true);
    }
    Ok(false)
}

/// Ask the source what it has for this query and upload the first row it calls
/// bit-exact — unless that row announces itself as a stand-in.
///
/// The row's own quality fields are the gate, read through
/// [`row_says_lossless`] so both vocabularies work: a Qobuz-style bit-depth claim
/// (`"24-bit / 48 kHz"`) and a BitChord-style tier (`"LOSSLESS"`) or container
/// (`"flac"`). A row with none of them is skipped; the upload itself is still
/// gated by `may_hold_a_master()` and the real byte count.
///
/// If nothing in the answer is usable, the query is retried ONCE with the
/// parenthetical and feature credits removed: those are what push the source
/// towards remixes, and the plain title is what finds the original.
async fn backfill_for_query(client: &Client, q: &str) -> Result<(), String> {
    // PACED, before the first request to the source: see [`FILL_GAP_SECS`] for the
    // measurement. A refusal here is a normal outcome and is logged as a skip.
    if let Err(why) = fill_gate() {
        return Err(why);
    }
    if fill_from_rows(client, q, &source_rows(q).await?).await? {
        return Ok(());
    }
    let Some(reduced) = reduced_query(q) else {
        return Err("source had no lossless row for this query".to_string());
    };
    log(
        "backfill: retrying the source with a reduced query",
        &json!({"q": q, "reduced": reduced}),
    );
    if fill_from_rows(client, &reduced, &source_rows(&reduced).await?).await? {
        return Ok(());
    }
    Err(format!(
        "source had no lossless row for this query (also tried \"{reduced}\")"
    ))
}

/* ── the backfill itself ────────────────────────────────────────────────── */

/// A string with its query removed, for logs.
///
/// MEASURED 2026-11-09 from our own Heroku log: the signed CDN links carry `hmac`
/// and `etsp`, a short-lived signature, and reqwest puts the whole url into its
/// error text — so `size probe: error sending request for url (https://…&hmac=…)`
/// wrote one into the log. A signature in a log file is a credential in a log
/// file, and Heroku logs are readable by anyone with dashboard access. Everything
/// from the first `?` goes.
fn redact_query(s: &str) -> String {
    match s.split_once('?') {
        Some((head, _)) => format!("{head}?…"),
        None => s.to_string(),
    }
}

/// A source-signed URL that fetches a SLICE of the recording, not the recording.
///
/// MEASURED 2026-11-08 on `heisenberg…` — the whole of "the upload is broke, only
/// 1 MB". Every `/stream/{id}` answer, for every track including a 24-bit/192 kHz
/// edition, came back as
/// `…/file?eid=…&fmt=5&profile=raw&range=20-30&app_id=…&etsp=…&hmac=…` with
/// `"format":"mp3"` and `"quality":"Qobuz MP3 · 16-bit · 44.1 kHz"`: the host had
/// lost its full-quality credential and fell back to signing Qobuz's 30-second
/// PREVIEW. Nothing in the payload says "preview", and `quality` still contains
/// "bit", so the old gate (`quality.contains("bit")`) waved it through and a
/// 1,200,588-byte clip was uploaded and indexed as a 283-second song.
///
/// The signed URL is the tell: a full master is signed with no `range` parameter
/// at all, a preview with `range=20-30`.
fn is_preview_url(url: &str) -> bool {
    let Some((_, query)) = url.split_once('?') else {
        return false;
    };
    query.split('&').any(|pair| {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        k.eq_ignore_ascii_case("range") && !v.trim().is_empty()
    })
}

/// Formats that can hold a bit-exact master.
///
/// `m4a` is a CONTAINER, not proof — that one is settled by bitrate in
/// [`crate::music::is_lossless`]. An empty `format` is allowed through here and
/// decided by the size check below, because a source that names no format must
/// not be trusted either way on the name alone.
fn may_hold_a_master(fmt: &str) -> bool {
    matches!(
        fmt.to_ascii_lowercase().as_str(),
        "" | "flac" | "wav" | "alac" | "aiff" | "aif" | "ape" | "dsf" | "dff" | "m4a"
    )
}

/// The container a `/stream` answer names, in the vocabulary this engine reads.
///
/// Addons disagree on the field name: the Qobuz-style answer says `format`, while
/// a verified-probe answer says `codec` + `container` and no `format` at all.
/// Reading ONLY `format` made such an answer arrive as `""`, and
/// [`may_hold_a_master("")`](may_hold_a_master) is deliberately `true` — so the
/// lossy-format refusal went DARK instead of firing. The order matters: `format`
/// is the bare token this engine has always used, then `codec`, then `container`
/// (`container` is the loosest, e.g. `mp4` for an `.m4a` that holds ALAC).
fn container_of_answer(info: &Value) -> String {
    for key in ["format", "codec", "container"] {
        let value = jstr(info, key).to_ascii_lowercase();
        if !value.is_empty() {
            return value;
        }
    }
    String::new()
}

/// Which upstream an answer came from, for a source-qualified ledger key.
///
/// Ids are PER SOURCE: `qb:123` means nothing to a different addon, so a key
/// that assumes one source collides as soon as there are two.
fn source_of_answer(info: &Value) -> String {
    for key in ["source", "provider"] {
        let value = jstr(info, key).to_ascii_lowercase();
        if !value.is_empty() {
            return value.split_whitespace().collect::<Vec<_>>().join("-");
        }
    }
    "qobuz".to_string()
}

#[cfg(test)]
mod vocabulary_tests {
    use super::*;

    #[test]
    fn an_answer_that_names_only_a_codec_still_names_a_container() {
        let codec_only = json!({"codec": "flac", "container": "flac"});
        assert_eq!(container_of_answer(&codec_only), "flac");
        let qobuz_style = json!({"format": "FLAC", "quality": "Lossless"});
        assert_eq!(container_of_answer(&qobuz_style), "flac");
        assert_eq!(container_of_answer(&json!({})), "");
        // A codec must win over a looser container word.
        assert_eq!(
            container_of_answer(&json!({"codec": "alac", "container": "mp4"})),
            "alac"
        );
    }

    #[test]
    fn a_codec_only_row_is_recognised_as_lossless() {
        assert!(row_says_lossless(&json!({"codec": "flac"})));
        assert!(row_says_lossless(&json!({"container": "alac"})));
        assert!(row_says_lossless(&json!({"format": "flac"})));
        // The exact-token rule still holds: a substring must not pass.
        assert!(!row_says_lossless(&json!({"format": "mixtape"})));
        assert!(!row_says_lossless(&json!({"codec": "mp3", "container": "mp4"})));
    }

    #[test]
    fn the_ledger_key_is_source_qualified() {
        assert_eq!(source_of_answer(&json!({"provider": "Netease"})), "netease");
        assert_eq!(source_of_answer(&json!({"source": "Internet Archive"})), "internet-archive");
        // An answer that says nothing keeps the historical prefix.
        assert_eq!(source_of_answer(&json!({})), "qobuz");
    }

    #[test]
    fn a_codec_only_answer_cannot_slip_past_the_master_gate() {
        // What the two helpers together must do: name the container, then refuse
        // the lossy one. The empty string is still allowed through by design.
        let lossy = json!({"codec": "mp3", "container": "mp3"});
        assert!(!may_hold_a_master(&container_of_answer(&lossy)));
        let lossless = json!({"codec": "flac"});
        assert!(may_hold_a_master(&container_of_answer(&lossless)));
        let silent = json!({});
        assert!(may_hold_a_master(&container_of_answer(&silent)));
    }
}

/// Bits per second below which a "master" is a clip, not a recording.
///
/// No lossless master encodes under this: CD 16/44.1 is 1411 kbps and even a
/// quiet mono FLAC stays well above 600. The 2026-11-08 preview measured 34 kbps
/// over a 283-second track (1,200,588 B), so this catches a clip even if a future
/// source signs one without a `range` marker.
const MIN_MASTER_KBPS: u64 = 128;

/// Total bytes of a signed CDN URL.
///
/// MEASURED: this CDN answers `HEAD` with no `Content-Length` at all, so the
/// first version of this file refused every track with "master has no
/// content-length". A one-byte `Range` request always carries the real total in
/// `Content-Range: bytes 0-0/<total>`, which is the same header the probe used to
/// report 24,805,746 B for the Sial master.
async fn master_size(http: &reqwest::Client, url: &str) -> Result<u64, String> {
    let resp = http
        .get(url)
        .header("Range", "bytes=0-0")
        .timeout(std::time::Duration::from_secs(20))
        .send()
        .await
        // SECURITY: a signature never reaches the log. reqwest embeds the WHOLE
        // url in its error text, and the source's CDN links carry `hmac` + `etsp`
        // — so this line once wrote a live signature into the Heroku log
        // (`size probe: error sending request for url (https://…&hmac=…)`).
        .map_err(|e| format!("size probe: {}", redact_query(&e.to_string())))?;
    // A non-2xx is NOT a size. MEASURED 2026-11-10 against
    // `tracks.monochrome.st`: one probe read the total from
    // `content-range: bytes 0-0/53975130`, and a later one was answered with a
    // 44-byte error body, which this function then reported as "the master is 44
    // bytes" and the fill refused the track as a clip. A rate-limit page read as a
    // file size is worse than an error: it turns a retryable refusal into a
    // confident "this is not the track".
    if !resp.status().is_success() {
        // 429 here is the source's own limiter — the status that actually showed up
        // on the deployed fleet, ten skips in one window. Cooling the source off is
        // the only useful answer to it; the gate below then keeps us quiet.
        if resp.status().as_u16() == 429 {
            source_cool_off(FILL_COOL_OFF_SECS);
        }
        return Err(format!("size probe: HTTP {}", resp.status().as_u16()));
    }
    if let Some(range) = resp.headers().get("content-range") {
        if let Ok(text) = range.to_str() {
            if let Some(total) = text.rsplit('/').next() {
                if let Ok(n) = total.trim().parse::<u64>() {
                    if n > 0 {
                        return Ok(n);
                    }
                }
            }
        }
    }
    // No `Content-Range` (a server that ignored the Range header) — then a plain
    // length is the whole file and is trustworthy.
    match resp.content_length() {
        Some(n) if n > 1 => Ok(n),
        _ => Err("master size unknown: no content-range and no content-length".to_string()),
    }
}

/// Fetch `track_id` from the hi-res source and send it to a relay account,
/// returning the handle that plays it back.
///
/// Fails (never panics) on: an already-known track, a source that cannot name a
/// stream, a source that will not say its size, no usable relay account, and any
/// send error. Every failure is a log line plus a message — a backfill runs on a
/// user's request, so "no" has to be explainable.
pub async fn backfill(
    client: &Client,
    track_id: &str,
    title: &str,
    artist: &str,
    duration: i64,
    relay: &str,
    artwork: &str,
) -> Result<Backfilled, String> {
    let key = format!("qobuz:{track_id}");

    // RESERVE THE RECORDING BEFORE THE UPLOAD, not after it.
    //
    // MEASURED 2026-11-07 from the live log: two different queries raced into the
    // same recording and each uploaded it — `Zack Tabudlo - Pano` twice
    // (messages 1629 and 1630) and `Mariah Deborah - Na Para Bang` (85 MB) twice
    // (1632 and 1634), ~113 MB duplicated in under a minute. The per-query guard
    // cannot see that, because `pano` and `pano zack tabudlo` are different keys,
    // and the ledger was only written once the upload had finished, so both tasks
    // read "not backfilled yet".
    //
    // A reservation with a 5-minute lease closes that window and self-clears: a
    // crashed or failed attempt does not block the recording forever, and a large
    // upload in progress is not restarted by a second query's trigger.
    // TWO keys, because the same recording has two identities:
    //
    //   `qobuz:<track_id>` — the source's id for this edition. Two queries that
    //   both resolve to it are already covered by this one.
    //
    //   `title:<title>|<artist>` — the SONG. The same recording reaches us under
    //   a different Qobuz id (a remaster, a re-issue) and under a different
    //   query, and the id key cannot see that. This is the "same title" gate.
    //
    // Both are reserved in one decision — see `reserve()` for why that has to be
    // under a lock and not two separate reads.
    let mut keys = vec![key.clone()];
    if let Some(tk) = title_key(title, artist) {
        keys.push(tk);
    }
    let stamp = crate::storage::now();
    reserve(&keys, stamp)?;

    // A FAILED ATTEMPT MUST COST NOTHING.
    //
    // MEASURED 2026-11-07 from `storage\_server_lease.out`: the source answers
    // `503` with `retry-after: 15` when it throttles, and the first version of
    // this file wrote the reservation and then returned on every error path —
    // so a track the source paused for 15 seconds stayed unfillable for the
    // whole 300-second lease. The retry here is the next client request, which
    // is free, so release the reservation and let it happen.
    let result = backfill_body(client, track_id, title, artist, duration, relay, artwork).await;
    match &result {
        // Carry the completed entry onto the title key as well, so the NEXT
        // edition of the same song (a different source id) is refused by the
        // title gate instead of being uploaded a second time.
        Ok(_) => {
            if let Some(done) = ledger_get(&key) {
                for k in keys.iter().filter(|k| **k != key) {
                    ledger_put(k, done.clone());
                }
            }
        }
        Err(_) => {
            for k in &keys {
                release_lease(k, stamp);
            }
        }
    }
    result
}

/// The same recording reaches us under more than one source id, so the song —
/// not the edition id — has to be one of the gate keys. Alphanumerics only, so
/// case, punctuation, accents-as-separators and stray spacing all fold together.
fn title_key(title: &str, artist: &str) -> Option<String> {
    let slug = |s: &str| -> String {
        s.chars()
            .filter(|c| c.is_alphanumeric())
            .flat_map(|c| c.to_lowercase())
            .collect()
    };
    let t = slug(title);
    if t.is_empty() {
        return None;
    }
    Some(format!("title:{}|{}", t, slug(artist)))
}

/// One process owns the ledger file, so a mutex around the decision is what makes
/// the gate real.
///
/// MEASURED 2026-11-07: the old code read the ledger, decided, and *then* wrote
/// the reservation — three steps. Two queries arriving in the same millisecond
/// both read "not backfilled yet" and both uploaded, which is exactly how
/// `Zack Tabudlo - Pano` (27 MB, messages 1629/1630) and
/// `Mariah Deborah - Na Para Bang` (85 MB, 1632/1634) each went out twice.
static RESERVE: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Reserve every key at once, or refuse with the reason.
///
/// A refusal names the key that blocked it: "already backfilled" from the id key
/// and from the title key mean different things to whoever reads the log.
fn reserve(keys: &[String], stamp: i64) -> Result<(), String> {
    let _guard = RESERVE.lock().unwrap_or_else(|e| e.into_inner());
    let now = crate::storage::now();
    for key in keys {
        let Some(prev) = ledger_get(key) else {
            continue;
        };
        let in_flight = prev.get("status").and_then(|v| v.as_str()) == Some("in_flight");
        let at = prev.get("at").and_then(|v| v.as_i64()).unwrap_or(0);
        let lease_live = at > 0 && now - at < 300;
        if !in_flight {
            // A catalogue hit is the only non-upload terminal state, and it
            // already carries the short code the origin minted.
            let code = prev.get("short_code").and_then(|v| v.as_str()).unwrap_or("");
            if !code.is_empty() {
                return Err(format!("already in the catalogue: short_code={code}"));
            }
            return Err(format!(
                "already backfilled: message_id={} size={} ({key})",
                prev.get("message_id").and_then(|v| v.as_i64()).unwrap_or(0),
                prev.get("size").and_then(|v| v.as_u64()).unwrap_or(0)
            ));
        }
        if lease_live {
            return Err(format!("backfill already running for this recording ({key})"));
        }
        // Stale lease: take it over rather than leaving the track unfillable.
    }
    let entry = json!({"status": "in_flight", "at": stamp});
    for key in keys {
        ledger_put(key, entry.clone());
    }
    Ok(())
}

/// Delete the reservation WE made, and only if it is still ours.
///
/// The `stamp` check is the whole point: a stale lease can be taken over by
/// another task, and the entry that task completes must never be deleted by the
/// attempt that lost the race.
fn release_lease(key: &str, stamp: i64) {
    let ours = ledger_get(key)
        .map(|cur| {
            cur.get("status").and_then(|v| v.as_str()) == Some("in_flight")
                && cur.get("at").and_then(|v| v.as_i64()) == Some(stamp)
        })
        .unwrap_or(false);
    if ours {
        ledger_del(key);
    }
}

/// The largest thumbnail Telegram accepts for a document, in bytes.
///
/// Its documented ceiling is 200 KB for a document thumb; anything bigger is
/// rejected outright, which would fail the SEND, not just the picture.
const THUMB_MAX_BYTES: usize = 200_000;

/// A cover at thumb size: `…_600.jpg` → `…_230.jpg`.
///
/// MEASURED on the source's own artwork URLs
/// (`static.qobuz.com/images/covers/jb/ev/eicwm9xtqevjb_600.jpg`): the size is part
/// of the file name, so the same cover exists at `_230` (230x230, well under the
/// byte cap and inside Telegram's recommended 320 px). No image crate, no
/// resampling — the CDN already has the size we need.
fn shrink_artwork(url: &str) -> String {
    let Some((head, tail)) = url.rsplit_once('_') else {
        return url.to_string();
    };
    let size = tail
        .trim_end_matches(".jpeg")
        .trim_end_matches(".jpg")
        .trim_end_matches(".png");
    if size.is_empty() || !size.chars().all(|c| c.is_ascii_digit()) {
        return url.to_string();
    }
    format!("{head}_230.jpg")
}

/// The cover image for a row, as bytes Telegram will accept as a thumb.
///
/// Tries the `_230` variant first and the original URL only if that fails, and
/// only returns something that is really a JPEG under [`THUMB_MAX_BYTES`] —
/// Telegram rejects a non-JPEG or oversized thumb, and a rejection there fails the
/// whole send. `None` means "send it without artwork", never "fail the fill".
async fn fetch_thumb(http: &reqwest::Client, artwork: &str) -> Option<Vec<u8>> {
    let artwork = artwork.trim();
    if !(artwork.starts_with("http://") || artwork.starts_with("https://")) {
        return None;
    }
    let small = shrink_artwork(artwork);
    let mut urls = vec![small];
    if urls[0] != artwork {
        urls.push(artwork.to_string());
    }
    for url in urls {
        let Ok(resp) = http
            .get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
        else {
            continue;
        };
        if !resp.status().is_success() {
            continue;
        }
        let Ok(bytes) = resp.bytes().await else {
            continue;
        };
        if bytes.len() > THUMB_MAX_BYTES || bytes.len() < 100 {
            continue;
        }
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(bytes.to_vec());
        }
    }
    None
}

async fn backfill_body(
    client: &Client,
    track_id: &str,
    title: &str,
    artist: &str,
    duration: i64,
    relay: &str,
    artwork: &str,
) -> Result<Backfilled, String> {
    // 1. the source's signed URL — the first base that answers
    let info = source_get(&format!("/stream/{track_id}"), 8).await?;
    // The ledger key is SOURCE-QUALIFIED, from the answer itself. It used to be
    // the hardcoded `qobuz:{track_id}`, which is one source's id space: with a
    // second source the same numeric id lands in the same slot, so a lease or a
    // completed upload belonging to one provider blocks the other.
    let key = format!("{}:{track_id}", source_of_answer(&info));
    let media_url = jstr(&info, "url");
    if media_url.is_empty() {
        return Err(format!("source /stream/{track_id} carried no url"));
    }

    // 1b. NOTHING THAT IS NOT THE RECORDING MAY BE UPLOADED.
    //
    // Both checks are free and happen before a byte is fetched, so a degraded
    // source costs one API call per attempt instead of a garbage 1 MB upload.
    // Every refusal below returns `Err`, which releases the lease, so the track
    // is retried the moment the source can serve a real master again.
    if is_preview_url(&media_url) {
        log(
            "backfill: refused a preview slice",
            &json!({"track_id": track_id, "url": redact_query(&media_url)}),
        );
        return Err(format!(
            "source /stream/{track_id} signed a preview slice (range=), not the master"
        ));
    }
    let signed_format = container_of_answer(&info);
    if !may_hold_a_master(&signed_format) {
        log(
            "backfill: refused a lossy format",
            &json!({
                "track_id": track_id,
                "format": signed_format,
                "quality": jstr(&info, "quality"),
            }),
        );
        return Err(format!(
            "source /stream/{track_id} is {signed_format}, which cannot hold a master"
        ));
    }

    // 2. how big is it — needed BOTH to plan the upload and to key the dedupe
    // CONNECT timeout only. A total `timeout()` here kills the upload mid-stream:
    // a 25 MB body piped into `upload_stream` takes minutes, and reqwest aborts
    // the whole request — measured as
    // `upload_stream …: error decoding response body` 32 s in.
    let http = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    let size = master_size(&http, &media_url).await?;
    // The path is silent from here to the finished send — a 50 MB fetch, a
    // catalogue lookup and a Telegram upload — so a stall looks identical to a
    // no-op in the log. MEASURED 2026-11-10: a fill sat for six minutes with
    // nothing between `telegram session resumed` and the end, and there was no way
    // to tell which step it was in.
    log(
        "backfill: source master sized",
        &json!({
            "track_id": track_id,
            "bytes": size,
            "kbps": if duration > 0 { crate::music::kbps(size, duration) } else { 0 },
        }),
    );

    // 2b. A master is never this thin. Backstop for a source that signs a slice
    //     without saying `range=` and without naming a lossy format.
    if duration > 0 && size > 0 {
        let kbps = crate::music::kbps(size, duration);
        if kbps < MIN_MASTER_KBPS {
            log(
                "backfill: refused a clip",
                &json!({
                    "track_id": track_id,
                    "bytes": size,
                    "duration": duration,
                    "kbps": kbps,
                }),
            );
            return Err(format!(
                "source master is {kbps} kbps over {duration}s — a clip, not the track"
            ));
        }
    }

    // 3. duplicate gates
    match already_indexed(title, artist, size).await {
        Some(code) => {
            // Terminal, not transient: this exact byte size is in the
            // catalogue, so no future request should upload it again. Written
            // as a COMPLETED entry, so `release_lease` leaves it in place.
            ledger_put(
                &key,
                json!({
                    "status": "catalogue",
                    "short_code": code,
                    "size": size,
                    "at": crate::storage::now(),
                }),
            );
            return Err(format!(
                "already in the catalogue: short_code={code} size={size}"
            ));
        }
        None => {}
    }

    // 4. relay peer
    let (peer, username) = resolve_relay(client, relay).await?;

    // 4b. the cover, so the indexed row HAS a thumbnail.
    //
    // A fill was landing with no artwork at all: `/thumb/{code}` reads the
    // document's own thumbs back out of Telegram, and a send that carried none
    // has nothing to read, so the track showed a blank square. Best effort — a
    // missing cover must never cost the fill.
    let thumb = fetch_thumb(&http, artwork).await;
    if thumb.is_none() && !artwork.trim().is_empty() {
        log(
            "backfill: no usable cover",
            &json!({"artwork": redact_query(artwork)}),
        );
    }

    // Say where the silence starts. Everything above is bounded by a timeout;
    // this step is a 50 MB fetch piped into a Telegram upload and logs nothing
    // until it finishes, so without this line a stall and a no-op look the same.
    log(
        "backfill: streaming the master into the send",
        &json!({"track_id": track_id, "bytes": size, "relay": username}),
    );

    // 5. stream the body into the upload, in BOUNDED RANGED CHUNKS.
    //
    // ONE LONG GET DOES NOT SURVIVE THE SEND. MEASURED 2026-11-10 on a 53 975 130
    // byte FLAC through `tracks.monochrome.st`:
    //
    //   backfill: streaming the master into the send {"bytes":53975130}
    //   on-demand fill skipped {"reason":"upload+send: upload_stream …:
    //                            error decoding response body"}
    //
    // That error is reqwest decoding the SOURCE body, not the upload: the send
    // consumes the reader at Telegram's pace, so a single GET holds a mostly idle
    // connection for the whole upload and the CDN closes it. Asking for 8 MB at a
    // time keeps every connection short — one chunk read into memory, handed to the
    // reader, connection done — and the total is already known from `master_size`,
    // with `Accept-Ranges: bytes` confirmed in the same answer.
    //
    // Still no temp file: the chunks feed the same `StreamReader` the upload reads.
    const CHUNK: u64 = 8 * 1024 * 1024;
    let chunk_http = http.clone();
    let chunk_url = media_url.clone();
    // `Box::pin` because `unfold` is not `Unpin` and `StreamReader` requires it.
    let chunks = Box::pin(futures_util::stream::unfold(0u64, move |pos| {
        let http = chunk_http.clone();
        let url = chunk_url.clone();
        async move {
            if pos >= size {
                return None;
            }
            let end = (pos + CHUNK - 1).min(size - 1);
            let resp = http
                .get(&url)
                .header("Range", format!("bytes={pos}-{end}"))
                .send()
                .await;
            match resp {
                Err(e) => {
                    // SECURITY: a signature in the URL must never reach the log.
                    let why = redact_query(&e.to_string());
                    return Some((Err(std::io::Error::other(format!("chunk {pos}: {why}"))), pos));
                }
                Ok(r) if r.status().is_success() => match r.bytes().await {
                    Err(e) => {
                        let why = redact_query(&e.to_string());
                        Some((Err(std::io::Error::other(format!("chunk {pos}: {why}"))), pos))
                    }
                    Ok(b) => {
                        let n = b.len() as u64;
                        if n == 0 {
                            return None;
                        }
                        Some((Ok(b), pos + n))
                    }
                },
                Ok(r) => Some((
                    Err(std::io::Error::other(format!(
                        "chunk {pos}: HTTP {}",
                        r.status().as_u16()
                    ))),
                    pos,
                )),
            }
        }
    }));
    let mut reader = StreamReader::new(chunks);

    let ext = jstr(&info, "format");
    let ext = if ext.is_empty() { "flac".to_string() } else { ext };
    let mime = jstr(&info, "mimeType");
    let mime = if mime.is_empty() {
        "audio/flac".to_string()
    } else {
        mime
    };
    let file_name = safe_name(artist, title, &ext);
    // Read BEFORE the send: `thumb` moves into it, and whether the file carries
    // artwork is the only way to tell a covered fill from a blank one in the log.
    let has_thumb = thumb.as_ref().is_some_and(|b| !b.is_empty());

    let sent = crate::telegram::upload_and_send_audio(
        client,
        peer,
        &mut reader,
        size,
        &file_name,
        &mime,
        title,
        artist,
        duration,
        thumb,
        REQUEST_CAPTION,
    )
    .await
    .map_err(|e| format!("upload+send: {e}"))?;

    // Read the size ONCE, before the target is moved into the return value:
    // `DocTarget` is not `Copy`, and three reads of `sent.target.size` around a
    // move is exactly how a working build becomes a borrow error later.
    let sent_size = sent.target.size;

    // NO short_code here, deliberately. The short code is the API backend's to
    // mint: its webhook writes the row, generates the code and owns the identity.
    // This side only has to be able to PLAY the file it just sent, and that needs
    // the send result — `dc_id` + the `(id, access_hash, file_reference)` triple —
    // never a short code. Ledgered under the SOURCE id (`qobuz:<id>`) so a
    // `pmq_` request can be answered before the backend has indexed anything.
    ledger_put(
        &key,
        json!({
            "message_id": sent.message_id,
            "key": sent.key,
            "size": sent_size,
            "dc_id": sent.target.dc_id,
            "doc_id": sent.target.doc_id(),
            "access_hash": sent.target.access_hash(),
            "file_ref": crate::fileid::base64url_encode(&sent.target.file_ref()),
            "file_name": file_name,
            "relay": username,
        }),
    );
    log(
        "backfill done",
        &json!({
            "track_id": track_id,
            "file_name": file_name,
            "bytes": sent_size,
            "message_id": sent.message_id,
            "relay": username,
            "thumb": has_thumb,
        }),
    );

    Ok(Backfilled {
        target: sent.target,
        message_id: sent.message_id,
        size: sent_size,
        file_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The exact rows the source returned for `Billie Eilish & Khalid lovely` on
    /// 2026-11-07 — the query the real song simply does not exist for. Each one
    /// was, or would have been, uploaded as "the song" before this guard.
    #[test]
    fn a_stand_in_is_never_mistaken_for_the_song() {
        assert!(is_a_stand_in(
            "Lovely - Tribute to Billie Eilish and Khalid",
            "DJ Moodz",
            "Lovely - Tribute to Billie Eilish and Khalid",
        ));
        assert!(is_a_stand_in(
            "Billie Eilish, Khalid - Lovely (Samy Jebari Remix)",
            "Samy Jebari",
            "Billie Eilish, Khalid - Lovely (Samy Jebari Remix)",
        ));
        assert!(is_a_stand_in(
            "Lovely (Originally Performed by Billie Eilish & Khalid)",
            "Sing2Piano",
            "Lovely",
        ));
        assert!(is_a_stand_in(
            "Lovely - Tribute to Billie Eilish and Khalid (Instrumental Version)",
            "DJ Moodz",
            "Lovely - Tribute to Billie Eilish and Khalid",
        ));
    }

    /// The guard must never eat a real recording — these are the songs the live
    /// traffic actually filled, and several carry words a sloppy matcher catches.
    #[test]
    fn a_real_recording_is_never_called_a_stand_in() {
        assert!(!is_a_stand_in("Namumula", "Maki", "KOLORCOASTER"));
        assert!(!is_a_stand_in("Tingin", "Cup of Joe", "Silakbo"));
        assert!(!is_a_stand_in("Genius", "LSD", "Labrinth, Sia & Diplo"));
        assert!(!is_a_stand_in(
            "when the party's over",
            "Billie Eilish",
            "WHEN WE ALL FALL ASLEEP, WHERE DO WE GO?",
        ));
        assert!(!is_a_stand_in("Libu-Libong Buwan (Uuwian)", "Kyle Raphael", ""));
    }

    /// Two editions of one song must fold to one title key, and a genuinely
    /// different title must not.
    #[test]
    fn the_title_key_folds_one_song_and_not_two() {
        assert_eq!(
            title_key("Namumula", "Maki"),
            title_key("NAMUMULA!", "  maki ")
        );
        assert_ne!(title_key("Namumula", "Maki"), title_key("Namumula", "HeartSilence"));
        assert_eq!(title_key("", "Maki"), None);
    }

    /// The retry query. Measured case: the parenthetical is what sent the source
    /// to the `(REMIXES)` album and lost the 24-bit original.
    #[test]
    fn a_parenthetical_and_a_feature_credit_are_stripped_for_the_retry() {
        assert_eq!(
            reduced_query("LSD Genius (feat. Sia, Diplo & Labrinth)").as_deref(),
            Some("LSD Genius")
        );
        assert_eq!(
            reduced_query("Song Title feat. Somebody").as_deref(),
            Some("Song Title")
        );
        assert_eq!(
            reduced_query("Tingin [Remastered]").as_deref(),
            Some("Tingin")
        );
    }

    /// Nothing to strip means no second source call.
    #[test]
    fn a_plain_query_is_not_retried() {
        assert_eq!(reduced_query("Namumula"), None);
        assert_eq!(reduced_query("namumula maki"), None);
        assert_eq!(reduced_query("(live)"), None);
        assert_eq!(reduced_query("   "), None);
    }

    /// The measured failure, 2026-11-08: an apostrophe in the title must be
    /// DELETED from the file name. Left in, the origin spaces it out, the stored
    /// tokens become `what i ve done`, and the client's `what ive done` matches
    /// nothing — seven copies of one 45.9 MB master, none findable.
    #[test]
    fn an_apostrophe_is_deleted_from_the_file_name_not_spaced() {
        assert_eq!(
            safe_name("Linkin Park", "What I've Done", "flac"),
            "Linkin Park - What Ive Done.flac"
        );
        assert_eq!(
            safe_name("Guns N' Roses", "Sweet Child O' Mine", "flac"),
            "Guns N Roses - Sweet Child O Mine.flac"
        );
        // A phone keyboard sends the curly form.
        assert_eq!(
            safe_name("Aliyah\u{2019}s Interlude", "It\u{2019}s", "flac"),
            "Aliyahs Interlude - Its.flac"
        );
        // A hyphenated title keeps its hyphen; only apostrophes go.
        assert_eq!(
            safe_name("Kyle Raphael", "Libu-Libong Buwan (Uuwian)", "flac"),
            "Kyle Raphael - Libu-Libong Buwan (Uuwian).flac"
        );
    }

    /// The duplicate gate has to try both spellings, because the catalogue holds
    /// rows from before the fix (`What I ve Done`) and after it (`What Ive Done`).
    #[test]
    fn the_duplicate_gate_tries_both_spellings() {
        let q = search_spellings("What I've Done", "Linkin Park");
        assert!(q.contains(&"What Ive Done Linkin Park".to_string()), "got {q:?}");
        assert!(q.contains(&"What I ve Done Linkin Park".to_string()), "got {q:?}");
        // No apostrophe means one query, not two.
        assert_eq!(search_spellings("Namumula", "Maki"), vec!["Namumula Maki"]);
        assert!(search_spellings("", "").is_empty());
    }

    /// The 2026-11-08 outage, byte for byte: the source signed a 10-second slice
    /// (`range=20-30`) of a 283-second track for EVERY id, and only this marker —
    /// absent from a real master's URL — separates the preview from the master.
    #[test]
    fn a_signed_slice_is_a_preview_and_a_signed_master_is_not() {
        // verbatim from `/a/stream/368761249`, a 24-bit/192 kHz edition
        // (the signature is replaced: a live `hmac` has no business in a repo)
        let preview = "https://streaming-qobuz-std.akamaized.net/file?eid=368761249&fmt=5&profile=raw&range=20-30&app_id=312369995&etsp=1&hmac=TEST";
        // the same host's master shape: fmt=7 (24-bit FLAC), no slice
        let master = "https://streaming-qobuz-std.akamaized.net/file?eid=368761249&fmt=7&profile=raw&app_id=312369995&etsp=1&hmac=TEST";
        assert!(is_preview_url(preview));
        assert!(!is_preview_url(master));
        assert!(!is_preview_url("https://example.test/file?orange=3"));
        assert!(!is_preview_url("https://example.test/file?range="));
        assert!(!is_preview_url("https://example.test/file"));
    }

    /// A lossy name is refused before a byte is fetched; a container that can hold
    /// a master is let through and settled by bitrate.
    #[test]
    /// `PM_HIRES_BASE` is a LIST, and unset or malformed means DISABLED.
    #[test]
    fn the_source_list_is_parsed_and_empty_means_disabled() {
        assert!(parse_bases("").is_empty(), "unset/blank disables backfill");
        assert!(parse_bases("   ").is_empty());
        assert!(
            parse_bases("example.com/c").is_empty(),
            "a bare host is skipped, not probed"
        );

        assert_eq!(
            parse_bases("https://h/c/"),
            vec!["https://h/c".to_string()],
            "trailing slash trimmed"
        );

        assert_eq!(
            parse_bases("https://a/x, https://b/y;https://a/x\nhttps://c/z"),
            vec![
                "https://a/x".to_string(),
                "https://b/y".to_string(),
                "https://c/z".to_string()
            ],
            "comma, semicolon and newline split, and duplicates collapse"
        );
    }

    /// The source row gate speaks both vocabularies and still refuses a lossy row.
    ///
    /// MEASURED 2026-11-10: `bit-chord-addon.vercel.app/c` answers
    /// `{"format":"flac","audioQuality":"LOSSLESS"}`, so the old `contains("bit")`
    /// gate rejected every row it had and the fill reported "no lossless row".
    #[test]
    fn a_row_without_a_bit_depth_can_still_be_lossless() {
        let bit_chord = serde_json::json!({"format": "flac", "audioQuality": "LOSSLESS"});
        assert!(row_says_lossless(&bit_chord), "a flac LOSSLESS row");

        let qobuz = serde_json::json!({"format": "flac", "quality": "Qobuz FLAC · 24-bit · 96 kHz"});
        assert!(row_says_lossless(&qobuz), "the Qobuz wording still works");

        let mp3 = serde_json::json!({"format": "mp3", "audioQuality": "HIGH"});
        assert!(!row_says_lossless(&mp3), "a lossy row is never waved through");

        let mixtape = serde_json::json!({"format": "mixtape"});
        assert!(!row_says_lossless(&mixtape), "\"mixtape\" is not APE");

        assert!(!row_says_lossless(&serde_json::json!({})), "an empty row");
    }

    fn a_lossy_format_is_never_waved_through() {
        for ok in ["flac", "FLAC", "wav", "alac", "m4a", "aiff", "ape", ""] {
            assert!(may_hold_a_master(ok), "{ok} should pass");
        }
        for no in ["mp3", "aac", "opus", "ogg", "MP3", "webm"] {
            assert!(!may_hold_a_master(no), "{no} should be refused");
        }
    }

    /// The 1,200,588-byte clip the preview bug uploaded for a 283-second song is
    /// 34 kbps; the real master the same id produced earlier that day is not.
    #[test]
    fn a_one_megabyte_clip_is_not_a_master() {
        assert!(crate::music::kbps(1_200_588, 283) < MIN_MASTER_KBPS);
        assert!(crate::music::kbps(32_059_583, 283) > MIN_MASTER_KBPS);
        // An unknown duration cannot be judged, and must not be refused.
        assert_eq!(crate::music::kbps(1_200_588, 0), 0);
    }

    /// The cover is asked for at thumb size, because Telegram rejects a document
    /// thumb over 200 KB — and a rejected thumb fails the whole send.
    #[test]
    fn a_cover_is_asked_for_at_thumb_size() {
        assert_eq!(
            shrink_artwork("https://static.qobuz.com/images/covers/jb/ev/eicwm9xtqevjb_600.jpg"),
            "https://static.qobuz.com/images/covers/jb/ev/eicwm9xtqevjb_230.jpg"
        );
        assert_eq!(
            shrink_artwork("https://static.qobuz.com/images/covers/jb/ev/eicwm9xtqevjb_1400.jpg"),
            "https://static.qobuz.com/images/covers/jb/ev/eicwm9xtqevjb_230.jpg"
        );
        // No numeric size suffix: left alone, and `fetch_thumb` then tries the
        // original unchanged.
        assert_eq!(
            shrink_artwork("https://cdn.test/covers/album.jpg"),
            "https://cdn.test/covers/album.jpg"
        );
        assert_eq!(
            shrink_artwork("https://cdn.test/covers/abc_def.jpg"),
            "https://cdn.test/covers/abc_def.jpg"
        );
        assert_eq!(shrink_artwork(""), "");
    }

    /// A signature must never reach a log line, including through a reqwest error
    /// string, which is how `hmac` got into the Heroku log on 2026-11-09.
    #[test]
    fn a_signed_url_is_never_written_to_the_log() {
        assert_eq!(
            redact_query("https://cdn.test/file?eid=1&fmt=7&hmac=SECRET"),
            "https://cdn.test/file?…"
        );
        assert!(!redact_query("https://cdn.test/file?hmac=SECRET").contains("SECRET"));
        // The exact shape reqwest produces in an error's Display.
        let err = "error sending request for url (https://cdn.test/file?hmac=SECRET)";
        assert!(!redact_query(err).contains("SECRET"));
        // A url with no query is left whole — the path is what makes it useful.
        assert_eq!(redact_query("https://cdn.test/file"), "https://cdn.test/file");
        assert_eq!(redact_query("no url here"), "no url here");
    }

    /// THE 2026-11-10 BUG, as arithmetic: a fill that hung is not a fill that is
    /// running. One hung `upload_stream` held the process-wide gate for over an
    /// hour, so every later search answered `a fill is already running in this
    /// process - deferred` while nothing was streaming.
    #[test]
    fn a_fill_lease_expires_so_a_hung_owner_cannot_hold_the_gate() {
        let start = 1_700_000_000i64;
        assert_eq!(fill_gate_state(0, start), FillGateState::Free);
        assert_eq!(fill_gate_state(-5, start), FillGateState::Free);

        // A live fill is protected: a second one must not start on top of it.
        assert_eq!(
            fill_gate_state(start, start + 60),
            FillGateState::Busy {
                left_s: FILL_LEASE_SECS - 60
            }
        );
        // The measured hour of silence is past any legitimate fill.
        assert_eq!(
            fill_gate_state(start, start + 3600),
            FillGateState::Expired { held_s: 3600 }
        );
        // Exactly at the TTL it is dead, not "one second more of waiting".
        assert_eq!(
            fill_gate_state(start, start + FILL_LEASE_SECS),
            FillGateState::Expired {
                held_s: FILL_LEASE_SECS
            }
        );
        // ... one second before it, the owner still counts as alive.
        assert_eq!(
            fill_gate_state(start, start + FILL_LEASE_SECS - 1),
            FillGateState::Busy { left_s: 1 }
        );
    }

    /// Two consecutive fills in ONE process must both start: the second is
    /// refused while the first holds the lease, and free the instant it ends —
    /// no `a fill is already running` left behind (the fleet's symptom). A dead
    /// owner is taken over, and ITS drop must not clear the new owner's gate.
    ///
    /// One test, not two: `FILL_OWNER_SINCE` is process-wide and cargo runs tests
    /// on threads, so two tests mutating it would race.
    #[test]
    fn the_gate_is_free_again_the_moment_its_owner_ends() {
        use std::sync::atomic::Ordering::SeqCst;

        let first = acquire_fill_lease().expect("a fresh process has the gate free");
        assert!(
            acquire_fill_lease().is_err(),
            "a second fill must not run alongside the first"
        );
        drop(first);
        let second = acquire_fill_lease().expect("the gate must be free after the first fill ends");
        drop(second);
        assert!(acquire_fill_lease().is_ok(), "and free again after that one");

        // A fill that hung two hours ago: its lease is dead, so the next fill runs.
        let dead = crate::storage::now() - 7200;
        FILL_OWNER_SINCE.store(dead, SeqCst);
        let fresh = acquire_fill_lease().expect("an expired lease is taken over");
        // …and when that hung future eventually ends, it must not free the gate
        // the new owner now holds.
        drop(FillLease(dead));
        assert!(
            acquire_fill_lease().is_err(),
            "the dead owner's release must not clear a live lease"
        );
        drop(fresh);
        assert!(acquire_fill_lease().is_ok(), "the live owner's own drop does free it");
        // Leave the static clean for any later test in this binary.
        FILL_OWNER_SINCE.store(0, SeqCst);
    }
}
