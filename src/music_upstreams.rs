//! Validate a BitChord-protocol music addon BEFORE it is saved as a fill
//! source.
//!
//! `backfill` (see [`crate::backfill`]) fills a track the catalogue does not
//! hold, from an addon that speaks the BitChord protocol. It used to be turned on
//! only by `PM_HIRES_BASE`, a server-side env var. The dashboard can now add a
//! source, and this module is the gate: a wrong or hostile URL must be refused at
//! entry instead of being discovered later as a stream of failed fills.
//!
//! Every rule here was established by measurement, and each one has a host that
//! actually did it:
//!
//! | Rule                                     | The measurement                                                                |
//! | ---------------------------------------- | ------------------------------------------------------------------------------ |
//! | normalise like BitChord's `normalizeBase` | a pasted `…/manifest.json` would otherwise be probed as `…/manifest.json/manifest.json` |
//! | `id` must be non-empty                    | an error page/HTML body parses as `{}` and looks "fine"                          |
//! | `bitchordAuth: true` → REFUSE             | `bitchord.confih.tech` only serves the BitChord app; every fill would 401        |
//! | 401 + `WWW-Authenticate: BitChord` → REFUSE | the same lock, discovered from `/search` instead of the manifest               |
//!
//! THE `bitchordAuth` REFUSAL IS PERMANENT, not a "we have not implemented it
//! yet". BitChord's addon auth (protocol v1) requires `X-BitChord-Token` — an
//! Ed25519 JWT whose `cnf.jwk` is a P-256 key held in the PHONE's TEE/StrongBox —
//! plus `X-BitChord-Proof`, an ECDSA signature by that key over
//! `bitchord-addon-proof/v1\\n<host>\\n<unix-ms>\\n<base64url(SHA-256(token))>`.
//! The key can only be created behind Google-signed Android key attestation on an
//! untampered, locked phone; BitChord's auth service refuses emulators, rooted
//! and unlocked devices, and the docs say so plainly: "Only the official BitChord
//! Android app can authenticate" — desktop and web clients included. A server has
//! no such key and cannot obtain one, and a token lifted off a phone is useless
//! without its hardware key. So there is nothing to implement on this side: the
//! message below exists to stop the next reader from trying.
//! | `resources` must list `search`            | an addon with no search endpoint can never be filled from                       |
//! | `/search?q=test&limit=3` needs `tracks`   | `bit-chord-addon.vercel.app/a｜/b｜/d｜/e｜/f` answered 502/500/502/400/502 while `/` and `/c` answered 200 |
//! | at least one LOSSLESS row                 | a source with no lossless rendition is useless for a fill                        |
//! | `/stream` is NOT required                 | it costs a CDN request and those hosts rate-limit (measured: `size probe: HTTP 429` after four fills in twenty seconds) |
//!
//! The network half ([`probe`]) is thin on purpose: every decision is a pure
//! function (`normalize_base`, `read_manifest`, `read_search`) so the accept and
//! refuse rules are pinned by tests with fixtures, never by a network call.

use std::time::Duration;

use serde_json::{json, Value};

/// The fixed probe query. Anything answerable works; this is the one the
/// operator-facing report quotes, so it must be a constant.
pub const PROBE_QUERY: &str = "test";

/// How many rows the probe asks for. Three is enough to tell "no lossless at
/// all" from "lossless is the exception", and small enough to be cheap.
const PROBE_LIMIT: usize = 3;

/// The OUTER bound for one probe request.
///
/// This is the one that actually holds: the inner `.timeout()` on the request
/// only reaches the helper, and a server that accepts the connection and then
/// dribbles its body is not a connect timeout. MEASURED 2026-11-10 — exactly that
/// cost an hour of diagnosis, because the fill logged its trigger and then sat
/// silent with no status line at all.
const PROBE_TIMEOUT_SECS: u64 = 12;

/// Inner bound, so a single slow hop gives up before the outer one fires and
/// the report can name the host rather than just "timed out".
const REQUEST_TIMEOUT_SECS: u64 = 10;

