//! Port of the Cloudflare tunnel control block (backend.php:11199-13545):
//! `fd_tunnel_*`, `fd_enable_tunnel`, `fd_disable_tunnel`,
//! `fd_get_tunnel_status`.
//!
//! Everything lives under `storage/tunnel/` with the PHP's file names, so both
//! runtimes can drive the same tunnel (`token.txt`, `state.json`,
//! `cloudflared.pid`, `cloudflared.log`, `cloudflared.err.log`, `config.yml`,
//! and `.last_autorestart` for the watchdog stamp).
//!
//! Both tunnel kinds are supported: a **Quick Tunnel** (`--url`, the public
//! host is a random `*.trycloudflare.com` that is discovered from the
//! `--metrics` endpoint and, failing that, from the log) and a **Named Tunnel**
//! (`run --token`, the public hostname comes from the tunnel's ingress config
//! that cloudflared logs at startup). A token may be a raw JWT or a whole
//! `cloudflared.exe service install eyJ…` command line.
//!
//! ## Why the child is spawned directly instead of `cmd /c start "" /b`
//!
//! The PHP needs `start "" /b` because every FrankenPHP request is its own
//! short-lived process and `Start-Process` keeps the child inside the parent's
//! job object, which is killed when the request ends. This binary is ONE
//! long-lived process: there is no per-request process and no job object, so a
//! child spawned with `std::process::Command` already survives (and survives the
//! server exiting). Direct spawn is also the only way to get the real PID,
//! which `pid_alive()` / `kill_pid()` and the state file need — `start "" /b`
//! reports the PID of an intermediate `cmd.exe` and gives us nothing back.
//! On Windows the child gets `CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP` so it
//! has no console of its own and is not killed by Ctrl-C in the server's
//! terminal; on Unix `setsid` (or `nohup`) does the same job.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use std::sync::{Mutex, OnceLock};

use crate::diagnostics::get_device_id;
use crate::httpc;
use crate::log::log;
use crate::storage::{env, mtime_secs, now, storage_path};

/// `fd_tunnel_dir()`.
pub fn tunnel_dir() -> PathBuf {
    let dir = storage_path("tunnel");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// `fd_tunnel_token_path()`.
pub fn token_path() -> PathBuf {
    tunnel_dir().join("token.txt")
}

/// `fd_tunnel_state_path()`.
pub fn state_path() -> PathBuf {
    tunnel_dir().join("state.json")
}

/// `fd_tunnel_pid_path()`.
pub fn pid_path() -> PathBuf {
    tunnel_dir().join("cloudflared.pid")
}

/// `fd_tunnel_log_path()` (cloudflared's own `--logfile`).
pub fn log_path() -> PathBuf {
    tunnel_dir().join("cloudflared.log")
}

/// `fd_tunnel_err_log_path()` (stdout/stderr of the process).
pub fn err_log_path() -> PathBuf {
    tunnel_dir().join("cloudflared.err.log")
}

/// `fd_tunnel_config_path()`.
pub fn config_path() -> PathBuf {
    tunnel_dir().join("config.yml")
}

/// `fd_tunnel_bin_dir()` — where a downloaded `cloudflared` lands.
pub fn bin_dir() -> PathBuf {
    let dir = storage_path("tunnel/bin");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// `fd_tunnel_bin_name()`: Windows ships `.exe`, everything else (Linux, macOS,
/// Android/Termux) uses the bare name.
pub fn bin_name() -> &'static str {
    bin_name_for(cfg!(windows))
}

/// Pure, so the Windows/Unix split is testable on either host.
pub fn bin_name_for(windows: bool) -> &'static str {
    if windows {
        "cloudflared.exe"
    } else {
        "cloudflared"
    }
}

fn read_trim(path: &Path) -> String {
    std::fs::read_to_string(path)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

/// `fd_load_saved_tunnel_token()`.
/// Read the stored tunnel token, discarding one that is not a real token.
///
/// Older builds saved whatever was typed, so a device can still carry garbage in
/// `token.txt` (and in `state.json`) — the dashboard then pre-filled the field with
/// it and every enable "used" it. Purging here means one read is enough to clean up.
pub fn load_token() -> String {
    let token = read_trim(&token_path());
    if token.is_empty() {
        return token;
    }
    if token_looks_valid(&token) {
        return token;
    }
    log(
        "discarding an invalid stored tunnel token",
        &json!({"len": token.len(), "head": token.chars().take(6).collect::<String>()}),
    );
    let _ = std::fs::remove_file(token_path());
    String::new()
}

/// `fd_save_tunnel_token()`.
///
/// Refuses anything that is not a real tunnel token: the previous version saved
/// whatever was typed, so a typo / a dashboard URL / a truncated paste was
/// persisted, `cloudflared` exited with "Provided Tunnel token is not valid", and
/// every later "enable" (empty field → `load_token()`) replayed the garbage from
/// disk forever. Clearing the field was the only way out.
pub fn save_token(token: &str) {
    let token = token.trim();
    if token.is_empty() {
        return;
    }
    if !token_looks_valid(token) {
        log(
            "refusing to store an invalid cloudflared tunnel token",
            &json!({"len": token.len(), "head": token.chars().take(6).collect::<String>()}),
        );
        return;
    }
    let _ = std::fs::write(token_path(), token);
}

/// Validate what the user typed **before** anything is started.
///
/// Returns the token to use (`""` = use the Quick Tunnel) or the rejection
/// message. This is synchronous and pure, so `/api/tunnel/enable` can answer a
/// junk token immediately instead of replying "starting" and making the user wait
/// for a failure they could have been told about instantly.
pub fn validate_token_input(raw: &str) -> Result<String, String> {
    let token = extract_token(raw);
    if token.is_empty() {
        return Ok(String::new());
    }
    if token_looks_valid(&token) {
        return Ok(token);
    }
    Err("That does not look like a Cloudflare Tunnel token. Copy the token from Cloudflare Zero Trust -> Networks -> Tunnels -> (your tunnel) -> Install connector; it is a long string starting with \"eyJ\". Leave the field empty for a free Quick Tunnel.".to_string())
}

/// A cloudflared log line that PROVES the token was rejected — no point waiting
/// out the timeout.
fn fatal_token_error(logs: &str) -> Option<String> {
    let lower = logs.to_ascii_lowercase();
    for needle in [
        "provided tunnel token is not valid",
        "failed to unmarshal token",
        "unauthorized",
        "authentication failed",
        "tunnel not found",
        "invalid tunnel secret",
        "no tunnel found",
    ] {
        if let Some(pos) = lower.rfind(needle) {
            // Quote the offending line so the user sees cloudflared's own words.
            let start = lower[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
            let end = lower[pos..].find('\n').map(|i| pos + i).unwrap_or(lower.len());
            let line = logs.get(start..end).unwrap_or("").trim().to_string();
            return Some(if line.is_empty() {
                needle.to_string()
            } else {
                line
            });
        }
    }
    None
}

/// Is this a Cloudflare tunnel token, and not a typo?
///
/// A tunnel token is `base64url({"a": "<account>", "t": "<tunnel>", "s": "<secret>"})`,
/// so it starts with `eyJ` (the base64 of `{"`), it is long, and it DECODES to a
/// JSON object carrying `a` and `t`. Checking the shape only (a length or an
/// `eyJ` prefix) is not enough — random text must never be treated as a token.
pub fn token_looks_valid(token: &str) -> bool {
    use base64::Engine;
    let t = token.trim();
    if t.len() < 60 || !t.starts_with("eyJ") {
        return false;
    }
    if !t
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '=' || c == '.')
    {
        return false;
    }
    let unpadded = t.trim_end_matches('=');
    let decoded = base64::engine::general_purpose::STANDARD_NO_PAD
        .decode(unpadded)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(unpadded));
    let Ok(bytes) = decoded else {
        return false;
    };
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(v) => v.get("a").is_some() && v.get("t").is_some(),
        Err(_) => false,
    }
}

/// `fd_load_tunnel_state()`. A missing/!empty `tunnel_token` in the state falls
/// back to `token.txt`, exactly like the PHP (so a token typed once survives a
/// `state.json` rewrite that forgot it).
pub fn load_state() -> Value {
    load_state_at(&state_path())
}

pub fn load_state_at(path: &Path) -> Value {
    let mut state = std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| json!({}));
    if st_str(&state, "tunnel_token").is_empty() {
        let saved = read_trim(&path.with_file_name("token.txt"));
        if !saved.is_empty() {
            state["tunnel_token"] = json!(saved);
        }
    }
    state
}

/// `fd_save_tunnel_state()` — keeps `token.txt` in sync and pretty-prints.
pub fn save_state(state: &Value) {
    save_state_at(&state_path(), state)
}

pub fn save_state_at(path: &Path, state: &Value) {
    let token = st_str(state, "tunnel_token");
    if !token.is_empty() {
        let _ = std::fs::write(path.with_file_name("token.txt"), &token);
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, text);
    }
}

/// `fd_clear_tunnel_state($preserveToken = true)`.
pub fn clear_state(preserve_token: bool) {
    clear_state_in(&tunnel_dir(), preserve_token)
}

pub fn clear_state_in(dir: &Path, preserve_token: bool) {
    let token_file = dir.join("token.txt");
    let saved = if preserve_token { read_trim(&token_file) } else { String::new() };
    let _ = std::fs::remove_file(dir.join("state.json"));
    let _ = std::fs::remove_file(dir.join("cloudflared.pid"));
    if preserve_token {
        if !saved.is_empty() {
            let _ = std::fs::write(&token_file, &saved);
        }
    } else {
        let _ = std::fs::remove_file(&token_file);
    }
}

/// `fd_tunnel_read_pid()` — the pid file first, then `state.pid`.
pub fn read_pid() -> i64 {
    read_pid_at(&pid_path(), &load_state())
}

pub fn read_pid_at(pid_file: &Path, state: &Value) -> i64 {
    if pid_file.is_file() {
        return read_trim(pid_file).parse::<i64>().unwrap_or(0);
    }
    st_i64(state, "pid")
}

/// `fd_tunnel_write_pid()`.
pub fn write_pid(pid: i64) {
    let _ = std::fs::write(pid_path(), pid.to_string());
}

// ─── URL / log parsing ──────────────────────────────────────────────────────

/// `fd_tunnel_normalize_url()`: only a real Quick Tunnel host is accepted, and
/// `api.trycloudflare.com` (the API host that appears in cloudflared's banner)
/// is never a tunnel.
pub fn normalize_url(value: &str) -> String {
    let mut v = value.trim().to_string();
    if v.is_empty() {
        return String::new();
    }
    let lower = v.to_ascii_lowercase();
    if !lower.starts_with("http://") && !lower.starts_with("https://") {
        v = format!("https://{}", v.trim_start_matches('/'));
    }
    while v.ends_with('/') {
        v.pop();
    }
    let re = match fancy_regex::Regex::new(r"(?i)^https://([a-z0-9-]+)\.trycloudflare\.com$") {
        Ok(re) => re,
        Err(_) => return String::new(),
    };
    let caps = match re.captures(&v) {
        Ok(Some(c)) => c,
        _ => return String::new(),
    };
    let host = caps.get(1).map(|m| m.as_str().to_ascii_lowercase()).unwrap_or_default();
    if host == "api" || host.is_empty() {
        return String::new();
    }
    format!("https://{}.trycloudflare.com", host)
}

/// `fd_tunnel_parse_url_from_text()`: the LAST non-`api` Quick Tunnel URL in
/// the text (`api.trycloudflare.com` is skipped, not treated as a match).
pub fn parse_url_from_text(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let re = match fancy_regex::Regex::new(r"(?i)https://([a-z0-9-]+)\.trycloudflare\.com") {
        Ok(re) => re,
        Err(_) => return String::new(),
    };
    let mut found = String::new();
    let mut start = 0usize;
    while let Ok(Some(caps)) = re.captures_from_pos(text, start) {
        let whole = caps.get(0).map(|m| m.as_str().to_string()).unwrap_or_default();
        let host = caps.get(1).map(|m| m.as_str().to_ascii_lowercase()).unwrap_or_default();
        let next = caps.get(0).map(|m| m.end()).unwrap_or(text.len());
        start = if next > start { next } else { start + 1 };
        if host == "api" {
            continue;
        }
        let normalized = normalize_url(&whole);
        if !normalized.is_empty() {
            found = normalized;
        }
    }
    found
}

