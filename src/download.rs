//! Port of the download path: backend.php:1884 (payload decode), 11107-11126
//! (payload-from-path) and 19699-20360 (the `/api/download` handler), with
//! `downloadToBrowser()` replaced by grammers' `iter_download()`.
//!
//! The RETRY LADDER is the PHP's, not a simplification:
//! * `$maxDownloadAttempts = ($shortCode !== '') ? 4 : 1`;
//! * every reference failure re-resolves the short_code with the cache bypassed
//!   (`force=1&nocache=1`);
//! * when the fresh resolve returns the SAME id, walk the pool and try an
//!   alternate bot (`file reference expired, trying alternate bot from pool`);
//! * any of it only happens before the response body starts. Unlike the PHP we
//!   go one step further and only commit headers after the FIRST chunk arrives,
//!   so a failed first fetch can still answer a JSON error instead of leaving
//!   the client waiting on a `Content-Length` that never completes.
//!
//! Range handling: a Range start is rounded DOWN to 4096 and the leading bytes
//! are discarded; the chunk size is the largest `4096 * 2^k` (<= 512 KB) that
//! divides that offset and does not cross a 1 MB boundary, so every
//! `upload.getFile` call satisfies Telegram's alignment rules.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{StatusCode, header};
use axum::response::Response;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bytes::Bytes;
use futures_util::StreamExt;
use serde_json::Value;
use tokio_stream::wrappers::ReceiverStream;

use crate::auth::{ReqCtx, is_authenticated};
use crate::log::log;
use crate::respond::json_response;
use crate::resolve::{
    invalidate_resolve_cache, resolve_shortcode, resolve_shortcode_cached,
    resolve_shortcode_concurrent, resolve_shortcode_fresh, save_resolve_cache,
};
use crate::session::{
    auto_provision_guest, get_bot_id, has_local_session, is_guest_provision_in_progress,
    pick_pool_bot,
};
use crate::storage::now;
use std::sync::OnceLock;

/// Live transfer counters, read by `/api/diagnostics` and used by the Android
/// app to drive its notification indicator and battery saver from real activity
/// (the TDLib app's `markActive()` / `ACTIVE_LOCK_TIMEOUT_MS` design) instead of
/// holding a wake lock forever.
static ACTIVE_STREAMS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static ACTIVE_PREFLIGHTS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub fn active_streams() -> usize {
    ACTIVE_STREAMS.load(std::sync::atomic::Ordering::Relaxed)
}
pub fn active_preflights() -> usize {
    ACTIVE_PREFLIGHTS.load(std::sync::atomic::Ordering::Relaxed)
}

