//! Port of the `diagnostics` group, backend.php:11293-11654:
//! `fd_diagnostics_settings_path/settings`, `fd_diagnostics_auto_report_enabled`,
//! `fd_diagnostics_set_auto_report`, `fd_reports_dir`, `fd_device_info`,
//! `fd_diag_prune`, `fd_diag_record`, `fd_diag_list`, `fd_diag_counts`,
//! `fd_diag_clear`, plus the uploader (`fd_diag_report_send` / `fd_diag_flush_pending`).
//!
//! Same storage paths and file naming (`<type>_<unix_ms>.json`, 10 per type),
//! same category names, and the SAME `list()` shape: `fd_diag_list()` returns a
//! SUMMARY row carrying `file` + `sent`, and the dashboard's "Send Reports"
//! button reads both (`pending = reports.filter(r => !r.sent).map(r => r.file)`).
//! Returning the raw report files instead left `file` undefined, so the button
//! could never enqueue a send and always claimed "already sent".
//!
//! Auto-upload on record is still NOT wired (the PHP spawns a detached worker
//! from `fd_diag_record()`); `?send=` / `?flush=1` and `report_send()` are.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use rand::RngCore;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::config::app_version;
use crate::storage::{now, storage_path};

pub const REPORT_TYPES: [&str; 5] = [
    "crash",
    "anr",
    "stream_incident",
    "playback_incident",
    "self_test",
];

pub fn settings_path() -> PathBuf {
    storage_path("diagnostics.json")
}

pub fn settings() -> Value {
    fs::read_to_string(settings_path())
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .filter(|v| v.is_object())
        .unwrap_or_else(|| serde_json::json!({}))
}

pub fn auto_report_enabled() -> bool {
    match settings().get("auto_report") {
        Some(v) => v.as_bool().unwrap_or(true),
        None => true,
    }
}

pub fn set_auto_report(enabled: bool) -> Value {
    let mut s = settings();
    if !s.is_object() {
        s = serde_json::json!({});
    }
    s["auto_report"] = serde_json::json!(enabled);
    let _ = fs::write(settings_path(), s.to_string());
    s
}

pub fn reports_dir() -> PathBuf {
    let dir = storage_path("reports");
    let _ = fs::create_dir_all(&dir);
    dir
}

pub fn device_id_path() -> PathBuf {
    storage_path("device_id.txt")
}

/// The identifier this install presents to the origin (`X-PM-Device`, the
/// `/api/session` `device_id`, and the binding inside a provision ticket).
///
/// Preference order:
///  1. `PM_DEVICE_ID` — a platform id only the HOST app can read. On Android
///     that is `Settings.Secure.ANDROID_ID`, which needs a `JNIEnv` + `Context`
///     that a subprocess (which is what this engine is, under Termux/proot or
///     spawned by the APK shell) simply does not have. The shell injects it
///     instead of us pretending to have JNI;
///  2. an id already in `storage/device_id.txt` — existing installs keep the
///     value the origin knows, so no ticket or session is invalidated;
///  3. the OS machine id, **hashed** — DESKTOP ONLY (Windows/Linux/macOS).
///     Survives a reinstall and a wiped `storage/`, so the origin recognises the
///     same machine. Nothing is written to disk: it is derived, so there is no
///     file to keep in sync;
///  4. random, persisted — Android (see the note in `read_machine_uid`) and any
///     container/ROM that exposes no trustworthy machine id.
///
/// Android is deliberately excluded from (3): `/etc/machine-id` there is a
/// system-wide id shared by every app, not the per-app identity Android exposes,
/// and it is missing on some ROMs. The shell injects `ANDROID_ID` as
/// `PM_DEVICE_ID` instead (see `RustServer.kt`); anything else lands on (4).
pub fn get_device_id() -> String {
    if let Some(injected) = crate::storage::env("PM_DEVICE_ID") {
        let injected = injected.trim();
        if !injected.is_empty() {
            // Hashed like the machine id: 12 hex chars on the wire either way.
            return derive_device_id_from_machine(injected);
        }
    }

    if let Ok(v) = fs::read_to_string(device_id_path()) {
        let v = v.trim().to_string();
        if !v.is_empty() {
            return v;
        }
    }

    let uid = machine_uid();
    if !uid.trim().is_empty() {
        return derive_device_id_from_machine(&uid);
    }

    let mut buf = [0u8; 6];
    rand::rng().fill_bytes(&mut buf);
    let id = hex::encode(buf);
    let _ = fs::write(device_id_path(), &id);
    id
}

/// The OS machine id, read directly (cached once per process).
///
/// Deliberately dependency-free: `machineid-rs` only implements `get_hwid()`
/// for linux/windows/macos, so it cannot build for Android — the platform where
/// a stable id matters most (Termux/proot installs get wiped often).
fn machine_uid() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE.get_or_init(read_machine_uid).clone()
}

