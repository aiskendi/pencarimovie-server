//! The Windows notification-area icon, owned by the server process itself.
//!
//! `pencarimovie-server.exe --tray` shows the icon and serves behind it. No
//! `tray.ps1`, no PowerShell, no `pencarimovie.ico` beside the binary: the icon,
//! the menu and every action are in here, so a folder holding nothing but the
//! exe is still a working tray install.
//!
//! Why this is native rather than a script (the failure it fixes):
//!
//! * Windows keys the per-app tray settings in
//!   `HKCU\Control Panel\NotifyIconSettings` by the **hosting executable**. The
//!   old tray ran inside `powershell.exe`, so it shared that one entry with every
//!   other PowerShell-hosted tray (it was `9Router` on the machine that reported
//!   this), could only ever appear in the "hidden icons" flyout, and could not be
//!   promoted independently of them.
//! * `start-tray.bat` was the only thing that ever created an icon, so
//!   double-clicking `pencarimovie-server.exe` — the obvious thing to do —
//!   produced a console and no tray at all.
//! * `tray.ps1` set `$ErrorActionPreference = 'SilentlyContinue'`, so a WinForms
//!   failure left a PowerShell process alive with no icon and no message.
//!
//! Only `user32` and `shell32` are called, declared below as `extern "system"`,
//! so this adds no crate dependency and the release stays one self-contained
//! binary (same rule as the embedded dashboard in `assets.rs`).
//!
//! Everything here is `#[cfg(windows)]`; Linux/macOS keep their shipped
//! `start-tray.sh` / LaunchAgent paths.

use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicIsize, AtomicU16, Ordering};

/// `CREATE_NO_WINDOW`: a helper (`reg`, `clip`) must never flash a console.
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

// ── Win32 types ─────────────────────────────────────────────────────────────

type Wnd = isize;
type Menu = isize;
type HIcon = isize;

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Msg {
    hwnd: Wnd,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    pt: Point,
    lprivate: u32,
    /// `sizeof(MSG)` grew `lPrivate` (and its tail padding) in the modern SDK;
    /// the extra field keeps `GetMessageW` inside our buffer.
    _pad: u32,
}

#[repr(C)]
struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8; 8],
}

/// `NOTIFYICONDATAW`, Windows Vista layout (`NOTIFYICON_VERSION_4`).
#[repr(C)]
struct NotifyIconDataW {
    cb_size: u32,
    hwnd: Wnd,
    id: u32,
    flags: u32,
    callback_message: u32,
    hicon: HIcon,
    tip: [u16; 128],
    state: u32,
    state_mask: u32,
    info: [u16; 256],
    version: u32,
    info_title: [u16; 64],
    info_flags: u32,
    guid: Guid,
    balloon_icon: HIcon,
}

