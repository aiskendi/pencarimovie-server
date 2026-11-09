//! Music addon — `fd_eclipse_audio_quality_score()` and
//! `fd_eclipse_format_track()` (backend.php:14046-14259).
//!
//! TITLE AND ARTIST COME FROM THE ORIGIN, not from parsing here:
//!
//! * `main.php` (`fastdownloader_api_audio_caption_title`) reads the caption —
//!   the only place a contraction survives, because `tg_file_new.title` is the
//!   file name and stores every `'` as a dot (`01.My.Body.Isn.t.Ready.m4a`) —
//!   and returns it as `song_title` with the artist as `performer`;
//! * a re-sent file carries the real tags, nested as
//!   `"audio": {"title":"Be Her","performer":"Ella Langley","duration":217}`.
//!
//! This module only maps those onto the addon's `track` object, with a
//! mechanical file-name fallback so nothing is ever blank: the Eclipse client
//! matches a track by `title + artist`, so an empty artist makes it unplayable.

use serde_json::Value;

fn rx(p: &str) -> Option<fancy_regex::Regex> {
    fancy_regex::Regex::new(p).ok()
}

fn rep(p: &str, text: &str, with: &str) -> String {
    match rx(p) {
        Some(re) => re.replace_all(text, with).into_owned(),
        None => text.to_string(),
    }
}

fn collapse(s: &str) -> String {
    rep(r"\s+", s, " ").trim().to_string()
}

