//! Port of `fd_is_debug_enabled`, `fd_set_debug_enabled`, `fd_log_stderr` and
//! `fd_log` (backend.php:368-510).
//!
//! Same on-disk paths (`storage/debug_mode.txt`, `storage/debug.log`), same 5 MB
//! cap, same `[YYYY-MM-DD HH:MM:SS] [PencariMovie Server] <msg> <json>` line
//! shape. Timestamps are UTC (the PHP used the process timezone; the port has no
//! tz database, so it documents UTC instead of guessing).

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde_json::Value;

use crate::config::{debug_from_env, MAX_LOG_SIZE};
use crate::storage::{limit_file_size, now, storage_path};

/// `--tray` has no console to print to, so every stderr line is ALSO appended to
/// `storage/server.log` while it is active. That is what makes "Open log" work in
/// a background install started by a double-click or the `Run` key, without a
/// shell redirect (`start ""` opens a fresh console and drops `>>`).
static MIRROR_STDERR: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Mirror [`stderr_log`] into `storage/server.log` (called once by `--tray`).
pub fn mirror_stderr_to_server_log() {
    MIRROR_STDERR.store(true, std::sync::atomic::Ordering::Relaxed);
}

/// Is stderr still attached to a real console? Cleared when the tray detaches.
///
/// `FreeConsole()` closes the console out from under the process, so the stale
/// stderr handle starts failing. `eprintln!` PANICS on a failed write, which
/// would kill the tray thread on its very next log line — hence this guard
/// instead of "just write and ignore the error".
static STDERR_LIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Stop writing to stderr (the console is gone; [`server_log_path`] takes over).
pub fn silence_stderr() {
    STDERR_LIVE.store(false, std::sync::atomic::Ordering::Relaxed);
}

/// The file the tray's "Open log" item opens.
pub fn server_log_path() -> PathBuf {
    storage_path("server.log")
}

pub fn debug_toggle_path() -> PathBuf {
    storage_path("debug_mode.txt")
}

pub fn debug_log_path() -> PathBuf {
    storage_path("debug.log")
}

pub fn is_debug_enabled() -> bool {
    if debug_from_env() {
        return true;
    }
    for file in [
        storage_path("debug_mode.txt"),
        storage_path("storage/debug_mode.txt"),
    ] {
        if let Ok(v) = fs::read_to_string(&file) {
            let s = v.trim().to_ascii_lowercase();
            if s == "1" || s == "true" || s == "on" {
                return true;
            }
        }
    }
    false
}

pub fn set_debug_enabled(enabled: bool) {
    let file = debug_toggle_path();
    if enabled {
        if let Some(parent) = file.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&file, "1");
    } else {
        let _ = fs::remove_file(&file);
    }
}

/// Write a line to stderr WITHOUT the panic path.
///
/// `eprint!`/`eprintln!` PANIC when the write fails, and that panic kills the
/// caller's task: the response is never written and the client sees the
/// connection close with nothing on it. `STDERR_LIVE` does not cover it, because
/// that flag is a STARTUP snapshot — a console that dies later (a recycled
/// terminal, a detached daemon) still passes the guard and then panics on the
/// next line.
///
/// Measured on this workstation: a long-lived instance answered `/api/version`
/// (which does not log) in 2.9 ms and `/thumb/…` (cache hit, no log) in 3.9 ms,
/// while `/manifest.json` closed the connection with no response in 27 ms.
fn write_stderr(line: &str) {
    use std::io::Write;
    let _ = std::io::stderr().write_all(line.as_bytes());
}

/// Always-on stderr line (`fd_log_stderr`).
pub fn stderr_log(message: &str, ctx: Option<&Value>) {
    let suffix = match ctx {
        Some(v) if v.as_object().map(|o| !o.is_empty()).unwrap_or(false) => format!(" {}", v),
        _ => String::new(),
    };
    if STDERR_LIVE.load(std::sync::atomic::Ordering::Relaxed) {
        write_stderr(&format!("[PencariMovie] {}{}\n", message, suffix));
    }
    if MIRROR_STDERR.load(std::sync::atomic::Ordering::Relaxed) {
        let line = format!("[{}] [PencariMovie] {}{}\n", stamp(now()), message, suffix);
        let path = server_log_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
            let _ = f.write_all(line.as_bytes());
        }
    }
}

pub fn log(message: &str, ctx: &Value) {
    if !is_debug_enabled() {
        return;
    }
    let empty = ctx.as_object().map(|o| o.is_empty()).unwrap_or(true);
    let suffix = if empty {
        String::new()
    } else {
        format!(" {}", ctx)
    };
    let line = format!(
        "[{}] [PencariMovie Server] {}{}\n",
        stamp(now()),
        message,
        suffix
    );
    let path = debug_log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
    // SAME GUARD AS `stderr_log()`, and for the same documented reason (see the
    // note on `silence_stderr` above): `eprint!` PANICS when the write fails, so
    // a process whose console has gone away (tray `FreeConsole()`, a closed
    // terminal, a detached daemon) killed the calling task on its very next log
    // line. Measured on this workstation: an instance whose terminal was gone
    // answered `/api/version` (does not log) with 200 while `/stream`, `/manifest.json`,
    // `/api/diagnostics` and `/api/download` closed the connection with NO
    // response in 0.2-0.5 s — the exact signature of a panicking handler. A fresh
    // process with a live stderr served the same download at 5.3 MB/s.
    if STDERR_LIVE.load(std::sync::atomic::Ordering::Relaxed) {
        write_stderr(&format!("[PencariMovie Server] {}{}\n", message, suffix));
    }
    limit_file_size(&path, MAX_LOG_SIZE);
}

fn stamp(ts: i64) -> String {
    let days = ts.div_euclid(86400);
    let secs = ts.rem_euclid(86400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        y,
        m,
        d,
        secs / 3600,
        (secs % 3600) / 60,
        secs % 60
    )
}

/// Howard Hinnant's `civil_from_days`, valid for any i64 day count.
fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_is_1970() {
        assert_eq!(stamp(0), "1970-01-01 00:00:00");
    }

    #[test]
    fn known_timestamp() {
        // 2026-10-19T16:11:06Z
        assert_eq!(stamp(1792426266), "2026-10-19 16:11:06");
    }
}
