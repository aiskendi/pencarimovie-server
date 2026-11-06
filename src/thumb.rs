//! Posters that come from Telegram itself.
//!
//! The origin used to mint a per-file `thumbnail_url` with a Bot API `getFile`
//! call. That endpoint is gone, so every FILE row now answers the same
//! `tg-placeholder/document-bl.png` — thousands of unrelated titles sharing one
//! blank picture — and `/meta` for a `pm_file_` id has nothing real to show.
//!
//! This module fills that gap the same way `/api/download` fills the media gap:
//! the thumbnail is not a second file on Telegram, it is the SAME
//! `id`/`access_hash`/`file_reference`/dc with `thumb_size` set, so the
//! `file_id_mt` the resolve already returns carries its own poster and nothing
//! new has to be minted upstream.
//!
//! `/thumb/<short_code>` is PUBLIC (an `<img>` cannot send the app token) and
//! answers `image/jpeg`. Measured against three live titles: `thumb_size="s"`
//! answers a 320x180 JPEG of 7.3 KB / 15.8 KB / 14.0 KB, `"m"` answers the SAME
//! bytes, and `"x"`/`"y"`/`"w"` answer the FILE — see [`fetch_thumb()`].

use std::sync::Arc;

use serde_json::Value;
use tokio::sync::Semaphore;

use crate::cache;
use crate::resolve::{resolve_shortcode, resolve_shortcode_cached, resolve_shortcode_fresh};
use crate::respond::rtrim;
use crate::telegram;
use crate::AppState;

/// A poster of a given file never changes, so a hit is permanent in practice;
/// a week is short enough that a re-upload behind the same short code is not
/// pinned forever, and long enough that browsing a catalog costs one part per file.
const TTL: i64 = 7 * 24 * 3600;

/// A FAILED thumb is remembered for ten minutes. A broken row (origin resolve
/// answering 503, or a document with no embedded thumb) otherwise costs a full
/// resolve — with its retry ladder — on every page view, which is exactly what a
/// 44-code stress burst showed: the same 7 codes re-resolved on all four rounds.
const MISS_TTL: i64 = 600;

/// The origin's "no poster for this file" image.
const PLACEHOLDER: &str = "tg-placeholder";

/// The rest-api's own thumbnail route (`https://pencarimovie.com/tgthumb/<file_id>.jpg`)
/// — the dependency this module exists to replace. Measured 2026-11-01: it still
/// answers the same 7288-byte JPEG that `thumb_size="s"` does, so the two are the
/// same picture, which is what makes dropping the host a pure win.
const ORIGIN_THUMB_ROUTE: &str = "/tgthumb/";

/// The only size that exists on a media document. `m` returns the same image, and
/// anything larger makes Telegram answer with the file instead of a thumb.
const THUMB_SIZE: &str = "s";

/// Sizes tried against a stored `thumb_file_id`. `s` and `m` answer the same
/// JPEG on every row measured, so two attempts is the whole ladder.
const STORED_THUMB_SIZES: [&str; 2] = ["s", "m"];

/// The largest real thumb measured is 16 KB. A reply at the cap means Telegram
/// ignored `thumb_size` and streamed the media from offset 0.
const CAP: u64 = 512 * 1024;

/// Thumb work runs under its own ceiling, NOT the streaming budget: a catalog
/// page asks for up to a hundred posters at once, and they must never queue ahead
/// of a player's `/api/download` parts. The permit spans resolve **and** read,
/// because an unbounded resolve fan-out is what the origin fails on.
fn gate() -> &'static Semaphore {
    static GATE: std::sync::OnceLock<Semaphore> = std::sync::OnceLock::new();
    GATE.get_or_init(|| Semaphore::new(4))
}

