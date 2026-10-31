//! Port of the auth block, backend.php:1262-1813:
//! `fd_is_cloudflare_tunnel_request`, `fd_is_https_request`,
//! `fd_is_local_request`, `fd_require_local_request`,
//! `fd_is_loopback_request`, `fd_auth_privileged`, `fd_require_privileged`,
//! `fd_url_host_is_public`, `fd_auth_cache`, `fd_auth_load`,
//! `fd_auth_valid_tokens`, `fd_auth_save`, `fd_auth_enabled`, `fd_auth_token`,
//! `fd_auth_rotate_token`, `fd_auth_verify_password`, `fd_auth_set_password`,
//! `fd_auth_client_ip`, `fd_auth_lockout_check`, `fd_auth_record_failure`,
//! `fd_auth_record_success`, `fd_auth_token_from_request`,
//! `fd_is_authenticated`, `fd_require_auth`, `fd_strip_token_prefix`.
//!
//! Password-hashing note (documented parity gap): a Rust process cannot verify
//! the PHP's bcrypt hash without the bcrypt crate, so this port DOES verify
//! `$2y$`/`$2b$` hashes (`bcrypt`) and otherwise stores its own
//! `sha256$<salt>$<hash>` (20 000 rounds). The two runtimes are separate
//! installs; they do not share `auth.json`.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;

use axum::response::Response;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::config::{AUTH_COOKIE, AUTH_DEFAULT_PASSWORD};
use crate::respond::json_response;
use crate::storage::{env, now, storage_path};

/* ── request context ─────────────────────────────────────────────────────── */

pub struct ReqCtx {
    pub method: String,
    /// Token-prefix-stripped, normalised path (always starts with `/`).
    pub path: String,
    /// Path as received.
    pub raw_path: String,
    pub query: Vec<(String, String)>,
    pub headers: axum::http::HeaderMap,
    pub cookies: HashMap<String, String>,
    pub remote: String,
    pub local: String,
    pub base_url: String,
}

impl ReqCtx {
    pub fn hdr(&self, name: &str) -> String {
        match self.headers.get(name) {
            Some(v) => v.to_str().unwrap_or("").trim().to_string(),
            None => String::new(),
        }
    }

    pub fn q(&self, name: &str) -> Option<&str> {
        self.query
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn qs(&self, name: &str) -> String {
        self.q(name).unwrap_or("").to_string()
    }

    pub fn has_q(&self, name: &str) -> bool {
        self.query.iter().any(|(k, _)| k == name)
    }

    pub fn first_param(&self, names: &[&str]) -> String {
        for n in names {
            if let Some(v) = self.q(n) {
                if !v.is_empty() {
                    return v.to_string();
                }
            }
        }
        String::new()
    }
}

pub fn parse_cookies(header: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for part in header.split(';') {
        if let Some(i) = part.find('=') {
            let k = part[..i].trim();
            if !k.is_empty() {
                out.insert(k.to_string(), part[i + 1..].trim().to_string());
            }
        }
    }
    out
}

/* ── request classification ──────────────────────────────────────────────── */

pub fn is_cloudflare_tunnel_request(ctx: &ReqCtx) -> bool {
    for raw in [ctx.hdr("host"), ctx.hdr("x-forwarded-host")] {
        for part in raw.split(',') {
            let h = part.trim().to_ascii_lowercase();
            if h.is_empty() {
                continue;
            }
            if h.contains("trycloudflare.com")
                || h.ends_with(".pencarimovie.com")
                || h.ends_with(".cfargotunnel.com")
            {
                return true;
            }
        }
    }
    !ctx.hdr("cf-connecting-ip").is_empty()
        || !ctx.hdr("cf-ray").is_empty()
        || !ctx.hdr("cf-visitor").is_empty()
}

pub fn is_https_request(ctx: &ReqCtx) -> bool {
    ctx.hdr("x-forwarded-proto").to_ascii_lowercase() == "https"
        || ctx.hdr("cf-visitor").contains("https")
}

pub fn is_public_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            if o[0] == 10 || o[0] == 127 || o[0] == 0 {
                return false;
            }
            if o[0] == 169 && o[1] == 254 {
                return false;
            }
            if o[0] == 172 && (16..=31).contains(&o[1]) {
                return false;
            }
            if o[0] == 192 && o[1] == 168 {
                return false;
            }
            if o[0] == 100 && (64..=127).contains(&o[1]) {
                return false;
            }
            true
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() {
                return false;
            }
            let seg = v6.segments()[0];
            if (seg & 0xffc0) == 0xfe80 || (seg & 0xfe00) == 0xfc00 {
                return false;
            }
            true
        }
    }
}