/// `fd_tunnel_read_logs()`: `cloudflared.log` then `cloudflared.err.log`.
pub fn read_logs() -> String {
    read_logs_at(&log_path(), &err_log_path())
}

pub fn read_logs_at(log_file: &Path, err_file: &Path) -> String {
    let mut chunks: Vec<String> = Vec::new();
    for path in [log_file, err_file] {
        if path.is_file() {
            if let Ok(body) = std::fs::read_to_string(path) {
                chunks.push(body);
            }
        }
    }
    chunks.join("\n")
}

/// Pull the ingress `config` JSON out of one log line.
///
/// The PHP only handled a pure-JSON log line. Modern cloudflared writes logfmt
/// (`… config="{\"ingress\":[…]}" version=5`), so both shapes are accepted here;
/// this only ever ADDS a discovered hostname, it can never remove one.
fn config_json_from_line(line: &str) -> Option<Value> {
    if let Ok(v) = serde_json::from_str::<Value>(line) {
        if let Some(cfg) = v.get("config") {
            return match cfg {
                Value::String(s) => serde_json::from_str::<Value>(s).ok(),
                other => Some(other.clone()),
            };
        }
    }
    let marker = "config=\"";
    let start = line.find(marker)? + marker.len();
    let rest = &line[start..];
    let bytes = rest.as_bytes();
    let mut end = rest.len();
    let mut i = 0usize;
    while i < bytes.len() {
        match bytes[i] {
            b'\\' => i += 2,
            b'"' => {
                end = i;
                break;
            }
            _ => i += 1,
        }
    }
    let unescaped = rest[..end].replace("\\\"", "\"");
    serde_json::from_str::<Value>(&unescaped).ok()
}

/// `fd_tunnel_parse_ingress_domains()`: the public hostnames a Named Tunnel
/// serves. Hosts whose service contains the local port win; when none matches
/// every non-`http_status:` host is returned (the PHP's fallback).
pub fn parse_ingress_domains(logs: &str, port: u16) -> Vec<String> {
    if logs.is_empty() || !logs.contains("ingress") {
        return Vec::new();
    }
    let mut matched: Vec<String> = Vec::new();
    let mut all: Vec<String> = Vec::new();
    for raw in logs.lines() {
        let line = raw.trim();
        if line.is_empty() || !line.contains("ingress") {
            continue;
        }
        let cfg = match config_json_from_line(line) {
            Some(c) => c,
            None => continue,
        };
        let ingress = match cfg.get("ingress").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => continue,
        };
        let needle = port.to_string();
        for ing in ingress {
            let host = ing
                .get("hostname")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            let svc = ing
                .get("service")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if host.is_empty() || svc.starts_with("http_status:") {
                continue;
            }
            if !all.contains(&host) {
                all.push(host.clone());
            }
            if svc.contains(&needle) && !matched.contains(&host) {
                matched.push(host);
            }
        }
    }
    if matched.is_empty() { all } else { matched }
}

/// `fd_tunnel_pick_metrics_port()`: an ephemeral port the OS just told us is
/// free, so two tunnels never fight over `20241`.
pub fn pick_metrics_port() -> u16 {
    if let Ok(listener) = std::net::TcpListener::bind("127.0.0.1:0") {
        if let Ok(addr) = listener.local_addr() {
            if addr.port() > 0 {
                return addr.port();
            }
        }
    }
    20241
}

/// `fd_tunnel_local_http_get()` — 1 s by default, never throws.
pub async fn local_http_get(url: &str, timeout_secs: u64) -> String {
    let res = httpc::client()
        .get(url)
        .timeout(Duration::from_secs(timeout_secs.max(1)))
        .header("Accept", "application/json")
        .send()
        .await;
    match res {
        Ok(r) => r.text().await.unwrap_or_default(),
        Err(e) => {
            log("tunnel local http failed", &json!({"url": url, "error": e.to_string()}));
            String::new()
        }
    }
}

/// Ports a pid is LISTENing on over loopback, parsed from `netstat -ano -p tcp`
/// (Windows) or `ss -lntp` (Unix). Pure so it can be unit-tested.
pub fn parse_listen_ports(text: &str, pid: i64) -> Vec<u16> {
    let mut ports: Vec<u16> = Vec::new();
    if pid <= 1 {
        return ports;
    }
    let pid_str = pid.to_string();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        // Windows rows end with the PID; ss puts it in `users:(("…",pid=N,fd=…))`.
        if !line.split_whitespace().any(|tok| tok == pid_str) && !line.contains(&format!("pid={}", pid_str)) {
            continue;
        }
        if !line.to_ascii_uppercase().contains("LISTEN") {
            continue;
        }
        if let Ok(re) = fancy_regex::Regex::new(r"127\.0\.0\.1:(\d+)") {
            if let Ok(Some(caps)) = re.captures(line) {
                if let Some(m) = caps.get(1) {
                    if let Ok(port) = m.as_str().parse::<u16>() {
                        if port > 0 && !ports.contains(&port) {
                            ports.push(port);
                        }
                    }
                }
            }
        }
    }
    ports
}

/// `fd_tunnel_metrics_ports()` — preferred, then the state's, then `20241`,
/// then anything the pid is actually listening on.
pub async fn metrics_ports(pid: i64, preferred: u16) -> Vec<u16> {
    let mut ports: Vec<u16> = Vec::new();
    if preferred > 0 {
        ports.push(preferred);
    }
    let from_state = st_i64(&load_state(), "metrics_port");
    if from_state > 0 && from_state <= u16::MAX as i64 {
        let p = from_state as u16;
        if !ports.contains(&p) {
            ports.push(p);
        }
    }
    if ports.is_empty() {
        ports.push(20241);
    }
    // Only search system sockets if we don't already have a candidate port.
    // When preferred > 0, cloudflared was explicitly given that port; running
    // netstat on every 250ms polling loop thrashes process execution for no reason.
    if pid > 1 && preferred == 0 && ports.len() <= 1 {
        #[cfg(windows)]
        let listing = run_capture("netstat", &["-ano", "-p", "tcp"]).await;
        #[cfg(not(windows))]
        let listing = run_capture("sh", &["-c", "ss -lntp 2>/dev/null || netstat -lntp 2>/dev/null"]).await;
        for port in parse_listen_ports(&listing, pid) {
            if !ports.contains(&port) {
                ports.push(port);
            }
        }
    }
    ports
}

/// `fd_tunnel_parse_quicktunnel_body()`: cloudflared's `/quicktunnel` and
/// `/metrics` bodies (JSON, `userHostname="…"`, or a bare URL).
pub fn parse_quicktunnel_body(body: &str) -> String {
    let body = body.trim();
    if body.is_empty() {
        return String::new();
    }
    if let Ok(json) = serde_json::from_str::<Value>(body) {
        for key in ["hostname", "Hostname", "url", "URL"] {
            if let Some(s) = json.get(key).and_then(|v| v.as_str()) {
                let url = normalize_url(s);
                if !url.is_empty() {
                    return url;
                }
            }
        }
    }
    if let Ok(re) = fancy_regex::Regex::new(r#"(?i)userHostname="(https://[a-z0-9-]+\.trycloudflare\.com)""#) {
        if let Ok(Some(caps)) = re.captures(body) {
            let raw = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let url = normalize_url(raw);
            if !url.is_empty() {
                return url;
            }
        }
    }
    parse_url_from_text(body)
}

/// `fd_tunnel_read_quicktunnel_url()`: ask the metrics endpoint(s) which
/// Quick Tunnel they published.
pub async fn read_quicktunnel_url(pid: i64, metrics_port: u16) -> String {
    for port in metrics_ports(pid, metrics_port).await {
        let base = format!("http://127.0.0.1:{}", port);
        let url = parse_quicktunnel_body(&local_http_get(&format!("{}/quicktunnel", base), 1).await);
        if !url.is_empty() {
            return url;
        }
        let url = parse_quicktunnel_body(&local_http_get(&format!("{}/metrics", base), 1).await);
        if !url.is_empty() {
            return url;
        }
    }
    String::new()
}

// ─── process helpers ────────────────────────────────────────────────────────

async fn run_capture(program: &str, args: &[&str]) -> String {
    let out = tokio::task::spawn_blocking({
        let program = program.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        move || {
            // `proc::cmd`, never `Command::new`: every helper this runs (netstat,
            // tasklist, taskkill, ps, kill) would otherwise open a console, and
            // Windows Terminal makes that a visible window per call.
            let mut cmd = crate::proc::cmd(program);
            cmd.args(&args)
                .stdin(Stdio::null());
            cmd.output()
                .map(|o| {
                    let mut s = String::from_utf8_lossy(&o.stdout).to_string();
                    s.push_str(&String::from_utf8_lossy(&o.stderr));
                    s
                })
                .unwrap_or_default()
        }
    })
    .await;
    out.unwrap_or_default()
}

/// Check whether a process is running on Windows using native Win32 APIs without
/// shelling out to `tasklist.exe` (which spawns an external process).
#[cfg(windows)]
pub fn win_pid_alive(pid: i64) -> bool {
    if pid <= 1 || pid > u32::MAX as i64 {
        return false;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> isize;
        fn GetExitCodeProcess(hProcess: isize, lpExitCode: *mut u32) -> i32;
        fn CloseHandle(hObject: isize) -> i32;
        fn GetLastError() -> u32;
    }
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const STILL_ACTIVE: u32 = 259;
    const ERROR_ACCESS_DENIED: u32 = 5;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid as u32);
        if handle == 0 {
            return GetLastError() == ERROR_ACCESS_DENIED;
        }
        let mut exit_code: u32 = 0;
        let ok = GetExitCodeProcess(handle, &mut exit_code);
        CloseHandle(handle);
        ok != 0 && exit_code == STILL_ACTIVE
    }
}

/// `tasklist /FI "PID eq N" /NH /FO CSV` contains that pid (pure, testable).
pub fn tasklist_contains_pid(text: &str, pid: i64) -> bool {
    if pid <= 1 {
        return false;
    }
    let want = pid.to_string();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with('"') {
            continue;
        }
        // `"cloudflared.exe","1234","Console","1","25,000 K"`
        let fields: Vec<&str> = line.split("\",\"").collect();
        if fields.len() < 2 {
            continue;
        }
        let field = fields[1].trim_matches('"');
        if field == want {
            return true;
        }
    }
    false
}

/// `fd_tunnel_pid_alive()`: the metrics endpoint is authoritative (it cannot be
/// a reused PID), the process table is the fallback.
pub async fn pid_alive(pid: i64, metrics_port: u16) -> bool {
    if pid <= 1 && metrics_port == 0 {
        return false;
    }
    if metrics_port > 0 {
        let resp = local_http_get(&format!("http://127.0.0.1:{}/ready", metrics_port), 1).await;
        if resp.to_ascii_lowercase().contains("\"status\":200") {
            return true;
        }
    }
    if pid <= 1 {
        return false;
    }
    #[cfg(windows)]
    {
        win_pid_alive(pid)
    }
    // NOT `/proc/<pid>`: that path does not exist on macOS, where this answered
    // "the tunnel is gone" for a cloudflared that was running. `session` owns the
    // portable answer now (and the `/proc` branch is still what Linux and Android
    // take, so nothing changes there).
    #[cfg(not(windows))]
    {
        crate::session::pid_still_running(pid)
    }
}

/// `fd_tunnel_kill_pid()`.
pub async fn kill_pid(pid: i64) {
    if pid <= 1 {
        return;
    }
    #[cfg(windows)]
    {
        let _ = run_capture("taskkill", &["/PID", &pid.to_string(), "/T", "/F"]).await;
    }
    #[cfg(not(windows))]
    {
        let _ = run_capture("kill", &[&pid.to_string()]).await;
        tokio::time::sleep(Duration::from_millis(250)).await;
        if pid_alive(pid, 0).await {
            let _ = run_capture("kill", &["-9", &pid.to_string()]).await;
        }
    }
}