/// Does the ENGINE have to produce this poster?
///
/// Two values come back from the origin for a FILE row now:
///
/// * `…/tg-placeholder/document-bl.png` — "no poster for this file", one image
///   shared by thousands of unrelated titles;
/// * `…/tgthumb/<file_id>.jpg` — the rest-api's thumbnail route, i.e. the host
///   this module replaces.
///
/// Both become `/thumb/<short_code>`. A post row's featured image
/// (`…/wp-content/uploads/…`) is a real, per-post poster and is left alone —
/// those rows have no short code, so rewriting them would blank the poster.
pub fn needs_local(url: &str) -> bool {
    let u = url.trim().to_ascii_lowercase();
    u.is_empty() || u.contains(PLACEHOLDER) || u.contains(ORIGIN_THUMB_ROUTE)
}

/// The poster a catalog item / `/meta` should carry.
///
/// A REAL origin poster is kept as-is: post rows still point at their own
/// featured image, and only the per-FILE placeholder is replaced. When there is
/// no usable poster AND no short code, the origin value is returned unchanged so
/// the client's own placeholder logic decides (the dashboard has an `onerror`
/// fallback on every card).
/// `/api/thumb` — the public, external-site twin of `/api/download`.
///
/// Same idea: an `<img>` on someone else's page cannot carry a header, so the
/// poster has to be reachable by plain URL, with CORS and `Range`-free GETs.
pub fn is_public_thumb_path(path: &str) -> bool {
    path == "/api/thumb" || path.starts_with("/api/thumb/")
}

/// Accept a filename-ish tail (`<code>.jpg`, `<code>.jpeg`, `<code>.png`) so an
/// Open Graph consumer or an `<img>` can ask for an image by name.
pub fn strip_thumb_ext(s: &str) -> &str {
    let mut t = s.trim();
    for ext in [".jpg", ".jpeg", ".png", ".webp"] {
        if t.len() > ext.len() && t.to_ascii_lowercase().ends_with(ext) {
            t = &t[..t.len() - ext.len()];
            break;
        }
    }
    t
}

/// The bot that will actually fetch, and its connections.
async fn entry_for(state: &Arc<AppState>) -> Option<(String, Arc<telegram::BotEntry>)> {
    let bot = {
        let active = crate::session::get_bot_id();
        if active.is_empty() {
            crate::session::pick_pool_bot()
                .map(|b| jstr(&b, "bot_id"))
                .unwrap_or_default()
        } else {
            active
        }
    };
    if bot.is_empty() {
        return None;
    }
    let entry = match state.telegram.get(&bot).await {
        Some(e) => e,
        None => state.telegram.boot(&bot, None).await.ok()?,
    };
    Some((bot, entry))
}

/// The MUSIC variant of [`poster_url`]: only the retired `/tgthumb/` route is
/// rewritten, the shared placeholder is kept.
///
/// Measured on `search-music` (2026-11-06), three tracks in one answer:
///
/// * `…/tgthumb/2863479626640247468.jpg` → `/thumb/F7L35B` answers a **7913-byte
///   JPEG**, so rewriting it costs nothing and drops the origin host;
/// * `…/tg-placeholder/document-bl.png` ×2 → `/thumb/<code>` answers **404**,
///   because an audio document usually carries no Telegram thumb at all.
///
/// A blanket `poster_url()` would therefore turn two working covers into broken
/// images. The placeholder is already the honest answer ("no cover for this
/// file"), so it stays — and the dashboard keeps its own `onerror`.
pub fn poster_url_music(base_url: &str, origin_thumb: &str, short_code: &str) -> String {
    let origin = origin_thumb.trim();
    if !origin.to_ascii_lowercase().contains(ORIGIN_THUMB_ROUTE) {
        return origin.to_string();
    }
    poster_url(base_url, origin, short_code)
}

pub fn poster_url(base_url: &str, origin_thumb: &str, short_code: &str) -> String {
    let origin = origin_thumb.trim();
    if !needs_local(origin) {
        return origin.to_string();
    }
    let code = short_code.trim();
    if code.is_empty() {
        return String::new();
    }
    // No token segment: `/thumb` is public like `/catalog` and `/meta`, so an
    // image URL in a client's cache never has to carry the auth token.
    format!("{}/thumb/{}", rtrim(base_url), code)
}