fn parse_ip(s: &str) -> Option<IpAddr> {
    s.trim().parse::<IpAddr>().ok()
}

pub fn is_local_request(ctx: &ReqCtx) -> bool {
    if is_cloudflare_tunnel_request(ctx) {
        return false;
    }
    for raw in [ctx.hdr("host"), ctx.hdr("x-forwarded-host")] {
        for part in raw.split(',') {
            let h = part.trim().to_ascii_lowercase();
            let h = h.split(':').next().unwrap_or("").to_string();
            if h.is_empty() {
                continue;
            }
            if h.ends_with(".herokuapp.com")
                || h == "herokuapp.com"
                || h.ends_with(".up.railway.app")
                || h.ends_with(".onrender.com")
                || h.ends_with(".fly.dev")
            {
                return false;
            }
        }
    }
    let fwd = ctx.hdr("x-forwarded-for");
    if !fwd.is_empty() {
        let client_ip = fwd.split(',').next().unwrap_or("").trim();
        if let Some(ip) = parse_ip(client_ip) {
            if is_public_ip(&ip) {
                return false;
            }
        }
    }
    let remote = ctx.remote.as_str();
    if remote.is_empty() {
        return true;
    }
    if remote == "127.0.0.1" || remote == "::1" || remote.starts_with("::ffff:127.0.0.1") {
        return true;
    }
    if !ctx.local.is_empty() && remote == ctx.local {
        return true;
    }
    match parse_ip(remote) {
        Some(ip) => !is_public_ip(&ip),
        None => false,
    }
}

pub fn is_loopback_request(ctx: &ReqCtx) -> bool {
    // The tunnel check MUST come first. cloudflared dials us from 127.0.0.1, so
    // testing `remote` first classified every PUBLIC request arriving through a
    // quick tunnel as "loopback" — which made it privileged, and walked straight
    // through `require_privileged` on /api/provision (and revealed `api_secret`
    // on /api/session). `is_local_request` above already checks this first.
    if is_cloudflare_tunnel_request(ctx) {
        return false;
    }
    let remote = ctx.remote.as_str();
    if remote.is_empty() {
        return false;
    }
    if remote == "127.0.0.1" || remote == "::1" || remote.starts_with("::ffff:127.0.0.1") {
        return true;
    }
    !ctx.local.is_empty() && remote == ctx.local && is_local_request(ctx)
}

pub fn require_local_request(ctx: &ReqCtx) -> Option<Response> {
    if is_local_request(ctx) {
        None
    } else {
        Some(json_response(
            403,
            &serde_json::json!({"ok": 0, "message": "This endpoint is restricted to local requests only."}),
        ))
    }
}

pub fn require_privileged(ctx: &ReqCtx) -> Option<Response> {
    if auth_privileged(ctx) {
        None
    } else {
        Some(json_response(
            403,
            &serde_json::json!({
                "ok": 0,
                "message": "This action must be performed on the machine itself (or with a valid token).",
                "local_only": true
            }),
        ))
    }
}

/// The gate for the routes the DASHBOARD drives from a browser: same network, or
/// a token.
///
/// Neither existing gate fits. [`require_auth`] answers true for every caller when
/// no password is set (it returns true when `!auth_enabled()`), so it closes
/// nothing on the common install. [`require_privileged`] refuses a LAN peer, which
/// broke the dashboard opened at `http://192.168.x.x:8088`: the page loaded, the
/// catalogs rendered, and then "Add bot" and every catalog toggle answered 403.
///
/// The boundary these routes actually need is "not the internet". `is_local_request`
/// already refuses a tunnelled request first, and refuses a public
/// `x-forwarded-for`, so a Cloudflare Tunnel, a PaaS host and any proxied caller
/// still need the token. A machine on the same network is inside the trust boundary
/// this whole install is built for, and it is already exempted on `/api/download`
/// and from the per-client limits.
pub fn require_trusted_device(ctx: &ReqCtx) -> Option<Response> {
    if is_local_request(ctx) || auth_privileged(ctx) {
        None
    } else {
        Some(json_response(
            403,
            &serde_json::json!({
                "ok": 0,
                "message": "This action must be performed on this machine or network (or with a valid token).",
                "local_only": true
            }),
        ))
    }
}