#[repr(C)]
struct WndClassW {
    style: u32,
    wnd_proc: Option<unsafe extern "system" fn(Wnd, u32, usize, isize) -> isize>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: isize,
    icon: HIcon,
    cursor: isize,
    background: isize,
    menu_name: *const u16,
    class_name: *const u16,
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassW(class: *const WndClassW) -> u16;
    fn CreateWindowExW(
        ex_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: isize,
        menu: isize,
        instance: isize,
        param: *mut core::ffi::c_void,
    ) -> Wnd;
    fn DefWindowProcW(hwnd: Wnd, msg: u32, wparam: usize, lparam: isize) -> isize;
    fn DestroyWindow(hwnd: Wnd) -> i32;
    fn GetMessageW(msg: *mut Msg, hwnd: Wnd, min: u32, max: u32) -> i32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn PostQuitMessage(code: i32);
    fn PostMessageW(hwnd: Wnd, msg: u32, wparam: usize, lparam: isize) -> i32;
    fn RegisterWindowMessageW(name: *const u16) -> u32;
    fn LoadImageW(
        instance: isize,
        name: *const u16,
        ty: u32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> isize;
    fn LoadIconW(instance: isize, name: *const u16) -> HIcon;
    fn DestroyIcon(icon: HIcon) -> i32;
    fn CreatePopupMenu() -> Menu;
    fn AppendMenuW(menu: Menu, flags: u32, id: usize, item: *const u16) -> i32;
    fn TrackPopupMenu(
        menu: Menu,
        flags: u32,
        x: i32,
        y: i32,
        reserved: i32,
        hwnd: Wnd,
        rect: *const core::ffi::c_void,
    ) -> i32;
    fn DestroyMenu(menu: Menu) -> i32;
    fn SetForegroundWindow(hwnd: Wnd) -> i32;
    fn GetCursorPos(pt: *mut Point) -> i32;
}

#[link(name = "shell32")]
unsafe extern "system" {
    fn Shell_NotifyIconW(msg: u32, data: *const NotifyIconDataW) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> isize;
    /// Detach from the attached console. Unlike `ShowWindow(SW_HIDE)` this
    /// releases ownership, so the conhost window actually CLOSES.
    fn FreeConsole() -> i32;
    fn GetStdHandle(handle: u32) -> isize;
    fn SetStdHandle(handle: u32, val: isize) -> i32;
    fn CloseHandle(handle: isize) -> i32;
}

// ── constants ───────────────────────────────────────────────────────────────

const WM_NULL: u32 = 0x0000;
const WM_DESTROY: u32 = 0x0002;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_CONTEXTMENU: u32 = 0x007B;

/// `WM_APP + 1`: the tray callback. Every notification arrives here.
const TRAY_CALLBACK: u32 = 0x8000 + 1;

const NIM_ADD: u32 = 0;
const NIM_DELETE: u32 = 2;
const NIM_SETVERSION: u32 = 4;

const NIF_MESSAGE: u32 = 0x01;
const NIF_ICON: u32 = 0x02;
const NIF_TIP: u32 = 0x04;
const NIF_SHOWTIP: u32 = 0x80;

const NOTIFYICON_VERSION_4: u32 = 4;

const MF_STRING: u32 = 0x0000;
const MF_SEPARATOR: u32 = 0x0800;
const MF_CHECKED: u32 = 0x0008;

const TPM_RIGHTBUTTON: u32 = 0x0002;
const TPM_NONOTIFY: u32 = 0x0080;
const TPM_RETURNCMD: u32 = 0x0100;

const IMAGE_ICON: u32 = 1;
const LR_LOADFROMFILE: u32 = 0x0010;
const LR_DEFAULTSIZE: u32 = 0x0040;
/// `MAKEINTRESOURCE(IDI_APPLICATION)`: the fallback when no `.ico` ships.
const IDI_APPLICATION: usize = 32512;

const MENU_OPEN: usize = 1;
const MENU_ADDON: usize = 2;
const MENU_COPY: usize = 3;
const MENU_LOG: usize = 4;
const MENU_AUTOSTART: usize = 5;
const MENU_HIDE: usize = 6;
const MENU_STOP: usize = 7;

const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_NAME: &str = "PencariMovie Server";

/// The HWND the tray thread created, or 0. Kept as an `isize` so the statics
/// stay `Sync` without a new dependency.
static WINDOW: AtomicIsize = AtomicIsize::new(0);
static ICON: AtomicIsize = AtomicIsize::new(0);
static PORT: AtomicU16 = AtomicU16::new(8088);
/// The bind address this process actually serves on, so the login entry it
/// writes reproduces it instead of silently falling back to loopback.
static HOST: OnceLock<String> = OnceLock::new();
/// `RegisterWindowMessageW("TaskbarCreated")`: Explorer restarted, re-add.
static TASKBAR_CREATED: OnceLock<u32> = OnceLock::new();

fn port() -> u16 {
    PORT.load(Ordering::Relaxed)
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

fn copy_into(dst: &mut [u16], text: &str) {
    for (slot, ch) in dst.iter_mut().zip(text.encode_utf16()) {
        *slot = ch;
    }
}

fn exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()))
}

// ── entry point ─────────────────────────────────────────────────────────────

/// Whether this process should show a tray icon.
///
/// `--tray` forces it, `--no-tray` (or `PM_NO_TRAY`) forbids it, and a
/// double-clicked console exe gets one by default — a human is looking at it.
pub fn wanted(_stdout_is_terminal: bool) -> bool {
    if std::env::args().any(|a| a == "--no-tray") {
        return false;
    }
    if std::env::var_os("PM_NO_TRAY").is_some() {
        return false;
    }
    std::env::args().any(|a| a == "--tray") || std::env::var_os("PM_TRAY").is_some()
}