fn jstr(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").trim().to_string()
}

fn file_id_of(v: &Value) -> String {
    let a = jstr(v, "file_id_mt");
    if a.is_empty() { jstr(v, "file_id") } else { a }
}

/// A JPEG SOI/APP0 header. A thumb is one; the media is not, so this is how the
/// "Telegram answered with the file" case is kept out of a poster.
fn is_jpeg(b: &[u8]) -> bool {
    b.len() > 3 && b[0] == 0xFF && b[1] == 0xD8 && b[2] == 0xFF
}

/// One `upload.getFile` for this file's poster.
///
/// Two sources, in this order:
///
/// 1. the STORED `thumb_file_id` the origin now returns (`tg_file_id.thumb_file_id`)
///    — the picture Telegram already made. It decodes as type 0 (`thumbnail`) and
///    is read through `inputDocumentFileLocation` + `thumb_size` like any other
///    document; `inputPhotoFileLocation` answers FILE_REFERENCE_EXPIRED for these
///    ids, measured. This is the only source that exists for DOCUMENTS and AUDIO,
///    which carry no thumbs of their own.
/// 2. `thumb_size` on the MEDIA id — what every video has, and the fallback when
///    the stored thumb was minted by a different bot (a Bot API file_id carries
///    that bot's auth bytes, so ours cannot fetch a foreign one: measured
///    FILE_REFERENCE_EXPIRED for all sizes on a foreign row, a JPEG on our own).
///
/// One re-resolve on `FILE_REFERENCE_EXPIRED` — the same recovery `/api/download`
/// performs — is applied to the media path only.
async fn fetch_thumb(code: &str, bot: &str, entry: &Arc<telegram::BotEntry>) -> Option<Vec<u8>> {
    let mut res = resolve_shortcode_cached(code, bot).unwrap_or(Value::Null);
    if file_id_of(&res).is_empty() {
        res = resolve_shortcode(code, bot, false).await;
    }
    let mut file_id = file_id_of(&res);
    if file_id.is_empty() {
        crate::log::log("thumb: no file_id", &serde_json::json!({"short_code": code, "bot_id": bot}));
        return None;
    }

    let client = entry.pick();

    // 1. The stored thumbnail file id.
    let stored = jstr(&res, "thumb_file_id");
    if !stored.is_empty() {
        for size in STORED_THUMB_SIZES {
            let Ok(target) = telegram::prepare_thumb_target(&stored, size) else {
                break;
            };
            match telegram::fetch_range(&client, &target, 0, CAP).await {
                Ok(bytes) if is_jpeg(&bytes) && (bytes.len() as u64) < CAP => return Some(bytes),
                Ok(bytes) => {
                    // No such size on this thumbnail: Telegram answered with the
                    // object itself, which is not a poster.
                    crate::log::log(
                        "thumb: stored thumb answered non-thumbnail bytes",
                        &serde_json::json!({"short_code": code, "size": size, "len": bytes.len()}),
                    );
                    break;
                }
                Err(e) => {
                    // Foreign bot / dead reference: fall through to the media.
                    crate::log::log(
                        "thumb: stored thumb unusable, trying the media",
                        &serde_json::json!({"short_code": code, "size": size, "error": e.message}),
                    );
                    break;
                }
            }
        }
    }
    for attempt in 0..2 {
        let target = match telegram::prepare_thumb_target(&file_id, THUMB_SIZE) {
            Ok(t) => t,
            Err(e) => {
                crate::log::log("thumb: not a document", &serde_json::json!({"short_code": code, "error": e}));
                return None;
            }
        };
        match telegram::fetch_range(&client, &target, 0, CAP).await {
            Ok(bytes) => {
                // Two ways Telegram says "no such thumb": the bytes are the media
                // (wrong kind) or they fill the whole request. Both are rejected —
                // a poster route must never serve 512 KB of the movie.
                if is_jpeg(&bytes) && (bytes.len() as u64) < CAP {
                    return Some(bytes);
                }
                crate::log::log(
                    "thumb: thumb_size answered with non-thumbnail bytes",
                    &serde_json::json!({"short_code": code, "len": bytes.len()}),
                );
                return None;
            }
            Err(e) => {
                if e.file_reference && attempt == 0 {
                    let fresh = resolve_shortcode_fresh(code, bot).await;
                    let id = file_id_of(&fresh);
                    // The same id means the origin served its cache (or a 503 with
                    // a stale fallback); retrying spends a round-trip to fail alike.
                    if id.is_empty() || id == file_id {
                        return None;
                    }
                    file_id = id;
                    continue;
                }
                crate::log::log(
                    "thumb: fetch failed",
                    &serde_json::json!({"short_code": code, "error": e.message}),
                );
                return None;
            }
        }
    }
    None
}

