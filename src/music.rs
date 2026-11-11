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

/* ── FLAC STREAMINFO ─────────────────────────────────────────────────────── */

/// What a FLAC's STREAMINFO block states about the recording itself.
///
/// WHY THIS EXISTS. Some origins send a FLAC as a Telegram `document` with no
/// Audio attribute, so the row has no `duration`. Every quality signal below
/// reads the duration, and [`kbps()`] answers 0 without one, so [`is_hi_res()`]
/// can never be true: a real 24/96 master ranks as a CD copy forever, and
/// `tier_satisfied()` keeps triggering a fill for a file the catalogue already
/// holds.
///
/// The numbers are in the file. STREAMINFO is the first metadata block — 34
/// bytes at offset 8 — and the head of every stream is already fetched
/// (`PRIME_BYTES` is 512 KB, for a block that ends at byte 42).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlacInfo {
    pub sample_rate: u32,
    pub channels: u8,
    pub bit_depth: u8,
    pub total_samples: u64,
}

impl FlacInfo {
    /// Whole seconds, or `0` when the encoder wrote no total (a streaming encoder
    /// may). Rounded DOWN, so `kbps` errs high — the safe direction for a clip
    /// gate that exists to refuse slices.
    pub fn duration_secs(&self) -> u32 {
        if self.sample_rate == 0 {
            return 0;
        }
        (self.total_samples / u64::from(self.sample_rate)) as u32
    }
}

/// Parse STREAMINFO out of the first bytes of a FLAC file.
///
/// Layout (FLAC format specification, "STREAMINFO"): 2+2 bytes block sizes, 3+3
/// bytes frame sizes, then a 20-bit sample rate, 3-bit `channels-1`, 5-bit
/// `bitDepth-1` and a 36-bit total sample count.
///
/// Byte 4 is the metadata-block header: bit 7 is "last block", bits 0-6 the
/// type. STREAMINFO is type 0 and always comes first. A set "last block" bit is
/// ACCEPTED here — a FLAC whose only block is STREAMINFO (no tags, no seek
/// table) sets it and plays perfectly; rejecting it hides a valid file.
pub fn flac_info(head: &[u8]) -> Option<FlacInfo> {
    if head.len() < 42 || &head[0..4] != b"fLaC" {
        return None;
    }
    if head[4] & 0x7f != 0 {
        return None;
    }
    let block_len =
        (usize::from(head[5]) << 16) | (usize::from(head[6]) << 8) | usize::from(head[7]);
    if block_len < 34 {
        return None;
    }
    let si = &head[8..42];
    let sample_rate =
        (u32::from(si[10]) << 12) | (u32::from(si[11]) << 4) | (u32::from(si[12]) >> 4);
    if sample_rate == 0 {
        return None;
    }
    let channels = ((si[12] >> 1) & 0x7) + 1;
    let bit_depth = (((si[12] & 0x1) << 4) | (si[13] >> 4)) + 1;
    let total_samples = (u64::from(si[13] & 0x0f) << 32)
        | (u64::from(si[14]) << 24)
        | (u64::from(si[15]) << 16)
        | (u64::from(si[16]) << 8)
        | u64::from(si[17]);
    Some(FlacInfo {
        sample_rate,
        channels,
        bit_depth,
        total_samples,
    })
}

/// The length a FLAC's own header states, when it states one.
pub fn flac_duration(head: &[u8]) -> Option<u32> {
    flac_info(head)
        .map(|i| i.duration_secs())
        .filter(|secs| *secs > 0)
}

#[cfg(test)]
mod flac_tests {
    use super::*;

    /// A `fLaC` head whose STREAMINFO states exactly these values.
    fn head_with(
        last_block: bool,
        sample_rate: u32,
        channels: u8,
        bit_depth: u8,
        total: u64,
    ) -> Vec<u8> {
        let mut v = vec![0u8; 42];
        v[0..4].copy_from_slice(b"fLaC");
        v[4] = if last_block { 0x80 } else { 0x00 };
        v[7] = 34;
        let si = &mut v[8..42];
        si[10] = (sample_rate >> 12) as u8;
        si[11] = (sample_rate >> 4) as u8;
        si[12] = (((sample_rate & 0x0f) as u8) << 4)
            | ((channels - 1) << 1)
            | ((bit_depth - 1) >> 4);
        si[13] = (((bit_depth - 1) & 0x0f) << 4) | (((total >> 32) & 0x0f) as u8);
        si[14] = (total >> 24) as u8;
        si[15] = (total >> 16) as u8;
        si[16] = (total >> 8) as u8;
        si[17] = total as u8;
        v
    }