/// The BitChord `normalizeBase` step, and the only place a user-typed string
/// becomes something we will fetch.
///
/// A pasted `https://host/c/manifest.json` and a pasted `https://host/c` must
/// reach the SAME base, otherwise the stored source is later probed as
/// `…/manifest.json/manifest.json` and every fill fails on a URL nobody would
/// recognise. A `.json` file name is always the manifest, never a path segment.
pub fn normalize_base(raw: &str) -> Result<String, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("Enter the addon base URL first.".to_string());
    }
    // `stremio://` is what an addon install link carries; it is the same host.
    let with_scheme = match trimmed.strip_prefix("stremio://") {
        Some(rest) => format!("https://{rest}"),
        None => trimmed.to_string(),
    };
    let lower = with_scheme.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(format!(
            "\"{trimmed}\" is not an http(s) URL — paste the addon base, e.g. https://host/c"
        ));
    }
    // A query or fragment never belongs to a base (a manifest link copied out of
    // a browser can carry both).
    let mut base = with_scheme
        .split(['?', '#'])
        .next()
        .unwrap_or("")
        .to_string();
    // Strip a trailing `/manifest.json` — or ANY `.json` file name.
    if let Some(pos) = base.rfind('/') {
        if base[pos + 1..].to_ascii_lowercase().ends_with(".json") {
            base.truncate(pos);
        }
    }
    while base.ends_with('/') {
        base.pop();
    }
    // A host is mandatory: `https://` alone normalises to nothing usable.
    match url::Url::parse(&base) {
        Ok(parsed) if parsed.host_str().map(|h| !h.is_empty()).unwrap_or(false) => Ok(base),
        _ => Err(format!(
            "\"{trimmed}\" has no host — paste the addon base, e.g. https://host/c"
        )),
    }
}

/// A stable id for a source, derived from its base so re-adding the same URL is
/// the same source rather than a second row.
pub fn source_id(base: &str) -> String {
    let slug: String = base
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    let collapsed = slug.split('-').filter(|s| !s.is_empty()).collect::<Vec<_>>().join("-");
    if collapsed.is_empty() {
        "ms-source".to_string()
    } else {
        format!("ms-{collapsed}")
    }
}

/// What the manifest says, once it is known to be a manifest.
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct ManifestInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub resources: Vec<String>,
    pub bit_chord_auth: bool,
}

/// `resources` accepts both shapes an addon ships: `["search"]` and
/// `[{"name": "search", "types": [...]}]`.
fn resource_names(manifest: &Value) -> Vec<String> {
    let Some(list) = manifest.get("resources").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    list.iter()
        .filter_map(|entry| match entry {
            Value::String(name) => Some(name.trim().to_string()),
            Value::Object(o) => o
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string()),
            _ => None,
        })
        .filter(|s| !s.is_empty())
        .collect()
}

/// `"bitchordAuth": true` under either spelling the addons use.
fn says_bit_chord_auth(manifest: &Value) -> bool {
    ["bitchordAuth", "bitChordAuth"]
        .iter()
        .any(|key| manifest.get(*key).and_then(|v| v.as_bool()) == Some(true))
}

/// What a manifest says, WITHOUT judging it — used to fill the report on a
/// refusal, where the dashboard still needs `name`/`version`/`requires_bit_chord_auth`
/// to explain itself. (`requires_bit_chord_auth: false` next to the message "this
/// addon only serves the BitChord app" would read as a bug.)
fn partial_info(manifest: &Value) -> ManifestInfo {
    let field = |key: &str| {
        manifest
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string()
    };
    ManifestInfo {
        id: field("id"),
        name: field("name"),
        version: field("version"),
        resources: resource_names(manifest),
        bit_chord_auth: says_bit_chord_auth(manifest),
    }
}