pub async fn url_host_is_public(raw: &str) -> bool {
    let parsed = match url::Url::parse(raw) {
        Ok(u) => u,
        Err(_) => return false,
    };
    let host = match parsed.host_str() {
        Some(h) => h.to_ascii_lowercase(),
        None => return false,
    };
    if host.is_empty() || host == "localhost" || host.ends_with(".localhost") {
        return false;
    }
    if let Some(ip) = parse_ip(&host) {
        return is_public_ip(&ip);
    }
    match tokio::net::lookup_host((host.as_str(), 443u16)).await {
        Ok(addrs) => {
            let list: Vec<IpAddr> = addrs.map(|a| a.ip()).collect();
            !list.is_empty() && list.iter().all(is_public_ip)
        }
        Err(_) => false,
    }
}

/* ── auth state ──────────────────────────────────────────────────────────── */

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct AuthData {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub password_hash: Option<String>,
    #[serde(default)]
    pub env_password: Option<bool>,
}

fn cache() -> &'static Mutex<Option<AuthData>> {
    static C: std::sync::OnceLock<Mutex<Option<AuthData>>> = std::sync::OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

pub fn auth_path() -> std::path::PathBuf {
    storage_path("auth.json")
}

fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::rng().fill_bytes(&mut buf);
    hex::encode(buf)
}

