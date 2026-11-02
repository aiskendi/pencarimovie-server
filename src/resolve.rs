//! Port of the `resolve` group, backend.php:3397-3967 and 4196-4470:
//! `fd_resolve_cache_store`, `fd_resolve_shortcode_cached`,
//! `fd_save_resolve_cache`, `fd_invalidate_resolve_cache`,
//! `fd_resolve_shortcode_fresh`, `fd_resolve_shortcode_concurrent`,
//! `fd_resolve_shortcode`, `fd_resolve_shortcodes_batch`,
//! `fd_warmup_resolve_batch`.
//!
//! The same 2h expiry (Telegram `file_reference` tokens die), the same 10 s fresh
//! memo, the same "poll 3x with force=1&nocache=1" retry ladder and the same
//! "not found for this request" short-circuit.
//!
//! Storage is [`crate::cache`] (L1 memory + redb), keyed `resolve` / `resolve_fresh`
//! on `short_code:bot_id` — a file id is **bot-specific**, so the bot is part of
//! the key. The old `resolve_cache_<md5>.json` files and the unbounded
//! `Mutex<HashMap>` are gone (the latter never evicted).

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Duration;

use serde_json::Value;

use crate::cache;
use crate::config::wp_api_base;
use crate::httpc::{http_json, http_json_h_status};
use crate::log::log;
use crate::session::{get_bot_id, get_bot_pool, pick_pool_bot};
use crate::storage::now;

/// Freshness window for a resolved `file_id_mt` (the embedded `file_reference`
/// dies on Telegram's side; the download path re-resolves on expiry).
///
/// 2 h was wrong. Measured on 2026-10-24: every Eclipse track logged
/// `FILE_REFERENCE_EXPIRED (400)` on its FIRST byte, then a 7-20 s re-resolve the
/// browser never waited for, so playback died while the origin resolved the same
/// code instantly. A reference lives ~1 h, so the cache has to expire sooner than
/// it does: 50 min keeps a hit usable and still saves the resolve on the next tap.
const RESOLVE_TTL: i64 = 3000;
/// The "one refresh per burst" memo the player's parallel Range requests share.
const FRESH_MEMO_TTL: i64 = 10;
/// How long a FAILED prewarm resolve suppresses a retry for that code.
///
/// Without it every `/stream` re-attempted the same 12 unresolvable codes: the
/// stress run recorded ~4 400 L1 misses and ~1 700 background resolve attempts in
/// one 12 s pass, all for codes the origin had already refused, and the extra
/// origin traffic is exactly what makes a Play slow. 60 s is short enough that a
/// newly indexed file still warms on the next browse.
///
/// ONLY a proven absence (`ok:0` + "File not found") may set this. A code the
/// origin merely QUEUED ("Enqueued for background warmup" / "Deferred (warmup
/// paced)") is not missing — suppressing it for 60 s is how the one row a viewer
/// taps stays cold.
const RESOLVE_MISS_TTL: i64 = 60;

/// How many short codes ONE `/resolve-files` request may carry.
///
/// The origin hard-caps the batch at 50 (`array_slice($short_codes, 0, 50)`,
/// main.php:5553) and SILENTLY DROPS the rest — so this must never exceed it.
/// 50 is also what the origin's own caller uses, and the batch endpoint is cheap
/// by construction (Redis per code + three batched Manticore queries for the
/// whole set), so a bigger payload costs the origin almost nothing extra while
/// saving a round trip.
pub const BATCH_CHUNK: usize = 50;

/// Client-side memo for a finished batch answer.
///
/// Two tabs opening the same title, or a re-open inside a minute, currently
/// re-hit a signed endpoint: every origin call carries a fresh PM-Auth
/// nonce/proof (httpc.rs) and a `fastdownloader_api_record_usage()` write
/// (main.php:5517), so request COUNT is the cost, not payload. 90 s is long
/// enough to collapse a browse burst and short enough that a newly indexed
/// file still appears.
const BATCH_MEMO_TTL: i64 = 90;

/// Memo TTL when the answer was mostly "queued at the origin".
///
/// Measured live: three codes came back `Enqueued for background warmup`, the
/// origin's drain warmed them seconds later, and a 90 s memo kept handing the
/// stale "queued" answer back — so the list never picked up the ids it had just
/// been given. A queued answer is a PROMISE, not a result, so it is memoised only
/// long enough to collapse the burst that produced it.
const BATCH_PENDING_MEMO_TTL: i64 = 15;

/// Hard ceiling on how many codes one prefetch pass may send upstream.
///
/// This is NOT a per-request limit (see [`BATCH_CHUNK`]). It mirrors the ORIGIN'S
/// OWN PENDING CAP — `TGFILE_PREWARM_MAX_PENDING` (core-files.php:5332), which is
/// 40 and keeps the NEWEST codes:
///
/// ```php
/// $pending = array_values(array_unique(array_merge($list, $pending)));
/// if (count($pending) > $max_pending) { $pending = array_slice($pending, 0, $max_pending); }
/// ```
///
/// So anything past the first 40 of a pass is DISCARDED on arrival. Measured
/// before this cap: browsing seven episodes of one series pushed 83+79+67+66+58+
/// 73+68 = 494 codes into a 40-slot queue — ~92 % of the upload discarded, and each
/// episode evicting the previous episode's queue. Sending 40 produces the identical
/// warm set with half the chunks and 43 fewer Redis/Manticore lookups per request.
///
/// 40 is still 3.3× the old `PREWARM_LIMIT` of 12, and it is the most the origin
/// can hold anyway.
pub const PREWARM_HARD_CAP: usize = 40;

fn miss_key(short_code: &str, bot_id: &str) -> String {
    cache_key(short_code, bot_id)
}

fn cache_key(short_code: &str, bot_id: &str) -> String {
    format!("{}:{}", short_code, bot_id)
}

fn has_id(v: &Value) -> bool {
    let a = v.get("file_id_mt").and_then(|x| x.as_str()).unwrap_or("");
    let b = v.get("file_id").and_then(|x| x.as_str()).unwrap_or("");
    !a.is_empty() || !b.is_empty()
}