/// Decide whether a `/manifest.json` answer is usable, and read out of it what
/// the dashboard displays.
pub fn read_manifest(manifest: &Value) -> Result<ManifestInfo, String> {
    if !manifest.is_object() {
        return Err(
            "That URL did not answer a JSON manifest — check the base points at a BitChord-protocol addon."
                .to_string(),
        );
    }
    let id = manifest
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if id.is_empty() {
        return Err(
            "The manifest has no `id`, so it is not a BitChord-protocol addon (this is what an error page looks like)."
                .to_string(),
        );
    }
    let name = manifest
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let version = manifest
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let resources = resource_names(manifest);
    let bit_chord_auth = says_bit_chord_auth(manifest);
    if bit_chord_auth {
        return Err(
            "This addon only serves the official BitChord app (`bitchordAuth`): every request must be signed by a hardware key inside a phone, which a server can never produce. It cannot be used as a fill source."
                .to_string(),
        );
    }
    if !resources.is_empty() && !resources.iter().any(|r| r.eq_ignore_ascii_case("search")) {
        return Err(format!(
            "The manifest lists resources [{}] but not \"search\" — it cannot be searched, so it can never be filled from.",
            resources.join(", ")
        ));
    }
    Ok(ManifestInfo {
        id,
        name: if name.is_empty() { "Music addon".to_string() } else { name },
        version,
        resources,
        bit_chord_auth,
    })
}

/// What a `/search` answer gave us: how many rows, and how many of them name a
/// lossless rendition.
#[derive(Debug, PartialEq, Eq)]
pub struct SearchInfo {
    pub rows: usize,
    pub lossless_rows: usize,
}

/// Decide whether a `/search` answer is usable.
///
/// A 502/500/400 from the host is a REFUSAL, not a warning: measured 2026-11-10,
/// `bit-chord-addon.vercel.app/a|/b|/d|/e|/f` answered 502/500/502/400/502 while
/// `/` and `/c` answered 200 with full-length FLAC masters. Accepting the broken
/// sibling means every fill pays a request to a host that will never answer it.
pub fn read_search(status: u16, body: &Value) -> Result<SearchInfo, String> {
    if status != 200 {
        return Err(format!(
            "/search answered HTTP {status} — a source that cannot answer a probe cannot fill a track."
        ));
    }
    let Some(tracks) = body.get("tracks").and_then(|v| v.as_array()) else {
        return Err(
            "/search answered 200 without a `tracks` array — that is not a BitChord-protocol search endpoint."
                .to_string(),
        );
    };
    let rows = tracks.len();
    let lossless_rows = tracks
        .iter()
        .filter(|row| crate::backfill::row_says_lossless(row))
        .count();
    if lossless_rows == 0 {
        return Err(format!(
            "\"{PROBE_QUERY}\" answered {rows} row(s) and none names a lossless rendition — a source with no master is useless for a fill."
        ));
    }
    Ok(SearchInfo {
        rows,
        lossless_rows,
    })
}

/// Whether a 401 answer names the BitChord app in `WWW-Authenticate`.
pub fn says_bit_chord_auth_challenge(www_authenticate: &str) -> bool {
    let value = www_authenticate.to_ascii_lowercase();
    value.contains("bitchord") || value.contains("bit-chord")
}

/// The addon's OWN words for its refusal, when it writes any.
///
/// MEASURED 2026-11-10 against `bitchord.confih.tech`: `/search` and `/stream`
/// both answer `401`, `WWW-Authenticate: BitChord realm="bitchord-addons",
/// error="missing", error_description="This addon only works in the official
/// BitChord app"` and a body `{"error":"bitchord_auth","code":"missing",
/// "message":"This addon only works in the official BitChord app"}`. Quoting it
/// beats paraphrasing: the operator reads the host's reason, not ours.
pub fn challenge_note(www_authenticate: &str, body: &Value) -> String {
    let text = |s: &str| s.trim().trim_matches('"').trim().to_string();
    let from_body = body
        .get("message")
        .and_then(|v| v.as_str())
        .map(text)
        .filter(|s| !s.is_empty());
    if let Some(m) = from_body {
        return format!(" The addon says: \"{m}\".");
    }
    // `error_description="…"` inside the challenge header.
    let lower = www_authenticate.to_ascii_lowercase();
    if let Some(pos) = lower.find("error_description=") {
        let rest = &www_authenticate[pos + "error_description=".len()..];
        let quoted = rest.trim_start_matches('"');
        let end = quoted.find('"').unwrap_or(quoted.len());
        let note = quoted[..end].trim();
        if !note.is_empty() {
            return format!(" The addon says: \"{note}\".");
        }
    }
    String::new()
}

