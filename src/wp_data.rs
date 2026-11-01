//! WordPress data layer for the stream route — the `wp_data` group in
//! [`port/COVERAGE.md`](../../COVERAGE.md) that the title matcher needs in
//! order to see any files at all.
//!
//! What `/stream/*`, `/catalog/*` and `/meta/*` need is here: the cache-first
//! AJAX fetch ([`fetch_stream_ajax`] = `fd_fetch_stream_ajax`), paged post files
//! ([`fetch_post_files_paged`] = `fd_fetch_post_files_paged`), the post list /
//! search, the trending keyword map ([`trending_keywords`] =
//! `fd_get_trending_keywords_map`) and the two identity lookups the non-`pm:`
//! ids need.
//!
//! ## The contract that is easy to get wrong
//!
//! `fd_fetch_stream_ajax('post_files')` does **not** hit `admin-ajax.php`. It
//! lowercases the action to a REST route by replacing `_` with `-`:
//!
//! ```text
//! post_files   ->  GET {WP_API_BASE}/post-files?post_id=…&limit=…&offset=…
//! search_files ->  GET {WP_API_BASE}/search-files?search=…&limit=…&offset=…
//! ```
//!
//! and then unwraps WordPress's envelope: `{success, data}` when `success` is
//! truthy, else `data`, else `items`, else the body itself. Post files live
//! under the unwrapped `files` key, not `items`, and the loop must honour
//! `has_more` **or** a full page — the endpoint returns newest-episode-first and
//! silently truncates a long series on a single call (65-episode post listed
//! only E2..E65 with one `limit=200` request).

use serde_json::Value;

use crate::config;
use crate::httpc;
use crate::log::log;

/// `fd_stream_ajax_ttl()`.
pub fn stream_ajax_ttl(action: &str) -> i64 {
    match action {
        "search" | "search_files" => 300,
        "posts" | "post_files" | "get_post" | "search_music" => 600,
        "categories" | "trending" => 3600,
        _ => 300,
    }
}

fn ajax_timeout(action: &str) -> u64 {
    if matches!(action, "search" | "search_files") {
        6
    } else {
        8
    }
}

/// A probe is SPECULATIVE — it exists to fill in a list that is already thin.
///
/// The episode ladder fires every spelling of an episode number at once
/// (`EP5`, `E5`, `EP05`, `E05`, `S01E05`, each with and without the year), so one
/// unhealthy origin costs the full timeout for the whole batch. Measured while
/// browsing: 13 probes, all 13 transport failures, and the list then came back
/// empty anyway. A speculative probe that has not answered in 3 s is not going to
/// rescue the list; letting it run to 6 s only delays the answer the viewer gets.
fn probe_timeout() -> u64 {
    3
}

/// `post_files` -> `/post-files`.
fn rest_route(action: &str) -> String {
    format!(
        "{}/{}",
        config::wp_api_base().trim_end_matches('/'),
        action.replace('_', "-")
    )
}

/// WordPress envelope unwrap (see the module docs).
pub fn unwrap_data(v: &Value) -> Value {
    if v.get("success").and_then(|s| s.as_bool()).unwrap_or(false) {
        if let Some(d) = v.get("data") {
            return d.clone();
        }
    }
    if let Some(d) = v.get("data") {
        return d.clone();
    }
    if let Some(i) = v.get("items") {
        return i.clone();
    }
    v.clone()
}

fn query_add(mut url: String, params: &[(String, String)]) -> String {
    if params.is_empty() {
        return url;
    }
    let mut ser = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in params {
        ser.append_pair(k, v);
    }
    let q = ser.finish();
    url.push(if url.contains('?') { '&' } else { '?' });
    url.push_str(&q);
    url
}

/// `fd_fetch_stream_ajax()`'s `bot_id` rule: every action gets one EXCEPT
/// `trending`. The keyword table is global, so a per-bot `bot_id` would only
/// multiply the cache keys (and the origin calls) for an identical answer — the
/// PHP explicitly unsets it for that action.
pub fn stream_ajax_wants_bot(action: &str) -> bool {
    action != "trending"
}

/// How long a stale ajax body may still be served when a refresh fails. The old
/// code read the file "at any age" for this; bounded now.
const AJAX_STALE_GRACE: i64 = 21_600;

/// How long an EMPTY ajax payload may be cached — deliberately far shorter than
/// every [`stream_ajax_ttl`], so a transient empty answer self-heals in a minute
/// instead of pinning a catalog row for ten.
const EMPTY_PAYLOAD_TTL: i64 = 60;

/// `[]`, `{}`, or an object whose array members are all empty (`{"files":[]}`).
fn is_empty_payload(v: &Value) -> bool {
    match v {
        Value::Array(a) => a.is_empty(),
        Value::Object(m) => m.values().all(|x| match x {
            Value::Array(a) => a.is_empty(),
            Value::Null => true,
            _ => false,
        }),
        _ => false,
    }
}

/// Freshness of the PER-EPISODE `stream-files` search.
///
/// The PHP fetches that exact URL through `fd_cached_wp_json($url, 15)`
/// (backend.php:10185) — 15 seconds, not the 300 the generic
/// `fd_fetch_stream_ajax()` layer uses — so a freshly uploaded episode shows up
/// almost immediately. A longer TTL here means a new release can stay invisible
/// for minutes after the PHP would already list it.
///
/// KEPT AT THE PHP'S 15 s ON PURPOSE. Raising it was considered while fixing the
/// slow episode list and rejected on evidence: the cache key includes season AND
/// episode, so `…:1:4` and `…:1:5` are different entries — a longer TTL would NOT
/// have made episode-to-episode browsing any faster. It only covers a REPEAT view
/// of the same episode (player retries, back-navigation), and the price is that a
/// brand-new upload stays invisible for that much longer. The 30 s stall came from
/// retrying a spent timeout and consulting the stale copy last, both fixed in
/// [`crate::httpc::cached_wp_json`] — not from this number.
pub const PER_EPISODE_SEARCH_TTL: i64 = 15;

/// `fd_fetch_stream_ajax()`: cache + single-flight + stale fallback, `bot_id` added
/// when the caller did not supply one (the origin resolves files per bot, so a
/// missing `bot_id` changes the answer, not just the cache key).
pub async fn fetch_stream_ajax(action: &str, params: &[(String, String)]) -> Value {
    fetch_stream_ajax_timeout(action, params, None).await
}