/// Only a resolved id is worth keeping: an error body must never be cached, or the
/// next request would replay it for two hours.
pub fn resolve_shortcode_cached(short_code: &str, bot_id: &str) -> Option<Value> {
    let key = cache_key(short_code, bot_id);
    let hit = cache::get("resolve", &key)?;
    // Two origin answers carry an id but NOT the catalog metadata:
    //   • `/resolve-files` (batch) — `ok/short_code/file_id_mt/bot_id` only;
    //   • `/resolve-file`'s stale-refresh branch — `refreshed:1` + the id, no
    //     `title`.
    // Cached as a "resolve" they are ids with nothing attached, and every
    // metadata consumer breaks: `/eclipse/stream/<code>` answered
    // `{"error":"Track not found"}` in **4.6 ms** for a code the origin resolves
    // perfectly. A real resolve always knows the size AND the file name, so
    // anything less is a MISS and gets resolved again.
    if has_id(&hit) && file_size_of(&hit) > 0 && !title_of(&hit).is_empty() {
        Some((*hit).clone())
    } else {
        None
    }
}

fn jstr(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").trim().to_string()
}

/// The name a resolve is USABLE by: `title`, else `file_name`.
///
/// A stale-refresh answer (`refreshed:1`) carries the fresh id and the real
/// `file_size`, but its `title` comes from `tg_file_new` and music rows simply do
/// not have one. Requiring `title` alone dropped those ids on the floor: the
/// origin had just minted a working reference, the engine refused to keep it, and
/// the next Play re-resolved a code it had already answered. `file_name` is the
/// same fact under a different key, and both consumers already fall back to it —
/// the stream builder and the eclipse builder in [`crate::routes`] each read
/// `title` and then `file_name`.
fn title_of(v: &Value) -> String {
    let t = jstr(v, "title");
    if !t.is_empty() {
        return t;
    }
    jstr(v, "file_name")
}

fn file_size_of(v: &Value) -> u64 {
    v.get("file_size")
        .and_then(|x| match x {
            Value::Number(n) => n.as_u64().or_else(|| n.as_i64().map(|i| i.max(0) as u64)),
            Value::String(s) => s.trim().parse::<u64>().ok(),
            _ => None,
        })
        .unwrap_or(0)
}

pub fn save_resolve_cache(short_code: &str, bot_id: &str, data: &Value) {
    if !has_id(data) {
        return;
    }
    // The single choke point for every writer, so a metadata-less id can never
    // be persisted as a resolve (see `resolve_shortcode_cached`). The id is
    // still returned to the caller and the origin keeps it in `tg_file_id`, so
    // the next single resolve is instant.
    if file_size_of(data) == 0 || title_of(data).is_empty() {
        log(
            "not caching an id-only answer as a resolve",
            &serde_json::json!({
                "short_code": short_code,
                "bot_id": bot_id,
                "has_size": file_size_of(data) > 0,
                "has_title": !jstr(data, "title").is_empty(),
                "has_file_name": !jstr(data, "file_name").is_empty(),
            }),
        );
        return;
    }
    // Stamp WHEN the id was minted. A Telegram `file_reference` lives ~1 h, and
    // the download path refuses an id that was never PROVEN by a real fetch once
    // it is older than that — otherwise every Play starts with a guaranteed
    // `FILE_REFERENCE_EXPIRED` plus a re-resolve the browser player does not
    // wait for (its `<audio>` element errors after ~20 s).
    let mut stamped = data.clone();
    if let Some(obj) = stamped.as_object_mut() {
        obj.insert("resolved_at".to_string(), serde_json::json!(now()));
    }
    let key = cache_key(short_code, bot_id);
    cache::put("resolve", &key, &stamped, RESOLVE_TTL, 0);
    // A success always clears the negative marker, so a code that resolves later is
    // immediately eligible again.
    cache::invalidate("resolve_miss", &key);
}

pub fn invalidate_resolve_cache(short_code: &str, bot_id: &str) {
    let key = cache_key(short_code, bot_id);
    cache::invalidate("resolve", &key);
    cache::invalidate("resolve_fresh", &key);
}

/// `fd_resolve_shortcode_fresh()`: one refresh per burst, memoised 10 s so the
/// player's other in-flight Range requests reuse it instead of re-resolving.
pub async fn resolve_shortcode_fresh(short_code: &str, bot_id: &str) -> Value {
    let key = cache_key(short_code, bot_id);
    if let Some(v) = cache::get("resolve_fresh", &key) {
        if has_id(&v) {
            log(
                "fresh resolve memo hit - reusing the refresh",
                &serde_json::json!({"short_code": short_code, "bot_id": bot_id}),
            );
            return (*v).clone();
        }
    }
    log(
        "fresh resolve memo miss - resolving with force+nocache",
        &serde_json::json!({"short_code": short_code, "bot_id": bot_id}),
    );
    // `force=1&nocache=1` IS the refresh: the origin drops the ephemeral caches,
    // re-mints the reference with one send and persists it. There is no separate
    // `stale` parameter to send — one knob, one behaviour.
    let res = resolve_shortcode(short_code, bot_id, true).await;
    if has_id(&res) {
        cache::put("resolve_fresh", &key, &res, FRESH_MEMO_TTL, 0);
    }
    res
}

fn not_found_result(v: &Value) -> bool {
    v.get("message")
        .and_then(|m| m.as_str())
        .map(|m| m.to_ascii_lowercase().contains("not found for this request"))
        .unwrap_or(false)
}

/// `fd_resolve_shortcode()`: WordPress `/resolve-file` with the retry ladder.
pub async fn resolve_shortcode(short_code: &str, bot_id: &str, bypass_cache: bool) -> Value {
    resolve_shortcode_status(short_code, bot_id, bypass_cache).await.0
}

