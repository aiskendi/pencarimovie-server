//! HTTP entry point. `axum` + `tokio`, `127.0.0.1:8088` by default.
//!
//! ```text
//!   pencarimovie-server --port 8088 [--host 127.0.0.1] [--workers N]
//!                     [--inflight 8] [--max-streams 256] [--tray]
//!   pencarimovie-server --decode <file_id_mt> [expected_id]
//! ```
//!
//! `--tray` puts the icon in the system tray and runs the server behind it, in
//! this same process (`tray.rs` on Windows, `tray_unix.rs` on Linux). It also
//! hides its own console, which is what lets the launchers drop
//! `powershell -WindowStyle Hidden`. `--no-tray` forces it off.
//!
//! `--inflight N` (default 8) caps `upload.getFile` parts in flight **per
//! connection**: each `Client` in a `BotEntry` owns its own semaphore, so
//! `--workers 8 --inflight 8` is 64 parts across 8 connections. `--max-streams`
//! (default 256) caps concurrent `/api/download` bodies; excess viewers queue for
//! 5 s and then get 503 + `Retry-After`.
//!
//! `--workers N` creates N independent grammers `Client`s (each with its own
//! `SenderPool`, therefore its own connection set) and round-robins requests
//! over them. That is the only way to get more than one connection per DC with
//! grammers; see README.md for the measured difference between
//! "1 client x 4 tasks" and "4 clients x 1 task".

mod assets;
mod auth;
mod cache;
mod config;
mod credentials;
mod diagnostics;
mod download;
mod eclipse;
mod fileid;
mod httpc;
mod log;
mod matcher;
mod proc;
mod ratelimit;
mod resolve;
mod respond;
mod routes;
mod session;
mod storage;
mod telegram;
// The tray lives in the binary (`--tray`), so a folder holding only the
// executable still gets an icon: no `tray.ps1`, no PowerShell, no loose script.
// `tray.rs` is the native Windows icon; `tray_unix.rs` drives `yad` on Linux.
// Android has no notification area, so it gets neither.
#[cfg(windows)]
mod tray;
#[cfg(all(unix, not(target_os = "android")))]
mod tray_unix;
#[cfg(windows)]
use crate::tray as tray_impl;
#[cfg(all(unix, not(target_os = "android")))]
use crate::tray_unix as tray_impl;
mod tunnel;
mod update;
mod upstream;
mod wp_data;

use std::io::IsTerminal;
use std::net::SocketAddr;
use std::process::Stdio;
use std::sync::Arc;

use axum::Router;
use axum::extract::Request;
use axum::response::Response;
use tokio::net::TcpListener;

pub struct AppState {
    pub port: u16,
    pub workers: usize,
    /// Max `upload.getFile` parts in flight per grammers `Client` connection.
    pub inflight: usize,
    /// Global cap on concurrent `/api/download` responses (`--max-streams`).
    pub download_gate: Arc<tokio::sync::Semaphore>,
    pub max_streams: usize,
    /// Per-client abuse limits (`ratelimit`). In-memory, so per process — the same
    /// caveat the resolve and catalog caches carry.
    pub limits: Arc<ratelimit::Limiter>,
    /// Distinct short codes one client may have in flight (`0` = off).
    pub max_titles_per_ip: usize,
    /// Concurrent bodies one client may hold, a socket-flood backstop (`0` = off).
    pub max_bodies_per_ip: usize,
    /// Cold starts one client may spend per `cold_start_window` (`0` = off).
    pub cold_starts_per_ip: usize,
    /// The cold-start window, in seconds.
    pub cold_start_window: u64,
    pub telegram: telegram::TelegramPool,
}

fn arg_value(flag: &str) -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    let i = args.iter().position(|a| a == flag)?;
    args.get(i + 1).cloned()
}

fn arg_pos(flag: &str) -> Option<usize> {
    let args: Vec<String> = std::env::args().collect();
    args.iter().position(|a| a == flag)
}

/// `0` means the counter is off, and the banner says so instead of printing a
/// misleading `0`.
fn cap_or_off(n: usize) -> String {
    if n == 0 {
        "unlimited".to_string()
    } else {
        n.to_string()
    }
}