fn read_machine_uid() -> String {
    #[cfg(target_os = "windows")]
    {
        // Written by Windows Setup; survives reinstalls of this app.
        if let Ok(out) = std::process::Command::new("reg")
            .args([
                "query",
                r"HKLM\SOFTWARE\Microsoft\Cryptography",
                "/v",
                "MachineGuid",
            ])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.contains("MachineGuid") {
                    if let Some(v) = line.split_whitespace().last() {
                        let v = v.trim();
                        if !v.is_empty() {
                            return v.to_string();
                        }
                    }
                }
            }
        }
        String::new()
    }

    #[cfg(target_os = "linux")]
    {
        for path in ["/etc/machine-id", "/var/lib/dbus/machine-id"] {
            if let Ok(v) = fs::read_to_string(path) {
                let v = v.trim().to_string();
                if !v.is_empty() {
                    return v;
                }
            }
        }
        String::new()
    }

    #[cfg(target_os = "android")]
    {
        // No `/etc/machine-id` on Android. It is a SYSTEM-WIDE identifier shared
        // by every app on the device (not the per-app id Android intends apps to
        // use), and it is absent or unreadable on some ROMs. The platform id is
        // `Settings.Secure.ANDROID_ID`, which only the shell can read — the engine
        // is a child process started by `RustServer.kt`, so it has no JNIEnv — and
        // the shell passes it in as PM_DEVICE_ID. Everything else falls back to
        // the per-install id in `storage/device_id.txt`.
        String::new()
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(out) = std::process::Command::new("ioreg")
            .args(["-rd1", "-c", "IOPlatformExpertDevice"])
            .output()
        {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if line.contains("IOPlatformUUID") {
                    if let Some(v) = line.split('=').nth(1) {
                        let v = v.trim().trim_matches('"');
                        if !v.is_empty() {
                            return v.to_string();
                        }
                    }
                }
            }
        }
        String::new()
    }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "linux",
        target_os = "android",
        target_os = "macos"
    )))]
    {
        String::new()
    }
}

/// `sha256("pencarimovie-device|" + machine_uid)` truncated to 12 hex chars.
///
/// Stable for a given machine, and the raw id cannot be read back out of it.
fn derive_device_id_from_machine(uid: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"pencarimovie-device|");
    hasher.update(uid.trim().as_bytes());
    hex::encode(hasher.finalize())[..12].to_string()
}

#[cfg(test)]
mod device_id_tests {
    use super::derive_device_id_from_machine;

    /// Android must never fall back to `/etc/machine-id`: it is a system-wide id
    /// shared by every app, not the per-app identity, and the shell is what
    /// supplies `ANDROID_ID` via `PM_DEVICE_ID`.
    #[cfg(target_os = "android")]
    #[test]
    fn android_never_reads_a_system_machine_id() {
        assert_eq!(super::machine_uid(), "");
    }

    #[test]
    fn machine_derived_id_is_stable_and_twelve_hex_chars() {
        let a = derive_device_id_from_machine("0f3c1c2a-9d1b-4b7e-8f00-1a2b3c4d5e6f");
        let b = derive_device_id_from_machine("0f3c1c2a-9d1b-4b7e-8f00-1a2b3c4d5e6f");
        assert_eq!(a, b);
        assert_eq!(a.len(), 12);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));

        // Whitespace from the OS file must not change the id.
        let c = derive_device_id_from_machine("  0f3c1c2a-9d1b-4b7e-8f00-1a2b3c4d5e6f\n");
        assert_eq!(a, c);
    }

    #[test]
    fn different_machines_get_different_ids() {
        let a = derive_device_id_from_machine("machine-one");
        let b = derive_device_id_from_machine("machine-two");
        assert_ne!(a, b);
    }

    #[test]
    fn raw_machine_id_is_not_recoverable_from_the_output() {
        let uid = "0f3c1c2a-9d1b-4b7e-8f00-1a2b3c4d5e6f";
        let id = derive_device_id_from_machine(uid);
        assert!(!id.contains("0f3c1c2a"));
        assert!(!uid.contains(&id));
    }
}

fn read_first_line(path: &str) -> String {
    // `read_to_string` fails outright on a file with one invalid byte, and some
    // `/proc` entries (Android's `/proc/version`) are not guaranteed UTF-8.
    fs::read(path)
        .ok()
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .and_then(|t| t.lines().next().map(|l| l.trim().to_string()))
        .unwrap_or_default()
}

/// `ro.product.model=…` out of an Android `build.prop` (pure, testable).
///
/// Only the first `=` splits, values may contain spaces, and comments/blank
/// lines are skipped — the real file has all three.
pub fn build_prop_lookup(text: &str, key: &str) -> String {
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return v.trim().to_string();
            }
        }
    }
    String::new()
}

/// Android's `/proc` is heavily restricted and `uname` fields are useless for a
/// support report, so read the same `build.prop` the PHP/Kotlin apps read.
/// Every step degrades to an empty string: this must never fail a request.
fn android_build_prop() -> String {
    for path in ["/system/build.prop", "/vendor/build.prop", "/product/build.prop"] {
        if let Ok(bytes) = fs::read(path) {
            return String::from_utf8_lossy(&bytes).to_string();
        }
    }
    String::new()
}