/// [`resolve_shortcode()`] that ALSO reports the last origin HTTP status
/// (`0` = never completed, `200` = a cached hit).
///
/// The music route needs "the origin is BUSY" apart from "no such track". The
/// plugin answers `503 Server busy resolving file` with `retry_after: 5` while a
/// Telegram relay runs for that code, and the fast ladder below (1.2/1.5/1.5 s)
/// is entirely inside that window — so a busy origin reached the music client as
/// `404 Track not found` and the track was skipped. Measured live: four 503s in
/// 4.3 s, then `{"error":"Track not found"}`; the same code resolved in one try a
/// minute later.
///
/// Hard-bounded by [`RESOLVE_BUDGET`]: a ladder that runs out of time answers
/// status `0`, which both callers already read as "origin busy" → 503 +
/// `Retry-After`, instead of hanging into the router's own kill.
/// The one body a resolve answers when the origin could not be reached at all.
///
/// `status 0` is what every caller already treats as "origin busy", and
/// `/api/download` turns it into `503 ORIGIN_UNREACHABLE` instead of a 400.
fn origin_unreachable_result() -> (Value, u16) {
    (
        serde_json::json!({
            "ok": 0,
            "error": crate::httpc::ORIGIN_UNREACHABLE,
            "message": crate::httpc::ORIGIN_UNREACHABLE_MSG,
        }),
        0,
    )
}

pub async fn resolve_shortcode_status(
    short_code: &str,
    bot_id: &str,
    bypass_cache: bool,
) -> (Value, u16) {
    match tokio::time::timeout(
        RESOLVE_BUDGET,
        resolve_shortcode_status_inner(short_code, bot_id, bypass_cache),
    )
    .await
    {
        Ok(pair) => pair,
        Err(_) => {
            log(
                "resolve budget exceeded - answering busy instead of hanging",
                &serde_json::json!({
                    "short_code": short_code,
                    "bot_id": bot_id,
                    "budget_secs": RESOLVE_BUDGET.as_secs(),
                    "bypass_cache": bypass_cache,
                }),
            );
            (
                serde_json::json!({
                    "ok": 0,
                    "message": "The origin is busy resolving this file. Retry in a few seconds.",
                }),
                0,
            )
        }
    }
}

/// How long ONE `/resolve-file` ladder may occupy a request.
///
/// Heroku kills a request at 30 s (`H12`, then a 503 with no body — indistinguishable
/// from a dead file). The ladder in [`resolve_shortcode_status_inner`] costs, in the
/// worst case where the origin times out on every rung:
/// `6 + 5 + 6 + (1.2+6) + (1.5+6) + (1.5+6) ≈ 39 s`. One Play can trigger that
/// twice (the foreground resolve plus `/api/download`'s own refresh), which is
/// exactly the steady 3–6 `503` per concurrency level the stress run recorded —
/// including the warm-up, where 2 of 20 URLs died at 30.8 s each. 20 s leaves the
/// router headroom to send the answer.
const RESOLVE_BUDGET: Duration = Duration::from_secs(20);

/// The body of [`resolve_shortcode_status()`]. Deliberately unbounded: the budget
/// lives in the wrapper so there is exactly one place that can end a ladder early.
async fn resolve_shortcode_status_inner(
    short_code: &str,
    bot_id: &str,
    bypass_cache: bool,
) -> (Value, u16) {
    let mut bot = bot_id.trim().to_string();
    if bot.is_empty() {
        bot = get_bot_id();
        if bot.is_empty() {
            return (resolve_shortcode_concurrent(short_code, &[]).await, 0);
        }
    }
    if !bypass_cache {
        if let Some(cached) = resolve_shortcode_cached(short_code, &bot) {
            return (cached, 200);
        }
    }

    // Fail fast while the origin gate is armed.
    //
    // Without this, every Play re-ran the whole 20 s ladder to rediscover the same
    // outage: measured against a dead origin, two consecutive `/api/download` for
    // different codes took **20.0 s each** and the answer arrived after the player
    // had already given up. The cache lookup above still runs first, so a code whose
    // id is already held keeps working (and stays instant) through an outage.
    let down = crate::httpc::wp_origin_down_secs();
    if down > 0 {
        log(
            "resolve skipped - origin down, answering unreachable",
            &serde_json::json!({
                "short_code": short_code,
                "bot_id": bot,
                "for_secs": down,
            }),
        );
        return origin_unreachable_result();
    }

    let url = format!("{}/resolve-file", wp_api_base());
    let mut params = vec![
        ("short_code".to_string(), short_code.to_string()),
        ("bot_id".to_string(), bot.clone()),
    ];
    if bypass_cache {
        params.push(("force".to_string(), "1".to_string()));
        params.push(("nocache".to_string(), "1".to_string()));
    }

    let (mut res, mut status) = http_json_h_status(&url, &params, "GET", 6, &[]).await;
    if has_id(&res) {
        save_resolve_cache(short_code, &bot, &res);
        return (res, status);
    }
    // An unanswered resolve used to leave no trail at all: the caller answered
    // "Track not found" and nothing said why.
    log(
        "resolve returned no file id",
        &serde_json::json!({
            "short_code": short_code,
            "bot_id": bot,
            "status": status,
            "has_size": file_size_of(&res) > 0,
            "message": res.get("message").cloned().unwrap_or(Value::Null),
        }),
    );
    // The origin is not going to answer five seconds from now: the call that just
    // failed ARMED the gate. Everything below — the 5 s backoff and the three
    // 1.2/1.5/1.5 s polls — was pure dead wait with the answer already known,
    // measured at 9.4 s per Play against a dead origin.
    if status == 0 && crate::httpc::wp_origin_down_secs() > 0 {
        return origin_unreachable_result();
    }
    // An origin 5xx (or a request that never completed) is NOT a miss: the relay
    // is still running and the origin advertised `retry_after: 5`. Wait that out
    // once before the fast ladder, or a busy origin looks like a dead track.
    if status >= 500 || status == 0 {
        tokio::time::sleep(Duration::from_secs(5)).await;
        let (r2, s2) = http_json_h_status(&url, &params, "GET", 6, &[]).await;
        res = r2;
        status = s2;
        if has_id(&res) {
            save_resolve_cache(short_code, &bot, &res);
            return (res, status);
        }
    }
    // A refusing origin is worth remembering: the prefetchers must not pile on.
    if note_resolve_rate_limit(&res.to_string()) {
        return (res, status);
    }
    if not_found_result(&res) {
        return (res, status);
    }

    // Telegram background relay was just triggered: poll 3x (1.2s, 1.5s, 1.5s).
    let mut retry = params.clone();
    if !retry.iter().any(|(k, _)| k == "force") {
        retry.push(("force".to_string(), "1".to_string()));
        retry.push(("nocache".to_string(), "1".to_string()));
    }
    for delay in [1200u64, 1500, 1500] {
        tokio::time::sleep(Duration::from_millis(delay)).await;
        let (r, s) = http_json_h_status(&url, &retry, "GET", 6, &[]).await;
        res = r;
        status = s;
        if has_id(&res) {
            save_resolve_cache(short_code, &bot, &res);
            return (res, status);
        }
        if not_found_result(&res) {
            return (res, status);
        }
    }
    (res, status)
}

