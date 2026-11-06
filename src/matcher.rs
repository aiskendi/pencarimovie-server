//! Title / episode matcher — port of the `matching` group in
//! [`port/COVERAGE.md`](../../COVERAGE.md) (`backend.php:7804-9460`).
//!
//! This is the group that decides whether a Telegram file belongs to the title
//! the user opened, so `/stream/movie/pm:post:…`, `/stream/series/…` and `tt…`
//! can list anything at all. Until it existed those routes answered
//! `{"streams":[]}` (see PARITY.md) even for titles that have 100 playable
//! files.
//!
//! Translation, not redesign: the ported source of truth is the Kotlin twin in
//! [`android/pencarimovie-server`](../../../android/pencarimovie-server/app/src/main/java/com/pencarimovie/server/NativeHttpServer.kt)
//! (`movieFileMatchesTitle` / `seriesFileMatchesTitle` / `classifySeasonEpisode`
//! / `extractSplitPartInfo` / `applyStreamFilters`), which was verified
//! on-device against `backend.php` and carries every fix from the
//! "Do not regress" sections of `AGENTS.md`. When PHP, Kotlin and this file
//! disagree, the Kotlin is the tie-breaker because it was diffed against the
//! PHP twice.
//!
//! ## The regression traps that are load-bearing here
//!
//! Each of these is pinned by a unit test at the bottom of this file:
//!
//! 1. **A year EMBEDDED in a series name is never stripped.** "Kudrat 1968:
//!    High Council 2026" keeps its `1968`; only a *trailing* release year is
//!    removed. Stripping the first occurrence left a phrase that appears in no
//!    file name, so every episode of that show was rejected.
//! 2. **A codec suffix is not a split-part index.** `…AAC2.0.H.264.mkv` used to
//!    parse as "part 264", and because `exclude_part_title` defaults to ON,
//!    every H.264/H.265 release vanished from every stream list. `263..265` are
//!    excluded.
//! 3. **A lone "Part N" is a TITLE, not a split.** `Detective.Conan…Part.1`
//!    (1.28 GB, one complete episode) was hidden. A part is only flagged when a
//!    sibling shares the base key with a *different* number, or the caption
//!    declares a total.
//! 4. **A title whose last word is a type word.** The origin's keyword cleaner
//!    answers `resolved_title = "The Super Mario Galaxy"` for the film *The
//!    Super Mario Galaxy Movie*, so strict equality played 1 stream instead of
//!    100. A same-year file that is the target plus a type word matches.
//! 5. **Accents.** The catalogue says `Siccîn 9`, the files say `Siccin.9`, so
//!    the folded forms are compared too. Fold the *already-normalised* strings,
//!    never the raw input (`iconv` turns `•` into `*` and collapses 100 streams
//!    to 0).
//! 6. **Sibling works.** One Piece anime must not list the LEGO / MOVIE / OVA /
//!    LIVE-ACTION releases; kind markers the entry title does not carry reject
//!    the file.
//! 7. **A quality tag is not an episode subtitle.** `…E01.450p.HANrel…` kept
//!    "450p hanerl re encoded to mq" as a "subtitle" and the episode-name guard
//!    then rejected a legitimate release. Resolution tokens stop the region
//!    generically, and a ONE-WORD tail (an uploader/group name, or `end`) is
//!    inconclusive — fail open.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use unicode_normalization::UnicodeNormalization;

/* ── shared pattern constants (Kotlin `private val …`) ─────────────────────── */

const RELEASE_TAGS: &str = "(?i)\\b(?:2160p|1080p|720p|540p|480p|360p|4k|uhd|fhd|hd|sd|bluray|blu-ray|bdrip|brrip|web-?dl|webrip|hdrip|hdtv|cam|predvd|predvdrip|ts|tc|r5|dvdrip|remux|hevc|x264|x265|h264|h265|aac.*|lubokvideo|yts|lulustream|galaxyrg|pahe)\\b.*";
const EDITION_TAGS: &str = "(?i)\\b(?:extended(?:\\s*cut)?|directors?\\s*cut|unrated|imax|special\\s*edition|remastered|criterion|sub\\s*malay|malay\\s*sub|eng\\s*sub|sub\\s*eng|indosub|sub\\s*indo|subtitle|dubbed|multi\\s*audio|clean\\s*audio|hc|hardsub)\\b.*";
const SEQUEL_RE: &str = "(?i)\\b(?:part\\s*(?:ii|iii|iv|v|\\d+)|(?:2|3|4|5|6|7|8|9)|chapter\\s*\\d+)\\b";
const UPLOADER_RE: &str = "(?i)^(?:(?:sklink|office|movie|movies|king|film|films|cinema|cinemas|media|channel|tv|tele|tg|hub|world|zone|group|team|club|gang|bot|vip|official|download|downloads|upload|uploads|site|store|station|blasters|rockers|wap|villa|dub|yogi|verse|flix|pahe|galaxy|pms|mkv|link|links|hd)\\b\\s*)+$";
const TARGET_TAGS: &str = "(?i)\\b(?:2160p|1080p|720p|480p|360p|uhd|fhd|hd|sd|hdtv|web-?dl|webrip|bluray|blu-ray|remux|dvdrip|hevc|x264|x265|h264|h265|\\d+(?:\\.\\d+)?\\s*(?:gb|mb))\\b";
// The separator class after a known group is `.`, `_`, `-` OR whitespace, not just
// whitespace. Telegram uploads separate words with DOTS (`PrakyTV.Avaraachan.&.Sons.
// 2026.Tamil…`), so with `\s+` the `prakytv` entry never fired, the uploader prefix
// stayed part of the title, and every such file was rejected — measured on
// `pm:post:9296788885` "Avaraachan & Sons 2026", whose six origin files all failed
// while `pm_search_files` listed seven. Stripping is followed by the strict title
// comparison, so widening the separator can only remove a PREFIX, never turn a
// different film into a match.
const LEADING_GROUPS: &str = "(?i)^(?:prakytv|ngefilm\\s*store|runningmovieshd|kannadachallengers|mkvcinemas|vegamovies|moviesmod|pahe|galaxy|wetv|studioghibli|animerg|anime\\s*time\\s*studio\\s*ghibli\\s*movie\\s*\\d*|mcu|pms|pahe\\.li|layarkaca\\d*|cinemaindo|indoxxi|sklink(?:\\s*office)?|movie\\s*king|movie\\s*world|film\\s*world|tamilblasters|1tamilmv|tamilmv|tamilrockers|tamilyogi|tamilgun|isaidub|kuttymovies|moviesda|bolly4u|desiremovies|worldfree4u|katmoviehd|skymovieshd|filmywap|filmyzilla|extramovies|9xmovies|movieswood|jiorockers|todaypk|cinevood|hdhub4u|dotmovies|topmovies|modmobile|uhdmovies|allmovieshub|moviesverse|luxmovie|cinemalover|flixhub|movies4u|plexmovies|moviesrocker|mkvking|psa|yts)[._\\s-]+";
const COUNTRY_PREFIX: &str = "(?i)^(?:ph|id|ina|my|kr|kor|jp|jpn|th|thai|cn|chi|us|uk|hk|vn|tw|ru|tr|es|fr|de|it|in)\\s+";
/// `^(?:video(\.\d+)*|\d+|document|file)\.(?:mp4|mkv|avi|mov|ts|flv)$` — a file
/// whose NAME carries nothing, so the caption's first line is the real title.
const GENERIC_NAME: &str =
    "(?i)^(?:video(?:\\.\\d+)*|\\d+|document|file)\\.(?:mp4|mkv|avi|mov|ts|flv)$";

/* ── tiny regex helpers ────────────────────────────────────────────────────── */

thread_local! {
    /// Compiled-pattern cache. Patterns are a fixed set of literals plus a few
    /// per-title dynamic ones (the country-prefix lookahead uses the target
    /// title), so the cache is capped instead of growing with title variety.
    static RE_CACHE: RefCell<HashMap<String, Rc<fancy_regex::Regex>>> =
        RefCell::new(HashMap::new());
}

fn rx(pattern: &str) -> Rc<fancy_regex::Regex> {
    RE_CACHE.with(|cell| {
        let mut map = cell.borrow_mut();
        if let Some(hit) = map.get(pattern) {
            return hit.clone();
        }
        if map.len() > 256 {
            map.clear();
        }
        let compiled = Rc::new(
            fancy_regex::Regex::new(pattern)
                .unwrap_or_else(|e| panic!("matcher: bad regex {pattern:?}: {e}")),
        );
        map.insert(pattern.to_string(), compiled.clone());
        compiled
    })
}

fn has(pattern: &str, text: &str) -> bool {
    rx(pattern).is_match(text).unwrap_or(false)
}

/// All capture groups (index 0 = whole match) as owned strings.
fn groups(pattern: &str, text: &str) -> Option<Vec<String>> {
    let re = rx(pattern);
    let caps = re.captures(text).ok().flatten()?;
    Some(
        (0..caps.len())
            .map(|i| caps.get(i).map(|m| m.as_str().to_string()).unwrap_or_default())
            .collect(),
    )
}

/// First match as `(start, end)` BYTE offsets.
fn span(pattern: &str, text: &str) -> Option<(usize, usize)> {
    let re = rx(pattern);
    let m = re.find(text).ok().flatten()?;
    Some((m.start(), m.end()))
}

fn replace_all(pattern: &str, text: &str, replacement: &str) -> String {
    rx(pattern)
        .replace_all(text, replacement)
        .into_owned()
}

fn escape(literal: &str) -> String {
    fancy_regex::escape(literal).into_owned()
}

/// `\s+` collapsed to one space, trimmed.
fn squash(s: &str) -> String {
    replace_all("\\s+", s, " ").trim().to_string()
}

/// `[^\p{L}\p{N}\s]` -> " ", collapse, trim, lowercase (Kotlin `normWords`).
fn norm_words(s: &str) -> String {
    let a = replace_all("[^\\p{L}\\p{N}\\s]", s, " ");
    squash(&a).to_lowercase()
}

/* ── fd_clean_media_title / fd_fold_latin_accents / decorations ────────────── */

/// Port of `fd_clean_media_title()` (backend.php:7897).
pub fn clean_media_title(title: &str) -> String {
    if title.is_empty() {
        return String::new();
    }
    let mut t = replace_all("[\\u{1F300}-\\u{1F9FF}\\u{2600}-\\u{26FF}\\u{2700}-\\u{27BF}]", title, " ");
    t = replace_all(
        "(?i)^(?:on9[._\\s]stream[._\\s]+|stream[._\\s]+|www\\.[a-z0-9.-]+\\.[a-z]{2,}[._\\s]+)",
        &t,
        "",
    );
    t = replace_all("(?i)forwarded[._\\s]from.*$", &t, "");
    t = replace_all(
        "(?i)(?:Join[._\\s]Channel|Join[._\\s]Group|Join[._\\s]us|Join[._\\s]@).*$",
        &t,
        "",
    );
    t = replace_all("(?i)kumpulan[._\\s]drama.*$", &t, "");
    t = replace_all(
        "(?i)(?:https?://|httpst\\.me|https?\\.?t\\.me|\\bt\\.me/)[\\w./?=&_-]*",
        &t,
        "",
    );
    t = replace_all(
        "(?i)(?:[.\\s_-]*\\d+(?:[.,]\\d+)?[.\\s_-]*(?:MB|GB|KB|TB))+(?:[.\\s_-]*https)?(?:[.\\s_-]*Open[.\\s_-]*Mini[.\\s_-]*App)?$",
        &t,
        "",
    );
    t.trim_matches(|c: char| matches!(c, ' ' | '.' | '_' | '-' | '=' | '\t' | '\n' | '\r'))
        .to_string()
}

/// Port of `fd_fold_latin_accents()` (backend.php:4997).
///
/// NFD + strip combining marks. Two behaviours matter and both are asserted in
/// the tests: a script that cannot be transliterated (Cyrillic, Hebrew, Arabic,
/// Devanagari, Thai, Hangul, CJK, full-width) is returned UNCHANGED, and the
/// bullet characters are turned into spaces only as part of the same pass.
pub fn fold_latin_accents(s: &str) -> String {
    if s.is_empty() || !has("[^\\x20-\\x7E]", s) {
        return s.to_string();
    }
    if has(
        "[\\u{0400}-\\u{04FF}\\u{0590}-\\u{05FF}\\u{0600}-\\u{06FF}\\u{0900}-\\u{0DFF}\\u{0E00}-\\u{0EFF}\\u{1100}-\\u{11FF}\\u{2E80}-\\u{9FFF}\\u{AC00}-\\u{D7AF}\\u{F900}-\\u{FAFF}\\u{FF00}-\\u{FFEF}]",
        s,
    ) {
        return s.to_string();
    }
    let decomposed: String = s.nfd().collect();
    let stripped: String = decomposed
        .chars()
        .filter(|c| !is_combining_mark(*c))
        .collect();
    stripped.replace('•', " ").replace('·', " ")
}

fn is_combining_mark(c: char) -> bool {
    // Unicode general category M (Mn/Mc/Me) — the ranges Unicode calls
    // combining diacritical marks plus the common extended blocks.
    matches!(c,
        '\u{0300}'..='\u{036F}'   // Combining Diacritical Marks
        | '\u{0483}'..='\u{0489}' // Cyrillic
        | '\u{0591}'..='\u{05BD}'
        | '\u{0610}'..='\u{061A}'
        | '\u{064B}'..='\u{065F}'
        | '\u{1AB0}'..='\u{1AFF}'
        | '\u{1DC0}'..='\u{1DFF}'
        | '\u{20D0}'..='\u{20FF}'
        | '\u{FE20}'..='\u{FE2F}'
        | '\u{3099}'..='\u{309A}'
    )
}

/// Port of `fd_strip_title_decorations()` (backend.php:4957).
pub fn strip_title_decorations(title: &str) -> String {
    let mut t = replace_all("[\\u{2022}\\u{2027}\\u{00B7}|]+", title, " ");
    t = replace_all("\\b(19\\d\\d|20\\d\\d)\\b", &t, " ");
    t = replace_all("(?i)\\b(movie|tvseries|tv\\s*series|series)\\s*$", &t, " ");
    t = replace_all(
        "(?i)\\b(?:2160p|1440p|1080p|720p|576p|480p|360p|4k|uhd|blu-?ray|bluray|bdrip|brrip|remux|web-?dl|webrip|web|hdtv|hdrip|dvdrip|dvd|hdts|xvid|divx|hevc|x265|x264|h\\.?26[45]|avc|av1|vp9|aac|ac3|eac3|dts(?:-hd)?|truehd|atmos|ddp?|flac|10bit|8bit|hdr10\\+?|hdr|dovi|proper|repack)\\b.*$",
        &t,
        "",
    );
    squash(&t)
}

/// The season a title names, or `0` when it names none.
///
/// An addon item can BE one season of a show: AIOMetadata publishes Black Clover
/// season 2 as a separate entry titled "Black Clover Season 2 2026" whose videos
/// are numbered 1..13. Those episodes are `S02E01..S02E13` in the origin's files,
/// so treating the item's episode 2 as "S01E02" listed the FIRST season under a
/// season-2 episode (reported: "it show season 1").
pub fn season_from_title(title: &str) -> i32 {
    let t = decode_html_entities(title);
    if let Some(g) = groups("(?i)\\bseason\\s*0*(\\d{1,2})\\b", &t) {
        if let Ok(n) = g[1].parse::<i32>() {
            if n > 0 {
                return n;
            }
        }
    }
    if let Some(g) = groups("(?i)\\b0*(\\d{1,2})(?:st|nd|rd|th)\\s+season\\b", &t) {
        if let Ok(n) = g[1].parse::<i32>() {
            if n > 0 {
                return n;
            }
        }
    }
    for (word, n) in [
        ("second", 2i32),
        ("third", 3),
        ("fourth", 4),
        ("fifth", 5),
        ("sixth", 6),
    ] {
        if has(&format!("(?i)\\b{}\\s+season\\b", word), &t) {
            return n;
        }
    }
    0
}

/// `normTitleForMatch`: decorations stripped, lowercased, punctuation collapsed.
pub fn norm_title_for_match(title: &str) -> String {
    let t = strip_title_decorations(title).to_lowercase();
    let a = replace_all("[^a-z0-9]+", &t, " ");
    squash(&a)
}

/// Port of `fd_stream_keyword_variants()` (backend.php:9820).
///
/// The origin's `search_files` is a literal Manticore match, so an apostrophe in
/// a catalogue title makes the query miss files the uploaders named differently:
/// `"Grey's Anatomy"` is stored as `Greys Anatomy`, `Grey s Anatomy` or
/// `Grey Anatomy` depending on the uploader. Returns the keyword plus each
/// normalised form, in the PHP's order, de-duplicated and empty-free.
///
/// Used by the zero-result fallback ladder in the stream route
/// (`fd_fetch_episode_stream_files()` step 3, backend.php:10291).
///
/// The third form reproduces the PHP exactly, quirk included: the alternation is
/// `['’`]s|s['’`]|\bs\b`, scanned left to right, so a word ending in `s` before
/// the possessive (`"Princess's"`) matches the `s'` branch first and yields
/// `"Princes"` — while `"Grey's"` correctly yields `"Grey"`. Verified against
/// `bin/php.exe` for both, so do NOT "fix" it: a different form would issue a
/// query the PHP never issues.
pub fn stream_keyword_variants(keyword: &str) -> Vec<String> {
    let keyword = keyword.trim();
    if keyword.is_empty() {
        return Vec::new();
    }
    fn push(out: &mut Vec<String>, v: String) {
        if !v.is_empty() && !out.iter().any(|x| x == &v) {
            out.push(v);
        }
    }
    let mut out: Vec<String> = vec![keyword.to_string()];
    // 1. apostrophe -> space ("Princess's" -> "Princess s").
    push(&mut out, squash(&replace_all("['\u{2019}`]", keyword, " ")));
    // 2. apostrophe removed entirely ("Princess's" -> "Princesss").
    push(&mut out, squash(&replace_all("['\u{2019}`]", keyword, "")));
    // 3. possessive removed ("Princess's" -> "Princess", "Grey's" -> "Grey").
    push(
        &mut out,
        squash(&replace_all("(?:['\u{2019}`]s|s['\u{2019}`]|\\bs\\b)", keyword, "")),
    );
    out
}

