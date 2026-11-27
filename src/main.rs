//! HTTP entry point. `axum` + `tokio`, `127.0.0.1:8088` by default.
//!
//! ```text
//!   pencarimovie-server --port 8088 [--host 127.0.0.1] [--workers N]
//!                     [--inflight 8] [--stream-prefetch 20] [--max-streams 256]
//!                     [--public] [--tray]
//!   pencarimovie-server --decode <file_id_mt> [expected_id]
//! ```
//!
//! `--tray` puts the icon in the system tray and runs the server behind it, in
//! this same process (`tray.rs` on Windows, `tray_unix.rs` on Linux). It also
//! hides its own console, which is what lets the launchers drop
//! `powershell -WindowStyle Hidden`. `--no-tray` forces it off.
//!
//! `--inflight N` (default 8) caps `upload.getFile` parts in flight **per
//! connection**: each `Client` in a `BotEntry` owns its own semaphore, so
//! `--workers 8 --inflight 8` is 64 parts across 8 connections. `--max-streams`
//! (default 256) caps concurrent `/api/download` bodies; excess viewers queue for
//! 5 s and then get 503 + `Retry-After`.
//!
//! `--workers N` creates N independent grammers `Client`s (each with its own
//! `SenderPool`, therefore its own connection set) and round-robins requests
//! over them. That is the only way to get more than one connection per DC with
//! grammers; see README.md for the measured difference between
//! "1 client x 4 tasks" and "4 clients x 1 task".

mod assets;
mod auth;
mod cache;
mod config;
mod credentials;
mod backfill;
mod diagnostics;
mod download;
mod music;
mod music_addon;
mod fileid;
mod httpc;
mod lanes;
mod log;
mod matcher;
mod media_probe;
mod mint;
mod music_upstreams;
mod proc;
mod public_config;
mod ratelimit;
mod resolve;
mod respond;
mod routes;
mod session;
mod split_index;
mod split_remux;
mod storage;
mod telegram;
mod thumb;
// The tray lives in the binary (`--tray`), so a folder holding only the
// executable still gets an icon: no `tray.ps1`, no PowerShell, no loose script.
// `tray.rs` is the native Windows icon; `tray_unix.rs` drives `yad` on Linux.
// Android has no notification area, so it gets neither.
#[cfg(windows)]
mod tray;
#[cfg(all(unix, not(target_os = "android")))]
mod tray_unix;
#[cfg(windows)]
use crate::tray as tray_impl;

#[cfg(all(unix, not(target_os = "android")))]
use crate::tray_unix as tray_impl;

mod tunnel;
mod update;
mod upstream;
mod wp_data;
mod zip_split;

use std::io::IsTerminal;

use std::net::SocketAddr;

use std::process::Stdio;

use std::sync::Arc;

use axum::Router;

use axum::extract::Request;

use axum::response::Response;

use tokio::net::TcpListener;

/// Parts per connection when nothing else says otherwise. One number for the server
/// and every CLI probe: the probes used to hardcode 8, so a diagnostic run measured a
/// shape no user install had (2026-11-25, seen as `inflight_per_connection: 8` in the
/// boot line of a mint probe while the server ran with 3).
pub const DEFAULT_INFLIGHT: usize = 2;

/// Telegram connections (sockets) per bot, for every platform.
///
/// Throughput is meant to come from MORE sockets with FEW parts each: a media
/// socket asked for too many parts at once is dropped by the server without an
/// error, which is the `part stalled: no part in 12001ms` stall (16 parts x 3 and
/// `danog/MadelineProto#1948` settles on 2 per socket, opening more instead).
/// Measured on the LAN against a 2160p ZIP entry: 16 x 3 serves the same ~0.5 MB/s
/// that 4 x 3 did, with zero stalls, so raising the socket count is free.
/// 10, not 16: the per-bot part budget this feeds (`workers x inflight`) is also
/// the ceiling the learned window may grow to, and MEASURED 2026-11-27 on the
/// VPS that ceiling sets a bot's rate — 16 x 2 = 32 per bot over-subscribed the
/// DC (parts stalled, p50 7.4 s, bodies 503) while the single-response peak came
/// from ~16-20 parts per bot. 10 x 2 = 20 sits on the measured knee; `--workers`
/// still overrides it, and `--lane-workers` does the same per api_id lane.
pub const DEFAULT_WORKERS: usize = 10;

pub struct AppState {
    pub port: u16,
    pub workers: usize,
    /// Max `upload.getFile` parts in flight per grammers `Client` connection.
    pub inflight: usize,
    /// Global cap on concurrent `/api/download` responses (`--max-streams`).
    pub download_gate: Arc<tokio::sync::Semaphore>,
    pub max_streams: usize,
    /// Per-client abuse limits (`ratelimit`). In-memory, so per process â€” the same
    /// caveat the resolve and catalog caches carry.
    pub limits: Arc<ratelimit::Limiter>,
    /// Distinct short codes one client may have in flight (`0` = off).
    pub max_titles_per_ip: usize,
    /// Concurrent bodies one client may hold, a socket-flood backstop (`0` = off).
    pub max_bodies_per_ip: usize,
    /// Cold starts one client may spend per `cold_start_window` (`0` = off).
    pub cold_starts_per_ip: usize,
    /// The cold-start window, in seconds.
    pub cold_start_window: u64,
    /// `Arc` so `telegram::share_pool()` can publish the SAME pool (and therefore
    /// the same bot sessions) to `mint.rs`, which is reached from the resolve
    /// layer where no `AppState` exists.
    pub telegram: Arc<telegram::TelegramPool>,
}

fn arg_value(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let i = args.iter().position(|a| a == flag)?;
    args.get(i + 1).cloned()
}

fn arg_pos(flag: &str) -> Option<usize> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag)
}

/// `0` means the counter is off, and the banner says so instead of printing a
/// misleading `0`.
fn cap_or_off(n: usize) -> String {
    if n == 0 {
        "unlimited".to_string()
    } else {
        n.to_string()
    }
}

/// One `AppState` per process; the router has a single fallback, so the state is
/// stored here instead of behind an extractor (a plain `Arc<AppState>` argument
/// is not an extractor, and `State<..>` would force a generic router).
static STATE: std::sync::OnceLock<Arc<AppState>> = std::sync::OnceLock::new();

#[allow(dead_code)]
fn _assert_future_send() {
    fn need<T: Send>(_: T) {}
    let req = Request::builder().body(axum::body::Body::empty()).unwrap();
    need(serve_all(req));
}

async fn serve_all(req: Request) -> Response {
    let state = STATE
        .get()
        .expect("AppState must be initialised before serving")
        .clone();
    routes::handle(state, req).await
}

/// Wakes `shutdown_signal()`. The tray's "Stop server" item is the second way to
/// ask for a graceful stop (Ctrl-C is the first); a `Notify` carries a permit, so
/// a click that arrives before the server starts awaiting is not lost.
static SHUTDOWN: std::sync::OnceLock<tokio::sync::Notify> = std::sync::OnceLock::new();

fn shutdown_trigger() -> &'static tokio::sync::Notify {
    SHUTDOWN.get_or_init(tokio::sync::Notify::new)
}

/// Ask the process to stop serving (called from the tray thread).
#[cfg(windows)]
pub(crate) fn request_shutdown() {
    shutdown_trigger().notify_one();
}

/// `--thumb-probe`: what those bytes actually are. A byte count alone cannot
/// tell a 4 KB JPEG thumbnail from a 4 KB slice of the media, and a "success"
/// that returns the wrong object is worse than an error.
fn bytes_kind(b: &[u8]) -> &'static str {
    if b.is_empty() {
        "empty"
    } else if b.len() >= 3 && b[0] == 0xFF && b[1] == 0xD8 && b[2] == 0xFF {
        "jpeg"
    } else if b.starts_with(&[0x89, 0x50, 0x4E, 0x47]) {
        "png"
    } else if b.len() >= 12 && &b[0..4] == b"RIFF".as_slice() && &b[8..12] == b"WEBP".as_slice() {
        "webp"
    } else {
        "unknown"
    }
}

