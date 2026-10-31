//! Port of the `telegram_session` group, backend.php:554-1075 and 6324-6459:
//! `fd_get_bot_id`, `fd_clear_bot_id`, `fd_get_bot_session_path`,
//! `fd_has_local_session`, `fd_load_session_meta`, `fd_save_session_meta`,
//! `fd_clear_session_meta`, `fd_get_bot_pool`, `fd_save_bot_pool`,
//! `fd_add_pool_bot`, `fd_remove_pool_bot`, `fd_pick_pool_bot`,
//! `fd_is_guest_provision_in_progress`, `fd_auto_provision_guest`,
//! `fd_find_orphan_session_bot_id`, `fd_clear_session_directory`,
//! `fd_clear_session`, `fd_measure_clock_offset`, `fd_clock_skew_message`,
//! `fd_friendly_login_error`.
//!
//! The bot is never copied from the PHP install: provisioning asks WordPress
//! for a guest token (`GET {WP_API_BASE}/provision-session` -> `{ok, bot_token,
//! api_secret}`) and this port logs that bot in with grammers, persisting its
//! own session under `storage/sessions/<bot_id>/session.rust.json`.
//!
//! Locking: the PHP uses `flock()`. Here the same semantics (exclusive,
//! non-blocking, 30s wait, 300s staleness reclaim, heartbeat file) are
//! reproduced with an `O_EXCL` lock file and a heartbeat stamp.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rand::RngCore;
use serde_json::Value;

use crate::config::wp_api_base;
use crate::credentials::{save_api_secret, get_api_secret};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use crate::httpc::{http_json, http_json_h};
use crate::log::log;
use crate::storage::{now, storage_path};

/* ── identity ────────────────────────────────────────────────────────────── */

pub fn session_meta_path() -> PathBuf {
    storage_path("session_meta.json")
}

pub fn bot_pool_path() -> PathBuf {
    storage_path("bot_pool.json")
}

pub fn bot_id_cache_path() -> PathBuf {
    storage_path("bot_id.txt")
}

pub fn load_session_meta() -> Value {
    fs::read_to_string(session_meta_path())
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}))
}

pub fn save_session_meta(bot_id: &str, bot_username: &str, bot_name: &str) {
    let body = serde_json::json!({
        "bot_id": bot_id,
        "bot_username": bot_username,
        "bot_name": bot_name,
    });
    let _ = fs::write(session_meta_path(), body.to_string());
    add_pool_bot(serde_json::json!({
        "bot_id": bot_id,
        "bot_username": bot_username,
        "bot_name": bot_name,
        "is_active": true,
        "status": "online",
        "updated_at": now(),
    }));
    clear_bot_id();
}

pub fn clear_session_meta() {
    let _ = fs::remove_file(session_meta_path());
}

/// Bot tokens supplied by the environment, in order.
///
/// `BOT_TOKENS` (plural) is a comma / semicolon / whitespace separated list;
/// `TG_BOT_TOKENS` is an alias, and the legacy single `BOT_TOKEN` /
/// `TG_BOT_TOKEN` counts as a one-element list.
///
/// Why this exists: the bot pool is per-process DISK state (`bot_pool.json`
/// plus `sessions/<bot_id>/session.rust.json`). On a PaaS with more than one
/// instance — Heroku `web.1..web.N`, Fly machines, a scaled Render service —
/// every instance has its own ephemeral disk, so a bot added through
/// `/api/bots/add` exists on exactly ONE instance. Measured on Heroku with 5
/// dynos: 14 `/api/bots` reads returned **5 distinct pools**, each holding only
/// its own auto-provisioned guest bot, and the bot the user added was visible
/// on one of them. An env list is the only bot state every instance shares.
/// Split a `BOT_TOKENS` value (comma / semicolon / any whitespace, so a
/// newline-separated `heroku config:set` paste works) and append each token not
/// already present.
fn parse_token_list(raw: &str, out: &mut Vec<String>) {
    for part in raw.split([',', ';', '\n', '\r', '\t', ' ']) {
        let token = part.trim();
        // A bot token is always `<digits>:<secret>`; requiring the colon keeps a
        // stray word in the list from being treated as a credential.
        if !token.is_empty() && token.contains(':') && !out.iter().any(|t| t == token) {
            out.push(token.to_string());
        }
    }
}

pub fn env_bot_tokens() -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    // `std::env::var` directly (not the `env()` helper) so an EMPTY `BOT_TOKENS`
    // cannot shadow the singular fallbacks.
    for key in ["BOT_TOKENS", "TG_BOT_TOKENS"] {
        if let Ok(raw) = std::env::var(key) {
            parse_token_list(&raw, &mut out);
        }
    }
    for key in ["BOT_TOKEN", "TG_BOT_TOKEN"] {
        if let Ok(raw) = std::env::var(key) {
            parse_token_list(&raw, &mut out);
        }
    }
    out
}

/// The env token that owns `bot_id`, so a bot listed only in `BOT_TOKENS` can be
/// logged in on demand (it has no session file on a freshly booted instance).
pub fn env_token_for(bot_id: &str) -> Option<String> {
    let id = bot_id.trim();
    if id.is_empty() {
        return None;
    }
    env_bot_tokens()
        .into_iter()
        .find(|t| t.split(':').next().unwrap_or("").trim() == id)
}

pub fn get_bot_id() -> String {
    let meta = load_session_meta();
    let from_meta = meta
        .get("bot_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if !from_meta.is_empty() {
        return from_meta;
    }
    if let Some(first) = env_bot_tokens().first() {
        let id = first.split(':').next().unwrap_or("").trim();
        if !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()) {
            return id.to_string();
        }
    }
    let pool = read_pool_file();
    for b in &pool {
        let active = b.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false);
        let id = b.get("bot_id").and_then(|v| v.as_str()).unwrap_or("");
        if active && !id.is_empty() {
            return id.to_string();
        }
    }
    if let Some(first) = pool.first() {
        let id = first.get("bot_id").and_then(|v| v.as_str()).unwrap_or("");
        if !id.is_empty() {
            return id.to_string();
        }
    }
    fs::read_to_string(bot_id_cache_path())
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

pub fn clear_bot_id() {
    let _ = fs::remove_file(bot_id_cache_path());
}

pub fn has_local_session(bot_id: &str) -> bool {
    let id = bot_id.trim();
    if !id.is_empty() {
        return crate::telegram::has_session_file(id);
    }
    let active = get_bot_id();
    if !active.is_empty() && crate::telegram::has_session_file(&active) {
        return true;
    }
    for b in get_bot_pool() {
        let bid = b.get("bot_id").and_then(|v| v.as_str()).unwrap_or("");
        if !bid.is_empty() && crate::telegram::has_session_file(bid) {
            return true;
        }
    }
    false
}

/* ── bot pool ────────────────────────────────────────────────────────────── */

fn read_pool_file() -> Vec<Value> {
    fs::read_to_string(bot_pool_path())
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default()
}

/// The on-disk pool, repaired with the active bot when `session_meta.json`
/// proves it existed. Every MUTATION uses this (never the env-merged view), so a
/// bot supplied by `BOT_TOKENS` is never written into `bot_pool.json`.
fn file_pool() -> Vec<Value> {
    let mut pool = read_pool_file();
    let active = get_bot_id();
    // Only repair a genuinely missing entry (session_meta.json on disk), exactly
    // like the PHP: never resurrect a cleared session.
    if !active.is_empty()
        && !pool
            .iter()
            .any(|b| b.get("bot_id").and_then(|v| v.as_str()) == Some(active.as_str()))
        && session_meta_path().exists()
    {
        let meta = load_session_meta();
        pool.push(serde_json::json!({
            "bot_id": active,
            "bot_username": meta.get("bot_username").and_then(|v| v.as_str()).unwrap_or(""),
            "bot_name": meta.get("bot_name").and_then(|v| v.as_str()).unwrap_or(active.as_str()),
            "status": "online",
            "added_at": now(),
            "updated_at": now(),
            "is_active": true,
        }));
        save_bot_pool(&pool);
    }
    pool
}