/// Detach this process from its console, closing the window.
///
/// Called whenever a tray icon is created: with an icon to right-click, the black
/// window has nothing to offer, and a double-clicked exe used to leave one sitting
/// there for the life of the server.
///
/// `FreeConsole()` rather than `ShowWindow(SW_HIDE)`: hiding leaves the process
/// still OWNING a console, so conhost keeps the window (and the `cmd` that started
/// us) alive — measured as a window that was invisible but never went away. This
/// releases it, so the window closes and the launcher's prompt returns.
///
/// The log does not depend on that window: while the tray runs,
/// [`crate::log::stderr_log`] also appends to `storage/server.log`, so "Open log"
/// works with no console and no shell redirection.
///
/// This deliberately does NOT fork a second copy: a fork inside `cmd`/PowerShell
/// kept the caller waiting on the inherited console and re-bound the port in the
/// parent — that was the "hang" an earlier version of this produced.
pub fn hide_console() {
    // Stop stderr first: after `FreeConsole` the old handle fails, and a failed
    // `eprintln!` panics (which would kill this thread).
    crate::log::silence_stderr();
    unsafe {
        // Returns 0 when there was no console to begin with (`start ""`, the Run
        // key, a service): nothing to do and nothing to break.
        FreeConsole();
        // Also close and detach inherited standard pipe handles (-10, -11, -12).
        // If launched from a terminal/batch/Node wrapper, the parent launcher
        // waits for pipe EOF before considering the command finished.
        for id in [0xFFFF_FFF6u32, 0xFFFF_FFF5u32, 0xFFFF_FFF4u32] {
            let h = GetStdHandle(id);
            SetStdHandle(id, 0);
            if h != 0 && h != -1 {
                CloseHandle(h);
            }
        }
    }
}

/// Start the tray thread. Returns immediately; the server keeps serving.
pub fn spawn(server_port: u16, listen_host: &str) {
    PORT.store(server_port, Ordering::Relaxed);
    let _ = HOST.set(listen_host.to_string());
    hide_console();
    // No console to print to from here on: keep `storage/server.log` current so
    // the tray's "Open log" item has something to open.
    crate::log::mirror_stderr_to_server_log();
    std::thread::Builder::new()
        .name("tray".to_string())
        .spawn(|| unsafe { run_message_loop() })
        .map(|_| {
            crate::log::stderr_log(
                "tray icon requested",
                Some(&serde_json::json!({ "port": server_port })),
            );
        })
        .unwrap_or_else(|e| {
            crate::log::stderr_log(
                "tray thread failed to start",
                Some(&serde_json::json!({ "error": e.to_string() })),
            );
        });
}

unsafe fn run_message_loop() {
    let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    let class_name = wide("PencariMovieServerTray");
    let icon = load_icon();
    ICON.store(icon, Ordering::Relaxed);

    let class = WndClassW {
        style: 0,
        wnd_proc: Some(wnd_proc),
        cls_extra: 0,
        wnd_extra: 0,
        instance,
        icon,
        cursor: 0,
        background: 0,
        menu_name: std::ptr::null(),
        class_name: class_name.as_ptr(),
    };
    // A duplicate registration (two processes) is harmless.
    unsafe {
        RegisterClassW(&class);
    }

    let title = wide("PencariMovie Server");
    // A normal (never-shown) top-level window, NOT `HWND_MESSAGE`: message-only
    // windows do not receive the broadcast `TaskbarCreated`, so the icon would
    // never come back after an Explorer restart.
    let window = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            instance,
            std::ptr::null_mut(),
        )
    };
    if window == 0 {
        crate::log::stderr_log("tray window could not be created", None);
        return;
    }
    WINDOW.store(window, Ordering::Relaxed);
    TASKBAR_CREATED.get_or_init(|| unsafe { RegisterWindowMessageW(wide("TaskbarCreated").as_ptr()) });

    add_icon(window);

    let mut msg: Msg = unsafe { std::mem::zeroed() };
    loop {
        let got = unsafe { GetMessageW(&mut msg, 0, 0, 0) };
        if got <= 0 {
            break;
        }
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
    // Explorer-less shutdown (logoff): make sure the icon is not left behind.
    remove_icon(window);
    let icon = ICON.swap(0, Ordering::Relaxed);
    if icon != 0 {
        unsafe { DestroyIcon(icon) };
    }
}

/// The PencariMovie logo, compiled in. `tray.ico` is a tracked product asset
/// (`release.yml` copies it into every artifact), so no build can be missing it.
const LOGO_ICO: &[u8] = include_bytes!("../tray.ico");

