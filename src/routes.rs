//! Route dispatch — the port of backend.php:14112-20527.
//!
//! Route bodies, response shapes, status codes and message strings follow the
//! PHP one-for-one, and the ORDER is the PHP's:
//! `/api/sub-proxy` -> `/api/auth/*` -> `/api/version` -> `/api/lan-ip` ->
//! `/api/tunnel/*` -> `/api/catalog-settings` -> `/api/media-ids` ->
//! `/api/validate-manifest` -> `/api/country` -> `/api/session` ->
//! `/api/clock-check` -> `/api/provision` -> `/api/bots*` -> [version gate] ->
//! `/api/diagnostics*` -> `/api/reports` -> `/api/debug-mode` -> `/api/logs` ->
//! `/api/botlogin` -> `/api/botlogout` -> `/api/proxy-stream` -> `/api/download`
//! -> `/api/resolve-shortcode` -> `/api/warmup-resolve` -> manifest/catalog/meta/
//! stream/eclipse.
//!
//! Auth gates exactly as the PHP: public = `/manifest.json` (+ `/nuvio`,
//! `/stremio`), `/catalog/*`, `/meta/*`; token-gated = `/stream/*`,
//! `/eclipse/stream/*`, `/api/download*`, `/api/media-ids`, `/api/logs`,
//! `/api/debug-mode`; privileged = `/api/auth/token/rotate`, `/api/auth/reset`,
//! `/api/tunnel/*`, `/api/diagnostics/settings` POST, `/api/reports?clear|send|flush`.
//! A locked `/stream` answer is a STREAM ITEM, never a 401.

use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::response::Response;
use serde_json::Value;

use crate::auth::{
    ReqCtx, auth_client_ip, auth_cookie_header, auth_enabled, auth_lockout_check, auth_privileged,
    auth_record_failure, auth_record_success, auth_rotate_token, auth_set_password, auth_token,
    auth_verify_password, is_authenticated, parse_cookies, require_privileged, strip_token_prefix,
    url_host_is_public,
};
use crate::config::{AUTH_DEFAULT_PASSWORD, app_version, wp_api_base, wp_version_url};
use crate::credentials::{clear_api_secret, get_api_secret};
use crate::diagnostics::{
    auto_report_enabled, clear as diag_clear, counts as diag_counts, device_info,
    get_device_id, list as diag_list, set_auto_report,
};
use crate::download::{
    guess_video_mime, handle_download, is_public_download_path, stream_filename,
};
use crate::httpc::{cached_wp_json, get_contents, http_json};
use crate::log::{is_debug_enabled, log, set_debug_enabled};
use crate::resolve::{
    resolve_shortcode, resolve_shortcode_concurrent, warmup_resolve_batch,
};
use crate::respond::{
    allow_private_network, eclipse_locked_response, json_response, not_found_json, redirect, rtrim,
    stremio_json, stremio_locked_stream, text_response,
};
use crate::session::{
    add_pool_bot, auto_provision_guest, clear_session, clock_skew_message, env_bot_tokens,
    env_token_for, get_bot_id, get_bot_pool, has_local_session, is_guest_provision_in_progress,
    load_session_meta, measure_clock_offset, remove_pool_bot, save_session_meta,
};
use crate::storage::{env, now, storage_path};
use crate::tunnel;
use crate::upstream;
use crate::AppState;

const MAX_BODY: usize = 4 * 1024 * 1024;

fn jstr(v: &Value, key: &str) -> String {
    v.get(key).and_then(|x| x.as_str()).unwrap_or("").trim().to_string()
}

fn header_json(req: &Request<Body>) -> HashMap<String, String> {
    let _ = req;
    HashMap::new()
}

pub async fn handle(state: Arc<AppState>, req: Request<Body>) -> Response {
    let (parts, body) = req.into_parts();
    let body_bytes = match axum::body::to_bytes(body, MAX_BODY).await {
        Ok(b) => b,
        Err(_) => bytes::Bytes::new(),
    };
    let body_text = String::from_utf8_lossy(&body_bytes).to_string();

    let raw_uri = parts.uri.clone();
    let raw_path = if raw_uri.path().is_empty() {
        "/".to_string()
    } else {
        raw_uri.path().to_string()
    };
    let query: Vec<(String, String)> = raw_uri
        .query()
        .map(|q| {
            url::form_urlencoded::parse(q.as_bytes())
                .map(|(k, v)| (k.into_owned(), v.into_owned()))
                .collect()
        })
        .unwrap_or_default();
    let cookies = parse_cookies(
        parts
            .headers
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or(""),
    );
    let remote = parts
        .extensions
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_default();
    let method = parts.method.as_str().to_uppercase();

    let mut ctx = ReqCtx {
        method: method.clone(),
        path: raw_path.clone(),
        raw_path: raw_path.clone(),
        query,
        headers: parts.headers,
        cookies,
        remote,
        local: String::new(),
        base_url: String::new(),
    };
    ctx.base_url = base_url_of(&ctx, state.port);

    if method == "OPTIONS" {
        // Echo the headers the browser asks for. A fixed list made the Eclipse
        // Music web client (https://eclipsemusic.app -> here) fail every
        // preflight: it sends its own `X-Client-Platform` / `X-Client-Version`,
        // and Chrome refuses the real request when one of them is not allowed.
        // `Allow-Origin: *` means no credentials are involved, so echoing is safe.
        let requested = ctx
            .headers
            .get(header::ACCESS_CONTROL_REQUEST_HEADERS)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                "Content-Type, Authorization, X-API-Secret, X-Auth-Token, Range, X-Requested-With"
                    .to_string()
            });
        return Response::builder()
            .status(StatusCode::NO_CONTENT)
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .header(
                header::ACCESS_CONTROL_ALLOW_METHODS,
                "GET, POST, OPTIONS, DELETE, HEAD",
            )
            .header(header::ACCESS_CONTROL_ALLOW_HEADERS, requested)
            .header(
                "Access-Control-Expose-Headers",
                "Accept-Ranges, Content-Range, Content-Length, Content-Type",
            )
            .header("Access-Control-Allow-Private-Network", "true")
            .header(header::ACCESS_CONTROL_MAX_AGE, "86400")
            .body(Body::empty())
            .unwrap_or_else(|_| json_response(500, &serde_json::json!({"ok": 0})));
    }

    ctx.path = strip_token_prefix(&ctx.path);
    crate::storage::prune_cache_files(false);
    let p = ctx.path.clone();
    let m = method.clone();

    /* ── legacy `/stremio` alias → 301 `/nuvio` (backend.php:14688) ───────── */
    // Only the BARE path: `/stremio/manifest.json` and every other addon route
    // stay aliased to their `/nuvio` equivalent below.
    if p == "/stremio" || p == "/stremio/" {
        let mut resp = redirect("/nuvio");
        *resp.status_mut() = StatusCode::MOVED_PERMANENTLY;
        return resp;
    }

    /* ── /api/sub-proxy — subtitle fetch + SRT→VTT (backend.php:18321) ────── */
    if p == "/api/sub-proxy" && m == "GET" {
        let sub_url = ctx.qs("url");
        if sub_url.is_empty() || !(sub_url.starts_with("http://") || sub_url.starts_with("https://")) {
            return text_response(400, "text/plain; charset=utf-8", "URL is required".to_string());
        }
        if !url_host_is_public(&sub_url).await {
            return text_response(400, "text/plain; charset=utf-8", "URL is not permitted".to_string());
        }
        // 24 h cache, keyed on the subtitle url itself. The converted VTT is a
        // string, so it is stored as `{"vtt": "…"}` and served straight from the
        // cache — the old version wrote a `.vtt` file and re-read it every time.
        if let Some(hit) = crate::cache::get("sub_vtt", &sub_url) {
            if let Some(vtt) = hit.get("vtt").and_then(|v| v.as_str()) {
                return Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, "text/vtt; charset=utf-8")
                    .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                    .body(Body::from(vtt.to_string()))
                    .unwrap_or_else(|_| json_response(500, &serde_json::json!({"ok": 0})));
            }
        }
        let raw = get_contents(&sub_url, 10, header_json(&req_for_headers())).await;
        let Some(raw) = raw else {
            return text_response(502, "text/plain; charset=utf-8", "Failed to fetch subtitle".to_string());
        };
        let mut vtt = raw.clone();
        if !raw.trim_start().starts_with("WEBVTT") {
            let normalized = raw.replace("\r\n", "\n").replace('\r', "\n");
            let converted = srt_timestamps_to_vtt(&normalized);
            vtt = format!("WEBVTT\n\n{}", converted.trim_start());
        }
        crate::cache::put(
            "sub_vtt",
            &sub_url,
            &serde_json::json!({ "vtt": vtt }),
            86_400,
            0,
        );
        return text_response(200, "text/vtt; charset=utf-8", vtt);
    }

    /* ── cache observability (local/privileged) ───────────────────────────── */
    // The cache is now one redb file plus an in-process map, i.e. far less
    // inspectable than a folder of JSON — these two routes replace that.
    if p == "/api/cache/stats" && m == "GET" {
        if let Some(resp) = crate::auth::require_privileged(&ctx) {
            return resp;
        }
        return json_response(200, &crate::cache::stats());
    }
    if p == "/api/cache/clear" && (m == "POST" || m == "GET") {
        if let Some(resp) = crate::auth::require_privileged(&ctx) {
            return resp;
        }
        let prefix = ctx.qs("prefix");
        let prefix = if prefix.is_empty() {
            None
        } else {
            Some(prefix.as_str())
        };
        let removed = crate::cache::clear(prefix);
        log(
            "cache cleared",
            &serde_json::json!({"prefix": prefix.unwrap_or("*"), "l1_removed": removed}),
        );
        return json_response(
            200,
            &serde_json::json!({"ok": 1, "prefix": prefix.unwrap_or("*"), "l1_removed": removed}),
        );
    }

    /* ── auth (backend.php:18369-18460) ───────────────────────────────────── */
    if p == "/api/auth/status" && m == "GET" {
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "enabled": auth_enabled(),
                "authenticated": is_authenticated(&ctx),
                "token": if auth_privileged(&ctx) { auth_token() } else { String::new() },
                // Only to a caller that already got past the gate: telling an
                // anonymous visitor "the password is still 123456" is a map to
                // the front door, so the flag is blank unless privileged.
                "default_password": auth_privileged(&ctx)
                    && crate::auth::auth_uses_default_password(),
            }),
        );
    }

    if p == "/api/auth/login" && m == "POST" {
        let ip = auth_client_ip(&ctx);
        if let Some(wait) = auth_lockout_check(&ip) {
            if wait > 0 {
                let mut resp = json_response(
                    429,
                    &serde_json::json!({
                        "ok": 0,
                        "message": format!("Too many failed attempts. Try again in {}s. (Or reset via: pms reset-password)", wait),
                        "retryAfter": wait,
                        "locked": true
                    }),
                );
                resp.headers_mut()
                    .insert(header::RETRY_AFTER, header::HeaderValue::from_str(&wait.to_string()).unwrap());
                return resp;
            }
        }
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        let pw = input.get("password").and_then(|v| v.as_str()).unwrap_or("");
        if !auth_verify_password(pw) {
            let fail = auth_record_failure(&ip);
            if fail.locked {
                let mut resp = json_response(
                    429,
                    &serde_json::json!({
                        "ok": 0,
                        "message": format!("Too many failed attempts. Try again in {}s. (Or reset via: pms reset-password)", fail.retry_after),
                        "retryAfter": fail.retry_after,
                        "locked": true
                    }),
                );
                resp.headers_mut().insert(
                    header::RETRY_AFTER,
                    header::HeaderValue::from_str(&fail.retry_after.to_string()).unwrap(),
                );
                return resp;
            }
            return json_response(
                401,
                &serde_json::json!({
                    "ok": 0,
                    "message": format!("Invalid password. {} attempt(s) left before lockout.", fail.remaining),
                    "remainingBeforeLock": fail.remaining
                }),
            );
        }
        auth_record_success(&ip);
        let token = auth_token();
        let mut resp = json_response(200, &serde_json::json!({"ok": 1, "token": token}));
        if let Ok(v) = header::HeaderValue::from_str(&auth_cookie_header(&ctx, &token)) {
            resp.headers_mut().insert(header::SET_COOKIE, v);
        }
        return resp;
    }

    if p == "/api/auth/logout" && m == "POST" {
        let mut resp = json_response(200, &serde_json::json!({"ok": 1}));
        resp.headers_mut().insert(
            header::SET_COOKIE,
            header::HeaderValue::from_static("pm_auth=; Path=/; Max-Age=0"),
        );
        return resp;
    }

    if p == "/api/auth/password" && m == "POST" {
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        let current = input.get("current").and_then(|v| v.as_str()).unwrap_or("");
        if !auth_verify_password(current) {
            return json_response(401, &serde_json::json!({"ok": 0, "message": "Current password is wrong"}));
        }
        let next = input.get("next").and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
        if next.len() < 4 {
            return json_response(
                400,
                &serde_json::json!({"ok": 0, "message": "Password must be at least 4 characters"}),
            );
        }
        auth_set_password(&next);
        // The password change ROTATES the token. Re-issue the cookie in the same
        // response or the browser keeps a dead `pm_auth` while the JSON body
        // hands it a live `X-Auth-Token`: the gate never reappears, but every
        // `/<old-token>/…` URL already installed in a player answers locked.
        let token = auth_token();
        let mut resp = json_response(200, &serde_json::json!({"ok": 1, "token": token}));
        if let Ok(v) = header::HeaderValue::from_str(&auth_cookie_header(&ctx, &token)) {
            resp.headers_mut().insert(header::SET_COOKIE, v);
        }
        return resp;
    }

    if p == "/api/auth/token/rotate" && m == "POST" {
        if let Some(resp) = require_privileged(&ctx) {
            return resp;
        }
        let token = auth_rotate_token();
        let mut resp = json_response(200, &serde_json::json!({"ok": 1, "token": token}));
        if let Ok(v) = header::HeaderValue::from_str(&auth_cookie_header(&ctx, &token)) {
            resp.headers_mut().insert(header::SET_COOKIE, v);
        }
        return resp;
    }

    if p == "/api/auth/reset" && m == "POST" {
        if let Some(resp) = require_privileged(&ctx) {
            return resp;
        }
        auth_set_password(AUTH_DEFAULT_PASSWORD);
        let _ = std::fs::remove_file(storage_path("cache/auth_lockout.json"));
        let token = auth_token();
        let mut resp = json_response(200, &serde_json::json!({"ok": 1, "token": token}));
        if let Ok(v) = header::HeaderValue::from_str(&auth_cookie_header(&ctx, &token)) {
            resp.headers_mut().insert(header::SET_COOKIE, v);
        }
        return resp;
    }

    /* ── lightweight public routes (no version gate) ──────────────────────── */
    if p == "/api/version" {
        return json_response(200, &check_version().await);
    }

    if p == "/api/lan-ip" {
        return json_response(
            200,
            &serde_json::json!({"ok": 1, "lan_ip": lan_ip(), "port": state.port}),
        );
    }

    // backend.php:18477-18493 — status also runs the watchdog, enable spawns
    // cloudflared, disable kills it and clears the state (keeping the token).
    if p == "/api/tunnel/status" && m == "GET" {
        let mut status = tunnel::get_tunnel_status(state.port).await;
        let starting = tunnel::is_starting();
        let enabled = status.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
        // `starting` lets the dashboard show "Starting…" instead of "off" for the
        // 10-90 s a cold enable needs (see tunnel::start_async).
        if let Some(obj) = status.as_object_mut() {
            obj.insert("starting".to_string(), serde_json::json!(starting));
            // A rejected token, or a connector that never registered, must SAY so:
            // the POST only answers "starting", so without this the dashboard polled,
            // saw `enabled:false`, and displayed a bare "Tunnel is off".
            let err = tunnel::last_error();
            if !starting && !enabled {
                // A failed enable leaves no tunnel: never report the stray pid of a
                // cloudflared that already exited as a running tunnel.
                obj.insert("running".to_string(), serde_json::json!(false));
                obj.insert("pid".to_string(), serde_json::json!(0));
                if !err.is_empty() {
                    obj.insert("error".to_string(), serde_json::json!(true));
                    obj.insert("message".to_string(), serde_json::json!(err));
                }
            }
        }
        return json_response(200, &status);
    }

    if p == "/api/tunnel/enable" && m == "POST" {
        // Changing the machine's internet exposure is an operator action.
        if let Some(resp) = require_privileged(&ctx) {
            return resp;
        }
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        let raw_token = input
            .get("tunnel_token")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        // A malformed token is rejected HERE, synchronously: the user gets the reason
        // in the click's own response instead of a "starting" spinner that ends in
        // "Tunnel is off" twenty seconds later. Nothing is stored, nothing is killed.
        let token = match tunnel::validate_token_input(&raw_token) {
            Ok(t) => t,
            Err(message) => {
                return json_response(
                    200,
                    &serde_json::json!({
                        "ok": 0,
                        "enabled": false,
                        "starting": false,
                        "error": true,
                        "message": message,
                    }),
                );
            }
        };
        // NON-BLOCKING: a cold enable can take 10-90 s (install the Termux
        // cloudflared, write a CA bundle, wait for the connector to register), and
        // holding the request that long reads as a hang on a phone. The dashboard
        // already polls /api/tunnel/status every 1.5 s and finishes on
        // `enabled && tunnel_url`, so answer "starting" and let it watch.
        let fresh = tunnel::start_async(state.port, &token);
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "enabled": false,
                "starting": true,
                "message": if fresh {
                    "Starting Cloudflare tunnel..."
                } else {
                    "Cloudflare tunnel is already starting..."
                },
            }),
        );
    }

    if p == "/api/tunnel/disable" && m == "POST" {
        if let Some(resp) = require_privileged(&ctx) {
            return resp;
        }
        return json_response(200, &tunnel::disable().await);
    }

    /* ── catalog settings (backend.php:18496-18657) ───────────────────────── */
    if p == "/api/catalog-settings" && m == "GET" {
        // `fd_load_catalog_settings()` always returns a FULL document; a missing
        // `storage/catalog_settings.json` is not an empty settings object. The
        // dashboard renders the Catalog Configuration / Streams Configuration
        // panels from this shape, so returning `{}` left every control blank.
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "settings": load_catalog_settings_merged(),
                "catalog_options": default_catalog_options(),
                "is_tunnel": false,
            }),
        );
    }

    if p == "/api/catalog-settings" && m == "POST" {
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        if !input.is_object() {
            return json_response(400, &serde_json::json!({"ok": 0, "error": "Invalid JSON input"}));
        }
        let mut current = load_catalog_settings_merged();
        if current.get("enabled_types").is_none() {
            current["enabled_types"] = serde_json::json!({});
        }
        if current.get("enabled_catalogs").is_none() {
            current["enabled_catalogs"] = serde_json::json!({});
        }
        if let Some(v) = input.get("catalogs_enabled") {
            current["catalogs_enabled"] = serde_json::json!(v.as_bool().unwrap_or(false));
        }
        if let Some(v) = input.get("country") {
            let code: String = v
                .as_str()
                .unwrap_or("")
                .chars()
                .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                .collect::<String>()
                .to_ascii_uppercase();
            current["country"] = serde_json::json!(code);
        }
        if let Some(types) = input.get("enabled_types").and_then(|v| v.as_object()) {
            for key in ["movie", "series", "other"] {
                if let Some(v) = types.get(key) {
                    current["enabled_types"][key] = serde_json::json!(v.as_bool().unwrap_or(false));
                }
            }
        }
        if let Some(cats) = input.get("enabled_catalogs").and_then(|v| v.as_object()) {
            for (cid, v) in cats {
                current["enabled_catalogs"][cid] = serde_json::json!(v.as_bool().unwrap_or(false));
            }
        }
        // Master upstream switch: keep the stored list when disabling with an
        // empty incoming list (backend.php:18540-18566).
        let upstream_disabled = input
            .get("upstream_enabled")
            .map(|v| !v.as_bool().unwrap_or(true))
            .unwrap_or(false);
        if let Some(v) = input.get("upstream_enabled") {
            current["upstream_enabled"] = serde_json::json!(v.as_bool().unwrap_or(false));
        }
        if let Some(list) = input.get("upstream_manifests").and_then(|v| v.as_array()) {
            let cleaned: Vec<Value> = list
                .iter()
                .filter(|m| m.get("url").and_then(|u| u.as_str()).map(|u| !u.trim().is_empty()).unwrap_or(false))
                .map(|m| {
                    serde_json::json!({
                        "name": m.get("name").and_then(|v| v.as_str()).unwrap_or("Addon").trim(),
                        "url": m.get("url").and_then(|v| v.as_str()).unwrap_or("").trim(),
                        "id": m.get("id").and_then(|v| v.as_str()).unwrap_or("").trim(),
                        "version": m.get("version").and_then(|v| v.as_str()).unwrap_or("").trim(),
                    })
                })
                .collect();
            if !(upstream_disabled && cleaned.is_empty()) {
                current["upstream_manifests"] = Value::Array(cleaned);
            }
        }
        if let Some(sc_in) = input.get("stream_config").and_then(|v| v.as_object()) {
            if current.get("stream_config").map(|v| v.is_object()).unwrap_or(false) == false {
                current["stream_config"] = serde_json::json!({});
            }
            let sc = current["stream_config"].as_object_mut();
            if let Some(sc) = sc {
                let mut sc_value = Value::Object(sc.clone());
                apply_stream_config(&mut sc_value, sc_in);
                current["stream_config"] = sc_value;
            }
        }
        let saved = write_catalog_settings(&current);
        if saved {
            // New stream configuration must apply immediately. This port does not
            // cache stream RESPONSES (the AIOStreams filters run per request), so
            // only the (currently empty) `stream` prefix would need dropping — done
            // anyway so a future response cache cannot silently serve old filters.
            crate::cache::clear(Some("stream"));
        }
        return json_response(
            200,
            &serde_json::json!({
                "ok": if saved { 1 } else { 0 },
                "settings": current,
                "message": if saved { "Catalog settings updated successfully" } else { "Failed to save settings" },
            }),
        );
    }

    /* ── media-ids (backend.php:18668-18712) ──────────────────────────────── */
    if p == "/api/media-ids" && (m == "POST" || m == "DELETE") {
        if let Some(resp) = crate::auth::require_auth(&ctx) {
            return resp;
        }
        if m == "DELETE" {
            let raw = ctx.qs("media_ids");
            let mut pairs = serde_json::Map::new();
            for chunk in raw.split(',') {
                let t = chunk.trim();
                if t.is_empty() {
                    continue;
                }
                if let Some((k, v)) = t.split_once(':') {
                    if !k.is_empty() && !v.is_empty() {
                        pairs.insert(k.to_string(), Value::String(v.to_string()));
                    }
                }
            }
            if pairs.is_empty() {
                return json_response(
                    400,
                    &serde_json::json!({"ok": 0, "error": "Missing media_ids (prefix:value list)"}),
                );
            }
            let url = format!("{}/media-ids", wp_api_base());
            let r = http_json(
                &url,
                &[
                    ("media_ids".to_string(), Value::Object(pairs.clone()).to_string()),
                    ("delete".to_string(), "1".to_string()),
                ],
                "POST",
                12,
            )
            .await;
            return json_response(
                200,
                &serde_json::json!({"ok": 1, "deleted": pairs.len(), "upstream": r}),
            );
        }
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        if !input.is_object() {
            return json_response(400, &serde_json::json!({"ok": 0, "error": "Invalid JSON body"}));
        }
        let media_ids = input
            .get("media_ids")
            .cloned()
            .filter(|v| v.is_object())
            .unwrap_or_else(|| serde_json::json!({}));
        let title = jstr(&input, "title");
        if media_ids.as_object().map(|o| o.is_empty()).unwrap_or(true) || title.is_empty() {
            return json_response(
                400,
                &serde_json::json!({"ok": 0, "error": "media_ids and title are required"}),
            );
        }
        // The port has no local Manticore; WordPress owns media_ids_idx.
        let url = format!("{}/media-ids", wp_api_base());
        let params = vec![
            ("media_ids".to_string(), media_ids.to_string()),
            ("title".to_string(), title),
            ("year".to_string(), input.get("year").and_then(|v| v.as_i64()).unwrap_or(0).to_string()),
            ("imdb_id".to_string(), jstr(&input, "imdb_id")),
            (
                "media_type".to_string(),
                {
                    let t = jstr(&input, "media_type");
                    if t.is_empty() { "movie".to_string() } else { t }
                },
            ),
            (
                "post_id".to_string(),
                input.get("post_id").and_then(|v| v.as_i64()).unwrap_or(0).to_string(),
            ),
        ];
        let written = http_json(&url, &params, "POST", 12).await;
        let count = written
            .get("written")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| {
                if written.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
                    media_ids.as_object().map(|o| o.len() as i64).unwrap_or(0)
                } else {
                    0
                }
            });
        return json_response(
            if count > 0 { 200 } else { 500 },
            &serde_json::json!({
                "ok": if count > 0 { 1 } else { 0 },
                "post_id": input.get("post_id").and_then(|v| v.as_i64()).unwrap_or(0),
                "written": count,
            }),
        );
    }

    /* ── validate-manifest (backend.php:18715-18751) ──────────────────────── */
    if p == "/api/validate-manifest" && m == "POST" {
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        let mut url = jstr(&input, "url");
        if url.is_empty() {
            return json_response(400, &serde_json::json!({"ok": 0, "error": "Please provide a manifest URL."}));
        }
        if let Some(rest) = url.strip_prefix("stremio://") {
            url = format!("https://{}", rest);
        }
        if !url.to_ascii_lowercase().contains("/manifest.json") {
            url = format!("{}/manifest.json", rtrim(&url));
        }
        let m2 = http_json(&url, &[], "GET", 8).await;
        let id = jstr(&m2, "id");
        let name = jstr(&m2, "name");
        if id.is_empty() || name.is_empty() {
            return json_response(
                400,
                &serde_json::json!({
                    "ok": 0,
                    "error": "Invalid Stremio manifest. Make sure the URL points to a valid manifest.json responding with id and name."
                }),
            );
        }
        let version = {
            let v = jstr(&m2, "version");
            if v.is_empty() { "1.0.0".to_string() } else { v }
        };
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "manifest": {
                    "id": id,
                    "name": name,
                    "version": version,
                    "description": jstr(&m2, "description"),
                    "url": url,
                    "resources": m2.get("resources").cloned().unwrap_or_else(|| serde_json::json!([])),
                },
                "message": "Manifest validated successfully.",
            }),
        );
    }

    /* ── country (backend.php:18755-18802) ────────────────────────────────── */
    if p == "/api/country" && m == "GET" {
        let settings = read_catalog_settings();
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "country": detect_country(&ctx),
                "configured_country": jstr(&settings, "country"),
                "available_countries": country_list(),
            }),
        );
    }

    if p == "/api/country" && m == "POST" {
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        if !input.is_object() {
            return json_response(400, &serde_json::json!({"ok": 0, "error": "Invalid JSON input"}));
        }
        let code: String = jstr(&input, "country")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>()
            .to_ascii_uppercase();
        let mut settings = read_catalog_settings();
        if !settings.is_object() {
            settings = serde_json::json!({});
        }
        settings["country"] = serde_json::json!(code);
        let saved = write_catalog_settings(&settings);
        return json_response(
            200,
            &serde_json::json!({
                "ok": if saved { 1 } else { 0 },
                "country": detect_country(&ctx),
                "configured_country": code,
                "message": if saved { "Country updated successfully" } else { "Failed to save country" },
            }),
        );
    }

    /* ── session / clock / provision (backend.php:18804-18925) ────────────── */
    if p == "/api/session" {
        let bot_id = get_bot_id();
        let meta = load_session_meta();
        let has_session = has_local_session("") && !bot_id.is_empty();
        let pool = get_bot_pool();
        let mut bot_username = jstr(&meta, "bot_username");
        let mut bot_name = jstr(&meta, "bot_name");
        if (bot_username.is_empty() || bot_name.is_empty()) && !pool.is_empty() {
            if let Some(pb) = pool.iter().find(|b| jstr(b, "bot_id") == bot_id) {
                if bot_username.is_empty() {
                    bot_username = jstr(pb, "bot_username");
                }
                if bot_name.is_empty() {
                    bot_name = jstr(pb, "bot_name");
                }
            }
        }
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                // `version` is what a UI displays, so it must be a clean X.Y.Z:
                // `app_version()` can be a git stamp (`2.9.0-9-ga8a1f3d-dirty`),
                // which the settings badge rendered verbatim.
                "version": crate::config::descriptor_version(),
                "engine_version": app_version(),
                "client_version": crate::config::client_version(),
                "has_session": has_session,
                "is_provisioning": is_guest_provision_in_progress(),
                "bot_id": if has_session { bot_id } else { String::new() },
                "bot_username": if has_session { bot_username } else { String::new() },
                "bot_name": if has_session { bot_name } else { String::new() },
                "api_secret": if has_session && auth_privileged(&ctx) { get_api_secret() } else { String::new() },
                "device_id": get_device_id(),
                "bot_count": pool.len(),
                "bot_pool": pool,
                "environment_ok": true,
                "environment_fatal": false,
                "environment_problems": [],
                "environment_hints": [],
                // Neutral label: this is shown in the dashboard's bot/session
                // card, so it must not name the engine.
                "runtime": "pencarimovie-server",
            }),
        );
    }

    if p == "/api/clock-check" && m == "GET" {
        let probe = measure_clock_offset().await;
        let abs = probe.offset.abs();
        return json_response(
            200,
            &serde_json::json!({
                "ok": if probe.ok { 1 } else { 0 },
                "offset_seconds": probe.offset,
                "server_time": probe.server_time,
                "local_time": probe.local_time,
                "skewed": probe.ok && abs > 30,
                "critical": probe.ok && abs > 60,
                "message": probe.error.unwrap_or_default(),
            }),
        );
    }

    if p == "/api/provision" && (m == "GET" || m == "POST") {
        // SECURITY — this endpoint SPENDS a provision at the origin and answers
        // with the WordPress `api_secret`, but it had no gate at all. The engine
        // binds 0.0.0.0 on Android and on a detected PaaS, so any device on the
        // same wifi could drain the guest-bot pool and harvest the secret with a
        // plain GET. The dashboard calls it from loopback, which stays allowed.
        if let Some(resp) = require_privileged(&ctx) {
            return resp;
        }
        // SECURITY — `Access-Control-Allow-Origin: *` is sent on every response,
        // so without this a page on ANY site the user visits could POST here from
        // the browser, read the JSON (CORS lets it), and walk away with the
        // secret: the request comes FROM this machine, so `require_privileged`
        // alone cannot tell it apart. Same-origin dashboard use (127.0.0.1,
        // localhost, or the LAN/tunnel address it was loaded from) is allowed.
        let origin = ctx.hdr("origin");
        if !origin.is_empty() {
            let port = state.port;
            let allowed = [
                ctx.base_url.trim_end_matches('/').to_string(),
                format!("http://127.0.0.1:{port}"),
                format!("http://localhost:{port}"),
            ];
            if !allowed.iter().any(|a| a == &origin) {
                return json_response(
                    403,
                    &serde_json::json!({
                        "ok": 0,
                        "message": "Cross-origin provisioning is not allowed."
                    }),
                );
            }
        }
        // The user emptied the pool on purpose (removed the last bot or pressed
        // Disconnect All). Handing them another guest bot here is what made those
        // buttons look like no-ops, so answer with the token-gate reason instead.
        return match auto_provision_guest(&state.telegram).await {
            Ok(prov) => json_response(
                200,
                &serde_json::json!({
                    "ok": 1,
                    "message": "Guest bot session initialized successfully.",
                    "bot_id": prov.bot_id,
                    "bot_username": prov.bot_username,
                    "bot_name": prov.bot_name,
                    // Privileged callers only (checked above): the engine itself
                    // reads this file when it talks to WordPress.
                    "api_secret": get_api_secret(),
                    "pool": get_bot_pool(),
                }),
            ),
            Err(prov) => json_response(
                500,
                &serde_json::json!({
                    "ok": 0,
                    "message": prov.error.unwrap_or_else(|| "Could not obtain guest bot session from server.".to_string()),
                }),
            ),
        };
    }

    /* ── bots (backend.php:18928-19112) ───────────────────────────────────── */
    if p == "/api/bots" && m == "GET" {
        let pool = get_bot_pool();
        let active = get_bot_id();
        let list: Vec<Value> = pool
            .iter()
            .map(|b| {
                let id = jstr(b, "bot_id");
                let mut out = b.clone();
                if let Some(obj) = out.as_object_mut() {
                    obj.insert("has_session".into(), serde_json::json!(has_local_session(&id)));
                    obj.insert("is_active".into(), serde_json::json!(id == active));
                }
                out
            })
            .collect();
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "active_bot_id": active,
                "total_bots": list.len(),
                // How many of the entries above come from `BOT_TOKENS` (they are
                // shared by every instance; the rest are this instance's disk
                // state and are not visible to its siblings).
                "env_bots": env_bot_tokens().len(),
                "bots": list,
            }),
        );
    }

    if p == "/api/bots/add" && m == "POST" {
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        if !input.is_object() {
            return json_response(400, &serde_json::json!({"ok": 0, "message": "Invalid JSON body"}));
        }
        let mut tokens: Vec<String> = Vec::new();
        let single = jstr(&input, "bot_token");
        if !single.is_empty() {
            tokens.push(single);
        } else if let Some(list) = input.get("tokens").and_then(|v| v.as_array()) {
            tokens.extend(list.iter().filter_map(|t| t.as_str()).map(|t| t.trim().to_string()));
        } else if let Some(text) = input.get("tokens_text").and_then(|v| v.as_str()) {
            tokens.extend(
                text.split(|c: char| c == ',' || c.is_whitespace())
                    .map(|t| t.trim().to_string())
                    .filter(|t| !t.is_empty()),
            );
        }
        let mut seen = std::collections::HashSet::new();
        tokens.retain(|t| !t.is_empty() && seen.insert(t.clone()));
        if tokens.is_empty() {
            return json_response(
                400,
                &serde_json::json!({"ok": 0, "message": "Provide bot_token, tokens[] or tokens_text."}),
            );
        }
        let reserve = is_guest_provision_in_progress();
        let mut added: Vec<Value> = Vec::new();
        let mut failed: Vec<Value> = Vec::new();
        for token in tokens {
            let id_part = token.split(':').next().unwrap_or("").to_string();
            let hint = if id_part.is_empty() {
                format!("add_{}", rand_suffix())
            } else {
                id_part.clone()
            };
            match state.telegram.boot(&hint, Some(&token)).await {
                Ok(entry) => {
                    add_pool_bot(serde_json::json!({
                        "bot_id": entry.identity.id,
                        "bot_username": entry.identity.username,
                        "bot_name": entry.identity.first_name,
                        "status": "online",
                        "is_active": true,
                        "updated_at": now(),
                    }));
                    added.push(serde_json::json!({
                        "bot_id": entry.identity.id,
                        "bot_username": entry.identity.username,
                        "bot_name": entry.identity.first_name,
                    }));
                }
                Err(e) => failed.push(serde_json::json!({
                    "token": if id_part.is_empty() { "…".to_string() } else { format!("{}:…", id_part) },
                    "error": e,
                })),
            }
        }
        if !added.is_empty() && get_bot_id().is_empty() {
            let first = &added[0];
            save_session_meta(
                &jstr(first, "bot_id"),
                &jstr(first, "bot_username"),
                &jstr(first, "bot_name"),
            );
        } else if !added.is_empty() {
        }
        let ok = !added.is_empty();
        return json_response(
            if ok { 200 } else { 400 },
            &serde_json::json!({
                "ok": if ok { 1 } else { 0 },
                "message": if ok {
                    format!("Added {} bot(s) to the pool.", added.len())
                } else {
                    "No bot could be added (check the token).".to_string()
                },
                "added": added,
                "failed": failed,
                "added_count": added.len(),
                "pool": get_bot_pool(),
                "tunnel_restarted": reserve,
            }),
        );
    }

    if p == "/api/bots/remove" && m == "POST" {
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        if !input.is_object() {
            return json_response(400, &serde_json::json!({"ok": 0, "message": "Invalid JSON body"}));
        }
        let bot_id = jstr(&input, "bot_id");
        if bot_id.is_empty() {
            return json_response(400, &serde_json::json!({"ok": 0, "message": "bot_id is required."}));
        }
        // A `BOT_TOKENS` bot has no file entry, so deleting it here would only
        // hide it until the next request re-adds it from the environment.
        if env_token_for(&bot_id).is_some() {
            return json_response(
                400,
                &serde_json::json!({
                    "ok": 0,
                    "message": "This bot comes from the BOT_TOKENS environment variable. Remove its token there and restart the server.",
                    "limited_by_env": true,
                    "pool": get_bot_pool(),
                    "active_bot_id": get_bot_id(),
                }),
            );
        }
        let pool = remove_pool_bot(&bot_id);
        // `has_session` and `bots_remaining` are what the dashboard needs to
        // choose between "the primary switched to another bot" and "show the bot
        // token gate". Without them it re-read `/api/session`, which for a pool
        // row that can never log in answered `has_session: false` and dropped the
        // UI into guest provisioning instead of the gate.
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "message": "Bot removed from pool.",
                "pool": pool,
                "active_bot_id": get_bot_id(),
                "bots_remaining": pool.len(),
                "has_session": has_local_session(""),
            }),
        );
    }

    if p == "/api/bots/set-active" && m == "POST" {
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        if !input.is_object() {
            return json_response(400, &serde_json::json!({"ok": 0, "message": "Invalid JSON body"}));
        }
        let bot_id = jstr(&input, "bot_id");
        if bot_id.is_empty() {
            return json_response(400, &serde_json::json!({"ok": 0, "message": "bot_id is required."}));
        }
        let target = get_bot_pool()
            .into_iter()
            .find(|b| jstr(b, "bot_id") == bot_id);
        let Some(target) = target else {
            return json_response(404, &serde_json::json!({"ok": 0, "message": "Bot ID not found in pool."}));
        };
        let username = jstr(&target, "bot_username");
        let name = jstr(&target, "bot_name");
        save_session_meta(&bot_id, &username, &name);
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "active_bot_id": bot_id,
                "bot_username": username,
                "bot_name": name,
                "tunnel_restarted": false,
            }),
        );
    }

    /* ── version gate: REMOVED on purpose ─────────────────────────────────── */
    // The PHP build blocked every /api/* call with HTTP 426 when the engine was
    // older than the WordPress minimum. That made an outdated binary unusable
    // instead of merely out of date, and it fought the native build (which
    // reports `git describe` version strings). The check still RUNS — `/api/version`
    // returns `update_needed`, `minimum_version` and `update_url`, and the
    // dashboard shows a banner in #settings / #addon with a one-click update —
    // but nothing is ever blocked.
    let _ = check_version;

    /* ── desktop self-update ──────────────────────────────────────────────── */
    // `check` only reads the GitHub release feed. `apply` downloads and then hands
    // the archive to the shipped apply-update script, so it is privileged (a LAN
    // peer must not be able to push a binary onto the host).
    if p == "/api/update/check" && m == "GET" {
        return json_response(200, &crate::update::check().await);
    }
    if p == "/api/update/apply" && m == "POST" {
        if let Some(resp) = require_privileged(&ctx) {
            return resp;
        }
        return json_response(200, &crate::update::apply().await);
    }

    /* ── diagnostics / reports (backend.php:19149-19202) ──────────────────── */
    if p == "/api/diagnostics" && m == "GET" {
        // Auto-upload, driven by the Android service's 3s poll. The PHP spawns a
        // detached CLI worker per incident; this port spawns no processes, so the
        // flush rides here instead — otherwise `auto_report: true` uploaded
        // nothing at all (it was stored and reported, never acted on).
        // `has_pending()` first so an idle poll costs one `read_dir`, and
        // `take_auto_flush_slot()` throttles a failing origin to one try / 120s.
        if crate::diagnostics::has_pending() && crate::diagnostics::take_auto_flush_slot() {
            let (sent, failed) = crate::diagnostics::flush_pending().await;
            if sent + failed > 0 {
                crate::log::log(
                    "diag auto flush",
                    &serde_json::json!({"sent": sent, "failed": failed}),
                );
            }
        }
        let mut reports = diag_list();
        reports.truncate(10);
        // `activity` is what the Android app polls to decide whether it is
        // "Active" (hold the wake/Wi-Fi lock, say so in the notification) or
        // "Idle" (release them after a grace period) — the TDLib app's
        // markActive/ACTIVE_LOCK_TIMEOUT_MS behaviour.
        let streams = crate::download::active_streams();
        let preflights = crate::download::active_preflights();
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "info": device_info(),
                "counts": diag_counts(),
                "activity": {
                    "streams": streams,
                    "preflights": preflights,
                    "total": streams + preflights,
                },
                "reports": reports,
            }),
        );
    }

    if p == "/api/diagnostics/settings" {
        if m == "POST" || ctx.has_q("enabled") {
            if let Some(resp) = require_privileged(&ctx) {
                return resp;
            }
            let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
            if let Some(v) = input.get("auto_report") {
                set_auto_report(v.as_bool().unwrap_or(true));
            } else if ctx.has_q("enabled") {
                let v = ctx.qs("enabled").to_ascii_lowercase();
                set_auto_report(v == "1" || v == "true" || v == "on");
            }
        }
        return json_response(
            200,
            &serde_json::json!({"ok": 1, "auto_report": auto_report_enabled()}),
        );
    }

    if p == "/api/reports" && m == "GET" {
        if ctx.has_q("clear") || ctx.has_q("send") || ctx.has_q("flush") {
            if let Some(resp) = require_privileged(&ctx) {
                return resp;
            }
        }
        if ctx.has_q("clear") {
            diag_clear();
            return json_response(200, &serde_json::json!({"ok": 1, "cleared": true}));
        }
        if ctx.has_q("send") {
            let file = ctx.qs("send");
            let base = std::path::Path::new(&file)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            if base.is_empty() {
                return json_response(400, &serde_json::json!({"ok": 0, "message": "file is required"}));
            }
            let path = crate::diagnostics::reports_dir().join(&base);
            let ok = crate::diagnostics::report_send(&path).await;
            return json_response(
                200,
                &serde_json::json!({
                    "ok": if ok { 1 } else { 0 },
                    "file": base,
                    "message": if ok { "Report sent." } else { "Report upload failed." },
                }),
            );
        }
        if ctx.has_q("flush") {
            let (sent, failed) = crate::diagnostics::flush_pending().await;
            return json_response(200, &serde_json::json!({"ok": 1, "sent": sent, "failed": failed}));
        }
        if ctx.has_q("file") {
            let name = ctx.qs("file");
            let base = std::path::Path::new(&name)
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            let path = crate::diagnostics::reports_dir().join(base);
            return match std::fs::read_to_string(path)
                .ok()
                .and_then(|t| serde_json::from_str::<Value>(&t).ok())
            {
                Some(v) => json_response(200, &serde_json::json!({"ok": 1, "report": v})),
                None => json_response(404, &serde_json::json!({"ok": 0, "message": "Report not found"})),
            };
        }
        let limit: usize = ctx.qs("limit").parse().unwrap_or(20).max(1);
        let mut reports = diag_list();
        reports.truncate(limit);
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "auto_report": auto_report_enabled(),
                "counts": diag_counts(),
                "reports": reports,
            }),
        );
    }

    /* ── debug-mode / logs (backend.php:19206-19260) ──────────────────────── */
    if p == "/api/debug-mode" {
        if m == "POST" {
            let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
            set_debug_enabled(input.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false));
            return json_response(200, &serde_json::json!({"ok": 1, "enabled": is_debug_enabled()}));
        }
        if m == "GET" {
            if ctx.has_q("enable") {
                let v = ctx.qs("enable").to_ascii_lowercase();
                set_debug_enabled(v == "1" || v == "true" || v == "on");
            }
            return json_response(200, &serde_json::json!({"ok": 1, "enabled": is_debug_enabled()}));
        }
    }

    if p == "/api/logs" && m == "GET" {
        if !is_debug_enabled() {
            return json_response(
                403,
                &serde_json::json!({"ok": 0, "message": "Debug logging is currently disabled. Enable it via /api/debug-mode?enable=1"}),
            );
        }
        let req_file = {
            let a = ctx.qs("file");
            let a = if a.is_empty() { ctx.qs("path") } else { a };
            let a = if a.is_empty() { "debug.log".to_string() } else { a };
            a.to_ascii_lowercase()
        };
        let base = std::path::Path::new(&req_file)
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let target = match base.as_str() {
            "debug.log" => Some(storage_path("debug.log")),
            _ => None,
        };
        let Some(target) = target else {
            return json_response(400, &serde_json::json!({"ok": 0, "message": "File not allowed"}));
        };
        let text = std::fs::read_to_string(target).unwrap_or_default();
        let lines: Vec<&str> = text.split('\n').collect();
        let tail: Vec<&str> = lines.iter().rev().take(400).rev().cloned().collect();
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "file": base,
                "total_lines": lines.len(),
                "lines": tail,
            }),
        );
    }

    /* ── bot login/logout (backend.php:19416-19576) ───────────────────────── */
    if p == "/api/botlogin" && m == "POST" {
        let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
        let token = {
            let a = jstr(&input, "bot_token");
            if a.is_empty() { jstr(&input, "token") } else { a }
        };
        if token.is_empty() {
            return json_response(400, &serde_json::json!({"ok": 0, "message": "bot_token is required."}));
        }
        let hint = {
            let id = token.split(':').next().unwrap_or("").to_string();
            if id.is_empty() {
                format!("login_{}", rand_suffix())
            } else {
                id.clone()
            }
        };
        return match state.telegram.boot(&hint, Some(&token)).await {
            Ok(entry) => {
                save_session_meta(&entry.identity.id, &entry.identity.username, &entry.identity.first_name);
                json_response(
                    200,
                    &serde_json::json!({
                        "ok": 1,
                        "message": "Bot logged in successfully.",
                        "bot_id": entry.identity.id,
                        "bot_username": entry.identity.username,
                        "bot_name": entry.identity.first_name,
                        "api_secret": get_api_secret(),
                        "pool": get_bot_pool(),
                    }),
                )
            }
            Err(e) => json_response(500, &serde_json::json!({"ok": 0, "message": e})),
        };
    }

    if p == "/api/botlogout" && m == "POST" {
        // Identity first, then sessions/pool (the PHP's self-resurrecting-pool fix).
        let active = get_bot_id();
        let _ = std::fs::remove_file(storage_path("session_meta.json"));
        let _ = std::fs::remove_file(storage_path("bot_id.txt"));
        clear_session();
        let _ = std::fs::remove_file(storage_path("bot_pool.json"));
        clear_api_secret();
        log("bot logout completed", &serde_json::json!({"previous_bot_id": active}));
        return json_response(200, &serde_json::json!({"ok": 1, "message": "Bot session cleared."}));
    }

    /* ── proxy-stream (backend.php:19577+) ────────────────────────────────── */
    if p == "/api/proxy-stream" && m == "GET" {
        let action = ctx.qs("action");
        if action.is_empty() {
            return json_response(400, &serde_json::json!({"ok": 0, "message": "action is required."}));
        }
        // The serializer is NOT Send: build the query string in its own scope so
        // it is dropped before the `.await` (a `url::form_urlencoded::Serializer`
        // held across an await makes the whole handler future non-Send).
        let query_string = {
            let mut qs = url::form_urlencoded::Serializer::new(String::new());
            for (k, v) in &ctx.query {
                if k != "action" {
                    qs.append_pair(k, v);
                }
            }
            qs.finish()
        };
        // REST, not `admin-ajax.php`: the same mapping `wp_data::fetch_stream_ajax`
        // uses (`posts` -> `/posts`, `post_files` -> `/post-files`). The AJAX bridge
        // was the old PHP dashboard's transport, and keeping it here meant the port
        // still depended on a PHP endpoint that no other route touches. This route
        // exists only so the shell can read the origin without CORS.
        let url = format!(
            "{}/{}?{}",
            wp_api_base().trim_end_matches('/'),
            action.replace('_', "-"),
            query_string
        );
        let mut extra = HashMap::new();
        extra.insert("Accept".to_string(), "application/json".to_string());
        let body = get_contents(&url, 15, extra).await;
        return match body {
            None => {
                log("proxy-stream failed", &serde_json::json!({"action": action}));
                json_response(
                    502,
                    &serde_json::json!({"ok": 0, "message": "Failed to fetch data from PencariMovie."}),
                )
            }
            Some(text) => match serde_json::from_str::<Value>(&text) {
                Ok(v) => json_response(200, &v),
                Err(_) => text_response(200, "application/json; charset=utf-8", text),
            },
        };
    }

    /* ── download ─────────────────────────────────────────────────────────── */
    if is_public_download_path(&p) {
        // The Eclipse web client lives on a public HTTPS origin and plays a
        // LOOPBACK url, so the media response itself must carry
        // `Access-Control-Allow-Private-Network` or Chrome drops it after the
        // preflight succeeded.
        return allow_private_network(handle_download(&state, &ctx).await);
    }

    /* ── addon aliases + manifest ─────────────────────────────────────────── */
    let mut addon_path = p.clone();
    if let Some(rest) = p.strip_prefix("/nuvio") {
        addon_path = rest.to_string();
    } else if let Some(rest) = p.strip_prefix("/stremio") {
        addon_path = rest.to_string();
    }
    if addon_path.is_empty() {
        addon_path = "/manifest.json".to_string();
    }
    if addon_path == "/manifest.json" {
        // `ETag` + `max-age=1800, stale-while-revalidate=3600`, 304 on a matching
        // `If-None-Match` — the PHP's descriptor response (backend.php:15468-15488).
        return manifest_response(&ctx, &manifest(&ctx).await);
    }

    /* ── /stream ──────────────────────────────────────────────────────────── */
    if let Some(rest) = addon_path.strip_prefix("/stream/") {
        if let Some((type_part, id_part)) = rest.split_once('/') {
            let id = id_part.strip_suffix(".json").map(|s| s.to_string());
            if let Some(id) = id {
                if !is_authenticated(&ctx) {
                    // The sponsored item still goes on top: the user is being
                    // told to re-install, which is exactly when they should see
                    // it. Never cached — a locked answer must not outlive the
                    // token/URL it was produced for.
                    return stremio_locked_stream(&ctx.base_url, sponsor_item().await);
                }
                let decoded_id = crate::download::percent_decode(&id);
                return handle_stream(&state, &ctx, type_part, &decoded_id).await;
            }
        }
        return not_found_json();
    }

    if addon_path.starts_with("/catalog/") {
        return handle_catalog(&ctx, &addon_path).await;
    }
    if addon_path.starts_with("/meta/") {
        return handle_meta(&ctx, &addon_path).await;
    }
    if addon_path.starts_with("/subtitles/") {
        return handle_subtitles(&addon_path).await;
    }

    /* ── eclipse ──────────────────────────────────────────────────────────── */
    // `/music` is the canonical name for this addon; `/eclipse` is kept as
    // an alias so every URL already installed in a client keeps resolving.
    let p = if p == "/music" || p.starts_with("/music/") {
        p.replacen("/music", "/eclipse", 1)
    } else {
        p
    };
    if p.starts_with("/eclipse") {
        if !is_authenticated(&ctx) {
            return eclipse_locked_response(&ctx.base_url);
        }
        if p == "/eclipse" || p == "/eclipse/manifest.json" {
            // A UNIVERSAL music addon, not an Eclipse-branded one: the id no
            // longer names a client, so any Stremio-compatible music app can
            // install it. `/music` is the canonical path and `/eclipse` stays as
            // the alias every already-installed addon URL uses.
            //
            // The descriptor itself is pinned to backend.php:14315-14346: same
            // `types`, the `latest` catalog row and the `quality` setting, since
            // clients render their catalog list and Settings sheet straight from
            // it. The icon stays the SHIPPED logo (the PHP points at a WordPress
            // upload; operator choice here is to never depend on that host).
            return stremio_json(
                200,
                &serde_json::json!({
                    "id": "org.pencarimovie.music",
                    "name": "PencariMusic",
                    "version": "1.1.0",
                    "description": "Universal music addon - stream Telegram music and audio files via PencariMovie Server",
                    "resources": ["search", "stream", "catalog"],
                    "types": ["track", "album", "artist"],
                    "contentType": "music",
                    "icon": format!("{}/logo.png", rtrim(&ctx.base_url)),
                    "catalogs": [
                        {"id": "latest", "type": "track", "name": "Latest Telegram Tracks"}
                    ],
                    "settings": [{
                        "key": "quality",
                        "type": "select",
                        "label": "Audio quality",
                        "default": "high",
                        "options": [
                            {"value": "high", "label": "Original / High Quality"},
                            {"value": "normal", "label": "Normal Quality"}
                        ]
                    }]
                }),
                "max-age=3600",
            );
        }
        if p.starts_with("/eclipse/search") {
            let q = {
                let a = ctx.qs("q");
                if a.is_empty() { ctx.qs("query") } else { a }
            };
            let tracks = eclipse_search(&q, 30).await;
            return stremio_json(200, &serde_json::json!({"tracks": tracks}), "no-store");
        }
        if let Some(rest) = p.strip_prefix("/eclipse/catalog/") {
            // `latest` (and any keyword catalog) reuses the music search: the
            // origin indexes tracks, not rows, so "latest" is a plain search.
            let cat = rest.trim_end_matches(".json").trim();
            let q = if cat == "latest" || cat.is_empty() { "" } else { cat };
            let tracks = eclipse_search(q, 50).await;
            return stremio_json(200, &serde_json::json!({"tracks": tracks}), "no-store");
        }
        if let Some(rest) = p.strip_prefix("/eclipse/stream/") {
            return eclipse_stream(&ctx, rest.trim_end_matches(".json")).await;
        }
        if let Some(rest) = p.strip_prefix("/eclipse/play/") {
            return eclipse_stream(&ctx, rest.trim_end_matches(".json")).await;
        }
        return stremio_json(404, &serde_json::json!({"error": "Not found"}), "no-store");
    }

    /* ── /api/resolve-shortcode, /api/warmup-resolve ──────────────────────── */
    // GET **and** POST (`$_REQUEST` in the PHP, backend.php:19313), and the plural
    // `short_codes` batch form the dashboard uses to pre-warm a search page
    // (public/app.js:5064). The batch branch must come first.
    if p == "/api/resolve-shortcode" && (m == "GET" || m == "POST") {
        let body: Value = if m == "POST" {
            serde_json::from_str(&body_text).unwrap_or(Value::Null)
        } else {
            Value::Null
        };
        // PHP's `$_REQUEST`: the query string wins, then the JSON/form body.
        let field = |name: &str| -> String {
            let q = ctx.qs(name);
            if !q.is_empty() {
                return q.trim().to_string();
            }
            body.get(name)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string()
        };
        let raw_codes = field("short_codes");
        let bot_id = field("bot_id");
        // ── batch mode (backend.php:19327-19331) ──────────────────────────
        if !raw_codes.is_empty() {
            let codes: Vec<String> = raw_codes
                .split(|c: char| c.is_whitespace() || c == ',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let result = warmup_resolve_batch(&codes, &bot_id).await;
            return json_response(
                200,
                &serde_json::json!({
                    "ok": 1,
                    "total": result.total,
                    "resolved": result.resolved,
                    // Queued at the origin is NOT failed: the origin's own paced
                    // drain is already fetching these. Reported apart so a caller
                    // never retries them inline and never suppresses them.
                    "queued": result.pending,
                    "queued_codes": result.pending_codes,
                    "failed": result.failed,
                    "failed_codes": result.failed_codes,
                    "results": result.results,
                    "bot_id": result.bot_id,
                }),
            );
        }
        let short_code = {
            let a = field("short_code");
            if a.is_empty() {
                field("sc")
            } else {
                a
            }
        };
        if short_code.is_empty() {
            return json_response(
                400,
                &serde_json::json!({"ok": 0, "message": "short_code or short_codes is required."}),
            );
        }
        let mut resolved = if bot_id.is_empty() {
            resolve_shortcode_concurrent(&short_code, &[]).await
        } else {
            resolve_shortcode(&short_code, &bot_id, false).await
        };
        let has_id = {
            let a = jstr(&resolved, "file_id_mt");
            let b = jstr(&resolved, "file_id");
            !a.is_empty() || !b.is_empty()
        };
        if !has_id {
            // The PHP answers the resolver's own body with HTTP 200
            // (backend.php:19341-19344), so the dashboard can show `message`
            // instead of failing on an HTTP error.
            if resolved.get("ok").is_none() {
                if let Some(obj) = resolved.as_object_mut() {
                    obj.insert("ok".into(), serde_json::json!(0));
                }
            }
            return json_response(200, &resolved);
        }
        // A batch-warmed cache row can lack file_size/title/mime. Enrich it with
        // a single resolve so the download payload is complete.
        let lacks = resolved.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0) == 0
            || (jstr(&resolved, "title").is_empty() && jstr(&resolved, "file_name").is_empty())
            || jstr(&resolved, "mime").is_empty();
        if lacks {
            let bot = {
                let a = jstr(&resolved, "bot_id");
                if a.is_empty() { bot_id.clone() } else { a }
            };
            if !bot.is_empty() {
                let rich = resolve_shortcode(&short_code, &bot, true).await;
                if !jstr(&rich, "file_id_mt").is_empty() || !jstr(&rich, "file_id").is_empty() {
                    if let (Some(dst), Some(src)) = (resolved.as_object_mut(), rich.as_object()) {
                        for (k, v) in src {
                            dst.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
        }
        let mut out = resolved;
        if let Some(obj) = out.as_object_mut() {
            obj.insert("ok".into(), serde_json::json!(1));
            obj.insert("short_code".into(), serde_json::json!(short_code));
        }
        return json_response(200, &out);
    }

    if p == "/api/warmup-resolve" && (m == "GET" || m == "POST") {
        let mut codes: Vec<String> = Vec::new();
        if m == "POST" {
            let input: Value = serde_json::from_str(&body_text).unwrap_or(Value::Null);
            let raw = input.get("short_codes").or_else(|| input.get("query"));
            match raw {
                Some(Value::Array(list)) => {
                    codes.extend(list.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()));
                }
                Some(Value::String(s)) => {
                    codes.extend(s.split(|c: char| c == ',' || c.is_whitespace()).map(|x| x.to_string()));
                }
                _ => {}
            }
        } else {
            codes.extend(
                ctx.qs("short_codes")
                    .split(|c: char| c == ',' || c.is_whitespace())
                    .map(|x| x.to_string()),
            );
        }
        let codes: Vec<String> = codes
            .into_iter()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let result = warmup_resolve_batch(&codes, &ctx.qs("bot_id")).await;
        return json_response(
            200,
            &serde_json::json!({
                "ok": 1,
                "total": result.total,
                "resolved": result.resolved,
                "queued": result.pending,
                "queued_codes": result.pending_codes,
                "failed": result.failed,
                "failed_codes": result.failed_codes,
                "results": result.results,
                "bot_id": result.bot_id,
            }),
        );
    }

    if p.starts_with("/api/") {
        return json_response(404, &serde_json::json!({"ok": 0, "message": "Unknown API endpoint"}));
    }

    /* ── root / configure / static / unknown ──────────────────────────────── */
    // Stremio's gear icon (manifest `behaviorHints.configurable: true`) opens
    // `<transportUrl>/configure`, and the transport URL IS the manifest — so the
    // request that arrives is `/…/manifest.json/configure`. Only the bare
    // `/configure` was routed, so in a client the gear was a 404 (measured:
    // `GET /<token>/manifest.json/configure` → 404, while `/configure` → 200).
    //
    // It lands on CATALOGS, not Addon: "configure this addon" means choosing which
    // rows and filters it publishes, which is the Catalogs panel. The Addon panel
    // only carries the address and the token, and nothing there changes what a
    // client sees. The bare `#configure` hash means the same thing in the
    // dashboard, so an old link and the gear reach the same panel.
    let configure_path = p.trim_end_matches('/');
    if configure_path == "/configure"
        || configure_path == "/config"
        || configure_path == "/manifest.json/configure"
        || configure_path == "/manifest.json/config"
    {
        return redirect("/#catalogs");
    }
    // Everything below `/api/*`, `/manifest.json`, `/catalog|meta|stream|
    // subtitles|eclipse` has already been answered above, so anything left is
    // the dashboard and its assets. They are served FROM DISK — this port does
    // not need the PHP runtime for the UI.
    if let Some(resp) = serve_static(&p) {
        return resp;
    }
    if p == "/" || p == "/index.html" {
        // `public/` is missing (a bare binary next to no checkout): say so
        // instead of returning a 404 that looks like a routing bug.
        let mut resp = text_response(
            200,
            "text/html; charset=utf-8",
            "<!doctype html><html><head><meta charset=\"utf-8\"><title>PencariMovie Server</title></head>\n\
<body style=\"background:#141414;color:#eee;font-family:system-ui,sans-serif;padding:40px;line-height:1.6\">\n\
<h1>PencariMovie Server</h1>\n\
<p><code>public/index.html</code> was not found next to the binary, so the dashboard cannot be served.\n\
The REST API and the Stremio addon routes are unaffected:</p>\n\
<ul><li><a style=\"color:#ff6b35\" href=\"/manifest.json\">/manifest.json</a></li>\n\
<li><a style=\"color:#ff6b35\" href=\"/api/version\">/api/version</a></li>\n\
<li><a style=\"color:#ff6b35\" href=\"/api/session\">/api/session</a></li>\n\
<li><a style=\"color:#ff6b35\" href=\"/api/bots\">/api/bots</a></li></ul></body></html>"
                .to_string(),
        );
        resp.headers_mut()
            .insert(header::CACHE_CONTROL, header::HeaderValue::from_static("no-store"));
        return resp;
    }

    not_found_json()
}

/// Serve the dashboard, with an SPA fallback to `index.html` for extension-less
/// paths (it is a hash-router app, so `/anything` must still boot it).
///
/// The assets are **embedded in this binary** ([`crate::assets`]), which is what
/// makes the server a single `pencarimovie-server` executable — no `public/`
/// folder to ship and no PHP runtime. `PM_ASSETS=disk` switches to reading `public/` from the
/// checkout instead, for frontend iteration without a rebuild.
///
/// A path containing `..` or a dot-segment is rejected: `storage/` (sessions,
/// `api_secret.key`, `auth.json`) sits beside `public/` in the source tree and
/// must stay private.
fn serve_static(p: &str) -> Option<Response> {
    let rel = p.trim_start_matches('/');
    if rel.split('/').any(|seg| seg == ".." || seg.starts_with('.')) {
        return None;
    }
    if crate::storage::env("PM_ASSETS").as_deref() != Some("disk") {
        if let Some(bytes) = crate::assets::get(rel) {
            // `/` resolves to index.html, so the content type must come from the
            // RESOLVED name — deriving it from an empty path produced
            // `application/octet-stream` and the browser DOWNLOADED the
            // dashboard instead of rendering it.
            let named = if rel.is_empty() { "index.html" } else { rel };
            let ctype = content_type_for(std::path::Path::new(named));
            return crate::respond::bytes_response(ctype, bytes::Bytes::from_static(bytes));
        }
        if rel.is_empty() || !rel.contains('.') {
            if let Some(index) = crate::assets::get("index.html") {
                return crate::respond::bytes_response(
                    "text/html; charset=utf-8",
                    bytes::Bytes::from_static(index),
                );
            }
        }
        if !crate::assets::is_empty() {
            return None;
        }
    }
    serve_static_from_disk(rel)
}

/// `PM_ASSETS=disk` (and any build where `public/` was absent at compile time).
fn serve_static_from_disk(rel: &str) -> Option<Response> {
    let public = crate::storage::public_dir()?;
    if rel.is_empty() {
        return crate::respond::file_response(&public.join("index.html"), "text/html; charset=utf-8");
    }
    let candidate = public.join(rel);
    if candidate.is_file() {
        let ctype = content_type_for(&candidate);
        return crate::respond::file_response(&candidate, ctype);
    }
    if !rel.contains('.') {
        return crate::respond::file_response(&public.join("index.html"), "text/html; charset=utf-8");
    }
    None
}

fn content_type_for(path: &std::path::Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => "text/html; charset=utf-8",
        "js" | "mjs" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "map" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        "woff" => "font/woff",
        "woff2" => "font/woff2",
        "ttf" => "font/ttf",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "txt" | "md" => "text/plain; charset=utf-8",
        "xml" => "application/xml",
        _ => "application/octet-stream",
    }
}

/* ── stream route ─────────────────────────────────────────────────────────── */

async fn handle_stream(state: &Arc<AppState>, ctx: &ReqCtx, item_type: &str, item_id: &str) -> Response {
    // `/stream/*` response cache (300 s), the equivalent of the PHP's
    // `stream_cache_*.json`. Checked BEFORE any origin/upstream work, so a warm
    // list costs one hash lookup. `finish_streams()` fills it.
    let skey = stream_cache_key(item_type, item_id, &ctx.base_url);
    if let Some(hit) = crate::cache::get("stream", &skey) {
        if let Some(list) = hit.get("streams").and_then(|s| s.as_array()) {
            log(
                "stremio stream served from cache",
                &serde_json::json!({"type": item_type, "id": item_id}),
            );
            // The cached body is the LIST only; the sponsor and the update notice
            // are re-fetched per answer, so a warm list can never serve a missing
            // sponsor or a stale "update available" card.
            let streams = decorate_streams(&ctx.base_url, list.clone()).await;
            return stremio_json(200, &serde_json::json!({"streams": streams}), "no-store");
        }
    }

    // NO BOT SESSION -> a TAPPABLE CARD, never an empty list.
    //
    // backend.php answers a stream ITEM here because Stremio/Nuvio render a
    // stream's `description` and never an HTTP error body; this port had no gate
    // at all, so with no session every title fell through to the origin search
    // and answered `{"streams":[]}` — indistinguishable from "this title has no
    // sources", and it never told the user a guest bot was being provisioned.
    //
    // Only the clock probe runs inline (one unauthenticated request, ~150ms: a
    // skewed clock breaks the MTProto handshake, so it must be diagnosed BEFORE
    // provisioning). The provisioner itself is spawned, exactly like
    // `fd_spawn_guest_provision()`, so this response is immediate.
    if !has_local_session("") {
        let base = ctx.base_url.trim_end_matches('/');
        let clock = measure_clock_offset().await;
        if clock.ok && clock.offset.abs() > 30 {
            log(
                "stream blocked: clock skew",
                &serde_json::json!({"offset_seconds": clock.offset}),
            );
            return single_card_streams(
                &ctx.base_url,
                serde_json::json!({
                    "name": "PencariMovie",
                    "description": format!(
                        "{}\nTap to open PencariMovie and re-check the clock, then retry.",
                        clock_skew_message(clock.offset)
                    ),
                    "externalUrl": format!("{}/#settings", base),
                }),
            )
            .await;
        }
        let already = is_guest_provision_in_progress();
        if !already {
            let st = state.clone();
            tokio::spawn(async move {
                let _ = auto_provision_guest(&st.telegram).await;
            });
        }
        log(
            "stream blocked: provisioning guest bot",
            &serde_json::json!({"already_running": already, "type": item_type, "id": item_id}),
        );
        return single_card_streams(
            &ctx.base_url,
            serde_json::json!({
                "name": "PencariMovie",
                "description": format!(
                    "{}\n⏳ {} a guest Telegram bot — this takes a few seconds. Reopen this title in a moment.",
                    if already {
                        "Telegram bot not connected yet."
                    } else {
                        "Telegram bot not connected."
                    },
                    if already { "Still provisioning" } else { "Provisioning" }
                ),
                "externalUrl": format!("{}/#settings", base),
            }),
        )
        .await;
    }

    let is_direct = item_id.starts_with("pm_file_") || item_id.starts_with("pm:file:");
    let code = item_id
        .strip_prefix("pm_file_")
        .or_else(|| item_id.strip_prefix("pm:file:"))
        .unwrap_or("");
    if is_direct && !code.is_empty() && code.chars().all(|c| c.is_ascii_alphanumeric()) {
        let active_bot = get_bot_id();
        // Keep the STATUS: a busy origin (503 "Server busy resolving file") is not
        // a file that vanished, and it must not render as a dead card.
        let (resolved, resolve_status) = if active_bot.is_empty() {
            (resolve_shortcode_concurrent(code, &[]).await, 0u16)
        } else {
            crate::resolve::resolve_shortcode_status(code, &active_bot, false).await
        };
        let file_id = {
            let a = jstr(&resolved, "file_id_mt");
            if a.is_empty() { jstr(&resolved, "file_id") } else { a }
        };
        if file_id.is_empty() {
            let msg = jstr(&resolved, "message");
            // The origin is busy or unreachable: answer 503 + `Retry-After` so the
            // client retries the same item, instead of the "File could not be
            // resolved" card that looks like the release is gone.
            if resolve_status >= 500 || resolve_status == 0 {
                log(
                    "stream resolve: origin unavailable",
                    &serde_json::json!({"short_code": code, "origin_status": resolve_status}),
                );
                let mut resp = json_response(
                    503,
                    &serde_json::json!({
                        "error": "File temporarily unavailable",
                        "message": "The origin is busy resolving this file. Retry in a few seconds.",
                    }),
                );
                resp.headers_mut().insert(
                    axum::http::header::RETRY_AFTER,
                    axum::http::HeaderValue::from_static("5"),
                );
                return resp;
            }
            return single_card_streams(
                &ctx.base_url,
                serde_json::json!({
                    "name": "PencariMovie",
                    "description": format!("File could not be resolved.\n{}", msg),
                }),
            )
            .await;
        }
        let size = resolved.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0);
        let title = {
            let t = jstr(&resolved, "title");
            if t.is_empty() {
                let f = jstr(&resolved, "file_name");
                if f.is_empty() { format!("{}.mp4", code) } else { f }
            } else {
                t
            }
        };
        let file_name = stream_filename(&title, &jstr(&resolved, "mime"));
        // NO `notWebReady` here either — see `stream_item()`: the flag made
        // Stremio (web AND Android) hide every Matroska row, and the PHP never
        // sent it on a playable URL. `behaviorHints.filename` still carries the
        // container, which is what a player should decide from.
        let mime = {
            let m = jstr(&resolved, "mime");
            let m = if m.is_empty() { jstr(&resolved, "file_type") } else { m };
            guess_video_mime(&file_name, &m)
        };
        let bot_for_payload = {
            let b = jstr(&resolved, "bot_id");
            if b.is_empty() { active_bot.clone() } else { b }
        };
        let payload = serde_json::json!({
            "file_id_mt": file_id,
            "file_size": size,
            "file_name": file_name,
            "mime": mime,
            "short_code": code,
            "bot_id": bot_for_payload,
        });
        let b64 = crate::fileid::base64url_encode(payload.to_string().as_bytes());
        let url = build_stream_url(&ctx.base_url, &b64, &file_name);
        // A direct `pm_file_` item keeps its file_id in the payload (no
        // resolution at download time) and still gets the sponsored message on
        // top, exactly like every other stream list.
        return finish_streams(
            ctx,
            item_type,
            &title,
            item_id,
            vec![{
                let _ = &url;
                serde_json::json!({
                    "name": "PencariMovie",
                    "description": format!("{}\n💾 {} • ⚡ Telegram", file_name, format_bytes(size)),
                    "url": url,
                    "behaviorHints": {
                        "filename": file_name,
                        "videoSize": size,
                        "bingeGroup": "pencarimovie"
                    },
                })
            }],
            false,
            1,
        )
        .await;
    }

    // `pm:post:<id>[:S:E]` — the post's own file set through the matcher.
    if let Some(rest) = item_id.strip_prefix("pm:post:") {
        return handle_post_stream(ctx, item_type, item_id, rest, None).await;
    }

    // Any other id (`tt…`, `tmdb:…`, `kitsu:…`, `mal:…`): resolve the identity,
    // search the origin by title, then let the matcher decide which of the
    // returned files belong to this title. Upstreams that declare `stream` are
    // asked CONCURRENTLY — the task is spawned here and awaited at the end, so
    // they never add their latency to the local list (backend.php:16606-16739
    // does the same with `\Amp\async`).
    if parse_external_id(item_id).is_some() {
        let upstream = spawn_upstream_streams(item_type, item_id);
        if let Some(resp) = handle_external_stream(ctx, item_type, item_id, upstream).await {
            return resp;
        }
    }

    // A shape the local matcher cannot use (an unknown `foo:` prefix, anything
    // odd): an upstream declaring `stream` may still know it, so ask before
    // giving up. A `pm:` id never reaches here (handled above) and never fans
    // out. (`tvdb:` / `anilist:` / `tvmaze:` are parsed above, so they take the
    // local path in `handle_external_stream()`.)
    let upstream_items = await_upstream(spawn_upstream_streams(item_type, item_id), true).await;
    if !upstream_items.is_empty() {
        return finish_streams(ctx, item_type, item_id, item_id, upstream_items, false, 0).await;
    }
    crate::diagnostics::record(
        "stream_incident",
        "stream list requested for an unsupported item shape",
        "",
        "",
        "NO_STREAMS",
        "",
        item_id,
        serde_json::json!({"item_type": item_type, "runtime": "pencarimovie-server"}),
    );
    // An empty array is not a stream list: the user gets the tappable
    // "No Streams Found" card, and — like every other answer — the sponsor above
    // it. It used to be a bare `{"streams":[]}`, the one shape in the whole route
    // with no sponsored item at all.
    finish_streams(ctx, item_type, item_id, item_id, Vec::new(), false, 0).await
}

/// Upstream stream-list task (`fd_http_get_many_amp` over the upstream
/// `/stream/:type/:id.json` URLs, backend.php:16706).
///
/// Spawned BEFORE the local resolution so the two overlap. A `pm:` / `pm_` id
/// never fans out — it is ours and no upstream can answer it
/// (`$isLocalPmId` in the PHP). `None` also means "nothing configured", which
/// keeps the zero-upstream path allocation-free.
fn spawn_upstream_streams(
    item_type: &str,
    item_id: &str,
) -> Option<tokio::task::JoinHandle<Vec<Value>>> {
    if item_id.starts_with("pm:") || item_id.starts_with("pm_") {
        return None;
    }
    let settings = load_catalog_settings_merged();
    if upstream::configured(&settings, true).is_empty() {
        return None;
    }
    let ty = item_type.to_string();
    let id = item_id.to_string();
    Some(tokio::spawn(async move {
        upstream::streams(&settings, &ty, &id).await
    }))
}

/// Await an upstream task with the PHP's non-blocking bound
/// (backend.php:18121): 0.8 s when local streams already exist, 2.5 s when the
/// list is still empty. Losing the race costs nothing — the local list is
/// delivered without the external items.
async fn await_upstream(
    handle: Option<tokio::task::JoinHandle<Vec<Value>>>,
    local_empty: bool,
) -> Vec<Value> {
    let Some(handle) = handle else {
        return Vec::new();
    };
    let wait_ms: u64 = if local_empty { 2500 } else { 800 };
    match tokio::time::timeout(std::time::Duration::from_millis(wait_ms), handle).await {
        Ok(Ok(items)) => items,
        Ok(Err(e)) => {
            log(
                "upstream stream task failed",
                &serde_json::json!({"error": e.to_string()}),
            );
            Vec::new()
        }
        Err(_) => {
            log(
                "upstream stream merge timed out",
                &serde_json::json!({"wait_ms": wait_ms}),
            );
            Vec::new()
        }
    }
}

/// `fd_stream_keyword_from_post_title()` (backend.php:9805) — the CLEANED
/// search keyword for a post title.
///
/// `"Positively Yours 2026 TvSeries"` → `"Positively Yours"`, `"X • Movie"` →
/// `"X"`. The origin's per-episode search is keyed on this keyword; asking it
/// with the raw decorated title returns only the post's own rows (or nothing),
/// which is exactly the "only one source per episode" symptom.
fn stream_keyword_from_post_title(title: &str) -> String {
    let mut k = crate::matcher::decode_html_entities(title);
    k = k
        .chars()
        .map(|c| if (c as u32) < 0x20 { ' ' } else { c })
        .collect();
    if let Some(i) = k.find(['•', '·']) {
        k.truncate(i);
    }
    let mut k = k.trim().to_string();
    for pattern in [
        r"\s*\(\d{4}\)\s*$",                 // trailing "(2024)"
        r"\s+\d{4}\s*$",                     // trailing " 2024"
        r"\b(?:tvseries|tv\s*series|movie)\b", // type words, anywhere
    ] {
        if let Ok(re) = fancy_regex::Regex::new(&format!("(?i){pattern}")) {
            k = re.replace_all(&k, "").to_string();
        }
    }
    let k = k.split_whitespace().collect::<Vec<_>>().join(" ");
    k.trim_matches(|c: char| " \t\r\n.!?:;,-".contains(c))
        .to_string()
}

/// `pm:post:<id>[:S:E]` — Fast Path 1/2a, the same shape as the PHP.
///
/// A series ROOT (`pm:post:<id>` with no `:S:E`) is **S01E01** in the PHP:
/// `fd_fetch_episode_stream_files($postId, $targetSeason ?? 1, $targetEpisode ?? 1, 250)`
/// (backend.php:16828). Matching the root with season/episode 0 instead read the
/// post's own files with no episode filter and listed 5 E01-ish rows where the
/// PHP lists the whole of episode 1.
async fn handle_post_stream(
    ctx: &ReqCtx,
    item_type: &str,
    item_id: &str,
    rest: &str,
    upstream: Option<tokio::task::JoinHandle<Vec<Value>>>,
) -> Response {
    let mut it = rest.split(':');
    let post_id = it.next().unwrap_or("").trim().to_string();
    let season: i32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let episode: i32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    if post_id.is_empty() || !post_id.chars().all(|c| c.is_ascii_digit()) {
        return finish_streams(ctx, item_type, item_id, item_id, Vec::new(), false, 0).await;
    }
    let is_series = item_type.contains("series") || season > 0 || episode > 0;
    let (eff_season, eff_episode) = effective_season_episode(is_series, season, episode);

    // The post record and media-id index are independent origin lookups. Start
    // both together so a cold click pays the slower request once, not twice.
    let (post, row) = tokio::join!(
        crate::wp_data::get_post(&post_id),
        crate::wp_data::lookup_id("post", &post_id),
    );
    let mut title = crate::wp_data::post_title(&post);
    let mut year = crate::wp_data::post_year(&post);
    // Step 2b: the `media_ids_idx` ROW is the fallback when the id is not a real
    // WordPress post (pseudo ids `9000000000+`, e.g. `pm:post:9001801064`). It
    // also carries the ALTERNATE title — which is exactly what makes
    // "Khurafat: Perjanjian Syaitan" playable while the origin indexes it as
    // "Superstition" (backend.php's AKA chain).
    if title.is_empty() {
        title = jstr(&row, "title");
    }
    if year.is_empty() {
        year = jstr(&row, "year");
        if year.is_empty() {
            year = jstr(&row, "release_year");
        }
    }
    let mut aka = jstr(&row, "aka");
    if title.is_empty() && !aka.is_empty() {
        title = std::mem::take(&mut aka);
    }
    // The id the upstream episode-name lookup asks about. Only the `media_ids_idx`
    // row knows it for a post id; an empty value simply means "no evidence"
    // (the guard fails OPEN), exactly like the PHP's `$episodeMediaId`.
    let imdb_id = jstr(&row, "imdb_id");
    if title.is_empty() {
        log(
            "stream: post has no title",
            &serde_json::json!({"post_id": post_id, "item_id": item_id}),
        );
        crate::diagnostics::record(
            "stream_incident",
            "post files could not be identified",
            "",
            "",
            "NO_STREAMS",
            "",
            item_id,
            serde_json::json!({"item_type": item_type, "post_id": post_id, "runtime": "pencarimovie-server"}),
        );
        // No title means the matcher has nothing to match against — answer with
        // the tappable "No Streams Found" card instead of an empty array. An
        // upstream item, when there is one, replaces that card: the PHP merges
        // upstreams BEFORE deciding `$hasPlayableStreams`.
        let merged = await_upstream(upstream, true).await;
        return finish_streams(ctx, item_type, item_id, item_id, merged, false, 0).await;
    }

    let mut files = if is_series {
        // Fast Path 1 — the whole `fd_fetch_episode_stream_files()` ladder, which
        // also owns the post's own file set (its step 0).
        let resolved = resolve_episode_files(
            &post_id, &title, &year, &imdb_id, eff_season, eff_episode,
        )
        .await;
        title = resolved.title;
        year = resolved.year;
        resolved.files
    } else {
        // Fast Path 2a — the MOVIE's own file set, paged (never a single `limit=200`
        // call: the origin answers newest-first and truncates a long list).
        crate::wp_data::fetch_post_files_paged(&post_id, 200, 600, 3).await
    };
    if files.is_empty() && !aka.is_empty() {
        // Nothing at all yet: the alternate title may be the one the origin uses.
        let kind0 = if is_series { "series" } else { "movie" };
        let (_, _, f2) = crate::wp_data::stream_files_by_title(kind0, &aka, &year, item_id).await;
        files = f2;
    }
    build_streams_from_files(
        ctx,
        item_type,
        item_id,
        &title,
        &year,
        &aka,
        &imdb_id,
        files,
        eff_season,
        eff_episode,
        upstream,
    )
    .await
}

/// Season/episode the matcher runs with.
///
/// A series ROOT resolves to **S01E01** — the PHP passes
/// `$targetSeason ?? 1` / `$targetEpisode ?? 1`. A MOVIE keeps 0/0 so
/// [`build_streams_from_files()`] takes the movie branch.
fn effective_season_episode(is_series: bool, season: i32, episode: i32) -> (i32, i32) {
    if is_series {
        (season.max(1), episode.max(1))
    } else {
        (0, 0)
    }
}

/// Result of [`resolve_episode_files()`] — `title`/`year` may have been adopted
/// from the origin or derived from the post's own file set, and the caller (and
/// the matcher) must use the resolved values.
struct EpisodeResolution {
    title: String,
    year: String,
    files: Vec<crate::wp_data::WpFile>,
}

/// The `$add()` closure of `fd_fetch_episode_stream_files()` (backend.php:10060):
/// dedupe by `short_code`, then the episode filter, then the strict series
/// matcher. A file must clear all three to join the list.
#[allow(clippy::too_many_arguments)]
fn add_episode_files(
    all: &mut Vec<crate::wp_data::WpFile>,
    seen: &mut std::collections::HashSet<String>,
    files: &[crate::wp_data::WpFile],
    title: &str,
    keyword: &str,
    year: &str,
    season: i32,
    episode: i32,
    names: &[String],
    max: usize,
) {
    let series_title = if title.trim().is_empty() { keyword } else { title };
    for f in files {
        if all.len() >= max {
            return;
        }
        if f.short_code.is_empty() || seen.contains(&f.short_code) {
            continue;
        }
        let parsed = crate::matcher::classify_season_episode(
            &f.title,
            f.season_num,
            f.episode_num,
            &f.caption,
        );
        if !crate::matcher::file_matches_episode(parsed, season, episode) {
            continue;
        }
        if !crate::matcher::series_file_matches_title(
            &f.title,
            series_title,
            season,
            episode,
            year,
            &f.caption,
            names,
        ) {
            continue;
        }
        seen.insert(f.short_code.clone());
        all.push(f.clone());
    }
}

/// The per-episode token list of `fd_fetch_episode_stream_files()` step 3
/// (backend.php:10301-10312), RAREST FIRST so the `max` cap is spent on the
/// conventions that return few results rather than on `S01E01`.
fn episode_tokens(season: i32, episode: i32) -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    if episode <= 0 {
        return tokens;
    }
    if episode < 10 {
        // Rare, unpadded forms first (EP1, E1) — few uploaders use these.
        tokens.push(format!("EP{}", episode));
        tokens.push(format!("E{}", episode));
    }
    // Padded EP form (EP01) — also uncommon.
    tokens.push(format!("EP{:02}", episode));
    // Common forms last (E01, S01E01) — these return the most results.
    tokens.push(format!("E{:02}", episode));
    tokens.push(format!("S{:02}E{:02}", season.max(1), episode));
    tokens
}

/// The ordered `search_files` probe list of step 3 (backend.php:10331-10352).
///
/// Order matters: `$add()` stops at the cap, so the targeted per-token queries
/// run BEFORE the broad OR-group, which would otherwise fill the page with
/// `S01E01` variants and hide the single `EP1` release. When the year is known
/// each token is also probed WITH the year, because Manticore ANDs every term and
/// the year sits between the title and the episode token in names like
/// `Agent.Kim.Reactivated.2026.EP01.MalaySub.mp4`.
fn episode_probe_queries(keyword: &str, year: &str, season: i32, episode: i32) -> Vec<String> {
    let keyword = keyword.trim();
    if keyword.is_empty() {
        return Vec::new();
    }
    let tokens = episode_tokens(season, episode);
    if tokens.is_empty() {
        return Vec::new();
    }
    let group = format!("({})", tokens.join(" | "));
    let mut out: Vec<String> = Vec::new();
    for tok in &tokens {
        out.push(format!("{} {}", keyword, tok));
        if !year.is_empty() {
            out.push(format!("{} {} {}", keyword, year, tok));
        }
    }
    if !year.is_empty() {
        out.push(format!("{} {} {}", keyword, year, group));
    }
    out.push(format!("{} {}", keyword, group));
    out.push(format!("{} {}", group, keyword));
    // Possessive variants ("Princess's" vs "Princesss" vs "Princess") only add
    // probes for the OR-group, and only when the hint actually has an apostrophe.
    for variant in crate::matcher::stream_keyword_variants(keyword)
        .into_iter()
        .skip(1)
    {
        out.push(format!("{} {}", variant, group));
    }
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    out.retain(|q| seen.insert(q.to_lowercase()));
    out
}

/// Port of `fd_fetch_episode_stream_files()` (backend.php:9987-10388): every
/// playable file of ONE series episode.
///
/// Steps, in the PHP's order:
///
/// 0. the POST'S OWN file set — one fetch serves every episode (the per-episode
///    origin call below is a distinct URL, so without this every episode started
///    cold). This is also where the YEAR is derived from a clear majority of the
///    post's files, which is what keeps a 1999 anime from listing the 2023
///    live-action's episodes (the ±1-year guard is inert without a year).
/// 1. the PER-EPISODE index search — NOT gated on step 0 being empty: the post's
///    own files are a SUBSET, so another uploader's MalaySub/quality variant of
///    the SAME episode only appears here. It also supplies `resolved_title` /
///    `resolved_year` when nothing else knows the title.
/// 2. `get_post` when the title is still unknown, so the ladder below has a
///    keyword to search with.
/// 3. the parallel `search_files` probe ladder (rare token conventions first),
///    run only when the per-episode search did NOT already confirm an answer —
///    the origin answers `ok` even with zero items, and the PHP stops there
///    rather than burning seconds on pointless scans (backend.php:10265).
async fn resolve_episode_files(
    post_id: &str,
    title: &str,
    year: &str,
    imdb_id: &str,
    season: i32,
    episode: i32,
) -> EpisodeResolution {
    resolve_episode_files_with(post_id, "", Vec::new(), title, year, imdb_id, season, episode).await
}

/// The same ladder for a series reached through an EXTERNAL id (`tt…`, `tmdb:…`).
///
/// The PHP runs
/// `fd_fetch_episode_stream_files(0, $targetSeason, $targetEpisode, 250,
/// ['title' => …, 'year' => …, 'id' => $itemId, 'imdb_id' => …, 'episode_names' => …])`
/// (backend.php:17093): postId **0**, so step 0 has no post to read, the origin's
/// `id` parameter is the EXTERNAL id, and every other step runs off the resolved
/// title. Skipping it left the `tt…` route with only the title search's first
/// page — measured `tt21344706` S01E01 → **20** files, while the same episode
/// through `pm:post:8798798206` → **86**.
///
/// `seed` is whatever the caller already resolved (the title search). It goes
/// through the same `$add()` filter below, so it cannot smuggle in a foreign file.
#[allow(clippy::too_many_arguments)]
async fn resolve_external_episode_files(
    external_id: &str,
    title: &str,
    year: &str,
    imdb_id: &str,
    season: i32,
    episode: i32,
    seed: Vec<crate::wp_data::WpFile>,
) -> EpisodeResolution {
    resolve_episode_files_with("", external_id, seed, title, year, imdb_id, season, episode).await
}

#[allow(clippy::too_many_arguments)]
async fn resolve_episode_files_with(
    post_id: &str,
    external_id: &str,
    seed: Vec<crate::wp_data::WpFile>,
    title: &str,
    year: &str,
    imdb_id: &str,
    season: i32,
    episode: i32,
) -> EpisodeResolution {
    const MAX_FILES: usize = 250;
    /// `$thinThreshold = max(3, ceil($maxFiles / 4))` — backend.php:10290.
    const THIN_MIN: usize = 3;

    // A post id is only usable when it is all digits. The external route arrives
    // with none, exactly like the PHP's `$postId = 0`.
    let has_post = !post_id.is_empty() && post_id.chars().all(|c| c.is_ascii_digit());

    let mut title = title.to_string();
    let mut year = year.to_string();
    let mut keyword = stream_keyword_from_post_title(&title);
    let mut all: Vec<crate::wp_data::WpFile> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Verified episode names, resolved ONCE for every retrieval path below —
    // `$episodeNames` in the PHP (backend.php:10052-10055). They let the matcher
    // reject a file that names ANOTHER show's episode after this episode's marker
    // (the sibling-work case). An empty list means "no evidence", and the guard
    // then fails OPEN, so this can never hide a stream on its own.
    // These three reads are independent once the post/title/id are known. Run
    // them together so source discovery waits for the slowest origin, not the
    // sum of metadata, post-file and episode-index latencies.
    let episode_names_future = async {
        if imdb_id.trim().is_empty() {
            Vec::new()
        } else {
            let settings = load_catalog_settings_merged();
            upstream::episode_names(&settings, imdb_id, season, episode, &title, &year).await
        }
    };
    let post_files_future = async {
        if has_post {
            crate::wp_data::fetch_post_files_paged(post_id, 200, 600, 3).await
        } else {
            Vec::new()
        }
    };
    let episode_index_id = if has_post {
        format!("post:{}", post_id)
    } else {
        external_id.to_string()
    };
    let episode_index_future = async {
        if has_post || !external_id.is_empty() {
            Some(crate::wp_data::stream_files_episode(
                "series",
                &keyword,
                &episode_index_id,
                season,
                episode,
                MAX_FILES,
            ).await)
        } else {
            None
        }
    };
    let (episode_names, post_files, more) = tokio::join!(
        episode_names_future,
        post_files_future,
        episode_index_future,
    );

    // ── 0a. what the caller already resolved ─────────────────────────────────
    // On the external route this is the title search. It runs through `$add()`
    // like every other source, so the strict matcher still decides.
    if !seed.is_empty() {
        add_episode_files(
            &mut all,
            &mut seen,
            &seed,
            &title,
            &keyword,
            &year,
            season,
            episode,
            &episode_names,
            MAX_FILES,
        );
        log(
            "stream: episode files from the title search",
            &serde_json::json!({
                "external_id": external_id, "season": season, "episode": episode,
                "searched": seed.len(), "matched": all.len(),
            }),
        );
    }

    // ── 0. the post's own file set ───────────────────────────────────────────
    if !post_files.is_empty() {
        if year.is_empty() {
            if let Some(derived) = crate::wp_data::derive_year_from_files(&post_files) {
                log(
                    "stream: series year derived from post set",
                    &serde_json::json!({
                        "post_id": post_id, "year": derived,
                        "tagged": post_files.len(), "total": post_files.len(),
                    }),
                );
                year = derived;
            }
        }
        add_episode_files(
            &mut all,
            &mut seen,
            &post_files,
            &title,
            &keyword,
            &year,
            season,
            episode,
            &episode_names,
            MAX_FILES,
        );
        log(
            "stream: episode files from cached post set",
            &serde_json::json!({
                "post_id": post_id, "season": season, "episode": episode,
                "post_files": post_files.len(), "matched": all.len(),
            }),
        );
    }

    // ── 1. the per-episode index search ──────────────────────────────────────
    let mut fast_path_done = false;
    let more = more.unwrap_or_default();
    if all.len() < MAX_FILES && (has_post || !external_id.is_empty()) {
        if more.ok {
            fast_path_done = true;
            if title.is_empty() && !more.resolved_title.is_empty() {
                title = more.resolved_title;
                keyword = stream_keyword_from_post_title(&title);
                if !more.resolved_year.is_empty() {
                    year = more.resolved_year;
                } else if year.is_empty() {
                    year = first_year_of(&title);
                }
                log(
                    "stream: episode title adopted from the origin",
                    &serde_json::json!({
                        "post_id": post_id, "title": title, "year": year,
                    }),
                );
            }
        }
        let before = all.len();
        add_episode_files(
            &mut all,
            &mut seen,
            &more.files,
            &title,
            &keyword,
            &year,
            season,
            episode,
            &episode_names,
            MAX_FILES,
        );
        if all.len() > before {
            log(
                "stream: episode index search merged",
                &serde_json::json!({
                    "post_id": post_id, "season": season, "episode": episode,
                    "keyword": keyword, "before": before, "after": all.len(),
                }),
            );
        }
    }

    // ── 2. `get_post` when the title is still unknown ────────────────────────
    let mut fetched_fallback_post = false;
    if title.is_empty() && has_post {
        // Cache-first, so this is normally free: the route already asked for it.
        let p = crate::wp_data::get_post(post_id).await;
        let t = crate::wp_data::post_title(&p);
        if !t.is_empty() {
            title = t;
            keyword = stream_keyword_from_post_title(&title);
            if year.is_empty() {
                year = crate::wp_data::post_year(&p);
            }
            fetched_fallback_post = true;
        }
    }

    // ── 3. the probe ladder ──────────────────────────────────────────────────
    //
    // Skipped when the per-episode search CONFIRMED an answer (`ok` — even with
    // zero items) and no fallback post was needed: at that point the origin has
    // already been asked about this exact episode, and the PHP returns rather
    // than running up to 11 more searches (measured: +4-5 s for nothing).
    let thin = THIN_MIN.max((MAX_FILES as f64 / 4.0).ceil() as usize);
    let skip_ladder = fast_path_done && !keyword.is_empty() && !fetched_fallback_post;
    if all.len() < thin && !keyword.is_empty() && !skip_ladder {
        let queries = episode_probe_queries(&keyword, &year, season, episode);
        if !queries.is_empty() {
            let results = futures_util::future::join_all(queries.iter().map(|q| {
                let q = q.clone();
                // `search_files_probe`, not `search_files`: speculative work gets a
                // short budget, so a sick origin cannot hold the list open. Measured:
                // 13 probes, all timing out, only made the same empty list arrive
                // slower.
                async move { crate::wp_data::search_files_probe(&q, 50).await }
            }))
            .await;
            for (i, _q) in queries.iter().enumerate() {
                if all.len() >= MAX_FILES {
                    break;
                }
                add_episode_files(
                    &mut all,
                    &mut seen,
                    &results[i],
                    &title,
                    &keyword,
                    &year,
                    season,
                    episode,
                    &episode_names,
                    MAX_FILES,
                );
            }
            log(
                "stream: episode fallback probes",
                &serde_json::json!({
                    "post_id": post_id, "season": season, "episode": episode,
                    "queries": queries.len(), "matched": all.len(),
                }),
            );
        }
    }

    EpisodeResolution {
        title,
        year,
        files: all,
    }
}

/// The first 4-digit year in a string (`"2023–"`, `"2019-03-01"`, `"2019"`).
fn first_year_of(s: &str) -> String {
    let d: Vec<char> = s.chars().collect();
    if d.len() < 4 {
        return String::new();
    }
    for i in 0..=(d.len() - 4) {
        if d[i..i + 4].iter().all(|c| c.is_ascii_digit()) {
            return d[i..i + 4].iter().collect();
        }
    }
    String::new()
}

/// `tt…` / `tmdb:…` / `kitsu:…` / `mal:…` / `tvdb:…` / `anilist:…` /
/// `tvmaze:…` — identity lookup, origin title search, matcher. Returns `None`
/// when the id shape is not one of ours.
async fn handle_external_stream(
    ctx: &ReqCtx,
    item_type: &str,
    item_id: &str,
    upstream: Option<tokio::task::JoinHandle<Vec<Value>>>,
) -> Option<Response> {
    let (prefix, value, season, episode) = parse_external_id(item_id)?;
    let row = crate::wp_data::lookup_id(prefix, &value).await;
    let mut title = jstr(&row, "title");
    let mut year = jstr(&row, "year");
    if year.is_empty() {
        year = jstr(&row, "release_year");
    }
    let aka = jstr(&row, "aka");
    // The episode-name lookup asks about the IMDb id when the row knows one —
    // the PHP's `$imdbId !== '' ? $imdbId : $itemId` (backend.php:17084). A
    // `tvdb:` / `tvmaze:` id is usually NOT served by a meta provider (measured:
    // AIOMetadata answers `anilist:21` but not `tvdb:81797`), while the row's
    // `imdb_id` always is, so using the item id alone left the guard inert for
    // exactly the prefixes this port just learned to resolve.
    let episode_media_id = {
        let imdb = jstr(&row, "imdb_id");
        if imdb.is_empty() {
            item_id.to_string()
        } else {
            imdb
        }
    };

    // The by-id page, when it did NOT cover the requested episode: kept as a SEED
    // for the episode ladder below instead of being returned on its own.
    let mut by_id_seed: Vec<crate::wp_data::WpFile> = Vec::new();

    // A row may already know which catalog post owns the id — then the post path
    // is exact and the title search is unnecessary.
    if title.is_empty() {
        if let Some(pid) = row
            .get("post_id")
            .and_then(|v| v.as_u64().map(|n| n.to_string()).or_else(|| v.as_str().map(|s| s.to_string())))
        {
            let rest = if season > 0 {
                format!("{}:{}:{}", pid, season, episode)
            } else {
                pid
            };
            return Some(handle_post_stream(ctx, item_type, item_id, &rest, upstream).await);
        }
        // (1) ID-ONLY ORIGIN PROBE. `/stream-files` resolves the identity from
        // `id` itself and answers `resolved_title`, so an id that is not in the
        // local `media_ids_idx` can still be recovered in ONE origin call.
        //
        // Two measured rules decide whether to spend that call (it is on the
        // critical path of every episode, before any file can be listed):
        //
        // * the origin resolves `tt…` / `tmdb:…` by id, and answers
        //   `{"ok":0,"error":"title or id required"}` — an HTTP 400 — for a
        //   `mal:`/`kitsu:`/`anilist:`/`tvdb:`/`tvmaze:` id, so for those the
        //   call can only ever return zero files;
        // * the id must be BARE. `id=tt0944947` answers 499 files, while the
        //   item id carries the `:S:E` suffix and `id=mal:54492:1:12` was
        //   rejected outright — the season/episode is not part of an origin id.
        let kind = if item_type == "series" || season > 0 { "series" } else { "movie" };
        let (by_id_title, by_id_year, by_id_files) = if origin_resolves_by_id(prefix) {
            let bare_id = origin_probe_id(prefix, &value);
            crate::wp_data::stream_files_by_title(kind, "", "", &bare_id).await
        } else {
            (String::new(), String::new(), Vec::new())
        };
        if !by_id_files.is_empty() {
            let t = if by_id_title.is_empty() {
                item_id.to_string()
            } else {
                by_id_title
            };
            // The origin resolved the year too (PHP backend.php:17212-17217): it is
            // the only place it exists for an id that is not in `media_ids_idx`, and
            // the ±1-year guard in the matcher is inert without it.
            if year.is_empty() && !by_id_year.is_empty() {
                year = by_id_year.clone();
            }
            log(
                "stream: external id resolved by origin id search",
                &serde_json::json!({"item_id": item_id, "title": &t, "files": by_id_files.len()}),
            );
            // A SERIES' by-id page is ONE page of the whole show, not the episode.
            // Measured on `tt35324004` ("Tigers Sniff the Rose") on a phone running
            // 1.0.92: the probe answered 5 rows, none of them the requested
            // episode, and returning here sent those 5 straight to the matcher —
            // E1 answered "No Streams Found" while the show's own post carried 3
            // playable files. Only short-circuit when the page really covers the
            // episode; otherwise keep it as a SEED and let the ladder run.
            if by_id_page_covers_episode(kind, &by_id_files, season, episode) {
                return Some(
                    build_streams_from_files(
                        ctx,
                        item_type,
                        item_id,
                        &t,
                        &year,
                        &aka,
                        &episode_media_id,
                        by_id_files,
                        season,
                        episode,
                        upstream,
                    )
                    .await,
                );
            }
            log(
                "stream: by-id page does not cover the episode, running the ladder",
                &serde_json::json!({
                    "item_id": item_id, "title": &t, "files": by_id_files.len(),
                    "season": season, "episode": episode,
                }),
            );
            title = t;
            by_id_seed = by_id_files;
        }

        // (2) UPSTREAM META. `backend.php`'s `fd_resolve_external_media_metadata()`
        // asks the meta-capable addons (AIOMetadata) for `/meta/<type>/<id>`;
        // without this an unindexed id had NO title at all, so the matcher had
        // nothing to search and the item answered "No Streams Found". Measured
        // on a real phone: tt6751668, tt21952924, tt34809853, tt34991493.
        let settings = load_catalog_settings_merged();
        if let Some(body) = upstream::forward_meta(&settings, item_type, item_id).await {
            let meta = body.get("meta").cloned().unwrap_or(Value::Null);
            let name = jstr(&meta, "name");
            if !name.is_empty() {
                title = name;
                if year.is_empty() {
                    let rel = {
                        let r = jstr(&meta, "releaseInfo");
                        if r.is_empty() { jstr(&meta, "year") } else { r }
                    };
                    year = first_year_of(&rel);
                }
                log(
                    "stream: external id resolved from upstream meta",
                    &serde_json::json!({"item_id": item_id, "title": title, "year": year}),
                );
            }
        }
    }

    if title.is_empty() {
        log(
            "stream: external id resolved without a title",
            &serde_json::json!({"item_id": item_id}),
        );
        crate::diagnostics::record(
            "stream_incident",
            "external id resolved without a title",
            "",
            "",
            "NO_STREAMS",
            "",
            item_id,
            serde_json::json!({"item_type": item_type, "runtime": "pencarimovie-server"}),
        );
        let merged = await_upstream(upstream, true).await;
        return Some(finish_streams(ctx, item_type, item_id, item_id, merged, false, 0).await);
    }

    let kind = if item_type == "series" || season > 0 { "series" } else { "movie" };
    let (resolved, resolved_year, mut files) =
        crate::wp_data::stream_files_by_title(kind, &title, &year, item_id).await;
    if year.is_empty() && !resolved_year.is_empty() {
        year = resolved_year;
    }
    // AKA fallback: "Superstition" finds nothing, "Khurafat" finds the files.
    if files.is_empty() && !aka.is_empty() {
        let (r2, _, f2) = crate::wp_data::stream_files_by_title(kind, &aka, &year, item_id).await;
        if !f2.is_empty() {
            files = f2;
            title = if r2.is_empty() { aka.clone() } else { r2 };
        }
    }
    // Clean keyword fallback: if title with symbols/decorations returned no files,
    // retry with cleaned title (e.g. without punctuation / trailing type tags).
    if files.is_empty() {
        let clean = stream_keyword_from_post_title(&title);
        if !clean.is_empty() && clean != title {
            let (r3, _, f3) = crate::wp_data::stream_files_by_title(kind, &clean, &year, item_id).await;
            if !f3.is_empty() {
                files = f3;
                if !r3.is_empty() {
                    title = r3;
                }
            }
        }
    }
    // The origin answers `resolved_title` with the SEARCH KEYWORD, not the real
    // title (it drops a trailing type word: "The Super Mario Galaxy"), so the
    // matcher is given both names — the type-word rule is what keeps that case
    // from collapsing to a single stream.
    if !resolved.is_empty() && resolved != title {
        log(
            "stream: origin resolved title differs",
            &serde_json::json!({"item_id": item_id, "title": title, "resolved": resolved}),
        );
        title = resolved;
    }

    // The by-id page joins the ladder as an extra seed: it is already resolved,
    // already paid for, and the `$add()` filter below decides what it may
    // contribute.
    if !by_id_seed.is_empty() {
        files.append(&mut by_id_seed);
    }

    // A SERIES must run the FULL `fd_fetch_episode_stream_files()` ladder, not
    // just the title search. The title search answers a single page; the
    // per-episode index search and the probe ladder are what surface the
    // episode's other releases (another group's copy, other qualities). The PHP
    // enters that ladder with postId 0 on this branch (backend.php:17093), which
    // is exactly what `resolve_external_episode_files()` is for.
    let (eff_season, eff_episode) = effective_season_episode(kind == "series", season, episode);
    if kind == "series" {
        let r = resolve_external_episode_files(
            item_id,
            &title,
            &year,
            &episode_media_id,
            eff_season,
            eff_episode,
            files,
        )
        .await;
        title = r.title;
        year = r.year;
        files = r.files;
    }

    Some(
        build_streams_from_files(
            ctx,
            item_type,
            item_id,
            &title,
            &year,
            &aka,
            &episode_media_id,
            files,
            eff_season,
            eff_episode,
            upstream,
        )
        .await,
    )
}

/// Does the origin's by-id page actually carry the requested episode?
///
/// A MOVIE's page is the film, so any row covers it. A SERIES' page is one slice
/// of a whole show — `tt35324004` answered 5 rows and none of them was E1 — and
/// returning those 5 on their own is what made an episode with 3 playable files
/// answer "No Streams Found". Only a row that parses to the requested
/// season/episode lets the fast path short-circuit; otherwise the caller keeps
/// the page as a seed and runs the episode ladder.
///
/// An unknown episode (`0`, a series ROOT) is covered by any row, because there
/// is no episode to contradict.
fn by_id_page_covers_episode(
    kind: &str,
    files: &[crate::wp_data::WpFile],
    season: i32,
    episode: i32,
) -> bool {
    if kind != "series" || episode <= 0 {
        return true;
    }
    files.iter().any(|f| {
        let (s, e, _) = crate::matcher::classify_season_episode(
            &f.title,
            f.season_num,
            f.episode_num,
            &f.caption,
        );
        e > 0 && s.max(1) == season.max(1) && e == episode
    })
}

/// `tt1234567[:S:E]`, `tmdb:123`, `kitsu:12:1`, `mal:5`, `tvdb:81797:1:1`,
/// `anilist:21:1:1`, `tvmaze:82:1:1`.
///
/// Every prefix the PHP resolves through `fd_resolve_external_media_metadata`
/// belongs here. `wp_data::lookup_id()` is prefix-agnostic (the WordPress
/// `/lookup-id` handler reads the generic `media_ids_idx` row), so a bridged
/// catalog row carrying `tvdb:` / `anilist:` / `tvmaze:` gets a LOCAL stream
/// list instead of only the meta bridge — which previously opened the item and
/// then answered the empty card, because `handle_external_stream()` returned
/// `None` and the route fell through to the upstream-only branch.
/// Can the ORIGIN resolve this prefix from an id alone?
///
/// `parse_external_id()` names the imdb prefix `"imdb"` and keeps the `tt` inside
/// the value (`("imdb", "tt0944947")`), which is exactly the pair the origin
/// indexes. Measured against the live origin:
/// `id=tt0944947` → 499 files + `resolved_title`, `id=imdb:tt0944947` → same,
/// `id=tmdb:1399` → 15 files, while `id=tmdb1399`, `id=mal:54492` and
/// `id=mal:54492:1:12` all answer HTTP 400 `{"ok":0,"error":"title or id
/// required"}`. Asking anyway spent a round-trip on the critical path of every
/// `mal:` / `kitsu:` / `anilist:` episode, before any file could be listed.
fn origin_resolves_by_id(prefix: &str) -> bool {
    prefix == "imdb" || prefix == "tmdb"
}

/// The id an origin probe may send: BARE. Never the item id — it carries the
/// `:S:E` suffix, which the origin rejects outright — `imdb` already holds
/// `tt0944947`, a bare `tt…` value is accepted as-is by the origin, and every
/// other prefix keeps its colon (`tmdb:1399`; `tmdb1399` is a 400).
fn origin_probe_id(prefix: &str, value: &str) -> String {
    match prefix {
        "imdb" => value.to_string(),
        "tt" => format!("tt{}", value),
        _ => format!("{}:{}", prefix, value),
    }
}

fn parse_external_id(item_id: &str) -> Option<(&'static str, String, i32, i32)> {
    let id = item_id.trim();
    if let Some(rest) = id.strip_prefix("tt") {
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.len() >= 6 {
            // `tt0944947:8:6` — the season/episode suffix is REQUIRED for a
            // series, exactly as the OpenSubtitles/Stremio id shape carries it.
            let tail = &rest[digits.len()..];
            let mut parts = tail.trim_start_matches(':').split(':');
            let season: i32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            let episode: i32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            return Some(("imdb", format!("tt{}", digits), season, episode));
        }
        return None;
    }
    for (name, prefix) in [
        ("tmdb", "tmdb:"),
        ("kitsu", "kitsu:"),
        ("mal", "mal:"),
        ("tvdb", "tvdb:"),
        ("anilist", "anilist:"),
        ("tvmaze", "tvmaze:"),
    ] {
        if let Some(rest) = id.strip_prefix(prefix) {
            let mut parts = rest.split(':');
            let value = parts.next().unwrap_or("").trim().to_string();
            if value.is_empty() {
                return None;
            }
            let season: i32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            let episode: i32 = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            return Some((name, value, season, episode));
        }
    }
    // `pm:post:` and `pm:file:` are handled before this point; an id with a
    // prefix we do not know is left to the upstream-only branch.
    None
}

/// The shared tail of both stream paths: matcher → split parts → filters →
/// sort → up to 100 playable items.
#[allow(clippy::too_many_arguments)]
async fn build_streams_from_files(
    ctx: &ReqCtx,
    item_type: &str,
    item_id: &str,
    title: &str,
    year: &str,
    aka: &str,
    episode_media_id: &str,
    files: Vec<crate::wp_data::WpFile>,
    season: i32,
    episode: i32,
    upstream: Option<tokio::task::JoinHandle<Vec<Value>>>,
) -> Response {
    let is_series = item_type == "series" || season > 0 || episode > 0;
    // Verified episode names (`fd_upstream_episode_names`) let the matcher
    // reject a file that names ANOTHER show's episode after this episode's
    // marker — the sibling-work case. It is looked up once per stream request,
    // 7 d cached, and `[]` (no metadata / transport failure / placeholder name)
    // means "no evidence": the matcher then fails OPEN, so adding this can never
    // hide a stream on its own.
    let episode_names: Vec<String> = if is_series && !episode_media_id.trim().is_empty() {
        let settings = load_catalog_settings_merged();
        crate::upstream::episode_names(
            &settings,
            episode_media_id,
            season,
            episode,
            title,
            year,
        )
        .await
    } else {
        Vec::new()
    };
    let mut matched: Vec<crate::wp_data::WpFile> = files
        .iter()
        .filter(|f| {
            if is_series {
                crate::matcher::series_file_matches_title(
                    &f.title,
                    title,
                    season.max(1),
                    episode.max(1),
                    year,
                    &f.caption,
                    &episode_names,
                )
            } else {
                crate::matcher::movie_file_matches_title(&f.title, title, year, &f.caption, aka)
            }
        })
        .cloned()
        .collect();
    // Last resort for a renamed film: match the files against the ALTERNATE
    // title only ("Khurafat: Perjanjian Syaitan" vs the indexed "Superstition").
    if matched.is_empty() && !is_series && !aka.is_empty() && aka != title {
        matched = files
            .iter()
            .filter(|f| crate::matcher::movie_file_matches_title(&f.title, aka, year, &f.caption, title))
            .cloned()
            .collect();
    }
    // Exact year beats the ±1 rescue window: a same-name film released a year
    // apart must not ride along ("Dhoom Dhaam" 2025 serving the 2024 rip). The
    // window survives only for a title whose files ALL carry an imprecise year,
    // which is what it was written for. Movies only — the series branch has its
    // own year rules in `series_file_matches_title()`.
    if !is_series && !year.trim().is_empty() {
        let tiers: Vec<crate::matcher::YearTier> = matched
            .iter()
            .map(|f| crate::matcher::movie_year_tier(&f.title, &f.caption, year))
            .collect();
        let keep = crate::matcher::year_keep_mask(&tiers);
        let before = matched.len();
        let mut looked = 0usize;
        matched.retain(|_| {
            let keep_this = keep.get(looked).copied().unwrap_or(true);
            looked += 1;
            keep_this
        });
        if matched.len() != before {
            log(
                "stream: dropped adjacent-year files",
                &serde_json::json!({
                    "item_id": item_id, "title": title, "year": year,
                    "dropped": before - matched.len(),
                }),
            );
        }
    }

    if matched.is_empty() {
        log(
            "stream: no file matched the title",
            &serde_json::json!({
                "item_id": item_id, "title": title, "year": year,
                "season": season, "episode": episode, "is_series": is_series,
            }),
        );
        crate::diagnostics::record(
            "stream_incident",
            "no file matched the title",
            title,
            year,
            "NO_STREAMS",
            "",
            item_id,
            serde_json::json!({
                "item_type": item_type, "title": title, "year": year,
                "season": season, "episode": episode, "runtime": "pencarimovie-server",
            }),
        );
        let merged = await_upstream(upstream, true).await;
        return finish_streams(ctx, item_type, title, item_id, merged, false, 0).await;
    }

    // Split-part annotation + ordering runs BEFORE the filters, because
    // `exclude_part_title` reads the flag it writes.
    //
    // The runtime is filled in FIRST, while `items` is still index-aligned with
    // `matched` — `group_split_parts` sorts in place, so a later lookup by index
    // would pair an item with the wrong file.
    let duration_bot = get_bot_id();
    let codes: Vec<String> = matched.iter().map(|f| f.short_code.clone()).collect();
    let mut items: Vec<crate::matcher::FileItem> = matched
        .iter()
        .enumerate()
        .map(|(i, f)| crate::matcher::FileItem {
            title: f.title.clone(),
            caption: f.caption.clone(),
            file_size: f.file_size,
            // A row that carries its own runtime saves the resolve entirely.
            duration: if f.duration > 0 {
                f.duration
            } else {
                cached_duration(&codes[i], &duration_bot)
            },
            ..Default::default()
        })
        .collect();
    // The origin's own `part` IS the ordinal — the only cheap one there is:
    // `PrevUID`/`NextUID`, chapters and `DateUTC` are all absent on the real split
    // uploads, and duration cannot prove order.
    //
    // It can arrive in two places and today arrives in neither: the
    // `post-files`/`search-files` row does not carry it (measured keys: id, title,
    // caption, short_code, file_type, mime, file_size, extension, added_date,
    // play_url, ddl_url, thumbnail_url), and neither does `/resolve-file`. So read
    // BOTH — the row first, then the cached resolve entry we already hold for its
    // duration — and one field on either side turns every 🧩 badge into a real
    // `Part n` with no further code change here.
    for (i, (item, f)) in items.iter_mut().zip(matched.iter()).enumerate() {
        let part = if f.part > 0 {
            f.part
        } else {
            cached_part(&codes[i], &duration_bot)
        };
        if part > 0 {
            item.is_split_part = true;
            item.part_num = part;
            if item.clean_base.is_empty() {
                item.clean_base = item.title.trim().to_lowercase();
            }
        }
    }
    // A file's RUNTIME is the only evidence left when several chunks of one
    // release all carry the SAME file name (tt16311594: seven files named
    // `…H265.AOC.mkv`; Obsession: eight named `…Org.AMZN.mkv`, 6621 s together).
    //
    // Deliberately CACHE-ONLY. Awaiting a resolve per unknown chunk here measured
    // **13 s** on the first load of `pm:post:8999920299` (12 codes, 4 of them
    // hitting the retry ladder) and still missed the group it was meant to
    // classify. `prewarm_streams_batch` fills the cache in the background instead,
    // and the codes it is handed are ordered so duplicate-name groups come first
    // (see below) — so the next load classifies them, at zero added latency.
    let perm = crate::matcher::group_split_parts(&mut items);
    // `group_split_parts` reorders `items` in place (the PHP sorts in place too),
    // and `matched` is the parallel vector everything printed here is read from.
    // Re-apply the same permutation, or `matched[*idx]` describes a DIFFERENT
    // file than the annotation at `items[*idx]` — which is how a dropped split
    // part still reached the player under a kept row's payload
    // (`pm:post:9000009680`: 77 flagged + filtered, 16 part names still listed).
    if perm.iter().enumerate().any(|(i, &p)| i != p) {
        matched = crate::matcher::reorder_with(&matched, &perm);
    }
    crate::matcher::detect_parts_by_duration(&mut items);

    let settings = read_catalog_settings();
    let cfg = crate::matcher::StreamFilterConfig::from_json(
        settings.get("stream_config").unwrap_or(&Value::Null),
    );
    let kept = crate::matcher::apply_stream_filters(&items, &cfg);

    if kept.is_empty() {
        log(
            "stream: files filtered out by stream_config",
            &serde_json::json!({"item_id": item_id, "matched": matched.len()}),
        );
        crate::diagnostics::record(
            "stream_incident",
            "streams filtered out by the AIOStreams config",
            title,
            year,
            "FILTERED_OUT",
            "",
            item_id,
            serde_json::json!({
                "item_type": item_type, "title": title,
                "files_before_filter": matched.len(), "runtime": "pencarimovie-server",
            }),
        );
        let merged = await_upstream(upstream, true).await;
        return finish_streams(ctx, item_type, title, item_id, merged, true, matched.len()).await;
    }

    let mut scored: Vec<(f64, Value)> = kept
        .iter()
        .filter_map(|(idx, _res_key)| {
            let f = &matched[*idx];
            if !f.short_code.is_empty() && crate::cache::get("flagged_malware", &f.short_code).is_some() {
                return None;
            }
            let score = crate::matcher::stream_sort_score(
                &f.title,
                &f.caption,
                f.file_size,
                "auto",
            );
            Some((
                score,
                stream_item(
                    ctx,
                    &f.short_code,
                    &f.title,
                    &f.caption,
                    f.file_size,
                    &f.mime,
                    &items[*idx],
                ),
            ))
        })
        .collect();
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(100);

    let mut streams: Vec<Value> = scored.into_iter().map(|(_, v)| v).collect();
    let built = streams.len();
    // A browser / WebView player cannot demux Matroska: Chrome rejects
    // `video/x-matroska` inside `<video>`, aborts on the container and re-requests the
    // same file every few seconds. Measured on a real phone: `range: bytes=2848468527-`
    // (the last ~178 KB of a 2.8 GB MKV) repeating every 4-5 s while the server
    // answered 206 correctly — "it's not streaming". So for a browser UA the
    // playable containers come first. NOTHING is dropped: an MKV-only title still
    // lists, just after any MP4/WEBM alternative.
    if crate::matcher::wants_web_container(&ctx.hdr("user-agent")) {
        let (playable, matroska): (Vec<Value>, Vec<Value>) = streams.into_iter().partition(|s| {
            let hint = s
                .get("behaviorHints")
                .and_then(|b| b.get("filename"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let url = s.get("url").and_then(|v| v.as_str()).unwrap_or("");
            let name = if hint.is_empty() { url } else { hint };
            !name.to_ascii_lowercase().ends_with(".mkv")
        });
        let moved = matroska.len();
        streams = playable.into_iter().chain(matroska).collect();
        if moved > 0 {
            log(
                "stream: matroska moved last for a browser client",
                &serde_json::json!({"item_id": item_id, "mkv": moved}),
            );
        }
    }
    // Warm the resolve cache for everything in this list (backend.php:3701,
    // `fd_prewarm_streams_batch`). The list is built from metadata only, so the
    // Bot API file_id is resolved on the first Range request — the Kotlin app got
    // an instant Play out of exactly this and the port did not. Background, so the
    // list still returns immediately; a resolve error only logs.
    // Order matters: one prefetch pass is bounded (PREWARM_HARD_CAP) and the
    // origin's own queue is capped at 40 pending codes newest-first, so the
    // members of a duplicate-name group — the only rows a runtime can classify —
    // go first. Otherwise a list full of other groups' codes leaves them cold
    // load after load.
    let mut name_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    for item in items.iter() {
        *name_counts
            .entry(item.title.trim().to_lowercase())
            .or_default() += 1;
    }
    let mut grouped: Vec<String> = Vec::new();
    let mut singles: Vec<String> = Vec::new();
    for (idx, _) in kept.iter() {
        let code = matched[*idx].short_code.clone();
        if name_counts
            .get(&items[*idx].title.trim().to_lowercase())
            .copied()
            .unwrap_or(0)
            > 1
        {
            grouped.push(code);
        } else {
            singles.push(code);
        }
    }
    grouped.extend(singles);
    crate::resolve::prewarm_streams_batch(grouped, &get_bot_id(), "stream");
    // Upstream items go AFTER ours (backend.php:18124) — a local file is always
    // the better source, and the sponsor is still prepended by
    // `finish_streams()`, so the order ends up sponsor → ours → upstream.
    let upstream_items = await_upstream(upstream, built == 0).await;
    let upstream_count = upstream_items.len();
    streams.extend(upstream_items);
    log(
        "stream: list built",
        &serde_json::json!({
            "item_id": item_id, "title": title, "matched": matched.len(),
            "kept": built, "upstream": upstream_count,
        }),
    );
    finish_streams(ctx, item_type, title, item_id, streams, false, matched.len()).await
}

/// Final assembly, exactly like `backend.php:18206-18244`:
///
/// 1. an empty list is NOT `{"streams":[]}` — it carries ONE tappable card
///    ("Streams Filtered Out" + `#catalogs`, or "No Streams Found" + the
///    keyword search on THIS origin), so the user has somewhere to go;
/// 2. the sponsored message is prepended whenever the WordPress version
///    endpoint configures one — **even when the list is empty** (PHP: "ALWAYS
///    add the sponsored message on top if configured").
/// How long a finished stream list is reused (`/stream/*` response cache).
///
/// The PHP cached the same responses for 300 s and the missing cache was the
/// single biggest latency item here: a warm `/stream` measured **430-470 ms**
/// (origin + upstream addons recomputed every time) versus **~1 ms** from the cache.
/// 300 s is short enough that a freshly indexed file or a changed sponsor still
/// appears quickly.
const STREAM_CACHE_TTL: i64 = 300;

/// Cache key for a finished stream list.
///
/// The TYPE is part of the key: `tt123` is a different request as a movie and as a
/// series root. The base url is too, because every download url in the payload is
/// absolute (`http://192.168.1.5:8088/api/download/...`).
///
/// The version prefix retires bodies written by an older build, because a cached
/// list outlives a restart (L2 is on disk) and cannot be "upgraded":
///
/// * `v2` — the sponsor used to be baked into the cached body, which could not be
///   distinguished from a body cached during a WordPress outage;
/// * `v3` — every row carried `behaviorHints.notWebReady`, which made Stremio
///   (web AND Android) hide all of them;
/// * `v4` — every row carried a `title` field that is not part of stremio-core's
///   stream model.
///
/// Each of those bodies would otherwise be served for the rest of its short TTL
/// after the fix, which is exactly long enough to look like the fix failed.
fn stream_cache_key(item_type: &str, item_id: &str, base_url: &str) -> String {
    format!("v4|{}|{}|{}", item_type, item_id, base_url)
}

/// Place the per-answer rows above a finished stream list.
///
/// The sponsored item is ALWAYS [0], the WordPress minimum-version notice sits
/// between it and the media rows, and everything else keeps its order. ONE
/// function for every `/stream/*` answer — a real list, an empty list, a single
/// "no session yet" / "file could not be resolved" card, and the locked
/// "Addon URL changed or password required" card. Before this, only
/// [`finish_streams()`] inserted the sponsor, so every answer that built its own
/// array lost it (reported as "the sponsored message is not always on top" and
/// "when the URL changed / password message appears the sponsored message is
/// gone").
fn with_top_items(
    mut streams: Vec<Value>,
    notice: Option<Value>,
    sponsor: Option<Value>,
) -> Vec<Value> {
    if let Some(notice) = notice {
        streams.insert(0, notice);
    }
    if let Some(sponsor) = sponsor {
        streams.insert(0, sponsor);
    }
    streams
}

/// Fetch the per-answer rows (update notice, sponsor) and place them above
/// `streams` with [`with_top_items()`].
///
/// Both come from their own caches (300 s / 600 s), so this is a map lookup on
/// the hot path; neither is ever stored in the `/stream` response cache.
async fn decorate_streams(base_url: &str, streams: Vec<Value>) -> Vec<Value> {
    // The update notice as a NON-BLOCKING item: the PHP returned this card INSTEAD
    // of the streams, which made an outdated engine unusable. Placed below the
    // sponsor item (if any), above media streams.
    let notice = update_notice(base_url).await;
    let sponsor = sponsor_item().await;
    with_top_items(streams, notice, sponsor)
}

/// A stream answer whose list is ONE card (no Telegram session yet, a file that
/// could not be resolved, ...): the card keeps its place BELOW the sponsor.
async fn single_card_streams(base_url: &str, card: Value) -> Response {
    let streams = decorate_streams(base_url, vec![card]).await;
    stremio_json(200, &serde_json::json!({"streams": streams}), "no-store")
}

async fn finish_streams(
    ctx: &ReqCtx,
    item_type: &str,
    title: &str,
    item_id: &str,
    mut streams: Vec<Value>,
    filtered: bool,
    files_before_filter: usize,
) -> Response {
    if streams.is_empty() {
        streams.push(no_streams_card(ctx, title, item_id, filtered, files_before_filter));
    }
    // The cache holds the LIST ONLY. Baking the sponsor (and the update notice)
    // into the cached body meant one failed WordPress call hid the sponsor from
    // every client for the next [`STREAM_CACHE_TTL`] seconds, and froze an
    // "update available" card long after the update, both intermittent
    // "the sponsor is not on top" reports.
    crate::cache::put(
        "stream",
        &stream_cache_key(item_type, item_id, &ctx.base_url),
        &serde_json::json!({"streams": &streams}),
        STREAM_CACHE_TTL,
        0,
    );
    let streams = decorate_streams(&ctx.base_url, streams).await;
    stremio_json(200, &serde_json::json!({"streams": streams}), "no-store")
}

/// Build the "update available" item from a `/api/update/check` answer
/// (the same feed the dashboard header card reads).
///
/// Non-blocking by design: it never replaces the streams, and it only notifies.
///
/// The link goes to THIS server's dashboard, not to GitHub. The reader got the
/// stream from this server, so the dashboard is reachable by definition, and its
/// header card names the one update path that works on this host: "Update now" for a
/// binary that can replace itself, `docker compose pull` for a container, the APK for
/// an app shell, the release page for a manual install. A GitHub link made every host
/// read the notes and guess which case it was.
fn build_update_notice(check: &Value, base_url: &str) -> Value {
    let current_raw = check
        .get("current_display")
        .or_else(|| check.get("current"))
        .or_else(|| check.get("current_version"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let current = crate::config::clean_version(current_raw);

    let latest_raw = check
        .get("latest_display")
        .or_else(|| check.get("latest"))
        .or_else(|| check.get("minimum_version"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let latest = crate::config::clean_version(latest_raw);

    // The notice NOTIFIES. One line, one version pair, no instructions: the page
    // behind the link is where the how-to lives.
    serde_json::json!({
        "name": "\u{2b06}\u{fe0f} Update available",
        "description": format!("PencariMovie v{latest} is available (this install is v{current})."),
        "externalUrl": format!("{}/", base_url.trim_end_matches('/')),
    })
}

/// `None` unless a newer release is published (same feed as the header card).
async fn update_notice(base_url: &str) -> Option<Value> {
    let check = crate::update::check().await;
    let available = check
        .get("update_available")
        .and_then(|v| v.as_bool())
        .or_else(|| check.get("update_needed").and_then(|v| v.as_bool()))
        .unwrap_or(false);
    if !available {
        return None;
    }
    Some(build_update_notice(&check, base_url))
}

/// Non-default `stream_config` entries that can hide a file, as short phrases.
///
/// The card used to say "hidden by your active filters (Resolution / Codec /
/// Size / Keywords)" — true, but it never named the one that was doing it, so a
/// leftover `required_keywords` looked exactly like a backend outage.
fn active_filter_notes(stream_config: &Value) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let text = |k: &str| {
        stream_config
            .get(k)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string()
    };
    let num = |k: &str| stream_config.get(k).and_then(|v| v.as_i64()).unwrap_or(0);
    let flag = |k: &str| stream_config.get(k).and_then(|v| v.as_bool()).unwrap_or(false);

    let required = text("required_keywords");
    if !required.is_empty() {
        out.push(format!("required keywords \"{required}\""));
    }
    let excluded = text("excluded_keywords");
    if !excluded.is_empty() {
        out.push(format!("excluded keywords \"{excluded}\""));
    }
    if num("max_size_gb") > 0 {
        out.push(format!("max size {} GB", num("max_size_gb")));
    }
    if num("min_size_mb") > 0 {
        out.push(format!("min size {} MB", num("min_size_mb")));
    }
    if num("max_streams_total") > 0 {
        out.push(format!("first {} stream(s) only", num("max_streams_total")));
    }
    if num("max_streams_per_resolution") > 0 {
        out.push(format!(
            "{} per resolution",
            num("max_streams_per_resolution")
        ));
    }
    if flag("exclude_cam") {
        out.push("CAM releases excluded".to_string());
    }
    for (group, label) in [
        ("resolutions", "resolution"),
        ("qualities", "quality"),
        ("encodes", "codec"),
        ("visual_tags", "tag"),
    ] {
        if let Some(map) = stream_config.get(group).and_then(|v| v.as_object()) {
            let off: Vec<&str> = map
                .iter()
                .filter(|(_, v)| v.as_bool() == Some(false))
                .map(|(k, _)| k.as_str())
                .collect();
            if !off.is_empty() {
                out.push(format!("{label} off: {}", off.join(", ")));
            }
        }
    }
    out
}

/// `⚠️ Streams Filtered Out` vs `No Streams Found` (backend.php:18206-18234).
fn no_streams_card(
    ctx: &ReqCtx,
    title: &str,
    item_id: &str,
    filtered: bool,
    files_before_filter: usize,
) -> Value {
    if filtered && files_before_filter > 0 {
        let settings = load_catalog_settings_merged();
        let notes = active_filter_notes(&settings["stream_config"]);
        let detail = if notes.is_empty() {
            "your active filters (Resolution / Codec / Size / Keywords)".to_string()
        } else {
            format!("your active filters: {}", notes.join("; "))
        };
        return serde_json::json!({
            "name": "⚠️ Streams Filtered Out",
            "description": format!(
                "🔍 {} stream(s) found on PencariMovie, but hidden by {}.\n👉 Tap to open Catalogs and adjust filters.",
                files_before_filter, detail
            ),
            "externalUrl": format!("{}/#catalogs", ctx.base_url.trim_end_matches('/')),
        });
    }
    // The keyword comes from the TITLE, falling back to the item id. The link
    // points at the keyword search on the CURRENT origin, not at the sponsor.
    let keyword = if !title.trim().is_empty() {
        title.trim().to_string()
    } else {
        item_id.to_string()
    };
    let base = ctx.base_url.trim_end_matches('/');
    serde_json::json!({
        "name": "No Streams Found",
        "description": "⚠️ No streams available for this title.",
        "externalUrl": format!(
            "https://pencarimovie.com/search/{}?ddldomain={}",
            urlencode_component(&keyword),
            urlencode_component(base)
        ),
    })
}

/// The sponsored message from `GET {WP_API_BASE}/version` (`sponsor.{name,
/// description,url}`), cached 10 min. Absent configuration means no item.
async fn sponsor_item() -> Option<Value> {
    let version = crate::httpc::cached_wp_json(&crate::config::wp_version_url(), 600, 8).await;
    let sponsor = version.get("sponsor")?;
    let url = sponsor.get("url").and_then(|v| v.as_str()).unwrap_or("").trim();
    if url.is_empty() {
        return None;
    }
    let name = sponsor.get("name").and_then(|v| v.as_str()).unwrap_or("").trim();
    let desc = sponsor
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    Some(serde_json::json!({
        "name": if name.is_empty() { "PencariMovie" } else { name },
        "description": if desc.is_empty() { "Join our Telegram channel for updates & requests" } else { desc },
        "externalUrl": url,
    }))
}

/// One Stremio stream item, in the PHP's exact shape
/// (`backend.php:17832-17907`):
///
/// ```text
/// line 1: <clean release filename>
/// line 2: 💾 1.6 GB • ⚡ Telegram • BluRay • HEVC • AAC5.1
/// ```
///
/// `behaviorHints.filename` carries the real name and `videoSize` the byte
/// count. **`notWebReady` is deliberately NOT sent** — see the note inside, the
/// one flag that made Stremio drop every row. `bingeGroup` (series only) carries
/// RELEASE attributes and never an episode number, because Stremio compares it
/// with `===` across episodes.
#[allow(clippy::too_many_arguments)]
fn stream_item(
    ctx: &ReqCtx,
    short_code: &str,
    title: &str,
    caption: &str,
    size: u64,
    mime_hint: &str,
    item: &crate::matcher::FileItem,
) -> Value {
    // Generic `video.mp4` style names take their display title from the caption
    // (backend.php:17739-17744).
    let mut display_title = title.to_string();
    if !caption.is_empty()
        && fancy_regex::Regex::new("(?i)^(?:video(?:\\.\\d+)*|\\d+|document|file)\\.(?:mp4|mkv|avi|mov|ts|flv)$")
            .map(|re| re.is_match(title.trim()).unwrap_or(false))
            .unwrap_or(false)
    {
        if let Some(first) = caption.lines().next() {
            let first = first.trim();
            if !first.is_empty() {
                display_title = first.to_string();
            }
        }
    }
    let tags = crate::matcher::extract_media_tags(&display_title, caption);
    let clean = crate::matcher::clean_media_title(&display_title);
    let display_name = if !clean.is_empty() {
        clean.clone()
    } else if !display_title.is_empty() {
        display_title.clone()
    } else {
        format!("File ShortCode: {}", short_code)
    };
    let file_name = stream_filename(
        if clean.is_empty() { &display_title } else { &clean },
        mime_hint,
    );
    let mime = guess_video_mime(&file_name, mime_hint);

    // `name` = "PencariMovie <resolution badge>" plus a visual-tag line
    // (backend.php:17783-17796).
    let res_badge = if tags.resolution.is_empty() {
        "Direct".to_string()
    } else {
        tags.resolution.clone()
    };
    let mut visual_pills: Vec<&str> = Vec::new();
    for tag in ["HDR", "DV", "10bit", "IMAX"] {
        if tags.visual.iter().any(|v| v == tag) {
            visual_pills.push(tag);
        }
    }
    let name = if visual_pills.is_empty() {
        format!("PencariMovie {}", res_badge)
    } else {
        format!("PencariMovie {}\n{}", res_badge, visual_pills.join(" | "))
    };

    // metaPills order: platform, source, visual*, codec, audio, edition — unique
    // (backend.php:17750-17771).
    let mut meta_pills: Vec<String> = Vec::new();
    if !tags.platform.is_empty() {
        meta_pills.push(tags.platform.clone());
    }
    if !tags.source.is_empty() {
        meta_pills.push(tags.source.clone());
    }
    for v in &tags.visual {
        meta_pills.push(v.clone());
    }
    if !tags.codec.is_empty() {
        meta_pills.push(tags.codec.clone());
    }
    if !tags.audio.is_empty() {
        meta_pills.push(tags.audio.clone());
    }
    if !tags.edition.is_empty() {
        meta_pills.push(tags.edition.clone());
    }
    let part = part_label(item);
    let mut spec_bits: Vec<String> = Vec::new();
    if size > 0 {
        spec_bits.push(format!("💾 {}", format_bytes(size)));
    }
    // Runtime, when the row or the resolve cache knows it.
    //
    // Deliberately on EVERY row, not only on split parts: a normal file's runtime
    // is useful metadata in its own right (it is what lets a viewer compare a
    // 2.5 GB copy with a 25 GB one, or spot a truncated upload), and the 🧩 badge
    // below is what marks a chunk. Restricting this to `is_split_part` would make
    // it invisible whenever `exclude_part_title` (ON by default) hides the parts.
    if item.duration > 0 {
        spec_bits.push(format!("⏱ {}", format_duration(item.duration)));
    }
    spec_bits.push("⚡ Telegram".to_string());
    for pill in meta_pills {
        if pill != "HDR" {
            spec_bits.push(pill);
        }
    }

    // Issue #9. The real client (Nuvio) renders the `name` badge, then the FIRST
    // line of `description` large, then the pill line — and it ignores `title`
    // entirely. So a part marker that only lives in the pills still leaves six
    // chunks looking identical on the line people actually read: it goes on the
    // file-name line, which is line 1 of `description`.
    let file_line = match &part {
        Some(p) => format!("{} · {}", display_name, p),
        None => display_name.clone(),
    };
    let mut description = format!("{}\n{}", file_line, spec_bits.join(" • "));
    let flags = language_flags(&display_title, caption);
    if !flags.is_empty() {
        description.push_str(&format!("\n💬 {}", flags.join(" / ")));
    }

    let mut hints = serde_json::Map::new();
    hints.insert("filename".into(), serde_json::json!(file_name.clone()));
    // NO `notWebReady`. The PHP never sent it on a playable URL (only on the
    // `externalUrl` variant), and it is the flag that emptied the list in
    // Stremio: `web.stremio.com` AND the Android app HIDE every stream that
    // carries it, so a Matroska-only title showed the sponsored item and nothing
    // else — 100 of 100 rows gone while Nuvio, which ignores the hint, listed
    // them all. `filename` still tells a player what container it is getting, so
    // the dashboard can choose libmpv itself; a row the user cannot SEE is worse
    // than a row the web player cannot demux.
    if size > 0 {
        hints.insert("videoSize".into(), serde_json::json!(size));
    }
    hints.insert(
        "bingeGroup".into(),
        serde_json::json!(format!(
            "pencarimovie|{}|{}|{}",
            if tags.resolution.is_empty() { "unknown" } else { &tags.resolution },
            if tags.source.is_empty() { "unknown" } else { &tags.source },
            if tags.codec.is_empty() { "unknown" } else { &tags.codec },
        )),
    );

    let payload = serde_json::json!({
        "file_size": size,
        "file_name": file_name,
        "mime": mime,
        "short_code": short_code,
    });
    let b64 = crate::fileid::base64url_encode(payload.to_string().as_bytes());
    let url = build_stream_url(&ctx.base_url, &b64, &file_name);
    // NO `title`. stremio-core (web AND Android) parses a stream into
    // `{ name, description, url|externalUrl|infoHash, behaviorHints }` — `title`
    // is not part of that model, and a row carrying it was dropped while the
    // sponsor (which never carried it) still rendered: "streams list for stremio
    // is broken, only show sponsored message". `description` is the field every
    // client renders, and it already carries the same text.
    serde_json::json!({
        "name": name,
        "description": description,
        "url": url,
        "behaviorHints": Value::Object(hints),
    })
}

/// Runtime in seconds from the resolve cache (`0` = not resolved yet).
fn cached_duration(short_code: &str, bot_id: &str) -> u32 {
    if short_code.is_empty() || bot_id.is_empty() {
        return 0;
    }
    let Some(entry) = crate::resolve::resolve_shortcode_cached(short_code, bot_id) else {
        return 0;
    };
    duration_of(&entry)
}

fn duration_of(entry: &Value) -> u32 {
    entry
        .get("duration")
        .and_then(|d| {
            d.as_u64()
                .or_else(|| d.as_str().and_then(|s| s.trim().parse().ok()))
        })
        .unwrap_or(0) as u32
}

/// `part` as carried by a row or a resolve entry (`0` = absent / not a part).
fn part_of(entry: &Value) -> i32 {
    entry
        .get("part")
        .and_then(|p| {
            p.as_i64()
                .or_else(|| p.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
        })
        .unwrap_or(0)
        .clamp(0, i32::MAX as i64) as i32
}

/// The split-part index carried by a CACHED resolve entry (`0` = absent).
///
/// The cheap path: `/resolve-file` is the one payload the engine already fetches
/// (for `duration`), so its `part` costs nothing extra — no media probe, no extra
/// request. `prewarm_streams_batch` warms those entries in the background.
fn cached_part(short_code: &str, bot_id: &str) -> i32 {
    if short_code.is_empty() || bot_id.is_empty() {
        return 0;
    }
    let Some(entry) = crate::resolve::resolve_shortcode_cached(short_code, bot_id) else {
        return 0;
    };
    part_of(&entry)
}

/// `22 min` / `2h 36m` — the runtime pill. Rounded to the minute: a chunk is
/// never "1185 s" to a viewer.
fn format_duration(secs: u32) -> String {
    let minutes = (secs as u64 + 30) / 60;
    if minutes >= 60 {
        format!("{}h {:02}m", minutes / 60, minutes % 60)
    } else {
        format!("{} min", minutes)
    }
}

/// The part badge for one row, including the case where the ORDER is unknown.
///
/// Issue #9: six rows of one release read identically because the payload had no
/// `title`; the badges were missing too, because a part was only labelled when a
/// name carried `partNNN`. A duration-detected chunk has no ordinal at all, and
/// that is precisely the row that needs to say so.
fn part_label(item: &crate::matcher::FileItem) -> Option<String> {
    if !item.is_split_part {
        return None;
    }
    let total = item.total_parts;
    Some(match (item.part_num, total) {
        (n, t) if n > 0 && t > 1 => format!("🧩 Part {}/{}", n, t),
        (n, _) if n > 0 => format!("🧩 Part {}", n),
        (_, t) if t > 1 => format!("🧩 Split · {} parts", t),
        // Detected as a chunk, but neither the count nor the position is knowable
        // from the container (measured: no PrevUID/NextUID, no chapters, no
        // per-part DateUTC, identical statistics tags).
        _ => "🧩 Split · order unknown".to_string(),
    })
}


/// `💬 🇲🇾 Malay / 🇬🇧 English` (backend.php:17803-17830). Compiled once.
fn language_flags(title: &str, caption: &str) -> Vec<&'static str> {
    static PATTERNS: std::sync::OnceLock<Vec<(fancy_regex::Regex, &'static str)>> =
        std::sync::OnceLock::new();
    let patterns = PATTERNS.get_or_init(|| {
        let table: [(&str, &'static str); 8] = [
            (r"(?i)\b(malay|malaysub|sub\s*malay|msia|melayu)\b", "🇲🇾 Malay"),
            (r"(?i)\b(eng|english|esub|sub\s*eng)\b", "🇬🇧 English"),
            (r"(?i)\b(indo|indonesia|indosub)\b", "🇮🇩 Indo"),
            (r"(?i)\b(hindi|hin|dub\s*hindi)\b", "🇮🇳 Hindi"),
            (r"(?i)\b(korean|kor|kdrama)\b", "🇰🇷 Korean"),
            (r"(?i)\b(japanese|jap|anime)\b", "🇯🇵 Japanese"),
            (r"(?i)\b(chinese|mandarin|cantonese|c-drama)\b", "🇨🇳 Chinese"),
            (r"(?i)\b(thai)\b", "🇹🇭 Thai"),
        ];
        table
            .into_iter()
            .filter_map(|(p, label)| fancy_regex::Regex::new(p).ok().map(|re| (re, label)))
            .collect()
    });
    let text = format!("{} {}", title, caption);
    let mut out: Vec<&'static str> = Vec::new();
    for (re, label) in patterns {
        if re.is_match(&text).unwrap_or(false) && !out.contains(label) {
            out.push(label);
        }
    }
    out
}

fn build_stream_url(base_url: &str, payload_b64: &str, file_name: &str) -> String {
    let token = if auth_enabled() { auth_token() } else { String::new() };
    let token_segment = if token.is_empty() {
        String::new()
    } else {
        format!("/{}", urlencode_component(&token))
    };
    format!(
        "{}{}/api/download/{}/{}",
        rtrim(base_url),
        token_segment,
        urlencode_component(payload_b64),
        urlencode_component(file_name)
    )
}

/* ── small helpers ───────────────────────────────────────────────────────── */

fn req_for_headers() -> Request<Body> {
    Request::builder().body(Body::empty()).unwrap()
}

fn base_url_of(ctx: &ReqCtx, port: u16) -> String {
    let proto = {
        let p = ctx.hdr("x-forwarded-proto");
        if p.is_empty() {
            if ctx.hdr("cf-visitor").contains("https") {
                "https".to_string()
            } else {
                "http".to_string()
            }
        } else {
            p.split(',').next().unwrap_or("http").trim().to_string()
        }
    };
    let host = {
        let h = ctx.hdr("x-forwarded-host");
        if h.is_empty() {
            let h2 = ctx.hdr("host");
            if h2.is_empty() {
                format!("127.0.0.1:{}", port)
            } else {
                h2
            }
        } else {
            h
        }
    };
    format!("{}://{}", proto, host.split(',').next().unwrap_or("").trim())
}

fn urlencode_component(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn srt_timestamps_to_vtt(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.lines() {
        if line.contains("-->") {
            out.push_str(&line.replace(',', "."));
        } else {
            out.push_str(line);
        }
        out.push('\n');
    }
    out
}

fn rand_suffix() -> String {
    format!("{:08x}", rand::random::<u32>())
}

/// RFC1918 only: 10/8, 172.16/12, 192.168/16.
fn is_private_ipv4(ip: &str) -> bool {
    let parts: Vec<u8> = ip.split('.').filter_map(|p| p.parse().ok()).collect();
    if parts.len() != 4 {
        return false;
    }
    matches!(
        (parts[0], parts[1]),
        (10, _) | (192, 168) | (172, 16..=31)
    )
}

/// Prefixes that are never the LAN a caller wants: cellular WAN, tunnels/VPNs, and
/// the vendor pseudo-interfaces. `vgate` is OxygenOS/ColorOS's virtual gateway
/// (`POINTOPOINT`, a private `/32`, reachable from the phone and nowhere else);
/// `p2p` is Wi-Fi Direct, `ifb`/`dummy` are traffic shaping.
///
/// Names are the LAST line of defence, not the rule — a name list is exactly what
/// missed `vgate0` for months. [`is_usable_lan_addr`] is what keeps an unknown
/// vendor device out.
fn is_never_lan_iface(name: &str) -> bool {
    const NEVER: [&str; 17] = [
        "lo", "rmnet", "ccmni", "pdp", "wwan", "clat", "dummy", "vgate", "p2p", "ifb", "sit",
        "tun", "tap", "wg", "ppp", "ipsec", "tailscale",
    ];
    let n = name.to_ascii_lowercase();
    NEVER.iter().any(|p| n.starts_with(p))
}

/// One `inet` line of `ip -4 addr`, with the interface it belongs to.
struct LanAddr {
    name: String,
    ip: String,
    prefix: u8,
    point_to_point: bool,
}

/// Parse `ip -4 addr` into one entry per IPv4 address.
///
/// Android's toybox prints the link flags in the interface header
/// (`32: vgate0: <POINTOPOINT,MULTICAST,NOARP,…>`), so `POINTOPOINT` is readable
/// without knowing anything about the device. When a header carries no `<…>` the
/// address is treated as broadcast-capable and only the name guard applies.
fn parse_lan_addrs(text: &str) -> Vec<LanAddr> {
    let mut out: Vec<LanAddr> = Vec::new();
    let mut name = String::new();
    let mut point_to_point = false;
    for line in text.lines() {
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        let starts_block = t
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && t.contains(':');
        if starts_block {
            // "32: vgate0: <POINTOPOINT,MULTICAST,…> mtu 65500 …"
            name = t.split(':').nth(1).unwrap_or("").trim().to_string();
            point_to_point = t
                .split('<')
                .nth(1)
                .and_then(|flags| flags.split('>').next())
                .map(|flags| {
                    flags
                        .split(',')
                        .any(|f| f.trim().eq_ignore_ascii_case("POINTOPOINT"))
                })
                .unwrap_or(false);
            continue;
        }
        let Some(rest) = t.strip_prefix("inet ") else {
            continue;
        };
        let mut parts = rest.split('/');
        let ip = parts.next().unwrap_or("").trim().to_string();
        let prefix = parts
            .next()
            .and_then(|p| p.split_whitespace().next())
            .and_then(|p| p.parse::<u8>().ok())
            .unwrap_or(32);
        out.push(LanAddr {
            name: name.clone(),
            ip,
            prefix,
            point_to_point,
        });
    }
    out
}

/// Wi-Fi and tethering interfaces win over everything else.
fn lan_iface_priority(name: &str) -> i32 {
    let n = name.to_ascii_lowercase();
    if n.starts_with("ap") || n.starts_with("softap") || n.starts_with("swlan") || n.starts_with("tether_wlan") {
        100
    } else if n.starts_with("wlan") {
        90
    } else if n.starts_with("eth") || n.starts_with("rndis") || n.starts_with("usb") {
        80
    } else {
        50
    }
}

/// A candidate has to pass three independent tests.
///
/// 1. Structural: broadcast-capable and on a real subnet. `/31` and `/32` have no
///    LAN to be on, and a `POINTOPOINT` link has no peers to reach — this is what
///    drops `vgate0` without naming it.
/// 2. RFC1918: a public carrier address is never advertised.
/// 3. The name guard above, for the ones that slip past 1 and 2.
fn is_usable_lan_addr(a: &LanAddr) -> bool {
    !a.point_to_point
        && a.prefix < 31
        && is_private_ipv4(&a.ip)
        && !is_never_lan_iface(&a.name)
}

/// Choose the address to advertise. Pure, so the OnePlus snapshot in the tests can
/// be replayed without a device.
///
/// The default-route address (the UDP-connect trick, no packet sent) only wins when
/// its OWN interface is a usable LAN. Trusting it on the RFC1918 test alone is how
/// `http://172.30.242.191:8088` reached the dashboard: a carrier or an OEM gateway
/// can report a private egress that no device on the wifi can reach.
pub(crate) fn pick_lan_ip(egress: Option<&str>, ip_addr_output: &str) -> String {
    let addrs = parse_lan_addrs(ip_addr_output);
    if addrs.is_empty() {
        // No interface table at all — Windows and macOS have no `ip`, and a denied
        // child process produces empty stdout. The default-route address is the only
        // signal left, and RFC1918 is the only test available.
        return match egress {
            Some(ip) => ip.to_string(),
            None => "127.0.0.1".to_string(),
        };
    }
    let usable: Vec<&LanAddr> = addrs.iter().filter(|a| is_usable_lan_addr(a)).collect();
    if let Some(ip) = egress {
        if usable.iter().any(|a| a.ip == ip) {
            return ip.to_string();
        }
    }
    usable
        .iter()
        .max_by_key(|a| lan_iface_priority(&a.name))
        .map(|a| a.ip.clone())
        .unwrap_or_else(|| "127.0.0.1".to_string())
}

/// The address a device on the SAME NETWORK can reach, never the carrier's.
///
/// `127.0.0.1` means "nothing here is a LAN", and the caller falls back to the
/// request's own host — hiding the address is correct on mobile data.
pub(crate) fn lan_ip() -> String {
    let egress = (|| -> Option<String> {
        let sock = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
        sock.connect("8.8.8.8:80").ok()?;
        let ip = sock.local_addr().ok()?.ip().to_string();
        if ip.contains('.') && !ip.starts_with("127.") && is_private_ipv4(&ip) {
            Some(ip)
        } else {
            None
        }
    })();
    // `ip -4 addr` is present on Android (toybox) and Linux; parse `inet`.
    // `proc::cmd` so a stray `ip.exe` on Windows cannot flash a console.
    let ifaces = crate::proc::cmd("ip")
        .args(["-4", "addr"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .unwrap_or_default();
    pick_lan_ip(egress.as_deref(), &ifaces)
}

/// PHP `fd_load_catalog_settings()` defaults. The `stream_config` keys are the
/// ones the AIOStreams filter block reads, so an absent file must still produce
/// them (`exclude_unplayable` / `exclude_part_title` default to ON).
/// The seeded `enabled_catalogs` map.
///
/// `backend.php:11964-11967` seeds **every** option from its own `default` flag:
///
/// ```php
/// foreach ($catalogOptions as $id => $info) {
///     $defaults['enabled_catalogs'][$id] = !empty($info['default']);
/// }
/// ```
///
/// which is what makes a fresh install come up with Korea / Japan / Thailand /
/// Philippines (movie AND series) UNCHECKED and everything else checked. The port
/// shipped `{}` and relied on the "a missing key means enabled" rule, so every row
/// including those four came up CHECKED — the four `"default": false` flags in
/// [`default_catalog_options()`] were dead weight. Seed the map explicitly; the
/// stored file is applied over it in [`load_catalog_settings_merged()`].
fn default_enabled_catalogs() -> Value {
    let mut map = serde_json::Map::new();
    if let Some(opts) = default_catalog_options().as_object() {
        for (id, info) in opts {
            let on = info.get("default").and_then(|v| v.as_bool()).unwrap_or(false);
            map.insert(id.clone(), Value::Bool(on));
        }
    }
    Value::Object(map)
}

fn default_catalog_settings() -> Value {
    serde_json::json!({
        "catalogs_enabled": true,
        "country": "",
        "enabled_types": {"movie": true, "series": true, "other": true},
        "enabled_catalogs": default_enabled_catalogs(),
        "upstream_enabled": true,
        // Fresh installs ship with the PHP's two defaults (AIOMetadata for
        // catalog/meta, OpenSubtitles v3 for subtitles). `upstream_enabled` is
        // the single master switch; the stored list is kept when it is off, so
        // turning it back on restores everything without re-adding URLs.
        "upstream_manifests": upstream::default_manifests(),
        "stream_config": {
            "resolutions": {"4k": true, "1080p": true, "720p": true, "sd": true, "unknown": true},
            "qualities": {"remux": true, "bluray": true, "webdl": true, "webrip": true, "hdtv": true, "cam": true},
            "encodes": {"hevc": true, "avc": true, "av1": true},
            "visual_tags": {"hdr": true, "dv": true},
            "exclude_cam": false,
            "exclude_unplayable": true,
            "exclude_part_title": true,
            "preferred_resolution": "auto",
            "max_streams_per_resolution": 0,
            "max_streams_total": 0,
            "min_size_mb": 0,
            "max_size_gb": 0,
            "excluded_keywords": "",
            "required_keywords": ""
        }
    })
}

/// Stored settings merged over the defaults (one level deep for `stream_config`,
/// so a file written before a key existed still gains it).
fn load_catalog_settings_merged() -> Value {
    let mut base = default_catalog_settings();
    let stored = read_catalog_settings();
    let Some(stored_obj) = stored.as_object() else {
        return base;
    };
    for (k, v) in stored_obj {
        if k == "stream_config" {
            if let Some(sc) = v.as_object() {
                for (sk, sv) in sc {
                    base["stream_config"][sk] = sv.clone();
                }
            }
            continue;
        }
        // `upstream_manifests` is handled below: an EMPTY stored list must not
        // wipe the built-in defaults (that is the PHP's rule — see
        // backend.php:12011-12016 — otherwise every upgrade silently loses the
        // upstream addons, and the AIOMetadata default in particular).
        if k == "upstream_manifests" {
            continue;
        }
        base[k] = v.clone();
    }
    if let Some(list) = stored_obj.get("upstream_manifests").and_then(|v| v.as_array()) {
        let valid: Vec<Value> = list
            .iter()
            .filter(|m| !jstr(m, "url").is_empty())
            .cloned()
            .collect();
        if !valid.is_empty() {
            base["upstream_manifests"] = Value::Array(valid);
        }
    }
    base
}

/// `PM_CATALOG_SETTINGS` (a JSON object) with `storage/catalog_settings.json`
/// laid OVER it. Ephemeral hosts — Render, a Docker redeploy — discard the
/// container disk, so without an env layer the dashboard's catalog/upstream/
/// stream configuration silently reverted to the built-in defaults on every
/// restart. The file still wins, so in-session edits from `#addon` keep working.
fn read_catalog_settings() -> Value {
    let env_layer = env("PM_CATALOG_SETTINGS")
        .and_then(|raw| serde_json::from_str::<Value>(raw.trim()).ok());
    let file_layer = std::fs::read_to_string(storage_path("catalog_settings.json"))
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok());
    merge_settings_layers(env_layer, file_layer)
}

/// Env layer as the base, stored file keys over it. A value that is not a JSON
/// object is ignored rather than trusted (a typo must not blank every setting).
fn merge_settings_layers(env_layer: Option<Value>, file_layer: Option<Value>) -> Value {
    let mut base = match env_layer {
        Some(v) if v.is_object() => v,
        _ => serde_json::json!({}),
    };
    if let Some(obj) = file_layer.as_ref().and_then(|v| v.as_object()) {
        for (k, v) in obj {
            base[k] = v.clone();
        }
    }
    base
}

/// Persist `storage/catalog_settings.json`, beside-then-rename.
///
/// Two silent faults in the plain `fs::write` this replaces:
///
/// * It truncated before writing, while [`read_catalog_settings()`] sits on the
///   config path of every `/catalog`, `/meta`, `/stream` and `/api/settings`
///   request. A reader inside that window saw an EMPTY file, `from_str` failed,
///   and the whole configuration — catalog rows, upstream manifests, stream
///   filters — silently answered with the `PM_CATALOG_SETTINGS` env layer alone.
/// * A serialization failure was mapped to `"{}"`, i.e. it WIPED the file and
///   still reported success. A failed encode must write nothing at all.
///
/// `rename` is atomic on both NTFS and POSIX; same pattern (and same reason) as
/// [`crate::credentials::save_auth_token`]. `storage/` is untracked, so the
/// `.tmp` is only ever a transient.
fn write_catalog_settings(data: &Value) -> bool {
    let Ok(body) = serde_json::to_string_pretty(data) else {
        return false;
    };
    let path = storage_path("catalog_settings.json");
    let tmp = path.with_file_name("catalog_settings.json.tmp");
    if std::fs::write(&tmp, body).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return false;
    }
    match std::fs::rename(&tmp, &path) {
        Ok(()) => true,
        Err(_) => {
            let _ = std::fs::remove_file(&tmp);
            false
        }
    }
}

/// `fd_get_default_catalog_options()` (backend.php:11743) — an **id → info map**,
/// not a list.
///
/// The dashboard's Catalog Configuration panel iterates
/// `Object.entries(catalog_options)` and files each row under
/// `info.group === 'special'` or the main list ([`public/app.js:700`]), so the
/// previous `{catalogs: [manifest rows]}` shape rendered an EMPTY panel: none of
/// its keys carried a `group`, so every entry was skipped. The PHP map is
/// reproduced verbatim, plus the port's two merged trending post rows.
fn default_catalog_options() -> Value {
    serde_json::json!({
        // Every row here is one the descriptor can advertise (`pm_catalogs()` +
        // the keyword rows), so a checkbox always does something. The
        // `pm_movies` / `pm_latest_movies` / `pm_series` / `pm_latest_series`
        // rows this port used to ship are GONE: they exist in no PHP manifest and
        // their ids map to no client's home row, which is why the catalog was
        // reported "broken".
        //
        // `pm_series_top` and `pm_files_year` are the GATE keys for the series
        // and Telegram-file `top` rows (backend.php:15323-15327), not row ids.
        // Special catalogs: Popular, New, Search, Telegram Files, Trending.
        "top": {"type": "movie", "group": "special", "name": "Popular (Movies)", "default": true},
        "year": {"type": "movie", "group": "special", "name": "New (Movies)", "default": true},
        "pm_search_movie": {"type": "movie", "group": "special", "name": "Search Movies", "default": true},
        "pm_series_top": {"type": "series", "group": "special", "name": "Popular (Series)", "default": true},
        "pm_series_year": {"type": "series", "group": "special", "name": "New (Series)", "default": true},
        "pm_search_series": {"type": "series", "group": "special", "name": "Search Series", "default": true},
        // OFF on a fresh install by request: the Telegram-file "New" row and the
        // trending-keyword file rows are the two that cost an origin scan per
        // boot and neither is a browsing surface people land on. Both stay
        // switchable — the dashboard writes an explicit `true` to turn them back
        // on — but a fresh settings file must not ship them enabled, because
        // `default_catalog_settings()` seeds every switch from this flag and a
        // missing key means ENABLED ([`catalog_switch()`]).
        "pm_files_year": {"type": "other", "group": "special", "name": "New (Telegram Files)", "default": false},
        "pm_search_files": {"type": "other", "group": "special", "name": "Telegram Files (Search)", "default": true},
        "pm_trending_keywords": {"type": "other", "group": "special", "name": "Trending Keywords (Files)", "default": false},
        // The port's own merged trending rows. `group: "trending"` puts them in
        // the main list (only `special` gets its own column) and gives each row
        // its own checkbox, so it can be hidden without touching the PHP's
        // `pm_trending_keywords` (file-catalog) master toggle.
        "pm_trending_movies": {"type": "movie", "group": "trending", "name": "Trending Movies", "default": true},
        "pm_trending_series": {"type": "series", "group": "trending", "name": "Trending Series", "default": true},
        // Movies country catalogs.
        "pm_movies_malay": {"type": "movie", "group": "country", "name": "Malaysia (Movie)", "default": true},
        "pm_movies_indo": {"type": "movie", "group": "country", "name": "Indonesia (Movie)", "default": true},
        "pm_movies_korean": {"type": "movie", "group": "country", "name": "Korea (Movie)", "default": false},
        "pm_movies_japan": {"type": "movie", "group": "country", "name": "Japan (Movie)", "default": false},
        "pm_movies_anime": {"type": "movie", "group": "country", "name": "Anime (Movie)", "default": true},
        "pm_movies_chinese": {"type": "movie", "group": "country", "name": "China / HK (Movie)", "default": true},
        "pm_movies_thai": {"type": "movie", "group": "country", "name": "Thailand (Movie)", "default": false},
        "pm_movies_bollywood": {"type": "movie", "group": "country", "name": "India (Movie)", "default": true},
        "pm_movies_philippines": {"type": "movie", "group": "country", "name": "Philippines (Movie)", "default": false},
        "pm_movies_english": {"type": "movie", "group": "country", "name": "English (Movie)", "default": true},
        // Series country catalogs.
        "pm_series_kdrama": {"type": "series", "group": "country", "name": "K-Drama (Series)", "default": true},
        "pm_series_anime": {"type": "series", "group": "country", "name": "Anime (Series)", "default": true},
        "pm_series_japan": {"type": "series", "group": "country", "name": "J-Drama (Series)", "default": false},
        "pm_series_malay": {"type": "series", "group": "country", "name": "Malaysia (Series)", "default": true},
        "pm_series_cdrama": {"type": "series", "group": "country", "name": "C-Drama (Series)", "default": true},
        "pm_series_thai": {"type": "series", "group": "country", "name": "Thailand (Series)", "default": false},
        "pm_series_philippines": {"type": "series", "group": "country", "name": "Philippines (Series)", "default": false},
        "pm_series_india": {"type": "series", "group": "country", "name": "India (Series)", "default": true},
        "pm_series_english": {"type": "series", "group": "country", "name": "English (Series)", "default": true},
        "pm_series_indo": {"type": "series", "group": "country", "name": "Indonesia (Series)", "default": true}
    })
}

/// One checkbox in `enabled_catalogs`. A missing key means ENABLED (the PHP's
/// `!isset($map[$id]) || !empty($map[$id])` rule).
///
/// This fallback is for a settings file written BEFORE a row existed — it is NOT
/// what decides a fresh install. [`default_catalog_settings()`] seeds every option
/// from its own `default` flag (backend.php:11964), so the four rows the PHP ships
/// as `default: false` arrive here as an explicit `false` and stay unchecked.
fn catalog_switch(settings: &Value, id: &str) -> bool {
    match settings.get("enabled_catalogs").and_then(|m| m.get(id)) {
        Some(v) => v.as_bool().unwrap_or(true),
        None => true,
    }
}

/// PHP's per-row gate on the descriptor (backend.php:15303-15332): a local row
/// is advertised when its media type is enabled AND its catalog switch is on.
///
/// Three details are load-bearing and each one silently drops rows when missed:
/// 1. `series` + `top` is gated by `pm_series_top`, and `other` + `top` by
///    `pm_files_year` (the PHP checks a DIFFERENT key than the row's id);
/// 2. every `pm_topkw_<hash>` row also needs the `pm_trending_keywords` master
///    toggle, so one checkbox hides all keyword rows;
/// 3. a row whose `type` is not in `enabled_types` never reaches a client.
fn catalog_advertised(settings: &Value, cat: &Value) -> bool {
    let ctype = cat.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let type_on = settings
        .get("enabled_types")
        .and_then(|t| t.get(ctype))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if !type_on {
        return false;
    }
    let id = cat.get("id").and_then(|v| v.as_str()).unwrap_or("");
    if id.starts_with("pm_topkw_") && !catalog_switch(settings, "pm_trending_keywords") {
        return false;
    }
    let check_id = match (ctype, id) {
        ("series", "top") => "pm_series_top",
        ("other", "top") => "pm_files_year",
        _ => id,
    };
    catalog_switch(settings, check_id)
}

fn apply_stream_config(sc: &mut Value, input: &serde_json::Map<String, Value>) {
    if let Some(res) = input.get("resolutions").and_then(|v| v.as_object()) {
        sc["resolutions"] = serde_json::json!({
            "4k": res.get("4k").and_then(|v| v.as_bool()).unwrap_or(false),
            "1080p": res.get("1080p").and_then(|v| v.as_bool()).unwrap_or(false),
            "720p": res.get("720p").and_then(|v| v.as_bool()).unwrap_or(false),
            "sd": res.get("sd").and_then(|v| v.as_bool()).unwrap_or(false),
            "unknown": res.get("unknown").and_then(|v| v.as_bool()).unwrap_or(false),
        });
    }
    if let Some(q) = input.get("qualities").and_then(|v| v.as_object()) {
        sc["qualities"] = serde_json::json!({
            "remux": q.get("remux").and_then(|v| v.as_bool()).unwrap_or(false),
            "bluray": q.get("bluray").and_then(|v| v.as_bool()).unwrap_or(false),
            "webdl": q.get("webdl").and_then(|v| v.as_bool()).unwrap_or(false),
            "webrip": q.get("webrip").and_then(|v| v.as_bool()).unwrap_or(false),
            "hdtv": q.get("hdtv").and_then(|v| v.as_bool()).unwrap_or(false),
            "cam": q.get("cam").and_then(|v| v.as_bool()).unwrap_or(false),
            "unknown": q.get("unknown").and_then(|v| v.as_bool()).unwrap_or(false),
        });
    }
    if let Some(e) = input.get("encodes").and_then(|v| v.as_object()) {
        sc["encodes"] = serde_json::json!({
            "hevc": e.get("hevc").and_then(|v| v.as_bool()).unwrap_or(false),
            "avc": e.get("avc").and_then(|v| v.as_bool()).unwrap_or(false),
            "av1": e.get("av1").and_then(|v| v.as_bool()).unwrap_or(false),
        });
    }
    if let Some(v) = input.get("visual_tags").and_then(|v| v.as_object()) {
        sc["visual_tags"] = serde_json::json!({
            "hdr": v.get("hdr").and_then(|x| x.as_bool()).unwrap_or(false),
            "dv": v.get("dv").and_then(|x| x.as_bool()).unwrap_or(false),
        });
    }
    if let Some(pref) = input.get("preferred_resolution") {
        let pref = pref.as_str().unwrap_or("auto").trim().to_ascii_lowercase();
        let allowed = ["auto", "4k", "1080p", "720p", "sd"];
        sc["preferred_resolution"] =
            serde_json::json!(if allowed.contains(&pref.as_str()) { pref } else { "auto".to_string() });
    }
    let clamp = |v: Option<&Value>, lo: i64, hi: i64| -> i64 {
        let n = v.and_then(|x| x.as_i64()).unwrap_or(0);
        n.clamp(lo, hi)
    };
    if input.contains_key("max_streams_per_resolution") {
        sc["max_streams_per_resolution"] =
            serde_json::json!(clamp(input.get("max_streams_per_resolution"), 0, 50));
    }
    if input.contains_key("max_streams_total") {
        sc["max_streams_total"] = serde_json::json!(clamp(input.get("max_streams_total"), 0, 200));
    }
    if input.contains_key("min_size_mb") {
        sc["min_size_mb"] = serde_json::json!(clamp(input.get("min_size_mb"), 0, 100_000));
    }
    if input.contains_key("max_size_gb") {
        sc["max_size_gb"] = serde_json::json!(clamp(input.get("max_size_gb"), 0, 200));
    }
    for key in ["exclude_cam", "exclude_unplayable", "exclude_part_title"] {
        if let Some(v) = input.get(key) {
            sc[key] = serde_json::json!(v.as_bool().unwrap_or(false));
        }
    }
    for key in ["excluded_keywords", "required_keywords"] {
        if let Some(v) = input.get(key) {
            sc[key] = serde_json::json!(v.as_str().unwrap_or("").trim());
        }
    }
}

/// ISO-3166-1 alpha-2 countries offered by the region picker.
///
/// The origin's own catalog rows exist only for the first block, so those are
/// listed first (the PHP's order, i.e. what a user is most likely to want);
/// everything after it is alphabetical so the picker is searchable by name.
/// The PHP shipped only these 20 — the list was widened here because the field
/// is type-to-search, so hiding 180 countries bought nothing.
const COUNTRY_NAMES: &[(&str, &str)] = &[
    ("MY", "Malaysia"),
    ("ID", "Indonesia"),
    ("SG", "Singapore"),
    ("TH", "Thailand"),
    ("PH", "Philippines"),
    ("VN", "Vietnam"),
    ("KR", "Korea"),
    ("JP", "Japan"),
    ("CN", "China"),
    ("HK", "Hong Kong"),
    ("TW", "Taiwan"),
    ("IN", "India"),
    ("US", "United States"),
    ("GB", "United Kingdom"),
    ("AU", "Australia"),
    ("DE", "Germany"),
    ("NL", "Netherlands"),
    ("FR", "France"),
    ("CA", "Canada"),
    ("AF", "Afghanistan"),
    ("AL", "Albania"),
    ("DZ", "Algeria"),
    ("AD", "Andorra"),
    ("AO", "Angola"),
    ("AG", "Antigua and Barbuda"),
    ("AR", "Argentina"),
    ("AM", "Armenia"),
    ("AT", "Austria"),
    ("AZ", "Azerbaijan"),
    ("BS", "Bahamas"),
    ("BH", "Bahrain"),
    ("BD", "Bangladesh"),
    ("BB", "Barbados"),
    ("BY", "Belarus"),
    ("BE", "Belgium"),
    ("BZ", "Belize"),
    ("BJ", "Benin"),
    ("BT", "Bhutan"),
    ("BO", "Bolivia"),
    ("BA", "Bosnia and Herzegovina"),
    ("BW", "Botswana"),
    ("BR", "Brazil"),
    ("BN", "Brunei"),
    ("BG", "Bulgaria"),
    ("BF", "Burkina Faso"),
    ("BI", "Burundi"),
    ("KH", "Cambodia"),
    ("CM", "Cameroon"),
    ("CV", "Cape Verde"),
    ("CF", "Central African Republic"),
    ("TD", "Chad"),
    ("CL", "Chile"),
    ("CO", "Colombia"),
    ("KM", "Comoros"),
    ("CG", "Congo"),
    ("CD", "Congo (DRC)"),
    ("CR", "Costa Rica"),
    ("CI", "Cote d'Ivoire"),
    ("HR", "Croatia"),
    ("CU", "Cuba"),
    ("CY", "Cyprus"),
    ("CZ", "Czechia"),
    ("DK", "Denmark"),
    ("DJ", "Djibouti"),
    ("DM", "Dominica"),
    ("DO", "Dominican Republic"),
    ("EC", "Ecuador"),
    ("EG", "Egypt"),
    ("SV", "El Salvador"),
    ("GQ", "Equatorial Guinea"),
    ("ER", "Eritrea"),
    ("EE", "Estonia"),
    ("SZ", "Eswatini"),
    ("ET", "Ethiopia"),
    ("FJ", "Fiji"),
    ("FI", "Finland"),
    ("GA", "Gabon"),
    ("GM", "Gambia"),
    ("GE", "Georgia"),
    ("GH", "Ghana"),
    ("GR", "Greece"),
    ("GD", "Grenada"),
    ("GT", "Guatemala"),
    ("GN", "Guinea"),
    ("GW", "Guinea-Bissau"),
    ("GY", "Guyana"),
    ("HT", "Haiti"),
    ("HN", "Honduras"),
    ("HU", "Hungary"),
    ("IS", "Iceland"),
    ("IQ", "Iraq"),
    ("IE", "Ireland"),
    ("IT", "Italy"),
    ("JM", "Jamaica"),
    ("JO", "Jordan"),
    ("KZ", "Kazakhstan"),
    ("KE", "Kenya"),
    ("KI", "Kiribati"),
    ("KW", "Kuwait"),
    ("KG", "Kyrgyzstan"),
    ("LA", "Laos"),
    ("LV", "Latvia"),
    ("LB", "Lebanon"),
    ("LS", "Lesotho"),
    ("LR", "Liberia"),
    ("LY", "Libya"),
    ("LI", "Liechtenstein"),
    ("LT", "Lithuania"),
    ("LU", "Luxembourg"),
    ("MG", "Madagascar"),
    ("MW", "Malawi"),
    ("MV", "Maldives"),
    ("ML", "Mali"),
    ("MT", "Malta"),
    ("MH", "Marshall Islands"),
    ("MR", "Mauritania"),
    ("MU", "Mauritius"),
    ("MX", "Mexico"),
    ("FM", "Micronesia"),
    ("MD", "Moldova"),
    ("MC", "Monaco"),
    ("MN", "Mongolia"),
    ("ME", "Montenegro"),
    ("MA", "Morocco"),
    ("MZ", "Mozambique"),
    ("MM", "Myanmar"),
    ("NA", "Namibia"),
    ("NR", "Nauru"),
    ("NP", "Nepal"),
    ("NZ", "New Zealand"),
    ("NI", "Nicaragua"),
    ("NE", "Niger"),
    ("NG", "Nigeria"),
    ("KP", "North Korea"),
    ("MK", "North Macedonia"),
    ("NO", "Norway"),
    ("OM", "Oman"),
    ("PK", "Pakistan"),
    ("PW", "Palau"),
    ("PS", "Palestine"),
    ("PA", "Panama"),
    ("PG", "Papua New Guinea"),
    ("PY", "Paraguay"),
    ("PE", "Peru"),
    ("PL", "Poland"),
    ("PT", "Portugal"),
    ("PR", "Puerto Rico"),
    ("QA", "Qatar"),
    ("RO", "Romania"),
    ("RU", "Russia"),
    ("RW", "Rwanda"),
    ("KN", "Saint Kitts and Nevis"),
    ("LC", "Saint Lucia"),
    ("VC", "Saint Vincent and the Grenadines"),
    ("WS", "Samoa"),
    ("SM", "San Marino"),
    ("ST", "Sao Tome and Principe"),
    ("SA", "Saudi Arabia"),
    ("SN", "Senegal"),
    ("RS", "Serbia"),
    ("SC", "Seychelles"),
    ("SL", "Sierra Leone"),
    ("SK", "Slovakia"),
    ("SI", "Slovenia"),
    ("SB", "Solomon Islands"),
    ("SO", "Somalia"),
    ("ZA", "South Africa"),
    ("SS", "South Sudan"),
    ("ES", "Spain"),
    ("LK", "Sri Lanka"),
    ("SD", "Sudan"),
    ("SR", "Suriname"),
    ("SE", "Sweden"),
    ("CH", "Switzerland"),
    ("SY", "Syria"),
    ("TJ", "Tajikistan"),
    ("TZ", "Tanzania"),
    ("TL", "Timor-Leste"),
    ("TG", "Togo"),
    ("TO", "Tonga"),
    ("TT", "Trinidad and Tobago"),
    ("TN", "Tunisia"),
    ("TR", "Turkey"),
    ("TM", "Turkmenistan"),
    ("TV", "Tuvalu"),
    ("UG", "Uganda"),
    ("UA", "Ukraine"),
    ("AE", "United Arab Emirates"),
    ("UY", "Uruguay"),
    ("UZ", "Uzbekistan"),
    ("VU", "Vanuatu"),
    ("VA", "Vatican City"),
    ("VE", "Venezuela"),
    ("YE", "Yemen"),
    ("ZM", "Zambia"),
    ("ZW", "Zimbabwe"),
];

fn country_list() -> Value {
    let mut out: Vec<Value> = Vec::with_capacity(COUNTRY_NAMES.len() + 1);
    out.push(serde_json::json!({ "code": "", "name": "Auto (Detected)" }));
    for (code, name) in COUNTRY_NAMES {
        out.push(serde_json::json!({ "code": code, "name": name }));
    }
    Value::Array(out)
}

fn country_name(code: &str) -> String {
    country_list()
        .as_array()
        .and_then(|a| a.iter().find(|c| jstr(c, "code") == code))
        .map(|c| jstr(c, "name"))
        .filter(|n| !n.is_empty() && n != "Auto (Detected)")
        .unwrap_or_else(|| code.to_string())
}

/// Port of `fd_detect_country()` (backend.php:12139).
///
/// Returns the OBJECT `{country_code, country_name, source}`, not a bare code:
/// the dashboard reads `res.country.country_code` / `.country_name`, so a string
/// made the "Auto (…)" label read `undefined` and left detection looking broken.
///
/// Order is the PHP's: the user's configured country wins, then Cloudflare's
/// `cf-ipcountry` (or `x-country-code`), and when neither exists — which is
/// EVERY localhost/LAN/Android install, since there is no Cloudflare in front —
/// it falls back to `MY` with `source: "default"` instead of an empty string.
fn detect_country(ctx: &ReqCtx) -> Value {
    let settings = load_catalog_settings_merged();
    let configured = jstr(&settings, "country").trim().to_ascii_uppercase();
    let (mut code, mut source) = if !configured.is_empty() {
        (configured, "configured")
    } else {
        let mut found = String::new();
        for h in ["cf-ipcountry", "x-country-code"] {
            let v = ctx.hdr(h).trim().to_ascii_uppercase();
            if !v.is_empty() {
                found = v;
                break;
            }
        }
        (found, "cf-header")
    };
    // `XX` = Cloudflare "unknown", `T1` = Tor: both mean "not a real country".
    if code.len() != 2 || !code.chars().all(|c| c.is_ascii_alphabetic()) || code == "XX" || code == "T1" {
        code = "MY".to_string();
        source = "default";
    }
    serde_json::json!({
        "country_code": code,
        "country_name": country_name(&code),
        "source": source,
    })
}

fn format_bytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut v = bytes as f64;
    let mut i = 0usize;
    while v >= 1024.0 && i < units.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if v >= 10.0 || i == 0 {
        format!("{:.0} {}", v, units[i])
    } else {
        format!("{:.1} {}", v, units[i])
    }
}

/// Minimal descriptor; the PHP's catalog builder is not ported (gap list).
/* ── catalog / meta builders (backend.php:11626 / 12082) ──────────────────── */

/// WordPress category slug per catalog id (backend.php:15869-15893, verbatim —
/// including the `filipino` alias for the Philippines rows and the
/// `pm_series_india` → `bollywood` alias the origin uses for one India bucket).
const CATALOG_CATEGORY: &[(&str, &str)] = &[
    ("pm_movies_malay", "malay"),
    ("pm_movies_indo", "indonesian"),
    ("pm_movies_korean", "korea"),
    ("pm_movies_japan", "japan"),
    ("pm_movies_anime", "anime"),
    ("pm_movies_chinese", "china"),
    ("pm_movies_thai", "thai"),
    ("pm_movies_bollywood", "bollywood"),
    ("pm_movies_philippines", "filipino"),
    ("pm_movies_pinoy", "filipino"),
    ("pm_movies_english", "english"),
    ("pm_series_malay", "malay"),
    ("pm_series_indo", "indonesian"),
    ("pm_series_kdrama", "korea"),
    ("pm_series_anime", "anime"),
    ("pm_series_japan", "japan"),
    ("pm_series_cdrama", "china"),
    ("pm_series_thai", "thai"),
    ("pm_series_philippines", "filipino"),
    ("pm_series_pinoy", "filipino"),
    ("pm_series_india", "bollywood"),
    ("pm_series_english", "english"),
];

/// Country slug → ISO-3166 code, for a `pm_<type>_<country>` id the category
/// map above does not cover. The WordPress `/posts` handler accepts the same
/// `country=<CC>` parameter the PHP passes for its detected-region `top` rows
/// (backend.php:15898-15901), so an id family the dashboard emits later
/// (`pm_movies_canada`, `pm_series_vietnam`, …) resolves without a code change.
const COUNTRY_SLUG_CC: &[(&str, &str)] = &[
    ("malay", "MY"),
    ("malaysia", "MY"),
    ("my", "MY"),
    ("indo", "ID"),
    ("indonesian", "ID"),
    ("indonesia", "ID"),
    ("id", "ID"),
    ("korean", "KR"),
    ("korea", "KR"),
    ("kdrama", "KR"),
    ("kr", "KR"),
    ("japan", "JP"),
    ("japanese", "JP"),
    ("jdrama", "JP"),
    ("jp", "JP"),
    ("chinese", "CN"),
    ("china", "CN"),
    ("cdrama", "CN"),
    ("cn", "CN"),
    ("hongkong", "HK"),
    ("hk", "HK"),
    ("thai", "TH"),
    ("thailand", "TH"),
    ("th", "TH"),
    ("philippines", "PH"),
    ("filipino", "PH"),
    ("pinoy", "PH"),
    ("ph", "PH"),
    ("bollywood", "IN"),
    ("india", "IN"),
    ("indian", "IN"),
    ("hindi", "IN"),
    ("in", "IN"),
    ("english", "US"),
    ("hollywood", "US"),
    ("usa", "US"),
    ("us", "US"),
    ("vietnam", "VN"),
    ("vn", "VN"),
    ("singapore", "SG"),
    ("sg", "SG"),
    ("taiwan", "TW"),
    ("tw", "TW"),
    ("australia", "AU"),
    ("au", "AU"),
    ("germany", "DE"),
    ("de", "DE"),
    ("netherlands", "NL"),
    ("nl", "NL"),
    ("france", "FR"),
    ("fr", "FR"),
    ("canada", "CA"),
    ("ca", "CA"),
];

/// `pm_movies_malay_2024` → `pm_movies_malay`, `pm_series_malay_year` →
/// `pm_series_malay` (backend.php:15865 `$baseCatId`). The year itself still
/// arrives through the `year`/`genre` extra, so only the id is normalised.
fn base_catalog_id(id: &str) -> &str {
    for suffix in ["_latest", "_year", "_genre", "_older"] {
        if let Some(base) = id.strip_suffix(suffix) {
            return base;
        }
    }
    if let Some((head, tail)) = id.rsplit_once('_') {
        if tail.len() == 4 && tail.chars().all(|c| c.is_ascii_digit()) {
            return head;
        }
    }
    id
}

fn country_code_for_slug(slug: &str) -> Option<&'static str> {
    let s = slug.trim().to_ascii_lowercase();
    COUNTRY_SLUG_CC
        .iter()
        .find(|(k, _)| *k == s.as_str())
        .map(|(_, cc)| *cc)
}

/// `pm_<type>_<country>` → ISO code, when the category map has no entry.
/// `pm_cat_*` / `pm_topkw_*` / `*_top` never match, so this can never shadow a
/// row that already worked.
fn catalog_country_code(cat_id: &str) -> Option<&'static str> {
    let rest = cat_id.strip_prefix("pm_")?;
    let (_type_word, slug) = rest.split_once('_')?;
    if slug.is_empty() {
        return None;
    }
    country_code_for_slug(slug)
}

/// The `<extra>` path segment EXACTLY as it arrived (still percent-encoded).
///
/// [`split_addon`] decodes it, which is right for the local catalog parser and
/// wrong for forwarding: the upstream addon expects the encoding it published.
fn raw_addon_extra(path: &str, prefix: &str) -> String {
    let Some(rest) = path.strip_prefix(prefix) else {
        return String::new();
    };
    let rest = rest.strip_suffix(".json").unwrap_or(rest);
    let mut it = rest.splitn(3, '/');
    let _item_type = it.next();
    let _id = it.next();
    it.next().unwrap_or("").to_string()
}

/// `/catalog/<type>/<id>[/<extra>].json`
///
/// Every segment is percent-decoded. The web UI (and Stremio clients) send the
/// id encoded (`pm%3Apost%3A9000016669`), and an undecoded `pm%3Apost%3A…` never
/// matches `pm:post:` — the meta router then answered `{"meta":{}}`, so a series
/// page rendered its title from the catalog row but "No episodes found." and
/// "No description.".
fn split_addon(path: &str, prefix: &str) -> Option<(String, String, String)> {
    let rest = path.strip_prefix(prefix)?;
    let rest = rest.strip_suffix(".json").unwrap_or(rest);
    let mut it = rest.splitn(3, '/');
    let a = crate::download::percent_decode(it.next()?);
    let b = crate::download::percent_decode(it.next()?);
    let c = crate::download::percent_decode(it.next().unwrap_or(""));
    Some((a, b, c))
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn post_id(post: &Value) -> String {
    post.get("id")
        .map(|v| {
            v.as_u64()
                .map(|n| n.to_string())
                .unwrap_or_else(|| v.as_str().unwrap_or("").to_string())
        })
        .unwrap_or_default()
}

/// `fd_clean_post_title()`: drop the "• Movie" / "| TvSeries" decoration.
fn clean_post_title(raw: &str) -> String {
    let decoded = crate::matcher::decode_html_entities(raw);
    let re = fancy_regex::Regex::new(
        "(?i)\\s*[•|·]\\s*(?:movie|tv\\s*series|tvseries|series|drama|episode|bahagian)\\s*$",
    );
    let stripped = match re {
        Ok(re) => re.replace_all(decoded.trim(), "").into_owned(),
        Err(_) => decoded.trim().to_string(),
    };
    collapse_ws(&stripped)
}

/// `fd_clean_post_plot()`: strip HTML, decode entities, cap the length.
fn clean_post_plot(raw: &str) -> String {
    let no_tags = match fancy_regex::Regex::new("<[^>]+>") {
        Ok(re) => re.replace_all(raw, " ").into_owned(),
        Err(_) => raw.to_string(),
    };
    let text = collapse_ws(&crate::matcher::decode_html_entities(&no_tags));
    text.chars().take(400).collect()
}

fn post_genres(post: &Value) -> Vec<String> {
    post.get("categories")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|c| c.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn post_release_year(post: &Value) -> String {
    for key in ["year", "release_year"] {
        if let Some(v) = post.get(key) {
            let s = v
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| v.to_string());
            if s.len() >= 4 && s[..4].chars().all(|c| c.is_ascii_digit()) {
                return s[..4].to_string();
            }
        }
    }
    let hay = format!("{} {}", jstr(post, "title"), jstr(post, "date"));
    if let Ok(re) = fancy_regex::Regex::new("\\b(19\\d{2}|20\\d{2})\\b") {
        if let Ok(Some(m)) = re.captures(&hay) {
            if let Some(g) = m.get(1) {
                return g.as_str().to_string();
            }
        }
    }
    String::new()
}

fn post_is_series(post: &Value) -> bool {
    let hay = format!(
        "{} {}",
        jstr(post, "title"),
        post_genres(post).join(" ")
    );
    fancy_regex::Regex::new("(?i)tvseries|tv\\s*series|\\bseries\\b|season|episode|drama")
        .map(|re| re.is_match(&hay).unwrap_or(false))
        .unwrap_or(false)
}

/// How many post-files prefetches may run at once.
///
/// The Kotlin app landed on the same bound after measuring that a catalog page's 24
/// simultaneous `post_files` fetches were themselves what tripped the origin's rate
/// limiter (`[Resolve] upstream rate limit …` and `[PostFiles] page 0 failed: timeout`
/// in the same second). Two at a time warms a page without competing with the user.
static POST_PREFETCH_INFLIGHT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
const MAX_POST_PREFETCH_INFLIGHT: usize = 2;

/// Kotlin `prefetchPostFiles()`: warm ONE post's file set off the critical path.
///
/// The catalog/meta warm caches the META but not the file set the stream route
/// reads, so the first Play of a title just browsed still paid a cold origin fetch
/// (measured in the Android app: 10.4 s cold → 2.2 s warm). Bounds, all three of
/// which matter:
///   1. skipped while a download is streaming, so it can never compete with live
///      playback for Telegram/origin bandwidth;
///   2. skipped while the origin is refusing resolves (`resolve_cooldown.marker`);
///   3. at most one warm per post per 300 s, checked BEFORE the mark is written so a
///      suppressed call can still warm later.
fn prefetch_post_files(post_id: &str) {
    let post_id = post_id.trim();
    if post_id.is_empty() || post_id == "0" {
        return;
    }
    if crate::download::active_streams() > 0 {
        log("post files prefetch skipped (stream active)", &serde_json::json!({"post": post_id}));
        return;
    }
    if crate::resolve::resolve_cooldown_remaining_sec() > 0 {
        log("post files prefetch skipped (resolve cooldown)", &serde_json::json!({"post": post_id}));
        return;
    }
    // A cache entry, not a 0-byte marker file: reading a marker cost two syscalls
    // (~200 us) on every info page, and expiry is now the entry's own deadline.
    if crate::cache::get("post_prefetch", post_id).is_some() {
        return;
    }
    crate::cache::put("post_prefetch", post_id, &serde_json::json!(1), 300, 0);
    let pid = post_id.to_string();
    tokio::spawn(async move {
        // Bounded wait for a slot rather than an unbounded spawn storm.
        let mut waited = 0u32;
        while POST_PREFETCH_INFLIGHT.load(std::sync::atomic::Ordering::SeqCst)
            >= MAX_POST_PREFETCH_INFLIGHT
        {
            if waited >= 100 {
                return;
            }
            waited += 1;
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        POST_PREFETCH_INFLIGHT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let files = crate::wp_data::fetch_post_files_paged(&pid, 200, 600, 3).await;
        POST_PREFETCH_INFLIGHT.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        log(
            "post files prefetched",
            &serde_json::json!({"post": pid, "files": files.len()}),
        );
    });
}

/// `buildPostMeta()` — the Stremio meta for one catalog post.
fn meta_from_post(post: &Value, item_type: &str) -> Value {
    let pid = post_id(post);
    let plot = {
        let e = jstr(post, "excerpt");
        if e.is_empty() {
            jstr(post, "content")
        } else {
            e
        }
    };
    let mut m = serde_json::json!({
        "id": format!("pm:post:{}", pid),
        "type": if item_type == "series" { "series" } else { "movie" },
        "name": clean_post_title(&jstr(post, "title")),
        "poster": jstr(post, "thumbnail_url"),
        "posterShape": "poster",
        "description": clean_post_plot(&plot),
        "genres": post_genres(post),
    });
    let year = post_release_year(post);
    if !year.is_empty() {
        m["releaseInfo"] = serde_json::json!(year);
    }
    m
}

/// The two merged trending rows advertised in the descriptor.
const TRENDING_MOVIE_CATALOG: &str = "pm_trending_movies";
const TRENDING_SERIES_CATALOG: &str = "pm_trending_series";

/// How many entries one trending row keeps. The PHP's per-keyword file catalogs
/// fetch 50; a merged post row is capped lower so the row stays browsable.
const TRENDING_ROW_MAX: usize = 40;
/// Keywords searched concurrently per round. `search` is a Manticore query, so a
/// large fan-out competes with the origin; two rounds of five fill a 40-entry
/// row for both types.
const TRENDING_BATCH: usize = 5;

/// One entry of a trending row, appended only when it belongs to this type.
/// `seen` dedupes by post id ACROSS keywords (the same title trends twice as
/// often as not) and never resets, so a later page cannot repeat an earlier one.
fn push_trending_metas(
    metas: &mut Vec<Value>,
    seen: &mut std::collections::HashSet<String>,
    posts: &[Value],
    item_type: &str,
    target: usize,
) {
    let want_series = item_type == "series";
    for post in posts {
        if metas.len() >= target {
            return;
        }
        let pid = post_id(post);
        if pid.is_empty() || !seen.insert(pid) {
            continue;
        }
        if post_is_series(post) != want_series {
            continue;
        }
        metas.push(meta_from_post(post, item_type));
    }
}

/// `pm_trending_movies` / `pm_trending_series`.
///
/// For every trending keyword the post search is asked once (30 posts), and only
/// the entries whose movie/series classification matches the row are kept. The
/// PHP turns each keyword into its own `pm_topkw_*` **file** catalog; this row
/// merges the keywords into one post row per type, so both depend on the same
/// keyword map ([`crate::wp_data::trending_keywords`]).
///
/// Keywords are searched a batch at a time so a cold request costs one round
/// trip per batch, not one per keyword, and the loop stops as soon as the row is
/// full.
async fn build_trending_row(item_type: &str, skip: usize) -> Vec<Value> {
    let keywords = crate::wp_data::trending_keywords().await;
    if keywords.is_empty() {
        return Vec::new();
    }
    let target = TRENDING_ROW_MAX + skip.min(400);
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut metas: Vec<Value> = Vec::new();

    for chunk in keywords.chunks(TRENDING_BATCH) {
        if metas.len() >= target {
            break;
        }
        let futures = chunk.iter().map(|(kw, _)| {
            let kw = kw.clone();
            async move {
                let params = vec![
                    ("limit".to_string(), "30".to_string()),
                    ("offset".to_string(), "0".to_string()),
                    ("search".to_string(), kw),
                ];
                crate::wp_data::search_posts(&params).await
            }
        });
        // `join_all` keeps the keyword order, which is the origin's relevance
        // order — the same order the PHP's map preserves.
        for posts in futures_util::future::join_all(futures).await {
            push_trending_metas(&mut metas, &mut seen, &posts, item_type, target);
            if metas.len() >= target {
                break;
            }
        }
    }

    let pool = metas.len();
    let page: Vec<Value> = metas
        .into_iter()
        .skip(skip)
        .take(TRENDING_ROW_MAX)
        .collect();
    log(
        "trending catalog built",
        &serde_json::json!({
            "type": item_type,
            "keywords": keywords.len(),
            "pool": pool,
            "skip": skip,
            "count": page.len(),
        }),
    );
    page
}

/// The `\b`-bounded patterns the PHP uses for pill detection
/// (backend.php:15793-15801 catalog rows, 16129-16143 direct-file meta).
///
/// The boundary is load-bearing: the PHP matches a bare `\bhd\b` / `\bsd\b`, so
/// `Movie.720p.HD.mkv` gets a resolution pill while `HDU` does not. This port
/// used plain `contains()` and dropped `hd`/`sd` altogether, which lost the pill
/// on every `…HD…` release. Compiled once (a per-file compile would dominate a
/// 100-item catalog).
struct PillRes {
    p4k: fancy_regex::Regex,
    p1080: fancy_regex::Regex,
    p720: fancy_regex::Regex,
    psd: fancy_regex::Regex,
    pblu: fancy_regex::Regex,
    pweb: fancy_regex::Regex,
    phdtv: fancy_regex::Regex,
    phdr: fancy_regex::Regex,
    phevc: fancy_regex::Regex,
    pavc: fancy_regex::Regex,
}

static PILL_RES: std::sync::LazyLock<PillRes> = std::sync::LazyLock::new(|| PillRes {
    p4k: fancy_regex::Regex::new(r"(?i)\b(2160p|4k|uhd)\b").unwrap(),
    p1080: fancy_regex::Regex::new(r"(?i)\b(1080p|fhd)\b").unwrap(),
    p720: fancy_regex::Regex::new(r"(?i)\b(720p|hd)\b").unwrap(),
    psd: fancy_regex::Regex::new(r"(?i)\b(480p|360p|sd)\b").unwrap(),
    pblu: fancy_regex::Regex::new(r"(?i)\b(bluray|blu-ray|remux)\b").unwrap(),
    pweb: fancy_regex::Regex::new(r"(?i)\b(web-?dl|webrip)\b").unwrap(),
    phdtv: fancy_regex::Regex::new(r"(?i)\b(hdtv|tvrip)\b").unwrap(),
    phdr: fancy_regex::Regex::new(r"(?i)\b(hdr10\+|hdr10|hdr|dolby\s*vision|dovi|dv)\b").unwrap(),
    phevc: fancy_regex::Regex::new(r"(?i)\b(hevc|x265|h265)\b").unwrap(),
    pavc: fancy_regex::Regex::new(r"(?i)\b(avc|x264|h264)\b").unwrap(),
});

fn pill_hit(re: &fancy_regex::Regex, text: &str) -> bool {
    re.is_match(text).unwrap_or(false)
}

/// Resolution / source / codec pills for a Telegram-FILE catalog row
/// (backend.php:15793-15803).
fn file_pills(title: &str, size: u64) -> Vec<String> {
    let r = &*PILL_RES;
    let mut pills: Vec<String> = Vec::new();
    if pill_hit(&r.p4k, title) {
        pills.push("4K".to_string());
    } else if pill_hit(&r.p1080, title) {
        pills.push("1080p".to_string());
    } else if pill_hit(&r.p720, title) {
        pills.push("720p".to_string());
    } else if pill_hit(&r.psd, title) {
        pills.push("SD".to_string());
    }
    if pill_hit(&r.pblu, title) {
        pills.push("BluRay".to_string());
    } else if pill_hit(&r.pweb, title) {
        pills.push("WEB-DL".to_string());
    }
    if pill_hit(&r.phevc, title) {
        pills.push("HEVC".to_string());
    }
    if size > 0 {
        pills.push(format_bytes(size));
    }
    pills
}

/// The direct-file META's pills (backend.php:16129-16143). These differ from the
/// catalog row's on purpose: `4K UHD`, `HDTV`, `HDR` and `AVC` are extra, and the
/// caller joins them with `  •  `.
fn file_meta_pills(title: &str, size: u64) -> Vec<String> {
    let r = &*PILL_RES;
    let mut pills: Vec<String> = Vec::new();
    if pill_hit(&r.p4k, title) {
        pills.push("4K UHD".to_string());
    } else if pill_hit(&r.p1080, title) {
        pills.push("1080p".to_string());
    } else if pill_hit(&r.p720, title) {
        pills.push("720p".to_string());
    } else if pill_hit(&r.psd, title) {
        pills.push("SD".to_string());
    }
    if pill_hit(&r.pblu, title) {
        pills.push("BluRay".to_string());
    } else if pill_hit(&r.pweb, title) {
        pills.push("WEB-DL".to_string());
    } else if pill_hit(&r.phdtv, title) {
        pills.push("HDTV".to_string());
    }
    if pill_hit(&r.phdr, title) {
        pills.push("HDR".to_string());
    }
    if pill_hit(&r.phevc, title) {
        pills.push("HEVC".to_string());
    } else if pill_hit(&r.pavc, title) {
        pills.push("AVC".to_string());
    }
    if size > 0 {
        pills.push(format_bytes(size));
    }
    pills
}

/// `date('Y-m-d\TH:i:s\Z', $secs)` in UTC, without a date crate.
fn iso_from_unix(secs: i64) -> String {
    let days = secs.div_euclid(86400);
    let rem = secs.rem_euclid(86400);
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let mut y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    if m <= 2 {
        y += 1;
    }
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m,
        d,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// A WordPress `date` (`2024-02-19 20:01:04`) as RFC3339 UTC.
///
/// The origin sends site-local time with no zone; it is read here at day
/// granularity for a Stremio `released`, so the value is emitted as `Z` rather
/// than guessed into an offset. `None` when the string does not start with a
/// real date.
fn iso_from_wp_date(raw: &str) -> Option<String> {
    let raw = raw.trim();
    let b = raw.as_bytes();
    if b.len() < 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let y: i32 = raw.get(0..4)?.parse().ok()?;
    let mo: u32 = raw.get(5..7)?.parse().ok()?;
    let d: u32 = raw.get(8..10)?.parse().ok()?;
    if !(1900..=2999).contains(&y) || !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        return None;
    }
    // `[ T]HH:MM:SS` when a time follows the date, midnight otherwise.
    let (h, mi, s) = match raw.get(11..19) {
        Some(t) if t.len() == 8 && t.as_bytes()[2] == b':' && t.as_bytes()[5] == b':' => (
            t.get(0..2)?.parse::<u32>().ok()?,
            t.get(3..5)?.parse::<u32>().ok()?,
            t.get(6..8)?.parse::<u32>().ok()?,
        ),
        _ => (0, 0, 0),
    };
    if h > 23 || mi > 59 || s > 59 {
        return None;
    }
    Some(format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z"))
}

/// The series-level date every episode falls back to when its own file row has
/// no `added_date`: the POST's publish date, else `{release year}-01-01`.
///
/// `None` when the post carries neither — the caller then emits `null`, because
/// a fabricated "now" is worse than an absent value: clients sort on it.
fn series_released_anchor(post: &Value) -> Option<String> {
    if let Some(date) = iso_from_wp_date(&jstr(post, "date")) {
        return Some(date);
    }
    let year = post_release_year(post);
    if year.len() == 4 && year.chars().all(|c| c.is_ascii_digit()) {
        return Some(format!("{year}-01-01T00:00:00Z"));
    }
    None
}

/// The Stremio `videos` list for one `pm:post:` series — one entry per distinct
/// `(season, episode)` the post's own files carry.
///
/// Two rules, both regressions the port introduced and issue #10 reported:
///
///   1. `released` is the FILE's own `added_date` — a real stored date. It must
///      never be the current clock: with every sort key equal, a client that
///      sorts by release (or badges "new episodes") sees nonsense, and the
///      `/meta` body differs on every request. A row without one falls back to
///      the series anchor, and to `null` only when the post has no date at all.
///   2. the list is sorted by `(season, episode)` ASCENDING. The PHP does this
///      explicitly (backend.php:16339-16348, with `raw_index` as a tiebreak);
///      the port kept the raw scan order, so the episode list rendered — and
///      next-episode/autoplay advanced — out of order.
fn series_videos(
    media_id: &str,
    files: &[crate::wp_data::WpFile],
    poster: &Value,
    anchor: Option<&str>,
) -> Vec<Value> {
    let mut seen = std::collections::BTreeSet::new();
    let mut videos: Vec<Value> = Vec::new();
    for f in files {
        let (s, e, _end) = crate::matcher::classify_season_episode(
            &f.title,
            f.season_num,
            f.episode_num,
            &f.caption,
        );
        if e <= 0 || !seen.insert((s, e)) {
            continue;
        }
        // The PHP's episode label is `S{s}E{e}` with NO zero padding.
        let thumb = if f.thumbnail.is_empty() {
            poster.clone()
        } else {
            serde_json::json!(f.thumbnail)
        };
        let released = if f.added_date > 0 {
            serde_json::json!(iso_from_unix(f.added_date))
        } else {
            anchor.map(|a| serde_json::json!(a)).unwrap_or(Value::Null)
        };
        videos.push(serde_json::json!({
            "id": format!("{}:{}:{}", media_id, s, e),
            "name": format!("S{}E{}", s, e),
            "season": s,
            "episode": e,
            "number": e,
            "released": released,
            "thumbnail": thumb,
        }));
    }
    // `seen` already guarantees a unique (s, e) pair, so this is a total order
    // and needs no tiebreak.
    videos.sort_by_key(|v| {
        (
            v["season"].as_i64().unwrap_or(0),
            v["episode"].as_i64().unwrap_or(0),
        )
    });
    videos
}

/// The `(season, episode)` pairs a series' file rows do NOT cover, per season.
///
/// A post is a slice of the show, not the whole show: `pm:post:9000018021`
/// ("To My Beloved Thief 2026") owns E03–E14 and E16, while the origin index
/// also holds E01, E02 and E15. `/stream` answers those because its ladder
/// searches the index; the episode PICKER is `meta.videos`, built from the post
/// — so the episodes were unplayable.
///
/// `fd_fetch_series_episode_files()` (backend.php:10398) computes exactly this
/// set, but its early return tested only `$hasExplicitEp`, never `$hasGaps`, so
/// the PHP never backfilled either. This is the divergence that makes the list
/// agree with what `/stream` can actually answer.
fn episode_gaps(files: &[crate::wp_data::WpFile]) -> std::collections::BTreeSet<(i32, i32)> {
    use std::collections::{BTreeSet};

    let mut present: BTreeSet<(i32, i32)> = BTreeSet::new();
    for f in files {
        let (s, e, _) = crate::matcher::classify_season_episode(
            &f.title,
            f.season_num,
            f.episode_num,
            &f.caption,
        );
        if e > 0 {
            present.insert((s.max(1), e));
        }
    }
    let mut gaps: BTreeSet<(i32, i32)> = BTreeSet::new();
    let seasons: BTreeSet<i32> = present.iter().map(|(s, _)| *s).collect();
    for s in seasons {
        let max_ep = present
            .iter()
            .filter(|(ps, _)| *ps == s)
            .map(|(_, e)| *e)
            .max()
            .unwrap_or(0);
        for e in 1..=max_ep {
            if !present.contains(&(s, e)) {
                gaps.insert((s, e));
            }
        }
    }
    gaps
}

/// Episode entries for the gaps [`episode_gaps()`] found, resolved against the
/// origin index with the SAME strict matcher `/stream` uses, so the picker can
/// never advertise an episode the player cannot answer.
///
/// Costs one index search, and only for a series whose numbering has a hole.
/// The outcome — including "nothing to fill" — is cached for 10 min: a series
/// like "The Witch 2025" has E11–E15 missing everywhere, and must not pay an
/// origin search on every `/meta` request.
async fn gap_episode_videos(
    media_id: &str,
    post_id: &str,
    title: &str,
    year: &str,
    files: &[crate::wp_data::WpFile],
    poster: &Value,
    anchor: Option<&str>,
) -> Vec<Value> {
    let gaps = episode_gaps(files);
    if gaps.is_empty() || title.trim().is_empty() {
        return Vec::new();
    }

    let ckey = format!("{post_id}|{title}|{year}");
    if let Some(hit) = crate::cache::get("series_gap", &ckey) {
        if let Some(list) = hit.get("videos").and_then(|v| v.as_array()) {
            return list.clone();
        }
    }

    let keyword = stream_keyword_from_post_title(title);
    let mut found: std::collections::BTreeMap<(i32, i32), crate::wp_data::WpFile> =
        std::collections::BTreeMap::new();
    if !keyword.is_empty() {
        // Two variants at most: the plain keyword, then its possessive/accent
        // form. More than that spends the origin's time for a handful of rows.
        for kw in crate::matcher::stream_keyword_variants(&keyword)
            .into_iter()
            .take(2)
        {
            for row in crate::wp_data::search_files(&kw, 1000).await {
                let (s, e, _) = crate::matcher::classify_season_episode(
                    &row.title,
                    row.season_num,
                    row.episode_num,
                    &row.caption,
                );
                let key = (s.max(1), e);
                if e <= 0 || !gaps.contains(&key) || found.contains_key(&key) {
                    continue;
                }
                if !crate::matcher::series_file_matches_title(
                    &row.title,
                    title,
                    key.0,
                    key.1,
                    year,
                    &row.caption,
                    &[],
                ) {
                    continue;
                }
                found.insert(key, row);
            }
            if found.len() == gaps.len() {
                break;
            }
        }
    }

    let extra: Vec<crate::wp_data::WpFile> = found.into_values().collect();
    let videos = if extra.is_empty() {
        Vec::new()
    } else {
        series_videos(media_id, &extra, poster, anchor)
    };
    log(
        "meta: episode gaps backfilled from the index",
        &serde_json::json!({
            "post_id": post_id,
            "gaps": gaps
                .iter()
                .map(|(s, e)| format!("S{}E{}", s, e))
                .collect::<Vec<_>>(),
            "filled": videos.len(),
        }),
    );
    crate::cache::put(
        "series_gap",
        &ckey,
        &serde_json::json!({ "videos": videos }),
        600,
        0,
    );
    videos
}

/// `other/*` and `pm_topkw_*` rows: the origin's newest / keyword FILE lists,
/// rendered exactly like the PHP's Telegram-file catalog items
/// (backend.php:15750-15850).
///
/// The search term is, in order: the row's own `search` extra (the PHP ignores it
/// on `pm_search_files`, which made that row useless), the selected year, the
/// `pm_topkw_<hash>` keyword, else `__latest__` (the origin's "newest files"
/// token).
async fn file_catalog_metas(
    cat_id: &str,
    search: &str,
    genre: &str,
    year: &str,
    skip: usize,
) -> Vec<Value> {
    let term = if !search.trim().is_empty() {
        search.trim().to_string()
    } else if !year.trim().is_empty() {
        year.trim().to_string()
    } else if cat_id.starts_with("pm_topkw_") {
        // A keyword row whose keyword has fallen out of the cached map renders
        // EMPTY, like the PHP — never the newest files, which would present
        // unrelated releases under a trending name.
        match topkw_map().await.into_iter().find(|(id, _)| id == cat_id) {
            Some((_, kw)) => kw,
            None => return Vec::new(),
        }
    } else {
        "__latest__".to_string()
    };
    let limit = if genre.is_empty() && year.is_empty() { 50 } else { 100 };
    let files = crate::wp_data::search_files_at(&term, limit, skip).await;
    let mut metas: Vec<Value> = Vec::new();
    for f in &files {
        // The PHP's file title is the raw name with HTML entities decoded
        // (backend.php:15788), NOT `fd_clean_post_title()` — that would strip a
        // trailing "• Movie" from a real filename and collapse its spacing.
        let decoded = crate::matcher::decode_html_entities(f.title.trim());
        let title = if decoded.is_empty() {
            "Telegram File".to_string()
        } else {
            decoded
        };
        let pills = file_pills(&title, f.file_size);
        let pill_line = if pills.is_empty() {
            "Ready to stream".to_string()
        } else {
            pills.join(" · ")
        };
        // `$primaryLabel` (backend.php:15806): 'Year' on the year rows,
        // 'Trending File' on EVERY other Telegram-Files row — including
        // `other/top` and `pm_search_files`.
        let primary = if cat_id == "pm_files_year" || cat_id == "pm_files_latest" {
            "Year"
        } else {
            "Trending File"
        };
        let mut genres: Vec<String> = vec![primary.to_string()];
        for p in &pills {
            if !genres.iter().any(|g| g == p) {
                genres.push(p.clone());
            }
        }
        // On a file row the `genre` extra is a QUALITY pill; the PHP compares it
        // EXACTLY against the genre list it just built (backend.php:15810).
        if !genre.is_empty() && !genres.iter().any(|g| g == genre.trim()) {
            continue;
        }
        if !year.is_empty() && !title.contains(year.trim()) {
            continue;
        }
        // `{$descriptionPrefix} · {$pillLine}\n\n{$fTitle}` (backend.php:15819-15831),
        // with the PHP's calendar/lightning glyphs dropped: plain text labels.
        let prefix = if cat_id == "pm_files_year" || cat_id == "pm_files_latest" {
            "File"
        } else if cat_id.starts_with("pm_topkw_") {
            "Trending File"
        } else {
            "Direct Telegram File"
        };
        metas.push(serde_json::json!({
            "id": format!("pm_file_{}", f.short_code),
            "type": "other",
            "name": title,
            "poster": f.thumbnail,
            "posterShape": "poster",
            "description": format!("{} · {}\n\n{}", prefix, pill_line, title),
            "genres": genres,
        }));
    }
    metas
}

async fn handle_catalog(ctx: &ReqCtx, path: &str) -> Response {
    let Some((item_type, cat_id, extra)) = split_addon(path, "/catalog/") else {
        return stremio_json(200, &serde_json::json!({"metas": []}), "no-store");
    };
    // ── upstream bridged catalog (`up_<i>_<realId>`) ──────────────────────
    // Forwarded verbatim (backend.php:15497-15537) with a 1 h disk cache, so a
    // row the user browsed repeatedly costs one request. The extra segment is
    // forwarded RAW: it still carries its own percent-encoding, and re-encoding
    // a decoded `genre=Science+Fiction` would corrupt it.
    if let Some((idx, real_id)) = upstream::parse_bridged(&cat_id) {
        let raw_extra = raw_addon_extra(path, "/catalog/");
        let query = ctx
            .query
            .iter()
            .map(|(k, v)| format!("{}={}", urlencode_component(k), urlencode_component(v)))
            .collect::<Vec<_>>()
            .join("&");
        let settings = load_catalog_settings_merged();
        if let Some(body) = upstream::forward_catalog(
            &settings,
            idx,
            &item_type,
            &real_id,
            &raw_extra,
            &query,
        )
        .await
        {
            return stremio_json(200, &body, "max-age=3600, public");
        }
        // A stale `up_*` id (the addon was removed, or the upstream is down):
        // an empty list, never the SPA fallback.
        return stremio_json(200, &serde_json::json!({"metas": []}), "no-store");
    }
    let mut search = String::new();
    let mut genre = String::new();
    let mut year = String::new();
    let mut skip = 0usize;
    for pair in extra.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            match k {
                "search" => search = v.replace('+', " "),
                "genre" => genre = v.replace('+', " "),
                "year" => year = v.to_string(),
                "skip" => skip = v.parse().unwrap_or(0),
                _ => {}
            }
        }
    }
    // A year sub-catalog carries it in the ID (`pm_movies_malay_2024`), and a
    // 4-digit `genre` is really a year (backend.php:15568-15578 / 15928-15938).
    if year.is_empty() {
        if let Some((_, tail)) = cat_id.rsplit_once('_') {
            if tail.len() == 4 && tail.chars().all(|c| c.is_ascii_digit()) {
                year = tail.to_string();
            }
        }
    }
    if year.is_empty() && genre.len() == 4 && genre.chars().all(|c| c.is_ascii_digit()) {
        year = genre.clone();
        genre.clear();
    }
    // `/catalog/*` response cache (the PHP's `cat_cache_`, 10 min). A warm row cost
    // ~190-470 ms to rebuild even with `posts` cached (per-post regex/derivation work
    // over 49-100 posts), and a client asks for every row of the home screen. Only a
    // non-empty list is stored, so an origin outage is never pinned for 10 minutes.
    let cat_key = format!("{}|{}|{}", item_type, cat_id, extra);
    if let Some(hit) = crate::cache::get("catalog", &cat_key) {
        if hit.get("metas").and_then(|m| m.as_array()).map(|a| !a.is_empty()).unwrap_or(false) {
            log(
                "catalog served from cache",
                &serde_json::json!({"type": item_type, "id": cat_id}),
            );
            return stremio_json(200, &hit, "no-store");
        }
    }
    // ── Telegram-FILE rows (`other/*`, `pm_topkw_*`) ───────────────────────
    // backend.php:15750-15850: these catalogs list FILES (`search-files`). With a
    // SEARCH query the PHP also lists matching POSTS first
    // (backend.php:15615-15656), except on `pm_search_files`, which it resets to
    // files only (backend.php:15662).
    if item_type == "other" || cat_id.starts_with("pm_topkw_") {
        let mut metas: Vec<Value> = Vec::new();
        if item_type == "other" && !search.is_empty() && cat_id != "pm_search_files" {
            let params = vec![
                ("search".to_string(), search.clone()),
                ("limit".to_string(), "30".to_string()),
                ("offset".to_string(), skip.to_string()),
            ];
            for post in crate::wp_data::search_posts(&params).await {
                if post_id(&post).is_empty() {
                    continue;
                }
                if !year.is_empty() {
                    let ry = post_release_year(&post);
                    if !ry.is_empty() && ry != year {
                        continue;
                    }
                }
                let ptype = if post_is_series(&post) { "series" } else { "movie" };
                metas.push(meta_from_post(&post, ptype));
            }
        }
        metas.extend(file_catalog_metas(&cat_id, &search, &genre, &year, skip).await);
        let body = serde_json::json!({"metas": metas});
        if body
            .get("metas")
            .and_then(|m| m.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false)
        {
            // A SEARCH result is cached for 300 s, a browse row for 600 s — the
            // PHP's own windows (backend.php:15586 / 15595).
            let ttl = if search.is_empty() { 600 } else { 300 };
            crate::cache::put("catalog", &cat_key, &body, ttl, 0);
        }
        log(
            "file catalog built",
            &serde_json::json!({
                "type": item_type,
                "id": cat_id,
                "search": search,
                "count": body["metas"].as_array().map(|a| a.len()).unwrap_or(0),
            }),
        );
        return stremio_json(200, &body, "no-store");
    }
    let limit = if search.is_empty() { 100 } else { 30 };
    // ── trending rows (merged over the trending keyword map) ───────────────
    // Only the bare row (and its `skip` paging) is trending-derived; a client
    // that adds `search`/`genre` to this id wants the ordinary catalog path.
    if (cat_id == TRENDING_MOVIE_CATALOG || cat_id == TRENDING_SERIES_CATALOG)
        && search.is_empty()
        && genre.is_empty()
        && year.is_empty()
    {
        let row_type = if cat_id == TRENDING_MOVIE_CATALOG { "movie" } else { "series" };
        let metas = build_trending_row(row_type, skip).await;
        return stremio_json(200, &serde_json::json!({"metas": metas}), "no-store");
    }
    let mut params: Vec<(String, String)> = vec![
        ("limit".to_string(), limit.to_string()),
        ("offset".to_string(), skip.to_string()),
    ];
    let posts = if !search.is_empty() {
        params.push(("search".to_string(), search.clone()));
        crate::wp_data::search_posts(&params).await
    } else {
        params.push((
            "media_type".to_string(),
            if item_type == "series" { "series" } else { "movie" }.to_string(),
        ));
        // A year/genre sub-catalog (`pm_movies_malay_2024`) resolves through its
        // base id, exactly like the PHP's `$baseCatId`.
        let base_id = base_catalog_id(&cat_id);
        if let Some((_, slug)) = CATALOG_CATEGORY
            .iter()
            .find(|(id, _)| *id == base_id || *id == cat_id)
        {
            params.push(("category".to_string(), (*slug).to_string()));
        } else if let Some(slug) = cat_id.strip_prefix("pm_cat_") {
            // The dashboard's own genre rows (`pm_cat_action`, `pm_cat_indo`, …).
            let slug = if slug == "indo" { "indonesian" } else { slug };
            params.push(("category".to_string(), slug.to_string()));
        } else if let Some(cc) = catalog_country_code(base_id) {
            // `pm_<type>_<country>` the map above does not name — the origin's
            // own country filter, same parameter its `top` rows use.
            params.push(("country".to_string(), cc.to_string()));
        } else if cat_id == "top" || cat_id.ends_with("_top") {
            if year.is_empty() {
                params.push(("category".to_string(), "popular".to_string()));
            }
        }
        if !genre.is_empty() {
            params.push(("genre".to_string(), genre.clone()));
        }
        if !year.is_empty() {
            params.push(("year".to_string(), year.clone()));
        }
        crate::wp_data::posts(&params).await
    };

    let mut metas: Vec<Value> = Vec::new();
    for post in &posts {
        if post_id(post).is_empty() {
            continue;
        }
        let is_series = post_is_series(post);
        if item_type == "series" && !is_series {
            continue;
        }
        if item_type == "movie" && is_series {
            continue;
        }
        if !year.is_empty() {
            let ry = post_release_year(post);
            if !ry.is_empty() && ry != year {
                continue;
            }
        }
        metas.push(meta_from_post(post, &item_type));
    }
    log(
        "catalog built",
        &serde_json::json!({"type": item_type, "id": cat_id, "search": search, "count": metas.len()}),
    );
    // Warm the POST'S FILE SET for the movies on this page (Kotlin
    // `prefetchMetaFor`'s second loop). A catalog row's Play would otherwise pay a
    // cold `post_files` fetch; each call is bounded (2 at a time, one per post per
    // 300 s, skipped while streaming or rate-limited) so a page of 24 rows queues
    // instead of stampeding the origin.
    if item_type == "movie" {
        for m in metas.iter().take(24) {
            let id = m.get("id").and_then(|v| v.as_str()).unwrap_or("");
            if let Some(pid) = id.strip_prefix("pm:post:") {
                prefetch_post_files(pid);
            }
        }
    }
    let body = serde_json::json!({"metas": metas});
    if body.get("metas").and_then(|m| m.as_array()).map(|a| !a.is_empty()).unwrap_or(false) {
        // The PHP caches a SEARCH result for 300 s and a browse row for 600 s
        // (backend.php:15586 / 15595).
        let ttl = if search.is_empty() { 600 } else { 300 };
        crate::cache::put("catalog", &cat_key, &body, ttl, 0);
    }
    stremio_json(200, &body, "no-store")
}

async fn handle_meta(_ctx: &ReqCtx, path: &str) -> Response {
    let Some((item_type, id, _)) = split_addon(path, "/meta/") else {
        // A malformed `/meta/…` path: the PHP's route regex does not match it
        // either, so it never reaches the meta builder.
        return not_found_json();
    };
    // A catalog post: details, plus a `videos` list for a series (one entry per
    // distinct S/E that the post's own files carry).
    if let Some(pid) = id.strip_prefix("pm:post:") {
        let post = crate::wp_data::get_post(pid).await;
        let mut meta = meta_from_post(&post, &item_type);
        if meta["name"].as_str().unwrap_or("").is_empty() {
            let row = crate::wp_data::lookup_id("post", pid).await;
            let title = jstr(&row, "title");
            if !title.is_empty() {
                meta["name"] = serde_json::json!(clean_post_title(&title));
                let year = jstr(&row, "year");
                if !year.is_empty() {
                    meta["releaseInfo"] = serde_json::json!(year);
                }
            }
        }
        if item_type == "series" {
            let files = crate::wp_data::fetch_post_files_paged(pid, 200, 600, 3).await;
            let anchor = series_released_anchor(&post);
            let mut videos = series_videos(&id, &files, &meta["poster"], anchor.as_deref());
            // The post is not always the whole show: episodes whose files live
            // only in the origin index are answerable by `/stream` but were
            // absent from the picker. See `gap_episode_videos()`.
            let series_title = meta["name"].as_str().unwrap_or("").to_string();
            let series_year = meta
                .get("releaseInfo")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            videos.extend(
                gap_episode_videos(
                    &id,
                    pid,
                    &series_title,
                    &series_year,
                    &files,
                    &meta["poster"],
                    anchor.as_deref(),
                )
                .await,
            );
            // The merge must not disturb the (season, episode) order the rest of
            // this builder guarantees.
            videos.sort_by_key(|v| {
                (
                    v["season"].as_i64().unwrap_or(0),
                    v["episode"].as_i64().unwrap_or(0),
                )
            });
            // A series with no resolvable episode still needs ONE entry, or
            // Stremio refuses the meta entirely (backend.php:16357-16368).
            if videos.is_empty() {
                videos.push(serde_json::json!({
                    "id": format!("{}:1:1", id),
                    "name": "Episode 1",
                    "season": 1,
                    "episode": 1,
                    "number": 1,
                    "released": anchor.map(|a| serde_json::json!(a)).unwrap_or(Value::Null),
                    "thumbnail": meta["poster"].clone(),
                }));
            }
            meta["videos"] = serde_json::json!(videos);
        } else {
            // Kotlin `buildAndCachePostMeta()` warms the post's file set as it builds a
            // MOVIE meta (line 7503). A series already reads its files above, so this
            // only fills the gap that made the first Play of a just-opened movie pay a
            // cold origin fetch.
            prefetch_post_files(pid);
        }
        // The PHP echoes the id the CLIENT asked with (backend.php:16375) and
        // always sends `background` plus `year` alongside `releaseInfo`.
        meta["id"] = serde_json::json!(id);
        meta["background"] = meta["poster"].clone();
        if let Some(y) = meta.get("releaseInfo").cloned() {
            meta["year"] = y;
        }
        return stremio_json(200, &serde_json::json!({"meta": meta}), "no-store");
    }
    // ── direct Telegram file (`pm_file_`, `pm:file_`, `pm:file:`) ──────────
    // backend.php:16073-16165. The Telegram-Files catalog rows emit
    // `pm_file_<code>`, so this is the path EVERY one of those items takes; the
    // previous `pm:file:`-only branch left them with a null meta — a blank info
    // page with no poster, description or Play button.
    if let Some(code) = id
        .strip_prefix("pm_file_")
        .or_else(|| id.strip_prefix("pm:file_"))
        .or_else(|| id.strip_prefix("pm:file:"))
    {
        let code = code.to_string();
        let cache_key = format!("file_{}", code);
        // The PHP pre-seeds and re-reads this for an hour (backend.php:16083).
        if let Some(hit) = crate::cache::get("meta", &cache_key) {
            if hit.get("meta").map(|m| !m.is_null()).unwrap_or(false) {
                return stremio_json(200, &hit, "max-age=3600, public");
            }
        }
        let mut title = String::new();
        let mut thumb = String::new();
        let mut size: u64 = 0;
        // 1. `search_files` answers an exact short_code lookup; the PHP also
        // accepts a single-result response (backend.php:16100-16109).
        let hits = crate::wp_data::search_files(&code, 1).await;
        if let Some(f) = hits.iter().find(|f| f.short_code == code).or_else(|| {
            if hits.len() == 1 {
                hits.first()
            } else {
                None
            }
        }) {
            title = f.title.trim().to_string();
            thumb = f.thumbnail.clone();
            size = f.file_size;
        }
        // 2. Missing metadata or thumbnail → a full resolve (backend.php:16113).
        if title.is_empty() || thumb.is_empty() || size == 0 {
            let bot = get_bot_id();
            let res = if bot.is_empty() {
                resolve_shortcode_concurrent(&code, &[]).await
            } else {
                resolve_shortcode(&code, &bot, false).await
            };
            if title.is_empty() {
                title = jstr(&res, "title");
                if title.is_empty() {
                    title = jstr(&res, "file_name");
                }
            }
            if thumb.is_empty() {
                thumb = jstr(&res, "thumbnail_url");
            }
            if size == 0 {
                size = res.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0);
            }
        }
        // A direct file is NEVER null: the PHP falls back to `File <code>`
        // (backend.php:16121-16123), because a null meta hides the Play button.
        if title.trim().is_empty() {
            title = format!("File {}", code);
        }
        let clean = {
            let c = crate::matcher::clean_media_title(&title);
            if c.is_empty() {
                title.clone()
            } else {
                c
            }
        };
        let pills = file_meta_pills(&title, size);
        let pill_line = if pills.is_empty() {
            "Ready to stream".to_string()
        } else {
            pills.join("  •  ")
        };
        let mut genres: Vec<String> = vec!["Direct Telegram File".to_string()];
        for p in &pills {
            if !genres.iter().any(|g| g == p) {
                genres.push(p.clone());
            }
        }
        let body = serde_json::json!({"meta": {
            "id": id,
            // The PHP keeps the requested type, defaulting to `other`.
            "type": if item_type.is_empty() { "other".to_string() } else { item_type.clone() },
            "name": clean,
            "poster": thumb.clone(),
            "posterShape": "poster",
            "background": thumb.clone(),
            "logo": thumb,
            "description": format!("Direct Telegram Cloud File\n{}\n\nFile: {}", pill_line, clean),
            "genres": genres,
        }});
        crate::cache::put("meta", &cache_key, &body, 3600, 0);
        return stremio_json(200, &body, "max-age=600, public");
    }
    // ── meta bridge (backend.php:16410-16455) ─────────────────────────────
    // An id that is NOT ours (`tt…`, `tmdb:…`, `tvdb:…`, …) is forwarded to the
    // first upstream that declares `meta`. Without this every catalog row
    // bridged from AIOMetadata opens as a BLANK page: the client resolves the
    // upstream's own id against whichever addon advertises `meta` for that
    // prefix, and this addon advertises `tt`/`tmdb:`/… .
    let settings = load_catalog_settings_merged();
    if let Some(body) = upstream::forward_meta(&settings, &item_type, &id).await {
        return stremio_json(200, &body, "max-age=3600, public");
    }
    // ── local `media_ids_idx` fallback (backend.php:16456-16478) ───────────
    // An upstream miss must not cost the Play button when this server already
    // knows the title. The PHP's prefix parse is `^([A-Za-z0-9_-]+):([^:]+)`,
    // so `tmdb:123` becomes (tmdb, 123) while a bare `tt0944947` stays
    // (tt, tt0944947) — reproduced exactly, including its behaviour on a
    // `tt…:S:E` id (prefix = the id, value = the season), which finds nothing.
    let (prefix, value) = {
        let mut it = id.splitn(2, ':');
        let head = it.next().unwrap_or("");
        let rest = it.next().unwrap_or("");
        let valid_head = !head.is_empty()
            && head
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
        if !rest.is_empty() && valid_head {
            (
                head.to_ascii_lowercase(),
                rest.split(':').next().unwrap_or("").to_string(),
            )
        } else {
            ("tt".to_string(), id.clone())
        }
    };
    let mut local = crate::wp_data::lookup_id(&prefix, &value).await;
    if jstr(&local, "title").is_empty() && (prefix == "tt" || prefix == "imdb") {
        let alt = if prefix == "tt" { "imdb" } else { "tt" };
        let second = crate::wp_data::lookup_id(alt, &value).await;
        if !jstr(&second, "title").is_empty() {
            local = second;
        }
    }
    let local_title = jstr(&local, "title");
    if !local_title.is_empty() {
        let year = jstr(&local, "year");
        let body = serde_json::json!({"meta": {
            "id": id,
            "type": item_type,
            "name": local_title,
            "year": year,
            "releaseInfo": year,
            "posterShape": "poster",
        }});
        return stremio_json(200, &body, "max-age=3600, public");
    }
    // `{"meta": null}` with HTTP 404 — the PHP's last line
    // (backend.php:16485). A 200 here makes Stremio treat the item as existing
    // and render an empty page instead of falling back.
    stremio_json(
        404,
        &serde_json::json!({"meta": Value::Null}),
        "no-cache, no-store, must-revalidate",
    )
}

/// Movies > Country catalogs, exactly the PHP's ids, names and order
/// (`$movieCountryCatalogs`, backend.php:15075-15086).
const MOVIE_COUNTRY_CATALOGS: &[(&str, &str)] = &[
    ("pm_movies_malay", "Malaysia"),
    ("pm_movies_indo", "Indonesia"),
    ("pm_movies_korean", "Korea"),
    ("pm_movies_japan", "Japan"),
    ("pm_movies_anime", "Anime"),
    ("pm_movies_chinese", "China / HK"),
    ("pm_movies_thai", "Thailand"),
    ("pm_movies_bollywood", "India"),
    ("pm_movies_philippines", "Philippines"),
    ("pm_movies_english", "English"),
];

/// Series > Country catalogs (`$seriesCountryCatalogs`, backend.php:15106-15117).
const SERIES_COUNTRY_CATALOGS: &[(&str, &str)] = &[
    ("pm_series_malay", "Malaysia"),
    ("pm_series_indo", "Indonesia"),
    ("pm_series_kdrama", "K-Drama"),
    ("pm_series_anime", "Anime"),
    ("pm_series_japan", "J-Drama"),
    ("pm_series_cdrama", "C-Drama"),
    ("pm_series_thai", "Thailand"),
    ("pm_series_philippines", "Philippines"),
    ("pm_series_india", "India"),
    ("pm_series_english", "English"),
];

/// `country_code` → the country catalog of each type (`$cMapMovie` /
/// `$cMapSeries`, backend.php:15087-15132).
const COUNTRY_CATALOG_MAP: &[(&str, &str, &str)] = &[
    ("my", "pm_movies_malay", "pm_series_malay"),
    ("id", "pm_movies_indo", "pm_series_indo"),
    ("kr", "pm_movies_korean", "pm_series_kdrama"),
    ("jp", "pm_movies_japan", "pm_series_japan"),
    ("cn", "pm_movies_chinese", "pm_series_cdrama"),
    ("hk", "pm_movies_chinese", "pm_series_cdrama"),
    ("th", "pm_movies_thai", "pm_series_thai"),
    ("in", "pm_movies_bollywood", "pm_series_india"),
    ("ph", "pm_movies_philippines", "pm_series_philippines"),
];

/// `$allGenreOptions` (backend.php:15035-15059) — the genre filter dropdown.
///
/// These are NOT a `genres` key on the catalog: the PHP emits them as
/// `extra[name=genre].options`, which is what Stremio/Nuvio render as the
/// Discover filter. Omitting them (as this port previously did) leaves the
/// genre dropdown empty.
fn all_genre_options() -> Vec<String> {
    [
        "Action",
        "Adventure",
        "Animation",
        "Anime",
        "Biography",
        "Comedy",
        "Crime",
        "Documentary",
        "Drama",
        "Family",
        "Fantasy",
        "History",
        "Horror",
        "Music",
        "Musical",
        "Mystery",
        "Romance",
        "Sci-Fi",
        "Sport",
        "Thriller",
        "War",
        "Western",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

/// `$allYearOptions` (backend.php:15063-15067) — the current year descending
/// to 1980, as strings.
fn all_year_options() -> Vec<String> {
    (1980..=current_year()).rev().map(|y| y.to_string()).collect()
}

/// `$filterOptionsCombined = array_merge($allYearOptions, $allGenreOptions)`
/// (backend.php:15134). The YEAR list comes FIRST, exactly as the PHP merges it.
fn filter_options_combined() -> Vec<String> {
    let mut out = all_year_options();
    out.extend(all_genre_options());
    out
}

/// `date('Y')` for the local clock.
fn current_year() -> i32 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    // Howard Hinnant's `civil_from_days`, valid for any i64 day count.
    let z = secs.div_euclid(86400) + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    year as i32
}

/// One `extra` entry, exactly the PHP's shape: `name`, optional `options`, then
/// `isRequired` (backend.php:15141-15146).
fn extra_entry(name: &str, options: Option<Vec<String>>, required: bool) -> Value {
    let mut e = serde_json::Map::new();
    e.insert("name".to_string(), Value::String(name.to_string()));
    if let Some(opts) = options {
        e.insert("options".to_string(), serde_json::json!(opts));
    }
    e.insert("isRequired".to_string(), Value::Bool(required));
    Value::Object(e)
}

/// One descriptor catalog row. `supported`/`required` reproduce the PHP's legacy
/// `extraSupported` / `extraRequired` arrays; the keyword rows emit NEITHER
/// (backend.php:15282-15291), so pass empty slices for those.
fn catalog_row(
    ty: &str,
    id: &str,
    name: &str,
    extra: Vec<Value>,
    supported: &[&str],
    required: &[&str],
) -> Value {
    let mut row = serde_json::json!({
        "type": ty,
        "id": id,
        "name": name,
        "extra": extra,
    });
    if !supported.is_empty() {
        row["extraSupported"] = serde_json::json!(supported);
    }
    if !required.is_empty() {
        row["extraRequired"] = serde_json::json!(required);
    }
    row
}

/// The LOCAL catalog rows — a full port of the PHP's `$manifestCatalogs`
/// (backend.php:15135-15292), in the PHP's order:
///
/// * movie: `top` (Popular), `year` (New), `pm_search_movie`, then the 10
///   country rows;
/// * series: `top`, `pm_series_year`, `pm_search_series`, then the 10 country
///   rows;
/// * other (Telegram Files): `top`, `pm_files_year`, `pm_search_files`.
///
/// Plus this port's two merged TRENDING rows (`pm_trending_movies` /
/// `pm_trending_series`), which replace the PHP's per-keyword `pm_topkw_*` rows
/// for post discovery; the keyword FILE rows are appended by [`topkw_catalogs`].
///
/// The previous hand-written six-row list (`pm_movies`, `pm_latest_movies`, …)
/// advertised ids that exist nowhere in the PHP and in no client's home-row map,
/// while every real row the dashboard's Catalog Configuration panel offers
/// (`top`, `pm_movies_malay`, `pm_files_year`, …) was missing — which is what
/// looked like "manifest.json catalog is broken".
fn pm_catalogs() -> Value {
    let genres = all_genre_options();
    let years = all_year_options();
    // The country rows use `$filterOptionsCombined` (years FIRST, then genres).
    let combined = filter_options_combined();
    let mut rows: Vec<Value> = Vec::new();
    // ── 1. Movies ──────────────────────────────────────────────────────────
    rows.push(catalog_row(
        "movie",
        "top",
        "Popular",
        vec![
            extra_entry("search", None, false),
            extra_entry("genre", Some(genres.clone()), false),
            extra_entry("year", Some(years.clone()), false),
            extra_entry("skip", None, false),
        ],
        &["search", "genre", "year", "skip"],
        &[],
    ));
    // `year` is the PHP's "New" row: its `genre` extra carries the YEAR list
    // (backend.php:15154), not the genre list.
    rows.push(catalog_row(
        "movie",
        "year",
        "New",
        vec![
            extra_entry("genre", Some(years.clone()), true),
            extra_entry("skip", None, false),
        ],
        &["genre", "skip"],
        &["genre"],
    ));
    rows.push(catalog_row(
        "movie",
        "pm_search_movie",
        "Search Movies",
        vec![
            extra_entry("search", None, true),
            extra_entry("genre", Some(genres.clone()), false),
            extra_entry("year", Some(years.clone()), false),
            extra_entry("skip", None, false),
        ],
        &["search", "genre", "year", "skip"],
        &["search"],
    ));
    for (id, name) in MOVIE_COUNTRY_CATALOGS {
        rows.push(catalog_row(
            "movie",
            id,
            name,
            vec![
                extra_entry("genre", Some(combined.clone()), false),
                extra_entry("year", Some(years.clone()), false),
                extra_entry("skip", None, false),
            ],
            &["genre", "year", "skip"],
            &[],
        ));
    }
    // ── 2. Series ──────────────────────────────────────────────────────────
    rows.push(catalog_row(
        "series",
        "top",
        "Popular",
        vec![
            extra_entry("search", None, false),
            extra_entry("genre", Some(genres.clone()), false),
            extra_entry("year", Some(years.clone()), false),
            extra_entry("skip", None, false),
        ],
        &["search", "genre", "year", "skip"],
        &[],
    ));
    rows.push(catalog_row(
        "series",
        "pm_series_year",
        "New",
        vec![
            extra_entry("genre", Some(years.clone()), true),
            extra_entry("skip", None, false),
        ],
        &["genre", "skip"],
        &["genre"],
    ));
    rows.push(catalog_row(
        "series",
        "pm_search_series",
        "Search Series",
        vec![
            extra_entry("search", None, true),
            extra_entry("genre", Some(genres.clone()), false),
            extra_entry("year", Some(years.clone()), false),
            extra_entry("skip", None, false),
        ],
        &["search", "genre", "year", "skip"],
        &["search"],
    ));
    for (id, name) in SERIES_COUNTRY_CATALOGS {
        rows.push(catalog_row(
            "series",
            id,
            name,
            vec![
                extra_entry("genre", Some(combined.clone()), false),
                extra_entry("year", Some(years.clone()), false),
                extra_entry("skip", None, false),
            ],
            &["genre", "year", "skip"],
            &[],
        ));
    }
    // ── 3. Other (Telegram files) ──────────────────────────────────────────
    // The PHP names these "Telegram Files" / "New" / "Telegram Files"
    // (backend.php:15244-15277) — not the longer labels this port used.
    rows.push(catalog_row(
        "other",
        "top",
        "Telegram Files",
        vec![
            extra_entry("search", None, false),
            extra_entry("genre", Some(years.clone()), false),
            extra_entry("skip", None, false),
        ],
        &["search", "genre", "skip"],
        &[],
    ));
    rows.push(catalog_row(
        "other",
        "pm_files_year",
        "New",
        vec![
            extra_entry("genre", Some(years.clone()), true),
            extra_entry("skip", None, false),
        ],
        &["genre", "skip"],
        &["genre"],
    ));
    rows.push(catalog_row(
        "other",
        "pm_search_files",
        "Telegram Files",
        vec![
            extra_entry("search", None, true),
            extra_entry("genre", Some(years.clone()), false),
            extra_entry("skip", None, false),
        ],
        &["search", "genre", "skip"],
        &["search"],
    ));
    // ── 4. This port's merged trending post rows ───────────────────────────
    rows.push(catalog_row(
        "movie",
        TRENDING_MOVIE_CATALOG,
        "Trending Movies",
        vec![extra_entry("skip", None, false)],
        &["skip"],
        &[],
    ));
    rows.push(catalog_row(
        "series",
        TRENDING_SERIES_CATALOG,
        "Trending Series",
        vec![extra_entry("skip", None, false)],
        &["skip"],
        &[],
    ));
    Value::Array(rows)
}

/// `substr(md5(strtolower($kw)), 0, 10)` — the keyword catalog id suffix
/// (backend.php:11859).
fn topkw_id_suffix(kw: &str) -> String {
    crate::fileid::md5_hex(kw.to_lowercase().as_bytes())
        .chars()
        .take(10)
        .collect()
}

/// `ucwords($kw)`.
fn ucwords(raw: &str) -> String {
    raw.split_whitespace()
        .map(|w| {
            let mut it = w.chars();
            match it.next() {
                Some(first) => first.to_uppercase().collect::<String>() + it.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

/// `pm_topkw_<md5(kw)[0..10]>` → keyword, at most FIVE entries
/// (`fd_get_trending_keywords_map()`, backend.php:11853-11865).
///
/// The map is memoised by [`crate::wp_data::trending_keywords`] (1 h), so the
/// descriptor does not pay an origin call per request.
async fn topkw_map() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    for (kw, _) in crate::wp_data::trending_keywords().await {
        let kw = kw.trim().to_string();
        if !crate::wp_data::is_topkeyword_valid(&kw) {
            continue;
        }
        out.push((format!("pm_topkw_{}", topkw_id_suffix(&kw)), kw));
        if out.len() >= 5 {
            break;
        }
    }
    out
}

/// The keyword FILE catalogs the PHP appends to the descriptor
/// (backend.php:15279-15292).
///
/// The PHP emits a quality-keyword `genre` list, the year list, and **no**
/// `extraSupported` / `extraRequired` at all on these rows — matched exactly.
async fn topkw_catalogs() -> Vec<Value> {
    let years = all_year_options();
    let quality: Vec<String> = ["4K", "1080p", "720p", "BluRay", "WEB-DL", "HEVC"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    topkw_map()
        .await
        .into_iter()
        .map(|(id, kw)| {
            catalog_row(
                "other",
                &id,
                &ucwords(&kw),
                vec![
                    extra_entry("genre", Some(quality.clone()), false),
                    extra_entry("year", Some(years.clone()), false),
                    extra_entry("skip", None, false),
                ],
                &[],
                &[],
            )
        })
        .collect()
}

/// Move the detected home country's movie/series rows to the top of their type,
/// like the PHP (backend.php:15087-15132).
fn prioritise_home_country(catalogs: Vec<Value>, settings: &Value) -> Vec<Value> {
    let cc = settings
        .get("country")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    let Some((_, movie, series)) = COUNTRY_CATALOG_MAP.iter().find(|(c, _, _)| *c == cc) else {
        return catalogs;
    };
    let type_rank = |t: &str| match t {
        "movie" => 0,
        "series" => 1,
        _ => 2,
    };
    let mut rows = catalogs;
    rows.sort_by_key(|c| {
        let ty = c.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let id = c.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let home = id == *movie || id == *series;
        (type_rank(ty), if home { 0 } else { 1 })
    });
    rows
}

/* ── Eclipse music (backend.php:13568-14259) ─────────────────────────────── */

/// How long a music search answer is reused.
///
/// The `t=` cache-buster exists so a newly indexed track is never hidden by an
/// INTERMEDIATE cache (the documented "freshly added track does not match" bug).
/// It was never a reason for us to keep no answer at all — that made the music
/// browser the one screen that went blank whenever the origin hiccuped, while
/// every video list survived on stale cache. 60 s keeps a fresh upload visible
/// almost immediately and still lets a browse burst reuse one call.
const MUSIC_SEARCH_TTL: i64 = 60;

/// How long a music answer may be served STALE (same 6 h as `WP_STALE_GRACE`).
const MUSIC_STALE_GRACE: i64 = 21_600;

/// `{WP_API_BASE}/search-music?search=…&limit=…&t=…` → formatted `track` objects.
///
/// NOW BEHAVES LIKE THE STREAM PATH: cache + single-flight + stale fallback, and
/// the same origin circuit breaker. The `t=<unixtime>` buster stays in the OUTBOUND
/// url (it defeats an intermediate cache) but is deliberately NOT part of our cache
/// key, which is what made every query a guaranteed cold call before.
///
/// Observed live with the origin failing: `search-music` returned `rows:0` on every
/// query, so the music browser showed nothing at all — while `/stream` on the same
/// origin still answered from its stale copy. That asymmetry is what this fixes.
async fn eclipse_search(query: &str, limit: usize) -> Vec<Value> {
    // Key WITHOUT the cache-buster: same query + limit = same answer.
    let key = format!("{}|{}", limit, query.trim().to_ascii_lowercase());
    let q_owned = query.to_string();
    let body = crate::cache::fetch_cached(
        "music_search",
        &key,
        MUSIC_SEARCH_TTL,
        MUSIC_STALE_GRACE,
        move || async move {
            // Shared breaker: once the origin has failed to answer, do not spend
            // another 15 s here.
            let down = crate::httpc::wp_origin_down_secs();
            if down > 0 {
                log(
                    "wp origin down - music search skipped",
                    &serde_json::json!({"q": q_owned, "for_secs": down}),
                );
                return (Value::Null, false);
            }
            let url = format!(
                "{}/search-music?search={}&limit={}&offset=0&t={}",
                crate::config::wp_api_base().trim_end_matches('/'),
                urlencode_component(&q_owned),
                limit,
                crate::storage::now()
            );
            let (mut res, mut status) =
                crate::httpc::http_json_h_status(&url, &[], "GET", 15, &[]).await;
            if res.is_null() && status != 0 {
                // ONE retry, because a transport blip used to reach the user as "no
                // tracks at all". Measured on a CPH2581: two consecutive
                // `/search-music?search=wildflower` calls logged
                // `http_json failed {"error":"error sending request"}` and the addon
                // answered `{"tracks":[]}`; the very next call for the same query
                // returned 39 rows. The client paints an empty list on that, and the
                // user reads it as "the song is not there".
                tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                let again = crate::httpc::http_json_h_status(&url, &[], "GET", 15, &[]).await;
                res = again.0;
                status = again.1;
            } else if res.is_null() {
                // `status == 0` = the whole 15 s was already spent. Retrying would
                // double it for a request the origin is demonstrably not answering —
                // the same trap that made the episode list take 30 s.
                log(
                    "music search timed out - not retrying",
                    &serde_json::json!({"q": q_owned}),
                );
            }
            crate::httpc::wp_note_status(status);
            // A failed call is NOT cacheable: storing it would pin an empty music
            // browser for the whole TTL, which is exactly the bug being fixed.
            if status == 0 {
                return (Value::Null, false);
            }
            let rows = if let Some(arr) = res.as_array() {
                arr.clone()
            } else {
                res.get("tracks")
                    .or_else(|| res.get("items"))
                    .or_else(|| res.get("files"))
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default()
            };
            let tracks: Vec<Value> = rows
                .iter()
                .filter(|r| !str_at(r, "short_code").is_empty())
                .map(crate::eclipse::format_track)
                .collect();
            // The ORIGIN'S ORDER IS THE RANKING — do not re-sort here.
            //
            // This used to sort by `quality_score(format, duration)`, which silently
            // overrode the origin's size-first ranking (with a rip-title penalty).
            // That is why the phone listed `05.WILDFLOWER.flac` (47.3 MB) ABOVE
            // `05.WILDFLOWER.m4a` (48.9 MB): same album, the bigger file was demoted
            // because the other one is FLAC. The track shape this route returns
            // carries no size at all, so the engine has nothing to rank by — it can
            // only preserve or destroy the origin's order, and destroying it was the
            // bug.
            //
            // One line per query: "the client found nothing to play" and "we
            // answered nothing" look identical from the player, so log both sides.
            log(
                "music search",
                &serde_json::json!({
                    "q": q_owned,
                    "limit": limit,
                    "rows": rows.len(),
                    "tracks": tracks.len(),
                }),
            );
            (Value::Array(tracks), true)
        },
    )
    .await;

    let tracks: Vec<Value> = body.as_array().cloned().unwrap_or_default();
    if !tracks.is_empty() {
        music_prewarm(&tracks);
        return tracks;
    }
    // Same stale fallback the stream path has: a known album beats a blank screen.
    if let Some(stale) = crate::cache::get_stale("music_search", &key) {
        if let Some(arr) = stale.as_array() {
            if !arr.is_empty() {
                log(
                    "music search served stale",
                    &serde_json::json!({"q": query, "tracks": arr.len()}),
                );
                let out = arr.clone();
                music_prewarm(&out);
                return out;
            }
        }
    }
    tracks
}

/// How many of a search's top tracks to warm.
///
/// A viewer plays one track at a time and almost always one of the first few, so
/// this is deliberately small: the origin's prewarm queue is shared with video and
/// capped at 40, and a 120-track "latest" listing must not own it.
const MUSIC_PREWARM_LIMIT: usize = 8;

/// Warm the ids of the tracks at the top of a music search — the SAME cheap batch
/// the stream path uses.
///
/// [`eclipse_stream`] consults `resolve_shortcode_cached` before doing anything, so
/// a warmed track plays **without ever touching `/resolve-file`**. That is the whole
/// point: when the origin's relay subsystem is saturated it answers `503 Server busy
/// resolving file` to every cold resolve, and a cold track is then simply skipped by
/// the player. Measured in this session: `ZMIPWf` took five consecutive 503s and a
/// 20 s budget exhaustion because nothing had ever warmed it, while `l13Asn` — which
/// had a cached id — played straight through the same outage.
///
/// Never blocks and never fails the search: `prewarm_streams_batch_limited` skips
/// itself while a stream is playing or the origin cooldown is armed.
fn music_prewarm(tracks: &[Value]) {
    let bot = get_bot_id();
    if bot.is_empty() {
        return;
    }
    let codes: Vec<String> = tracks
        .iter()
        .take(MUSIC_PREWARM_LIMIT)
        .map(|t| str_at(t, "id"))
        .filter(|c| !c.is_empty())
        .collect();
    if codes.is_empty() {
        return;
    }
    crate::resolve::prewarm_streams_batch_limited(
        codes,
        &bot,
        "music",
        MUSIC_PREWARM_LIMIT,
    );
}

fn str_at(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

/// `/eclipse/stream/<short_code>` and `/eclipse/play/<short_code>` — resolve and
/// hand back a downloadable URL. The URL extension must match the ADVERTISED
/// format (`.wav` for an ALAC `.m4a`, which [`crate::download`] decodes to PCM
/// behind a WAV header), or the client skips the track.
async fn eclipse_stream(ctx: &ReqCtx, short_code: &str) -> Response {
    let code = short_code
        .split('/')
        .next()
        .unwrap_or("")
        .trim()
        .to_string();
    if code.is_empty() {
        return stremio_json(404, &serde_json::json!({"error": "Not found"}), "no-store");
    }
    log(
        "music stream requested",
        &serde_json::json!({"code": code, "quality": ctx.qs("quality")}),
    );
    let bot = get_bot_id();
    // The STATUS matters here: a busy origin (503 "Server busy resolving file",
    // retry_after 5) is not a missing track. Before this, every origin hiccup
    // reached the music client as `404 Track not found` and the track was skipped.
    let (resolved, resolve_status) = if bot.is_empty() {
        (resolve_shortcode_concurrent(&code, &[]).await, 0u16)
    } else {
        crate::resolve::resolve_shortcode_status(&code, &bot, false).await
    };
    let file_name = {
        let t = jstr(&resolved, "title");
        if t.is_empty() {
            jstr(&resolved, "file_name")
        } else {
            t
        }
    };
    if file_name.is_empty() {
        // Silent before: a 404 with no trail is impossible to debug from a phone.
        log(
            "eclipse stream: the resolve carried no file name",
            &serde_json::json!({"short_code": code, "bot_id": bot, "origin_status": resolve_status}),
        );
        // A 5xx / a request that never completed means the origin is busy or
        // unreachable, NOT that the track is missing: answer 503 + `Retry-After`
        // so the client retries the same track instead of dropping it.
        if resolve_status >= 500 || resolve_status == 0 {
            let mut resp = json_response(
                503,
                &serde_json::json!({
                    "error": "Track temporarily unavailable",
                    "message": "The origin is busy resolving this file. Retry in a few seconds.",
                }),
            );
            resp.headers_mut().insert(
                axum::http::header::RETRY_AFTER,
                axum::http::HeaderValue::from_static("5"),
            );
            return resp;
        }
        return stremio_json(404, &serde_json::json!({"error": "Track not found"}), "no-store");
    }
    let mime = {
        let m = jstr(&resolved, "mime");
        let m = if m.is_empty() { jstr(&resolved, "file_type") } else { m };
        m
    };
    let size = resolved.get("file_size").and_then(|v| v.as_u64()).unwrap_or(0);
    // NO renaming. The advertised `format` and `mimeType` are the file's own, and
    // the payload names the file `/api/download` will actually read. The old
    // `.wav` alias (inherited from backend.php) lied about the container: the
    // payload said `X.wav` / `audio/wav`, so `is_alac_target()` answered "not
    // ALAC" and the raw ALAC/MP4 bytes went out under `content-type: audio/wav`
    // (measured: magic `/isom`, not `RIFF/WAVE`) — the player's "Playback error
    // - stream unavailable". When the bytes ARE transcoded, the download
    // response says so itself (`content-type: audio/wav`, `.wav` disposition).
    let served_mime = if mime.is_empty() {
        "audio/mpeg".to_string()
    } else {
        mime.clone()
    };
    let ext = file_name
        .rsplit_once('.')
        .map(|(_, e)| e.to_lowercase())
        .unwrap_or_else(|| "mp3".to_string());
    let payload = serde_json::json!({
        "file_size": size,
        "file_name": file_name,
        "mime": mime,
        "short_code": code,
    });
    let b64 = crate::fileid::base64url_encode(payload.to_string().as_bytes());
    let url = build_stream_url(&ctx.base_url, &b64, &file_name);
    stremio_json(
        200,
        &serde_json::json!({
            "url": url,
            "format": ext,
            "mimeType": served_mime,
        }),
        "no-store",
    )
}

/* ── subtitles (backend.php:13789 + fd_get_item_subtitles) ────────────────── */

/// Default subtitle provider (PHP `FD_DEFAULT_SUBTITLE_MANIFEST_URL`). It matches
/// on the **IMDb id in the path** only — `videoHash`/`filename` are ignored by v3
/// — so every `pm:` id has to be translated to `tt…` first, and a series id must
/// keep its `:S:E` suffix or it answers an empty list.
const DEFAULT_SUBTITLE_BASE: &str = "https://opensubtitles-v3.strem.io";

/// Translate any id the addon can be asked about into an OpenSubtitles id.
/// `lookup_imdb_by_title()` with the 600 s NEGATIVE cache the TDLib app has
/// (`subtitleResolveMiss`) and the port was missing.
///
/// Only successes were ever cached, so a title with no IMDb entry re-ran the whole
/// chain — index lookup + own API search + upstream search, 4-7 s — on **every**
/// episode request, on the same event loop as the real work. Measured in PHP: one
/// episode cost 3.2 s with 2 chain calls, the next 0.5 s with none once the marker
/// existed. Keep the TTL short: it delays a title that appears in the index later.
const TITLE_IMDB_MISS_TTL: i64 = 600;

async fn imdb_by_title_cached(title: &str, year: &str) -> String {
    let key = format!("{}|{}", title.trim().to_lowercase(), year.trim());
    // A NEGATIVE cache entry (value `1`), so a hit is one hash lookup instead of a
    // marker-file stat, and the TTL is enforced by the cache itself.
    if crate::cache::get("title_imdb_miss", &key).is_some() {
        return String::new();
    }
    let imdb = crate::wp_data::lookup_imdb_by_title(title, year).await;
    if imdb.is_empty() {
        crate::cache::put(
            "title_imdb_miss",
            &key,
            &serde_json::json!(1),
            TITLE_IMDB_MISS_TTL,
            0,
        );
        log(
            "title imdb miss cached",
            &serde_json::json!({"title": title, "year": year, "ttl": TITLE_IMDB_MISS_TTL}),
        );
    } else {
        crate::cache::invalidate("title_imdb_miss", &key);
    }
    imdb
}

async fn subtitle_target_id(item_type: &str, id: &str) -> String {
    let lower = id.to_lowercase();
    if lower.starts_with("tt") {
        return id.to_string();
    }
    // `pm:post:<id>[:S:E]`
    if let Some(rest) = id.strip_prefix("pm:post:") {
        let mut parts = rest.split(':');
        let pid = parts.next().unwrap_or("");
        let season = parts.next().unwrap_or("");
        let episode = parts.next().unwrap_or("");
        let row = crate::wp_data::lookup_id("post", pid).await;
        let mut imdb = jstr(&row, "imdb_id");
        if imdb.is_empty() {
            imdb = jstr(&row, "imdb");
        }
        if imdb.is_empty() {
            // Resolve by title (two steps, both on our own stack).
            let title = jstr(&row, "title");
            let year = jstr(&row, "year");
            if !title.is_empty() {
                imdb = imdb_by_title_cached(&title, &year).await;
            }
        }
        if imdb.is_empty() {
            return String::new();
        }
        if !season.is_empty() && !episode.is_empty() {
            return format!("{}:{}:{}", imdb, season, episode);
        }
        if item_type == "series" {
            // A series without S/E can never match — answer empty rather than
            // letting the provider return a junk list.
            return String::new();
        }
        return imdb;
    }
    // `pm:file:<short_code>` -> title -> imdb
    if let Some(code) = id.strip_prefix("pm:file:") {
        let bot = get_bot_id();
        let resolved = if bot.is_empty() {
            resolve_shortcode_concurrent(code, &[]).await
        } else {
            resolve_shortcode(code, &bot, false).await
        };
        let mut title = jstr(&resolved, "title");
        if title.is_empty() {
            title = jstr(&resolved, "file_name");
        }
        if title.is_empty() {
            return String::new();
        }
        return imdb_by_title_cached(&title, "").await;
    }
    String::new()
}

async fn handle_subtitles(path: &str) -> Response {
    let Some((item_type, id, _extra)) = split_addon(path, "/subtitles/") else {
        return stremio_json(200, &serde_json::json!({"subtitles": []}), "no-store");
    };
    let target = subtitle_target_id(&item_type, &id).await;
    if target.is_empty() {
        log(
            "subtitles: no target id",
            &serde_json::json!({"item_type": item_type, "id": id}),
        );
        return stremio_json(200, &serde_json::json!({"subtitles": []}), "no-store");
    }
    let url = format!(
        "{}/subtitles/{}/{}.json",
        DEFAULT_SUBTITLE_BASE, item_type, target
    );
    let body = crate::httpc::cached_wp_json(&url, 3600, 12).await;
    let subs = body
        .get("subtitles")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    log(
        "subtitles served",
        &serde_json::json!({"id": id, "target": target, "count": subs.len()}),
    );
    stremio_json(200, &serde_json::json!({"subtitles": subs}), "no-store")
}

/// `fd_stremio_manifest_identity()` (backend.php:10831-10952).
///
/// The descriptor `id` and `description` follow the ADDRESS the addon was
/// installed from — `.../addon.local` on this device, `.../addon.lan` on the
/// Wi-Fi, `.../addon.tunnel` over Cloudflare, `.../addon.server` otherwise — and
/// the description NAMES that address. A single fixed description (as this port
/// used) loses the "which URL did I install?" hint Stremio shows on the addon
/// card. `?mode=lan|localhost|tunnel|server` forces one, like the PHP.
fn manifest_identity(ctx: &ReqCtx, origin: &str) -> (&'static str, String) {
    let mut host = ctx.hdr("x-forwarded-host");
    if host.is_empty() {
        host = ctx.hdr("host");
    }
    let host = host.split(',').next().unwrap_or("").trim().to_string();
    let fallback = if host.is_empty() {
        "127.0.0.1:8088".to_string()
    } else {
        host
    };
    let host_name = url::Url::parse(&format!("http://{}", fallback))
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_ascii_lowercase()))
        .unwrap_or_else(|| fallback.to_ascii_lowercase());

    let mut is_tunnel = host_name.ends_with(".trycloudflare.com")
        || host_name == "trycloudflare.com"
        || host_name.ends_with(".tunnel.pencarimovie.com")
        || host_name.ends_with("-tunnel.pencarimovie.com")
        || host_name == "tunnel.pencarimovie.com";
    // A Zero Trust Named Tunnel serves a CUSTOM hostname; the PHP also accepts
    // whatever `custom_domain(s)` the tunnel state recorded.
    if !is_tunnel {
        let st = tunnel::load_state();
        let mut domains: Vec<String> = Vec::new();
        if let Some(cd) = st.get("custom_domain").and_then(|v| v.as_str()) {
            if !cd.trim().is_empty() {
                domains.push(cd.trim().to_ascii_lowercase());
            }
        }
        if let Some(arr) = st.get("custom_domains").and_then(|v| v.as_array()) {
            for d in arr {
                if let Some(s) = d.as_str() {
                    if !s.trim().is_empty() {
                        domains.push(s.trim().to_ascii_lowercase());
                    }
                }
            }
        }
        is_tunnel = domains.iter().any(|d| *d == host_name);
    }

    let forced = ctx.q("mode").unwrap_or("").trim().to_ascii_lowercase();
    let mode: &'static str = match forced.as_str() {
        "tunnel" => "tunnel",
        "localhost" => "localhost",
        "lan" => "lan",
        "server" => "server",
        _ if is_tunnel => "tunnel",
        _ if matches!(host_name.as_str(), "127.0.0.1" | "localhost" | "::1") => "localhost",
        _ if is_private_ipv4(&host_name) => "lan",
        _ => "server",
    };
    let description = match mode {
        "tunnel" => format!(
            "Stream movies and series from Telegram via Cloudflare Tunnel HTTPS. Address: {}",
            origin
        ),
        "localhost" => format!(
            "Stream movies and series from Telegram on this device only. Address: {}",
            origin
        ),
        "lan" => format!(
            "Stream movies and series from Telegram on your Wi-Fi / LAN. Address: {}",
            origin
        ),
        _ => format!(
            "Stream movies and series from Telegram on your server. Address: {}",
            origin
        ),
    };
    (mode, description)
}

/// The addon descriptor, with the upstream catalogs bridged in
/// (backend.php:15343-15489).
///
/// `up_<index>_<realCatalogId>` keeps the upstream's `type`/`name`/`extra` but
/// TRIMMED, because copying an upstream catalog verbatim carried every `extra`
/// `options` list and blew past Stremio's descriptor limit. The result is then
/// fitted to [`upstream::MANIFEST_DESCRIPTOR_BUDGET`] by dropping trailing
/// bridged catalogs — never the local `pm_*` ones.
async fn manifest(ctx: &ReqCtx) -> Value {
    let base_url = rtrim(&ctx.base_url).to_string();
    let (mode, description) = manifest_identity(ctx, &base_url);

    let settings = load_catalog_settings_merged();
    let mut catalogs: Vec<Value> = pm_catalogs().as_array().cloned().unwrap_or_default();
    // `catalogs_enabled: false` advertises NO catalogs at all (the PHP empties
    // the list AND drops the `catalog` resource); otherwise each row still has
    // to pass its own media-type + checkbox gate.
    let catalogs_enabled = settings
        .get("catalogs_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if catalogs_enabled {
        // The PHP appends one `other` FILE catalog per trending keyword
        // (`pm_topkw_<hash>`, at most five) — backend.php:15279-15292. Skipped
        // while catalogs are off: the keyword map is an origin call.
        catalogs.extend(topkw_catalogs().await);
        // A detected home country puts its rows first inside each type.
        catalogs = prioritise_home_country(catalogs, &settings);
        catalogs.retain(|c| catalog_advertised(&settings, c));
    } else {
        catalogs.clear();
    }
    let local_count = catalogs.len();
    // PHP: `$activeTypes` comes from `enabled_types` (backend.php:15336-15341),
    // so a disabled media type disappears from the top-level `types` AND from
    // every resource's `types` — the client then never asks for a row that
    // cannot exist. Bridged upstream types are appended (backend.php:15369-15371).
    let mut types = active_types(&settings);
    let (bridged, bridged_types) = upstream::bridged_catalogs(&settings).await;
    for t in bridged_types {
        if !types.contains(&t) {
            types.push(t);
        }
    }
    let bridged_count = bridged.len();
    catalogs.extend(bridged);
    log(
        "manifest built",
        &serde_json::json!({
            "local_catalogs": local_count,
            "bridged_catalogs": bridged_count,
            "types": types.clone(),
        }),
    );

    // `resources` is an array of OBJECTS, exactly like the PHP
    // (backend.php:15418-15438): each one declares the `types` it serves, and
    // meta/stream/subtitles also carry the full `idPrefixes`. A bare string
    // array loses that scoping — the client cannot tell that (say) a `tvdb:`
    // id is answerable — and `catalog` is only advertised when a row actually
    // survived the filters.
    let resources = manifest_resources(&types, !catalogs.is_empty());

    // The SHIPPED logo only. The PHP keeps a raw.githubusercontent.com copy as a
    // fallback for deployments without `public/logo.png`; this port embeds the
    // asset, so the GitHub URL is deliberately NOT ported (operator decision) —
    // every build serves its own logo and never depends on that host.
    let logo_url = format!("{}/logo.png", base_url);

    let mut doc = serde_json::json!({
        "id": format!("org.pencarimovie.addon.{}", mode),
        // The APP version (`1.0.52` on Android), not "1.0.0" and not the engine's
        // `git describe`: Stremio/Nuvio parse and DISPLAY this, so a fixed value
        // made every install look like the first release, and an engine stamp
        // made every APK release look like the same addon version.
        "version": crate::config::descriptor_version(),
        "name": "PencariMovie",
        // Names the address the addon was installed from (backend.php:10881-10951).
        "description": description,
        "resources": resources,
        "types": types,
        // `tvdb:` / `anilist:` / `kitsu` are declared so a row bridged from an
        // upstream (whose ids are one of those) is still resolved against THIS
        // addon: that is what routes it through the meta bridge instead of
        // leaving the client with no addon to ask.
        "idPrefixes": ID_PREFIXES,
        "catalogs": catalogs,
        "logo": logo_url.clone(),
        "background": logo_url,
        // Stremio only renders the CONFIGURE gear when `configurable` is true —
        // the PHP declares all four keys (backend.php manifest builder), so an
        // empty object here meant the addon had no Configure button at all.
        "behaviorHints": {
            "configurable": true,
            "configurationRequired": false,
            "adult": false,
            "p2p": false,
        },
    });

    if let Some(cfg) = env("STREMIO_ADDONS_CONFIG")
        .or_else(|| env("PM_STREMIO_ADDONS_CONFIG"))
        .and_then(|raw| {
            let t = raw.trim();
            if t.is_empty() || t == "false" || t == "0" || t == "null" {
                Some(Value::Null)
            } else {
                serde_json::from_str::<Value>(t).ok()
            }
        })
        .or_else(|| settings.get("stremioAddonsConfig").cloned())
    {
        if cfg.is_null() {
            doc.as_object_mut().map(|m| m.remove("stremioAddonsConfig"));
        } else {
            doc["stremioAddonsConfig"] = cfg;
        }
    }

    upstream::fit_descriptor(doc, local_count)
}

/// The addon's `idPrefixes`, in the PHP's ORDER (backend.php:15458). Order is
/// observable: the client shows the addon under the first matching prefix.
const ID_PREFIXES: [&str; 9] = [
    "pm_", "pm:", "tt", "tmdb:", "kitsu:", "kitsu", "mal:", "anilist:", "tvdb:",
];

/// `$activeTypes` (backend.php:15336-15341): the enabled media types, in the
/// PHP's order, falling back to all three when nothing is selected.
///
/// A missing `enabled_types` map means the PHP's default (all three on); an
/// EXISTING map with a missing/false/zero key means that type is off.
fn active_types(settings: &Value) -> Vec<String> {
    let all = || {
        vec![
            "movie".to_string(),
            "series".to_string(),
            "other".to_string(),
        ]
    };
    let Some(map) = settings.get("enabled_types").and_then(|v| v.as_object()) else {
        return all();
    };
    let mut out: Vec<String> = Vec::new();
    for t in ["movie", "series", "other"] {
        if crate::session::truthy(map.get(t)) {
            out.push(t.to_string());
        }
    }
    if out.is_empty() {
        // Never advertise an empty `types`: the client would ask for nothing.
        return all();
    }
    out
}

/// The `resources` array (backend.php:15418-15438): objects, each declaring the
/// `types` it serves, with `idPrefixes` on meta/stream/subtitles. `catalog` is
/// advertised only when a row actually survived the filters.
fn manifest_resources(types: &[String], has_catalogs: bool) -> Vec<Value> {
    let mut resources: Vec<Value> = Vec::new();
    if has_catalogs {
        resources.push(serde_json::json!({
            "name": "catalog",
            "types": types,
        }));
    }
    for name in ["meta", "stream", "subtitles"] {
        resources.push(serde_json::json!({
            "name": name,
            "types": types,
            "idPrefixes": ID_PREFIXES,
        }));
    }
    resources
}

/// The descriptor RESPONSE (backend.php:15468-15488): a byte-stable `ETag`, a
/// `max-age=1800, stale-while-revalidate=3600` cache policy, and a 304 when the
/// client's `If-None-Match` already matches — so a TV box re-opening the addon
/// does not re-download a 40 KB descriptor on every launch.
fn manifest_response(ctx: &ReqCtx, doc: &Value) -> Response {
    let text = serde_json::to_string(doc).unwrap_or_else(|_| "{}".to_string());
    let etag = format!("\"{}\"", crate::fileid::md5_hex(text.as_bytes()));
    const CACHE: &str = "max-age=1800, stale-while-revalidate=3600";
    let client = ctx
        .hdr("if-none-match")
        .trim_matches(|c| c == '"' || c == '\'' || c == ' ')
        .to_string();
    if !client.is_empty() && client == etag.trim_matches('"') {
        return Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::ETAG, &etag)
            .header(header::CACHE_CONTROL, CACHE)
            .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .body(Body::empty())
            .unwrap_or_else(|_| not_found_json());
    }
    let mut resp = stremio_json(200, doc, CACHE);
    if let Ok(v) = axum::http::HeaderValue::from_str(&etag) {
        resp.headers_mut().insert(header::ETAG, v);
    }
    resp
}

/* ── version (fd_check_version, backend.php:7719) ─────────────────────────── */

fn version_less(a: &str, b: &str) -> bool {
    // A component can be a git stamp rather than a number. `3.0.5-14-gf154c62`
    // splits into "3", "0", "5-14-gf154c62", and `parse()` on the last one FAILS.
    // Falling back to 0 made that build compare as 3.0.0, so the moment the
    // WordPress `min_version` reached the same major.minor (`3.0.4`) every build
    // from a git checkout reported `update_needed: true`: the dashboard showed an
    // update pill for the newest binary, and `tools/smoke_boot.mjs` failed its
    // version-payload check. Read the leading digits instead.
    let num = |s: &str| -> i64 {
        s.trim()
            .trim_start_matches('v')
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(0)
    };
    let pa: Vec<i64> = a.split('.').map(num).collect();
    let pb: Vec<i64> = b.split('.').map(num).collect();
    for i in 0..pa.len().max(pb.len()) {
        let x = *pa.get(i).unwrap_or(&0);
        let y = *pb.get(i).unwrap_or(&0);
        if x != y {
            return x < y;
        }
    }
    false
}

/// The `/api/version` body (backend.php `fd_check_version`).
///
/// `client` is the APP SHELL that launched this engine (`RustServer.kt` passes the
/// APK's `versionName`), empty on a desktop build. `current` is this binary's own
/// version (`config::app_version()`, which IS `client` when one was supplied).
///
/// **The WordPress `min_version` may only gate a DESKTOP build.** It is a minimum
/// for the PHP ENGINE lineage (`2.7.9`), so an app shell — whose product version
/// is the APK's (`1.0.55`) — is ALWAYS "older" and the dashboard demanded an
/// upgrade to v2.7.9 on every install ("android app? its wrongly check v2.x.x
/// version"). An app-managed build reports `app_shell: true` and
/// `update_needed: false`; the APK has its own `apk-server-latest` updater.
fn version_answer(
    client: &str,
    current: &str,
    min_version: &str,
    update_url: &str,
    notes: &str,
) -> Value {
    let app_shell = !client.trim().is_empty();
    // Clean X.Y.Z for the dashboard badge; `current` may be a git stamp
    // (`2.8.9-4-g02e7dea`), which is not for humans.
    let display = crate::config::clean_version(if app_shell { client } else { current });
    let mut out = serde_json::json!({
        "ok": !min_version.is_empty(),
        "update_needed": false,
        "app_shell": app_shell,
        "current_version": current,
        "version": display,
        "client_version": client,
        "build": crate::config::build_id(),
    });
    if !min_version.is_empty() {
        out["minimum_version"] = serde_json::json!(min_version);
        out["update_url"] = serde_json::json!(update_url);
        out["release_notes"] = serde_json::json!(notes);
        // Informational only for an app shell: it is not comparable to the APK's
        // version, and the app updates itself.
        out["update_needed"] = serde_json::json!(!app_shell && version_less(current, min_version));
    }
    out
}

async fn check_version() -> Value {
    let client = crate::config::client_version();
    let current = app_version();
    // The WordPress body is already cached by `cached_wp_json`; this extra memo is
    // for the DERIVED answer (version comparison), so it is keyed `version`.
    if let Some(hit) = crate::cache::get("version", "check") {
        let mut v = (*hit).clone();
        let min = v
            .get("minimum_version")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string();
        // Everything that describes THIS process is recomputed on the way out, so
        // a cached body (TTL 1h) can never tell an app shell to update to the
        // ENGINE minimum, hide a desktop update, or report another run's version.
        if let Some(obj) = v.as_object_mut() {
            let app_shell = !client.trim().is_empty();
            obj.insert("client_version".to_string(), serde_json::json!(client));
            obj.insert(
                "build".to_string(),
                serde_json::json!(crate::config::build_id()),
            );
            obj.insert("app_shell".to_string(), serde_json::json!(app_shell));
            obj.insert(
                "current_version".to_string(),
                serde_json::json!(current.clone()),
            );
            obj.insert(
                "version".to_string(),
                serde_json::json!(crate::config::clean_version(if app_shell {
                    client.as_str()
                } else {
                    current.as_str()
                })),
            );
            obj.insert(
                "update_needed".to_string(),
                serde_json::json!(!app_shell && !min.is_empty() && version_less(&current, &min)),
            );
        }
        return v;
    }
    let r = cached_wp_json(&wp_version_url(), 3600, 8).await;
    let data = version_answer(
        &client,
        &current,
        &jstr(&r, "min_version"),
        &jstr(&r, "update_url"),
        &jstr(&r, "release_notes"),
    );
    crate::cache::put("version", "check", &data, 3600, 0);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `git describe` build must compare by its X.Y.Z, not by a failed parse.
    ///
    /// `3.0.5-14-gf154c62` used to read its patch component as 0, so it compared
    /// as 3.0.0 and claimed `update_needed: true` against the WordPress plugin's
    /// `3.0.4` minimum. Every build from a git checkout showed an update pill for
    /// the newest binary, and the cold-start gate failed its version-payload check.
    #[test]
    fn a_git_stamped_build_is_not_older_than_its_release() {
        assert!(!version_less("3.0.5-14-gf154c62", "3.0.4"));
        assert!(!version_less("3.0.4", "3.0.4"));
        assert!(version_less("3.0.3-4-gf839d36", "3.0.4"));
        assert!(version_less("v3.0.5", "3.0.6"));
        let desktop = version_answer("", "3.0.5-14-gf154c62", "3.0.4", "http://telegra.my", "");
        assert_eq!(
            desktop["update_needed"], false,
            "a build at or above the minimum must not be told to update: {desktop}"
        );
        assert_eq!(desktop["version"], "3.0.5", "the badge stays X.Y.Z");
    }

    #[test]
    fn lan_ip_accepts_only_rfc1918() {
        // A carrier address must never be advertised as the LAN IP.
        for bad in [
            "175.139.1.2",
            "8.8.8.8",
            "127.0.0.1",
            "169.254.10.1",
            "172.32.0.1",
            "192.169.1.1",
            "",
            "not-an-ip",
        ] {
            assert!(!super::is_private_ipv4(bad), "{bad} must be rejected");
        }
        for good in ["10.0.0.5", "172.16.4.9", "172.31.255.254", "192.168.2.118"] {
            assert!(super::is_private_ipv4(good), "{good} must be accepted");
        }
    }

    /// Real `ip -4 addr` from a OnePlus 12 (CPH2581) on mobile data only: no
    /// `wlan0` at all, a public carrier address on `rmnet_data3`, and OxygenOS's
    /// virtual gateway holding a private `/32`. The dashboard advertised the
    /// vgate address as "Wi-Fi / LAN" and the APK as "LAN: http://172.30.242.191:8088".
    const ONEPLUS_MOBILE_DATA: &str = "\
1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN group default qlen 1000
    inet 127.0.0.1/8 scope host lo
27: rmnet_data3@rmnet_ipa0: <UP,LOWER_UP> mtu 1500 qdisc mq state UNKNOWN
    inet 22.39.85.142/30 scope global rmnet_data3
32: vgate0: <POINTOPOINT,MULTICAST,NOARP,ALLMULTI,PROMISC,DEBUG,DYNAMIC,AUTOMEDIA,PORTSEL,UP,LOWER_UP> mtu 65500
    inet 172.30.242.191/32 scope global vgate0
";

    #[test]
    fn a_vendor_point_to_point_interface_is_never_the_lan_address() {
        // Nothing here is a LAN, so the caller must fall back to the page's host.
        assert_eq!(super::pick_lan_ip(None, ONEPLUS_MOBILE_DATA), "127.0.0.1");
        // Not even when the kernel reports the vgate address as the egress: the
        // egress only wins when its own interface is a usable LAN.
        assert_eq!(
            super::pick_lan_ip(Some("172.30.242.191"), ONEPLUS_MOBILE_DATA),
            "127.0.0.1"
        );
    }

    #[test]
    fn wifi_wins_over_a_private_carrier_address() {
        // A carrier that NATs on 10/8 passes the RFC1918 test, so the interface —
        // not the address — has to decide.
        const WIFI_AND_CELLULAR: &str = "\
27: rmnet_data0: <UP,LOWER_UP> mtu 1500
    inet 10.71.4.22/29 scope global rmnet_data0
32: vgate0: <POINTOPOINT,NOARP,UP,LOWER_UP> mtu 65500
    inet 172.30.242.191/32 scope global vgate0
9: wlan0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500
    inet 192.168.1.7/24 brd 192.168.1.255 scope global wlan0
";
        assert_eq!(
            super::pick_lan_ip(Some("192.168.1.7"), WIFI_AND_CELLULAR),
            "192.168.1.7"
        );
        assert_eq!(
            super::pick_lan_ip(Some("10.71.4.22"), WIFI_AND_CELLULAR),
            "192.168.1.7"
        );
        // No egress hint at all: the Wi-Fi address is still found.
        assert_eq!(super::pick_lan_ip(None, WIFI_AND_CELLULAR), "192.168.1.7");
    }

    #[test]
    fn a_hotspot_ap_interface_still_counts() {
        const HOTSPOT: &str = "\
14: ap0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500
    inet 192.168.43.1/24 brd 192.168.43.255 scope global ap0
";
        assert_eq!(super::pick_lan_ip(None, HOTSPOT), "192.168.43.1");
    }

    #[test]
    fn no_interface_table_falls_back_to_the_default_route() {
        // Windows and macOS have no `ip`; RFC1918 egress is all there is, which is
        // what the desktop builds have always advertised.
        assert_eq!(super::pick_lan_ip(Some("192.168.2.118"), ""), "192.168.2.118");
        assert_eq!(super::pick_lan_ip(None, ""), "127.0.0.1");
    }

    #[test]
    fn series_root_is_s01e01_like_the_php() {
        // `pm:post:<id>` with no `:S:E` resolves to `$targetSeason ?? 1` /
        // `$targetEpisode ?? 1`, so the ROOT must run the episode resolver too.
        assert_eq!(effective_season_episode(true, 0, 0), (1, 1));
        assert_eq!(effective_season_episode(true, 2, 0), (2, 1));
        assert_eq!(effective_season_episode(true, 1, 7), (1, 7));
        // A MOVIE keeps 0/0 so `build_streams_from_files()` takes the movie branch.
        assert_eq!(effective_season_episode(false, 0, 0), (0, 0));
    }

    #[test]
    fn episode_tokens_put_the_rare_conventions_first() {
        // The cap is spent on the conventions that return FEW results first.
        assert_eq!(
            episode_tokens(1, 1),
            vec!["EP1", "E1", "EP01", "E01", "S01E01"]
        );
        // Episode 12 is never written unpadded, so those forms are dropped.
        assert_eq!(episode_tokens(1, 12), vec!["EP12", "E12", "S01E12"]);
        assert!(episode_tokens(1, 0).is_empty());
    }

    #[test]
    fn probe_queries_mirror_the_php_ladder() {
        let q = episode_probe_queries("Agent Kim", "2026", 1, 1);
        assert_eq!(q[0], "Agent Kim EP1");
        // The year sits BETWEEN the title and the token in real release names, and
        // Manticore ANDs every term, so each token is probed with it as well.
        assert_eq!(q[1], "Agent Kim 2026 EP1");
        assert_eq!(q[2], "Agent Kim E1");
        assert!(q.contains(&"Agent Kim 2026 (EP1 | E1 | EP01 | E01 | S01E01)".to_string()));
        // The reversed group finds releases that put the episode tag FIRST.
        assert_eq!(q[q.len() - 1], "(EP1 | E1 | EP01 | E01 | S01E01) Agent Kim");
        // An unknown year adds no year-qualified probes.
        let no_year = episode_probe_queries("Agent Kim", "", 1, 1);
        assert!(!no_year.iter().any(|s| s.contains("2026")));
        assert_eq!(no_year[0], "Agent Kim EP1");
        // An apostrophe adds the possessive variants of the OR-group only.
        let apos = episode_probe_queries("Grey's Anatomy", "", 1, 1);
        assert!(apos.iter().any(|s| s.starts_with("Greys Anatomy (")));
        assert!(apos.iter().any(|s| s.starts_with("Grey Anatomy (")));
        // A title without an apostrophe gains no extra probes.
        assert_eq!(episode_probe_queries("Agent Kim", "", 1, 1).len(), no_year.len());
        // Nothing to probe -> no queries at all.
        assert!(episode_probe_queries("", "", 1, 1).is_empty());
    }

    #[test]
    fn base_catalog_id_normalises_sub_catalogs() {
        // A year/genre sub-catalog resolves through its base id, exactly like
        // the PHP's `$baseCatId` (backend.php:15865).
        assert_eq!(base_catalog_id("pm_movies_malay_2024"), "pm_movies_malay");
        assert_eq!(base_catalog_id("pm_series_malay_year"), "pm_series_malay");
        assert_eq!(base_catalog_id("pm_movies_indo_genre"), "pm_movies_indo");
        assert_eq!(base_catalog_id("pm_movies_malay_latest"), "pm_movies_malay");
        // Ids that only LOOK similar are left alone.
        assert_eq!(base_catalog_id("pm_latest_movies"), "pm_latest_movies");
        assert_eq!(base_catalog_id("pm_movies"), "pm_movies");
        assert_eq!(base_catalog_id("pm_trending_movies"), "pm_trending_movies");
    }

    #[test]
    fn country_ids_fall_back_to_the_country_parameter() {
        // Known country slugs whose category row does NOT exist in the map.
        assert_eq!(catalog_country_code("pm_movies_canada"), Some("CA"));
        assert_eq!(catalog_country_code("pm_series_vietnam"), Some("VN"));
        assert_eq!(catalog_country_code("pm_movies_us"), Some("US"));
        // The mapping also has to work for the ones the category map DOES cover,
        // because it is only consulted when that map misses.
        assert_eq!(catalog_country_code("pm_movies_malay"), Some("MY"));
        // Rows that are not `<type>_<country>` never match, so they keep their
        // existing behaviour.
        assert_eq!(catalog_country_code("pm_cat_action"), None);
        assert_eq!(catalog_country_code("pm_movies_top"), None);
        assert_eq!(catalog_country_code("top"), None);
        assert_eq!(catalog_country_code("pm_movies"), None);
    }

    #[test]
    fn trending_row_splits_by_type_and_dedupes() {
        let posts = vec![
            serde_json::json!({"id": 1, "title": "The Rookie 2018 • TvSeries"}),
            serde_json::json!({"id": 2, "title": "The Rookie 1990 • Movie"}),
            serde_json::json!({"id": 3, "title": "The Rookies 2019 • Movie"}),
            serde_json::json!({"id": 2, "title": "The Rookie 1990 • Movie"}),
            serde_json::json!({"title": "no id"}),
        ];
        let mut seen = std::collections::HashSet::new();
        let mut movies = Vec::new();
        push_trending_metas(&mut movies, &mut seen, &posts, "movie", 40);
        assert_eq!(movies.len(), 2);
        assert_eq!(movies[0]["id"], "pm:post:2");
        assert_eq!(movies[1]["id"], "pm:post:3");
        assert_eq!(movies[0]["type"], "movie");

        let mut seen = std::collections::HashSet::new();
        let mut series = Vec::new();
        push_trending_metas(&mut series, &mut seen, &posts, "series", 40);
        assert_eq!(series.len(), 1);
        assert_eq!(series[0]["id"], "pm:post:1");
        assert_eq!(series[0]["type"], "series");

        // The cap is honoured, and a row that only ever sees the other type
        // stays empty rather than borrowing the sibling's entries.
        let mut seen = std::collections::HashSet::new();
        let mut capped = Vec::new();
        push_trending_metas(&mut capped, &mut seen, &posts, "movie", 1);
        assert_eq!(capped.len(), 1);
        let mut seen = std::collections::HashSet::new();
        let only_movies = vec![serde_json::json!({"id": 9, "title": "Something 2020 • Movie"})];
        let mut empty = Vec::new();
        push_trending_metas(&mut empty, &mut seen, &only_movies, "series", 40);
        assert!(empty.is_empty());
    }

    #[test]
    fn catalog_options_are_a_grouped_map() {
        // The dashboard iterates `Object.entries(catalog_options)` and files
        // each row by `info.group` (public/app.js:700). A list-of-manifest-rows
        // shape made the panel render EMPTY, which is what this pins.
        let opts = default_catalog_options();
        let obj = opts.as_object().expect("options must be a map");
        assert!(obj.contains_key("top"), "PHP special row kept");
        assert!(obj.contains_key("pm_movies_malay"), "PHP country row kept");
        for (id, info) in obj {
            assert!(
                info.get("type").and_then(|v| v.as_str()).is_some()
                    && info.get("group").and_then(|v| v.as_str()).is_some()
                    && info.get("name").and_then(|v| v.as_str()).is_some(),
                "option {id} needs type/group/name"
            );
        }
        let tm = &opts["pm_trending_movies"];
        assert_eq!(tm["type"], "movie");
        assert_eq!(tm["group"], "trending");
        assert_eq!(tm["name"], "Trending Movies");
        let ts = &opts["pm_trending_series"];
        assert_eq!(ts["type"], "series");
        assert_eq!(ts["group"], "trending");
        // Every row the manifest advertises must have its own checkbox.
        for cat in pm_catalogs().as_array().cloned().unwrap_or_default() {
            let id = cat["id"].as_str().unwrap_or("");
            assert!(obj.contains_key(id), "no catalog option for {id}");
        }
    }

    /// A fresh install must NOT come up all-checked.
    ///
    /// backend.php:11964-11967 seeds `enabled_catalogs[$id] = !empty($info['default'])`
    /// for every option, so Korea / Japan / Thailand / Philippines (movie AND
    /// series) start UNCHECKED. The port used to ship `enabled_catalogs: {}` and
    /// lean on the "missing key means enabled" rule, which checked all of them.
    #[test]
    fn fresh_install_seeds_every_catalog_switch_from_its_default() {
        let settings = default_catalog_settings();
        let seeded = settings["enabled_catalogs"]
            .as_object()
            .expect("enabled_catalogs must be a seeded map, not {}");
        let opts = default_catalog_options();
        let opts = opts.as_object().expect("options map");

        // Every option is seeded explicitly — no gaps for the fallback rule.
        for id in opts.keys() {
            assert!(seeded.contains_key(id), "{id} was not seeded");
        }

        let off = [
            "pm_movies_korean",
            "pm_movies_japan",
            "pm_movies_thai",
            "pm_movies_philippines",
            "pm_series_japan",
            "pm_series_thai",
            "pm_series_philippines",
            // Off by request on top of the PHP's four: the Telegram-file "New"
            // row and the trending-keyword file rows. Both spend an origin scan
            // per boot and neither is a browsing surface people land on.
            "pm_files_year",
            "pm_trending_keywords",
        ];
        for id in off {
            assert_eq!(
                seeded[id].as_bool(),
                Some(false),
                "{id} must start unchecked (default: false in the PHP)"
            );
        }
        for id in [
            "top",
            "pm_series_top",
            "pm_search_files",
            "pm_trending_movies",
            "pm_movies_malay",
            "pm_movies_english",
            "pm_series_kdrama",
        ] {
            assert_eq!(seeded[id].as_bool(), Some(true), "{id} must start checked");
        }

        // …and the gate honours it: those four rows are not advertised on a
        // fresh install, while a default-on row is.
        assert!(!catalog_advertised(&settings, &row("movie", "pm_movies_korean")));
        assert!(!catalog_advertised(&settings, &row("series", "pm_series_thai")));
        assert!(catalog_advertised(&settings, &row("movie", "pm_movies_malay")));
    }

    fn catalog_rows() -> Vec<Value> {
        pm_catalogs().as_array().cloned().unwrap_or_default()
    }

    fn row(ty: &str, id: &str) -> Value {
        catalog_rows()
            .into_iter()
            .find(|c| c["type"] == ty && c["id"] == id)
            .unwrap_or_else(|| panic!("no {ty} row {id}"))
    }

    #[test]
    fn catalog_gate_matches_the_php_remaps() {
        // Fresh install: every row is advertised EXCEPT the ones the PHP ships
        // as `default: false` (backend.php:11764-11779 — Korea / Japan / Thailand
        // / Philippines, movie and series). This used to assert "every row",
        // which only held because `enabled_catalogs` was `{}` and a missing key
        // meant enabled; that is the bug this pins.
        let fresh = default_catalog_settings();
        let opts = default_catalog_options();
        for c in catalog_rows() {
            let id = c["id"].as_str().unwrap_or("");
            // `top` is shared by three rows, and the gate reads a DIFFERENT
            // switch for two of them (`pm_series_top`, `pm_files_year`), so the
            // expectation has to be remapped exactly like
            // [`catalog_advertised()`] remaps the check.
            let key = match (c["type"].as_str().unwrap_or(""), id) {
                ("series", "top") => "pm_series_top",
                ("other", "top") => "pm_files_year",
                _ => id,
            };
            let want = opts[key]["default"].as_bool().unwrap_or(true);
            assert_eq!(
                catalog_advertised(&fresh, &c),
                want,
                "fresh-install gate for {} {} must match its `default` flag",
                c["type"],
                id
            );
        }
        // A media type switched off hides only its own rows.
        let mut no_movies = default_catalog_settings();
        no_movies["enabled_types"]["movie"] = serde_json::json!(false);
        assert!(!catalog_advertised(&no_movies, &row("movie", "top")));
        assert!(catalog_advertised(&no_movies, &row("series", "top")));
        // An explicit per-row `false` hides just that row; `true` and an absent
        // key both keep it (the PHP's `!isset || !empty` rule).
        let mut hidden = default_catalog_settings();
        hidden["enabled_catalogs"]["pm_movies_malay"] = serde_json::json!(false);
        assert!(!catalog_advertised(&hidden, &row("movie", "pm_movies_malay")));
        assert!(catalog_advertised(&hidden, &row("movie", "top")));
        let mut shown = default_catalog_settings();
        shown["enabled_catalogs"]["pm_movies_malay"] = serde_json::json!(true);
        assert!(catalog_advertised(&shown, &row("movie", "pm_movies_malay")));
        // The series and Telegram-file `top` rows are gated by a DIFFERENT key
        // than their own id (backend.php:15323-15327).
        let mut series_top_off = default_catalog_settings();
        series_top_off["enabled_catalogs"]["pm_series_top"] = serde_json::json!(false);
        assert!(!catalog_advertised(&series_top_off, &row("series", "top")));
        assert!(catalog_advertised(&series_top_off, &row("movie", "top")));
        let mut files_off = default_catalog_settings();
        files_off["enabled_catalogs"]["pm_files_year"] = serde_json::json!(false);
        assert!(!catalog_advertised(&files_off, &row("other", "top")));
        assert!(catalog_advertised(&files_off, &row("other", "pm_search_files")));
        // One master toggle hides every `pm_topkw_*` row.
        // Keyword rows carry the PHP's quality `genre` list, the year list and
        // NO `extraSupported` / `extraRequired` (backend.php:15282-15291).
        let topkw = catalog_row(
            "other",
            "pm_topkw_0123456789",
            "Trending",
            vec![
                extra_entry("genre", None, false),
                extra_entry("year", None, false),
                extra_entry("skip", None, false),
            ],
            &[],
            &[],
        );
        assert!(topkw.get("extraSupported").is_none());
        assert!(topkw.get("extraRequired").is_none());
        // The master toggle now ships OFF — a fresh install does not spend an
        // origin scan on keyword file rows — and an explicit `true` brings the
        // whole `pm_topkw_*` family back.
        let mut kw_off = default_catalog_settings();
        kw_off["enabled_catalogs"]["pm_trending_keywords"] = serde_json::json!(false);
        assert!(!catalog_advertised(&kw_off, &topkw));
        assert!(!catalog_advertised(&default_catalog_settings(), &topkw));
        let mut kw_on = default_catalog_settings();
        kw_on["enabled_catalogs"]["pm_trending_keywords"] = serde_json::json!(true);
        assert!(catalog_advertised(&kw_on, &topkw));
        // The port's trending rows follow their own id.
        let mut trending_off = default_catalog_settings();
        trending_off["enabled_catalogs"][TRENDING_MOVIE_CATALOG] = serde_json::json!(false);
        assert!(!catalog_advertised(&trending_off, &row("movie", TRENDING_MOVIE_CATALOG)));
        assert!(catalog_advertised(&trending_off, &row("series", TRENDING_SERIES_CATALOG)));
    }

    #[test]
    fn descriptor_advertises_every_php_catalog_row() {
        // The PHP's `$manifestCatalogs` (backend.php:15135-15292): anything
        // missing here is a row every Stremio/Nuvio home screen loses.
        for (ty, id) in [
            ("movie", "top"),
            ("movie", "year"),
            ("movie", "pm_search_movie"),
            ("series", "top"),
            ("series", "pm_series_year"),
            ("series", "pm_search_series"),
            ("other", "top"),
            ("other", "pm_files_year"),
            ("other", "pm_search_files"),
        ] {
            row(ty, id); // panics when absent
        }
        for (id, _) in MOVIE_COUNTRY_CATALOGS {
            assert_eq!(row("movie", id)["id"], *id, "movie country row {id}");
        }
        for (id, _) in SERIES_COUNTRY_CATALOGS {
            assert_eq!(row("series", id)["id"], *id, "series country row {id}");
        }
        assert_eq!(catalog_rows().len(), 31, "13 movie + 13 series + 3 other + 2 trending");
        // A row that needs a value says so, or the client never sends it.
        assert_eq!(row("movie", "pm_search_movie")["extraRequired"][0], "search");
        assert_eq!(row("movie", "year")["extraRequired"][0], "genre");
        assert_eq!(row("series", "pm_search_series")["extraRequired"][0], "search");
        assert_eq!(row("other", "pm_search_files")["extraRequired"][0], "search");
        assert!(row("movie", "top").get("extraRequired").is_none());
        // Never a `genres` ARRAY on the catalog: Stremio caps the descriptor at
        // ~49 KB and the genre values belong in the `genre` extra.
        assert!(catalog_rows().iter().all(|c| c.get("genres").is_none()));
        // The option LISTS are the PHP's, untrimmed — they drive the Discover
        // filter pills. A country row carries `$filterOptionsCombined`, i.e. the
        // YEAR list FIRST and then the genres (backend.php:15134).
        assert_eq!(all_genre_options().len(), 22, "PHP `$allGenreOptions`");
        assert_eq!(
            all_year_options().len(),
            (current_year() - 1980 + 1) as usize,
            "PHP `$allYearOptions` = date('Y') down to 1980"
        );
        let country = row("movie", MOVIE_COUNTRY_CATALOGS[0].0);
        let genre_opts = country["extra"][0]["options"].as_array().unwrap();
        assert_eq!(
            genre_opts.len(),
            all_year_options().len() + all_genre_options().len(),
            "country `genre` extra = combined year+genre list"
        );
        assert_eq!(
            genre_opts[0].as_str(),
            Some(all_year_options()[0].as_str()),
            "years come first"
        );
        assert_eq!(
            genre_opts[all_year_options().len()].as_str(),
            Some(all_genre_options()[0].as_str()),
            "genres follow the years"
        );
        // The `year` / `pm_series_year` / `pm_files_year` rows carry the YEAR list
        // on their `genre` extra (backend.php:15154, 15208, 15260).
        for (ty, id) in [
            ("movie", "year"),
            ("series", "pm_series_year"),
            ("other", "pm_files_year"),
        ] {
            assert_eq!(
                row(ty, id)["extra"][0]["options"].as_array().unwrap().len(),
                all_year_options().len(),
                "{ty}/{id} `genre` extra is the year list"
            );
        }
        // `top` / `pm_search_*` expose the genre list on `genre` and the year
        // list on `year`.
        let top = row("movie", "top");
        assert_eq!(top["extra"][1]["options"].as_array().unwrap().len(), 22);
        assert_eq!(
            top["extra"][2]["options"].as_array().unwrap().len(),
            all_year_options().len()
        );
        // The local rows are the PHP's, so they are NOT small — but they must
        // still leave room for bridged catalogs under the 45 KB descriptor
        // budget (`upstream::fit_descriptor` drops bridged rows first, never a
        // local one).
        let local = pm_catalogs().to_string().len();
        assert!(
            local < 32_000,
            "local rows are {local} B; must leave room under the 45 KB budget"
        );
    }

    #[test]
    fn keyword_catalog_ids_match_the_php_md5_prefix() {
        // `substr(md5(strtolower($kw)), 0, 10)` — md5("abc") is the canonical
        // vector 900150983cd24fb0d6963f7d28e17f72.
        assert_eq!(topkw_id_suffix("abc"), "900150983c");
        // Case-folded, exactly like the PHP.
        assert_eq!(topkw_id_suffix("Malay"), topkw_id_suffix("malay"));
        assert_eq!(ucwords("kudrat 1968 high council"), "Kudrat 1968 High Council");
    }

    #[test]
    fn app_shell_never_gets_the_engine_minimum() {
        // Desktop: the WordPress minimum still gates.
        let desktop = version_answer("", "2.9.0-10-ge683767-dirty", "2.7.9", "http://telegra.my", "");
        assert_eq!(desktop["app_shell"], false);
        assert_eq!(desktop["update_needed"], false);
        assert_eq!(desktop["version"], "2.9.0");
        assert_eq!(desktop["ok"], true);
        let older = version_answer("", "2.8.0", "2.9.0", "http://telegra.my", "");
        assert_eq!(older["update_needed"], true);
        // App shell: 1.0.55 < 2.7.9 numerically, but the minimum is informational
        // (the APK has its own updater).
        let android = version_answer("1.0.55", "1.0.55", "2.7.9", "http://telegra.my", "");
        assert_eq!(android["app_shell"], true);
        assert_eq!(android["update_needed"], false);
        assert_eq!(android["version"], "1.0.55");
        assert_eq!(android["client_version"], "1.0.55");
        assert_eq!(android["minimum_version"], "2.7.9", "still reported");
        // No maximum to compare against at all.
        let no_min = version_answer("", "2.9.0", "", "", "");
        assert_eq!(no_min["ok"], false);
        assert_eq!(no_min["update_needed"], false);
    }

    #[test]
    fn descriptor_advertises_the_two_trending_rows() {
        let movies = row("movie", TRENDING_MOVIE_CATALOG);
        assert_eq!(movies["name"], "Trending Movies");
        assert_eq!(movies["extra"][0]["name"], "skip");
        let series = row("series", TRENDING_SERIES_CATALOG);
        assert_eq!(series["name"], "Trending Series");
        assert_eq!(series["extra"][0]["name"], "skip");
    }

    #[test]
    fn home_country_row_comes_first_inside_its_type() {
        let rows = pm_catalogs().as_array().cloned().unwrap_or_default();
        let mut settings = default_catalog_settings();
        settings["country"] = serde_json::json!("MY");
        let sorted = prioritise_home_country(rows, &settings);
        let first_movie = sorted.iter().find(|c| c["type"] == "movie").expect("a movie row");
        let first_series = sorted.iter().find(|c| c["type"] == "series").expect("a series row");
        assert_eq!(first_movie["id"], "pm_movies_malay");
        assert_eq!(first_series["id"], "pm_series_malay");
        // Types stay grouped movie → series → other, so a client's tab order holds.
        let types: Vec<&str> = sorted
            .iter()
            .map(|c| c["type"].as_str().unwrap_or(""))
            .collect();
        let mut seen: Vec<&str> = Vec::new();
        for t in types {
            if seen.last() != Some(&t) {
                assert!(!seen.contains(&t), "{t} rows must stay contiguous");
                seen.push(t);
            }
        }
    }

    #[test]
    fn external_id_prefixes_cover_every_php_resolvable_prefix() {
        // `tt…` keeps the full id and the S:E suffix.
        assert_eq!(
            parse_external_id("tt0944947:8:6"),
            Some(("imdb", "tt0944947".to_string(), 8, 6))
        );
        assert_eq!(
            parse_external_id("tt11540908:1:1"),
            Some(("imdb", "tt11540908".to_string(), 1, 1))
        );
        // An `tt` id shorter than 6 digits is not an IMDb id.
        assert_eq!(parse_external_id("tt123"), None);
        // The scalar prefixes, with and without `:S:E`.
        for (id, prefix, value) in [
            ("tmdb:123", "tmdb", "123"),
            ("kitsu:12:1", "kitsu", "12"),
            ("mal:5", "mal", "5"),
            // Task A: a bridged catalog row carrying one of these used to open
            // (meta bridge) and then answer the empty card, because the local
            // stream path never recognised the shape.
            ("tvdb:81797", "tvdb", "81797"),
            ("tvdb:81797:1:1", "tvdb", "81797"),
            ("anilist:21", "anilist", "21"),
            ("anilist:21:1:3", "anilist", "21"),
            ("tvmaze:82", "tvmaze", "82"),
            ("tvmaze:82:2:5", "tvmaze", "82"),
        ] {
            let parsed = parse_external_id(id).unwrap_or_else(|| panic!("{id} must parse"));
            assert_eq!(parsed.0, prefix, "{id} prefix");
            assert_eq!(parsed.1, value, "{id} value");
        }
        assert_eq!(
            parse_external_id("tvdb:81797:1:1"),
            Some(("tvdb", "81797".to_string(), 1, 1))
        );
        assert_eq!(
            parse_external_id("tvmaze:82:2:5"),
            Some(("tvmaze", "82".to_string(), 2, 5))
        );
        // A prefix with no value, an unknown prefix, and our own ids stay out.
        assert_eq!(parse_external_id("tvdb:"), None);
        assert_eq!(parse_external_id("anilist:"), None);
        assert_eq!(parse_external_id("foo:12"), None);
        assert_eq!(parse_external_id("pm:post:9000031194"), None);
        assert_eq!(parse_external_id("pm_file_1v9OPj"), None);
        assert_eq!(parse_external_id(""), None);
    }

    /* ── descriptor parity (backend.php:15303-15489) ──────────────────────── */

    fn ctx_with(headers: &[(&str, &str)], base: &str, query: &[(&str, &str)]) -> ReqCtx {
        let mut hm = axum::http::HeaderMap::new();
        for (k, v) in headers {
            hm.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                axum::http::HeaderValue::from_str(v).unwrap(),
            );
        }
        ReqCtx {
            method: "GET".into(),
            path: "/manifest.json".into(),
            raw_path: "/manifest.json".into(),
            query: query
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            headers: hm,
            cookies: HashMap::new(),
            remote: "127.0.0.1".into(),
            local: String::new(),
            base_url: base.to_string(),
        }
    }

    /* ── issue #9: every entry needs a `title` ────────────────────────────── */

    #[test]
    fn part_is_read_from_numbers_strings_and_absent_cleanly() {
        assert_eq!(part_of(&serde_json::json!({"part": 4})), 4);
        assert_eq!(part_of(&serde_json::json!({"part": "7"})), 7);
        assert_eq!(part_of(&serde_json::json!({})), 0);
        assert_eq!(part_of(&serde_json::json!({"part": null})), 0);
        assert_eq!(part_of(&serde_json::json!({"part": "unknown"})), 0);
        // A negative must never leak through as an ordinal.
        assert_eq!(part_of(&serde_json::json!({"part": -3})), 0);
    }

    fn item_json(item: &crate::matcher::FileItem) -> serde_json::Value {
        let ctx = ctx_with(&[], "http://127.0.0.1:8088", &[]);
        let name = "F1.The.Movie.2025.Hybrid.2160p.WEB.DL.DV.HDR.DDP5.1.Atmos.H265.AOC.mkv";
        stream_item(&ctx, "d63Zs9", name, "", 4_159_930_455, "video/x-matroska", item)
    }

    /// HARD RULE — a `/stream` item must NEVER carry a `title` field.
    ///
    /// stremio-core (the engine inside the Stremio web app AND the Stremio
    /// Android app) deserializes a stream into exactly
    /// `{ name, description, url | externalUrl | infoHash, behaviorHints }`.
    /// A row carrying an unknown `title` was DROPPED, so a 100-row list collapsed
    /// to the only row without one — the sponsored item — even though the JSON
    /// was valid addon JSON and Nuvio rendered all 100 rows. Reproduced and fixed
    /// on 2026-10-26 (`7f76825`, then the title removal): the field was added for
    /// issue #9 and is the single reason "streams list for stremio is broken,
    /// only show sponsored message" happened.
    ///
    /// Do NOT re-add it "because issue #9 asked for a title": that text already
    /// lives on line 1 of `description`, which every client renders, plus the
    /// pill line. If a future client needs a second line, extend `description`.
    #[test]
    fn a_stream_item_never_carries_a_title_field() {
        let item = crate::matcher::FileItem {
            title: "Movie.2026.1080p.mkv".to_string(),
            duration: 5400,
            ..Default::default()
        };
        let j = item_json(&item);
        let mut keys: Vec<&str> = j
            .as_object()
            .expect("a stream item is a JSON object")
            .keys()
            .map(|k| k.as_str())
            .collect();
        keys.sort_unstable();
        assert!(
            !keys.contains(&"title"),
            "stremio-core DROPS any stream carrying `title` — the list then shows \
             only the sponsored item. Keys were {keys:?}"
        );
        // The exact model stremio-core accepts. Anything outside it risks the
        // same silent drop, so a new field must be a deliberate decision here.
        assert_eq!(
            keys,
            vec!["behaviorHints", "description", "name", "url"],
            "stream keys must match stremio-core's model exactly"
        );
        // …and the text issue #9 wanted is still there, on line 1 / line 2.
        // (`item_json()` builds every item from the same F1 file name.)
        assert!(first_line_of(&item).starts_with("F1.The.Movie"), "{}", first_line_of(&item));
        assert!(pills_of(&item).contains("💾 "), "size pill missing");
        assert!(pills_of(&item).contains("⏱ 1h 30m"), "runtime pill missing");
    }

    /// Line 2 of `description` — the pill line (size, runtime, source, codec).
    fn pills_of(item: &crate::matcher::FileItem) -> String {
        item_json(item)
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .lines()
            .nth(1)
            .unwrap_or("")
            .to_string()
    }

    /// Stremio — the web app AND the Android app — HIDES every stream that
    /// carries `behaviorHints.notWebReady`, so a Matroska-only title listed the
    /// sponsored item and nothing else (reported as "streams list for stremio is
    /// broken, only show sponsored message" while Nuvio showed all 100 rows).
    /// The PHP never sent the flag on a playable URL; neither may we.
    #[test]
    fn a_playable_stream_never_carries_not_web_ready() {
        let item = crate::matcher::FileItem {
            title: "Movie.2026.1080p.mkv".to_string(),
            ..Default::default()
        };
        let j = item_json(&item);
        let hints = &j["behaviorHints"];
        assert!(
            hints.get("notWebReady").is_none(),
            "an MKV row must stay visible in Stremio: {hints}"
        );
        // The container is still advertised, which is what a player decides from.
        assert!(hints["filename"]
            .as_str()
            .unwrap_or("")
            .to_ascii_lowercase()
            .ends_with(".mkv"));
        assert!(j["url"].as_str().unwrap_or("").contains("/api/download/"));
    }

    /// Line 1 of `description` — the line the Nuvio app renders large.
    fn first_line_of(item: &crate::matcher::FileItem) -> String {
        item_json(item)
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .lines()
            .next()
            .unwrap_or("")
            .to_string()
    }

    #[test]
    fn stream_item_carries_a_runtime_and_a_part_marker() {
        use crate::matcher::FileItem;

        // A whole file: name · size · runtime · tags. The runtime is INTENTIONALLY
        // here and not only on parts — a normal file's runtime is metadata too
        // (compare a 2.5 GB copy against a 25 GB one).
        let whole = FileItem {
            title: "x".into(),
            duration: 9332,
            ..Default::default()
        };
        let pills = pills_of(&whole);
        assert!(pills.contains("💾 3.9 GB"), "size belongs on the pill line: {pills}");
        assert!(pills.contains("⏱ 2h 36m"), "runtime belongs on the pill line: {pills}");
        assert!(!pills.contains("Part"), "a whole file is not a part: {pills}");
        assert!(!pills.contains("Split"), "a whole file is not a split: {pills}");
        assert!(!first_line_of(&whole).contains('·'), "no badge on a whole file");

        // A part whose ordinal is known (name carried `partNNN`). The marker goes
        // on the FILE-NAME line because that is the line Nuvio renders large.
        let known = FileItem {
            title: "x".into(),
            duration: 1401,
            is_split_part: true,
            part_num: 4,
            total_parts: 7,
            ..Default::default()
        };
        assert!(
            first_line_of(&known).contains("🧩 Part 4/7"),
            "the prominent line must carry the ordinal: {}",
            first_line_of(&known)
        );
        assert!(pills_of(&known).contains("⏱ 23 min"), "chunk runtime");

        // A part detected from DURATION only — same name, no ordinal, count known.
        // "Part 0/7" would be a lie, so it states the shape instead.
        let anonymous = FileItem {
            title: "x".into(),
            duration: 1323,
            is_split_part: true,
            total_parts: 7,
            ..Default::default()
        };
        assert!(first_line_of(&anonymous).contains("🧩 Split · 7 parts"));
        assert!(!pills_of(&anonymous).contains("Part 0"), "never invent an ordinal");

        // Detected as a chunk with NEITHER count nor position — the badge must
        // still appear (that is the whole point of the issue).
        let unknown = FileItem {
            title: "x".into(),
            duration: 1185,
            is_split_part: true,
            ..Default::default()
        };
        assert!(first_line_of(&unknown).contains("🧩 Split · order unknown"));
        assert!(pills_of(&unknown).contains("⏱ 20 min"), "chunk runtime");

        // An ordinal but no known total (caption carried `part005` only).
        let single = FileItem {
            title: "x".into(),
            is_split_part: true,
            part_num: 5,
            ..Default::default()
        };
        assert!(first_line_of(&single).contains("🧩 Part 5"), "single known part");
        assert!(!pills_of(&single).contains("Part 5/0"), "no fake total");

        // The pill line must NOT repeat the badge.
        let pills = item_json(&known)
            .get("description")
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .lines()
            .nth(1)
            .unwrap_or("")
            .to_string();
        assert!(!pills.contains("🧩"), "the badge is on line 1 only: {pills}");
    }

    #[test]
    fn active_types_follow_enabled_types_with_a_three_type_fallback() {
        assert_eq!(
            active_types(&default_catalog_settings()).join(","),
            "movie,series,other"
        );
        let mut no_movies = default_catalog_settings();
        no_movies["enabled_types"]["movie"] = serde_json::json!(false);
        assert_eq!(active_types(&no_movies).join(","), "series,other");
        // backend.php:15341 — an empty result falls back to ALL THREE.
        let mut none = default_catalog_settings();
        for t in ["movie", "series", "other"] {
            none["enabled_types"][t] = serde_json::json!(false);
        }
        assert_eq!(active_types(&none).join(","), "movie,series,other");
        // No map at all is the PHP's `?? [all true]` default.
        assert_eq!(
            active_types(&serde_json::json!({})).join(","),
            "movie,series,other"
        );
        // PHP `!empty()`: a missing key INSIDE an existing map is off.
        assert_eq!(
            active_types(&serde_json::json!({"enabled_types": {"movie": true}})).join(","),
            "movie"
        );
        // `1` and `"1"` are true to PHP, `0` is false.
        assert_eq!(
            active_types(&serde_json::json!({
                "enabled_types": {"movie": 1, "series": "1", "other": 0}
            }))
            .join(","),
            "movie,series"
        );
    }

    #[test]
    fn manifest_resources_are_objects_scoped_to_the_active_types() {
        let types: Vec<String> = ["movie", "series", "other"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let with = manifest_resources(&types, true);
        assert_eq!(with.len(), 4);
        assert!(with.iter().all(|r| r.is_object()), "resources are objects");
        assert_eq!(with[0]["name"], "catalog");
        assert_eq!(with[0]["types"], serde_json::json!(types));
        assert!(with[0].get("idPrefixes").is_none(), "catalog has no idPrefixes");
        for r in &with[1..] {
            assert_eq!(r["types"], serde_json::json!(types));
            assert_eq!(r["idPrefixes"], serde_json::json!(ID_PREFIXES));
        }
        let names: Vec<&str> = with.iter().map(|r| r["name"].as_str().unwrap_or("")).collect();
        assert_eq!(names, ["catalog", "meta", "stream", "subtitles"]);
        // No catalogs → no `catalog` resource at all (backend.php:15418).
        let without = manifest_resources(&types, false);
        assert_eq!(without.len(), 3);
        assert!(without.iter().all(|r| r["name"] != "catalog"));
        // Only the ENABLED types are advertised (backend.php:15421).
        let only_series = manifest_resources(&["series".to_string()], true);
        assert_eq!(only_series[0]["types"], serde_json::json!(["series"]));
        // The PHP's order: `pm_` BEFORE `pm:` (backend.php:15458).
        assert_eq!(
            ID_PREFIXES.to_vec(),
            vec![
                "pm_",
                "pm:",
                "tt",
                "tmdb:",
                "kitsu:",
                "kitsu",
                "mal:",
                "anilist:",
                "tvdb:"
            ]
        );
    }

    #[test]
    fn manifest_response_sets_an_etag_and_answers_304() {
        let doc = serde_json::json!({"id": "org.pencarimovie.addon.localhost", "resources": []});
        let first = manifest_response(&ctx_with(&[], "http://127.0.0.1:8088", &[]), &doc);
        assert_eq!(first.status(), axum::http::StatusCode::OK);
        assert_eq!(
            first
                .headers()
                .get(axum::http::header::CACHE_CONTROL)
                .unwrap(),
            "max-age=1800, stale-while-revalidate=3600"
        );
        let etag = first
            .headers()
            .get(axum::http::header::ETAG)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        assert!(etag.starts_with('"') && etag.ends_with('"'), "{etag}");
        // A matching `If-None-Match` (quoted and bare) is a 304 carrying the tag.
        for hv in [etag.clone(), etag.trim_matches('"').to_string()] {
            let cached = manifest_response(
                &ctx_with(&[("if-none-match", &hv)], "http://127.0.0.1:8088", &[]),
                &doc,
            );
            assert_eq!(cached.status(), axum::http::StatusCode::NOT_MODIFIED);
            assert_eq!(
                cached.headers().get(axum::http::header::ETAG).unwrap(),
                etag.as_str()
            );
            assert_eq!(
                cached
                    .headers()
                    .get(axum::http::header::ACCESS_CONTROL_ALLOW_ORIGIN)
                    .unwrap(),
                "*"
            );
        }
        // A different tag still gets the body.
        let other = manifest_response(
            &ctx_with(
                &[("if-none-match", "\"deadbeef\"")],
                "http://127.0.0.1:8088",
                &[],
            ),
            &doc,
        );
        assert_eq!(other.status(), axum::http::StatusCode::OK);
        // The same document always hashes to the same tag.
        let again = manifest_response(&ctx_with(&[], "http://127.0.0.1:8088", &[]), &doc);
        assert_eq!(
            again.headers().get(axum::http::header::ETAG).unwrap(),
            etag.as_str()
        );
    }

    #[test]
    fn manifest_identity_names_the_address_it_was_installed_from() {
        let local = ctx_with(&[("host", "127.0.0.1:8088")], "http://127.0.0.1:8088", &[]);
        let (mode, desc) = manifest_identity(&local, "http://127.0.0.1:8088");
        assert_eq!(mode, "localhost");
        assert!(desc.contains("http://127.0.0.1:8088"), "{desc}");
        assert!(desc.contains("this device only"), "{desc}");
        // A LAN address is its own mode.
        let lan = ctx_with(&[("host", "192.168.1.5:8088")], "http://192.168.1.5:8088", &[]);
        let (mode, desc) = manifest_identity(&lan, "http://192.168.1.5:8088");
        assert_eq!(mode, "lan");
        assert!(desc.contains("192.168.1.5"), "{desc}");
        // A tunnel hostname wins over the IP check, and names the HTTPS address.
        let tunnel = ctx_with(
            &[("host", "abc-tunnel.pencarimovie.com")],
            "https://abc-tunnel.pencarimovie.com",
            &[],
        );
        let (mode, desc) = manifest_identity(&tunnel, "https://abc-tunnel.pencarimovie.com");
        assert_eq!(mode, "tunnel");
        assert!(desc.contains("https://abc-tunnel.pencarimovie.com"), "{desc}");
        // `?mode=` forces the identity so a tunneled page can still install an
        // HTTP transport URL.
        let forced = ctx_with(
            &[("host", "127.0.0.1:8088")],
            "http://127.0.0.1:8088",
            &[("mode", "lan")],
        );
        let (mode, desc) = manifest_identity(&forced, "http://127.0.0.1:8088");
        assert_eq!(mode, "lan");
        assert!(desc.contains("http://127.0.0.1:8088"), "{desc}");
        // A public host that is not a tunnel is a plain server.
        let server = ctx_with(&[("host", "vps.example.com")], "http://vps.example.com", &[]);
        let (mode, _desc) = manifest_identity(&server, "http://vps.example.com");
        assert_eq!(mode, "server");
    }

    #[test]
    fn file_pills_use_the_php_word_boundaries() {
        // A standalone `hd` / `sd` IS a resolution pill in PHP (`\b(720p|hd)\b`).
        assert_eq!(file_pills("Movie.2026.HD.mkv", 0), vec!["720p"]);
        assert_eq!(file_pills("Movie.2026.SD.mkv", 0), vec!["SD"]);
        // ... but never inside a word.
        assert!(file_pills("HDU.Release.mkv", 0).is_empty());
        assert!(file_pills("Shadow.Release.mkv", 0).is_empty());
        assert_eq!(file_pills("Movie.2026.2160p.mkv", 0), vec!["4K"]);
        assert_eq!(
            file_pills("Movie.4K.WEB-DL.x265.mkv", 0),
            vec!["4K", "WEB-DL", "HEVC"]
        );
        assert_eq!(
            file_pills("Movie.1080p.BluRay.HEVC.mkv", 0),
            vec!["1080p", "BluRay", "HEVC"]
        );
        // One resolution, one source, one codec, then the size pill.
        assert_eq!(
            file_pills("Movie.720p.mkv", 1_500_000_000),
            vec!["720p".to_string(), format_bytes(1_500_000_000)]
        );
        assert!(file_pills("Untagged.mkv", 0).is_empty());
    }

    #[test]
    fn file_meta_pills_use_the_php_meta_set() {
        // The META path adds `4K UHD`, `HDTV`, `HDR` and `AVC` (backend.php:16129).
        assert_eq!(
            file_meta_pills("Movie.2160p.HDR.x264.mkv", 0),
            vec!["4K UHD", "HDR", "AVC"]
        );
        assert_eq!(
            file_meta_pills("Movie.720p.HDTV.mkv", 0),
            vec!["720p", "HDTV"]
        );
        assert_eq!(
            file_meta_pills("Movie.1080p.WEB-DL.x265.mkv", 0),
            vec!["1080p", "WEB-DL", "HEVC"]
        );
    }

    #[test]
    fn update_notice_is_a_non_blocking_item() {
        let check = serde_json::json!({
            "update_needed": true,
            "current_version": "2.8.8-10-gabc-dirty",
            "minimum_version": "v2.9.3",
            "update_url": "https://example.test/download",
        });
        let desktop = build_update_notice(&check, "http://127.0.0.1:8088");
        let desc = desktop["description"].as_str().unwrap();
        assert!(desc.contains("v2.9.3"), "{desc}");
        // The install version is shown clean, never the git stamp.
        assert!(desc.contains("v2.8.8"), "{desc}");
        assert!(!desc.contains("-10-g"), "{desc}");
        // The link is THIS server even though the payload carries a custom
        // update_url: the dashboard card names the update path for this host, and
        // the reader already reached this server to get the stream.
        let base = serde_json::json!("http://127.0.0.1:8088/");
        assert_eq!(desktop["externalUrl"], base);
        // ONE line, one full stop, no instructions. The old text carried "the
        // streams below still work" (it described the list the reader was looking
        // at, and needed `streams_below` plumbing through three functions) and then
        // a per-host "To update, ..." hint. The notice notifies; the page behind the
        // link explains how.
        assert_eq!(desc.matches('\n').count(), 0, "{desc}");
        assert!(desc.ends_with('.'), "{desc}");
        assert!(!desc.contains("still work"), "{desc}");
        assert!(!desc.contains("To update"), "{desc}");
        // Same text AND same destination on every host: the dashboard card decides
        // what "update" means on this machine.
        let container = build_update_notice(&check, "http://127.0.0.1:8088");
        assert_eq!(container["description"].as_str().unwrap(), desc);
        assert_eq!(container["externalUrl"], base);
        let self_update = serde_json::json!({
            "current_display": "2.8.8",
            "latest_display": "2.9.3",
            "can_apply": true,
        });
        let card = build_update_notice(&self_update, "http://127.0.0.1:8088/");
        assert_eq!(card["externalUrl"], base);
        assert_eq!(card["description"].as_str().unwrap(), desc);

        let manual = serde_json::json!({
            "current_display": "2.8.8",
            "latest_display": "2.9.3",
            "can_apply": false,
            "notes_url": "https://github.com/aiskendi/pencarimovie-server/releases/tag/v2.9.3",
        });
        let manual_card = build_update_notice(&manual, "http://127.0.0.1:8088/");
        assert_eq!(manual_card["externalUrl"], base);
        assert_eq!(manual_card["description"].as_str().unwrap(), desc);

        // Also accepts the /api/update/check feed payload format (like web #settings / #addon)
        let check_feed = serde_json::json!({
            "ok": true,
            "update_available": true,
            "current_display": "2.8.8",
            "latest_display": "2.9.0",
            "releases_url": "https://github.com/aiskendi/pencarimovie-server/releases/latest"
        });
        let desktop_feed = build_update_notice(&check_feed, "http://127.0.0.1:8088");
        let desc_feed = desktop_feed["description"].as_str().unwrap();
        assert!(desc_feed.contains("v2.9.0"), "{desc_feed}");
        assert!(desc_feed.contains("v2.8.8"), "{desc_feed}");

        // Android app_managed feed payload
        let android_feed = serde_json::json!({
            "ok": true,
            "app_managed": true,
            "update_available": true,
            "current_display": "1.0.71",
            "latest_display": "1.0.72",
            "releases_url": "https://github.com/aiskendi/pencarimovie-server/releases/download/apk-server-latest/pencarimovie-server_arm64-v8a.apk"
        });
        let android_card = build_update_notice(&android_feed, "http://127.0.0.1:8088");
        let and_desc = android_card["description"].as_str().unwrap();
        assert!(and_desc.contains("v1.0.72"), "{and_desc}");
        assert!(and_desc.contains("v1.0.71"), "{and_desc}");
        assert_eq!(android_card["externalUrl"], base);

        // Placement: sponsor at [0], update notice at [1], then the media rows.
        // This is `with_top_items()`, the ONE placement every `/stream` answer
        // goes through (real list, empty list, single card, locked addon).
        let placed = with_top_items(
            vec![serde_json::json!({"name": "Movie.1080p.mkv"})],
            Some(desktop_feed),
            Some(serde_json::json!({"name": "⭐️ Sponsored message"})),
        );
        assert_eq!(placed[0]["name"], "⭐️ Sponsored message");
        assert_eq!(placed[1]["name"], "⬆️ Update available");
        assert_eq!(placed[2]["name"], "Movie.1080p.mkv");
    }

    /// The sponsor is [0] even when the list below it is a single card — the
    /// locked "Addon URL changed or password required" answer, "No Streams
    /// Found", "File could not be resolved". This is the bug report: "the
    /// sponsored message is not always on top" / "when the URL changed or
    /// password message appears the sponsored message is gone".
    #[test]
    fn sponsor_is_first_above_a_single_card_and_an_empty_list() {
        let sponsor = serde_json::json!({"name": "⭐️ Sponsored message"});
        let card = serde_json::json!({"name": "PencariMovie", "description": "locked"});

        let one = with_top_items(vec![card.clone()], None, Some(sponsor.clone()));
        assert_eq!(one.len(), 2);
        assert_eq!(one[0]["name"], "⭐️ Sponsored message");
        assert_eq!(one[1]["description"], "locked");

        let none = with_top_items(Vec::new(), None, Some(sponsor));
        assert_eq!(none.len(), 1);
        assert_eq!(none[0]["name"], "⭐️ Sponsored message");

        // Nothing configured, nothing invented: the list is untouched.
        let bare = with_top_items(vec![card], None, None);
        assert_eq!(bare.len(), 1);
        assert_eq!(bare[0]["description"], "locked");
    }

    #[test]
    fn filtered_out_message_names_the_active_filters() {
        // A leftover `required_keywords` is the single most confusing cause: the
        // card must say so instead of a generic "adjust filters".
        let sc = serde_json::json!({
            "required_keywords": "avengers",
            "excluded_keywords": "",
            "max_size_gb": 5,
            "max_streams_total": 0,
            "exclude_cam": false,
            "resolutions": {"4k": false, "1080p": true},
            "qualities": {"cam": true},
            "encodes": {"hevc": true},
            "visual_tags": {"dv": false}
        });
        let notes = active_filter_notes(&sc);
        assert!(notes.iter().any(|n| n.contains("required keywords \"avengers\"")), "{notes:?}");
        assert!(notes.iter().any(|n| n.contains("max size 5 GB")), "{notes:?}");
        assert!(notes.iter().any(|n| n.contains("resolution off: 4k")), "{notes:?}");
        assert!(notes.iter().any(|n| n.contains("tag off: dv")), "{notes:?}");
        // Shipped defaults restrict nothing, so the card keeps the generic text.
        assert!(active_filter_notes(&default_catalog_settings()["stream_config"]).is_empty());
    }

    /// An origin probe must send a BARE id, and only when the origin can resolve
    /// that prefix from an id at all (live measurements in [`origin_resolves_by_id`]).
    #[test]
    fn an_origin_probe_uses_a_bare_id_and_only_for_resolvable_prefixes() {
        // `parse_external_id` names the imdb prefix "imdb" and keeps the `tt`.
        assert_eq!(origin_probe_id("imdb", "tt0944947"), "tt0944947");
        assert_eq!(origin_probe_id("mal", "54492"), "mal:54492");
        assert_eq!(origin_probe_id("tmdb", "1399"), "tmdb:1399");
        assert!(origin_resolves_by_id("imdb"));
        assert!(origin_resolves_by_id("tmdb"));
        for prefix in ["mal", "kitsu", "anilist", "tvdb", "tvmaze"] {
            assert!(
                !origin_resolves_by_id(prefix),
                "{prefix} answers 400 `title or id required` on the origin"
            );
        }
    }

    #[test]
    fn catalog_settings_env_layer_is_the_base() {
        // The env layer supplies what an ephemeral host would otherwise lose ...
        let merged = merge_settings_layers(
            Some(serde_json::json!({"catalogs_enabled": false, "country": "MY"})),
            Some(serde_json::json!({"country": "ID"})),
        );
        assert_eq!(merged["catalogs_enabled"], serde_json::json!(false));
        // ... and a file written by the dashboard still wins over it.
        assert_eq!(merged["country"], serde_json::json!("ID"));
        assert_eq!(merge_settings_layers(None, None), serde_json::json!({}));
        // A malformed / non-object env value must not blank every setting.
        assert_eq!(
            merge_settings_layers(Some(serde_json::json!("nope")), None),
            serde_json::json!({})
        );
    }

    /// A dropped split part must never lend its NAME to a row that survived.
    ///
    /// `group_split_parts` reorders its input (the PHP sorts in place), while the
    /// route keeps the source rows in a parallel vector. Real shape from
    /// `pm:post:9000009680` — split groups scattered among whole files — the
    /// pairing used to break here: 77 parts were flagged and filtered out, yet 16
    /// part names still reached the player, each sitting on another file's
    /// payload. The permutation is what keeps `rows[i]` and `items[i]` the same
    /// file.
    #[test]
    fn a_dropped_split_part_never_lends_its_name_to_a_kept_row() {
        let names = [
            "Movie.2026.4K.HDR.part003.mkv",
            "Movie.2026.1080p.WEB-DL.mkv",
            "Movie.2026.4K.SDR.part001.mkv",
            "Movie.2026.720p.WEB-DL.mkv",
            "Movie.2026.4K.HDR.part001.mkv",
            "Movie.2026.4K.SDR.part002.mkv",
            "Movie.2026.4K.HDR.part002.mkv",
        ];
        let mut items: Vec<crate::matcher::FileItem> = names
            .iter()
            .map(|n| crate::matcher::FileItem {
                title: n.to_string(),
                ..Default::default()
            })
            .collect();
        let mut rows: Vec<String> = names.iter().map(|n| n.to_string()).collect();
        let perm = crate::matcher::group_split_parts(&mut items);
        assert_eq!(perm.len(), rows.len());
        rows = crate::matcher::reorder_with(&rows, &perm);

        let kept = crate::matcher::apply_stream_filters(
            &items,
            &crate::matcher::StreamFilterConfig::default(),
        );
        assert_eq!(kept.len(), 2, "only the two whole files survive");
        for (idx, _) in kept {
            assert_eq!(rows[idx], items[idx].title, "row {idx} lost its pairing");
            assert!(
                !crate::matcher::extract_split_part_info(&rows[idx], "").is_part,
                "{} is a split part",
                rows[idx]
            );
        }
    }

    #[test]
    fn iso_from_unix_is_rfc3339_utc() {
        assert_eq!(iso_from_unix(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso_from_unix(1_700_000_000), "2023-11-14T22:13:20Z");
        let s = iso_from_unix(crate::storage::now());
        assert_eq!(s.len(), 20, "{s}");
        assert!(s.ends_with('Z'), "{s}");
        assert_eq!(&s[4..5], "-");
        assert_eq!(&s[7..8], "-");
        assert_eq!(&s[10..11], "T");
        assert_eq!(&s[13..14], ":");
        assert_eq!(&s[16..17], ":");
        assert!(s[11..13].parse::<u32>().unwrap() < 24, "{s}");
        assert!(s[14..16].parse::<u32>().unwrap() < 60, "{s}");
    }

    #[test]
    fn iso_from_wp_date_reads_the_origin_format() {
        assert_eq!(
            iso_from_wp_date("2024-02-19 20:01:04").as_deref(),
            Some("2024-02-19T20:01:04Z")
        );
        assert_eq!(
            iso_from_wp_date("2018-05-02T09:07:06").as_deref(),
            Some("2018-05-02T09:07:06Z")
        );
        // Date only -> midnight.
        assert_eq!(
            iso_from_wp_date("2018-05-02").as_deref(),
            Some("2018-05-02T00:00:00Z")
        );
        // Junk months/days and non-dates are refused, not guessed.
        assert_eq!(iso_from_wp_date("2024-13-01 00:00:00"), None);
        assert_eq!(iso_from_wp_date("2024-02-32 00:00:00"), None);
        assert_eq!(iso_from_wp_date(""), None);
        assert_eq!(iso_from_wp_date("not a date"), None);
    }

    /// Issue #10 bug 1: `released` came from `Utc::now()`, so it changed on
    /// every request and every episode of a 2018 series claimed "just now".
    /// It must come from stored data and be identical across calls.
    #[test]
    fn episode_released_is_stored_not_the_wall_clock() {
        use crate::wp_data::WpFile;
        let files = vec![
            WpFile {
                title: "Welcome.To.Waikiki.2018.S01E02.720p.WEBRip.mkv".into(),
                added_date: 1_792_294_706,
                ..Default::default()
            },
            // No `added_date`: falls back to the series anchor, never to now.
            WpFile {
                title: "Welcome.To.Waikiki.2018.S01E01.720p.WEBRip.mkv".into(),
                ..Default::default()
            },
        ];
        let poster = serde_json::json!("https://example.invalid/p.jpg");
        let anchor = Some("2018-01-01T00:00:00Z");
        let a = series_videos("pm:post:1", &files, &poster, anchor);
        let b = series_videos("pm:post:1", &files, &poster, anchor);
        assert_eq!(a, b, "the videos list must be stable across calls");
        // Sorted (S01E01 first), so index 1 is the row that HAS an added_date.
        assert_eq!(a[0]["released"], serde_json::json!("2018-01-01T00:00:00Z"));
        assert_eq!(a[1]["released"], serde_json::json!("2026-10-18T03:38:26Z"));
        // The stored date is used, not the request clock.
        assert_ne!(
            a[1]["released"],
            serde_json::json!(iso_from_unix(crate::storage::now()))
        );

        // No anchor at all -> `null`, not a fabricated date.
        let no_anchor = series_videos("pm:post:1", &files, &poster, None);
        assert_eq!(no_anchor[0]["released"], Value::Null);
    }

    /// Issue #10 bug 2: the PHP sorts by (season, episode) and the port had
    /// dropped the sort, so 11 of 12 K-drama series came back scrambled or
    /// descending. The scan order must not survive into the response.
    /// The origin index holds the whole show; a POST holds only its slice.
    /// `pm:post:9000018021` ("To My Beloved Thief 2026") owns E03–E14 and E16 —
    /// the 13 rows the origin's `post-files` returns — so E01, E02 and E15 had
    /// no entry in the picker while `/stream` answered all three.
    #[test]
    fn episode_gaps_are_the_numbers_between_what_the_post_owns() {
        use crate::wp_data::WpFile;

        let owned: Vec<WpFile> = (3..=14)
            .chain(std::iter::once(16))
            .map(|e| WpFile {
                title: format!("OLD.E{:02}.To.My.Beloved.Thief.2026.mp4", e),
                ..Default::default()
            })
            .collect();
        let gaps: Vec<(i32, i32)> = episode_gaps(&owned).into_iter().collect();
        assert_eq!(gaps, vec![(1, 1), (1, 2), (1, 15)]);

        // A post that covers its own numbering spends nothing on a search.
        let complete: Vec<WpFile> = (1..=16)
            .map(|e| WpFile {
                title: format!("Show.S01E{:02}.1080p.mkv", e),
                ..Default::default()
            })
            .collect();
        assert!(episode_gaps(&complete).is_empty());

        // No resolvable episode at all (a movie-ish or pack-only row set) has no
        // numbering to complete, so the guard must not fire.
        assert!(episode_gaps(&[]).is_empty());
        assert!(episode_gaps(&[WpFile {
            title: "Show.Complete.Series.mkv".to_string(),
            ..Default::default()
        }])
        .is_empty());

        // Gaps are per SEASON: S1E01..E03 plus S2E02 leaves S2E01 missing and
        // never reports S1/S2 cross-talk.
        let two_seasons: Vec<WpFile> = ["Show.S01E01.mkv", "Show.S01E02.mkv", "Show.S01E03.mkv", "Show.S02E02.mkv"]
            .iter()
            .map(|t| WpFile {
                title: (*t).to_string(),
                ..Default::default()
            })
            .collect();
        let gaps: Vec<(i32, i32)> = episode_gaps(&two_seasons).into_iter().collect();
        assert_eq!(gaps, vec![(2, 1)]);
    }

    /// A series' by-id page is ONE page of the whole show, so it must never be
    /// treated as that episode's answer. Measured on `tt35324004` ("Tigers Sniff
    /// the Rose") on a phone running 1.0.92: the probe answered 5 rows, none of
    /// them E1, and E1 answered "No Streams Found" while 3 playable files
    /// existed. The page is now a seed and the episode ladder runs.
    #[test]
    fn by_id_page_must_cover_the_requested_episode() {
        use crate::wp_data::WpFile;

        let page: Vec<WpFile> = (5..=9)
            .map(|e| WpFile {
                title: format!("LayarAsia.Tigers.Sniff.the.Rose.{}.1080p.mp4", e),
                ..Default::default()
            })
            .collect();
        assert!(!by_id_page_covers_episode("series", &page, 1, 1));
        assert!(by_id_page_covers_episode("series", &page, 1, 7));

        // A MOVIE's page IS the film, and a series ROOT has no episode to
        // contradict — both keep the fast path.
        assert!(by_id_page_covers_episode("movie", &page, 0, 0));
        assert!(by_id_page_covers_episode("series", &page, 1, 0));
        assert!(by_id_page_covers_episode("series", &[], 1, 0));

        // A different SEASON is not coverage either.
        let s2 = vec![WpFile {
            title: "Show.S02E01.1080p.mkv".to_string(),
            ..Default::default()
        }];
        assert!(!by_id_page_covers_episode("series", &s2, 1, 1));
        assert!(by_id_page_covers_episode("series", &s2, 2, 1));
        assert!(!by_id_page_covers_episode("series", &[], 1, 1));
    }

    #[test]
    fn series_videos_are_sorted_by_season_then_episode() {
        use crate::wp_data::WpFile;
        let names = [
            "Perfect.Marriage.Revenge.2023.S01E12.720p.mkv",
            "Perfect.Marriage.Revenge.2023.S01E11.720p.mkv",
            "Perfect.Marriage.Revenge.2023.S01E10.720p.mkv",
            "Perfect.Marriage.Revenge.2023.S01E09.720p.mkv",
            "Perfect.Marriage.Revenge.2023.S01E01.720p.mkv",
            "Perfect.Marriage.Revenge.2023.S01E05.720p.mkv",
            "Perfect.Marriage.Revenge.2023.S02E03.720p.mkv",
            "Perfect.Marriage.Revenge.2023.S02E01.720p.mkv",
        ];
        let files: Vec<WpFile> = names
            .iter()
            .map(|t| WpFile {
                title: (*t).to_string(),
                added_date: 1_700_000_000,
                ..Default::default()
            })
            .collect();
        let videos = series_videos("pm:post:2", &files, &Value::Null, None);
        let seq: Vec<(i64, i64)> = videos
            .iter()
            .map(|v| (v["season"].as_i64().unwrap(), v["episode"].as_i64().unwrap()))
            .collect();
        assert_eq!(
            seq,
            vec![
                (1, 1),
                (1, 5),
                (1, 9),
                (1, 10),
                (1, 11),
                (1, 12),
                (2, 1),
                (2, 3)
            ],
            "{seq:?}"
        );
        // The id/name must still describe the same episode after sorting.
        for v in &videos {
            assert_eq!(
                v["id"],
                serde_json::json!(format!(
                    "pm:post:2:{}:{}",
                    v["season"], v["episode"]
                ))
            );
            assert_eq!(
                v["name"],
                serde_json::json!(format!("S{}E{}", v["season"], v["episode"]))
            );
        }
    }

    /// The series anchor is the post's own publish date, else the release year.
    #[test]
    fn series_anchor_prefers_the_post_date_over_the_year() {
        let post = serde_json::json!({"title": "Welcome to Waikiki 2018 • TvSeries", "date": "2024-02-19 20:01:04"});
        assert_eq!(
            series_released_anchor(&post).as_deref(),
            Some("2024-02-19T20:01:04Z")
        );
        let no_date = serde_json::json!({"title": "Welcome to Waikiki 2018 • TvSeries"});
        assert_eq!(
            series_released_anchor(&no_date).as_deref(),
            Some("2018-01-01T00:00:00Z")
        );
        let nothing = serde_json::json!({"title": "Untitled"});
        assert_eq!(series_released_anchor(&nothing), None);
    }
}