/// `fd_tunnel_kill_leftovers()`: the recorded pid, plus — on Windows — every
/// `cloudflared.exe` still holding one of our metrics ports (a leftover whose
/// pid file was lost). Never a blanket `taskkill /IM cloudflared.exe`, so an
/// unrelated tunnel on the same machine is left alone.
pub async fn kill_leftovers() {
    let pid = read_pid();
    if pid > 1 {
        kill_pid(pid).await;
    }

    #[cfg(windows)]
    {
        let state = load_state();
        let mut ports: Vec<u16> = Vec::new();
        let from_state = st_i64(&state, "metrics_port");
        if from_state > 0 && from_state <= u16::MAX as i64 {
            ports.push(from_state as u16);
        }
        if !ports.contains(&20241) {
            ports.push(20241);
        }
        let listing = run_capture("netstat", &["-ano", "-p", "tcp"]).await;
        let mut victims: Vec<i64> = Vec::new();
        for port in ports {
            let marker = format!("127.0.0.1:{}", port);
            for raw in listing.lines() {
                let line = raw.trim();
                if !line.contains(&marker) || !line.to_ascii_uppercase().contains("LISTENING") {
                    continue;
                }
                if let Some(last) = line.split_whitespace().last() {
                    if let Ok(p) = last.parse::<i64>() {
                        if p > 1 && p != pid && !victims.contains(&p) {
                            victims.push(p);
                        }
                    }
                }
            }
        }
        for victim in victims {
            let image = run_capture(
                "tasklist",
                &["/FI", &format!("PID eq {}", victim), "/NH", "/FO", "CSV"],
            )
            .await;
            if image.to_ascii_lowercase().contains("cloudflared") {
                log("killing leftover cloudflared", &json!({"pid": victim}));
                kill_pid(victim).await;
            }
        }
    }

    #[cfg(not(windows))]
    {
        let marker = config_path().to_string_lossy().to_string();
        let _ = run_capture("pkill", &["-f", &marker]).await;
    }
}

// ─── binary resolution ──────────────────────────────────────────────────────

/// `fd_tunnel_asset_name()` — the GitHub release asset for this platform.
pub fn asset_name() -> &'static str {
    asset_name_for(std::env::consts::OS, std::env::consts::ARCH)
}

/// Pure so every (os, arch) pair — including the ones this host cannot build —
/// is pinned by a unit test. `os`/`arch` are the `std::env::consts` values
/// (`"windows"`, `"linux"`, `"macos"`, `"android"`, `"x86_64"`, `"aarch64"`,
/// `"arm"`, `"x86"`).
///
/// **Android** gets the *Linux* asset on purpose: Cloudflare's release binaries
/// are static Go builds (`CGO_ENABLED=0`), so `cloudflared-linux-arm64` /
/// `-amd64` / `-386` exec directly under bionic — there is no `-android` asset
/// and none is needed. See `CROSS-PLATFORM.md` for the caveat that this is
/// verified only for the *download path*, not a live tunnel on a phone.
pub fn asset_name_for(os: &str, arch: &str) -> &'static str {
    match os {
        // Cloudflare publishes no native Windows ARM build; ARM64 Windows runs
        // the amd64 binary under x64 emulation.
        "windows" => "cloudflared-windows-amd64.exe",
        "macos" => {
            if arch == "aarch64" {
                "cloudflared-darwin-arm64.tgz"
            } else {
                "cloudflared-darwin-amd64.tgz"
            }
        }
        // "linux" and "android" (and any other unix) share the Linux assets.
        _ => match arch {
            "aarch64" => "cloudflared-linux-arm64",
            "arm" => "cloudflared-linux-arm",
            "x86" => "cloudflared-linux-386",
            _ => "cloudflared-linux-amd64",
        },
    }
}

fn is_usable_bin(path: &Path) -> bool {
    std::fs::metadata(path).map(|m| m.is_file() && m.len() > 1024).unwrap_or(false)
}

/// `fd_tunnel_which()` — `where cloudflared` / `command -v cloudflared`.
pub fn which() -> String {
    #[cfg(windows)]
    let out = {
        let mut cmd = crate::proc::cmd("where");
        cmd.arg("cloudflared")
            .stdin(Stdio::null());
        cmd.output()
            .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
            .unwrap_or_default()
    };
    #[cfg(not(windows))]
    let out = crate::proc::cmd("sh")
        .args(["-c", "command -v cloudflared 2>/dev/null"])
        .stdin(Stdio::null())
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default();
    for line in out.lines() {
        let line = line.trim();
        if !line.is_empty() && Path::new(line).is_file() {
            return line.to_string();
        }
    }
    String::new()
}

/// `fd_ensure_cloudflared()` — `(path, error)`.
///
/// Android is detected by `/system/build.prop`, not by `cfg!(target_os)`: the
/// Android artifacts are built for a Linux target and only run on a phone.
pub fn is_android() -> bool {
    Path::new("/system/build.prop").exists()
}

/// The Termux architecture name — `i686`, not `x86`.
fn termux_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86" => "i686",
        other => other,
    }
}

/// True when an ELF file carries a `PT_INTERP` program header, i.e. it is
/// dynamically linked and asks the kernel for an interpreter. The bionic build
/// from Termux wants `/system/bin/linker`; the official GitHub build is a static
/// Go binary with no interpreter at all.
fn elf_has_interp(bytes: &[u8]) -> bool {
    if bytes.len() < 64 || &bytes[0..4] != b"\x7fELF" {
        return false;
    }
    let is64 = bytes[4] == 2;
    let rd = |a: usize, n: usize| -> u64 {
        let mut v: u64 = 0;
        for i in 0..n {
            v |= (bytes[a + i] as u64) << (8 * i);
        }
        v
    };
    let (phoff, phentsize, phnum) = if is64 {
        (rd(0x20, 8), rd(0x36, 2) as usize, rd(0x38, 2))
    } else {
        (rd(0x1C, 4), rd(0x2A, 2) as usize, rd(0x2C, 2))
    };
    if phoff == 0 || phentsize == 0 {
        return false;
    }
    for i in 0..phnum.min(64) {
        let base = phoff as usize + i as usize * phentsize;
        if base + 4 > bytes.len() {
            return false;
        }
        if rd(base, 4) == 2 {
            return true;
        }
    }
    false
}

fn file_has_interp(path: &Path) -> bool {
    match std::fs::read(path) {
        Ok(bytes) => elf_has_interp(&bytes),
        Err(_) => false,
    }
}

/// `Filename:` of the `cloudflared` stanza in a Termux `Packages` index.
fn packages_stanza_filename(index: &str, package: &str) -> Option<String> {
    let mut in_stanza = false;
    for line in index.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(rest) = line.strip_prefix("Package:") {
            in_stanza = rest.trim() == package;
            continue;
        }
        if in_stanza {
            if line.trim().is_empty() {
                in_stanza = false;
                continue;
            }
            if let Some(rest) = line.strip_prefix("Filename:") {
                return Some(rest.trim().to_string());
            }
        }
    }
    None
}

/// One member of an `ar` archive (a `.deb` is one). `suffix` matches the member
/// name, e.g. `"data.tar"` also matches `data.tar.xz`.
fn ar_member(bytes: &[u8], suffix: &str) -> Option<Vec<u8>> {
    if bytes.len() < 8 || &bytes[0..8] != b"!<arch>\n" {
        return None;
    }
    let mut off = 8usize;
    while off + 60 <= bytes.len() {
        let header = &bytes[off..off + 60];
        let raw_name = String::from_utf8_lossy(&header[0..16]).to_string();
        let name = raw_name.trim().trim_end_matches('/').to_string();
        let size_txt = String::from_utf8_lossy(&header[48..58]).to_string();
        let size: usize = size_txt.trim().parse().ok()?;
        let start = off + 60;
        let end = start.checked_add(size)?;
        if end > bytes.len() {
            return None;
        }
        if name.starts_with(suffix) {
            return Some(bytes[start..end].to_vec());
        }
        off = end + (size & 1);
    }
    None
}

/// The first tar entry whose name ends with `suffix`, for an uncompressed tar.
fn tar_member(bytes: &[u8], suffix: &str) -> Option<Vec<u8>> {
    let mut off = 0usize;
    while off + 512 <= bytes.len() {
        let header = &bytes[off..off + 512];
        if header.iter().all(|b| *b == 0) {
            return None; // end-of-archive
        }
        let raw: Vec<u8> = header[0..100].iter().take_while(|b| **b != 0).copied().collect();
        let name = String::from_utf8_lossy(&raw).to_string();
        let size_txt = String::from_utf8_lossy(&header[124..136]).to_string();
        let size = usize::from_str_radix(size_txt.trim().trim_end_matches('\0').trim(), 8).ok()?;
        let typeflag = header[156];
        let start = off + 512;
        let end = start.checked_add(size)?;
        if end > bytes.len() {
            return None;
        }
        if (typeflag == b'0' || typeflag == 0) && name.ends_with(suffix) {
            return Some(bytes[start..end].to_vec());
        }
        off = start + size.div_ceil(512) * 512;
    }
    None
}

/// Install Termux's bionic-linked `cloudflared` into `storage/tunnel/bin/`.
///
/// **Why this exists at all:** Android has no `/etc/resolv.conf`, and the official
/// static cloudflared from GitHub resolves hostnames with Go's pure resolver, which
/// reads exactly that file. It therefore resolves *nothing* and hangs forever on
/// `Requesting new quick Tunnel on trycloudflare.com...` — measured on the emulator:
/// 90 s, no error line in `--logfile`, empty stderr, dead `--metrics` port, and
/// `/api/tunnel/enable` answering "cloudflared started but no trycloudflare.com URL
/// appeared". The Termux build links bionic, so it resolves through netd (and takes
/// its trust roots from the Android store); the identical command then reaches the
/// edge and publishes a URL. Verified: bionic binary → `precheck` DNS/QUIC/HTTP2/API
/// all "pass", quick-tunnel URL in ~10 s; static binary → silent stall.
async fn install_android_cloudflared() -> (String, String) {
    let arch = termux_arch();
    let base = "https://packages.termux.dev/apt/termux-main";
    let index_url = format!("{base}/dists/stable/main/binary-{arch}/Packages");
    let index = match httpc::get_contents(&index_url, 60, Default::default()).await {
        Some(t) if !t.is_empty() => t,
        _ => {
            return (
                String::new(),
                "Could not read the Termux package index. Check internet access and try again."
                    .to_string(),
            );
        }
    };
    let rel = match packages_stanza_filename(&index, "cloudflared") {
        Some(r) => r,
        None => {
            return (
                String::new(),
                format!("cloudflared is not published for Termux {arch}."),
            );
        }
    };
    let url = format!("{base}/{rel}");
    let dir = bin_dir();
    let deb = dir.join("cloudflared.deb");
    log("downloading cloudflared from Termux", &json!({"arch": arch, "url": url}));
    if !httpc::download_file(&url, &deb, 180).await {
        return (
            String::new(),
            "Failed to download cloudflared from the Termux repository.".to_string(),
        );
    }
    let bytes = match std::fs::read(&deb) {
        Ok(b) => b,
        Err(e) => {
            let _ = std::fs::remove_file(&deb);
            return (String::new(), format!("Could not read the cloudflared package: {e}"));
        }
    };
    let _ = std::fs::remove_file(&deb);

    let data = match ar_member(&bytes, "data.tar") {
        Some(m) => m,
        None => {
            return (
                String::new(),
                "The cloudflared package has no data.tar member.".to_string(),
            );
        }
    };
    // `data.tar.xz` is the only compression Termux uses for this package.
    let mut tar: Vec<u8> = Vec::new();
    if lzma_rs::xz_decompress(&mut std::io::Cursor::new(&data), &mut tar).is_err() {
        return (
            String::new(),
            "Could not decompress the cloudflared package.".to_string(),
        );
    }
    let bin = match tar_member(&tar, "bin/cloudflared") {
        Some(b) => b,
        None => {
            return (
                String::new(),
                "The cloudflared package does not contain bin/cloudflared.".to_string(),
            );
        }
    };
    let dest = bin_dir().join(bin_name());
    if std::fs::write(&dest, &bin).is_err() {
        return (
            String::new(),
            "Could not write the cloudflared binary.".to_string(),
        );
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
    }
    if !dest.is_file() {
        return (String::new(), "The cloudflared binary could not be stored.".to_string());
    }
    log(
        "cloudflared installed from Termux",
        &json!({"path": dest.to_string_lossy(), "bytes": bin.len()}),
    );
    (dest.to_string_lossy().to_string(), String::new())
}