/// The JPEG for this short code, from the cache or from Telegram.
pub async fn serve(state: &Arc<AppState>, short_code: &str) -> Option<Vec<u8>> {
    let code = short_code.trim();
    if code.is_empty() || !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
        return None;
    }
    if let Some(hit) = cache::get("thumb", code) {
        if let Some(b64) = hit.get("b64").and_then(|v| v.as_str()) {
            let bytes = crate::fileid::base64url_decode(b64);
            if is_jpeg(&bytes) {
                return Some(bytes);
            }
        }
    }
    // A code that already failed inside the window costs NOTHING to ask again.
    // Without this, 7 of 44 codes whose origin resolve answers 503/0 were
    // re-resolved on every burst, and a catalog page paid ~4 origin calls per
    // broken row on every view (measured).
    if cache::get("thumb_miss", code).is_some() {
        return None;
    }

    let (bot, entry) = entry_for(state).await?;

    // The permit covers the RESOLVE as well as the read. A catalog page asks for
    // ~50 posters at once, and the origin's `/resolve-file` answers 503 and
    // connection errors under a 44-wide burst (measured: 7/44 codes lost), so the
    // fan-out has to be bounded here — bounding only `upload.getFile` still let
    // every request resolve first, all at the same time.
    let _permit = gate().acquire().await.ok()?;

    let Some(bytes) = fetch_thumb(code, &bot, &entry).await else {
        cache::put("thumb_miss", code, &serde_json::json!({"miss": 1}), MISS_TTL, 0);
        return None;
    };
    cache::put(
        "thumb",
        code,
        &serde_json::json!({"b64": crate::fileid::base64url_encode(&bytes)}),
        TTL,
        0,
    );
    Some(bytes)
}