/// The user-facing report. `ok` is the only field a caller must branch on;
/// `reason` is written to be shown verbatim under the URL field.
fn report(
    ok: bool,
    reason: &str,
    base: &str,
    info: Option<&ManifestInfo>,
    search_ok: bool,
    sample_tracks: usize,
    lossless_rows: usize,
    warnings: &[String],
) -> Value {
    json!({
        "ok": ok,
        "reason": reason,
        "base": base,
        "id": info.map(|i| i.id.clone()).unwrap_or_default(),
        "name": info.map(|i| i.name.clone()).unwrap_or_default(),
        "version": info.map(|i| i.version.clone()).unwrap_or_default(),
        "resources": info.map(|i| i.resources.clone()).unwrap_or_default(),
        "requires_bit_chord_auth": info.map(|i| i.bit_chord_auth).unwrap_or(false),
        "search_ok": search_ok,
        "sample_tracks": sample_tracks,
        "lossless_rows": lossless_rows,
        "warnings": warnings,
    })
}

/// One bounded GET. Returns the status, the `WWW-Authenticate` header (for the
/// BitChord lock) and the parsed body (`Value::Null` when the body is not JSON).
async fn probe_get(url: &str) -> Result<(u16, String, Value), String> {
    let call = async {
        let resp = crate::httpc::client()
            .get(url)
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let www = resp
            .headers()
            .get("www-authenticate")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let text = resp.text().await.unwrap_or_default();
        Ok::<(u16, String, String), String>((status, www, text))
    };
    let (status, www, text) = match tokio::time::timeout(
        Duration::from_secs(PROBE_TIMEOUT_SECS),
        call,
    )
    .await
    {
        Ok(Ok(t)) => t,
        Ok(Err(e)) => return Err(e),
        // The OUTER bound fired — the one a dribbling server cannot outlive.
        Err(_) => return Err(format!("no answer within {PROBE_TIMEOUT_SECS}s")),
    };
    let body = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
    Ok((status, www, body))
}

/// Probe `raw` and answer the dashboard's report.
///
/// Never panics, never returns an error: every failure is a 200 with
/// `ok: false` and a `reason` meant for a human, because this runs from a form.
pub async fn probe(raw: &str) -> Value {
    let base = match normalize_base(raw) {
        Ok(b) => b,
        Err(why) => return report(false, &why, "", None, false, 0, 0, &[]),
    };
    let mut warnings: Vec<String> = Vec::new();

    // SSRF: this route makes the SERVER fetch a URL a form supplied, so a
    // loopback/LAN/169.254 target must be refused before a request is made.
    if !crate::auth::url_host_is_public(&base).await {
        return report(
            false,
            "That host does not resolve to a public address — this server only fills from a public addon.",
            &base,
            None,
            false,
            0,
            0,
            &[],
        );
    }

    // 1. the manifest
    let manifest_url = format!("{base}/manifest.json");
    let (status, _www, manifest) = match probe_get(&manifest_url).await {
        Ok(r) => r,
        Err(why) => {
            return report(
                false,
                &format!("Could not fetch {manifest_url}: {why}"),
                &base,
                None,
                false,
                0,
                0,
                &[],
            )
        }
    };
    if status == 429 {
        warnings.push(format!(
            "/manifest.json answered HTTP 429 — the host is rate-limiting this server, not rejecting the URL."
        ));
    }
    if status != 200 {
        return report(
            false,
            &format!("/manifest.json answered HTTP {status} — this addon is not serving a manifest right now."),
            &base,
            None,
            false,
            0,
            0,
            &warnings,
        );
    }
    let info = match read_manifest(&manifest) {
        Ok(i) => i,
        Err(why) => {
            let partial = partial_info(&manifest);
            // Only worth reporting when it is a manifest at all; `Value::Null`
            // (an HTML error page) has nothing to say.
            let partial = if partial.id.is_empty() { None } else { Some(&partial) };
            return report(false, &why, &base, partial, false, 0, 0, &warnings);
        }
    };

    // 2. the search — the only endpoint a fill actually depends on.
    let search_url = format!("{base}/search?q={PROBE_QUERY}&limit={PROBE_LIMIT}");
    let (sstatus, swww, sbody) = match probe_get(&search_url).await {
        Ok(r) => r,
        Err(why) => {
            let mut out = info.clone();
            out.bit_chord_auth = false;
            return report(
                false,
                &format!("/search did not answer: {why}"),
                &base,
                Some(&out),
                false,
                0,
                0,
                &warnings,
            );
        }
    };
    if sstatus == 429 {
        warnings.push(
            "/search answered HTTP 429 — the host is rate-limiting this server, not rejecting the URL."
                .to_string(),
        );
        return report(
            false,
            "The host rate-limited the probe (HTTP 429) — that is about the host, not the URL; try again in a minute.",
            &base,
            Some(&info),
            false,
            0,
            0,
            &warnings,
        );
    }
    if sstatus == 401 && says_bit_chord_auth_challenge(&swww) {
        let mut locked = info.clone();
        locked.bit_chord_auth = true;
        // Nothing to implement here, and the message must make that clear rather
        // than read like a missing feature: see the module doc for the protocol.
        let reason = format!(
            "This addon only serves the official BitChord app (401 + `WWW-Authenticate: BitChord`): every request must be signed by a phone's hardware key, which a server can never produce.{}",
            challenge_note(&swww, &sbody)
        );
        return report(false, &reason, &base, Some(&locked), false, 0, 0, &warnings);
    }
    let search = match read_search(sstatus, &sbody) {
        Ok(s) => s,
        Err(why) => return report(false, &why, &base, Some(&info), false, 0, 0, &warnings),
    };

    report(
        true,
        &format!(
            "OK — {} {}, {} lossless row(s) of {} for \"{}\".",
            info.name,
            if info.version.is_empty() { "" } else { info.version.as_str() },
            search.lossless_rows,
            search.rows,
            PROBE_QUERY
        )
        .replace("  ", " "),
        &base,
        Some(&info),
        true,
        search.rows,
        search.lossless_rows,
        &warnings,
    )
}