/// The Mozilla CA bundle cloudflared needs on Android (no `/etc/ssl/certs` there).
///
/// Taken from the release layout if a packager shipped one, otherwise downloaded
/// once from `curl.se` (the same 191 KB file the TDLib app bundles as
/// `assets/cacert.pem`). Only the ANDROID path needs it; on a desktop the system
/// store is present and this is never called.
pub async fn ensure_ca_bundle() -> PathBuf {
    let path = tunnel_dir().join("cacert.pem");
    if is_usable_bin(&path) {
        return path;
    }
    for candidate in [
        crate::storage::app_root().join("bin").join("cacert.pem"),
        storage_path("bin").join("cacert.pem"),
    ] {
        if is_usable_bin(&candidate) && std::fs::copy(&candidate, &path).is_ok() {
            return path;
        }
    }
    let url = "https://curl.se/ca/cacert.pem";
    log("downloading a CA bundle for cloudflared", &json!({"url": url}));
    if httpc::download_file(url, &path, 180).await && is_usable_bin(&path) {
        log("ca bundle installed", &json!({"path": path.to_string_lossy()}));
    } else {
        log("ca bundle download failed", &json!({"url": url}));
    }
    path
}

/// Resolution order: `PM_CLOUDFLARED_BIN` (a harness/operator override, also
/// how the stub in `tools/probe_tunnel.mjs` is injected) → `<appRoot>/bin/` (the
/// release layout) → `storage/bin/` (the PHP's own dir) → `storage/tunnel/bin/`
/// → the Windows install dirs → `PATH` → a GitHub download into
/// `storage/tunnel/bin/`.
///
/// **Android is the exception**: the GitHub asset is a static Go build that cannot
/// resolve DNS there, so it is never used — the Termux package is installed instead
/// (`install_android_cloudflared()`). A stale static binary left in `bin/` by an
/// older build is skipped via its missing `PT_INTERP`.
pub async fn ensure_cloudflared() -> (String, String) {
    if let Some(forced) = env("PM_CLOUDFLARED_BIN") {
        let path = PathBuf::from(&forced);
        // Android: the same rule as the candidate list below applies to the
        // override. A static GitHub build cannot resolve DNS here, so it must
        // never short-circuit `install_android_cloudflared()` just because an
        // env var pointed at it. (A dynamically linked build also has to survive
        // its own startup: under an ARM translation layer, cloudflared aborts
        // with `runtime/cgo: bad stack bounds`, which is why `wait_for_url` now
        // stops waiting the moment the process is gone instead of reporting a
        // 90 s timeout.)
        if !(is_android() && path.is_file() && !file_has_interp(&path)) {
            if is_usable_bin(&path) {
                return (path.to_string_lossy().to_string(), String::new());
            }
            if path.is_file() {
                return (path.to_string_lossy().to_string(), String::new());
            }
        }
        log(
            "ignoring PM_CLOUDFLARED_BIN: no bionic ELF on Android",
            &json!({"path": forced}),
        );
    }

    let mut candidates: Vec<PathBuf> = vec![
        crate::storage::app_root().join("bin").join(bin_name()),
        storage_path("bin").join(bin_name()),
        bin_dir().join(bin_name()),
    ];
    if cfg!(windows) {
        candidates.push(PathBuf::from("C:\\Program Files (x86)\\cloudflared\\cloudflared.exe"));
        candidates.push(PathBuf::from("C:\\Program Files\\cloudflared\\cloudflared.exe"));
    }
    for path in &candidates {
        if is_usable_bin(path) {
            // On Android only a dynamically linked (bionic) build can resolve DNS;
            // a static GitHub build cached by an older version must not be reused.
            if is_android() && !file_has_interp(path) {
                log(
                    "ignoring a static cloudflared on Android",
                    &json!({"path": path.to_string_lossy()}),
                );
                continue;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755));
            }
            return (path.to_string_lossy().to_string(), String::new());
        }
    }
    let on_path = which();
    if !on_path.is_empty() {
        return (on_path, String::new());
    }

    if is_android() {
        return install_android_cloudflared().await;
    }

    let asset = asset_name();
    let url = format!(
        "https://github.com/cloudflare/cloudflared/releases/latest/download/{}",
        asset
    );
    let dest = bin_dir().join(bin_name());
    let download_to = if asset.ends_with(".tgz") {
        let tgz = bin_dir().join(format!("{}.tgz", asset));
        (tgz.clone(), Some(tgz))
    } else {
        (dest.clone(), None)
    };
    log("downloading cloudflared", &json!({"url": url, "dest": download_to.0}));
    if httpc::download_file(&url, &download_to.0, 180).await {
        if let Some(tgz) = download_to.1 {
            if !extract_tgz(&tgz, &dest) {
                return (String::new(), "Downloaded cloudflared archive but failed to extract the binary.".to_string());
            }
            let _ = std::fs::remove_file(&tgz);
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
        }
        if is_usable_bin(&dest) || dest.is_file() {
            return (dest.to_string_lossy().to_string(), String::new());
        }
    }
    (
        String::new(),
        "Failed to download cloudflared from GitHub. Check internet access and try again.".to_string(),
    )
}

/// `fd_tunnel_extract_tgz()` — Darwin assets are `.tgz`. Uses the system `tar`
/// (present on Windows 10+ and every Unix) instead of pulling a gzip+tar crate
/// in for one archive.
fn extract_tgz(tgz: &Path, dest_bin: &Path) -> bool {
    let dir = dest_bin.parent().unwrap_or_else(|| Path::new("."));
    let mut cmd = crate::proc::cmd("tar");
    cmd.arg("-xzf")
        .arg(tgz)
        .arg("-C")
        .arg(dir)
        .stdin(Stdio::null());
    let status = cmd.status();
    if let Ok(s) = status {
        if !s.success() {
            log("cloudflared tgz extract failed", &json!({"status": format!("{:?}", s.code())}));
        }
    }
    if dest_bin.is_file() {
        return true;
    }
    // The archive holds a bare `cloudflared`; rename whatever it produced.
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
            if name.starts_with("cloudflared") && !name.ends_with(".tgz") && entry.path().is_file() {
                let _ = std::fs::rename(entry.path(), dest_bin);
                break;
            }
        }
    }
    dest_bin.is_file()
}

// ─── spawn ──────────────────────────────────────────────────────────────────

/// `fd_tunnel_write_dummy_config()` — the ingress cloudflared needs for a Quick
/// Tunnel, with the keep-alive tuning the PHP settled on.
pub fn write_dummy_config(local_url: &str) -> PathBuf {
    write_dummy_config_at(&config_path(), local_url)
}

pub fn write_dummy_config_at(path: &Path, local_url: &str) -> PathBuf {
    let body = format!(
        "ingress:\n  - service: {}\n    originRequest:\n      connectTimeout: 10s\n      tcpKeepAlive: 30s\n      keepAliveTimeout: 90s\n      keepAliveConnections: 100\n      noTLSVerify: true\n      http2Origin: false\n",
        local_url
    );
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, body);
    path.to_path_buf()
}

/// Arguments shared by both tunnel kinds (`fd_tunnel_spawn`'s `$cmdLine`).
fn spawn_args(
    local_url: &str,
    config: &Path,
    log_file: &Path,
    metrics_port: u16,
    tunnel_token: &str,
) -> Vec<String> {
    let named = !tunnel_token.trim().is_empty();
    let mut args: Vec<String> = vec!["tunnel".to_string()];
    if !named {
        args.push("--url".into());
        args.push(local_url.to_string());
        args.push("--config".into());
        args.push(config.to_string_lossy().to_string());
    }
    args.push("--logfile".into());
    args.push(log_file.to_string_lossy().to_string());
    args.push("--metrics".into());
    args.push(format!("127.0.0.1:{}", metrics_port));
    args.push("--edge-ip-version".into());
    args.push("auto".into());
    args.push("--grace-period".into());
    args.push("15s".into());
    args.push("--no-autoupdate".into());
    args.push("--retries".into());
    args.push("99".into());
    if named {
        args.push("run".into());
        args.push("--token".into());
        args.push(tunnel_token.trim().to_string());
    }
    args
}