/// `fd_resolve_shortcode_concurrent()`: the PHP races candidate bots with
/// `curl_multi` and returns the first winner. This port walks the same
/// candidate order serially — identical results, and it avoids N concurrent
/// WordPress calls from one request (which the origin rate-limits).
///
/// Bounded by [`RESOLVE_BUDGET`] exactly like [`resolve_shortcode_status`], and
/// for the same reason: this is the fan-out every caller reaches when it has no
/// bot id, so `N bots × one budgeted attempt` was still an UNBOUNDED request.
/// Measured against an unreachable origin, `/api/resolve-shortcode` hung for
/// **76 s** — far past the router's 30 s kill — because the per-attempt budget
/// bounded each bot and never the walk.
pub async fn resolve_shortcode_concurrent(short_code: &str, candidates: &[String]) -> Value {
    match tokio::time::timeout(
        RESOLVE_BUDGET,
        resolve_shortcode_concurrent_inner(short_code, candidates),
    )
    .await
    {
        Ok(v) => v,
        Err(_) => {
            log(
                "concurrent resolve budget exceeded - answering busy instead of hanging",
                &serde_json::json!({
                    "short_code": short_code,
                    "budget_secs": RESOLVE_BUDGET.as_secs(),
                }),
            );
            serde_json::json!({
                "ok": 0,
                "message": "The origin is busy resolving this file. Retry in a few seconds.",
            })
        }
    }
}

async fn resolve_shortcode_concurrent_inner(short_code: &str, candidates: &[String]) -> Value {
    // The candidate ORDER is a cache decision, not a load-balancing one.
    //
    // A `file_id` is bot-specific and `resolve_shortcode_cached()` is keyed
    // `short_code:bot_id`, so whichever bot answers FIRST is the bot every later
    // request has to ask. This used to start at `pick_pool_bot()`, which
    // round-robins: consecutive resolves of the same code each picked a different
    // bot and each missed the row the previous one had just written. Measured on
    // 2026-10-26 against a fresh cache — one code, three requests, **three
    // different file_ids** in 468/159/13632 ms, while the same code with the
    // active bot pinned answered the IDENTICAL id in 5 ms and 3 ms.
    //
    // So: explicit candidates first (a caller that names bots owns the order),
    // then the ACTIVE bot — what the single-bot path and every `/stream` already
    // use — and only then the pool rotation as fallback.
    let mut bots: Vec<String> = Vec::new();
    for c in candidates {
        let c = c.trim();
        if !c.is_empty() && !bots.iter().any(|b| b == c) {
            bots.push(c.to_string());
        }
    }
    let active = get_bot_id();
    if !active.is_empty() && !bots.iter().any(|b| b == &active) {
        bots.push(active);
    }
    if let Some(picked) = pick_pool_bot() {
        let id = picked.get("bot_id").and_then(|v| v.as_str()).unwrap_or("");
        if !id.is_empty() && !bots.iter().any(|b| b == id) {
            bots.push(id.to_string());
        }
    }
    for b in get_bot_pool() {
        let id = b.get("bot_id").and_then(|v| v.as_str()).unwrap_or("");
        if !id.is_empty() && !bots.iter().any(|b| b == id) {
            bots.push(id.to_string());
        }
    }

    let mut last = Value::Object(serde_json::Map::new());
    for bot in bots {
        // Boxed: `resolve_shortcode_concurrent` is reachable from
        // `resolve_shortcode`, so the future would otherwise be infinitely sized.
        let res = Box::pin(resolve_shortcode(short_code, &bot, false)).await;
        if has_id(&res) {
            return res;
        }
        if not_found_result(&res) {
            return res;
        }
        if res.is_object() && !res.as_object().map(|o| o.is_empty()).unwrap_or(true) {
            last = res;
        }
    }
    last
}

/// A batch row the origin QUEUED rather than resolved.
///
/// These are the origin's own paced prewarm queue talking back: its batch
/// endpoint never relays inline, it stores the code in `tgfile_prewarm_pending_{bot}`
/// (cap 40, newest-first) and lets `tgfile_messageids_drain` do the send at its own
/// rate. Such a code is **not** missing and must never be retried inline.
fn is_pending_row(v: &Value) -> bool {
    let m = v
        .get("message")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    m.contains("enqueued")
        || m.contains("deferred")
        || m.contains("background warmup")
        || v.get("deferred").map(|d| !d.is_null()).unwrap_or(false)
}

/// A row that positively proves the file does not exist.
fn is_absent_row(v: &Value) -> bool {
    v.get("message")
        .and_then(|m| m.as_str())
        .map(|m| m.to_ascii_lowercase().contains("not found"))
        .unwrap_or(false)
}