fn str_at(item: &Value, key: &str) -> String {
    item.get(key)
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// A flat row column first (`song_title`, `performer`), the nested `audio.<key>`
/// second — both shapes are what the origin actually sends.
fn tag(item: &Value, key: &str) -> String {
    let flat = str_at(item, key);
    if !flat.is_empty() {
        return flat;
    }
    item.get("audio")
        .and_then(|a| a.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// `Ella.Langley.Be.Her.mp3` → `Ella Langley Be Her`
fn name_words(file_name: &str) -> String {
    let mut n = file_name.to_string();
    // `song.mp3.mp3` is real, so strip in a loop.
    for _ in 0..4 {
        let stripped = rep(r"(?i)\.(mp3|m4a|flac|wav|ogg|opus|aac|mp4|mkv|mpg)$", &n, "");
        if stripped == n {
            break;
        }
        n = stripped;
    }
    let n = collapse(&n.replace('.', " ").replace('_', " "));
    collapse(&rep(r"^\d+[\s.\-_]+", &n, ""))
}

/// Drop a leading `performer` (and any separator after it) from a name.
fn strip_artist(name: &str, artist: &str) -> String {
    if artist.is_empty() {
        return name.to_string();
    }
    collapse(&rep(
        &format!("(?i)^{}[\\s\\-:\u{2013}\u{2014}]+", fancy_regex::escape(artist)),
        name,
        "",
    ))
}

/// The bitrate floor that separates a lossless `.m4a` from a lossy one.
///
/// MEASURED 2026-11-07, "Sial" by Mahalini, same 243 s, both `.m4a`, both
/// `mime_type = audio/mp4`:
///
/// | file | bytes | kbps | what it is |
/// | ---- | ----- | ---- | ---------- |
/// | `07.Sial.m4a` (`Xt8C7v`) | 8,187,833 | 269 | AAC 256k |
/// | `07.Sial.m4a` (`J2YkvY`) | 26,704,889 | 879 | ALAC (Qobuz serves the same master at 886) |
///
/// AAC tops out around 320 kbps and HE-AAC well below that, while ALAC at 16/44.1
/// is ~900 and 24/48 higher still, so 600 sits in empty space between the two:
/// nothing legitimate encodes AAC there, and no ALAC lands under it.
pub const LOSSLESS_KBPS: u64 = 600;

/// Bits per second ÷ 1000 for a row, or 0 when the row does not say.
pub fn kbps(file_size: u64, duration_secs: i64) -> u64 {
    if file_size == 0 || duration_secs <= 0 {
        return 0;
    }
    (u128::from(file_size) * 8 / duration_secs as u128 / 1000) as u64
}

/// `flac`/`wav`/`alac` are bit-exact on their own; `m4a` is a CONTAINER, so it
/// needs the bitrate test above before it may be called lossless.
pub fn is_lossless(ext: &str, file_size: u64, duration_secs: i64) -> bool {
    match ext.to_ascii_lowercase().as_str() {
        "flac" | "wav" | "alac" => true,
        "m4a" => kbps(file_size, duration_secs) >= LOSSLESS_KBPS,
        _ => false,
    }
}

/// The tier both addon protocols display and rank on. One place decides it,
/// because only here are `file_size` and `duration` both in hand.
pub fn tier(ext: &str, file_size: u64, duration_secs: i64) -> &'static str {
    if is_lossless(ext, file_size, duration_secs) {
        "LOSSLESS"
    } else {
        "HIGH"
    }
}

/// CD's ceiling. 16-bit / 44.1 kHz stereo is ~1411 kbps even uncompressed, so a
/// lossless file with MORE bits per second than that cannot be a CD copy.
pub const CD_KBPS: u64 = 1411;

/// Lossless AND above CD bitrate — i.e. a genuine hi-res master.
///
/// MEASURED 2026-11-07: `Guns N' Roses Sweet Child O' Mine` came back with 30
/// rows and the backfill stayed silent, because a `LOSSLESS` row already existed.
/// But the catalogue cannot say WHETHER that row is a CD copy or a 24/96 master:
/// `tier()` only distinguishes `LOSSLESS` from `HIGH`, and a 16/44.1 FLAC and a
/// 24/96 FLAC both read `LOSSLESS`. Bitrate is the one signal that separates them,
/// which is why `format_track()` now emits `size` for the trigger to read.
pub fn is_hi_res(ext: &str, file_size: u64, duration_secs: i64) -> bool {
    is_lossless(ext, file_size, duration_secs) && kbps(file_size, duration_secs) > CD_KBPS
}

/// `fd_eclipse_audio_quality_score()` — higher is better.
pub fn quality_score(ext: &str, size_bytes: u64) -> i64 {
    let weight = match ext.to_lowercase().as_str() {
        "flac" | "wav" => 2000,
        "m4a" | "alac" => 1200,
        "aac" | "opus" => 800,
        "mp3" => 400,
        _ => 100,
    };
    let bonus = std::cmp::min(500, (size_bytes as f64 / (100.0 * 1024.0)).round() as i64);
    weight + bonus
}

/// `fd_eclipse_format_track()` — one `track` object for the addon.
pub fn format_track(item: &Value) -> Value {
    // `file_name` is the real name of the file; the flat `title` column is that
    // same name, so it is a name, never a song title.
    let file_name = {
        let f = tag(item, "file_name");
        if f.is_empty() {
            str_at(item, "title")
        } else {
            f
        }
    };
    let mut artist = tag(item, "performer");
    let mut title = tag(item, "song_title");

    let name = name_words(&file_name);
    if title.is_empty() {
        title = strip_artist(&name, &artist);
    }
    if artist.is_empty() {
        artist = name.clone();
    }

    let ext = {
        let e = tag(item, "extension").to_lowercase();
        if e.is_empty() || e == "noext" {
            file_name
                .rsplit_once('.')
                .map(|(_, e)| e.to_lowercase())
                .unwrap_or_default()
        } else {
            e
        }
    };
    let ext = if ext.is_empty() { "mp3".to_string() } else { ext };

    let duration = item
        .get("audio")
        .and_then(|a| a.get("duration"))
        .and_then(|v| v.as_i64())
        .or_else(|| item.get("duration").and_then(|v| v.as_i64()))
        .unwrap_or(0);
    // `/search-music` sends `file_size` on every row (main.php:2565); it is the
    // only free way to tell an ALAC `.m4a` from an AAC one.
    let file_size = item.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0);

    let mut track = serde_json::json!({
        "id": str_at(item, "short_code"),
        "title": title,
        "artist": artist,
        "duration": duration,
        "durationMs": if duration > 0 { duration * 1000 } else { 180000 },
        // The byte count, so a caller can tell a CD FLAC from a hi-res master
        // (`is_hi_res`). Addons ignore keys they do not know.
        "size": file_size,
        // The file's real container. No `.wav` alias: that rename is the
        // legacy trick that made the download path skip the ALAC transcode and
        // answer `audio/wav` with raw MP4 bytes.
        "format": ext,
        // BitChord reads this to pick a rendition; Eclipse ignores it.
        "audioQuality": tier(&ext, file_size, duration),
    });
    for (key, value) in [("album", tag(item, "album")), ("isrc", tag(item, "isrc"))] {
        if !value.is_empty() {
            track[key] = serde_json::json!(value);
        }
    }
    let thumb = tag(item, "thumbnail_url");
    if !thumb.is_empty() {
        track["artworkURL"] = serde_json::json!(thumb);
    }
    track
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The re-sent-file shape: real tags, nothing to derive.
    #[test]
    fn uses_the_audio_tags_from_the_resolve() {
        let track = format_track(&serde_json::json!({
            "short_code": "wULLeI",
            "audio": {
                "duration": 217,
                "file_name": "Ella Langley - Be Her.flac",
                "mime_type": "audio/x-flac",
                "title": "Be Her",
                "performer": "Ella Langley"
            }
        }));
        assert_eq!(track["title"], "Be Her");
        assert_eq!(track["artist"], "Ella Langley");
        assert_eq!(track["duration"], 217);
        assert_eq!(track["durationMs"], 217000);
        assert_eq!(track["format"], "flac");
    }

    /// The `/search-music` row shape: `song_title` is the origin's caption
    /// parse, which is the only place a `'` survives.
    #[test]
    fn song_title_from_the_origin_beats_the_file_name() {
        let track = format_track(&serde_json::json!({
            "short_code": "8VRbcL",
            "title": "01.My.Body.Isn.t.Ready.m4a",
            "song_title": "My Body Isn't Ready",
            "performer": "sombr",
            "extension": "m4a",
            "duration": 217
        }));
        assert_eq!(track["title"], "My Body Isn't Ready");
        assert_eq!(track["artist"], "sombr");
        assert_eq!(track["format"], "m4a");
    }

    /// No `song_title` (an older origin): the name minus the performer.
    #[test]
    fn falls_back_to_the_name_minus_the_performer() {
        let track = format_track(&serde_json::json!({
            "short_code": "wULLeI",
            "title": "Ella.Langley.Be.Her.mp3",
            "performer": "Ella Langley",
            "extension": "mp3",
            "duration": 217
        }));
        assert_eq!(track["artist"], "Ella Langley");
        assert_eq!(track["title"], "Be Her");
        assert_eq!(track["format"], "mp3");
    }

    /// The client re-matches a track by `title + artist`, so this must never be
    /// empty — `backend.php` derives one from the name too.
    #[test]
    fn a_name_without_a_separator_is_its_own_artist() {
        let track = format_track(&serde_json::json!({
            "short_code": "dKnM62",
            "title": "back.to.friends.wav",
            "extension": "wav"
        }));
        assert_eq!(track["artist"], "back to friends");
        assert_eq!(track["title"], "back to friends");
        assert_eq!(track["durationMs"], 180000);
    }

    /// No `.wav` alias: the advertised format is the container that is actually
    /// stored, and the transcode announces itself in the response headers.
    #[test]
    fn an_alac_m4a_keeps_its_real_format() {
        let track = format_track(&serde_json::json!({
            "short_code": "x1",
            "title": "Adele.Hello.m4a",
            "performer": "Adele",
            "extension": "m4a"
        }));
        assert_eq!(track["title"], "Hello");
        assert_eq!(track["format"], "m4a");
    }

    /// The measured pair from `Sial`: same 243 s, both `.m4a`, 269 vs 879 kbps.
    #[test]
    fn a_lossless_m4a_is_labelled_lossless_and_a_lossy_one_is_not() {
        let alac = format_track(&serde_json::json!({
            "short_code": "J2YkvY",
            "title": "07.Sial.m4a",
            "performer": "Mahalini",
            "extension": "m4a",
            "duration": 243,
            "file_size": 26_704_889_u64
        }));
        assert_eq!(alac["audioQuality"], "LOSSLESS");
        assert_eq!(alac["format"], "m4a");

        let aac = format_track(&serde_json::json!({
            "short_code": "Xt8C7v",
            "title": "07.Sial.m4a",
            "performer": "Mahalini",
            "extension": "m4a",
            "duration": 243,
            "file_size": 8_187_833_u64
        }));
        assert_eq!(aac["audioQuality"], "HIGH");
    }

    /// No size or no duration means no bitrate, and no bitrate means no claim.
    #[test]
    fn an_m4a_without_a_size_is_not_called_lossless() {
        assert_eq!(tier("m4a", 0, 243), "HIGH");
        assert_eq!(tier("m4a", 26_704_889, 0), "HIGH");
        assert_eq!(tier("flac", 0, 243), "LOSSLESS");
        assert_eq!(kbps(26_704_889, 243), 879);
        assert_eq!(kbps(8_187_833, 243), 269);
    }

    #[test]
    fn quality_scores_order() {
        assert!(quality_score("flac", 10_000_000) > quality_score("m4a", 10_000_000));
        assert!(quality_score("m4a", 10_000_000) > quality_score("mp3", 10_000_000));
        assert!(quality_score("mp3", 100 * 1024) > quality_score("mp3", 0));
    }

    /// CD versus hi-res is a bitrate question, and it is the whole reason the
    /// trigger exists: both read `LOSSLESS`, only one is worth a 40 MB upload.
    /// 240 s: a CD FLAC lands near 900 kbps, a 24/96 master near 4600.
    #[test]
    fn only_above_cd_bitrate_counts_as_hi_res() {
        let cd = 27_000_000u64;
        let hi = 138_000_000u64;
        assert_eq!(kbps(cd, 240), 900);
        assert_eq!(kbps(hi, 240), 4600);
        assert!(is_lossless("flac", cd, 240), "a CD FLAC is still lossless");
        assert!(!is_hi_res("flac", cd, 240), "…but it is not hi-res");
        assert!(is_hi_res("flac", hi, 240));
        assert!(!is_hi_res("mp3", hi, 240), "a big mp3 is never hi-res");
        assert!(!is_hi_res("flac", 0, 240), "no size, no claim");
    }

    /// The row carries the byte count the trigger reads; without it every song
    /// looks like it has no hi-res master.
    #[test]
    fn a_track_row_carries_the_size_the_trigger_needs() {
        let t = format_track(&serde_json::json!({
            "short_code": "abc123",
            "title": "Guns N' Roses - Sweet Child O' Mine.flac",
            "performer": "Guns N' Roses",
            "song_title": "Sweet Child O' Mine",
            "extension": "flac",
            "duration": 356,
            "file_size": 138_000_000_u64
        }));
        assert_eq!(t["size"], 138_000_000u64);
        assert_eq!(t["title"], "Sweet Child O' Mine");
        assert!(is_hi_res(
            t["format"].as_str().unwrap_or(""),
            t["size"].as_u64().unwrap_or(0),
            t["duration"].as_i64().unwrap_or(0)
        ));
    }
}
