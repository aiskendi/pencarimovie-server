//! Linux tray icon, started by the server itself.
//!
//! `pencarimovie-server --tray` writes the tray script (compiled into the binary
//! from [`desktop/linux/tray.sh`](../desktop/linux/tray.sh)) into
//! `storage/tray/pm-tray.sh` and runs it. There is nothing to ship beside the
//! executable and nothing to keep in sync: the packaged copy and the embedded
//! copy are the same file, one source of truth.
//!
//! The only external piece is the icon renderer, `yad --notification` (GTK).
//! Drawing an AppIndicator/StatusNotifierItem icon without a GUI toolkit would
//! mean shipping a D-Bus + GTK client, which is a dependency this project does
//! not want; `yad` is a package install (`apt install yad`) and, when it is
//! absent, the server still runs and prints the dashboard URL.
//!
//! macOS has no menu-bar support here: an `NSStatusBar` item needs AppKit, i.e.
//! a compiled `.app` bundle. The LaunchAgent (`desktop/macos/autostart-on.sh`)
//! covers "always running", so `--tray` is accepted and logged rather than
//! silently doing nothing on a platform where it cannot work.

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// The tray script, embedded from the tracked Linux launcher.
const SCRIPT: &str = include_str!("../desktop/linux/tray.sh");

/// Same rules as the Windows tray: `--tray` forces it, `--no-tray` / `PM_NO_TRAY`
/// forbid it, and a human-launched console gets one by default.
pub fn wanted(_stdout_is_terminal: bool) -> bool {
    if std::env::args().any(|a| a == "--no-tray") {
        return false;
    }
    if std::env::var_os("PM_NO_TRAY").is_some() {
        return false;
    }
    std::env::args().any(|a| a == "--tray") || std::env::var_os("PM_TRAY").is_some()
}

pub fn spawn(server_port: u16, _listen_host: &str) {
    if cfg!(target_os = "macos") {
        crate::log::stderr_log(
            "no menu-bar icon on macOS (the LaunchAgent covers login start)",
            None,
        );
        return;
    }

    let Some(script) = write_script() else {
        crate::log::stderr_log("tray script could not be written", None);
        return;
    };
    let exe = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("pencarimovie-server"));
    let icon = exe
        .parent()
        .map(|d| d.join("pencarimovie.png"))
        .unwrap_or_else(|| PathBuf::from("pencarimovie.png"));

    let spawned = Command::new("sh")
        .arg(&script)
        .env("PORT", server_port.to_string())
        .env("PM_TRAY_EXE", &exe)
        .env("PM_TRAY_ICON", &icon)
        .env(
            "PM_TRAY_LOG",
            crate::storage::storage_dir().join("server.log"),
        )
        // "Stop server" signals this pid: the tray is a child of the server it
        // controls, so nothing has to be discovered by name.
        .env("PM_TRAY_PID", std::process::id().to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();

    match spawned {
        Ok(child) => crate::log::stderr_log(
            "tray launched",
            Some(&serde_json::json!({
                "port": server_port,
                "script": script.display().to_string(),
                "pid": child.id(),
            })),
        ),
        Err(e) => crate::log::stderr_log(
            "tray could not be launched",
            Some(&serde_json::json!({ "error": e.to_string() })),
        ),
    }
}

/// Write the embedded script to `storage/tray/pm-tray.sh` (0755) when it is not
/// already exactly the embedded bytes.
fn write_script() -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt;

    let dir = crate::storage::storage_dir().join("tray");
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("pm-tray.sh");
    if std::fs::read(&path).ok().as_deref() != Some(SCRIPT.as_bytes()) {
        std::fs::write(&path, SCRIPT).ok()?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).ok()?;
    }
    Some(path)
}