/// Per-key async locks, so N concurrent callers for the same code set run ONE
/// origin request and the rest await its memo.
fn batch_lock(key: &str) -> Arc<tokio::sync::Mutex<()>> {
    static LOCKS: OnceLock<Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>> = OnceLock::new();
    let map = LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut g = match map.lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    if g.len() > 256 {
        // Drop only the entries nobody else is holding.
        g.retain(|_, v| Arc::strong_count(v) > 1);
    }
    g.entry(key.to_string())
        .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
        .clone()
}

/// Stable memo key for a code set: ORDER-INDEPENDENT, so the same list built in a
/// different order reuses one answer.
fn batch_memo_key(codes: &[String], bot_id: &str) -> String {
    let mut sorted: Vec<&str> = codes.iter().map(|s| s.as_str()).collect();
    sorted.sort_unstable();
    sorted.dedup();
    format!("{}|{}", bot_id, sorted.join(","))
}

/// Split a code list into `/resolve-files` payloads the origin will not truncate.
fn chunk_codes(codes: Vec<String>) -> Vec<Vec<String>> {
    if codes.is_empty() {
        return Vec::new();
    }
    codes.chunks(BATCH_CHUNK).map(|c| c.to_vec()).collect()
}

/// `fd_resolve_shortcodes_batch()`: the WordPress BATCH endpoint, one round trip
/// per [`BATCH_CHUNK`].
///
/// This is the CHEAP endpoint and it must stay the only one a prefetch uses. The
/// origin answers it from Redis plus three batched Manticore queries and never
/// relays inline (main.php:5515). `/resolve-file` is a different animal: it takes
/// a 15 s `tgfile_relay_lock_{bot}_{code}` and runs a real bot-to-bot media relay
/// (core-files.php:3685), so a per-code fallback here converts one cheap request
/// into N expensive ones — measured on a phone as a 285-code warm turning into 285
/// consecutive HTTP 503s that starved the user's own Play.
///
/// `allow_singles` therefore defaults OFF for every prefetch path. It exists for
/// the interactive refresh during playback, where a human is waiting on the first
/// byte and there is no later chance to fill the cache.
pub async fn resolve_shortcodes_batch(
    short_codes: &[String],
    bot_id: &str,
    timeout: u64,
) -> HashMap<String, Value> {
    resolve_shortcodes_batch_ext(short_codes, bot_id, timeout, true).await
}

/// [`resolve_shortcodes_batch()`] that never falls back to `/resolve-file`.
pub async fn resolve_shortcodes_batch_prefetch(
    short_codes: &[String],
    bot_id: &str,
    timeout: u64,
) -> HashMap<String, Value> {
    resolve_shortcodes_batch_ext(short_codes, bot_id, timeout, false).await
}

pub async fn resolve_shortcodes_batch_ext(
    short_codes: &[String],
    bot_id: &str,
    timeout: u64,
    allow_singles: bool,
) -> HashMap<String, Value> {
    let mut out: HashMap<String, Value> = HashMap::new();
    let mut seen: HashSet<String> = HashSet::new();
    let codes: Vec<String> = short_codes
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && seen.insert(s.clone()))
        .collect();
    if codes.is_empty() {
        return out;
    }
    let effective = if bot_id.trim().is_empty() {
        get_bot_id()
    } else {
        bot_id.trim().to_string()
    };

    // ── Request coalescing + client-side memo ────────────────────────────────
    // The memo key is order-independent, so two callers that walk the same list
    // differently still share one origin request. The lock makes the SECOND
    // caller wait for the first instead of firing its own — the standard
    // single-flight arrangement, and the only thing that stops a browse burst
    // (several tabs, or one tab re-opened) from multiplying the request count.
    let memo_key = batch_memo_key(&codes, &effective);
    if let Some(hit) = cache::get("resolve_batch", &memo_key) {
        if let Some(obj) = hit.get("results").and_then(|v| v.as_object()) {
            log(
                "resolve batch memo hit - no origin request",
                &serde_json::json!({"codes": codes.len(), "bot_id": effective}),
            );
            return obj
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<String, Value>>();
        }
    }
    let gate = batch_lock(&memo_key);
    let _held = gate.lock().await;
    // Re-check under the lock: the caller we queued behind may have just filled it.
    if let Some(hit) = cache::get("resolve_batch", &memo_key) {
        if let Some(obj) = hit.get("results").and_then(|v| v.as_object()) {
            log(
                "resolve batch memo hit after coalescing",
                &serde_json::json!({"codes": codes.len(), "bot_id": effective}),
            );
            return obj
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<HashMap<String, Value>>();
        }
    }

    let chunks = chunk_codes(codes.clone());
    let mut origin_calls = 0usize;
    for chunk in &chunks {
        let url = format!("{}/resolve-files", wp_api_base());
        // NO `_t` cache-buster: PM-Auth already proves freshness, and a unique
        // URL per call only defeats the origin's own caching.
        let params = vec![
            ("short_codes".to_string(), chunk.join(",")),
            ("bot_id".to_string(), effective.clone()),
        ];
        let res = http_json(&url, &params, "GET", timeout).await;
        origin_calls += 1;
        // A batch answer can be the origin's way of saying "slow down". Learn it
        // here as well, or a batch-only design has no feedback signal at all
        // (this used to be reachable only from the singular ladder).
        note_resolve_rate_limit(&res.to_string());
        if let Some(results) = res.get("results").and_then(|v| v.as_object()) {
            for code in chunk {
                let Some(row) = results.get(code) else { continue };
                if !row.is_object() {
                    continue;
                }
                out.insert(code.clone(), row.clone());
                if has_id(row) && !effective.is_empty() {
                    // The batch row now carries the SAME shape as a single resolve
                    // — title, mime, extension, file_size, file_type, performer,
                    // duration (main.php:5820) — so persisting it is what makes
                    // `cached_duration()`/`cached_part()` work on the next list
                    // open without a second round trip.
                    let mut merged = resolve_shortcode_cached(code, &effective)
                        .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
                    if let (Some(dst), Some(src)) = (merged.as_object_mut(), row.as_object()) {
                        for (k, v) in src {
                            dst.insert(k.clone(), v.clone());
                        }
                    }
                    save_resolve_cache(code, &effective, &merged);
                }
            }
        }
    }

    if allow_singles {
        for code in &codes {
            // A refusing origin must STOP the per-code fallback, not walk it.
            if resolve_cooldown_remaining_sec() > 0 {
                log(
                    "resolve batch singles stopped (origin busy)",
                    &serde_json::json!({"remaining_codes": codes.len(), "bot_id": effective}),
                );
                break;
            }
            let Some(row) = out.get(code) else {
                // The origin did not answer for this code at all (truncated list,
                // dropped chunk) — that is not evidence of absence, but it is also
                // not something the batch can fix. Leave it for the Play.
                continue;
            };
            if has_id(row) {
                continue;
            }
            // QUEUED is not MISSING. Asking `/resolve-file` here is exactly the
            // storm this function is built to avoid: the origin already has the
            // code in its paced queue, and a relay now would fight that drain for
            // the same per-code lock.
            if is_pending_row(row) || is_absent_row(row) {
                continue;
            }
            let single = resolve_shortcode(code, &effective, false).await;
            out.insert(code.clone(), single);
        }
    } else {
        let queued = codes
            .iter()
            .filter(|c| out.get(*c).map(is_pending_row).unwrap_or(false))
            .count();
        if queued > 0 {
            log(
                "resolve batch queued at origin (no relay fired)",
                &serde_json::json!({
                    "queued": queued, "codes": codes.len(),
                    "chunks": chunks.len(), "bot_id": effective,
                }),
            );
        }
    }

    if !out.is_empty() {
        // A purely-queued answer must not be memoised as if it were a result: the
        // origin's own drain fills these within seconds, and a long memo would keep
        // replaying "queued" long after the ids exist.
        let all_queued = out
            .values()
            .all(|r| !has_id(r) && is_pending_row(r));
        let ttl = if all_queued {
            BATCH_PENDING_MEMO_TTL
        } else {
            BATCH_MEMO_TTL
        };
        cache::put(
            "resolve_batch",
            &memo_key,
            &serde_json::json!({ "results": out }),
            ttl,
            0,
        );
    }
    log(
        "resolve batch done",
        &serde_json::json!({
            "codes": codes.len(), "origin_calls": origin_calls,
            "allow_singles": allow_singles, "bot_id": effective,
        }),
    );
    out
}