/// Decrements its counter when dropped, including on every early `return`
/// (client gone, Telegram error, task cancelled).
struct ActivityGuard(&'static std::sync::atomic::AtomicUsize);
impl ActivityGuard {
    fn new(c: &'static std::sync::atomic::AtomicUsize) -> Self {
        c.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        ActivityGuard(c)
    }
}
impl Drop for ActivityGuard {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Cap on ONE pre-flight attempt chain (resolve + retries + priming chunk), in
/// seconds.
///
/// Nothing in this phase has been written to the client yet, so a player that
/// gives up leaves a request doing work for the whole retry chain: with the
/// backoff below that reached ~60 s per abandoned attempt, and a looping player
/// then had several of them running at once. An abandoned pre-flight is capped
/// so the work ends quickly; a real client is unaffected because a working
/// resolve finishes in ~1-3 s.
///
/// The budget REFRESHES when an attempt makes progress (a re-resolve returned a
/// new `file_id`, or an alternate bot was found) — see `hard_deadline` below.
const PREFLIGHT_BUDGET_SECS: u64 = 20;

/// Absolute ceiling on the whole pre-flight, across however many budget refreshes
/// it earns, so a pathological resolve chain cannot hold a worker for minutes.
const PREFLIGHT_HARD_CAP_SECS: u64 = 45;

/// `file_id_mt` (preferred) or `file_id` from a resolve result, or `""`.
fn id_of(v: &Value) -> String {
    let a = jstr(v, "file_id_mt");
    if a.is_empty() {
        jstr(v, "file_id")
    } else {
        a
    }
}

/// How many bytes the FIRST response writes before it starts streaming.
///
/// A whole 1 MB window means the player waits for a megabyte before byte one
/// (~2.5 s at the throughput a phone sees), which reads as buffering after a
/// seek. 512 KB halves that wait and still costs exactly the SAME two parts for
/// window 0 (512 KB prime + 512 KB remainder), so steady-state throughput is
/// untouched — 256 KB was one part more per window 0 for no extra benefit.
const PRIME_BYTES: u64 = 512 * 1024;

/// `(skip0, prime_want)` — how much to pull from `aligned` so a body that must
/// start at `start` can be produced.
///
/// `aligned` sits up to 1 MB BELOW `start`, so priming with just `prime` bytes
/// threw the WHOLE prime away as `skip` on any seek/tail request, and the
/// follow-on windows then over-ran EOF: the response promised a `Content-Length`
/// and delivered **0 bytes**. Measured on a real phone for a 3 229 198 462-byte
/// file:
///
/// ```text
/// -r 3229130462-   (68 KB to EOF) -> 206, content-length 68000, 0 bytes in 25 s
/// -r 3229198459-   (final 3 bytes) -> 206, content-length 3,    0 bytes in 25 s
/// -r 0-1048575                     -> 206, 1 048 576 bytes in 1.59 s
/// ```
///
/// Every player probes the tail before it starts (MP4 `moov`, Matroska `Cues`),
/// so that is what made playback stall right after the first request.
fn prime_window(start: u64, aligned: u64, total: u64, file_size: u64) -> (u64, u64) {
    let prime = total.min(PRIME_BYTES);
    let skip0 = start.saturating_sub(aligned);

    // Round the head up to a whole part (`MAX_CHUNK`, 512 KB — the largest part
    // grammers' parser can accept), then never beyond one 1 MB block.
    //
    // This is not cosmetic. `plan_limit()` can only pick a big part when the
    // position DIVIDES by it, so a head that ends on an arbitrary offset leaves
    // the next window at an offset only 4 KB can divide — and that window is then
    // fetched as 128-256 tiny parts instead of one. Measured on a seek
    // (`start = aligned + 1`, 8 MB): **245 KB/s**, against 2.9 MB/s for the same
    // request one byte earlier. Rounding the head keeps every following window at
    // a part-friendly offset.
    //
    // The grain is 512 KB rather than a whole part so the FIRST bytes of the body
    // are not delayed by a full-megabyte fetch; a head that lands on a 512 KB
    // offset is completed to the next 1 MB boundary by one extra 512 KB window in
    // `handle_download()` (`p % mb != 0`).
    //
    // MadelineProto gets the same property for free: `downloadToCallable()` plans
    // `$x = $offset - $offset % $part_size` and steps by a whole `$part_size`
    // (1 MB), so only its FIRST and LAST parts are partial.
    let want = skip0.saturating_add(prime);
    let want = want
        .saturating_add(crate::telegram::MAX_CHUNK - 1)
        / crate::telegram::MAX_CHUNK
        * crate::telegram::MAX_CHUNK;
    let want = want
        .min(crate::telegram::MB)
        .min(file_size.saturating_sub(aligned));
    (skip0, want)
}

/// Fetch the preflight range in PARALLEL whenever it needs more than one part.
///
/// WHICH requests this affects: `prime_window()` hands back 512 KB only when
/// `start` sits exactly on a 1 MB boundary (byte 0, or a whole-MB offset). Every
/// other range, which is every seek and every partial read a player sends,
/// rounds the head up to 1 MB. `fetch_range_core()` holds ONE permit for the
/// whole call, so that head was two 512 KB parts fetched back to back and the
/// player waited for both before byte one. Measured here against a 406 MB 4K
/// source, 4 MB range windows at 1 MB-block offsets, median of 8 rounds: first
/// byte **954 ms serial, 723 ms split**. A byte-0 prefix still measures the same
/// on both builds, because it was never two parts to begin with
/// (`prefix_range_takes_no_skip`).
///
/// COST, stated because the part budget is shared by every viewer on the
/// connection: the split holds 2 permits at once instead of 1, for half as long.
/// Part-seconds are unchanged, so a neighbouring stream loses a slot for one RTT
/// rather than two. Under `--inflight 1` the two futures simply queue, and the
/// result is byte-identical to the serial fetch.
async fn fetch_prime_gated(
    client: &grammers_client::Client,
    gate: &tokio::sync::Semaphore,
    target: &DocTarget,
    pos: u64,
    want: u64,
) -> Result<Vec<u8>, crate::telegram::GramErr> {
    let (want, split) = prime_split(want);
    let Some((first_want, second_want)) = split else {
        return crate::telegram::fetch_range_gated(client, gate, target, pos, want).await;
    };
    // `pos` arrives 4096-aligned (it is `aligned`) and `MAX_CHUNK` is a whole
    // number of `MIN_CHUNK` units, so `second_pos` stays legal for `plan_limit()`.
    // Mis-align it and every following window collapses to 4 KB parts.
    let second_pos = pos.saturating_add(first_want);
    log(
        "preflight prime split across two parts",
        &serde_json::json!({"pos": pos, "first": first_want, "second": second_want}),
    );
    let (mut first, second) = tokio::try_join!(
        crate::telegram::fetch_range_gated(client, gate, target, pos, first_want),
        crate::telegram::fetch_range_gated(client, gate, target, second_pos, second_want),
    )?;
    first.extend_from_slice(&second);
    Ok(first)
}

/// `(clamped_want, Some((first, second)))`, or `(clamped_want, None)` when one
/// Telegram part already covers the head.
///
/// The 1 MB cap is applied HERE and not trusted from the caller. Without it, a
/// head above 1 MB would hand `fetch_range_core()` a `second` overrun, and it
/// answers an overrun by fetching the extra parts SERIALLY inside the same
/// permit: no error, no log, and the latency this function exists to remove is
/// back. `PRIME_BYTES` is the knob that would do it, so the guard sits next to
/// the arithmetic instead of riding on `prime_window()`'s `.min(MB)`.
fn prime_split(want: u64) -> (u64, Option<(u64, u64)>) {
    let want = want.min(crate::telegram::MB);
    if want <= crate::telegram::MAX_CHUNK {
        return (want, None);
    }
    (want, Some((crate::telegram::MAX_CHUNK, want - crate::telegram::MAX_CHUNK)))
}

/// One fetch window, clamped so it never asks past the last byte. A window that
/// over-runs EOF makes Telegram error and the body dies short of the
/// `Content-Length` already sent.
fn window(p: u64, want: u64, file_size: u64) -> Option<(u64, u64)> {
    let avail = file_size.saturating_sub(p);
    let w = want.min(avail);
    (w > 0).then_some((p, w))
}

/// How many 1 MB windows to keep in flight per stream (`PM_PIPELINE` overrides,
/// clamped to 1..=64).
///
/// **The part size is not tunable.** grammers' parser asserts
/// `len < MessageContainer::MAXIMUM_SIZE`
/// ([`grammers-mtproto-0.11.0/src/manual_tl.rs:75`](C:/Users/ewangtlex/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/grammers-mtproto-0.11.0/src/manual_tl.rs:75)),
/// where that limit is `1_044_456 - SIZE_OVERHEAD`, so a FULL 1 MB
/// `upload::GetFile` reply aborts the connection's runner
/// (`SenderPoolRunner was dropped during retry at send`) and 512 KB
/// (`telegram::MAX_CHUNK`) is a hard ceiling. MadelineProto's 1 MB parts — the
/// reason the PHP build felt faster — are out of reach without a custom MTProto
/// client, so concurrency is the only lever here.
///
/// Measured on a phone (OnePlus CPH2581, 32 MB range mid-file, same 4K MKV):
/// `1 x 8` (the old defaults) **0.81 MB/s**; `4 x 8` 1.44; `8 x 8` 1.56;
/// `1 x 32` 1.40-1.62; `8 x 32` 1.46-1.82; `16 x 32` 1.40 (over-subscribed).
/// **Pipeline depth is NOT a reliable knob**: `8 x 32` measured 1.93 MB/s with
/// `PM_PIPELINE=64` in one run and 0.52 MB/s with the same settings in the next —
/// the phone's Telegram path varies far more than the depth helps. Hence the
/// modest default of 16, and 8 connections x 32 parts as the app's setting.
fn pipeline_depth() -> usize {
    static DEPTH: OnceLock<usize> = OnceLock::new();
    *DEPTH.get_or_init(|| {
        crate::storage::env("PM_PIPELINE")
            .and_then(|v| v.trim().parse::<usize>().ok())
            .map(|v| v.clamp(1, 64))
            .unwrap_or(16)
    })
}
use crate::telegram::{DocTarget, prepare_file_target};
use crate::AppState;

/* ── payload ─────────────────────────────────────────────────────────────── */

pub fn decode_download_payload(payload: &str) -> Value {
    let mut s = payload.replace('-', "+").replace('_', "/");
    while s.len() % 4 != 0 {
        s.push('=');
    }
    match STANDARD.decode(s) {
        Ok(bytes) => serde_json::from_slice::<Value>(&bytes)
            .ok()
            .filter(|v| v.is_object())
            .unwrap_or_else(|| Value::Object(serde_json::Map::new())),
        Err(_) => Value::Object(serde_json::Map::new()),
    }
}

/// `fd_extract_download_payload_from_path()`: skip a segment that is a media name.
pub fn extract_payload_from_path(path: &str) -> String {
    let rest = match path.strip_prefix("/api/download/") {
        Some(r) => r,
        None => return String::new(),
    };
    let segment = rest.split('/').next().unwrap_or("");
    if segment.is_empty() {
        return String::new();
    }
    let decoded = percent_decode(segment);
    let ext = decoded
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();
    if is_media_extension(&ext) {
        return String::new();
    }
    decoded
}

/// Known media extensions.
pub const MEDIA_EXTS: [&str; 19] = [
    "mp4", "m4v", "mkv", "webm", "avi", "mov", "ts", "m2ts", "flv", "wmv", "3gp", "mpg", "mpeg",
    "mp3", "m4a", "flac", "wav", "ogg", "opus",
];

/// Known non-media extensions (software, installers, archives, documents).
/// Downloads matching these extensions are legitimate non-media files and MUST NEVER
/// be flagged as disguised binaries.
pub const NON_MEDIA_EXTS: [&str; 27] = [
    "exe", "msi", "bat", "cmd", "bin", "apk", "xapk", "apks", "zip", "rar", "7z", "tar",
    "gz", "bz2", "xz", "iso", "dmg", "pkg", "deb", "rpm", "pdf", "epub", "txt", "doc", "docx",
    "srt", "vtt",
];

pub fn is_media_extension(ext: &str) -> bool {
    let lower = ext.to_ascii_lowercase();
    MEDIA_EXTS.contains(&lower.as_str())
}

/// Check if a download target claims to be video or audio media.
pub fn is_media_target(file_name: &str, mime: &str) -> bool {
    let ext = file_name
        .rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default();

    if NON_MEDIA_EXTS.contains(&ext.as_str()) {
        return false;
    }

    is_media_extension(&ext)
        || mime.starts_with("video/")
        || mime.starts_with("audio/")
        || mime == "application/x-matroska"
}

/// Detect if a file claiming to be video/audio is actually an executable binary (Windows PE, Linux ELF, Mach-O).
/// Returns true ONLY when the file claims to be media AND the header bytes prove it is an executable binary.
pub fn is_disguised_executable(buf: &[u8], file_name: &str, mime: &str) -> bool {
    if buf.len() < 4 {
        return false;
    }

    if !is_media_target(file_name, mime) {
        return false;
    }

    // 1. Windows PE / DOS binary: starts with 'M','Z'
    if buf[0] == b'M' && buf[1] == b'Z' {
        if buf.len() >= 0x40 {
            let pe_offset = u32::from_le_bytes([buf[0x3C], buf[0x3D], buf[0x3E], buf[0x3F]]) as usize;
            if pe_offset + 4 <= buf.len() && &buf[pe_offset..pe_offset + 4] == b"PE\0\0" {
                return true;
            }
        }
        let haystack = &buf[..buf.len().min(512)];
        if haystack.windows(12).any(|w| w == b"This program") {
            return true;
        }
        // Claimed video file starting with MZ header is a disguised binary
        return true;
    }

    // 2. Linux ELF binary: \x7fELF
    if buf.len() >= 4 && &buf[0..4] == b"\x7fELF" {
        return true;
    }

    // 3. macOS Mach-O binary
    if buf.len() >= 4 {
        let magic = u32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
        if matches!(magic, 0xFEEDFACE | 0xFEEDFACF | 0xCEFAEDFE | 0xCFFAEDFE | 0xCAFEBABE) {
            return true;
        }
    }

    false
}

/// Asynchronously report a flagged/malicious file to WordPress REST API for soft-deletion from the index.
pub fn report_and_delete_flagged_file(short_code: &str, file_name: &str, reason: &str, bot_id: &str) {
    if short_code.trim().is_empty() {
        return;
    }
    let sc = short_code.trim().to_string();
    let fn_ = file_name.to_string();
    let r = reason.to_string();
    let b = bot_id.to_string();
    tokio::spawn(async move {
        let url = format!("{}/delete-file", crate::config::wp_api_base());
        let params = vec![
            ("short_code".to_string(), sc.clone()),
            ("file_name".to_string(), fn_),
            ("reason".to_string(), r),
            ("bot_id".to_string(), b),
        ];
        let res = crate::httpc::http_json(&url, &params, "POST", 10).await;
        log(
            "flagged file deletion reported to origin",
            &serde_json::json!({
                "short_code": sc,
                "response": res,
            }),
        );
    });
}

pub fn is_public_download_path(path: &str) -> bool {
    path == "/api/download" || path.starts_with("/api/download/")
}

pub fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (hexval(bytes[i + 1]), hexval(bytes[i + 2])) {
                out.push(h * 16 + l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn hexval(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/* ── Range ───────────────────────────────────────────────────────────────── */

#[derive(Debug, Clone, Copy)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

pub fn parse_range_header(header: &str, size: u64) -> Option<ByteRange> {
    if size == 0 {
        return None;
    }
    let spec = header.trim().strip_prefix("bytes=")?;
    let (raw_start, raw_end) = spec.split_once('-')?;
    let raw_start = raw_start.trim();
    let raw_end = raw_end.trim();
    if raw_start.is_empty() && raw_end.is_empty() {
        return None;
    }
    if raw_start.is_empty() {
        let n: u64 = raw_end.parse().ok()?;
        if n == 0 {
            return None;
        }
        let start = size.saturating_sub(n);
        return Some(ByteRange {
            start,
            end: size - 1,
        });
    }
    let start: u64 = raw_start.parse().ok()?;
    if start >= size {
        return None;
    }
    let mut end: u64 = if raw_end.is_empty() {
        size - 1
    } else {
        raw_end.parse().unwrap_or(size - 1)
    };
    if end >= size {
        end = size - 1;
    }
    if end < start {
        return None;
    }
    Some(ByteRange { start, end })
}

/* ── mime / filename helpers ─────────────────────────────────────────────── */

pub fn mime_by_ext(ext: &str) -> Option<&'static str> {
    Some(match ext {
        "mp4" | "m4v" => "video/mp4",
        "mkv" => "video/x-matroska",
        "webm" => "video/webm",
        "avi" => "video/x-msvideo",
        "mov" => "video/quicktime",
        "ts" | "m2ts" => "video/mp2t",
        "flv" => "video/x-flv",
        "wmv" => "video/x-ms-wmv",
        "3gp" => "video/3gpp",
        "mpg" | "mpeg" => "video/mpeg",
        "mp3" => "audio/mpeg",
        "m4a" => "audio/mp4",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "opus" => "audio/opus",
        _ => return None,
    })
}

pub fn ext_by_mime(mime: &str) -> &'static str {
    match mime {
        "video/mp4" => "mp4",
        "video/x-matroska" => "mkv",
        "video/webm" => "webm",
        "video/x-msvideo" => "avi",
        "video/quicktime" => "mov",
        "video/mp2t" => "ts",
        "video/x-flv" => "flv",
        "video/mpeg" => "mpg",
        "audio/mpeg" => "mp3",
        "audio/mp4" => "m4a",
        "audio/flac" => "flac",
        "audio/ogg" => "ogg",
        _ => "mp4",
    }
}

fn ext_of(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, e)| e.to_ascii_lowercase())
        .unwrap_or_default()
}

/// `fd_guess_video_mime()`.
pub fn guess_video_mime(file_name: &str, mime: &str) -> String {
    let m = mime.trim().to_ascii_lowercase();
    if !m.is_empty() && m.contains('/') && !m.contains(' ') {
        if m == "application/octet-stream" {
            return mime_by_ext(&ext_of(file_name))
                .unwrap_or("application/octet-stream")
                .to_string();
        }
        return m;
    }
    mime_by_ext(&ext_of(file_name))
        .unwrap_or("application/octet-stream")
        .to_string()
}

/// `fd_stremio_stream_filename()`.
pub fn stream_filename(file_name: &str, mime: &str) -> String {
    let mut name: String = file_name
        .chars()
        .map(|c| match c {
            '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\r' | '\n' => '_',
            other => other,
        })
        .collect();
    name = name.trim().to_string();
    if name.is_empty() {
        name = "video".to_string();
    }
    let ext = ext_of(&name);
    if mime_by_ext(&ext).is_none() {
        let want = ext_by_mime(&guess_video_mime(&name, mime));
        return format!("{}.{}", name, want);
    }
    name
}

/* ── handler ─────────────────────────────────────────────────────────────── */

fn jstr(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").trim().to_string()
}

fn jnum(v: &Value, key: &str) -> u64 {
    v.get(key)
        .and_then(|x| match x {
            Value::Number(n) => n.as_u64(),
            Value::String(s) => s.parse::<u64>().ok(),
            _ => None,
        })
        .unwrap_or(0)
}

/* ── diagnostics ─────────────────────────────────────────────────────────── */

/// Record a request the client is about to see fail.
///
/// `http_status` is the number the PLAYER renders verbatim — Nuvio shows
/// `HTTP 400 Bad Request [ERROR_CODE_IO_BAD_HTTP_STATUS]` and nothing else — so
/// it is what turns a screenshot back into a request. It is also the field the
/// old incidents never carried: the two **400** payload refusals below left no
/// trace at all, which is why "the stream source is blocked or restricted" was
/// unanswerable. The token is never stored; `token_url` is only a bool.
fn record_refusal(
    ctx: &ReqCtx,
    started: std::time::Instant,
    status: u16,
    stage: &str,
    message: &str,
    short_code: &str,
    extra: Value,
) {
    let mut info = serde_json::json!({
        "http_status": status,
        "stage": stage,
        "range": ctx.hdr("range"),
        "user_agent": ctx.hdr("user-agent"),
        "client_platform": ctx.hdr("x-client-platform"),
        "token_url": !crate::auth::auth_token_from_request(ctx).is_empty(),
        "tunneled": !ctx.hdr("cf-ray").is_empty(),
        "elapsed_ms": started.elapsed().as_millis() as u64,
    });
    if let (Some(dst), Some(src)) = (info.as_object_mut(), extra.as_object()) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
    }
    crate::diagnostics::record_throttled(
        "playback_incident",
        &format!("download refused (HTTP {status})"),
        message,
        stage,
        short_code,
        info,
    );
    log(
        "download refused",
        &serde_json::json!({
            "http_status": status,
            "stage": stage,
            "short_code": short_code,
            "range": ctx.hdr("range"),
            "user_agent": ctx.hdr("user-agent"),
        }),
    );
}