/// The pool as every reader sees it: the on-disk entries plus every bot the
/// environment supplies. On a multi-instance deploy this is what makes all
/// instances agree on the pool — see [`env_bot_tokens()`]. Env entries carry
/// `"is_env": true` and are NOT persisted, so unsetting the variable removes
/// them again.
pub fn get_bot_pool() -> Vec<Value> {
    let mut pool = file_pool();
    for token in env_bot_tokens() {
        let id = token.split(':').next().unwrap_or("").trim().to_string();
        if id.is_empty() {
            continue;
        }
        if pool
            .iter()
            .any(|b| b.get("bot_id").and_then(|v| v.as_str()) == Some(id.as_str()))
        {
            continue;
        }
        pool.push(serde_json::json!({
            "bot_id": id,
            "bot_username": "",
            "bot_name": format!("Bot {}", id),
            "status": "online",
            "source": "env",
            "is_env": true,
            "added_at": now(),
            "updated_at": now(),
        }));
    }
    pool
}

/// The pool as WORK selection must see it: only bots this instance can actually
/// stream with.
///
/// [`get_bot_pool()`] is the display/administration view — a bot that is listed
/// but has never logged in HERE still belongs in it, or the dashboard could not
/// offer it and `remove_pool_bot()` could not find it. It must never be the
/// download view: a bot with no session file cannot fetch a byte, and while a
/// login `FLOOD_WAIT` is recorded it cannot even be logged in. Picking one
/// therefore guarantees a doomed `boot()` that RE-ARMS the wait — measured live:
/// `retry_after: 2421`, because the login attempt itself was the flood.
pub fn pool_ready() -> Vec<Value> {
    get_bot_pool()
        .into_iter()
        .filter(|b| {
            let id = b.get("bot_id").and_then(|v| v.as_str()).unwrap_or("");
            !id.is_empty() && crate::telegram::has_session_file(id)
        })
        .collect()
}

pub fn save_bot_pool(pool: &[Value]) {
    let body = serde_json::to_string_pretty(&Value::Array(pool.to_vec()))
        .unwrap_or_else(|_| "[]".to_string());
    let _ = fs::write(bot_pool_path(), body);
}

pub fn add_pool_bot(bot_data: Value) -> Vec<Value> {
    let bot_id = bot_data.get("bot_id").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    if bot_id.is_empty() {
        return Vec::new();
    }
    let mut pool = file_pool();
    let mut found = false;
    for item in pool.iter_mut() {
        if item.get("bot_id").and_then(|v| v.as_str()) == Some(bot_id.as_str()) {
            if let (Some(dst), Some(src)) = (item.as_object_mut(), bot_data.as_object()) {
                for (k, v) in src {
                    dst.insert(k.clone(), v.clone());
                }
            }
            found = true;
            break;
        }
    }
    if !found {
        let mut entry = serde_json::json!({
            "bot_id": bot_id,
            "bot_username": "",
            "bot_name": "",
            "status": "online",
            "added_at": now(),
            "updated_at": now(),
        });
        if let (Some(dst), Some(src)) = (entry.as_object_mut(), bot_data.as_object()) {
            for (k, v) in src {
                dst.insert(k.clone(), v.clone());
            }
        }
        pool.push(entry);
    }
    save_bot_pool(&pool);
    pool
}

/// Record a successfully booted env-supplied bot's REAL identity, so the pool
/// list shows `@username` instead of the placeholder name derived from its id
/// ("Bot 8990816622"). `get_bot_pool()` only fills that placeholder for an env
/// bot that has no on-disk entry yet, and this is what creates one.
pub fn remember_env_identity(bot_id: &str, username: &str, name: &str) {
    let id = bot_id.trim();
    if id.is_empty() {
        return;
    }
    add_pool_bot(serde_json::json!({
        "bot_id": id,
        "bot_username": username,
        "bot_name": if name.is_empty() { username } else { name },
        "status": "online",
        "source": "env",
        "is_env": true,
        "updated_at": now(),
    }));
}

/// Which bot becomes primary after a removal: one that holds a session file can
/// serve requests immediately, a bare pool row cannot.
///
/// Kept separate from [`remove_pool_bot()`] so the choice is testable without a
/// storage directory.
fn pick_replacement<'a>(
    pool: &'a [Value],
    has_session: impl Fn(&str) -> bool,
) -> Option<&'a Value> {
    pool.iter()
        .find(|b| {
            b.get("bot_id")
                .and_then(|v| v.as_str())
                .map(&has_session)
                .unwrap_or(false)
        })
        .or_else(|| pool.first())
}

/// Make `bot_id` the only entry flagged `is_active` in `bot_pool.json`.
///
/// [`get_bot_id()`] falls back to the first `is_active` row and the dashboard
/// renders its "Primary" badge from it, so two rows claiming to be active made
/// the primary depend on file order.
fn mark_only_active(bot_id: &str) {
    let mut pool = file_pool();
    let mut changed = false;
    for b in pool.iter_mut() {
        let is_target = b.get("bot_id").and_then(|v| v.as_str()) == Some(bot_id);
        if let Some(obj) = b.as_object_mut() {
            let was = obj.get("is_active").and_then(|v| v.as_bool()).unwrap_or(false);
            if was != is_target {
                obj.insert("is_active".to_string(), Value::Bool(is_target));
                changed = true;
            }
        }
    }
    if changed {
        save_bot_pool(&pool);
    }
}

/// Remove a bot from the pool.
///
/// Removing the PRIMARY bot must hand the primary role to another bot that can
/// actually log in, and removing the LAST bot must leave the instance with no
/// session at all — the dashboard turns that into the "enter a bot token" gate.
///
/// It used to promote `next.first()` unconditionally. When that entry had no
/// session file (a row that was added but never logged in, or whose session
/// directory had already been cleaned) `/api/session` answered
/// `has_session: false` while the pool still listed a bot, so the dashboard
/// skipped the token gate and fell into guest provisioning — the endless
/// "Provisioning guest bot..." spinner. Preferring a bot with a session file and
/// clearing the identity files with the last bot keeps both paths honest.
pub fn remove_pool_bot(bot_id: &str) -> Vec<Value> {
    let id = bot_id.trim();
    if id.is_empty() {
        return get_bot_pool();
    }
    // File pool only: an env-supplied bot has no file entry, and filtering the
    // merged view would persist the env entries into `bot_pool.json`.
    let next: Vec<Value> = file_pool()
        .into_iter()
        .filter(|b| b.get("bot_id").and_then(|v| v.as_str()) != Some(id))
        .collect();
    save_bot_pool(&next);
    clear_session_directory(&crate::telegram::session_dir(id));
    if next.is_empty() {
        let _ = fs::remove_file(bot_pool_path());
    }
    let active = get_bot_id();
    if active == id || active.is_empty() {
        // A bot that holds a session file can be used immediately; a bare pool
        // row cannot. Only fall back to the first row when nothing can log in.
        let replacement = pick_replacement(&next, crate::telegram::has_session_file);
        match replacement {
            Some(bot) => {
                let promoted = bot
                    .get("bot_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                save_session_meta(
                    &promoted,
                    bot.get("bot_username").and_then(|v| v.as_str()).unwrap_or(""),
                    bot.get("bot_name").and_then(|v| v.as_str()).unwrap_or(""),
                );
                mark_only_active(&promoted);
            }
            None => {
                // With `BOT_TOKENS` set the machine still has bots, so promote the
                // first env bot instead of leaving the instance with no active
                // session. Otherwise drop every identity file, including the
                // legacy `bot_id.txt` cache — leaving it behind let `get_bot_id()`
                // report the removed bot as the active one.
                match env_bot_tokens().first() {
                    Some(token) => {
                        let env_id = token.split(':').next().unwrap_or("").trim().to_string();
                        save_session_meta(&env_id, "", "");
                    }
                    None => {
                        clear_session_meta();
                        clear_session();
                        clear_bot_id();
                    }
                }
            }
        }
    }
    get_bot_pool()
}