/// How long the origin is refusing resolves for, in seconds (0 = clear).
///
/// The TDLib app keeps `resolveCooldownUntilMs` for exactly this reason: opening a
/// catalog page fires 24 prefetches at once, and before this gate the device logged
/// `[Resolve] upstream rate limit — pausing resolves for 5s` AND `[PostFiles] page 0
/// failed: timeout` **in the same second** — the prefetches were the load that
/// tripped the limiter. Prefetchers consult this before adding work.
pub fn resolve_cooldown_remaining_sec() -> i64 {
    // Was `resolve_cooldown.marker` (mtime + a number in the file = two syscalls on
    // every prewarm decision). The deadline is now the value itself.
    let Some(v) = cache::get("cooldown", "resolve") else {
        return 0;
    };
    let until = v.get("until").and_then(|x| x.as_i64()).unwrap_or(0);
    (until - crate::storage::now()).max(0)
}

fn note_resolve_rate_limit(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    let hit = lower.contains("rate limit")
        || lower.contains("flood")
        || lower.contains("too many requests")
        || lower.contains("retry after")
        || lower.contains("server busy");
    if hit {
        // 15 min cap: the resolver has been measured sending `retry_after: 137`.
        let secs = 5i64.max(15);
        cache::put(
            "cooldown",
            "resolve",
            &serde_json::json!({"until": crate::storage::now() + secs}),
            secs,
            0,
        );
        log(
            "resolve rate limited - pausing prefetches",
            &serde_json::json!({"seconds": secs, "message": msg}),
        );
    }
    hit
}

/// `fd_prewarm_streams_batch()` (backend.php:3701) and the Kotlin twin.
///
/// A stream list is built from file METADATA only; the Bot API `file_id` is
/// resolved later, when the player asks for bytes — so the first Play paid a
/// 1-3 s resolve. This warms every short code of the list that is not already
/// cached, in the BACKGROUND, so Play on a title you just browsed is instant.
/// Never blocks the response and never fails it: a resolve error is only logged.
///
/// Returns how many codes were sent to the background (0 = all already warm).
pub fn prewarm_streams_batch(codes: Vec<String>, bot_id: &str, context: &str) -> usize {
    prewarm_streams_batch_limited(codes, bot_id, context, 12)
}

/// Formerly "how many codes one prewarm may resolve" — now historical.
///
/// The measurement behind it is still true: warming a 285-code movie list turned
/// into 285 separate `/resolve-file` calls, every one **HTTP 503** ("server busy
/// resolving file"), starving the user's own Play (11.7 s to first byte). But the
/// conclusion drawn from it was wrong. The 503 storm came from the **per-code
/// `/resolve-file` fallback**, not from asking about many codes — and capping the
/// pass at 12 only hid that while leaving every other code permanently cold.
///
/// Both halves are fixed properly now: the fallback is gone from every prefetch
/// path ([`resolve_shortcodes_batch_prefetch`]), and the payload bound is
/// [`BATCH_CHUNK`] (50, the origin's own hard cap). Kept as a named constant only
/// so existing callers keep compiling; it no longer truncates anything.
pub const PREWARM_LIMIT: usize = 12;