/// [`fetch_stream_ajax()`] with an explicit timeout, for a SPECULATIVE caller that
/// must not hold the request open for the action's full budget — see
/// [`probe_timeout()`].
pub async fn fetch_stream_ajax_timeout(
    action: &str,
    params: &[(String, String)],
    timeout_secs: Option<u64>,
) -> Value {
    let mut params: Vec<(String, String)> = params.to_vec();
    if stream_ajax_wants_bot(action) && !params.iter().any(|(k, _)| k == "bot_id") {
        let bot = crate::session::get_bot_id();
        if !bot.is_empty() {
            params.push(("bot_id".to_string(), bot));
        }
    }
    let url = query_add(rest_route(action), &params);
    // Key on the action AND the full url: the same action resolves to different
    // answers per `bot_id`, season, episode and search term.
    let key = format!("{}|{}", action, url);
    let ttl = stream_ajax_ttl(action);

    let url_owned = url.clone();
    let action_owned = action.to_string();
    let body = crate::cache::fetch_cached("wp_ajax", &key, ttl, AJAX_STALE_GRACE, move || async move {
        // Same gate as `cached_wp_json`: once the origin has failed to answer, a
        // speculative probe is the LAST thing worth waiting on. Without this the
        // episode ladder fired 13 searches at a dead origin, every one burning its
        // full budget, and still returned an empty list.
        let down = httpc::wp_origin_down_secs();
        if down > 0 {
            log(
                "wp origin down - probe skipped",
                &serde_json::json!({"action": action_owned, "for_secs": down}),
            );
            return (Value::Null, false);
        }
        let body = httpc::get_contents(
            &url_owned,
            timeout_secs.unwrap_or_else(|| ajax_timeout(&action_owned)),
            [("Accept".to_string(), "application/json".to_string())]
                .into_iter()
                .collect(),
        )
        .await;
        let decoded = match body.and_then(|b| serde_json::from_str::<Value>(&b).ok()) {
            Some(v) => v,
            None => return (Value::Null, false),
        };
        let data = unwrap_data(&decoded);
        // Only a real payload is cacheable; an error envelope must not be pinned.
        let cacheable = data.is_object() || data.is_array();
        (data, cacheable)
    })
    .await;

    // An EMPTY payload is cached for at most a minute, never for the action's full
    // TTL. One origin hiccup used to pin an empty `posts`/`search_files` answer for
    // 600 s, and because `/catalog/*` itself refuses to cache an empty row, the
    // only symptom was a catalog that stayed empty for ten minutes and looked
    // "broken" with no error anywhere (measured twice while porting the PHP's
    // catalog rows: `series/top` and `pm_series_year` answered `{"metas":[]}`
    // while the origin returned 69 and 100 posts for the identical query).
    if !body.is_null() && is_empty_payload(&body) {
        crate::cache::put("wp_ajax", &key, &body, EMPTY_PAYLOAD_TTL, AJAX_STALE_GRACE);
    }
    if body.is_null() {
        if let Some(stale) = crate::cache::get_stale("wp_ajax", &key) {
            log(
                "wp ajax served stale",
                &serde_json::json!({"action": action}),
            );
            return (*stale).clone();
        }
        log(
            "wp ajax fetch empty",
            &serde_json::json!({"action": action, "had_stale": false}),
        );
    }
    body
}

/// One file row as the matcher and the stream builder consume it.
#[derive(Debug, Clone, Default)]
pub struct WpFile {
    pub short_code: String,
    pub title: String,
    pub caption: String,
    pub file_size: u64,
    pub mime: String,
    pub season_num: i32,
    pub episode_num: i32,
    /// `duration` in seconds, when the row provides it. `0` = not in the row, and
    /// the stream builder falls back to the resolve cache. A runtime is the only
    /// evidence that N identically-named files are the chunks of one release.
    pub duration: u32,
    /// `part` — the origin's own split-part index (`tg_file_new.part`). `0` = not
    /// a part / not populated. This is the ONE ordinal available without probing
    /// the media: the container carries none (`PrevUID`/`NextUID`, chapters and
    /// `DateUTC` are all absent on the real split uploads), and duration cannot
    /// prove order.
    pub part: i32,
    /// `thumbnail_url` — the poster a Telegram-FILE catalog row shows
    /// (backend.php:15829 `'poster' => $fThumb`). Empty for post files.
    pub thumbnail: String,
    /// `added_date` — Unix SECONDS for when the origin created this file row. A
    /// STRING on the wire (`"1792386106"`), a number on some rows, and absent on
    /// older ones.
    ///
    /// This is the only date the source actually stores, and the PHP used it
    /// verbatim as a Stremio episode's `released` (backend.php:16273 / 16322).
    /// The port dropped the field and stamped `date(...)` — i.e. the wall clock —
    /// instead, so every episode of every series reported "released just now"
    /// and the `/meta` body changed on each request (issue #10). `0` = absent.
    pub added_date: i64,
}