pub fn plan_download_candidates(
    bot_id: &str,
    short_code: &str,
    active_bot: &str,
    picked_bot_id: Option<&str>,
    pool_bot_ids: &[String],
    usable: impl Fn(&str) -> bool,
) -> (String, Vec<String>) {
    let bot_bound = !bot_id.is_empty();
    let mut candidates: Vec<String> = Vec::new();
    if !bot_bound && !short_code.is_empty() {
        if let Some(id) = picked_bot_id {
            // Enforced here and not only in `pick_pool_bot()`: a suggestion that
            // cannot serve must never become candidate #1, because `chosen_bot`
            // takes the head of this list.
            if !id.is_empty() && usable(id) {
                candidates.push(id.to_string());
            }
        }
    }
    if !active_bot.is_empty() && !candidates.contains(&active_bot.to_string()) && usable(active_bot) {
        candidates.push(active_bot.to_string());
    }
    if !bot_id.is_empty() && !candidates.contains(&bot_id.to_string()) && usable(bot_id) {
        candidates.push(bot_id.to_string());
    }
    // Pool bots are only useful when the payload does not name one; with a
    // bound bot they would download with a foreign file_id and 410.
    if !bot_bound {
        for id in pool_bot_ids {
            if !id.is_empty() && !candidates.contains(id) && usable(id) {
                candidates.push(id.clone());
            }
        }
    }
    if candidates.is_empty() {
        let fallback = if active_bot.is_empty() { bot_id.to_string() } else { active_bot.to_string() };
        candidates.push(fallback);
    }
    let chosen_bot = if bot_id.is_empty() || !usable(bot_id) {
        candidates.first().cloned().unwrap_or_default()
    } else {
        bot_id.to_string()
    };
    (chosen_bot, candidates)
}