pub fn auth_load() -> AuthData {
    if let Ok(guard) = cache().lock() {
        if let Some(d) = guard.as_ref() {
            return d.clone();
        }
    }
    let mut data: AuthData = std::fs::read_to_string(auth_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default();

    let env_pw = env("SERVER_PASSWORD").unwrap_or_default();
    if !env_pw.trim().is_empty() {
        data.env_password = Some(true);
    } else if data.password_hash.is_none() {
        // NEVER hash on the load path (backend.php:1515).
        data.password_hash = Some(String::new());
    }
    match env("SERVER_TOKEN") {
        Some(t) => {
            data.token = Some(t.split(',').next().unwrap_or("").trim().to_string());
        }
        None => {
            if data.token.as_deref().unwrap_or("").is_empty() {
                data.token = Some(random_hex(16));
            }
        }
    }
    if data.enabled.is_none() {
        data.enabled = Some(true);
    }
    if !auth_path().exists() {
        auth_save(&data);
    }
    if let Ok(mut guard) = cache().lock() {
        *guard = Some(data.clone());
    }
    data
}

pub fn auth_save(data: &AuthData) {
    if let Ok(mut guard) = cache().lock() {
        *guard = Some(data.clone());
    }
    if let Ok(text) = serde_json::to_string(data) {
        let _ = std::fs::write(auth_path(), text);
    }
}

pub fn auth_enabled() -> bool {
    auth_load().enabled.unwrap_or(true)
}

/// True while the server still accepts the built-in default password.
///
/// Asking "is the stored hash empty" is wrong: the reset path stores a real
/// hash of `AUTH_DEFAULT_PASSWORD` (`auth_set_password(AUTH_DEFAULT_PASSWORD)`),
/// so an empty-hash test answered "no" while the dashboard still let `123456`
/// in. This asks the verifier instead, which already handles `SERVER_PASSWORD`,
/// salt+sha256 and bcrypt, so it is true for every route to the default.
///
/// `enabled: true` alone never meant somebody chose a password — that was the
/// trap the dashboard checklist fell into. The row stays open while this is true.
pub fn auth_uses_default_password() -> bool {
    auth_verify_password(AUTH_DEFAULT_PASSWORD)
}

pub fn auth_token() -> String {
    auth_load().token.unwrap_or_default()
}

pub fn auth_rotate_token() -> String {
    let mut data = auth_load();
    data.token = Some(random_hex(16));
    auth_save(&data);
    data.token.unwrap_or_default()
}

/// The `Set-Cookie` value for a freshly minted token.
///
/// Every route that rotates the token MUST send this. `auth_set_password()`,
/// `auth_rotate_token()` and the reset path all replace the token in
/// `auth.json`, and a client that keeps the old `pm_auth` cookie ends up
/// half-authenticated: the cookie is dead, while the new token handed back in
/// the JSON body still passes `is_authenticated()` through `X-Auth-Token`. The
/// dashboard then opens with no password gate while every `/<old-token>/…`
/// URL the player holds answers with the locked two-row body (issue #12).
pub fn auth_cookie_header(ctx: &ReqCtx, token: &str) -> String {
    let secure = if is_https_request(ctx) { "; Secure" } else { "" };
    format!(
        "pm_auth={}; Path=/; HttpOnly; SameSite=Lax; Max-Age=31536000{}",
        token, secure
    )
}

fn ct_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for i in 0..a.len() {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

pub fn auth_valid_tokens() -> Vec<String> {
    let mut tokens: Vec<String> = Vec::new();
    let main = auth_token().to_ascii_lowercase();
    if !main.is_empty() {
        tokens.push(main);
    }
    if let Some(list) = env("SERVER_TOKEN") {
        for raw in list.split(',') {
            let t = raw.trim().to_ascii_lowercase();
            if !t.is_empty() && !tokens.contains(&t) {
                tokens.push(t);
            }
        }
    }
    tokens
}

fn hash_password(password: &str, salt_hex: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt_hex.as_bytes());
    hasher.update(password.as_bytes());
    let mut digest = hasher.finalize().to_vec();
    for _ in 0..20_000 {
        let mut h = Sha256::new();
        h.update(&digest);
        h.update(password.as_bytes());
        digest = h.finalize().to_vec();
    }
    hex::encode(digest)
}

pub fn auth_verify_password(password: &str) -> bool {
    if let Some(env_pw) = env("SERVER_PASSWORD") {
        let env_pw = env_pw.trim();
        if !env_pw.is_empty() {
            return ct_eq(env_pw, password);
        }
    }
    let hash = auth_load().password_hash.unwrap_or_default();
    if hash.is_empty() {
        return ct_eq(AUTH_DEFAULT_PASSWORD, password);
    }
    if let Some(rest) = hash.strip_prefix("sha256$") {
        if let Some((salt, want)) = rest.split_once('$') {
            return ct_eq(&hash_password(password, salt), want);
        }
        return false;
    }
    if hash.starts_with("$2") {
        return bcrypt::verify(password, &hash).unwrap_or(false);
    }
    false
}

pub fn auth_set_password(password: &str) {
    let mut data = auth_load();
    let salt = random_hex(16);
    data.password_hash = Some(format!("sha256${}${}", salt, hash_password(password, &salt)));
    data.token = Some(random_hex(16));
    data.enabled = Some(true);
    auth_save(&data);
}

pub fn auth_client_ip(ctx: &ReqCtx) -> String {
    if is_cloudflare_tunnel_request(ctx) {
        let cf = ctx.hdr("cf-connecting-ip");
        if !cf.is_empty() {
            return cf;
        }
    }
    let fwd = ctx.hdr("x-forwarded-for");
    if !fwd.is_empty() {
        let ip = fwd.split(',').next().unwrap_or("").trim().to_string();
        if parse_ip(&ip).is_some() {
            return ip;
        }
    }
    if ctx.remote.is_empty() {
        "127.0.0.1".to_string()
    } else {
        ctx.remote.clone()
    }
}

/* ── lockout ladder ──────────────────────────────────────────────────────── */

fn lockout_path() -> std::path::PathBuf {
    storage_path("cache/auth_lockout.json")
}

fn read_lockout() -> serde_json::Map<String, serde_json::Value> {
    std::fs::read_to_string(lockout_path())
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.as_object().cloned())
        .unwrap_or_default()
}

fn write_lockout(state: &serde_json::Map<String, serde_json::Value>) {
    let _ = std::fs::write(lockout_path(), serde_json::Value::Object(state.clone()).to_string());
}

pub fn auth_lockout_check(ip: &str) -> Option<i64> {
    let state = read_lockout();
    let entry = state.get(ip)?;
    let until = entry.get("lockUntil").and_then(|v| v.as_i64()).unwrap_or(0);
    let now = now();
    if until > now {
        Some(until - now)
    } else {
        None
    }
}

pub struct FailureInfo {
    pub locked: bool,
    pub retry_after: i64,
    pub remaining: i64,
}