/// `fd_title_words_prefix_match()`: only a whole-word PREFIX extension is the
/// same work, so `balan the boy` is NOT `the boy` while `khurafat perjanjian
/// syaitan` IS `khurafat`.
/// True when the client is a browser/WebView, i.e. one that cannot demux Matroska
/// inside `<video>`.
///
/// Chrome rejects `video/x-matroska`, so a browser handed an `.mkv` aborts on the
/// container and re-requests the same URL every few seconds (measured on a real
/// phone: `range: bytes=2848468527-` — the last ~178 KB of a 2.8 GB file — repeating
/// every 4-5 s while the server correctly answered 206). Native players
/// (ExoPlayer/okhttp, the Stremio desktop app, VLC) play MKV, so their order is
/// untouched. Used only to ORDER the list, never to filter it.
pub fn wants_web_container(user_agent: &str) -> bool {
    let ua = user_agent.trim().to_ascii_lowercase();
    if ua.is_empty() {
        return false;
    }
    for native in ["okhttp", "exoplayer", "stremio", "vlc", "mpv", "kodi"] {
        if ua.contains(native) {
            return false;
        }
    }
    ua.contains("mozilla/")
        || ua.contains("chrome/")
        || ua.contains("safari/")
        || ua.contains("firefox/")
        || ua.contains("edg/")
}

#[cfg(test)]
mod web_container_tests {
    use super::wants_web_container;

    #[test]
    fn browsers_need_a_web_container_and_native_players_do_not() {
        // Chrome / WebView (the phone that looped on an MKV).
        assert!(wants_web_container(
            "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36"
        ));
        // Native players: MKV is fine, keep the score order.
        assert!(!wants_web_container("okhttp/4.12.0"));
        assert!(!wants_web_container("ExoPlayerLib/2.19.1"));
        assert!(!wants_web_container("Stremio/1.6.9"));
        assert!(!wants_web_container("VLC/3.0.20 LibVLC/3.0.20"));
        assert!(!wants_web_container(""));
    }
}

pub fn title_words_prefix_match(a: &str, b: &str) -> bool {
    let an = norm_title_for_match(&fold_latin_accents(a));
    let bn = norm_title_for_match(&fold_latin_accents(b));
    if an.is_empty() || bn.is_empty() {
        return false;
    }
    if an == bn {
        return true;
    }
    let aw: Vec<&str> = an.split(' ').collect();
    let bw: Vec<&str> = bn.split(' ').collect();
    let (short, long) = if aw.len() <= bw.len() { (&aw, &bw) } else { (&bw, &aw) };
    long[..short.len()] == short[..]
}