/// A thumb read straight from a `file_id_mt` the caller already holds — the same
/// id `/api/download` streams with. No resolve and no cache write: the caller may
/// be a one-off external page that already resolved the file itself.
pub async fn serve_file_id(state: &Arc<AppState>, file_id_mt: &str) -> Option<Vec<u8>> {
    let id = file_id_mt.trim();
    if id.is_empty() {
        return None;
    }
    let (_bot, entry) = entry_for(state).await?;
    let _permit = gate().acquire().await.ok()?;
    let client = entry.pick();
    let target = telegram::prepare_thumb_target(id, THUMB_SIZE).ok()?;
    let bytes = telegram::fetch_range(&client, &target, 0, CAP).await.ok()?;
    if is_jpeg(&bytes) && (bytes.len() as u64) < CAP {
        Some(bytes)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_public_thumb_path_matches_only_its_own_prefix() {
        assert!(is_public_thumb_path("/api/thumb"));
        assert!(is_public_thumb_path("/api/thumb/Cu20Qm"));
        assert!(is_public_thumb_path("/api/thumb/Cu20Qm.jpg"));
        assert!(!is_public_thumb_path("/api/thumbnail"));
        assert!(!is_public_thumb_path("/thumb/Cu20Qm"));
        assert!(!is_public_thumb_path("/api/download/x/y"));
    }

    #[test]
    fn an_image_extension_is_stripped_but_a_short_code_is_not_mangled() {
        assert_eq!(strip_thumb_ext("Cu20Qm.jpg"), "Cu20Qm");
        assert_eq!(strip_thumb_ext("Cu20Qm.JPEG"), "Cu20Qm");
        assert_eq!(strip_thumb_ext("Cu20Qm.png"), "Cu20Qm");
        assert_eq!(strip_thumb_ext("Cu20Qm"), "Cu20Qm");
        // A code that merely CONTAINS the letters stays whole.
        assert_eq!(strip_thumb_ext("jpg1x2"), "jpg1x2");
    }

    #[test]
    fn a_real_origin_poster_is_kept() {
        let real = "https://m.media-amazon.com/images/M/MV5B.jpg";
        assert_eq!(poster_url("http://host:8088", real, "Cu20Qm"), real);
    }

    #[test]
    fn the_origin_placeholder_is_replaced_by_the_local_thumb() {
        let ph = "https://pencarimovie.com/wp-content/uploads/tg-placeholder/document-bl.png?v=1";
        assert_eq!(
            poster_url("http://host:8088/", ph, "Cu20Qm"),
            "http://host:8088/thumb/Cu20Qm"
        );
        assert!(needs_local(ph));
    }

    #[test]
    fn the_retired_origin_thumb_route_is_replaced_by_the_local_thumb() {
        // The rest-api's own thumbnail route: the dependency being removed.
        let origin = "https://pencarimovie.com/tgthumb/4836250587675518684.jpg";
        assert!(needs_local(origin));
        assert_eq!(
            poster_url("https://pm.example.com", origin, "yhlYlW"),
            "https://pm.example.com/thumb/yhlYlW"
        );
    }

    /// Music: the retired `/tgthumb/` route becomes local, the shared placeholder
    /// does not (there is no audio thumb behind it — measured 404).
    #[test]
    fn music_keeps_the_placeholder_but_drops_the_origin_thumb_route() {
        let placeholder = "https://pencarimovie.com/wp-content/uploads/tg-placeholder/document-bl.png?v=1";
        assert_eq!(
            poster_url_music("http://host:8088", placeholder, "2NIQv4"),
            placeholder
        );
        assert_eq!(
            poster_url_music(
                "http://host:8088",
                "https://pencarimovie.com/tgthumb/2863479626640247468.jpg",
                "F7L35B"
            ),
            "http://host:8088/thumb/F7L35B"
        );
        assert_eq!(poster_url_music("http://host:8088", "", "F7L35B"), "");
    }

    #[test]
    fn a_post_featured_image_is_never_rewritten() {
        // A post row's own poster lives under `wp-content/uploads` and is REAL.
        // Those rows carry no short code, so rewriting would blank the poster.
        let post = "https://pencarimovie.com/wp-content/uploads/2026/01/poster.jpg";
        assert!(!needs_local(post));
        assert_eq!(poster_url("http://host:8088", post, ""), post);
    }

    #[test]
    fn an_empty_poster_falls_back_to_the_local_thumb_without_a_second_slash() {
        assert_eq!(
            poster_url("https://pm.example.com", "", "yhlYlW"),
            "https://pm.example.com/thumb/yhlYlW"
        );
    }

    #[test]
    fn a_row_without_a_short_code_gets_no_fabricated_url() {
        assert_eq!(poster_url("http://host:8088", "", ""), "");
        assert_eq!(poster_url("http://host:8088", "tg-placeholder/document-bl.png", ""), "");
    }

    #[test]
    fn only_a_jpeg_counts_as_a_thumb() {
        assert!(is_jpeg(&[0xFF, 0xD8, 0xFF, 0xE0, 0x00]));
        // The fallback Telegram sends for an unknown `thumb_size`: media bytes.
        assert!(!is_jpeg(&[0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70]));
        assert!(!is_jpeg(b""));
    }
}
