//! Upstream Stremio addon fan-out — the port of `backend.php:2900-3040`
//! (`fd_upstream_manifest_json`, `fd_manifest_has_resource`,
//! `fd_upstream_manifests_with_resource`, `fd_get_upstream_resolver_manifests`)
//! and the `up_*` branches of the manifest / catalog / meta / stream handlers
//! (`backend.php:15343-15416`, `15497-15537`, `16410-16455`, `16690-16739`,
//! `18116-18142`).
//!
//! What the PHP does, and what is reproduced here:
//!
//! 1. **Manifest bridge** — every configured upstream whose manifest declares
//!    `catalog` contributes its catalogs as `up_<index>_<realCatalogId>`, where
//!    `<index>` is the entry's position in `settings.upstream_manifests`. The
//!    catalog object is TRIMMED (`extra` keeps only `name`/`isRequired` and an
//!    `options` list of at most 12) because AIOMetadata alone ships ~6,700
//!    option strings, which pushed the descriptor past Stremio's limit.
//! 2. **Catalog forwarding** — `/catalog/:type/up_<i>_<id>[/extra].json` is
//!    forwarded verbatim to `{upstream}/catalog/{type}/{id}[/extra].json` and
//!    disk-cached for 1 h (`up_cat_<md5>.json`).
//! 3. **Meta forwarding** — an id that is NOT ours (`pm:` / `pm_`) is forwarded
//!    to the first upstream that declares `meta` and answers a non-empty
//!    `meta.name`; cached 1 h (`up_meta_<md5>.json`). Without this a bridged
//!    catalog row opens as a blank page.
//! 4. **Stream merge** — `/stream/*` also asks upstreams that declare `stream`
//!    and appends their items AFTER ours (the sponsor stays first overall).
//! 5. **Master switch** — `upstream_enabled: false` empties every list, so the
//!    stored manifest URLs are preserved but nothing is bridged. The resolver
//!    list deliberately IGNORES the switch, exactly like the PHP: playing an
//!    external id from another addon must keep working with bridging off.
//! 6. **Descriptor budget** — the manifest must stay under
//!    [`MANIFEST_DESCRIPTOR_BUDGET`]; trailing bridged catalogs are dropped
//!    until it fits (Stremio rejects a descriptor over ~49 KB with `code 20004`
//!    "Max descriptor size reached").

use std::collections::HashMap;
use std::sync::OnceLock;

use serde_json::{json, Value};

use crate::httpc::http_json;
use crate::log::log;

/// `FD_DEFAULT_UPSTREAM_MANIFEST_URL` (backend.php:205).
///
/// The PHP default is the elfhosted AIOMetadata deployment; `aiometadata.strem.io`
/// does not resolve (verified: `fetch failed` from this machine) so the
/// reachable default is used.
pub const DEFAULT_UPSTREAM_MANIFEST_URL: &str =
    "https://aiometadata.elfhosted.com/stremio/136f7ae5-2f15-471d-8f3f-79ee335bbd24/manifest.json";
pub const DEFAULT_UPSTREAM_MANIFEST_NAME: &str = "AIOMetadata";

/// `FD_DEFAULT_SUBTITLE_MANIFEST_URL` / `..._NAME` (backend.php:223-224).
/// Declares `subtitles` only, so it is never asked for a catalog, meta or
/// stream — the fan-outs are resource-aware for exactly this reason.
pub const DEFAULT_SUBTITLE_MANIFEST_URL: &str = "https://opensubtitles-v3.strem.io/manifest.json";
pub const DEFAULT_SUBTITLE_MANIFEST_NAME: &str = "OpenSubtitles v3";

/// Stremio's `addonCollectionSet` rejects a descriptor over ~49 KB
/// (`code 20004`, "Max descriptor size reached"). The PHP manifest reached
/// 71 KB with a 170-catalog AIOMetadata config, so both runtimes cap it; the
/// Android server uses the same 45,000 B budget.
pub const MANIFEST_DESCRIPTOR_BUDGET: usize = 45_000;

/// One entry of `settings.upstream_manifests`.
#[derive(Clone, Debug, PartialEq)]
pub struct Upstream {
    /// Position in the STORED list — the `up_<index>_…` catalog prefix, so it
    /// must not be renumbered when an entry without a URL sits in between.
    pub index: usize,
    pub url: String,
    pub name: String,
    pub id: String,
}

/// The two upstreams a fresh install ships with (`fd_load_catalog_settings`
/// defaults). An existing `storage/catalog_settings.json` that lists at least
/// one valid manifest always overrides this.
pub fn default_manifests() -> Value {
    json!([
        {"url": DEFAULT_UPSTREAM_MANIFEST_URL, "name": DEFAULT_UPSTREAM_MANIFEST_NAME},
        {"url": DEFAULT_SUBTITLE_MANIFEST_URL, "name": DEFAULT_SUBTITLE_MANIFEST_NAME}
    ])
}