/// `getprop <key>` — the ONLY reliable source on Android 10+.
///
/// Modern Android removed `/system/build.prop` (it is compiled into the
/// build-system image and the app UID cannot read it), so the build.prop probe
/// above silently yields nothing and every report from a real phone carried an
/// EMPTY `model` and an `os_version` of just `"android"`. Measured on a
/// CPH2581 / Android 16: `model:""`, `hostname:""`, `os_version:"android"`.
fn getprop(key: &str) -> String {
    std::process::Command::new("getprop")
        .arg(key)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

fn android_model_and_version() -> (String, String, String) {
    static CACHE: OnceLock<(String, String, String)> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let prop = android_build_prop();
            let mut model = if prop.is_empty() {
                String::new()
            } else {
                build_prop_lookup(&prop, "ro.product.model")
            };
            let mut release = if prop.is_empty() {
                String::new()
            } else {
                build_prop_lookup(&prop, "ro.build.version.release")
            };
            let mut sdk = if prop.is_empty() {
                String::new()
            } else {
                build_prop_lookup(&prop, "ro.build.version.sdk")
            };
            let mut brand = if prop.is_empty() {
                String::new()
            } else {
                build_prop_lookup(&prop, "ro.product.brand")
            };
            // `/system/build.prop` is unreadable on Android 10+; `getprop` is not.
            if model.is_empty() {
                model = getprop("ro.product.model");
            }
            if brand.is_empty() {
                brand = getprop("ro.product.brand");
            }
            if release.is_empty() {
                release = getprop("ro.build.version.release");
            }
            if sdk.is_empty() {
                sdk = getprop("ro.build.version.sdk");
            }
            let version = if release.is_empty() {
                String::new()
            } else if sdk.is_empty() {
                format!("Android {}", release)
            } else {
                format!("Android {} (API {})", release, sdk)
            };
            (model, version, brand)
        })
        .clone()
}

/// Free space on the volume holding the app root, in MiB.
///
/// No `sysinfo`/`fs2` dependency: Windows asks kernel32 directly, Unix shells
/// out to `df -kP`. Both are cached by the caller, so `/api/diagnostics` (which
/// the Android service polls every 3 s) never spawns a process repeatedly.
#[cfg(windows)]
fn disk_free_mb_uncached() -> Option<u64> {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetDiskFreeSpaceExW(
            dir: *const u16,
            free_bytes_available: *mut u64,
            total_bytes: *mut u64,
            total_free_bytes: *mut u64,
        ) -> i32;
    }
    let mut wide: Vec<u16> = crate::storage::app_root().as_os_str().encode_wide().collect();
    wide.push(0);
    let (mut avail, mut total, mut free) = (0u64, 0u64, 0u64);
    let ok = unsafe { GetDiskFreeSpaceExW(wide.as_ptr(), &mut avail, &mut total, &mut free) };
    if ok == 0 { None } else { Some(avail / (1024 * 1024)) }
}

#[cfg(not(windows))]
fn disk_free_mb_uncached() -> Option<u64> {
    let out = std::process::Command::new("df")
        .arg("-kP")
        .arg(crate::storage::app_root())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let kb: u64 = text.lines().nth(1)?.split_whitespace().nth(3)?.parse().ok()?;
    Some(kb / 1024)
}

fn disk_free_mb() -> Option<u64> {
    static CACHE: OnceLock<Mutex<(i64, Option<u64>)>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new((0, None)));
    let now_s = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let mut guard = match cache.lock() {
        Ok(g) => g,
        Err(_) => return disk_free_mb_uncached(),
    };
    if now_s - guard.0 > 60 {
        *guard = (now_s, disk_free_mb_uncached());
    }
    guard.1
}

/// Human OS string for the support report. Windows used to read
/// `/proc/sys/kernel/osrelease`, which does not exist there, so the dashboard
/// showed `OS: ?`.
fn os_version_string() -> String {
    static CACHE: OnceLock<String> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            if cfg!(target_os = "android") {
                let (_, v, _) = android_model_and_version();
                if !v.is_empty() {
                    return v;
                }
            }
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                let mut cmd = std::process::Command::new("cmd");
                cmd.args(["/c", "ver"]).creation_flags(0x0800_0000);
                if let Ok(o) = cmd.output() {
                    let t = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    if !t.is_empty() {
                        return t;
                    }
                }
                return "Windows".to_string();
            }
            #[cfg(not(windows))]
            {
                let release = read_first_line("/proc/sys/kernel/osrelease");
                if release.is_empty() {
                    std::env::consts::OS.to_string()
                } else {
                    format!("{} {}", std::env::consts::OS, release)
                }
            }
        })
        .clone()
}

/* ── live process metrics ───────────────────────────────────────────────── */

/// Stamped from `main` before the server accepts, so this is the PROCESS uptime,
/// not `/proc/uptime`: a NAS that has been on for months answers the host value
/// even when this server restarted a minute ago.
static PROC_START: OnceLock<std::time::Instant> = OnceLock::new();

pub fn record_process_start() {
    let _ = PROC_START.set(std::time::Instant::now());
}

pub fn process_uptime_s() -> u64 {
    PROC_START.get().map(|s| s.elapsed().as_secs()).unwrap_or(0)
}

/// Percent of ONE core, over the interval between two calls. The first call has
/// no previous sample and answers `None`; the tile shows `—` until the next poll.
pub fn cpu_percent_now() -> Option<f64> {
    static LAST: OnceLock<Mutex<Option<(u64, std::time::Instant)>>> = OnceLock::new();
    let now = std::time::Instant::now();
    let total = cpu_total_ns()?;
    let cell = LAST.get_or_init(|| Mutex::new(None));
    let mut prev = match cell.lock() {
        Ok(g) => g,
        Err(p) => p.into_inner(),
    };
    let percent = match *prev {
        Some((prev_total, prev_at)) => percent_between(prev_total, prev_at, total, now),
        None => None,
    };
    *prev = Some((total, now));
    percent
}

fn percent_between(
    prev_cpu_ns: u64,
    prev_at: std::time::Instant,
    cpu_ns: u64,
    now: std::time::Instant,
) -> Option<f64> {
    let wall = now.checked_duration_since(prev_at)?.as_nanos();
    if wall == 0 {
        return None;
    }
    let cpu = cpu_ns.saturating_sub(prev_cpu_ns);
    Some(cpu as f64 / wall as f64 * 100.0)
}