pub fn filter_and_pick_pool_bot<'a>(
    pool: &'a [Value],
    ts: i64,
    has_session: impl Fn(&str) -> bool,
    idx: usize,
) -> Option<(&'a Value, usize)> {
    let valid: Vec<&Value> = pool
        .iter()
        .filter(|b| {
            let id = b.get("bot_id").and_then(|v| v.as_str()).unwrap_or("");
            if id.is_empty() {
                return false;
            }
            let cooldown = b.get("cooldown_until").and_then(|v| v.as_i64()).unwrap_or(0);
            if cooldown > ts {
                return false;
            }
            has_session(id)
        })
        .collect();
    if valid.is_empty() {
        return None;
    }
    let picked_idx = idx % valid.len();
    let next_idx = (idx + 1) % valid.len();
    Some((valid[picked_idx], next_idx))
}

pub fn pick_pool_bot() -> Option<Value> {
    let pool = get_bot_pool();
    let active = get_bot_id();
    let ts = now();
    let rr = storage_path("rr_bot_index.txt");
    let idx: usize = fs::read_to_string(&rr)
        .ok()
        .and_then(|t| t.trim().parse::<usize>().ok())
        .unwrap_or(0);

    if let Some((picked, next)) = filter_and_pick_pool_bot(&pool, ts, crate::telegram::has_session_file, idx) {
        let _ = fs::write(&rr, next.to_string());
        return Some(picked.clone());
    }

    // The fallback used to hand back the active bot unconditionally, i.e. a bot
    // that has never logged in on this instance was returned as if it could
    // stream — and every caller then paid a login attempt for it. `filter_and_pick`
    // above already requires a session file; the fallback has to agree, or it
    // silently undoes the filter whenever the pool is empty or fully gated.
    if !active.is_empty() && crate::telegram::has_session_file(&active) {
        let meta = load_session_meta();
        return Some(serde_json::json!({
            "bot_id": active,
            "bot_username": meta.get("bot_username").and_then(|v| v.as_str()).unwrap_or(""),
            "bot_name": meta.get("bot_name").and_then(|v| v.as_str()).unwrap_or(""),
        }));
    }
    None
}

/// Exactly ONE guest bot is leased per instance — no more, and there is **no
/// knob** to raise it.
///
/// Every guest bot is a lease from the origin's shared pool
/// (`provision-session`, `<20` provisions per bot, 300 candidates ≈ 6 000
/// leases total). A per-instance multiplier would therefore let anyone drain that
/// budget just by scaling out or by setting an environment variable, so the limit
/// is a constant rather than a setting.
///
/// A user who wants a bigger pool adds **their own bots** — `BOT_TOKENS` in the
/// environment, or the dashboard's Bot Pool UI — which costs the origin nothing.
/// Their bots count towards this same one-bot check, so adding one means no guest
/// is leased at all.
pub const GUEST_BOTS_PER_INSTANCE: usize = 1;

/* ── login flood guard ───────────────────────────────────────────────────── */

/// When Telegram rate-limited us, and until when — kept in the server's own cache
/// (`storage/cache.redb`, see [`crate::cache`]), not in a file of its own.
///
/// Telegram answers `auth.importBotAuthorization` with `420 FLOOD_WAIT` after too
/// many logins, and **every further attempt re-arms the wait**. A public addon is
/// the worst case for that: a client that retries (a player reopening a stream,
/// a dozen viewers at once) turns one throttle into a permanent lockout.
/// Measured on this deployment: 6 bots x 5 dynos logging in on one restart
/// produced `FLOOD_WAIT (value: 455..961)` and every subsequent
/// `/api/download` answered 403 for ~15 minutes.
///
/// The entry's TTL is the wait itself, so it expires by construction and nothing
/// has to clean it up.
const FLOOD_NS: &str = "telegram_flood";
const FLOOD_KEY: &str = "global";

/// The recorded wait for ONE bot token (0 = none / expired).
///
/// The wait is PER TOKEN, not per instance. Measured 2026-10-26 against the six
/// `BOT_TOKENS` bots, one `POST /api/botlogin` each:
///
/// ```text
/// 8990816622  FLOOD_WAIT (value: 1365)      8749344367  logged in, 376 ms
/// 8622165803  FLOOD_WAIT (value: 1369)      8469916399  logged in, 398 ms
/// 8848916386  FLOOD_WAIT (value: 1379)
/// 8773958156  FLOOD_WAIT (value: 1381)
/// ```
///
/// Four separate counters, each shortening on its own, while two bots signed in
/// immediately. A single global flag therefore took two healthy bots offline to
/// punish four, and — worse — refused logins that would have worked, turning a
/// partial outage into a total one.
pub fn flood_remaining_for(bot_id: &str) -> i64 {
    let id = bot_id.trim();
    if id.is_empty() {
        return flood_remaining();
    }
    crate::cache::get(FLOOD_NS, &format!("bot:{}", id))
        .and_then(|v| v.get("until").and_then(|u| u.as_i64()))
        .map(|until| (until - now()).max(0))
        .unwrap_or(0)
}

/// Record a `FLOOD_WAIT` against ONE bot token. Returns the seconds recorded.
pub fn note_flood_for(bot_id: &str, error: &str) -> i64 {
    let id = bot_id.trim();
    if id.is_empty() {
        return note_flood(error);
    }
    let secs = parse_flood_seconds(error);
    if secs <= 0 {
        return 0;
    }
    if secs <= flood_remaining_for(id) {
        return secs;
    }
    let body = serde_json::json!({
        "until": now() + secs,
        "seconds": secs,
        "recorded_at": now(),
        "bot_id": id,
        "error": error,
    });
    crate::cache::put(FLOOD_NS, &format!("bot:{}", id), &body, secs, secs);
    log("telegram login flood wait recorded (per bot)", &body);
    secs
}

/// Seconds of a flood wait quoted inside an error string (0 = not a flood).
pub fn flood_seconds_in(text: &str) -> i64 {
    parse_flood_seconds(text)
}

/// Seconds left on the recorded INSTANCE-wide wait (0 = nothing / expired).
///
/// Only the guest lease uses this: a guest bot has no id to key a per-token wait
/// on until the lease answers. Every named bot uses [`flood_remaining_for()`].
pub fn flood_remaining() -> i64 {
    crate::cache::get(FLOOD_NS, FLOOD_KEY)
        .and_then(|v| v.get("until").and_then(|u| u.as_i64()))
        .map(|until| (until - now()).max(0))
        .unwrap_or(0)
}

/// The Telegram error text that produced the recorded wait, verbatim.
///
/// Kept so a refused login can quote Telegram instead of inventing its own
/// wording — "FLOOD_WAIT ... (value: 460)" is what an operator can act on.
pub fn flood_last_error() -> String {
    crate::cache::get(FLOOD_NS, FLOOD_KEY)
        .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()))
        .unwrap_or_default()
}

/// Record a `FLOOD_WAIT` seen in a boot error. Returns the seconds recorded
/// (0 when the error was not a flood wait). Never shortens an existing wait.
pub fn note_flood(error: &str) -> i64 {
    let secs = parse_flood_seconds(error);
    if secs <= 0 {
        return 0;
    }
    if secs <= flood_remaining() {
        return secs;
    }
    let body = serde_json::json!({
        "until": now() + secs,
        "seconds": secs,
        "recorded_at": now(),
        "error": error,
    });
    crate::cache::put(FLOOD_NS, FLOOD_KEY, &body, secs, secs);
    log("telegram login flood wait recorded", &body);
    secs
}

/// `(value: 460)`, `value: 460`, `FLOOD_WAIT_460` — grammers/Telegram render the
/// wait in all three shapes.
fn parse_flood_seconds(error: &str) -> i64 {
    let lower = error.to_ascii_lowercase();
    if !lower.contains("flood_wait") && !lower.contains("flood wait") {
        return 0;
    }
    let digits_after = |needle: &str| -> i64 {
        if let Some(pos) = lower.find(needle) {
            let rest = &lower[pos + needle.len()..];
            let mut num = String::new();
            for ch in rest.chars() {
                if ch.is_ascii_digit() {
                    num.push(ch);
                } else if !num.is_empty() {
                    break;
                } else if ch != '_' && ch != ' ' && ch != ':' && ch != '(' {
                    break;
                }
            }
            num.parse::<i64>().unwrap_or(0)
        } else {
            0
        }
    };
    let mut secs = digits_after("value:");
    if secs == 0 {
        secs = digits_after("flood_wait_");
    }
    if secs == 0 {
        // Last resort: the first number after the FLOOD_WAIT marker.
        if let Some(pos) = lower.find("flood_wait") {
            let rest = &lower[pos..];
            let num: String = rest
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(|c| c.is_ascii_digit())
                .collect();
            secs = num.parse::<i64>().unwrap_or(0);
        }
    }
    if secs > 0 {
        secs.min(86_400)
    } else {
        0
    }
}

