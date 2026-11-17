//! Per-link public configuration.
//!
//! The public Configure page puts the user's choices in the FIRST path segment,
//! exactly the shape the Stremio add-on SDK documents (`/:config?/manifest.json`):
//!
//! ```text
//! https://host/<base64url(JSON)>/manifest.json
//! ```
//!
//! There is no `/p/` and no `v1.` prefix. The version lives INSIDE the document
//! (`"v": 1`) so a future schema can coexist without changing the URL grammar.
//! The payload is base64url of a JSON object, so it always starts with `eyJ`
//! (base64 of `{"`), which is never a 32+ hex admin token; the router can tell
//! the two apart without a namespace.
//!
//! The merge is deliberately one-way: a public link can hide a catalog, disable
//! a source, or tighten a filter, but it can never turn on something the operator
//! turned off or add a URL the server would fetch. That is the whole security
//! model of the stateless design.

use std::collections::BTreeMap;

use base64::Engine;
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

/// A config that decodes larger than this is rejected before parsing. The
/// configure page keeps generated URLs well under this; the cap is here so a
/// hand-made URL cannot make the server allocate or parse an unbounded blob.
pub const MAX_DECODED: usize = 4096;

const MAX_HIDDEN_CATALOGS: usize = 128;
const MAX_DISABLED_SOURCES: usize = 64;
const MAX_STREAM_KEYS: usize = 64;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PublicConfig {
    /// `catalogs_enabled` request; `None` means "use the operator default".
    pub catalogs_enabled: Option<bool>,
    /// Media-type bitmask: 1 movie, 2 series, 4 other. `None`/0 = no override.
    pub types: Option<u8>,
    /// Catalog ids the user wants hidden. Hide-only.
    pub hide_catalogs: Vec<String>,
    /// Catalog ids the user wants shown even though the operator's default is
    /// off. Built-in catalog rows only.
    pub show_catalogs: Vec<String>,
    /// Legacy `d` field from older public links. Decoded for compatibility,
    /// ignored by [`merge`]: public links no longer control sources.
    pub disable_sources: Vec<String>,
    /// Legacy `u` field from older public links. Decoded for compatibility,
    /// ignored by [`merge`].
    pub source_overrides: BTreeMap<String, bool>,
    /// Optional user-owned loopback server (`127.0.0.1:8088`) for this link.
    pub local_node: Option<String>,
    /// A subset of `stream_config`; validated by [`apply_public_stream_config`].
    pub stream: Map<String, Value>,
}

/// The first segment of a personalized addon URL, plus the remainder.
///
/// Returns `None` for a 32+ hex token (the existing `/<token>/...` transport),
/// for normal routes (`/manifest.json`, `/api/...`, `/catalog/...`), and for an
/// empty path. A segment that *looks* like a config (`eyJ...`) is returned even
/// when it later fails to decode, so the caller can answer a helpful
/// "invalid configuration" manifest instead of a 404.
pub fn parse_segment(raw_path: &str) -> Option<(String, String)> {
    let rest = raw_path.strip_prefix('/')?;
    if rest.is_empty() {
        return None;
    }
    let (seg, tail) = match rest.split_once('/') {
        Some((s, t)) => (s, format!("/{t}")),
        None => (rest, "/".to_string()),
    };
    if is_token(seg) || !looks_like_config(seg) {
        return None;
    }
    Some((seg.to_string(), tail))
}

/// A config payload is base64url of a JSON object, so it starts with `eyJ`.
/// The check is intentionally narrow: it is what keeps the router from trying
/// to decode every unknown path as a config.
pub fn looks_like_config(seg: &str) -> bool {
    seg.starts_with("eyJ")
        && seg.len() <= 6_000
        && seg
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'=')
}

fn is_token(seg: &str) -> bool {
    seg.len() >= 32 && seg.chars().all(|c| c.is_ascii_hexdigit())
}

/// The only local node a public link may name: the user's own loopback server.
/// This is emitted to the client, never fetched by this process, so strict loopback
/// validation is enough to keep the link from becoming an open redirect target.
pub fn valid_local_node(raw: &str) -> Option<String> {
    let value = raw.trim().trim_end_matches('/');
    match value.to_ascii_lowercase().as_str() {
        "http://127.0.0.1:8088" | "http://localhost:8088" | "http://[::1]:8088" => {
            Some(value.to_ascii_lowercase())
        }
        _ => None,
    }
}

