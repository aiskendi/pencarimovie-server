# PencariMovie Downloader — Engineering Handoff

**Handoff date:** 2026-09-15
**Prepared by:** outgoing engineer
**Repo:** `pencarimovie-downloader` (workspace `c:/Users/ewangtlex/Desktop/pencarimovie`)

---

## 1. 🚀 Project Overview & Current State

### Core Objective

A **local, browser-based Telegram file downloader** that runs on the user's own machine. It boots MadelineProto with a bot session and streams Telegram files directly to the browser via `downloadToBrowser()`. It also exposes a **Stremio/Nuvio addon** (`/manifest.json`, `/stream/*`) and an **Eclipse Music addon** (`/eclipse/*`) so the same local server can serve music and video into those clients.

**Scope guard (from `AGENTS.md`):** this is _not_ a WordPress search rebuild and _not_ a Telegram forward/relay. Search/listing metadata comes from the WordPress plugin; this repo only provides the downloader + addon layer.

### Current Branch

`main` — all work is committed directly to `main`. There is no feature-branch workflow in use.

### Latest Milestone

**ALAC → FLAC on-the-fly conversion is working end-to-end**, plus a set of Eclipse search-quality fixes. Specifically, in this session:

1. **ALAC → FLAC conversion** — a 13 MB ALAC `.m4a` now downloads in ~13s and encodes to an 80 MB FLAC in ~1s (previously ~184s total). Implemented as: web request downloads the raw file with `downloadToCallable(seekable: true)`, then a detached encode-only CLI worker runs FFmpeg.
2. **Eclipse search freshness** — added a `t=<unixtime>` cache-buster and raised the timeout 5s → 15s.
3. **Eclipse `Artist - Title` fallback** — progressive title-only retry chain.
4. **Eclipse caption parsing** — handles the `<short_code> <Title> <Artist>` format.
5. **Eclipse stream URL extension** — rewritten to `.flac` so clients don't skip ALAC tracks.
6. **Amp request retry** — retries once on cancellation instead of falling through to a slow cURL path.
7. **Cross-platform packaging** — `audio-encode-worker.php` added to all 6 packagers; Termux FFmpeg handling added.

---

## 2. 📝 Work in Progress (WIP)

### Active Tasks

**Bundling the audio-encode FFmpeg binary into release tarballs — ✅ COMPLETED.**

The user's last request was:

> "why not ship the tar with audio ffmpeg??? that thing is small."

This is **correct and the right call**. Verified sizes:

| Target                   | Tarball size          | Extracted binary |
| ------------------------ | --------------------- | ---------------- |
| `arm64-linux-gnu`        | 7,219,978 B (~7.2 MB) | ~4.5 MB          |
| `x86_64-linux-gnu`       | 7,234,815 B (~7.2 MB) | ~4.9 MB          |
| `x86_64-w64-mingw32`     | 7,185,263 B (~7.2 MB) | ~4.9 MB          |
| `arm64-apple-macos11`    | 7,027,431 B (~7.0 MB) | ~4.5 MB          |
| `x86_64-apple-macos10.9` | 7,121,762 B (~7.1 MB) | ~4.9 MB          |

All five asset URLs return HTTP `200` from:
`https://github.com/acoustid/ffmpeg-build/releases/download/v8.1.2-1/`

**Implementation completed.** All six packagers now bundle the binary, [`fd_ensure_audio_ffmpeg()`](backend.php:6474) checks the app-root `bin/` first, and the Termux `PATH` bug is fixed. See §4 for the verification results.

### Blockers or Hurdles

**Blocker 1 — Termux/Android FFmpeg is broken (the reason for the bundling request).**

The log shows:

```
15:09:08  installing native Termux ffmpeg package
15:09:08  starting downloadToBrowser {"file_name":"04.Treasure.m4a","mime":"audio/mp4",...}
```

The `pkg install -y ffmpeg` runs, then the code **falls through to `downloadToBrowser`** — meaning [`fd_ensure_audio_ffmpeg()`](backend.php:6474) returned an empty path and the raw ALAC was served instead of a FLAC.