pub async fn handle_download(state: &Arc<AppState>, ctx: &ReqCtx) -> Response {
    let started = std::time::Instant::now();
    if !is_authenticated(ctx) {
        record_refusal(ctx, started, 401, "AUTH_REQUIRED", "Server password or token required", "", serde_json::json!({}));
        return json_response(
            401,
            &serde_json::json!({"ok": 0, "message": "Password required", "auth_required": true}),
        );
    }

    let encoded_qs = ctx.qs("d");
    let mut encoded = if encoded_qs.is_empty() {
        extract_payload_from_path(&ctx.path)
    } else {
        encoded_qs
    };
    if encoded.is_empty() {
        encoded = extract_payload_from_path(&ctx.raw_path);
    }
    let payload = if encoded.is_empty() {
        Value::Object(serde_json::Map::new())
    } else {
        decode_download_payload(&encoded)
    };

    let mut file_id = {
        let a = jstr(&payload, "file_id_mt");
        if a.is_empty() { jstr(&payload, "file_id") } else { a }
    };
    if file_id.is_empty() {
        file_id = ctx.first_param(&["file_id_mt", "file_id"]);
    }
    let mut short_code = jstr(&payload, "short_code");
    if short_code.is_empty() {
        short_code = ctx.first_param(&["short_code", "sc"]);
    }
    if short_code.is_empty()
        && file_id.is_empty()
        && !encoded.is_empty()
        && payload.as_object().map(|o| o.is_empty()).unwrap_or(true)
    {
        short_code = encoded.clone();
    }
    let mut file_size = jnum(&payload, "file_size");
    if file_size == 0 {
        file_size = ctx.first_param(&["file_size", "size"]).parse().unwrap_or(0);
    }
    let mut file_name = {
        let a = jstr(&payload, "file_name");
        if a.is_empty() { ctx.first_param(&["file_name", "name"]) } else { a }
    };
    let mut file_mime = {
        let a = jstr(&payload, "mime");
        if a.is_empty() { ctx.first_param(&["mime", "mime_type"]) } else { a }
    };
    let mut bot_id = {
        let a = jstr(&payload, "bot_id");
        if a.is_empty() { ctx.first_param(&["bot_id"]) } else { a }
    };

    if !short_code.is_empty() && crate::cache::get("flagged_malware", &short_code).is_some() {
        record_refusal(
            ctx,
            started,
            415,
            "MALICIOUS_PAYLOAD_DETECTED",
            "The file is an executable binary disguised as media.",
            &short_code,
            serde_json::json!({"file_name": file_name, "bot_id": bot_id}),
        );
        return json_response(
            415,
            &serde_json::json!({
                "ok": 0,
                "error": "MALICIOUS_PAYLOAD_DETECTED",
                "message": "The file is an executable binary disguised as media and has been removed.",
                "short_code": short_code,
            }),
        );
    }

    // Prefer the id an earlier request PROVED works, and take it EXACTLY as cached.
    //
    // This used to run only when the payload carried an id (`!file_id.is_empty()`).
    // Stream URLs built by this addon carry only the short_code, so the condition
    // was false for every real Play and each Range request resolved again — the
    // player then re-asks on every seek, so a single playback fired a resolve per
    // range. That is what the user saw as "while playing it keeps requesting a new
    // file_id", and against a busy origin (HTTP 503 on `/resolve-file`) it is what
    // stretched the first byte to 11.7 s.
    //
    // A cached id is used whether or not it has been verified: it came from a
    // successful resolve, and the FILE_REFERENCE_EXPIRED path below re-resolves
    // exactly once when Telegram disagrees. 6 h is shorter than the 2 h cache TTL
    // only for the *verified* preference; an unverified hit is still better than a
    // network round-trip.
    let eff_bot = if bot_id.is_empty() { get_bot_id() } else { bot_id.clone() };
    if !short_code.is_empty() && !eff_bot.is_empty() {
        if let Some(entry) = resolve_shortcode_cached(&short_code, &eff_bot) {
            let cached_id = {
                let a = jstr(&entry, "file_id_mt");
                if a.is_empty() { jstr(&entry, "file_id") } else { a }
            };
            // A Bot API file_id is minted FOR ONE BOT: another bot cannot download
            // with it (and the reference inside it is not interchangeable either).
            // The cache is already keyed `short_code:bot_id`, and this second check
            // rejects an entry that names a different bot than the one we are about
            // to use — belt and braces, because using a foreign id can only fail
            // later (FILE_REFERENCE_EXPIRED) after a wasted round-trip.
            let entry_bot = jstr(&entry, "bot_id");
            let same_bot = entry_bot.is_empty() || entry_bot == eff_bot;
            let verified = entry.get("verified_at").and_then(|v| v.as_i64()).unwrap_or(0);
            let resolved = entry.get("resolved_at").and_then(|v| v.as_i64()).unwrap_or(0);
            // A `file_reference` lives ~1 h, so an id is trusted only while it is
            // younger than that — measured, not guessed:
            //   • the reference inside a 2 h-cached id was ALWAYS dead, so every
            //     Eclipse track logged `FILE_REFERENCE_EXPIRED` on its first byte
            //     and then re-resolved;
            //   • the browser's `<audio>` gives the response ~20 s before it fires
            //     `error` ("Playback error - stream unavailable"), and the forced
            //     re-resolve can take longer than that.
            // An id that was never PROVEN by a real fetch (`verified_at == 0`) is
            // therefore only trusted while it is minutes old; anything older — or
            // any entry saved by an older build, which has neither stamp — is
            // refreshed BEFORE the first byte instead of after a failed one.
            // Two windows, not one 3000 s window: a Telegram `file_reference`
            // dies well inside an hour, so a 50-minute "fresh enough" test let a
            // dead reference start a Play and answer 400 on the first chunk —
            // measured on the phone (`F2tdzm`, `pVid3z`, `q8RbRv`). A PROVEN id
            // gets 20 minutes; one that was never fetched yet (prewarm output)
            // gets 10, because nobody has checked it.
            let stamped = if verified > 0 { verified } else { resolved };
            let window = if verified > 0 { 1200 } else { 600 };
            let fresh_enough = stamped > 0 && now() - stamped < window;
            if !cached_id.is_empty() && !(fresh_enough && same_bot) {
                // Drop it, or the resolve below hands the same dead reference
                // straight back and the Play fails on the first byte.
                invalidate_resolve_cache(&short_code, &eff_bot);
                log(
                    "cached file_id refused as stale - refreshing before the first byte",
                    &serde_json::json!({
                        "short_code": short_code,
                        "verified_age_s": if verified > 0 { now() - verified } else { -1 },
                        "resolved_age_s": if resolved > 0 { now() - resolved } else { -1 },
                    }),
                );
            }
            if !cached_id.is_empty() && fresh_enough && same_bot {
                if cached_id != file_id {
                    log(
                        "download using the cached file_id (no re-resolve)",
                        &serde_json::json!({"short_code": short_code, "verified": verified > 0}),
                    );
                }
                file_id = cached_id;
                bot_id = eff_bot.clone();
                let fs = entry.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0);
                if file_size == 0 && fs > 0 {
                    file_size = fs;
                }
                if file_name.is_empty() {
                    let t = jstr(&entry, "title");
                    file_name = if t.is_empty() { jstr(&entry, "file_name") } else { t };
                }
                if file_mime.is_empty() {
                    let m = jstr(&entry, "mime");
                    file_mime = if m.is_empty() { jstr(&entry, "file_type") } else { m };
                }
            }
        }
    }

    log(
        "download request received",
        &serde_json::json!({
            "file_id_present": !file_id.is_empty(),
            "short_code_present": !short_code.is_empty(),
            "encoded_present": !encoded.is_empty(),
            "file_size": file_size,
            "file_name": file_name,
            "mime": file_mime,
            // PHP logs the UA, and without it a retry loop cannot be attributed:
            // a browser/WebView cannot demux Matroska and aborts on the container,
            // while ExoPlayer plays it. `range` tells an initial `bytes=0-` probe
            // from a seek, which is how a looping player is told apart from one
            // that is merely slow to start.
            "ua": ctx.hdr("user-agent"),
            "range": ctx.hdr("range"),
        }),
    );

    let mut active_bot = get_bot_id();
    if !has_local_session("") {
        match auto_provision_guest(&state.telegram).await {
            Ok(prov) => {
                active_bot = prov.bot_id.clone();
                if bot_id.is_empty() || !has_local_session(&bot_id) {
                    bot_id = active_bot.clone();
                }
            }
            Err(prov) => {
                let connecting = is_guest_provision_in_progress();
                // A recorded login flood is retryable and the client has to be told
                // WHEN to come back: `429` + `retry_after` (the remaining wait), not
                // a `403` that reads like "connect your bot". Logging in again before
                // that duration ends is what extends it, so nothing here retries the
                // login — it only reports the deadline the login has to wait for.
                let flood =
                    crate::session::flood_seconds_in(prov.error.as_deref().unwrap_or(""))
                        .max(crate::session::flood_remaining_for(&bot_id));
                let message = if connecting {
                    "Connecting guest bot in progress. Please refresh or try again in a few seconds.".to_string()
                } else if flood > 0 {
                    format!(
                        "Telegram rate limit: bot login paused for another {}s. It clears by itself; playback works again after that.",
                        flood
                    )
                } else {
                    prov.error
                        .clone()
                        .unwrap_or_else(|| "Telegram Bot is not connected. Please connect your bot token in Settings to stream.".to_string())
                };
                let status = if connecting {
                    503
                } else if flood > 0 {
                    429
                } else {
                    403
                };
                let mut body = serde_json::json!({
                    "ok": 0,
                    "message": message,
                    // Verbatim reason from Telegram / the origin, never hidden.
                    "error_detail": prov.error.unwrap_or_default(),
                    "hint": if connecting {
                        "Guest bot session is initializing. Please refresh playback shortly."
                    } else if flood > 0 {
                        "Retrying the login before that wait expires only makes it longer, so wait for it to run out."
                    } else {
                        "Open Settings and connect your bot token."
                    },
                    "short_code": short_code,
                    "bot_id": bot_id,
                });
                if flood > 0 {
                    body["retry_after"] = serde_json::json!(flood);
                }
                record_refusal(
                    ctx, started, status,
                    if flood > 0 { "LOGIN_FLOOD" } else { "NO_SESSION" },
                    &body["message"].as_str().unwrap_or("").to_string(),
                    &short_code,
                    serde_json::json!({"flood_remaining": flood, "connecting": connecting}),
                );
                return json_response(status, &body);
            }
        }
    }

    // A bot is usable when this instance already holds its session file, OR when
    // the environment supplies its token (`BOT_TOKENS`) and it can be logged in
    // here on demand. Without the second half a scaled-out deploy only ever uses
    // the bots whose session happens to live on the instance the request landed
    // on — measured on Heroku: 5 dynos, 5 independent pools, the added bot used
    // by one dyno in five.
    //
    // The second half is gated on THIS bot having no recorded login wait. Telegram
    // applies the wait per token — measured on the six `BOT_TOKENS` bots: four were
    // waiting (1365/1369/1379/1381 s) while two signed in in under 400 ms — so a
    // session-less bot whose token is NOT waiting is perfectly usable and must stay
    // a candidate. Judging every bot by one bot's wait is what took the healthy
    // ones offline with the sick ones.
    let usable = |id: &str| {
        has_local_session(id)
            || (crate::session::flood_remaining_for(id) == 0
                && crate::session::env_token_for(id).is_some())
    };

    // A Bot API `file_id` belongs to the bot that MINTED it — its embedded
    // `file_reference` is not interchangeable — so rotating bots for a request
    // that already carries a file_id is guaranteed to fail. Measured on a pool
    // burst against one stream URL: 5 of 12 ranges returned `410
    // FILE_REFERENCE_EXPIRED`, and every 410 body named a DIFFERENT bot than the
    // one the id had been resolved for. Rotation therefore only happens when we
    // still have to resolve (no `bot_id` in the payload), where `pick_pool_bot()`
    // is exactly what spreads the *resolves* over the pool, and the id it returns
    // is then bound to the bot that will use it.
    let bot_bound = !bot_id.is_empty();
    let picked = if !bot_bound && !short_code.is_empty() {
        pick_pool_bot().and_then(|p| {
            let id = jstr(&p, "bot_id");
            if id.is_empty() { None } else { Some(id) }
        })
    } else {
        None
    };
    // `pool_ready()`, not `get_bot_pool()`: the pool listing includes bots that
    // have never logged in on this instance, and those must not become download
    // candidates (see `session::pool_ready`).
    let pool_bot_ids: Vec<String> = if !bot_bound {
        crate::session::pool_ready().iter().filter_map(|b| {
            let id = jstr(b, "bot_id");
            if id.is_empty() { None } else { Some(id) }
        }).collect()
    } else {
        Vec::new()
    };
    let (chosen_bot, candidates) = plan_download_candidates(
        &bot_id,
        &short_code,
        &active_bot,
        picked.as_deref(),
        &pool_bot_ids,
        &usable,
    );
    bot_id = chosen_bot;

    if file_id.is_empty() && !short_code.is_empty() {
        let resolved = resolve_shortcode_concurrent(&short_code, &candidates).await;
        let candidate_id = {
            let a = jstr(&resolved, "file_id_mt");
            if a.is_empty() { jstr(&resolved, "file_id") } else { a }
        };
        let message = jstr(&resolved, "message");
        if candidate_id.is_empty() && message.to_ascii_lowercase().contains("not found for this request") {
            log(
                "download refused - WordPress does not have this short_code",
                &serde_json::json!({"short_code": short_code}),
            );
            return json_response(
                404,
                &serde_json::json!({
                    "ok": 0,
                    "message": message,
                    "short_code": short_code,
                    "bot_id": bot_id,
                    "retried_bots": candidates.len(),
                }),
            );
        }
        if !candidate_id.is_empty() {
            file_id = candidate_id;
            let rs = jnum(&resolved, "file_size");
            if rs > 0 {
                file_size = rs;
            }
            let name = {
                let t = jstr(&resolved, "title");
                if t.is_empty() { jstr(&resolved, "file_name") } else { t }
            };
            if !name.is_empty() {
                file_name = name;
            }
            let mime = {
                let m = jstr(&resolved, "mime");
                if m.is_empty() {
                    let ft = jstr(&resolved, "file_type");
                    if ft.is_empty() { file_mime.clone() } else { ft }
                } else {
                    m
                }
            };
            file_mime = guess_video_mime(&file_name, &mime);
            let rb = jstr(&resolved, "bot_id");
            if !rb.is_empty() {
                bot_id = rb;
            }
        }
    }

    if file_id.is_empty() {
        record_refusal(
            ctx, started, 400, "MISSING_FILE_ID",
            "file_id or valid short_code is required for local browser download.",
            &short_code,
            serde_json::json!({
                "payload_len": encoded.len(),
                "bot_id": bot_id,
                "pool_bots": candidates.len(),
            }),
        );
        return json_response(
            400,
            &serde_json::json!({
                "ok": 0,
                "message": "file_id or valid short_code is required for local browser download.",
                "hint": "Pass a Bot API file id or shortcode to resolve the file.",
            }),
        );
    }

    file_name = stream_filename(&file_name, &file_mime);
    file_mime = guess_video_mime(&file_name, &file_mime);

    if file_size == 0 || file_name.is_empty() || file_mime.is_empty() {
        record_refusal(
            ctx, started, 400, "MISSING_FILE_META",
            "For Bot API file_id download, file_size, file_name, and mime are required.",
            &short_code,
            serde_json::json!({
                "has_file_size": file_size > 0,
                "has_file_name": !file_name.is_empty(),
                "has_mime": !file_mime.is_empty(),
                "file_id_len": file_id.len(),
                "bot_id": bot_id,
            }),
        );
        return json_response(
            400,
            &serde_json::json!({
                "ok": 0,
                "message": "For Bot API file_id download, file_size, file_name, and mime are required.",
                "hint": "Include file_size, file_name, and mime from your PencariMovie metadata response.",
            }),
        );
    }

    /* ── global stream cap ──────────────────────────────────────────────── */
    // At most `--max-streams` (default 256) downloads run at once; the rest
    // QUEUE for up to 5 s and then answer 503 + Retry-After, so 400+ viewers
    // queue deterministically instead of all opening a Telegram part at once.
    //
    // `--max-streams 0` / `PM_MAX_STREAMS=0` disables the cap: the gate is sized
    // to `Semaphore::MAX_PERMITS`, so this acquire is free and never 503s.
    let stream_permit = match tokio::time::timeout(
        std::time::Duration::from_secs(5),
        state.download_gate.clone().acquire_owned(),
    )
    .await
    {
        Ok(Ok(permit)) => Some(permit),
        Ok(Err(_)) => None,
        Err(_) => {
            log(
                "download refused - max-streams gate saturated",
                &serde_json::json!({"max_streams": state.max_streams, "short_code": short_code}),
            );
            record_refusal(
                ctx, started, 503, "STREAM_GATE_SATURATED",
                "Server busy streaming other viewers. Please retry shortly.",
                &short_code,
                serde_json::json!({"max_streams": state.max_streams, "retry_after": 1}),
            );
            return Response::builder()
                .status(StatusCode::SERVICE_UNAVAILABLE)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::RETRY_AFTER, "1")
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "ok": 0,
                        "message": "Server busy streaming other viewers. Please retry shortly.",
                        "retry_after": 1,
                    }))
                    .unwrap_or_default(),
                ))
                .unwrap_or_else(|_| {
                    json_response(503, &serde_json::json!({"ok": 0, "retry_after": 1}))
                });
        }
    };

    /* ── pick a bot ─────────────────────────────────────────────────────── */
    let mut boot_err: Option<String> = None;
    let mut entry = None;
    let mut tried: Vec<String> = Vec::new();
    for candidate in std::iter::once(bot_id.clone())
        .chain(std::iter::once(active_bot.clone()))
        .chain(candidates.iter().cloned())
    {
        // The SAME predicate the candidate planner used, so planning and booting
        // cannot disagree: `has_local_session` alone would skip an env-supplied bot
        // on an instance that has not logged it in yet, so accept it and pass its
        // token through — unless a login flood makes that login impossible, in
        // which case skipping it is the whole point.
        let env_token = crate::session::env_token_for(&candidate);
        if candidate.is_empty() || !usable(&candidate) || tried.contains(&candidate) {
            continue;
        }
        tried.push(candidate.clone());
        match state.telegram.boot(&candidate, env_token.as_deref()).await {
            Ok(e) => {
                bot_id = candidate;
                entry = Some(e);
                break;
            }
            Err(e) => {
                // Recorded against THIS bot: the wait is per token, so the next
                // candidate — and the next request — may sign in fine. `boot()`
                // records it too; this keeps the caller's view honest.
                crate::session::note_flood_for(&candidate, &e);
                log("download boot failed for bot", &serde_json::json!({"bot_id": candidate, "error": e}));
                boot_err = Some(e);
            }
        }
    }
    if entry.is_none() {
        match auto_provision_guest(&state.telegram).await {
            Ok(prov) => {
                bot_id = prov.bot_id.clone();
                match state.telegram.boot(&prov.bot_id, None).await {
                    Ok(e) => entry = Some(e),
                    Err(e) => boot_err = Some(e),
                }
            }
            Err(prov) => boot_err = prov.error,
        }
    }
    let Some(entry) = entry else {
        // A Telegram login flood is retryable and must NOT look like "your bot is
        // gone": answer 429 + retry_after so a player backs off instead of
        // hammering the login (each retry re-arms FLOOD_WAIT).
        // Advertise the wait THIS bot quoted, not a global flag: `boot()` records
        // it per token and the error text carries the value.
        let flood = crate::session::flood_seconds_in(boot_err.as_deref().unwrap_or(""))
            .max(crate::session::flood_remaining_for(&bot_id));
        let status = if flood > 0 { 429 } else { 403 };
        let reason = if flood > 0 { "RATE_LIMITED" } else { "NO_SESSION" };
        record_refusal(
            ctx, started, status, reason,
            &boot_err.clone().unwrap_or_else(|| "No valid Telegram session.".to_string()),
            &short_code,
            serde_json::json!({
                "bots_tried": tried.len(),
                "boot_error": boot_err.clone().unwrap_or_default(),
                "flood_remaining": flood,
            }),
        );
        let detail = boot_err
            .clone()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                let last = crate::session::flood_last_error();
                if last.is_empty() { None } else { Some(last) }
            })
            .unwrap_or_default();
        let mut body = serde_json::json!({
            "ok": 0,
            "message": boot_err.unwrap_or_else(|| "No valid Telegram session.".to_string()),
            // The verbatim Telegram error, always present so nothing is hidden
            // behind a friendly line: "rpc error 420: FLOOD_WAIT caused by
            // auth.importBotAuthorization (value: 460)", "API credentials are
            // missing: …", "bot sign-in failed: …".
            "error_detail": detail,
            "hint": if flood > 0 {
                "Telegram rate limited the bot login. This clears by itself; retry shortly."
            } else {
                "Call /api/botlogin first to authenticate."
            },
        });
        if flood > 0 {
            body["retry_after"] = serde_json::json!(flood);
        }
        return json_response(status, &body);
    };

    // Which bot actually serves this request. `pick_pool_bot()` round-robins over
    // the pool, so a burst must show several bot ids here — that is how streaming
    // load balancing is verified (group the lines by bot_id in the logs).
    log(
        "download served by bot",
        &serde_json::json!({
            "bot_id": bot_id,
            "short_code": short_code,
            "pool_size": candidates.len(),
            "range": ctx.hdr("range"),
        }),
    );

    file_name = stream_filename(&file_name, &file_mime);
    file_mime = guess_video_mime(&file_name, &file_mime);

    /* ── stream ─────────────────────────────────────────────────────────── */
    let range_header = ctx.hdr("range");
    let range = if range_header.is_empty() {
        None
    } else {
        parse_range_header(&range_header, file_size)
    };
    if !range_header.is_empty() && range.is_none() {
        record_refusal(
            ctx, started, 416, "BAD_RANGE",
            "The client sent a Range this file cannot satisfy.",
            &short_code,
            serde_json::json!({"range_header": range_header, "file_size": file_size}),
        );
        return Response::builder()
            .status(StatusCode::RANGE_NOT_SATISFIABLE)
            .header(header::CONTENT_RANGE, format!("bytes */{}", file_size))
            .body(Body::empty())
            .unwrap_or_else(|_| json_response(500, &serde_json::json!({"ok": 0})));
    }
    let (start, end) = match range {
        Some(r) => (r.start, r.end),
        None => (0, file_size.saturating_sub(1)),
    };
    let total = end.saturating_sub(start) + 1;
    // Align the fetch base to **1 MB**, not 4096.
    //
    // `plan_limit(pos, MB)` picks the largest `4096 << k` that divides `pos` and
    // does not cross a 1 MB boundary. For a 4096-only-aligned `pos` that can
    // collapse to 4 KB, and `fetch_range_once()` then has to SKIP `pos / 4 KB`
    // chunks — for a seek to ~900 MB that is ~220,000 skipped parts, i.e. the
    // request hangs (measured: `bytes=900000000-` returned 0 bytes in 15 s).
    // 1 MB alignment makes every window start a legal full-size part, so a seek
    // costs at most one discarded chunk instead of a 220k-step walk.
    let aligned = (start / crate::telegram::MB) * crate::telegram::MB;

    let mut target: DocTarget = match prepare_file_target(&file_id, file_size) {
        Ok(t) => t,
        Err(e) => {
            record_refusal(
                ctx, started, 500, "BAD_FILE_ID", &e, &short_code,
                serde_json::json!({"file_id_len": file_id.len(), "file_size": file_size, "bot_id": bot_id}),
            );
            return json_response(500, &serde_json::json!({"ok": 0, "message": e}));
        }
    };

    // NO ALAC TRANSCODE — every file takes the byte-for-byte path below.
    //
    // There used to be a branch here that pulled the whole `.m4a`, decoded ALAC
    // to ~150 MB of PCM and served it as WAV, because symphonia 0.5 has no ALAC
    // decoder and neither does Android. It cost 43 s to the first byte on a
    // 49 MB track, a per-track disk cache, a codec probe with head and tail
    // windows, and most of the audio bugs of this release:
    //   * a SHORT probe read was taken as a verdict, so raw ALAC went out as
    //     `audio/mp4` and the player could not decode it (`kuMkZf`);
    //   * a dead `file_reference` did the same (`K58jYr`);
    //   * a cold cache was slower than the player's patience, so the track was
    //     skipped and the next one started;
    //   * the background warm-up answered 503 forever (`L5vw9B`).
    // The same recording is indexed as FLAC — which every Android build plays
    // natively and which streams raw with Range support — so the search ranking
    // now prefers the playable containers (see `fastdownloader_api_search_music`
    // on the origin) and `.m4a` is served like any other file. An AAC `.m4a`
    // plays natively; an ALAC one is no longer ranked first, and nothing tries
    // to transcode it.

    // The in-flight gate belongs to the same connection as the client.
    let (mut client, mut client_gate) = entry.pick_slot();

    let max_attempts = if short_code.is_empty() { 1 } else { 4 };
    let mut first: Option<Vec<u8>> = None;
    let mut last_err: Option<crate::telegram::GramErr> = None;
    // Counts as activity for the app's indicator/battery saver while we resolve
    // and pull the priming chunk; dropped when this function returns (either the
    // stream takes over or we give up).
    let _preflight_activity = ActivityGuard::new(&ACTIVE_PREFLIGHTS);
    let preflight_started = std::time::Instant::now();
    let mut preflight_deadline =
        preflight_started + std::time::Duration::from_secs(PREFLIGHT_BUDGET_SECS);
    // The budget REFRESHES whenever an attempt earns a retry by making progress
    // (fresh `file_id`, or an alternate bot whose reference is valid), but never
    // past this hard cap.
    //
    // Without the refresh, a re-resolve whose new reference was never tried killed
    // playback. Measured on the phone (`short_code QbgnM0`, libmpv): attempt 1
    // answered `FILE_REFERENCE_EXPIRED (400)` after ~140 s, the forced re-resolve
    // then returned a valid id, and the loop-top check compared against the
    // ORIGINAL 20 s deadline, broke out immediately, and the player got
    // `410 FILE_REFERENCE_EXPIRED` — with a working reference sitting in `target`.
    let hard_deadline = preflight_started + std::time::Duration::from_secs(PREFLIGHT_HARD_CAP_SECS);

    // Prime with ONE chunk, never with `total`. `total` is the whole remaining
    // range, so a plain GET of a 4 GB file (which is what `<video>` sends on
    // startup — no Range header) tried to download 4 GB into memory before the
    // response headers were written: the player sat at `readyState 0 /
    // networkState 2` forever and Nuvio logged "still downloading data".
    // A Range request looked fine only because there `total` was already small.
    // See `prime_window()`: the priming fetch must cross the `skip0` gap between
    // `aligned` and `start`, or the prime is discarded and the tail range stalls.
    let (skip0, prime_want) = prime_window(start, aligned, total, file_size);
    let _ = skip0;
    for attempt in 1..=max_attempts {
        if std::time::Instant::now() >= preflight_deadline {
            log(
                "pre-flight budget exhausted - giving up before the client has data",
                &serde_json::json!({"short_code": short_code, "attempt": attempt, "budget_s": PREFLIGHT_BUDGET_SECS}),
            );
            break;
        }
        match fetch_prime_gated(&client, client_gate.as_ref(), &target, aligned, prime_want).await {
            Ok(buf) => {
                first = Some(buf);
                break;
            }
            Err(e) => {
                log(
                    "download attempt failed",
                    &serde_json::json!({
                        "attempt": attempt,
                        "max_attempts": max_attempts,
                        "error": e.message,
                        "short_code": short_code,
                    }),
                );
                let file_ref = e.file_reference;
                last_err = Some(e);
                if file_ref && !short_code.is_empty() {
                    invalidate_resolve_cache(&short_code, &bot_id);
                    log(
                        "file reference expired, re-resolving short_code with nocache",
                        &serde_json::json!({"short_code": short_code, "attempt": attempt}),
                    );
                    let mut fresh = resolve_shortcode_fresh(&short_code, &bot_id).await;
                    let mut new_id = id_of(&fresh);
                    // Keep any `retry_after` seen along the way: the FORCED resolve
                    // is the one that answers "busy" and carries it, and the plain
                    // fallback below overwrites `fresh` with a 200 that has none.
                    let mut retry_after = jnum(&fresh, "retry_after");
                    // The FORCED resolve can fail upstream while the plain one still
                    // works. Measured on the phone: every
                    // `resolve-file?…&force=1&nocache=1` answered **HTTP 503** while
                    // `/api/resolve-shortcode` for the same short_code returned a
                    // valid file_id, so all four attempts re-tried the same expired
                    // reference and playback died with
                    // `410 FILE_REFERENCE_EXPIRED`. A stale reference must never be
                    // retried while a working resolve exists.
                    if new_id.is_empty() || new_id == file_id {
                        log(
                            "forced resolve gave no new id - falling back to the plain resolve",
                            &serde_json::json!({"short_code": short_code, "bot_id": bot_id}),
                        );
                        fresh = resolve_shortcode(&short_code, &bot_id, false).await;
                        new_id = id_of(&fresh);
                        retry_after = retry_after.max(jnum(&fresh, "retry_after"));
                    }
                    // Last resort: the payload may name a bot whose session no longer
                    // exists (a reinstall wipes it). The ACTIVE bot can still resolve.
                    if new_id.is_empty() || new_id == file_id {
                        let active = get_bot_id();
                        if !active.is_empty() && active != bot_id {
                            log(
                                "resolve still stale - trying the active bot",
                                &serde_json::json!({"short_code": short_code, "active_bot_id": active}),
                            );
                            fresh = resolve_shortcode(&short_code, &active, false).await;
                            new_id = id_of(&fresh);
                            retry_after = retry_after.max(jnum(&fresh, "retry_after"));
                        }
                    }
                    if !new_id.is_empty() && new_id != file_id {
                        file_id = new_id;
                        let fs = jnum(&fresh, "file_size");
                        if fs > 0 {
                            file_size = fs;
                        }
                        match prepare_file_target(&file_id, file_size) {
                            Ok(t) => target = t,
                            Err(err) => {
                                record_refusal(
                                    ctx, started, 500, "BAD_FILE_ID_AFTER_RESOLVE", &err, &short_code,
                                    serde_json::json!({"file_size": file_size, "bot_id": bot_id, "attempt": attempt}),
                                );
                                return json_response(500, &serde_json::json!({"ok": 0, "message": err}));
                            }
                        }
                        let fb = jstr(&fresh, "bot_id");
                        if !fb.is_empty() && fb != bot_id && has_local_session(&fb) {
                            bot_id = fb;
                            if let Ok(e) = state.telegram.boot(&bot_id, None).await {
                                let (c, g) = e.pick_slot();
                                client = c;
                                client_gate = g;
                            }
                        }
                        // Progress: give the retry that uses this new reference
                        // its own window (bounded by the hard cap).
                        preflight_deadline = (std::time::Instant::now()
                            + std::time::Duration::from_secs(PREFLIGHT_BUDGET_SECS))
                            .min(hard_deadline);
                        continue;
                    }
                    // Same id back: walk the pool for a bot whose reference is valid.
                    let mut switched = false;
                    for alt_bot in candidates.clone() {
                        if alt_bot.is_empty() || alt_bot == bot_id || !has_local_session(&alt_bot) {
                            continue;
                        }
                        let alt = resolve_shortcode(&short_code, &alt_bot, true).await;
                        let alt_id = {
                            let a = jstr(&alt, "file_id_mt");
                            if a.is_empty() { jstr(&alt, "file_id") } else { a }
                        };
                        if alt_id.is_empty() || alt_id == file_id {
                            continue;
                        }
                        let Ok(alt_entry) = state.telegram.boot(&alt_bot, None).await else {
                            continue;
                        };
                        bot_id = alt_bot.clone();
                        file_id = alt_id;
                        let fs = jnum(&alt, "file_size");
                        if fs > 0 {
                            file_size = fs;
                        }
                        if let Ok(t) = prepare_file_target(&file_id, file_size) {
                            target = t;
                        }
                        let (c, g) = alt_entry.pick_slot();
                        client = c;
                        client_gate = g;
                        log(
                            "file reference expired, trying alternate bot from pool",
                            &serde_json::json!({"short_code": short_code, "alt_bot_id": alt_bot}),
                        );
                        switched = true;
                        break;
                    }
                    let _ = switched;
                    // The plugin refuses a refresh with
                    // `503 {"ok":0,"message":"Server busy resolving file, please
                    // retry shortly.","retry_after":5}` when its relay is busy.
                    // Four attempts fired back-to-back all land inside that same
                    // window (measured: 4 × 503 in ~20 s, then
                    // `410 FILE_REFERENCE_EXPIRED`), so the wait it asks for is
                    // the difference between "plays" and "this title is broken".
                    if attempt < max_attempts {
                        // http_json() drops the body of a non-2xx, so the plugin's
                        // own `retry_after` (5 s) is usually lost — fall back to a
                        // short linear backoff. Without it the four attempts run
                        // back-to-back (~2 s apart, measured) inside one "Server
                        // busy resolving file" window and the stream dies with 410.
                        let secs = if retry_after > 0 {
                            retry_after.clamp(1, 10)
                        } else {
                            (attempt as u64).clamp(1, 4)
                        };
                        log(
                            "resolve busy upstream - backing off before the next attempt",
                            &serde_json::json!({"short_code": short_code, "wait_s": secs, "attempt": attempt}),
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
                        // Same reasoning as above: the next attempt is a real
                        // attempt, not a repeat of a dead reference.
                        preflight_deadline = (std::time::Instant::now()
                            + std::time::Duration::from_secs(PREFLIGHT_BUDGET_SECS))
                            .min(hard_deadline);
                    }
                    continue;
                }
                // Not a reference error: re-boot the session and retry.
                if let Ok(e) = state.telegram.boot(&bot_id, None).await {
                    let (c, g) = e.pick_slot();
                    client = c;
                    client_gate = g;
                }
            }
        }
    }

    let Some(first_buf) = first else {
        let err = last_err.unwrap_or(crate::telegram::GramErr {
            message: "download failed".to_string(),
            ..Default::default()
        });
        log(
            "download failed",
            &serde_json::json!({
                "error": err.message,
                "short_code": short_code,
                "bot_id": bot_id,
                "flood_wait": err.flood_wait,
            }),
        );
        // A rate limit is a 429, never a 5xx/404: a client reads those as fatal
        // and never retries (root AGENTS.md: "A rate limit must answer 429").
        if let Some(secs) = err.flood_wait {
            let retry_after = secs.max(1);
            record_refusal(
                ctx,
                started,
                429,
                "RATE_LIMITED",
                &err.message,
                &short_code,
                serde_json::json!({"bot_id": bot_id, "retry_after": retry_after}),
            );
            return Response::builder()
                .status(StatusCode::TOO_MANY_REQUESTS)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::RETRY_AFTER, retry_after.to_string())
                .body(Body::from(
                    serde_json::to_vec(&serde_json::json!({
                        "ok": 0,
                        "message": "Upstream Telegram rate limit, please retry shortly.",
                        "retry_after": retry_after,
                        "short_code": short_code,
                        "bot_id": bot_id,
                    }))
                    .unwrap_or_default(),
                ))
                .unwrap_or_else(|_| {
                    json_response(
                        429,
                        &serde_json::json!({
                            "ok": 0,
                            "message": "Upstream Telegram rate limit, please retry shortly.",
                            "retry_after": retry_after,
                        }),
                    )
                });
        }
        let status = if err.file_reference { 410 } else { 500 };
        let reason = if err.file_reference { "FILE_REFERENCE_EXPIRED" } else { "DOWNLOAD_FAILED" };
        record_refusal(
            ctx,
            started,
            status,
            reason,
            &err.message,
            &short_code,
            serde_json::json!({"bot_id": bot_id, "flood_wait": err.flood_wait}),
        );
        return json_response(
            status,
            &serde_json::json!({
                "ok": 0,
                "message": err.message,
                "short_code": short_code,
                "bot_id": bot_id,
            }),
        );
    };

    if start == 0 && is_disguised_executable(&first_buf, &file_name, &file_mime) {
        log(
            "malicious payload detected: executable binary disguised as media",
            &serde_json::json!({
                "short_code": short_code,
                "file_name": file_name,
                "bot_id": bot_id,
            }),
        );
        crate::cache::put("flagged_malware", &short_code, &serde_json::json!(1), 86400, 0);
        invalidate_resolve_cache(&short_code, &bot_id);
        report_and_delete_flagged_file(&short_code, &file_name, "disguised_executable", &bot_id);
        record_refusal(
            ctx,
            started,
            415,
            "MALICIOUS_PAYLOAD_DETECTED",
            "The file is an executable binary disguised as media.",
            &short_code,
            serde_json::json!({"bot_id": bot_id, "file_name": file_name}),
        );
        return json_response(
            415,
            &serde_json::json!({
                "ok": 0,
                "error": "MALICIOUS_PAYLOAD_DETECTED",
                "message": "The file is an executable binary disguised as media and has been removed.",
                "short_code": short_code,
            }),
        );
    }

    /* ── commit headers, then stream ────────────────────────────────────── */
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Bytes, std::io::Error>>(4);
    let target_for_task = target.clone();
    // The whole connection pool, not one slot: a single stream must STRIPE its
    // windows across connections (see the closure below).
    let entry_for_task = entry.clone();
    let short_code_for_task = short_code.clone();
    let bot_for_task = bot_id.clone();
    let file_size_for_task = file_size;
    let persist_id = file_id.clone();
    // Captured BEFORE the spawn: the streaming task cannot borrow `ctx`, and a
    // mid-body failure is only diagnosable with the Range it was serving.
    let status_code: u16 = if range.is_some() { 206 } else { 200 };
    let client_ctx = serde_json::json!({
        "http_status": status_code,
        "range": range_header,
        "user_agent": ctx.hdr("user-agent"),
        "client_platform": ctx.hdr("x-client-platform"),
        "file_size": file_size,
        "mime": file_mime,
    });
    tokio::spawn(async move {
        // Held for the whole body, so `--max-streams` covers the live stream and
        // is released when this task ends (completed, aborted, or client gone).
        let _stream_permit = stream_permit;
        // Reported to the app as "streaming" activity for the whole body.
        let _activity = ActivityGuard::new(&ACTIVE_STREAMS);
        // PIPELINE the chunks: fetch up to `pipeline_depth()` 1 MB windows
        // concurrently and emit them IN ORDER (`StreamExt::buffered`). Fetching one
        // chunk at a time made every megabyte pay a full Telegram round-trip —
        // measured ~870 KiB/s, below what a 1080p stream needs, so the player sat at
        // `readyState 2 / networkState 2` and reported a playback stall.
        //
        // The DEPTH is the throughput knob, and it is per-stream: measured on a
        // phone (Android 16, loopback so the Wi-Fi is out of the picture) a single
        // 8 MB range ran at 0.3-0.7 MiB/s with 4 windows, while the PHP port
        // measured 7.5-9.7 MiB/s with 8 parallel chunks. The per-connection
        // `--inflight` gate still bounds the real parts in flight, so raising this
        // fills the pipe rather than raising load. `PM_PIPELINE` overrides it
        // (1..=32) for tuning without a rebuild.
        let pipeline = pipeline_depth();
        use futures_util::StreamExt as _;

        let mut skip = start.saturating_sub(aligned) as usize;
        let mut remaining = total;
        let primed = first_buf;

        // Emit the primed head BEFORE planning: it is the bytes that were pulled
        // while the headers were being written, and the player can start on them
        // immediately instead of waiting for a whole window.
        let plen = primed.len() as u64;
        if plen > 0 && remaining > 0 {
            if (skip as u64) < plen {
                let at = skip;
                let take = ((plen - at as u64) as usize).min(remaining as usize);
                if take > 0 {
                    let out = Bytes::copy_from_slice(&primed[at..at + take]);
                    if tx.send(Ok(out)).await.is_err() {
                        return;
                    }
                    remaining -= take as u64;
                }
                skip = 0;
            } else {
                skip -= plen as usize;
            }
        }

        // Windows continue from the end of the primed piece. When that piece is
        // shorter than a 1 MB window the REST OF THAT WINDOW is fetched first, so
        // every later window starts on an exact 1 MB boundary (full-size legal
        // parts, unchanged throughput); only the head of the body is finer.
        let mut plan: Vec<(u64, u64)> = Vec::new();
        {
            let mut p = aligned + plen;
            // `+ skip`: the windows start at an ALIGNED base, so they must cover
            // the bytes discarded before `start` as well. Without it every seek
            // answered `total - skip` bytes — up to 1 MB SHORT of the
            // Content-Length we just sent, and a player reads a short body as a
            // truncated stream (the "buffers after fast-forward" symptom).
            let mut rem = remaining + skip as u64;
            let mb = crate::telegram::MB;
            // Finish the CURRENT 1 MB window first, so every later window starts on
            // an exact 1 MB boundary — full-size Telegram parts.
            //
            // This used to be `plen < mb`, which silently skipped the step whenever
            // the prime crossed a whole window. `prime_window()` now fetches
            // `skip0 + PRIME` (up to ~1.5 MB), so `p` could stay unaligned;
            // `plan_limit()` then collapses the part size to 4 KB and one 1 MB
            // window costs 256 parts instead of 2 — measured as the "speed is not
            // full speed" regression after the tail-range fix.
            //
            // MadelineProto keeps the same invariant: `downloadToCallable()` plans
            // `$x = $offset - $offset % $part_size` and steps by a whole part, so
            // only the FIRST and LAST parts are partial.
            if rem > 0 && plen > 0 && p % mb != 0 {
                let want = (mb - (p % mb)).min(rem);
                if let Some(win) = window(p, want, file_size) {
                    plan.push(win);
                    p += win.1;
                    rem -= win.1;
                }
            }
            while rem > 0 {
                match window(p, rem.min(mb), file_size) {
                    Some(win) => {
                        plan.push(win);
                        p += win.1;
                        rem -= win.1;
                    }
                    None => break,
                }
            }
        }

        // STRIPE the windows across the bot's connections instead of pinning the
        // whole stream to ONE of them.
        //
        // This is the throughput fix, and it is NOT the same thing as the pipeline
        // depth. MEASURED from a phone with a 240-250 ms RTT to the remote: ONE
        // connection tops out at ~0.4-0.5 MB/s whatever the destination and
        // whatever the depth — Telegram direct 0.33-0.38 MB/s, and the Heroku
        // addon (which is itself next to Telegram and measured 4.85 MB/s from a
        // wired PC) 0.49 MB/s from that same phone. That is a TCP window/BDP
        // limit (`window / RTT`), so the only way past it is parallel connections:
        // with `--workers 8` each in-flight window gets its own. Before this,
        // `pick_slot()` was called ONCE per request, so `--workers` only ever
        // helped *different* streams and a single viewer was stuck on one
        // connection.
        let stream = futures_util::stream::iter(plan.into_iter().enumerate().map(|(i, (p, w))| {
            let entry = entry_for_task.clone();
            let target = target_for_task.clone();
            async move {
                let (client, gate) = entry.pick_slot();
                let res =
                    crate::telegram::fetch_range_gated(&client, gate.as_ref(), &target, p, w).await;
                (i, res)
            }
        }))
        .buffered(pipeline);

        futures_util::pin_mut!(stream);
        loop {
            // A seek or a closed tab drops the receiver. Detecting it only when
            // the next WINDOW is ready to send means up to `pipeline_depth`
            // megabytes of Telegram parts keep holding their `--inflight`
            // permits while the request the player just opened queues behind
            // them (seen on a phone as ttfb 1.0-5.2 s with a second stream
            // open). `is_closed()` is a plain non-blocking check — deliberately
            // NOT a `select!`, which could end a body while the client is still
            // reading and make playback look slow instead.
            if tx.is_closed() {
                return;
            }
            let Some((index, result)) = stream.next().await else {
                break;
            };
            let data = match result {
                Ok(d) => d,
                Err(e) => {
                        let mut info = client_ctx.clone();
                        info["bytes_sent"] = serde_json::json!(total.saturating_sub(remaining));
                        info["bytes_total"] = serde_json::json!(total);
                        info["bot_id"] = serde_json::json!(bot_for_task);
                        log(
                            "download stream aborted mid-body",
                            &serde_json::json!({
                                "error": e.message,
                                "short_code": short_code_for_task,
                                "bytes_sent": total.saturating_sub(remaining),
                            }),
                        );
                        crate::diagnostics::record_throttled(
                            "playback_incident",
                            "stream aborted",
                            &e.message,
                            "DOWNLOAD_FAILED",
                            &short_code_for_task,
                            info,
                        );
                        return;
                    }
            };
            let _ = index;
            let slice_start = skip.min(data.len());
            let take = (data.len() - slice_start).min(remaining as usize);
            if take > 0 {
                let out = Bytes::copy_from_slice(&data[slice_start..slice_start + take]);
                if tx.send(Ok(out)).await.is_err() {
                    // Client disconnected: stop pulling from Telegram.
                    return;
                }
                remaining -= take as u64;
            }
            skip = 0;
            if remaining == 0 {
                break;
            }
        }
        // Persist the id that worked so the next Range starts from a live
        // reference (backend.php writes `verified_at` for the same reason).
        if !short_code_for_task.is_empty() && !bot_for_task.is_empty() {
            let mut merged = resolve_shortcode_cached(&short_code_for_task, &bot_for_task)
                .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
            if !merged.is_object() {
                merged = Value::Object(serde_json::Map::new());
            }
            merged["file_id_mt"] = Value::String(persist_id);
            merged["file_size"] = serde_json::json!(file_size_for_task);
            merged["verified_at"] = serde_json::json!(now());
            save_resolve_cache(&short_code_for_task, &bot_for_task, &merged);
        }
    });

    let mut builder = Response::builder()
        .status(StatusCode::from_u16(status_code).unwrap_or(StatusCode::OK))
        .header(header::CONTENT_TYPE, if file_mime.is_empty() { "application/octet-stream" } else { &file_mime })
        .header(header::CONTENT_LENGTH, total.to_string())
        .header(header::ACCEPT_RANGES, "bytes")
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename*=UTF-8''{}", percent_encode(&file_name)),
        )
        .header(header::CACHE_CONTROL, "no-store")
        .header("X-Content-Type-Options", "nosniff")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(
            "Access-Control-Expose-Headers",
            "Accept-Ranges, Content-Range, Content-Length, Content-Type",
        )
        .header("X-Accel-Buffering", "no");
    if range.is_some() {
        builder = builder.header(header::CONTENT_RANGE, format!("bytes {}-{}/{}", start, end, file_size));
    }
    let stream = ReceiverStream::new(rx).map(|item| match item {
        Ok(b) => Ok::<Bytes, std::io::Error>(b),
        Err(e) => Err(e),
    });
    builder
        .body(Body::from_stream(stream))
        .unwrap_or_else(|_| json_response(500, &serde_json::json!({"ok": 0, "message": "response build failed"})))
}