pub fn decode(payload: &str) -> Result<PublicConfig, String> {
    let raw = payload.trim();
    if raw.is_empty() {
        return Err("empty config".to_string());
    }
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(raw.trim_end_matches('='))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(raw))
        .map_err(|_| "invalid base64".to_string())?;
    if bytes.len() > MAX_DECODED {
        return Err(format!("config is {} bytes (max {MAX_DECODED})", bytes.len()));
    }
    let doc: Value = serde_json::from_slice(&bytes).map_err(|_| "invalid JSON".to_string())?;
    let obj = doc
        .as_object()
        .ok_or_else(|| "config must be a JSON object".to_string())?;
    if obj.get("v").and_then(|v| v.as_i64()) != Some(1) {
        return Err("unsupported config version".to_string());
    }

    let catalogs_enabled = obj.get("c").and_then(boolish);
    let types = obj
        .get("t")
        .and_then(|v| v.as_u64())
        .filter(|n| (1..=7).contains(n))
        .map(|n| n as u8);

    let hide_catalogs = string_list(obj.get("h"), MAX_HIDDEN_CATALOGS);
    let show_catalogs = string_list(obj.get("e"), MAX_HIDDEN_CATALOGS);
    let disable_sources = string_list(obj.get("d"), MAX_DISABLED_SOURCES)
        .into_iter()
        .filter(|s| s.len() == 12 && s.chars().all(|c| c.is_ascii_hexdigit()))
        .collect();
    let source_overrides: BTreeMap<String, bool> = obj
        .get("u")
        .and_then(|v| v.as_object())
        .map(|map| {
            map.iter()
                .filter_map(|(key, value)| {
                    let key = key.trim();
                    if key.len() != 12 || !key.chars().all(|c| c.is_ascii_hexdigit()) {
                        return None;
                    }
                    value.as_bool().map(|on| (key.to_string(), on))
                })
                .take(MAX_DISABLED_SOURCES)
                .collect()
        })
        .unwrap_or_default();
    let local_node = obj
        .get("l")
        .and_then(|v| v.as_str())
        .and_then(valid_local_node);

    let stream = match obj.get("s") {
        Some(Value::Object(map)) if map.len() <= MAX_STREAM_KEYS => map.clone(),
        Some(Value::Object(_)) => return Err("too many stream keys".to_string()),
        Some(Value::Null) | None => Map::new(),
        Some(_) => return Err("s must be an object".to_string()),
    };

    Ok(PublicConfig {
        catalogs_enabled,
        types,
        hide_catalogs,
        show_catalogs,
        disable_sources,
        source_overrides,
        local_node,
        stream,
    })
}

fn boolish(v: &Value) -> Option<bool> {
    match v {
        Value::Bool(b) => Some(*b),
        Value::Number(n) => n.as_i64().map(|i| i != 0),
        _ => None,
    }
}

fn string_list(v: Option<&Value>, max: usize) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    if let Some(list) = v.and_then(|x| x.as_array()) {
        for item in list.iter().take(max) {
            let Some(s) = item.as_str() else { continue };
            let s = s.trim().to_string();
            if !s.is_empty() && !out.contains(&s) {
                out.push(s);
            }
        }
    }
    out
}

/// A short id for the INSTALL URL itself. The manifest id must change when a
/// catalog-only choice changes too, so this hashes the payload, not the stream
/// response fingerprint (which deliberately ignores catalog choices so the
/// stream cache can be shared).
pub fn payload_id(payload: &str) -> String {
    let digest = Sha256::digest(payload.trim().as_bytes());
    hex::encode(&digest[..6])
}

pub fn to_json(cfg: &PublicConfig) -> Value {
    let mut out = Map::new();
    out.insert("v".to_string(), json!(1));
    if let Some(c) = cfg.catalogs_enabled {
        out.insert("c".to_string(), json!(if c { 1 } else { 0 }));
    }
    if let Some(t) = cfg.types {
        out.insert("t".to_string(), json!(t));
    }
    if !cfg.show_catalogs.is_empty() {
        out.insert("e".to_string(), json!(cfg.show_catalogs));
    }
    if !cfg.hide_catalogs.is_empty() {
        out.insert("h".to_string(), json!(cfg.hide_catalogs));
    }
    if let Some(local) = &cfg.local_node {
        out.insert("l".to_string(), json!(local));
    }
    if !cfg.source_overrides.is_empty() {
        out.insert("u".to_string(), json!(cfg.source_overrides));
    }
    if !cfg.disable_sources.is_empty() {
        out.insert("d".to_string(), json!(cfg.disable_sources));
    }
    if !cfg.stream.is_empty() {
        out.insert("s".to_string(), Value::Object(cfg.stream.clone()));
    }
    Value::Object(out)
}

