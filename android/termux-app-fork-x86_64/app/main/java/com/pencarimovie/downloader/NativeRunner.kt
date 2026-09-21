package com.pencarimovie.downloader

import android.os.Build
import android.util.Log
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import java.io.BufferedReader
import java.io.File
import java.io.InputStreamReader
import java.net.HttpURLConnection
import java.net.URL

/**
 * Runs the PencariMovie Server on Android via proot + FrankenPHP.
 *
 * Strategy:
 *   1. Download the Linux release tarball (contains frankenphp + vendor
 *      dependencies).
 *   2. Run it via `proot --link2symlink -0` to allow glibc binaries to work
 *      on Android/bionic.
 *
 * The bootstrap (installed by [TermuxInstaller]) provides bash, coreutils,
 * proot, wget, and essential utilities for first-time setup — all compiled
 * natively for Android with TERMUX_APP__PACKAGE_NAME="com.pencarimovie.downloader".
 * PHP is NOT included in the bootstrap; runtime PHP comes from the bundled
 * FrankenPHP binary in the Linux release tarball.
 */
class NativeRunner(
    /** Bootstrap prefix directory (set by [TermuxInstaller.PREFIX_DIR_PATH]). */
    private val prefixDir: String = TermuxInstaller.PREFIX_DIR_PATH,
    /** Server host (default: 0.0.0.0). */
    private val host: String = "0.0.0.0",
    /** Server default base port (default: 8088). */
    private val basePort: Int = 8088
) {
    companion object {
        private const val TAG = "NativeRunner"
        private const val REPO = "aiskendi/pencarimovie-server"
        private const val APP_DIR_NAME = "pencarimovie-server"
        private const val RELEASE_TAG_FILE = ".release-tag"

        /** Marker file indicating first-run setup was attempted. */
        private const val SETUP_MARKER = ".setup_done"
        private const val EXTENDED_TIMEOUT_MS = 300_000L // 5 min for setup
        private val TAG_PATTERN = Regex("""^v\d""")
    }

    /** Progress callback — called for each stdout/stderr line from the process. */
    var onProgress: ((String) -> Unit)? = null

    /**
     * Process death callback — called when the server process dies unexpectedly
     * (i.e., while [running] is still true). This is set by ServerService before
     * [start] is called, and should dispatch to the main thread since it is
     * invoked from the stdout reader daemon thread.
     */
    var onProcessDied: (() -> Unit)? = null

    private var process: Process? = null
    private var stdoutReader: Thread? = null
    private var running = false
    private var activePort: Int = basePort

    val isRunning: Boolean get() = running
    val port: Int get() = activePort

    /** Path to bash binary inside the bootstrap prefix. */
    private val shellBin: String get() = "$prefixDir/bin/bash"

    /** Path to proot binary inside the bootstrap prefix. */
    private val prootBin: String get() = "$prefixDir/bin/proot"

    /** The download directory for first-time setup. */
    private val downloadDir: String get() = "$prefixDir/root"

    /** App directory where the PencariMovie code is stored. */
    private val appDir: String get() = "$downloadDir/$APP_DIR_NAME"

    /** Marker file path. */
    private val setupMarkerFile: String get() = "$prefixDir/$SETUP_MARKER"

    /**
     * Start the server.
     *
     * On every start:
     *   1. Check GitHub `releases/latest` against `$appDir/.release-tag`
     *   2. If a newer tag exists (or the app is missing), download the
     *      linux tarball and extract over the app dir, skipping `storage/`
     *   3. Start with proot + frankenphp
     *
     * Offline / GitHub failure keeps the installed copy.
     *
     * @param progressCallback optional callback invoked for each output line
     * @return the port on success
     * @throws RuntimeException on failure
     */
    /**
     * Probes ports sequentially starting from [startPort] to find the first free, bindable port.
     */
    private fun findAvailablePort(startPort: Int = basePort, maxAttempts: Int = 20): Int {
        for (candidate in startPort until (startPort + maxAttempts)) {
            var ss: java.net.ServerSocket? = null
            try {
                ss = java.net.ServerSocket(candidate)
                ss.reuseAddress = true
                return candidate
            } catch (_: Exception) {
                Log.d(TAG, "Port $candidate is busy, trying next port...")
            } finally {
                try { ss?.close() } catch (_: Exception) {}
            }
        }
        return startPort
    }

    /**
     * Returns true if something is accepting TCP connections on [port] on localhost.
     * Used as a reliable startup indicator for the server process.
     */
    private fun isPortListening(port: Int): Boolean {
        return try {
            java.net.Socket().use { socket ->
                socket.connect(java.net.InetSocketAddress("127.0.0.1", port), 300)
                true
            }
        } catch (_: Exception) {
            false
        }
    }

    suspend fun start(
        lanIp: String? = null,
        progressCallback: ((String) -> Unit)? = null
    ): Int = withContext(Dispatchers.IO) {
        onProgress = progressCallback
        emitBanner()

        // Pre-cleanup: kill any stale processes from previous runs belonging to our app
        try {
            killAllServerProcesses()
            Log.i(TAG, "Pre-cleanup completed")
        } catch (e: Exception) {
            Log.w(TAG, "Pre-cleanup error (non-fatal): ${e.message}")
        }

        // Detect available port (handles collisions if another app or external Termux is using 8088)
        activePort = findAvailablePort(basePort)
        if (activePort != basePort) {
            Log.w(TAG, "Default port $basePort in use by another app. Selected alternative free port: $activePort")
            emitProgress("[server] Port $basePort is in use. Using port $activePort...")
        } else {
            Log.i(TAG, "Using port $activePort")
        }

        // Ensure directories exist
        File(downloadDir).mkdirs()
        File("$prefixDir/tmp").mkdirs()

        // Migration: If the old "pencarimovie-downloader" folder exists from a previous version,
        // migrate its storage directory (to keep bot session/keys) and remove the old directory.
        val legacyDir = File("$downloadDir/pencarimovie-downloader")
        if (legacyDir.isDirectory && legacyDir.absolutePath != File(appDir).absolutePath) {
            try {
                val legacyStorage = File(legacyDir, "storage")
                val targetStorage = File(appDir, "storage")
                if (legacyStorage.isDirectory && !targetStorage.exists()) {
                    File(appDir).mkdirs()
                    legacyStorage.renameTo(targetStorage)
                    Log.i(TAG, "Migrated storage from legacy pencarimovie-downloader to pencarimovie-server")
                }
                legacyDir.deleteRecursively()
                Log.i(TAG, "Removed legacy directory: ${legacyDir.path}")
            } catch (e: Exception) {
                Log.w(TAG, "Migration from legacy directory error (non-fatal): ${e.message}")
            }
        }

        val hadApp = isAppInstalled()
        val updated = installOrUpdate()
        if (updated || !hadApp) {
            try { File(setupMarkerFile).writeText("done") } catch (_: Exception) {}
        }
        if (!isAppInstalled()) {
            throw RuntimeException("Failed to download and extract app release")
        }

        // Strategy: try FrankenPHP via proot first (bundled binary, fast download).
        //
        // The native Termux PHP fallback exists ONLY for the case where proot
        // itself cannot run on this device (e.g. Samsung One UI seccomp
        // restrictions on PRoot syscalls, or an Android version where proot's
        // loader cannot exec). It is NOT a general-purpose retry: if proot works
        // but FrankenPHP dies for some other reason (bad Caddyfile, port clash,
        // missing binary), falling back would silently mask a real bug and
        // install a whole PHP toolchain to work around it.
        //
        // So: probe proot first. Only when the probe fails do we take the
        // native-PHP path; otherwise a FrankenPHP failure is reported as-is.
        emitProgress("[server] Starting FrankenPHP...")
        var started = tryStartFrankenphp(lanIp)

        if (!started) {
            if (isProotRunnable()) {
                // proot works — the failure is FrankenPHP's own. Do not mask it.
                Log.e(TAG, "FrankenPHP failed to start but proot is runnable — not falling back")
                throw RuntimeException(
                    "FrankenPHP failed to start (proot is runnable, so this is not a proot issue)"
                )
            }
            emitProgress("[server] proot cannot run on this device, trying native Termux PHP fallback...")
            Log.w(TAG, "proot is not runnable, falling back to native Termux PHP")
            started = tryStartNativePhp(lanIp)
        }

        if (!started) {
            throw RuntimeException("Both FrankenPHP (proot) and native PHP servers failed to start")
        }

        emitProgress("[server] Server started successfully on port $activePort")

        activePort
    }

    private fun isAppInstalled(): Boolean {
        return File(appDir).isDirectory && (
            File("$appDir/backend.php").isFile || File("$appDir/bin/frankenphp").isFile
        )
    }

    private fun releaseTarget(): String {
        val abis = Build.SUPPORTED_ABIS.map { it.lowercase() }
        val primary = abis.firstOrNull() ?: "arm64-v8a"
        return when {
            primary.contains("x86_64") || primary == "amd64" -> "linux-x86_64"
            abis.any { it.contains("arm64") || it.contains("aarch64") } -> "linux-aarch64"
            // For 32-bit arm (e.g. Xiaomi Mi Box / Android TV running 32-bit OS), fallback to universal linux-aarch64
            else -> "linux-aarch64"
        }
    }

    private fun readCurrentTag(): String {
        return try {
            val file = File("$appDir/$RELEASE_TAG_FILE")
            if (!file.isFile) "" else file.readText().trim().replace("\r", "")
        } catch (_: Exception) {
            ""
        }
    }

    private fun writeReleaseTag(tag: String) {
        try {
            File(appDir).mkdirs()
            File("$appDir/$RELEASE_TAG_FILE").writeText("$tag\n")
        } catch (e: Exception) {
            Log.w(TAG, "Failed to write $RELEASE_TAG_FILE: ${e.message}")
        }
    }

    private fun parseTagFromUrl(url: String): String? {
        // Strip any query/fragment first: latestTagUrl() appends a cache-busting
        // `?t=<epoch>`, and if a redirect echoes it back the tag would otherwise
        // parse as "v2.3.3?t=123" and fail TAG_PATTERN.
        val path = url.trim().substringBefore('?').substringBefore('#').trimEnd('/')
        val tag = path.substringAfterLast('/')
        return if (TAG_PATTERN.containsMatchIn(tag)) tag else null
    }

    /**
     * Cache-busting query for the `releases/latest` lookup.
     *
     * GitHub serves `/releases/latest` through a CDN that caches the 302
     * redirect, so a freshly published release can keep resolving to the
     * previous tag for minutes. A unique query string makes each lookup a
     * distinct URL and bypasses the cached redirect.
     */
    private fun latestTagUrl(): String =
        "https://github.com/$REPO/releases/latest?t=${System.currentTimeMillis()}"

    /** Follow GitHub /releases/latest and return the tag (e.g. v1.0.1). */
    private fun fetchLatestTag(): String? {
        fetchLatestTagHttp()?.let { return it }
        return fetchLatestTagWget()
    }

    private fun fetchLatestTagHttp(): String? {
        var currentUrl = latestTagUrl()
        var maxRedirects = 5
        return try {
            while (maxRedirects-- > 0) {
                val conn = (URL(currentUrl).openConnection() as HttpURLConnection).apply {
                    instanceFollowRedirects = false
                    requestMethod = "GET"
                    setRequestProperty("User-Agent", "pencarimovie-downloader")
                    connectTimeout = 15_000
                    readTimeout = 15_000
                }
                try {
                    val code = conn.responseCode
                    val loc = conn.getHeaderField("Location")
                    val tag = parseTagFromUrl(loc ?: currentUrl)
                    if (tag != null) {
                        return tag
                    }
                    if ((code in 301..308) && !loc.isNullOrEmpty()) {
                        currentUrl = if (loc.startsWith("http")) loc else URL(URL(currentUrl), loc).toString()
                        continue
                    }
                    break
                } finally {
                    conn.disconnect()
                }
            }
            null
        } catch (e: Exception) {
            Log.w(TAG, "GitHub latest HTTP check failed: ${e.message}")
            null
        }
    }

    private fun fetchLatestTagWget(): String? {
        val url = latestTagUrl()
        return try {
            val script = """
                if wget --version >/dev/null 2>&1; then
                  wget --no-check-certificate -q --max-redirect=5 --server-response '$url' -O /dev/null 2>&1 \
                    | awk 'BEGIN{IGNORECASE=1} /^  Location:/{loc=${'$'}2} END{print loc}' | tr -d '\r'
                elif curl --version >/dev/null 2>&1; then
                  curl -k -fsSL -o /dev/null -w '%{url_effective}' '$url' 2>/dev/null || true
                fi
            """.trimIndent()
            val proc = ProcessBuilder(listOf(shellBin, "-c", script)).apply {
                environment()["PATH"] = "$prefixDir/bin:/system/bin"
                redirectErrorStream(true)
            }.start()
            val loc = BufferedReader(InputStreamReader(proc.inputStream)).readText().trim()
            proc.waitFor(15, java.util.concurrent.TimeUnit.SECONDS)
            parseTagFromUrl(loc)
        } catch (e: Exception) {
            Log.w(TAG, "GitHub latest wget check failed: ${e.message}")
            null
        }
    }

    /**
     * Downloads a file using Android's native HttpURLConnection (uses system CA store, follows redirects).
     */
    private fun downloadFileNative(urlStr: String, destFile: File): Boolean {
        var currentUrl = urlStr
        var redirects = 5
        while (redirects-- > 0) {
            try {
                val conn = (URL(currentUrl).openConnection() as HttpURLConnection).apply {
                    instanceFollowRedirects = false
                    connectTimeout = 30_000
                    readTimeout = 120_000
                    setRequestProperty("User-Agent", "pencarimovie-downloader")
                }
                val code = conn.responseCode
                if (code in 301..308) {
                    val loc = conn.getHeaderField("Location")
                    conn.disconnect()
                    if (loc.isNullOrEmpty()) return false
                    currentUrl = if (loc.startsWith("http")) loc else URL(URL(currentUrl), loc).toString()
                    continue
                }
                if (code != HttpURLConnection.HTTP_OK) {
                    Log.w(TAG, "Download failed with HTTP $code from $currentUrl")
                    conn.disconnect()
                    return false
                }
                destFile.parentFile?.mkdirs()
                conn.inputStream.use { input ->
                    destFile.outputStream().use { output ->
                        input.copyTo(output)
                    }
                }
                conn.disconnect()
                return destFile.isFile && destFile.length() > 0
            } catch (e: Exception) {
                Log.w(TAG, "downloadFileNative error: ${e.message}")
                return false
            }
        }
        return false
    }

    /**
     * Same start-time OTA as the one-file installers.
     * Returns true if files were installed or updated.
     */
    private suspend fun installOrUpdate(): Boolean = withContext(Dispatchers.IO) {
        val target = releaseTarget()
        emitProgress("[setup] Checking GitHub for updates...")
        val latest = fetchLatestTag()
        val current = readCurrentTag()
        val hadApp = isAppInstalled()

        if (latest.isNullOrEmpty()) {
            if (hadApp) {
                emitProgress("[setup] Could not check GitHub for updates; using installed copy.")
                return@withContext false
            }
            // No installed copy and no reachable release: nothing to install.
            Log.e(TAG, "GitHub unavailable and no installed copy to fall back to")
            emitProgress("[setup] GitHub unavailable and no installed copy — cannot install.")
            return@withContext false
        }

        if (hadApp && current == latest) {
            emitProgress("[setup] Up to date ($current).")
            Log.i(TAG, "App already at $current")
            return@withContext false
        }

        val effectiveTarget = if (hadApp) "server" else target
        if (!hadApp) {
            emitProgress("[setup] Downloading PencariMovie Server $latest ($target)...")
        } else {
            emitProgress("[setup] Fast updating PencariMovie Server $current -> $latest (universal server package)...")
        }

        val ok = downloadExtract(effectiveTarget, latest!!)
        if (!ok) {
            if (hadApp) {
                emitProgress("[setup] Update failed; using installed copy.")
                return@withContext false
            }
            Log.e(TAG, "Download/extract failed for $latest")
            return@withContext false
        }
        Log.i(TAG, "Installed $latest ($target)")
        true
    }

    /** Download a release tarball and copy over [appDir], leaving storage/ in place. */
    private fun downloadExtract(target: String, tag: String): Boolean {
        val (url, fallbackUrl) = if (target == "server") {
            val fullTarget = releaseTarget()
            "https://github.com/$REPO/releases/download/$tag/pencarimovie-server.tar.gz" to
            "https://github.com/$REPO/releases/download/$tag/pencarimovie-downloader-$fullTarget.tar.gz"
        } else {
            "https://github.com/$REPO/releases/download/$tag/pencarimovie-downloader-$target.tar.gz" to
            "https://github.com/$REPO/releases/download/$tag/pencarimovie-server.tar.gz"
        }
        val tmp = "$prefixDir/tmp/pencarimovie-ota"
        val tarFile = File("$tmp/pencarimovie.tar.gz")

        // 1. Download tarball: Use Android's native HttpURLConnection first (uses OS CA trust store)
        emitProgress("[setup] Downloading $url...")
        var downloaded = downloadFileNative(url, tarFile)
        if (!downloaded) {
            Log.w(TAG, "Primary native download failed, trying fallback: $fallbackUrl")
            downloaded = downloadFileNative(fallbackUrl, tarFile)
        }
        if (!downloaded) {
            Log.w(TAG, "Native downloads failed, falling back to wget...")
            val downloadScript = """
                mkdir -p '$tmp'
                if ! wget --no-check-certificate -O '$tmp/pencarimovie.tar.gz' '$url' 2>&1; then
                    echo "[setup] Primary wget failed, trying fallback: $fallbackUrl"
                    wget --no-check-certificate -O '$tmp/pencarimovie.tar.gz' '$fallbackUrl' 2>&1
                fi
            """.trimIndent()
            downloaded = runShell(downloadScript) && tarFile.isFile && tarFile.length() > 0
        }

        if (!downloaded) {
            Log.e(TAG, "Failed to download $url or fallback $fallbackUrl")
            return false
        }

        // 2. Extract and install
        val script = """
            set -e
            TMP='$tmp'
            APP_DIR='$appDir'
            TAG='$tag'
            mkdir -p "${'$'}TMP/extract" "${'$'}APP_DIR"
            echo "[setup] Extracting..."
            tar -xzf "${'$'}TMP/pencarimovie.tar.gz" -C "${'$'}TMP/extract"
            SRC="${'$'}TMP/extract"
            if [ ! -f "${'$'}SRC/backend.php" ] && [ ! -f "${'$'}SRC/start.sh" ]; then
              FOUND=${'$'}(find "${'$'}SRC" -maxdepth 2 -type f \( -name backend.php -o -name start.sh \) 2>/dev/null | head -1 || true)
              [ -n "${'$'}FOUND" ] && SRC=${'$'}(dirname "${'$'}FOUND")
            fi
            echo "[setup] Installing (keeping storage/)..."
            for item in "${'$'}SRC"/*; do
              [ -e "${'$'}item" ] || continue
              name=${'$'}(basename "${'$'}item")
              if [ "${'$'}name" = "storage" ]; then
                mkdir -p "${'$'}APP_DIR/storage"
                continue
              fi
              if [ "${'$'}name" = "bin" ]; then
                mkdir -p "${'$'}APP_DIR/bin"
                # Preserve existing binaries (like bin/frankenphp) when updating from pencarimovie-server.tar.gz
                cp -R "${'$'}item"/* "${'$'}APP_DIR/bin/" 2>/dev/null || true
                continue
              fi
              rm -rf "${'$'}APP_DIR/${'$'}name"
              cp -R "${'$'}item" "${'$'}APP_DIR/${'$'}name"
            done
            for f in "${'$'}APP_DIR"/*.sh; do
              [ -f "${'$'}f" ] || continue
              tr -d '\r' < "${'$'}f" > "${'$'}f.tmp" && mv "${'$'}f.tmp" "${'$'}f"
            done
            printf '%s\n' "${'$'}TAG" > "${'$'}APP_DIR/$RELEASE_TAG_FILE"
            rm -rf "${'$'}TMP"
            echo "[setup] Done (${'$'}TAG)."
        """.trimIndent()

        return try {
            val ok = runShell(script)
            if (!ok) Log.e(TAG, "downloadExtract extraction exited non-zero for $tag")
            ok
        } catch (e: Exception) {
            Log.e(TAG, "downloadExtract failed", e)
            false
        }
    }

    private fun runShell(script: String): Boolean {
        val cmd = listOf(shellBin, "-c", script)
        val env = mapOf(
            "HOME" to "$prefixDir/home",
            "PREFIX" to prefixDir,
            "PATH" to "$prefixDir/bin:/system/bin",
            "LD_LIBRARY_PATH" to "$prefixDir/lib",
            "TMPDIR" to "$prefixDir/tmp"
        )
        val pb = ProcessBuilder(cmd)
        pb.environment().putAll(env)
        pb.redirectErrorStream(true)
        val proc = pb.start()
        val reader = BufferedReader(InputStreamReader(proc.inputStream))
        var line: String?
        while (reader.readLine().also { line = it } != null) {
            emitProgress(line!!)
        }
        val exitCode = proc.waitFor()
        return exitCode == 0
    }

    private fun runShellSilent(script: String): Boolean {
        return try {
            val cmd = listOf(shellBin, "-c", script)
            val pb = ProcessBuilder(cmd)
            pb.environment().apply {
                put("HOME", "$prefixDir/home")
                put("PREFIX", prefixDir)
                put("PATH", "$prefixDir/bin:/system/bin")
                put("LD_LIBRARY_PATH", "$prefixDir/lib")
                put("TMPDIR", "$prefixDir/tmp")
            }
            pb.redirectErrorStream(true)
            val proc = pb.start()
            val exitCode = proc.waitFor()
            exitCode == 0
        } catch (_: Exception) {
            false
        }
    }

    /**
     * Try starting with proot + frankenphp.
     * Returns true if the server started successfully.
     */
    /**
     * Probe whether proot can actually exec a binary on this device.
     *
     * proot is built with PROOT_UNBUNDLE_LOADER, so it execs its own
     * statically-linked loader from $prefixDir/libexec/proot/loader instead of
     * the target binary. If that loader is missing, proot falls back to a direct
     * execve() of the target, which needs the ELF interpreter
     * /system/bin/linker64 -> /apex/... and fails with ENOENT because /apex is
     * not traversable by untrusted apps on Android 10+.
     *
     * We run `proot --version` (no target to exec) plus a trivial
     * `/system/bin/sh -c true` to confirm the loader path works end to end.
     * Returns true only if both succeed.
     */
    private fun isProotRunnable(): Boolean {
        val proot = File(prootBin)
        if (!proot.isFile || !proot.canExecute()) {
            Log.w(TAG, "proot not executable at $prootBin")
            return false
        }
        val loader = File("$prefixDir/libexec/proot/loader")
        if (!loader.isFile) {
            Log.w(TAG, "proot loader missing at ${loader.path}")
            return false
        }
        return try {
            val pb = ProcessBuilder(
                prootBin, "--link2symlink", "-0",
                "/system/bin/sh", "-c", "true"
            )
            pb.environment().apply {
                put("HOME", "$prefixDir/home")
                put("PREFIX", prefixDir)
                put("PATH", "$prefixDir/bin:/system/bin")
                put("TMPDIR", "$prefixDir/tmp")
                remove("LD_LIBRARY_PATH")
                remove("LD_PRELOAD")
            }
            pb.redirectErrorStream(true)
            val proc = pb.start()
            // Drain output so the child cannot block on a full pipe.
            proc.inputStream.readBytes()
            val ok = proc.waitFor(10, java.util.concurrent.TimeUnit.SECONDS) && proc.exitValue() == 0
            Log.i(TAG, "proot probe: ${if (ok) "runnable" else "NOT runnable (exit ${if (proc.isAlive) "timeout" else proc.exitValue()})"}")
            ok
        } catch (e: Exception) {
            Log.w(TAG, "proot probe failed: ${e.message}")
            false
        }
    }

    private suspend fun tryStartFrankenphp(lanIp: String? = null): Boolean = withContext(Dispatchers.IO) {
        val appDirPath = appDir
        val frankenphpBin = "$appDirPath/bin/frankenphp"
        val tmpDir = "$prefixDir/tmp"
        val usableLanIp = lanIp?.trim().orEmpty()

        // Check that frankenphp exists in the release
        if (!File(frankenphpBin).exists()) {
            Log.w(TAG, "FrankenPHP binary not found at $frankenphpBin")
            return@withContext false
        }

        // Fix permissions — tar extraction on Android (bionic) may not preserve
        // Unix execute bits. This matches the setup script's chmod logic.
        try {
            // Ensure temp directory exists with proper permissions
            File(tmpDir).mkdirs()
            File(tmpDir).setReadable(true, true)
            File(tmpDir).setWritable(true, true)
            File(tmpDir).setExecutable(true, true) // chmod 700 equivalent
            Log.i(TAG, "Ensured tmpDir exists with 700 permissions")
        } catch (e: Exception) {
            Log.w(TAG, "Failed to set tmpDir permissions: ${e.message}")
        }

        // Set +x on all executables that the setup script chmods
        val executablesToFix = listOf(
            frankenphpBin,                      // bin/frankenphp
            "$appDirPath/bin/php",              // bin/php (MadelineProto IPC worker)
            "$appDirPath/bin/addon",            // bin/addon (Bun Addon daemon)
            "$appDirPath/start.sh",             // start script
            "$appDirPath/stop.sh",              // stop script
            "$appDirPath/restart.sh",           // restart script
            "$appDirPath/install.sh",           // install script
            "$appDirPath/start-termux.sh",      // Termux start script
            "$appDirPath/install-termux.sh",    // Termux install script
            "$appDirPath/restart-termux.sh"     // Termux restart script
        )
        for (filePath in executablesToFix) {
            val file = File(filePath)
            if (file.exists()) {
                try {
                    // Strip Windows CRLF line endings from shell scripts & wrapper binaries
                    if (file.name.endsWith(".sh") || file.name == "php") {
                        val content = file.readText()
                        if (content.contains("\r\n")) {
                            file.writeText(content.replace("\r\n", "\n"))
                            Log.i(TAG, "Converted CRLF to LF for ${file.name}")
                        }
                    }
                    file.setExecutable(true, false)
                    Log.i(TAG, "Set +x on ${file.name}")
                } catch (e: Exception) {
                    Log.w(TAG, "Failed to prepare ${file.name}: ${e.message}")
                }
            } else {
                Log.d(TAG, "Skipping +x on ${file.name} (not found)")
            }
        }

        // Ensure bin/php.ini has display_errors=0 and no dynamic extension loading.
        // The release tarball's bin/php.ini may be the Windows version (with
        // extension=*.dll entries) or may not have display_errors=0 at all.
        // Without display_errors=0, PHP warnings/notices echo into JSON response
        // bodies, breaking all API calls with HTTP 500 / non-JSON responses.
        // We override the ini by writing a minimal config with known-good settings.
        //
        // CRITICAL: max_execution_time must be 0 (unlimited) because:
        //   1. The release tarball's old backend.php uses 20-second Amp timeouts
        //      for fd_check_version().
        //   2. Combined with Composer autoloader loading time, the request
        //      easily exceeds PHP's default 30s max_execution_time.
        //   3. When the limit is hit, PHP kills the request and returns
        //      HTTP 500 with empty body — no error message, no JSON.
        val phpIniFile = File("$appDirPath/bin/php.ini")
        try {
            phpIniFile.writeText("""
[PHP]
display_errors = 0
log_errors = 1
html_errors = 0
max_execution_time = 0
max_input_time = -1
opcache.enable = 0
opcache.enable_cli = 0

""".trimIndent())
            Log.i(TAG, "Wrote php.ini with display_errors=0, max_execution_time=0 at $phpIniFile")
        } catch (e: Exception) {
            Log.w(TAG, "Failed to write php.ini: ${e.message}")
        }

        // FrankenPHP under proot cannot exec Termux ifconfig. Persist the
        // Kotlin-detected LAN IP so backend.php can return it to the Nuvio card.
        val lanIpFile = File("$appDirPath/storage/lan_ip.txt")
        try {
            File("$appDirPath/storage").mkdirs()
            if (usableLanIp.isNotEmpty()) {
                lanIpFile.writeText(usableLanIp)
                Log.i(TAG, "Wrote LAN_IP $usableLanIp to ${lanIpFile.path}")
            } else if (lanIpFile.exists()) {
                lanIpFile.delete()
            }
        } catch (e: Exception) {
            Log.w(TAG, "Failed to write lan_ip.txt: ${e.message}")
        }

        // Build the proot command matching the setup script with PHPRC fix.
        // PHPRC must point to bin/ so FrankenPHP loads bin/php.ini (which was
        // copied from bin/php.ini.unix above). Without PHPRC, static FrankenPHP
        // builds may not find any php.ini, leaving display_errors=1 and breaking
        // JSON responses.
        // Generate resolv.conf for both Termux prefix and proot DNS resolution on Android
        val resolvContent = "nameserver 1.1.1.1\nnameserver 8.8.8.8\nnameserver 1.0.0.1\nnameserver 8.8.4.4\n"
        try {
            val resolvFile = File("$tmpDir/resolv.conf")
            resolvFile.writeText(resolvContent)
            resolvFile.setReadable(true, false)

            // Also write to $PREFIX/etc/resolv.conf so native Termux tools & PHP can read it directly without proot
            val prefixEtc = File("$prefixDir/etc")
            prefixEtc.mkdirs()
            val termuxResolv = File(prefixEtc, "resolv.conf")
            termuxResolv.writeText(resolvContent)
            termuxResolv.setReadable(true, false)
        } catch (e: Exception) {
            Log.w(TAG, "Failed to write resolv.conf: ${e.message}")
        }

        // The Termux prefix must stay on PATH so `pkg` (bionic-linked) resolves
        // inside proot.
        //
        // ulimit -n 65536: Android app UIDs default to a soft RLIMIT_NOFILE of
        // 1024 while the hard limit is much higher. PHP's stream_select() is
        // hard-capped at FD_SETSIZE (1024), so once the MadelineProto IPC worker
        // accumulates 1025+ descriptors the event loop throws
        //   stream_select(): You MUST recompile PHP with a larger value of FD_SETSIZE
        // which corrupts the Revolt suspension state, kills the worker, and makes
        // the IPC client respawn it in a tight loop until the server wedges.
        // Set once in the outer shell: RLIMIT_NOFILE is inherited across exec, so
        // proot -> /system/bin/sh -> frankenphp all keep the raised limit.
        //
        // FRANKENPHP_NUM_THREADS / FRANKENPHP_MAX_THREADS / PHP_MEMORY_LIMIT /
        // FD_DOWNLOAD_PARALLEL_CHUNKS are intentionally NOT set: the Caddyfile
        // already declares num_threads/max_threads/php_ini memory_limit with its
        // own defaults, and pinning them here capped concurrency and memory below
        // what the device can handle.
        val shellCommand = buildString {
            append("export PATH=\"$appDirPath/bin:$prefixDir/bin:\$PATH\" && ")
            append("export PHP_BINDIR=\"$appDirPath/bin\" && ")
            append("export PHPRC=\"$appDirPath/bin\" && ")
            append("export PREFIX=\"$prefixDir\" && ")
            append("export MALLOC_ARENA_MAX=2 && ")
            append("export GODEBUG=\"madvdontneed=1\" && ")
            append("export GOGC=80 && ")
            append("export GOMEMLIMIT=450MiB && ")
            if (usableLanIp.isNotEmpty()) {
                append("export LAN_IP=\"$usableLanIp\" && ")
            }
            append("ulimit -n 65536 2>/dev/null || ulimit -n \"\$(ulimit -Hn 2>/dev/null)\" 2>/dev/null || true; ")
            append("exec \"$prootBin\" --link2symlink -0 ")
            append("-w \"$appDirPath\" ")
            append("-b \"$appDirPath:$appDirPath\" ")
            append("-b \"$tmpDir:/tmp\" ")
            append("-b \"$tmpDir/resolv.conf:/etc/resolv.conf\" ")
            append("/system/bin/sh -c '")
            append("export PATH=\"$appDirPath/bin:$prefixDir/bin:\$PATH\"; ")
            append("export PHP_BINDIR=\"$appDirPath/bin\"; ")
            append("export PHPRC=\"$appDirPath/bin\"; ")
            append("export PREFIX=\"$prefixDir\"; ")
            append("export MALLOC_ARENA_MAX=2; ")
            append("export GODEBUG=\"madvdontneed=1\"; ")
            append("export GOGC=80; ")
            append("export GOMEMLIMIT=450MiB; ")
            if (usableLanIp.isNotEmpty()) {
                append("export LAN_IP=\"$usableLanIp\"; ")
            }
            append("if [ -f \"$appDirPath/Caddyfile\" ]; then ")
            append("exec \"$frankenphpBin\" run --config \"$appDirPath/Caddyfile\"; ")
            append("else ")
            append("exec \"$frankenphpBin\" php-server --listen \"$host:$activePort\" --root \"$appDirPath\"; ")
            append("fi")
            append("' 2>&1")
        }

        return@withContext try {
            startProcess(listOf(shellBin, "-c", shellCommand), "FrankenPHP")
        } catch (e: Exception) {
            Log.w(TAG, "FrankenPHP failed: ${e.message}")
            false
        }
    }

    /**
     * Try starting with native Termux PHP (no proot).
     * Downloads/installs php and php-gd only on demand if missing.
     */
    private suspend fun tryStartNativePhp(lanIp: String? = null): Boolean = withContext(Dispatchers.IO) {
        val appDirPath = appDir
        val routerFile = "$appDirPath/router.php"
        val usableLanIp = lanIp?.trim().orEmpty()

        if (!File(routerFile).exists()) {
            Log.w(TAG, "router.php not found at $routerFile")
            return@withContext false
        }

        // Ensure all prefix executables and apt helper binaries (+x)
        try {
            val dirsToChmod = listOf(
                File("$prefixDir/bin"),
                File("$prefixDir/libexec"),
                File("$prefixDir/lib/apt/methods")
            )
            for (dir in dirsToChmod) {
                if (dir.isDirectory) {
                    dir.listFiles()?.forEach { file ->
                        if (file.isFile) file.setExecutable(true, false)
                    }
                }
            }
        } catch (_: Exception) {}

        // Check if native php + gd are installed; if missing, install on-demand via apt/pkg
        // Notice: upstream .deb archives from packages.termux.dev contain ./data/data/com.termux
        // which causes dpkg --unpack to fail with permission denied on custom package IDs.
        // We download the debs with apt-get and unpack data.tar directly into $PREFIX.
        val checkPhpScript = """
            export DEBIAN_FRONTEND=noninteractive
            export PHP_INI_SCAN_DIR="${'$'}PREFIX/etc/php/conf.d"
            export OPENSSL_CONF="$prefixDir/etc/tls/openssl.cnf"
            export SSL_CERT_FILE="$prefixDir/etc/tls/cert.pem"
            export SSL_CERT_DIR="$prefixDir/etc/tls/certs"

            mkdir -p "$prefixDir/etc/tls" "$prefixDir/etc/tls/certs"
            [ -f "$prefixDir/etc/tls/openssl.cnf" ] || echo "# OpenSSL config" > "$prefixDir/etc/tls/openssl.cnf"

            if ! [ -f "$prefixDir/bin/php" ] || ! "$prefixDir/bin/php" -v >/dev/null 2>&1; then
                echo '[setup] Installing/repairing native Termux PHP and extensions (php, php-gd, libcurl, libngtcp2)...'
                if command -v apt-get >/dev/null 2>&1; then
                    dpkg -r --force-depends dpkg-scanpackages dpkg-perl 2>&1 || true
                    dpkg --configure -a 2>&1 || true
                    apt-get update -y 2>&1
                    # Download php, php-gd, libcurl and its dynamic library dependencies (libngtcp2, libnghttp3, libssh2, openldap)
                    apt-get install -y -d php php-gd libcurl openldap libngtcp2 libnghttp3 libnghttp2 libssh2 openssl nodejs ca-certificates 2>&1 || true
                    
                    EXTRACT_DIR="${'$'}TMPDIR/deb-extract-tmp"
                    rm -rf "${'$'}EXTRACT_DIR" && mkdir -p "${'$'}EXTRACT_DIR"
                    
                    # Find all downloaded deb packages across cache directories
                    for deb in ${'$'}(find "${'$'}PREFIX" "${'$'}TMPDIR" /data/data/com.pencarimovie.downloader -name "*.deb" 2>/dev/null); do
                        [ -f "${'$'}deb" ] || continue
                        echo "[setup] Extracting ${'$'}(basename "${'$'}deb")..."
                        dpkg-deb -x "${'$'}deb" "${'$'}EXTRACT_DIR" 2>&1 || true
                    done
                    
                    # Copy all extracted files under 'usr' directly into our prefix
                    for usr_dir in ${'$'}(find "${'$'}EXTRACT_DIR" -type d -name "usr" 2>/dev/null); do
                        echo "[setup] Found usr dir at ${'$'}usr_dir, copying to prefix..."
                        cp -rf "${'$'}usr_dir"/* "$prefixDir/" 2>&1 || true
                    done
                    rm -rf "${'$'}EXTRACT_DIR"
                    
                    mkdir -p "$prefixDir/etc/tls" "$prefixDir/etc/tls/certs"
                    [ -f "$prefixDir/etc/tls/openssl.cnf" ] || echo "# OpenSSL config" > "$prefixDir/etc/tls/openssl.cnf"
                    chmod +x "$prefixDir/bin/"* 2>/dev/null || true
                    
                    if "$prefixDir/bin/php" -v >/dev/null 2>&1; then
                        echo "[setup] Native PHP installed successfully: $($prefixDir/bin/php -v 2>&1 | head -n 1)"
                    else
                        echo "[setup] ERROR: php binary failed to execute: $($prefixDir/bin/php -v 2>&1 | head -n 2)"
                        exit 1
                    fi
                elif command -v pkg >/dev/null 2>&1; then
                    pkg install -y php php-gd libcurl libngtcp2 openssl nodejs ca-certificates 2>&1
                else
                    echo '[setup] Package manager not found'
                    exit 1
                fi
            fi
        """.trimIndent()

        try {
            emitProgress("[setup] Verifying native PHP installation...")
            val installed = runShell(checkPhpScript)
            if (!installed) {
                Log.w(TAG, "Failed to verify or install native Termux PHP")
                return@withContext false
            }
        } catch (e: Exception) {
            Log.w(TAG, "Native PHP check error: ${e.message}")
            return@withContext false
        }

        val shellCommand = buildString {
            append("cd \"$appDirPath\" && ")
            if (usableLanIp.isNotEmpty()) {
                append("export LAN_IP=\"$usableLanIp\" && ")
            }
            append("exec \"$prefixDir/bin/php\" -S \"$host:$activePort\" \"$routerFile\" 2>&1")
        }

        return@withContext try {
            startProcess(listOf(shellBin, "-c", shellCommand), "NativePHP")
        } catch (e: Exception) {
            Log.w(TAG, "Native PHP failed: ${e.message}")
            false
        }
    }

    /**
     * Start a server process and wait for it to be ready.
     * Returns true if the process started and stayed alive (indicating the server is running).
     */
    private suspend fun startProcess(command: List<String>, mode: String): Boolean = withContext(Dispatchers.IO) {
        Log.i(TAG, "[$mode] Starting: ${command.joinToString(" ")}")

        val pb = ProcessBuilder(command)
        val env = pb.environment()
        env["HOME"] = "$prefixDir/home"
        env["PREFIX"] = prefixDir
        env["PATH"] = "$prefixDir/bin:$appDir/bin:/system/bin"
        env["LD_LIBRARY_PATH"] = "$prefixDir/lib"
        env["TMPDIR"] = "$prefixDir/tmp"
        env["PORT"] = activePort.toString()
        env["HOST"] = host
        val processLanIp = File("$appDir/storage/lan_ip.txt")
            .takeIf { it.isFile }
            ?.readText()
            ?.trim()
            .orEmpty()
        if (processLanIp.isNotEmpty()) {
            env["LAN_IP"] = processLanIp
        }
        // PHPRC tells FrankenPHP's static build where to find php.ini (must have
        // display_errors=0 to avoid corrupting JSON responses with error HTML).
        // The php.ini.unix copy above ensures bin/php.ini has the correct settings.
        env["PHPRC"] = "$appDir/bin"
        pb.redirectErrorStream(true)

        val proc = pb.start()
        this@NativeRunner.process = proc
        running = true

        val reader = BufferedReader(InputStreamReader(proc.inputStream))
        val logLines = mutableListOf<String>()
        startStdoutReader(reader, logLines)

        // Wait up to 15 seconds for the process to either stabilize or crash
        val startTime = System.currentTimeMillis()
        val timeoutMs = 15000L

        while (System.currentTimeMillis() - startTime < timeoutMs) {
            if (!running) break

            // Check if process died
            try {
                proc.exitValue()
                running = false
                val exitCode = proc.exitValue()
                Log.w(TAG, "[$mode] Process exited with code $exitCode after ${System.currentTimeMillis() - startTime}ms")
                logLines.takeLast(10).forEach { Log.w(TAG, "  $it") }
                return@withContext false
            } catch (_: IllegalThreadStateException) {
                // Still running — good sign
            }

            // Read logLines under synchronized lock to ensure visibility of
            // additions made by the stdout reader thread (Java memory model).
            val snapshot: List<String>
            synchronized(logLines) {
                snapshot = logLines.toList()
            }

            // Check for startup indicators (log text OR actual TCP port listening)
            val hasStartup = snapshot.any { line ->
                line.contains("Listening on") ||
                line.contains("listening on") ||
                line.contains("ready") ||
                line.contains("PencariMovie Server is running") ||
                line.contains("PencariMovie Downloader is running") ||
                line.contains("started successfully") ||
                line.contains("started on") ||
                line.contains("FrankenPHP")
            } || isPortListening(activePort)
            val elapsed = System.currentTimeMillis() - startTime
            if (hasStartup && elapsed > 3000) {
                // Process is alive and reported startup
                Log.i(TAG, "[$mode] Server startup detected at ${elapsed}ms")
                return@withContext true
            }

            Thread.sleep(500)
        }

        // Timeout reached — if process is still running, assume it started
        try {
            proc.exitValue()
            running = false
            Log.w(TAG, "[$mode] Process died within timeout")
            return@withContext false
        } catch (_: IllegalThreadStateException) {
            // Still running — good!
            Log.i(TAG, "[$mode] Process stable after $timeoutMs ms — assuming started")
            return@withContext true
        }
    }

    private fun startStdoutReader(reader: BufferedReader, logLines: MutableList<String>) {
        stdoutReader = Thread {
            var diedUnexpectedly = false
            try {
                var line: String? = null
                while (running && reader.readLine().also { line = it } != null) {
                    val msg = line!!
                    synchronized(logLines) {
                        logLines.add(msg)
                        if (logLines.size > 1000) logLines.removeAt(0)
                    }
                    emitProgress(msg)
                }
                // If we exit the loop because readLine() returned null
                // (process stdout closed == process died), but running is
                // still true, the process died unexpectedly.
                if (running) {
                    Log.w(TAG, "Process stdout closed — process died unexpectedly")
                    diedUnexpectedly = true
                }
            } catch (e: Exception) {
                if (running) {
                    Log.e(TAG, "Stdout reader error — process may have died", e)
                    diedUnexpectedly = true
                }
            } finally {
                running = false
                if (diedUnexpectedly) {
                    Log.i(TAG, "Invoking onProcessDied callback")
                    onProcessDied?.invoke()
                }
            }
        }.apply {
            isDaemon = true
            start()
        }
    }

    private fun emitBanner() {
        emitProgress("========================================")
        emitProgress("         PencariMovie Server")
        emitProgress("========================================")
    }

    private fun emitProgress(msg: String) {
        onProgress?.invoke(msg)
    }

    /**
     * Kills all running server processes (FrankenPHP, PRoot, Native PHP) and any process listening on the port.
     */
    private fun killAllServerProcesses() {
        try {
            val cleanupScript = """
                # 1. Standard pattern matching kill
                pkill -9 -f 'frankenphp' 2>/dev/null || true
                pkill -9 -f 'proot.*pencarimovie' 2>/dev/null || true
                pkill -9 -f 'php.*router' 2>/dev/null || true
                pkill -9 -f 'php -S' 2>/dev/null || true
                # 2. Kill by port using lsof or fuser if available
                if command -v lsof >/dev/null 2>&1; then
                    lsof -ti tcp:$activePort 2>/dev/null | xargs -r kill -9 2>/dev/null || true
                elif command -v fuser >/dev/null 2>&1; then
                    fuser -k -9 $activePort/tcp 2>/dev/null || true
                fi

                # 3. /proc inspection fallback: kill any process owned by our app binding or running php/frankenphp
                for pid in ${'$'}(ls -d /proc/[0-9]* 2>/dev/null | sed 's|/proc/||'); do
                    cmd="${'$'}(cat /proc/${'$'}pid/cmdline 2>/dev/null | tr '\0' ' ' || true)"
                    if echo "${'$'}cmd" | grep -qE 'frankenphp|php.*-S|proot.*pencarimovie'; then
                        kill -9 "${'$'}pid" 2>/dev/null || true
                    fi
                done
                sleep 0.5
            """.trimIndent()

            val pb = ProcessBuilder(listOf(shellBin, "-c", cleanupScript))
            pb.environment()["PATH"] = "$prefixDir/bin:/system/bin"
            val proc = pb.start()
            proc.waitFor(3, java.util.concurrent.TimeUnit.SECONDS)
        } catch (e: Exception) {
            Log.w(TAG, "killAllServerProcesses error: ${e.message}")
        }
    }

    private fun stopProcessOnly() {
        running = false
        val proc = process
        if (proc != null) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                proc.destroyForcibly()
            } else {
                proc.destroy()
            }
            process = null
        }
        stdoutReader?.interrupt()
        stdoutReader = null
    }


    fun stop() {
        onProcessDied = null
        stopProcessOnly()
        killAllServerProcesses()

        // Small delay to allow the OS to release the port (TIME_WAIT)
        try { Thread.sleep(500) } catch (_: InterruptedException) {}

        Log.i(TAG, "Server stopped")
    }
}