**Root cause identified:** in [`start-termux.sh:156`](start-termux.sh:156) the proot environment sets:

```sh
export PATH="$1/bin:$PATH"
```

where `$1` is `$ROOT_DIR` (the **app directory**), not the Termux prefix. So `PATH` becomes `<appdir>/bin:...` and **does not include** `/data/data/com.termux/files/usr/bin`, where `pkg` and `ffmpeg` live. `pkg install` therefore fails silently and `command -v ffmpeg` finds nothing.

**Blocker 2 — the acoustid ARM64 build cannot run natively in Termux.**

Verified by inspecting the binary: it is **glibc-linked** (`/lib/ld-linux-aarch64.so.1`, `libc.so.6`). Termux uses **bionic libc**. So even if the download succeeded, the binary would not execute without proot.

**This is exactly why bundling is the right fix** — but note the bundled ARM64 binary still needs proot to run. Two options are open (see Next Steps).

### Stashed/Uncommitted Changes

`git status` shows **113 changed files** on `main`, none committed. The relevant ones from this session:

**Modified:**

- [`backend.php`](backend.php) — all the fixes listed in §1
- [`install-termux.sh`](install-termux.sh) — added `ffmpeg` to the `pkg install` list
- [`scripts/package-windows.bat`](scripts/package-windows.bat) — added `audio-encode-worker.php`
- [`scripts/package-unix.sh`](scripts/package-unix.sh) — added `audio-encode-worker.php`
- [`scripts/build-release.bat`](scripts/build-release.bat) — added `audio-encode-worker.php` to all 4 inline packagers
- [`AGENTS.md`](AGENTS.md) — documented all of the above

**Created:**

- [`audio-encode-worker.php`](audio-encode-worker.php) — the detached encode-only CLI worker
- [`HANDOFF.md`](HANDOFF.md) — this document

**Deleted:**

- `audio-convert-worker.php` — the old worker that booted its own MadelineProto (caused the stall)

---

## 3. 🛠️ Environment & Configuration Updates

### New Dependencies

**None.** No new npm/pip/composer packages were added. The FFmpeg binary is a standalone static executable downloaded at runtime (or, after the pending change, bundled).

### Environment Variables

**No new `.env` keys.** The project does not use a `.env` file. Configuration lives in:

- `storage/api_secret.key` — WordPress API secret (written during bot login)
- `storage/api_credentials.json` — cached `api_id`/`api_hash`
- `storage/session.madeline` — MadelineProto session (per bot, under `storage/sessions/<bot_id>/`)
- `storage/bot_pool.json` — bot pool
- `PORT` — optional, overrides the default `8088`

### Database Migrations

**None.** This project has no database. It talks to WordPress (which owns Manticore) over REST.

---

## 4. ✅ Completed Work (this session)