/// Merge a public config over the operator's settings.
///
/// The public link owns its own presentation choices (catalogs, media types,
/// filters, exclusions) and may turn an operator default on or off. It cannot
/// add a source URL; source toggles address only the operator's stored list.
pub fn merge(global: &Value, cfg: &PublicConfig) -> Value {
    let mut out = global.clone();
    if !out.is_object() {
        out = json!({});
    }

    // Public links never use upstream addons. Clearing the list (not only the
    // master switch) also stops `resolver_manifests()`, which deliberately
    // ignores the master switch, from fetching an upstream for metadata.
    out["upstream_enabled"] = json!(false);
    out["upstream_manifests"] = json!([]);
    if let Some(local) = &cfg.local_node {
        out["public_local_node"] = json!(local);
    }

    if let Some(user) = cfg.catalogs_enabled {
        out["catalogs_enabled"] = json!(user);
    }

    if let Some(mask) = cfg.types {
        // Media types are a user preference, not a source lock. The operator's
        // map is the starting point; a link may enable a type or turn one off.
        out["enabled_types"] = json!({
            "movie": mask & 1 != 0,
            "series": mask & 2 != 0,
            "other": mask & 4 != 0,
        });
    }

    if !cfg.show_catalogs.is_empty() {
        let known = crate::routes::default_catalog_options();
        if !out
            .get("enabled_catalogs")
            .map(|v| v.is_object())
            .unwrap_or(false)
        {
            out["enabled_catalogs"] = json!({});
        }
        if let Some(map) = out.get_mut("enabled_catalogs").and_then(|v| v.as_object_mut()) {
            for id in &cfg.show_catalogs {
                if known.get(id).is_some() {
                    map.insert(id.clone(), json!(true));
                }
            }
        }
    }

    if !cfg.hide_catalogs.is_empty() {
        let known = crate::routes::default_catalog_options();
        if !out
            .get("enabled_catalogs")
            .map(|v| v.is_object())
            .unwrap_or(false)
        {
            out["enabled_catalogs"] = json!({});
        }
        if let Some(map) = out.get_mut("enabled_catalogs").and_then(|v| v.as_object_mut()) {
            for id in &cfg.hide_catalogs {
                if known.get(id).is_none() {
                    continue;
                }
                // Missing key is enabled in the engine; set the explicit false.
                if map.get(id).and_then(|v| v.as_bool()).unwrap_or(true) {
                    map.insert(id.clone(), json!(false));
                }
            }
        }
    }

    // Public links no longer expose upstream sources. `u`/`d` still decode so
    // an old link opens, but they must not change which upstreams run.
    if !cfg.stream.is_empty() {
        let mut sc = out
            .get("stream_config")
            .cloned()
            .unwrap_or_else(|| json!({}));
        apply_public_stream_config(&mut sc, &cfg.stream);
        out["stream_config"] = sc;
    }

    out
}