/* ── guest provisioning ──────────────────────────────────────────────────── */

fn provision_lock_path() -> PathBuf {
    storage_path("guest_provision.lock")
}

fn provision_heartbeat_path() -> PathBuf {
    storage_path("guest_provision.heartbeat")
}

/// Is `pid` a process that still exists?
///
/// The provision lock is a FILE, so a holder that is killed (Android restarts the
/// service, the watchdog fires, the user swipes the app away) can never remove it.
/// The recorded pid is the only durable proof of liveness on one machine, and it
/// is what separates "someone is provisioning right now" from "someone was
/// provisioning when the process died".
fn pid_still_running(pid: i64) -> bool {
    if pid <= 1 {
        return false;
    }
    #[cfg(windows)]
    {
        crate::tunnel::win_pid_alive(pid)
    }
    #[cfg(unix)]
    {
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }
    #[cfg(not(any(windows, unix)))]
    {
        true
    }
}

/// The pid recorded in the lock file, when it looks sane.
fn lock_owner_pid(path: &Path) -> Option<i64> {
    let text = fs::read_to_string(path).ok()?;
    let first = text.split_whitespace().next()?;
    first.parse::<i64>().ok().filter(|p| *p > 1)
}

/// `fd_is_guest_provision_in_progress()`: a dead holder or a >300s lock is reclaimed.
pub fn is_guest_provision_in_progress() -> bool {
    let file = provision_lock_path();
    if fs::metadata(&file).is_err() {
        return false;
    }
    let age = crate::storage::mtime_secs(&file)
        .map(|m| now() - m)
        .unwrap_or(0);
    // Measured on a CPH2581: the engine was restarted at 06:54:03 with a provision
    // in flight, so its lock file survived with nobody to drop it. Every play then
    // answered `NO_SESSION` for seven minutes, each attempt first waiting 30 s on
    // this lock, until the 300 s age fallback finally fired at 07:01:27. A dead
    // owner is reclaimed on the FIRST check instead.
    if let Some(owner) = lock_owner_pid(&file) {
        if !pid_still_running(owner) {
            log(
                "reclaimed guest_provision.lock from a dead process",
                &serde_json::json!({"pid": owner, "age": age}),
            );
            let _ = fs::remove_file(&file);
            return false;
        }
    }
    if age > 300 {
        log("reclaimed stale guest_provision.lock", &serde_json::json!({"age": age}));
        let _ = fs::remove_file(&file);
        return false;
    }
    if let Ok(hb) = fs::read_to_string(provision_heartbeat_path()) {
        if let Ok(v) = hb.trim().parse::<i64>() {
            if now() - v < 30 {
                return true;
            }
        }
    }
    age < 120
}

struct ProvisionLock;

impl ProvisionLock {
    async fn acquire(wait_ms: u64) -> Option<ProvisionLock> {
        let file = provision_lock_path();
        let start = std::time::Instant::now();
        loop {
            match fs::OpenOptions::new().write(true).create_new(true).open(&file) {
                Ok(mut f) => {
                    // Our own pid: the next process to look at this file can tell a
                    // live holder from a killed one.
                    let _ = f.write_all(std::process::id().to_string().as_bytes());
                    let _ = fs::write(provision_heartbeat_path(), now().to_string());
                    return Some(ProvisionLock);
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let dead_owner = lock_owner_pid(&file)
                        .map(|p| !pid_still_running(p))
                        .unwrap_or(false);
                    let too_old = crate::storage::mtime_secs(&file)
                        .map(|m| now() - m > 300)
                        .unwrap_or(false);
                    if dead_owner || too_old {
                        let owner = lock_owner_pid(&file).unwrap_or(0);
                        log(
                            "reclaimed guest_provision.lock",
                            &serde_json::json!({"pid": owner, "dead_owner": dead_owner, "too_old": too_old}),
                        );
                        let _ = fs::remove_file(&file);
                        continue;
                    }
                    if start.elapsed().as_millis() as u64 > wait_ms {
                        return None;
                    }
                    // Deliberately NO heartbeat write here. A waiter refreshing the
                    // holder's heartbeat is what kept a dead lock looking alive for
                    // the whole age window.
                    tokio::time::sleep(Duration::from_millis(250)).await;
                }
                Err(_) => return None,
            }
        }
    }
}

impl Drop for ProvisionLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(provision_lock_path());
        let _ = fs::remove_file(provision_heartbeat_path());
    }
}

pub struct ClockProbe {
    pub ok: bool,
    pub offset: i64,
    pub server_time: i64,
    pub local_time: i64,
    pub error: Option<String>,
}

fn parse_http_date(raw: &str) -> Option<i64> {
    // RFC 7231 IMF-fixdate: "Mon, 20 Oct 2026 00:32:39 GMT" — the FIRST token
    // is the weekday, so it must be skipped (reading parts[0] as the day made
    // every probe report "clock probe unreachable").
    let mut parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.len() >= 6 && parts[0].ends_with(',') {
        parts.remove(0);
    }
    let parts = parts;
    if parts.len() < 5 {
        return None;
    }
    let day: i64 = parts[0].parse().ok()?;
    let month = match parts[1] {
        "Jan" => 1,
        "Feb" => 2,
        "Mar" => 3,
        "Apr" => 4,
        "May" => 5,
        "Jun" => 6,
        "Jul" => 7,
        "Aug" => 8,
        "Sep" => 9,
        "Oct" => 10,
        "Nov" => 11,
        "Dec" => 12,
        _ => return None,
    };
    let year: i64 = parts[2].parse().ok()?;
    let time: Vec<&str> = parts[3].split(':').collect();
    if time.len() != 3 {
        return None;
    }
    let hh: i64 = time[0].parse().ok()?;
    let mm: i64 = time[1].parse().ok()?;
    let ss: i64 = time[2].parse().ok()?;
    Some(days_from_civil(year, month, day) * 86400 + hh * 3600 + mm * 60 + ss)
}

fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

fn date_of(resp: &reqwest::Response) -> Option<i64> {
    let date = resp.headers().get(reqwest::header::DATE)?;
    parse_http_date(date.to_str().ok()?)
}

/// `fd_measure_clock_offset()`: `Date` header vs local time.
pub async fn measure_clock_offset() -> ClockProbe {
    let local = now();
    for url in ["https://pencarimovie.com/", "https://core.telegram.org/"] {
        // HEAD is cheap, but some edges answer it without a Date header, so a
        // GET is tried before giving up on the host.
        for use_head in [true, false] {
            let resp = {
                let c = crate::httpc::client();
                let req = if use_head { c.head(url) } else { c.get(url) };
                req.timeout(Duration::from_secs(4)).send().await
            };
            if let Ok(resp) = resp {
                if let Some(server) = date_of(&resp) {
                    return ClockProbe {
                        ok: true,
                        // PHP: `offset = $localTime - $serverTime` — POSITIVE
                        // means the local clock is AHEAD. Subtracting the other
                        // way round flips the "ahead of"/"behind" wording of
                        // clock_skew_message(), and a device 40 s BEHIND was
                        // told it was ahead — the one fact the user cannot
                        // check for themselves. Pinned by a unit test.
                        offset: now() - server,
                        server_time: server,
                        local_time: now(),
                        error: None,
                    };
                }
            }
        }
    }
    ClockProbe {
        ok: false,
        offset: 0,
        server_time: 0,
        local_time: local,
        error: Some("clock probe unreachable".to_string()),
    }
}