fn s(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// Configured upstreams, skipping entries without a URL.
///
/// `respect_master_switch` is false only for the resolver list.
pub fn configured(settings: &Value, respect_master_switch: bool) -> Vec<Upstream> {
    // An absent key means "on": the merged document always supplies it, and the
    // PHP default is `true`.
    let enabled = settings
        .get("upstream_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if respect_master_switch && !enabled {
        return Vec::new();
    }
    let Some(list) = settings.get("upstream_manifests").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    list.iter()
        .enumerate()
        .filter_map(|(index, m)| {
            let url = s(m, "url");
            if url.is_empty() {
                return None;
            }
            let id = {
                let id = s(m, "id");
                if id.is_empty() {
                    format!("up{}", index)
                } else {
                    id
                }
            };
            let name = {
                let n = s(m, "name");
                if n.is_empty() {
                    "Addon".to_string()
                } else {
                    n
                }
            };
            Some(Upstream {
                index,
                url,
                name,
                id,
            })
        })
        .collect()
}

/* ── manifest fetch + cache (fd_upstream_manifest_json) ───────────────────── */

/// `fd_upstream_manifest_json()`: cache (L1+redb, 2 h) → network, single-flight.
///
/// A FAILED fetch is not stored (see [`crate::cache::fetch_cached`]), so an
/// unreachable addon is retried instead of being disabled for two hours.
pub async fn manifest_json(url: &str) -> Value {
    let url = url.trim();
    if url.is_empty() {
        return json!({});
    }
    let url_owned = url.to_string();
    let decoded = crate::cache::fetch_cached("manifest", url, 7200, 0, move || async move {
        let body = http_json(&url_owned, &[], "GET", 6).await;
        // Cached whenever the response is a valid descriptor — NOT only when it
        // lists catalogs (OpenSubtitles v3 declares `catalogs: []`).
        let cacheable = !s(&body, "id").is_empty();
        (body, cacheable)
    })
    .await;
    if s(&decoded, "id").is_empty() {
        log(
            "upstream manifest unavailable",
            &json!({"url": url}),
        );
    }
    decoded
}

/// Fetch upstream manifests in bounded parallel batches, keeping configured order.
///
/// Cold addon/catalog requests used to pay each upstream's network latency one at a
/// time. Four concurrent checks reduce that wait without creating unbounded fan-out
/// when a user configures many upstreams.
const UPSTREAM_FANOUT: usize = 4;

async fn manifests_for(upstreams: &[Upstream]) -> Vec<Value> {
    let mut manifests = Vec::with_capacity(upstreams.len());
    for batch in upstreams.chunks(UPSTREAM_FANOUT) {
        let batch_manifests = futures_util::future::join_all(
            batch.iter().map(|up| manifest_json(&up.url)),
        )
        .await;
        manifests.extend(batch_manifests);
    }
    manifests
}

/// `fd_manifest_has_resource()`: accepts both `"resources": ["subtitles"]` and
/// the object form `"resources": [{"name":"subtitles", …}]`.
pub fn has_resource(manifest: &Value, resource: &str) -> bool {
    let Some(list) = manifest.get("resources").and_then(|v| v.as_array()) else {
        return false;
    };
    for entry in list {
        let name = match entry {
            Value::String(x) => x.clone(),
            Value::Object(o) => o
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            _ => String::new(),
        };
        if !name.is_empty() && name.eq_ignore_ascii_case(resource) {
            return true;
        }
    }
    false
}

/// `fd_upstream_manifests_with_resource()`.
///
/// A manifest that cannot be fetched is KEPT, so a transient network failure
/// never silently disables bridging (the PHP comment calls this "keep legacy
/// behaviour").
pub async fn with_resource(
    settings: &Value,
    resource: &str,
    respect_master_switch: bool,
) -> Vec<Upstream> {
    let upstreams = configured(settings, respect_master_switch);
    let manifests = manifests_for(&upstreams).await;
    let mut out = Vec::new();
    for (up, m) in upstreams.into_iter().zip(manifests) {
        if s(&m, "id").is_empty() {
            out.push(up);
            continue;
        }
        if has_resource(&m, resource) {
            out.push(up);
        }
    }
    out
}

/// `fd_get_upstream_resolver_manifests()`: meta-capable upstreams, IGNORING the
/// master switch (an external id from any other addon must still resolve), with
/// the AIOMetadata default as the fallback when nothing is configured.
pub async fn resolver_manifests(settings: &Value) -> Vec<Upstream> {
    let all = configured(settings, false);
    let mut valid = all.clone();
    if !valid.is_empty() {
        let meta_capable = with_resource(settings, "meta", false).await;
        if !meta_capable.is_empty() {
            valid = meta_capable;
        }
    }
    if valid.is_empty() {
        valid.push(Upstream {
            index: 0,
            url: DEFAULT_UPSTREAM_MANIFEST_URL.to_string(),
            name: DEFAULT_UPSTREAM_MANIFEST_NAME.to_string(),
            id: "up0".to_string(),
        });
    }
    valid
}

/// `/manifest.json` is advertised without `.json`, with or without a query.
pub fn base_of(manifest_url: &str) -> String {
    let trimmed = manifest_url.trim().trim_end_matches('/');
    let lower = trimmed.to_ascii_lowercase();
    if let Some(pos) = lower.find("/manifest.json") {
        return trimmed[..pos].trim_end_matches('/').to_string();
    }
    trimmed.to_string()
}

/* ── catalog bridge ───────────────────────────────────────────────────────── */

/// `up_<index>_<realCatalogId>` → (index, real id).
pub fn parse_bridged(catalog_id: &str) -> Option<(usize, String)> {
    let rest = catalog_id.strip_prefix("up_")?;
    let (idx, real) = rest.split_once('_')?;
    if real.is_empty() || idx.is_empty() || !idx.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    idx.parse().ok().map(|i| (i, real.to_string()))
}

/// PHP `urlencode()` — space becomes `+`. Used for the forwarded catalog id.
fn urlencode(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    for b in v.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// PHP `rawurlencode()` — space becomes `%20`. Used for meta/stream ids.
fn rawurlencode(v: &str) -> String {
    let mut out = String::with_capacity(v.len());
    for b in v.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// The bridged catalogs of every configured upstream, plus the catalog `type`
/// values they add (PHP appends them to `activeTypes`).
///
/// **`catalogs_enabled: false` empties this too.** The PHP builds
/// `$manifestCatalogs` (local + upstream) and only keeps it inside
/// `if (!empty($catSettings['catalogs_enabled']))` (backend.php:15303), so
/// turning catalogs off removes the bridged rows even with
/// `upstream_enabled: true`. Gating only the local rows left 40 upstream
/// catalogs in the descriptor.
pub async fn bridged_catalogs(settings: &Value) -> (Vec<Value>, Vec<String>) {
    let catalogs_enabled = settings
        .get("catalogs_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if !catalogs_enabled {
        return (Vec::new(), Vec::new());
    }
    let mut catalogs: Vec<Value> = Vec::new();
    let mut types: Vec<String> = Vec::new();
    let upstreams = configured(settings, true);
    let manifests = manifests_for(&upstreams).await;
    for (up, m) in upstreams.into_iter().zip(manifests) {
        let Some(list) = m.get("catalogs").and_then(|v| v.as_array()) else {
            continue;
        };
        for cat in list {
            let ty = s(cat, "type");
            let id = s(cat, "id");
            if ty.is_empty() || id.is_empty() {
                continue;
            }
            if !types.contains(&ty) {
                types.push(ty.clone());
            }
            let name = {
                let n = s(cat, "name");
                let n = if n.is_empty() {
                    "Catalog".to_string()
                } else {
                    n
                };
                format!("{} ({})", n, up.name)
            };
            let mut bridged = json!({
                "type": ty,
                "id": format!("up_{}_{}", up.index, id),
                "name": name,
            });
            if let Some(extra) = cat.get("extra").and_then(|v| v.as_array()) {
                let mut trimmed: Vec<Value> = Vec::new();
                for ex in extra {
                    let ex_name = s(ex, "name");
                    if ex_name.is_empty() {
                        continue;
                    }
                    let mut entry = json!({"name": ex_name});
                    if let Some(req) = ex.get("isRequired") {
                        entry["isRequired"] = json!(req.as_bool().unwrap_or(false));
                    }
                    // Keep a small option list so simple filters still render;
                    // drop the huge ones (languages, seasons, studios, decades)
                    // that blow up the descriptor.
                    if let Some(opts) = ex.get("options").and_then(|v| v.as_array()) {
                        if opts.len() <= 12 {
                            entry["options"] = Value::Array(opts.clone());
                        }
                    }
                    trimmed.push(entry);
                }
                if !trimmed.is_empty() {
                    bridged["extra"] = Value::Array(trimmed);
                }
            }
            catalogs.push(bridged);
        }
    }
    (catalogs, types)
}

/// Drop trailing bridged catalogs until the descriptor fits the budget.
///
/// The LOCAL catalogs (`local_count`, i.e. `pm_*`) are never dropped — they are
/// the addon's own rows, and trimming those would break the dashboard.
pub fn fit_descriptor(mut manifest: Value, local_count: usize) -> Value {
    let start_len = serde_json::to_string(&manifest)
        .map(|t| t.len())
        .unwrap_or(0);
    let mut dropped: Vec<String> = Vec::new();
    loop {
        let len = serde_json::to_string(&manifest)
            .map(|t| t.len())
            .unwrap_or(0);
        if len <= MANIFEST_DESCRIPTOR_BUDGET {
            break;
        }
        let cats = match manifest.get_mut("catalogs").and_then(|v| v.as_array_mut()) {
            Some(c) => c,
            None => break,
        };
        if cats.len() <= local_count {
            log(
                "manifest over the descriptor budget with only local catalogs",
                &json!({"bytes": len, "catalogs": cats.len()}),
            );
            break;
        }
        match cats.pop() {
            Some(d) => dropped.push(s(&d, "id")),
            None => break,
        }
    }
    // ONE line, never one per dropped row. An upstream advertising 300+ catalogs
    // made this loop drop 193 rows on EVERY manifest build and emit 193 log lines
    // each time — measured 3,021 of the last 4,000 log lines, which buried the
    // events that actually need reading (failed downloads, NO_STREAMS incidents).
    if !dropped.is_empty() {
        let end_len = serde_json::to_string(&manifest)
            .map(|t| t.len())
            .unwrap_or(start_len);
        log(
            "manifest descriptor over budget, bridged catalogs dropped",
            &json!({
                "from_bytes": start_len,
                "to_bytes": end_len,
                "budget": MANIFEST_DESCRIPTOR_BUDGET,
                "dropped": dropped.len(),
                "first_dropped": dropped.first().cloned().unwrap_or_default(),
                "last_dropped": dropped.last().cloned().unwrap_or_default(),
            }),
        );
    }
    manifest
}

/* ── catalog forwarding ───────────────────────────────────────────────────── */

/// Forward `/catalog/:type/up_<i>_<id>[/extra].json` to the upstream addon.
///
/// `extra_raw` is the path segment EXACTLY as it arrived (still percent-encoded)
/// — re-encoding a decoded value would corrupt `genre=Science+Fiction`.
/// `query` is the request's own query string, appended like the PHP does.
pub async fn forward_catalog(
    settings: &Value,
    index: usize,
    item_type: &str,
    real_id: &str,
    extra_raw: &str,
    query: &str,
) -> Option<Value> {
    let ups = configured(settings, true);
    let up = ups.iter().find(|u| u.index == index)?;
    let base = base_of(&up.url);
    let mut target = format!(
        "{}/catalog/{}/{}",
        base,
        urlencode(item_type),
        urlencode(real_id)
    );
    if !extra_raw.is_empty() {
        target.push('/');
        target.push_str(extra_raw);
    }
    target.push_str(".json");
    if !query.is_empty() {
        target.push('?');
        target.push_str(query);
    }

    // The upstream url IS the key: no md5 to compute, and a dump/stats readout is
    // human-readable. `catalogs` are 8-40 KB, so a 1 h TTL is the whole point.
    if let Some(cached) = crate::cache::get("up_cat", &target) {
        if cached.get("metas").is_some() {
            log(
                "upstream catalog served from cache",
                &json!({"url": target, "index": index}),
            );
            return Some((*cached).clone());
        }
    }
    let body = http_json(&target, &[], "GET", 8).await;
    if body.get("metas").is_some() {
        crate::cache::put("up_cat", &target, &body, 3600, 0);
        let count = body
            .get("metas")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        log(
            "upstream catalog forwarded",
            &json!({"url": target, "index": index, "metas": count}),
        );
        return Some(body);
    }
    log(
        "upstream catalog unavailable",
        &json!({"url": target, "index": index}),
    );
    None
}

/* ── meta forwarding ──────────────────────────────────────────────────────── */

/// Forward a non-`pm:` id to the first meta-capable upstream that answers a
/// non-empty `meta.name`. Returns the upstream body verbatim (`{"meta": …}`),
/// so `videos`, `poster`, `background` and `genres` all come through.
pub async fn forward_meta(settings: &Value, item_type: &str, item_id: &str) -> Option<Value> {
    let meta_key = format!("{}_{}", item_type, item_id);
    if let Some(cached) = crate::cache::get("up_meta", &meta_key) {
        let name = s(cached.get("meta").unwrap_or(&Value::Null), "name");
        if !name.is_empty() {
            let has_videos = cached
                .get("meta")
                .and_then(|m| m.get("videos"))
                .and_then(|v| v.as_array())
                .map(|a| !a.is_empty())
                .unwrap_or(false);
            // A series meta without `videos` is useless to the player; keep
            // looking (the PHP re-queries in that case too).
            if item_type != "series" || has_videos {
                log(
                    "upstream meta served from cache",
                    &json!({"type": item_type, "id": item_id}),
                );
                return Some((*cached).clone());
            }
        }
    }
    for up in resolver_manifests(settings).await {
        let base = base_of(&up.url);
        let target = format!(
            "{}/meta/{}/{}.json",
            base,
            urlencode(item_type),
            rawurlencode(item_id)
        );
        let body = http_json(&target, &[], "GET", 4).await;
        let name = s(body.get("meta").unwrap_or(&Value::Null), "name");
        if name.is_empty() {
            continue;
        }
        crate::cache::put("up_meta", &meta_key, &body, 3600, 0);
        log(
            "upstream meta forwarded",
            &json!({"type": item_type, "id": item_id, "from": up.name, "name": name}),
        );
        return Some(body);
    }
    None
}

/* ── stream merge ─────────────────────────────────────────────────────────── */

/// Streams from upstreams declaring `stream`, appended after ours.
///
/// A local `pm:` / `pm_` id never fans out (the PHP gates on `$isLocalPmId`),
/// and an item is kept only when it can actually play (`url`, `infoHash` or
/// `externalUrl`).
pub async fn streams(settings: &Value, item_type: &str, item_id: &str) -> Vec<Value> {
    if item_id.starts_with("pm:") || item_id.starts_with("pm_") {
        return Vec::new();
    }
    let upstreams = with_resource(settings, "stream", true).await;
    let mut out: Vec<Value> = Vec::new();
    for batch in upstreams.chunks(UPSTREAM_FANOUT) {
        let responses = futures_util::future::join_all(batch.iter().map(|up| async move {
            let base = base_of(&up.url);
            let target = format!(
                "{}/stream/{}/{}.json",
                base,
                urlencode(item_type),
                urlencode(item_id)
            );
            (up, http_json(&target, &[], "GET", 4).await)
        }))
        .await;
        for (up, body) in responses {
            let Some(list) = body.get("streams").and_then(|v| v.as_array()) else {
                continue;
            };
            let mut added = 0usize;
            for item in list {
                let playable = ["url", "infoHash", "externalUrl"]
                    .iter()
                    .any(|k| !s(item, k).is_empty());
                if playable {
                    out.push(item.clone());
                    added += 1;
                }
            }
            log(
                "upstream streams merged",
                &json!({"from": up.name, "id": item_id, "added": added}),
            );
        }
    }
    out
}

/* ── verified episode names (fd_upstream_episode_names) ───────────────────── */
//
// A bridged/catalog row for a series can name an episode; the series matcher
// uses that name to reject a file that clearly belongs to a SIBLING work (the
// same show title, same episode number, another show's episode title after the
// marker). Everything here FAILS OPEN: no manifest, a transport failure, a
// placeholder name, or a one-word filename tail all mean "no evidence", and no
// evidence never hides a stream. The matcher owns that contract
// (`episode_subtitle_matches`); this module only supplies VERIFIED names.

/// Largest number of resolver upstreams asked (PHP: `array_slice(…, 0, 3)`).
const MAX_NAME_PROVIDERS: usize = 3;
/// Per-provider budget. The PHP bounds the Amp fan-out to 2 s for the same
/// reason: this runs on the episode hot path.
const NAME_FETCH_TIMEOUT_SECS: u64 = 2;
const TTL_NAMES: i64 = 604_800; // 7 d — verified, names known
const TTL_VERIFIED_EMPTY: i64 = 3_600; // 1 h — verified, no names for this episode
const TTL_TRANSPORT: i64 = 300; // 5 min — could not verify anything

fn year_re() -> &'static fancy_regex::Regex {
    static RE: OnceLock<fancy_regex::Regex> = OnceLock::new();
    RE.get_or_init(|| fancy_regex::Regex::new(r"\b(19\d{2}|20\d{2})\b").expect("year regex"))
}

fn episode_id_re() -> &'static fancy_regex::Regex {
    static RE: OnceLock<fancy_regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        fancy_regex::Regex::new(r"(?i)^(?:tt\d{6,10}|[a-z][a-z0-9_-]*:\d+)$").expect("media id regex")
    })
}