    #[test]
    fn a_hi_res_flac_states_its_own_length_and_depth() {
        let h = head_with(false, 96_000, 2, 24, 96_000 * 300);
        let info = flac_info(&h).expect("a valid STREAMINFO parses");
        assert_eq!(info.sample_rate, 96_000);
        assert_eq!(info.bit_depth, 24);
        assert_eq!(info.channels, 2);
        assert_eq!(info.duration_secs(), 300);
    }

    /// THE BUG THIS PARSER EXISTS FOR.
    ///
    /// A 90 MB FLAC whose row says nothing can never be hi-res, because `kbps()`
    /// answers 0 without a duration. The same bytes, once the header is read,
    /// clear CD's ceiling — and no other field had to change.
    #[test]
    fn an_unknown_length_hides_a_hi_res_master_until_the_header_is_read() {
        let bytes = 90_000_000u64;
        let h = head_with(false, 96_000, 2, 24, 96_000 * 300);
        assert!(!is_hi_res("flac", bytes, 0), "duration 0 can never be hi-res");

        let secs = flac_duration(&h).expect("the header states 300 s") as i64;
        assert_eq!(secs, 300);
        assert!(kbps(bytes, secs) > CD_KBPS, "{} kbps", kbps(bytes, secs));
        assert!(is_hi_res("flac", bytes, secs));
    }

    /// Reading the header must not promote everything: a CD copy stays a CD copy.
    #[test]
    fn a_16_44_flac_is_not_promoted_by_reading_its_header() {
        let bytes = 35_188_598u64;
        let h = head_with(false, 44_100, 2, 16, 44_100 * 200);
        let secs = flac_duration(&h).expect("the header states 200 s") as i64;
        assert_eq!(secs, 200);
        assert!(is_lossless("flac", bytes, secs));
        assert!(!is_hi_res("flac", bytes, secs), "{} kbps", kbps(bytes, secs));
    }

    /// A file with no tags sets "last metadata block" on STREAMINFO itself. That
    /// is a valid FLAC and must not be hidden.
    #[test]
    fn a_lone_streaminfo_block_is_still_a_flac() {
        let h = head_with(true, 44_100, 2, 16, 44_100 * 12);
        assert_eq!(flac_duration(&h), Some(12));
    }

    #[test]
    fn a_streaming_flac_with_no_total_sample_count_reports_no_length() {
        let h = head_with(false, 48_000, 2, 24, 0);
        assert_eq!(flac_duration(&h), None, "0 samples means unknown, not zero");
        assert!(flac_info(&h).is_some(), "rate and depth are still readable");
    }

    #[test]
    fn anything_that_is_not_a_flac_head_is_refused() {
        assert_eq!(flac_info(b""), None);
        assert_eq!(flac_info(b"ID3\x03\x00\x00\x00"), None);
        assert_eq!(flac_info(&vec![0x00u8; 64]), None);
        // Right magic, wrong first block: STREAMINFO is type 0 and always first.
        let mut v = head_with(false, 44_100, 2, 16, 44_100);
        v[4] = 0x04;
        assert_eq!(flac_info(&v), None);
        let mut html = b"<!DOCTYPE html>".to_vec();
        html.resize(64, b' ');
        assert_eq!(flac_info(&html), None);
    }

    /// THE WIRING, not just the arithmetic: a head read once by the download
    /// prime must make the same track rank as hi-res on the next stream answer.
    #[test]
    fn a_head_read_once_makes_a_hi_res_master_rank_as_one() {
        let code = "TESTFLACHEAD0001";
        let bytes = 90_000_000u64;
        let h = head_with(false, 96_000, 2, 24, 96_000 * 300);
        assert!(!is_hi_res("flac", bytes, 0), "before: unknown length");

        remember_head(code, bytes, &h);

        let probe = known_probe(code).expect("the head is cached for this code");
        let secs = probe.get("duration").and_then(|v| v.as_i64()).unwrap_or(0);
        assert_eq!(secs, 300);
        assert_eq!(probe["sampleRate"], serde_json::json!(96_000));
        assert_eq!(probe["bitDepth"], serde_json::json!(24));
        let size = probe["size"].as_u64().unwrap_or(0);
        assert!(is_hi_res("flac", size, secs), "after: {} kbps", kbps(size, secs));
    }
}

/* ── MP4 / M4A (the ALAC path) ───────────────────────────────────────────── */