pub fn clock_skew_message(offset_seconds: i64) -> String {
    let off = offset_seconds.abs();
    let mins = off / 60;
    let secs = off % 60;
    let human = if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    };
    // offset is `local - server` (PHP), so positive = local is ahead.
    let dir = if offset_seconds > 0 { "ahead of" } else { "behind" };
    format!("Your clock is {} {} Telegram's server time.", human, dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sign convention is PHP's (`local - server`) and it drives the only
    /// user-facing sentence here, so it is worth pinning: getting it backwards
    /// tells a fast clock it is slow.
    #[test]
    fn clock_skew_message_says_ahead_for_a_fast_clock() {
        assert!(clock_skew_message(40).contains("40s ahead of"));
        assert!(clock_skew_message(-40).contains("40s behind"));
        assert!(clock_skew_message(72).contains("1m 12s ahead of"));
        assert_eq!(
            clock_skew_message(-432),
            "Your clock is 7m 12s behind Telegram's server time."
        );
    }

    /// The guest-bot limit is a constant, not a setting: raising it must stay a
    /// code change, so nobody can farm the origin's provisioning from a config var.
    #[test]
    fn one_guest_bot_per_instance_is_a_constant() {
        assert_eq!(GUEST_BOTS_PER_INSTANCE, 1);
    }

    /// Removing the primary must hand the role to a bot that can log in.
    /// Picking the first row regardless is what sent the dashboard into the
    /// endless "Provisioning guest bot..." spinner: `/api/session` answered
    /// `has_session: false` while the pool still listed a bot, so the token gate
    /// was skipped and guest provisioning took over.
    #[test]
    fn removal_promotes_a_bot_that_can_actually_log_in() {
        let pool = vec![
            serde_json::json!({"bot_id": "333", "bot_name": "never logged in"}),
            serde_json::json!({"bot_id": "222", "bot_name": "ready"}),
        ];
        let picked = pick_replacement(&pool, |id| id == "222").expect("a replacement");
        assert_eq!(picked["bot_id"], "222");

        // Nothing can log in: fall back to the first row, and the UI gates.
        let picked = pick_replacement(&pool, |_| false).expect("a replacement");
        assert_eq!(picked["bot_id"], "333");

        // An empty pool has no replacement at all.
        assert!(pick_replacement(&[], |_| true).is_none());

        // A row without a bot_id is never picked as the primary.
        let junk = vec![
            serde_json::json!({"bot_name": "no id"}),
            serde_json::json!({"bot_id": "444"}),
        ];
        assert_eq!(pick_replacement(&junk, |_| true).unwrap()["bot_id"], "444");
    }

    /// Telegram renders the login wait in several shapes, and a missed parse means
    /// the guard does nothing while the addon keeps re-arming the throttle.
    #[test]
    fn flood_wait_is_parsed_from_every_shape() {
        assert_eq!(
            parse_flood_seconds(
                "bot sign-in failed: request error: rpc error 420: FLOOD_WAIT caused by \
                 auth.importBotAuthorization (value: 460)"
            ),
            460
        );
        assert_eq!(parse_flood_seconds("FLOOD_WAIT_120"), 120);
        assert_eq!(parse_flood_seconds("rpc error 420: FLOOD_WAIT 45"), 45);
        assert_eq!(parse_flood_seconds("bot sign-in failed: invalid token"), 0);
        assert_eq!(parse_flood_seconds(""), 0);
    }

    /// Both flood tests below read and write the ONE instance-wide key, and
    /// `cargo test` runs test functions on parallel threads of a single process.
    /// The 30 s wait that `a_recorded_flood_wait_blocks_the_next_login` records
    /// was therefore exactly what `flood_remaining()` returned while the test
    /// below asserted 0 (`left: 30, right: 0`) — a shared-state race, not a
    /// behaviour bug, because the cache is per-process global. Serialize the two
    /// and start each from a clean `telegram_flood` namespace.
    static FLOOD_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn flood_test_guard() -> std::sync::MutexGuard<'static, ()> {
        // A panicking sibling poisons the lock; the cache is disposable, so take
        // the inner guard rather than cascading one failure into both tests.
        let guard = FLOOD_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::cache::clear(Some(FLOOD_NS));
        guard
    }

    /// The recorded wait must actually gate the next login — that is the whole
    /// point of the guard (a public addon otherwise retries itself into a lockout).
    /// The wait is PER TOKEN. Measured against the six `BOT_TOKENS` bots: four
    /// answered `FLOOD_WAIT (value: 1365/1369/1379/1381)` while two signed in in
    /// under 400 ms. Recording one bot's wait globally took the healthy bots
    /// offline with the sick ones and refused logins that would have worked.
    #[test]
    fn a_flood_wait_belongs_to_one_bot_only() {
        let _guard = flood_test_guard();
        let secs = note_flood_for(
            "bot_a",
            "bot sign-in failed: request error: rpc error 420: FLOOD_WAIT caused by \
             auth.importBotAuthorization (value: 1365)",
        );
        assert_eq!(secs, 1365);
        assert!(flood_remaining_for("bot_a") > 0, "the refused bot waits");
        assert_eq!(flood_remaining_for("bot_b"), 0, "another token is unaffected");
        assert_eq!(flood_remaining(), 0, "a named bot must not arm the guest gate");
    }

    #[test]
    fn a_recorded_flood_wait_blocks_the_next_login() {
        let _guard = flood_test_guard();
        assert_eq!(flood_remaining(), 0);
        let secs = note_flood(
            "rpc error 420: FLOOD_WAIT caused by auth.importBotAuthorization (value: 30)",
        );
        assert_eq!(secs, 30);
        assert!(flood_remaining() > 0, "a recorded wait must gate the next login");
        assert!(flood_remaining() <= 30);
    }

    /// `BOT_TOKENS` is the only bot state every instance of a scaled-out deploy
    /// shares, so the parsing has to survive whatever an operator pastes into a
    /// PaaS config var: commas, spaces, newlines, duplicates, and the usual
    /// accidental junk.
    /// A lock left by a process that is GONE must be reclaimed on the first look.
    ///
    /// Measured on a CPH2581: the engine was restarted with a provision in flight,
    /// the lock file outlived the process that owned it, and every play answered
    /// `NO_SESSION` for seven minutes (each attempt waiting 30 s on the lock first)
    /// until the 300 s age fallback fired.
    #[test]
    fn a_lock_from_a_dead_process_is_reclaimed_at_once() {
        let path = provision_lock_path();
        let _ = fs::remove_file(&path);
        // A pid no process can have, so `pid_still_running` answers false on every OS.
        fs::write(&path, "2147483646").unwrap();
        assert!(
            !is_guest_provision_in_progress(),
            "a dead holder must not count as a provision in progress"
        );
        assert!(!path.exists(), "the dead lock must be removed");
    }

    /// The other half: a LIVE holder is still respected, or two logins would run.
    #[test]
    fn a_lock_from_a_live_process_counts_as_in_progress() {
        let path = provision_lock_path();
        let _ = fs::remove_file(&path);
        fs::write(&path, std::process::id().to_string()).unwrap();
        assert!(
            is_guest_provision_in_progress(),
            "a live holder must be respected"
        );
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn bot_token_list_parses_every_separator_and_dedupes() {
        let mut out = Vec::new();
        parse_token_list(
            "111:AAA, 222:BBB\n333:CCC\t111:AAA  ,\n  ;444:DDD",
            &mut out,
        );
        assert_eq!(out, vec!["111:AAA", "222:BBB", "333:CCC", "444:DDD"]);

        // A word without the `id:secret` colon is not a credential.
        let mut out = Vec::new();
        parse_token_list("BOT_TOKEN 555:EEE notatoken", &mut out);
        assert_eq!(out, vec!["555:EEE"]);

        let mut out = Vec::new();
        parse_token_list("", &mut out);
        assert!(out.is_empty());
    }

    #[test]
    fn round_robin_spreads_across_valid_pool_bots() {
        let pool = vec![
            serde_json::json!({"bot_id": "111", "cooldown_until": 0}),
            serde_json::json!({"bot_id": "222", "cooldown_until": 0}),
            serde_json::json!({"bot_id": "333", "cooldown_until": 0}),
        ];
        let has_session = |_id: &str| true;
        let ts = 1000;

        let (first, next1) = filter_and_pick_pool_bot(&pool, ts, has_session, 0).unwrap();
        assert_eq!(first["bot_id"], "111");
        assert_eq!(next1, 1);

        let (second, next2) = filter_and_pick_pool_bot(&pool, ts, has_session, next1).unwrap();
        assert_eq!(second["bot_id"], "222");
        assert_eq!(next2, 2);

        let (third, next3) = filter_and_pick_pool_bot(&pool, ts, has_session, next2).unwrap();
        assert_eq!(third["bot_id"], "333");
        assert_eq!(next3, 0);

        // Wraps back to first bot
        let (fourth, next4) = filter_and_pick_pool_bot(&pool, ts, has_session, next3).unwrap();
        assert_eq!(fourth["bot_id"], "111");
        assert_eq!(next4, 1);
    }

    #[test]
    fn round_robin_skips_cooldown_and_sessionless_bots() {
        let ts = 1000;
        let pool = vec![
            serde_json::json!({"bot_id": "111", "cooldown_until": 2000}), // on cooldown (2000 > 1000)
            serde_json::json!({"bot_id": "222", "cooldown_until": 0}),    // valid
            serde_json::json!({"bot_id": "333", "cooldown_until": 0}),    // has no session
            serde_json::json!({"bot_id": "444", "cooldown_until": 0}),    // valid
        ];
        let has_session = |id: &str| id != "333";

        let (first, next1) = filter_and_pick_pool_bot(&pool, ts, has_session, 0).unwrap();
        assert_eq!(first["bot_id"], "222");
        assert_eq!(next1, 1);

        let (second, next2) = filter_and_pick_pool_bot(&pool, ts, has_session, next1).unwrap();
        assert_eq!(second["bot_id"], "444");
        assert_eq!(next2, 0);

        // Wraps between 222 and 444 only
        let (third, next3) = filter_and_pick_pool_bot(&pool, ts, has_session, next2).unwrap();
        assert_eq!(third["bot_id"], "222");
        assert_eq!(next3, 1);
    }

}

