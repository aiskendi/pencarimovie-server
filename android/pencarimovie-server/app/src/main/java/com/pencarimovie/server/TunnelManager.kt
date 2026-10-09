package com.pencarimovie.server

import android.content.Context
import android.os.Build
import okhttp3.MediaType.Companion.toMediaTypeOrNull
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.net.HttpURLConnection
import java.net.ServerSocket
import java.net.URL

/**
 * Native Cloudflare tunnel manager — a Kotlin port of backend.php's fd_tunnel_* helpers.
 *
 * It:
 *  - downloads the cloudflared binary matching the device ABI from GitHub releases;
 *  - spawns it (quick tunnel via `--url`, or a named tunnel via `run --token`);
 *  - reads the public `https://<id>.trycloudflare.com` URL from the local metrics
 *    endpoint (`/quicktunnel`) with a log-file fallback;
 *  - runs the same watchdog as the PHP build: if the state says enabled but the
 *    process died, it restarts it (with a cooldown to avoid restart loops).
 *
 * Like backend.php, a quick tunnel is surfaced on the stable relay subdomain
 * `https://<deviceId>-tunnel.pencarimovie.com` (public_url), while `tunnel_url`
 * keeps the raw trycloudflare host. The live URL is registered with the relay via
 * the same VPS + Cloudflare Worker endpoints backend.php uses.
 */