#[cfg(windows)]
#[repr(C)]
struct FileTime {
    low: u32,
    high: u32,
}

#[cfg(windows)]
fn filetime_100ns(t: FileTime) -> u64 {
    ((t.high as u64) << 32) | t.low as u64
}

/// Cumulative CPU time of this process, in nanoseconds.
#[cfg(windows)]
fn cpu_total_ns() -> Option<u64> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn GetProcessTimes(
            process: isize,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> i32;
    }
    let (mut creation, mut exit, mut kernel, mut user) = (
        FileTime { low: 0, high: 0 },
        FileTime { low: 0, high: 0 },
        FileTime { low: 0, high: 0 },
        FileTime { low: 0, high: 0 },
    );
    let ok = unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    };
    if ok == 0 {
        return None;
    }
    Some((filetime_100ns(kernel) + filetime_100ns(user)) * 100)
}

/// `/proc/self/stat` is readable on Linux, Android and Termux; macOS without
/// `/proc` has no counter here, so the tile shows `—` rather than a made-up 0.
/// `USER_HZ` is 100 on Linux and Android, hence the 10 ms tick.
#[cfg(not(windows))]
fn cpu_total_ns() -> Option<u64> {
    let stat = fs::read_to_string("/proc/self/stat").ok()?;
    let rest = &stat[stat.rfind(')')? + 1..];
    let mut fields = rest.split_whitespace();
    // Field 14 (`utime`) is the 12th value after the closing `)`: fields 3-13.
    let utime: u64 = fields.nth(11)?.parse().ok()?;
    let stime: u64 = fields.next()?.parse().ok()?;
    Some((utime + stime) * 10_000_000)
}

#[cfg(windows)]
fn process_rss_kb() -> u64 {
    #[repr(C)]
    struct Counters {
        cb: u32,
        page_faults: u32,
        peak_working_set: usize,
        working_set: usize,
        quota_peak_paged: usize,
        quota_paged: usize,
        quota_peak_nonpaged: usize,
        quota_nonpaged: usize,
        pagefile: usize,
        peak_pagefile: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> isize;
        fn K32GetProcessMemoryInfo(process: isize, counters: *mut Counters, cb: u32) -> i32;
    }
    let mut counters = Counters {
        cb: std::mem::size_of::<Counters>() as u32,
        page_faults: 0,
        peak_working_set: 0,
        working_set: 0,
        quota_peak_paged: 0,
        quota_paged: 0,
        quota_peak_nonpaged: 0,
        quota_nonpaged: 0,
        pagefile: 0,
        peak_pagefile: 0,
    };
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
    if ok == 0 {
        0
    } else {
        counters.working_set as u64 / 1024
    }
}

#[cfg(not(windows))]
fn process_rss_kb() -> u64 {
    proc_value("/proc/self/status", "VmRSS")
}

fn proc_value(path: &str, key: &str) -> u64 {
    let text = fs::read_to_string(path).unwrap_or_default();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix(&format!("{}:", key)) {
            let num: String = rest.chars().filter(|c| c.is_ascii_digit()).collect();
            return num.parse().unwrap_or(0);
        }
    }
    0
}

/// The memory ceiling this PROCESS actually has, in kB, and whether it came from a
/// cgroup (i.e. we are inside a container).
///
/// `/proc/meminfo` reports the HOST's RAM: on a Heroku Standard-2X dyno it says
/// **62 GB** while the process is killed at 1024 MB, so "617 MB / 62 GB" is a number
/// nobody can act on and every user reads as "plenty of room". cgroup v2
/// (`memory.max`) and v1 (`memory/memory.limit_in_bytes`) carry the real ceiling, so
/// that wins; the host total is only the fallback (bare metal, Termux, Android).
fn memory_limit_kb() -> (u64, bool) {
    for path in [
        "/sys/fs/cgroup/memory.max",
        "/sys/fs/cgroup/memory/memory.limit_in_bytes",
    ] {
        let raw = read_first_line(path);
        let t = raw.trim();
        // cgroup v2 writes the literal `max` for "no limit"; v1 writes a sentinel
        // near 2^63. Neither is a ceiling a user should see.
        if t.is_empty() || t.eq_ignore_ascii_case("max") {
            continue;
        }
        if let Ok(v) = t.parse::<u64>() {
            if v > 0 && v < (1u64 << 50) {
                return (v / 1024, true);
            }
        }
    }
    (0, false)
}