fn placeholder_re() -> &'static fancy_regex::Regex {
    static RE: OnceLock<fancy_regex::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        fancy_regex::Regex::new(r"(?i)^(?:episode|ep|season|s)\s*\d*$").expect("placeholder regex")
    })
}

/// `^(?:19|20)\d{2}$` — the PHP only applies the ±1-year rule to a real year.
fn is_release_year(value: &str) -> bool {
    let b = value.as_bytes();
    b.len() == 4
        && b.iter().all(|c| c.is_ascii_digit())
        && (value.starts_with("19") || value.starts_with("20"))
}

/// The FIRST 4-digit year in `text` (`preg_match('/\b(19\d{2}|20\d{2})\b/')`).
fn first_year(text: &str) -> Option<i32> {
    let caps = year_re().captures(text).ok().flatten()?;
    caps.get(1)?.as_str().parse().ok()
}

/// `preg_replace('/:\d+:\d+$/', '', $mediaId)` — the `:S:E` suffix a Stremio id
/// carries is not part of the metadata id. Only a TRAILING pair of numeric
/// segments is removed, so `pm:post:9000031194` is left alone (and then
/// rejected by the id shape check, exactly like the PHP).
pub fn strip_episode_suffix(media_id: &str) -> String {
    let trimmed = media_id.trim();
    let mut parts: Vec<&str> = trimmed.split(':').collect();
    let n = parts.len();
    if n >= 3 {
        let is_num = |s: &str| !s.is_empty() && s.chars().all(|c| c.is_ascii_digit());
        if is_num(parts[n - 1]) && is_num(parts[n - 2]) {
            parts.truncate(n - 2);
            return parts.join(":");
        }
    }
    trimmed.to_string()
}