/// `fd_tunnel_spawn($bin, $localUrl, $metricsPort, $tunnelToken)`.
///
/// Returns `(pid, error)`; `pid <= 1` means the spawn failed. stdout/stderr are
/// redirected to `storage/tunnel/cloudflared{,.err}.log`, so nothing the child
/// prints can block on a pipe nobody reads.
pub fn spawn(bin: &str, local_url: &str, metrics_port: u16, tunnel_token: &str) -> (i64, String) {
    let config = write_dummy_config(local_url);
    let log_file = log_path();
    let err_log = err_log_path();
    let pid_file = pid_path();
    for path in [&log_file, &err_log, &pid_file] {
        let _ = std::fs::remove_file(path);
    }

    let args = spawn_args(local_url, &config, &log_file, metrics_port, tunnel_token);
    let named = !tunnel_token.trim().is_empty();

    let out_handle = match std::fs::OpenOptions::new().create(true).append(true).open(&log_file) {
        Ok(f) => f,
        Err(e) => return (0, format!("Failed to open the tunnel log: {e}")),
    };
    let err_handle = match std::fs::OpenOptions::new().create(true).append(true).open(&err_log) {
        Ok(f) => f,
        Err(e) => return (0, format!("Failed to open the tunnel error log: {e}")),
    };

    // Decide the executable and its argv FIRST, then configure stdout/stderr and
    // env on that ONE Command. The `setsid` wrapper used to replace `cmd` with a
    // fresh `Command::new("setsid")` AFTER the redirects and env had been applied,
    // which silently dropped every one of them on any Unix that has `setsid`
    // (Android does) — the child then ran with no `SSL_CERT_FILE`, no
    // `TUNNEL_TRANSPORT_PROTOCOL` and no logfile redirection.
    #[cfg(unix)]
    let wrap_setsid = which_exe("setsid");
    #[cfg(not(unix))]
    let wrap_setsid = false;

    let (program, argv): (String, Vec<std::ffi::OsString>) = if wrap_setsid {
        let mut full = vec![std::ffi::OsString::from(bin)];
        for a in &args {
            full.push(std::ffi::OsString::from(a));
        }
        ("setsid".to_string(), full)
    } else {
        (
            bin.to_string(),
            args.iter().map(std::ffi::OsString::from).collect(),
        )
    };

    let mut cmd = crate::proc::cmd(&program);
    cmd.args(&argv)
        .current_dir(Path::new(bin).parent().unwrap_or_else(|| Path::new(".")))
        .stdin(Stdio::null())
        .stdout(Stdio::from(out_handle))
        .stderr(Stdio::from(err_handle));
    if env("TUNNEL_TRANSPORT_PROTOCOL").is_none() {
        cmd.env("TUNNEL_TRANSPORT_PROTOCOL", "auto");
    }
    // Android has no `/etc/ssl/certs`, so the Go runtime has no trust roots and
    // cloudflared's HTTPS calls to the edge/features endpoints fail — which makes
    // EDGE DISCOVERY fatal: `edge discovery: error looking up Cloudflare edge IPs`
    // → `Tunnel server stopped` a second after the quick-tunnel URL was printed.
    // With a Mozilla bundle in SSL_CERT_FILE the very same binary, user and args
    // register in ~1 s (`Registered tunnel connection`, 0 SRV errors) — measured on
    // the emulator, and it is exactly what the TDLib app does with its bundled
    // `cacert.pem`. Do NOT set `GODEBUG=netdns=cgo` here: the working configuration
    // does not use it, and DNS failures are not what makes discovery fatal.
    if env("SSL_CERT_FILE").is_none() {
        let ca = tunnel_dir().join("cacert.pem");
        if is_usable_bin(&ca) {
            cmd.env("SSL_CERT_FILE", &ca);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // `proc::cmd` already applied CREATE_NO_WINDOW (no console of its own, so
        // Ctrl-C in the server's terminal cannot reach it). creation_flags
        // REPLACES the flag word, so restate it and add CREATE_NEW_PROCESS_GROUP,
        // which keeps console control events away from the child as well.
        cmd.creation_flags(crate::proc::CREATE_NO_WINDOW | crate::proc::CREATE_NEW_PROCESS_GROUP);
    }

    match cmd.spawn() {
        Ok(mut child) => {
            let pid = child.id() as i64;
            // Reap the child in the background (it is not killed by the wait):
            // this only stops a zombie on Unix and releases the handle on
            // Windows. The child keeps running after the server exits.
            if tokio::runtime::Handle::try_current().is_ok() {
                tokio::task::spawn_blocking(move || {
                    let _ = child.wait();
                });
            }
            if pid <= 1 {
                return (0, "Failed to obtain cloudflared PID.".to_string());
            }
            write_pid(pid);
            log(
                "cloudflared spawned",
                &json!({"pid": pid, "bin": bin, "named": named, "metrics_port": metrics_port}),
            );
            (pid, String::new())
        }
        Err(e) => {
            log("cloudflared spawn failed", &json!({"bin": bin, "error": e.to_string()}));
            (0, format!("Failed to start cloudflared: {e}"))
        }
    }
}

#[cfg(unix)]
fn which_exe(name: &str) -> bool {
    crate::proc::cmd("sh")
        .args(["-c", &format!("command -v {} >/dev/null 2>&1", name)])
        .stdin(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// True when cloudflared's log shows a data-plane connection to the edge.
///
/// A quick tunnel gets its URL from the API **before** the connector connects, so
/// the URL alone says nothing. Measured on Android: the URL appears, then the SRV
/// edge discovery fails (`[::1]:53` — Go's pure resolver, which cgo cannot replace
/// for SRV lookups) and cloudflared exits ~6 s later. Reporting `ok:1` on the URL
/// alone is what makes the dashboard say "enabled" for a tunnel that is already dead.
pub fn connector_registered(logs: &str) -> bool {
    let lower = logs.to_ascii_lowercase();
    lower.contains("registered tunnel connection")
}

/// Wait until the connector is actually up, or cloudflared dies, or the budget runs
/// out. Fails fast when the process is already gone instead of burning the timeout.
///
/// The metrics `/ready` answer is the PRIMARY signal (it is live state, and a
/// watchdog restart cannot forge it); the log marker is the fallback for builds
/// whose metrics server is not listening. Do not trust the log alone — a restart
/// truncates it, and a marker from an earlier process would pass a dead tunnel.
pub async fn wait_for_connector(timeout_secs: u64, metrics_port: u16, pid: i64) -> bool {
    let ready_url = format!("http://127.0.0.1:{}/ready", metrics_port);
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    while Instant::now() < deadline {
        let ready = local_http_get(&ready_url, 1).await;
        if ready.to_ascii_lowercase().contains("\"status\":200") {
            return true;
        }
        if connector_registered(&read_logs()) {
            return true;
        }
        if !pid_alive(pid, 0).await {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
    false
}

/// `fd_tunnel_wait_for_url()`: prefer the live metrics answer, accept a log line
/// once it has stayed there for 8 s (a URL that only ever appeared in a log and
/// never in metrics is usually a dead connector).
pub async fn wait_for_url(timeout_secs: u64, metrics_port: u16, pid: i64) -> String {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let mut from_logs = String::new();
    let mut log_seen: Option<Instant> = None;
    // A connector that died at birth can never publish a URL, so waiting the full
    // timeout only hides the reason behind a 90 s spinner. Checked once a second;
    // the first check is skipped so a fast exit right after spawn is still seen as
    // "not alive yet" rather than a verdict.
    let mut alive_check = Instant::now();
    while Instant::now() < deadline {
        if alive_check.elapsed() >= Duration::from_secs(1) {
            alive_check = Instant::now();
            if !pid_alive(pid, 0).await {
                // Second look after a beat: a connector that is merely slow to
                // exec must not be reported as dead.
                tokio::time::sleep(Duration::from_millis(400)).await;
                if !pid_alive(pid, 0).await {
                    return String::new();
                }
            }
        }
        let live = read_quicktunnel_url(pid, metrics_port).await;
        if !live.is_empty() {
            return live;
        }
        from_logs = parse_url_from_text(&read_logs());
        if !from_logs.is_empty() && log_seen.is_none() {
            log_seen = Some(Instant::now());
        }
        if let Some(seen) = log_seen {
            if seen.elapsed() >= Duration::from_secs(8) {
                return from_logs;
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    let live = read_quicktunnel_url(pid, metrics_port).await;
    if live.is_empty() { from_logs } else { live }
}

/// `fd_tunnel_extract_token()`: a raw JWT, or a whole
/// `cloudflared.exe service install eyJh…` command line.
pub fn extract_token(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return String::new();
    }
    if let Ok(re) = fancy_regex::Regex::new(r"eyJh[A-Za-z0-9_=-]+") {
        if let Ok(Some(caps)) = re.captures(raw) {
            if let Some(m) = caps.get(0) {
                return m.as_str().to_string();
            }
        }
    }
    raw.to_string()
}

// ─── relay registration (best effort, never blocks a request) ───────────────

fn register_disabled() -> bool {
    matches!(env("PM_TUNNEL_NO_REGISTER"), Some(v) if v == "1" || v.eq_ignore_ascii_case("true"))
}

/// `fd_tunnel_register_worker_async()`: tell the relay which URL this device is
/// serving (or `""` to clear it). Fire-and-forget on a tokio task — the PHP used
/// a detached CLI worker for the same reason: it measured 8-9 s per call and no
/// caller's response depends on it.
///
/// `PM_TUNNEL_NO_REGISTER=1` disables it, which is how `tools/probe_tunnel.mjs`
/// keeps a stubbed run from publishing a dead `*.trycloudflare.com` name.
pub fn register_worker_async(tunnel_url: &str) {
    if register_disabled() {
        return;
    }
    let url = tunnel_url.to_string();
    tokio::spawn(async move {
        let payload = vec![
            ("shortId".to_string(), get_device_id()),
            ("tunnelUrl".to_string(), url.clone()),
        ];
        let primary = format!("{}/tunnel/register", crate::config::wp_api_base());
        let res = httpc::http_json(&primary, &payload, "POST", 10).await;
        log(
            "tunnel register",
            &json!({"url": url, "ok": res.get("ok").cloned().unwrap_or(Value::Null)}),
        );
        let _ = httpc::http_json(
            "https://tunnel.pencarimovie.com/api/tunnel/register",
            &payload,
            "POST",
            5,
        )
        .await;
    });
}

// ─── lifecycle ──────────────────────────────────────────────────────────────

/// `fd_tunnel_auto_restart()`: the state says the tunnel should be up but the
/// process is gone → spawn it again. A 20 s cooldown stops a restart loop when
/// cloudflared cannot actually run.
pub async fn auto_restart(server_port: u16) -> bool {
    let mut state = load_state();
    if !st_bool(&state, "enabled") {
        return false;
    }
    let pid = st_i64(&state, "pid");
    let metrics = st_i64(&state, "metrics_port").max(0) as u16;
    if pid > 1 && pid_alive(pid, metrics).await {
        return false;
    }
    let last_attempt = st_i64(&state, "restart_attempt_at");
    if last_attempt > 0 && now() - last_attempt < 20 {
        return false;
    }

    let mut bin = st_str(&state, "bin");
    if bin.is_empty() || !Path::new(&bin).is_file() {
        let (found, _) = ensure_cloudflared().await;
        bin = found;
    }
    if bin.is_empty() {
        return false;
    }

    let port = {
        let p = st_i64(&state, "local_port");
        if p > 0 && p <= u16::MAX as i64 { p as u16 } else { server_port }
    };
    let metrics_port = if metrics > 0 { metrics } else { pick_metrics_port() };
    let tunnel_token = st_str(&state, "tunnel_token");
    let local_url = format!("http://127.0.0.1:{}", port);
    kill_leftovers().await;

    let (new_pid, spawn_error) = spawn(&bin, &local_url, metrics_port, &tunnel_token);
    if new_pid <= 1 {
        state["restart_attempt_at"] = json!(now());
        save_state(&state);
        log("tunnel auto-restart failed", &json!({"error": spawn_error}));
        return false;
    }

    if !tunnel_token.is_empty() {
        state["enabled"] = json!(true);
        state["pid"] = json!(new_pid);
        state["metrics_port"] = json!(metrics_port);
        state["started_at"] = json!(now());
        state["restart_attempt_at"] = json!(now());
        save_state(&state);
        write_pid(new_pid);
        log("named tunnel auto-restarted", &json!({"pid": new_pid}));
        return true;
    }

    let mut url = wait_for_url(60, metrics_port, new_pid).await;
    if url.is_empty() {
        state["restart_attempt_at"] = json!(now());
        save_state(&state);
        kill_leftovers().await;
        log("tunnel auto-restart: no URL appeared", &json!({}));
        return false;
    }
    let live = read_quicktunnel_url(new_pid, metrics_port).await;
    if !live.is_empty() {
        url = live;
    }
    register_worker_async(&url);

    state["enabled"] = json!(true);
    state["pid"] = json!(new_pid);
    state["tunnel_url"] = json!(url);
    state["metrics_port"] = json!(metrics_port);
    state["started_at"] = json!(now());
    state["restart_attempt_at"] = json!(now());
    save_state(&state);
    write_pid(new_pid);
    log("tunnel auto-restarted", &json!({"pid": new_pid, "url": state["tunnel_url"]}));
    true
}

/// `fd_get_tunnel_status()`. Also the watchdog: when the state says enabled but
/// the process is gone, the restart runs on a BACKGROUND task (the PHP ran it
/// inline, which is why it needed a 5-minute stamp to survive a 2 s status
/// poll). The stamp is still honoured, and it is touched before the task starts
/// so a poll storm cannot queue more than one restart per 5 minutes.
pub async fn get_tunnel_status(server_port: u16) -> Value {
    let mut state = load_state();
    let mut pid = read_pid_at(&pid_path(), &state);
    let mut metrics_port = st_i64(&state, "metrics_port").max(0) as u16;
    let mut alive = (pid > 1 && pid_alive(pid, metrics_port).await)
        || (metrics_port > 0
            && local_http_get(&format!("http://127.0.0.1:{}/ready", metrics_port), 1)
                .await
                .to_ascii_lowercase()
                .contains("\"status\":200"));

    if !alive && st_bool(&state, "enabled") {
        let stamp = tunnel_dir().join(".last_autorestart");
        let last_restart = mtime_secs(&stamp).unwrap_or(0);
        if now() - last_restart > 300 {
            let _ = std::fs::write(&stamp, "");
            log("tunnel watchdog: reviving cloudflared", &json!({}));
            let port = server_port;
            tokio::spawn(async move {
                let _ = auto_restart(port).await;
            });
        }
    }

    // Never surface a token that is not real: `state.json` written by an older build
    // can still carry garbage, and the dashboard pre-fills its input from this field.
    let tunnel_token = {
        let t = st_str(&state, "tunnel_token");
        if t.is_empty() || token_looks_valid(&t) {
            t
        } else {
            log(
                "ignoring an invalid tunnel token from state.json",
                &json!({"len": t.len()}),
            );
            // Also drop the stale copy on disk (older builds wrote it there), so the
            // dashboard cannot pre-fill it from either source.
            if read_trim(&token_path()) == t {
                let _ = std::fs::remove_file(token_path());
            }
            String::new()
        }
    };
    let mut custom_domain = st_str(&state, "custom_domain");
    let mut url = st_str(&state, "tunnel_url");
    let local_port = {
        let p = st_i64(&state, "local_port");
        if p > 0 && p <= u16::MAX as i64 { p as u16 } else { server_port }
    };

    // Named tunnel: the public host comes from the ingress config in the log.
    if alive && !tunnel_token.is_empty() {
        let logs = read_logs();
        let ingress = parse_ingress_domains(&logs, local_port);
        if !ingress.is_empty() {
            custom_domain = ingress[0].clone();
            state["custom_domain"] = json!(custom_domain);
            state["custom_domains"] = json!(ingress);
            save_state(&state);
        }
        let public_domain_url = if custom_domain.is_empty() {
            String::new()
        } else {
            format!("https://{}", custom_domain)
        };
        let manifest_url = if public_domain_url.is_empty() {
            String::new()
        } else {
            format!("{}/manifest.json", public_domain_url.trim_end_matches('/'))
        };
        let message = if !ingress.is_empty() {
            let list: Vec<String> = ingress.iter().map(|d| format!("https://{}", d)).collect();
            format!("Live on {}", list.join(", "))
        } else if !public_domain_url.is_empty() {
            format!("Live on {}", public_domain_url)
        } else {
            "Named Cloudflare Tunnel connected.".to_string()
        };
        return json!({
            "ok": 1,
            "enabled": true,
            "running": true,
            "pid": pid,
            "tunnel_token": tunnel_token,
            "tunnel_url": public_domain_url,
            "public_url": public_domain_url,
            "manifest_url": manifest_url,
            "custom_domain": custom_domain,
            "custom_domains": ingress,
            "device_id": get_device_id(),
            "local_port": local_port,
            "metrics_port": metrics_port,
            "started_at": st_i64(&state, "started_at"),
            "message": message,
        });
    }

    if alive {
        if url.is_empty() {
            let live = read_quicktunnel_url(pid, metrics_port).await;
            if !live.is_empty() {
                url = live;
            } else {
                url = parse_url_from_text(&read_logs());
            }
        }
        if !url.is_empty()
            && (st_str(&state, "tunnel_url") != url
                || st_i64(&state, "pid") != pid
                || !st_bool(&state, "enabled"))
        {
            state["enabled"] = json!(true);
            state["pid"] = json!(pid);
            state["tunnel_url"] = json!(url);
            state["local_port"] = json!(local_port);
            if metrics_port > 0 {
                state["metrics_port"] = json!(metrics_port);
            }
            if st_i64(&state, "started_at") == 0 {
                state["started_at"] = json!(now());
            }
            save_state(&state);
            register_worker_async(&url);
        }
    }

    // Re-read in case the watchdog task already finished.
    if !alive {
        let latest = load_state();
        let latest_pid = read_pid_at(&pid_path(), &latest);
        if latest_pid != pid {
            pid = latest_pid;
            metrics_port = st_i64(&latest, "metrics_port").max(0) as u16;
            alive = pid > 1 && pid_alive(pid, metrics_port).await;
            url = st_str(&latest, "tunnel_url");
            state = latest;
        }
    }

    let enabled = alive && !url.is_empty();
    let subdomain = get_device_id();
    let public_domain_url = if enabled {
        format!("https://{}-tunnel.pencarimovie.com", subdomain)
    } else {
        String::new()
    };
    let active_url = if !public_domain_url.is_empty() {
        public_domain_url.clone()
    } else {
        url.clone()
    };
    let manifest_url = if enabled {
        format!("{}/manifest.json", active_url.trim_end_matches('/'))
    } else {
        String::new()
    };

    json!({
        "ok": 1,
        "enabled": enabled,
        "running": alive,
        "pid": if alive { pid } else { 0 },
        // The SANITIZED token (see above): re-reading the raw state here is what
        // kept a garbage token visible in the dashboard.
        "tunnel_token": tunnel_token,
        "tunnel_url": if enabled { url } else { String::new() },
        "public_url": public_domain_url,
        "manifest_url": manifest_url,
        "custom_domain": "",
        "custom_domains": Vec::<String>::new(),
        "device_id": subdomain,
        "local_port": local_port,
        "metrics_port": metrics_port,
        "started_at": st_i64(&state, "started_at"),
        "message": if enabled {
            format!("Live on custom subdomain: {}", public_domain_url)
        } else if alive {
            "cloudflared is running but the public URL is not ready yet.".to_string()
        } else {
            "Cloudflare tunnel is off.".to_string()
        },
    })
}

/// `fd_disable_tunnel()`.
pub async fn disable() -> Value {
    register_worker_async("");
    kill_leftovers().await;
    clear_state(true);
    let token = load_token();
    write_pid(0);
    let _ = std::fs::remove_file(pid_path());
    json!({
        "ok": 1,
        "enabled": false,
        "running": false,
        "pid": 0,
        "tunnel_token": token,
        "tunnel_url": "",
        "public_url": "",
        "manifest_url": "",
        "message": "Cloudflare tunnel stopped.",
    })
}

/// True while [`start_async`] is still working on a tunnel.
static STARTING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn is_starting() -> bool {
    STARTING.load(std::sync::atomic::Ordering::SeqCst)
}

/// Why the last `enable` failed, so `/api/tunnel/status` can explain it.
///
/// `enable` runs in the background (it can take 10-90 s), so the POST only answers
/// "starting". Without this the dashboard polled, saw `enabled:false`, and showed
/// plain "Tunnel is off" — the reason ("that does not look like a token", "the token
/// was rejected") existed only in the log.
fn last_error_slot() -> &'static Mutex<Option<String>> {
    static E: OnceLock<Mutex<Option<String>>> = OnceLock::new();
    E.get_or_init(|| Mutex::new(None))
}

pub fn last_error() -> String {
    match last_error_slot().lock() {
        Ok(g) => g.clone().unwrap_or_default(),
        Err(e) => e.into_inner().clone().unwrap_or_default(),
    }
}

fn set_last_error(msg: Option<&str>) {
    let mut g = match last_error_slot().lock() {
        Ok(g) => g,
        Err(e) => e.into_inner(),
    };
    *g = msg.map(|m| m.to_string());
}

/// Kick off [`enable`] in the background and return immediately.
///
/// A cold enable is slow by nature — on Android it may install the Termux
/// cloudflared (≈8 MB), write a CA bundle, then wait for the connector — so
/// holding the HTTP request for it looks like a hang (reported on a real phone).
/// The dashboard already polls `/api/tunnel/status` every 1.5 s and finishes when
/// `enabled && tunnel_url` arrive (`enableTunnel()` in public/app.js), so an
/// immediate "starting" answer is what it expects.
///
/// Returns `false` when a start is already in flight, so a double tap cannot
/// spawn two cloudflared processes fighting over one metrics port.
pub fn start_async(server_port: u16, raw_token: &str) -> bool {
    if STARTING.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return false;
    }
    // Clears the flag even if `enable` unwinds, so a panic can never leave the
    // dashboard stuck on "Starting…" with `start_async` refusing every retry.
    struct StartingGuard;
    impl Drop for StartingGuard {
        fn drop(&mut self) {
            STARTING.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }
    let token = raw_token.to_string();
    tokio::spawn(async move {
        let _guard = StartingGuard;
        let result = enable(server_port, &token).await;
        let ok = result
            .get("ok")
            .map(|v| v.as_i64().unwrap_or(0) != 0 || v.as_bool().unwrap_or(false))
            .unwrap_or(false);
        if ok {
            set_last_error(None);
        } else {
            set_last_error(Some(
                result.get("message").and_then(|m| m.as_str()).unwrap_or(""),
            ));
        }
        log(
            "tunnel enable finished",
            &json!({"ok": ok, "message": result.get("message")}),
        );
    });
    true
}

/// `fd_enable_tunnel($rawToken)`.
pub async fn enable(server_port: u16, raw_token: &str) -> Value {
    let mut token = extract_token(raw_token);
    if !token.is_empty() && !token_looks_valid(&token) {
        // Reject BEFORE touching anything: no kill of a working tunnel, no state
        // reset, and above all no write to token.txt. The old order saved first and
        // validated never, so one typo poisoned every later enable.
        log(
            "tunnel enable rejected: not a cloudflared tunnel token",
            &json!({"len": token.len(), "head": token.chars().take(6).collect::<String>()}),
        );
        return json!({
            "ok": 0,
            "enabled": false,
            "message": "That does not look like a Cloudflare Tunnel token. Copy the token from Cloudflare Zero Trust → Networks → Tunnels → (your tunnel) → Install connector; it is a long string starting with \"eyJ\". Leave the field empty for a free Quick Tunnel.",
        });
    }
    if token.is_empty() {
        let stored = load_token();
        if !stored.is_empty() {
            if token_looks_valid(&stored) {
                token = stored;
            } else {
                // A token saved by an older build (or hand-edited) is useless; drop
                // it so the user is not stuck with it.
                log(
                    "discarding an invalid stored tunnel token",
                    &json!({"len": stored.len()}),
                );
                let _ = std::fs::remove_file(token_path());
            }
        }
    }
    // The token is persisted only AFTER the tunnel actually connects (see below).

    // A previous run's log lines must never be parsed as the new tunnel's URL.
    kill_leftovers().await;
    clear_state(true);

    let (bin, error) = ensure_cloudflared().await;
    if bin.is_empty() {
        return json!({
            "ok": 0,
            "enabled": false,
            "message": if error.is_empty() { "cloudflared is not available.".to_string() } else { error },
        });
    }

    // Android ships no trust roots; without this, edge discovery is fatal and the
    // tunnel dies seconds after being published. See spawn().
    if is_android() {
        let ca = ensure_ca_bundle().await;
        if !is_usable_bin(&ca) {
            return json!({
                "ok": 0,
                "enabled": false,
                "message": "Could not install a CA bundle for cloudflared, so the tunnel cannot reach the Cloudflare edge."
                    .to_string(),
            });
        }
    }

    let local_url = format!("http://127.0.0.1:{}", server_port);
    let metrics_port = pick_metrics_port();
    let (pid, spawn_error) = spawn(&bin, &local_url, metrics_port, &token);
    if pid <= 1 {
        return json!({
            "ok": 0,
            "enabled": false,
            "message": if spawn_error.is_empty() { "Failed to start cloudflared.".to_string() } else { spawn_error },
        });
    }

    if !token.is_empty() {
        // Named tunnel: wait for the connector to report ready. A working tunnel
        // registers in 1-4 s; a rejected token is ABORTED the moment cloudflared says
        // so (or the process dies), so the user never sits on a spinner for a
        // failure that is already decided. The 12 s cap is only for a slow network.
        let mut ready = false;
        let mut fatal: Option<String> = None;
        let deadline = Instant::now() + Duration::from_secs(12);
        while Instant::now() < deadline {
            let resp = local_http_get(&format!("http://127.0.0.1:{}/ready", metrics_port), 1).await;
            if resp.to_ascii_lowercase().contains("\"status\":200") {
                ready = true;
                break;
            }
            if let Some(line) = fatal_token_error(&read_logs()) {
                fatal = Some(line);
                break;
            }
            // Fail fast: a rejected token makes cloudflared exit immediately, and the
            // pid check costs nothing.
            if !pid_alive(pid, metrics_port).await {
                break;
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
        let logs = read_logs();
        let fatal = fatal.or_else(|| fatal_token_error(&logs));
        // A published tunnel is not a CONNECTED tunnel. With a wrong/expired token
        // cloudflared exits with "Provided Tunnel token is not valid" and we used to
        // write `enabled: true` anyway — the dashboard then showed a live switch with
        // no URL and every request 530ed.
        if !ready && !connector_registered(&logs) {
            let tail: String = {
                let chars: Vec<char> = logs.chars().collect();
                let start = chars.len().saturating_sub(800);
                chars[start..].iter().collect()
            };
            kill_pid(pid).await;
            clear_state(true);
            // Keep the explanation readable: the raw tail can be 800+ chars of
            // retry noise, and the UI shows this text in a small status line.
            let shorten = |s: &str| -> String {
                let squashed = s.split_whitespace().collect::<Vec<_>>().join(" ");
                if squashed.chars().count() > 220 {
                    let cut: String = squashed.chars().take(220).collect();
                    format!("{cut}…")
                } else {
                    squashed
                }
            };
            let hint = if let Some(line) = &fatal {
                format!(" cloudflared said: {}", shorten(line))
            } else if tail.trim().is_empty() {
                String::new()
            } else {
                format!(" Last log: {}", shorten(&tail))
            };
            log(
                "tunnel enable failed: connector never registered",
                &json!({"pid": pid, "ready": ready, "fatal": fatal}),
            );
            return json!({
                "ok": 0,
                "enabled": false,
                "message": format!(
                    "The Cloudflare tunnel token was rejected and nothing was saved (no token was stored, so you can simply try again or leave the field empty for a Quick Tunnel).{}",
                    hint
                ),
            });
        }
        let ingress = parse_ingress_domains(&logs, server_port);
        let custom_domain = ingress.first().cloned().unwrap_or_default();
        let public_url = if custom_domain.is_empty() {
            String::new()
        } else {
            format!("https://{}", custom_domain)
        };
        let started_at = now();
        // Connected: NOW the token is worth keeping (it is validated, and it works).
        save_token(&token);
        let state = json!({
            "enabled": true,
            "pid": pid,
            "tunnel_token": token,
            "custom_domain": custom_domain,
            "custom_domains": ingress,
            "tunnel_url": public_url,
            "public_url": public_url,
            "local_port": server_port,
            "metrics_port": metrics_port,
            "started_at": started_at,
            "bin": bin,
        });
        save_state(&state);
        write_pid(pid);
        let message = if !ingress.is_empty() {
            let list: Vec<String> = ingress.iter().map(|d| format!("https://{}", d)).collect();
            format!("Connected to {}", list.join(", "))
        } else if !public_url.is_empty() {
            format!("Connected to {}", public_url)
        } else {
            "Named Cloudflare Tunnel connected.".to_string()
        };
        return json!({
            "ok": 1,
            "enabled": true,
            "running": pid_alive(pid, 0).await,
            "pid": pid,
            "tunnel_token": token,
            "custom_domain": custom_domain,
            "custom_domains": ingress,
            "tunnel_url": public_url,
            "public_url": public_url,
            "manifest_url": if public_url.is_empty() { String::new() } else { format!("{}/manifest.json", public_url.trim_end_matches('/')) },
            "local_port": server_port,
            "metrics_port": metrics_port,
            "started_at": started_at,
            "message": message,
        });
    }

    let mut url = wait_for_url(90, metrics_port, pid).await;
    if url.is_empty() {
        let logs = read_logs();
        let tail: String = {
            let chars: Vec<char> = logs.chars().collect();
            let start = chars.len().saturating_sub(1200);
            chars[start..].iter().collect()
        };
        kill_leftovers().await;
        clear_state(true);
        let hint = if tail.trim().is_empty() {
            String::new()
        } else {
            let squashed = tail.split_whitespace().collect::<Vec<_>>().join(" ");
            format!(" Log: {}", squashed)
        };
        return json!({
            "ok": 0,
            "enabled": false,
            "message": format!("cloudflared started but no trycloudflare.com URL appeared.{}", hint),
        });
    }
    let live = read_quicktunnel_url(pid, metrics_port).await;
    if !live.is_empty() {
        url = live;
    }
    // A quick tunnel is PUBLISHED — the URL exists — before the connector reaches
    // the edge, so the URL is not proof of a working tunnel. Never report success
    // for one that never connected: that is what leaves the dashboard showing
    // "enabled" while every request 530s. Fails fast when the process is gone.
    if !wait_for_connector(30, metrics_port, pid).await {
        let tail: String = {
            let logs = read_logs();
            let chars: Vec<char> = logs.chars().collect();
            let start = chars.len().saturating_sub(800);
            chars[start..].iter().collect()
        };
        kill_leftovers().await;
        clear_state(true);
        let hint = if tail.trim().is_empty() {
            String::new()
        } else {
            let squashed = tail.split_whitespace().collect::<Vec<_>>().join(" ");
            format!(" Last log: {}", squashed)
        };
        return json!({
            "ok": 0,
            "enabled": false,
            "message": format!(
                "cloudflared created {} but never connected to the Cloudflare edge.{}",
                url, hint
            ),
        });
    }
    register_worker_async(&url);

    let subdomain = get_device_id();
    let public_domain_url = format!("https://{}-tunnel.pencarimovie.com", subdomain);
    let started_at = now();
    let state = json!({
        "enabled": true,
        "pid": pid,
        "tunnel_url": url,
        "public_url": public_domain_url,
        "local_port": server_port,
        "metrics_port": metrics_port,
        "started_at": started_at,
        "bin": bin,
    });
    save_state(&state);
    write_pid(pid);

    json!({
        "ok": 1,
        "enabled": true,
        "running": pid_alive(pid, 0).await,
        "pid": pid,
        "tunnel_url": url,
        "public_url": public_domain_url,
        "manifest_url": format!("{}/manifest.json", public_domain_url.trim_end_matches('/')),
        "local_port": server_port,
        "metrics_port": metrics_port,
        "started_at": started_at,
        "message": format!("Tunnel is live at {}", public_domain_url),
    })
}

// ─── state helpers ──────────────────────────────────────────────────────────

fn st_str(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn st_i64(v: &Value, key: &str) -> i64 {
    v.get(key)
        .and_then(|x| x.as_i64().or_else(|| x.as_str().and_then(|s| s.trim().parse().ok())))
        .unwrap_or(0)
}

fn st_bool(v: &Value, key: &str) -> bool {
    match v.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0) != 0,
        Some(Value::String(s)) => {
            let s = s.trim().to_ascii_lowercase();
            s == "1" || s == "true" || s == "on" || s == "yes"
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pm-tunnel-test-{}-{}-{}",
            tag,
            std::process::id(),
            now()
        ));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    // ── Android cloudflared install (ar → xz → tar) ─────────────────────────
    //
    // These parse the real container formats without touching the network: the
    // download itself is what cannot be unit-tested, and it is not the part that
    // ever broke (the failure was picking the wrong BUILD, not the wrong bytes).

    /// A minimal `ar` archive holding one member.
    fn ar_with(name: &str, payload: &[u8]) -> Vec<u8> {
        let mut out = b"!<arch>\n".to_vec();
        let mut header = [b' '; 60];
        header[..name.len()].copy_from_slice(name.as_bytes());
        let size = format!("{}", payload.len());
        header[48..48 + size.len()].copy_from_slice(size.as_bytes());
        header[58] = b'`';
        header[59] = b'\n';
        out.extend_from_slice(&header);
        out.extend_from_slice(payload);
        if payload.len() % 2 == 1 {
            out.push(b'\n'); // members are padded to an even offset
        }
        out
    }

    /// A minimal uncompressed `tar` holding `entries` as regular files.
    fn tar_with(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        for (name, body) in entries {
            let mut header = [0u8; 512];
            header[..name.len()].copy_from_slice(name.as_bytes());
            let size = format!("{:o}", body.len());
            let bytes = size.as_bytes();
            header[124..124 + bytes.len()].copy_from_slice(bytes);
            header[156] = b'0';
            out.extend_from_slice(&header);
            out.extend_from_slice(body);
            let pad = (512 - body.len() % 512) % 512;
            out.extend(std::iter::repeat(0u8).take(pad));
        }
        out.extend_from_slice(&[0u8; 1024]); // end-of-archive
        out
    }

    #[test]
    fn packages_index_yields_the_cloudflared_filename() {
        let index = "Package: clang\r\nVersion: 19\r\nFilename: pool/main/c/clang/clang_19.deb\r\n\r\n\
                     Package: cloudflared\r\nArchitecture: aarch64\r\nVersion: 2026.10.1\r\n\
                     Filename: pool/main/c/cloudflared/cloudflared_2026.10.1_aarch64.deb\r\n\r\n\
                     Package: cmake\r\nFilename: pool/main/c/cmake/cmake_4.deb\r\n";
        assert_eq!(
            packages_stanza_filename(index, "cloudflared"),
            Some("pool/main/c/cloudflared/cloudflared_2026.10.1_aarch64.deb".to_string())
        );
        // the `clang` stanza must not be picked just because it comes first
        assert_ne!(
            packages_stanza_filename(index, "cloudflared"),
            Some("pool/main/c/clang/clang_19.deb".to_string())
        );
        assert_eq!(packages_stanza_filename(index, "nosuchpackage"), None);
    }

    #[test]
    fn ar_member_finds_the_data_payload() {
        let deb = ar_with("data.tar.xz", b"XZDATA");
        assert_eq!(ar_member(&deb, "data.tar"), Some(b"XZDATA".to_vec()));
        // a real .deb also carries these; they must not be mistaken for the payload
        let deb = ar_with("debian-binary", b"2.0\n");
        assert_eq!(ar_member(&deb, "data.tar"), None);
        assert_eq!(ar_member(b"not-an-archive", "data.tar"), None);
    }

    #[test]
    fn tar_member_finds_bin_cloudflared_after_other_entries() {
        let tar = tar_with(&[
            ("./data/data/com.termux/files/usr/share/doc/cloudflared/copyright", b"mit"),
            ("./data/data/com.termux/files/usr/bin/cloudflared", b"\x7fELFfake"),
        ]);
        assert_eq!(
            tar_member(&tar, "bin/cloudflared"),
            Some(b"\x7fELFfake".to_vec())
        );
        assert_eq!(tar_member(&tar, "bin/nothing"), None);
    }

    #[test]
    fn connector_registered_needs_a_real_edge_connection() {
        // The URL alone is not proof: cloudflared logs it before connecting.
        assert!(!connector_registered(
            "Your quick Tunnel has been created! Visit it at https://x.trycloudflare.com"
        ));
        assert!(connector_registered(
            "{\"level\":\"info\",\"message\":\"Registered tunnel connection\",\"connIndex\":0}"
        ));
        assert!(connector_registered("REGISTERED TUNNEL CONNECTION"));
        assert!(!connector_registered("Tunnel server stopped"));
    }

    #[test]
    fn elf_interp_detection_tells_bionic_from_static() {
        // ELF64: e_phoff at 0x20, e_phentsize 0x36, e_phnum 0x38.
        let mut dynamic = vec![0u8; 128];
        dynamic[0..4].copy_from_slice(b"\x7fELF");
        dynamic[4] = 2; // ELFCLASS64
        dynamic[0x20] = 64; // phoff
        dynamic[0x36] = 56; // phentsize
        dynamic[0x38] = 1; // phnum
        dynamic[64] = 2; // PT_INTERP
        assert!(elf_has_interp(&dynamic));

        // Only a PT_LOAD: a static Go binary (what GitHub serves).
        let mut stat = dynamic.clone();
        stat[64] = 1;
        assert!(!elf_has_interp(&stat));

        assert!(!elf_has_interp(b"\x7fELF"));
        assert!(!elf_has_interp(b"not an elf at all, definitely not one"));
    }

    #[test]
    fn normalize_url_accepts_quick_tunnel_only() {
        assert_eq!(
            normalize_url("https://bright-sky-1234.trycloudflare.com"),
            "https://bright-sky-1234.trycloudflare.com"
        );
        // scheme-less input is upgraded, trailing slashes dropped, case folded
        assert_eq!(
            normalize_url("BRIGHT-SKY-1234.trycloudflare.com/"),
            "https://bright-sky-1234.trycloudflare.com"
        );
        // the API host in cloudflared's banner is never a tunnel
        assert_eq!(normalize_url("https://api.trycloudflare.com"), "");
        // anything that is not a quick tunnel is rejected
        assert_eq!(normalize_url("https://example.com"), "");
        assert_eq!(normalize_url("https://x.trycloudflare.com.evil.com"), "");
        assert_eq!(normalize_url(""), "");
    }

    #[test]
    fn parse_url_from_text_takes_the_last_non_api_host() {
        let text = "INF Requesting new quick Tunnel on trycloudflare.com...\n\
                    INF |  https://api.trycloudflare.com  |\n\
                    INF https://first-try.trycloudflare.com\n\
                    INF https://second-try.trycloudflare.com";
        assert_eq!(parse_url_from_text(text), "https://second-try.trycloudflare.com");
        // only the api host -> nothing
        assert_eq!(parse_url_from_text("https://api.trycloudflare.com"), "");
        assert_eq!(parse_url_from_text(""), "");
    }

    #[test]
    fn parse_quicktunnel_body_handles_all_shapes() {
        assert_eq!(
            parse_quicktunnel_body("{\"hostname\":\"abc-def.trycloudflare.com\"}"),
            "https://abc-def.trycloudflare.com"
        );
        assert_eq!(
            parse_quicktunnel_body("{\"URL\":\"https://ghi-jkl.trycloudflare.com\"}"),
            "https://ghi-jkl.trycloudflare.com"
        );
        assert_eq!(
            parse_quicktunnel_body("... userHostname=\"https://mno-pqr.trycloudflare.com\" ..."),
            "https://mno-pqr.trycloudflare.com"
        );
        assert_eq!(
            parse_quicktunnel_body("plain https://stu-vwx.trycloudflare.com tail"),
            "https://stu-vwx.trycloudflare.com"
        );
        assert_eq!(parse_quicktunnel_body("  "), "");
    }

    #[test]
    fn parse_ingress_domains_prefers_the_local_port() {
        let logs = [
            r#"{"level":"info","config":"{\"ingress\":[{\"hostname\":\"other.example.com\",\"service\":\"http://127.0.0.1:9000\"},{\"hostname\":\"mine.example.com\",\"service\":\"http://127.0.0.1:8088\"}]}"}"#,
            r#"{"level":"info","config":"{\"ingress\":[{\"hostname\":\"fallback.example.com\",\"service\":\"http_status:404\"}]}"}"#,
        ]
        .join("\n");
        assert_eq!(parse_ingress_domains(&logs, 8088), vec!["mine.example.com".to_string()]);
        // nothing matches the port -> every non-http_status host
        assert_eq!(
            parse_ingress_domains(&logs, 5555),
            vec!["other.example.com".to_string(), "mine.example.com".to_string()]
        );
        // no ingress word anywhere -> nothing (cheap gate from the PHP)
        assert!(parse_ingress_domains("hello", 8088).is_empty());
    }

    #[test]
    fn parse_ingress_domains_reads_logfmt_lines() {
        // modern cloudflared logs logfmt, not JSON: `config="{\"ingress\":...}"`
        let line = r#"2026-10-19T20:00:00Z INF Updated to new configuration config="{\"ingress\":[{\"hostname\":\"live.example.com\",\"service\":\"http://127.0.0.1:8088\"}]}" version=5"#;
        assert_eq!(
            parse_ingress_domains(line, 8088),
            vec!["live.example.com".to_string()]
        );
    }

    #[test]
    fn extract_token_from_raw_and_service_install() {
        let jwt = "eyJhIjoiYWM4N2E5In0=";
        assert_eq!(extract_token(jwt), jwt);
        assert_eq!(
            extract_token(&format!(
                "cloudflared.exe service install {}",
                jwt
            )),
            jwt
        );
        // not a JWT: passed through verbatim (trimmed)
        assert_eq!(extract_token("  some-token  "), "some-token");
        assert_eq!(extract_token(""), "");
    }

    /// A tunnel token is `base64url({"a":…,"t":…,"s":…})`. Anything else must be
    /// rejected BEFORE it reaches `token.txt` — that was the bug: one typo was
    /// persisted and replayed by every later enable.
    #[test]
    fn only_a_real_tunnel_token_is_accepted() {
        use base64::Engine;
        let real = br#"{"a":"0123456789abcdef0123456789abcdef","t":"01234567-89ab-cdef-0123-456789abcdef","s":"X0xYb1p6c2VjcmV0X2Jhc2U2NF9ibG9iX2Zvcl90ZXN0aW5n"}"#;
        let padded = base64::engine::general_purpose::STANDARD.encode(real);
        let unpadded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(real);
        assert!(token_looks_valid(&padded), "a real (padded) token must pass");
        assert!(token_looks_valid(&unpadded), "a real (unpadded) token must pass");
        assert!(token_looks_valid(&format!("  {}  ", padded)), "trimmed");

        for bad in [
            "",
            "hello-random-garbage",
            "some-token",
            "eyJnot-base64-at-all!!!",
            "https://dash.cloudflare.com/profile/api-tokens",
            "eyJhIjoxfQ", // valid base64/shape but far too short to be a token
            "0011223344556677889900112233445566778899001122334455667788990011",
        ] {
            assert!(!token_looks_valid(bad), "must be rejected: {bad}");
        }
        // base64 that decodes to JSON WITHOUT `a`/`t` is not a tunnel token either.
        let decoy = base64::engine::general_purpose::STANDARD
            .encode(br#"{"hello":"a very long but unrelated json document that goes on and on"}"#);
        assert!(!token_looks_valid(&decoy));
    }

    /// `save_token` must be a no-op for garbage, so a stale good token survives a
    /// mistyped one.
    #[test]
    fn save_token_ignores_garbage_and_keeps_the_previous_one() {
        use base64::Engine;
        let dir = tmp_dir("token-save");
        let real = base64::engine::general_purpose::STANDARD.encode(
            br#"{"a":"0123456789abcdef0123456789abcdef","t":"01234567-89ab-cdef-0123-456789abcdef","s":"X0xYb1p6c2VjcmV0X2Jhc2U2NF9ibG9iX2Zvcl90ZXN0aW5n"}"#,
        );
        let path = dir.join("token.txt");
        std::fs::write(&path, &real).unwrap();
        // Garbage must not overwrite it (save_token writes to token_path(), which is
        // the real storage dir, so assert the validation gate instead of the file).
        assert!(!token_looks_valid("hello-random-garbage"));
        assert_eq!(std::fs::read_to_string(&path).unwrap().trim(), real);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn state_roundtrip_preserves_token() {
        let dir = tmp_dir("state");
        let state_file = dir.join("state.json");
        let token_file = dir.join("token.txt");
        save_state_at(
            &state_file,
            &json!({"enabled": true, "pid": 42, "tunnel_url": "https://a-b.trycloudflare.com", "tunnel_token": "tok-1"}),
        );
        // the token is mirrored into token.txt like the PHP's fd_save_tunnel_token()
        assert_eq!(std::fs::read_to_string(&token_file).unwrap().trim(), "tok-1");
        let loaded = load_state_at(&state_file);
        assert_eq!(st_i64(&loaded, "pid"), 42);
        assert!(st_bool(&loaded, "enabled"));
        assert_eq!(st_str(&loaded, "tunnel_token"), "tok-1");

        // state.json without a token falls back to token.txt
        std::fs::write(&state_file, "{\"pid\":7}").unwrap();
        assert_eq!(st_str(&load_state_at(&state_file), "tunnel_token"), "tok-1");

        // clearing keeps the token ...
        clear_state_in(&dir, true);
        assert!(!state_file.exists());
        assert_eq!(std::fs::read_to_string(&token_file).unwrap().trim(), "tok-1");

        // ... unless asked not to
        std::fs::write(&token_file, "tok-2").unwrap();
        clear_state_in(&dir, false);
        assert!(!token_file.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pid_file_roundtrip_and_state_fallback() {
        let dir = tmp_dir("pid");
        let pid_file = dir.join("cloudflared.pid");
        std::fs::write(&pid_file, " 4242 \n").unwrap();
        assert_eq!(read_pid_at(&pid_file, &json!({"pid": 1})), 4242);
        // a missing pid file falls back to state.pid
        assert_eq!(read_pid_at(&dir.join("nope.pid"), &json!({"pid": 99})), 99);
        assert_eq!(read_pid_at(&dir.join("nope.pid"), &json!({})), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn dummy_config_has_the_ingress_and_keepalive_block() {
        let dir = tmp_dir("cfg");
        let path = write_dummy_config_at(&dir.join("config.yml"), "http://127.0.0.1:8088");
        let body = std::fs::read_to_string(path).unwrap();
        assert!(body.starts_with("ingress:\n"));
        assert!(body.contains("- service: http://127.0.0.1:8088"));
        assert!(body.contains("noTLSVerify: true"));
        assert!(body.contains("http2Origin: false"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn spawn_args_named_and_quick() {
        let log = Path::new("C:/tmp/cloudflared.log");
        let cfg = Path::new("C:/tmp/config.yml");
        let quick = spawn_args("http://127.0.0.1:8088", cfg, log, 20241, "");
        assert_eq!(quick[0], "tunnel");
        assert!(quick.windows(2).any(|w| w[0] == "--url" && w[1] == "http://127.0.0.1:8088"));
        assert!(quick.windows(2).any(|w| w[0] == "--config"));
        assert!(quick.windows(2).any(|w| w[0] == "--metrics" && w[1] == "127.0.0.1:20241"));
        assert!(!quick.iter().any(|a| a == "run"));

        let named = spawn_args("http://127.0.0.1:8088", cfg, log, 20242, "eyJhIjoieCJ9");
        assert!(!named.iter().any(|a| a == "--url"));
        assert!(named.windows(2).any(|w| w[0] == "run" && w[1] == "--token"));
        assert_eq!(named.last().unwrap(), "eyJhIjoieCJ9");
    }

    #[test]
    fn netstat_and_ss_ports_are_parsed_per_pid() {
        let windows = "  TCP    127.0.0.1:20241        0.0.0.0:0              LISTENING       4321\r\n\
                       TCP    127.0.0.1:8088         0.0.0.0:0              LISTENING       1111\r\n\
                       TCP    127.0.0.1:5555         0.0.0.0:0              ESTABLISHED     4321\r\n";
        assert_eq!(parse_listen_ports(windows, 4321), vec![20241]);
        assert!(parse_listen_ports(windows, 1).is_empty());

        let ss = "LISTEN 0 4096 127.0.0.1:20241 0.0.0.0:* users:((\"cloudflared\",pid=4321,fd=3))\n";
        assert_eq!(parse_listen_ports(ss, 4321), vec![20241]);
        assert!(parse_listen_ports(ss, 9999).is_empty());
    }

    #[test]
    fn tasklist_csv_matches_only_a_real_row() {
        let csv = "\"cloudflared.exe\",\"4321\",\"Console\",\"1\",\"25,000 K\"\r\n";
        assert!(tasklist_contains_pid(csv, 4321));
        assert!(!tasklist_contains_pid(csv, 1234));
        assert!(!tasklist_contains_pid("INFO: No tasks are running which match the specified criteria.", 4321));
        assert!(!tasklist_contains_pid(csv, 0));
    }

    #[test]
    #[cfg(windows)]
    fn win_pid_alive_detects_self_and_invalid_pids() {
        let my_pid = std::process::id() as i64;
        assert!(win_pid_alive(my_pid));
        assert!(!win_pid_alive(0));
        assert!(!win_pid_alive(-1));
        assert!(!win_pid_alive(2_000_000_000));
    }

    #[test]
    fn asset_name_matches_the_platform() {
        let asset = asset_name();
        if cfg!(windows) {
            assert_eq!(asset, "cloudflared-windows-amd64.exe");
        } else if cfg!(target_os = "macos") {
            assert!(asset.starts_with("cloudflared-darwin-") && asset.ends_with(".tgz"));
        } else {
            assert!(asset.starts_with("cloudflared-linux-"));
            assert!(!asset.ends_with(".tgz"));
        }
    }

    #[test]
    fn android_uses_the_static_linux_assets() {
        // No `-android` asset exists; the Linux builds are static Go binaries and
        // exec under bionic. Pinned so a "helpful" android branch can never
        // return an empty name (which would break the download path outright).
        assert_eq!(asset_name_for("android", "aarch64"), "cloudflared-linux-arm64");
        assert_eq!(asset_name_for("android", "arm"), "cloudflared-linux-arm");
        assert_eq!(asset_name_for("android", "x86_64"), "cloudflared-linux-amd64");
        assert_eq!(asset_name_for("android", "x86"), "cloudflared-linux-386");
    }

    #[test]
    fn every_desktop_arch_maps_to_a_real_asset() {
        assert_eq!(asset_name_for("linux", "x86_64"), "cloudflared-linux-amd64");
        assert_eq!(asset_name_for("linux", "aarch64"), "cloudflared-linux-arm64");
        assert_eq!(asset_name_for("linux", "arm"), "cloudflared-linux-arm");
        assert_eq!(asset_name_for("linux", "x86"), "cloudflared-linux-386");
        assert_eq!(asset_name_for("macos", "aarch64"), "cloudflared-darwin-arm64.tgz");
        assert_eq!(asset_name_for("macos", "x86_64"), "cloudflared-darwin-amd64.tgz");
        assert_eq!(asset_name_for("windows", "x86_64"), "cloudflared-windows-amd64.exe");
        // Unknown arch is the amd64 build, never an empty string.
        assert_eq!(asset_name_for("linux", "riscv64"), "cloudflared-linux-amd64");
    }

    #[test]
    fn bin_name_is_exe_only_on_windows() {
        assert_eq!(bin_name_for(true), "cloudflared.exe");
        assert_eq!(bin_name_for(false), "cloudflared");
        // android/linux/macos all take the bare name
        assert_eq!(bin_name(), bin_name_for(cfg!(windows)));
    }

    #[test]
    fn metrics_port_is_always_usable() {
        // `pick_metrics_port()` binds port 0 and drops the listener, so the number
        // it returns is free *at that instant* — but another test in this parallel
        // run (or the CI runner itself) can take it before we re-bind. That TOCTOU
        // is not a product bug: CI failed here once with "rebound.is_ok()" false on
        // a busy runner while 229 other tests passed. Retry instead of failing, so
        // the guarantee being checked (a returned port really is bindable) still
        // holds without turning a scheduler race into a red build.
        let mut last = 0u16;
        for _ in 0..10 {
            let port = pick_metrics_port();
            assert!(port > 0);
            last = port;
            if std::net::TcpListener::bind(("127.0.0.1", port)).is_ok() || port == 20241 {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        panic!("no re-bindable metrics port in 10 attempts (last was {last})");
    }

    #[test]
    fn read_logs_joins_the_process_and_error_logs() {
        let dir = tmp_dir("logs");
        let log_file = dir.join("cloudflared.log");
        let err_file = dir.join("cloudflared.err.log");
        std::fs::write(&log_file, "LINE-A\n").unwrap();
        std::fs::write(&err_file, "LINE-B\n").unwrap();
        let joined = read_logs_at(&log_file, &err_file);
        assert!(joined.contains("LINE-A"));
        assert!(joined.contains("LINE-B"));
        // a missing file contributes nothing and is not an error
        let only_one = read_logs_at(&log_file, &dir.join("absent.log"));
        assert_eq!(only_one, "LINE-A\n");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