pub fn is_clock_or_authkey_error(message: &str) -> bool {
    let s = message.to_ascii_lowercase();
    if s.is_empty() {
        return false;
    }
    if s.contains("session is busy") || s.contains("lock") || s.contains("econnrefused") {
        return false;
    }
    s.contains("auth_key")
        || s.contains("new_nonce_hash")
        || s.contains("msg_id too")
        || s.contains("message id")
        || s.contains("time delta")
        || s.contains("clock")
        || s.contains("sync your date")
}

pub fn friendly_login_error(message: &str) -> String {
    if is_clock_or_authkey_error(message) {
        return "Your device clock is out of sync with Telegram.\nEnable 'Set time automatically' (Automatic date and time / NTP) in your device Settings, then restart the PencariMovie Server.".to_string();
    }
    message.to_string()
}

pub fn find_orphan_session_bot_id() -> String {
    let dir = storage_path("sessions");
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if crate::telegram::has_session_file(&name) {
                return name;
            }
        }
    }
    String::new()
}

pub fn clear_session_directory(dir: &std::path::Path) {
    let _ = fs::remove_dir_all(dir);
}

pub fn clear_session() {
    let _ = fs::remove_dir_all(storage_path("sessions"));
    // Legacy PHP (MadelineProto) session directory, if this install ever had one.
    let _ = fs::remove_dir_all(storage_path("session.madeline"));
    crate::telegram::disconnect_all();
}

pub struct ProvisionResult {
    pub bot_id: String,
    pub bot_username: String,
    pub bot_name: String,
    pub error: Option<String>,
}

impl ProvisionResult {
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            bot_id: String::new(),
            bot_username: String::new(),
            bot_name: String::new(),
            error: Some(msg.into()),
        }
    }
}

/// PHP truthiness for a JSON scalar (`"ok": 1` counts as true).
pub fn truthy(v: Option<&Value>) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0 || n.as_f64().unwrap_or(0.0) != 0.0,
        Some(Value::String(s)) => !s.is_empty() && s != "0" && s != "false",
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
        Some(Value::Null) | None => false,
    }
}

