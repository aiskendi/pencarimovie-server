package com.pencarimovie.server

import android.content.Context
import android.content.Intent
import android.net.Uri
import android.os.Build
import android.util.Log
import androidx.core.content.FileProvider
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.io.BufferedReader
import java.io.File
import java.io.FileOutputStream
import java.io.InputStreamReader
import java.net.HttpURLConnection
import java.net.URL
import java.security.MessageDigest

/**
 * Checks the rolling `apk-latest` GitHub release for a newer APK.
 *
 * The release publishes a small `version.json` asset alongside the APK, so the
 * check is a few hundred bytes rather than a 15 MB download:
 *
 * ```json
 * {
 *   "applicationId": "com.pencarimovie.server",
 *   "versionName": "1.0.1",
 *   "versionCode": 2,
 *   "apks": {
 *     "arm64-v8a":   { "url": "https://github.com/.../pencarimovie-server_release_arm64-v8a.apk",   "sha256": "..." },
 *     "armeabi-v7a": { "url": "https://github.com/.../pencarimovie-server_release_armeabi-v7a.apk", "sha256": "..." },
 *     "universal":   { "url": "https://github.com/.../pencarimovie-server_release_universal.apk",   "sha256": "..." }
 *   },
 *   "builtAt": "2026-09-22T06:00:00Z",
 *   "commit": "abc1234"
 * }
 * ```
 *
 * The updater picks the asset for the device ABI, falling back to `universal`.
 * A legacy single `apkUrl` (+ `sha256`) is still accepted: the ABI token in the
 * filename is swapped for the device ABI, else `_universal`, before falling back
 * to the raw URL. A `applicationId` that doesn't match this app is ignored.
 *
 * Comparison uses [versionCode] (a monotonically increasing integer derived from
 * the semver parts by the build workflow), which is far more reliable than
 * string-comparing version names.
 */
object UpdateChecker {

    private const val TAG = "UpdateChecker"

    // NOTE: the server app has its OWN rolling release feed (`apk-server-latest`,
    // published by .github/workflows/android-server-apk.yml). It is deliberately
    // NOT the `apk-latest` tag, which feeds the legacy pencarimovie.downloader
    // app (android-apk.yml) — sharing it would offer the wrong app's APK.
    private const val VERSION_JSON_URL =
        "https://github.com/aiskendi/pencarimovie-server/releases/download/apk-server-latest/version.json"

    /** Where the user is sent if the direct download fails. */
    const val RELEASE_PAGE_URL =
        "https://github.com/aiskendi/pencarimovie-server/releases/tag/apk-server-latest"

    /** Subdirectory of cacheDir where the APK is staged for the installer. */
    private const val UPDATE_DIR = "updates"
    private const val APK_NAME = "pencarimovie-update.apk"

    /** Result of a successful check. */
    data class UpdateInfo(
        val versionName: String,
        val versionCode: Int,
        val apkUrl: String,
        val sha256: String,
        val builtAt: String,
        val commit: String
    )

    /**
     * Fetch the published version and return it only when it is newer than
     * [currentVersionCode]. Returns null when up to date, offline, or on any
     * parse/network error — a failed check must never block the UI.
     */
    suspend fun checkForUpdate(currentVersionCode: Int, currentVersionName: String = BuildConfig.VERSION_NAME): UpdateInfo? =
        withContext(Dispatchers.IO) {
            try {
                val json = fetchVersionJson() ?: return@withContext null
                val info = parse(json) ?: return@withContext null

                val isNewer = info.versionCode > currentVersionCode ||
                    (info.versionCode >= currentVersionCode && isSemverNewer(info.versionName, currentVersionName))

                if (isNewer) {
                    Log.i(
                        TAG,
                        "Update available: ${info.versionName} (${info.versionCode}) " +
                            "> installed ($currentVersionName, code $currentVersionCode)"
                    )
                    info
                } else {
                    Log.i(TAG, "Up to date (installed $currentVersionName / code $currentVersionCode, remote ${info.versionName} / code ${info.versionCode})")
                    null
                }
            } catch (e: Exception) {
                Log.w(TAG, "Update check failed: ${e.message}")
                null
            }
        }