/// `fd_device_info()` (backend.php:11339) with `/proc` metrics.
///
/// **Android**: every `/proc` read here degrades to `0`/`""` because a
/// non-rooted app sees a stripped `/proc` (`/proc/sys/kernel/hostname` is not
/// readable, `/proc/self/status` and `/proc/self/fd` are). That is deliberate —
/// a diagnostics report must never fail because a counter is missing. Model /
/// OS / brand come from `build.prop` when it is readable (Android ≤9) and from
/// `getprop` otherwise (Android 10+ removed `/system/build.prop`).
pub fn device_info() -> Value {
    let mem_total_kb = proc_value("/proc/meminfo", "MemTotal");
    let mem_avail_kb = proc_value("/proc/meminfo", "MemAvailable");
    // What the user can act on: the container's ceiling when there is one.
    let (limit_kb, container) = memory_limit_kb();
    let effective_kb = if limit_kb > 0 { limit_kb } else { mem_total_kb };
    let rss_kb = process_rss_kb();
    let uptime_s = {
        let raw = read_first_line("/proc/uptime");
        raw.split('.').next().unwrap_or("0").parse::<u64>().unwrap_or(0)
    };
    let fd_count = fs::read_dir("/proc/self/fd").map(|d| d.count()).unwrap_or(0);
    let hostname = {
        let h = read_first_line("/proc/sys/kernel/hostname");
        if h.is_empty() {
            std::env::var("COMPUTERNAME")
                .or_else(|_| std::env::var("HOSTNAME"))
                .unwrap_or_default()
        } else {
            h
        }
    };
    let (model, brand) = if cfg!(target_os = "android") {
        let (m, _, b) = android_model_and_version();
        (m, if b.is_empty() { std::env::consts::ARCH.to_string() } else { b })
    } else if hostname.is_empty() {
        // A desktop has no "model"; the hostname is the useful identifier and
        // beats the literal "?" the panel used to print.
        (std::env::consts::ARCH.to_string(), std::env::consts::ARCH.to_string())
    } else {
        (hostname.clone(), std::env::consts::ARCH.to_string())
    };
    serde_json::json!({
        "device_id": get_device_id(),
        "platform": std::env::consts::OS,
        "model": model,
        "brand": brand,
        "os_version": os_version_string(),
        // `2.9.2-10-g5c61011-dirty` is support info, not a version a user should
        // read in the diagnostics panel.
        "app_version": crate::config::clean_version(&app_version()),
        "auto_report": auto_report_enabled(),
        "disk_free_mb": disk_free_mb(),
        "hostname": hostname,
        // The ceiling that matters: cgroup limit inside a container, host RAM bare metal.
        "total_memory_mb": effective_kb / 1024,
        // Kept for anyone comparing against the host; never shown as "available to us".
        "host_total_memory_mb": mem_total_kb / 1024,
        "container": container,
        "free_memory_mb": mem_avail_kb / 1024,
        "process_rss_mb": rss_kb / 1024,
        "open_fds": fd_count,
        // `uptime_s` stays the HOST value for every existing reader; the
        // process's own uptime is the new key the overview tile reads.
        "uptime_s": uptime_s,
        "host_uptime_s": uptime_s,
        "proc_uptime_s": process_uptime_s(),
        "runtime": "pencarimovie-server",
    })
}

/// `fd_diag_prune()`: keep at most 10 files per type.
pub fn prune() {
    let dir = reports_dir();
    let entries = match fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    let mut by_type: std::collections::HashMap<String, Vec<(i64, PathBuf)>> = std::collections::HashMap::new();
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().to_string();
        if !name.ends_with(".json") {
            continue;
        }
        let base = name.trim_end_matches(".json").to_string();
        let ty = match base.rsplit_once('_') {
            Some((t, tail)) if tail.chars().all(|c| c.is_ascii_digit()) => t.to_string(),
            _ => continue,
        };
        let m = crate::storage::mtime_secs(&e.path()).unwrap_or(0);
        by_type.entry(ty).or_default().push((m, e.path()));
    }
    for (_, mut files) in by_type {
        files.sort_by(|a, b| b.0.cmp(&a.0));
        for (_, path) in files.into_iter().skip(10) {
            let _ = fs::remove_file(&path);
            // `fd_diag_prune()` unlinks `$old . '.sent'` too: leaving the
            // sidecar behind means a later same-name report looks pre-sent.
            let _ = fs::remove_file(format!("{}.sent", path.display()));
        }
    }
}

/// `fd_diag_record()`: write one incident locally.
pub fn record(
    report_type: &str,
    title: &str,
    message: &str,
    stack: &str,
    reason: &str,
    short_code: &str,
    item_id: &str,
    extra: Value,
) -> PathBuf {
    prune();
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let file = reports_dir().join(format!("{}_{}.json", report_type, ms));
    let mut payload = serde_json::json!({
        "report_type": report_type,
        // MUST match `device_info()["platform"]`, not a literal "server".
        // Reports were filed as `platform:"server"` while `/api/diagnostics`
        // reported `platform:"android"`, so filtering the columnar table by
        // platform could never find a device's own incidents — and a
        // `platform=php|android` filter (the documented vocabulary) matched
        // neither. `runtime` keeps the engine identity that "server" was
        // standing in for.
        "platform": std::env::consts::OS,
        "runtime": "pencarimovie-server",
        "device_id": get_device_id(),
        // `2.9.2-10-g5c61011-dirty` is support info; `/api/diagnostics` shows the
        // cleaned version, so a report must not disagree with the panel.
        "app_version": crate::config::clean_version(&app_version()),
        "created_at": now(),
        "error_title": title,
        "error_message": message,
        "stack_trace": stack,
        "failure_reason": reason,
        "short_code": short_code,
        "item_id": item_id,
    });
    if let (Some(dst), Some(src)) = (payload.as_object_mut(), extra.as_object()) {
        for (k, v) in src {
            dst.insert(k.clone(), v.clone());
        }
    }
    let _ = fs::write(&file, serde_json::to_string_pretty(&payload).unwrap_or_default());
    file
}

/// Debounce window for [`record_throttled`].
pub const THROTTLE_WINDOW_S: i64 = 300;

static THROTTLE: OnceLock<Mutex<HashMap<String, i64>>> = OnceLock::new();