fn random_suffix() -> String {
    let mut buf = [0u8; 8];
    rand::rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

/// `fd_auto_provision_guest()` (backend.php:976): same order of checks, same
/// error strings, same "try twice" loop — but the login is this stack's own.
pub async fn auto_provision_guest(
    tg: &crate::telegram::TelegramPool,
) -> Result<ProvisionResult, ProvisionResult> {
    // ── login flood guard ─────────────────────────────────────────────────
    // Never attempt a login while Telegram's FLOOD_WAIT is still running: the
    // attempt would fail AND push the wait further out, which is how a retrying
    // public addon locks itself out. Answer instantly instead.
    // Only the GUEST lease is gated instance-wide: it has no bot id to key on
    // yet. A named bot's wait is per token, so a BOT_TOKENS deploy never blocks
    // its healthy bots behind an unrelated bot's FLOOD_WAIT.
    let flood = if env_bot_tokens().is_empty() { flood_remaining() } else { 0 };
    if flood > 0 {
        let last = flood_last_error();
        return Err(ProvisionResult::error(if last.is_empty() {
            format!(
                "Telegram rate limit: bot login paused for another {}s. Please retry shortly.",
                flood
            )
        } else {
            // Quote Telegram, do not paraphrase it.
            format!(
                "Telegram rate limit: bot login paused for another {}s.\nTelegram said: {}",
                flood, last
            )
        }));
    }

    // ── clock pre-check ───────────────────────────────────────────────────
    let clock = measure_clock_offset().await;
    if clock.ok && clock.offset.abs() > 30 {
        log(
            "auto provision aborted: clock skew too large",
            &serde_json::json!({"offset_seconds": clock.offset}),
        );
        let human = clock_skew_message(clock.offset);
        return Err(ProvisionResult::error(format!(
            "Device clock out of sync!\n{}\n\nEnable 'Set time automatically' (Automatic date and time / NTP) in your device Settings, then restart the PencariMovie Server.",
            human
        )));
    }

    // ── existing session ──────────────────────────────────────────────────
    let existing = get_bot_id();
    if !existing.is_empty() && crate::telegram::has_session_file(&existing) {
        let meta = load_session_meta();
        match tg.boot(&existing, None).await {
            Ok(_) => {
                return Ok(ProvisionResult {
                    bot_id: existing,
                    bot_username: meta.get("bot_username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    bot_name: meta.get("bot_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    error: None,
                })
            }
            Err(e) => {
                if is_clock_or_authkey_error(&e) {
                    return Err(ProvisionResult::error(friendly_login_error(&e)));
                }
                log(
                    "existing session boot failed",
                    &serde_json::json!({"bot_id": existing, "error": e}),
                );
            }
        }
    }

    // 3 s, not 30. A real provision takes seconds, and the old 30 s wait was paid by
    // EVERY play attempt while a stale lock existed: measured on a CPH2581, eight
    // attempts in seven minutes, each blocked for 30 s before the 503. A caller that
    // cannot get the lock in 3 s is told "in progress" and retries, which is cheap.
    let _lock = match ProvisionLock::acquire(3_000).await {
        Some(l) => l,
        None => {
            // A provision that finished while we waited is a SUCCESS: answer with the
            // session instead of "in progress".
            let done = get_bot_id();
            if !done.is_empty() && crate::telegram::has_session_file(&done) {
                let meta = load_session_meta();
                return Ok(ProvisionResult {
                    bot_id: done,
                    bot_username: meta.get("bot_username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    bot_name: meta.get("bot_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    error: None,
                });
            }
            log(
                "auto provision skipped: another provision is currently in progress",
                &serde_json::json!({}),
            );
            return Err(ProvisionResult::error(
                "Connecting guest bot in progress. Please refresh or try again in a few seconds.",
            ));
        }
    };

    // Re-check after acquiring the lock (another worker may have finished).
    let recheck = get_bot_id();
    if !recheck.is_empty() && crate::telegram::has_session_file(&recheck) {
        let meta = load_session_meta();
        if tg.boot(&recheck, None).await.is_ok() {
            return Ok(ProvisionResult {
                bot_id: recheck,
                bot_username: meta.get("bot_username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                bot_name: meta.get("bot_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                error: None,
            });
        }
    }

    // Orphan session, INSIDE the lock and AFTER the re-check.
    //
    // A login writes `storage/sessions/<bot_id>/` and persists `bot_id` a moment
    // later, so the check must run while nothing else is provisioning. Holding the
    // lock first does that (the re-check above already serves the "another worker
    // just finished" case).
    //
    // This used to REFUSE service: "A previous login did not finish. Please restart
    // the PencariMovie Server and try again." A restart could never honour that,
    // because nothing on the boot path removes the directory. Measured on a
    // CPH2581: `sessions/8582733572` survived an engine restart and every provision
    // answered 500 for as long as it existed, which is the "auto provision not
    // working" report. A session file with no `bot_id` is a COMPLETED login whose
    // bookkeeping was lost, so ADOPT it first: it is a working bot and costs no
    // origin provision. Only a session that will not boot is discarded, and then a
    // fresh lease follows in the same call.
    if existing.is_empty() && get_bot_id().is_empty() {
        let orphan = find_orphan_session_bot_id();
        if !orphan.is_empty() {
            match tg.boot(&orphan, None).await {
                Ok(entry) => {
                    save_session_meta(
                        &entry.identity.id,
                        &entry.identity.username,
                        &entry.identity.first_name,
                    );
                    log(
                        "adopted an orphan session instead of refusing to provision",
                        &serde_json::json!({"bot_id": entry.identity.id}),
                    );
                    return Ok(ProvisionResult {
                        bot_id: entry.identity.id.clone(),
                        bot_username: entry.identity.username.clone(),
                        bot_name: entry.identity.first_name.clone(),
                        error: None,
                    });
                }
                Err(e) => {
                    log(
                        "discarding an orphan session that will not boot",
                        &serde_json::json!({"bot_id": orphan, "error": e}),
                    );
                    clear_session_directory(&storage_path(&format!("sessions/{}", orphan)));
                }
            }
        }
    }

    // Env-supplied tokens win when present (container / multi-instance installs).
    // Every token is tried in order: on a PaaS each instance must be able to log
    // in pool bots it did not add itself, and one expired token must not take the
    // whole pool down.
    let mut env_err: Option<String> = None;
    for token in env_bot_tokens() {
        let id_part = token.split(':').next().unwrap_or("").trim().to_string();
        let target = if !id_part.is_empty() && id_part.chars().all(|c| c.is_ascii_digit()) {
            id_part
        } else {
            format!("env_{}", random_suffix())
        };
        match tg.boot(&target, Some(&token)).await {
            Ok(entry) => {
                remember_env_identity(&entry.identity.id, &entry.identity.username, &entry.identity.first_name);
                save_session_meta(&entry.identity.id, &entry.identity.username, &entry.identity.first_name);
                let meta = load_session_meta();
                return Ok(ProvisionResult {
                    bot_id: meta.get("bot_id").and_then(|v| v.as_str()).unwrap_or(&entry.identity.id).to_string(),
                    bot_username: meta.get("bot_username").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    bot_name: meta.get("bot_name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    error: None,
                });
            }
            Err(e) => {
                // Per token, and then KEEP GOING: the wait belongs to this token
                // alone (see `flood_remaining_for`), so the next token may sign in
                // fine — measured, 2 of 6 did while 4 were waiting. Stopping the
                // loop here is what turned a partial outage into a total one.
                let secs = note_flood_for(&target, &e);
                log(
                    "env bot token failed to boot",
                    &serde_json::json!({"bot_id": target, "error": e, "flood_secs": secs}),
                );
                env_err = Some(format!("bot {}: {}", target, e));
            }
        }
    }

    // An operator that supplies `BOT_TOKENS` has already said which bots this
    // instance may use; never lease a GUEST bot on top of that.
    //
    // On a PaaS this is not a nicety: `storage/` is per-instance and ephemeral, so
    // every restart/scale event would lease another guest bot, and each lease is a
    // fresh `auth.ImportBotAuthorization` — the measured shape of the 55-minute
    // lockout (10 dynos x 7 env bots logging in at boot). A deploy with tokens
    // either signs one of them in or reports why it could not.
    if !env_bot_tokens().is_empty() {
        let env_bots = env_bot_tokens().len();
        log(
            "guest bot lease skipped: BOT_TOKENS supplies the pool",
            &serde_json::json!({"env_bots": env_bots, "pool_error": env_err.clone()}),
        );
        return Err(ProvisionResult::error(match env_err {
            Some(env) => format!(
                "None of this instance's {} configured BOT_TOKENS could sign in, and a BOT_TOKENS deploy never leases a guest bot.\n{}",
                env_bots, env
            ),
            None => format!(
                "This instance is configured from {} BOT_TOKENS, so it does not lease a guest bot.",
                env_bots
            ),
        }));
    }

    // The origin keeps its own pool of bots and hands out the least-used one on
    // every `provision-session` call, so leasing is also how a multi-instance
    // deploy builds a pool without anyone handling tokens — see
    // `ensure_pool_size()`.
    match lease_guest_bot(tg).await {
        Ok(res) => {
            save_session_meta(&res.bot_id, &res.bot_username, &res.bot_name);
            log(
                "guest bot provisioned",
                &serde_json::json!({"bot_id": res.bot_id, "username": res.bot_username}),
            );
            Ok(res)
        }
        Err(e) => Err(ProvisionResult::error(match env_err {
            // Report BOTH sides: the origin lease failure and why the pool bots
            // could not be used (usually a Telegram FLOOD_WAIT).
            Some(env) => format!("{} (pool bots: {})", e, env),
            None => e,
        })),
    }
}

/// One lease from the origin's bot pool: `GET /provision-session`, then log that
/// bot in here. Never touches `session_meta.json` — the caller decides which bot
/// becomes the active one (provisioning takes the first, `ensure_pool_size()`
/// keeps the bot that is already active).
async fn lease_guest_bot(tg: &crate::telegram::TelegramPool) -> Result<ProvisionResult, String> {
    let mut last_error: Option<String> = None;
    // One keypair per install: the origin seals the bot token to it, so a
    // captured response (proxy log, CDN, another install) cannot be reused.
    let (provision_pk, provision_sk) = crate::credentials::provision_keypair();
    // The install's Ed25519 key: the origin mints a token bound to its
    // thumbprint, so a captured token cannot be used off this machine.
    let (sign_pk, _sign_sk) = crate::credentials::sign_keypair();
    let device_id = crate::diagnostics::get_device_id();
    let lease_headers = vec![
        ("X-PM-Pub".to_string(), STANDARD.encode(provision_pk)),
        ("X-PM-Sign-Pub".to_string(), crate::credentials::b64u(sign_pk)),
        ("X-PM-Device".to_string(), device_id.clone()),
    ];

    // Up to three DIFFERENT bots. A refused token retires that bot at the origin
    // (report_failed_lease below), so the next iteration leases another one
    // instead of failing the guest.
    for attempt in 0..3 {
        if attempt > 0 {
            // A failed login already cost the origin a lease; do not hammer it.
            tokio::time::sleep(std::time::Duration::from_millis(250 * attempt as u64)).await;
        }
        let url = format!(
            "{}/provision-session?_nocache={}_{}",
            wp_api_base(),
            now(),
            (rand::random::<u32>() % 9000) + 1000
        );
        // 20s, not 6: the origin measured 10s on the first request after a PHP
        // reload (cold opcache). Timing out there made the client re-lease and
        // burn a provision for a request that was going to succeed.
        let resp = http_json_h(&url, &[], "GET", 20, &lease_headers).await;
        // WordPress answers `"ok": 1` (a NUMBER) here, not `true`; `as_bool()`
        // alone silently rejected every valid provision response.
        let ok = truthy(resp.get("ok"));
        // Prefer the sealed token: when the origin sealed the credential for our
        // key it does not send the plaintext one at all.
        let token = match resp.get("sealed_token").and_then(|v| v.as_str()) {
            Some(sealed) if !sealed.is_empty() => {
                let raw = STANDARD.decode(sealed).unwrap_or_default();
                match crate::credentials::open_sealed_token(&raw, &provision_pk, &provision_sk) {
                    Some(t) => t.trim().to_string(),
                    None => {
                        log(
                            "sealed token did not open",
                            &serde_json::json!({"bot_id": resp.get("bot_id")}),
                        );
                        String::new()
                    }
                }
            }
            _ => resp
                .get("bot_token")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string(),
        };
        if !ok || token.is_empty() {
            // Surface what the origin actually said. This used to be logged only,
            // so every failure reached the player as the useless
            // "Could not obtain guest bot session from server." — including
            // "No eligible guest bot available (<10 subscribers, <20 provisions)"
            // and a Cloudflare/WP error page, which need completely different
            // reactions. `message` is the plugin's own error text.
            let detail = resp
                .get("message")
                .and_then(|m| m.as_str())
                .map(|m| m.trim().to_string())
                .unwrap_or_default();
            last_error = Some(if detail.is_empty() {
                format!(
                    "provision-session returned {}",
                    if ok { "no bot_token" } else { "ok=0" }
                )
            } else {
                format!("provision-session: {}", detail)
            });
            log(
                "provision endpoint returned error or incomplete payload",
                &serde_json::json!({"resp": resp, "attempt": attempt + 1}),
            );
            continue;
        }
        if let Some(secret) = resp.get("api_secret").and_then(|v| v.as_str()) {
            save_api_secret(secret);
        }
        // Kept so a refused login can be reported back against THIS lease.
        let ticket = resp
            .get("ticket")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        let target = match resp.get("bot_id") {
            Some(Value::String(s)) if !s.trim().is_empty() => s.trim().to_string(),
            Some(Value::Number(n)) => n.to_string(),
            _ => format!("prov_{}", random_suffix()),
        };

        // Keep the bot-scoped ticket: it is what authorises /resolve-file(s)
        // now, so the site-wide api_secret never has to be used.
        if let Some(t) = resp.get("ticket").and_then(|v| v.as_str()) {
            crate::credentials::save_provision_ticket(
                t,
                resp.get("expires_at").and_then(|v| v.as_i64()).unwrap_or(0),
                &target,
            );
        }
        // Option A: the install-scoped token (60 min) beats both the ticket and
        // the api_secret — it is bound to our key and revocable on its own.
        if let Some(tok) = resp.get("auth_token").and_then(|v| v.as_str()) {
            crate::credentials::save_auth_token(
                tok,
                resp.get("auth_expires_at").and_then(|v| v.as_i64()).unwrap_or(0),
            );
            log(
                "install token stored",
                &serde_json::json!({
                    "install_id": resp.get("install_id"),
                    "expires_at": resp.get("auth_expires_at"),
                }),
            );
        }

        match tg.boot(&target, Some(&token)).await {
            Ok(entry) => {
                let real_id = if entry.identity.id.is_empty() {
                    target.clone()
                } else {
                    entry.identity.id.clone()
                };
                return Ok(ProvisionResult {
                    bot_id: real_id,
                    bot_username: entry.identity.username.clone(),
                    bot_name: entry.identity.first_name.clone(),
                    error: None,
                });
            }
            Err(e) => {
                // PER TOKEN, and then straight on to the NEXT lease. A FLOOD_WAIT
                // belongs to the one bot Telegram refused, so it must never end the
                // provision — and it must never arm the instance-wide gate, which is
                // what made the following attempt fail before it even started.
                let flood_secs = note_flood_for(&target, &e);
                log(
                    "auto provision login failed",
                    &serde_json::json!({
                        "error": e,
                        "bot_id": target,
                        "attempt": attempt + 1,
                        "flood_secs": flood_secs,
                    }),
                );
                last_error = Some(e.clone());
                if flood_secs > 0 {
                    // Do NOT report this bot as failed at the origin: the wait is
                    // temporary, and retiring it would permanently remove a healthy
                    // bot from every guest's pool. Just lease a different one.
                    log(
                        "leased guest bot is flood-waited; leasing the next bot",
                        &serde_json::json!({
                            "bot_id": target,
                            "remaining_secs": flood_remaining_for(&target),
                            "attempt": attempt + 1,
                        }),
                    );
                    continue;
                }
                if is_clock_or_authkey_error(&e) {
                    return Err(friendly_login_error(&e));
                }
                // Telegram refused this bot's token. Retire it at the origin so
                // the round-robin stops handing it to every other guest, then
                // loop and lease a different bot.
                crate::credentials::clear_provision_ticket();
                // The install token names a bot too; a refused login must not
                // leave it behind, so the next lease mints a fresh one.
                crate::credentials::clear_auth_token();
                report_failed_lease(&target, &ticket, &device_id, &e).await;
                continue;
            }
        }
    }
    Err(last_error
        .unwrap_or_else(|| "Could not obtain guest bot session from server.".to_string()))
}

/// `POST /provision-failed`: quarantine a bot whose token Telegram refused.
///
/// Best effort on purpose — the guest must still get the `last_error` from the
/// caller, so a failure to report is logged and ignored.
async fn report_failed_lease(bot_id: &str, ticket: &str, device_id: &str, reason: &str) {
    if ticket.is_empty() || bot_id.is_empty() {
        // Older origins do not mint tickets; nothing to authenticate the report.
        return;
    }
    let url = format!("{}/provision-failed", wp_api_base());
    let payload = vec![
        ("bot_id".to_string(), bot_id.to_string()),
        ("ticket".to_string(), ticket.to_string()),
        ("device_id".to_string(), device_id.to_string()),
        ("reason".to_string(), reason.chars().take(200).collect::<String>()),
    ];
    let resp = http_json(&url, &payload, "POST", 8).await;
    log(
        "reported failed lease",
        &serde_json::json!({
            "bot_id": bot_id,
            "ok": resp.get("ok").cloned().unwrap_or(serde_json::Value::Null),
            "quarantined_for": resp.get("quarantined_for").cloned().unwrap_or(serde_json::Value::Null),
        }),
    );
}

/// Make sure this instance holds at least [`GUEST_BOTS_PER_INSTANCE`] bot with a
/// usable session, leasing ONE guest bot from the origin when it does not.
///
/// Why per instance and not shared: `bot_pool.json` and `sessions/<bot_id>/` are
/// per-instance disk state, and the download candidate filter skips a pool bot
/// whose session lives on another instance. A pooled fleet therefore aggregates
/// one bot per instance (measured on Heroku: 5 dynos, 14 `/api/bots` reads, 5
/// distinct pools of one bot each), and nobody can multiply that from a config
/// var — see [`GUEST_BOTS_PER_INSTANCE`].
///
/// Returns how many bots the pool holds with a usable session.
pub async fn ensure_guest_bot(tg: &crate::telegram::TelegramPool) -> usize {
    let count = || {
        get_bot_pool()
            .iter()
            .filter(|b| {
                let id = b.get("bot_id").and_then(|v| v.as_str()).unwrap_or("");
                !id.is_empty() && crate::telegram::has_session_file(id)
            })
            .count()
    };
    // A user-supplied bot (env or dashboard) satisfies the requirement, so no
    // guest lease is spent.
    if count() >= GUEST_BOTS_PER_INSTANCE {
        return count();
    }
    // The same lock provisioning uses, so a startup top-up and a
    // request-triggered provision can never log in bots concurrently.
    let _lock = match ProvisionLock::acquire(60_000).await {
        Some(l) => l,
        None => return count(),
    };
    // Re-check after the lock: a concurrent provision may have just satisfied it.
    if count() >= GUEST_BOTS_PER_INSTANCE {
        return count();
    }
    let had_active = !get_bot_id().is_empty();
    match lease_guest_bot(tg).await {
        Ok(res) => {
            add_pool_bot(serde_json::json!({
                "bot_id": res.bot_id,
                "bot_username": res.bot_username,
                "bot_name": res.bot_name,
                "status": "online",
                "is_active": !had_active,
                "updated_at": now(),
            }));
            if !had_active {
                save_session_meta(&res.bot_id, &res.bot_username, &res.bot_name);
            }
            log(
                "pool leased guest bot",
                &serde_json::json!({"bot_id": res.bot_id, "bots": count()}),
            );
        }
        Err(e) => log("pool lease failed", &serde_json::json!({"error": e})),
    }
    count()
}

/// Only used by `/api/provision` to echo the secret; kept here so the route has
/// one import.
pub fn api_secret_for_response() -> String {
    get_api_secret()
}