/// The length an MP4 `.m4a` states in `moov/mvhd`.
///
/// ALAC is the container that most needs this. `is_lossless("m4a", size, dur)` is
/// a BITRATE test, because the extension genuinely cannot tell ALAC from AAC —
/// and with no duration the bitrate is 0, so a real ALAC is classified lossy and
/// `codec_of()` answers `""` instead of `alac`. Same root cause as the FLAC case,
/// different box.
///
/// `moov` may sit at the END of the file (no faststart), where the head does not
/// contain it. This then returns `None` on purpose: an unknown length is better
/// than a guessed one, and the tail a player probes anyway is where `moov` lives.
pub fn mp4_duration(head: &[u8]) -> Option<u32> {
    let (moov_from, moov_to) = find_box(head, 0, head.len(), b"moov")?;
    let (mvhd_from, mvhd_to) = find_box(head, moov_from, moov_to, b"mvhd")?;
    let body = head.get(mvhd_from..mvhd_to)?;
    if body.len() < 20 {
        return None;
    }
    let (timescale, duration) = if body[0] == 1 {
        // version 1: 64-bit creation, modification and duration.
        if body.len() < 32 {
            return None;
        }
        (
            u32::from_be_bytes([body[20], body[21], body[22], body[23]]),
            u64::from_be_bytes([
                body[24], body[25], body[26], body[27], body[28], body[29], body[30], body[31],
            ]),
        )
    } else {
        (
            u32::from_be_bytes([body[12], body[13], body[14], body[15]]),
            u64::from(u32::from_be_bytes([
                body[16], body[17], body[18], body[19],
            ])),
        )
    };
    if timescale == 0 {
        return None;
    }
    let secs = duration / u64::from(timescale);
    if secs == 0 || secs > u64::from(u32::MAX) {
        return None;
    }
    Some(secs as u32)
}

/// `(payload_start, payload_end)` of the first box named `kind` in `head`, inside
/// the byte range `[from, to)`. Payload starts after the box header.
fn find_box(head: &[u8], from: usize, to: usize, kind: &[u8; 4]) -> Option<(usize, usize)> {
    let to = to.min(head.len());
    let mut p = from;
    while p + 8 <= to {
        let size = u32::from_be_bytes([head[p], head[p + 1], head[p + 2], head[p + 3]]) as usize;
        let name = &head[p + 4..p + 8];
        // size 1 = a 64-bit size follows the type; size 0 = to end of file.
        let (header, box_size) = match size {
            0 => (8usize, to - p),
            1 => {
                if p + 16 > to {
                    return None;
                }
                let large = u64::from_be_bytes([
                    head[p + 8],
                    head[p + 9],
                    head[p + 10],
                    head[p + 11],
                    head[p + 12],
                    head[p + 13],
                    head[p + 14],
                    head[p + 15],
                ]) as usize;
                (16usize, large)
            }
            _ => (8usize, size),
        };
        if box_size < header {
            return None;
        }
        let end = p.saturating_add(box_size).min(to);
        if name == kind {
            return Some((p + header, end));
        }
        p = p.saturating_add(box_size);
    }
    None
}

/// The codec an `.m4a` may be called, from its bitrate. The ONE rule for ALAC,
/// because the extension cannot tell it from AAC: a lossless-class bitrate is
/// claimed as `alac`, anything lower is left UNKNOWN rather than guessed as
/// `aac` (a wrong codec makes the player pick the wrong engine before byte one).
///
/// `routes::codec_of()` calls this, so the rule is testable beside the duration
/// it depends on.
pub fn alac_or_unknown(file_size: u64, duration_secs: i64) -> &'static str {
    if is_lossless("m4a", file_size, duration_secs) {
        "alac"
    } else {
        ""
    }
}

/// Cache namespace for what a file's header said about itself.
const PROBE_PREFIX: &str = "audio_probe";
/// A file's own header does not change, so this can live far longer than a
/// resolve. The entry is three small numbers.
const PROBE_TTL_SECS: i64 = 30 * 24 * 3600;

/// Publish what the head of a file says about it, keyed by short code.
///
/// Called from the download prime, where the bytes are already in hand: the 512 KB
/// prime exists for ranges, and a FLAC header ends at byte 42. Costs a slice.
///
/// Uses the engine's ONE cache ([`crate::cache`]) rather than a second one; the
/// value is `{duration, sampleRate, bitDepth, channels}`, all of them read from
/// the bytes, none of them inferred.
pub fn remember_head(short_code: &str, file_size: u64, head: &[u8]) {
    if short_code.is_empty() {
        return;
    }
    let mut body = serde_json::json!({"size": file_size});
    let mut found = false;
    if let Some(info) = flac_info(head) {
        found = true;
        body["sampleRate"] = serde_json::json!(info.sample_rate);
        body["bitDepth"] = serde_json::json!(info.bit_depth);
        body["channels"] = serde_json::json!(info.channels);
    }
    if let Some(secs) = flac_duration(head) {
        body["duration"] = serde_json::json!(secs);
    }
    if let Some(secs) = mp4_duration(head) {
        found = true;
        body["duration"] = serde_json::json!(secs);
    }
    if !found {
        return;
    }
    crate::cache::put(PROBE_PREFIX, short_code, &body, PROBE_TTL_SECS, 0);
}