/// Port of `fd_episode_meta_identity_matches()` (backend.php:8636).
///
/// **A verified record is the only thing allowed to supply episode titles.**
/// Unknown or mismatched metadata is NOT evidence that a stream is wrong, so
/// every inconclusive branch returns `false` and the caller simply keeps the
/// existing title/year checks. The id must match exactly (case-insensitively),
/// the type must be `series`, a KNOWN title must normalise to the same words,
/// and a KNOWN year that differs by more than one invalidates the record (the
/// catalog title itself may carry years, hence `releaseInfo`).
pub fn episode_meta_identity_matches(
    meta: &Value,
    media_id: &str,
    expected_title: &str,
    expected_year: &str,
) -> bool {
    let actual_id = s(meta, "id");
    if actual_id.is_empty() || !actual_id.eq_ignore_ascii_case(media_id.trim()) {
        return false;
    }
    if s(meta, "type").to_ascii_lowercase() != "series" {
        return false;
    }
    if !expected_title.trim().is_empty() {
        use crate::matcher::norm_title_for_match;
        let actual = norm_title_for_match(&s(meta, "name"));
        let wanted = norm_title_for_match(expected_title);
        if actual.is_empty() || wanted.is_empty() || actual != wanted {
            return false;
        }
    }
    let expected_year = expected_year.trim();
    if is_release_year(expected_year) {
        let mut release = s(meta, "releaseInfo");
        if release.is_empty() {
            release = s(meta, "year");
        }
        if let Some(y) = first_year(&release) {
            if let Ok(want) = expected_year.parse::<i32>() {
                if (y - want).abs() > 1 {
                    return false;
                }
            }
        }
    }
    true
}