/// Minimal HTML-entity decoder (`fd_clean_html_entities()`), enough for the
/// WordPress captions this matcher sees.
pub fn decode_html_entities(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(idx) = rest.find('&') {
        out.push_str(&rest[..idx]);
        let tail = &rest[idx..];
        let semi = match tail.find(';') {
            Some(i) if i <= 10 => i,
            _ => {
                out.push('&');
                rest = &tail[1..];
                continue;
            }
        };
        let entity = &tail[1..semi];
        let decoded: Option<char> = if let Some(num) = entity.strip_prefix('#') {
            if let Some(hex) = num.strip_prefix('x').or_else(|| num.strip_prefix('X')) {
                u32::from_str_radix(hex, 16).ok().and_then(char::from_u32)
            } else {
                num.parse::<u32>().ok().and_then(char::from_u32)
            }
        } else {
            match entity {
                "amp" => Some('&'),
                "lt" => Some('<'),
                "gt" => Some('>'),
                "quot" => Some('"'),
                "apos" => Some('\''),
                "nbsp" => Some(' '),
                "ndash" => Some('–'),
                "mdash" => Some('—'),
                "rsquo" => Some('\u{2019}'),
                "lsquo" => Some('\u{2018}'),
                "hellip" => Some('…'),
                _ => None,
            }
        };
        match decoded {
            Some(c) => {
                out.push(c);
                rest = &tail[semi + 1..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

/* ── fd_extract_media_tags ─────────────────────────────────────────────────── */

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaTags {
    pub resolution: String,
    pub source: String,
    pub platform: String,
    pub codec: String,
    pub audio: String,
    pub visual: Vec<String>,
    pub edition: String,
}

fn first_tag(text: &str, table: &[(&str, &str)], fallback: &str) -> String {
    for (pattern, value) in table {
        if has(pattern, text) {
            return (*value).to_string();
        }
    }
    fallback.to_string()
}

/// Port of `fd_extract_media_tags()` (backend.php:7926).
pub fn extract_media_tags(title: &str, caption: &str) -> MediaTags {
    let text = format!("{} {}", title, caption);
    let resolution = first_tag(
        &text,
        &[
            ("(?i)\\b(2160p|4k|uhd)\\b", "4K"),
            ("(?i)\\b(1080p|fhd)\\b", "1080p"),
            ("(?i)\\b(720p|hd)\\b", "720p"),
            ("(?i)\\b(540p)\\b", "540p"),
            ("(?i)\\b(480p|sd)\\b", "480p"),
            ("(?i)\\b(360p)\\b", "360p"),
        ],
        "",
    );
    let source = first_tag(
        &text,
        &[
            ("(?i)\\b(remux)\\b", "REMUX"),
            ("(?i)\\b(bluray|blu-ray|bdrip|bbrip|brrip)\\b", "BluRay"),
            // `WEB.DL` (every scene name separates tags with dots) and `WEBDL` are
            // the same source as `WEB-DL`. Without the dot/underscore forms the
            // `web` rule below caught them and the pill said WEBRip.
            ("(?i)\\bweb[._-]?dl\\b", "WEB-DL"),
            ("(?i)\\b(webrip|web)\\b", "WEBRip"),
            ("(?i)\\b(hdrip)\\b", "HDRip"),
            ("(?i)\\b(hdtv|tvrip|pdtv)\\b", "HDTV"),
            ("(?i)\\b(dvdrip|dvd)\\b", "DVDRip"),
            ("(?i)\\b(hdcam|camrip|cam|telesync|ts|tc)\\b", "CAM"),
        ],
        "",
    );
    let platform = first_tag(
        &text,
        &[
            ("(?i)\\b(nf|netflix)\\b", "NF"),
            ("(?i)\\b(amzn|primevideo|prime)\\b", "AMZN"),
            ("(?i)\\b(dsnp|disney\\+?|disney)\\b", "DSNP"),
            ("(?i)\\b(atvp|apple\\s*tv\\+?)\\b", "ATVP"),
            ("(?i)\\b(hmax|hbo\\s*max)\\b", "HMAX"),
            ("(?i)\\b(zee5)\\b", "ZEE5"),
            ("(?i)\\b(hotstar)\\b", "Hotstar"),
            ("(?i)\\b(viki)\\b", "Viki"),
            ("(?i)\\b(wetv)\\b", "WeTV"),
            ("(?i)\\b(iqiyi)\\b", "iQIYI"),
            ("(?i)\\b(starzplay)\\b", "StarzPlay"),
        ],
        "",
    );
    let codec = first_tag(
        &text,
        &[
            ("(?i)\\b(hevc|x265|h\\.?265)\\b", "HEVC"),
            ("(?i)\\b(avc|x264|h\\.?264)\\b", "H.264"),
            ("(?i)\\b(av1)\\b", "AV1"),
            ("(?i)\\b(xvid|divx)\\b", "XviD"),
        ],
        "",
    );
    let audio = first_tag(
        &text,
        &[
            ("(?i)\\b(atmos)\\b", "Atmos"),
            ("(?i)\\b(ddp\\s*5\\.1|dd\\+\\s*5\\.1|eac3\\s*5\\.1)\\b", "DDP5.1"),
            ("(?i)\\b(ddp\\s*2\\.0|dd\\+\\s*2\\.0|eac3\\s*2\\.0)\\b", "DDP2.0"),
            ("(?i)\\b(ddp|dd\\+|eac3)\\b", "DDP"),
            ("(?i)\\b(dd\\s*5\\.1|ac3\\s*5\\.1)\\b", "DD5.1"),
            ("(?i)\\b(ac3|dd)\\b", "AC3"),
            ("(?i)\\b(dts-hd\\s*ma)\\b", "DTS-HD MA"),
            ("(?i)\\b(dts-hd)\\b", "DTS-HD"),
            ("(?i)\\b(dts)\\b", "DTS"),
            ("(?i)\\b(truehd)\\b", "TrueHD"),
            ("(?i)\\b(aac\\s*5\\.1|5\\.1\\s*aac)\\b", "AAC5.1"),
            ("(?i)\\b(aac\\s*2\\.0|2\\.0\\s*aac|aac2)\\b", "AAC2.0"),
            ("(?i)\\b(aac)\\b", "AAC"),
            ("(?i)\\b(flac)\\b", "FLAC"),
            ("(?i)\\b(opus)\\b", "Opus"),
        ],
        "",
    );
    let mut visual = Vec::new();
    if has("(?i)\\b(hdr10\\+|hdr10|hdr)\\b", &text) {
        visual.push("HDR".to_string());
    }
    if has("(?i)\\b(dolby\\s*vision|dovi|dv)\\b", &text) {
        visual.push("DV".to_string());
    }
    if has("(?i)\\b(10bit|10-bit|hi10p?)\\b", &text) {
        visual.push("10bit".to_string());
    }
    if has("(?i)\\b(imax)\\b", &text) {
        visual.push("IMAX".to_string());
    }
    let edition = first_tag(
        &text,
        &[
            ("(?i)\\b(remastered)\\b", "Remastered"),
            ("(?i)\\b(extended)\\b", "Extended"),
            ("(?i)\\b(uncut)\\b", "Uncut"),
            // `UNRATED` is an edition, not noise: it is how a viewer tells the
            // theatrical cut from the unrated one. It was silently dropped, so
            // `Obsession.2026.2160p.iT.WEB.DL.UNRATED.DV.HDR10…` showed neither
            // the pill nor the distinction.
            ("(?i)\\b(unrated|unrated\\s+cut)\\b", "Unrated"),
            ("(?i)\\b(repack|proper)\\b", "Proper"),
        ],
        "",
    );
    MediaTags {
        resolution,
        source,
        platform,
        codec,
        audio,
        visual,
        edition,
    }
}

/* ── season / episode classification ───────────────────────────────────────── */

/// Port of `fd_is_series_file()` (backend.php:8521).
pub fn is_series_file(title: &str, caption: &str) -> bool {
    let mut text = clean_media_title(title);
    if !caption.is_empty() && has(GENERIC_NAME, text.trim()) {
        let first = caption.lines().next().unwrap_or("").trim();
        if !first.is_empty() {
            text = clean_media_title(first);
        }
    }
    if has(
        "(?i)(?:^|[^a-z0-9])S\\d{1,2}\\s*[ ._-]*(?:E(?:P|PS|PISODE)?\\s*[ ._-]*\\d{1,4})+",
        &text,
    ) {
        return true;
    }
    if has(
        "(?:^|[^a-z0-9])\\d{1,2}\\s*[xX]\\s*\\d{1,4}(?![0-9])(?:[^a-z0-9]|$)",
        &text,
    ) {
        return true;
    }
    if has(
        "(?i)(?:^|[^a-z0-9])(?:season|musim)\\s*[ ._-]*\\d{1,2}(?:[^a-z0-9]|$)",
        &text,
    ) {
        return true;
    }
    if let Some(g) = groups(
        "(?i)(?:^|[^a-z0-9])(?:EP|EPS|EPISODE|EPISOD|BAHAGIAN|BABAK)\\s*[ ._-]*0*(\\d{1,4})(?:[^a-z0-9]|$)",
        &text,
    ) {
        if let Ok(n) = g[1].parse::<i32>() {
            if n > 0 && !(1900..=2100).contains(&n) {
                return true;
            }
        }
    }
    if let Some(g) = groups(
        "(?i)(?:^|[^a-z0-9])E[ ._-]*0*(\\d{1,4})(?:[^a-z0-9]|$)",
        &text,
    ) {
        if let Ok(n) = g[1].parse::<i32>() {
            if n > 0 && !(1900..=2100).contains(&n) {
                return true;
            }
        }
    }
    has(
        "(?i)(?:^|[^a-z0-9])S(\\d{1,2})(?=[ ._\\[\\(-]+(?:2160p|1080p|720p|480p|360p|4k|uhd|fhd|hd|sd|web|bluray|hdtv|complete|batch|ongoing|x264|x265|hevc)|$)",
        &text,
    )
}

/// Port of `fd_classify_season_episode()` (backend.php:8354).
/// Returns `(season, episode, episode_end)`.
pub fn classify_season_episode(
    raw_title: &str,
    season_num: i32,
    episode_num: i32,
    caption: &str,
) -> (i32, i32, i32) {
    let mut season = 0i32;
    let mut episode = 0i32;
    let mut episode_end = 0i32;
    let mut title = clean_media_title(raw_title);
    if !caption.is_empty() && has(GENERIC_NAME, title.trim()) {
        let first = caption.replace('\r', "\n");
        let first = first.lines().next().unwrap_or("").trim().to_string();
        if !first.is_empty() {
            title = clean_media_title(&first);
        }
    }

    let num = |v: &str| v.parse::<i32>().ok();

    // 1. Explicit SxxExx forms.
    if let Some(g) = groups(
        "(?i)(?:^|[^a-z0-9])S(\\d{1,2})\\s*[ ._-]*E(?:P|PS|PISODE)?\\s*[ ._-]*(\\d{1,4})\\s*(?:[ ._-]+E(?:P|PS|PISODE)?|\\s*[-~–—]\\s*|\\s+(?:to|hingga|sampai)\\s+)\\s*(\\d{1,4})(?:[^a-z0-9]|$)",
        &title,
    ) {
        season = num(&g[1]).unwrap_or(0);
        episode = num(&g[2]).unwrap_or(0);
        episode_end = num(&g[3]).unwrap_or(0);
    } else if let Some(g) = groups(
        "(?i)(?:^|[^a-z0-9])S(\\d{1,2})\\s*[ ._-]*E(?:P|PS|PISODE)?\\s*[ ._-]*0*(\\d{1,4})(?=[^0-9]|$)",
        &title,
    ) {
        season = num(&g[1]).unwrap_or(0);
        episode = num(&g[2]).unwrap_or(0);
    } else if let Some(g) = groups(
        "(?i)(?:^|[^a-z0-9])S(\\d{1,2})\\.0*(\\d{1,4})(?=[^0-9]|$)",
        &title,
    ) {
        season = num(&g[1]).unwrap_or(0);
        episode = num(&g[2]).unwrap_or(0);
    } else if let Some(g) = groups(
        "(?:^|[^a-z0-9])(\\d{1,2})\\s*[xX]\\s*(\\d{1,4})(?![0-9])(?:[^a-z0-9]|$)",
        &title,
    ) {
        season = num(&g[1]).unwrap_or(0);
        episode = num(&g[2]).unwrap_or(0);
    } else if let Some(g) = groups(
        "(?i)(?:^|[^a-z0-9])(?:PART|VOL|VOLUME|COUR)\\s*[ ._-]*0*(\\d{1,2})[ ._-]+(?:E(?:P|PS|PISODE)?\\s*[ ._-]*)?0*(\\d{1,4})(?=[ ._\\-\\]\\)]|$)",
        &title,
    ) {
        let n2 = num(&g[2]).unwrap_or(0);
        if !(1900..=2100).contains(&n2) {
            season = num(&g[1]).unwrap_or(0);
            episode = n2;
        }
    }

    // 4. Season / Musim / Part keyword fallback.
    if season == 0 {
        let candidates = [
            "(?i)(?:^|[^a-z0-9])0*(\\d{1,2})(?:st|nd|rd|th)\\s*[ ._-]*(?:season|musim)(?:[^a-z0-9]|$)",
            "(?i)(?:^|[^a-z0-9])(?:season|musim)\\s*[ ._-]*0*(\\d{1,2})(?:[^a-z0-9]|$)",
            "(?i)(?:^|[^a-z0-9])0*(\\d{1,2})\\s*[ ._-]*(?:season|musim)(?:[^a-z0-9]|$)",
            "(?i)(?:^|[^a-z0-9])(?:PART|VOL|VOLUME|COUR)\\s*[ ._-]*0*(\\d{1,2})(?=[ ._-]+(?:EP|E|\\d))",
            "(?i)(?:^|[^a-z0-9])S(\\d{1,2})(?=[^a-z0-9]|$)",
        ];
        for pattern in candidates {
            if let Some(g) = groups(pattern, &title) {
                season = num(&g[1]).unwrap_or(0);
                break;
            }
        }
    }

    // 5. Episode range tokens.
    if episode == 0 {
        if let Some(g) = groups(
            "(?i)(?:^|[^a-z0-9])(?:EP|EPS|EPISODE|EPISOD|E)\\s*[ ._-]*0*(\\d{1,4})\\s*(?:[ ._-]+(?:EP|EPS|EPISODE|EPISOD|E)|\\s*[-~–—]\\s*|\\s+(?:to|hingga|sampai)\\s+)\\s*0*(\\d{1,4})(?:[^a-z0-9]|$)",
            &title,
        ) {
            let n1 = num(&g[1]).unwrap_or(0);
            let n2 = num(&g[2]).unwrap_or(0);
            let n1_ok = n1 > 0 && !(1900..=2100).contains(&n1);
            let n2_ok = n2 > 0 && !(1900..=2100).contains(&n2);
            if n1_ok && n2_ok && n2 >= n1 && (n2 - n1) <= 150 {
                episode = n1;
                episode_end = n2;
            }
        }
    }
    // 6. EP/Episode/Bahagian token.
    if episode == 0 {
        if let Some(g) = groups(
            "(?i)(?:^|[^a-z0-9])(?:EP|EPS|EPISODE|EPISOD|BAHAGIAN|BABAK)\\s*[ ._-]*0*(\\d{1,4})(?=[^0-9]|$)",
            &title,
        ) {
            let n = num(&g[1]).unwrap_or(0);
            if n > 0 && !(1900..=2100).contains(&n) {
                episode = n;
            }
        }
    }
    // 7. E<digits> token.
    if episode == 0 {
        if let Some(g) = groups(
            "(?i)(?:^|[^a-z0-9])E[ ._-]*0*(\\d{1,4})(?:[^a-z0-9]|$)",
            &title,
        ) {
            let n = num(&g[1]).unwrap_or(0);
            if n > 0 && !(1900..=2100).contains(&n) {
                episode = n;
            }
        }
    }
    // 7b. Part/Vol episode fallback (only when the season was not taken from it).
    if episode == 0 && season == 0 {
        if let Some(g) = groups(
            "(?i)(?:^|[^a-z0-9])(?:PART|VOL|VOLUME)\\s*[ ._-]*0*(\\d{1,4})(?:[^a-z0-9]|$)",
            &title,
        ) {
            let n = num(&g[1]).unwrap_or(0);
            if n > 0 && !(1900..=2100).contains(&n) {
                episode = n;
            }
        }
    }
    // 8. Bare episode numbers.
    if episode == 0 {
        let mut clean = replace_all(
            "(?i)\\.(mp4|mkv|avi|mov|ts|flv|webm)$",
            &title,
            "",
        );
        clean = replace_all("\\b(?:19|20)\\d{2}\\b", &clean, " ");
        let hit = groups(
            "(?i)(?<![0-9])[ ._\\[\\(-](\\d{1,3})[ ._\\]\\)-]+(?:2160p|1080p|720p|540p|480p|360p|4k|uhd|fhd|hd|sd|web|bluray|hdtv|malaysub|end|final|x264|x265|hevc|aac|anitv|ani|tv|raw|[a-z0-9_-]+|\\d{3,4}x\\d{3,4})",
            &clean,
        )
        .or_else(|| groups("(?<![0-9])[ ._\\[\\(-](\\d{1,3})[ ._\\]\\)]*$", &clean));
        if let Some(g) = hit {
            let n = num(&g[1]).unwrap_or(0);
            if n > 0 && !(1900..=2100).contains(&n) {
                episode = n;
            }
        }
    }
    // Fallbacks to the DB media-rank columns.
    if season == 0 && season_num > 0 && (episode == 0 || episode_num == episode) {
        season = season_num;
    }
    if episode == 0 && episode_num > 0 {
        episode = episode_num;
    }
    if season == 0 {
        season = 1;
    }
    (season, episode, episode_end)
}

/// Port of `fd_file_matches_episode()` (backend.php:8498).
pub fn file_matches_episode(parsed: (i32, i32, i32), target_season: i32, target_episode: i32) -> bool {
    let (season, episode, episode_end) = parsed;
    if episode == 0 {
        return false;
    }
    if season != target_season {
        return false;
    }
    if episode_end > episode && target_episode >= episode && target_episode <= episode_end {
        return true;
    }
    episode == target_episode
}

/* ── verified episode names (PR #5 parity) ─────────────────────────────────── */

/// Port of `fd_normalize_episode_name()`. The input is a TRUSTED episode title,
/// never a file name.
pub fn normalize_episode_name(name: &str) -> String {
    let decoded = decode_html_entities(name);
    let mut n = replace_all("[^\\p{L}\\p{N}]+", &decoded, " ")
        .trim()
        .to_lowercase();
    n = replace_all(
        "(?i)^(?:(?:season\\s*\\d+\\s*)?episode\\s*\\d+|s\\d{1,2}\\s*e\\d{1,4}|ep\\s*\\d+|\\d{1,3})\\s+",
        &n,
        "",
    );
    squash(&n)
}

/// Port of `fd_filename_episode_subtitle()`.
///
/// The region AFTER an explicit episode marker and BEFORE the first technical
/// release tag. Empty means "no episode subtitle to verify" (fail open).
pub fn filename_episode_subtitle(file_title: &str, series_title: &str) -> String {
    let cleaned = clean_media_title(file_title);
    let mut text = replace_all("[^\\p{L}\\p{N}]+", &cleaned, " ").to_lowercase();
    text = squash(&text);
    let mut title_norm = normalize_episode_name(series_title);
    title_norm = replace_all("\\s+(?:19|20)\\d{2}$", &title_norm, "");

    let marker = span(
        "(?i)(?:^|\\s)(?:s\\d{1,2}\\s*e(?:p|ps|pisode)?\\s*\\d{1,4}|\\d{1,2}\\s*x\\s*\\d{1,4}|e(?:p|ps|pisode)?\\s*\\d{1,4})(?=\\s|$)",
        &text,
    );

    let after_marker: usize = match marker {
        Some((start, end)) => {
            // Some Asian releases put the SHOW title AFTER E01 (CDL.E01.Show.Name).
            // Only interpret the tail as an episode subtitle when the show title
            // already appears BEFORE the marker.
            if !title_norm.is_empty() {
                let prefix = text.get(..start).unwrap_or("").trim();
                let hay = format!(" {} ", prefix);
                let needle = format!(" {} ", title_norm);
                if !hay.contains(&needle) {
                    return String::new();
                }
            }
            end
        }
        None => {
            // Bare numeric episodes are valid (Show.01.Pilot.mkv) but only when an
            // exact known SHOW title precedes the number, with an optional year.
            if title_norm.is_empty() {
                return String::new();
            }
            let pattern = format!(
                "(?i)(?:^|\\s){}\\s+(?:(?:19|20)\\d{{2}}\\s+)?0*[1-9]\\d{{0,2}}(?=\\s|$)",
                escape(&title_norm)
            );
            match span(&pattern, &text) {
                Some((_, end)) => end,
                None => return String::new(),
            }
        }
    };

    let mut tail = text.get(after_marker..).unwrap_or("").trim().to_string();
    // Combined episode pack: never mistake E02/E03 or a year for a subtitle.
    tail = replace_all(
        "(?i)^(?:(?:e(?:p|ps|pisode)?\\s*\\d{1,4})|(?:19|20)\\d{2})\\s+",
        &tail,
        "",
    );
    // A resolution / release token ends the subtitle region. Matched GENERICALLY:
    // the enumerated 2160p/1080p/... list missed `450p`, so
    // `Wild.Romance.E01.450p.HANerl.re.encoded.to.MQ.avi` kept a "subtitle" and
    // the episode-name guard rejected the whole legitimate release.
    if let Some((start, _)) = span(
        "(?i)\\b(?:\\d{3,4}[pi]|4k|uhd|fhd|hdtv|web|webrip|bluray|bdrip|brrip|remux|dvdrip|h264|h265|hevc|x264|x265|xvid|avc|aac|dts|flac|ddp|atmos|hdr|dv|nf|amzn|dsnp|multi|dual|audio|dub|dubbed|sub|subs|subbed|engsub|indosub|malaysub|hindi|eng|english|indo|indonesian|malay|korean|japanese|spanish|french|arabic|thai|batch|complete|mkv|mp4|avi|mov|ts|flv)\\b",
        &tail,
    ) {
        tail.truncate(start);
    }
    normalize_episode_name(&squash(&tail))
}

/// Port of `fd_is_release_annotation()`: release-group labels are not titles.
pub fn is_release_annotation(subtitle: &str) -> bool {
    has(
        "(?i)^(?:proper|repack|rerip|extended|uncut|uncensored|remastered|restored|internal|limited|multi|dual|dubbed|subbed|complete|batch)(?:\\s|$)",
        subtitle,
    )
}

/// Port of `fd_episode_subtitle_matches()`.
///
/// Fail open on anything inconclusive; reject only a definite descriptive
/// mismatch (another show's episode title). A ONE-WORD tail is never evidence —
/// it is the uploader/group between the marker and the quality tag.
pub fn episode_subtitle_matches(file_title: &str, series_title: &str, episode_names: &[String]) -> bool {
    episode_subtitle_matches_with_siblings(file_title, series_title, episode_names, &[])
}

/// [`episode_subtitle_matches()`] plus the OTHER episodes' titles.
///
/// The rule is deliberately asymmetric, and this is what makes it safe:
///
///   * the post-marker text matches THIS episode's title -> accept;
///   * it matches a DIFFERENT episode of the same season -> REJECT (that is the
///     pinned "a file that names another show's episode" case);
///   * it matches nothing at all -> **fail open**. A file named in the
///     uploader's own ordering is still this episode: Disney's "Oswald" is
///     numbered 1..52 by theTVDB while the uploader ships two segments per file,
///     so `S01E02 Sleepover & Big Banana` is E2 even though theTVDB calls E2
///     "The Camping Trip". Rejecting on "not the title I expected" dropped all
///     50 Oswald files and answered "No Streams Found" for every episode.
pub fn episode_subtitle_matches_with_siblings(
    file_title: &str,
    series_title: &str,
    episode_names: &[String],
    sibling_names: &[String],
) -> bool {
    if episode_names.is_empty() && sibling_names.is_empty() {
        return true;
    }
    let subtitle = filename_episode_subtitle(file_title, series_title);
    if subtitle.is_empty() || is_release_annotation(&subtitle) {
        return true;
    }
    if !subtitle.contains(' ') {
        return true;
    }
    // THIS episode's title, possibly followed by more text: an own match is
    // enough to keep the file.
    let hits_own = |list: &[String]| {
        list.iter().any(|raw| {
            let expected = normalize_episode_name(raw);
            !expected.is_empty()
                && (subtitle == expected || subtitle.starts_with(&format!("{} ", expected)))
        })
    };
    // A sibling's title must match the WHOLE subtitle. A prefix is not enough:
    // theTVDB files Oswald's "Sleepover" as E13 and "Big Banana" as E14 while the
    // uploader's E2 is the pair, so `S01E02 Sleepover & Big Banana` merely
    // STARTS WITH a sibling title and is still the right file for its episode.
    let hits_sibling = |list: &[String]| {
        list.iter().any(|raw| {
            let expected = normalize_episode_name(raw);
            !expected.is_empty() && subtitle == expected
        })
    };
    if hits_own(episode_names) {
        return true;
    }
    if hits_sibling(sibling_names) {
        // A KNOWN other episode's title, in full: this file is that episode.
        return false;
    }
    // Nothing we know: no evidence, so it cannot hide a stream.
    true
}

/* ── fd_series_file_matches_title ──────────────────────────────────────────── */

const STOP_WORDS: [&str; 12] = ["the", "a", "an", "of", "in", "on", "at", "to", "for", "and", "dan", "the"];

/// Port of `fd_series_file_matches_title()` (backend.php:9220).
#[allow(clippy::too_many_arguments)]
pub fn series_file_matches_title(
    file_title: &str,
    series_title: &str,
    season: i32,
    episode: i32,
    series_year: &str,
    file_caption: &str,
    episode_names: &[String],
) -> bool {
    series_file_matches_title_with_siblings(
        file_title, series_title, season, episode, series_year, file_caption, episode_names, &[],
    )
}

/// [`series_file_matches_title()`] plus the sibling episode titles — see
/// [`episode_subtitle_matches_with_siblings()`] for why the extra evidence makes
/// the guard stricter instead of looser.
#[allow(clippy::too_many_arguments)]
pub fn series_file_matches_title_with_siblings(
    file_title: &str,
    series_title: &str,
    season: i32,
    episode: i32,
    series_year: &str,
    file_caption: &str,
    episode_names: &[String],
    sibling_names: &[String],
) -> bool {
    let parsed = classify_season_episode(file_title, season, episode, file_caption);
    if !file_matches_episode(parsed, season, episode) {
        return false;
    }

    let st = series_title.trim();
    if st.is_empty() {
        return true;
    }

    let mut clean_series = replace_all("\\s*[•··]\\s*.+$", &decode_html_entities(st), "");
    clean_series = replace_all(
        "(?i)\\b(?:tvseries|series|season|musim|drama|episode|bahagian)\\b.*",
        &clean_series,
        "",
    );
    let sy = series_year.trim();
    // Strip only the TRAILING release year. NEVER the first occurrence anywhere:
    // "Kudrat 1968: High Council 2026" must keep its embedded 1968, otherwise the
    // remaining phrase appears in no file name and every episode is rejected.
    clean_series = replace_all("\\s*\\b(?:19\\d{2}|20\\d{2})\\b\\s*$", &clean_series, " ");
    clean_series = norm_words(&clean_series);
    let mut target_lower = clean_series;
    if target_lower.is_empty() {
        target_lower = norm_words(st);
    }
    let target_no_the = target_lower.strip_prefix("the ").unwrap_or(&target_lower).trim().to_string();

    let content_words: Vec<&str> = target_lower
        .split(' ')
        .filter(|w| !w.is_empty() && !STOP_WORDS.contains(w))
        .collect();
    let is_single_word = content_words.len() <= 1;

    // The CAPTION often holds the full release name, and with it the year the
    // file name omits (`One.Piece.2023.Season.1.S01.E01...mkv`). Consulted only
    // when the NAME carries no year, so it can never override a real one.
    let mut year_source = file_title.to_string();
    if !file_caption.is_empty() && !has("\\b(19\\d{2}|20\\d{2})\\b", file_title) {
        let cap_first = file_caption.lines().next().unwrap_or("").trim();
        if !cap_first.is_empty() {
            year_source = format!("{} {}", file_title, cap_first);
        }
    }
    let file_year: i32 = match groups("\\b(19\\d{2}|20\\d{2})\\b", &year_source) {
        Some(g) => g[1].parse().unwrap_or(0),
        None => 0,
    };
    let has_file_year = file_year != 0;
    let target_year: i32 = sy.parse().unwrap_or(0);

    if !sy.is_empty() && has_file_year {
        // `fym` is the FIRST 4-digit number of the file name: for a show whose
        // NAME embeds a year that is the name year, not a release year, and the
        // series title holds BOTH years — so a file year that appears in the
        // series title is accepted. Unrelated years are still rejected, which is
        // what keeps "Glory 2025" away from The Glory's 2022 files.
        let series_years: HashSet<i32> = rx("\\b(?:19\\d{2}|20\\d{2})\\b")
            .find_iter(st)
            .filter_map(|m| m.ok())
            .filter_map(|m| m.as_str().parse::<i32>().ok())
            .collect();
        let year_ok = if is_single_word {
            file_year == target_year
        } else {
            (file_year - target_year).abs() <= 1
        };
        if !year_ok && !series_years.contains(&file_year) {
            return false;
        }
    }

    let mut f_clean = replace_all("(?i)\\.(mp4|mkv|avi|mov|ts|flv|webm)$", file_title, "");
    f_clean = clean_media_title(&f_clean);
    f_clean = replace_all(
        "(?i)^(?:prakytv|ngefilm\\s*store|runningmovieshd|kannadachallengers|mkvcinemas|vegamovies|moviesmod|pahe|galaxy|wetv|studioghibli|animerg|pms|pahe\\.li|layarkaca\\d*|cinemaindo|indoxxi|kdg|fanszz|dramaost|mkvdrama|dusklight|primefix|nodrakorid|nodrakor|layarasia|old|oppa\\.biz|drakor\\s*id|nunadrama|queendramainfo|adrama(?:\\s*lovers)?)\\s+",
        &f_clean,
        "",
    );
    let f_clean_norm = norm_words(&f_clean);

    const QUALITY_PATTERN: &str = "(?i)\\b(?:2160p|1080p|720p|540p|480p|360p|4k|uhd|fhd|hd|sd|bluray|blu-ray|bdrip|brrip|web-?dl|webrip|hdrip|hdtv|cam|ts|dvdrip|remux|hevc|x264|x265|h264|h265|aac.*|lubokvideo|yts|lulustream|galaxyrg|pahe|fanszz|malaysub|sub|engsub|indosub|kdf)\\b.*";
    const EP_PATTERN: &str = "(?i)(?:s\\d{1,2}\\s*[ ._-]*e(?:p|ps|pisode)?\\s*\\d{1,4}|(?<![a-z0-9])e(?:p|ps|pisode)?\\s*\\d{1,4}|\\b\\d{1,2}x\\d{1,4}\\b|season\\s*\\d+|musim\\s*\\d+|bahagian\\s*\\d+|part\\s*\\d+)";

    let mut base_f = replace_all(
        "(?i)(?:s\\d{1,2}\\s*[ ._-]*e(?:p|ps|pisode)?\\s*\\d{1,4}|(?<![a-z0-9])e(?:p|ps|pisode)?\\s*\\d{1,4}|\\b\\d{1,2}x\\d{1,4}\\b|season\\s*\\d+|musim\\s*\\d+|bahagian\\s*\\d+|part\\s*\\d+).*",
        &f_clean_norm,
        "",
    );
    base_f = replace_all(QUALITY_PATTERN, &base_f, "");
    base_f = replace_all("\\b(?:19\\d{2}|20\\d{2})\\b.*", &base_f, "");
    base_f = squash(&base_f);

    let mut after_ep = String::new();
    if let Some((_, end)) = span(EP_PATTERN, &f_clean_norm) {
        let after = f_clean_norm.get(end..).unwrap_or("");
        let after = replace_all(QUALITY_PATTERN, after, "");
        let after = replace_all("\\b(?:19\\d{2}|20\\d{2})\\b.*", &after, "");
        after_ep = squash(&after);
    }

    const UPLOADER_SUFFIX: &str = "(?i)^(?:(?:sklink|office|movie|movies|king|film|films|cinema|cinemas|media|channel|tv|tele|tg|hub|world|zone|group|team|club|gang|bot|vip|official|download|downloads|upload|uploads|site|store|station|blasters|rockers|wap|villa|dub|yogi|verse|flix|pahe|galaxy|pms|mkv|link|links|hd|nunadrama|drakor|drama|adrama|lovers|queendramainfo)\\b\\s*)+$";

    // ── the post-episode text is the EPISODE-TITLE slot ─────────────────────
    //
    // `Im.Not.Afraid.S01E01.The.Witch.1080p.NF.WEB.DL…` is **I'm Not Afraid**,
    // whose S01E01 happens to be *titled* "The Witch". Taking `after_ep` as a
    // candidate title let that file join "The Witch 2025" — three foreign
    // episodes in one list. The owning show is whatever precedes the marker;
    // `after_ep` may only stand in for the title when there IS no real phrase
    // before the marker, i.e. the prefix is a channel/uploader tag
    // (`CDL.E01.The.Rebel.Princess`, `DramaOST.E01.The.Witch`, `E01.The.Witch`)
    // — the exact shape Pattern B exists for.
    let after_ep_is_title_slot = {
        let words = base_f.split_whitespace().count();
        words <= 1 || (!base_f.is_empty() && has(UPLOADER_SUFFIX, &base_f))
    };
    let after_ep_title: &str = if after_ep_is_title_slot { after_ep.as_str() } else { "" };

    // Strict leading-"The" check. NOTE it is also the discriminator that the
    // single-word foreign-indicator guard below relies on.
    let has_leading_the = target_lower.starts_with("the ");
    let file_has_leading_the = base_f.starts_with("the ") || after_ep_title.starts_with("the ");
    if has_leading_the != file_has_leading_the {
        return false;
    }

    // Single-word titles with a known year and no leading "The" (e.g. "Glory
    // 2025"): reject un-yeared files carrying foreign platform/region
    // indicators — they belong to a DIFFERENT, same-named show. `hindi` is
    // deliberately not an indicator (Hindi+Korean dual-audio is normal naming).
    if !has_file_year && !sy.is_empty() && is_single_word && !has_leading_the {
        let region = format!("{} {}", file_title, file_caption);
        if has("(?i)\\b(netflix|\\bnf\\b|k\\.o|korean|korea|rarbg|kdg)\\b", &region) {
            return false;
        }
    }

    // 5c. Kind markers: a marker the ENTRY title does not carry means the file
    // is a different work (One Piece anime vs LEGO / MOVIE / OVA / LOG /
    // LIVE ACTION). The entry's own title is the exemption.
    let entry_norm = norm_words(st);
    if !entry_norm.is_empty() && (!f_clean_norm.is_empty() || !after_ep.is_empty()) {
        let file_region = format!("{} {}", f_clean_norm, after_ep).trim().to_string();
        for kind in ["movie", "film", "ova", "oav", "oad", "ona", "special", "recap", "log", "lego"] {
            let pattern = format!("(?i)\\b{}\\b", kind);
            if has(&pattern, &file_region) && !has(&pattern, &entry_norm) {
                return false;
            }
        }
        if has("(?i)\\btv\\s*sp\\b|live[ ._-]?action", &file_region)
            && !has("(?i)live[ ._-]?action", &entry_norm)
        {
            return false;
        }
    }

    // 5d. Generic episode-name guard. Fail open when the metadata is unknown.
    if !episode_subtitle_matches_with_siblings(file_title, &target_lower, episode_names, sibling_names) {
        return false;
    }

    let mut candidates: Vec<String> = Vec::new();
    if !base_f.is_empty() {
        candidates.push(base_f.clone());
    }
    if !after_ep_title.is_empty() {
        candidates.push(after_ep_title.to_string());
    }

    for tgt in [target_lower.as_str(), target_no_the.as_str()] {
        if tgt.is_empty() {
            continue;
        }
        if let Some(prefix) = base_f.strip_suffix(&format!(" {}", tgt)) {
            let prefix = prefix.trim();
            if !prefix.is_empty() && has(UPLOADER_SUFFIX, prefix) {
                candidates.push(tgt.to_string());
            }
        }
        if let Some(prefix) = after_ep_title.strip_suffix(&format!(" {}", tgt)) {
            let prefix = prefix.trim();
            if !prefix.is_empty() && has(UPLOADER_SUFFIX, prefix) {
                candidates.push(tgt.to_string());
            }
        }
    }

    let target_variants: Vec<String> = vec![
        target_lower.clone(),
        target_no_the.clone(),
        target_lower.replace(" and ", " dan "),
        target_lower.replace(" dan ", " and "),
        target_lower.replace(" and ", " "),
        target_lower.replace(" dan ", " "),
        target_lower.trim_end_matches('s').to_string(),
    ];

    for c in &candidates {
        if target_variants.contains(c) {
            return true;
        }
        if !is_single_word {
            let starts = c.starts_with(&format!("{} ", target_lower))
                || (!target_no_the.is_empty() && c.starts_with(&format!("{} ", target_no_the)));
            if starts {
                let suffix = if c.starts_with(&target_lower) {
                    c.get(target_lower.len()..).unwrap_or("").trim()
                } else {
                    c.get(target_no_the.len()..).unwrap_or("").trim()
                };
                if has(
                    "(?i)^(?:part|vol|volume|chapter|season|musim|bahagian)\\s*\\d+$",
                    suffix,
                ) {
                    return true;
                }
            }
        }
    }

    if content_words.len() >= 2 {
        // Searched in the TITLE region only, never the whole file name: a
        // foreign show's episode title sitting after `SxxExx` is not this
        // entry's name ("Other.Show.S01E01.My.Demon" is not "My Demon").
        //
        // When the post-marker text is not a title slot, the region is the file
        // name UP TO the marker — still the full pre-marker text, because
        // `base_f` is truncated at the first 4-digit number and would lose a
        // title with a year EMBEDDED in it ("Kudrat.1968.High.Council.2026.EP1"
        // → base_f "kudrat").
        let title_region = if after_ep_is_title_slot {
            f_clean_norm.clone()
        } else {
            match span(EP_PATTERN, &f_clean_norm) {
                Some((_, end)) => f_clean_norm.get(..end).unwrap_or("").to_string(),
                None => f_clean_norm.clone(),
            }
        };
        let q = format!("\\b{}\\b", escape(&target_lower));
        let q_no_the = if target_no_the.is_empty() {
            None
        } else {
            Some(format!("\\b{}\\b", escape(&target_no_the)))
        };
        if has(&q, &title_region) || q_no_the.as_deref().is_some_and(|p| has(p, &title_region)) {
            return true;
        }

        // 3+ content words: also match the core TAIL 2-word phrase against the
        // extracted candidates. PHP has this clause and the app did not, so a
        // long-titled series whose files carry only the tail answered
        // "No Streams Found". PHP backend.php:8206-8215.
        if content_words.len() >= 3 {
            let tail2 = content_words[content_words.len() - 2..].join(" ");
            let tail2_no_the = tail2.strip_prefix("the ").unwrap_or(&tail2).trim().to_string();
            if candidates.iter().any(|c| {
                *c == tail2 || (!tail2_no_the.is_empty() && *c == tail2_no_the)
            }) {
                return true;
            }
        }
    }

    false
}

/* ── fd_movie_file_matches_title ───────────────────────────────────────────── */

/// How a file's year evidence compares with the year of the catalog item.
///
/// The PHP's ±1 window (backend.php:8923) exists because a rip often carries a
/// festival or pre-release year. That makes it a *rescue* for an imprecise year,
/// and [`year_keep_mask()`] treats it as one: `Near` files are dropped as soon
/// as the title has an `Exact` one, so a same-name film released a year apart
/// cannot ride along ("Dhoom Dhaam" 2024 next to "Dhoom Dhaam" 2025).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YearTier {
    /// No year in the file name and no year in the caption — nothing to compare.
    NoEvidence,
    /// A year in the file NAME equals the searched year.
    Exact,
    /// A year in the file NAME is off by one — the PHP's rescue window.
    Near,
    /// Contradictory evidence: the file-name year is more than one year away, or
    /// the ONLY year is in the CAPTION and it is not exact. A caption is written
    /// metadata, not a guess the way a file name is, so it gets no ±1 window.
    Conflict,
}

fn years_in_text(text: &str) -> Vec<i32> {
    rx("\\b(19\\d{2}|20\\d{2})\\b")
        .find_iter(text)
        .filter_map(|m| m.ok())
        .filter_map(|m| m.as_str().parse::<i32>().ok())
        .collect()
}

/// Classify one file against the searched year (movie branch only — the series
/// branch keeps its own rules in [`series_file_matches_title()`]).
///
/// Mirrors the PHP's year guard in `fd_movie_file_matches_title()`
/// (backend.php:8910-8931) with one deliberate tightening: when the file name
/// carries no year the caption is the only evidence, and there it must be
/// exact.
pub fn movie_year_tier(file_title: &str, file_caption: &str, searched_year: &str) -> YearTier {
    let sy = searched_year.trim();
    if sy.is_empty() {
        return YearTier::NoEvidence;
    }
    // An unparseable year used to fall back to 0, which rejected every dated
    // file. Keep that: a garbage year must not become a licence to widen.
    let ty: i32 = match sy.parse::<i32>() {
        Ok(v) => v,
        Err(_) => return YearTier::Conflict,
    };
    let name_years = years_in_text(file_title);
    if !name_years.is_empty() {
        if name_years.contains(&ty) {
            return YearTier::Exact;
        }
        return if name_years.iter().any(|y| (*y - ty).abs() <= 1) {
            YearTier::Near
        } else {
            YearTier::Conflict
        };
    }
    let cap_first = file_caption.lines().next().unwrap_or("").trim();
    let cap_years = years_in_text(cap_first);
    if cap_years.is_empty() {
        return YearTier::NoEvidence;
    }
    if cap_years.contains(&ty) {
        YearTier::Exact
    } else {
        YearTier::Conflict
    }
}

/// Which candidates survive the year rule, given every candidate's tier.
///
/// `Conflict` never survives. `Near` survives only when no `Exact` candidate
/// exists — that is the whole point of the rescue window: it recovers a title
/// whose only files carry an imprecise year, and it must never widen an exact
/// match to also pull in the neighbouring year's release.
#[must_use]
pub fn year_keep_mask(tiers: &[YearTier]) -> Vec<bool> {
    let has_exact = tiers.contains(&YearTier::Exact);
    tiers
        .iter()
        .map(|t| *t != YearTier::Conflict && !(has_exact && *t == YearTier::Near))
        .collect()
}

/// Port of `fd_movie_file_matches_title()` (backend.php:8896).
///
/// Rule order is load-bearing: the year guard runs FIRST, then the strict
/// equalities, and the widening rules (type-word suffix, uploader prefix, AKA
/// prefix, accent-folded forms) come last so they can only ADD matches.
pub fn movie_file_matches_title(
    file_title: &str,
    searched_title: &str,
    searched_year: &str,
    file_caption: &str,
    searched_aka: &str,
) -> bool {
    if is_series_file(file_title, file_caption) {
        return false;
    }
    let st = searched_title.trim();
    if st.is_empty() {
        return true;
    }

    // 1. Strict year check. [`movie_year_tier()`] owns the classification so the
    // caller can apply the exact-before-±1 policy over the whole file list; here
    // only a contradiction rejects outright.
    let year_tier = movie_year_tier(file_title, file_caption, searched_year);
    if year_tier == YearTier::Conflict {
        return false;
    }
    let sy = searched_year.trim();
    // Any year evidence at all: in the file name, or in the caption when the file
    // name carries none. Section 9's sequence rule requires it.
    let has_file_year = year_tier != YearTier::NoEvidence;

    // 2. Normalised target title.
    let mut clean_target = replace_all("\\s*[•··]\\s*.+$", &decode_html_entities(st), "");
    clean_target = replace_all("\\s*&\\s*", &clean_target, " and ");
    clean_target = replace_all(TARGET_TAGS, &clean_target, " ");
    if !sy.is_empty() {
        clean_target = replace_all(&format!("\\b{}\\b", escape(sy)), &clean_target, " ");
    }
    let no_apos_target = norm_words(&replace_all("['’`]", &clean_target, ""));
    let clean_target = squash(&replace_all("[^\\p{L}\\p{N}\\s]", &clean_target, " "));
    let mut target_lower = clean_target.to_lowercase();
    if target_lower.is_empty() {
        target_lower = norm_words(st);
    }

    // 2a. Alternate-title (AKA) target, when the caller supplied one.
    let mut aka_lower = String::new();
    let mut no_apos_aka_lower = String::new();
    if !searched_aka.is_empty() {
        let mut clean_aka = replace_all("\\s*[•··]\\s*.+$", &decode_html_entities(searched_aka), "");
        clean_aka = replace_all("\\s*&\\s*", &clean_aka, " and ");
        clean_aka = replace_all(TARGET_TAGS, &clean_aka, " ");
        if !sy.is_empty() {
            clean_aka = replace_all(&format!("\\b{}\\b", escape(sy)), &clean_aka, " ");
        }
        no_apos_aka_lower = norm_words(&replace_all("['’`]", &clean_aka, ""));
        clean_aka = squash(&replace_all("[^\\p{L}\\p{N}\\s]", &clean_aka, " "));
        aka_lower = clean_aka.to_lowercase();
    }

    // 2b. Caption release-title check.
    if !file_caption.is_empty() {
        let caption = file_caption.replace('\r', "\n");
        let first_line = caption.lines().next().unwrap_or("").trim().to_string();
        if has("(?i)\\.(?:mp4|mkv|avi|mov|ts|flv|wmv)\\b", &first_line)
            || has("\\b(19\\d{2}|20\\d{2})\\b", &first_line)
        {
            let cap_clean = clean_media_title(&first_line);
            let mut cap_clean_norm = norm_words(&cap_clean);
            cap_clean_norm = replace_all(
                "(?i)[._\\s-]part[._\\s-]*0*\\d{1,4}",
                &cap_clean_norm,
                "",
            );
            let mut base_cap = replace_all(RELEASE_TAGS, &cap_clean_norm, "");
            if !sy.is_empty() {
                base_cap = replace_all(&format!("\\b{}\\b.*", escape(sy)), &base_cap, "");
            } else {
                base_cap = replace_all("\\b(?:19\\d{2}|20\\d{2})\\b.*", &base_cap, "");
            }
            base_cap = replace_all(EDITION_TAGS, &base_cap, "");
            base_cap = squash(&base_cap);
            let matches_title = base_cap == target_lower
                || (!no_apos_target.is_empty() && base_cap == no_apos_target)
                || (!aka_lower.is_empty() && base_cap == aka_lower)
                || (!no_apos_aka_lower.is_empty() && base_cap == no_apos_aka_lower);
            if !base_cap.is_empty() && matches_title {
                if sy.is_empty() {
                    return true;
                }
                match span("\\b(19\\d{2}|20\\d{2})\\b", &first_line) {
                    None => return true,
                    Some(_) => {
                        if let Some(g) = groups("\\b(19\\d{2}|20\\d{2})\\b", &first_line) {
                            let cy: i32 = g[1].parse().unwrap_or(0);
                            let ty: i32 = sy.parse().unwrap_or(0);
                            if (cy - ty).abs() <= 1 {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Clean filename, strip leading channel/group tags and known groups.
    let mut f_clean = clean_media_title(file_title);
    f_clean = replace_all(
        "(?i)^(?:@\\w+[._\\s]+|\\[[^\\]]+\\][._\\s]*|\\([^\\)]+\\)[._\\s]*)",
        &f_clean,
        "",
    );
    f_clean = replace_all(LEADING_GROUPS, &f_clean, "");
    let f_clean_norm = norm_words(&f_clean);

    // 4/5. Strip split parts, release tags, year and editions.
    let mut f_without_split = replace_all(
        "(?i)[._\\s-]part[._\\s-]*0*\\d{1,4}",
        &f_clean_norm,
        "",
    );
    f_without_split = replace_all(
        "(?i)\\.(?:mp4|mkv|avi|mov|ts|flv|wmv)\\.0*\\d{1,4}$",
        &f_without_split,
        "",
    );
    let mut base_f = replace_all(RELEASE_TAGS, &f_without_split, "");
    if !sy.is_empty() {
        base_f = replace_all(&format!("\\b{}\\b.*", escape(sy)), &base_f, "");
    }
    if !has("\\b(19\\d{2}|20\\d{2})\\b", &target_lower) {
        base_f = replace_all("\\b(?:19\\d{2}|20\\d{2})\\b.*", &base_f, "");
    }
    base_f = replace_all(EDITION_TAGS, &base_f, "");
    base_f = replace_all(
        "\\b(?:mp4|mkv|avi|mov|ts|flv|wmv)\\b",
        &base_f,
        "",
    );
    base_f = squash(&base_f);

    // Strip a leading 2-4 letter country code (never "the").
    let country_pat = format!("{}(?={})", COUNTRY_PREFIX, escape(&target_lower));
    base_f = replace_all(&country_pat, &base_f, "");
    if !aka_lower.is_empty() {
        let aka_pat = format!("{}(?={})", COUNTRY_PREFIX, escape(&aka_lower));
        base_f = replace_all(&aka_pat, &base_f, "");
    }
    base_f = squash(&base_f);

    // 6. Sequel guard.
    if !has(SEQUEL_RE, &target_lower) && has(SEQUEL_RE, &base_f) {
        return false;
    }

    let collapse_s = |s: &str| replace_all("(?i)\\b(\\w+)\\s+s\\b", s, "$1s");
    let base_f_collapsed = collapse_s(&base_f);
    // Accent-insensitive comparison forms, computed from the ALREADY-NORMALISED
    // strings. Folding the raw input would rewrite "•" and collapse matches.
    let target_fold_lower = squash(&fold_latin_accents(&target_lower));
    let base_fold = squash(&fold_latin_accents(&base_f));

    // 7. Direct comparison.
    if !base_f.is_empty() {
        if base_f == target_lower
            || (!no_apos_target.is_empty() && base_f == no_apos_target)
            || base_f_collapsed == target_lower
            || (!no_apos_target.is_empty() && base_f_collapsed == no_apos_target)
        {
            return true;
        }
        if !aka_lower.is_empty()
            && (base_f == aka_lower
                || (!no_apos_aka_lower.is_empty() && base_f == no_apos_aka_lower))
        {
            return true;
        }
        if !base_fold.is_empty() && !target_fold_lower.is_empty() && base_fold == target_fold_lower {
            return true;
        }
        if !base_fold.is_empty()
            && !aka_lower.is_empty()
            && base_fold == squash(&fold_latin_accents(&aka_lower))
        {
            return true;
        }
        // A catalogue title ending in a type word the ORIGIN's keyword cleaner
        // strips ("The Super Mario Galaxy Movie" -> resolved_title "The Super
        // Mario Galaxy"). The ±1-year check already ran, so a same-year file
        // whose only extra word is a type word is that film.
        const TYPE_WORD: &str = "(?i)^(?:movie|film|the\\s+movie|tv\\s*series|series|show)$";
        if let Some(extra) = base_f.strip_prefix(&format!("{} ", target_lower)) {
            if has(TYPE_WORD, extra.trim()) {
                return true;
            }
        }
        if let Some(prefix) = base_f.strip_suffix(&format!(" {}", target_lower)) {
            if is_uploader_prefix(prefix.trim()) {
                return true;
            }
        }
        if !no_apos_target.is_empty() && no_apos_target != target_lower {
            if let Some(prefix) = base_f.strip_suffix(&format!(" {}", no_apos_target)) {
                if is_uploader_prefix(prefix.trim()) {
                    return true;
                }
            }
        }
    }

    // 7b. AKA-PREFIX rule: the alternate title an upstream carries is often only
    // the FIRST part of the uploader's name ("Khurafat" for "Khurafat Perjanjian
    // Syaitan 2011 …"). The year guard above is what keeps this strict.
    if aka_lower.chars().count() >= 4 && base_f.starts_with(&format!("{} ", aka_lower)) {
        return true;
    }
    if no_apos_aka_lower.chars().count() >= 4
        && base_f.starts_with(&format!("{} ", no_apos_aka_lower))
    {
        return true;
    }

    // 8. Ampersand / dan / possessive variants.
    let collapsed_s_target = collapse_s(&target_lower);
    let mut variants: Vec<String> = vec![
        target_lower.replace(" and ", " dan "),
        target_lower.replace(" dan ", " and "),
        target_lower.replace(" and ", " "),
        target_lower.replace(" dan ", " "),
        target_lower.trim_end_matches('s').to_string(),
    ];
    if !collapsed_s_target.is_empty() && collapsed_s_target != target_lower {
        variants.push(collapsed_s_target.clone());
        variants.push(collapsed_s_target.trim_end_matches('s').to_string());
    }
    if !no_apos_target.is_empty() && no_apos_target != target_lower {
        variants.push(no_apos_target.clone());
        variants.push(no_apos_target.replace(" and ", " dan "));
        variants.push(no_apos_target.replace(" dan ", " and "));
        variants.push(no_apos_target.replace(" and ", " "));
        variants.push(no_apos_target.replace(" dan ", " "));
        variants.push(no_apos_target.trim_end_matches('s').to_string());
    }
    if !aka_lower.is_empty() && aka_lower != target_lower {
        variants.push(aka_lower.clone());
        variants.push(aka_lower.replace(" and ", " dan "));
        variants.push(aka_lower.replace(" dan ", " and "));
        variants.push(aka_lower.replace(" and ", " "));
        variants.push(aka_lower.replace(" dan ", " "));
    }
    if !no_apos_aka_lower.is_empty()
        && no_apos_aka_lower != aka_lower
        && no_apos_aka_lower != target_lower
    {
        variants.push(no_apos_aka_lower.clone());
        variants.push(no_apos_aka_lower.replace(" and ", " "));
        variants.push(no_apos_aka_lower.replace(" dan ", " "));
    }
    if variants.contains(&base_f) || variants.contains(&base_f_collapsed) {
        return true;
    }
    if !base_fold.is_empty() {
        if variants
            .iter()
            .any(|v| squash(&fold_latin_accents(v)) == base_fold)
        {
            return true;
        }
        if !target_fold_lower.is_empty() {
            if let Some(extra) = base_fold.strip_prefix(&format!("{} ", target_fold_lower)) {
                if has(
                    "(?i)^(?:movie|film|the\\s+movie|tv\\s*series|series|show)$",
                    extra.trim(),
                ) {
                    return true;
                }
            }
        }
    }
    for tv in &variants {
        if !tv.is_empty() {
            if let Some(prefix) = base_f.strip_suffix(&format!(" {}", tv)) {
                if is_uploader_prefix(prefix.trim()) {
                    return true;
                }
            }
        }
    }

    // 9. Multi-word exact word sequence match.
    if has_file_year && !sy.is_empty() {
        let tw: Vec<&str> = target_lower.split(' ').filter(|w| w.len() > 1).collect();
        let fw: Vec<&str> = base_f.split(' ').filter(|w| w.len() > 1).collect();
        if tw.len() >= 2 && fw == tw {
            return true;
        }
        if !no_apos_target.is_empty() && no_apos_target != target_lower {
            let natw: Vec<&str> = no_apos_target.split(' ').filter(|w| w.len() > 1).collect();
            if natw.len() >= 2 && fw == natw {
                return true;
            }
        }
    }

    false
}

/// True when a filename prefix is composed only of uploader/group/site words.
pub fn is_uploader_prefix(prefix: &str) -> bool {
    if prefix.is_empty() {
        return false;
    }
    if !has(UPLOADER_RE, prefix) {
        return false;
    }
    !has(
        "(?i)\\b(?:the|a|an|blade|late|maze|avatar|chapter|part)\\b",
        prefix,
    )
}

/* ── split parts, ordering, scoring, filters ───────────────────────────────── */

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitPartInfo {
    pub is_part: bool,
    pub base_key: String,
    pub clean_base: String,
    pub part_num: i32,
    pub total_parts: i32,
}

/// Port of `fd_extract_split_part_info()` (backend.php:8144).
pub fn extract_split_part_info(filename: &str, caption: &str) -> SplitPartInfo {
    let f = filename.trim();
    let cap = caption.trim();
    let mut part_num = 0i32;
    let mut total_parts = 0i32;
    let mut matched = false;
    let mut clean_base = f.to_string();

    const RE_PART: &str = "(?i)[._\\s-]part[._\\s-]*0*(\\d{1,4})(?:[._\\s/-]+(?:of[._\\s-]+)?0*(\\d{1,4}))?";
    const RE_DOT_PART: &str = "(?i)[._\\s-]0*(\\d{1,3})\\.(mp4|mkv|avi|webm)$";
    const RE_EXT_PART: &str = "(?i)\\.(?:mp4|mkv|avi|webm)\\.0*(\\d{1,4})$";

    if let Some(g) = groups(RE_PART, f) {
        part_num = g[1].parse().unwrap_or(0);
        if !g[2].is_empty() {
            total_parts = g[2].parse().unwrap_or(0);
        }
        matched = true;
        clean_base = replace_all(RE_PART, f, "");
    } else if let Some(g) = groups(RE_DOT_PART, f) {
        let num: i32 = g[1].parse().unwrap_or(0);
        let whole = &g[0];
        // NEVER read a codec suffix as a part index: "…AAC2.0.H.264.mkv" is
        // "." + "264" + ".mkv", so every H.264/H.265 release was flagged as
        // "part 264" and then dropped by `exclude_part_title` (ON by default).
        if !has("(?i)\\b(2160p|1080p|720p|480p|360p)\\b", whole) && !(263..=265).contains(&num) {
            part_num = num;
            matched = true;
            let g1 = &g[1];
            let g2 = &g[2];
            let pattern = format!("(?i)[._\\s-]0*{}\\.{}$", escape(g1), g2);
            clean_base = replace_all(&pattern, f, &format!(".{}", g2));
        }
    } else if let Some(g) = groups(RE_EXT_PART, f) {
        part_num = g[1].parse().unwrap_or(0);
        matched = true;
        let pattern = format!("(?i)\\.0*{}$", escape(&g[1]));
        clean_base = replace_all(&pattern, f, "");
    }

    if matched && total_parts == 0 && !cap.is_empty() {
        // The literal "Part 1 of 2" form. PHP's `[/|of]` was a CHARACTER CLASS,
        // which is why a one-file part release could not prove it was a split.
        let pattern = format!(
            "(?i)part[._\\s-]*0*{}\\s*(?:/|\\||\\bof\\b)\\s*0*(\\d{{1,4}})",
            part_num
        );
        if let Some(g) = groups(&pattern, cap) {
            total_parts = g[1].parse().unwrap_or(0);
        }
    }

    // The part index is sometimes ONLY in the caption. Measured on
    // `Obsession.2026.2160p.4K.HDR10.DV.BluRay.x265.10bit.HEVC.Org.AMZN.mkv`:
    // all eight chunks share that exact title (the indexer dropped `partNNN`
    // from it) while the caption still says `…J3RICO.part005.mkv` /
    // `…J3RICO.part008.mkv`. Without this they read as eight identical films —
    // and the caption is the only place an ORDER is available, because duration
    // and the origin's `added_date` cannot prove one.
    if !matched {
        let caption_first = cap.lines().next().unwrap_or("").trim();
        if let Some(g) = groups(RE_PART, caption_first) {
            let n: i32 = g[1].parse().unwrap_or(0);
            if n > 0 {
                part_num = n;
                if !g[2].is_empty() {
                    total_parts = g[2].parse().unwrap_or(0);
                }
                matched = true;
                // Group by the FILE name, never by the caption: the eight
                // siblings share one title, and the title is what
                // `detect_parts_by_duration` keys on too.
                clean_base = f.to_string();
            }
        }
    }

    if !matched || part_num <= 0 {
        return SplitPartInfo {
            is_part: false,
            base_key: String::new(),
            clean_base: String::new(),
            part_num: 0,
            total_parts: 0,
        };
    }

    let base_key = replace_all("[^\\p{L}\\p{N}]+", &clean_base, ".")
        .trim_matches('.')
        .to_lowercase();
    SplitPartInfo {
        is_part: true,
        base_key,
        clean_base,
        part_num,
        total_parts,
    }
}

/// One file as the split-part grouper sees it. The PHP mutates the JSON object
/// (`is_split_part`, `part_num`, `total_parts`, `clean_base`); this struct is
/// the same annotation without the JSON round-trip.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileItem {
    pub title: String,
    pub caption: String,
    pub file_size: u64,
    /// Runtime in seconds (`fd_resolve_shortcode`'s `duration`). `0` = unknown.
    /// Used by [`detect_parts_by_duration`], which is the only evidence left when
    /// seven chunks of one film all carry the SAME file name.
    pub duration: u32,
    /// Annotation written by [`group_split_parts`] / [`detect_parts_by_duration`].
    pub is_split_part: bool,
    pub part_num: i32,
    pub total_parts: i32,
    pub clean_base: String,
}

/// Port of `fd_group_split_parts()` (backend.php:8215).
///
/// A "part" only exists relative to SIBLINGS. Flagging a name that merely
/// CONTAINS "Part 1" hid whole single files — catalogue item
/// `pm_file_ZMf0Rn` ("Detective.Conan.Episode.100.Malay.Dub.Special.Episode
/// .Part.1…", 1.28 GB, one complete episode) answered "No Streams Found"
/// because `exclude_part_title` is ON by default.
/// Returns the permutation that was applied: `files_new[i] == files_old[perm[i]]`.
///
/// The caller MUST use it. This function reorders its input (PHP sorts in
/// place), and the stream route keeps a second parallel vector of the source
/// rows (`matched`). Pairing the sorted annotations with an unsorted row list by
/// index displayed a DROPPED part's file name under a KEPT row's payload —
/// measured on `pm:post:9000009680`: 77 rows flagged and filtered out, yet 16
/// part names reached the player because `matched[i]` and `items[i]` were no
/// longer the same file. With the permutation the two vectors are re-aligned and
/// `items[i]` is `matched[i]` again.
pub fn group_split_parts(files: &mut [FileItem]) -> Vec<usize> {
    // Order parts of the same base ascending (PHP sorts in place).
    let infos: Vec<SplitPartInfo> = files
        .iter()
        .map(|f| extract_split_part_info(&f.title, &f.caption))
        .collect();
    let mut perm: Vec<usize> = (0..files.len()).collect();
    perm.sort_by(|&a, &b| {
        let (ai, bi) = (&infos[a], &infos[b]);
        if ai.is_part && bi.is_part && ai.base_key == bi.base_key {
            ai.part_num.cmp(&bi.part_num)
        } else {
            std::cmp::Ordering::Equal
        }
    });
    let moved = perm.iter().enumerate().any(|(i, &p)| i != p);
    let mut infos = infos;
    if moved {
        let sorted: Vec<FileItem> = perm.iter().map(|&i| files[i].clone()).collect();
        for (slot, item) in files.iter_mut().zip(sorted) {
            *slot = item;
        }
        infos = perm.iter().map(|&i| infos[i].clone()).collect();
    }

    let mut parts_by_base: HashMap<String, HashSet<i32>> = HashMap::new();
    for info in &infos {
        if info.is_part && !info.base_key.is_empty() {
            parts_by_base
                .entry(info.base_key.clone())
                .or_default()
                .insert(info.part_num);
        }
    }

    for (file, info) in files.iter_mut().zip(infos.iter()) {
        if !info.is_part {
            continue;
        }
        let sibling_parts = parts_by_base.get(&info.base_key).map(|s| s.len()).unwrap_or(0);
        let is_real_split = info.total_parts > 1 || sibling_parts > 1;
        if !is_real_split {
            continue;
        }
        file.is_split_part = true;
        file.part_num = info.part_num;
        file.total_parts = info.total_parts;
        file.clean_base = info.clean_base.clone();
    }
    perm
}

/// Re-align a parallel vector (the route's source rows) with the order
/// [`group_split_parts`] left the annotations in.
///
/// Split parts only exist relative to siblings, and the PHP sorts the part list
/// ascending — so the annotation vector is permuted while the row vector is not.
/// Every `rows[i]` / `items[i]` pairing after that point is a different file
/// unless the permutation is replayed here.
pub fn reorder_with<T: Clone>(rows: &[T], perm: &[usize]) -> Vec<T> {
    if perm.len() != rows.len() {
        return rows.to_vec();
    }
    perm.iter().map(|&i| rows[i].clone()).collect()
}

/// A chunk that is still shorter than a feature on its own. Above this a file is
/// a whole film/episode, not a slice of one (measured: the two 1080p copies of
/// `F1.The.Movie.2025…YTS.MX.mp4` are 9 948 / 9 947 s).
pub const PART_MAX_CHUNK_SECS: u32 = 3900;

/// The chunks of a split release have to ADD UP to something feature-length,
/// otherwise they are unrelated short clips that merely share a file name.
pub const PART_MIN_TOTAL_SECS: u32 = 3600;

/// Detect split parts from DURATION when the file names carry no part index.
///
/// Measured on `tt16311594` (F1 The Movie 2025, 4K DV): seven files are all named
/// `F1.The.Movie.2025.Hybrid.2160p.WEB.DL.DV.HDR.DDP5.1.Atmos.H265.AOC.mkv`, so
/// [`extract_split_part_info`] sees nothing, and the player shows a wall of
/// identical rows. Their runtimes are the only proof: 1361 + 1323 + 1323 + 1401
/// + 1289 + 1313 + 1322 s = 9332 s (155.5 min) — the film, chopped into ~22 min
/// chunks that each restart their Matroska timeline at 0.
///
/// The rule is deliberately NARROW — identical names only, and all three
/// conditions must hold:
///
/// 1. at least two files share the exact name (one file is never a part);
/// 2. every one of them has a known duration;
/// 3. no chunk is feature-length (`max < PART_MAX_CHUNK_SECS`) while together
///    they are (`sum >= PART_MIN_TOTAL_SECS`).
///
/// Condition 3 is what keeps a duplicated WHOLE file out: two copies of a 2h45m
/// film have `max ≈ sum/2 ≈ 9.9 ks` and fail the cap.
///
/// The ORDINAL is never invented here: duration cannot prove which chunk comes
/// first (measured: across 99 multi-part groups the origin's `added_date` agrees
/// with part order 44 times and disagrees 51 — a coin flip). An ordinal already
/// set by the name, the caption or the origin's `part` column is preserved.
pub fn detect_parts_by_duration(files: &mut [FileItem]) {
    let mut by_name: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, f) in files.iter().enumerate() {
        if f.duration == 0 {
            continue; // nothing to reason about
        }
        let key = f.title.trim().to_lowercase();
        if key.is_empty() {
            continue;
        }
        by_name.entry(key).or_default().push(i);
    }

    for (name, idxs) in by_name {
        if idxs.len() < 2 {
            continue;
        }
        let max = idxs
            .iter()
            .map(|i| files[*i].duration)
            .max()
            .unwrap_or(0);
        let sum: u64 = idxs.iter().map(|i| files[*i].duration as u64).sum();
        if max >= PART_MAX_CHUNK_SECS || sum < PART_MIN_TOTAL_SECS as u64 {
            continue;
        }
        let total = idxs.len() as i32;
        for i in idxs {
            files[i].is_split_part = true;
            files[i].total_parts = total;
            if files[i].clean_base.is_empty() {
                files[i].clean_base = name.clone();
            }
        }
    }
}

/// Port of `fd_calculate_stream_sort_score()` (backend.php:8084).
pub fn stream_sort_score(title: &str, caption: &str, file_size: u64, preferred_res: &str) -> f64 {
    let tags = extract_media_tags(title, caption);
    let mut res_score = match tags.resolution.to_lowercase().as_str() {
        "4k" => 6_000_000_000.0,
        "1080p" => 5_000_000_000.0,
        "720p" => 4_000_000_000.0,
        "540p" => 3_000_000_000.0,
        "480p" => 2_000_000_000.0,
        "360p" => 1_000_000_000.0,
        _ => 500_000_000.0,
    };
    if preferred_res != "auto" && tags.resolution.to_lowercase() == preferred_res.to_lowercase() {
        res_score += 10_000_000_000.0;
    }
    let source_score = match tags.source.to_uppercase().as_str() {
        "REMUX" => 80_000_000.0,
        "BLURAY" => 70_000_000.0,
        "WEB-DL" => 60_000_000.0,
        "WEBRIP" => 50_000_000.0,
        "HDRIP" => 40_000_000.0,
        "HDTV" => 30_000_000.0,
        "DVDRIP" => 20_000_000.0,
        "CAM" => 1_000_000.0,
        _ => 30_000_000.0,
    };
    let mut visual_score = 0.0;
    if tags.visual.iter().any(|v| v == "DV") {
        visual_score += 4_000_000.0;
    }
    if tags.visual.iter().any(|v| v == "HDR") {
        visual_score += 2_000_000.0;
    }
    if tags.visual.iter().any(|v| v == "10bit") {
        visual_score += 1_000_000.0;
    }
    let codec_score = match tags.codec.to_uppercase().as_str() {
        "AV1" => 300_000.0,
        "HEVC" => 200_000.0,
        "H.264" => 100_000.0,
        _ => 0.0,
    };
    let size_score = (file_size as f64 / (1024.0 * 1024.0)).clamp(0.0, 99_999.0);
    res_score + source_score + visual_score + codec_score + size_score
}

/// `$categorizeResolution()`.
pub fn categorize_resolution_key(title: &str, caption: &str) -> String {
    match extract_media_tags(title, caption).resolution.to_lowercase().as_str() {
        "4k" => "4k",
        "1080p" => "1080p",
        "720p" => "720p",
        "540p" | "480p" | "360p" => "sd",
        _ => "unknown",
    }
    .to_string()
}

/// `$categorizeQuality()` — note `webrip` intentionally maps to `unknown`.
pub fn categorize_quality_key(title: &str, caption: &str) -> String {
    match extract_media_tags(title, caption).source.to_lowercase().as_str() {
        "remux" => "remux",
        "bluray" => "bluray",
        "web-dl" => "webdl",
        "hdrip" => "webrip",
        "hdtv" => "hdtv",
        "cam" => "cam",
        _ => "unknown",
    }
    .to_string()
}

/// The AIOStreams-style filter config (`stream_config` in
/// `storage/catalog_settings.json`).
///
/// `Default` is written by hand because the PHP's defaults are not all `false`:
/// `exclude_unplayable` and `exclude_part_title` default to **ON**, and getting
/// that wrong silently un-hides every archive and split part.
#[derive(Debug, Clone)]
pub struct StreamFilterConfig {
    pub resolutions: HashMap<String, bool>,
    pub qualities: HashMap<String, bool>,
    pub encodes: HashMap<String, bool>,
    pub visual_tags: HashMap<String, bool>,
    pub exclude_cam: bool,
    pub exclude_unplayable: bool,
    pub exclude_part_title: bool,
    pub min_size_mb: i64,
    pub max_size_gb: i64,
    pub excluded_keywords: Vec<String>,
    pub required_keywords: Vec<String>,
}

impl Default for StreamFilterConfig {
    fn default() -> Self {
        StreamFilterConfig {
            resolutions: HashMap::new(),
            qualities: HashMap::new(),
            encodes: HashMap::new(),
            visual_tags: HashMap::new(),
            exclude_cam: false,
            exclude_unplayable: true,
            exclude_part_title: true,
            min_size_mb: 0,
            max_size_gb: 0,
            excluded_keywords: Vec::new(),
            required_keywords: Vec::new(),
        }
    }
}

impl StreamFilterConfig {
    pub fn from_json(cfg: &serde_json::Value) -> Self {
        fn table(v: Option<&serde_json::Value>) -> HashMap<String, bool> {
            let mut out = HashMap::new();
            if let Some(serde_json::Value::Object(map)) = v {
                for (k, val) in map {
                    out.insert(k.clone(), val.as_bool().unwrap_or(true));
                }
            }
            out
        }
        fn words(v: Option<&serde_json::Value>) -> Vec<String> {
            v.and_then(|x| x.as_str())
                .unwrap_or("")
                .to_lowercase()
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        }
        let flag = |k: &str, default: bool| {
            cfg.get(k).and_then(|v| v.as_bool()).unwrap_or(default)
        };
        StreamFilterConfig {
            resolutions: table(cfg.get("resolutions")),
            qualities: table(cfg.get("qualities")),
            encodes: table(cfg.get("encodes")),
            visual_tags: table(cfg.get("visual_tags")),
            exclude_cam: flag("exclude_cam", false),
            exclude_unplayable: flag("exclude_unplayable", true),
            exclude_part_title: flag("exclude_part_title", true),
            min_size_mb: cfg.get("min_size_mb").and_then(|v| v.as_i64()).unwrap_or(0).max(0),
            max_size_gb: cfg.get("max_size_gb").and_then(|v| v.as_i64()).unwrap_or(0).max(0),
            excluded_keywords: words(cfg.get("excluded_keywords")),
            required_keywords: words(cfg.get("required_keywords")),
        }
    }
}

/// Port of the AIOStreams filter block.
///
/// Returns `(index, resolution_key)` for every kept file. **A direct
/// `pm_file_*` item must NOT be passed through here** — it is one explicitly
/// chosen file with no alternative source, so the filters can only turn it into
/// the filtered-out card (PHP and the app both skip them).
pub fn apply_stream_filters(files: &[FileItem], cfg: &StreamFilterConfig) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (idx, item) in files.iter().enumerate() {
        let mut title = item.title.clone();
        let caption = item.caption.clone();
        let size = item.file_size;

        if !caption.is_empty() && has(GENERIC_NAME, title.trim()) {
            let first = caption.lines().next().unwrap_or("").trim();
            if !first.is_empty() {
                title = first.to_string();
            }
        }
        let lower_title = title.to_lowercase();
        let tags = extract_media_tags(&title, &caption);

        // 1. Resolution.
        let r_key = categorize_resolution_key(&title, &caption);
        if let Some(false) = cfg.resolutions.get(&r_key) {
            continue;
        }
        // 2. Quality.
        let q_key = categorize_quality_key(&title, &caption);
        if cfg.exclude_cam && q_key == "cam" {
            continue;
        }
        if let Some(false) = cfg.qualities.get(&q_key) {
            continue;
        }
        // Unplayable (archives / raw split chunks).
        let has_playable_ext = has(
            "(?i)\\.(mp4|m4v|mkv|webm|avi|mov|ts|m2ts|flv|wmv|3gp|mpg|mpeg|mp3|m4a|flac|wav|ogg|opus|aac)$",
            &title,
        );
        let is_archive = has(
            "(?i)\\.(?:zip|rar|7z|tar|gz|bz2|xz|iso|bin|exe|apk|pdf|epub)$",
            &title,
        );
        let is_raw_split = !has_playable_ext && has("\\.(?:0\\d{2,3}|\\d{3})$", &title);
        if cfg.exclude_unplayable && !has_playable_ext && (is_raw_split || is_archive) {
            continue;
        }
        // 3. Encodes.
        if tags.codec == "HEVC" {
            if let Some(false) = cfg.encodes.get("hevc") {
                continue;
            }
        }
        if tags.codec == "H.264" {
            if let Some(false) = cfg.encodes.get("avc") {
                continue;
            }
        }
        if tags.codec == "AV1" {
            if let Some(false) = cfg.encodes.get("av1") {
                continue;
            }
        }
        // 4. Visual tags.
        if tags.visual.iter().any(|v| v == "HDR") {
            if let Some(false) = cfg.visual_tags.get("hdr") {
                continue;
            }
        }
        if tags.visual.iter().any(|v| v == "DV") {
            if let Some(false) = cfg.visual_tags.get("dv") {
                continue;
            }
        }
        // 5. Size range.
        if cfg.min_size_mb > 0 && size > 0 && (size as i64) < cfg.min_size_mb * 1_048_576 {
            continue;
        }
        if cfg.max_size_gb > 0 && size > 0 && (size as i64) > cfg.max_size_gb * 1_073_741_824 {
            continue;
        }
        // 6/7. Keyword gates.
        if !cfg.excluded_keywords.is_empty()
            && cfg.excluded_keywords.iter().any(|k| lower_title.contains(k))
        {
            continue;
        }
        if !cfg.required_keywords.is_empty()
            && !cfg.required_keywords.iter().any(|k| lower_title.contains(k))
        {
            continue;
        }
        // 8. Exclude split-release parts (flag comes from group_split_parts).
        if cfg.exclude_part_title && item.is_split_part {
            continue;
        }
        out.push((idx, r_key));
    }
    out
}

/* ── regression suite ──────────────────────────────────────────────────────── */

#[cfg(test)]
mod tests {
    use super::*;

    fn names(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    /* fd_stream_keyword_variants -------------------------------------------- */

    #[test]
    fn stream_keyword_variants_cover_the_possessive_forms() {
        // Values verified against `bin/php.exe` running the real
        // `fd_stream_keyword_variants()`: every form it produces must be probe-able
        // or the fallback ladder finds nothing.
        assert_eq!(
            stream_keyword_variants("Grey's Anatomy"),
            vec!["Grey's Anatomy", "Grey s Anatomy", "Greys Anatomy", "Grey Anatomy"]
        );
        // The PHP's `s['’`]` branch wins before `\bs\b` for a word ending in "s",
        // so "Princess's" yields "Princes" — identical in PHP (asserted there), and
        // deliberately reproduced rather than corrected.
        assert_eq!(
            stream_keyword_variants("Princess's"),
            vec!["Princess's", "Princess s", "Princesss", "Princes"]
        );
        // A plain title has exactly one form, so the ladder adds no queries.
        assert_eq!(stream_keyword_variants("Positively Yours"), vec!["Positively Yours"]);
        // Empty in, nothing out (never a query of "").
        assert!(stream_keyword_variants("   ").is_empty());
    }

    /* trap 4: a title whose LAST word is a type word ------------------------ */

    #[test]
    fn movie_accepts_type_word_suffix() {
        // The origin answers resolved_title "The Super Mario Galaxy" for the film
        // "The Super Mario Galaxy Movie"; 111 of 112 files were rejected before
        // this rule existed.
        assert!(movie_file_matches_title(
            "The.Super.Mario.Galaxy.Movie.2026.1080p.WEB-DL.mkv",
            "The Super Mario Galaxy",
            "2026",
            "",
            ""
        ));
    }

    #[test]
    fn movie_rejects_a_different_film() {
        // post 9000031785 "The Boy 2026" resolved to Balan: The Boy — 100 wrong
        // streams were served. The prefix rule must not accept a substring.
        assert!(!movie_file_matches_title(
            "Balan.The.Boy.2026.1080p.mkv",
            "The Boy",
            "2026",
            "",
            ""
        ));
    }

    /* reported: a title with an ampersand ------------------------------- */

    /// `pm:post:9296788885` "Avaraachan & Sons 2026 • Movie": the origin
    /// returned SIX files for this post and the matcher rejected all of them, so
    /// the title answered "No Streams Found" while everything it needed was
    /// present. The `&` is the suspect: the TARGET is rewritten to " and ", the
    /// FILE side keeps a bare `&` (dropped as punctuation by `clean_media_title`)
    /// or spells it "and", and the two forms have to meet.
    /// Reported: "Oswald" answered "No Streams Found" for every episode although
    /// the origin held 50 files. theTVDB numbers the show 1..52 and names E2
    /// "The Camping Trip"; the uploader ships two segments per file and calls it
    /// "S01E02 Sleepover & Big Banana". The guard read "not my title" as "another
    /// episode" and dropped the whole list.
    #[test]
    fn an_episode_title_in_another_ordering_is_not_a_mismatch() {
        let own = vec!["the camping trip".to_string()];
        let siblings = vec![
            "the polka dot umbrella".to_string(),
            "chasing the ice cream truck".to_string(),
        ];
        // Named differently from what the metadata calls it -> KEEP (fail open).
        assert!(episode_subtitle_matches_with_siblings(
            "Oswald.S01E02.Sleepover.&.Big.Banana.480p.AMZN.WEB.DL.mkv",
            "oswald",
            &own,
            &siblings
        ));
        // Named as a DIFFERENT known episode -> reject (the pinned protection).
        assert!(!episode_subtitle_matches_with_siblings(
            "Oswald.S01E02.The.Polka.Dot.Umbrella.480p.WEB.mkv",
            "oswald",
            &own,
            &siblings
        ));
        // Its own title -> keep.
        assert!(episode_subtitle_matches_with_siblings(
            "Oswald.S01E02.The.Camping.Trip.480p.WEB.mkv",
            "oswald",
            &own,
            &siblings
        ));
        // No metadata at all -> keep (the guard can never hide a stream on its own).
        assert!(episode_subtitle_matches_with_siblings(
            "Oswald.S01E02.Sleepover.&.Big.Banana.mkv",
            "oswald",
            &[],
            &[]
        ));
        // A whole title still has to be matched as a whole: a one-word subtitle is
        // the uploader tag, not evidence.
        assert!(episode_subtitle_matches_with_siblings(
            "Oswald.S01E02.PrakyTV.1080p.WEB.mkv",
            "oswald",
            &own,
            &siblings
        ));
    }

    /// Reported: opening Black Clover **season 2** listed season 1's files.
    /// The addon item is "Black Clover Season 2 2026" (`kitsu:50024`) whose
    /// episodes are numbered from 1, while the origin files them `S02E01..13`.
    #[test]
    fn season_from_title_reads_the_markers_addons_use() {
        assert_eq!(season_from_title("Black Clover Season 2 2026"), 2);
        assert_eq!(season_from_title("Black Clover Second Season 2026"), 2);
        assert_eq!(season_from_title("Foo 3rd Season"), 3);
        assert_eq!(season_from_title("Foo Season 02"), 2);
        // No marker: the caller keeps whatever the id said.
        assert_eq!(season_from_title("Black Clover 2017"), 0);
        assert_eq!(season_from_title("The 100"), 0);
        assert_eq!(season_from_title("Oswald"), 0);
    }

    #[test]
    fn movie_matches_a_title_with_an_ampersand() {
        let title = "Avaraachan & Sons 2026 • Movie";
        for f in [
            "PrakyTV.Avaraachan.&.Sons.2026.Tamil.TRUE.WEB.DL.mkv",
            "PrakyTV.Avaraachan.&.Sons.2026.Tamil.HQ.HDRip.720.mkv",
            "Avaraachan.and.Sons.2026.Submalay.mp4",
        ] {
            assert!(
                movie_file_matches_title(f, title, "2026", "", ""),
                "the ampersand title must match {f}"
            );
        }
        // The control: the same title with the entity decoded must behave the
        // same way, or the `&` is not what the rule is testing.
        for f in ["PrakyTV.Avaraachan.&.Sons.2026.Tamil.TRUE.WEB.DL.mkv"] {
            assert!(movie_file_matches_title(
                f,
                "Avaraachan & Sons 2026 • Movie",
                "2026",
                "",
                ""
            ));
        }
    }

    /* trap 5: accents -------------------------------------------------------- */

    #[test]
    fn movie_matches_accent_folded_title() {
        assert!(movie_file_matches_title(
            "Siccin.9.2026.1080p.NF.WEB-DL.mkv",
            "Siccîn 9 2026 • Movie",
            "2026",
            "",
            ""
        ));
        assert_eq!(fold_latin_accents("Siccîn"), "Siccin");
    }

    #[test]
    fn fold_leaves_untransliterable_scripts_alone() {
        assert_eq!(fold_latin_accents("映画"), "映画");
        assert_eq!(fold_latin_accents("Фильм"), "Фильм");
    }

    /* alternate titles ------------------------------------------------------- */

    #[test]
    fn movie_accepts_aka_prefix() {
        // "Khurafat: Perjanjian Syaitan" is indexed as "Superstition"; the
        // upstream only knows the first word.
        assert!(movie_file_matches_title(
            "Khurafat.Perjanjian.Syaitan.2011.1080p.NF.WEB-DL.mkv",
            "Superstition",
            "2011",
            "",
            "Khurafat"
        ));
    }

    #[test]
    fn movie_title_words_prefix_match_is_prefix_only() {
        assert!(title_words_prefix_match("Khurafat: Perjanjian Syaitan", "Khurafat"));
        assert!(!title_words_prefix_match("Balan: The Boy", "The Boy"));
    }

    /* reported leak: same name, adjacent year -------------------------------- */

    // "Dhoom Dhaam" was released twice (2024 and 2025). The 2025 title's `/stream`
    // list served the 2024 rip because the year guard's ±1 window accepted it. The
    // window is now a RESCUE that yields to an exact-year file, so the pair can
    // never land in one list again.
    #[test]
    fn movie_rejects_the_adjacent_year_release() {
        let tiers = vec![
            movie_year_tier("Dhoom.Dhaam.2024.1080p.WEB-DL.x264.mkv", "", "2025"),
            movie_year_tier("Dhoom.Dhaam.2025.1080p.WEB-DL.x264.mkv", "", "2025"),
        ];
        assert_eq!(tiers, vec![YearTier::Near, YearTier::Exact]);
        assert_eq!(
            year_keep_mask(&tiers),
            vec![false, true],
            "LEAK: the 2024 rip survived next to the 2025 rip"
        );

        // Mirror image: the 2024 title must not serve the 2025 rip either.
        let tiers = vec![
            movie_year_tier("Dhoom.Dhaam.2025.1080p.WEB-DL.x264.mkv", "", "2024"),
            movie_year_tier("Dhoom.Dhaam.2024.1080p.WEB-DL.x264.mkv", "", "2024"),
        ];
        assert_eq!(tiers, vec![YearTier::Near, YearTier::Exact]);
        assert_eq!(year_keep_mask(&tiers), vec![false, true]);
    }

    #[test]
    fn movie_year_tier_separates_exact_near_and_conflict() {
        let file = "Dhoom.Dhaam.2024.1080p.WEB-DL.x264.mkv";
        assert_eq!(movie_year_tier(file, "", "2024"), YearTier::Exact);
        assert_eq!(movie_year_tier(file, "", "2025"), YearTier::Near);
        assert_eq!(movie_year_tier(file, "", "2023"), YearTier::Near);
        // Two years away is a contradiction, and stays a hard reject.
        assert_eq!(movie_year_tier(file, "", "2022"), YearTier::Conflict);
        assert!(!movie_file_matches_title(file, "Dhoom Dhaam", "2022", "", ""));
        // A file with no year anywhere has nothing to contradict.
        assert_eq!(
            movie_year_tier("Dhoom.Dhaam.1080p.mkv", "", "2025"),
            YearTier::NoEvidence
        );
        // An unknown searched year never gates.
        assert_eq!(movie_year_tier(file, "", ""), YearTier::NoEvidence);
    }

    // The control: a title whose only files carry an imprecise year still gets
    // them, or a "fix" that simply rejects everything would look green.
    #[test]
    fn movie_keeps_the_rescue_when_no_exact_year_exists() {
        let file = "Dhoom.Dhaam.2024.1080p.WEB-DL.x264.mkv";
        assert_eq!(movie_year_tier(file, "", "2025"), YearTier::Near);
        assert_eq!(year_keep_mask(&[YearTier::Near]), vec![true]);
        assert!(movie_file_matches_title(file, "Dhoom Dhaam", "2025", "", ""));
        // ... and the correctly-yeared file always matches.
        assert!(movie_file_matches_title(
            "Dhoom.Dhaam.2025.1080p.WEB-DL.x264.mkv",
            "Dhoom Dhaam",
            "2025",
            "",
            ""
        ));
    }

    // A rip that omits the year from its file name carries it in the caption. The
    // caption is then the ONLY year evidence, and for a MOVIE it must be exact —
    // no ±1 window, because a caption is written metadata, not a guess. The
    // series branch keeps its own rules and is deliberately untouched.
    #[test]
    fn movie_caption_year_vetoes_an_unyeared_filename() {
        let file = "Dhoom.Dhaam.1080p.WEB-DL.x264.mkv";
        let caption = "Dhoom Dhaam (2024) 1080p WEB-DL\nTelegram rip";
        assert_eq!(movie_year_tier(file, caption, "2024"), YearTier::Exact);
        assert_eq!(movie_year_tier(file, caption, "2025"), YearTier::Conflict);
        assert_eq!(movie_year_tier(file, caption, "2023"), YearTier::Conflict);
        assert!(
            !movie_file_matches_title(file, "Dhoom Dhaam", "2025", caption, ""),
            "LEAK: an unyeared file name with a 2024 caption joined the 2025 title"
        );
        assert!(movie_file_matches_title(file, "Dhoom Dhaam", "2024", caption, ""));
    }

    /* trap 1: a year EMBEDDED in a series name ------------------------------- */

    #[test]
    fn series_keeps_embedded_year() {
        // "Kudrat 1968: High Council 2026" — all 9 S01E01 files must match.
        for file in [
            "Kudrat.1968.High.Council.2026.EP1.MalaySub.mp4",
            "Kudrat.1968.High.Council.S01E01.1968.1080p.NF.WEB.DL.AAC2.0.H.264.mkv",
            "FNP22.Kudrat.1968.High.Council.2026.EP01.Subs.Malay.mp4",
        ] {
            assert!(
                series_file_matches_title(file, "Kudrat 1968: High Council 2026 • TvSeries", 1, 1, "2026", "", &[]),
                "{file} should match Kudrat 1968"
            );
        }
    }

    #[test]
    fn series_matches_hey_kay_navin_unyeared_file() {
        assert!(series_file_matches_title(
            "Hey.Kay.Navin.S01E01.Something.New.1080p.ZEE5.WEB.DL.DDP5.1.mkv",
            "Hey Kay Navin? 2026 • TvSeries",
            1,
            1,
            "2026",
            "",
            &["Something New".to_string()]
        ));
        assert!(series_file_matches_title(
            "Hey.Kay.Navin.S01E01.Something.New.1080p.ZEE5.WEB.DL.DDP5.1.mkv",
            "Hey Kay Navin",
            1,
            1,
            "2026",
            "",
            &["Something New".to_string()]
        ));
    }

    /// Reported leak: "The Witch" (2025) listed three episodes of **I'm Not
    /// Afraid**, whose S01E01 is simply *titled* "The Witch".
    ///
    /// `Im.Not.Afraid.S01E01.The.Witch…` puts the OWNING show before the episode
    /// marker (`im not afraid`) and a FOREIGN show's name in the episode-title
    /// slot. The post-episode text (`after_ep`) is therefore only allowed to
    /// supply the title when the pre-episode part is a channel/uploader tag
    /// (`CDL.E01.The.Rebel.Princess`, `DramaOST.E01.The.Witch`), never when it is
    /// a real multi-word phrase — that phrase is the file's own series name.
    #[test]
    fn series_does_not_take_a_foreign_episode_name_as_its_title() {
        for file in [
            "Im.Not.Afraid.S01E01.The.Witch.1080p.NF.WEB.DL.DDP5.1.Atmos.DV.HDR.mkv",
            "Im.Not.Afraid.S01E01.The.Witch.1080p.NF.WEB.DL.DDP5.1.Atmos.AV1.mkv",
            "Im.Not.Afraid.S01E01.The.Witch.1080p.NF.WEB.DL.DD.2B5.1.Atmos.H.mkv",
        ] {
            assert!(
                !series_file_matches_title(file, "The Witch 2025", 1, 1, "2025", "", &[]),
                "LEAK: {file} belongs to I'm Not Afraid, not The Witch"
            );
        }

        // The same shape with a multi-word title: the whole-name phrase search
        // must not find "my demon" inside another show's episode title either.
        assert!(!series_file_matches_title(
            "Other.Show.S01E01.My.Demon.1080p.NF.WEB.DL.x264.mkv",
            "My Demon 2023",
            1,
            1,
            "2023",
            "",
            &[]
        ));

        // Controls — the rest of the S01E01 list the live server answered for
        // `pm:post:8798831284` ("The Witch 2025"). Every one of these IS an
        // episode of The Witch and must survive the tightening above.
        for file in [
            "The.Witch.EP1.1080p.MalaySub.mp4",
            "The.Witch.2025.EP1.720p.AAC.x264.Me&Friend.mp4",
            "The.Witch.EP1.MalaySub.mp4",
            "The.Witch.2025.EP01.MalaySub.mp4",
            "The.Witch.S01E01.The.Female.Student.1080p.NF.WEB.DL.x265.mkv",
            "The.Witch.S01E01.1080p.VIKI.WEB.DL.AAC2.0.x264.CPTN5.mkv",
            "The.Witch.S01E01.720p.VIKI.WEB.DL.AAC2.0.x264.CPTN5D.mkv",
            "The.Witch.S01E01.The.Female.Student.720p.NF.WEB.DL.x.mkv",
            "The.Witch.2025.S01E01.720p.WEB.DL.Korean.with.Hindi.Eng.Sub.mkv",
            "The.Witch.S01E01.NF.x264.1080p.mkv",
            "The.Witch.2025.S01E01.480p.WEB.DL.Korean.with.Hindi.Eng.Sub.mkv",
            // Post-episode title with NO pre-episode prefix at all.
            "E01.The.Witch.540p.ICONFLIX.mkv",
            // Post-episode title behind a known uploader/site tag.
            "DramaOST.E01.The.Witch.1080p.WEB.DL.viki.mkv",
            "DramaOST.E01.The.Witch.720p.WEB.DL.viki.mkv",
            "DramaOST.E01.The.Witch.480p.WEB.DL.viki.mkv",
            "OLD.E01.The.Witch.2025.mp4",
            "ADrama.Lovers.EP01.The.Witch.2025.720p.HDTV.mkv",
        ] {
            assert!(
                series_file_matches_title(file, "The Witch 2025", 1, 1, "2025", "", &[]),
                "{file} is an episode of The Witch and must stay in the list"
            );
        }

        // Pattern B with an unknown channel tag must survive: the PHP names
        // `CDL.E01.The.Rebel.Princess` / `YDF.E01.…` as the whole reason
        // `after_ep` is consulted at all.
        assert!(series_file_matches_title(
            "CDL.E01.The.Rebel.Princess.2021.mp4",
            "The Rebel Princess 2021",
            1,
            1,
            "2021",
            "",
            &[]
        ));
        assert!(series_file_matches_title(
            "YDF.E01.The.Rebel.Princess.mp4",
            "The Rebel Princess 2021",
            1,
            1,
            "2021",
            "",
            &[]
        ));
    }

    /// Reported right after the Witch leak: "To My Beloved Thief" (2026,
    /// `pm:post:9000018021`). Every row of its S01E01–E05 lists is the show's
    /// own file — including the `nunadrama.` / `LayarAsia.` / `OLD.` prefixes
    /// and the `2025` some rips carry on a 2026 entry — so the tightening above
    /// must leave all of them in place. Filenames and episode numbers are
    /// verbatim from the live answer.
    #[test]
    fn series_keeps_every_row_of_its_own_show() {
        for (file, episode) in [
            ("nunadrama.To.My.Beloved.Thief.E01.1080p.mp4", 1),
            ("To.My.Beloved.Thief.EP1.1080p.MalaySub.OMG.mp4", 1),
            ("To.My.Beloved.Thief.EP1.MalaySub.mp4", 1),
            ("OLD.E01.To.My.Beloved.Thief.2025.mp4", 1),
            ("To.My.Beloved.Thief.E01.NF.1080p.x265.mkv", 1),
            ("To.My.Beloved.Thief.S01E01.NF.x264.540p.mkv", 1),
            ("To.My.Beloved.Thief.S01E02.x265.720p.NF.WEB.DL.MrHulk.mkv", 2),
            ("LayarAsia.To.My.Beloved.Thief.E03.720p.mp4", 3),
            ("OLD.E03.To.My.Beloved.Thief.2026.mp4", 3),
            ("LayarAsia.To.My.Beloved.Thief.E05.360p.mp4", 5),
            ("To.My.Beloved.Thief.E05.NF.540p.x265.mkv", 5),
        ] {
            assert!(
                series_file_matches_title(
                    file,
                    "To My Beloved Thief 2026",
                    1,
                    episode,
                    "2026",
                    "",
                    &[]
                ),
                "{file} is an episode of To My Beloved Thief and must stay in the list"
            );
        }
    }

    /// Verbatim port of the `MUST NOT REGRESS` cases in
    /// `legacy/php/test_kudrat_series.php` (which backend.php:9347 names as this
    /// guard's regression test). `series_year` is the FIRST year of the entry
    /// title, exactly as the PHP derives `$postYear`.
    ///
    /// Do NOT relax: "The Glory" (2022, Netflix Korean) and "Glory" (2025) are
    /// different shows. The year guard exists to keep them apart, and the
    /// single-word foreign-indicator guard must NOT run for a leading-"The"
    /// title or the whole Korean/NF catalogue of The Glory disappears.
    #[test]
    fn series_glory_cases_from_php_regression_suite() {
        // (title, year, file, caption, season, episode, expect)
        let cases: [(&str, &str, &str, &str, i32, i32, bool); 11] = [
            (
                "Kudrat 1968: High Council 2026 • TvSeries",
                "1968",
                "Kudrat.1968.High.Council.2026.EP1.MalaySub.mp4",
                "Kudrat 1968: High Council (2026) EP1 MalaySub",
                1,
                1,
                true,
            ),
            (
                "Kudrat 1968: High Council 2026 • TvSeries",
                "1968",
                "KUDRAT.1968.High.Council.2026.EP1.kakifilem.mp4",
                "KUDRAT (1968) High Council (2026) EP1 kakifilem",
                1,
                1,
                true,
            ),
            (
                "Kudrat 1968: High Council 2026 • TvSeries",
                "1968",
                "FNP22.Kudrat.1968.High.Council.2026.EP01.Subs.Malay.mp4",
                "KUDRAT 1968: HIGH COUNCIL (2026) MALAY #EP01",
                1,
                1,
                true,
            ),
            (
                "Kudrat 1968: High Council 2026 • TvSeries",
                "1968",
                "Kudrat.1968.High.Council.S01E01.Teguh.1080p.NF.WEB.DL.AAC2.0.H.264.mkv",
                "Kudrat.1968.High.Council.S01E01.Teguh.1080p.NF.WEB-DL.AAC2.0.H.264-snomyuoua",
                1,
                1,
                true,
            ),
            (
                "Kudrat 1968: High Council 2026 • TvSeries",
                "1968",
                "Kudrat.1968.High.Council.S01E04.Teguh.1080p.NF.WEB.DL.AAC2.0.H.264.mkv",
                "Kudrat.1968.High.Council.S01E04.Teguh.1080p.NF.WEB-DL.AAC2.0.H.264-snomyuoua",
                1,
                4,
                true,
            ),
            (
                "Kudrat 1968: High Council 2026 • TvSeries",
                "1968",
                "Kudrat.1968.High.Council.S01E01.1968.1080p.NF.WEB.DL.AAC2.0.H.264.mkv",
                "Kudrat.1968.High.Council.S01E01.1968.1080p.NF.WEB-DL.AAC2.0.H.264-snomyuoua",
                1,
                1,
                true,
            ),
            (
                "Kudrat 1968: High Council 2026 • TvSeries",
                "1968",
                "KUDRAT.1968.High.Council.2026.EP2.kakifilem.mp4",
                "KUDRAT (1968) High Council (2026) EP2 kakifilem",
                1,
                1,
                false,
            ),
            (
                "Kudrat 1968: High Council • TvSeries",
                "1968",
                "Kudrat.1968.High.Council.2026.EP1.MalaySub.mp4",
                "Kudrat 1968: High Council EP1 MalaySub",
                1,
                1,
                true,
            ),
            (
                "The Glory 2022 • TvSeries",
                "2022",
                "The.Glory.S01E16.Hindi.English.Korean.480p.WEB.DL.ESubs.BollyFlix.mkv",
                "The Glory S01E16 Hindi English Korean 480p WEB-DL ESubs BollyFlix",
                1,
                16,
                true,
            ),
            (
                "The Glory 2022 • TvSeries",
                "2022",
                "The.Glory.2022.S01E16.1080p.WEB.DL.Hindi.Korean.AAC.mkv",
                "The Glory 2022 S01E16 1080p WEB-DL Hindi Korean AAC",
                1,
                16,
                true,
            ),
            (
                "Glory 2025 • TvSeries",
                "2025",
                "The.Glory.S01E16.Hindi.English.Korean.480p.WEB.DL.ESubs.BollyFlix.mkv",
                "The Glory S01E16 Hindi English Korean 480p WEB-DL ESubs BollyFlix",
                1,
                16,
                false,
            ),
        ];
        for (i, (title, year, file, caption, s, e, expect)) in cases.iter().enumerate() {
            assert_eq!(
                series_file_matches_title(file, title, *s, *e, year, caption, &[]),
                *expect,
                "php case {} {}  S{}E{}\n  series: {}\n  file:   {}",
                i + 1,
                if *expect { "expect=match" } else { "expect=no-match" },
                s,
                e,
                title,
                file
            );
        }
    }

    #[test]
    fn series_rejects_unrelated_year() {
        assert!(!series_file_matches_title(
            "Some.Other.Show.2015.S01E01.1080p.mkv",
            "Kudrat 1968: High Council 2026",
            1,
            1,
            "2026",
            "",
            &[]
        ));
    }

    /* trap 6: sibling works -------------------------------------------------- */

    #[test]
    fn series_rejects_kind_markers_the_entry_lacks() {
        let title = "One Piece";
        for file in [
            "LEGO.ONE.PIECE.S01E01.1080p.mkv",
            "One.Piece.Movie.S01E01.1080p.mkv",
            "One.Piece.OVA.S01E01.1080p.mkv",
            "One.Piece.Live.Action.1999.S01E01.1080p.mkv",
        ] {
            assert!(
                !series_file_matches_title(file, title, 1, 1, "1999", "", &[]),
                "{file} must not match the anime entry"
            );
        }
        // The entry that IS that work keeps its files.
        assert!(series_file_matches_title(
            "One.Piece.Film.Strong.World.2009.S01E01.1080p.mkv",
            "One Piece Film: Strong World",
            1,
            1,
            "2009",
            "",
            &[]
        ));
    }

    /* trap 7: a quality tag is not an episode subtitle ------------------------ */

    #[test]
    fn episode_subtitle_ignores_uploader_and_quality_tails() {
        // 450p is not in the enumerated resolution list, and the uploader sits
        // exactly where the subtitle would be.
        assert_eq!(
            filename_episode_subtitle("Wild.Romance.E01.450p.HANerl.re.encoded.to.MQ.avi", "Wild Romance"),
            ""
        );
        assert!(episode_subtitle_matches(
            "Wild.Romance.E01.450p.HANerl.re.encoded.to.MQ.avi",
            "wild romance",
            &names(&["The playball that no one wanted"])
        ));
        // E16.END -> one-word tail -> inconclusive.
        assert!(episode_subtitle_matches(
            "Wild.Romance.E16.END.450p.mkv",
            "wild romance",
            &names(&["Uncaught third strike"])
        ));
    }

    #[test]
    fn episode_subtitle_rejects_another_shows_episode() {
        // REVISED 2026-11-06 — the guard is now "names a DIFFERENT KNOWN episode
        // of this show", not "does not match the one name I was given".
        //
        // The old expectation rejected this file, and the file is right: it IS
        // One Piece S01E01 ("Romance Dawn"); the single verified name belonged to
        // another episode. That same over-strictness dropped all 50 files of
        // Disney's "Oswald", where theTVDB numbers 1..52 two-per-file and the
        // uploader uses its own segment titles, so every episode of the show
        // answered "No Streams Found" while the files were all there.
        assert!(episode_subtitle_matches(
            "One.Piece.S01E01.Romance.Dawn.1080p.mkv",
            "one piece",
            &names(&["Uncaught Third Strike"])
        ));
        assert!(series_file_matches_title(
            "One.Piece.S01E01.Romance.Dawn.1080p.mkv",
            "One Piece",
            1,
            1,
            "",
            "",
            &names(&["Uncaught Third Strike"])
        ));

        // The cross-show protection that actually carries the weight is the
        // SERIES TITLE, and it still holds: this file's prefix is another show.
        assert!(!series_file_matches_title(
            "Im.Not.Afraid.S01E01.The.Witch.1080p.NF.WEB.DL.mkv",
            "The Witch",
            1,
            1,
            "2025",
            "",
            &names(&["The Witch"])
        ));

        // And the name guard still rejects a file that names a DIFFERENT KNOWN
        // episode of the SAME show — the sibling evidence is what makes that
        // decidable instead of guessed.
        assert!(!episode_subtitle_matches_with_siblings(
            "One.Piece.S01E01.Uncaught.Third.Strike.1080p.mkv",
            "one piece",
            &names(&["Romance Dawn"]),
            &names(&["Uncaught Third Strike"])
        ));
        // A tail that matches NOTHING known is content, not evidence.
        assert!(episode_subtitle_matches_with_siblings(
            "One.Piece.S01E01.Some.Other.Cut.1080p.mkv",
            "one piece",
            &names(&["Romance Dawn"]),
            &names(&["Uncaught Third Strike"])
        ));
    }

    /* trap 2: a codec suffix is not a split part ----------------------------- */

    #[test]
    fn split_part_ignores_codec_suffix() {
        for file in [
            "Movie.Name.AAC2.0.H.264.mkv",
            "Movie.Name.AAC2.0.H.265.mkv",
            "Movie.Name.2019.1080p.BluRay.x264.mkv",
        ] {
            let info = extract_split_part_info(file, "");
            assert!(!info.is_part, "{file} must not be a split part");
        }
    }

    /* trap 3: a lone "Part N" is a title ------------------------------------ */

    #[test]
    fn lone_part_title_is_not_flagged() {
        let mut alone = vec![FileItem {
            title: "Detective.Conan.Episode.100.Malay.Dub.Special.Episode.Part.1".into(),
            ..Default::default()
        }];
        group_split_parts(&mut alone);
        assert!(!alone[0].is_split_part);
    }

    #[test]
    fn real_split_siblings_are_flagged_and_ordered() {
        let mut files = vec![
            FileItem { title: "The.Movie.2020.Part.2.mkv".into(), ..Default::default() },
            FileItem { title: "The.Movie.2020.Part.1.mkv".into(), ..Default::default() },
            FileItem { title: "Unrelated.Part.1.mkv".into(), ..Default::default() },
        ];
        group_split_parts(&mut files);
        let movie: Vec<&FileItem> = files.iter().filter(|f| f.title.starts_with("The.Movie")).collect();
        assert_eq!(movie.len(), 2);
        assert!(movie.iter().all(|f| f.is_split_part));
        assert_eq!(movie[0].part_num, 1, "parts must be ordered ascending");
        assert_eq!(movie[1].part_num, 2);
        assert!(
            !files.iter().find(|f| f.title.starts_with("Unrelated")).unwrap().is_split_part
        );
    }

    #[test]
    fn caption_can_declare_the_total() {
        let info = extract_split_part_info("Movie.Part.1.mkv", "Movie 1080p Part 1 of 2");
        assert!(info.is_part);
        assert_eq!(info.part_num, 1);
        assert_eq!(info.total_parts, 2);
    }

    /* classification --------------------------------------------------------- */

    #[test]
    fn classifies_common_episode_shapes() {
        assert_eq!(classify_season_episode("Show.S02E05.1080p.mkv", 0, 0, ""), (2, 5, 0));
        assert_eq!(classify_season_episode("Show.1x05.720p.mkv", 0, 0, ""), (1, 5, 0));
        assert_eq!(classify_season_episode("Show Season 3 EP07.mkv", 0, 0, ""), (3, 7, 0));
        assert_eq!(classify_season_episode("Show.EP12.mkv", 0, 0, ""), (1, 12, 0));
        // "E01E02" with no separator is not a range in PHP or Kotlin either — the
        // range form needs a separator, so only the first episode is classified.
        // Asserted so a future "improvement" here is a deliberate, visible change
        // rather than a silent divergence from the twins.
        assert_eq!(classify_season_episode("Show.S01E01E02.mkv", 0, 0, ""), (1, 1, 0));
    }

    #[test]
    fn series_file_detection() {
        assert!(is_series_file("Show.S01E01.1080p.mkv", ""));
        assert!(is_series_file("Show.EP05.mkv", ""));
        assert!(!is_series_file("The.Boy.2026.1080p.mkv", ""));
    }

    #[test]
    fn episode_range_matches_targets_inside_it() {
        let parsed = classify_season_episode("Show.S01E01-E04.mkv", 0, 0, "");
        assert_eq!(parsed, (1, 1, 4));
        assert!(file_matches_episode(parsed, 1, 3));
        assert!(!file_matches_episode(parsed, 1, 5));
    }

    /* filters ---------------------------------------------------------------- */

    #[test]
    fn filters_hide_unplayable_and_split_parts() {
        let mut files = vec![
            FileItem { title: "Movie.2020.1080p.mkv".into(), file_size: 2_000_000_000, ..Default::default() },
            FileItem { title: "Movie.2020.1080p.zip".into(), file_size: 10, ..Default::default() },
            FileItem { title: "Movie.2020.1080p.mkv.001".into(), file_size: 10, ..Default::default() },
        ];
        let cfg = StreamFilterConfig::default();
        let kept = apply_stream_filters(&files, &cfg);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].0, 0);
        assert_eq!(kept[0].1, "1080p");

        files[0].is_split_part = true;
        let kept = apply_stream_filters(&files, &cfg);
        assert!(kept.is_empty(), "exclude_part_title defaults to ON");
    }

    /* web.dl / webdl are WEB-DL, not WEBRip ---------------------------------- */

    #[test]
    fn web_dl_survives_every_scene_separator() {
        // `Obsession.2026.2160p.iT.WEB.DL.UNRATED.DV.HDR10.MULTiBen.The.Men.mp4`
        // is a WEB-DL release, but the pattern only accepted `web-dl`, so `WEB.DL`
        // fell through to the `web` rule and the pill said WEBRip — the wrong
        // source on screen AND 50M instead of 60M in `stream_sort_score`.
        for name in [
            "Obsession.2026.2160p.iT.WEB.DL.UNRATED.DV.HDR10.MULTiBen.The.Men.mp4",
            "Obsession.2026.2160p.iT.WEB-DL.DV.HDR.mp4",
            "Obsession.2026.2160p.iT.WEBDL.DV.HDR.mp4",
            "Obsession.2026.2160p.iT.WEB_DL.DV.HDR.mp4",
        ] {
            assert_eq!(extract_media_tags(name, "").source, "WEB-DL", "{name}");
        }
        // A genuine WEBRip must stay a WEBRip, and the bare `web` fallback too.
        assert_eq!(
            extract_media_tags("Obsession.2026.1080p.WEBRip.x264.mp4", "").source,
            "WEBRip"
        );
        assert_eq!(
            extract_media_tags("Obsession.2026.1080p.WEB.x264.mp4", "").source,
            "WEBRip"
        );
    }

    /* the part index can live in the CAPTION only ---------------------------- */

    const BLURAY: &str = "Obsession.2026.2160p.4K.HDR10.DV.BluRay.x265.10bit.HEVC.Org.AMZN.mkv";

    fn obsession_caption(part: u32) -> String {
        format!(
            "Obsession (2026) 2160p 4K HDR10 DV BluRay x265 10bit HEVC [Org AMZN Hindi DDP 5.1 \
             640Kbps  English TrueHD Atmos 7.1] ESub  J3RICO.part00{part}.mkv\n\
             [KP] from HM DRIVE [DUMP] 3779026928  "
        )
    }

    #[test]
    fn tags_come_from_the_title_and_the_caption() {
        let tags = extract_media_tags(BLURAY, &obsession_caption(5));
        assert_eq!(tags.resolution, "4K");
        // BluRay outranks AMZN, and both are shown (`platform` + `source`).
        assert_eq!(tags.source, "BluRay");
        assert_eq!(tags.platform, "AMZN");
        assert_eq!(tags.codec, "HEVC");
        // The audio only exists in the caption: `Atmos` is preferred over DDP 5.1.
        assert_eq!(tags.audio, "Atmos");
        assert_eq!(
            tags.visual,
            vec!["HDR".to_string(), "DV".to_string(), "10bit".to_string()]
        );
    }

    #[test]
    fn part_number_in_the_caption_is_the_only_ordinal() {
        // All eight rows share this exact title; only the caption keeps the index.
        assert!(!extract_split_part_info(BLURAY, "").is_part);
        let five = extract_split_part_info(BLURAY, &obsession_caption(5));
        let eight = extract_split_part_info(BLURAY, &obsession_caption(8));
        assert!(five.is_part && eight.is_part);
        assert_eq!((five.part_num, eight.part_num), (5, 8));
        assert_eq!(five.base_key, eight.base_key, "siblings must group together");
        assert_eq!(five.clean_base, BLURAY);
    }

    #[test]
    fn group_split_parts_uses_the_caption_when_the_name_has_no_index() {
        let mut files = vec![
            FileItem {
                title: BLURAY.to_string(),
                caption: obsession_caption(5),
                ..Default::default()
            },
            FileItem {
                title: BLURAY.to_string(),
                caption: obsession_caption(8),
                ..Default::default()
            },
        ];
        group_split_parts(&mut files);
        assert!(files.iter().all(|f| f.is_split_part));
        let mut parts: Vec<i32> = files.iter().map(|f| f.part_num).collect();
        parts.sort_unstable();
        assert_eq!(parts, vec![5, 8]);
    }

    #[test]
    fn a_caption_part_word_without_a_number_is_not_a_part() {
        // "Dune Part Two", "in 4 parts" — the word alone is never evidence.
        assert!(!extract_split_part_info("Dune.Part.Two.2024.1080p.mkv", "Dune Part Two 2024")
            .is_part);
        assert!(!extract_split_part_info("Some.Movie.2024.mkv", "Watch it in four parts").is_part);
        // And a real part index in the NAME still works exactly as before.
        let named = extract_split_part_info("Movie.2024.part002.mkv", "");
        assert!(named.is_part && named.part_num == 2);
    }

    #[test]
    fn duration_keeps_an_ordinal_that_came_from_the_row() {
        // The origin's `part` column (or the caption) may already know the index.
        // Duration must add the "this is a chunk" evidence without erasing it.
        let mut files = chunks(&[1361, 1323, 1323]);
        files[0].is_split_part = true;
        files[0].part_num = 1;
        files[0].clean_base = AOC.to_lowercase();
        detect_parts_by_duration(&mut files);
        assert!(files.iter().all(|f| f.is_split_part));
        assert_eq!(files[0].part_num, 1, "a known ordinal is never overwritten");
        assert_eq!(files[1].part_num, 0, "an unknown one is never invented");
        assert!(files.iter().all(|f| f.total_parts == 3));
    }

    #[test]
    fn duration_flags_the_four_obsession_multiben_chunks() {
        // Real numbers from `/api/resolve-shortcode` (2026-10-25):
        //   rUaIMd 1185 s / 4.19 GB   ySYWSG 1162 s / 4.18 GB
        //   28YMMS 1167 s / 4.18 GB   2p1Acd  710 s / 2.49 GB
        // Four files, one name, 4224 s (70.4 min) together, no `part` in the name.
        let name = "Obsession.2026.2160p.iT.WEB.DL.UNRATED.DV.HDR10.MULTiBen.The.Men.mp4";
        let mut files: Vec<FileItem> = [1185u32, 1167, 1162, 710]
            .iter()
            .map(|d| FileItem {
                title: name.to_string(),
                duration: *d,
                ..Default::default()
            })
            .collect();
        detect_parts_by_duration(&mut files);
        assert!(files.iter().all(|f| f.is_split_part), "all four must be flagged");
        assert!(files.iter().all(|f| f.total_parts == 4));
    }

    #[test]
    fn unrated_is_an_edition() {
        let name = "Obsession.2026.2160p.iT.WEB.DL.UNRATED.DV.HDR10.MULTiBen.The.Men.mp4";
        let tags = extract_media_tags(name, "");
        assert_eq!(tags.edition, "Unrated");
        // The rest of that name, as measured: 4K / DV+HDR10 / no codec or audio
        // token in the name at all (`iT` is iTunes, `MULTi` is multi-audio — both
        // are NOT in the tag tables and must not be guessed).
        assert_eq!(tags.resolution, "4K");
        assert_eq!(tags.visual, vec!["HDR".to_string(), "DV".to_string()]);
        assert_eq!(tags.codec, "");
        assert_eq!(tags.audio, "");
    }

    /* duration-based part detection ------------------------------------------ */

    const AOC: &str = "F1.The.Movie.2025.Hybrid.2160p.WEB.DL.DV.HDR.DDP5.1.Atmos.H265.AOC.mkv";

    fn chunks(durations: &[u32]) -> Vec<FileItem> {
        durations
            .iter()
            .map(|d| FileItem {
                title: AOC.to_string(),
                file_size: 4_160_000_000,
                duration: *d,
                ..Default::default()
            })
            .collect()
    }

    #[test]
    fn duration_flags_the_seven_anonymous_f1_chunks() {
        // Measured from the origin (`tg_file_new.part` + `/resolve-shortcode`):
        // seven files, one name, 9332 s together — a 2h35m film in ~22 min chunks.
        let mut files = chunks(&[1361, 1323, 1323, 1401, 1289, 1313, 1322]);
        detect_parts_by_duration(&mut files);
        assert!(files.iter().all(|f| f.is_split_part), "every chunk must be flagged");
        assert!(files.iter().all(|f| f.total_parts == 7));
        assert!(files.iter().all(|f| f.part_num == 0), "duration cannot prove the ordinal");
        assert_eq!(files[0].clean_base, AOC.to_lowercase());
    }

    #[test]
    fn duration_leaves_a_duplicated_whole_file_alone() {
        // Two copies of the same 2h45m film: max ≈ sum/2, so the cap rejects them.
        let mut files = chunks(&[9948, 9947]);
        detect_parts_by_duration(&mut files);
        assert!(files.iter().all(|f| !f.is_split_part));
    }

    #[test]
    fn duration_never_flags_a_lone_file() {
        // A single 15-min clip that happens to be short is NOT a part.
        let mut files = chunks(&[900]);
        detect_parts_by_duration(&mut files);
        assert!(!files[0].is_split_part);
    }

    #[test]
    fn duration_requires_every_chunk_to_be_known() {
        // One unknown duration = no evidence; silence beats a wrong label.
        let mut files = chunks(&[1361, 0, 1323]);
        detect_parts_by_duration(&mut files);
        assert!(files.iter().all(|f| !f.is_split_part));
    }

    #[test]
    fn duration_ignores_unrelated_short_clips() {
        // 3 × 8 min = 24 min: named the same, but they do not add up to a feature.
        let mut files = chunks(&[480, 480, 480]);
        detect_parts_by_duration(&mut files);
        assert!(files.iter().all(|f| !f.is_split_part));
    }

    #[test]
    fn duration_detection_feeds_the_default_exclude_filter() {
        // The whole point: `exclude_part_title` (ON by default) must act on the
        // chunks once duration proves them, and must leave a whole file alone.
        let cfg = StreamFilterConfig::default();
        let mut parts = chunks(&[1361, 1323, 1323, 1401, 1289, 1313, 1322]);
        detect_parts_by_duration(&mut parts);
        assert!(apply_stream_filters(&parts, &cfg).is_empty());

        let mut whole = chunks(&[9332, 9333]);
        detect_parts_by_duration(&mut whole);
        assert_eq!(apply_stream_filters(&whole, &cfg).len(), 2);

        // Unchecking `exclude_part_title` gives the seven straight back: the flag
        // is the ONLY thing that removed them. The Obsession chunks are a
        // different story — they never reach this filter, because they are not in
        // the post's file set and their 2026 name is a `Near` year against a 2025
        // post, so `year_keep_mask()` drops them first.
        let off = StreamFilterConfig {
            exclude_part_title: false,
            ..StreamFilterConfig::default()
        };
        assert_eq!(apply_stream_filters(&parts, &off).len(), 7);
    }

    #[test]
    fn required_keywords_can_hide_everything() {
        // This is the "Streams Filtered Out" case: a leftover `required_keywords`
        // hides every non-matching release and looks exactly like an outage.
        let files = vec![FileItem {
            title: "Movie.2020.1080p.WEB-DL.mkv".into(),
            file_size: 1,
            ..Default::default()
        }];
        let cfg = StreamFilterConfig::from_json(&serde_json::json!({
            "required_keywords": "hindi, hin"
        }));
        assert!(apply_stream_filters(&files, &cfg).is_empty());
    }

    #[test]
    fn sort_score_prefers_higher_resolution() {
        let uhd = stream_sort_score("Movie.2020.2160p.WEB-DL.mkv", "", 1_000_000_000, "auto");
        let sd = stream_sort_score("Movie.2020.480p.WEBRip.mkv", "", 1_000_000_000, "auto");
        assert!(uhd > sd);
    }

    #[test]
    fn category_keys() {
        assert_eq!(categorize_resolution_key("Movie.2160p.mkv", ""), "4k");
        assert_eq!(categorize_resolution_key("Movie.480p.mkv", ""), "sd");
        assert_eq!(categorize_quality_key("Movie.WEB-DL.mkv", ""), "webdl");
        assert_eq!(categorize_quality_key("Movie.WEBRip.mkv", ""), "unknown");
    }

    #[test]
    fn decorations_and_entities() {
        assert_eq!(strip_title_decorations("Veil of Shadows 2026 • TvSeries"), "Veil of Shadows");
        assert_eq!(decode_html_entities("Tom & Jerry 'Show'"), "Tom & Jerry 'Show'");
        assert_eq!(norm_title_for_match("Avengers.Infinity.War.2018.2160p.mkv"), "avengers infinity war");
    }
}