    private fun isSemverNewer(remoteVersion: String, currentVersion: String): Boolean {
        try {
            val remoteClean = remoteVersion.trim().removePrefix("v").substringBefore("-").substringBefore("+")
            val currentClean = currentVersion.trim().removePrefix("v").substringBefore("-").substringBefore("+")
            val remoteParts = remoteClean.split('.').map { it.toIntOrNull() ?: 0 }
            val currentParts = currentClean.split('.').map { it.toIntOrNull() ?: 0 }
            val maxLen = maxOf(remoteParts.size, currentParts.size)
            for (i in 0 until maxLen) {
                val r = remoteParts.getOrElse(i) { 0 }
                val c = currentParts.getOrElse(i) { 0 }
                if (r > c) return true
                if (r < c) return false
            }
        } catch (_: Exception) {}
        return false
    }

    private fun fetchVersionJson(): String? {
        // Cache-buster: the apk-latest asset is replaced in place, and the CDN
        // would otherwise serve the previous version.json for minutes.
        val url = "$VERSION_JSON_URL?t=${System.currentTimeMillis()}"
        val conn = (URL(url).openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            setRequestProperty("User-Agent", "pencarimovie-server")
            setRequestProperty("Accept", "application/json")
            connectTimeout = 15_000
            readTimeout = 15_000
            instanceFollowRedirects = true
        }
        return try {
            val code = conn.responseCode
            if (code != HttpURLConnection.HTTP_OK) {
                Log.w(TAG, "version.json returned HTTP $code")
                return null
            }
            BufferedReader(InputStreamReader(conn.inputStream)).use { it.readText() }
        } finally {
            conn.disconnect()
        }
    }

    private fun parse(json: String): UpdateInfo? {
        return try {
            // A BOM (PowerShell's Set-Content UTF8 emits one) makes org.json throw, which
            // silently disabled every update. Strip it, like backend.php
            // fd_fetch_credentials_from_wordpress() does for the WordPress body.
            val o = JSONObject(json.removePrefix("\uFEFF").trim())

            // Application-id guard: never offer an update built for another app
            // (e.g. the old com.pencarimovie.downloader release).
            val pkg = o.optString("applicationId", o.optString("package", "")).trim()
            if (pkg.isNotEmpty() && pkg != BuildConfig.APPLICATION_ID) {
                Log.i(TAG, "Ignoring update for applicationId=$pkg (this app is ${BuildConfig.APPLICATION_ID})")
                return null
            }

            val name = o.optString("versionName").trim()
            val code = o.optInt("versionCode", -1)
            if (name.isEmpty() || code <= 0) {
                Log.w(TAG, "version.json missing versionName/versionCode")
                return null
            }

            val apk = selectApk(o)
            if (apk == null) {
                Log.w(TAG, "version.json has no usable APK asset")
                return null
            }

            UpdateInfo(
                versionName = name,
                versionCode = code,
                apkUrl = apk.first,
                sha256 = apk.second,
                builtAt = o.optString("builtAt").trim(),
                commit = o.optString("commit").trim()
            )
        } catch (e: Exception) {
            Log.w(TAG, "version.json parse failed: ${e.message}")
            null
        }
    }