/// Apply only the stream keys the user sent, with the operator's value as the
/// floor/ceiling. Unlike `routes::apply_stream_config`, a missing key keeps its
/// current value: the dashboard sends complete groups, a public link sends a
/// sparse override.
fn apply_public_stream_config(base: &mut Value, user: &Map<String, Value>) {
    if !base.is_object() {
        *base = json!({});
    }

    for group in ["resolutions", "qualities", "encodes", "visual_tags"] {
        let Some(input) = user.get(group).and_then(|v| v.as_object()) else {
            continue;
        };
        if !base
            .get(group)
            .map(|v| v.is_object())
            .unwrap_or(false)
        {
            base[group] = json!({});
        }
        for (key, value) in input {
            let Some(user_on) = value.as_bool() else {
                continue;
            };
            // Group filters are preferences too. The operator's default is the
            // starting value; the user may turn a value on or off.
            base[group][key] = json!(user_on);
        }
    }

    for flag in ["exclude_cam", "exclude_unplayable", "exclude_part_title"] {
        let Some(user_on) = user.get(flag).and_then(|v| v.as_bool()) else {
            continue;
        };
        // Exclusions only hide rows from THIS user's own stream list. They are
        // not a security boundary like a source enable, so the public link may
        // turn an operator default off as well as on.
        base[flag] = json!(user_on);
    }

    set_tighter(base, user, "max_streams_per_resolution", 50);
    set_tighter(base, user, "max_streams_total", 200);
    set_min_max(base, user, "min_size_mb", 100_000, true);
    set_min_max(base, user, "max_size_gb", 200, false);

    if let Some(user_excluded) = user.get("excluded_keywords").and_then(|v| v.as_str()) {
        // Keywords are the user's own filter. The operator value is only the
        // placeholder/default shown by the page; a link may replace or clear it.
        base["excluded_keywords"] = json!(user_excluded.trim());
    }

    if let Some(user_required) = user.get("required_keywords").and_then(|v| v.as_str()) {
        base["required_keywords"] = json!(user_required.trim());
    }
}

fn set_tighter(base: &mut Value, user: &Map<String, Value>, key: &str, max: i64) {
    let Some(user_value) = user.get(key).and_then(|v| v.as_i64()) else {
        return;
    };
    let user_value = user_value.clamp(0, max);
    let global = base.get(key).and_then(|v| v.as_i64()).unwrap_or(0).clamp(0, max);
    let effective = match (global, user_value) {
        (0, 0) => 0,
        (0, v) => v,
        (g, 0) => g,
        (g, v) => g.min(v),
    };
    base[key] = json!(effective);
}

fn set_min_max(base: &mut Value, user: &Map<String, Value>, key: &str, max: i64, minimum: bool) {
    let Some(user_value) = user.get(key).and_then(|v| v.as_i64()) else {
        return;
    };
    let user_value = user_value.clamp(0, max);
    let global = base.get(key).and_then(|v| v.as_i64()).unwrap_or(0).clamp(0, max);
    let effective = if minimum {
        // Both are floors: the larger one is the stricter gate.
        global.max(user_value)
    } else {
        // Both are ceilings; 0 means "no ceiling".
        match (global, user_value) {
            (0, 0) => 0,
            (0, v) => v,
            (g, 0) => g,
            (g, v) => g.min(v),
        }
    };
    base[key] = json!(effective);
}

pub fn options(global: &Value) -> Value {
    json!({
        "ok": 1,
        "public": true,
        "version": crate::config::descriptor_version(),
        "catalog_options": crate::routes::default_catalog_options(),
        "defaults": {
            "catalogs_enabled": catalogs_enabled(global),
            "enabled_types": enabled_types(global),
            "enabled_catalogs": enabled_catalogs(global),
            "stream_config": global.get("stream_config").cloned().unwrap_or_else(|| json!({})),
        },
        "locks": {
            "catalogs": !catalogs_enabled(global),
            "types": {
                "movie": !enabled_types(global).get("movie").and_then(|v| v.as_bool()).unwrap_or(true),
                "series": !enabled_types(global).get("series").and_then(|v| v.as_bool()).unwrap_or(true),
                "other": !enabled_types(global).get("other").and_then(|v| v.as_bool()).unwrap_or(true),
            },
        },
        "limits": {"config_bytes": MAX_DECODED},
    })
}

pub fn view(global: &Value, payload: &str) -> Result<Value, String> {
    let cfg = decode(payload)?;
    let merged = merge(global, &cfg);
    Ok(json!({
        "ok": 1,
        "config": to_json(&cfg),
        "effective": {
            "catalogs_enabled": catalogs_enabled(&merged),
            "enabled_types": enabled_types(&merged),
            "enabled_catalogs": enabled_catalogs(&merged),
            "stream_config": merged.get("stream_config").cloned().unwrap_or_else(|| json!({})),
        },
    }))
}