/// One `AppState` per process; the router has a single fallback, so the state is
/// stored here instead of behind an extractor (a plain `Arc<AppState>` argument
/// is not an extractor, and `State<..>` would force a generic router).
static STATE: std::sync::OnceLock<Arc<AppState>> = std::sync::OnceLock::new();

#[allow(dead_code)]
fn _assert_future_send() {
    fn need<T: Send>(_: T) {}
    let req = Request::builder().body(axum::body::Body::empty()).unwrap();
    need(serve_all(req));
}

async fn serve_all(req: Request) -> Response {
    let state = STATE
        .get()
        .expect("AppState must be initialised before serving")
        .clone();
    routes::handle(state, req).await
}

/// Wakes `shutdown_signal()`. The tray's "Stop server" item is the second way to
/// ask for a graceful stop (Ctrl-C is the first); a `Notify` carries a permit, so
/// a click that arrives before the server starts awaiting is not lost.
static SHUTDOWN: std::sync::OnceLock<tokio::sync::Notify> = std::sync::OnceLock::new();

fn shutdown_trigger() -> &'static tokio::sync::Notify {
    SHUTDOWN.get_or_init(tokio::sync::Notify::new)
}

/// Ask the process to stop serving (called from the tray thread).
#[cfg(windows)]
pub(crate) fn request_shutdown() {
    shutdown_trigger().notify_one();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // `--version` / `-V`: one line, no storage, no network. This is what the
    // packaging scripts assert on every platform (`--version` must print *before*
    // anything tries to boot grammers or read `storage/`).
    if arg_pos("--version").is_some() || arg_pos("-V").is_some() {
        println!("{}", version_line());
        return Ok(());
    }

    // The APP shell that launched this engine (the Android APK's versionName).
    // `/api/version` reports it as `client_version`; without it the dashboard
    // could only show the engine version, so users could not tell which APK they
    // were running.
    if let Some(v) = arg_value("--app-version").or_else(|| storage::env("PM_APP_VERSION")) {
        config::set_client_version(&v);
    }

    // `--decode` / `--selftest`: print the Bot API file_id decoding, then exit.
    if let Some(i) = arg_pos("--decode").or_else(|| arg_pos("--selftest")) {
        let args: Vec<String> = std::env::args().collect();
        let Some(raw) = args.get(i + 1) else {
            eprintln!("usage: pencarimovie-server --decode <file_id_mt> [expected_id]");
            std::process::exit(2);
        };
        let expected = args.get(i + 2).map(|s| s.as_str());
        match fileid::decode_file_id(raw) {
            Ok(d) => {
                let mut out = d.to_json();
                if let Some(exp) = expected {
                    out["id_matches_expected"] = serde_json::json!(exp == d.id.to_string());
                }
                println!("{}", serde_json::to_string_pretty(&out)?);
                return Ok(());
            }
            Err(e) => {
                eprintln!("{{\"ok\":0,\"error\":\"{}\"}}", e);
                std::process::exit(1);
            }
        }
    }

    // `--audio-probe` was REMOVED with the ALAC decoder (`src/audio.rs` and the
    // `symphonia` dependency). It existed only to diagnose that decoder's log
    // lines; nothing decodes audio in this binary any more, so the flag would
    // have had nothing to print.

    let port: u16 = arg_value("--port")
        .or_else(|| storage::env("PM_LISTEN_PORT"))
        .or_else(|| storage::env("PORT"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(8088);
    let host = arg_value("--host")
        .or_else(|| storage::env("PM_LISTEN_HOST"))
        .or_else(|| storage::env("HOST"))
        // `set PM_LISTEN_HOST=0.0.0.0 && ...` keeps a trailing space in the value,
        // and "0.0.0.0 :8088" then fails to parse as a socket address.
        .map(|h| h.trim().to_string())
        .unwrap_or_else(|| default_bind_host(cfg!(target_os = "android"), paas_env_present()).to_string());
    // DEFAULT: 4 connections, on every platform.
    //
    // `workers × inflight` is the number of `upload.getFile` parts the engine can
    // have in flight — a SHARED budget for every viewer, not a per-viewer one.
    // With the old default of ONE connection that budget was 8 parts for the whole
    // process, so a single player that seeks once while another stream is open kept
    // the route queued: a 55-request real session at `--workers 1` logged repeated
    // `download refused` / `pre-flight budget exhausted`, while the same session at
    // `--workers 4` logged 55/55 served with `timed out: 0`, `gate saturated: 0`,
    // `attempt failed: 0`, `download refused: 0` (Telegram's `FLOOD_WAIT` 1-3 s on
    // individual parts is absorbed by the per-part retry path). 4 x 8 = 32 parts is
    // also the shape the engine's own inflight note describes as correct — the TS
    // port holds 200/200 with 4 sessions x 4 in flight — and `grammers` multiplexes
    // a whole DC over ONE connection, so parallel connections are the only way to
    // scale a busy host.
    //
    // Measured on a phone (CPH2581, API 36) as well as on desktop: 8/8 sequential
    // and 4/4 concurrent 512 KB reads succeed at the default, so this does not cost
    // correctness on Android. `PM_WORKERS` overrides it, matching the other knobs
    // (`PM_INFLIGHT`, `PM_LISTEN_PORT`); a handset can still pass `--workers 1`.
    let workers: usize = arg_value("--workers")
        .or_else(|| storage::env("PM_WORKERS"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(4)
        .max(1);
    // Telegram's own guidance is N connections x a handful of in-flight parts
    // per connection. grammers multiplexes a whole DC over ONE connection, so
    // 200 concurrent `upload.getFile` on one `Client` get reset by the DC
    // (measured: 914 x "Connection reset by peer (os error 104)"). The default
    // 8 is the shape the TS port holds 200/200 with (4 sessions x 4 in flight).
    let inflight: usize = arg_value("--inflight")
        .or_else(|| storage::env("PM_INFLIGHT"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(8)
        .max(1);
    // Global cap on concurrent download responses: beyond it viewers get a
    // fast `503 / retry_after: 1` instead of all opening a Telegram part at once.
    //
    // DEFAULT IS 0 = NO CAP, because this product is self-hosted: on a desktop,
    // an Android box or a VPS the real throttle is already the Telegram pool
    // (`workers × inflight` parts), and there is no 30 s router to kill queued
    // requests. A cap there only turns "slightly slower" into "refused".
    //
    // Hosts that DO have a short request timeout should set it explicitly: on
    // Heroku a 1× Basic dyno measured 250/400 served with `PM_MAX_STREAMS=256`
    // (clean JSON 503 + retry_after for the rest) versus 312/400 uncapped where
    // the 88 losers died on `H12` after 30 s and got an HTML error page.
    //
    // NOTE: `.max(1)` used to sit here and turned an explicit `0` into a gate of
    // ONE — one viewer, everyone else 503. Do not put it back.
    let max_streams: usize = arg_value("--max-streams")
        .or_else(|| storage::env("PM_MAX_STREAMS"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let gate_permits = if max_streams == 0 {
        tokio::sync::Semaphore::MAX_PERMITS
    } else {
        max_streams
    };

    // Per-client abuse limits on `/api/download` (`ratelimit.rs`).
    //
    // `--max-titles-per-ip` counts DISTINCT short codes in flight, NOT sockets. A
    // seek, a 16-connection download manager and a preload of the next episode all
    // operate on the same code and therefore all cost one; 40 different titles in
    // parallel cost 40, which is the scraper signature. Without that distinction a
    // capability limit would break segmented downloads — see the tests in
    // `ratelimit::tests`.
    //
    // `--max-bodies-per-ip` is the socket-flood backstop and sits above every real
    // download manager (aria2 defaults to 16, IDM to 8).
    //
    // `--cold-starts-per-ip` is charged only where a resolve is actually issued, so
    // a `Range` request, a seek and a warm short code are free.
    //
    // `0` disables a counter entirely — one integer compare, no map touched. A
    // self-hosted box that wants today's behaviour exactly sets all of them to 0.
    let max_titles_per_ip: usize = arg_value("--max-titles-per-ip")
        .or_else(|| storage::env("PM_MAX_TITLES_PER_IP"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(8);
    let max_bodies_per_ip: usize = arg_value("--max-bodies-per-ip")
        .or_else(|| storage::env("PM_MAX_BODIES_PER_IP"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(32);
    let cold_starts_per_ip: usize = arg_value("--cold-starts-per-ip")
        .or_else(|| storage::env("PM_COLD_STARTS_PER_IP"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(20);
    let cold_start_window: u64 = arg_value("--cold-start-window")
        .or_else(|| storage::env("PM_COLD_START_WINDOW"))
        .and_then(|s| s.parse().ok())
        .unwrap_or(300);

    let state = Arc::new(AppState {
        port,
        workers,
        inflight,
        download_gate: Arc::new(tokio::sync::Semaphore::new(gate_permits)),
        max_streams,
        limits: ratelimit::Limiter::new(),
        max_titles_per_ip,
        max_bodies_per_ip,
        cold_starts_per_ip,
        cold_start_window,
        telegram: telegram::TelegramPool::new(workers, inflight),
    });
    storage::prune_cache_files(false);

    // Bring every env-supplied pool bot up before the first request, so each
    // instance holds a session for all of them and `pick_pool_bot()` can spread
    // downloads across the pool. Detached and best-effort: a slow or expired
    // token must never delay the listener. Why an env list at all is explained on
    // `session::env_bot_tokens()` — the pool file is per-instance disk state.
    {
        let boot_state = state.clone();
        tokio::spawn(async move {
            let tokens = session::env_bot_tokens();
            if tokens.is_empty() {
                return;
            }
            log::stderr_log(
                "booting env bot pool",
                Some(&serde_json::json!({ "bots": tokens.len() })),
            );
            for (idx, token) in tokens.into_iter().enumerate() {
                let id = token.split(':').next().unwrap_or("").trim().to_string();
                if id.is_empty() {
                    continue;
                }
                match boot_state.telegram.boot(&id, Some(&token)).await {
                    Ok(entry) => {
                        // Real @username/name for the pool list (see
                        // `session::remember_env_identity`).
                        session::remember_env_identity(
                            &entry.identity.id,
                            &entry.identity.username,
                            &entry.identity.first_name,
                        );
                        if idx == 0 && session::get_bot_id().is_empty() {
                            session::save_session_meta(
                                &entry.identity.id,
                                &entry.identity.username,
                                &entry.identity.first_name,
                            );
                        }
                        log::stderr_log(
                            "env pool bot ready",
                            Some(&serde_json::json!({
                                "bot_id": entry.identity.id,
                                "username": entry.identity.username,
                            })),
                        );
                    }
                    Err(e) => log::stderr_log(
                        "env pool bot failed",
                        Some(&serde_json::json!({ "bot_id": id, "error": e })),
                    ),
                }
            }
        });
    }

    // Enrol for PM-Auth BEFORE anything talks to the origin.
    //
    // `/provision-session` used to be the only enrolment path, so an install that
    // already held a bot session never got a token and authenticated with the
    // fleet-wide `X-API-Secret` instead. Retiring that secret would therefore have
    // broken playback on every existing install — including this build — not just
    // on old ones. Detached: a slow origin must not delay the listener.
    {
        tokio::spawn(async move {
            let outcome = credentials::ensure_enrolled().await;
            // `outcome` apart from `enrolled`: "the origin was unreachable" and
            // "the origin refused this install" are the same `false` otherwise, and
            // they send an operator in opposite directions.
            log::stderr_log(
                "pm-auth enrolment",
                Some(&serde_json::json!({
                    "enrolled": outcome == credentials::EnrolOutcome::Enrolled,
                    "outcome": format!("{outcome:?}"),
                    "bot_id": session::get_bot_id(),
                })),
            );
        });
    }

    // Make sure this instance holds ONE guest bot. Detached for the same reason as
    // the warm above: a lease is an HTTP round-trip plus a Telegram login and must
    // not delay the listener. One per instance is a hard rule (a pooled fleet may
    // not multiply our provisioning — see `session::GUEST_BOTS_PER_INSTANCE`);
    // anyone who wants a bigger pool adds their own bots via `BOT_TOKENS`.
    {
        let pool_state = state.clone();
        tokio::spawn(async move {
            let have = session::ensure_guest_bot(&pool_state.telegram).await;
            log::stderr_log(
                "guest bot ready",
                Some(&serde_json::json!({ "bots": have })),
            );
        });
    }

    let _ = STATE.set(state.clone());
    // One fallback handles every path, exactly like backend.php's dispatch block.
    let app = Router::new().fallback(serve_all);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    // Double-clicking the executable must not end in a red error: if the port is
    // already taken, that is almost always a server this user started earlier, so
    // open it instead of failing.
    let listener = match TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => {
            let url = format!("http://127.0.0.1:{port}/");
            println!("\n  PencariMovie Server is already running on {url}");
            println!("  Opening it in your browser instead of starting a second copy.");
            println!("  To restart it, stop the running one first (pms stop, stop.bat, or close its window).\n");
            open_browser(&url);
            return Ok(());
        }
        Err(e) => return Err(e.into()),
    };

    log::stderr_log(
        "PencariMovie Server listening",
        Some(&serde_json::json!({
            "port": port,
            "host": host,
            "workers": workers,
            "inflight": inflight,
            "max_streams": max_streams,
            // The per-client caps, reported so an operator (and a bug report)
            // shows what is actually enforced instead of what the docs claim.
            "max_titles_per_ip": state.max_titles_per_ip,
            "max_bodies_per_ip": state.max_bodies_per_ip,
            "cold_starts_per_ip": state.cold_starts_per_ip,
            "cold_start_window": state.cold_start_window,
            "storage": storage::storage_dir().display().to_string(),
            "version": config::app_version(),
            "runtime": format!("pencarimovie-server {}", config::app_version()),
            "target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
        })),
    );
    // `--host 0.0.0.0` (Android, Docker, PaaS) is not reachable as 0.0.0.0, so
    // print the address a browser can actually use.
    let shown = if host == "0.0.0.0" || host == "::" {
        "127.0.0.1".to_string()
    } else {
        host.clone()
    };
    // Decide the tray ONCE, here, before anything is printed.
    //
    // `--tray` forces it; `--no-tray` / `PM_NO_TRAY` forbid it; otherwise a
    // human-launched console gets one. Linux and Windows share this decision via
    // `tray_impl`, and Android never does.
    let console_is_terminal =
        !cfg!(target_os = "android") && std::io::stdout().is_terminal();
    #[cfg(not(target_os = "android"))]
    let tray_requested = tray_impl::wanted(console_is_terminal);
    #[cfg(target_os = "android")]
    let tray_requested = false;

    // If a tray icon is going to be created, this console is about to be released
    // (`FreeConsole`), so a banner here would only flash and vanish. The same
    // information already goes to storage/server.log (tray "Open log"), and it has
    // already gone to stderr above in the "listening" line.
    if !tray_requested {
        println!(
            "\n  PencariMovie Server\n  Local:       http://{}:{}\n  Listening:   {}:{}\n  Storage:     {}\n  Workers:     {} (one Telegram connection each)\n  Inflight:    {} parts per connection ({} total)\n  Max streams: {}\n  Per client:  {} title(s) and {} connection(s) at once, {} new stream(s) per {}s\n",
            shown,
            port,
            host,
            port,
            storage::storage_dir().display(),
            workers,
            inflight,
            workers * inflight,
            max_streams,
            // Counted per TITLE, not per socket: a seek, a segmented download and a
            // preload of the same file all cost one. `0` = that counter is off.
            cap_or_off(max_titles_per_ip),
            cap_or_off(max_bodies_per_ip),
            cap_or_off(cold_starts_per_ip),
            cold_start_window,
            // Local dashboard, LAN devices and a tunnel's own loopback dial are exempt.
        );
        if host == "0.0.0.0" || host == "::" {
            println!("  LAN/TV:      http://<this-device-ip>:{}\n", port);
        }
    }

    // A user who double-clicks the exe expects a browser, not a black window with
    // logs. A terminal was inherited only when a human launched it, so that is the
    // signal; `--open` forces it, `--no-open` suppresses it (autostart, service
    // managers, scripts).
    //
    // `--tray` is background mode and implies `--no-open`: no browser, because
    // start-tray and the login entry are "start it and get out of the way". A
    // double-click still opens the browser — that is what opening the app means —
    // it just also drops the console once the icon exists.
    let url = format!("http://{shown}:{port}/");
    let tray_flag = arg_pos("--tray").is_some();
    let interactive = !cfg!(target_os = "android")
        && !paas_env_present()
        && !tray_flag
        && arg_pos("--no-open").is_none()
        && (arg_pos("--open").is_some() || console_is_terminal);
    if interactive {
        if !tray_requested {
            println!("  Close this window (or press Ctrl+C) to stop the server.\n");
        }
        open_browser(&url);
    }

    // The icon is this process, not a script beside it: `--tray` (start-tray,
    // the login entry) or a plain double-click. On Windows it must be the same
    // process because the shell stores tray settings per HOSTING executable — the
    // old PowerShell-hosted icon shared `powershell.exe`'s entry and could only
    // ever sit in the hidden-icons flyout.
    #[cfg(not(target_os = "android"))]
    if tray_requested {
        tray_impl::spawn(port, &host);
    }

    let shutdown = shutdown_signal();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await?;
    // The cache writes behind a 400 ms buffer; commit whatever is still queued so a
    // clean stop does not lose the last window.
    let flushed = cache::flush();
    if flushed > 0 {
        log::log("cache flushed on shutdown", &serde_json::json!({"entries": flushed}));
    }
    telegram::disconnect_all();
    Ok(())
}

fn rustc_version() -> &'static str {
    option_env!("RUSTC_VERSION").unwrap_or("unknown")
}

/// Open `url` in the user's default browser. Best effort: a box with no browser
/// (server install, headless CI) must never stop the server from running, so a
/// failure is only logged.
pub(crate) fn open_browser(url: &str) {
    // `proc::cmd`, not `Command::new`: a plain `cmd /C start` gets its own console,
    // so launching the server flashed a command prompt before the browser opened.
    let spawned = if cfg!(target_os = "windows") {
        // `start` is a cmd builtin, and its first quoted argument is the window
        // TITLE - passing an empty "" is what makes the second one the URL.
        proc::cmd("cmd")
            .args(["/C", "start", "", url])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    } else if cfg!(target_os = "macos") {
        proc::cmd("open")
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    } else {
        proc::cmd("xdg-open")
            .arg(url)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    };
    match spawned {
        Ok(mut child) => {
            // `xdg-open` blocks until the browser exits; reap it off the runtime.
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(e) => log::log(
            "could not open a browser",
            &serde_json::json!({"url": url, "error": e.to_string()}),
        ),
    }
}

/// One-line build identity, printed by `--version`:
/// `pencarimovie-server 1.0.50-3-g9fc84e1 (build 9fc84e1, rustc 1.99.1, x86_64-windows)`.
/// The first field is what this install reports (`/api/version` → `current_version`);
/// the second is the git state the binary was compiled from.
fn version_line() -> String {
    // `rustc` is only reported when it can actually be detected: a packaged
    // build has no toolchain on PATH, and "rustc unknown" is noise.
    let rustc = rustc_version();
    let compiler = if rustc.is_empty() || rustc == "unknown" {
        String::new()
    } else {
        format!("rustc {}, ", rustc)
    };
    format!(
        "pencarimovie-server {} (build {}, {}{}-{})",
        config::app_version(),
        config::build_id(),
        compiler,
        std::env::consts::ARCH,
        std::env::consts::OS
    )
}

/// The default bind address.
///
/// `127.0.0.1` is the documented desktop default (Windows, Linux, macOS) — do
/// not change it, `stop.bat` / `stop.sh` and every doc assume a loopback
/// listener. Two deployments need `0.0.0.0` instead:
///
/// - **Android / Termux**: a phone or TV box is only useful when the TV (or
///   another device on the LAN) can reach it, and there is no host firewall to
///   hide behind.
/// - **PaaS / Docker**: every PaaS (Heroku, Render, Fly, Railway, Cloud Run,
///   Azure) routes traffic to the container's external interface; a `127.0.0.1`
///   listener is unreachable and the health check fails.
///
/// `--host`, `PM_LISTEN_HOST` and `HOST` all override this.
pub fn default_bind_host(is_android: bool, paas: bool) -> &'static str {
    if is_android || paas {
        "0.0.0.0"
    } else {
        "127.0.0.1"
    }
}

/// Environment markers a PaaS sets that a desktop never does.
///
/// `PORT` is deliberately NOT one: the PHP launchers set `PORT=8088` on a
/// desktop, and treating it as "public" would silently expose a LAN listener.
const PAAS_MARKERS: [&str; 8] = [
    "DYNO",                  // Heroku
    "RENDER",                // Render
    "FLY_APP_NAME",          // Fly.io
    "RAILWAY_ENVIRONMENT",   // Railway
    "K_SERVICE",             // Google Cloud Run
    "WEBSITE_SITE_NAME",     // Azure App Service
    "PM_PAAS",               // explicit opt-in for any other container host
    "PM_BIND_PUBLIC",        // explicit opt-in on a VPS
];

/// Pure so it can be tested without mutating the process environment.
pub fn paas_env_predicate(get: impl Fn(&str) -> bool) -> bool {
    PAAS_MARKERS.iter().any(|k| get(k))
}

fn paas_env_present() -> bool {
    paas_env_predicate(|k| storage::env(k).is_some())
}

/// Ctrl-C everywhere, plus SIGTERM on Unix.
///
/// `tokio::signal::unix` is `#[cfg(unix)]` inside tokio, so the previous
/// unconditionally-unix version of this block was the ONLY thing that stopped
/// the Windows (MSVC) build — every dependency, including grammers, compiled.
/// Windows has no SIGTERM: `stop.bat` kills the listener and a console close
/// event is not interceptable from a background process, so Ctrl-C is the
/// graceful path there.
#[cfg(unix)]
async fn shutdown_signal() {
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("SIGTERM handler");
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = term.recv() => {}
        _ = shutdown_trigger().notified() => {}
    }
}

#[cfg(windows)]
async fn shutdown_signal() {
    // Ctrl-C, or the tray's "Stop server" (which goes through the same graceful
    // path, so the cache still flushes and the icon is removed).
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = shutdown_trigger().notified() => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_keeps_the_documented_loopback_default() {
        // `start.bat` / `start.sh` and every doc assume a 127.0.0.1 listener.
        assert_eq!(default_bind_host(false, false), "127.0.0.1");
    }

    #[test]
    fn android_and_paas_bind_all_interfaces() {
        assert_eq!(default_bind_host(true, false), "0.0.0.0");
        assert_eq!(default_bind_host(false, true), "0.0.0.0");
        assert_eq!(default_bind_host(true, true), "0.0.0.0");
    }

    #[test]
    fn port_alone_is_not_a_paas_marker() {
        // The PHP launchers export PORT=8088 on a desktop; that must not turn a
        // loopback listener into a LAN-exposed one.
        assert!(!paas_env_predicate(|k| k == "PORT"));
        assert!(!paas_env_predicate(|_| false));
    }

    #[test]
    fn every_paas_marker_is_recognised() {
        for marker in PAAS_MARKERS {
            assert!(
                paas_env_predicate(|k| k == marker),
                "{marker} must select 0.0.0.0"
            );
        }
    }

    #[test]
    fn version_line_names_this_target() {
        let line = version_line();
        assert!(line.starts_with("pencarimovie-server "));
        let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
        assert!(line.contains(&target), "{line} must contain {target}");
    }
}