pub fn prewarm_streams_batch_limited(
    codes: Vec<String>,
    bot_id: &str,
    context: &str,
    limit: usize,
) -> usize {
    let effective = if bot_id.trim().is_empty() {
        get_bot_id()
    } else {
        bot_id.trim().to_string()
    };
    if effective.is_empty() {
        return 0;
    }
    let mut missing: Vec<String> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for c in codes {
        let c = c.trim().to_string();
        if c.is_empty() || !seen.insert(c.clone()) {
            continue;
        }
        match resolve_shortcode_cached(&c, &effective) {
            Some(v) if has_id(&v) => {}
            _ => {
                // A code the origin just refused is not worth another attempt yet.
                if cache::get("resolve_miss", &miss_key(&c, &effective)).is_some() {
                    continue;
                }
                missing.push(c)
            }
        }
    }
    if missing.is_empty() {
        return 0;
    }
    // The `limit` argument no longer truncates the code set.
    //
    // It used to cap the PASS, which was protecting the wrong thing: the origin's
    // batch endpoint is Redis-only (main.php:5555) and paces its own enqueue out
    // of the request (10 codes per call, chunks of 5, 60 ms apart, cap 40 pending
    // per bot). Truncating to 12 meant a 285-row list seeded at most 12 codes into
    // that queue and left 273 permanently cold, while the pass itself was what
    // fell through to the expensive `/resolve-file` relay. The per-REQUEST bound
    // that actually matters lives in `chunk_codes()` ([`BATCH_CHUNK`] = 50, the
    // origin's own hard cap); here the whole pass is bounded by
    // [`PREWARM_HARD_CAP`], and `limit` is only honoured as that soft ceiling.
    let _ = limit;
    let before = missing.len();
    if missing.len() > PREWARM_HARD_CAP {
        log(
            "prewarm capped to the first codes of a very large list",
            &serde_json::json!({
                "context": context, "missing": before, "cap": PREWARM_HARD_CAP,
            }),
        );
        missing.truncate(PREWARM_HARD_CAP);
    }
    if resolve_cooldown_remaining_sec() > 0 {
        log(
            "prewarm skipped (resolve cooldown)",
            &serde_json::json!({"context": context, "missing": before}),
        );
        return 0;
    }
    // Never compete with live playback: a resolve in flight holds the same origin
    // and Telegram budget the stream needs.
    if crate::download::active_streams() > 0 {
        log(
            "prewarm skipped (stream active)",
            &serde_json::json!({"context": context, "missing": before}),
        );
        return 0;
    }
    let n = missing.len();
    let ctx = context.to_string();
    log(
        "prewarm streams batch",
        &serde_json::json!({"context": ctx, "missing": n, "bot_id": effective}),
    );
    // Detached: the list goes back to the player now. `warmup_resolve_batch`
    // persists each hit, so the NEXT Play skips the resolve entirely.
    //
    // This path NEVER falls back to `/resolve-file`: the origin's batch endpoint
    // already parked every code it could not answer in its own paced queue, and a
    // relay here would contend with that drain for the same `tgfile_relay_lock`.
    tokio::spawn(async move {
        let res = warmup_resolve_batch(&missing, &effective).await;
        // ONLY a proven absence suppresses a retry. A code the origin merely
        // QUEUED ("Deferred (warmup paced)" / "Enqueued for background warmup") is
        // in flight and must stay retryable — suppressing it for the miss TTL is
        // how the one row a viewer taps stayed cold.
        for code in &res.failed_codes {
            cache::put(
                "resolve_miss",
                &miss_key(code, &effective),
                &serde_json::json!(1),
                RESOLVE_MISS_TTL,
                0,
            );
        }
        // `suppressed_for_secs` is only meaningful when something was actually
        // suppressed. Reporting it unconditionally made a wholly-queued pass read
        // as "5 codes suppressed for 60 s" when in fact ZERO were — the exact
        // misreading that hid the queued-vs-missing bug in the first place.
        let mut ctx = serde_json::json!({
            "context": ctx,
            "resolved": res.resolved,
            "queued": res.pending,
            "failed": res.failed,
        });
        if res.failed > 0 {
            ctx["suppressed_for_secs"] = serde_json::json!(RESOLVE_MISS_TTL);
        }
        log("prewarm streams batch finished", &ctx);
    });
    n
}

pub struct WarmupResult {
    pub total: usize,
    pub resolved: usize,
    /// Codes the origin QUEUED into its own paced prewarm drain.
    ///
    /// Kept apart from [`Self::failed`] on purpose: a queued code will be warm in
    /// seconds without us doing anything, and treating it as a miss (which the
    /// caller turns into a `resolve_miss` suppression) is what kept the row a
    /// viewer was about to tap cold.
    pub pending: usize,
    pub failed: usize,
    pub failed_codes: Vec<String>,
    pub pending_codes: Vec<String>,
    pub results: HashMap<String, Value>,
    pub bot_id: String,
}