pub fn auth_record_failure(ip: &str) -> FailureInfo {
    let mut state = read_lockout();
    let now = now();
    let keys: Vec<String> = state.keys().cloned().collect();
    for k in keys {
        if let Some(v) = state.get(&k) {
            let last = v.get("lastFailAt").and_then(|x| x.as_i64()).unwrap_or(0);
            let until = v.get("lockUntil").and_then(|x| x.as_i64()).unwrap_or(0);
            if last > 0 && now - last > 3600 && until == 0 {
                state.remove(&k);
            }
        }
    }
    let tiers = [30i64, 120, 600, 1800];
    let mut entry = state
        .get(ip)
        .cloned()
        .unwrap_or_else(|| serde_json::json!({"fails": 0, "lockUntil": 0, "lockLevel": 0, "lastFailAt": 0}));
    let mut fails = entry.get("fails").and_then(|v| v.as_i64()).unwrap_or(0) + 1;
    let mut retry_after = 0i64;
    if fails >= 5 {
        let level = entry.get("lockLevel").and_then(|v| v.as_i64()).unwrap_or(0);
        let level = level.clamp(0, tiers.len() as i64 - 1) as usize;
        let duration = tiers[level];
        entry["lockUntil"] = serde_json::json!(now + duration);
        entry["lockLevel"] = serde_json::json!(level as i64 + 1);
        entry["fails"] = serde_json::json!(0);
        retry_after = duration;
        fails = 0;
    } else {
        entry["fails"] = serde_json::json!(fails);
    }
    entry["lastFailAt"] = serde_json::json!(now);
    state.insert(ip.to_string(), entry);
    write_lockout(&state);
    FailureInfo {
        locked: retry_after > 0,
        retry_after,
        remaining: (5 - fails).max(0),
    }
}

pub fn auth_record_success(ip: &str) {
    let mut state = read_lockout();
    if state.remove(ip).is_some() {
        write_lockout(&state);
    }
}

/* ── token transport ─────────────────────────────────────────────────────── */

fn find_token_prefix(path: &str) -> Option<(String, String)> {
    // ^/([0-9a-fA-F]{32,})(/.*)?$
    if let Some(rest) = path.strip_prefix('/') {
        let hex_len = rest
            .char_indices()
            .take_while(|(_, c)| c.is_ascii_hexdigit())
            .count();
        if hex_len >= 32 {
            let (tok, tail) = rest.split_at(hex_len);
            if tail.is_empty() || tail.starts_with('/') {
                return Some((
                    tok.to_string(),
                    if tail.is_empty() { "/".to_string() } else { tail.to_string() },
                ));
            }
        }
    }
    None
}

fn find_t_prefix(path: &str) -> Option<(String, String)> {
    // ^/t/([A-Za-z0-9]+)(/.*)?$
    let rest = path.strip_prefix("/t/")?;
    let tok_len = rest
        .char_indices()
        .take_while(|(_, c)| c.is_ascii_alphanumeric())
        .count();
    if tok_len == 0 {
        return None;
    }
    let (tok, tail) = rest.split_at(tok_len);
    if tail.is_empty() || tail.starts_with('/') {
        Some((
            tok.to_string(),
            if tail.is_empty() { "/".to_string() } else { tail.to_string() },
        ))
    } else {
        None
    }
}

/// Token from `/<hex32+>/` prefix, `/t/<token>/`, `?token=`, `X-Auth-Token`, or
/// the `pm_auth` cookie. Keep `{32,}` in sync with `strip_token_prefix()`.
pub fn auth_token_from_request(ctx: &ReqCtx) -> String {
    if let Some((t, _)) = find_token_prefix(&ctx.raw_path) {
        return t.to_ascii_lowercase();
    }
    if let Some((t, _)) = find_t_prefix(&ctx.raw_path) {
        return t;
    }
    let q = ctx.qs("token");
    if !q.is_empty() {
        return q;
    }
    let h = ctx.hdr("x-auth-token");
    if !h.is_empty() {
        return h;
    }
    ctx.cookies.get(AUTH_COOKIE).cloned().unwrap_or_default()
}

pub fn auth_privileged(ctx: &ReqCtx) -> bool {
    if !auth_enabled() {
        return is_loopback_request(ctx);
    }
    if is_loopback_request(ctx) {
        return true;
    }
    let token = auth_token_from_request(ctx).to_ascii_lowercase();
    if token.is_empty() {
        return false;
    }
    auth_valid_tokens().iter().any(|t| ct_eq(t, &token))
}