/// The tray icon. A Win32 icon handle has to come from a file or a resource and
/// this crate has no resource compiler, so the embedded logo is written to
/// `storage/tray/pencarimovie.ico` when no `.ico` sits beside the exe.
///
/// Falling straight back to `IDI_APPLICATION` (the old behaviour) is what makes
/// the icon look blank/generic — reported as "the tray doesn't have an icon".
fn load_icon() -> HIcon {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Some(dir) = exe_dir() {
        candidates.push(dir.join("pencarimovie.ico"));
        candidates.push(dir.join("tray.ico"));
    }
    if !candidates.iter().any(|p| p.is_file()) {
        // Only materialise the embedded copy when the package has none, so a
        // normal install never writes an icon into storage/.
        candidates.extend(cache_icon());
    }
    for ico in candidates {
        if !ico.is_file() {
            continue;
        }
        let path = wide(&ico.to_string_lossy());
        let handle = unsafe {
            LoadImageW(
                0,
                path.as_ptr(),
                IMAGE_ICON,
                0,
                0,
                LR_LOADFROMFILE | LR_DEFAULTSIZE,
            )
        };
        if handle != 0 {
            return handle;
        }
    }
    crate::log::stderr_log(
        "tray icon file could not be loaded, using the stock icon",
        Some(&serde_json::json!({ "embedded_bytes": LOGO_ICO.len() })),
    );
    unsafe { LoadIconW(0, IDI_APPLICATION as *const u16) }
}

/// Write the embedded logo to `storage/tray/pencarimovie.ico`, once.
fn cache_icon() -> Option<PathBuf> {
    let dir = crate::storage::storage_dir().join("tray");
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("pencarimovie.ico");
    if std::fs::read(&path).ok().as_deref() != Some(LOGO_ICO) {
        std::fs::write(&path, LOGO_ICO).ok()?;
    }
    Some(path)
}

fn add_icon(window: Wnd) {
    let mut data: NotifyIconDataW = unsafe { std::mem::zeroed() };
    data.cb_size = std::mem::size_of::<NotifyIconDataW>() as u32;
    data.hwnd = window;
    data.id = 1;
    data.flags = NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP;
    data.callback_message = TRAY_CALLBACK;
    data.hicon = ICON.load(Ordering::Relaxed);
    copy_into(
        &mut data.tip,
        &format!("PencariMovie Server - port {}", port()),
    );
    let added = unsafe { Shell_NotifyIconW(NIM_ADD, &data) } != 0;
    if !added {
        crate::log::stderr_log("tray icon was rejected by the shell", None);
        return;
    }
    // Version 4 makes Windows treat this as a real per-app tray icon: the
    // tooltip follows `NIF_SHOWTIP` and the menu is delivered as WM_CONTEXTMENU.
    data.version = NOTIFYICON_VERSION_4;
    unsafe { Shell_NotifyIconW(NIM_SETVERSION, &data) };
    crate::log::stderr_log(
        "tray icon added",
        Some(&serde_json::json!({ "port": port() })),
    );
}

fn remove_icon(window: Wnd) {
    let mut data: NotifyIconDataW = unsafe { std::mem::zeroed() };
    data.cb_size = std::mem::size_of::<NotifyIconDataW>() as u32;
    data.hwnd = window;
    data.id = 1;
    unsafe { Shell_NotifyIconW(NIM_DELETE, &data) };
}