/// `record()` at most once per [`THROTTLE_WINDOW_S`] for an identical
/// `(report_type, reason, short_code, http_status)` signature.
///
/// A URL that cannot play is re-requested on EVERY range and every player
/// reopen — the 52 `CLIENT_ABORTED` rows in `server_reports_idx` are that shape
/// — and [`prune`] keeps only 10 files per type, so an unthrottled failure
/// record buries every real incident instead of adding signal. Keyed on the
/// signature (never on the file), so two different titles failing the same way
/// both land.
///
/// ponytail: signatures with an empty `short_code` collapse into one report.
/// Add a per-`item_id` key when a refusal without a short code becomes common.
pub fn record_throttled(
    report_type: &str,
    title: &str,
    message: &str,
    reason: &str,
    short_code: &str,
    extra: Value,
) -> Option<PathBuf> {
    let status = extra.get("http_status").and_then(|v| v.as_u64()).unwrap_or(0);
    let key = format!("{report_type}|{reason}|{short_code}|{status}");
    let mut seen = THROTTLE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if !take_slot(&mut seen, &key, now(), THROTTLE_WINDOW_S) {
        return None;
    }
    drop(seen);
    Some(record(report_type, title, message, "", reason, short_code, "", extra))
}

/// One slot per `key` per `window_s`. Expired signatures are forgotten on every
/// call, so the map is bounded by "signatures seen in the last window" instead
/// of growing with every short_code ever requested.
fn take_slot(seen: &mut HashMap<String, i64>, key: &str, t: i64, window_s: i64) -> bool {
    seen.retain(|_, v| t - *v < window_s);
    if seen.contains_key(key) {
        return false;
    }
    seen.insert(key.to_string(), t);
    true
}

/// One `fd_diag_list()` row. Pure so the shape is pinned by a test — `file` and
/// `sent` are the two fields the dashboard's send loop depends on.
fn summary_row(name: &str, v: &Value, sent: bool) -> Value {
    serde_json::json!({
        "file": name,
        "report_type": v.get("report_type").and_then(|x| x.as_str()).unwrap_or(""),
        "created_at": v.get("created_at").and_then(|x| x.as_i64()).unwrap_or(0),
        "error_title": v.get("error_title").and_then(|x| x.as_str()).unwrap_or(""),
        "failure_reason": v.get("failure_reason").and_then(|x| x.as_str()).unwrap_or(""),
        "item_id": v.get("item_id").and_then(|x| x.as_str()).unwrap_or(""),
        "sent": sent,
    })
}

/// `fd_diag_list()`: SUMMARY rows, not the raw report files.
///
/// The shape is load-bearing, not cosmetic: the dashboard's "Send Reports"
/// button does
/// `pending = reports.filter(r => !r.sent).map(r => r.file).filter(Boolean)`
/// ([`public/app.js`](../public/app.js)). Returning the raw payload (which has
/// neither `file` nor `sent`) made `file` undefined, so `pending` was ALWAYS
/// empty and the button answered "Nothing to send — all N report(s) were
/// already sent" no matter how many unsent reports were on disk. Verified on a
/// CPH2581: `/api/diagnostics` listed 10 reports, none with `file`/`sent`.
///
/// The full report is still available from `/api/reports?file=<name>`.
pub fn list() -> Vec<Value> {
    let dir = reports_dir();
    let mut out: Vec<(i64, Value)> = Vec::new();
    if let Ok(entries) = fs::read_dir(&dir) {
        for e in entries.flatten() {
            let path = e.path();
            if path.extension().map(|x| x == "json").unwrap_or(false) == false {
                continue;
            }
            let Ok(text) = fs::read_to_string(&path) else { continue };
            let Ok(v) = serde_json::from_str::<Value>(&text) else { continue };
            let created = v.get("created_at").and_then(|x| x.as_i64()).unwrap_or(0);
            let sent = std::path::PathBuf::from(format!("{}.sent", path.display())).is_file();
            let name = path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_default();
            out.push((created, summary_row(&name, &v, sent)));
        }
    }
    out.sort_by(|a, b| b.0.cmp(&a.0));
    out.into_iter().map(|(_, v)| v).collect()
}

/// `fd_diag_counts()`: `<type>_<digits>.json` -> `{ type: count }`.
/// NOTE: strip only the trailing `_<digits>.json`, never split on `_`.
pub fn counts() -> Value {
    let mut map = serde_json::Map::new();
    for t in REPORT_TYPES {
        map.insert(t.to_string(), serde_json::json!(0));
    }
    if let Ok(entries) = fs::read_dir(reports_dir()) {
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let base = match name.strip_suffix(".json") {
                Some(b) => b.to_string(),
                None => continue,
            };
            let ty = match base.rsplit_once('_') {
                Some((t, tail)) if !t.is_empty() && tail.chars().all(|c| c.is_ascii_digit()) => t.to_string(),
                _ => continue,
            };
            let cur = map.get(&ty).and_then(|v| v.as_i64()).unwrap_or(0);
            map.insert(ty, serde_json::json!(cur + 1));
        }
    }
    Value::Object(map)
}