1. **Bundled the audio-encode FFmpeg into every release package.**
   - [`scripts/package-windows.bat`](scripts/package-windows.bat) — copies `storage/bin/ffmpeg.exe` if present, else downloads the `x86_64-w64-mingw32` asset and extracts `ffmpeg.exe` to `bin/`.
   - [`scripts/package-unix.sh`](scripts/package-unix.sh) — selects the asset from `$PACKAGE_TARGET` (`mac-arm64`, `mac-x86_64`, `*aarch64*`, else x86_64-linux-gnu), prefers `storage/bin/ffmpeg`, else downloads via curl/wget.
   - [`scripts/build-release.bat`](scripts/build-release.bat) — all four inline PowerShell packagers (Linux x64, Linux arm64, Mac, standalone server) now bundle FFmpeg. The Mac packager picks the asset per-arch inside the loop; the standalone server uses `arm64-linux-gnu` (runs under proot's glibc rootfs).

2. **Updated [`fd_ensure_audio_ffmpeg()`](backend.php:6474) to check the app-root `bin/` first.**
   - New resolution order: app-root `bin/` → `storage/bin/` → `storage/tunnel/bin/` → (Android) Termux prefix → system `PATH` → runtime download.
   - The Android branch now probes `$PREFIX`, `/data/data/com.termux/files/usr`, and `/data/data/com.pencarimovie.downloader/files/usr` **before** the generic `PATH` probe, so a proot `PATH` exposing a glibc binary cannot shadow the working bionic one.

3. **Fixed the Termux `PATH` bug in all three copies of the proot command.**
   - [`start-termux.sh`](start-termux.sh:155) — the repo launcher. Now passes the Termux prefix as `$7` and exports `PATH="$1/bin:$7/bin:$PATH"` plus `PREFIX="$7"`.
   - [`pencarimovie-termux.sh`](pencarimovie-termux.sh:544) — the **one-file installer served by `curl -fsSL telegra.my/termux | bash`**. This is a _separate_ copy of the proot command and had the same bug; same fix applied.
   - [`NativeRunner.kt`](android/termux-app-fork/app/main/java/com/pencarimovie/downloader/NativeRunner.kt:670) — the Android APK builds its own proot command in Kotlin. Both [`termux-app-fork`](android/termux-app-fork) and [`termux-app-fork-x86_64`](android/termux-app-fork-x86_64) (byte-identical files) now export `PATH="$appDirPath/bin:$prefixDir/bin:$PATH"` and `PREFIX="$prefixDir"` in both the outer shell and the inner `/system/bin/sh -c` block.
   - [`pencarimovie-linux.sh`](pencarimovie-linux.sh:471) runs FrankenPHP **natively** (no proot), so it is unaffected.

4. **Verification performed.**
   - `bin\php.exe -l backend.php` → no syntax errors.
   - `bin\php.exe -l audio-encode-worker.php` → no syntax errors.
   - All five acoustid asset URLs return HTTP `200`.
   - `scripts\package-windows.bat` → built a 61.62 MB ZIP containing `./bin/ffmpeg.exe` (4,874,752 B) and `./audio-encode-worker.php`.

### Remaining follow-up

- **Rebuild the Unix/Mac/standalone packages** with `scripts\build-release.bat` and confirm `bin/ffmpeg` is present in each archive.
- **Device-test on Android** that the bundled `arm64-linux-gnu` binary runs under proot, then consider gating the `pkg install -y ffmpeg` fallback in [`fd_ensure_audio_ffmpeg()`](backend.php:6524).

---

## 5. 🔍 Key Files & Architecture Notes

### Files Modified/Created

| File                                                                                                   | Role                                                                                                                       |
| ------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| [`backend.php`](backend.php)                                                                           | The entire backend (12,753 lines). All routes, MadelineProto boot, Eclipse/Stremio addons, FFmpeg resolution, HTTP client. |
| [`audio-encode-worker.php`](audio-encode-worker.php)                                                   | **New.** Detached CLI worker that runs FFmpeg on an already-downloaded `.m4a`. Does **not** boot MadelineProto.            |
| [`install-termux.sh`](install-termux.sh)                                                               | Termux dependency installer. Now installs `ffmpeg`.                                                                        |
| [`start-termux.sh`](start-termux.sh)                                                                   | Termux launcher. **`PATH` bug fixed** (line 151).                                                                          |
| [`NativeRunner.kt`](android/termux-app-fork/app/main/java/com/pencarimovie/downloader/NativeRunner.kt) | Android APK runner. **`PATH` bug fixed** (line 670). Byte-identical in both APK variants.                                  |
| [`scripts/package-windows.bat`](scripts/package-windows.bat)                                           | Windows ZIP packager.                                                                                                      |
| [`scripts/package-unix.sh`](scripts/package-unix.sh)                                                   | Linux/macOS tarball packager.                                                                                              |
| [`scripts/build-release.bat`](scripts/build-release.bat)                                               | Orchestrator with 4 inline PowerShell packagers.                                                                           |
| [`AGENTS.md`](AGENTS.md)                                                                               | Project memory. **Read this first** — it documents every architectural decision and gotcha.                                |

### Key Functions in `backend.php`

| Function                                              | Line | Purpose                                                                    |
| ----------------------------------------------------- | ---- | -------------------------------------------------------------------------- |
| [`fd_amphp_http_request()`](backend.php:1250)         | 1250 | Amp HTTP client wrapper with anycast DNS.                                  |
| [`fd_http_get_contents()`](backend.php:1308)          | 1308 | HTTP fetch: Amp first, cURL fallback. **Contains the retry logic.**        |
| [`fd_ensure_audio_ffmpeg()`](backend.php:6474)        | 6474 | Resolves the audio-encode FFmpeg binary. **Checks app-root `bin/` first.** |
| [`fd_audio_convert_lock_acquire()`](backend.php:6735) | 6735 | Exclusive `flock` so only one worker converts a short_code.                |
| [`fd_spawn_audio_encode_worker()`](backend.php:6761)  | 6761 | Spawns the detached encode worker.                                         |
| [`fd_eclipse_parse_audio_title()`](backend.php:7473)  | 7473 | Caption/filename → artist + title.                                         |
| [`fd_eclipse_format_track()`](backend.php:7738)       | 7738 | Builds the Eclipse track object; takes the search query for splitting.     |

### Architectural Decisions

**1. The web request downloads; the worker only encodes.**

This is the single most important decision in the ALAC path. The original design had the worker boot its own MadelineProto and download the file. That **stalled** — two IPC clients competing for the same session made `downloadToCallable` hang and the raw file stayed at 0 bytes.

The fix: the web request (which already holds a working `$madeline`) does the download, then hands the finished file to a worker that runs **only** FFmpeg. **Do not let the worker boot MadelineProto.**

**2. `downloadToCallable(seekable: true)` instead of `downloadToFile`.**

Measured: `downloadToCallable` with parallel 1 MB chunks runs at ~1.6 MB/s; `downloadToFile` is sequential at ~97 KB/s. This cut a 13 MB download from 139s to 13s.

**3. FFmpeg reads a file, never a Madeline stdin pipe.**

Piping Madeline chunks into FFmpeg stdin **deadlocks on Windows** (`proc_open(): Command conversion failed` under Madeline's exception handler) and cannot seek for moov-at-end files.

**4. No `2>&1` with `bypass_shell => true`.**

With no shell involved, `2>&1` is passed to FFmpeg as a **literal output filename** (`Error opening output file 2>&1`). stderr is captured via the descriptor spec instead.

**5. Exclusive `flock` per short_code.**

Without it, two concurrent `/api/download` requests spawn two workers; one deletes the raw `.m4a` after publishing, the other overwrites the good state with a failure.

**6. The known-artist list is declared as a `static` in BOTH parser functions.**

[`fd_eclipse_parse_audio_title()`](backend.php:7473) and [`fd_eclipse_format_track()`](backend.php:7738) each need their own copy. Using it in the latter without declaring it there throws `in_array(): Argument #2 ($haystack) must be of type array, null given` and the **entire** `/eclipse/search` response comes back empty.

**7. Amp retry before cURL fallback.**

The Amp client is fast when warm (~155ms) and a retry is nearly free (connection already pooled). The cURL fallback had a 5s connect timeout that also failed on cold TLS handshakes. Retrying Amp once recovers the request; the cURL `CURLOPT_CONNECTTIMEOUT` is now `max(10, timeout*0.6)`.

**8. Termux must not use the acoustid build.**

It is glibc-linked and cannot run under Termux's bionic libc. The code installs the native Termux `ffmpeg` package instead — **but this is currently broken** by the `PATH` bug in `start-termux.sh`.

---

## Quick Reference

**Start / stop (Windows):**

```bat
start.bat
stop.bat
restart.bat
```

**Start / stop (Linux / Termux):**

```bash
./start.sh
./stop.sh
./restart.sh
```

**Local URL:** `http://127.0.0.1:8088`

**Syntax check:**

```bat
bin\php.exe -l backend.php
bin\php.exe -l audio-encode-worker.php
```

**Build Windows package:**

```bat
scripts\package-windows.bat
```

**Build all packages:**

```bat
scripts\build-release.bat
```

**Key log:** `storage/debug.log` (enable via `storage/debug_mode.txt`)

**Read `AGENTS.md` before making changes** — it is the authoritative record of every gotcha discovered so far.