    /**
     * Pick the release asset matching this device's ABI, falling back to
     * `universal` and finally the raw `apkUrl`. Supports the per-ABI `apks` map
     * and the legacy single `apkUrl` (ABI token in the filename swapped for the
     * device ABI, e.g. `..._arm64-v8a.apk` -> `..._armeabi-v7a.apk` ->
     * `..._universal.apk`). Returns (url, sha256); the hash is only trusted when
     * it came from an explicitly listed asset, so a derived URL has a blank hash.
     */
    private fun selectApk(o: JSONObject): Pair<String, String>? {
        val abi = deviceAbi()

        // Preferred schema: explicit per-ABI entries.
        o.optJSONObject("apks")?.let { apks ->
            for (key in listOfNotNull(abi.ifEmpty { null }, "universal")) {
                val e = apks.optJSONObject(key) ?: continue
                val u = e.optString("url").trim()
                if (u.isNotEmpty()) return u to e.optString("sha256").trim()
            }
            val keys = apks.keys()
            while (keys.hasNext()) {
                val e = apks.optJSONObject(keys.next()) ?: continue
                val u = e.optString("url").trim()
                if (u.isNotEmpty()) return u to e.optString("sha256").trim()
            }
        }

        // Legacy single URL: derive the matching ABI sibling, else universal.
        val base = o.optString("apkUrl").trim()
        if (base.isEmpty()) return null
        val baseSha = o.optString("sha256").trim()

        if (abi.isNotEmpty()) {
            val abiUrl = swapAbi(base, abi)
            if (abiUrl == base) return base to baseSha          // published asset already matches this device
            if (urlExists(abiUrl)) return abiUrl to ""
        }
        val uniUrl = swapAbi(base, "universal")
        if (uniUrl != base && urlExists(uniUrl)) return uniUrl to ""
        return base to baseSha
    }