pub fn wp_file_from(v: &Value) -> Option<WpFile> {
    let short_code = v
        .get("short_code")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if short_code.is_empty() {
        return None;
    }
    let title = ["title", "file_name", "name"]
        .iter()
        .find_map(|k| v.get(*k).and_then(|x| x.as_str()))
        .unwrap_or("")
        .to_string();
    let caption = v
        .get("caption")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let file_size = v
        .get("file_size")
        .and_then(|x| x.as_u64().or_else(|| x.as_str().and_then(|s| s.parse().ok())))
        .unwrap_or(0);
    let mime = ["mime", "file_type", "mime_type"]
        .iter()
        .find_map(|k| v.get(*k).and_then(|x| x.as_str()))
        .unwrap_or("")
        .to_string();
    let thumbnail = v
        .get("thumbnail_url")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    // `added_date` is a STRING of Unix seconds on the real rows; `duration` is a
    // number on some rows and a string on others; `part` only appeared in
    // 2026-10 and is absent from most rows, so all three are optional.
    let added_date = v
        .get("added_date")
        .and_then(|x| {
            x.as_i64()
                .or_else(|| x.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
        })
        .filter(|n| *n > 0)
        .unwrap_or(0);
    let duration = v
        .get("duration")
        .and_then(|x| {
            x.as_u64()
                .or_else(|| x.as_i64().map(|n| n.max(0) as u64))
                .or_else(|| x.as_str().and_then(|s| s.trim().parse::<u64>().ok()))
        })
        .unwrap_or(0)
        .min(u32::MAX as u64) as u32;
    Some(WpFile {
        short_code,
        title,
        caption,
        file_size,
        mime,
        season_num: json_i32(v, "season_num"),
        episode_num: json_i32(v, "episode_num"),
        duration,
        part: json_i32(v, "part"),
        thumbnail,
        added_date,
    })
}

fn json_i32(v: &Value, key: &str) -> i32 {
    v.get(key)
        .and_then(|x| {
            x.as_i64()
                .or_else(|| x.as_str().and_then(|s| s.trim().parse().ok()))
        })
        .unwrap_or(0) as i32
}

/// `fd_fetch_post_files_paged()` — pages until a short page or `!has_more`.
pub async fn fetch_post_files_paged(
    post_id: &str,
    page_size: usize,
    max_files: usize,
    max_pages: usize,
) -> Vec<WpFile> {
    let page_size = page_size.clamp(1, 200);
    let mut all: Vec<WpFile> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut offset = 0usize;

    for _page in 0..max_pages.max(1) {
        let params = vec![
            ("post_id".to_string(), post_id.to_string()),
            ("limit".to_string(), page_size.to_string()),
            ("offset".to_string(), offset.to_string()),
        ];
        let res = fetch_stream_ajax("post_files", &params).await;
        // `post-files` answers `{files: […], has_more}`; the unwrapped `data` may
        // also be the bare array, so accept both shapes.
        let files: Vec<Value> = rows_of(&res, &["files", "items"]);
        if files.is_empty() {
            break;
        }
        let returned = files.len();
        let mut added = 0usize;
        for f in &files {
            let Some(item) = wp_file_from(f) else { continue };
            if !seen.insert(item.short_code.clone()) {
                continue;
            }
            all.push(item);
            added += 1;
            if all.len() >= max_files {
                return all;
            }
        }
        let has_more = res
            .get("has_more")
            .map(|v| v.as_bool().unwrap_or_else(|| v.as_i64().unwrap_or(0) != 0))
            .unwrap_or(false);
        if !has_more && returned < page_size {
            break;
        }
        if added == 0 {
            break;
        }
        offset += page_size;
    }
    all
}

/// `GET {WP_API_BASE}/posts` — the catalog list (`media_type`, `category`,
/// `genre`, `year`, `limit`, `offset`). Answers a bare array.
pub async fn posts(params: &[(String, String)]) -> Vec<Value> {
    rows_of(
        &fetch_stream_ajax("posts", params).await,
        &["posts", "items", "files"],
    )
}

/// `GET {WP_API_BASE}/search` — WordPress post search. Bare array as well.
pub async fn search_posts(params: &[(String, String)]) -> Vec<Value> {
    rows_of(
        &fetch_stream_ajax("search", params).await,
        &["posts", "items", "files"],
    )
}

/* ── trending keywords (backend.php:11655 / 11793) ───────────────────────── */

/// No keyword blocklist here on purpose: the WordPress REST endpoint is the
/// single place that decides which keywords are acceptable (it already drops
/// NSFW/filler terms), so an app-side list only duplicated that policy and
/// produced false negatives — it hid legitimate keywords such as `melayu`, and
/// `Up`-style short titles. The only thing dropped locally is an empty token.
pub fn is_topkeyword_valid(keyword: &str) -> bool {
    !keyword.trim().is_empty()
}

/// Pure parser for the `/trending` payload.
///
/// The origin answers `{success:true, data:[{keyword, hits, total_found}, …]}`
/// and [`unwrap_data`] has already unwrapped it by the time this runs, but a
/// bare array of strings and the `keywords`/`items` object shapes are accepted
/// too. Order is the ORIGIN's (relevance), not hits-descending — the PHP's map
/// keeps that order as well.
pub fn trending_keywords_from(v: &Value) -> Vec<(String, u64)> {
    let rows = rows_of(v, &["keywords", "items", "data"]);
    let mut out: Vec<(String, u64)> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    for row in rows {
        let kw = match row.as_str() {
            Some(s) => s.trim().to_string(),
            None => ["keyword", "query", "term"]
                .iter()
                .find_map(|k| row.get(*k).and_then(|x| x.as_str()))
                .unwrap_or("")
                .trim()
                .to_string(),
        };
        if !is_topkeyword_valid(&kw) {
            continue;
        }
        let hits = row
            .get("hits")
            .map(|h| {
                h.as_u64()
                    .or_else(|| h.as_str().and_then(|s| s.trim().parse().ok()))
                    .unwrap_or(0)
            })
            .unwrap_or(0);
        if !seen.insert(kw.to_lowercase()) {
            continue;
        }
        out.push((kw, hits));
    }
    out
}

/// `fd_get_trending_keywords_map()` — the trending keywords with their hit
/// counts, in the origin's order. No keyword is dropped (empty tokens aside):
/// content filtering belongs to the REST endpoint, not to the app.
///
/// Two caches, exactly like the PHP: the AJAX response itself is disk-cached for
/// an hour (`fd_stream_ajax_ttl('trending')`), and this list is memoised in
/// process for 10 minutes. The PHP additionally caps its map to the top **5**
/// keywords because each one becomes a separate `pm_topkw_*` catalog row; here
/// the keywords are merged into two type rows, so dropping the tail would make
/// the rarer type (movies) collapse to nothing. The list is therefore returned
/// whole and the row builder stops as soon as it is full.
pub async fn trending_keywords() -> Vec<(String, u64)> {
    /// TTL of the memoised keyword list.
    const TTL: i64 = 600;
    /// How long the last good list may still be served when the origin is empty.
    const GRACE: i64 = 21_600;

    fn decode(v: &Value) -> Option<Vec<(String, u64)>> {
        let arr = v.as_array()?;
        let mut out = Vec::with_capacity(arr.len());
        for item in arr {
            let pair = item.as_array()?;
            let k = pair.first()?.as_str()?.to_string();
            let n = pair.get(1).and_then(|x| x.as_u64()).unwrap_or(0);
            out.push((k, n));
        }
        (!out.is_empty()).then_some(out)
    }

    if let Some(hit) = crate::cache::get("trending", "keywords") {
        if let Some(list) = decode(&hit) {
            return list;
        }
    }
    // `fetch_stream_ajax` deliberately does NOT send `bot_id` for `trending`
    // (see its docs): the keyword table is global, and a per-bot cache key would
    // multiply the origin calls for no gain.
    let res = fetch_stream_ajax("trending", &[("limit".to_string(), "15".to_string())]).await;
    let list = trending_keywords_from(&res);
    if list.is_empty() {
        // Origin down / stale beyond the grace window: keep serving the last list.
        if let Some(stale) = crate::cache::get_stale("trending", "keywords") {
            if let Some(prev) = decode(&stale) {
                return prev;
            }
        }
        log(
            "trending keywords empty",
            &serde_json::json!({"shape": if res.is_array() { "array" } else { "object" }}),
        );
        return list;
    }
    crate::cache::put("trending", "keywords", &serde_json::json!(list), TTL, GRACE);
    list
}

/// `GET {WP_API_BASE}/lookup-id?prefix=imdb&title=…&year=…` — a Manticore title
/// match over `media_ids_idx.title`.
///
/// The title MUST be stripped of decorations first (bullet separator, year,
/// trailing type word) or the literal phrase match can never hit — the exact bug
/// documented for `fd_find_imdb_id_for_title()` ("Veil of Shadows 2026 • TvSeries").
pub async fn lookup_imdb_by_title(title: &str, year: &str) -> String {
    let clean = crate::matcher::strip_title_decorations(title);
    if clean.is_empty() {
        return String::new();
    }
    let mut params = vec![
        ("prefix".to_string(), "imdb".to_string()),
        ("title".to_string(), clean),
    ];
    if !year.is_empty() {
        params.push(("year".to_string(), year.to_string()));
    }
    let url = query_add(rest_route("lookup_id"), &params);
    let row = unwrap_data(&httpc::http_json(&url, &[], "GET", 8).await);
    let id = row
        .get("imdb_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if !id.is_empty() {
        return id;
    }
    row.get("imdb")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// `GET {WP_API_BASE}/lookup-id?prefix=…&id=…` — the local `media_ids_idx` row.
///
/// Cached, INCLUDING the miss. This sits on the critical path of every external
/// `/stream` request (it is what could give the route a `post_id`, i.e. an exact
/// post file set instead of a title scan), it is a pure index read, and the
/// origin answers 404 for most of the ids that reach it — measured on a real
/// box: `mal:54492` 404, `tt:0944947` 404. Un-cached, every episode of a series
/// paid that round-trip (83-350 ms) for an answer that had not changed.
///
/// 15 min: long enough to make an episode list free, short enough that a freshly
/// indexed id is not invisible for a browsing session.
pub async fn lookup_id(prefix: &str, id: &str) -> Value {
    let key = format!("{}:{}", prefix, id);
    if let Some(hit) = crate::cache::get("lookup_id", &key) {
        return (*hit).clone();
    }
    let url = query_add(
        rest_route("lookup_id"),
        &[
            ("prefix".to_string(), prefix.to_string()),
            ("id".to_string(), id.to_string()),
        ],
    );
    let v = unwrap_data(&httpc::http_json(&url, &[], "GET", 8).await);
    crate::cache::put("lookup_id", &key, &v, 900, 0);
    v
}

/* ── media_ids_idx write-back (`fd_media_ids_upsert`, backend.php:2801) ──── */

/// How long a discovered identity is remembered locally — the TTL of the PHP's
/// `ext_meta_<md5(id)>.json` (backend.php:3065: `(time() - filemtime) < 86400`).
const EXT_META_TTL: i64 = 86_400;

/// PHP `crc32()` — CRC-32/IEEE, the same polynomial `hashlib.crc32` uses. The
/// pseudo document id of a CUSTOM prefix is `(int) sprintf('%u', crc32(...))`
/// (backend.php:3380), so the port must produce the identical number or the two
/// lineages would index the same id under two different rows.
fn crc32_ieee(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

/// The `post_id` an upserted `media_ids_idx` row is keyed on.
///
/// `media_ids_idx` does not need a real WordPress post: the PHP invents a pseudo
/// id per id family — `9000000000 + imdb % 1e9` (backend.php:3144),
/// `8000000000 + tmdb` (3247), `7000000000 + kitsu` (3319), `crc32(prefix:value)`
/// (3380) — and a `pm:post:<pseudo>` id is exactly what those rows are read back
/// as later (`handle_post_stream` step 2b).
pub fn pseudo_post_id(prefix: &str, value: &str) -> i64 {
    match prefix {
        "imdb" | "tt" => {
            let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
            let n: i64 = digits.parse().unwrap_or(0);
            9_000_000_000 + (n % 1_000_000_000)
        }
        "tmdb" => 8_000_000_000 + value.trim().parse::<i64>().unwrap_or(0),
        "kitsu" => 7_000_000_000 + value.trim().parse::<i64>().unwrap_or(0),
        _ => crc32_ieee(format!("{}:{}", prefix, value).as_bytes()) as i64,
    }
}

/// The `(prefix, value)` pairs one id maps to: its OWN prefix plus the IMDb id
/// the resolver learned (backend.php:3145, 3243-3246, 3315-3318, 3376-3379).
///
/// An `imdb`/`tt` id is written under BOTH spellings, because `/lookup-id` is
/// asked with either one depending on the caller — the `/stream` path asks
/// `imdb`, the `/meta` path asks `tt` first and `imdb` second.
pub fn media_id_pairs(prefix: &str, value: &str, imdb_id: &str) -> Vec<(String, String)> {
    fn push(out: &mut Vec<(String, String)>, p: &str, v: &str) {
        let p = p.trim().to_lowercase();
        let v = v.trim();
        if p.is_empty() || v.is_empty() {
            return;
        }
        if out.iter().any(|(a, _)| *a == p) {
            return;
        }
        out.push((p, v.to_string()));
    }
    let mut out: Vec<(String, String)> = Vec::new();
    match prefix {
        "imdb" | "tt" => {
            push(&mut out, "imdb", value);
            push(&mut out, "tt", value);
        }
        _ => {
            push(&mut out, prefix, value);
            push(&mut out, "imdb", imdb_id);
        }
    }
    out
}

/// The `{name, year}` this id was resolved to earlier (`ext_meta_<id>.json`).
///
/// This is the LOCAL half of self-discovery, and the reason the whole thing is
/// instant: the origin's `media_ids_idx` write is not readable right away
/// (measured against the live origin — `POST /media-ids` answered
/// `{"written":3}` while `lookup-id?prefix=imdb&id=tt35062707` kept answering
/// 404 for minutes), so a resolution that only wrote to the origin would be
/// re-derived on every single request.
pub fn ext_meta_cached(media_id: &str) -> Option<(String, String)> {
    let key = media_id.trim().to_lowercase();
    if key.is_empty() {
        return None;
    }
    let hit = crate::cache::get("ext_meta", &key)?;
    let title = hit.get("name").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
    if title.is_empty() {
        return None;
    }
    let year = match hit.get("year") {
        Some(Value::String(s)) => s.trim().to_string(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    };
    Some((title, year))
}

fn ext_meta_store(media_id: &str, title: &str, year: &str) {
    let key = media_id.trim().to_lowercase();
    if key.is_empty() || title.trim().is_empty() {
        return;
    }
    crate::cache::put(
        "ext_meta",
        &key,
        &serde_json::json!({"name": title.trim(), "year": year.trim()}),
        EXT_META_TTL,
        0,
    );
}

/// `fd_media_ids_upsert()` (backend.php:2801): write the discovered mapping back
/// into the origin's `media_ids_idx`, so this id is answered by ONE index read
/// next time instead of the whole `lookup-id → upstream meta → title search`
/// ladder.
///
/// The PHP schedules the POST with `\Amp\async()` and returns before it runs, so
/// a stream response is never delayed by it; here that is one `tokio::spawn`,
/// which this server's long-lived runtime is guaranteed to drive to completion
/// (a per-request PHP worker is not).
///
/// The `lookup_id` cache is invalidated only AFTER a write that reported rows,
/// because that cache stores the MISS as well: without the invalidation a
/// freshly written id stays invisible for the full 900 s TTL, which is exactly
/// the "I added the id and nothing changed" symptom.
pub fn media_ids_upsert(
    post_id: i64,
    pairs: Vec<(String, String)>,
    title: &str,
    year: i64,
    imdb_id: &str,
    media_type: &str,
) {
    let title = title.trim().to_string();
    // Owned, because the spawned task outlives this call's borrows.
    let media_type = media_type.to_string();
    if post_id <= 0 || pairs.is_empty() || title.is_empty() {
        return;
    }
    let map: serde_json::Map<String, Value> = pairs
        .iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect();
    // Same wire shape as the `/api/media-ids` route: `media_ids` is the JSON text
    // of the map, because `httpc::http_json()` posts a FLAT object of strings.
    let params = vec![
        ("media_ids".to_string(), Value::Object(map).to_string()),
        ("title".to_string(), title.clone()),
        ("year".to_string(), year.to_string()),
        ("imdb_id".to_string(), imdb_id.trim().to_string()),
        ("media_type".to_string(), media_type.to_string()),
        ("post_id".to_string(), post_id.to_string()),
    ];
    let url = format!("{}/media-ids", config::wp_api_base().trim_end_matches('/'));
    let pairs_for_log: Vec<String> = pairs.iter().map(|(k, v)| format!("{}:{}", k, v)).collect();
    tokio::spawn(async move {
        let res = httpc::http_json(&url, &params, "POST", 8).await;
        let written = res
            .get("written")
            .and_then(|v| v.as_i64())
            .or_else(|| res.get("written").and_then(|v| v.as_u64()).map(|n| n as i64))
            .unwrap_or(0);
        let ok = res.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);
        if ok || written > 0 {
            for (p, v) in &pairs {
                crate::cache::invalidate("lookup_id", &format!("{}:{}", p, v));
            }
        }
        log(
            "media ids upsert",
            &serde_json::json!({
                "post_id": post_id,
                "ids": pairs_for_log,
                "title": title,
                "year": year,
                "media_type": media_type,
                "written": written,
                "ok": ok,
            }),
        );
    });
}

/// Persist one self-discovered identity: remember it locally AND write it back to
/// `media_ids_idx`. Returns whether an origin write was fired.
///
/// EVERY path that resolves a title for an id the origin does not know must call
/// this — that is what makes the second request cheap and what makes the id
/// exist for the scraper. The port had no such path at all (no `upsert` anywhere
/// in `src/`), so an unindexed `tt…` id stayed unresolved forever: `/stream`
/// answered its empty card, `lookup-id` kept answering 404, and the same
/// upstream call was paid again on every open.
#[allow(clippy::too_many_arguments)]
pub fn remember_external_identity(
    media_id: &str,
    prefix: &str,
    value: &str,
    title: &str,
    year: &str,
    imdb_id: &str,
    media_type: &str,
) -> bool {
    let title = title.trim();
    if title.is_empty() {
        return false;
    }
    ext_meta_store(media_id, title, year);
    let pairs = media_id_pairs(prefix, value, imdb_id);
    if pairs.is_empty() {
        return false;
    }
    let year_n: i64 = year.trim().parse().unwrap_or(0);
    media_ids_upsert(
        pseudo_post_id(prefix, value),
        pairs,
        title,
        year_n,
        imdb_id,
        media_type,
    );
    true
}

/// `GET {WP_API_BASE}/stream-files?type=…&title=…&year=…&id=…`.
///
/// Used for ids that are not one of our own posts (`tt…`, `tmdb:…`): the origin
/// searches by title and answers `{ok, resolved_title, items: […]}`. The
/// matcher still decides which of those items belong to the title, because the
/// origin's title search returns a superset (it ignores `year`).
/// Returns `(resolved_title, resolved_year, files)`.
///
/// It goes through [`httpc::cached_wp_json`] (the port of `fd_cached_wp_json()`),
/// **not** [`fetch_stream_ajax`]: that layer unwraps `items` (PHP:
/// `$decoded['data'] ?? $decoded['items'] ?? $decoded`, backend.php:9561), which
/// throws away `ok` and `resolved_title`/`resolved_year` — the two fields this
/// endpoint exists to supply. The PHP's fast paths call `fd_http_json()` /
/// `fd_cached_wp_json()` on the same URL for exactly that reason.
pub async fn stream_files_by_title(
    media_type: &str,
    title: &str,
    year: &str,
    id: &str,
) -> (String, String, Vec<WpFile>) {
    // With NO title AND NO id there is nothing to search: the origin would answer
    // `400 "title or id required"` and `cached_wp_json` would retry it once, so
    // skip the round-trip. An id WITHOUT a title is a valid call: the origin
    // resolves the identity from the id (`resolved_title`) and answers an empty
    // `items` list when it has never indexed it, which is exactly what the
    // id-only probe at `routes.rs` relies on.
    if title.trim().is_empty() && id.trim().is_empty() {
        return (title.to_string(), String::new(), Vec::new());
    }
    let mut params = vec![
        ("type".to_string(), media_type.to_string()),
        ("title".to_string(), title.to_string()),
    ];
    // Series files in the origin database almost never have the `year` column populated
    // (filenames are e.g. Show.S01E01.mkv). Passing `year` causes the origin to filter
    // `WHERE year = ...`, returning 0 rows or dropping 70-100% of episode files.
    // Like `stream_files_episode` and the legacy PHP engine, series queries must not pass `year`.
    if !year.is_empty() && media_type != "series" {
        params.push(("year".to_string(), year.to_string()));
    }
    if !id.is_empty() {
        params.push(("id".to_string(), id.to_string()));
    }
    let url = query_add(rest_route("stream_files"), &params);
    let res = httpc::cached_wp_json(&url, stream_ajax_ttl("stream_files"), 15).await;
    let resolved_title = res
        .get("resolved_title")
        .and_then(|v| v.as_str())
        .unwrap_or(title)
        .trim()
        .to_string();
    let resolved_year = match res.get("resolved_year") {
        Some(Value::String(s)) => s.trim().to_string(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    };
    // CRITICAL: `/stream-files` may answer a BARE JSON ARRAY of file rows instead
    // of an object with `items`. Reading only `items` made every `tt…` stream
    // request answer "no file matched the title" (measured: `tt11540908:1:1`
    // "Carrie" returned 75,264 B of real files that were parsed as ZERO).
    let items = rows_of(&res, &["items", "files"]);
    let mut files: Vec<WpFile> = items.iter().filter_map(wp_file_from).collect();

    // If year was passed (e.g. for a movie) and returned 0 files, fallback to querying
    // without year so files untagged with a release year in WordPress can still be found
    // and verified by the local matcher.
    if files.is_empty() && !year.is_empty() && media_type != "series" {
        let mut fallback_params = vec![
            ("type".to_string(), media_type.to_string()),
            ("title".to_string(), title.to_string()),
        ];
        if !id.is_empty() {
            fallback_params.push(("id".to_string(), id.to_string()));
        }
        let fallback_url = query_add(rest_route("stream_files"), &fallback_params);
        let fallback_res = httpc::cached_wp_json(&fallback_url, stream_ajax_ttl("stream_files"), 15).await;
        let fallback_items = rows_of(&fallback_res, &["items", "files"]);
        let fallback_files: Vec<WpFile> = fallback_items.iter().filter_map(wp_file_from).collect();
        if !fallback_files.is_empty() {
            files = fallback_files;
        }
    }

    (resolved_title, resolved_year, files)
}

/// The per-episode `stream-files` answer.
///
/// `ok` is the PHP's `isset($res['ok'])` — the origin answers `ok` **even with
/// zero items**, and that is what tells the caller to STOP instead of running the
/// expensive fallback ladder (backend.php:10265).
#[derive(Debug, Clone, Default)]
pub struct EpisodeFiles {
    pub ok: bool,
    pub resolved_title: String,
    pub resolved_year: String,
    pub files: Vec<WpFile>,
}

/// `GET {WP_API_BASE}/stream-files?type=series&season=…&episode=…&limit=…`
/// `[&title=<keyword>][&id=post:<id>]` — the PER-EPISODE index search, the
/// backend.php:10178-10219 step of `fd_fetch_episode_stream_files()`.
///
/// This is the call that surfaces the episode's other releases: the post's own
/// attached files are a subset, so the same episode uploaded by another group
/// only appears here. Cached with the PHP's own 15 s freshness
/// ([`PER_EPISODE_SEARCH_TTL`]) so a new upload appears as quickly as it does on
/// the PHP.
///
/// `year` is deliberately NOT sent: the PHP's URL carries only
/// `type/season/episode/limit/title/id`, and the origin ignores `year` anyway
/// (it matches on the resolved title alone). Sending it would also split the
/// cache key the moment the year becomes known from the post's own file set.
pub async fn stream_files_episode(
    media_type: &str,
    keyword: &str,
    id: &str,
    season: i32,
    episode: i32,
    limit: usize,
) -> EpisodeFiles {
    let mut params = vec![
        ("type".to_string(), media_type.to_string()),
        ("season".to_string(), season.to_string()),
        ("episode".to_string(), episode.to_string()),
        ("limit".to_string(), limit.to_string()),
    ];
    if !keyword.is_empty() {
        params.push(("title".to_string(), keyword.to_string()));
    }
    if !id.is_empty() {
        params.push(("id".to_string(), id.to_string()));
    }
    // Whole body, 15 s: the SAME shape the PHP uses (`fd_cached_wp_json($url, 15)`).
    // Going through `fetch_stream_ajax()` would unwrap `items` and lose `ok` —
    // which is the flag that stops the fallback ladder — plus `resolved_title`.
    let url = query_add(rest_route("stream_files"), &params);
    let res = httpc::cached_wp_json(&url, PER_EPISODE_SEARCH_TTL, 15).await;
    EpisodeFiles {
        ok: res.get("ok").is_some(),
        resolved_title: res
            .get("resolved_title")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string(),
        resolved_year: match res.get("resolved_year") {
            Some(Value::String(s)) => s.trim().to_string(),
            Some(Value::Number(n)) => n.to_string(),
            _ => String::new(),
        },
        files: rows_of(&res, &["items", "files"])
            .iter()
            .filter_map(wp_file_from)
            .collect(),
    }
}

/// `search_files` (`GET {WP_API_BASE}/search-files?search=…&limit=…`) — the
/// free-text index probe the episode fallback ladder races in parallel
/// (`fd_search_files_parallel()`, backend.php:9881).
pub async fn search_files(search: &str, limit: usize) -> Vec<WpFile> {
    search_files_at(search, limit, 0).await
}

/// [`search_files()`] for the speculative episode ladder: same query, a much
/// shorter budget ([`probe_timeout()`]).
///
/// The ladder fires every spelling of an episode number AT ONCE, so one unhealthy
/// origin costs the whole batch the full timeout. Measured while browsing
/// `VisionQuest` S01E05: 13 probes, 13 transport failures, and the list came back
/// empty anyway — the 6 s budget bought nothing but a slower failure.
pub async fn search_files_probe(search: &str, limit: usize) -> Vec<WpFile> {
    if search.trim().is_empty() {
        return Vec::new();
    }
    let params = vec![
        ("search".to_string(), search.to_string()),
        ("limit".to_string(), limit.to_string()),
        ("offset".to_string(), "0".to_string()),
    ];
    let res = fetch_stream_ajax_timeout("search_files", &params, Some(probe_timeout())).await;
    rows_of(&res, &["files", "items"])
        .iter()
        .filter_map(wp_file_from)
        .collect()
}

/// `search_files` with an `offset`, for the paged Telegram-FILE catalogs
/// (backend.php:15759 passes `offset => $skip`).
///
/// `search` may be the origin's `__latest__` token (newest files) or a 4-digit
/// year, both of which the endpoint understands.
pub async fn search_files_at(search: &str, limit: usize, offset: usize) -> Vec<WpFile> {
    if search.trim().is_empty() {
        return Vec::new();
    }
    let params = vec![
        ("search".to_string(), search.to_string()),
        ("limit".to_string(), limit.to_string()),
        ("offset".to_string(), offset.to_string()),
    ];
    let res = fetch_stream_ajax("search_files", &params).await;
    rows_of(&res, &["files", "items"])
        .iter()
        .filter_map(wp_file_from)
        .collect()
}

/// Derive the entry YEAR from a post's own file set
/// (backend.php:10121-10142).
///
/// `lookup-id?prefix=post&id=8798795472` answers with the title "One Piece" and
/// nothing else, yet all 600 of that post's files are `One.Piece.1999.<episode>`.
/// Without a year the ±1-year guard in `series_file_matches_title()` is inert, so
/// the 2023 live-action's and the 2026 remake's Episode-1 files were accepted
/// under the 1999 anime — the reported sibling-work mixing.
///
/// Adopted only from a clear majority: at least **20** tagged files, the winner
/// covering at least **60%** of them, and only for a plausible year (≥1950).
/// Ties resolve to the year seen FIRST, like PHP's stable `arsort`.
pub fn derive_year_from_files(files: &[WpFile]) -> Option<String> {
    if files.is_empty() {
        return None;
    }
    // Insertion-ordered counts (a map would lose the tie-break order).
    let mut order: Vec<(String, usize)> = Vec::new();
    let year_re = fancy_regex::Regex::new(r"\b(19\d{2}|20\d{2})\b").ok()?;
    for f in files {
        let Some(caps) = year_re.captures(&f.title).ok().flatten() else {
            continue;
        };
        let Some(m) = caps.get(1) else { continue };
        let y = m.as_str().to_string();
        match order.iter_mut().find(|(k, _)| *k == y) {
            Some(slot) => slot.1 += 1,
            None => order.push((y, 1)),
        }
    }
    let mut best: Option<&(String, usize)> = None;
    for slot in &order {
        if best.map(|b| slot.1 > b.1).unwrap_or(true) {
            best = Some(slot);
        }
    }
    let (year, count) = best?;
    let tagged: usize = order.iter().map(|(_, c)| c).sum();
    let plausible = year.parse::<i32>().map(|y| y >= 1950).unwrap_or(false);
    if plausible && *count >= 20 && (*count as f64) >= 0.6 * tagged as f64 {
        Some(year.clone())
    } else {
        None
    }
}

/// Rows from any of the shapes the origin uses: a bare array, or an object with
/// one of `keys` holding the array.
fn rows_of(v: &Value, keys: &[&str]) -> Vec<Value> {
    if let Some(arr) = v.as_array() {
        return arr.clone();
    }
    for key in keys {
        if let Some(arr) = v.get(*key).and_then(|x| x.as_array()) {
            return arr.clone();
        }
    }
    Vec::new()
}

/// The WordPress post behind a `pm:post:<id>` catalog item.
pub async fn get_post(post_id: &str) -> Value {
    let params = vec![("post_id".to_string(), post_id.to_string())];
    let res = fetch_stream_ajax("get_post", &params).await;
    // The handler answers either the post object or `{post: {...}}`.
    match res.get("post") {
        Some(p) if p.is_object() => p.clone(),
        _ => res,
    }
}

pub fn post_title(post: &Value) -> String {
    ["title", "post_title", "name"]
        .iter()
        .find_map(|k| post.get(*k).and_then(|v| v.as_str()))
        .unwrap_or("")
        .trim()
        .to_string()
}

pub fn post_year(post: &Value) -> String {
    for key in ["year", "release_year", "release_year_nr"] {
        if let Some(v) = post.get(key) {
            let s = v
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| v.to_string());
            let s = s.trim().to_string();
            if s.chars().count() >= 4 && s.chars().take(4).all(|c| c.is_ascii_digit()) {
                return s.chars().take(4).collect();
            }
        }
    }
    // Fall back to the year embedded in the title ("The Boy 2026 • Movie").
    let title = post_title(post);
    if let Some(caps) = fancy_regex::Regex::new(r"\b(19\d{2}|20\d{2})\b")
        .ok()
        .and_then(|re| re.captures(&title).ok().flatten())
    {
        if let Some(m) = caps.get(1) {
            return m.as_str().to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_payload_is_cached_only_briefly() {
        // `[]`, `{}` and `{"files":[]}` are EMPTY: one origin hiccup must not pin a
        // catalog row for the action's full TTL.
        assert!(is_empty_payload(&serde_json::json!([])));
        assert!(is_empty_payload(&serde_json::json!({})));
        assert!(is_empty_payload(&serde_json::json!({"files": []})));
        assert!(is_empty_payload(&serde_json::json!({"files": [], "items": []})));
        // A real payload is not.
        assert!(!is_empty_payload(&serde_json::json!({"files": [{"short_code": "x"}]})));
        assert!(!is_empty_payload(&serde_json::json!([{"id": 1}])));
        assert!(!is_empty_payload(&serde_json::json!({"files": [{"a": 1}], "has_more": true})));
        assert!(!is_empty_payload(&serde_json::json!({"id": 9000016, "title": "x"})));
        assert!(EMPTY_PAYLOAD_TTL < stream_ajax_ttl("search"));
        assert!(EMPTY_PAYLOAD_TTL < stream_ajax_ttl("posts"));
    }

    #[test]
    fn ttl_matches_php() {
        assert_eq!(stream_ajax_ttl("search"), 300);
        assert_eq!(stream_ajax_ttl("post_files"), 600);
        assert_eq!(stream_ajax_ttl("trending"), 3600);
        assert_eq!(stream_ajax_ttl("something_else"), 300);
        // The per-episode search is fresher than every generic action, exactly
        // like the PHP's explicit `fd_cached_wp_json($url, 15)`.
        assert_eq!(PER_EPISODE_SEARCH_TTL, 15);
        assert!(PER_EPISODE_SEARCH_TTL < stream_ajax_ttl("stream_files"));
    }

    fn files(titles: &[&str]) -> Vec<WpFile> {
        titles
            .iter()
            .enumerate()
            .map(|(i, t)| WpFile {
                short_code: format!("c{i}"),
                title: t.to_string(),
                ..Default::default()
            })
            .collect()
    }

    #[test]
    fn derives_year_from_a_clear_majority_of_the_post_set() {
        // The One Piece 1999 anime post: every file carries the year.
        let mut list = Vec::new();
        for i in 1..=25 {
            list.push(format!("One.Piece.1999.EP{:02}.1080p.mkv", i));
        }
        list.push("One.Piece.Live.Action.1080p.mkv".to_string());
        let year = derive_year_from_files(&files(&list.iter().map(|s| s.as_str()).collect::<Vec<_>>()));
        assert_eq!(year.as_deref(), Some("1999"));
    }

    #[test]
    fn year_derivation_needs_20_tagged_files() {
        // 19 tagged files is below the PHP's floor: the guard stays inert rather
        // than re-dating a show from a handful of rows.
        let list: Vec<String> = (1..=19).map(|i| format!("Show.2004.EP{i}.mkv")).collect();
        assert!(derive_year_from_files(&files(&list.iter().map(|s| s.as_str()).collect::<Vec<_>>())).is_none());
    }

    #[test]
    fn year_derivation_needs_a_60_percent_majority() {
        // 20 files say 1999 but 20 say 2023 — no majority, so nothing is adopted.
        let mut list: Vec<String> = (1..=20).map(|i| format!("Show.1999.EP{i}.mkv")).collect();
        list.extend((1..=20).map(|i| format!("Show.2023.EP{i}.mkv")));
        assert!(derive_year_from_files(&files(&list.iter().map(|s| s.as_str()).collect::<Vec<_>>())).is_none());
    }

    #[test]
    fn year_derivation_ignores_untagged_files_and_empty_sets() {
        assert!(derive_year_from_files(&[]).is_none());
        // Untagged rows must not count towards either the floor or the ratio.
        let mut list: Vec<String> = (1..=20).map(|i| format!("Show.2011.EP{i}.mkv")).collect();
        list.extend((1..=80).map(|i| format!("Show.EP{i}.mkv")));
        let year = derive_year_from_files(&files(&list.iter().map(|s| s.as_str()).collect::<Vec<_>>()));
        assert_eq!(year.as_deref(), Some("2011"));
    }

    #[test]
    fn trending_never_gets_a_bot_id() {
        // The cache key is the URL, so a `bot_id` here would give every bot its
        // own copy of the same global keyword table.
        assert!(!stream_ajax_wants_bot("trending"));
        assert!(stream_ajax_wants_bot("posts"));
        assert!(stream_ajax_wants_bot("search"));
        assert!(stream_ajax_wants_bot("post_files"));
    }

    #[test]
    fn topkeyword_filter_only_drops_empty_tokens() {
        // The app performs NO keyword filtering: whatever the REST endpoint
        // returns is what we show. Only an empty token is dropped, because it
        // would render an unusable row.
        assert!(!is_topkeyword_valid(""));
        assert!(!is_topkeyword_valid("   "));
        assert!(is_topkeyword_valid("the rookie"));
    }

    #[test]
    fn parses_trending_payload() {
        // The live shape: `{success:true, data:[{keyword,hits,total_found}]}`.
        let live = serde_json::json!([
            {"keyword": "the rookie", "hits": 1814, "total_found": 1814},
            {"keyword": "spiderman", "hits": "2251"},
            {"keyword": "THE ROOKIE", "hits": 3}
        ]);
        // Order is the origin's; only the case-insensitive duplicate is dropped.
        assert_eq!(
            trending_keywords_from(&live),
            vec![
                ("the rookie".to_string(), 1814),
                ("spiderman".to_string(), 2251)
            ]
        );
        // A bare array of strings is accepted (empty entries skipped), and an
        // envelope-shaped body is unwrapped by the caller — neither may panic.
        let bare = serde_json::json!(["kudrat", "", "hometown cha cha cha"]);
        assert_eq!(
            trending_keywords_from(&bare),
            vec![
                ("kudrat".to_string(), 0),
                ("hometown cha cha cha".to_string(), 0)
            ]
        );
        assert!(trending_keywords_from(&serde_json::json!({"message": "nope"})).is_empty());
    }

    #[test]
    fn route_replaces_underscores() {
        let r = rest_route("post_files");
        assert!(r.ends_with("/post-files"), "{r}");
        let r = rest_route("search_files");
        assert!(r.ends_with("/search-files"), "{r}");
        let r = rest_route("stream_files");
        assert!(r.ends_with("/stream-files"), "{r}");
    }

    #[test]
    fn unwraps_wordpress_envelope() {
        let v = serde_json::json!({"success": true, "data": {"files": [1]}});
        assert_eq!(unwrap_data(&v)["files"][0], 1);
        let v = serde_json::json!({"data": {"x": 2}});
        assert_eq!(unwrap_data(&v)["x"], 2);
        let v = serde_json::json!({"items": [3]});
        assert_eq!(unwrap_data(&v)[0], 3);
        let v = serde_json::json!({"ok": 1});
        assert_eq!(unwrap_data(&v)["ok"], 1);
    }

    #[test]
    fn reads_rows_from_either_shape() {
        // `/stream-files` answers a BARE ARRAY (this is the shape that broke
        // tt11540908 "Carrie": 75 KB of files read as zero).
        let arr = serde_json::json!([
            {"short_code": "m9qZjD", "title": "Carrie.2026.S01E01.1080p.mkv"}
        ]);
        assert_eq!(rows_of(&arr, &["items"]).len(), 1);
        let obj = serde_json::json!({"items": [{"short_code": "a"}]});
        assert_eq!(rows_of(&obj, &["items", "files"]).len(), 1);
        let obj = serde_json::json!({"files": [{"short_code": "a"}, {"short_code": "b"}]});
        assert_eq!(rows_of(&obj, &["files"]).len(), 2);
        assert!(rows_of(&serde_json::json!({"message": "nope"}), &["items"]).is_empty());
    }

    #[test]
    fn parses_a_file_row() {
        let v = serde_json::json!({
            "short_code": "1v9OPj",
            "title": "Avengers.Infinity.War.2018.2160p.mkv",
            "caption": "Avengers Infinity War (2018)",
            "file_size": "3967229576",
            "mime": "video/x-matroska",
            "season_num": 1,
            "episode_num": 3
        });
        let f = wp_file_from(&v).expect("row");
        assert_eq!(f.short_code, "1v9OPj");
        assert_eq!(f.file_size, 3_967_229_576);
        assert_eq!(f.mime, "video/x-matroska");
        assert_eq!(f.season_num, 1);
        assert_eq!(f.episode_num, 3);
        // Neither column is in the legacy row: both must stay at 0, not fail.
        assert_eq!((f.duration, f.part), (0, 0));
        assert!(wp_file_from(&serde_json::json!({"title": "no code"})).is_none());
    }

    #[test]
    fn parses_the_duration_and_part_columns_when_present() {
        // Shape of `tg_file_new`: `duration` is uint, `part` is uint. A row that
        // carries them lets the stream builder skip the resolve AND keep the
        // ordinal the origin already knows.
        let v = serde_json::json!({
            "short_code": "rUaIMd",
            "title": "Obsession.2026.2160p.iT.WEB.DL.UNRATED.DV.HDR10.MULTiBen.The.Men.mp4",
            "file_size": 4186973969u64,
            "duration": 1185,
            "part": 5
        });
        let f = wp_file_from(&v).expect("row");
        assert_eq!(f.duration, 1185);
        assert_eq!(f.part, 5);
        // Strings are accepted too: the origin sends some uints quoted.
        let g = wp_file_from(&serde_json::json!({
            "short_code": "x", "duration": "4224", "part": "2"
        }))
        .expect("row");
        assert_eq!((g.duration, g.part), (4224, 2));
        // An unknown/garbage value must degrade to 0, never to a wrong number.
        let h = wp_file_from(&serde_json::json!({
            "short_code": "y", "duration": "unknown", "part": -3
        }))
        .expect("row");
        assert_eq!((h.duration, h.part), (0, -3));
    }

    #[test]
    fn post_year_from_title_or_field() {
        let p = serde_json::json!({"title": "The Boy 2026 • Movie"});
        assert_eq!(post_year(&p), "2026");
        let p = serde_json::json!({"title": "Something", "year": "2011"});
        assert_eq!(post_year(&p), "2011");
        let p = serde_json::json!({"title": "No Year Here"});
        assert_eq!(post_year(&p), "");
    }

    /// The self-discovery write keys its row on a PSEUDO post id, so the number
    /// itself is the contract (`backend.php:3144/3247/3319/3380`). Two lineages
    /// that disagree on it index the same id twice and neither lookup finds the
    /// other's row.
    #[test]
    fn pseudo_post_ids_match_the_php_lineage() {
        // imdb: 9000000000 + (numeric imdb % 1000000000) — backend.php:3144.
        assert_eq!(pseudo_post_id("imdb", "tt35062707"), 9_035_062_707);
        assert_eq!(pseudo_post_id("tt", "tt35062707"), 9_035_062_707);
        // Leading zeros are digits too: tt0944947 -> 0944947 -> 944947.
        assert_eq!(pseudo_post_id("imdb", "tt0944947"), 9_000_944_947);
        // A 10-digit imdb id keeps only its last 9 digits (the modulo).
        assert_eq!(pseudo_post_id("imdb", "tt1000000000"), 9_000_000_000);
        // tmdb / kitsu — backend.php:3247 / 3319.
        assert_eq!(pseudo_post_id("tmdb", "1399"), 8_000_001_399);
        assert_eq!(pseudo_post_id("kitsu", "1"), 7_000_000_001);
        // Any other prefix is `(int) sprintf('%u', crc32("prefix:value"))`.
        assert_eq!(crc32_ieee(b"hello"), 0x3610_a686);
        assert_eq!(
            pseudo_post_id("tvdb", "81797"),
            crc32_ieee(b"tvdb:81797") as i64
        );
    }

    /// What one discovered id is written AS. An `imdb`/`tt` id must land under
    /// BOTH spellings (the `/stream` path asks `imdb`, the `/meta` path asks `tt`
    /// first), and a `tmdb:` id must carry the IMDb id it resolved to — that is
    /// what stops the next request from asking any addon at all.
    #[test]
    fn media_id_pairs_cover_both_spellings_and_the_imdb_alias() {
        assert_eq!(
            media_id_pairs("imdb", "tt35062707", ""),
            vec![
                ("imdb".to_string(), "tt35062707".to_string()),
                ("tt".to_string(), "tt35062707".to_string()),
            ]
        );
        assert_eq!(
            media_id_pairs("tmdb", "1399", "tt0944947"),
            vec![
                ("tmdb".to_string(), "1399".to_string()),
                ("imdb".to_string(), "tt0944947".to_string()),
            ]
        );
        // No IMDb id yet: only the prefix the client used, never an empty pair.
        assert_eq!(
            media_id_pairs("tmdb", "1399", ""),
            vec![("tmdb".to_string(), "1399".to_string())]
        );
        // Garbage in, nothing out — an empty pair would write a row no lookup
        // can ever find, and `media_ids_idx` is shared with the scraper.
        assert!(media_id_pairs("", "", "").is_empty());
        // A known IMDb id is worth writing even when the client's own value is
        // blank: the point of the row is that the id maps to a title at all.
        assert_eq!(
            media_id_pairs("tvdb", "", "tt1"),
            vec![("imdb".to_string(), "tt1".to_string())]
        );
    }

    /// Self-discovery must never persist a placeholder. The by-id probe falls
    /// back to the ITEM ID when the origin resolved no title, and writing that
    /// would poison the index with a row whose "title" is `tt35062707`.
    #[test]
    fn a_resolution_without_a_title_is_never_persisted() {
        assert!(!remember_external_identity(
            "tt35062707",
            "imdb",
            "tt35062707",
            "",
            "2026",
            "tt35062707",
            "movie"
        ));
        assert!(!remember_external_identity(
            "tt35062707",
            "imdb",
            "tt35062707",
            "   ",
            "",
            "",
            "movie"
        ));
    }
}