pub fn is_authenticated(ctx: &ReqCtx) -> bool {
    if !auth_enabled() {
        return true;
    }
    if is_local_request(ctx) {
        return true;
    }
    let token = auth_token_from_request(ctx).to_ascii_lowercase();
    if token.is_empty() {
        return false;
    }
    auth_valid_tokens().iter().any(|t| ct_eq(t, &token))
}

pub fn require_auth(ctx: &ReqCtx) -> Option<Response> {
    if is_authenticated(ctx) {
        None
    } else {
        Some(json_response(
            401,
            &serde_json::json!({"ok": 0, "message": "Password required", "auth_required": true}),
        ))
    }
}

pub fn strip_token_prefix(path: &str) -> String {
    if let Some((_, tail)) = find_token_prefix(path) {
        return tail;
    }
    if let Some((_, tail)) = find_t_prefix(path) {
        return tail;
    }
    path.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_hex_token_prefix() {
        assert_eq!(
            strip_token_prefix("/0123456789abcdef0123456789abcdef/manifest.json"),
            "/manifest.json"
        );
        assert_eq!(strip_token_prefix("/0123456789abcdef0123456789abcdef"), "/");
    }

    #[test]
    fn strips_t_token_prefix() {
        assert_eq!(strip_token_prefix("/t/abc123/manifest.json"), "/manifest.json");
    }

    #[test]
    fn leaves_normal_paths() {
        assert_eq!(strip_token_prefix("/api/session"), "/api/session");
    }

    #[test]
    fn private_ips() {
        assert!(!is_public_ip(&"192.168.1.5".parse().unwrap()));
        assert!(!is_public_ip(&"127.0.0.1".parse().unwrap()));
        assert!(is_public_ip(&"8.8.8.8".parse().unwrap()));
    }

    fn ctx_with(remote: &str, local: &str, headers: &[(&str, &str)]) -> ReqCtx {
        let mut map = axum::http::HeaderMap::new();
        for (k, v) in headers {
            map.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                axum::http::HeaderValue::from_str(v).unwrap(),
            );
        }
        ReqCtx {
            method: "POST".into(),
            path: "/api/provision".into(),
            raw_path: "/api/provision".into(),
            query: Vec::new(),
            headers: map,
            cookies: HashMap::new(),
            remote: remote.into(),
            local: local.into(),
            base_url: format!("http://{local}:8088"),
        }
    }

    /// A request that arrived through the Cloudflare tunnel is NEVER this
    /// machine, even though cloudflared dials us from 127.0.0.1. This is what
    /// `require_privileged` on /api/provision relies on.
    #[test]
    fn tunnel_request_is_not_loopback() {
        let via_tunnel = ctx_with("127.0.0.1", "192.168.2.225", &[("cf-ray", "8f00-HKG")]);
        assert!(!is_loopback_request(&via_tunnel));
        assert!(!auth_privileged(&via_tunnel));
        assert!(require_privileged(&via_tunnel).is_some());
    }

    #[test]
    fn hostname_alone_marks_a_tunnel_request() {
        let by_host = ctx_with(
            "127.0.0.1",
            "192.168.2.225",
            &[("host", "problems-apparel-para-school.trycloudflare.com")],
        );
        assert!(!is_loopback_request(&by_host));
        assert!(require_privileged(&by_host).is_some());
    }

    #[test]
    fn real_loopback_stays_privileged() {
        let local = ctx_with("127.0.0.1", "192.168.2.225", &[]);
        assert!(is_loopback_request(&local));
        assert!(require_privileged(&local).is_none());
    }

    /// A public peer can never be privileged, with or without auth enabled.
    #[test]
    fn public_peer_is_never_privileged() {
        let public = ctx_with("8.8.8.8", "192.168.2.225", &[]);
        assert!(!auth_privileged(&public));
        assert!(require_privileged(&public).is_some());
    }

    /// A LAN peer without a token cannot provision; the engine binds 0.0.0.0 on
    /// Android and PaaS, which is exactly the case this guards.
    #[test]
    fn lan_peer_needs_a_token() {
        let lan = ctx_with("192.168.2.50", "192.168.2.225", &[]);
        assert!(require_privileged(&lan).is_some());
    }
}
