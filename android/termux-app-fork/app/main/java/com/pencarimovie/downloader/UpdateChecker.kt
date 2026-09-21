package com.pencarimovie.downloader

import android.content.Context
import android.content.Intent
import android.net.Uri
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
 *   "versionName": "2.3.3",
 *   "versionCode": 20303,
 *   "apkUrl": "https://github.com/.../pencarimovie_arm64-v8a.apk",
 *   "sha256": "...",
 *   "builtAt": "2026-09-22T06:00:00Z",
 *   "commit": "abc1234"
 * }
 * ```
 *
 * Comparison uses [versionCode] (a monotonically increasing integer derived from
 * the semver parts by the build workflow), which is far more reliable than
 * string-comparing version names.
 */
object UpdateChecker {

    private const val TAG = "UpdateChecker"

    private const val VERSION_JSON_URL =
        "https://github.com/aiskendi/pencarimovie-server/releases/download/apk-latest/version.json"

    /** Where the user is sent if the direct download fails. */
    const val RELEASE_PAGE_URL =
        "https://github.com/aiskendi/pencarimovie-server/releases/tag/apk-latest"

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
    suspend fun checkForUpdate(currentVersionCode: Int): UpdateInfo? =
        withContext(Dispatchers.IO) {
            try {
                val json = fetchVersionJson() ?: return@withContext null
                val info = parse(json) ?: return@withContext null

                if (info.versionCode > currentVersionCode) {
                    Log.i(
                        TAG,
                        "Update available: ${info.versionName} (${info.versionCode}) " +
                            "> installed ($currentVersionCode)"
                    )
                    info
                } else {
                    Log.i(TAG, "Up to date (installed $currentVersionCode, remote ${info.versionCode})")
                    null
                }
            } catch (e: Exception) {
                Log.w(TAG, "Update check failed: ${e.message}")
                null
            }
        }

    private fun fetchVersionJson(): String? {
        // Cache-buster: the apk-latest asset is replaced in place, and the CDN
        // would otherwise serve the previous version.json for minutes.
        val url = "$VERSION_JSON_URL?t=${System.currentTimeMillis()}"
        val conn = (URL(url).openConnection() as HttpURLConnection).apply {
            requestMethod = "GET"
            setRequestProperty("User-Agent", "pencarimovie-downloader")
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
            val o = JSONObject(json)
            val name = o.optString("versionName").trim()
            val code = o.optInt("versionCode", -1)
            if (name.isEmpty() || code <= 0) {
                Log.w(TAG, "version.json missing versionName/versionCode")
                return null
            }
            UpdateInfo(
                versionName = name,
                versionCode = code,
                apkUrl = o.optString("apkUrl").trim(),
                sha256 = o.optString("sha256").trim(),
                builtAt = o.optString("builtAt").trim(),
                commit = o.optString("commit").trim()
            )
        } catch (e: Exception) {
            Log.w(TAG, "version.json parse failed: ${e.message}")
            null
        }
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
                setRequestProperty("User-Agent", "pencarimovie-downloader")
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