fn catalogs_enabled(settings: &Value) -> bool {
    settings
        .get("catalogs_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true)
}

fn enabled_types(settings: &Value) -> Value {
    settings
        .get("enabled_types")
        .cloned()
        .unwrap_or_else(|| json!({"movie": true, "series": true, "other": true}))
}

fn enabled_catalogs(settings: &Value) -> Value {
    settings
        .get("enabled_catalogs")
        .cloned()
        .unwrap_or_else(|| json!({}))
}

/// A cache-key fingerprint for everything in the effective settings that can
/// change a `/stream` ANSWER. Public links no longer choose sources, so only
/// the stream filters participate; catalog-only choices deliberately do not.
pub fn response_fingerprint(settings: &Value) -> String {
    let doc = json!({
        "s": settings.get("stream_config").cloned().unwrap_or(Value::Null),
    });
    let mut hasher = Sha256::new();
    hasher.update(serde_json::to_vec(&doc).unwrap_or_default());
    hex::encode(&hasher.finalize()[..6])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn global() -> Value {
        json!({
            "catalogs_enabled": true,
            "enabled_types": {"movie": true, "series": true, "other": true},
            "enabled_catalogs": {
                "pm_movies_korean": true,
                "pm_series_thai": true,
                "pm_movies_japan": false
            },
            "upstream_enabled": true,
            "upstream_manifests": [
                {"url": "https://a.example/manifest.json", "name": "A"},
                {"url": "https://b.example/manifest.json", "name": "B", "enabled": false}
            ],
            "stream_config": {
                "resolutions": {"4k": true, "1080p": true},
                "exclude_cam": false,
                "exclude_unplayable": true,
                "max_streams_total": 0,
                "min_size_mb": 0,
                "max_size_gb": 0,
                "excluded_keywords": "",
                "required_keywords": ""
            }
        })
    }

    fn enc(doc: &Value) -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(doc).unwrap())
    }

    #[test]
    fn segment_parsing_accepts_config_and_rejects_tokens_and_routes() {
        let payload = enc(&json!({"v": 1}));
        let path = format!("/{payload}/manifest.json");
        let (code, rest) = parse_segment(&path).expect("config segment");
        assert_eq!(code, payload);
        assert_eq!(rest, "/manifest.json");

        let token = "0123456789abcdef0123456789abcdef";
        assert!(parse_segment(&format!("/{token}/manifest.json")).is_none());
        assert!(parse_segment("/manifest.json").is_none());
        assert!(parse_segment("/api/version").is_none());
        assert!(parse_segment("/").is_none());
    }

    #[test]
    fn malformed_config_prefix_is_still_treated_as_a_config_attempt() {
        assert!(parse_segment("/eyJnot-json/manifest.json").is_some());
        assert!(decode("eyJnot-json").is_err());
    }

    #[test]
    fn decode_requires_version_one() {
        let payload = enc(&json!({"v": 2, "c": 0}));
        assert!(decode(&payload).is_err());
        let payload = enc(&json!({"c": 0}));
        assert!(decode(&payload).is_err());
    }

    #[test]
    fn decode_reads_the_short_keys() {
        let payload = enc(&json!({
            "v": 1,
            "c": 0,
            "t": 3,
            "h": ["pm_movies_korean"],
            "e": ["pm_movies_japan"],
            "d": ["a1b2c3d4e5f6"],
            "l": "http://127.0.0.1:8088",
            "s": {"max_size_gb": 10}
        }));
        let cfg = decode(&payload).unwrap();
        assert_eq!(cfg.catalogs_enabled, Some(false));
        assert_eq!(cfg.types, Some(3));
        assert_eq!(cfg.hide_catalogs, vec!["pm_movies_korean"]);
        assert_eq!(cfg.show_catalogs, vec!["pm_movies_japan"]);
        assert_eq!(cfg.disable_sources, vec!["a1b2c3d4e5f6"]);
        assert_eq!(cfg.local_node.as_deref(), Some("http://127.0.0.1:8088"));
        assert_eq!(cfg.stream.get("max_size_gb"), Some(&json!(10)));
    }

    #[test]
    fn merge_hides_catalogs() {
        let cfg = PublicConfig {
            catalogs_enabled: Some(true),
            types: Some(7),
            hide_catalogs: vec!["pm_movies_korean".into(), "pm_movies_japan".into()],
            show_catalogs: Vec::new(),
            stream: Map::new(),
            ..Default::default()
        };
        let out = merge(&global(), &cfg);
        assert_eq!(out["enabled_catalogs"]["pm_movies_korean"], json!(false));
        // Already false stays false; a hidden row wins over a show request.
        assert_eq!(out["enabled_catalogs"]["pm_movies_japan"], json!(false));
    }

    #[test]
    fn merge_can_show_a_default_off_catalog() {
        let cfg = PublicConfig {
            show_catalogs: vec!["pm_movies_japan".into()],
            ..Default::default()
        };
        let out = merge(&global(), &cfg);
        assert_eq!(out["enabled_catalogs"]["pm_movies_japan"], json!(true));
    }

    #[test]
    fn merge_applies_stream_filters_as_a_tighter_version() {
        let cfg = PublicConfig {
            stream: serde_json::from_value(json!({
                "resolutions": {"4k": false},
                "exclude_cam": true,
                "max_streams_total": 5,
                "min_size_mb": 100,
                "max_size_gb": 10
            }))
            .unwrap(),
            ..Default::default()
        };
        let out = merge(&global(), &cfg);
        assert_eq!(out["stream_config"]["resolutions"]["4k"], json!(false));
        assert_eq!(out["stream_config"]["resolutions"]["1080p"], json!(true));
        assert_eq!(out["stream_config"]["exclude_cam"], json!(true));
        assert_eq!(out["stream_config"]["max_streams_total"], json!(5));
        assert_eq!(out["stream_config"]["min_size_mb"], json!(100));
        assert_eq!(out["stream_config"]["max_size_gb"], json!(10));
    }

    #[test]
    fn merge_can_turn_off_a_default_exclusion() {
        let cfg = PublicConfig {
            stream: serde_json::from_value(json!({
                "exclude_unplayable": false,
                "exclude_part_title": false
            }))
            .unwrap(),
            ..Default::default()
        };
        let out = merge(&global(), &cfg);
        assert_eq!(out["stream_config"]["exclude_unplayable"], json!(false));
        assert_eq!(out["stream_config"]["exclude_part_title"], json!(false));
    }

    #[test]
    fn old_public_source_choices_are_ignored() {
        let mut source_overrides = BTreeMap::new();
        source_overrides.insert("a1b2c3d4e5f6".to_string(), true);
        let cfg = PublicConfig {
            disable_sources: vec!["b1b0a2852f3e".into()],
            source_overrides,
            ..Default::default()
        };
        let out = merge(&global(), &cfg);
        assert_eq!(out["upstream_enabled"], json!(false));
        assert_eq!(out["upstream_manifests"], json!([]));
    }

    #[test]
    fn merge_can_name_a_loopback_local_node() {
        let cfg = PublicConfig {
            local_node: Some("http://127.0.0.1:8088".to_string()),
            ..Default::default()
        };
        let out = merge(&global(), &cfg);
        assert_eq!(out["public_local_node"], json!("http://127.0.0.1:8088"));
        assert!(valid_local_node("https://127.0.0.1:8088").is_none());
        assert!(valid_local_node("http://127.0.0.1:9999").is_none());
        assert!(valid_local_node("http://example.com:8088").is_none());
    }

    #[test]
    fn merge_can_change_the_catalog_master_and_group_filters() {
        let cfg = PublicConfig {
            catalogs_enabled: Some(false),
            types: Some(3),
            stream: serde_json::from_value(json!({
                "resolutions": {"4k": false}
            }))
            .unwrap(),
            ..Default::default()
        };
        let out = merge(&global(), &cfg);
        assert_eq!(out["catalogs_enabled"], json!(false));
        assert_eq!(out["enabled_types"]["movie"], json!(true));
        assert_eq!(out["enabled_types"]["series"], json!(true));
        assert_eq!(out["enabled_types"]["other"], json!(false));
        assert_eq!(out["stream_config"]["resolutions"]["4k"], json!(false));
    }

    #[test]
    fn install_ids_differ_even_for_catalog_only_configs() {
        let a = payload_id("eyJ2IjoxLCJjIjowfQ");
        let b = payload_id("eyJ2IjoxLCJoIjpbXX0");
        assert_eq!(a.len(), 12);
        assert_ne!(a, b);
    }

    #[test]
    fn options_do_not_expose_upstreams() {
        let opts = options(&global());
        let text = opts.to_string();
        assert!(!text.contains("https://a.example"));
        assert!(!text.contains("https://b.example"));
        assert!(opts["defaults"].get("upstreams").is_none());
        assert!(opts["locks"].get("upstreams").is_none());
    }
}