/// Width x height from a JPEG's first SOF marker (`None` = not a JPEG, or no
/// marker in the bytes we fetched). Thumbnail sizes are chosen on these numbers,
/// so the probe reports them instead of "some bytes arrived".
fn jpeg_dims(b: &[u8]) -> Option<(u32, u32)> {
    if bytes_kind(b) != "jpeg" {
        return None;
    }
    let mut i = 2usize;
    while i + 9 < b.len() {
        if b[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = b[i + 1];
        // SOF0..SOF15, minus DHT (C4), JPG (C8) and DAC (CC), which share the range.
        if (0xC0..=0xCF).contains(&marker) && marker != 0xC4 && marker != 0xC8 && marker != 0xCC {
            let h = u16::from_be_bytes([b[i + 5], b[i + 6]]) as u32;
            let w = u16::from_be_bytes([b[i + 7], b[i + 8]]) as u32;
            return Some((w, h));
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        if len < 2 {
            return None;
        }
        i += 2 + len;
    }
    None
}

/// `--thumb-probe` read: one `upload.getFile`, and on `FILE_REFERENCE_EXPIRED`
/// exactly one re-resolve + retry â€” the recovery `/api/download` performs
/// (`resolve_shortcode_fresh` = `force=1&nocache=1`).
///
/// Without it the probe reports the resolve CACHE's expired reference as if it
/// were the answer: the first run against a short code failed all seven reads
/// with `FILE_REFERENCE_EXPIRED` while the same code streams fine through the
/// server, because the cached id was older than its ~1 h `file_reference`.
/// Both a media document and a stored `thumb_file_id` (type 0) are read through
/// `inputDocumentFileLocation`: the difference is only which object the id names.
/// Only a media read with `thumb_size` empty is a whole-file read.
fn probe_target(file_id: &str, file_size: u64, thumb_size: &str) -> Result<telegram::DocTarget, String> {
    if thumb_size.is_empty() {
        telegram::prepare_file_target(file_id, file_size)
    } else {
        telegram::prepare_thumb_target(file_id, thumb_size)
    }
}

async fn probe_fetch(
    client: &grammers_client::Client,
    file_id: &mut String,
    file_size: &mut u64,
    thumb_size: &str,
    short_code: &str,
    bot: &str,
) -> Result<Vec<u8>, String> {
    for attempt in 0..2 {
        let target = probe_target(file_id.as_str(), *file_size, thumb_size);
        let target = target?;
        match telegram::fetch_range(client, &target, 0, telegram::MAX_CHUNK).await {
            Ok(b) => return Ok(b),
            Err(e) => {
                if e.file_reference && attempt == 0 && !short_code.is_empty() {
                    let res = resolve::resolve_shortcode_fresh(short_code, bot).await;
                    let id = {
                        let a = res.get("file_id_mt").and_then(|v| v.as_str()).unwrap_or("");
                        if a.is_empty() {
                            res.get("file_id").and_then(|v| v.as_str()).unwrap_or("")
                        } else {
                            a
                        }
                    }
                    .trim()
                    .to_string();
                    if id.is_empty() {
                        return Err(format!("{} (re-resolve returned no file_id)", e));
                    }
                    // A refresh that hands back the SAME id is the origin serving
                    // its own cache (or a 503 with a stale fallback); retrying it
                    // only spends a round-trip to fail identically.
                    if &id == file_id {
                        return Err(format!("{} (re-resolve returned the same id)", e));
                    }
                    *file_id = id;
                    *file_size = res
                        .get("file_size")
                        .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                        .unwrap_or(*file_size);
                    continue;
                }
                return Err(e.to_string());
            }
        }
    }
    Err("probe: re-resolve retry did not settle".to_string())
}

/* â”€â”€ mint drivers (`--mint-batch`, `--mint-streams`) â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€â”€ */

/// How many short codes `--mint-streams` mints in one run.
///
/// Hardcoded, not a flag: every row is a real Telegram send to a relay account,
/// and the cap exists so pointing the driver at a 100-row `/stream` list cannot
/// turn into 100 messages from one command. Ten is enough to measure a list.
const MINT_STREAMS_CAP: usize = 10;

/// The prologue both mint drivers share: bot, relay peers, one connection.
///
/// Which bot mints matters: a `file_id` is bot-specific, so the ids the driver
/// prints are only usable by the bot that answered here. The peer list is the
/// origin's (`GET /userbots`) unless one is passed, so nothing is hardcoded.
///
/// One connection, exactly like `--mint-probe`: a diagnostic must not open a
/// second set of sessions for the same bot, and publishing it makes the mints run
/// through the production path (`telegram::shared_pool`). Every failure exits with
/// the same JSON `--mint-batch` has always printed.
async fn mint_driver_setup(
    peers_arg: Option<&str>,
) -> (String, Vec<String>, grammers_client::Client) {
    let bot = session::get_bot_id();
    if bot.is_empty() {
        eprintln!(
            "{{\"ok\":0,\"error\":\"no bot_id in storage: start the server once so a bot session exists\"}}"
        );
        std::process::exit(2);
    }
    let peers: Vec<String> = match peers_arg {
        Some(list) => list
            .split(',')
            .map(|p| p.trim().trim_start_matches('@').to_string())
            .filter(|p| !p.is_empty())
            .collect(),
        None => mint::peers().await,
    };
    if peers.is_empty() {
        eprintln!(
            "{{\"ok\":0,\"error\":\"no relay peers: pass [peer,peer,...], or make GET {}/userbots answer accounts[]\"}}",
            config::wp_api_base()
        );
        std::process::exit(2);
    }
    mint::seed_peers(peers.clone());
    let pool = Arc::new(telegram::TelegramPool::new(1, DEFAULT_INFLIGHT));
    telegram::share_pool(pool.clone());
    let entry = match pool.boot(&bot, None).await {
        Ok(e) => e,
        Err(e) => {
            eprintln!("{{\"ok\":0,\"error\":\"bot boot failed: {}\"}}", e);
            std::process::exit(1);
        }
    };
    let client = entry.pick();
    (bot, peers, client)
}

/// One refused candidate in a row's `tried` list.
fn tried_entry(field: &str, error: String) -> serde_json::Value {
    serde_json::json!({"field": field, "error": error})
}

/// The id forms a resolve reply carries, in the engine's own order
/// (`file_id_mt`, `file_id_tmp`, `file_id`).
///
/// Empties are dropped and duplicate VALUES are collapsed (the first field name
/// wins), because the three fields usually name the same document and re-sending an
/// identical id would only repeat the refusal. The order is preserved so
/// `file_id_mt` â€” the form the engine itself prefers â€” is always tried first.
fn resolve_id_candidates(resolved: &serde_json::Value) -> Vec<(&'static str, String)> {
    let mut candidates: Vec<(&'static str, String)> = Vec::new();
    for field in ["file_id_mt", "file_id_tmp", "file_id"] {
        let id = resolved
            .get(field)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if id.is_empty() || candidates.iter().any(|(_, seen)| seen == id) {
            continue;
        }
        candidates.push((field, id.to_string()));
    }
    candidates
}

/// ONE row of the mint drivers: resolve, mint, then read 512 KB through the
/// MINTED id.
///
/// Shared by `--mint-batch` and `--mint-streams` on purpose (one function, not a
/// copy per driver): a green run is evidence about production behaviour only if
/// it calls the same functions the resolve path calls (`resolve::resolve_shortcode`,
/// `mint::mint`, `telegram::fetch_range`).
///
/// The resolve reply can carry THREE id forms (`file_id_mt`, `file_id_tmp`,
/// `file_id`) and Telegram does not accept them interchangeably: measured on a
/// `/stream` list, `MEDIA_EMPTY (400)` for one form while another mints fine with
/// the same bot and peers, and every form decodes cleanly (version 4, dc 4,
/// document, 25-byte reference, access_hash present). So the row walks every
/// DISTINCT value in the engine's own order until a send is accepted, records the
/// field that won (`field`) and every refusal on the way (`tried`). A candidate is
/// never retried.
///
/// Returns `(row, ok)`. Rows are independent: one unresolvable code, one refused
/// send and one short read each carry their own reason, and a partial run is still
/// a measurement.
async fn mint_code_row(
    code: &str,
    bot: &str,
    client: &grammers_client::Client,
) -> (serde_json::Value, bool) {
    let mut row = serde_json::json!({
        "code": code,
        "resolve_ms": 0,
        "mint_ms": 0,
        "fetch_ms": 0,
        "bytes_fetched": 0,
        "minted_differs": serde_json::Value::Null,
        "peer": serde_json::Value::Null,
        "error": serde_json::Value::Null,
    });

    // `true` = bypass: this forces the ORIGIN to re-mint its own reference, which
    // is exactly the baseline the mint has to beat. With CHANGE 1 the bypass branch
    // re-sends a locally held id first and only falls through to the origin when
    // there is none â€” for a fresh code there is none, so the driver still measures
    // the origin baseline it was written for.
    let resolve_start = std::time::Instant::now();
    let resolved = resolve::resolve_shortcode(code, bot, true).await;
    row["resolve_ms"] = serde_json::json!(resolve_start.elapsed().as_millis() as u64);

    // Every id form the reply carried, in the engine's own order (`source_id`),
    // without empties and without duplicate values: the three fields usually name
    // the same document, and re-sending an identical id would only repeat the
    // refusal. The field name of the first occurrence is the one reported.
    let candidates = resolve_id_candidates(&resolved);
    if candidates.is_empty() {
        let why = resolved
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("the origin answered no file id");
        row["error"] = serde_json::json!(format!("resolve_failed: {}", why));
        return (row, false);
    }

    // ONE 20 s ceiling for the whole walk, exactly the budget the single send had:
    // a diagnostic must not sit on a wedged datacentre, and three stuck forms must
    // not multiply that by three.
    let mint_start = std::time::Instant::now();
    let mint_deadline = mint_start + std::time::Duration::from_secs(20);
    let mut tried: Vec<serde_json::Value> = Vec::new();
    let mut winner: Option<(&'static str, String, fileid::DecodedFileId, mint::Minted)> = None;
    for (field, id) in &candidates {
        let decoded = match fileid::decode_file_id(id) {
            Ok(d) => d,
            Err(e) => {
                tried.push(tried_entry(*field, format!("decode_failed: {}", e)));
                continue;
            }
        };
        // A photo/thumbnail id has no document to resend; sending it through
        // `inputMediaDocument` fails on the server naming the wrong object.
        if !fileid::is_document_file_id(&decoded) {
            tried.push(tried_entry(
                *field,
                format!("not_a_document: {}", decoded.type_name),
            ));
            continue;
        }
        let left = mint_deadline.saturating_duration_since(std::time::Instant::now());
        if left.is_zero() {
            tried.push(tried_entry(
                *field,
                "mint_failed: timed out after 20s".to_string(),
            ));
            break;
        }
        // `mint::mint` keeps its own peer round-robin (`NEXT_PEER`), so each form
        // goes to the next relay account exactly as a single send always did.
        match tokio::time::timeout(left, mint::mint(bot, id)).await {
            Ok(Ok(m)) => {
                winner = Some((*field, id.clone(), decoded, m));
                break;
            }
            Ok(Err(e)) => tried.push(tried_entry(*field, format!("mint_failed: {}", e))),
            Err(_) => {
                tried.push(tried_entry(
                    *field,
                    "mint_failed: timed out after 20s".to_string(),
                ));
                break;
            }
        }
    }
    row["mint_ms"] = serde_json::json!(mint_start.elapsed().as_millis() as u64);
    // The last refusal is what the row's `error` names below, so read it before the
    // walk moves into the JSON.
    let last_refusal = tried.last().map(|t| t["error"].clone());
    if !tried.is_empty() {
        row["tried"] = serde_json::Value::Array(tried);
    }
    let Some((field, sent_id, decoded, minted)) = winner else {
        // Every form was refused: `error` is the LAST refusal and `tried` carries
        // the whole walk.
        row["error"] = last_refusal
            .unwrap_or_else(|| serde_json::json!("mint_failed: no id form was accepted"));
        return (row, false);
    };
    row["field"] = serde_json::json!(field);
    row["minted_differs"] = serde_json::json!(minted.file_id != sent_id);
    row["peer"] = serde_json::json!(minted.peer);

    // Bytes through the MINTED id are the only evidence the new reference is
    // usable; `prepare_file_target` is the same gate the byte path uses.
    let target = match telegram::prepare_file_target(&minted.file_id, decoded.size) {
        Ok(t) => t,
        Err(e) => {
            row["error"] = serde_json::json!(format!("prepare_failed: {}", e));
            return (row, false);
        }
    };
    let fetch_start = std::time::Instant::now();
    match telegram::fetch_range(client, &target, 0, telegram::MAX_CHUNK).await {
        Ok(bytes) => {
            row["fetch_ms"] = serde_json::json!(fetch_start.elapsed().as_millis() as u64);
            row["bytes_fetched"] = serde_json::json!(bytes.len());
            if bytes.len() as u64 >= telegram::MAX_CHUNK {
                return (row, true);
            }
            // A SHORT read is not a failure of the mint, but it is not 512 KB of
            // evidence either, so it is recorded rather than rounded up.
            row["error"] = serde_json::json!(format!(
                "short_read: {} of {} bytes",
                bytes.len(),
                telegram::MAX_CHUNK
            ));
        }
        Err(e) => {
            row["fetch_ms"] = serde_json::json!(fetch_start.elapsed().as_millis() as u64);
            row["error"] = serde_json::json!(format!("fetch_failed: {}", e));
        }
    }
    (row, false)
}

/// Every row the drivers ran, plus how many read a full 512 KB.
async fn mint_code_rows(
    codes: &[String],
    bot: &str,
    client: &grammers_client::Client,
) -> (Vec<serde_json::Value>, usize) {
    let mut rows: Vec<serde_json::Value> = Vec::with_capacity(codes.len());
    let mut ok_count = 0usize;
    for code in codes {
        let (row, ok) = mint_code_row(code, bot, client).await;
        if ok {
            ok_count += 1;
        }
        rows.push(row);
    }
    (rows, ok_count)
}

/// "ok" means no code ended SILENTLY: every row either read 512 KB or says why it
/// did not. A partial run therefore reports `ok:1` with its failures counted,
/// which is the point of a diagnostic over several codes.
fn mint_rows_ok(rows: &[serde_json::Value]) -> bool {
    rows.iter().all(|r| {
        !r["error"].is_null()
            || r["bytes_fetched"].as_u64().unwrap_or(0) >= telegram::MAX_CHUNK
    })
}

/// GET a `/stream` list with the engine's plain HTTP client â€” no auth.
///
/// The driver's URL is the engine's own `http://127.0.0.1:<port>/stream/...` (or
/// any other node's), which is a public route: the point is to measure the rows a
/// player is actually handed, so no ticket, no PM-Auth proof, no token.
async fn fetch_streams(url: &str) -> Result<serde_json::Value, String> {
    let resp = crate::httpc::client()
        .get(url)
        .timeout(std::time::Duration::from_secs(12))
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", status));
    }
    resp.json::<serde_json::Value>()
        .await
        .map_err(|e| format!("not json: {}", e))
}

/// Every short code a `/stream` list names, deduped, in list order, capped at
/// [`MINT_STREAMS_CAP`].
///
/// Per row, in this order of preference (CHANGE 2):
///   1. a `short_code=` query parameter in `streams[].url`;
///   2. a `pm_file_<code>` (or `pm:file:<code>`) id anywhere in the row â€” the
///      catalog publishes those on `id`;
///   3. an `/api/download/<payload>` path segment whose base64 payload decodes to
///      a `short_code` (the join token segment before it is skipped).
///
/// A row that names no code is skipped rather than guessed at.
fn stream_codes(body: &serde_json::Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let Some(rows) = body.get("streams").and_then(|v| v.as_array()) else {
        return out;
    };
    for row in rows {
        let Some(code) = stream_row_code(row) else {
            continue;
        };
        if !out.iter().any(|c| c == &code) {
            out.push(code);
        }
        if out.len() >= MINT_STREAMS_CAP {
            break;
        }
    }
    out
}

fn stream_row_code(row: &serde_json::Value) -> Option<String> {
    let url = row.get("url").and_then(|v| v.as_str()).unwrap_or("");
    if let Some(code) = short_code_param(url) {
        return Some(code);
    }
    if let Some(code) = pm_file_id(row) {
        return Some(code);
    }
    download_payload_code(url)
}

/// `?short_code=<code>` (also accepts it as a later parameter or a fragment).
fn short_code_param(url: &str) -> Option<String> {
    let marker = "short_code=";
    let start = url.find(marker)? + marker.len();
    let rest = &url[start..];
    let end = rest
        .find(|c| c == '&' || c == '#' || c == ' ')
        .unwrap_or(rest.len());
    let code = rest[..end].trim();
    (!code.is_empty()).then(|| code.to_string())
}

/// The first `pm_file_<code>` / `pm:file:<code>` token in the row, in any field.
fn pm_file_id(row: &serde_json::Value) -> Option<String> {
    fn from_text(text: &str) -> Option<String> {
        for marker in ["pm_file_", "pm:file:"] {
            if let Some(at) = text.find(marker) {
                let rest = &text[at + marker.len()..];
                let end = rest
                    .find(|c: char| !c.is_ascii_alphanumeric())
                    .unwrap_or(rest.len());
                if end > 0 {
                    return Some(rest[..end].to_string());
                }
            }
        }
        None
    }
    match row {
        serde_json::Value::String(s) => from_text(s),
        serde_json::Value::Array(items) => items.iter().find_map(pm_file_id),
        serde_json::Value::Object(map) => map.values().find_map(pm_file_id),
        _ => None,
    }
}

/// The `short_code` inside an `/api/download/<payload>` segment.
fn download_payload_code(url: &str) -> Option<String> {
    let marker = "/api/download/";
    let at = url.find(marker)? + marker.len();
    let segment = url[at..]
        .split(|c| c == '/' || c == '?' || c == '#')
        .next()
        .unwrap_or("");
    if segment.is_empty() {
        return None;
    }
    let payload = crate::download::decode_download_payload(segment);
    let code = payload
        .get("short_code")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    (!code.is_empty()).then(|| code.to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `--version` / `-V`: one line, no storage, no network. This is what the
    // packaging scripts assert on every platform (`--version` must print *before*
    // anything tries to boot grammers or read `storage/`).
    if arg_pos("--version").is_some() || arg_pos("-V").is_some() {
        println!("{}", version_line());
        return Ok(());
    }

    // Before any request can ask how long the server has been up.
    diagnostics::record_process_start();

    // Name a panic instead of letting it become an anonymous 502.
    //
    // A panicking request task produces NO response and never reaches `log()`, so
    // the player gets whatever the edge substitutes and the operator gets nothing.
    // Measured on the Render instance (2026-11-03): `/stream/movie/pm%3Apost%3A9000016051.json`
    // answered **HTTP 502 with an empty body in ~130 ms**, reproducibly, while its
    // neighbours answered 200 â€” and `debug.log` held no line for it at all. This
    // hook turns that class of failure into one always-on stderr line carrying the
    // message and the exact source location.
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_default();
        let message = info
            .payload()
            .downcast_ref::<&str>()
            .map(|s| (*s).to_string())
            .or_else(|| info.payload().downcast_ref::<String>().cloned())
            .unwrap_or_else(|| "non-string panic payload".to_string());
        let ctx = serde_json::json!({"message": message, "location": location});
        // Always-on (the platform log), and debug.log too when it is enabled so
        // `/api/logs` can serve it remotely.
        log::stderr_log("panic", Some(&ctx));
        log::log("panic", &ctx);
    }));

    // The APP shell that launched this engine (the Android APK's versionName).
    // `/api/version` reports it as `client_version`; without it the dashboard
    // could only show the engine version, so users could not tell which APK they
    // were running.
    if let Some(v) = arg_value("--app-version").or_else(|| storage::env("PM_APP_VERSION")) {
        config::set_client_version(&v);
    }

    // `--decode` / `--selftest`: print the Bot API file_id decoding, then exit.
    if let Some(i) = arg_pos("--decode").or_else(|| arg_pos("--selftest")) {
        let args: Vec<String> = std::env::args().collect();
        let Some(raw) = args.get(i + 1) else {
            eprintln!("usage: pencarimovie-server --decode <file_id_mt> [expected_id]");
            std::process::exit(2);
        };
        let expected = args.get(i + 2).map(|s| s.as_str());
        match fileid::decode_file_id(raw) {
            Ok(d) => {
                let mut out = d.to_json();
                if let Some(exp) = expected {
                    out["id_matches_expected"] = serde_json::json!(exp == d.id.to_string());
                }
                println!("{}", serde_json::to_string_pretty(&out)?);
                return Ok(());
            }
            Err(e) => {
                eprintln!("{{\"ok\":0,\"error\":\"{}\"}}", e);
                std::process::exit(1);
            }
        }
    }

    // `--thumb-probe <file_id_mt|short_code> [bot_id]`: ask Telegram for this
    // document's THUMBNAILS the way `/api/download` asks for its bytes.
    //
    // The origin's thumbnail endpoint is gone, so the question is whether the
    // engine can mint posters from Telegram itself. A thumbnail is not a second
    // file: it is the same `id`/`access_hash`/`file_reference`/dc with
    // `thumb_size` set (`s`/`m`/`x`/`y`/`w`), so a `file_id_mt` that already
    // streams carries its own posters and needs no resolve of its own. This prints
    // what each size actually answers (bytes, jpeg/png, dimensions) rather than
    // leaving it as a guess.
    if let Some(i) = arg_pos("--thumb-probe") {
        let args: Vec<String> = std::env::args().collect();
        let Some(raw) = args.get(i + 1) else {
            eprintln!("usage: pencarimovie-server --thumb-probe <file_id_mt|short_code> [bot_id]");
            std::process::exit(2);
        };
        let bot = args
            .get(i + 2)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(session::get_bot_id);
        if bot.is_empty() {
            eprintln!("{{\"ok\":0,\"error\":\"no bot_id: pass one as the second argument\"}}");
            std::process::exit(2);
        }

        let mut file_id = raw.trim().to_string();
        let mut file_size = 0u64;
        // Only a short code can be re-resolved when Telegram says the reference
        // expired; a bare `file_id_mt` has no second source.
        let mut short_code = String::new();
        if fileid::decode_file_id(&file_id).is_err() {
            // Not a file_id, so treat it as a short code: resolve it through the
            // real path, exactly like an incoming `/api/download` would.
            short_code = raw.trim().to_string();
            let res = resolve::resolve_shortcode(&file_id, &bot, false).await;
            let id = {
                let a = res.get("file_id_mt").and_then(|v| v.as_str()).unwrap_or("");
                if a.is_empty() {
                    res.get("file_id").and_then(|v| v.as_str()).unwrap_or("")
                } else {
                    a
                }
            }
            .trim()
            .to_string();
            file_size = res
                .get("file_size")
                .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .unwrap_or(0);
            if id.is_empty() {
                eprintln!(
                    "{{\"ok\":0,\"error\":\"resolve returned no file_id_mt\",\"resolve\":{}}}",
                    serde_json::to_string(&res).unwrap_or_default()
                );
                std::process::exit(1);
            }
            file_id = id;
            // Two separate questions this probe answers:
            //   â€¢ is the origin's poster still a real per-file image at all;
            //   â€¢ is `â€¦/tgthumb/<n>.jpg` the MEDIA document's id (a thumb of the
            //     file we already resolve) or a SEPARATE thumbnail file id? The
            //     reply prints the decoded media id next to that number, so the two
            //     can be compared instead of assumed.
            let thumb_url = res
                .get("thumbnail_url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let doc_id = fileid::decode_file_id(&file_id)
                .map(|d| d.id.to_string())
                .unwrap_or_default();
            let thumb_n = thumb_url
                .split("/tgthumb/")
                .nth(1)
                .and_then(|s| s.split('.').next())
                .unwrap_or("")
                .to_string();
            println!(
                "{{\"bot_id\":\"{}\",\"short_code\":\"{}\",\"file_size\":{},\"origin_thumbnail_url\":\"{}\",\"media_document_id\":\"{}\",\"origin_thumb_id\":\"{}\",\"thumb_id_is_the_media_doc\":{}}}",
                bot,
                raw.trim(),
                file_size,
                thumb_url,
                doc_id,
                thumb_n,
                (!thumb_n.is_empty() && thumb_n == doc_id)
            );
            // Every field the origin actually sent, with the two id fields elided.
            // A dedicated `thumbnail_file_id` would show up here â€” and if it exists
            // it is a downloadable object in its own right, not just a `thumb_size`.
            let mut dump = res.clone();
            if let Some(o) = dump.as_object_mut() {
                for k in ["file_id_mt", "file_id"] {
                    if let Some(v) = o.get(k) {
                        let n = v.as_str().map(|s| s.len()).unwrap_or(0);
                        o.insert(k.to_string(), serde_json::json!(format!("<redacted len={}>", n)));
                    }
                }
            }
            println!("{}", serde_json::to_string_pretty(&dump).unwrap_or_default());
        } else {
            println!("{{\"bot_id\":\"{}\",\"file_id_mt\":\"{}\"}}", bot, file_id);
        }

        // One connection is enough for a probe, and it keeps the session's other
        // worker files untouched.
        let pool = telegram::TelegramPool::new(1, DEFAULT_INFLIGHT);
        let entry = match pool.boot(&bot, None).await {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{{\"ok\":0,\"error\":\"bot boot failed: {}\"}}", e);
                std::process::exit(1);
            }
        };
        let client = entry.pick();

        // A stored `thumb_file_id` is a PHOTO object, not a document: there is no
        // "the file itself" to read and the sizes are the photo's own. A media
        // document takes the media read plus `thumb_size`.
        let is_photo = fileid::decode_file_id(&file_id)
            .map(|d| fileid::location_kind(&d) == "photo")
            .unwrap_or(false);
        let sizes: &[&str] = if is_photo {
            &["s", "m", "x", "y", "w"]
        } else {
            &["", "s", "m", "x", "y", "w"]
        };

        if !is_photo {
            // The media first: without it nothing else means anything.
            match probe_fetch(&client, &mut file_id, &mut file_size, "", &short_code, &bot).await {
                Ok(b) => println!("media     bytes={} kind={}", b.len(), bytes_kind(&b)),
                Err(e) => println!("media     ERR {}", e),
            }
        }
        for size in sizes {
            let size: &str = size;
            let label = if size.is_empty() { "(none)" } else { size };
            match probe_fetch(&client, &mut file_id, &mut file_size, size, &short_code, &bot).await {
                Err(e) => println!("thumb {:<8} ERR {}", label, e),
                Ok(b) => {
                    let kind = bytes_kind(&b);
                    // A thumb_size the document does not have does NOT error:
                    // Telegram serves the FILE from offset 0 instead, so a poster
                    // route that trusts "some bytes arrived" hands the player the
                    // first 512 KB of the movie.
                    let note = if kind == "unknown" {
                        "  <-- FALLBACK: no such thumb_size, these are the file's bytes"
                    } else {
                        ""
                    };
                    match jpeg_dims(&b) {
                        Some((w, h)) => println!(
                            "thumb {:<8} bytes={} kind={} {}x{}{}",
                            label,
                            b.len(),
                            kind,
                            w,
                            h,
                            note
                        ),
                        None => {
                            println!("thumb {:<8} bytes={} kind={}{}", label, b.len(), kind, note)
                        }
                    }
                }
            }
        }
        return Ok(());
    }

    // `--mint-probe <file_id_mt> <user_id> [bot_id]`: mint a fresh `file_id` by
    // RE-SENDING the document, then read with the new id.
    //
    // A `file_reference` lives about an hour and only the bot that owns the file
    // can mint a replacement; that expiry is where every `FILE_REFERENCE_EXPIRED`
    // recovery in `download.rs` starts. The Bot API's
    // `sendDocument(chat_id, document)` is `messages.sendMedia` carrying an
    // `inputMediaDocument` built from the SAME `(id, access_hash,
    // file_reference)`, and the reply's document object holds the new reference.
    // This probe performs that round-trip and reads 512 KB with the minted id, so
    // "a resend mints a usable id" is measured instead of assumed.
    if let Some(i) = arg_pos("--mint-probe") {
        let args: Vec<String> = std::env::args().collect();
        let (Some(raw), Some(user_arg)) = (args.get(i + 1), args.get(i + 2)) else {
            eprintln!("usage: pencarimovie-server --mint-probe <file_id_mt> <@username|user_id> [bot_id]");
            std::process::exit(2);
        };
        // A bot cannot enumerate dialogs (`messages.getDialogs` answers
        // BOT_METHOD_INVALID), so a bare numeric id is unusable from a bot
        // session â€” there is no `access_hash` to address it with. A username IS
        // resolvable by a bot (`contacts.resolveUsername`), and that is how the
        // origin addresses its relay peers too.
        let peer_spec = user_arg.trim().to_string();
        let peer_id: Option<i64> = peer_spec.parse::<i64>().ok();
        let bot = args
            .get(i + 3)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(session::get_bot_id);
        if bot.is_empty() {
            eprintln!("{{\"ok\":0,\"error\":\"no bot_id: pass one as the third argument\"}}");
            std::process::exit(2);
        }

        let file_id = raw.trim().to_string();
        let decoded = match fileid::decode_file_id(&file_id) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("{{\"ok\":0,\"error\":\"not a file_id_mt: {}\"}}", e.replace('"', "'"));
                std::process::exit(2);
            }
        };
        // Only a document type has an `inputDocument` to resend: a photo id sent
        // through `inputMediaDocument` is addressed as a document and fails on
        // the server with an error that names the wrong object.
        if !fileid::is_document_file_id(&decoded) {
            eprintln!(
                "{{\"ok\":0,\"error\":\"file_id type {} has no document to resend\"}}",
                decoded.type_name
            );
            std::process::exit(2);
        }
        // A Bot API file_id carries no size (`decode_file_id` leaves it 0), so
        // the target records what is known; 0 is valid for a read.
        let file_size = decoded.size;
        let input_target = match telegram::prepare_file_target(&file_id, file_size) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{{\"ok\":0,\"error\":\"{}\"}}", e.replace('"', "'"));
                std::process::exit(1);
            }
        };

        // Same bootstrap as `--thumb-probe`: one connection is enough for a probe
        // and it keeps the session's other worker files untouched.
        let pool = telegram::TelegramPool::new(1, DEFAULT_INFLIGHT);
        let entry = match pool.boot(&bot, None).await {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{{\"ok\":0,\"error\":\"bot boot failed: {}\"}}", e);
                std::process::exit(1);
            }
        };
        let client = entry.pick();

        // The premise is that the CURRENT reference may already be dead, so its
        // state is recorded instead of assumed healthy.
        let baseline_start = std::time::Instant::now();
        let baseline = telegram::fetch_range(&client, &input_target, 0, telegram::MAX_CHUNK).await;
        let baseline_ms = baseline_start.elapsed().as_millis() as u64;
        let (input_bytes, input_error) = match &baseline {
            Ok(b) => (b.len(), serde_json::Value::Null),
            Err(e) => (0usize, serde_json::json!(e.to_string())),
        };

        // `@name` goes through contacts.resolveUsername (a bot may call it); a
        // bare id would need an `access_hash` the bot has no way to obtain, so
        // that branch only works when the id is already in the session's dialogs.
        let peer = if peer_spec.starts_with('@') {
            let name = peer_spec.trim_start_matches('@');
            match client.resolve_username(name).await {
                Ok(Some(p)) => p,
                Ok(None) => {
                    eprintln!("{{\"ok\":0,\"error\":\"username @{} does not resolve\"}}", name);
                    std::process::exit(1);
                }
                Err(e) => {
                    eprintln!(
                        "{{\"ok\":0,\"error\":\"resolve @{}: {}\"}}",
                        name,
                        e.to_string().replace('"', "'")
                    );
                    std::process::exit(1);
                }
            }
        } else {
            let Some(id) = peer_id else {
                eprintln!("{{\"ok\":0,\"error\":\"<peer> must be @username or a numeric user id\"}}");
                std::process::exit(2);
            };
            match telegram::resolve_peer_by_id(&client, id).await {
                // `PeerRef` â†’ owned `Peer`: `resolve_username` answers `Peer`, so
                // both branches share one type and `to_ref()` is taken per call.
                Ok(p) => match client.resolve_peer(p).await {
                    Ok(peer) => peer,
                    Err(e) => {
                        eprintln!(
                            "{{\"ok\":0,\"error\":\"peer {}: {}\"}}",
                            id,
                            e.to_string().replace('"', "'")
                        );
                        std::process::exit(1);
                    }
                },
                Err(e) => {
                    eprintln!(
                        "{{\"ok\":0,\"error\":\"peer {}: {}\"}}",
                        id,
                        e.message.replace('"', "'")
                    );
                    std::process::exit(1);
                }
            }
        };

        // `to_ref` is ASYNC in 0.12 and can answer `None` when the peer is not
        // usable from this session (no access hash). A bot has no dialog list to
        // fall back on, so that is a hard stop with the reason named.
        let peer_ref = match peer.to_ref().await {
            Ok(Some(p)) => p,
            Ok(None) => {
                eprintln!(
                    "{{\"ok\":0,\"error\":\"peer {} is not addressable from this session (no access hash)\"}}",
                    peer_spec
                );
                std::process::exit(1);
            }
            Err(e) => {
                eprintln!(
                    "{{\"ok\":0,\"error\":\"peer {}: {}\"}}",
                    peer_spec,
                    e.to_string().replace('"', "'")
                );
                std::process::exit(1);
            }
        };

        use grammers_client::media::Media;
        use grammers_client::message::InputMessage;
        use grammers_client::tl;

        // No upload: the document already lives on Telegram, so this is a pure
        // `messages.sendMedia`. `video_cover`/`video_timestamp` stay null because
        // the probe is not editing a video's attributes.
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
        let mint_start = std::time::Instant::now();
        let sent = client
            .send_message(peer_ref, InputMessage::new().media(media))
            .await;
        let mint_ms = mint_start.elapsed().as_millis() as u64;
        let msg = match sent {
            Ok(m) => m,
            Err(e) => {
                eprintln!(
                    "{{\"ok\":0,\"error\":\"send failed: {}\"}}",
                    e.to_string().replace('"', "'")
                );
                std::process::exit(1);
            }
        };
        let message_id = msg.id();

        // The triple the byte path needs comes from the SEND RESULT, never from a
        // re-resolve: this is also the only object that carries the NEW
        // `file_reference`.
        let doc = match msg.media() {
            Some(Media::Document(m)) => match m.raw.document {
                Some(tl::enums::Document::Document(d)) => d,
                _ => {
                    eprintln!("{{\"ok\":0,\"error\":\"sent media has no document\"}}");
                    std::process::exit(1);
                }
            },
            _ => {
                eprintln!("{{\"ok\":0,\"error\":\"sent media is not a document\"}}");
                std::process::exit(1);
            }
        };

        // The Bot API id for what came back, NOT a copy of the input: the reply's
        // `file_reference` is the whole point. Version 4 / sub 0 is the wire shape
        // this account's Bot API ids use (the live fixture in `fileid` is 4/0).
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
        let minted_file_id = fileid::encode_file_id(&minted);
        let minted_differs = minted_file_id != file_id;

        // Read through the NEW id: bytes from a resend are the only evidence the
        // minted reference is usable for playback.
        let (fetch_ms, fetched_bytes, fetch_error) =
            match telegram::prepare_file_target(&minted_file_id, minted.size) {
                Err(e) => (0u64, 0usize, serde_json::json!(e)),
                Ok(target) => {
                    let started = std::time::Instant::now();
                    match telegram::fetch_range(&client, &target, 0, telegram::MAX_CHUNK).await {
                        Ok(bytes) => (
                            started.elapsed().as_millis() as u64,
                            bytes.len(),
                            serde_json::Value::Null,
                        ),
                        Err(e) => (
                            started.elapsed().as_millis() as u64,
                            0usize,
                            serde_json::json!(e.to_string()),
                        ),
                    }
                }
            };

        // The probe no longer deletes: the message keeps the document referenced in
        // the chat, and `mint::mint` keeps it for the same reason. Reported as
        // `false` rather than dropped, so existing JSON readers keep a field.
        let deleted = false;

        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": 1,
                "bot_id": bot,
                "peer": peer_spec,
                "input_file_id": file_id,
                "input_fetch_bytes": input_bytes,
                "input_fetch_ms": baseline_ms,
                "input_fetch_error": input_error,
                "minted_file_id": minted_file_id,
                "minted_differs_from_input": minted_differs,
                "document_id": doc.id.to_string(),
                "access_hash": doc.access_hash.to_string(),
                "dc_id": doc.dc_id,
                "size": minted.size,
                "mint_ms": mint_ms,
                "fetch_ms": fetch_ms,
                "bytes_fetched": fetched_bytes,
                "fetch_error": fetch_error,
                "message_id": message_id,
                "message_deleted": deleted,
            }))
            .unwrap_or_default()
        );
        return Ok(());
    }

    // `--mint-batch <code1,code2,...> [peer,peer,...]`: the warm path for several
    // short codes in one run â€” resolve, mint, read 512 KB through the minted id.
    //
    // This is the E2E driver for `mint.rs`, and it calls the SAME functions the
    // resolve path calls (`mint::mint`, `mint::peers`) instead of a copy: a green
    // run is evidence about production behaviour, not about a diagnostic that
    // happens to agree with it. The peer list is the origin's (`GET /userbots`)
    // unless one is passed, so nothing here is hardcoded.
    //
    // Rows are independent on purpose: one unresolvable code, one refused send and
    // one short read each carry their own reason, and the summary counts them. A
    // partial run is still a measurement.
    if let Some(i) = arg_pos("--mint-batch") {
        let args: Vec<String> = std::env::args().collect();
        let Some(codes_arg) = args.get(i + 1) else {
            eprintln!("usage: pencarimovie-server --mint-batch <code1,code2,...> [peer,peer,...]");
            std::process::exit(2);
        };
        let codes: Vec<String> = codes_arg
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if codes.is_empty() {
            eprintln!("{{\"ok\":0,\"error\":\"no short codes in the first argument\"}}");
            std::process::exit(2);
        }
        // The prologue (bot, peers, one connection) is shared with
        // `--mint-streams` in `mint_driver_setup`: two drivers must not drift into
        // two different measurements.
        let (bot, peers, client) = mint_driver_setup(args.get(i + 2).map(|s| s.as_str())).await;

        let (rows, ok_count) = mint_code_rows(&codes, &bot, &client).await;
        let total = codes.len();
        let ok = mint_rows_ok(&rows);
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": if ok { 1 } else { 0 },
                "codes_total": total,
                "ok_count": ok_count,
                "failed_count": total - ok_count,
                "bot_id": bot,
                "peers": peers,
                "rows": rows,
            }))
            .unwrap_or_default()
        );
        return Ok(());
    }

    // `--mint-streams <streams-url> [peer,peer,...]`: the same driver over a REAL
    // `/stream` list instead of a hand-typed code list.
    //
    // The URL is this engine's own `http://127.0.0.1:<port>/stream/...` (or another
    // node's), fetched with the plain HTTP client and NO auth: it is a public route,
    // and the point is the exact rows a player is handed. Every short code the list
    // names is extracted ([`stream_codes`]: `short_code=`, then a `pm_file_` id,
    // then an `/api/download/<payload>` segment), deduped, order kept, capped at
    // [`MINT_STREAMS_CAP`] = 10, and then run through the SAME per-code pipeline
    // `--mint-batch` uses ([`mint_code_rows`]).
    if let Some(i) = arg_pos("--mint-streams") {
        let args: Vec<String> = std::env::args().collect();
        let Some(url) = args
            .get(i + 1)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
        else {
            eprintln!("usage: pencarimovie-server --mint-streams <streams-url> [peer,peer,...]");
            std::process::exit(2);
        };
        let body = match fetch_streams(&url).await {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "{}",
                    serde_json::json!({
                        "ok": 0,
                        "error": format!("streams fetch failed: {}", e),
                        "source_url": url,
                    })
                );
                std::process::exit(1);
            }
        };
        let codes = stream_codes(&body);
        if codes.is_empty() {
            eprintln!(
                "{}",
                serde_json::json!({
                    "ok": 0,
                    "error": "the streams list named no usable short code",
                    "source_url": url,
                    "streams_total": body
                        .get("streams")
                        .and_then(|v| v.as_array())
                        .map(|a| a.len())
                        .unwrap_or(0),
                })
            );
            std::process::exit(2);
        }
        let (bot, peers, client) = mint_driver_setup(args.get(i + 2).map(|s| s.as_str())).await;
        let (rows, ok_count) = mint_code_rows(&codes, &bot, &client).await;
        let total = codes.len();
        let ok = mint_rows_ok(&rows);
        // The batch arm's shape, plus what this driver was pointed at and what it
        // pulled out of it (so a surprising `codes_total` is explainable).
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "ok": if ok { 1 } else { 0 },
                "codes_total": total,
                "ok_count": ok_count,
                "failed_count": total - ok_count,
                "bot_id": bot,
                "peers": peers,
                "source_url": url,
                "codes": codes,
                "rows": rows,
            }))
            .unwrap_or_default()
        );
        return Ok(());
    }

    // `--probe-tracks <file_id_mt|short_code> [bot_id]`: read the first chunk of
    // the file and report the audio and subtitle tracks the CONTAINER declares.
    //
    // `/stream` still labels a row from its filename: a list render must not spend
    // MTProto parts, because `workers Ã— inflight` is the part budget shared by every
    // viewer (AGENTS.md Â§9) and probing a hundred rows would starve playback. This
    // answers the earlier question â€” what the file itself says â€” without touching
    // that budget, and is what the filter work is waiting on.
    if let Some(i) = arg_pos("--probe-tracks") {
        let args: Vec<String> = std::env::args().collect();
        let Some(raw) = args.get(i + 1) else {
            eprintln!("usage: pencarimovie-server --probe-tracks <file_id_mt|short_code> [bot_id]");
            std::process::exit(2);
        };
        let bot = args
            .get(i + 2)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(session::get_bot_id);
        if bot.is_empty() {
            eprintln!("{{\"ok\":0,\"error\":\"no bot_id: pass one as the second argument\"}}");
            std::process::exit(2);
        }
        let mut file_id = raw.trim().to_string();
        let mut file_size = 0u64;
        let mut short_code = String::new();
        if fileid::decode_file_id(&file_id).is_err() {
            short_code = raw.trim().to_string();
            let res = resolve::resolve_shortcode(&file_id, &bot, false).await;
            let id = {
                let a = res.get("file_id_mt").and_then(|v| v.as_str()).unwrap_or("");
                if a.is_empty() {
                    res.get("file_id").and_then(|v| v.as_str()).unwrap_or("")
                } else {
                    a
                }
            }
            .trim()
            .to_string();
            file_size = res
                .get("file_size")
                .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .unwrap_or(0);
            if id.is_empty() {
                eprintln!("{{\"ok\":0,\"error\":\"resolve returned no file_id_mt\"}}");
                std::process::exit(1);
            }
            file_id = id;
        }
        let pool = telegram::TelegramPool::new(1, DEFAULT_INFLIGHT);
        let entry = match pool.boot(&bot, None).await {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{{\"ok\":0,\"error\":\"bot boot failed: {}\"}}", e);
                std::process::exit(1);
            }
        };
        let client = entry.pick();
        match probe_fetch(&client, &mut file_id, &mut file_size, "", &short_code, &bot).await {
            Ok(bytes) => {
                // `None` is "could not tell" and must stay distinct from "no
                // tracks": an unreadable prefix is not evidence of a silent file.
                let tracks = match media_probe::probe(&bytes) {
                    Some(t) => serde_json::json!({
                        "audio": t.audio,
                        "channels": t.channels,
                        "audio_languages": t.audio_languages,
                        "subtitle_languages": t.subtitle_languages,
                    }),
                    None => serde_json::Value::Null,
                };
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "ok": 1,
                        "short_code": short_code,
                        "file_size": file_size,
                        "prefix_bytes": bytes.len(),
                        "kind": bytes_kind(&bytes),
                        "tracks": tracks,
                    }))
                    .unwrap_or_default()
                );
            }
            Err(e) => {
                eprintln!("{{\"ok\":0,\"error\":\"{}\"}}", e.replace('"', "'"));
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // `--file-id-lifetime <file_id_mt|short_code> [bot_id] [interval_s] [horizon_min]`:
    // how long a `file_reference` REALLY lives.
    //
    // Every timing decision in `download.rs` about re-resolving â€” the 60 s window
    // for an unverified id, `verified_at`, the mid-body refresh â€” is a guess about a
    // number Telegram never publishes. Its documentation says only that a
    // `file_reference` "can change at any moment", and MadelineProto is built on
    // that assumption rather than on a lifetime: it keeps the last reference per
    // file in a `ReferenceDatabase`, and on `FILE_REFERENCE_EXPIRED` re-fetches the
    // message that owns the media and repeats the same `upload.getFile`.
    //
    // This probe measures the number instead of assuming one: one 4 KB
    // `upload.getFile` at offset 0, repeated on a fixed interval with the SAME
    // `file_id`, printing `ok` until Telegram refuses it, then the elapsed time.
    // Run it with a `short_code` (the id comes from a real resolve, exactly as
    // `/api/download` gets it) or with a `file_id_mt` plus `--size <bytes>`.
    //
    // Exit codes: 0 = still alive at the horizon, 3 = it died (elapsed printed),
    // 1 = the probe could not start.
    if let Some(i) = arg_pos("--file-id-lifetime") {
        let args: Vec<String> = std::env::args().collect();
        let Some(raw) = args.get(i + 1) else {
            eprintln!(
                "usage: pencarimovie-server --file-id-lifetime <file_id_mt|short_code> [bot_id] [interval_s] [horizon_min] [--size <bytes>]"
            );
            std::process::exit(2);
        };
        let bot = args
            .get(i + 2)
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(session::get_bot_id);
        if bot.is_empty() {
            eprintln!("no bot_id: pass one as the second argument");
            std::process::exit(2);
        }
        let interval = args
            .get(i + 3)
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(60)
            .max(5);
        let horizon_min = args
            .get(i + 4)
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(180)
            .max(1);

        let mut file_id = raw.trim().to_string();
        let mut file_size = 0u64;
        if fileid::decode_file_id(&file_id).is_err() {
            let res = resolve::resolve_shortcode(&file_id, &bot, false).await;
            let id = {
                let a = res.get("file_id_mt").and_then(|v| v.as_str()).unwrap_or("");
                if a.is_empty() {
                    res.get("file_id").and_then(|v| v.as_str()).unwrap_or("")
                } else {
                    a
                }
            }
            .trim()
            .to_string();
            file_size = res
                .get("file_size")
                .and_then(|v| v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok())))
                .unwrap_or(0);
            if id.is_empty() {
                eprintln!("resolve returned no file_id_mt for {}", raw.trim());
                std::process::exit(1);
            }
            file_id = id;
        }
        if let Some(sz) = arg_value("--size").and_then(|s| s.trim().parse::<u64>().ok()) {
            file_size = sz;
        }
        if file_size == 0 {
            eprintln!("no file_size: pass a short code, or a file_id with --size <bytes>");
            std::process::exit(2);
        }

        let pool = telegram::TelegramPool::new(1, DEFAULT_INFLIGHT);
        let entry = match pool.boot(&bot, None).await {
            Ok(e) => e,
            Err(e) => {
                eprintln!("bot boot failed: {}", e);
                std::process::exit(1);
            }
        };
        let client = entry.pick();
        if let Ok(d) = fileid::decode_file_id(&file_id) {
            println!(
                "file_id id={} dc={} type={} reference={} bytes size={} -> probing every {}s up to {} min",
                d.id,
                d.dc_id,
                d.type_name,
                d.file_reference.as_ref().map(|r| r.len()).unwrap_or(0),
                file_size,
                interval,
                horizon_min
            );
        }
        let start = std::time::Instant::now();
        let horizon = std::time::Duration::from_secs(horizon_min * 60);
        loop {
            let target = match probe_target(&file_id, file_size, "") {
                Ok(t) => t,
                Err(e) => {
                    eprintln!("bad file_id: {}", e);
                    std::process::exit(1);
                }
            };
            let elapsed = start.elapsed().as_secs();
            match telegram::fetch_one_chunk(&client, &target, 0, 4096).await {
                Ok(n) => println!("t={:>5}s ok bytes={}", elapsed, n),
                Err(e) => {
                    println!("t={:>5}s DEAD {}", elapsed, e.message);
                    println!(
                        "file_id lived {}s ({} min) - reference={} bytes, bot {}",
                        elapsed,
                        elapsed / 60,
                        fileid::decode_file_id(&file_id)
                            .map(|d| d.file_reference.as_ref().map(|r| r.len()).unwrap_or(0))
                            .unwrap_or(0),
                        bot
                    );
                    std::process::exit(3);
                }
            }
            if start.elapsed() >= horizon {
                println!(
                    "file_id still alive after {}s ({} min) - horizon reached",
                    start.elapsed().as_secs(),
                    horizon_min
                );
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
        }
    }

    // `--audio-probe` was REMOVED with the ALAC decoder (`src/audio.rs` and the
    // `symphonia` dependency). It existed only to diagnose that decoder's log
    // lines; nothing decodes audio in this binary any more, so the flag would
    // have had nothing to print.

    let port: u16 = arg_value("--port")
        .or_else(|| storage::env("PM_LISTEN_PORT"))
        .or_else(|| storage::env("PORT"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(8088);
    let host = arg_value("--host")
        .or_else(|| storage::env("PM_LISTEN_HOST"))
        .or_else(|| storage::env("HOST"))
        // `set PM_LISTEN_HOST=0.0.0.0 && ...` keeps a trailing space in the value,
        // and "0.0.0.0 :8088" then fails to parse as a socket address.
        .map(|h| h.trim().to_string())
        .unwrap_or_else(|| default_bind_host(cfg!(target_os = "android"), paas_env_present()).to_string());

    // Public mode is a STARTUP decision and is set before anything calls
    // `auth_enabled()`. With it off, every branch below behaves exactly as before.
    let public = arg_pos("--public").is_some() || config::env_flag("PM_PUBLIC");
    config::set_public_mode(public);
    if public {
        if !config::env_flag("PM_ALLOW_DEFAULT_PASSWORD") && auth::auth_uses_default_password() {
            eprintln!(
                "\n  refusing to start: PM_PUBLIC=1 with the default admin password.\n  \
                 Set SERVER_PASSWORD to a strong value (or a strong password in the dashboard) first.\n  \
                 Emergency override: PM_ALLOW_DEFAULT_PASSWORD=1\n"
            );
            std::process::exit(2);
        }
        if let Some(raw) = storage::env("SERVER_TOKEN") {
            let token = raw.trim().to_ascii_lowercase();
            if KNOWN_SAMPLE_TOKENS.iter().any(|sample| *sample == token) {
                eprintln!(
                    "\n  refusing to start: PM_PUBLIC=1 with the published sample SERVER_TOKEN.\n  \
                     That token is an ADMIN credential once the addon surface is public.\n  \
                     Set a fresh SERVER_TOKEN (or leave it unset and let the server mint one).\n"
                );
                std::process::exit(2);
            }
        }
    }
    // DEFAULT: 4 connections, on every platform.
    //
    // `workers Ã— inflight` is the number of `upload.getFile` parts the engine can
    // have in flight â€” a SHARED budget for every viewer, not a per-viewer one.
    // With the old default of ONE connection that budget was 8 parts for the whole
    // process, so a single player that seeks once while another stream is open kept
    // the route queued: a 55-request real session at `--workers 1` logged repeated
    // `download refused` / `pre-flight budget exhausted`, while the same session at
    // `--workers 4` logged 55/55 served with `timed out: 0`, `gate saturated: 0`,
    // `attempt failed: 0`, `download refused: 0` (Telegram's `FLOOD_WAIT` 1-3 s on
    // individual parts is absorbed by the per-part retry path). 4 x 8 = 32 parts is
    // also the shape the engine's own inflight note describes as correct â€” the TS
    // port holds 200/200 with 4 sessions x 4 in flight â€” and `grammers` multiplexes
    // a whole DC over ONE connection, so parallel connections are the only way to
    // scale a busy host.
    //
    // Measured on a phone (CPH2581, API 36) as well as on desktop: 8/8 sequential
    // and 4/4 concurrent 512 KB reads succeed at the default, so this does not cost
    // correctness on Android. `PM_WORKERS` overrides it, matching the other knobs
    // (`PM_INFLIGHT`, `PM_LISTEN_PORT`); a handset can still pass `--workers 1`.
    let workers: usize = arg_value("--workers")
        .or_else(|| storage::env("PM_WORKERS"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_WORKERS)
        .max(1);
    // Telegram's own guidance is N connections x a handful of in-flight parts
    // per connection. grammers multiplexes a whole DC over ONE connection, so
    // 200 concurrent `upload.getFile` on one `Client` get reset by the DC
    // (measured: 914 x "Connection reset by peer (os error 104)"). The default
    // 8 is the shape the TS port holds 200/200 with (4 sessions x 4 in flight).
    //
    // 2026-11-25: default lowered 8 -> 3. Telegram does not refuse the extra
    // requests, it silently stops answering on that session â€” measured on a
    // one-bot install as `part stalled: no part in 12001ms with 12 of 12 permits
    // free`, i.e. OUR gate was idle while the server had gone quiet (502 reports
    // across 140 devices). MadelineProto reached the same conclusion in
    // danog/MadelineProto#1948 and now puts at most 2 file requests on a media
    // socket, learning the real limit per DC. grammers multiplexes a DC over one
    // connection, so `inflight` IS that per-socket count: 4x3 = 12 parts in flight
    // is the conservative shape that keeps a normal install playing.
    let inflight: usize = arg_value("--inflight")
        .or_else(|| storage::env("PM_INFLIGHT"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_INFLIGHT)
        .max(1);
    // How many Telegram parts ONE `/api/download` response keeps in flight.
    //
    // NOT a second budget: `workers x inflight` (32 at the defaults) stays the
    // process-wide part budget, enforced by the per-connection gate and the bot's
    // learned window. This is only what a single response ASKS for, which is what
    // stops one viewer being latency-bound at one 512 KB part per round trip
    // (measured 0.31 MB/s sequentially against 3.74 MB/s aggregate for four
    // concurrent streams on the same bot). 20 is MadelineProto's
    // `downloadParallelChunks`; `1` is the sequential escape hatch and the A/B
    // baseline, and the value is printed in the banner so a report shows it.
    let mut stream_prefetch: usize = arg_value("--stream-prefetch")
        .or_else(|| storage::env("PM_STREAM_PREFETCH"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(download::DEFAULT_STREAM_PREFETCH)
        // NOT clamped to `MAX_STREAM_PREFETCH` here: that is ONE lane's budget,
        // and `download` clamps at READ time against `MAX x lanes`, because the
        // lane count is only known after the registry is published. Clamping
        // here pinned every multi-lane body at 32 parts — 8 per lane — which is
        // exactly where each lane's learned window stopped growing.
        .clamp(1, download::MAX_STREAM_PREFETCH.saturating_mul(16));
    // Global cap on concurrent download responses: beyond it viewers get a
    // fast `503 / retry_after: 1` instead of all opening a Telegram part at once.
    //
    // DEFAULT IS 0 = NO CAP, because this product is self-hosted: on a desktop,
    // an Android box or a VPS the real throttle is already the Telegram pool
    // (`workers Ã— inflight` parts), and there is no 30 s router to kill queued
    // requests. A cap there only turns "slightly slower" into "refused".
    //
    // Hosts that DO have a short request timeout should set it explicitly: on
    // Heroku a 1Ã— Basic dyno measured 250/400 served with `PM_MAX_STREAMS=256`
    // (clean JSON 503 + retry_after for the rest) versus 312/400 uncapped where
    // the 88 losers died on `H12` after 30 s and got an HTML error page.
    //
    // NOTE: `.max(1)` used to sit here and turned an explicit `0` into a gate of
    // ONE â€” one viewer, everyone else 503. Do not put it back.
    let max_streams: usize = arg_value("--max-streams")
        .or_else(|| storage::env("PM_MAX_STREAMS"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let gate_permits = if max_streams == 0 {
        tokio::sync::Semaphore::MAX_PERMITS
    } else {
        max_streams
    };

    // Per-client abuse limits on `/api/download` (`ratelimit.rs`).
    //
    // `--max-titles-per-ip` counts DISTINCT short codes in flight, NOT sockets. A
    // seek, a 16-connection download manager and a preload of the next episode all
    // operate on the same code and therefore all cost one; 40 different titles in
    // parallel cost 40, which is the scraper signature. Without that distinction a
    // capability limit would break segmented downloads â€” see the tests in
    // `ratelimit::tests`.
    //
    // `--max-bodies-per-ip` is the socket-flood backstop and sits above every real
    // download manager (aria2 defaults to 16, IDM to 8).
    //
    // `--cold-starts-per-ip` is charged only where a resolve is actually issued, so
    // a `Range` request, a seek and a warm short code are free.
    //
    // `0` disables a counter entirely â€” one integer compare, no map touched. A
    // self-hosted box that wants today's behaviour exactly sets all of them to 0.
    let max_titles_per_ip: usize = arg_value("--max-titles-per-ip")
        .or_else(|| storage::env("PM_MAX_TITLES_PER_IP"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    let max_bodies_per_ip: usize = arg_value("--max-bodies-per-ip")
        .or_else(|| storage::env("PM_MAX_BODIES_PER_IP"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(32);
    let cold_starts_per_ip: usize = arg_value("--cold-starts-per-ip")
        .or_else(|| storage::env("PM_COLD_STARTS_PER_IP"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    let cold_start_window: u64 = arg_value("--cold-start-window")
        .or_else(|| storage::env("PM_COLD_START_WINDOW"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(300);

    // The per-response read-ahead is published AFTER the lane count is known —
    // see the `set_stream_prefetch` call below. Printed in the banner below, so a
    // throughput report always names the depth it was measured at.

    // ── api_id lanes ────────────────────────────────────────────────────────
    //
    // ONE process, several Telegram applications. Measured on the VPS
    // (2026-11-26): four api_ids in four processes did 37.9 MB/s while one lane
    // never passed ~11, and two bots on ONE api_id added up to nothing (5.74
    // combined). So a second lane is only worth opening on a DIFFERENT api_id —
    // which is what `api_ids.json` holds (or the legacy credential, alone).
    //
    // An install with one api_id gets exactly one lane, whose pool IS the process
    // pool, so every existing behaviour (candidate order, mint, provisioning) is
    // untouched and `download.rs` keeps its single-lane path.
    let lane_workers: usize = arg_value("--lane-workers")
        .or_else(|| storage::env("PM_LANE_WORKERS"))
        .and_then(|s| s.parse().ok())
        // 10 per bot, the same as `DEFAULT_WORKERS`: a lane IS a bot, and its
        // `workers x inflight` (20) is the window ceiling the dispatcher relies
        // on. 16 x 2 per lane over-subscribed the DC (measured 2026-11-27).
        .unwrap_or(10)
        .max(1);
    let stream_lanes: Option<usize> = match arg_value("--stream-lanes")
        .or_else(|| storage::env("PM_STREAM_LANES"))
    {
        Some(v) if !v.is_empty() && !v.eq_ignore_ascii_case("auto") => v.parse::<usize>().ok(),
        _ => None,
    };
    // The default api_id pool comes from the origin's `GET /api-ids`, and that is
    // the ONLY source: one list, one authority, so an operator who disables an id
    // in wp-admin sees it stop everywhere. The result is cached for 10 minutes, so
    // a restart loop does not hammer the origin; a stale/unreachable origin keeps
    // the previous cache. Bounded to 6 s and best effort — the listener must not
    // wait on WordPress.
    if !lanes::origin_cache_fresh(600) {
        tokio::time::timeout(
            std::time::Duration::from_secs(6),
            lanes::refresh_origin_cache(),
        )
        .await
        .ok();
    }
    let lane_entries = lanes::load_entries();
    let lane_specs: Vec<(i32, String)> = lane_entries
        .iter()
        .filter(|e| e.enabled)
        .map(|e| (e.api_id, e.api_hash.clone()))
        .collect();
    let default_spec = lane_specs.first().cloned();
    let default_api_id = default_spec.as_ref().map(|s| s.0).unwrap_or(0);

    // The PROCESS pool is lane 1, and it carries that lane's credentials. With
    // lanes configured, letting `boot()` re-read the credential file could sign
    // lane 1's bot in under a DIFFERENT api_id than the dispatcher believes —
    // i.e. two lanes quietly sharing one ceiling. `None` (no credential at all)
    // keeps the legacy path, where boot fetches them from the origin.
    let telegram_pool = match &default_spec {
        Some((api_id, hash)) => Arc::new(telegram::TelegramPool::for_lane(
            *api_id,
            hash,
            workers,
            inflight,
        )),
        None => Arc::new(telegram::TelegramPool::new(workers, inflight)),
    };
    // Published once, before the first request can resolve a short code: the mint
    // that `resolve.rs` performs goes through this same pool.
    telegram::share_pool(telegram_pool.clone());

    let default_lane =
        lanes::Lane::new(default_api_id, telegram_pool.clone(), &session::get_bot_id());
    let mut extra_lanes: Vec<std::sync::Arc<lanes::Lane>> = Vec::new();
    for (api_id, hash) in lane_specs.iter().skip(1) {
        let stored_bot = lane_entries
            .iter()
            .find(|e| e.api_id == *api_id)
            .map(|e| e.bot_id.clone())
            .unwrap_or_default();
        let pool = Arc::new(telegram::TelegramPool::for_lane(
            *api_id,
            hash,
            lane_workers,
            inflight,
        ));
        extra_lanes.push(lanes::Lane::new(*api_id, pool, &stored_bot));
    }
    // `--stream-lanes N` forces a subset (1 = the legacy single-lane path).
    if let Some(n) = stream_lanes {
        extra_lanes.truncate(n.saturating_sub(1));
    }
    let lane_count = 1 + extra_lanes.len();
    lanes::publish(lanes::LaneRegistry::new(default_lane, extra_lanes));

    // Publish the read-ahead now that the lane count is known.
    //
    // The default is a depth PER LANE: one global 20 over four lanes is 5 parts
    // each, below what a lane's learned window needs to grow (it only grows while
    // FULL), so a multi-lane install would default to a shape that cannot use its
    // lanes. An explicit `--stream-prefetch` / PM_STREAM_PREFETCH is left alone.
    if lane_count > 1
        && arg_value("--stream-prefetch").is_none()
        && storage::env("PM_STREAM_PREFETCH").is_none()
    {
        stream_prefetch = download::DEFAULT_STREAM_PREFETCH.saturating_mul(lane_count);
    }
    download::set_stream_prefetch(stream_prefetch);
    if lane_count > 1 {
        log::stderr_log(
            "api_id lanes configured",
            Some(&serde_json::json!({
                "lanes": lane_count,
                "lane_workers": lane_workers,
                "parts_per_lane": lane_workers * inflight,
                "parts_total": lane_count * lane_workers * inflight,
            })),
        );
    }
    let state = Arc::new(AppState {
        port,
        workers,
        inflight,
        download_gate: Arc::new(tokio::sync::Semaphore::new(gate_permits)),
        max_streams,
        limits: ratelimit::Limiter::new(),
        max_titles_per_ip,
        max_bodies_per_ip,
        cold_starts_per_ip,
        cold_start_window,
        telegram: telegram_pool,
    });
    storage::prune_cache_files(false);

    // Bring every env-supplied pool bot up before the first request, so each
    // instance holds a session for all of them and `pick_pool_bot()` can spread
    // downloads across the pool. Detached and best-effort: a slow or expired
    // token must never delay the listener. Why an env list at all is explained on
    // `session::env_bot_tokens()` â€” the pool file is per-instance disk state.
    {
        let boot_state = state.clone();
        tokio::spawn(async move {
            let tokens = session::env_bot_tokens();
            if tokens.is_empty() {
                return;
            }
            log::stderr_log(
                "booting env bot pool",
                Some(&serde_json::json!({ "bots": tokens.len() })),
            );
            for (idx, token) in tokens.into_iter().enumerate() {
                let id = token.split(':').next().unwrap_or("").trim().to_string();
                if id.is_empty() {
                    continue;
                }
                match boot_state.telegram.boot(&id, Some(&token)).await {
                    Ok(entry) => {
                        // Real @username/name for the pool list (see
                        // `session::remember_env_identity`).
                        session::remember_env_identity(
                            &entry.identity.id,
                            &entry.identity.username,
                            &entry.identity.first_name,
                        );
                        if idx == 0 && session::get_bot_id().is_empty() {
                            session::save_session_meta(
                                &entry.identity.id,
                                &entry.identity.username,
                                &entry.identity.first_name,
                            );
                        }
                        // Spread the operator's bots over the api_id lanes: one bot
                        // per lane is what makes the lane useful, and a bot can only
                        // serve the api_id it signed in with.
                        let bound = lanes::bind_operator_bot(
                            &entry.identity.id,
                            &entry.identity.username,
                            &entry.identity.first_name,
                        );
                        if bound > 0 {
                            log::stderr_log(
                                "env bot bound to an api_id lane",
                                Some(&serde_json::json!({
                                    "bot_id": entry.identity.id,
                                    "api_id": bound,
                                })),
                            );
                        }
                        log::stderr_log(
                            "env pool bot ready",
                            Some(&serde_json::json!({
                                "bot_id": entry.identity.id,
                                "username": entry.identity.username,
                            })),
                        );
                    }
                    Err(e) => log::stderr_log(
                        "env pool bot failed",
                        Some(&serde_json::json!({ "bot_id": id, "error": e })),
                    ),
                }
            }
        });
    }

    // Enrol for PM-Auth BEFORE anything talks to the origin.
    //
    // `/provision-session` used to be the only enrolment path, so an install that
    // already held a bot session never got a token and authenticated with the
    // fleet-wide `X-API-Secret` instead. Retiring that secret would therefore have
    // broken playback on every existing install â€” including this build â€” not just
    // on old ones. Detached: a slow origin must not delay the listener.
    {
        tokio::spawn(async move {
            let outcome = credentials::ensure_enrolled().await;
            // `outcome` apart from `enrolled`: "the origin was unreachable" and
            // "the origin refused this install" are the same `false` otherwise, and
            // they send an operator in opposite directions.
            log::stderr_log(
                "pm-auth enrolment",
                Some(&serde_json::json!({
                    "enrolled": outcome == credentials::EnrolOutcome::Enrolled,
                    "outcome": format!("{outcome:?}"),
                    "bot_id": session::get_bot_id(),
                })),
            );
        });
    }

    // Make sure this instance holds ONE guest bot. Detached for the same reason as
    // the warm above: a lease is an HTTP round-trip plus a Telegram login and must
    // not delay the listener. One per instance is a hard rule (a pooled fleet may
    // not multiply our provisioning â€” see `session::GUEST_BOTS_PER_INSTANCE`);
    // anyone who wants a bigger pool adds their own bots via `BOT_TOKENS`.
    {
        let pool_state = state.clone();
        let lanes_total = lane_count;
        tokio::spawn(async move {
            let have = session::ensure_guest_bot(&pool_state.telegram).await;
            log::stderr_log(
                "guest bot ready",
                Some(&serde_json::json!({ "bots": have })),
            );
            // The default lane follows the ACTIVE bot, which a fresh provision
            // may have just replaced: a lane pinned to the old id would boot a
            // dead session on every part.
            if let Some(reg) = lanes::registry() {
                let active = session::get_bot_id();
                if !active.is_empty() {
                    let lane = reg.default_lane();
                    lane.set_bot_id(&active);
                    reg.register_bot(&active, &lane);
                    // Persist it as the lane's bot too: `api_ids.json` is what
                    // decides the bot belongs to the api_id pool, and without this
                    // the same bot kept appearing in BOTH lists.
                    let meta = session::load_session_meta();
                    lanes::set_lane_bot_identity(
                        lane.api_id,
                        &active,
                        meta.get("bot_username").and_then(|v| v.as_str()).unwrap_or(""),
                        meta.get("bot_name").and_then(|v| v.as_str()).unwrap_or(""),
                    );
                }
            }
            // Extra lanes, AFTER the default one: a lane lease is an origin
            // round-trip plus a Telegram login, and the two are independent, so
            // they run in parallel with each other but not with the install's
            // own bot (which the origin has just counted).
            let Some(reg) = lanes::registry() else {
                return;
            };
            let all = reg.lanes();
            if all.len() < 2 {
                return;
            }
            let mut handles = Vec::new();
            for lane in all.into_iter().skip(1) {
                handles.push(tokio::spawn(async move {
                    match session::provision_lane(&lane).await {
                        Ok(res) => {
                            log::stderr_log(
                                "lane bot ready",
                                Some(&serde_json::json!({
                                    "api_id": lane.api_id,
                                    "bot_id": res.bot_id,
                                    "username": res.bot_username,
                                })),
                            );
                        }
                        Err(e) => log::stderr_log(
                            "lane provision failed",
                            Some(&serde_json::json!({"api_id": lane.api_id, "error": e})),
                        ),
                    }
                }));
            }
            for h in handles {
                let _ = h.await;
            }
            let ready = lanes::registry()
                .map(|r| r.lanes().iter().filter(|l| l.has_bot()).count())
                .unwrap_or(0);
            log::stderr_log(
                "api_id lanes ready",
                Some(&serde_json::json!({ "ready": ready, "lanes": lanes_total })),
            );
        });
    }

    let _ = STATE.set(state.clone());
    // One fallback handles every path, exactly like backend.php's dispatch block.
    let app = Router::new().fallback(serve_all);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    // Double-clicking the executable must not end in a red error: if the port is
    // already taken, that is almost always a server this user started earlier, so
    // open it instead of failing.
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            let url = format!("http://127.0.0.1:{port}/");
            println!("\n  PencariMovie Server is already running on {url}");
            println!("  Opening it in your browser instead of starting a second copy.");
            println!("  To restart it, stop the running one first (pms stop, stop.bat, or close its window).\n");
            open_browser(&url);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    log::stderr_log(
        "PencariMovie Server listening",
        Some(&serde_json::json!({
            "port": port,
            "host": host,
            "workers": workers,
            "inflight": inflight,
            "max_streams": max_streams,
            // The per-client caps, reported so an operator (and a bug report)
            // shows what is actually enforced instead of what the docs claim.
            "max_titles_per_ip": state.max_titles_per_ip,
            "max_bodies_per_ip": state.max_bodies_per_ip,
            "cold_starts_per_ip": state.cold_starts_per_ip,
            "cold_start_window": state.cold_start_window,
            "public": public,
            "public_configure": if public { "/configure" } else { "" },
            "public_admin": if public { "/admin" } else { "" },
            "storage": storage::storage_dir().display().to_string(),
            "version": config::app_version(),
            "runtime": format!("pencarimovie-server {}", config::app_version()),
            "target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        })),
    );
    // `--host 0.0.0.0` (Android, Docker, PaaS) is not reachable as 0.0.0.0, so
    // print the address a browser can actually use.
    let shown = if host == "0.0.0.0" || host == "::" {
        "127.0.0.1".to_string()
    } else {
        host.clone()
    };
    // Decide the tray ONCE, here, before anything is printed.
    //
    // `--tray` forces it; `--no-tray` / `PM_NO_TRAY` forbid it; otherwise a
    // human-launched console gets one. Linux and Windows share this decision via
    // `tray_impl`, and Android never does.
    let console_is_terminal =
        !cfg!(target_os = "android") && std::io::stdout().is_terminal();
    #[cfg(not(target_os = "android"))]
    let tray_requested = tray_impl::wanted(console_is_terminal);
    #[cfg(target_os = "android")]
    let tray_requested = false;

    // If a tray icon is going to be created, this console is about to be released
    // (`FreeConsole`), so a banner here would only flash and vanish. The same
    // information already goes to storage/server.log (tray "Open log"), and it has
    // already gone to stderr above in the "listening" line.
    // Lanes only print when there is more than one: a single-lane install has
    // nothing to say on the subject, and the line would read as a new knob.
    let lanes_banner = if lane_count > 1 {
        format!(
            "\n  Lanes:       {} api_id(s) x {} workers = {} parts in flight",
            lane_count,
            lane_workers,
            lane_count * lane_workers * inflight
        )
    } else {
        String::new()
    };
    if !tray_requested {
        println!(
            "\n  PencariMovie Server\n  Local:       http://{}:{}\n  Listening:   {}:{}\n  Storage:     {}\n  Workers:     {} (one Telegram connection each)\n  Inflight:    {} parts per connection ({} total){}\n  Prefetch:    {} part(s) in flight per response\n  Max streams: {}\n  Public mode: {}\n  Per client:  {} title(s) and {} connection(s) at once, {} new stream(s) per {}s\n",
            shown,
            port,
            host,
            port,
            storage::storage_dir().display(),
            workers,
            inflight,
            workers * inflight,
            lanes_banner,
            stream_prefetch,
            max_streams,
            if public {
                "ON  (users: /configure, admin: /admin)"
            } else {
                "off"
            },
            // Counted per TITLE, not per socket: a seek, a segmented download and a
            // preload of the same file all cost one. `0` = that counter is off.
            cap_or_off(max_titles_per_ip),
            cap_or_off(max_bodies_per_ip),
            cap_or_off(cold_starts_per_ip),
            cold_start_window,
            // Local dashboard, LAN devices and a tunnel's own loopback dial are exempt.
        );
        if host == "0.0.0.0" || host == "::" {
            println!("  LAN/TV:      http://<this-device-ip>:{}\n", port);
        }
    }

    // A user who double-clicks the exe expects a browser, not a black window with
    // logs. A terminal was inherited only when a human launched it, so that is the
    // signal; `--open` forces it, `--no-open` suppresses it (autostart, service
    // managers, scripts).
    //
    // `--tray` is background mode and implies `--no-open`: no browser, because
    // start-tray and the login entry are "start it and get out of the way". A
    // double-click still opens the browser â€” that is what opening the app means â€”
    // it just also drops the console once the icon exists.
    let url = format!("http://{shown}:{port}/");
    let tray_flag = arg_pos("--tray").is_some();
    let interactive = !cfg!(target_os = "android")
        && !paas_env_present()
        && !tray_flag
        && arg_pos("--no-open").is_none()
        && (arg_pos("--open").is_some() || console_is_terminal);
    if interactive {
        if !tray_requested {
            println!("  Close this window (or press Ctrl+C) to stop the server.\n");
        }
        open_browser(&url);
    }

    // The icon is this process, not a script beside it: `--tray` (start-tray,
    // the login entry) or a plain double-click. On Windows it must be the same
    // process because the shell stores tray settings per HOSTING executable â€” the
    // old PowerShell-hosted icon shared `powershell.exe`'s entry and could only
    // ever sit in the hidden-icons flyout.
    #[cfg(not(target_os = "android"))]
    if tray_requested {
        tray_impl::spawn(port, &host);
    }

    let shutdown = shutdown_signal();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await?;
    // The cache writes behind a 400 ms buffer; commit whatever is still queued so a
    // clean stop does not lose the last window.
    let flushed = cache::flush();
    if flushed > 0 {
        log::log("cache flushed on shutdown", &serde_json::json!({"entries": flushed}));
    }
    telegram::disconnect_all();
    Ok(())
}

fn rustc_version() -> &'static str {
    option_env!("RUSTC_VERSION").unwrap_or("unknown")
}

/// Open `url` in the user's default browser. Best effort: a box with no browser
/// (server install, headless CI) must never stop the server from running, so a
/// failure is only logged.
pub(crate) fn open_browser(url: &str) {
    // `proc::cmd`, not `Command::new`: a plain `cmd /C start` gets its own console,
    // so launching the server flashed a command prompt before the browser opened.
    let spawned = if cfg!(target_os = "windows") {
        // `start` is a cmd builtin, and its first quoted argument is the window
        // TITLE - passing an empty "" is what makes the second one the URL.
        proc::cmd("cmd")
            .args(["/C", "start", "", url])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    } else if cfg!(target_os = "macos") {
        proc::cmd("open")
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    } else {
        proc::cmd("xdg-open")
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    };
    match spawned {
        Ok(mut child) => {
            // `xdg-open` blocks until the browser exits; reap it off the runtime.
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => log::log(
            "could not open a browser",
            &serde_json::json!({"url": url, "error": e.to_string()}),
        ),
    }
}

/// One-line build identity, printed by `--version`:
/// `pencarimovie-server 1.0.50-3-g9fc84e1 (build 9fc84e1, rustc 1.99.1, x86_64-windows)`.
/// The first field is what this install reports (`/api/version` â†’ `current_version`);
/// the second is the git state the binary was compiled from.
fn version_line() -> String {
    // `rustc` is only reported when it can actually be detected: a packaged
    // build has no toolchain on PATH, and "rustc unknown" is noise.
    let rustc = rustc_version();
    let compiler = if rustc.is_empty() || rustc == "unknown" {
        String::new()
    } else {
        format!("rustc {}, ", rustc)
    };
    format!(
        "pencarimovie-server {} (build {}, {}{}-{})",
        config::app_version(),
        config::build_id(),
        compiler,
        std::env::consts::ARCH,
        std::env::consts::OS
    )
}

/// The default bind address.
///
/// `127.0.0.1` is the documented desktop default (Windows, Linux, macOS) â€” do
/// not change it, `stop.bat` / `stop.sh` and every doc assume a loopback
/// listener. Two deployments need `0.0.0.0` instead:
///
/// - **Android / Termux**: a phone or TV box is only useful when the TV (or
///   another device on the LAN) can reach it, and there is no host firewall to
///   hide behind.
/// - **PaaS / Docker**: every PaaS (Heroku, Render, Fly, Railway, Cloud Run,
///   Azure) routes traffic to the container's external interface; a `127.0.0.1`
///   listener is unreachable and the health check fails.
///
/// `--host`, `PM_LISTEN_HOST` and `HOST` all override this.
pub fn default_bind_host(_is_android: bool, _paas: bool) -> &'static str {
    "0.0.0.0"
}

/// The one-click Heroku/Render templates ship this token in a public repository.
/// In public mode it is an ADMIN credential, so it must never be accepted as
/// configured; refusing to boot is the only safe answer.
const KNOWN_SAMPLE_TOKENS: [&str; 2] = [
    "66ff097863dedb7ccd8401d5ffe616f2",
    "66ff097863dedb7ccd8401d5ffe616f3",
];

/// Environment markers a PaaS sets that a desktop never does.
///
/// `PORT` is deliberately NOT one: the PHP launchers set `PORT=8088` on a
/// desktop, and treating it as "public" would silently expose a LAN listener.
const PAAS_MARKERS: [&str; 8] = [
    "DYNO",                  // Heroku
    "RENDER",                // Render
    "FLY_APP_NAME",          // Fly.io
    "RAILWAY_ENVIRONMENT",   // Railway
    "K_SERVICE",             // Google Cloud Run
    "WEBSITE_SITE_NAME",     // Azure App Service
    "PM_PAAS",               // explicit opt-in for any other container host
    "PM_BIND_PUBLIC",        // explicit opt-in on a VPS
];

/// Pure so it can be tested without mutating the process environment.
pub fn paas_env_predicate(get: impl Fn(&str) -> bool) -> bool {
    PAAS_MARKERS.iter().any(|k| get(k))
}

pub fn paas_env_present() -> bool {
    paas_env_predicate(|k| storage::env(k).is_some())
}

/// Ctrl-C everywhere, plus SIGTERM on Unix.
///
/// `tokio::signal::unix` is `#[cfg(unix)]` inside tokio, so the previous
/// unconditionally-unix version of this block was the ONLY thing that stopped
/// the Windows (MSVC) build â€” every dependency, including grammers, compiled.
/// Windows has no SIGTERM: `stop.bat` kills the listener and a console close
/// event is not interceptable from a background process, so Ctrl-C is the
/// graceful path there.
#[cfg(unix)]
async fn shutdown_signal() {
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term.recv() => {}
        _ = shutdown_trigger().notified() => {}
    }
}

#[cfg(windows)]
async fn shutdown_signal() {
    // Ctrl-C, or the tray's "Stop server" (which goes through the same graceful
    // path, so the cache still flushes and the icon is removed).
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = shutdown_trigger().notified() => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_flag_accepts_the_documented_truthy_values() {
        for v in ["1", "true", "TRUE", "on", " yes "] {
            assert!(config::flag_value(v), "{v:?} must enable public mode");
        }
        for v in ["0", "false", "off", "no", ""] {
            assert!(!config::flag_value(v), "{v:?} must not enable public mode");
        }
    }

    #[test]
    fn default_bind_host_is_all_interfaces_on_all_platforms() {
        assert_eq!(default_bind_host(false, false), "0.0.0.0");
        assert_eq!(default_bind_host(true, false), "0.0.0.0");
        assert_eq!(default_bind_host(false, true), "0.0.0.0");
        assert_eq!(default_bind_host(true, true), "0.0.0.0");
    }

    #[test]
    fn port_alone_is_not_a_paas_marker() {
        // The PHP launchers export PORT=8088 on a desktop; that must not turn a
        // loopback listener into a LAN-exposed one.
        assert!(!paas_env_predicate(|k| k == "PORT"));
        assert!(!paas_env_predicate(|_| false));
    }

    #[test]
    fn every_paas_marker_is_recognised() {
        for marker in PAAS_MARKERS {
            assert!(
                paas_env_predicate(|k| k == marker),
                "{marker} must select 0.0.0.0"
            );
        }
    }

    /// The candidate walk's list: every DISTINCT id form in the engine's order,
    /// empties dropped and duplicate values collapsed (so a form is never re-sent),
    /// which is what lets one refusal fall through to the next form.
    #[test]
    fn id_candidates_keep_every_distinct_form_in_order() {
        let all = serde_json::json!({
            "file_id_mt": "MT", "file_id_tmp": "TMP", "file_id": "PLAIN"
        });
        assert_eq!(
            resolve_id_candidates(&all),
            vec![
                ("file_id_mt", "MT".to_string()),
                ("file_id_tmp", "TMP".to_string()),
                ("file_id", "PLAIN".to_string()),
            ]
        );
        // The usual answer: `file_id_mt` and `file_id` are the SAME value, so the
        // walk has exactly one candidate and sends once.
        let same = serde_json::json!({"file_id_mt": "AQADx", "file_id": "AQADx"});
        assert_eq!(
            resolve_id_candidates(&same),
            vec![("file_id_mt", "AQADx".to_string())]
        );
        // A reply that only carries the temp form still yields it, trimmed.
        let tmp = serde_json::json!({"file_id_tmp": " TMP ", "file_id": ""});
        assert_eq!(
            resolve_id_candidates(&tmp),
            vec![("file_id_tmp", "TMP".to_string())]
        );
        // No id at all: nothing to try (the row answers `resolve_failed`).
        assert!(resolve_id_candidates(&serde_json::json!({"ok": 0})).is_empty());
        assert!(resolve_id_candidates(&serde_json::json!({
            "file_id_mt": "  ", "file_id_tmp": "", "file_id": null
        }))
        .is_empty());
    }

    /// CHANGE 2: `--mint-streams` reads a REAL `/stream` list, so the extractor
    /// has to accept every shape that list has published â€” in this preference
    /// order, per row: `short_code=` in the url, then a `pm_file_` id, then an
    /// `/api/download/<payload>` segment.
    #[test]
    fn stream_codes_reads_the_three_shapes_in_order_of_preference() {
        let payload = serde_json::json!({"short_code": "b64code", "file_name": "Movie.mkv"});
        let b64 = crate::fileid::base64url_encode(payload.to_string().as_bytes());
        let body = serde_json::json!({
            "streams": [
                {"name": "x", "url": "http://127.0.0.1:8088/stream/movie/pm%3Apost%3A1.json?short_code=querycode&x=1"},
                {"id": "pm_file_pmcode", "name": "y", "url": "http://host/"},
                {"name": "z", "url": format!("http://127.0.0.1:8088/api/download/{}/Movie.mkv", b64)},
                {"name": "no-code", "url": "http://host/nothing"}
            ]
        });
        assert_eq!(stream_codes(&body), vec!["querycode", "pmcode", "b64code"]);
    }

    /// Order is the list's, duplicates collapse, and the hard cap holds â€” every
    /// code past it would be another real Telegram send.
    #[test]
    fn stream_codes_keeps_order_dedupes_and_caps_at_ten() {
        assert_eq!(MINT_STREAMS_CAP, 10);
        let streams: Vec<serde_json::Value> = (0..25)
            .map(|i| serde_json::json!({"url": format!("http://h/api/download?short_code=c{}", i % 12)}))
            .collect();
        let codes = stream_codes(&serde_json::json!({ "streams": streams }));
        assert_eq!(codes.len(), MINT_STREAMS_CAP);
        assert_eq!(codes.first().map(String::as_str), Some("c0"));
        assert_eq!(codes.last().map(String::as_str), Some("c9"));
        let mut unique = codes.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(unique.len(), codes.len(), "no code may appear twice");
    }

    /// A list with no usable code is empty, not a panic and not a guess.
    #[test]
    fn stream_codes_answer_nothing_for_a_list_without_codes() {
        assert!(stream_codes(&serde_json::json!({"streams": []})).is_empty());
        assert!(stream_codes(&serde_json::json!({"streams": [{"name": "x"}]})).is_empty());
        assert!(stream_codes(&serde_json::json!({"ok": 1})).is_empty());
    }

    #[test]
    fn version_line_names_this_target() {
        let line = version_line();
        assert!(line.starts_with("pencarimovie-server "));
        let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
        assert!(line.contains(&target), "{line} must contain {target}");
    }
}