/// The enabled sources a fill may use, in the order the dashboard lists them.
/// Pure, so the precedence rule below is testable without touching settings on
/// disk.
pub fn enabled_bases(settings: &Value) -> Vec<String> {
    let Some(list) = settings.get("music_sources").and_then(|v| v.as_array()) else {
        return Vec::new();
    };
    let mut out: Vec<String> = Vec::new();
    for entry in list {
        if entry.get("enabled").and_then(|v| v.as_bool()) == Some(false) {
            continue;
        }
        let raw = entry.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let Ok(base) = normalize_base(raw) else { continue };
        if !out.contains(&base) {
            out.push(base);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A pasted manifest URL and a pasted base are the SAME source, and a bare
    /// host is refused rather than probed (its failure would read as "down").
    #[test]
    fn a_base_is_normalised_the_way_bit_chord_does_it() {
        assert_eq!(
            normalize_base("https://bit-chord-addon.vercel.app/c").unwrap(),
            "https://bit-chord-addon.vercel.app/c"
        );
        assert_eq!(
            normalize_base("  https://bit-chord-addon.vercel.app/c/manifest.json  ").unwrap(),
            "https://bit-chord-addon.vercel.app/c"
        );
        assert_eq!(
            normalize_base("https://bit-chord-addon.vercel.app/c/").unwrap(),
            "https://bit-chord-addon.vercel.app/c"
        );
        assert_eq!(
            normalize_base("https://bit-chord-addon.vercel.app/c//").unwrap(),
            "https://bit-chord-addon.vercel.app/c"
        );
        // Any `.json` file name is the manifest, not a path segment.
        assert_eq!(
            normalize_base("https://host/c/addon.json").unwrap(),
            "https://host/c"
        );
        // A manifest link copied out of a browser carries a query.
        assert_eq!(
            normalize_base("https://host/c/manifest.json?x=1").unwrap(),
            "https://host/c"
        );
        assert_eq!(
            normalize_base("stremio://host/c/manifest.json").unwrap(),
            "https://host/c"
        );

        assert!(normalize_base("").is_err());
        assert!(normalize_base("   ").is_err());
        assert!(normalize_base("example.com/c").is_err(), "a bare host is not a URL");
        assert!(normalize_base("https://").is_err(), "no host");
        assert!(normalize_base("file:///etc/passwd").is_err());
    }

    #[test]
    fn a_source_id_is_stable_and_follows_the_base() {
        assert_eq!(source_id("https://host/c"), "ms-host-c");
        assert_eq!(source_id("http://host/c"), "ms-host-c");
        assert_eq!(
            source_id("https://bit-chord-addon.vercel.app/c"),
            "ms-bit-chord-addon-vercel-app-c"
        );
        // Re-adding the same base must not mint a second id.
        assert_eq!(
            source_id(&normalize_base("https://host/c/manifest.json").unwrap()),
            source_id("https://host/c")
        );
    }

    /// The measured manifest of a WORKING source: `bit-chord-addon.vercel.app/c`
    /// answers this shape, and its `/search` returns `format: flac` rows.
    #[test]
    fn a_working_manifest_is_accepted_and_described() {
        let manifest = json!({
            "id": "org.bitchord.addon.friend",
            "name": "BitChord Music",
            "version": "1.2.3",
            "resources": ["search", "stream"],
            "bitchordAuth": false
        });
        let info = read_manifest(&manifest).expect("a real manifest must pass");
        assert_eq!(info.id, "org.bitchord.addon.friend");
        assert_eq!(info.name, "BitChord Music");
        assert_eq!(info.version, "1.2.3");
        assert_eq!(info.resources, vec!["search", "stream"]);
        assert!(!info.bit_chord_auth);

        // The object form of `resources` is the same declaration.
        let object_form = json!({
            "id": "x",
            "resources": [{"name": "search", "types": ["movie"]}, {"name": "stream"}]
        });
        assert_eq!(read_manifest(&object_form).unwrap().resources, vec!["search", "stream"]);
    }

    /// The client-locked addon (`bitchord.confih.tech`) must be REFUSED, with a
    /// message that says why — it is not a URL typo and retrying cannot help.
    #[test]
    fn an_addon_that_only_serves_the_bit_chord_app_is_refused() {
        let locked = json!({
            "id": "org.bitchord.addon.hires",
            "name": "Bitchord Hi-Res",
            "version": "1.0.0",
            "resources": ["search"],
            "bitchordAuth": true
        });
        let err = read_manifest(&locked).expect_err("a locked addon is unusable");
        assert!(err.contains("BitChord app"), "{err}");
        // The other spelling ships too.
        let locked_alt = json!({"id": "x", "bitChordAuth": true});
        assert!(read_manifest(&locked_alt).is_err());
        // And the same lock discovered from `/search` instead of the manifest.
        assert!(says_bit_chord_auth_challenge("BitChord"));
        assert!(says_bit_chord_auth_challenge("BitChord realm=\"bitchord\""));
        assert!(!says_bit_chord_auth_challenge("Bearer"));
        assert!(!says_bit_chord_auth_challenge(""));
    }

    /// The addon's own refusal text is quoted, not paraphrased — the measured
    /// `bitchord.confih.tech` answer carries both a body `message` and an
    /// `error_description` in the challenge header.
    #[test]
    fn the_addons_own_refusal_words_are_quoted() {
        let body = json!({
            "error": "bitchord_auth",
            "code": "missing",
            "message": "This addon only works in the official BitChord app"
        });
        let www = r#"BitChord realm="bitchord-addons", error="missing", error_description="This addon only works in the official BitChord app""#;
        assert_eq!(
            challenge_note(www, &body),
            " The addon says: \"This addon only works in the official BitChord app\"."
        );
        // Header only (no JSON body).
        assert_eq!(
            challenge_note(www, &Value::Null),
            " The addon says: \"This addon only works in the official BitChord app\"."
        );
        // Nothing to quote: no invented text.
        assert_eq!(challenge_note("BitChord", &json!({})), "");
        assert_eq!(challenge_note("", &json!({"message": "  "})), "");
    }

    /// A refusal still has to explain itself: the report keeps the manifest's
    /// name and the `bitchordAuth` flag, so the dashboard never shows
    /// `requires_bit_chord_auth: false` under "only serves the BitChord app".
    #[test]
    fn a_refused_manifest_still_reports_the_facts_behind_the_refusal() {
        let locked = json!({
            "id": "org.bitchord.addon.hires",
            "name": "Bitchord Hi-Res",
            "version": "1.0.0",
            "resources": ["search"],
            "bitchordAuth": true
        });
        assert!(read_manifest(&locked).is_err());
        let partial = partial_info(&locked);
        assert!(partial.bit_chord_auth, "the flag must survive the refusal");
        assert_eq!(partial.name, "Bitchord Hi-Res");
        assert_eq!(partial.version, "1.0.0");
        assert_eq!(partial.resources, vec!["search"]);
    }

    /// An error page parses as `{}`; an addon with no `search` resource can never
    /// answer a fill.
    #[test]
    fn a_manifest_without_an_id_or_a_search_resource_is_refused() {
        let err = read_manifest(&json!({})).expect_err("an empty body is not a manifest");
        assert!(err.contains("`id`"), "{err}");
        assert!(read_manifest(&Value::Null).is_err(), "a non-JSON body");
        assert!(read_manifest(&json!({"id": "  "})).is_err(), "a blank id");

        let no_search = json!({"id": "x", "resources": ["stream", "subtitles"]});
        let err = read_manifest(&no_search).expect_err("no search endpoint");
        assert!(err.contains("search"), "{err}");

        // An addon that declares no resources at all is judged by `/search`, not
        // refused here — that is how a bare BitChord addon answers.
        assert!(read_manifest(&json!({"id": "x"})).is_ok());
    }

    /// The measured 502/500/400 siblings of a working addon, and the working
    /// answer next to them: three `flac` LOSSLESS rows.
    #[test]
    fn a_host_that_cannot_answer_search_is_refused_and_a_working_one_accepted() {
        for status in [502u16, 500, 400, 503, 404] {
            let err = read_search(status, &json!({}))
                .expect_err("a broken sibling must not be accepted");
            assert!(err.contains(&status.to_string()), "{err}");
        }

        let working = json!({
            "tracks": [
                {"id": "1", "title": "a", "artist": "b", "format": "flac", "audioQuality": "LOSSLESS"},
                {"id": "2", "title": "c", "artist": "d", "format": "flac", "audioQuality": "LOSSLESS"},
                {"id": "3", "title": "e", "artist": "f", "format": "mp3", "audioQuality": "HIGH"}
            ]
        });
        let found = read_search(200, &working).expect("a working addon must pass");
        assert_eq!(found.rows, 3);
        assert_eq!(found.lossless_rows, 2, "only the flac rows are masters");

        // 200 but not a search endpoint.
        let err = read_search(200, &json!({"error": "not found"})).expect_err("no tracks array");
        assert!(err.contains("tracks"), "{err}");

        // 200 with tracks, but nothing lossless: useless for a fill.
        let lossy = json!({"tracks": [{"id": "1", "format": "mp3", "quality": "MP3 320"}]});
        let err = read_search(200, &lossy).expect_err("no master, no use");
        assert!(err.contains("lossless"), "{err}");

        // An empty result set is the same refusal, not a pass.
        assert!(read_search(200, &json!({"tracks": []})).is_err());
    }

    /// PRECEDENCE: enabled dashboard sources first, `PM_HIRES_BASE` after them,
    /// deduped, and a disabled row costs nothing.
    #[test]
    fn enabled_sources_come_first_and_disabled_ones_are_skipped() {
        let settings = json!({
            "music_sources": [
                {"id": "a", "name": "A", "url": "https://a/addon", "enabled": true},
                {"id": "b", "name": "B", "url": "https://b/c", "enabled": false},
                {"id": "c", "name": "C", "url": "https://a/addon/manifest.json", "enabled": true}
            ]
        });
        assert_eq!(enabled_bases(&settings), vec!["https://a/addon".to_string()]);
        assert!(enabled_bases(&json!({})).is_empty());
        // A row with no url is skipped, not probed as "".
        assert!(enabled_bases(&json!({"music_sources": [{"id": "x", "enabled": true}]})).is_empty());
    }
}