/// `fd_diag_report_send()`: POST one report to WordPress and write the
/// `<file>.sent` sidecar on success.
///
/// **Idempotent on purpose** — re-uploading the same file inserts a second row
/// (the document id is `(time << 20) | rand`), which is the exact duplicate bug
/// documented for the Android `sendNow()`. A `.sent` sidecar short-circuits it.
pub async fn report_send(path: &std::path::Path) -> bool {
    let marker = std::path::PathBuf::from(format!("{}.sent", path.display()));
    if marker.is_file() {
        return true;
    }
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let url = format!(
        "{}/report-diagnostic",
        crate::config::wp_api_base().trim_end_matches('/')
    );
    // `/report-diagnostic` is an authenticated route, so this POST has to carry the
    // same PM-Auth proof as every other origin call — it used to send only
    // `X-API-Secret`, which the origin now refuses, and the phone logged
    // "diag upload rejected status=401" every two minutes while every report piled
    // up locally. The ticket is included too: it also authorises bot-scoped routes.
    let send = |url: String, text: String| async move {
        let mut req = crate::httpc::client()
            .post(&url)
            .timeout(std::time::Duration::from_secs(12))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .body(text);
        let secret = crate::credentials::get_api_secret();
        if !secret.is_empty() {
            req = req.header("X-API-Secret", secret);
        }
        for (k, v) in crate::credentials::ticket_headers()
            .into_iter()
            .chain(crate::httpc::pm_auth_headers("POST", &url))
        {
            req = req.header(k.as_str(), v.as_str());
        }
        req.send().await
    };

    let mut result = send(url.clone(), text.clone()).await;
    if let Ok(r) = &result {
        if r.status().as_u16() == 401 {
            // Same recovery as http_json_h: a token the origin no longer accepts
            // must be replaced, not retried.
            crate::credentials::clear_auth_token();
            if crate::credentials::ensure_enrolled_throttled().await {
                result = send(url, text).await;
            }
        }
    }
    match result {
        Ok(r) if r.status().is_success() => {
            let _ = fs::write(&marker, b"");
            true
        }
        Ok(r) => {
            crate::log::log(
                "diag upload rejected",
                &serde_json::json!({"status": r.status().as_u16(), "file": path.display().to_string()}),
            );
            false
        }
        Err(e) => {
            crate::log::log(
                "diag upload failed",
                &serde_json::json!({"error": e.to_string(), "file": path.display().to_string()}),
            );
            false
        }
    }
}

/// `fd_diag_flush_pending()`: send every report that has no `.sent` sidecar.
/// Returns `(sent, failed)`.
pub async fn flush_pending() -> (usize, usize) {
    let dir = reports_dir();
    let mut sent = 0usize;
    let mut failed = 0usize;
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) != Some("json") {
                continue;
            }
            // SKIP what is already uploaded. `report_send()` returns `true` for a
            // file that carries a `.sent` marker (that is its idempotency guard),
            // so counting its return value made `diag flush {"sent":N}` report
            // every older report as freshly uploaded: the 2026-10-22 line said
            // `sent:11` while eight of those files were already marked, and a
            // genuinely failing origin would still have looked like a success.
            if std::path::PathBuf::from(format!("{}.sent", path.display())).is_file() {
                continue;
            }
            if report_send(&path).await {
                sent += 1;
            } else {
                failed += 1;
            }
        }
    }
    if sent + failed > 0 {
        crate::log::log(
            "diag flush",
            &serde_json::json!({"sent": sent, "failed": failed}),
        );
    }
    (sent, failed)
}

/// `fd_diag_clear()` deletes EVERYTHING in the reports directory, including the
/// `.sent` sidecars and any temp file (`glob('*')`). Only removing `*.json` left
/// the sidecars behind, which is how a cleared install still looked "already
/// sent" for reports that no longer existed.
/// Background auto-upload, throttled.
///
/// The `auto_report` toggle was stored, reported and **never acted on**: nothing
/// called [`report_send`] outside the explicit `?send=` / `?flush=1` routes, so
/// "Auto-send: on" uploaded nothing. The PHP spawns a detached CLI worker per
/// incident; this port deliberately spawns no processes, so the flush hangs off
/// the `/api/diagnostics` poll the Android service already makes every 3s.
///
/// Throttled to one flush per [`AUTO_FLUSH_INTERVAL_S`]: a failing origin would
/// otherwise be retried every 3s for each pending report.
pub const AUTO_FLUSH_INTERVAL_S: i64 = 120;

static LAST_AUTO_FLUSH: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

/// True when this call should start a flush (and records the attempt).
///
/// `compare_exchange` rather than load-then-swap, so two concurrent
/// `/api/diagnostics` polls cannot both decide to flush.
pub fn take_auto_flush_slot() -> bool {
    if !auto_report_enabled() {
        return false;
    }
    let t = now();
    let last = LAST_AUTO_FLUSH.load(std::sync::atomic::Ordering::Relaxed);
    if t - last < AUTO_FLUSH_INTERVAL_S {
        return false;
    }
    LAST_AUTO_FLUSH
        .compare_exchange(last, t, std::sync::atomic::Ordering::Relaxed, std::sync::atomic::Ordering::Relaxed)
        .is_ok()
}

/// True when an unsent report exists — cheap, so a poll can skip the flush.
pub fn has_pending() -> bool {
    let Ok(entries) = fs::read_dir(reports_dir()) else {
        return false;
    };
    entries.flatten().any(|e| {
        let path = e.path();
        path.extension().map(|x| x == "json").unwrap_or(false)
            && !std::path::PathBuf::from(format!("{}.sent", path.display())).is_file()
    })
}