/// `fd_warmup_resolve_batch()`: resolve and PERSIST so the next Play is instant.
///
/// Prefetch ONLY — it never calls `/resolve-file`. Anything the batch endpoint
/// could not answer is already in the origin's queue by the time this returns.
pub async fn warmup_resolve_batch(short_codes: &[String], bot_id: &str) -> WarmupResult {
    let mut seen: HashSet<String> = HashSet::new();
    let unique: Vec<String> = short_codes
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && seen.insert(s.clone()))
        .collect();
    let effective = if bot_id.trim().is_empty() {
        get_bot_id()
    } else {
        bot_id.trim().to_string()
    };
    let results = resolve_shortcodes_batch_prefetch(&unique, &effective, 60).await;
    let mut resolved = 0usize;
    let mut failed_codes: Vec<String> = Vec::new();
    let mut pending_codes: Vec<String> = Vec::new();
    for code in &unique {
        match results.get(code) {
            Some(row) if has_id(row) => resolved += 1,
            Some(row) if is_pending_row(row) => pending_codes.push(code.clone()),
            // No row at all = the origin did not speak about this code (dropped
            // chunk / truncated list). That is "unknown", not "absent" — do not
            // suppress it, the next pass will ask again.
            None => pending_codes.push(code.clone()),
            _ => failed_codes.push(code.clone()),
        }
    }
    log(
        "warmup resolve batch completed",
        &serde_json::json!({
            "total": unique.len(),
            "resolved": resolved,
            "queued": pending_codes.len(),
            "failed": failed_codes.len(),
            "bot_id": effective
        }),
    );
    WarmupResult {
        total: unique.len(),
        resolved,
        pending: pending_codes.len(),
        failed: failed_codes.len(),
        failed_codes,
        pending_codes,
        results,
        bot_id: effective,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The real `/resolve-files` row shape (measured against the origin): an id
    /// and nothing else. Caching it as a resolve is what made `/eclipse/stream`
    /// answer "Track not found" for a code that resolves fine.
    #[test]
    fn a_batch_row_is_never_cached_as_a_resolve() {
        let batch = serde_json::json!({
            "ok": 1, "short_code": "batch1", "file_id_mt": "AQADbatch", "bot_id": 8469916399i64
        });
        save_resolve_cache("batch1", "test-bot", &batch);
        assert!(resolve_shortcode_cached("batch1", "test-bot").is_none());

        let full = serde_json::json!({
            "ok": 1, "file_id_mt": "AQADbatch", "file_size": 18552780,
            "title": "02.Billie.Eilish.LUNCH.flac", "mime": "audio/x-flac"
        });
        save_resolve_cache("batch1", "test-bot", &full);
        let hit = resolve_shortcode_cached("batch1", "test-bot").expect("a full row is usable");
        assert_eq!(hit["title"], "02.Billie.Eilish.LUNCH.flac");
    }

    /// Entries already on disk from an older build must read as a MISS, not as a
    /// metadata-less hit, or the 2 h redb copy keeps breaking playback after the
    /// upgrade.
    #[test]
    fn a_stored_entry_without_a_size_reads_as_a_miss() {
        let key = cache_key("poison1", "test-bot");
        cache::put(
            "resolve",
            &key,
            &serde_json::json!({"ok": 1, "file_id_mt": "AQADpoison"}),
            60,
            0,
        );
        assert!(cache::get("resolve", &key).is_some(), "the raw entry is there");
        assert!(resolve_shortcode_cached("poison1", "test-bot").is_none());
    }

    /// A stale-refresh answer carries the fresh id, a real `file_size` and a
    /// `file_name`, but no catalog `title` (music rows have none). Requiring
    /// `title` alone dropped the id the origin had just minted, so the next Play
    /// re-resolved a code that was already answered.
    #[test]
    fn a_refresh_row_with_a_file_name_is_usable_without_a_title() {
        let refreshed = serde_json::json!({
            "ok": 1, "file_id_mt": "AQADrefreshed", "refreshed": 1,
            "file_size": 4372611, "file_name": "03.SZA.Kill.Bill.flac",
            "mime": "audio/x-flac"
        });
        save_resolve_cache("refresh1", "test-bot", &refreshed);
        let hit = resolve_shortcode_cached("refresh1", "test-bot")
            .expect("an id with a size and a file name is a usable resolve");
        assert_eq!(hit["file_name"], "03.SZA.Kill.Bill.flac");
    }

    /// The gate is `a size AND a name`. An id with neither is still a MISS, or
    /// `/eclipse/stream` answers "Track not found" in 4.6 ms again
    /// (see `a_batch_row_is_never_cached_as_a_resolve`).
    #[test]
    fn an_id_without_a_size_or_a_name_is_still_refused() {
        let id_only = serde_json::json!({
            "ok": 1, "file_id_mt": "AQADnoname", "file_size": 1234, "title": ""
        });
        save_resolve_cache("noname1", "test-bot", &id_only);
        assert!(resolve_shortcode_cached("noname1", "test-bot").is_none());
    }

    /// The origin NEVER relays inside `/resolve-files` — it parks the code in its
    /// own paced queue and says so. Both of those wordings must read as "queued",
    /// because the alternative is a `/resolve-file` relay that fights the drain for
    /// the same per-code lock and answers 503 to the user's own Play.
    #[test]
    fn a_queued_batch_row_is_pending_and_never_a_miss() {
        for message in [
            "Enqueued for background warmup",
            "Deferred (warmup paced)",
        ] {
            let row = serde_json::json!({"ok": 0, "short_code": "q1", "message": message});
            assert!(is_pending_row(&row), "{message} must read as queued");
            assert!(!is_absent_row(&row), "{message} must never read as absent");
        }
    }

    /// Only a proven absence may suppress a retry.
    #[test]
    fn only_a_not_found_row_counts_as_absent() {
        let absent = serde_json::json!({"ok": 0, "short_code": "n1", "message": "File not found"});
        assert!(is_absent_row(&absent));
        assert!(!is_pending_row(&absent));

        let resolved = serde_json::json!({
            "ok": 1, "short_code": "n2", "file_id_mt": "AQADx", "bot_id": 1
        });
        assert!(!is_absent_row(&resolved));
        assert!(!is_pending_row(&resolved));
    }

    /// The origin silently drops everything past 50 codes
    /// (`array_slice($short_codes, 0, 50)`, main.php:5553), so a chunk must never
    /// exceed that — a bigger one would look answered while half the list vanished.
    #[test]
    fn a_batch_chunk_never_exceeds_the_origin_cap() {
        assert_eq!(BATCH_CHUNK, 50);
        let codes: Vec<String> = (0..285).map(|i| format!("c{i}")).collect();
        let chunks = chunk_codes(codes.clone());
        assert_eq!(chunks.len(), 6, "285 codes need 6 requests at 50 each");
        assert!(chunks.iter().all(|c| c.len() <= BATCH_CHUNK));
        let total: usize = chunks.iter().map(|c| c.len()).sum();
        assert_eq!(total, codes.len(), "no code may be dropped");
    }

    /// Order must not change the memo key, or two callers walking the same list
    /// differently each pay for their own origin request.
    #[test]
    fn the_batch_memo_key_ignores_order_and_duplicates() {
        let a = batch_memo_key(
            &["b".to_string(), "a".to_string(), "c".to_string()],
            "bot1",
        );
        let b = batch_memo_key(
            &["c".to_string(), "a".to_string(), "b".to_string(), "a".to_string()],
            "bot1",
        );
        assert_eq!(a, b);
        // A different bot is a different answer: file ids are bot-specific.
        assert_ne!(a, batch_memo_key(&["a".to_string(), "b".to_string(), "c".to_string()], "bot2"));
    }
}