    /** Best shipped ABI for this device (arm64, then arm32, then x86). */
    private fun deviceAbi(): String {
        val abis = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
            Build.SUPPORTED_ABIS
        } else {
            @Suppress("DEPRECATION") arrayOf(Build.CPU_ABI)
        }
        return when {
            abis.any { it == "arm64-v8a" } -> "arm64-v8a"
            abis.any { it == "armeabi-v7a" } -> "armeabi-v7a"
            abis.any { it == "x86_64" } -> "x86_64"
            abis.any { it == "x86" } -> "x86"
            else -> ""
        }
    }

    private val ABI_TOKEN = Regex("(arm64-v8a|armeabi-v7a|x86_64|x86|universal)")

    /** Replace the ABI token in an APK filename, keeping the rest of the name. */
    private fun swapAbi(url: String, abi: String): String {
        val idx = url.lastIndexOf('/')
        if (idx < 0) return url
        val file = url.substring(idx + 1)
        if (!ABI_TOKEN.containsMatchIn(file)) return url
        return url.substring(0, idx + 1) + ABI_TOKEN.replace(file, abi)
    }

    /** HEAD-probe a candidate asset URL so a 404 never reaches the installer. */
    private fun urlExists(url: String): Boolean = try {
        val conn = (URL(url).openConnection() as HttpURLConnection).apply {
            requestMethod = "HEAD"
            setRequestProperty("User-Agent", "pencarimovie-server")
            connectTimeout = 8_000
            readTimeout = 8_000
            instanceFollowRedirects = true
        }
        try { conn.responseCode in 200..399 } finally { conn.disconnect() }
    } catch (e: Exception) {
        false
    }

    /**
     * Download [info]'s APK into cacheDir/updates and verify its SHA-256.
     *
     * @param onProgress called with (bytesRead, totalBytes); totalBytes is -1
     *   when the server does not send Content-Length.
     * @return the staged APK file, or null on any failure.
     */
    suspend fun downloadApk(
        context: Context,
        info: UpdateInfo,
        onProgress: (downloaded: Long, total: Long) -> Unit = { _, _ -> }
    ): File? = withContext(Dispatchers.IO) {
        if (info.apkUrl.isEmpty()) {
            Log.w(TAG, "version.json has no apkUrl")
            return@withContext null
        }

        val dir = File(context.cacheDir, UPDATE_DIR).apply { mkdirs() }
        // Write to a temp name first so a partial download is never handed to
        // the installer, then rename once the hash checks out.
        val tmp = File(dir, "$APK_NAME.part")
        val dest = File(dir, APK_NAME)
        tmp.delete()

        try {
            val conn = (URL(info.apkUrl).openConnection() as HttpURLConnection).apply {
                requestMethod = "GET"
                setRequestProperty("User-Agent", "pencarimovie-server")
                connectTimeout = 20_000
                readTimeout = 60_000
                instanceFollowRedirects = true
            }

            try {
                val code = conn.responseCode
                if (code != HttpURLConnection.HTTP_OK) {
                    Log.w(TAG, "APK download returned HTTP $code")
                    return@withContext null
                }
                val total = conn.contentLengthLong

                conn.inputStream.use { input ->
                    FileOutputStream(tmp).use { output ->
                        val buf = ByteArray(64 * 1024)
                        var read: Int
                        var done = 0L
                        while (input.read(buf).also { read = it } > 0) {
                            output.write(buf, 0, read)
                            done += read
                            onProgress(done, total)
                        }
                        output.flush()
                    }
                }
            } finally {
                conn.disconnect()
            }

            // Verify the hash when the release published one. A mismatch means a
            // truncated or tampered download, so refuse to install it.
            if (info.sha256.isNotEmpty() && info.sha256.length == 64) {
                val actual = sha256(tmp)
                if (!actual.equals(info.sha256, ignoreCase = true)) {
                    Log.e(TAG, "APK sha256 mismatch: expected ${info.sha256}, got $actual")
                    tmp.delete()
                    return@withContext null
                }
                Log.i(TAG, "APK sha256 verified")
            } else {
                Log.w(TAG, "No usable sha256 in version.json; skipping verification")
            }

            dest.delete()
            if (!tmp.renameTo(dest)) {
                Log.e(TAG, "Failed to move downloaded APK into place")
                tmp.delete()
                return@withContext null
            }
            Log.i(TAG, "APK staged at ${dest.absolutePath} (${dest.length()} bytes)")
            dest
        } catch (e: Exception) {
            Log.w(TAG, "APK download failed: ${e.message}")
            tmp.delete()
            null
        }
    }

    /**
     * Hand the staged APK to the system package installer.
     *
     * The installer runs in a different process, so the file is exposed through
     * a content:// URI via the FileProvider declared in the manifest. Android
     * shows its own confirmation dialog — installation is never silent.
     *
     * @return true if the installer activity was launched.
     */
    fun installApk(context: Context, apk: File): Boolean {
        // Final guard: refuse to install an APK that isn't this exact app. A
        // stale/different release asset would otherwise install as a separate app.
        try {
            val archive = context.packageManager.getPackageArchiveInfo(apk.absolutePath, 0)
            if (archive == null || archive.packageName != context.packageName) {
                Log.e(TAG, "Refusing to install APK for package ${archive?.packageName} (expected ${context.packageName})")
                return false
            }
        } catch (e: Exception) {
            Log.w(TAG, "Could not read APK package name: ${e.message}")
        }
        return try {
            val uri: Uri = FileProvider.getUriForFile(
                context,
                "${context.packageName}.fileprovider",
                apk
            )
            val intent = Intent(Intent.ACTION_VIEW).apply {
                setDataAndType(uri, "application/vnd.android.package-archive")
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }

            // Pin the intent to the system package installer. Without this,
            // ACTION_VIEW + the APK MIME type matches any app that declares it
            // (file managers, archivers, ...) and Android shows an "Open with"
            // chooser instead of going straight to the installer.
            val installerPackages = listOf(
                "com.android.packageinstaller",   // AOSP
                "com.google.android.packageinstaller", // Google
                "com.android.permissioncontroller"     // Android 10+ (installer lives here)
            )
            val resolved = installerPackages.firstOrNull { pkg ->
                context.packageManager.resolveActivity(
                    Intent(intent).setPackage(pkg),
                    android.content.pm.PackageManager.MATCH_DEFAULT_ONLY
                ) != null
            }

            if (resolved != null) {
                intent.setPackage(resolved)
                Log.i(TAG, "Launching package installer ($resolved) for $uri")
            } else {
                // Fall back to the chooser rather than failing outright.
                Log.w(TAG, "No known package installer found; falling back to chooser")
            }

            context.startActivity(intent)
            true
        } catch (e: Exception) {
            Log.e(TAG, "Failed to launch package installer", e)
            false
        }
    }

    private fun sha256(file: File): String {
        val digest = MessageDigest.getInstance("SHA-256")
        file.inputStream().use { input ->
            val buf = ByteArray(64 * 1024)
            var read: Int
            while (input.read(buf).also { read = it } > 0) {
                digest.update(buf, 0, read)
            }
        }
        return digest.digest().joinToString("") { "%02x".format(it) }
    }
}