pub fn clear() {
    if let Ok(entries) = fs::read_dir(reports_dir()) {
        for e in entries.flatten() {
            let path = e.path();
            if path.is_file() {
                let _ = fs::remove_file(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trimmed real `build.prop` (Android 5.1 / API 22 emulator shape).
    const PROP: &str = "\
# begin build properties
ro.product.model=SM-A805N
ro.product.brand=samsung
ro.build.version.release=5.1.1
ro.build.version.sdk=22
ro.build.description=generic_x86-user 5.1.1 LMY48Z 30 test-keys
";

    #[test]
    fn reads_a_plain_build_prop_value() {
        assert_eq!(build_prop_lookup(PROP, "ro.product.model"), "SM-A805N");
        assert_eq!(build_prop_lookup(PROP, "ro.build.version.sdk"), "22");
    }

    #[test]
    fn build_prop_lookup_ignores_comments_and_absent_keys() {
        assert_eq!(build_prop_lookup(PROP, "# begin build properties"), "");
        assert_eq!(build_prop_lookup(PROP, "ro.not.there"), "");
        assert_eq!(build_prop_lookup("", "ro.product.model"), "");
    }

    #[test]
    fn build_prop_values_keep_their_spaces() {
        // `ro.build.description` contains spaces; only the FIRST `=` splits.
        let v = build_prop_lookup(PROP, "ro.build.description");
        assert_eq!(v, "generic_x86-user 5.1.1 LMY48Z 30 test-keys");
    }

    #[test]
    fn build_prop_handles_crlf_and_padding() {
        let crlf = "  ro.product.model = Pixel 7  \r\nro.build.version.release=14\r\n";
        assert_eq!(build_prop_lookup(crlf, "ro.product.model"), "Pixel 7");
        assert_eq!(build_prop_lookup(crlf, "ro.build.version.release"), "14");
    }

    #[test]
    fn cpu_percent_is_the_ratio_of_two_samples() {
        let t0 = std::time::Instant::now();
        let t1 = t0 + std::time::Duration::from_secs(2);
        // 200 ms of CPU over 2 s of wall is a tenth of one core.
        let pct = percent_between(0, t0, 200_000_000, t1).expect("a sample");
        assert!((pct - 10.0).abs() < 0.001, "got {pct}");
        // A counter that did not move is 0%, not a divide-by-zero.
        assert_eq!(percent_between(500, t0, 500, t1), Some(0.0));
        // No elapsed wall time cannot produce a ratio.
        assert_eq!(percent_between(0, t0, 100, t0), None);
    }

    #[test]
    fn device_info_never_panics_without_proc() {
        // Everything must degrade to a placeholder rather than fail - this is how
        // a diagnostics report survives Android's restricted /proc.
        let info = device_info();
        assert_eq!(info["runtime"], "pencarimovie-server");
        assert_eq!(info["platform"], std::env::consts::OS);
        assert!(info["device_id"].as_str().unwrap_or("").len() >= 6);
    }
    
    /// The dashboard's send loop needs `file` and `sent`; without them it can
    /// never enqueue a send (measured on-device: 10 reports, 0 sendable).
    #[test]
    fn summary_row_carries_file_and_sent() {
        let raw = serde_json::json!({
            "report_type": "playback_incident",
            "created_at": 1792659577,
            "error_title": "download refused (HTTP 400)",
            "failure_reason": "MISSING_FILE_ID",
            "item_id": "pm:post:9000031194",
            "short_code": "wyjXQu"
        });
        let row = summary_row("playback_incident_1792659577.json", &raw, false);
        assert_eq!(row["file"], "playback_incident_1792659577.json");
        assert_eq!(row["sent"], false);
        assert_eq!(row["report_type"], "playback_incident");
        assert_eq!(row["created_at"], 1792659577);
        assert_eq!(row["failure_reason"], "MISSING_FILE_ID");
        // The raw file's bulk (short_code here) is what `?file=` serves instead.
        assert!(row.get("short_code").is_none());
        assert_eq!(summary_row("x_1.json", &raw, true)["sent"], true);
    }

    /// A report file's `<name>.json.sent` sidecar must never be listed as its own
    /// report (only `*.json` is, and the sidecar ends in `.json.sent`).
    #[test]
    fn counts_strip_only_the_trailing_timestamp() {
        for name in [
            "playback_incident_1792659577.json",
            "stream_incident_1.json",
        ] {
            let ty = name.trim_end_matches(".json").rsplit_once('_').map(|(t, tail)| {
                assert!(tail.chars().all(|c| c.is_ascii_digit()));
                t.to_string()
            });
            let expected = name.split('_').next().unwrap();
            assert!(ty.as_deref().unwrap().starts_with(expected));
        }
        assert!(!"debug.log.sent".ends_with(".json"));
    }

    /// The debounce is what keeps a failing stream URL (re-requested on every
    /// range and every player reopen) from burying every other incident.
    #[test]
    fn refusal_throttle_is_once_per_window_per_signature() {
        let mut seen = std::collections::HashMap::new();
        assert!(take_slot(&mut seen, "a", 1000, 300), "first report is written");
        assert!(!take_slot(&mut seen, "a", 1299, 300), "same signature inside the window is dropped");
        assert!(take_slot(&mut seen, "b", 1299, 300), "a different signature is independent");
        assert!(take_slot(&mut seen, "a", 1300, 300), "window elapsed -> reportable again");
        assert_eq!(seen.len(), 2);

        // Expired signatures are forgotten, so the map cannot grow forever.
        assert!(take_slot(&mut seen, "c", 99_999, 300));
        assert_eq!(seen.len(), 1);
    }
}
