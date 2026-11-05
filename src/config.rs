//! Constants lifted from backend.php:172-260 so every module uses the same
//! values as the PHP runtime (same defaults, same paths, same strings).

use crate::storage::{env, env_or};

pub fn wp_api_base() -> String {
    env_or(
        "FD_WP_API_BASE",
        "https://pencarimovie.com/wp-json/pencarimovie-server/v1",
    )
}

/// DPI-fallback host: a Cloudflare Worker that reverse-proxies /wp-json back
/// to the origin (backend.php `FD_WP_FALLBACK_HOST`).
pub const WP_FALLBACK_HOST: &str = "telegra.my";

pub fn wp_api_base_fallback() -> String {
    format!("https://{}/wp-json/pencarimovie-server/v1", WP_FALLBACK_HOST)
}

pub fn wp_version_url() -> String {
    format!("{}/version", wp_api_base())
}

/// Host of the REST origin, so a UI can name the endpoint it is actually talking to.
///
/// `wp_api_base()` is env-overridable (`FD_WP_API_BASE`) and a self-hosted origin is
/// a first-class deployment, so the dashboard prints the CONFIGURED host instead of
/// hard-coding `pencarimovie.com`. Empty when the value does not parse as a URL, and
/// the UI then shows the state without a host.
pub fn wp_api_host() -> String {
    url::Url::parse(&wp_api_base())
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_string()))
        .unwrap_or_default()
}

// There is deliberately NO `admin-ajax.php` URL here. The PHP runtime proxied
// the dashboard's `stream_*` AJAX actions; this port reads the same data from
// the plugin's REST namespace (`wp_api_base()`), which is what every route
// already uses.

pub const AUTH_DEFAULT_PASSWORD: &str = "123456";
pub const AUTH_COOKIE: &str = "pm_auth";

/// The version this binary was BUILT from — `build.rs` stamps `git describe
/// --tags --always --dirty` into it, so every artifact is traceable even when
/// nothing was tagged. `PM_BUILD_VERSION` at build time overrides that.
pub fn build_id() -> &'static str {
    option_env!("PM_BUILD_VERSION").unwrap_or("0.0.0-dev")
}

/// The version of the APP SHELL that started us (the Android APK's versionName),
/// passed as `--app-version` / `PM_APP_VERSION`. `/api/version` reports it so the
/// dashboard can show what the user actually installed; without it the page only
/// ever showed the engine version, which never changes between APK releases.
static CLIENT_VERSION: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn set_client_version(v: &str) {
    let v = v.trim().trim_start_matches('v');
    if !v.is_empty() {
        let _ = CLIENT_VERSION.set(v.to_string());
    }
}

pub fn client_version() -> String {
    CLIENT_VERSION.get().cloned().unwrap_or_default()
}

/// `FD_APP_VERSION`: `<appRoot>/.release-tag` (v-prefix stripped) when the
/// install was stamped by the OTA updater, otherwise the version the binary was
/// BUILT from (`git describe`). The old hardcoded "2.7.9" made every native
/// build — whatever commit it came from — report the same number.
pub fn app_version() -> String {
    // The app shell's own versionName wins when it told us (`--app-version`).
    let client = client_version();
    if !client.is_empty() {
        return client;
    }
    // `build_id()` is authoritative: `build.rs` stamps `git describe --tags`
    // into the binary, and a CI build runs on the tagged commit, so a release
    // binary reports exactly its tag.
    //
    // `.release-tag` is deliberately NOT read any more. The only thing that
    // ever wrote it was the retired PHP OTA updater, so a native install that
    // replaced a PHP one still has that file — it made `pms --version` claim
    // `2.7.9` (and, on one machine, `1.0.0`) for a binary built from 2.8.9.
    build_id().to_string()
}

/// The version in `X.Y.Z` form, for the places that REQUIRE a plain version —
/// the addon descriptor's `version`, which Stremio/Nuvio parse and display.
///
/// A `git describe` stamp (`2.8.8-6-ge36d439-dirty`) is not a descriptor
/// version, and the manifest used to hardcode `1.0.0`, so every client showed
/// the addon as the first release forever no matter which build was installed.
fn xyz(raw: &str) -> Option<String> {
    let head = raw.trim().trim_start_matches('v');
    let head = head.split(['-', '+']).next().unwrap_or(head);
    let mut parts: Vec<String> = head
        .split('.')
        .map(|p| p.chars().take_while(|c| c.is_ascii_digit()).collect::<String>())
        .collect();
    // A bare commit sha (`--always` when no tag exists) carries no version.
    if parts.len() < 2 || parts[0].is_empty() {
        return None;
    }
    for p in parts.iter_mut() {
        if p.is_empty() {
            p.push('0');
        }
    }
    while parts.len() < 3 {
        parts.push("0".to_string());
    }
    parts.truncate(3);
    Some(parts.join("."))
}

pub fn semver() -> String {
    xyz(&app_version()).unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}

/// The version a CLIENT should DISPLAY (the addon descriptor's `version`).
///
/// On Android the app shell passes its own `versionName` as `--app-version`
/// (e.g. `1.0.52`) and that is the real product version; the engine's
/// `git describe` (`2.8.8-14-g…`) is an internal detail that never changes
/// between APK releases. Reporting the engine made every app update look like
/// the same addon version on the device.
pub fn descriptor_version() -> String {
    xyz(&client_version()).unwrap_or_else(semver)
}

/// A display version: `X.Y.Z` when the input has one, otherwise the input
/// without a leading `v`. Used so a UI never shows `2.8.9-4-g02e7dea`.
pub fn clean_version(raw: &str) -> String {
    xyz(raw).unwrap_or_else(|| raw.trim().trim_start_matches('v').to_string())
}

/// The port reports its own identity in the user agent; the PHP uses
/// `pencarimovie-server/<version>` and the WordPress logs show it.
pub fn user_agent() -> String {
    // No engine suffix: this string is sent to the origin and shows up in ITS
    // logs and dashboards, so it must read as the product, nothing else.
    format!("pencarimovie-server/{}", app_version())
}

pub fn imdb_aka_api_base() -> String {
    env_or("FD_IMDB_AKA_API_BASE", "https://pencarimovie.com/imdb-api")
}

pub fn debug_from_env() -> bool {
    for k in ["DEBUG", "DEBUG_MODE", "PM_DEBUG"] {
        if let Some(v) = env(k) {
            let s = v.trim().to_ascii_lowercase();
            if s == "1" || s == "true" || s == "on" || s == "yes" {
                return true;
            }
        }
    }
    false
}

pub const MAX_LOG_SIZE: u64 = 5 * 1024 * 1024;