// The ALAC → WAV transcoder lived here (`probe_codec`, `serve_alac`,
// `write_wav_cache`, `alac_serve_cached`, `alac_wav_path`, `is_alac_target`).
// It is GONE on purpose: it existed only to make an ALAC `.m4a` playable, and
// the same recordings are indexed as FLAC, which streams raw with Range support
// and plays on every Android build. Keeping it meant a codec probe that could
// not tell "not ALAC" from "could not read the file", which served raw ALAC as
// `audio/mp4` to a player that cannot decode it — the "track does not play" bug.
// `crate::audio` still owns `mp4_audio_fourcc` and the WAV helpers, which the
// engine's own audio-probe diagnostics use.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guesses_video_mime_when_omitted() {
        let name = "Sengkolo.Petaka.Satu.Suro.2026.1080p.MalaySub.WEB.MSM.mp4";
        let mime = guess_video_mime(name, "");
        assert_eq!(mime, "video/mp4");

        let mkv = "Movie.2026.mkv";
        assert_eq!(guess_video_mime(mkv, ""), "video/x-matroska");
    }

    /// The stream URL advertises `.wav` for an ALAC source, but the PAYLOAD
    /// names the file that will actually be read. If the renamed `.wav` leaks
    /// into the payload, this says "not ALAC" and the raw MP4 goes out under
    /// `content-type: audio/wav` — the player's "stream unavailable".
    #[test]
    fn stream_filename_and_mime_fallbacks() {
        let name = stream_filename("video", "video/mp4");
        assert_eq!(name, "video.mp4");
        assert_eq!(guess_video_mime(&name, ""), "video/mp4");
    }

    /// The exact ranges a player sends before it starts playing. Each of these
    /// returned `Content-Length` with **0 bytes** in 25 s before the fix, because
    /// the prime was fetched from `aligned` with only `prime` bytes.
    const REAL_SIZE: u64 = 3_229_198_462;

    fn plan_for(start: u64, end: u64) -> (u64, u64, u64) {
        let aligned = (start / crate::telegram::MB) * crate::telegram::MB;
        let total = end - start + 1;
        let (skip0, prime_want) = prime_window(start, aligned, total, REAL_SIZE);
        // The prime is served from `aligned`, so the body can start only if it
        // reaches `start`; anything less discards the whole prime (the bug).
        assert!(
            prime_want > skip0,
            "prime ({prime_want}) must cross the skip ({skip0}) for range {start}-{end}"
        );
        // Never more than one part, and never past EOF.
        assert!(prime_want <= crate::telegram::MB, "head is a single 1 MB part");
        assert!(aligned + prime_want <= REAL_SIZE);
        (aligned, skip0, prime_want)
    }

    #[test]
    fn tail_of_three_bytes_primes_across_the_skip() {
        let (aligned, skip0, want) = plan_for(REAL_SIZE - 3, REAL_SIZE - 1);
        assert!(skip0 > 0, "base must sit below the requested start");
        assert_eq!(aligned + want, REAL_SIZE, "the prime stops exactly at EOF");
        assert_eq!(want - skip0, 3, "exactly the 3 requested bytes remain");
    }

    #[test]
    fn open_ended_tail_range_primes_across_the_skip() {
        let (aligned, skip0, want) = plan_for(REAL_SIZE - 68_000, REAL_SIZE - 1);
        assert!(skip0 > 0);
        assert!(aligned + want <= REAL_SIZE, "never past EOF");
        // The head either reaches EOF or stops at one part; either way it yields
        // bytes, and any remainder is carried by the ALIGNED windows that follow.
        assert!(want - skip0 > 0);
        assert_eq!(
            aligned + want,
            REAL_SIZE.min(aligned + crate::telegram::MB),
            "head ends at EOF or at the next 1 MB boundary"
        );
    }

    #[test]
    fn prefix_range_takes_no_skip() {
        let (aligned, skip0, want) = plan_for(0, 1_048_575);
        assert_eq!((aligned, skip0), (0, 0));
        assert_eq!(want, PRIME_BYTES, "unchanged: one 512 KB prime");
    }

    #[test]
    fn mid_file_seek_still_primes_across_the_skip() {
        let start = 900_000_000u64;
        let (aligned, skip0, want) = plan_for(start, start + 1_048_575);
        assert!(skip0 > 0 && skip0 < crate::telegram::MB);
        // Rounding the head up to a part boundary lands it on a whole 1 MB part,
        // so the next window starts 1 MB aligned (no 4 KB collapse).
        assert_eq!(want, crate::telegram::MB);
        assert_eq!((aligned + want) % crate::telegram::MB, 0);
    }

    /// The split must never be reachable with a segment larger than one part,
    /// and must cover exactly the requested head.
    #[test]
    fn prime_split_covers_the_head_in_legal_parts() {
        use crate::telegram::{MAX_CHUNK, MB, MIN_CHUNK};

        // Whatever a 1 MB-aligned start can produce is one part: no split.
        for want in [1u64, MIN_CHUNK, PRIME_BYTES, MAX_CHUNK] {
            let (clamped, split) = prime_split(want);
            assert_eq!((clamped, split), (want, None), "want={want} fits one part");
        }

        // Every other start rounds the head up to 1 MB: exactly two parts.
        assert_eq!(prime_split(MB), (MB, Some((MAX_CHUNK, MAX_CHUNK))));

        // A head cut short by EOF keeps the remainder exact rather than rounded.
        let (clamped, split) = prime_split(700_000);
        assert_eq!(clamped, 700_000);
        assert_eq!(split, Some((MAX_CHUNK, 700_000 - MAX_CHUNK)));

        // The cap is enforced here, so a future `PRIME_BYTES` bump cannot push
        // an overrun into `fetch_range_core()`'s serial fallback.
        assert_eq!(prime_split(4 * MB).0, MB);
        assert_eq!(prime_split(4 * MB).1, Some((MAX_CHUNK, MAX_CHUNK)));

        // Why `second_pos` stays 4096-aligned without recomputing it.
        assert_eq!(MAX_CHUNK % MIN_CHUNK, 0);
    }

    /// Ties the real player shapes to whether the split engages at all. A byte-0
    /// prefix is the one everyday case that gets no benefit, and saying so here
    /// stops a future "this did nothing" report from looking like a regression.
    #[test]
    fn only_one_part_heads_skip_the_split() {
        use crate::telegram::{MAX_CHUNK, MB};

        let (aligned, _skip0, want) = plan_for(0, 1_048_575);
        assert_eq!(aligned, 0);
        assert_eq!(want, PRIME_BYTES);
        assert_eq!(prime_split(want).1, None, "a 0-based prefix primes one part");

        // A plain range inside the file, and a seek, both round up to 1 MB.
        for start in [900_000_000u64, 11_000_003] {
            let (_aligned, skip0, want) = plan_for(start, start + 1_048_575);
            assert!(skip0 > 0);
            assert_eq!(want, MB, "start={start} rounds the head up");
            assert_eq!(
                prime_split(want).1,
                Some((MAX_CHUNK, want - MAX_CHUNK)),
                "start={start} must use the parallel path"
            );
        }
    }

    #[test]
    fn head_never_leaves_a_part_hostile_offset() {
        // `start = aligned + 1` measured 245 KB/s before the head was rounded,
        // because the following window sat at an offset only 4 KB divides.
        for delta in [1u64, 4_095, 4_097, 707_072, 1_048_575] {
            let start = 1_000_000_000 + delta;
            let (aligned, _skip0, want) = plan_for(start, start + 8 * 1_048_576 - 1);
            let next = aligned + want;
            assert_eq!(
                next % crate::telegram::MAX_CHUNK,
                0,
                "head ends on a part boundary for start={start} (delta={delta})"
            );
        }
    }

    #[test]
    fn windows_never_cross_eof() {
        assert_eq!(window(REAL_SIZE - 3, 3, REAL_SIZE), Some((REAL_SIZE - 3, 3)));
        // Asking for a 1 MB window with 3 bytes left yields those 3 bytes only.
        assert_eq!(window(REAL_SIZE - 3, crate::telegram::MB, REAL_SIZE),
                   Some((REAL_SIZE - 3, 3)));
        // At EOF there is nothing left to fetch.
        assert_eq!(window(REAL_SIZE, 1024, REAL_SIZE), None);
        // A window well inside the file is untouched.
        assert_eq!(window(0, crate::telegram::MB, REAL_SIZE),
                   Some((0, crate::telegram::MB)));
    }

    #[test]
    fn download_candidates_round_robin_unbound_shortcodes() {
        let usable = |_id: &str| true;
        let pool = vec!["bot_1".to_string(), "bot_2".to_string(), "bot_3".to_string()];

        // Request 1: short_code provided, bot_id empty (unbound).
        // pick_pool_bot returns bot_1.
        let (chosen1, cands1) = plan_download_candidates("", "short1", "bot_active", Some("bot_1"), &pool, usable);
        assert_eq!(chosen1, "bot_1");
        assert_eq!(cands1[0], "bot_1");

        // Request 2: next round-robin pick is bot_2.
        let (chosen2, cands2) = plan_download_candidates("", "short2", "bot_active", Some("bot_2"), &pool, usable);
        assert_eq!(chosen2, "bot_2");
        assert_eq!(cands2[0], "bot_2");

        // Request 3: next round-robin pick is bot_3.
        let (chosen3, cands3) = plan_download_candidates("", "short3", "bot_active", Some("bot_3"), &pool, usable);
        assert_eq!(chosen3, "bot_3");
        assert_eq!(cands3[0], "bot_3");
    }

    #[test]
    fn download_candidates_preserves_bound_bot() {
        let usable = |_id: &str| true;
        let pool = vec!["bot_1".to_string(), "bot_2".to_string(), "bot_3".to_string()];

        // Bound request: bot_id is already fixed to "bot_2" because the payload carries a minted file_id.
        // Even if pick_pool_bot would suggest "bot_1", it must NOT rotate away from the bound bot.
        let (chosen, cands) = plan_download_candidates("bot_2", "short1", "bot_active", Some("bot_1"), &pool, usable);
        assert_eq!(chosen, "bot_2");
        // Other pool bots are not added to candidates to prevent 410 FILE_REFERENCE_EXPIRED
        assert!(!cands.contains(&"bot_1".to_string()));
        assert!(!cands.contains(&"bot_3".to_string()));
    }

    /// A bot with no session file must never be the SERVING candidate while a
    /// login flood is armed: it cannot stream, it cannot be logged in either, and
    /// choosing it spends an attempt that re-arms the wait (measured:
    /// `retry_after: 2421`). The logged-in bot has to win.
    #[test]
    fn a_sessionless_bot_is_not_a_candidate_during_a_login_flood() {
        let pool = vec!["bot_no_session".to_string(), "bot_logged_in".to_string()];
        let flood = true;
        let usable = |id: &str| id == "bot_logged_in" || (!flood && id == "bot_no_session");

        // Unbound: `pick_pool_bot()` suggested the session-less bot.
        let (chosen, cands) = plan_download_candidates(
            "",
            "short1",
            "bot_no_session",
            Some("bot_no_session"),
            &pool,
            usable,
        );
        assert_eq!(chosen, "bot_logged_in", "the session-backed bot must win");
        assert_eq!(cands, vec!["bot_logged_in".to_string()]);
    }

    /// With no flood recorded, an env-supplied bot that has not logged in yet is
    /// still a legitimate candidate — a fresh dyno must be able to boot one.
    #[test]
    fn a_sessionless_env_bot_is_usable_when_no_flood_is_armed() {
        let pool = vec!["bot_env".to_string()];
        let flood = false;
        let usable = |id: &str| id == "bot_env" && !flood;
        let (chosen, cands) =
            plan_download_candidates("", "short1", "bot_env", Some("bot_env"), &pool, usable);
        assert_eq!(chosen, "bot_env");
        assert!(cands.contains(&"bot_env".to_string()));
    }

    #[test]
    fn disguised_executable_detection() {
        // 1. Valid MP4 (ftyp at offset 4) must NOT be flagged
        let valid_mp4 = [0x00, 0x00, 0x00, 0x20, b'f', b't', b'y', b'p', b'i', b's', b'o', b'm'];
        assert!(!is_disguised_executable(&valid_mp4, "movie.mp4", "video/mp4"));

        // 2. Valid MKV (EBML ID \x1A\x45\xDF\xA3) must NOT be flagged
        let valid_mkv = [0x1A, 0x45, 0xDF, 0xA3, 0x93, 0x42, 0x86];
        assert!(!is_disguised_executable(&valid_mkv, "movie.mkv", "video/x-matroska"));

        // 3. Legitimate EXE file with MZ header must NOT be flagged (no false positive for apps)
        let mut pe_exe = vec![0u8; 512];
        pe_exe[0] = b'M';
        pe_exe[1] = b'Z';
        pe_exe[0x3C] = 0x80;
        pe_exe[0x80] = b'P';
        pe_exe[0x81] = b'E';
        pe_exe[0x82] = 0;
        pe_exe[0x83] = 0;
        assert!(!is_disguised_executable(&pe_exe, "setup.exe", "application/octet-stream"));
        assert!(!is_disguised_executable(&pe_exe, "app.apk", "application/vnd.android.package-archive"));
        assert!(!is_disguised_executable(&pe_exe, "archive.zip", "application/zip"));

        // 4. Disguised Windows PE executable claiming to be .mp4 MUST be flagged
        assert!(is_disguised_executable(&pe_exe, "Resident.Evil.2026.mp4", "video/mp4"));
        assert!(is_disguised_executable(&pe_exe, "movie.mkv", ""));

        // 5. DOS executable with "This program cannot be run in DOS mode" claiming to be .avi MUST be flagged
        let mut dos_exe = vec![0u8; 128];
        dos_exe[0] = b'M';
        dos_exe[1] = b'Z';
        dos_exe[0x20..0x20 + 12].copy_from_slice(b"This program");
        assert!(is_disguised_executable(&dos_exe, "video.avi", "video/x-msvideo"));

        // 6. Linux ELF binary claiming to be .mp4 MUST be flagged
        let elf = [0x7F, b'E', b'L', b'F', 0x02, 0x01, 0x01, 0x00];
        assert!(is_disguised_executable(&elf, "movie.mp4", "video/mp4"));
        // But legitimate ELF without media extension must NOT be flagged
        assert!(!is_disguised_executable(&elf, "my_binary.bin", "application/octet-stream"));

        // 7. Short buffers must NOT panic and return false
        assert!(!is_disguised_executable(&[b'M', b'Z'], "movie.mp4", "video/mp4"));
        assert!(!is_disguised_executable(&[], "movie.mp4", "video/mp4"));
    }
}