class TunnelManager(
    private val context: Context,
    private val localPort: Int,
    private val deviceIdProvider: () -> String
) {
    companion object {
        private const val TAG = "TunnelManager"
        private const val PREFS = "tunnel_state"
        private const val RESTART_COOLDOWN_MS = 20_000L
        private const val MIN_BIN_BYTES = 1_000_000L

        private val URL_RE = Regex("https://([a-z0-9-]+)\\.trycloudflare\\.com", RegexOption.IGNORE_CASE)
        private val HOSTNAME_RE =
            Regex("\"hostname\"\\s*:\\s*\"([a-z0-9-]+\\.trycloudflare\\.com)\"", RegexOption.IGNORE_CASE)
        private val INGRESS_RE =
            Regex("hostname\\s*[=:]\\s*\"?([a-z0-9-]+(?:\\.[a-z0-9-]+)+)\"?", RegexOption.IGNORE_CASE)
    }

    private val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
    private val dir = File(context.filesDir, "tunnel").apply { mkdirs() }
    private val binFile = File(dir, "cloudflared")
    private val configFile = File(dir, "config.yml")
    private val logFile = File(dir, "cloudflared.log")

    private val http = OkHttpClient()

    @Volatile private var process: Process? = null
    @Volatile private var metricsPort = 0
    @Volatile private var lastRestartAt = 0L

    private val lock = Any()

    private fun localUrl() = "http://127.0.0.1:$localPort"

    /** Pick the cloudflared build matching the device's primary ABI. */
    private fun primaryAbi(): String =
        try { Build.SUPPORTED_ABIS.firstOrNull()?.lowercase() ?: "" } catch (e: Throwable) { "" }

    /** Termux package architecture for the device's primary ABI. */
    private fun termuxArch(): String {
        val abi = primaryAbi()
        return when {
            abi.contains("arm64") -> "aarch64"
            abi.contains("armeabi") -> "arm"
            abi.contains("x86_64") -> "x86_64"
            else -> "i686"
        }
    }

    /** GitHub release asset (official static build) for the device's primary ABI. */
    private fun githubAsset(): String {
        val abi = primaryAbi()
        return when {
            abi.contains("arm64") -> "cloudflared-linux-arm64"
            abi.contains("armeabi") -> "cloudflared-linux-arm"
            abi.contains("x86_64") -> "cloudflared-linux-amd64"
            abi.contains("x86") -> "cloudflared-linux-386"
            else -> "cloudflared-linux-arm64"
        }
    }

    private fun ensureBinary(): String {
        // Termux's cloudflared links against bionic libc, so it resolves DNS via
        // netd. The official static build cannot: Android has no /etc/resolv.conf
        // and Go offers no way to override the path. The Termux build needs a
        // linker with DT_GNU_HASH support (Android 6+/API 23), so older devices
        // fall back to the static build.
        val useTermux = Build.VERSION.SDK_INT >= 23
        val key = if (useTermux) "termux-" + termuxArch() else "github-" + githubAsset()
        if (binFile.isFile && binFile.length() > MIN_BIN_BYTES &&
            prefs.getString("bin_asset", "") == key
        ) {
            binFile.setExecutable(true, false)
            return ""
        }
        return if (useTermux) ensureTermuxBinary(key) else ensureGithubBinary(key)
    }

    /** Download Termux's Android-linked cloudflared and extract the binary from the .deb. */
    private fun ensureTermuxBinary(key: String): String {
        val arch = termuxArch()
        val base = "https://packages.termux.dev/apt/termux-main"
        DebugLog.log(TAG, "Installing cloudflared from Termux ($arch)")
        return try {
            val index = httpGetText("$base/dists/stable/main/binary-$arch/Packages")
            val fn = Regex("Package: cloudflared\r?\n(?:.*\r?\n)*?Filename: (\\S+)")
                .find(index)?.groupValues?.get(1)
                ?: return "cloudflared not found in the Termux package index."
            val deb = File(dir, "cloudflared.deb")
            downloadWithFallback("$base/$fn", deb)
            val tmp = File(dir, "cloudflared.extract")
            val ok = try { extractCloudflaredFromDeb(deb, tmp) } catch (e: Exception) {
                DebugLog.log(TAG, "deb extract failed: ${e.message}"); false
            }
            deb.delete()
            if (!ok || tmp.length() < MIN_BIN_BYTES) {
                tmp.delete()
                return "Failed to extract cloudflared from the Termux package."
            }
            if (binFile.exists()) binFile.delete()
            if (!tmp.renameTo(binFile)) { tmp.delete(); return "Could not store the cloudflared binary." }
            binFile.setExecutable(true, false)
            prefs.edit().putString("bin_asset", key).apply()
            DebugLog.log(TAG, "cloudflared ready (${binFile.length()} bytes)")
            ""
        } catch (e: Exception) {
            "Failed to install cloudflared: ${e.message}"
        }
    }

    private fun ensureGithubBinary(key: String): String {
        val asset = githubAsset()
        val url = "https://github.com/cloudflare/cloudflared/releases/latest/download/$asset"
        DebugLog.log(TAG, "Downloading cloudflared ($asset)")
        val tmp = File(dir, "cloudflared.download")
        return try {
            downloadWithFallback(url, tmp)
            if (tmp.length() < MIN_BIN_BYTES) {
                val n = tmp.length(); tmp.delete()
                return "cloudflared download was truncated ($n bytes)."
            }
            if (binFile.exists()) binFile.delete()
            if (!tmp.renameTo(binFile)) { tmp.delete(); return "Could not store the cloudflared binary." }
            binFile.setExecutable(true, false)
            prefs.edit().putString("bin_asset", key).apply()
            DebugLog.log(TAG, "cloudflared ready (${binFile.length()} bytes)")
            ""
        } catch (e: Exception) {
            "Failed to download cloudflared: ${e.message}"
        }
    }

    private fun downloadWithFallback(url: String, dst: File) {
        try {
            downloadTo(url, dst, http)
        } catch (e: Exception) {
            // Old Android trust stores can miss the CDN CA ("Trust anchor ... not
            // found"); retry with a lenient context (the binary size is checked).
            DebugLog.log(TAG, "TLS verify failed (${e.message}); retrying with lenient trust")
            downloadTo(url, dst, lenientHttp)
        }
    }

    private fun httpGetText(url: String): String {
        val req = Request.Builder().url(url).build()
        http.newCall(req).execute().use { r ->
            if (!r.isSuccessful) throw java.io.IOException("HTTP ${r.code}")
            return r.body?.string() ?: ""
        }
    }

    /**
     * Extract ./…/bin/cloudflared out of a Termux .deb. Parsed inline (ar →
     * data.tar.xz → tar) because commons-compress needs java.nio.file (API 26+).
     */
    private fun extractCloudflaredFromDeb(deb: File, out: File): Boolean {
        java.io.FileInputStream(deb).use { fis ->
            val magic = ByteArray(8)
            if (!readFully(fis, magic, 8) || String(magic, Charsets.US_ASCII) != "!<arch>\n") return false
            val hdr = ByteArray(60)
            while (readFully(fis, hdr, 60)) {
                val name = String(hdr, 0, 16, Charsets.US_ASCII).trim().trimEnd('/')
                val size = String(hdr, 48, 10, Charsets.US_ASCII).trim().toLongOrNull() ?: return false
                if (name.startsWith("data.tar")) {
                    if (size !in 1..64_000_000L) return false
                    val buf = ByteArray(size.toInt())
                    if (!readFully(fis, buf, buf.size)) return false
                    org.tukaani.xz.XZInputStream(java.io.ByteArrayInputStream(buf)).use { xz ->
                        return readTarForCloudflared(xz, out)
                    }
                }
                if (!skipFully(fis, size + (size % 2))) return false
            }
        }
        return false
    }

    /** Scan an uncompressed tar stream and write the entry ending in bin/cloudflared. */
    private fun readTarForCloudflared(input: java.io.InputStream, out: File): Boolean {
        val block = ByteArray(512)
        var longName: String? = null
        while (readFully(input, block, 512)) {
            if (block.all { it == 0.toByte() }) return false // end-of-archive
            val name = (longName ?: String(block, 0, 100, Charsets.UTF_8).trimEnd('\u0000', ' ')).also { longName = null }
            val sizeStr = String(block, 124, 12, Charsets.US_ASCII).trim('\u0000', ' ')
            val size = try { if (sizeStr.isEmpty()) 0L else sizeStr.toLong(8) } catch (e: Exception) { 0L }
            val type = block[156].toInt().toChar()
            val padded = (size + 511) / 512 * 512
            when (type) {
                'L' -> { // GNU long name
                    if (size !in 1..4096L) return false
                    val nb = ByteArray(size.toInt())
                    if (!readFully(input, nb, nb.size)) return false
                    longName = String(nb, Charsets.UTF_8).trimEnd('\u0000')
                    if (!skipFully(input, padded - size)) return false
                }
                'x', 'g' -> { if (!skipFully(input, padded)) return false } // pax headers
                else -> {
                    if (name.endsWith("bin/cloudflared")) {
                        java.io.FileOutputStream(out).use { os ->
                            var remaining = size
                            val buf = ByteArray(64 * 1024)
                            while (remaining > 0) {
                                val n = input.read(buf, 0, minOf(buf.size.toLong(), remaining).toInt())
                                if (n < 0) break
                                os.write(buf, 0, n)
                                remaining -= n
                            }
                        }
                        return out.length() > MIN_BIN_BYTES
                    }
                    if (!skipFully(input, padded)) return false
                }
            }
        }
        return false
    }

    private fun readFully(input: java.io.InputStream, buf: ByteArray, len: Int): Boolean {
        var off = 0
        while (off < len) {
            val n = input.read(buf, off, len - off)
            if (n < 0) return false
            off += n
        }
        return true
    }

    private fun skipFully(input: java.io.InputStream, len: Long): Boolean {
        var remaining = len
        val tmp = ByteArray(8192)
        while (remaining > 0) {
            val n = input.read(tmp, 0, minOf(tmp.size.toLong(), remaining).toInt())
            if (n < 0) return false
            remaining -= n
        }
        return true
    }

    private fun downloadTo(url: String, dst: File, client: OkHttpClient) {
        val req = Request.Builder().url(url).build()
        client.newCall(req).execute().use { resp ->
            if (!resp.isSuccessful) throw java.io.IOException("HTTP ${resp.code}")
            val body = resp.body ?: throw java.io.IOException("empty response body")
            body.byteStream().use { input -> dst.outputStream().use { out -> input.copyTo(out) } }
        }
    }

    /** TLS context that accepts the CDN chain on old Android; used only as a download fallback. */
    private val lenientHttp: OkHttpClient by lazy {
        try {
            val trustAll = arrayOf<javax.net.ssl.TrustManager>(object : javax.net.ssl.X509TrustManager {
                override fun checkClientTrusted(chain: Array<java.security.cert.X509Certificate>, authType: String) {}
                override fun checkServerTrusted(chain: Array<java.security.cert.X509Certificate>, authType: String) {}
                override fun getAcceptedIssuers(): Array<java.security.cert.X509Certificate> = arrayOf()
            })
            val ctx = javax.net.ssl.SSLContext.getInstance("TLS")
            ctx.init(null, trustAll, java.security.SecureRandom())
            OkHttpClient.Builder()
                .sslSocketFactory(ctx.socketFactory, trustAll[0] as javax.net.ssl.X509TrustManager)
                .hostnameVerifier { _, _ -> true }
                .build()
        } catch (e: Exception) {
            http
        }
    }

    private fun writeConfig() {
        configFile.writeText(
            "ingress:\n" +
                "  - service: ${localUrl()}\n" +
                "    originRequest:\n" +
                "      connectTimeout: 10s\n" +
                "      tcpKeepAlive: 30s\n" +
                "      keepAliveTimeout: 90s\n" +
                "      keepAliveConnections: 100\n" +
                "      noTLSVerify: true\n" +
                "      http2Origin: false\n"
        )
    }

    private fun pickMetricsPort(): Int =
        try { ServerSocket(0).use { it.localPort } } catch (e: Exception) { 20241 }

    private fun spawn(token: String): String {
        val err = ensureBinary()
        if (err.isNotEmpty()) return err
        writeConfig()
        metricsPort = pickMetricsPort()
        val cmd = if (token.isNotBlank()) {
            listOf(
                binFile.absolutePath, "tunnel",
                "--metrics", "127.0.0.1:$metricsPort",
                "--logfile", logFile.absolutePath,
                "--edge-ip-version", "auto", "--grace-period", "15s",
                "--no-autoupdate", "--retries", "99",
                "run", "--token", token
            )
        } else {
            listOf(
                binFile.absolutePath, "tunnel",
                "--url", localUrl(),
                "--config", configFile.absolutePath,
                "--metrics", "127.0.0.1:$metricsPort",
                "--logfile", logFile.absolutePath,
                "--edge-ip-version", "auto", "--grace-period", "15s",
                "--no-autoupdate", "--retries", "99"
            )
        }
        return try {
            if (logFile.exists()) logFile.delete()
            val pb = ProcessBuilder(cmd)
            pb.directory(dir)
            pb.redirectErrorStream(true)
            val env = pb.environment()
            if (!env.containsKey("TUNNEL_TRANSPORT_PROTOCOL")) env["TUNNEL_TRANSPORT_PROTOCOL"] = "auto"
            // Go's crypto/x509 honours SSL_CERT_FILE for its system cert pool.
            // Android ships no /etc/ssl/certs bundle, so cloudflared cannot verify
            // the Cloudflare edge certificate unless we point it at our own copy.
            env["SSL_CERT_FILE"] = ensureCaBundle().absolutePath
            val p = pb.start()
            process = p
            drainToLog(p)
            ""
        } catch (e: Exception) {
            "Failed to start cloudflared: ${e.message}"
        }
    }

    /** Pipe the child's merged stdout/stderr into the log file (ProcessBuilder.Redirect is API 26+). */
    private fun drainToLog(p: Process) {
        val t = Thread {
            try {
                p.inputStream.bufferedReader().use { r ->
                    logFile.outputStream().bufferedWriter().use { w ->
                        val buf = CharArray(4096)
                        while (true) {
                            val n = r.read(buf)
                            if (n <= 0) break
                            w.write(buf, 0, n)
                            w.flush()
                        }
                    }
                }
            } catch (e: Exception) {}
        }
        t.isDaemon = true
        t.start()
    }

    /** Copy the bundled Mozilla CA roots to the app dir for cloudflared's SSL_CERT_FILE. */
    private fun ensureCaBundle(): File {
        val f = File(dir, "cacert.pem")
        if (f.isFile && f.length() > 10_000L) return f
        try {
            context.assets.open("cacert.pem").use { input ->
                f.outputStream().use { out -> input.copyTo(out) }
            }
        } catch (e: Exception) {
            DebugLog.log(TAG, "CA bundle copy failed: ${e.message}")
        }
        return f
    }

    private fun isAlive(): Boolean {
        val p = process ?: return false
        // Process.isAlive() is API 26+; exitValue() is the portable liveness check
        // (throws IllegalThreadStateException while the process is still running).
        return try {
            p.exitValue(); false
        } catch (e: IllegalThreadStateException) {
            true
        } catch (e: Throwable) {
            false
        }
    }

    /** Process.pid() is API 24+ / absent from this compileSdk, so resolve it reflectively. */
    private fun pidOf(): Int = try {
        val p = process ?: return 0
        (p.javaClass.getMethod("pid").invoke(p) as? Int) ?: 0
    } catch (e: Throwable) { 0 }

    private fun killProcess() {
        val p = process
        process = null
        if (p != null) {
            try { p.destroy() } catch (e: Throwable) {}
            // destroyForcibly() is API 26+; invoke it reflectively when present.
            try { p.javaClass.getMethod("destroyForcibly").invoke(p) } catch (e: Throwable) {}
        }
    }

    private fun readLog(): String = try {
        if (logFile.isFile) logFile.readText().takeLast(20000) else ""
    } catch (e: Exception) { "" }

    private fun httpGet(url: String): String = try {
        val conn = URL(url).openConnection() as HttpURLConnection
        conn.connectTimeout = 1500
        conn.readTimeout = 1500
        conn.requestMethod = "GET"
        val text = if (conn.responseCode in 200..299) conn.inputStream.bufferedReader().use { it.readText() } else ""
        conn.disconnect()
        text
    } catch (e: Exception) { "" }

    /** Real quick-tunnel hostnames look like `three-words-1234.trycloudflare.com`; ignore service hosts like api. */
    private fun isValidTunnelHost(host: String): Boolean {
        val h = host.lowercase().removeSuffix(".trycloudflare.com")
        if (h.isEmpty() || h in setOf("api", "www", "cdn", "dash", "one", "developers", "support")) return false
        return h.length >= 8 && h.contains('-')
    }

    private fun findTunnelUrl(text: String): String {
        if (text.isEmpty()) return ""
        HOSTNAME_RE.find(text)?.let {
            val h = it.groupValues[1].lowercase()
            if (isValidTunnelHost(h)) return "https://$h"
        }
        for (m in URL_RE.findAll(text)) {
            val h = m.groupValues[1].lowercase()
            if (isValidTunnelHost(h)) return "https://$h.trycloudflare.com"
        }
        return ""
    }

    private fun readQuickTunnelUrl(): String {
        if (metricsPort <= 0) return ""
        for (path in listOf("/quicktunnel", "/metrics")) {
            val body = httpGet("http://127.0.0.1:$metricsPort$path")
            if (body.isNotEmpty()) {
                val url = findTunnelUrl(body)
                if (url.isNotEmpty()) return url
            }
        }
        return ""
    }

    private fun waitReady(timeoutSec: Int): Boolean {
        val deadline = System.currentTimeMillis() + timeoutSec * 1000L
        while (System.currentTimeMillis() < deadline) {
            if (!isAlive()) return false
            val body = httpGet("http://127.0.0.1:$metricsPort/ready")
            if (body.contains("\"status\":200")) return true
            try { Thread.sleep(300) } catch (e: Exception) {}
        }
        return false
    }

    private fun waitForUrl(timeoutSec: Int): String {
        val deadline = System.currentTimeMillis() + timeoutSec * 1000L
        while (System.currentTimeMillis() < deadline) {
            if (!isAlive() && readLog().isEmpty()) break
            val live = readQuickTunnelUrl()
            if (live.isNotEmpty()) return live
            val fromLog = findTunnelUrl(readLog())
            if (fromLog.isNotEmpty()) return fromLog
            try { Thread.sleep(500) } catch (e: Exception) {}
        }
        val live = readQuickTunnelUrl()
        return if (live.isNotEmpty()) live else findTunnelUrl(readLog())
    }

    private fun extractToken(raw: String): String {
        var t = raw.trim().trim('"')
        if (t.isEmpty()) return ""
        if (t.startsWith("{")) {
            try { t = JSONObject(t).optString("tunnel_token", "").trim() } catch (e: Exception) {}
        }
        if (t.startsWith("http://") || t.startsWith("https://")) {
            t = t.substringAfterLast("/").substringBefore("?").trim()
        }
        return t
    }

    private fun parseIngressDomains(): List<String> {
        val found = LinkedHashSet<String>()
        for (m in INGRESS_RE.findAll(readLog())) {
            val d = m.groupValues[1].lowercase()
            if (d.contains("trycloudflare") || d.contains("argotunnel") || d == "127.0.0.1") continue
            found.add(d)
        }
        return found.toList()
    }

    /** Tell the pencarimovie relay (VPS + Cloudflare Worker) where the quick tunnel lives. */
    private fun registerWorker(quickUrl: String) {
        val body = JSONObject().put("shortId", deviceIdProvider()).put("tunnelUrl", quickUrl).toString()
        for (endpoint in listOf(
            "https://pencarimovie.com/wp-json/pencarimovie-server/v1/tunnel/register",
            "https://tunnel.pencarimovie.com/api/tunnel/register"
        )) {
            postJson(endpoint, body)
        }
    }

    private fun postJson(endpoint: String, body: String) {
        val media = "application/json; charset=utf-8".toMediaTypeOrNull()
        try {
            http.newCall(Request.Builder().url(endpoint).post(body.toRequestBody(media)).build()).execute().use { }
        } catch (e: Exception) {
            // Old Android trust stores can reject the relay's CDN cert; retry lenient.
            DebugLog.log(TAG, "relay register TLS failed ($endpoint): ${e.message}; retrying lenient")
            try {
                lenientHttp.newCall(Request.Builder().url(endpoint).post(body.toRequestBody(media)).build()).execute().use { }
            } catch (e2: Exception) {
                DebugLog.log(TAG, "relay register failed ($endpoint): ${e2.message}")
            }
        }
    }

    private fun statusJson(
        running: Boolean,
        quickUrl: String,
        token: String,
        customDomains: JSONArray,
        message: String
    ): JSONObject {
        val subdomain = deviceIdProvider()
        val customDomain = if (customDomains.length() > 0) customDomains.optString(0) else ""
        // Mirror backend.php's fd_get_tunnel_status(): quick tunnels are surfaced
        // on the stable <deviceId>-tunnel.pencarimovie.com relay subdomain, named
        // tunnels on their own hostname; tunnel_url keeps the raw trycloudflare host.
        val publicUrl = when {
            !running -> ""
            token.isNotEmpty() -> if (customDomain.isNotEmpty()) "https://$customDomain" else ""
            quickUrl.isNotEmpty() -> "https://$subdomain-tunnel.pencarimovie.com"
            else -> ""
        }
        val tunnelUrl = if (token.isNotEmpty()) publicUrl else if (running) quickUrl else ""
        val activeUrl = if (publicUrl.isNotEmpty()) publicUrl else tunnelUrl
        val enabled = running && activeUrl.isNotEmpty()
        val manifest = if (enabled) activeUrl.trimEnd('/') + "/manifest.json" else ""
        return JSONObject().apply {
            put("ok", 1)
            put("enabled", enabled)
            put("running", running)
            put("pid", if (running) pidOf() else 0)
            put("tunnel_token", token)
            put("tunnel_url", tunnelUrl)
            put("public_url", publicUrl)
            put("custom_domain", customDomain)
            put("custom_domains", customDomains)
            put("device_id", subdomain)
            put("manifest_url", manifest)
            put("local_port", localPort)
            put("started_at", prefs.getLong("started_at", 0L))
            put("message", message)
        }
    }

    fun status(): JSONObject {
        synchronized(lock) {
            val enabled = prefs.getBoolean("enabled", false)
            var alive = isAlive()
            if (!alive && enabled && System.currentTimeMillis() - lastRestartAt > RESTART_COOLDOWN_MS) {
                lastRestartAt = System.currentTimeMillis()
                DebugLog.log(TAG, "Watchdog: cloudflared not running, restarting")
                restartLocked()
                alive = isAlive()
            }
            val token = prefs.getString("token", "") ?: ""
            var url = prefs.getString("url", "") ?: ""
            if (alive) {
                val live = readQuickTunnelUrl()
                if (live.isNotEmpty()) url = live
                else if (url.isEmpty()) findTunnelUrl(readLog())?.let { url = it }
                if (url.isNotEmpty() && url != (prefs.getString("url", "") ?: "")) {
                    prefs.edit().putBoolean("enabled", true).putString("url", url).apply()
                    if (token.isEmpty()) registerWorker(url)
                }
            }
            val domains = if (alive && token.isNotEmpty()) JSONArray(parseIngressDomains()) else JSONArray()
            val sub = deviceIdProvider()
            val message = when {
                alive && token.isNotEmpty() && domains.length() > 0 -> "Live on https://" + domains.optString(0)
                alive && token.isNotEmpty() -> "Named Cloudflare Tunnel connected."
                alive && url.isNotEmpty() -> "Live on custom subdomain: https://$sub-tunnel.pencarimovie.com"
                alive -> "cloudflared is running but the public URL is not ready yet."
                else -> "Cloudflare tunnel is off."
            }
            return statusJson(alive, url, token, domains, message)
        }
    }

    private fun restartLocked() {
        var token = prefs.getString("token", "") ?: ""
        val err = spawn(token)
        if (err.isNotEmpty()) {
            DebugLog.log(TAG, "Watchdog restart failed: $err")
            return
        }
        if (token.isNotEmpty()) {
            waitReady(20)
            return
        }
        val url = waitForUrl(60)
        if (url.isEmpty()) {
            DebugLog.log(TAG, "Watchdog restart: no URL")
            killProcess()
            prefs.edit().putBoolean("enabled", false).putString("url", "").apply()
        } else {
            prefs.edit().putBoolean("enabled", true).putString("url", url).apply()
            registerWorker(url)
            DebugLog.log(TAG, "Watchdog restart live at $url")
        }
    }

    fun enable(rawToken: String): JSONObject {
        synchronized(lock) {
            var token = extractToken(rawToken)
            if (token.isEmpty()) token = prefs.getString("token", "") ?: ""
            else prefs.edit().putString("token", token).apply()

            killProcess()
            prefs.edit().putBoolean("enabled", false).putString("url", "").apply()

            val err = spawn(token)
            if (err.isNotEmpty()) {
                return JSONObject().put("ok", 0).put("enabled", false).put("message", err)
            }
            val startedAt = System.currentTimeMillis() / 1000L
            prefs.edit().putLong("started_at", startedAt).apply()

            if (token.isNotEmpty()) {
                val ready = waitReady(15)
                val domains = JSONArray(parseIngressDomains())
                val domain = if (domains.length() > 0) domains.optString(0) else ""
                val url = if (domain.isNotEmpty()) "https://$domain" else ""
                prefs.edit().putBoolean("enabled", true).putString("url", url).apply()
                val msg = when {
                    domain.isNotEmpty() -> "Connected to $url"
                    ready -> "Named Cloudflare Tunnel connected."
                    else -> "cloudflared started; connecting to the edge…"
                }
                return statusJson(isAlive(), url, token, domains, msg)
            }

            val url = waitForUrl(90)
            if (url.isEmpty()) {
                val tail = readLog().takeLast(600).replace(Regex("\\s+"), " ")
                killProcess()
                prefs.edit().putBoolean("enabled", false).putString("url", "").apply()
                val hint = if (tail.isNotEmpty()) " Log: $tail" else ""
                return JSONObject().put("ok", 0).put("enabled", false)
                    .put("message", "cloudflared started but no trycloudflare.com URL appeared.$hint")
            }
            prefs.edit().putBoolean("enabled", true).putString("url", url).apply()
            registerWorker(url)
            DebugLog.log(TAG, "Tunnel live at $url")
            val sub = deviceIdProvider()
            return statusJson(isAlive(), url, token, JSONArray(), "Live on custom subdomain: https://$sub-tunnel.pencarimovie.com")
        }
    }

    fun disable(): JSONObject {
        synchronized(lock) {
            killProcess()
            registerWorker("")
            prefs.edit().putBoolean("enabled", false).putString("url", "").apply()
            val token = prefs.getString("token", "") ?: ""
            return JSONObject().apply {
                put("ok", 1)
                put("enabled", false)
                put("running", false)
                put("pid", 0)
                put("tunnel_token", token)
                put("tunnel_url", "")
                put("public_url", "")
                put("manifest_url", "")
                put("message", "Cloudflare tunnel stopped.")
            }
        }
    }

    /** Stop the tunnel process (called when the server service is torn down). */
    fun stop() {
        synchronized(lock) {
            killProcess()
            if (prefs.getBoolean("enabled", false)) registerWorker("")
            prefs.edit().putBoolean("enabled", false).putString("url", "").apply()
        }
    }
}