/// Port of `fd_extract_episode_names()` (backend.php:8668).
///
/// Only `videos[]` entries whose EXPLICIT `season`/`episode` match are used; an
/// id suffix alone is accepted only when the entry carries no explicit
/// numbering, so a contradictory number is never overridden by the id.
/// Placeholders ("Episode 1", "S01") are skipped, and BOTH `name` and `title`
/// are read because providers alias them.
pub fn extract_episode_names(meta: &Value, season: i32, episode: i32) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let Some(videos) = meta.get("videos").and_then(|v| v.as_array()) else {
        return names;
    };
    for video in videos {
        if !video.is_object() {
            continue;
        }
        let has_season = video.get("season").is_some();
        let has_episode = video.get("episode").is_some() || video.get("number").is_some();
        let v_season = video.get("season").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let v_episode = video
            .get("episode")
            .or_else(|| video.get("number"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        let id = video.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let episode_id_matches = id.ends_with(&format!(":{}:{}", season, episode));
        // Never let an ID suffix override contradictory explicit numbering.
        if (has_season && v_season != season) || (has_episode && v_episode != episode) {
            continue;
        }
        if !(has_season && has_episode) && !episode_id_matches {
            continue;
        }
        for field in ["name", "title"] {
            let name = crate::matcher::normalize_episode_name(&s(video, field));
            if !name.is_empty() && !placeholder_re().is_match(&name).unwrap_or(false) {
                if !names.iter().any(|n| n == &name) {
                    names.push(name);
                }
            }
        }
    }
    names
}

/// Port of `fd_upstream_episode_names()` (backend.php:8747).
///
/// Asks the first **meta-capable** upstream (the resolver list deliberately
/// ignores the master switch — an external id must keep resolving with bridging
/// off) for `/meta/series/<id>.json`, verifies the record's identity, and keeps
/// the episode names of the FIRST verified provider that actually has names —
/// never a union of potentially contradictory providers. Cache: 7 d for names,
/// 1 h for "verified but no names", 5 min for a transport failure, in
/// `storage/cache/episode_names_<md5>.json` (the TTL lives in the file, the
/// freshness is the mtime, exactly like the PHP).
///
/// Returns `[]` for anything it cannot verify — callers must treat that as
/// "no evidence", never as "reject".
pub async fn episode_names(
    settings: &Value,
    media_id: &str,
    season: i32,
    episode: i32,
    expected_title: &str,
    expected_year: &str,
) -> Vec<String> {
    let media_id = strip_episode_suffix(media_id);
    if media_id.is_empty()
        || !episode_id_re().is_match(&media_id).unwrap_or(false)
        || season < 0
        || episode < 1
    {
        return Vec::new();
    }

    let mut urls: Vec<(String, String)> = Vec::new();
    for (i, up) in resolver_manifests(settings)
        .await
        .into_iter()
        .take(MAX_NAME_PROVIDERS)
        .enumerate()
    {
        let manifest = up.url.trim();
        if manifest.is_empty() || !(manifest.starts_with("http://") || manifest.starts_with("https://")) {
            continue;
        }
        urls.push((
            format!("up_{}", i),
            format!("{}/meta/series/{}.json", base_of(manifest), rawurlencode(&media_id)),
        ));
    }
    if urls.is_empty() {
        return Vec::new();
    }

    let cache_key = crate::fileid::md5_hex(
        json!([
            "v3",
            media_id.to_lowercase(),
            season,
            episode,
            crate::matcher::norm_title_for_match(expected_title),
            expected_year.trim(),
            urls.iter().map(|(_, u)| u.clone()).collect::<Vec<_>>(),
        ])
        .to_string()
        .as_bytes(),
    );
    // The TTL is now the cache entry's own deadline (7 d for names, 1 h for
    // "verified but no names", 5 min for a transport failure), instead of a `ttl`
    // field inside the JSON that the reader had to re-interpret against the mtime.
    if let Some(hit) = crate::cache::get("episode_names", &cache_key) {
        if let Some(list) = hit.get("names").and_then(|v| v.as_array()) {
            return list
                .iter()
                .filter_map(|v| v.as_str().map(|x| x.to_string()))
                .collect();
        }
    }

    // Bounded fan-out, no API secret (a third-party addon must never see it).
    let bodies = futures_util::future::join_all(urls.iter().map(|(_, url)| {
        let url = url.clone();
        async move { crate::httpc::get_contents(&url, NAME_FETCH_TIMEOUT_SECS, HashMap::new()).await }
    }))
    .await;

    let mut verified = false;
    let mut names: Vec<String> = Vec::new();
    let mut source = String::new();
    for (i, body) in bodies.iter().enumerate() {
        let Some(text) = body else { continue };
        let Ok(doc) = serde_json::from_str::<Value>(text) else {
            continue;
        };
        let meta = doc.get("meta").unwrap_or(&Value::Null);
        if !meta.is_object() {
            continue;
        }
        if !episode_meta_identity_matches(meta, &media_id, expected_title, expected_year) {
            continue;
        }
        verified = true;
        let candidate = extract_episode_names(meta, season, episode);
        if !candidate.is_empty() {
            names = candidate;
            source = urls[i].0.clone();
            break;
        }
    }

    let ttl = if !names.is_empty() {
        TTL_NAMES
    } else if verified {
        TTL_VERIFIED_EMPTY
    } else {
        TTL_TRANSPORT
    };
    crate::cache::put(
        "episode_names",
        &cache_key,
        &json!({"names": names, "ttl": ttl}),
        ttl,
        0,
    );
    log(
        "episode metadata lookup",
        &json!({
            "media_id": media_id,
            "season": season,
            "episode": episode,
            "verified": verified,
            "source": source,
            "name_count": names.len(),
        }),
    );
    names
}

/// Titles of the OTHER episodes of the same season.
///
/// These exist to keep the episode-title guard strict in the RIGHT direction.
/// A file whose post-marker text names a DIFFERENT known episode is that other
/// episode and must stay out; a file whose text matches nothing we know is
/// simply named differently and must NOT be dropped.
///
/// Measured, and the reason this exists: Disney's "Oswald" is numbered 1..52 by
/// theTVDB, while the uploader ships TWO segments per file — TVDB calls S01E02
/// "The Camping Trip" and the file says "S01E02 Sleepover & Big Banana". The
/// guard read that as "another episode's title" and rejected every Oswald file
/// but the combined packs, so every episode answered "No Streams Found" while
/// the origin held 50 of them.
///
/// Same cache discipline as [`episode_names()`]: 7 d for names, 1 h for
/// "verified but no names", 5 min for a transport failure.
pub async fn sibling_episode_names(
    settings: &Value,
    media_id: &str,
    season: i32,
    episode: i32,
    expected_title: &str,
    expected_year: &str,
) -> Vec<String> {
    let media_id = strip_episode_suffix(media_id);
    if media_id.is_empty()
        || !episode_id_re().is_match(&media_id).unwrap_or(false)
        || season < 1
        || episode < 1
    {
        return Vec::new();
    }
    let mut urls: Vec<(String, String)> = Vec::new();
    for (i, up) in resolver_manifests(settings)
        .await
        .into_iter()
        .take(MAX_NAME_PROVIDERS)
        .enumerate()
    {
        let manifest = up.url.trim();
        if manifest.is_empty() || !(manifest.starts_with("http://") || manifest.starts_with("https://")) {
            continue;
        }
        urls.push((
            format!("up_{}", i),
            format!("{}/meta/series/{}.json", base_of(manifest), rawurlencode(&media_id)),
        ));
    }
    if urls.is_empty() {
        return Vec::new();
    }
    let cache_key = crate::fileid::md5_hex(
        json!([
            "v1-siblings",
            media_id.to_lowercase(),
            season,
            episode,
            crate::matcher::norm_title_for_match(expected_title),
            expected_year.trim(),
            urls.iter().map(|(_, u)| u.clone()).collect::<Vec<_>>(),
        ])
        .to_string()
        .as_bytes(),
    );
    if let Some(hit) = crate::cache::get("episode_siblings", &cache_key) {
        if let Some(list) = hit.get("names").and_then(|v| v.as_array()) {
            return list
                .iter()
                .filter_map(|v| v.as_str().map(|x| x.to_string()))
                .collect();
        }
    }
    let bodies = futures_util::future::join_all(urls.iter().map(|(_, url)| {
        let url = url.clone();
        async move { crate::httpc::get_contents(&url, NAME_FETCH_TIMEOUT_SECS, HashMap::new()).await }
    }))
    .await;
    let mut verified = false;
    let mut names: Vec<String> = Vec::new();
    for body in bodies.iter() {
        let Some(text) = body else { continue };
        let Ok(doc) = serde_json::from_str::<Value>(text) else {
            continue;
        };
        let meta = doc.get("meta").unwrap_or(&Value::Null);
        if !meta.is_object() || !episode_meta_identity_matches(meta, &media_id, expected_title, expected_year) {
            continue;
        }
        verified = true;
        let candidate = extract_sibling_names(meta, season, episode);
        if !candidate.is_empty() {
            names = candidate;
            break;
        }
    }
    let ttl = if !names.is_empty() {
        TTL_NAMES
    } else if verified {
        TTL_VERIFIED_EMPTY
    } else {
        TTL_TRANSPORT
    };
    crate::cache::put(
        "episode_siblings",
        &cache_key,
        &json!({"names": names, "ttl": ttl}),
        ttl,
        0,
    );
    names
}

/// The season a meta's OWN video entry for `item_id` belongs to, or `0`.
///
/// An addon can publish one season of a show as a separate item whose episodes
/// are numbered from 1: AIOMetadata lists Black Clover season 2 as `kitsu:50024`
/// with 13 videos, and the ids of those videos (`kitsu:50024:1` … `:13`) are the
/// ids the client asks about. The parent record still says which SEASON each id
/// belongs to — measured: `kitsu:50024:2` carries `season: 2` while
/// `kitsu:13209:16` carries `season: 1` — which is the only thing that tells the
/// two apart, because the metadata's `name` is just "Black Clover" for both.
pub fn video_season_episode_for_item(meta: &Value, item_id: &str) -> Option<(i32, i32)> {
    let want = item_id.trim().to_ascii_lowercase();
    if want.is_empty() {
        return None;
    }
    let videos = meta.get("videos").and_then(|v| v.as_array())?;
    for video in videos {
        let id = video
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_ascii_lowercase();
        if id != want {
            continue;
        }
        let season = video.get("season").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let episode = video
            .get("episode")
            .or_else(|| video.get("number"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        if season > 0 && episode > 0 {
            return Some((season, episode));
        }
    }
    None
}

/// [`video_season_for_item()`] for a live item: fetches the same meta
/// [`episode_names()`] does (so the second call is a cache hit) and reports the
/// season the item's own video entry belongs to.
#[allow(clippy::too_many_arguments)]
pub async fn episode_item_position(
    settings: &Value,
    media_id: &str,
    item_id: &str,
    expected_title: &str,
    expected_year: &str,
) -> Option<(i32, i32)> {
    let media_id = strip_episode_suffix(media_id);
    if media_id.is_empty() || item_id.trim().is_empty() {
        return None;
    }
    let mut urls: Vec<String> = Vec::new();
    for up in resolver_manifests(settings)
        .await
        .into_iter()
        .take(MAX_NAME_PROVIDERS)
    {
        let manifest = up.url.trim();
        if manifest.is_empty() || !(manifest.starts_with("http://") || manifest.starts_with("https://")) {
            continue;
        }
        urls.push(format!(
            "{}/meta/series/{}.json",
            base_of(manifest),
            rawurlencode(&media_id)
        ));
    }
    if urls.is_empty() {
        return None;
    }
    let bodies = futures_util::future::join_all(urls.iter().map(|url| {
        let url = url.clone();
        async move { crate::httpc::get_contents(&url, NAME_FETCH_TIMEOUT_SECS, HashMap::new()).await }
    }))
    .await;
    for body in bodies.iter() {
        let Some(text) = body else { continue };
        let Ok(doc) = serde_json::from_str::<Value>(text) else {
            continue;
        };
        let meta = doc.get("meta").unwrap_or(&Value::Null);
        if !meta.is_object()
            || !episode_meta_identity_matches(meta, &media_id, expected_title, expected_year)
        {
            continue;
        }
        if let Some(pos) = video_season_episode_for_item(meta, item_id) {
            return Some(pos);
        }
    }
    None
}

/// The ABSOLUTE episode number of `(season, episode)`, or `0` when the meta
/// cannot say.
///
/// Plenty of long-running shows are uploaded as ONE continuous run
/// (`Naruto.E84.mkv`, no season marker) while the addon splits them into
/// theTVDB seasons — Naruto is S1 35, S2 48, S3 48 … so S03E01 is absolute
/// episode 84. Read literally, S03E01 matches nothing: the file parses as
/// season 1 episode 84, and every season past the first answered "No Streams
/// Found" while the origin held all 220 episodes.
///
/// Only a show whose earlier seasons are present in the meta can be mapped, and
/// a season whose episodes already start at 1 maps to `sum(previous) + episode`.
pub fn absolute_episode_from_videos(meta: &Value, season: i32, episode: i32) -> i32 {
    if season < 1 || episode < 1 {
        return 0;
    }
    let Some(videos) = meta.get("videos").and_then(|v| v.as_array()) else {
        return 0;
    };
    let mut before: i32 = 0;
    let mut this_min: i32 = i32::MAX;
    let mut seen_season = false;
    for video in videos {
        let s = video.get("season").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let e = video
            .get("episode")
            .or_else(|| video.get("number"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        // Specials (season 0) are not part of the continuous run.
        if s < 1 || e < 1 {
            continue;
        }
        if s == season {
            seen_season = true;
            if e < this_min {
                this_min = e;
            }
        }
        if s < season {
            before += 1;
        }
    }
    if !seen_season || before == 0 {
        return 0;
    }
    let offset = if this_min == i32::MAX || this_min <= 1 {
        episode
    } else {
        episode - this_min + 1
    };
    if offset < 1 {
        return 0;
    }
    before + offset
}

/// [`absolute_episode_from_videos()`] for a live item (same meta fetch/cache as
/// [`episode_names()`]).
pub async fn absolute_episode(
    settings: &Value,
    media_id: &str,
    season: i32,
    episode: i32,
    expected_title: &str,
    expected_year: &str,
) -> i32 {
    let media_id = strip_episode_suffix(media_id);
    if media_id.is_empty() || season < 2 || episode < 1 {
        return 0;
    }
    let mut urls: Vec<String> = Vec::new();
    for up in resolver_manifests(settings)
        .await
        .into_iter()
        .take(MAX_NAME_PROVIDERS)
    {
        let manifest = up.url.trim();
        if manifest.is_empty() || !(manifest.starts_with("http://") || manifest.starts_with("https://")) {
            continue;
        }
        urls.push(format!(
            "{}/meta/series/{}.json",
            base_of(manifest),
            rawurlencode(&media_id)
        ));
    }
    if urls.is_empty() {
        return 0;
    }
    let bodies = futures_util::future::join_all(urls.iter().map(|url| {
        let url = url.clone();
        async move { crate::httpc::get_contents(&url, NAME_FETCH_TIMEOUT_SECS, HashMap::new()).await }
    }))
    .await;
    for body in bodies.iter() {
        let Some(text) = body else { continue };
        let Ok(doc) = serde_json::from_str::<Value>(text) else {
            continue;
        };
        let meta = doc.get("meta").unwrap_or(&Value::Null);
        if !meta.is_object()
            || !episode_meta_identity_matches(meta, &media_id, expected_title, expected_year)
        {
            continue;
        }
        let abs = absolute_episode_from_videos(meta, season, episode);
        if abs > 0 {
            return abs;
        }
    }
    0
}

/// Every episode title of `season` EXCEPT the requested episode, normalised and
/// de-duplicated. Same field rules as [`extract_episode_names()`].
pub fn extract_sibling_names(meta: &Value, season: i32, episode: i32) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let Some(videos) = meta.get("videos").and_then(|v| v.as_array()) else {
        return names;
    };
    for video in videos {
        if !video.is_object() {
            continue;
        }
        let v_season = video.get("season").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        if v_season != season {
            continue;
        }
        let v_episode = video
            .get("episode")
            .or_else(|| video.get("number"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        if v_episode == episode {
            continue;
        }
        for field in ["name", "title"] {
            let name = crate::matcher::normalize_episode_name(&s(video, field));
            if !name.is_empty() && !placeholder_re().is_match(&name).unwrap_or(false) {
                if !names.iter().any(|n| n == &name) {
                    names.push(name);
                }
            }
        }
        if names.len() >= 80 {
            break;
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(enabled: bool, manifests: Value) -> Value {
        json!({"upstream_enabled": enabled, "upstream_manifests": manifests})
    }

    #[test]
    fn has_resource_accepts_both_shapes() {
        let s = json!({"resources": ["catalog", "meta"]});
        assert!(has_resource(&s, "catalog"));
        assert!(has_resource(&s, "META"));
        assert!(!has_resource(&s, "stream"));
        let o = json!({"resources": [{"name": "subtitles", "types": ["movie"]}]});
        assert!(has_resource(&o, "subtitles"));
        assert!(!has_resource(&o, "meta"));
        assert!(!has_resource(&json!({}), "catalog"));
        assert!(!has_resource(&json!({"resources": "catalog"}), "catalog"));
    }

    #[tokio::test]
    async fn catalogs_disabled_drops_bridged_rows_too() {
        // PHP gates the WHOLE manifest list (local + upstream) on
        // `catalogs_enabled`; gating only the local rows left 40 bridged
        // catalogs in the descriptor (measured).
        let settings = json!({
            "catalogs_enabled": false,
            "upstream_enabled": true,
            "upstream_manifests": [{"url": "https://example.invalid/manifest.json", "name": "X"}]
        });
        let (cats, types) = bridged_catalogs(&settings).await;
        assert!(cats.is_empty());
        assert!(types.is_empty());
    }

    #[test]
    fn configured_respects_master_switch_and_skips_url_less_entries() {
        let list = json!([
            {"url": "", "name": "empty"},
            {"url": "https://a/manifest.json", "name": "A"},
            {"url": "https://b/manifest.json"}
        ]);
        let on = configured(&settings(true, list.clone()), true);
        assert_eq!(on.len(), 2);
        // The index is the STORED position, so an empty entry in between must
        // not renumber the ones after it (`up_<index>_…` is what clients ask).
        assert_eq!(on[0].index, 1);
        assert_eq!(on[1].index, 2);
        assert_eq!(on[1].name, "Addon");
        assert_eq!(on[1].id, "up2");
        assert!(configured(&settings(false, list.clone()), true).is_empty());
        // The resolver list ignores the master switch.
        assert_eq!(configured(&settings(false, list), false).len(), 2);
    }

    #[test]
    fn parse_bridged_catalog_ids() {
        assert_eq!(
            parse_bridged("up_0_tmdb.top"),
            Some((0, "tmdb.top".to_string()))
        );
        assert_eq!(
            parse_bridged("up_12_some_cat"),
            Some((12, "some_cat".to_string()))
        );
        assert_eq!(parse_bridged("pm_movies"), None);
        assert_eq!(parse_bridged("up_x_tmdb"), None);
        assert_eq!(parse_bridged("up_0_"), None);
        assert_eq!(parse_bridged("up_0"), None);
    }

    #[test]
    fn base_of_strips_the_manifest_filename_only() {
        assert_eq!(
            base_of("https://ai.example.com/stremio/abc/manifest.json"),
            "https://ai.example.com/stremio/abc"
        );
        assert_eq!(
            base_of("https://ai.example.com/x/MANIFEST.JSON?k=1"),
            "https://ai.example.com/x"
        );
        assert_eq!(base_of("https://x.example.com/"), "https://x.example.com");
        assert_eq!(base_of("https://x.example.com"), "https://x.example.com");
    }

    #[test]
    fn fit_descriptor_drops_only_bridged_catalogs() {
        let local: Vec<Value> = (0..4)
            .map(|i| json!({"type": "movie", "id": format!("pm_{}", i), "name": "x"}))
            .collect();
        let bridged: Vec<Value> = (0..400)
            .map(|i| {
                json!({
                    "type": "movie",
                    "id": format!("up_0_cat{}", i),
                    "name": "A very chatty upstream catalog name",
                    "extra": [{"name": "genre", "options": ["a","b","c","d","e","f"]}]
                })
            })
            .collect();
        let mut all = local.clone();
        all.extend(bridged);
        let doc = json!({"catalogs": all});
        assert!(serde_json::to_string(&doc).unwrap().len() > MANIFEST_DESCRIPTOR_BUDGET);

        let fitted = fit_descriptor(doc, local.len());
        let text = serde_json::to_string(&fitted).unwrap();
        assert!(
            text.len() <= MANIFEST_DESCRIPTOR_BUDGET,
            "{} bytes",
            text.len()
        );
        let cats = fitted["catalogs"].as_array().unwrap();
        assert_eq!(&cats[..4], &local[..]);
        // Trailing bridged catalogs are the ones dropped.
        assert!(cats.len() < 404);
        assert!(cats.len() > 4);
    }

    #[test]
    fn fit_descriptor_keeps_every_local_row_even_with_a_big_upstream() {
        // The descriptor now advertises the PHP's full local set (29 rows + the
        // port's 2 trending rows + up to 5 keyword rows = 36). An upstream burst
        // must never eat into those, whatever the budget pressure.
        let local: Vec<Value> = (0..36)
            .map(|i| json!({"type": "movie", "id": format!("pm_{i}"), "name": "Row", "extra": [{"name": "skip"}]}))
            .collect();
        let bridged: Vec<Value> = (0..400)
            .map(|i| {
                json!({
                    "type": "movie",
                    "id": format!("up_0_cat{}", i),
                    "name": "A very chatty upstream catalog name",
                    "extra": [{"name": "genre", "options": ["a","b","c","d","e","f"]}]
                })
            })
            .collect();
        let mut all = local.clone();
        all.extend(bridged);
        let fitted = fit_descriptor(json!({"catalogs": all}), local.len());
        let cats = fitted["catalogs"].as_array().unwrap();
        assert_eq!(&cats[..36], &local[..], "every local row survives, in order");
        assert!(serde_json::to_string(&fitted).unwrap().len() <= MANIFEST_DESCRIPTOR_BUDGET);
        // When the LOCAL rows alone exceed the budget nothing is dropped either:
        // a descriptor with no catalogs at all is worse than a large one.
        let big: Vec<Value> = (0..40)
            .map(|i| json!({"type": "movie", "id": format!("pm_{i}"), "name": "x".repeat(2000)}))
            .collect();
        let fitted = fit_descriptor(json!({"catalogs": big.clone()}), big.len());
        assert_eq!(fitted["catalogs"].as_array().unwrap().len(), 40);
    }

    #[test]
    fn default_manifests_are_the_php_defaults() {
        let m = default_manifests();
        let list = m.as_array().unwrap();
        assert_eq!(list.len(), 2);
        assert!(s(&list[0], "url").contains("aiometadata"));
        assert_eq!(s(&list[0], "name"), "AIOMetadata");
        assert!(s(&list[1], "url").contains("opensubtitles-v3"));
        assert_eq!(s(&list[1], "name"), "OpenSubtitles v3");
    }

    /* ── verified episode names (fd_upstream_episode_names) ─────────────────── */

    fn series_meta(id: &str, name: &str, release: &str, videos: Value) -> Value {
        json!({"id": id, "type": "series", "name": name, "releaseInfo": release, "videos": videos})
    }

    #[test]
    fn strip_episode_suffix_only_removes_trailing_numbers() {
        assert_eq!(strip_episode_suffix("tt0944947:8:6"), "tt0944947");
        assert_eq!(strip_episode_suffix("tt0388629:1:1"), "tt0388629");
        assert_eq!(strip_episode_suffix("  tt11540908  "), "tt11540908");
        // A 2-part id keeps its scalar value, and a post id must NOT be
        // truncated into something that accidentally validates.
        assert_eq!(strip_episode_suffix("tmdb:123"), "tmdb:123");
        assert_eq!(strip_episode_suffix("pm:post:9000031194"), "pm:post:9000031194");
        assert_eq!(strip_episode_suffix("tt0944947:8"), "tt0944947:8");
    }

    #[test]
    fn episode_meta_identity_requires_id_type_title_and_year() {
        let meta = series_meta("tt11540908", "Carrie", "2026-", json!([]));
        assert!(episode_meta_identity_matches(&meta, "tt11540908", "Carrie 2026 • TvSeries", "2026"));
        // The id must match (case-insensitively) and never be empty.
        assert!(!episode_meta_identity_matches(&meta, "tt11540909", "Carrie", "2026"));
        let no_id = series_meta("", "Carrie", "2026-", json!([]));
        assert!(!episode_meta_identity_matches(&no_id, "tt11540908", "Carrie", "2026"));
        // Only `series` records may supply episode titles.
        let movie = json!({"id": "tt11540908", "type": "movie", "name": "Carrie"});
        assert!(!episode_meta_identity_matches(&movie, "tt11540908", "Carrie", ""));
        // A KNOWN title must normalise to the same words (decorations, the year
        // and a trailing type word are stripped by norm_title_for_match).
        assert!(!episode_meta_identity_matches(&meta, "tt11540908", "Carrie 2002", "2002"));
        let nameless = json!({"id": "tt11540908", "type": "series", "name": ""});
        assert!(!episode_meta_identity_matches(&nameless, "tt11540908", "Carrie", ""));
        // An unknown expected title is inconclusive, so it does not reject.
        assert!(episode_meta_identity_matches(&meta, "tt11540908", "", ""));
        // A year difference > 1 invalidates; ±1 and an unknown upstream year do not.
        assert!(!episode_meta_identity_matches(&meta, "tt11540908", "Carrie", "2022"));
        assert!(episode_meta_identity_matches(&meta, "tt11540908", "Carrie", "2025"));
        let no_year = json!({"id": "tt11540908", "type": "series", "name": "Carrie"});
        assert!(episode_meta_identity_matches(&no_year, "tt11540908", "Carrie", "2026"));
        // A non-year expected value skips the guard entirely (PHP regex).
        assert!(episode_meta_identity_matches(&no_year, "tt11540908", "Carrie", "season 1"));
    }

    #[test]
    fn episode_names_are_extracted_only_for_the_requested_episode() {
        let meta = series_meta(
            "tt11540908",
            "Carrie",
            "2026-",
            json!([
                {"id": "tt11540908:1:1", "season": 1, "episode": 1, "title": "FIRST PERIOD"},
                {"id": "tt11540908:1:2", "season": 1, "episode": 2, "title": "FOREIGN LANGUAGE"},
                {"id": "tt11540908:0:1", "season": 0, "episode": 1, "title": "Behind the Scenes"}
            ]),
        );
        assert_eq!(extract_episode_names(&meta, 1, 1), vec!["first period".to_string()]);
        assert_eq!(extract_episode_names(&meta, 1, 2), vec!["foreign language".to_string()]);
        // A season-0 special must never answer a season-1 request.
        assert!(extract_episode_names(&meta, 1, 3).is_empty());
        assert!(extract_episode_names(&meta, 2, 1).is_empty());
    }

    /// Reported: Naruto **season 3** answered "No Streams Found" while the origin
    /// held `Naruto.E84.mkv` — one continuous run, no season markers, against
    /// theTVDB's 5-way split (S1 35, S2 48, S3 48 …).
    #[test]
    fn absolute_episode_counts_the_earlier_seasons() {
        let mut videos: Vec<serde_json::Value> = Vec::new();
        for e in 1..=35 {
            videos.push(serde_json::json!({"season": 1, "episode": e}));
        }
        for e in 1..=48 {
            videos.push(serde_json::json!({"season": 2, "episode": e}));
        }
        for e in 1..=48 {
            videos.push(serde_json::json!({"season": 3, "episode": e}));
        }
        // A special must not shift the run.
        videos.push(serde_json::json!({"season": 0, "episode": 1}));
        let meta = serde_json::json!({"videos": videos});
        assert_eq!(absolute_episode_from_videos(&meta, 3, 1), 84);
        assert_eq!(absolute_episode_from_videos(&meta, 3, 48), 131);
        assert_eq!(absolute_episode_from_videos(&meta, 2, 1), 36);
        // Season 1 needs no mapping; a season the meta does not list is unmappable.
        assert_eq!(absolute_episode_from_videos(&meta, 1, 5), 0);
        assert_eq!(absolute_episode_from_videos(&meta, 4, 1), 0);
        assert_eq!(absolute_episode_from_videos(&serde_json::json!({}), 3, 1), 0);
    }

    /// Reported: Black Clover season 2 listed season 1's files, and Naruto's
    /// `kitsu:11:84` (S03E01) answered "No Streams Found". The addon's `name` is
    /// the same ("Black Clover", "Naruto") whichever id you ask about, so the
    /// ONLY evidence is the video entry: it carries both the season AND the
    /// season-relative episode number.
    #[test]
    fn a_video_entry_knows_its_own_season() {
        let meta = serde_json::json!({"videos": [
            {"id": "kitsu:13209:16", "season": 1, "episode": 16, "title": "The Diamond Mage"},
            {"id": "kitsu:50024:2", "season": 2, "episode": 2, "title": "The Door to Hell"},
            {"id": "kitsu:11:84", "season": 3, "episode": 1, "title": "Roar, Chidori! Brother vs. Brother!"},
        ]});
        // An anime id's trailing number is ABSOLUTE; the entry says where it goes.
        assert_eq!(video_season_episode_for_item(&meta, "kitsu:11:84"), Some((3, 1)));
        assert_eq!(video_season_episode_for_item(&meta, "kitsu:50024:2"), Some((2, 2)));
        assert_eq!(video_season_episode_for_item(&meta, "kitsu:13209:16"), Some((1, 16)));
        // Case and whitespace are not identity.
        assert_eq!(video_season_episode_for_item(&meta, " KITSU:50024:2 "), Some((2, 2)));
        // An id the meta does not publish is not evidence.
        assert_eq!(video_season_episode_for_item(&meta, "kitsu:50024:9"), None);
        assert_eq!(video_season_episode_for_item(&serde_json::json!({}), "kitsu:1"), None);
        assert_eq!(video_season_episode_for_item(&meta, ""), None);
    }

    #[test]
    fn episode_id_suffix_never_overrides_explicit_numbering() {
        // The id says :1:1 but the entry's own numbering says S1E2 — contradictory
        // metadata is dropped instead of trusting the suffix.
        let meta = series_meta(
            "tt11540908",
            "Carrie",
            "2026-",
            json!([
                {"id": "tt11540908:1:1", "season": 1, "episode": 2, "title": "Foreign Language"},
                {"id": "tt11540908:1:1", "title": "First Period"}
            ]),
        );
        assert_eq!(extract_episode_names(&meta, 1, 1), vec!["first period".to_string()]);
        // With no explicit numbering anywhere, the id suffix is the only evidence.
        let id_only = series_meta(
            "tt11540908",
            "Carrie",
            "2026-",
            json!([{"id": "tt11540908:1:1", "title": "First Period"}]),
        );
        assert_eq!(extract_episode_names(&id_only, 1, 1), vec!["first period".to_string()]);
        assert!(extract_episode_names(&id_only, 1, 2).is_empty());
    }

    #[test]
    fn episode_name_placeholders_and_aliases_are_handled() {
        let meta = series_meta(
            "tt11540908",
            "Carrie",
            "2026-",
            json!([
                {"id": "tt11540908:1:1", "season": 1, "episode": 1, "name": "Episode 1", "title": "1. First Period"},
                {"id": "tt11540908:1:1", "season": 1, "episode": 1, "name": "First Period"}
            ]),
        );
        // The alias adds nothing new, and the placeholder is never a name.
        assert_eq!(extract_episode_names(&meta, 1, 1), vec!["first period".to_string()]);
        // A record with only a placeholder yields NO names — verified-but-empty,
        // which the caller treats as "no evidence" (fail open).
        let only_placeholder = series_meta(
            "tt11540908",
            "Carrie",
            "2026-",
            json!([{"id": "tt11540908:1:1", "season": 1, "episode": 1, "title": "Episode 1"}]),
        );
        assert!(extract_episode_names(&only_placeholder, 1, 1).is_empty());
        // A release year is a YEAR, checked against the ±1 rule.
        assert_eq!(first_year("2026-"), Some(2026));
        assert_eq!(first_year("2011-2019"), Some(2011));
        assert_eq!(first_year("no year here"), None);
        assert!(is_release_year("2026"));
        assert!(!is_release_year("26"));
        assert!(!is_release_year("1999-"));
        assert!(!is_release_year("season 1"));
    }

    #[test]
    fn media_id_shape_gate_rejects_our_own_post_ids() {
        // The same gate the PHP applies before asking any provider:
        // `^(?:tt\d{6,10}|[a-z][a-z0-9_-]*:\d+)$`.
        for ok in ["tt0944947", "tt11540908", "tmdb:123", "kitsu:12", "anilist:21", "tvmaze:82"] {
            assert!(episode_id_re().is_match(ok).unwrap_or(false), "{ok} should pass");
        }
        for bad in ["pm:post:9000031194", "pm_file_1v9OPj", "tt123", "", "tt0944947:8:6"] {
            assert!(!episode_id_re().is_match(bad).unwrap_or(false), "{bad} should fail");
        }
        // Single-colon shapes pass the gate on purpose (`post:9000031194` matches
        // the PHP regex too); the caller only ever passes an imdb id or the
        // catalog id, and the `pm:post:` form is what gets rejected.
        assert!(episode_id_re().is_match("post:9000031194").unwrap_or(false));
    }
}