unsafe extern "system" fn wnd_proc(hwnd: Wnd, msg: u32, wparam: usize, lparam: isize) -> isize {
    if msg == TRAY_CALLBACK {
        // Version 4 packs the mouse event in the LOWORD of lParam (version 3
        // puts it in lParam directly), so masking works for both.
        let event = (lparam as usize) & 0xffff;
        match event as u32 {
            WM_RBUTTONUP | WM_CONTEXTMENU => show_menu(hwnd),
            WM_LBUTTONUP | WM_LBUTTONDBLCLK => open_page("/"),
            _ => {}
        }
        return 0;
    }
    if let Some(&created) = TASKBAR_CREATED.get()
        && msg == created
    {
        // Explorer restarted and dropped every icon.
        add_icon(hwnd);
        return 0;
    }
    if msg == WM_DESTROY {
        unsafe { PostQuitMessage(0) };
        return 0;
    }
    let _ = wparam;
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn show_menu(hwnd: Wnd) {
    let menu = unsafe { CreatePopupMenu() };
    if menu == 0 {
        return;
    }
    append(menu, MF_STRING, MENU_OPEN, "Open dashboard");
    append(menu, MF_STRING, MENU_ADDON, "Open add-on page");
    append(menu, MF_STRING, MENU_COPY, "Copy add-on URL");
    separator(menu);
    append(menu, MF_STRING, MENU_LOG, "Open log");
    let autostart = MF_STRING
        | if autostart_on() {
            MF_CHECKED
        } else {
            0
        };
    append(menu, autostart, MENU_AUTOSTART, "Run at login");
    separator(menu);
    append(menu, MF_STRING, MENU_HIDE, "Hide tray icon (server keeps running)");
    append(menu, MF_STRING, MENU_STOP, "Stop server");

    let mut point = Point { x: 0, y: 0 };
    unsafe {
        GetCursorPos(&mut point);
        // Without this the menu does not close when the user clicks elsewhere.
        SetForegroundWindow(hwnd);
    }
    let choice = unsafe {
        TrackPopupMenu(
            menu,
            TPM_RIGHTBUTTON | TPM_RETURNCMD | TPM_NONOTIFY,
            point.x,
            point.y,
            0,
            hwnd,
            std::ptr::null(),
        )
    };
    unsafe {
        // The documented dance that dismisses the menu on the next click.
        PostMessageW(hwnd, WM_NULL, 0, 0);
        DestroyMenu(menu);
    }

    match choice as usize {
        MENU_OPEN => open_page("/"),
        MENU_ADDON => open_page("/#addon"),
        MENU_COPY => copy_addon_url(),
        MENU_LOG => open_log(),
        MENU_AUTOSTART => toggle_autostart(),
        MENU_HIDE => {
            // The server is this process, so dropping the icon is enough; the
            // tray thread parks on GetMessage until the window is destroyed.
            remove_icon(hwnd);
            unsafe { DestroyWindow(hwnd) };
            WINDOW.store(0, Ordering::Relaxed);
        }
        MENU_STOP => {
            remove_icon(hwnd);
            crate::request_shutdown();
        }
        _ => {}
    }
}

fn append(menu: Menu, flags: u32, id: usize, text: &str) {
    let label = wide(text);
    unsafe { AppendMenuW(menu, flags, id, label.as_ptr()) };
}

fn separator(menu: Menu) {
    unsafe { AppendMenuW(menu, MF_SEPARATOR, 0, std::ptr::null()) };
}

// ── menu actions ────────────────────────────────────────────────────────────

fn open_page(path: &str) {
    crate::open_browser(&format!("http://127.0.0.1:{}{}", port(), path));
}

fn copy_addon_url() {
    // Same RFC1918-only LAN address the dashboard advertises, so a TV box on the
    // same wifi can install straight from what is on the clipboard.
    let url = format!(
        "http://{}:{}/manifest.json",
        crate::routes::lan_ip(),
        port()
    );
    // `clip.exe` ships with Windows; a Win32 clipboard round-trip would need
    // kernel32 global memory for no user-visible gain.
    let mut child = match Command::new("clip")
        .stdin(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return,
    };
    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        let _ = stdin.write_all(url.as_bytes());
    }
    let _ = child.wait();
    crate::log::stderr_log("add-on URL copied", Some(&serde_json::json!({ "url": url })));
}

fn open_log() {
    let server = crate::log::server_log_path();
    if server.is_file() {
        let _ = Command::new("notepad").arg(&server).spawn();
        return;
    }
    let debug = crate::log::debug_log_path();
    if debug.is_file() {
        let _ = Command::new("notepad").arg(&debug).spawn();
        return;
    }
    // Nothing logged yet: show the folder, which is what the user wants next.
    let _ = Command::new("explorer")
        .arg(crate::storage::storage_dir())
        .spawn();
}

fn autostart_on() -> bool {
    Command::new("reg")
        .args(["query", RUN_KEY, "/v", RUN_NAME])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Toggle `Run at login`. The entry points at THIS exe with `--tray`, so the
/// login start needs no script either.
fn toggle_autostart() {
    if autostart_on() {
        let _ = Command::new("reg")
            .args(["delete", RUN_KEY, "/v", RUN_NAME, "/f"])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        return;
    }
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    // Carry the host: `--host 0.0.0.0` is what makes the LAN/TV URL reachable,
    // and an entry without it came back on loopback after every reboot.
    let host = HOST
        .get()
        .map(|h| format!(" --host {h}"))
        .unwrap_or_default();
    let command = format!("\"{}\" --tray --port {}{}", exe.display(), port(), host);
    let _ = Command::new("reg")
        .args([
            "add",
            RUN_KEY,
            "/v",
            RUN_NAME,
            "/t",
            "REG_SZ",
            "/d",
            &command,
            "/f",
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .output();
}