/// What a previously seen file head said about this short code.
///
/// The ONLY source of a `sampleRate`/`bitDepth` this engine is willing to state:
/// both come from the file's own STREAMINFO, never from the extension.
pub fn known_probe(short_code: &str) -> Option<Value> {
    crate::cache::get(PROBE_PREFIX, short_code).map(|v| (*v).clone())
}

#[cfg(test)]
mod mp4_tests {
    use super::*;

    fn box_of(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut v = ((payload.len() + 8) as u32).to_be_bytes().to_vec();
        v.extend_from_slice(kind);
        v.extend_from_slice(payload);
        v
    }

    fn mvhd_v0(timescale: u32, duration: u32) -> Vec<u8> {
        let mut p = vec![0u8; 12]; // version + flags, creation, modification
        p.extend_from_slice(&timescale.to_be_bytes());
        p.extend_from_slice(&duration.to_be_bytes());
        p
    }

    #[test]
    fn an_alac_m4a_states_its_length_in_mvhd() {
        let mut head = box_of(b"ftyp", b"M4A \0\0\0\0M4A mp42isom");
        head.extend_from_slice(&box_of(b"moov", &box_of(b"mvhd", &mvhd_v0(44_100, 44_100 * 243))));
        assert_eq!(mp4_duration(&head), Some(243));
    }

    /// The whole point: ALAC's verdict is a bitrate test, so the length decides.
    #[test]
    fn an_unknown_length_classes_alac_as_lossy_until_mvhd_is_read() {
        let bytes = 26_704_889u64; // "Sial" ALAC, 243 s
        assert!(!is_lossless("m4a", bytes, 0), "no length means no bitrate");
        assert_eq!(alac_or_unknown(bytes, 0), "", "unknown, never guessed as aac");

        let mut head = box_of(b"ftyp", b"M4A \0\0\0\0");
        head.extend_from_slice(&box_of(b"moov", &box_of(b"mvhd", &mvhd_v0(44_100, 44_100 * 243))));
        let secs = mp4_duration(&head).expect("243 s") as i64;
        assert!(is_lossless("m4a", bytes, secs), "{} kbps", kbps(bytes, secs));
        assert_eq!(alac_or_unknown(bytes, secs), "alac");
    }

    /// No faststart: `moov` is at the end, so the head cannot answer. Say so.
    #[test]
    fn a_file_whose_moov_is_at_the_end_reports_nothing() {
        let mut head = box_of(b"ftyp", b"M4A \0\0\0\0");
        head.extend_from_slice(&box_of(b"free", &[0u8; 32]));
        assert_eq!(mp4_duration(&head), None);
    }

    #[test]
    fn a_header_that_states_no_length_is_not_treated_as_one() {
        // Right magic and a valid STREAMINFO, but the encoder wrote no total.
        let mut flac = vec![0u8; 42];
        flac[0..4].copy_from_slice(b"fLaC");
        flac[7] = 34;
        // STREAMINFO starts at file offset 8, so its byte 10 is file byte 18.
        flac[18] = 0x0a; // 44_100 Hz across the top 20 bits
        flac[19] = 0xc4;
        flac[20] = 0x40;
        assert!(flac_info(&flac).is_some(), "the rate is still readable");
        assert_eq!(flac_duration(&flac), None, "0 samples means unknown");
        // An MP3's tag is not an MP4 box.
        assert_eq!(mp4_duration(b"ID3\x04\x00\x00\x00\x00\x00\x00\x00"), None);
    }
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
    // An UNKNOWN length is omitted, never filled in.
    //
    // MEASURED 2026-11-11: this used to send `"durationMs": 180000` for every row
    // the catalogue had no duration for. That is a fabricated 3:00 — a client that
    // shows it gives the wrong length and a seek past the real end fails. The
    // `duration: 0` beside it is the honest value, and it is the ONE field every
    // quality signal is derived from (`is_lossless`, `is_hi_res`, `kbps`), so a
    // milliseconds twin that is guessed buys nothing and lies.
    if duration > 0 {
        track["durationMs"] = serde_json::json!(duration * 1000);
    }
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
    }

    /// An unknown length is OMITTED, never invented.
    ///
    /// A `duration: 0` row is a file Telegram holds as a `document`, so it carries
    /// no Audio attribute and the catalogue genuinely does not know its length.
    /// MEASURED 2026-11-11: every such row was answered `durationMs: 180000`, a
    /// fabricated 3:00 that a player displays and then cannot seek through.
    #[test]
    fn an_unknown_duration_is_omitted_never_invented() {
        let track = format_track(&serde_json::json!({
            "short_code": "dKnM62",
            "title": "back.to.friends.wav",
            "extension": "wav"
        }));
        assert_eq!(track["duration"], 0);
        assert!(
            track.get("durationMs").is_none(),
            "an unknown duration must not be filled in with a placeholder"
        );
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
