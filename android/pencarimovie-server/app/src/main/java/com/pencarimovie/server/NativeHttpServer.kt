package com.pencarimovie.server

import android.content.Context
import android.util.Log
import fi.iki.elonen.NanoHTTPD
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.Request
import org.drinkless.tdlib.TdApi
import org.json.JSONArray
import org.json.JSONObject
import android.util.Base64
import java.io.ByteArrayInputStream
import java.io.InputStream
import java.io.OutputStream
import java.net.Inet4Address
import java.net.NetworkInterface
import java.net.URLDecoder
import java.net.URLEncoder
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit

/**
 * Pure native embedded HTTP server for PencariMovie Server.
 * Replaces FrankenPHP/PHP runtime by directly serving:
 * - Static frontend assets from assets/web/ (public/)
 * - Core API endpoints (/api/auth, /api/session, /api/proxy-stream, /api/catalog-settings)
 * - Stremio Addon Protocol (/manifest.json, /catalog, /stream, /meta)
 * - Zero-disk on-the-fly streaming (/api/download) via TDLib
 */
class NativeHttpServer(
    private val context: Context,
    private val botPool: BotPool,
    port: Int = 8088
) : NanoHTTPD("0.0.0.0", port) {

    /** The active pool bot (falls back to the primary manager). */
    private val tdLibManager: TdLibManager get() = botPool.activeManager()

    companion object {
        private const val TAG = "NativeHttpServer"
        private const val WP_API_BASE = "https://pencarimovie.com/wp-json/pencarimovie-server/v1"

        /** Routes reachable without a token — mirrors backend.php's $alwaysPublicApi. */
        private val PUBLIC_API = setOf(
            "/api/version",
            "/api/clock-check",
            "/api/auth/status",
            "/api/auth/login"
        )

        /**
         * Minimal standalone password page served to unauthenticated BROWSER
         * navigations (mirrors backend.php fd_serve_auth_gate_html()). Deliberately
         * does not load app.js/dashboard CSS so no dashboard markup leaks, and posts
         * to /api/auth/login which sets the pm_auth cookie before reloading.
         */
        private val AUTH_GATE_HTML = """
<!DOCTYPE html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>PencariMovie Server</title>
<style>
body{margin:0;min-height:100vh;display:flex;align-items:center;justify-content:center;background:#141414;color:#fff;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif}
.card{width:100%;max-width:360px;padding:28px;background:#1c1c1c;border-radius:14px;box-shadow:0 8px 32px rgba(0,0,0,.5)}
h1{margin:0 0 6px;font-size:1.3rem}
p{margin:0 0 18px;color:#9a9a9a;font-size:.86rem}
input{width:100%;box-sizing:border-box;padding:12px;border-radius:8px;border:1px solid #333;background:#111;color:#fff;font-size:1rem}
button{width:100%;margin-top:12px;padding:12px;border:0;border-radius:8px;background:#ff6b35;color:#fff;font-size:1rem;font-weight:600;cursor:pointer}
button:disabled{opacity:.6;cursor:default}
.err{margin-top:10px;color:#ff5c5c;font-size:.84rem;min-height:1.1em}
</style></head><body>
<div class="card">
<h1>PencariMovie Server</h1>
<p>Enter server password to continue. (Default: <code>123456</code>)</p>
<form id="f">
<input id="pw" type="password" placeholder="Password" autocomplete="current-password" autofocus>
<button id="b" type="submit">Connect</button>
<div class="err" id="e"></div>
</form>
<div style="margin-top:16px;padding:10px 12px;background:rgba(255,255,255,.04);border:1px solid rgba(255,255,255,.08);border-radius:8px;font-size:.78rem;color:#aaa;line-height:1.45;text-align:left">
PencariMovie Server is completely free &mdash; we never sell access or subscriptions. If this is not your personal server, you can easily run your own for free on your phone, PC, or TV &mdash; get the installer at <a href="https://telegra.my" target="_blank" rel="noopener" style="color:#ff6b35;text-decoration:underline">telegra.my</a>.
</div>
</div>
<script>
document.getElementById('f').addEventListener('submit', async function(ev){
  ev.preventDefault();
  var b=document.getElementById('b'), e=document.getElementById('e');
  b.disabled=true; e.textContent='';
  try{
    var r=await fetch('/api/auth/login',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({password:document.getElementById('pw').value})});
    var d=await r.json();
    if(d.ok){
      if(d.token){ try{ localStorage.setItem('pm.auth', d.token); }catch(_){} }
      location.reload();
      return;
    }
    e.textContent=d.message||'Wrong password';
  }catch(err){e.textContent='Connection failed';}
  b.disabled=false;
});
</script></body></html>
""".trimIndent()
    }

    private val httpClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(15, TimeUnit.SECONDS)
        .build()

    // Longer-timeout client for the WordPress resolve-file call, which can be slow
    // on a cold cache (Telegram background relay). Matches backend.php's tolerant retry.
    private val resolveClient = httpClient.newBuilder()
        .connectTimeout(20, TimeUnit.SECONDS)
        .readTimeout(45, TimeUnit.SECONDS)
        .build()

    init {
        DebugLog.init(context)
        DebugLog.log("HTTP", "NativeHttpServer created on port $port")
    }

    private val auth by lazy { AuthManager(context) }

    override fun serve(session: IHTTPSession): Response {
        val rawUri = session.uri
        val method = session.method
        DebugLog.log("HTTP", "--> $method $rawUri")

        if (method == Method.OPTIONS) {
            return newFixedLengthResponse(Response.Status.NO_CONTENT, MIME_PLAINTEXT, "").apply {
                addCorsHeaders(this)
            }
        }

        // Access tokens may arrive as a /<token>/… or /t/<token>/… path segment;
        // strip it so the normal routes match (mirrors fd_strip_token_prefix()).
        val uri = auth.stripTokenPrefix(rawUri)

        // Remote requests require the token; localhost / LAN bypass (mirrors
        // backend.php fd_is_authenticated()).
        //
        // Only the routes that must work BEFORE login are exempt (backend.php's
        // $alwaysPublicApi). The old blanket `uri.startsWith("/api/auth/")` left
        // /api/auth/reset and /api/auth/token/rotate open to anyone who could
        // reach the tunnel — a remote password reset + token handout.
        if (!auth.isLocalRequest(session) &&
            !isAuthenticated(session, rawUri) &&
            uri !in PUBLIC_API
        ) {
            return authRequired(session, uri)
        }

        return try {
            when {
                uri == "/api/version" -> handleVersion()
                uri == "/api/lan-ip" -> handleLanIp()
                uri == "/api/session" -> handleSession()
                uri == "/api/provision" -> handleProvision(session)
                uri.startsWith("/api/auth/") -> handleAuth(uri, session)
                uri == "/api/botlogin" -> handleBotLogin(session)
                uri == "/api/botlogout" -> handleBotLogout()
                uri.startsWith("/api/bots") -> handleBots(uri, session)
                uri == "/api/clock-check" -> handleClockCheck()
                uri == "/api/tunnel/status" || uri.startsWith("/api/tunnel/") -> handleTunnel(uri, session)
                uri == "/api/country" -> handleCountry(session)
                uri == "/api/catalog-settings" -> handleCatalogSettings(session)
                uri == "/api/proxy-stream" -> handleProxyStream(session)
                uri == "/api/resolve-shortcode" -> handleResolveShortcode(session)
                uri == "/api/warmup-resolve" -> jsonResponse(JSONObject().put("ok", 1))
                uri == "/api/debug-mode" -> handleDebugMode(session)
                uri == "/api/debug-logs" -> handleDebugLogs(session)
                uri == "/api/validate-manifest" -> jsonResponse(JSONObject().put("ok", 1).put("valid", true))
                uri == "/manifest.json" -> handleManifest(session)
                uri == "/configure" || uri == "/configure/" -> handleConfigure()
                uri.startsWith("/stream/") -> handleStremioStream(uri, session)
                uri.startsWith("/catalog/") -> handleStremioCatalog(uri, session)
                uri.startsWith("/meta/") -> handleStremioMeta(uri, session)
                uri.startsWith("/api/download") -> handleDownload(uri, session)
                else -> serveStaticAsset(uri)
            }
        } catch (e: Exception) {
            DebugLog.log("HTTP", "EXCEPTION $uri: ${e.message}")
            Log.e(TAG, "Error handling request: $uri", e)
            jsonResponse(JSONObject().put("ok", 0).put("message", e.message ?: "Server Error"), Response.Status.INTERNAL_ERROR)
        }
    }

    /**
     * GET  /api/debug-mode            -> read current toggle state
     * POST /api/debug-mode?enabled=1  -> enable  (enabled=0 disables, toggle=1 flips)
     */
    private fun handleDebugMode(session: IHTTPSession): Response {
        val params = session.parameters
        val toggle = params["toggle"]?.firstOrNull()
        val enabledParam = params["enabled"]?.firstOrNull()
        if (session.method == Method.POST || toggle != null || enabledParam != null) {
            val newValue = when {
                toggle == "1" || toggle == "true" -> !DebugLog.isEnabled
                enabledParam == "1" || enabledParam == "true" -> true
                enabledParam == "0" || enabledParam == "false" || toggle == "0" -> false
                else -> DebugLog.isEnabled
            }
            DebugLog.setEnabled(context, newValue)
        }
        return jsonResponse(JSONObject().apply {
            put("ok", 1)
            put("enabled", DebugLog.isEnabled)
            put("count", DebugLog.count())
        })
    }

    /**
     * GET /api/debug-logs[?clear=1] -> dump the in-memory ring buffer.
     */
    private fun handleDebugLogs(session: IHTTPSession): Response {
        val clearAfter = session.parameters["clear"]?.firstOrNull() == "1"
        val logs = DebugLog.dumpArray()
        if (clearAfter) DebugLog.clear()
        return jsonResponse(JSONObject().apply {
            put("ok", 1)
            put("enabled", DebugLog.isEnabled)
            put("count", logs.length())
            put("logs", logs)
        })
    }

    private fun handleVersion(): Response {
        // app.js checkVersion() reads current_version / minimum_version /
        // update_needed / sponsor from here, and the web stream-modal sponsored
        // card is driven by `sponsor` — so it must be included (mirrors
        // backend.php fd_check_version).
        val currentVersion = BuildConfig.VERSION_NAME
        val json = JSONObject().apply {
            put("ok", 1)
            put("version", currentVersion)
            put("app_name", "PencariMovie Server")
            put("architecture", "Native Kotlin + TDLib (32/64-bit)")
            put("current_version", currentVersion)
        }
        val wp = fetchWpVersion()
        if (wp != null) {
            val minV = wp.optString("min_version", "").trim()
            if (minV.isNotEmpty()) json.put("minimum_version", minV)
            json.put("update_url", wp.optString("update_url", ""))
            json.put("release_notes", wp.optString("release_notes", ""))
            // The WP version gate targets the PHP build; this native app ships
            // its own updater, so never force-update the web UI from it.
            json.put("update_needed", false)
            wp.optJSONObject("sponsor")?.let { json.put("sponsor", it) }
        }
        return jsonResponse(json)
    }

    private fun handleLanIp(): Response {
        val ips = JSONArray()
        try {
            val interfaces = NetworkInterface.getNetworkInterfaces()
            while (interfaces.hasMoreElements()) {
                val iface = interfaces.nextElement()
                if (iface.isLoopback || !iface.isUp) continue
                val addresses = iface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val addr = addresses.nextElement()
                    if (addr is Inet4Address && !addr.isLoopbackAddress) {
                        ips.put(addr.hostAddress)
                    }
                }
            }
        } catch (ignored: Exception) {}

        val primaryIp = if (ips.length() > 0) ips.getString(0) else "127.0.0.1"
        // Mirrors backend.php's /api/lan-ip shape exactly: {ok, lan_ip, port}.
        val json = JSONObject().apply {
            put("ok", 1)
            put("lan_ip", primaryIp)
            put("port", listeningPort)
        }
        return jsonResponse(json)
    }

    private fun handleSession(): Response {
        // Report the ACTIVE bot — after removing the built-in primary bot the live
        // session lives in a pooled bot, so tdLibManager alone would report
        // has_session:false while /api/bots still lists working bots.
        val active = botPool.activeManager()
        val ready = active.hasSession
        val pool = botPool.list()
        // Runtime preflight: an unsupported ABI / unwritable storage can never be
        // fixed by entering a bot token, so report it here and let the frontend
        // show a fatal error instead of the bot-token gate.
        val env = environmentPreflight()
        val json = JSONObject().apply {
            put("ok", 1)
            put("version", BuildConfig.VERSION_NAME)
            put("has_session", ready)
            put("is_provisioning", active.isProvisioningInProgress)
            put("bot_id", if (ready) active.botId else "")
            put("bot_username", if (ready) active.botUsername else "")
            put("bot_name", if (ready) active.botName else "")
            put("api_secret", if (ready) active.apiSecret else "")
            put("device_id", deviceId())
            put("bot_count", botPool.count())
            put("bot_pool", pool)
            put("environment_ok", env.ok)
            put("environment_fatal", env.fatal)
            put("environment_problems", env.problems)
            put("environment_hints", env.hints)
        }
        return jsonResponse(json)
    }

    /** Stable short device id (mirrors backend.php fd_get_device_id format). */
    private fun deviceId(): String {
        val prefs = context.getSharedPreferences("pencarimovie_server", Context.MODE_PRIVATE)
        var id = prefs.getString("device_id", null)
        if (id == null || id.isEmpty()) {
            id = (1..6).joinToString("") { "0123456789abcdef".random().toString() }
            prefs.edit().putString("device_id", id).apply()
        }
        return id
    }

    private fun handleProvision(session: IHTTPSession): Response {
        val success = runBlocking {
            tdLibManager.autoProvisionGuest()
        }
        val message = if (success) {
            "Guest bot session initialized successfully."
        } else {
            friendlyLoginError(tdLibManager.lastError)
                .ifEmpty { "Auto-connect unavailable. Please enter your Telegram bot token." }
        }
        val json = JSONObject().apply {
            put("ok", if (success) 1 else 0)
            put("has_session", success)
            put("bot_id", if (success) tdLibManager.botId else "")
            put("bot_username", if (success) tdLibManager.botUsername else "")
            put("bot_name", if (success) tdLibManager.botName else "")
            put("api_secret", if (success) tdLibManager.apiSecret else "")
            put("pool", botPool.list())
            put("message", message)
        }
        return jsonResponse(json, if (success) Response.Status.OK else Response.Status.INTERNAL_ERROR)
    }

    private fun handleBotLogin(session: IHTTPSession): Response {
        val body = getRequestBody(session)
        var token = ""
        try {
            token = JSONObject(body).optString("bot_token", "")
        } catch (_: Exception) {}
        if (token.isEmpty()) token = session.parameters["bot_token"]?.firstOrNull() ?: ""
        token = token.trim()

        if (token.isEmpty()) {
            return jsonResponse(JSONObject().put("ok", 0).put("message", "Bot token is required"), Response.Status.BAD_REQUEST)
        }

        // Validate the token by actually connecting a TDLib session for it
        // (each bot lives in its own namespaced manager). Reports a friendly
        // message on failure instead of the old optimistic "Connecting bot...".
        val result = runBlocking { botPool.add(token) }
        if (!result.ok) {
            // Re-entering an already-pooled token is not an error — make it active.
            val existingId = botPool.findIdByToken(token)
            if (existingId.isNotEmpty()) {
                botPool.setActive(existingId)
                return jsonResponse(JSONObject().apply {
                    put("ok", 1)
                    put("bot_id", existingId)
                    put("message", "Bot is already connected.")
                })
            }
            return jsonResponse(
                JSONObject().put("ok", 0).put("message", friendlyLoginError(result.error)),
                Response.Status.BAD_REQUEST
            )
        }

        botPool.setActive(result.botId)
        return jsonResponse(JSONObject().apply {
            put("ok", 1)
            put("bot_id", result.botId)
            put("bot_username", result.username)
            put("bot_name", result.name)
            put("api_secret", botPool.activeManager().apiSecret)
            put("message", "Bot connected successfully.")
        })
    }

    private fun handleBotLogout(): Response {
        // "Disconnect all": clear the primary AND every pooled bot locally, then
        // empty the pool (mirrors backend.php /api/botlogout -> fd_clear_session()).
        // Previously only the active manager was logged out, so pooled bots like
        // @testmethebot stayed connected.
        botPool.clearAll()
        return jsonResponse(JSONObject().put("ok", 1).put("message", "Session cleared."))
    }

    /**
     * GET /api/clock-check — measure the device clock offset vs an accurate HTTP
     * server so the frontend can warn about a skewed clock BEFORE a login. A skew
     * makes every MTProto handshake fail with a confusing "message id too new/old".
     */
    private fun handleClockCheck(): Response {
        val (ok, offsetSec) = measureClockOffset()
        val absOffset = Math.abs(offsetSec)
        val json = JSONObject().apply {
            put("ok", ok)
            put("offset_seconds", if (ok) offsetSec else 0L)
            put("server_time", if (ok) System.currentTimeMillis() / 1000L + offsetSec else 0L)
            put("local_time", System.currentTimeMillis() / 1000L)
            // Telegram tolerates far less than ±300s in practice: a 49s offset was
            // enough to break the auth-key exchange. >30s skewed, >60s critical.
            put("skewed", ok && absOffset > 30)
            put("critical", ok && absOffset > 60)
            put("message", if (ok) "" else "Could not measure the clock offset (network unreachable).")
        }
        return jsonResponse(json)
    }

    // ── Runtime preflight (mirrors backend.php fd_environment_preflight()) ──────
    private class EnvPreflight(val ok: Boolean, val fatal: Boolean, val problems: JSONArray, val hints: JSONArray)

    @Volatile private var envCache: EnvPreflight? = null
    @Volatile private var envCacheAt: Long = 0L

    private fun environmentPreflight(): EnvPreflight {
        val now = System.currentTimeMillis()
        envCache?.let { if (now - envCacheAt < 60_000L) return it }

        val problems = JSONArray()
        val hints = JSONArray()

        // 1. Bundled Telegram native library. A missing/unsupported libtdjni can
        //    never be fixed by entering a bot token, so it is fatal.
        if (!TdLibManager.nativeLoaded) {
            problems.put("The Telegram library (libtdjni) could not be loaded on this device, so downloads cannot work.")
            hints.put("This device's CPU architecture is not supported by the bundled library. Install the build that matches this device's ABI.")
        }

        // 2. Internal storage must be writable for the bot session database.
        try {
            val dir = context.filesDir
            if (dir == null || !dir.exists() || !dir.canWrite()) {
                problems.put("Internal storage is not writable, so the bot session cannot be created.")
                hints.put("Free up device storage and make sure the app has storage permission, then restart the server.")
            }
        } catch (e: Exception) {
            problems.put("Internal storage is not accessible: ${e.message}")
        }

        val result = EnvPreflight(problems.length() == 0, problems.length() > 0, problems, hints)
        envCache = result
        envCacheAt = now
        return result
    }

    /** Translate raw TDLib/system errors into actionable text (mirrors fd_friendly_login_error()). */
    private fun friendlyLoginError(raw: String): String {
        val lower = raw.lowercase()
        return when {
            raw.isBlank() -> ""
            lower.contains("too new") || lower.contains("too old") || lower.contains("message id") || lower.contains("sync your date") ->
                "Device clock out of sync!\nTelegram requires accurate device time (within ~5 minutes).\n\n" +
                    "Enable 'Set time automatically' (Automatic date and time / NTP) in your device Settings, then restart the PencariMovie Server."
            lower.contains("unauthorized") || lower.contains("token_invalid") || lower.contains("token is invalid") || lower.contains("invalid token") ->
                "Invalid Bot Token!\nPlease double check the bot token from @BotFather. Ensure there are no extra spaces or missing characters."
            lower.contains("could not resolve host") || lower.contains("name or service not known") ||
                lower.contains("connection refused") || lower.contains("network is unreachable") ||
                lower.contains("failed to connect") || lower.contains("timeout") ->
                "Network connection failed!\nCannot reach Telegram or the PencariMovie API. Check your internet connection or private DNS settings."
            lower.contains("flood") ->
                "Too many login attempts. Please wait a few minutes and try again."
            lower.contains("did not connect") ->
                "Could not connect the bot.\nCheck that the token is correct and the device has internet access."
            else -> raw
        }
    }

    /** Measure the clock offset vs an accurate HTTP server's Date header (seconds). */
    private fun measureClockOffset(): Pair<Boolean, Long> {
        return try {
            val req = Request.Builder().url("https://pencarimovie.com/").get().build()
            val t0 = System.currentTimeMillis()
            httpClient.newCall(req).execute().use { resp ->
                val t1 = System.currentTimeMillis()
                val dateHeader = resp.header("Date") ?: return false to 0L
                val serverMs = parseHttpDate(dateHeader) ?: return false to 0L
                val localMs = (t0 + t1) / 2
                true to (serverMs - localMs) / 1000L
            }
        } catch (e: Exception) {
            DebugLog.log("HTTP", "clock-check failed: ${e.message}")
            false to 0L
        }
    }

    private fun parseHttpDate(value: String): Long? = try {
        val fmt = java.text.SimpleDateFormat("EEE, dd MMM yyyy HH:mm:ss zzz", java.util.Locale.US)
        fmt.timeZone = java.util.TimeZone.getTimeZone("GMT")
        fmt.parse(value)?.time
    } catch (e: Exception) {
        null
    }

    // Native cloudflared tunnel manager (port of backend.php's fd_tunnel_* helpers).
    private val tunnelManager by lazy { TunnelManager(context, listeningPort, ::deviceId) }

    /** Stops the cloudflared child process (called when the service shuts down). */
    fun stopTunnel() {
        tunnelManager.stop()
    }

    /**
     * Revive + re-register an enabled tunnel at server start. status() restarts
     * cloudflared if it died and re-registers the relay URL, so the stable
     * subdomain keeps working after an app/service restart without waiting for a
     * UI poll (previously it returned Cloudflare 530 until the app was opened).
     */
    fun refreshTunnel() {
        try {
            tunnelManager.status()
        } catch (e: Exception) {
            DebugLog.log("Tunnel", "boot refresh failed: ${e.message}")
        }
    }

    private fun handleTunnel(uri: String, session: IHTTPSession): Response {
        val json = when {
            uri.contains("/enable") -> {
                val body = getRequestBody(session)
                var token = ""
                try {
                    if (body.isNotEmpty()) token = JSONObject(body).optString("tunnel_token", "")
                } catch (_: Exception) {}
                if (token.isEmpty()) token = session.parameters["tunnel_token"]?.firstOrNull() ?: ""
                tunnelManager.enable(token)
            }
            uri.contains("/disable") -> tunnelManager.disable()
            else -> tunnelManager.status()
        }
        return jsonResponse(json)
    }

    private fun handleCountry(session: IHTTPSession): Response {
        val (code, name, source) = fdDetectCountry()
        val configured = try { loadCatalogSettings().optString("country", "") } catch (e: Exception) { "" }
        val available = JSONArray()
        available.put(JSONObject().put("code", "").put("name", "Auto (Detected)"))
        for (c in AVAILABLE_COUNTRY_CODES) {
            available.put(JSONObject().put("code", c).put("name", COUNTRY_NAMES[c] ?: c))
        }
        return jsonResponse(JSONObject().apply {
            put("ok", 1)
            put("country", JSONObject().apply {
                put("country_code", code)
                put("country_name", name)
                put("source", source)
            })
            put("configured_country", configured)
            put("available_countries", available)
        })
    }

    private fun handleResolveShortcode(session: IHTTPSession): Response {
        val params = session.parameters
        val shortCodes = params["short_codes"]?.firstOrNull() ?: params["short_code"]?.firstOrNull() ?: ""
        val url = "$WP_API_BASE/resolve-shortcode?short_codes=$shortCodes"
        return try {
            val req = Request.Builder().url(url).get().build()
            val upstreamResp = httpClient.newCall(req).execute()
            val bodyStr = upstreamResp.body?.string() ?: "{}"
            newFixedLengthResponse(Response.Status.OK, "application/json; charset=utf-8", bodyStr).apply {
                addCorsHeaders(this)
            }
        } catch (e: Exception) {
            jsonResponse(JSONObject().put("ok", 0).put("message", e.message ?: "Failed to resolve"))
        }
    }

    /** True when the request carries a valid access token (path / query / header / cookie). */
    private fun isAuthenticated(session: IHTTPSession, rawUri: String): Boolean {
        if (auth.isValidToken(auth.tokenFromPath(rawUri))) return true
        if (auth.isValidToken(session.parameters["token"]?.firstOrNull()?.trim() ?: "")) return true
        if (auth.isValidToken((session.headers["x-auth-token"] ?: "").trim())) return true
        val cookie = session.headers["cookie"] ?: ""
        val m = Regex("(?:^|;\\s*)pm_auth=([^;]+)").find(cookie) ?: return false
        return auth.isValidToken(m.groupValues[1].trim())
    }

    private fun authRequired(session: IHTTPSession, uri: String): Response {
        // Stremio expects a locked {streams:[…]} tile so it can prompt a re-install.
        if (uri.startsWith("/stream/")) {
            val host = (session.headers["host"] ?: "127.0.0.1:$listeningPort").substringBefore(",").trim()
            val scheme = if ((session.headers["x-forwarded-proto"] ?: "").lowercase() == "https") "https" else "http"
            return jsonResponse(JSONObject().put("streams", JSONArray().put(JSONObject().apply {
                put("name", "PencariMovie")
                put("description", "Addon URL changed or password required.\nOpen PencariMovie and re-install the addon from #addon.")
                put("externalUrl", "$scheme://$host/#addon")
            })))
        }
        // A browser navigating the tunnel URL wants the password page, not raw
        // JSON — otherwise the public URL looks like "no UI". API/XHR requests
        // (Accept without text/html) still get the 401 JSON below.
        val accept = session.headers["accept"] ?: ""
        if (accept.contains("text/html", ignoreCase = true)) {
            return newFixedLengthResponse(Response.Status.OK, "text/html; charset=utf-8", AUTH_GATE_HTML).apply {
                addHeader("Cache-Control", "no-store, no-cache, must-revalidate")
            }
        }
        return jsonResponse(
            JSONObject().put("ok", 0).put("message", "Password required").put("auth_required", true),
            Response.Status.UNAUTHORIZED
        )
    }

    private fun handleAuth(uri: String, session: IHTTPSession): Response {
        return when (uri) {
            "/api/auth/status" -> {
                val authed = auth.isLocalRequest(session) ||
                    isAuthenticated(session, session.uri)
                jsonResponse(JSONObject().apply {
                    put("ok", 1)
                    put("enabled", auth.enabled())
                    put("authenticated", authed)
                    put("token", if (authed) auth.token() else "")
                })
            }
            "/api/auth/login" -> handleAuthLogin(session)
            "/api/auth/logout" -> jsonResponse(JSONObject().put("ok", 1))
            "/api/auth/password" -> handleAuthPassword(session)
            "/api/auth/token/rotate" -> jsonResponse(JSONObject().put("ok", 1).put("token", auth.rotateToken()))
            // backend.php: fd_require_local_request() before resetting the password —
            // a tunnel/LAN visitor holding the token must not be able to reset it.
            "/api/auth/reset" -> if (!auth.isLocalRequest(session)) {
                jsonResponse(
                    JSONObject().put("ok", 0).put("message", "This endpoint is restricted to local requests only."),
                    Response.Status.FORBIDDEN
                )
            } else {
                jsonResponse(JSONObject().put("ok", 1).put("token", auth.setPassword("123456")))
            }
            else -> jsonResponse(JSONObject().apply {
                put("ok", 1)
                put("authenticated", true)
                put("auth_required", false)
            })
        }
    }

    private fun handleAuthLogin(session: IHTTPSession): Response {
        val body = getRequestBody(session)
        var pw = ""
        try { if (body.isNotEmpty()) pw = JSONObject(body).optString("password", "") } catch (_: Exception) {}
        if (!auth.verifyPassword(pw)) {
            return jsonResponse(
                JSONObject().put("ok", 0).put("message", "Invalid password."),
                Response.Status.UNAUTHORIZED
            )
        }
        // Set the pm_auth cookie so subsequent browser requests (the SPA + its
        // assets) authenticate without the token being in every URL.
        val token = auth.token()
        return jsonResponse(JSONObject().put("ok", 1).put("token", token)).apply {
            addHeader("Set-Cookie", "pm_auth=$token; Path=/; Max-Age=31536000; SameSite=Lax")
        }
    }

    private fun handleAuthPassword(session: IHTTPSession): Response {
        val body = getRequestBody(session)
        var current = ""
        var next = ""
        try {
            if (body.isNotEmpty()) {
                val j = JSONObject(body)
                current = j.optString("current", "")
                next = j.optString("next", "").trim()
            }
        } catch (_: Exception) {}
        if (!auth.verifyPassword(current)) {
            return jsonResponse(
                JSONObject().put("ok", 0).put("message", "Current password is wrong"),
                Response.Status.UNAUTHORIZED
            )
        }
        if (next.length < 4) {
            return jsonResponse(
                JSONObject().put("ok", 0).put("message", "Password must be at least 4 characters"),
                Response.Status.BAD_REQUEST
            )
        }
        return jsonResponse(JSONObject().put("ok", 1).put("token", auth.setPassword(next)))
    }

    /**
     * Bot pool management, mirroring backend.php's /api/bots routes:
     *   GET  /api/bots            -> {ok, active_bot_id, total_bots, bots[]}
     *   POST /api/bots/add        -> {ok, added_count, results[], pool[]}  (400 if none added)
     *   POST /api/bots/remove     -> {ok, message, pool[], active_bot_id}
     *   POST /api/bots/set-active -> {ok, active_bot_id, ...}              (404 if missing)
     */
    private fun handleBots(uri: String, session: IHTTPSession): Response {
        return when {
            uri.endsWith("/add") -> {
                val input = try {
                    val body = getRequestBody(session)
                    if (body.isNotEmpty()) JSONObject(body) else null
                } catch (_: Exception) { null }
                if (input == null) {
                    return jsonResponse(
                        JSONObject().put("ok", 0).put("message", "Invalid JSON body"),
                        Response.Status.BAD_REQUEST
                    )
                }
                // Token sources mirror backend.php precedence: bot_token, then
                // tokens[], then tokens_text (split on whitespace/comma/newline).
                val tokens = ArrayList<String>()
                val bt = input.optString("bot_token", "").trim()
                val arr = input.optJSONArray("tokens")
                val tt = input.optString("tokens_text", "")
                when {
                    bt.isNotEmpty() -> tokens.add(bt)
                    arr != null -> for (i in 0 until arr.length()) {
                        val v = arr.optString(i).trim()
                        if (v.isNotEmpty()) tokens.add(v)
                    }
                    tt.isNotEmpty() -> for (p in tt.split(Regex("[\\r\\n\\s,]+"))) {
                        val v = p.trim()
                        if (v.isNotEmpty()) tokens.add(v)
                    }
                }
                if (tokens.isEmpty()) {
                    return jsonResponse(
                        JSONObject().put("ok", 0).put("message", "bot_token or tokens array is required."),
                        Response.Status.BAD_REQUEST
                    )
                }
                val results = JSONArray()
                var addedCount = 0
                for (t in tokens) {
                    val r = runBlocking { botPool.add(t) }
                    if (r.ok) {
                        addedCount++
                        results.put(JSONObject().apply {
                            put("ok", 1)
                            put("bot_id", r.botId)
                            put("bot_username", r.username)
                            put("bot_name", r.name)
                        })
                    } else {
                        results.put(JSONObject().apply {
                            put("token_prefix", t.take(8) + "...")
                            put("ok", 0)
                            put("error", r.error)
                        })
                    }
                }
                jsonResponse(
                    JSONObject().apply {
                        put("ok", if (addedCount > 0) 1 else 0)
                        put("added_count", addedCount)
                        put("results", results)
                        put("pool", botPool.list())
                    },
                    if (addedCount > 0) Response.Status.OK else Response.Status.BAD_REQUEST
                )
            }
            uri.endsWith("/remove") -> {
                val input = try { JSONObject(getRequestBody(session)) } catch (_: Exception) { null }
                if (input == null) {
                    return jsonResponse(
                        JSONObject().put("ok", 0).put("message", "Invalid JSON body"),
                        Response.Status.BAD_REQUEST
                    )
                }
                val id = input.optString("bot_id", "").trim()
                if (id.isEmpty()) {
                    return jsonResponse(
                        JSONObject().put("ok", 0).put("message", "bot_id is required."),
                        Response.Status.BAD_REQUEST
                    )
                }
                // Mirrors backend.php: always ok:1 — an unknown id is a no-op there too.
                botPool.remove(id)
                jsonResponse(JSONObject().apply {
                    put("ok", 1)
                    put("message", "Bot removed from pool.")
                    put("pool", botPool.list())
                    put("active_bot_id", botPool.activeId())
                })
            }
            uri.endsWith("/set-active") -> {
                val input = try { JSONObject(getRequestBody(session)) } catch (_: Exception) { null }
                if (input == null) {
                    return jsonResponse(
                        JSONObject().put("ok", 0).put("message", "Invalid JSON body"),
                        Response.Status.BAD_REQUEST
                    )
                }
                val id = input.optString("bot_id", "").trim()
                if (id.isEmpty()) {
                    return jsonResponse(
                        JSONObject().put("ok", 0).put("message", "bot_id is required."),
                        Response.Status.BAD_REQUEST
                    )
                }
                if (!botPool.setActive(id)) {
                    return jsonResponse(
                        JSONObject().put("ok", 0).put("message", "Bot ID not found in pool."),
                        Response.Status.NOT_FOUND
                    )
                }
                jsonResponse(JSONObject().apply {
                    put("ok", 1)
                    put("active_bot_id", id)
                    put("tunnel_restarted", false)
                })
            }
            else -> {
                val bots = botPool.list()
                jsonResponse(JSONObject().apply {
                    put("ok", 1)
                    put("active_bot_id", botPool.activeId())
                    put("total_bots", bots.length())
                    put("bots", bots)
                })
            }
        }
    }

    /**
     * Mirror of backend.php fd_get_default_catalog_options(): the id -> {type, group,
     * name, default} map the Catalog Configuration UI renders as checkboxes.
     * group "special" -> "Core & Telegram Catalogs", everything else -> "Categories & Countries".
     */
    private fun defaultCatalogOptions(): JSONObject {
        fun opt(type: String, group: String, name: String, def: Boolean) = JSONObject().apply {
            put("type", type); put("group", group); put("name", name); put("default", def)
        }
        return JSONObject().apply {
            // Special catalogs
            put("top", opt("movie", "special", "Popular (Movies)", true))
            put("year", opt("movie", "special", "New (Movies)", true))
            put("pm_search_movie", opt("movie", "special", "Search Movies", true))
            put("pm_series_top", opt("series", "special", "Popular (Series)", true))
            put("pm_series_year", opt("series", "special", "New (Series)", true))
            put("pm_search_series", opt("series", "special", "Search Series", true))
            put("pm_files_year", opt("other", "special", "New (Telegram Files)", true))
            put("pm_search_files", opt("other", "special", "Telegram Files (Search)", true))
            put("pm_trending_keywords", opt("other", "special", "Trending Keywords (Files)", true))
            // Movie country catalogs
            put("pm_movies_malay", opt("movie", "country", "Malaysia (Movie)", false))
            put("pm_movies_indo", opt("movie", "country", "Indonesia (Movie)", false))
            put("pm_movies_korean", opt("movie", "country", "Korea (Movie)", false))
            put("pm_movies_japan", opt("movie", "country", "Japan (Movie)", false))
            put("pm_movies_anime", opt("movie", "country", "Anime (Movie)", false))
            put("pm_movies_chinese", opt("movie", "country", "China / HK (Movie)", false))
            put("pm_movies_thai", opt("movie", "country", "Thailand (Movie)", false))
            put("pm_movies_bollywood", opt("movie", "country", "Bollywood (Movie)", false))
            put("pm_movies_philippines", opt("movie", "country", "Philippines (Movie)", false))
            put("pm_movies_english", opt("movie", "country", "English (Movie)", false))
            // Series country catalogs
            put("pm_series_kdrama", opt("series", "country", "K-Drama (Series)", false))
            put("pm_series_anime", opt("series", "country", "Anime (Series)", false))
            put("pm_series_japan", opt("series", "country", "J-Drama (Series)", false))
            put("pm_series_malay", opt("series", "country", "Malaysia (Series)", false))
            put("pm_series_cdrama", opt("series", "country", "C-Drama (Series)", false))
            put("pm_series_thai", opt("series", "country", "Thailand (Series)", false))
            put("pm_series_philippines", opt("series", "country", "Philippines (Series)", false))
            put("pm_series_english", opt("series", "country", "English (Series)", false))
            put("pm_series_indo", opt("series", "country", "Indonesia (Series)", false))
        }
    }

    /** Mirror of backend.php fd_load_catalog_settings() defaults. */
    private fun defaultCatalogSettings(): JSONObject {
        val enabledCatalogs = JSONObject()
        val opts = defaultCatalogOptions()
        for (key in opts.keys()) {
            // Every catalog is available out of the box (country rows included), matching
            // the running PHP instance. The UI can still turn individual catalogs off.
            enabledCatalogs.put(key, true)
        }
        return JSONObject().apply {
            put("catalogs_enabled", true)
            put("country", "")
            put("enabled_types", JSONObject().apply {
                put("movie", true); put("series", true); put("other", true)
            })
            put("enabled_catalogs", enabledCatalogs)
            put("upstream_enabled", true)
            put("upstream_manifests", JSONArray().apply {
                put(JSONObject().apply {
                    put("url", "https://aiometadata.elfhosted.com/stremio/136f7ae5-2f15-471d-8f3f-79ee335bbd24/manifest.json")
                    put("name", "AIOMetadata")
                })
            })
            put("stream_config", JSONObject().apply {
                put("resolutions", JSONObject().apply {
                    put("4k", true); put("1080p", true); put("720p", true); put("sd", true); put("unknown", true)
                })
                put("qualities", JSONObject().apply {
                    put("remux", true); put("bluray", true); put("webdl", true); put("webrip", true)
                    put("hdtv", true); put("cam", true); put("unknown", true)
                })
                put("encodes", JSONObject().apply { put("hevc", true); put("avc", true); put("av1", true) })
                put("visual_tags", JSONObject().apply { put("hdr", true); put("dv", true) })
                put("preferred_resolution", "auto")
                put("max_streams_per_resolution", 0)
                put("max_streams_total", 0)
                put("min_size_mb", 0)
                put("max_size_gb", 0)
                put("exclude_cam", false)
                put("exclude_unplayable", true)
                put("excluded_keywords", "")
                put("required_keywords", "")
            })
        }
    }

    /** Recursive object merge; scalars and arrays replace. */
    private fun mergeSettings(base: JSONObject, patch: JSONObject) {
        for (key in patch.keys()) {
            val pv = patch.get(key)
            if (pv is JSONObject && base.opt(key) is JSONObject) {
                mergeSettings(base.getJSONObject(key), pv)
            } else {
                base.put(key, pv)
            }
        }
    }

    /** Mirrors backend.php fd_load_catalog_settings(): defaults merged with the saved blob. */
    private fun loadCatalogSettings(): JSONObject {
        val settings = defaultCatalogSettings()
        val saved = context.getSharedPreferences("catalog_settings", Context.MODE_PRIVATE)
            .getString("settings_json", null)
        if (saved != null) {
            try {
                mergeSettings(settings, JSONObject(saved))
            } catch (e: Exception) {
                DebugLog.log("Catalog", "settings parse failed: ${e.message}")
            }
        }
        return settings
    }

    private fun saveCatalogSettings(settings: JSONObject) {
        context.getSharedPreferences("catalog_settings", Context.MODE_PRIVATE).edit()
            .putString("settings_json", settings.toString()).apply()
    }

    private fun handleCatalogSettings(session: IHTTPSession): Response {
        if (session.method == Method.POST) {
            val body = getRequestBody(session)
            if (body.isNotEmpty()) {
                try {
                    val patch = JSONObject(body)
                    // Turning the master upstream switch off with an empty list keeps
                    // the stored list (mirrors backend.php), so re-enabling doesn't
                    // force the user to re-add every URL.
                    if (patch.has("upstream_enabled") && !patch.optBoolean("upstream_enabled", true)) {
                        val ups = patch.optJSONArray("upstream_manifests")
                        if (ups == null || ups.length() == 0) patch.remove("upstream_manifests")
                    }
                    val current = loadCatalogSettings()
                    mergeSettings(current, patch)
                    saveCatalogSettings(current)
                    val savedCount = patch.optJSONObject("enabled_catalogs")?.length() ?: -1
                    DebugLog.log("Catalog", "settings saved (catalogs=$savedCount)")
                } catch (e: Exception) {
                    return jsonResponse(
                        JSONObject().put("ok", 0).put("error", "Invalid JSON input"),
                        Response.Status.BAD_REQUEST
                    )
                }
            }
            return jsonResponse(JSONObject().put("ok", 1))
        }

        return jsonResponse(JSONObject().apply {
            put("ok", 1)
            put("settings", loadCatalogSettings())
            put("catalog_options", defaultCatalogOptions())
            put("is_tunnel", false)
        })
    }

    /**
     * Proxies streaming data from the WordPress AJAX API (port of backend.php
     * GET /api/proxy-stream): whitelists the params, injects the active bot id and
     * the detected country, and returns the upstream JSON verbatim.
     */
    private fun handleProxyStream(session: IHTTPSession): Response {
        val params = session.parameters
        val action = params["action"]?.firstOrNull()?.trim() ?: ""
        if (action.isEmpty()) {
            return jsonResponse(
                JSONObject().put("ok", 0).put("message", "action parameter is required."),
                Response.Status.BAD_REQUEST
            )
        }
        val route = action.replace('_', '-')

        val allowed = listOf(
            "category", "slug", "search", "limit", "offset", "post_id", "short_code",
            "type", "genre", "year", "sort", "bot_id", "country", "season", "episode"
        )
        val sb = StringBuilder("$WP_API_BASE/$route?")
        var firstQ = true
        fun q(k: String, v: String) {
            if (!firstQ) sb.append('&')
            firstQ = false
            sb.append(URLEncoder.encode(k, "UTF-8")).append('=').append(URLEncoder.encode(v, "UTF-8"))
        }
        for (key in allowed) {
            val v = params[key]?.firstOrNull()
            if (!v.isNullOrEmpty()) q(key, v)
        }
        // trending never carries a bot id; every other action does (unless supplied).
        if (action != "trending" && params["bot_id"]?.firstOrNull().isNullOrEmpty() && tdLibManager.botId.isNotEmpty()) {
            q("bot_id", tdLibManager.botId)
        }
        if (params["country"]?.firstOrNull().isNullOrEmpty()) {
            q("country", fdDetectCountry().first)
        }

        val request = Request.Builder()
            .url(sb.toString())
            .get()
            .addHeader("X-Requested-With", "XMLHttpRequest")
            .addHeader("Accept", "application/json")
            .build()

        return try {
            val upstreamResp = httpClient.newCall(request).execute()
            val bodyStr = upstreamResp.body?.string() ?: "{}"
            val contentType = upstreamResp.header("Content-Type", "application/json; charset=utf-8")
            newFixedLengthResponse(
                Response.Status.lookup(upstreamResp.code) ?: Response.Status.OK,
                contentType,
                bodyStr
            ).apply { addCorsHeaders(this) }
        } catch (e: Exception) {
            DebugLog.log("Proxy", "EXCEPTION ${e.message}")
            jsonResponse(
                JSONObject().put("ok", 0).put("message", "Failed to fetch data from PencariMovie."),
                Response.Status.INTERNAL_ERROR
            )
        }
    }

    private val shortCodeToFileCache = ConcurrentHashMap<String, Int>()
    private val shortCodeToSizeCache = ConcurrentHashMap<String, Long>()

    // WordPress /version payload (min_version, update_url, release_notes, sponsor),
    // cached 10 min. Feeds /api/version (the web UI reads sponsor from there) and
    // the stream sponsored tile.
    @Volatile private var wpVersionCache: JSONObject? = null
    @Volatile private var wpVersionCacheAt: Long = 0L

    private fun fetchWpVersion(): JSONObject? {
        val now = System.currentTimeMillis()
        wpVersionCache?.let { if (now - wpVersionCacheAt < 600_000L) return it }
        return try {
            val req = Request.Builder().url("$WP_API_BASE/version").get()
                .addHeader("Accept", "application/json").build()
            val resp = httpClient.newCall(req).execute()
            val body = resp.body?.string() ?: "{}"
            val json = JSONObject(if (body.contains("{")) body.substring(body.indexOf('{')) else body)
            wpVersionCache = json
            wpVersionCacheAt = now
            json
        } catch (e: Exception) {
            DebugLog.log("Version", "fetchWpVersion error: ${e.message}")
            null
        }
    }

    private fun fetchSponsor(): JSONObject? {
        val sp = fetchWpVersion()?.optJSONObject("sponsor")
        if (sp != null) DebugLog.log("Sponsor", "loaded url=${sp.optString("url", "").isNotEmpty()} name=${sp.optString("name", "")}")
        return sp
    }

    // Upstream manifest JSON, cached 2h in memory (backend.php caches on disk).
    private val upstreamManifestCache = java.util.concurrent.ConcurrentHashMap<String, Pair<Long, JSONObject>>()

    private fun fetchUpstreamManifest(url: String): JSONObject? {
        val now = System.currentTimeMillis()
        upstreamManifestCache[url]?.let { (at, j) -> if (now - at < 7_200_000L) return j }
        return try {
            val req = Request.Builder().url(url).get().addHeader("Accept", "application/json").build()
            val resp = httpClient.newCall(req).execute()
            val body = resp.body?.string() ?: return null
            val j = JSONObject(body)
            if (j.has("catalogs")) { upstreamManifestCache[url] = now to j; j } else null
        } catch (e: Exception) {
            DebugLog.log("Manifest", "upstream fetch failed ($url): ${e.message}")
            null
        }
    }

    // Upstream catalog responses, cached 1h in memory (backend.php caches on disk).
    private val upstreamCatalogCache = java.util.concurrent.ConcurrentHashMap<String, Pair<Long, String>>()

    /** Forward a bridged catalog (up_{index}_{id}) to its upstream addon. */
    private fun forwardUpstreamCatalog(type: String, uIdx: Int, realCatId: String, extraStr: String): Response {
        try {
            val cs = loadCatalogSettings()
            if (!cs.optBoolean("upstream_enabled", true)) return jsonResponse(JSONObject().put("metas", JSONArray()))
            val upstreams = cs.optJSONArray("upstream_manifests")
            if (upstreams == null || uIdx < 0 || uIdx >= upstreams.length()) {
                return jsonResponse(JSONObject().put("metas", JSONArray()))
            }
            val mUrl = upstreams.optJSONObject(uIdx)?.optString("url", "")?.trim() ?: ""
            if (mUrl.isEmpty()) return jsonResponse(JSONObject().put("metas", JSONArray()))

            val baseAddonUrl = mUrl.replace(Regex("/manifest\\.json(\\?.*)?$", RegexOption.IGNORE_CASE), "").trimEnd('/')
            var forwardUrl = "$baseAddonUrl/catalog/$type/" + java.net.URLEncoder.encode(realCatId, "UTF-8")
            if (extraStr.isNotEmpty()) forwardUrl += "/$extraStr"
            forwardUrl += ".json"

            val now = System.currentTimeMillis()
            upstreamCatalogCache[forwardUrl]?.let { (at, cached) ->
                if (now - at < 3_600_000L) {
                    return newFixedLengthResponse(Response.Status.OK, "application/json; charset=utf-8", cached)
                }
            }
            val req = Request.Builder().url(forwardUrl).get().addHeader("Accept", "application/json").build()
            val resp = httpClient.newCall(req).execute()
            val body = resp.body?.string() ?: "{}"
            if (body.contains("\"metas\"")) {
                upstreamCatalogCache[forwardUrl] = now to body
                return newFixedLengthResponse(Response.Status.OK, "application/json; charset=utf-8", body)
            }
        } catch (e: Exception) {
            DebugLog.log("Catalog", "upstream forward failed: ${e.message}")
        }
        return jsonResponse(JSONObject().put("metas", JSONArray()))
    }

    /**
     * Stremio/Nuvio manifest. Mirrors backend.php's /manifest.json handler:
     * Popular / New / Search catalogs, per-country catalogs, Telegram Files
     * catalogs, trending-keyword catalogs, identity labelled by the address the
     * client used, and the GitHub-hosted logo for non-HTTPS requests.
     */
    private fun handleManifest(session: IHTTPSession): Response {
        val host = (session.headers["host"] ?: "127.0.0.1:8088").substringBefore(",").trim()
        if (host.isEmpty()) return jsonResponse(JSONObject().put("ok", 0).put("message", "Bad host"))
        val hostName = host.substringBefore(":").lowercase()
        val forwardedProto = (session.headers["x-forwarded-proto"] ?: "").lowercase()
        val isTunnelHost = hostName.endsWith(".trycloudflare.com") || hostName.endsWith("tunnel.pencarimovie.com")
        val isHttps = forwardedProto == "https" || isTunnelHost
        val origin = (if (isHttps) "https://" else "http://") + host

        // ?mode=tunnel|localhost|lan|server overrides the auto-detected identity.
        val modeOverride = (session.parameters["mode"]?.firstOrNull() ?: "").lowercase().trim()

        val allGenreOptions = listOf(
            "Action", "Adventure", "Animation", "Anime", "Biography", "Comedy", "Crime",
            "Documentary", "Drama", "Family", "Fantasy", "History", "Horror", "Music",
            "Musical", "Mystery", "Romance", "Sci-Fi", "Sport", "Thriller", "War", "Western"
        )
        val currentYear = java.util.Calendar.getInstance().get(java.util.Calendar.YEAR)
        val allYearOptions = (currentYear downTo 1980).map { it.toString() }
        val filterCombined = allYearOptions + allGenreOptions

        fun ja(items: List<String>): JSONArray {
            val a = JSONArray()
            items.forEach { a.put(it) }
            return a
        }
        fun extra(name: String, required: Boolean, options: List<String>? = null): JSONObject =
            JSONObject().apply {
                put("name", name)
                put("isRequired", required)
                if (options != null) put("options", ja(options))
            }
        fun catalog(type: String, id: String, name: String, genres: List<String>, extraArr: JSONArray, extraSupported: List<String>, extraRequired: List<String> = emptyList()): JSONObject =
            JSONObject().apply {
                put("type", type)
                put("id", id)
                put("name", name)
                put("genres", ja(genres))
                put("extra", extraArr)
                put("extraSupported", ja(extraSupported))
                if (extraRequired.isNotEmpty()) put("extraRequired", ja(extraRequired))
            }

        // Detected country (device locale) promotes its catalog to the top, like backend.php.
        val countryCode = (java.util.Locale.getDefault().country ?: "").lowercase()

        val movieCountryCatalogs = linkedMapOf(
            "pm_movies_malay" to "Malaysia",
            "pm_movies_indo" to "Indonesia",
            "pm_movies_korean" to "Korea",
            "pm_movies_japan" to "Japan",
            "pm_movies_anime" to "Anime",
            "pm_movies_chinese" to "China / HK",
            "pm_movies_thai" to "Thailand",
            "pm_movies_bollywood" to "Bollywood",
            "pm_movies_philippines" to "Philippines",
            "pm_movies_english" to "English"
        )
        val movieFirst = mapOf(
            "my" to "pm_movies_malay", "id" to "pm_movies_indo", "kr" to "pm_movies_korean",
            "jp" to "pm_movies_japan", "cn" to "pm_movies_chinese", "hk" to "pm_movies_chinese",
            "th" to "pm_movies_thai", "in" to "pm_movies_bollywood", "ph" to "pm_movies_philippines"
        )[countryCode]
        if (movieFirst != null && movieCountryCatalogs.containsKey(movieFirst)) {
            val name = movieCountryCatalogs.remove(movieFirst)!!
            val reordered = linkedMapOf(movieFirst to name)
            reordered.putAll(movieCountryCatalogs)
            movieCountryCatalogs.clear()
            movieCountryCatalogs.putAll(reordered)
        }

        val seriesCountryCatalogs = linkedMapOf(
            "pm_series_malay" to "Malaysia",
            "pm_series_indo" to "Indonesia",
            "pm_series_kdrama" to "K-Drama",
            "pm_series_anime" to "Anime",
            "pm_series_japan" to "J-Drama",
            "pm_series_cdrama" to "C-Drama",
            "pm_series_thai" to "Thailand",
            "pm_series_philippines" to "Philippines",
            "pm_series_english" to "English"
        )
        val seriesFirst = mapOf(
            "my" to "pm_series_malay", "id" to "pm_series_indo", "kr" to "pm_series_kdrama",
            "jp" to "pm_series_japan", "cn" to "pm_series_cdrama", "hk" to "pm_series_cdrama",
            "th" to "pm_series_thai", "ph" to "pm_series_philippines"
        )[countryCode]
        if (seriesFirst != null && seriesCountryCatalogs.containsKey(seriesFirst)) {
            val name = seriesCountryCatalogs.remove(seriesFirst)!!
            val reordered = linkedMapOf(seriesFirst to name)
            reordered.putAll(seriesCountryCatalogs)
            seriesCountryCatalogs.clear()
            seriesCountryCatalogs.putAll(reordered)
        }

        var catalogs = JSONArray()

        // ── Movies: Popular / New / Search ──
        catalogs.put(catalog("movie", "top", "Popular", allGenreOptions,
            JSONArray().apply {
                put(extra("search", false))
                put(extra("genre", false, allGenreOptions))
                put(extra("year", false, allYearOptions))
                put(extra("skip", false))
            }, listOf("search", "genre", "year", "skip")))
        catalogs.put(catalog("movie", "year", "New", allYearOptions,
            JSONArray().apply {
                put(extra("genre", true, allYearOptions))
                put(extra("skip", false))
            }, listOf("genre", "skip"), listOf("genre")))
        catalogs.put(catalog("movie", "pm_search_movie", "Search Movies", allGenreOptions,
            JSONArray().apply {
                put(extra("search", true))
                put(extra("genre", false, allGenreOptions))
                put(extra("year", false, allYearOptions))
                put(extra("skip", false))
            }, listOf("search", "genre", "year", "skip"), listOf("search")))
        for ((id, name) in movieCountryCatalogs) {
            catalogs.put(catalog("movie", id, name, filterCombined,
                JSONArray().apply {
                    put(extra("genre", false, filterCombined))
                    put(extra("year", false, allYearOptions))
                    put(extra("skip", false))
                }, listOf("genre", "year", "skip")))
        }

        // ── Series: Popular / New / Search ──
        catalogs.put(catalog("series", "top", "Popular", allGenreOptions,
            JSONArray().apply {
                put(extra("search", false))
                put(extra("genre", false, allGenreOptions))
                put(extra("year", false, allYearOptions))
                put(extra("skip", false))
            }, listOf("search", "genre", "year", "skip")))
        catalogs.put(catalog("series", "pm_series_year", "New", allYearOptions,
            JSONArray().apply {
                put(extra("genre", true, allYearOptions))
                put(extra("skip", false))
            }, listOf("genre", "skip"), listOf("genre")))
        catalogs.put(catalog("series", "pm_search_series", "Search Series", allGenreOptions,
            JSONArray().apply {
                put(extra("search", true))
                put(extra("genre", false, allGenreOptions))
                put(extra("year", false, allYearOptions))
                put(extra("skip", false))
            }, listOf("search", "genre", "year", "skip"), listOf("search")))
        for ((id, name) in seriesCountryCatalogs) {
            catalogs.put(catalog("series", id, name, filterCombined,
                JSONArray().apply {
                    put(extra("genre", false, filterCombined))
                    put(extra("year", false, allYearOptions))
                    put(extra("skip", false))
                }, listOf("genre", "year", "skip")))
        }

        // ── Other: Telegram Files ──
        catalogs.put(catalog("other", "top", "Telegram Files", allYearOptions,
            JSONArray().apply {
                put(extra("search", false))
                put(extra("genre", false, allYearOptions))
                put(extra("skip", false))
            }, listOf("search", "genre", "skip")))
        catalogs.put(catalog("other", "pm_files_year", "New", allYearOptions,
            JSONArray().apply {
                put(extra("genre", true, allYearOptions))
                put(extra("skip", false))
            }, listOf("genre", "skip"), listOf("genre")))
        catalogs.put(catalog("other", "pm_search_files", "Telegram Files", allYearOptions,
            JSONArray().apply {
                put(extra("search", true))
                put(extra("genre", false, allYearOptions))
                put(extra("skip", false))
            }, listOf("search", "genre", "skip"), listOf("search")))

        // ── Trending keyword catalogs (top 5, validated like backend.php) ──
        try {
            val tReq = Request.Builder().url("$WP_API_BASE/trending?limit=15").get().build()
            val tResp = httpClient.newCall(tReq).execute()
            val tBody = tResp.body?.string() ?: "[]"
            val tJson = if (tBody.trimStart().startsWith("[")) JSONArray(tBody) else run {
                val o = JSONObject(tBody)
                o.optJSONArray("data") ?: o.optJSONArray("items") ?: o.optJSONArray("files") ?: JSONArray()
            }
            var added = 0
            for (i in 0 until tJson.length()) {
                if (added >= 5) break
                val kw = tJson.optJSONObject(i)?.optString("keyword", "")?.trim() ?: ""
                if (!isTopKeywordValid(kw)) continue
                val catId = "pm_topkw_" + md5Hex(kw.lowercase()).take(10)
                val displayName = kw.split(" ").joinToString(" ") { w ->
                    if (w.isEmpty()) w else w.substring(0, 1).uppercase() + w.substring(1)
                }
                val kwGenres = listOf("4K", "1080p", "720p", "BluRay", "WEB-DL", "HEVC")
                catalogs.put(catalog("other", catId, displayName, kwGenres,
                    JSONArray().apply {
                        put(extra("genre", false, kwGenres))
                        put(extra("year", false, allYearOptions))
                        put(extra("skip", false))
                    }, listOf("genre", "year", "skip")))
                added++
            }
            DebugLog.log("Manifest", "trending keyword catalogs added=$added")
        } catch (e: Exception) {
            DebugLog.log("Manifest", "trending keywords skipped: ${e.message}")
        }

        val idPrefixesList = listOf("pm_", "pm:", "tt", "tmdb:", "kitsu:", "kitsu", "mal:", "anilist:", "tvdb:")
        val idPrefixes = ja(idPrefixesList)

        // ── Apply catalog settings (defaults merged with the saved blob, mirrors backend.php) ──
        var activeTypes = listOf("movie", "series", "other")
        try {
            val cs = loadCatalogSettings()
            val enabledTypes = cs.optJSONObject("enabled_types")
            val enabledCatalogs = cs.optJSONObject("enabled_catalogs")
            val catalogsEnabled = cs.optBoolean("catalogs_enabled", true)
            if (!catalogsEnabled) {
                catalogs = JSONArray()
            } else {
                val filtered = JSONArray()
                for (i in 0 until catalogs.length()) {
                    val cat = catalogs.getJSONObject(i)
                    val cType = cat.optString("type", "")
                    val cId = cat.optString("id", "")
                    if (enabledTypes != null && !enabledTypes.optBoolean(cType, true)) continue
                    var checkId = cId
                    if (cType == "series" && cId == "top") checkId = "pm_series_top"
                    else if (cType == "other" && cId == "top") checkId = "pm_files_year"
                    if (cId.startsWith("pm_topkw_")) checkId = "pm_trending_keywords"
                    if (enabledCatalogs != null && enabledCatalogs.has(checkId) &&
                        !enabledCatalogs.optBoolean(checkId, true)) continue
                    filtered.put(cat)
                }
                catalogs = filtered
            }
            val at = ArrayList<String>()
            if (enabledTypes == null || enabledTypes.optBoolean("movie", true)) at.add("movie")
            if (enabledTypes == null || enabledTypes.optBoolean("series", true)) at.add("series")
            if (enabledTypes == null || enabledTypes.optBoolean("other", true)) at.add("other")
            if (at.isNotEmpty()) activeTypes = at
            DebugLog.log("Manifest", "catalog settings applied: catalogs=${catalogs.length()} types=$activeTypes")
        } catch (e: Exception) {
            DebugLog.log("Manifest", "catalog settings parse failed: ${e.message}")
        }

        // ── Bridge catalogs from configured upstream manifests (mirrors backend.php).
        // Independent of the local catalogs toggle, but ONLY when the master
        // upstream switch is on. IDs are namespaced up_{index}_{id}; requests for
        // them are forwarded to the upstream addon in handleStremioCatalog.
        try {
            val cs = loadCatalogSettings()
            val upstreams = if (cs.optBoolean("upstream_enabled", true)) cs.optJSONArray("upstream_manifests") else null
            if (upstreams != null) {
                val activeSet = java.util.LinkedHashSet<String>(activeTypes)
                for (uIdx in 0 until upstreams.length()) {
                    val up = upstreams.optJSONObject(uIdx) ?: continue
                    val mUrl = up.optString("url", "").trim()
                    if (mUrl.isEmpty()) continue
                    val mName = up.optString("name", "Addon").trim().ifEmpty { "Addon" }
                    val mJson = fetchUpstreamManifest(mUrl) ?: continue
                    val uCats = mJson.optJSONArray("catalogs") ?: continue
                    for (j in 0 until uCats.length()) {
                        val uCat = uCats.optJSONObject(j) ?: continue
                        val uType = uCat.optString("type", "")
                        val uId = uCat.optString("id", "")
                        if (uType.isEmpty() || uId.isEmpty()) continue
                        activeSet.add(uType)
                        // Trimmed catalog: drop the huge `options` arrays (AIOMetadata
                        // ships thousands) that blow past Stremio's descriptor limit.
                        val bridge = JSONObject().apply {
                            put("type", uType)
                            put("id", "up_${uIdx}_$uId")
                            put("name", uCat.optString("name", "Catalog") + " ($mName)")
                        }
                        uCat.optJSONArray("genres")?.let { bridge.put("genres", it) }
                        uCat.optJSONArray("extra")?.let { uExtra ->
                            val trimmed = JSONArray()
                            for (k in 0 until uExtra.length()) {
                                val ex = uExtra.optJSONObject(k) ?: continue
                                val exName = ex.optString("name", "")
                                if (exName.isEmpty()) continue
                                val entry = JSONObject().put("name", exName)
                                if (ex.has("isRequired")) entry.put("isRequired", ex.optBoolean("isRequired", false))
                                val opts = ex.optJSONArray("options")
                                if (opts != null && opts.length() in 1..12) entry.put("options", opts)
                                trimmed.put(entry)
                            }
                            if (trimmed.length() > 0) bridge.put("extra", trimmed)
                        }
                        catalogs.put(bridge)
                    }
                }
                activeTypes = activeSet.toList()
            }
            DebugLog.log("Manifest", "upstream bridge: catalogs=${catalogs.length()} types=$activeTypes")
        } catch (e: Exception) {
            DebugLog.log("Manifest", "upstream bridge failed: ${e.message}")
        }

        val resources = JSONArray().apply {
            if (catalogs.length() > 0) {
                put(JSONObject().apply { put("name", "catalog"); put("types", ja(activeTypes)) })
            }
            put(JSONObject().apply { put("name", "meta"); put("types", ja(activeTypes)); put("idPrefixes", idPrefixes) })
            put(JSONObject().apply { put("name", "stream"); put("types", ja(activeTypes)); put("idPrefixes", idPrefixes) })
            put(JSONObject().apply { put("name", "subtitles"); put("types", ja(activeTypes)); put("idPrefixes", idPrefixes) })
        }

        // Identity labelled by the address the client used (Localhost / Wi-Fi / Tunnel / Server).
        data class Identity(val id: String, val name: String, val description: String)
        val identity = when {
            modeOverride == "tunnel" || (modeOverride.isEmpty() && isTunnelHost) ->
                Identity("org.pencarimovie.addon.tunnel", "PencariMovie",
                    "Stream movies and series from Telegram via Cloudflare Tunnel HTTPS. Address: $origin")
            modeOverride == "localhost" || (modeOverride.isEmpty() && hostName in setOf("127.0.0.1", "localhost", "::1")) ->
                Identity("org.pencarimovie.addon.local", "PencariMovie",
                    "Stream movies and series from Telegram on this device only. Address: $origin")
            modeOverride == "lan" || (modeOverride.isEmpty() && isPrivateIpv4(hostName)) ->
                Identity("org.pencarimovie.addon.lan", "PencariMovie",
                    "Stream movies and series from Telegram on your Wi-Fi / LAN. Address: $origin")
            else ->
                Identity("org.pencarimovie.addon.server", "PencariMovie",
                    "Stream movies and series from Telegram on your server. Address: $origin")
        }

        // Self-host the addon logo. It ships inside the APK (assets/web/logo.png)
        // and is already served at /logo.png by this same server. The old
        // raw.githubusercontent.com URL made the logo vanish whenever GitHub was
        // slow/blocked (frequent on some ISPs) — even though the identical image
        // was available locally. Pointing at $origin keeps the logo working on
        // localhost, LAN and tunnel alike, and ships it with the app.
        val logoUrl = "$origin/logo.png"

        val json = JSONObject().apply {
            put("id", identity.id)
            put("version", BuildConfig.VERSION_NAME)
            put("name", identity.name)
            put("description", identity.description)
            put("logo", logoUrl)
            put("background", logoUrl)
            put("resources", resources)
            put("types", ja(activeTypes))
            put("idPrefixes", idPrefixes)
            put("catalogs", catalogs)
            put("behaviorHints", JSONObject().apply {
                put("configurable", true)
                put("configurationRequired", false)
                put("adult", false)
                put("p2p", false)
            })
        }
        DebugLog.log("Manifest", "mode=${identity.id} catalogs=${catalogs.length()} host=$host")
        return jsonResponse(json)
    }

    private val topKeywordBanned = setOf(
        "new", "movie", "movies", "film", "filem", "music", "song", "songs", "mp3",
        "latest", "baru", "naya", "putiya", "list", "lists", "start", "search",
        "audio", "video", "videos", "download", "free", "full", "hd", "online",
        "watch", "streaming", "series", "episode", "season", "part", "chapter",
        "test", "bot", "admin", "help", "hi", "hello", "hey", "hai", "helo", "ok",
        "sex", "sexy", "porn", "bokep", "lucah", "ngentot", "xxx", "hentai",
        "jav", "adult", "nsfw", "naked", "nude", "melayu", "colmek", "sange", "tetek"
    )

    private fun isTopKeywordValid(keyword: String): Boolean {
        val kw = keyword.lowercase().trim()
        if (kw.isEmpty() || kw.length < 3) return false
        return kw !in topKeywordBanned
    }

    private data class MediaTags(
        val resolution: String,
        val source: String,
        val platform: String,
        val codec: String,
        val audio: String,
        val visual: List<String>,
        val edition: String
    )

    /** Port of backend.php fd_extract_media_tags(). */
    private fun extractMediaTags(title: String, caption: String): MediaTags {
        val text = "$title $caption"
        fun m(p: String) = Regex(p, RegexOption.IGNORE_CASE).containsMatchIn(text)
        val res = when {
            m("\\b(2160p|4k|uhd)\\b") -> "4K"
            m("\\b(1080p|fhd)\\b") -> "1080p"
            m("\\b(720p|hd)\\b") -> "720p"
            m("\\b(540p)\\b") -> "540p"
            m("\\b(480p|sd)\\b") -> "480p"
            m("\\b(360p)\\b") -> "360p"
            else -> ""
        }
        val source = when {
            m("\\b(remux)\\b") -> "REMUX"
            m("\\b(bluray|blu-ray|bdrip|bbrip|brrip)\\b") -> "BluRay"
            m("\\b(web-?dl)\\b") -> "WEB-DL"
            m("\\b(webrip|web)\\b") -> "WEBRip"
            m("\\b(hdrip)\\b") -> "HDRip"
            m("\\b(hdtv|tvrip|pdtv)\\b") -> "HDTV"
            m("\\b(dvdrip|dvd)\\b") -> "DVDRip"
            m("\\b(hdcam|camrip|cam|telesync|ts|tc)\\b") -> "CAM"
            else -> ""
        }
        val platform = when {
            m("\\b(nf|netflix)\\b") -> "NF"
            m("\\b(amzn|primevideo|prime)\\b") -> "AMZN"
            m("\\b(dsnp|disney\\+?|disney)\\b") -> "DSNP"
            m("\\b(atvp|apple\\s*tv\\+?)\\b") -> "ATVP"
            m("\\b(hmax|hbo\\s*max)\\b") -> "HMAX"
            m("\\b(zee5)\\b") -> "ZEE5"
            m("\\b(hotstar)\\b") -> "Hotstar"
            m("\\b(viki)\\b") -> "Viki"
            m("\\b(wetv)\\b") -> "WeTV"
            m("\\b(iqiyi)\\b") -> "iQIYI"
            m("\\b(starzplay)\\b") -> "StarzPlay"
            else -> ""
        }
        val codec = when {
            m("\\b(hevc|x265|h\\.?265)\\b") -> "HEVC"
            m("\\b(avc|x264|h\\.?264)\\b") -> "H.264"
            m("\\b(av1)\\b") -> "AV1"
            m("\\b(xvid|divx)\\b") -> "XviD"
            else -> ""
        }
        val audio = when {
            m("\\b(atmos)\\b") -> "Atmos"
            m("\\b(ddp\\s*5\\.1|dd\\+\\s*5\\.1|eac3\\s*5\\.1)\\b") -> "DDP5.1"
            m("\\b(ddp\\s*2\\.0|dd\\+\\s*2\\.0|eac3\\s*2\\.0)\\b") -> "DDP2.0"
            m("\\b(ddp|dd\\+|eac3)\\b") -> "DDP"
            m("\\b(dd\\s*5\\.1|ac3\\s*5\\.1)\\b") -> "DD5.1"
            m("\\b(ac3|dd)\\b") -> "AC3"
            m("\\b(dts-hd\\s*ma)\\b") -> "DTS-HD MA"
            m("\\b(dts-hd)\\b") -> "DTS-HD"
            m("\\b(dts)\\b") -> "DTS"
            m("\\b(truehd)\\b") -> "TrueHD"
            m("\\b(aac\\s*5\\.1|5\\.1\\s*aac)\\b") -> "AAC5.1"
            m("\\b(aac\\s*2\\.0|2\\.0\\s*aac|aac2)\\b") -> "AAC2.0"
            m("\\b(aac)\\b") -> "AAC"
            m("\\b(flac)\\b") -> "FLAC"
            m("\\b(opus)\\b") -> "Opus"
            else -> ""
        }
        val visual = ArrayList<String>()
        if (m("\\b(hdr10\\+|hdr10|hdr)\\b")) visual.add("HDR")
        if (m("\\b(dolby\\s*vision|dovi|dv)\\b")) visual.add("DV")
        if (m("\\b(10bit|10-bit|hi10p?)\\b")) visual.add("10bit")
        if (m("\\b(imax)\\b")) visual.add("IMAX")
        val edition = when {
            m("\\b(remastered)\\b") -> "Remastered"
            m("\\b(extended)\\b") -> "Extended"
            m("\\b(uncut)\\b") -> "Uncut"
            m("\\b(repack|proper)\\b") -> "Proper"
            else -> ""
        }
        return MediaTags(res, source, platform, codec, audio, visual, edition)
    }

    /** Port of backend.php fd_clean_media_title(). */
    private fun cleanMediaTitle(title: String): String {
        if (title.isEmpty()) return ""
        var t = title
        t = t.replace(Regex("[\\x{1F300}-\\x{1F9FF}\\x{2600}-\\x{26FF}\\x{2700}-\\x{27BF}]"), " ")
        t = t.replace(Regex("^(?:on9[._\\s]stream[._\\s]+|stream[._\\s]+|www\\.[a-z0-9.-]+\\.[a-z]{2,}[._\\s]+)", RegexOption.IGNORE_CASE), "")
        t = t.replace(Regex("forwarded[._\\s]from.*$", RegexOption.IGNORE_CASE), "")
        t = t.replace(Regex("(?:Join[._\\s]Channel|Join[._\\s]Group|Join[._\\s]us|Join[._\\s]@).*$", RegexOption.IGNORE_CASE), "")
        t = t.replace(Regex("kumpulan[._\\s]drama.*$", RegexOption.IGNORE_CASE), "")
        t = t.replace(Regex("(?:https?://|httpst\\.me|https?\\.?t\\.me|\\bt\\.me/)[\\w./?=&_-]*", RegexOption.IGNORE_CASE), "")
        t = t.replace(Regex("(?:[.\\s_-]*\\d+(?:[.,]\\d+)?[.\\s_-]*(?:MB|GB|KB|TB))+(?:[.\\s_-]*https)?(?:[.\\s_-]*Open[.\\s_-]*Mini[.\\s_-]*App)?$", RegexOption.IGNORE_CASE), "")
        return t.trim(' ', '.', '_', '-', '=', '\t', '\n', '\r')
    }

    /** Port of backend.php fd_mime_to_extension(). */
    private fun mimeToExtension(mime: String, fallback: String = "mp4"): String {
        val m = mime.lowercase().trim()
        if (m.isEmpty()) return fallback
        return when {
            m.contains("matroska") -> "mkv"
            m.contains("webm") -> "webm"
            m.contains("quicktime") -> "mov"
            m.contains("x-msvideo") || m.contains("avi") -> "avi"
            m.contains("mp2t") || m.contains("m2ts") -> "ts"
            m.contains("audio/mpeg") || m.contains("mp3") -> "mp3"
            m.contains("audio/mp4") || m.contains("m4a") -> "m4a"
            m.contains("flac") -> "flac"
            m.contains("wav") || m.contains("wave") -> "wav"
            m.contains("ogg") || m.contains("opus") -> "ogg"
            m.contains("aac") -> "aac"
            m.contains("mp4") || m.contains("m4v") -> "mp4"
            else -> fallback
        }
    }

    /** Port of backend.php fd_guess_video_mime(). */
    private fun guessVideoMime(fileName: String, mime: String): String {
        val m = mime.lowercase().trim()
        if (m.isNotEmpty() && m.contains("/") &&
            m !in listOf("document", "application/document", "application/octet-stream", "binary/octet-stream")) {
            return m
        }
        val ext = fileName.substringAfterLast('.', "").lowercase()
        return when (ext) {
            "mp3" -> "audio/mpeg"
            "m4a" -> "audio/mp4"
            "flac" -> "audio/flac"
            "wav" -> "audio/wav"
            "ogg", "opus" -> "audio/ogg"
            "aac" -> "audio/aac"
            "mkv" -> "video/x-matroska"
            "webm" -> "video/webm"
            "avi" -> "video/x-msvideo"
            "mov" -> "video/quicktime"
            "ts", "m2ts" -> "video/mp2t"
            "m4v" -> "video/mp4"
            else -> "video/mp4"
        }
    }

    /** Port of backend.php fd_stremio_stream_filename(). */
    private fun streamFilename(fileName: String, mime: String): String {
        var name = fileName.trim()
        if (name.isEmpty()) name = "file"
        if (Regex("\\.(?:0\\d{2,3}|\\d{3}|zip|rar|7z|tar|gz|bz2|xz|iso|bin|exe|apk|pdf|epub)$", RegexOption.IGNORE_CASE).containsMatchIn(name)) {
            name = Regex("[^\\w.\\-]+").replace(name, "_")
            return name.trim('.', '_', '-')
        }
        val origExt = name.substringAfterLast('.', "").lowercase()
        val audioExts = listOf("mp3", "m4a", "flac", "wav", "ogg", "opus", "aac")
        val isAudio = mime.lowercase().startsWith("audio/") || origExt in audioExts
        val targetExt = if (origExt in audioExts) origExt else mimeToExtension(mime, if (isAudio) "mp3" else "mp4")
        val mediaPattern = Regex("\\.(mp4|m4v|mkv|webm|avi|mov|ts|m2ts|flv|wmv|3gp|mpg|mpeg|mp3|m4a|flac|wav|ogg|opus|aac)$", RegexOption.IGNORE_CASE)
        while (mediaPattern.containsMatchIn(name)) {
            name = mediaPattern.replace(name, "")
        }
        val defaultBase = if (isAudio) "audio" else "video"
        name = Regex("[^\\w.\\-]+").replace(name, "_").trim('.', '_', '-')
        if (name.isEmpty()) name = defaultBase
        return "$name.$targetExt"
    }

    /** Port of backend.php fd_format_bytes(). */
    private fun fdFormatBytes(bytes: Long): String {
        if (bytes <= 0) return "0 B"
        val units = listOf("B", "KB", "MB", "GB", "TB")
        var value = bytes.toDouble()
        var pow = 0
        while (value >= 1024.0 && pow < units.size - 1) {
            value /= 1024.0
            pow++
        }
        val rounded = Math.round(value * 10.0) / 10.0
        val s = if (rounded == Math.floor(rounded)) rounded.toLong().toString()
        else String.format(java.util.Locale.US, "%.1f", rounded)
        return "$s ${units[pow]}"
    }

    /** Port of the language-flag detection in backend.php's stream builder. */
    private fun detectLanguageFlags(title: String, caption: String): List<String> {
        val text = "$title $caption"
        fun m(p: String) = Regex(p, RegexOption.IGNORE_CASE).containsMatchIn(text)
        val flags = ArrayList<String>()
        if (m("\\b(malay|malaysub|sub\\s*malay|msia|melayu)\\b")) flags.add("🇲🇾 Malay")
        if (m("\\b(eng|english|esub|sub\\s*eng)\\b")) flags.add("🇬🇧 English")
        if (m("\\b(indo|indonesia|indosub)\\b")) flags.add("🇮🇩 Indo")
        if (m("\\b(hindi|hin|dub\\s*hindi)\\b")) flags.add("🇮🇳 Hindi")
        if (m("\\b(korean|kor|kdrama)\\b")) flags.add("🇰🇷 Korean")
        if (m("\\b(japanese|jap|anime)\\b")) flags.add("🇯🇵 Japanese")
        if (m("\\b(chinese|mandarin|cantonese|c-drama)\\b")) flags.add("🇨🇳 Chinese")
        if (m("\\b(thai)\\b")) flags.add("🇹🇭 Thai")
        return flags.distinct()
    }

    private fun md5Hex(input: String): String {
        return try {
            val md = java.security.MessageDigest.getInstance("MD5")
            md.digest(input.toByteArray(Charsets.UTF_8)).joinToString("") { "%02x".format(it.toInt() and 0xFF) }
        } catch (e: Exception) {
            input.hashCode().toString(16)
        }
    }

    private fun isPrivateIpv4(host: String): Boolean {
        val parts = host.split(".")
        if (parts.size != 4) return false
        val octets = parts.mapNotNull { it.toIntOrNull() }
        if (octets.size != 4 || octets.any { it !in 0..255 }) return false
        val (a, b) = octets
        return a == 10 || (a == 172 && b in 16..31) || (a == 192 && b == 168) || a == 169 && b == 254
    }

    private fun handleConfigure(): Response {
        return newFixedLengthResponse(Response.Status.REDIRECT, MIME_HTML, "").apply {
            addHeader("Location", "/#configure")
            addCorsHeaders(this)
        }
    }

    private fun handleStremioStream(uri: String, session: IHTTPSession): Response {
        val cleanPath = uri.removePrefix("/stream/").removeSuffix(".json")
        val slashIdx = cleanPath.indexOf('/')
        if (slashIdx == -1) {
            return jsonResponse(JSONObject().put("streams", JSONArray()))
        }
        val itemType = cleanPath.substring(0, slashIdx)
        val itemId = URLDecoder.decode(cleanPath.substring(slashIdx + 1), "UTF-8")

        Log.i(TAG, "Stremio stream request: type=$itemType, id=$itemId")
        DebugLog.log("Stream", "REQUEST type=$itemType id=$itemId")
        val streams = JSONArray()
        // Title/keyword used for the "No Streams Found" search fallback URL.
        var searchKeyword = ""

        try {
            val candidateFiles = ArrayList<JSONObject>()

            if (itemId.startsWith("pm_file_") || itemId.startsWith("pm:file_") || itemId.startsWith("pm:file:")) {
                val shortCode = itemId.substringAfterLast("file_").substringAfterLast("file:")
                val url = "$WP_API_BASE/search-files?search=$shortCode&limit=1"
                val req = Request.Builder().url(url).get().build()
                val resp = httpClient.newCall(req).execute()
                val bodyStr = resp.body?.string() ?: ""
                val json = JSONObject(bodyStr)
                val filesArr = json.optJSONObject("data")?.optJSONArray("files") ?: json.optJSONArray("files")
                if (filesArr != null && filesArr.length() > 0) {
                    for (i in 0 until filesArr.length()) {
                        val f = filesArr.getJSONObject(i)
                        // backend.php only carries title/size/mime for direct file IDs
                        // (no caption), so tag extraction runs on the title alone.
                        candidateFiles.add(JSONObject().apply {
                            put("short_code", shortCode)
                            put("title", f.optString("title", "Telegram File ($shortCode)"))
                            put("file_size", f.optLong("file_size", 0L))
                            put("mime", f.optString("mime", ""))
                            val fid = f.optString("file_id_mt", f.optString("file_id", ""))
                            if (fid.isNotEmpty()) put("file_id_mt", fid)
                        })
                    }
                } else {
                    candidateFiles.add(JSONObject().apply {
                        put("short_code", shortCode)
                        put("title", "Telegram File ($shortCode)")
                    })
                }
            } else if (itemId.startsWith("pm:post:") || itemId.startsWith("pm_post_") || itemId.startsWith("pm:post_")) {
                // WordPress post (movie or series episode) - mirrors backend.php
                val pm = Regex("^(?:pm_post_|pm:post_|pm:post:)(\\d+)(?::(\\d+):(\\d+))?$", RegexOption.IGNORE_CASE).find(itemId)
                val postId = pm?.groupValues?.getOrNull(1) ?: ""
                val season = pm?.groupValues?.getOrNull(2)?.toIntOrNull()
                val episode = pm?.groupValues?.getOrNull(3)?.toIntOrNull()
                if (postId.isNotEmpty()) {
                    val isSeries = itemType == "series" || (season != null && episode != null)
                    if (isSeries) {
                        addPostFiles(candidateFiles, postId, season ?: 1, episode ?: 1, filterEpisode = true)
                    } else {
                        val sfUrl = "$WP_API_BASE/stream-files?id=" + URLEncoder.encode("post:$postId", "UTF-8") + "&type=movie&limit=150"
                        // backend.php's movie fast path calls stream-files WITHOUT bot_id.
                        val sfBody = wpGet(sfUrl, 30)
                        DebugLog.log("Stream", "post stream-files len=${sfBody.length} head=${sfBody.take(80)}")
                        val sf = JSONObject(sfBody)
                        val resolvedTitle = sf.optString("resolved_title", "")
                        if (resolvedTitle.isNotEmpty()) searchKeyword = resolvedTitle
                        val resolvedYear = sf.optString("resolved_year", "")
                        val items = sf.optJSONArray("items") ?: sf.optJSONArray("files") ?: JSONArray()
                        for (i in 0 until items.length()) {
                            val f = items.getJSONObject(i)
                            val fTitle = decodeHtmlEntities(f.optString("title", ""))
                            val fCaption = f.optString("caption", "")
                            if (resolvedTitle.isEmpty() || movieFileMatchesTitle(fTitle, resolvedTitle, resolvedYear, fCaption)) {
                                candidateFiles.add(f)
                            }
                        }
                        if (candidateFiles.isEmpty()) addPostFiles(candidateFiles, postId, 0, 0, filterEpisode = false)
                    }
                }
            } else if (itemId.contains(":")) {
                // Standard Stremio ID (imdb/tmdb) with season:episode
                val url = "$WP_API_BASE/stream-files?id=" + URLEncoder.encode(itemId, "UTF-8") + "&type=$itemType&limit=60"
                val sf = JSONObject(wpGet(url, 30))
                if (searchKeyword.isEmpty()) searchKeyword = sf.optString("resolved_title", "")
                val itemsArr = sf.optJSONArray("items") ?: sf.optJSONArray("files")
                    ?: sf.optJSONObject("data")?.optJSONArray("items")
                    ?: sf.optJSONObject("data")?.optJSONArray("files")
                if (itemsArr != null) for (i in 0 until itemsArr.length()) candidateFiles.add(itemsArr.getJSONObject(i))
            } else {
                // Movie post or IMDb/TMDB ID
                val url = "$WP_API_BASE/stream-files?id=" + URLEncoder.encode(itemId, "UTF-8") + "&type=$itemType&limit=150"
                val sf = JSONObject(wpGet(url, 30))
                if (searchKeyword.isEmpty()) searchKeyword = sf.optString("resolved_title", "")
                val itemsArr = sf.optJSONArray("items") ?: sf.optJSONArray("files")
                    ?: sf.optJSONObject("data")?.optJSONArray("items")
                    ?: sf.optJSONObject("data")?.optJSONArray("files")
                if (itemsArr != null) for (i in 0 until itemsArr.length()) candidateFiles.add(itemsArr.getJSONObject(i))
            }

            DebugLog.log("Stream", "candidateFiles=${candidateFiles.size}")
            // Mirror backend.php's stream pipeline exactly:
            //   fd_group_split_parts() -> AIOStreams filters -> score sort -> per-res limit -> cap.
            val streamCfg = try {
                loadCatalogSettings().optJSONObject("stream_config") ?: JSONObject()
            } catch (e: Exception) { JSONObject() }
            groupSplitParts(candidateFiles)
            var filtered = applyStreamFilters(candidateFiles, streamCfg)
            val preferredRes = streamCfg.optString("preferred_resolution", "auto").ifEmpty { "auto" }
            filtered.sortWith(Comparator { a, b ->
                val sA = streamSortScore(a.optString("title", ""), a.optString("caption", ""), a.optLong("file_size", 0L), preferredRes)
                val sB = streamSortScore(b.optString("title", ""), b.optString("caption", ""), b.optLong("file_size", 0L), preferredRes)
                sB.compareTo(sA)
            })
            val maxPerRes = maxOf(0, streamCfg.optInt("max_streams_per_resolution", 0))
            if (maxPerRes > 0) {
                val resCounts = HashMap<String, Int>()
                val limited = ArrayList<JSONObject>()
                for (f in filtered) {
                    val rKey = f.optString("_resKey", "unknown")
                    val cnt = resCounts[rKey] ?: 0
                    if (cnt < maxPerRes) {
                        limited.add(f)
                        resCounts[rKey] = cnt + 1
                    }
                }
                filtered = limited
            }
            val maxTotal = maxOf(0, streamCfg.optInt("max_streams_total", 0))
            val streamCap = if (maxTotal > 0) maxTotal else 100
            if (filtered.size > streamCap) filtered = ArrayList(filtered.subList(0, streamCap))
            DebugLog.log("Stream", "filtered=${filtered.size} (cap=$streamCap)")
            candidateFiles.clear()
            candidateFiles.addAll(filtered)
            val hostHeader = session.headers["host"] ?: "127.0.0.1:8088"
            val fwdProto = (session.headers["x-forwarded-proto"] ?: "").lowercase()
            val streamOrigin = (if (fwdProto == "https") "https://" else "http://") + hostHeader

            for (f in candidateFiles) {
                val shortCode = f.optString("short_code", "")
                if (shortCode.isEmpty()) continue
                val title = f.optString("title", "Video $shortCode")
                val caption = f.optString("caption", "")
                val fileSize = f.optLong("file_size", 0L)
                val fileIdMt = f.optString("file_id_mt", f.optString("file_id", ""))
                val srcMime = f.optString("mime", "")

                val tags = extractMediaTags(title, caption)
                val resTag = tags.resolution.ifEmpty { "Direct" }
                val displayName = cleanMediaTitle(title).ifEmpty { title }
                val streamFileName = streamFilename(displayName, srcMime)
                val streamMime = guessVideoMime(streamFileName, srcMime)

                // Name: "PencariMovie <res>" + optional visual pills line
                val visualPills = tags.visual.filter { it == "HDR" || it == "DV" || it == "10bit" || it == "IMAX" }
                val streamName = "PencariMovie $resTag" +
                    (if (visualPills.isNotEmpty()) "\n" + visualPills.joinToString(" | ") else "")

                // Description: name / spec pills / language flags
                val metaPills = ArrayList<String>()
                if (tags.platform.isNotEmpty()) metaPills.add(tags.platform)
                if (tags.source.isNotEmpty()) metaPills.add(tags.source)
                tags.visual.forEach { if (it.isNotEmpty()) metaPills.add(it) }
                if (tags.codec.isNotEmpty()) metaPills.add(tags.codec)
                if (tags.audio.isNotEmpty()) metaPills.add(tags.audio)
                if (tags.edition.isNotEmpty()) metaPills.add(tags.edition)
                val metaPillsUnique = metaPills.distinct()

                val specBits = ArrayList<String>()
                if (fileSize > 0) specBits.add("💾 " + fdFormatBytes(fileSize))
                specBits.add("⚡ Telegram")
                metaPillsUnique.forEach { if (it != "HDR") specBits.add(it) }

                val langFlags = detectLanguageFlags(title, caption)
                val descLines = ArrayList<String>()
                descLines.add(displayName)
                descLines.add(specBits.joinToString(" • "))
                if (langFlags.isNotEmpty()) descLines.add("💬 " + langFlags.joinToString(" / "))
                val description = descLines.joinToString("\n")

                val payload = JSONObject().apply {
                    put("short_code", shortCode)
                    put("bot_id", tdLibManager.botId)
                    put("file_size", fileSize)
                    put("file_name", streamFileName)
                    put("mime", streamMime)
                    if (fileIdMt.isNotEmpty()) put("file_id_mt", fileIdMt)
                }
                val payloadB64 = Base64.encodeToString(
                    payload.toString().toByteArray(Charsets.UTF_8),
                    Base64.URL_SAFE or Base64.NO_WRAP or Base64.NO_PADDING
                )
                val streamUrl = "$streamOrigin/api/download/$payloadB64/" + URLEncoder.encode(streamFileName, "UTF-8")

                val ext = streamFileName.substringAfterLast('.', "").lowercase()
                val isWebReady = ext in listOf("mp4", "m4v", "webm")
                val bingeGroup = "pencarimovie-" + (if (resTag == "Direct") "direct" else resTag.lowercase())

                val streamItem = JSONObject().apply {
                    put("name", streamName)
                    put("description", description)
                    put("url", streamUrl)
                    put("behaviorHints", JSONObject().apply {
                        put("filename", streamFileName)
                        if (!isWebReady) put("notWebReady", true)
                        if (fileSize > 0) put("videoSize", fileSize)
                        put("bingeGroup", bingeGroup)
                    })
                }
                streams.put(streamItem)
            }
        } catch (e: Exception) {
            DebugLog.log("Stream", "EXCEPTION: ${e.message}")
            Log.e(TAG, "handleStremioStream error: ${e.message}", e)
        }

        // Mirror backend.php: when no playable stream exists, add a fallback tile.
        if (streams.length() == 0) {
            // Fallback tile links to the PencariMovie keyword search on the
            // CURRENT origin the client is using (localhost / LAN / tunnel / VPS
            // / custom domain) — not the sponsor URL.
            val host = (session.headers["host"] ?: "127.0.0.1:$listeningPort").substringBefore(",").trim()
            val hostName = host.substringBefore(":").lowercase()
            val forwardedProto = (session.headers["x-forwarded-proto"] ?: "").lowercase()
            val isTunnelHost = hostName.endsWith(".trycloudflare.com") || hostName.endsWith("tunnel.pencarimovie.com")
            val isHttps = forwardedProto == "https" || isTunnelHost
            val origin = (if (isHttps) "https://" else "http://") + host
            // Path segment is the title keyword. Resolve it like backend.php does:
            // strip a trailing ":season:episode" (series) and ask the API for the
            // resolved title, falling back to the base id.
            var keyword = searchKeyword
            if (keyword.isEmpty()) {
                val baseId = itemId.replace(Regex(":\\d+:\\d+$"), "")
                try {
                    val kUrl = "$WP_API_BASE/stream-files?id=" + java.net.URLEncoder.encode(baseId, "UTF-8") +
                        "&type=$itemType&limit=1"
                    keyword = JSONObject(wpGet(kUrl, 20)).optString("resolved_title", "").trim()
                } catch (_: Exception) {}
                if (keyword.isEmpty()) keyword = baseId
            }
            val searchUrl = "https://pencarimovie.com/search/" + java.net.URLEncoder.encode(keyword, "UTF-8") +
                "?ddldomain=" + java.net.URLEncoder.encode(origin, "UTF-8")
            streams.put(JSONObject().apply {
                put("name", "No Streams Found")
                put("description", "⚠️ No streams available for this title.")
                put("externalUrl", searchUrl)
            })
        }

        // Mirror backend.php: ALWAYS prepend the sponsored tile when configured.
        try {
            val sp = fetchSponsor()
            val adUrl = sp?.optString("url", "")?.trim() ?: ""
            if (adUrl.isNotEmpty()) {
                val sponsored = JSONObject().apply {
                    put("name", sp?.optString("name", "")?.takeIf { it.isNotEmpty() } ?: "PencariMovie")
                    put("description", sp?.optString("description", "")?.takeIf { it.isNotEmpty() }
                        ?: "Join our Telegram channel for updates & requests")
                    put("externalUrl", adUrl)
                }
                val withSponsor = JSONArray().apply {
                    put(sponsored)
                    for (i in 0 until streams.length()) put(streams.get(i))
                }
                DebugLog.log("Stream", "RESPONSE streams=${withSponsor.length()} (sponsored)")
                return jsonResponse(JSONObject().put("streams", withSponsor))
            }
        } catch (e: Exception) {
            DebugLog.log("Stream", "sponsor prepend failed: ${e.message}")
        }

        DebugLog.log("Stream", "RESPONSE streams=${streams.length()}")
        return jsonResponse(JSONObject().put("streams", streams))
    }

    private fun handleStremioCatalog(uri: String, session: IHTTPSession): Response {
        val cleanPath = uri.removePrefix("/catalog/").removeSuffix(".json")
        val parts = cleanPath.split("/")
        if (parts.size < 2) {
            return jsonResponse(JSONObject().put("metas", JSONArray()))
        }
        val type = parts[0]
        val catId = parts[1]
        val extraStr = if (parts.size > 2) parts[2] else ""

        // Bridged upstream catalog (up_{index}_{id}) -> forward to the upstream
        // addon (mirrors backend.php's up_* bridge in /catalog/...).
        Regex("^up_(\\d+)_(.+)$").find(catId)?.let { m ->
            return forwardUpstreamCatalog(type, m.groupValues[1].toIntOrNull() ?: -1, m.groupValues[2], extraStr)
        }

        var search = session.parameters["search"]?.firstOrNull() ?: ""
        var genre = session.parameters["genre"]?.firstOrNull() ?: ""
        var year = session.parameters["year"]?.firstOrNull() ?: ""
        var skip = session.parameters["skip"]?.firstOrNull()?.toIntOrNull() ?: 0
        if (extraStr.isNotEmpty()) {
            val decoded = try { URLDecoder.decode(extraStr, "UTF-8") } catch (e: Exception) { extraStr }
            for (pair in decoded.split("&")) {
                val idx = pair.indexOf('=')
                if (idx <= 0) continue
                val k = pair.substring(0, idx)
                val v = URLDecoder.decode(pair.substring(idx + 1), "UTF-8")
                when (k) {
                    "search" -> if (search.isEmpty()) search = v
                    "genre" -> if (genre.isEmpty()) genre = v
                    "year" -> if (year.isEmpty()) year = v
                    "skip" -> if (skip == 0) skip = v.toIntOrNull() ?: 0
                }
            }
        }
        // Year embedded in the catalog id, or supplied as a 4-digit "genre".
        if (year.isEmpty()) {
            val ym = Regex("_(\\d{4})$").find(catId)
            if (ym != null) year = ym.groupValues[1]
            else if (Regex("^\\d{4}$").matches(genre)) { year = genre; genre = "" }
            else if (catId == "year" || catId == "pm_series_year" || catId == "pm_files_year")
                year = java.util.Calendar.getInstance().get(java.util.Calendar.YEAR).toString()
        }

        DebugLog.log("Catalog", "REQUEST type=$type catId=$catId search=$search genre=$genre year=$year skip=$skip")

        val metas = JSONArray()
        try {
            val isFileCatalog = type == "other" || catId == "pm_files_year" || catId == "pm_files_latest" ||
                catId.startsWith("pm_topkw_") || catId == "pm_search_files"
            if (isFileCatalog) {
                buildFileCatalogMetas(metas, type, catId, search, genre, year, skip)
            } else {
                val limit = if (search.isNotEmpty()) 60 else 24
                buildPostCatalogMetas(metas, type, catId, search, genre, year, skip, limit)
            }
        } catch (e: Exception) {
            DebugLog.log("Catalog", "EXCEPTION: ${e.message}")
            Log.e(TAG, "handleStremioCatalog error: ${e.message}", e)
        }

        // Newest releases first (mirrors backend.php usort on releaseInfo).
        val sorted = ArrayList<JSONObject>()
        for (i in 0 until metas.length()) sorted.add(metas.getJSONObject(i))
        sorted.sortWith(compareByDescending { it.optString("releaseInfo", "").toIntOrNull() ?: 0 })
        val out = JSONArray()
        sorted.forEach { out.put(it) }

        DebugLog.log("Catalog", "RESPONSE metas=${out.length()}")
        return jsonResponse(JSONObject().put("metas", out))
    }

    private fun normalizeTitle(s: String): String =
        s.lowercase().replace(Regex("[^a-z0-9]+"), " ").trim()

    private val RELEASE_TAGS = "\\b(?:2160p|1080p|720p|540p|480p|360p|4k|uhd|fhd|hd|sd|bluray|blu-ray|bdrip|brrip|web-?dl|webrip|hdrip|hdtv|cam|predvd|predvdrip|ts|tc|r5|dvdrip|remux|hevc|x264|x265|h264|h265|aac.*|lubokvideo|yts|lulustream|galaxyrg|pahe)\\b.*"
    private val EDITION_TAGS = "\\b(?:extended(?:\\s*cut)?|directors?\\s*cut|unrated|imax|special\\s*edition|remastered|criterion|sub\\s*malay|malay\\s*sub|eng\\s*sub|sub\\s*eng|indosub|sub\\s*indo|subtitle|dubbed|multi\\s*audio|clean\\s*audio|hc|hardsub)\\b.*"
    private val SEQUEL_RE = "\\b(?:part\\s*(?:ii|iii|iv|v|\\d+)|(?:2|3|4|5|6|7|8|9)|chapter\\s*\\d+)\\b"
    private val UPLOADER_RE = "^(?:(?:sklink|office|movie|movies|king|film|films|cinema|cinemas|media|channel|tv|tele|tg|hub|world|zone|group|team|club|gang|bot|vip|official|download|downloads|upload|uploads|site|store|station|blasters|rockers|wap|villa|dub|yogi|verse|flix|pahe|galaxy|pms|mkv|link|links|hd)\\b\\s*)+$"
    /** Target-title release tag list (backend.php fd_movie_file_matches_title cleanTarget). */
    private val TARGET_TAGS = "\\b(?:2160p|1080p|720p|480p|360p|uhd|fhd|hd|sd|hdtv|web-?dl|webrip|bluray|blu-ray|remux|dvdrip|hevc|x264|x265|h264|h265|\\d+(?:\\.\\d+)?\\s*(?:gb|mb))\\b"
    /** Known leading release groups/domains stripped from filenames (backend.php line 6199). */
    private val LEADING_GROUPS = "^(?:prakytv|ngefilm\\s*store|runningmovieshd|kannadachallengers|mkvcinemas|vegamovies|moviesmod|pahe|galaxy|wetv|studioghibli|animerg|anime\\s*time\\s*studio\\s*ghibli\\s*movie\\s*\\d*|mcu|pms|pahe\\.li|layarkaca\\d*|cinemaindo|indoxxi|sklink(?:\\s*office)?|movie\\s*king|movie\\s*world|film\\s*world|tamilblasters|1tamilmv|tamilmv|tamilrockers|tamilyogi|tamilgun|isaidub|kuttymovies|moviesda|bolly4u|desiremovies|worldfree4u|katmoviehd|skymovieshd|filmywap|filmyzilla|extramovies|9xmovies|movieswood|jiorockers|todaypk|cinevood|hdhub4u|dotmovies|topmovies|modmobile|uhdmovies|allmovieshub|moviesverse|luxmovie|cinemalover|flixhub|movies4u|plexmovies|moviesrocker|mkvking|psa|yts)\\s+"
    /** Leading 2-4 letter country-code prefixes (backend.php line 6224). */
    private val COUNTRY_PREFIX = "^(?:ph|id|ina|my|kr|kor|jp|jpn|th|thai|cn|chi|us|uk|hk|vn|tw|ru|tr|es|fr|de|it|in)\\s+"

    private fun normWords(s: String): String =
        Regex("[^\\p{L}\\p{N}\\s]").replace(s, " ").lowercase().replace(Regex("\\s+"), " ").trim()

    /** Port of backend.php fd_is_series_file(). */
    private fun isSeriesFile(title: String, caption: String = ""): Boolean {
        var text = cleanMediaTitle(title)
        if (caption.isNotEmpty() &&
            Regex("^(?:video(?:\\.\\d+)*|\\d+|document|file)\\.(?:mp4|mkv|avi|mov|ts|flv)$", RegexOption.IGNORE_CASE).matches(text.trim())) {
            val firstCap = caption.split("\n")[0].trim()
            if (firstCap.isNotEmpty()) text = cleanMediaTitle(firstCap)
        }
        if (Regex("(?:^|[^a-z0-9])S\\d{1,2}\\s*[ ._-]*(?:E(?:P|PS|PISODE)?\\s*[ ._-]*\\d{1,4})+", RegexOption.IGNORE_CASE).containsMatchIn(text)) return true
        if (Regex("(?:^|[^a-z0-9])\\d{1,2}\\s*[xX]\\s*\\d{1,4}(?![0-9])(?:[^a-z0-9]|$)").containsMatchIn(text)) return true
        if (Regex("(?:^|[^a-z0-9])(?:season|musim)\\s*[ ._-]*\\d{1,2}(?:[^a-z0-9]|$)", RegexOption.IGNORE_CASE).containsMatchIn(text)) return true
        Regex("(?:^|[^a-z0-9])(?:EP|EPS|EPISODE|EPISOD|BAHAGIAN|BABAK)\\s*[ ._-]*0*(\\d{1,4})(?:[^a-z0-9]|$)", RegexOption.IGNORE_CASE).find(text)?.let {
            val n = it.groupValues[1].toIntOrNull() ?: 0
            if (n > 0 && (n < 1900 || n > 2100)) return true
        }
        Regex("(?:^|[^a-z0-9])E[ ._-]*0*(\\d{1,4})(?:[^a-z0-9]|$)", RegexOption.IGNORE_CASE).find(text)?.let {
            val n = it.groupValues[1].toIntOrNull() ?: 0
            if (n > 0 && (n < 1900 || n > 2100)) return true
        }
        if (Regex("(?:^|[^a-z0-9])S(\\d{1,2})(?=[ ._\\[\\(-]+(?:2160p|1080p|720p|480p|360p|4k|uhd|fhd|hd|sd|web|bluray|hdtv|complete|batch|ongoing|x264|x265|hevc)|$)", RegexOption.IGNORE_CASE).containsMatchIn(text)) return true
        return false
    }

    /** Port of backend.php fd_classify_season_episode(): (season, episode, episodeEnd). */
    private fun classifySeasonEpisode(rawTitle: String, seasonNum: Int = 0, episodeNum: Int = 0, caption: String = ""): Triple<Int, Int, Int> {
        var season = 0
        var episode = 0
        var episodeEnd = 0
        var title = cleanMediaTitle(rawTitle)
        if (caption.isNotEmpty() && Regex("^(?:video(?:\\.\\d+)*|\\d+|document|file)\\.(?:mp4|mkv|avi|mov|ts|flv)$", RegexOption.IGNORE_CASE).matches(title.trim())) {
            val first = caption.replace("\r", "\n").split("\n")[0].trim()
            if (first.isNotEmpty()) title = cleanMediaTitle(first)
        }
        fun g(p: String): List<String>? = Regex(p, RegexOption.IGNORE_CASE).find(title)?.groupValues

        // 1. Explicit SxxExx forms.
        val r1 = g("(?:^|[^a-z0-9])S(\\d{1,2})\\s*[ ._-]*E(?:P|PS|PISODE)?\\s*[ ._-]*(\\d{1,4})\\s*(?:[ ._-]+E(?:P|PS|PISODE)?|\\s*[-~–—]\\s*|\\s+(?:to|hingga|sampai)\\s+)\\s*(\\d{1,4})(?:[^a-z0-9]|$)")
        if (r1 != null) {
            season = r1[1].toIntOrNull() ?: 0; episode = r1[2].toIntOrNull() ?: 0; episodeEnd = r1[3].toIntOrNull() ?: 0
        } else {
            val r1b = g("(?:^|[^a-z0-9])S(\\d{1,2})\\s*[ ._-]*E(?:P|PS|PISODE)?\\s*[ ._-]*0*(\\d{1,4})(?=[^0-9]|$)")
            if (r1b != null) { season = r1b[1].toIntOrNull() ?: 0; episode = r1b[2].toIntOrNull() ?: 0 } else {
                val r1c = g("(?:^|[^a-z0-9])S(\\d{1,2})\\.0*(\\d{1,4})(?=[^0-9]|$)")
                if (r1c != null) { season = r1c[1].toIntOrNull() ?: 0; episode = r1c[2].toIntOrNull() ?: 0 } else {
                    val r2 = g("(?:^|[^a-z0-9])(\\d{1,2})\\s*[xX]\\s*(\\d{1,4})(?![0-9])(?:[^a-z0-9]|$)")
                    if (r2 != null) { season = r2[1].toIntOrNull() ?: 0; episode = r2[2].toIntOrNull() ?: 0 } else {
                        val r3 = g("(?:^|[^a-z0-9])(?:PART|VOL|VOLUME|COUR)\\s*[ ._-]*0*(\\d{1,2})[ ._-]+(?:E(?:P|PS|PISODE)?\\s*[ ._-]*)?0*(\\d{1,4})(?=[ ._\\-\\]\\)]|$)")
                        if (r3 != null) {
                            val n2 = r3[2].toIntOrNull() ?: 0
                            if (n2 < 1900 || n2 > 2100) { season = r3[1].toIntOrNull() ?: 0; episode = n2 }
                        }
                    }
                }
            }
        }

        // 4. Season / Musim / Part keyword fallback.
        if (season == 0) {
            val mm = g("(?:^|[^a-z0-9])0*(\\d{1,2})(?:st|nd|rd|th)\\s*[ ._-]*(?:season|musim)(?:[^a-z0-9]|$)")
                ?: g("(?:^|[^a-z0-9])(?:season|musim)\\s*[ ._-]*0*(\\d{1,2})(?:[^a-z0-9]|$)")
                ?: g("(?:^|[^a-z0-9])0*(\\d{1,2})\\s*[ ._-]*(?:season|musim)(?:[^a-z0-9]|$)")
                ?: g("(?:^|[^a-z0-9])(?:PART|VOL|VOLUME|COUR)\\s*[ ._-]*0*(\\d{1,2})(?=[ ._-]+(?:EP|E|\\d))")
                ?: g("(?:^|[^a-z0-9])S(\\d{1,2})(?=[^a-z0-9]|$)")
            if (mm != null) season = mm[1].toIntOrNull() ?: 0
        }

        // 5. Episode range tokens.
        if (episode == 0) {
            val r5 = g("(?:^|[^a-z0-9])(?:EP|EPS|EPISODE|EPISOD|E)\\s*[ ._-]*0*(\\d{1,4})\\s*(?:[ ._-]+(?:EP|EPS|EPISODE|EPISOD|E)|\\s*[-~–—]\\s*|\\s+(?:to|hingga|sampai)\\s+)\\s*0*(\\d{1,4})(?:[^a-z0-9]|$)")
            if (r5 != null) {
                val n1 = r5[1].toIntOrNull() ?: 0; val n2 = r5[2].toIntOrNull() ?: 0
                if (n1 > 0 && (n1 < 1900 || n1 > 2100) && n2 > 0 && (n2 < 1900 || n2 > 2100) && n2 >= n1 && (n2 - n1) <= 150) { episode = n1; episodeEnd = n2 }
            }
        }
        // 6. EP/Episode/Bahagian token.
        if (episode == 0) {
            val r6 = g("(?:^|[^a-z0-9])(?:EP|EPS|EPISODE|EPISOD|BAHAGIAN|BABAK)\\s*[ ._-]*0*(\\d{1,4})(?=[^0-9]|$)")
            if (r6 != null) { val n = r6[1].toIntOrNull() ?: 0; if (n > 0 && (n < 1900 || n > 2100)) episode = n }
        }
        // 7. E<digits> token.
        if (episode == 0) {
            val r7 = g("(?:^|[^a-z0-9])E[ ._-]*0*(\\d{1,4})(?:[^a-z0-9]|$)")
            if (r7 != null) { val n = r7[1].toIntOrNull() ?: 0; if (n > 0 && (n < 1900 || n > 2100)) episode = n }
        }
        // 7b. Part/Vol episode fallback (only when season not detected from it).
        if (episode == 0 && season == 0) {
            val r7b = g("(?:^|[^a-z0-9])(?:PART|VOL|VOLUME)\\s*[ ._-]*0*(\\d{1,4})(?:[^a-z0-9]|$)")
            if (r7b != null) { val n = r7b[1].toIntOrNull() ?: 0; if (n > 0 && (n < 1900 || n > 2100)) episode = n }
        }
        // 8. Bare episode numbers.
        if (episode == 0) {
            var clean = title.replace(Regex("\\.(mp4|mkv|avi|mov|ts|flv|webm)$", RegexOption.IGNORE_CASE), "")
            clean = clean.replace(Regex("\\b(?:19|20)\\d{2}\\b"), " ")
            val hit = Regex("(?<![0-9])[ ._\\[\\(-](\\d{1,3})[ ._\\]\\)-]+(?:2160p|1080p|720p|540p|480p|360p|4k|uhd|fhd|hd|sd|web|bluray|hdtv|malaysub|end|final|x264|x265|hevc|aac|anitv|ani|tv|raw|[a-z0-9_-]+|\\d{3,4}x\\d{3,4})", RegexOption.IGNORE_CASE).find(clean)
                ?: Regex("(?<![0-9])[ ._\\[\\(-](\\d{1,3})[ ._\\]\\)]*$").find(clean)
            if (hit != null) { val n = hit.groupValues[1].toIntOrNull() ?: 0; if (n > 0 && (n < 1900 || n > 2100)) episode = n }
        }
        // Fallbacks to DB media-rank columns.
        if (season == 0 && seasonNum > 0) { if (episode == 0 || episodeNum == episode) season = seasonNum }
        if (episode == 0 && episodeNum > 0) episode = episodeNum
        if (season == 0) season = 1
        return Triple(season, episode, episodeEnd)
    }

    /** Port of backend.php fd_movie_file_matches_title(). */
    private fun movieFileMatchesTitle(fileTitle: String, searchedTitle: String, searchedYear: String, fileCaption: String): Boolean {
        if (isSeriesFile(fileTitle, fileCaption)) return false
        val st = searchedTitle.trim()
        if (st.isEmpty()) return true

        // 1. Strict year check.
        val fileYears = Regex("\\b(19\\d{2}|20\\d{2})\\b").findAll(fileTitle).map { it.groupValues[1].toInt() }.toList()
        val hasFileYear = fileYears.isNotEmpty()
        val sy = searchedYear.trim()
        if (sy.isNotEmpty() && hasFileYear) {
            val ty = sy.toIntOrNull() ?: 0
            if (fileYears.none { Math.abs(it - ty) <= 1 }) return false
        }

        // 2. Normalised target title.
        var cleanTarget = decodeHtmlEntities(st).replace(Regex("\\s*[•··]\\s*.+$"), "")
        cleanTarget = cleanTarget.replace(Regex("\\s*&\\s*"), " and ")
        cleanTarget = cleanTarget.replace(Regex(TARGET_TAGS, RegexOption.IGNORE_CASE), " ")
        if (sy.isNotEmpty()) cleanTarget = cleanTarget.replace(Regex("\\b" + Regex.escape(sy) + "\\b"), " ")
        cleanTarget = cleanTarget.replace(Regex("[^\\p{L}\\p{N}\\s]"), " ").replace(Regex("\\s+"), " ").trim()
        var targetLower = cleanTarget.lowercase()
        if (targetLower.isEmpty()) targetLower = st.replace(Regex("[^\\p{L}\\p{N}\\s]"), " ").replace(Regex("\\s+"), " ").trim().lowercase()

        // 2b. Caption release-title check (backend.php lines 6166-6192).
        if (fileCaption.isNotEmpty()) {
            val firstLine = fileCaption.replace("\r", "\n").split("\n").firstOrNull()?.trim() ?: ""
            if (Regex("\\.(?:mp4|mkv|avi|mov|ts|flv|wmv)\\b", RegexOption.IGNORE_CASE).containsMatchIn(firstLine) ||
                Regex("\\b(19\\d{2}|20\\d{2})\\b").containsMatchIn(firstLine)) {
                val capClean = cleanMediaTitle(firstLine)
                var capCleanNorm = capClean.replace(Regex("[^\\p{L}\\p{N}\\s]"), " ").replace(Regex("\\s+"), " ").trim().lowercase()
                capCleanNorm = capCleanNorm.replace(Regex("[._\\s-]part[._\\s-]*0*\\d{1,4}", RegexOption.IGNORE_CASE), "")
                var baseCap = capCleanNorm.replace(Regex(RELEASE_TAGS, RegexOption.IGNORE_CASE), "")
                if (sy.isNotEmpty()) baseCap = baseCap.replace(Regex("\\b" + Regex.escape(sy) + "\\b.*"), "")
                else baseCap = baseCap.replace(Regex("\\b(?:19\\d{2}|20\\d{2})\\b.*"), "")
                baseCap = baseCap.replace(Regex(EDITION_TAGS, RegexOption.IGNORE_CASE), "")
                baseCap = baseCap.replace(Regex("\\s+"), " ").trim()
                if (baseCap.isNotEmpty() && baseCap == targetLower) {
                    if (sy.isNotEmpty()) {
                        val cym = Regex("\\b(19\\d{2}|20\\d{2})\\b").find(firstLine)
                        if (cym != null) {
                            if (Math.abs((cym.groupValues[1].toIntOrNull() ?: 0) - (sy.toIntOrNull() ?: 0)) <= 1) return true
                        } else {
                            return true
                        }
                    } else {
                        return true
                    }
                }
            }
        }

        // 3. Clean filename, strip leading channel/group tags and known release groups.
        var fClean = cleanMediaTitle(fileTitle)
        fClean = fClean.replace(Regex("^(?:@\\w+[._\\s]+|\\[[^\\]]+\\][._\\s]*|\\([^\\)]+\\)[._\\s]*)", RegexOption.IGNORE_CASE), "")
        fClean = fClean.replace(Regex(LEADING_GROUPS, RegexOption.IGNORE_CASE), "")
        var fCleanNorm = fClean.replace(Regex("[^\\p{L}\\p{N}\\s]"), " ").replace(Regex("\\s+"), " ").trim().lowercase()

        // 4/5. Strip split parts, release tags, year and editions.
        var fWithoutSplit = fCleanNorm.replace(Regex("[._\\s-]part[._\\s-]*0*\\d{1,4}", RegexOption.IGNORE_CASE), "")
        fWithoutSplit = fWithoutSplit.replace(Regex("\\.(?:mp4|mkv|avi|mov|ts|flv|wmv)\\.0*\\d{1,4}$", RegexOption.IGNORE_CASE), "")
        var baseF = fWithoutSplit.replace(Regex(RELEASE_TAGS, RegexOption.IGNORE_CASE), "")
        if (sy.isNotEmpty()) baseF = baseF.replace(Regex("\\b" + Regex.escape(sy) + "\\b.*"), "")
        if (!Regex("\\b(19\\d{2}|20\\d{2})\\b").containsMatchIn(targetLower)) baseF = baseF.replace(Regex("\\b(?:19\\d{2}|20\\d{2})\\b.*"), "")
        baseF = baseF.replace(Regex(EDITION_TAGS, RegexOption.IGNORE_CASE), "")
        baseF = baseF.replace(Regex("\\b(?:mp4|mkv|avi|mov|ts|flv|wmv)\\b", RegexOption.IGNORE_CASE), "")
        baseF = baseF.replace(Regex("\\s+"), " ").trim()

        // Strip leading 2-4 letter country code (never "the").
        baseF = baseF.replace(Regex(COUNTRY_PREFIX + "(?=" + Regex.escape(targetLower) + ")", RegexOption.IGNORE_CASE), "")
        baseF = baseF.replace(Regex("\\s+"), " ").trim()

        // 6. Sequel guard.
        if (!Regex(SEQUEL_RE, RegexOption.IGNORE_CASE).containsMatchIn(targetLower) &&
            Regex(SEQUEL_RE, RegexOption.IGNORE_CASE).containsMatchIn(baseF)) return false

        // 7. Direct comparison.
        if (baseF.isNotEmpty()) {
            if (baseF == targetLower) return true
            if (baseF.endsWith(" $targetLower")) {
                val prefix = baseF.removeSuffix(targetLower).trim()
                if (isUploaderPrefix(prefix)) return true
            }
        }

        // 8. Ampersand / dan / possessive variants.
        val variants = listOf(
            targetLower.replace(" and ", " dan "),
            targetLower.replace(" dan ", " and "),
            targetLower.replace(" and ", " "),
            targetLower.replace(" dan ", " "),
            targetLower.trimEnd('s')
        )
        if (variants.contains(baseF)) return true
        for (tv in variants) {
            if (tv.isNotEmpty() && baseF.endsWith(" $tv")) {
                val prefix = baseF.removeSuffix(tv).trim()
                if (isUploaderPrefix(prefix)) return true
            }
        }

        // 9. Multi-word exact word sequence match.
        if (hasFileYear && sy.isNotEmpty()) {
            val tw = targetLower.split(" ").filter { it.length > 1 }
            val fw = baseF.split(" ").filter { it.length > 1 }
            if (tw.size >= 2 && fw == tw) return true
        }
        return false
    }

    /** True when a filename prefix is composed only of uploader/group/site words (backend.php line 6254). */
    private fun isUploaderPrefix(prefix: String): Boolean {
        if (prefix.isEmpty()) return false
        if (!Regex(UPLOADER_RE, RegexOption.IGNORE_CASE).matches(prefix)) return false
        return !Regex("\\b(?:the|a|an|blade|late|maze|avatar|chapter|part)\\b", RegexOption.IGNORE_CASE).containsMatchIn(prefix)
    }

    private fun qualityRank(tags: MediaTags): Int = when (tags.resolution) {
        "4K" -> 4
        "1080p" -> 3
        "720p" -> 2
        "540p", "480p", "360p" -> 1
        else -> 0
    }

    /** Port of backend.php fd_calculate_stream_sort_score(). */
    private fun streamSortScore(title: String, caption: String, fileSize: Long, preferredRes: String = "auto"): Double {
        val tags = extractMediaTags(title, caption)
        var resScore = when (tags.resolution.lowercase()) {
            "4k" -> 6_000_000_000.0
            "1080p" -> 5_000_000_000.0
            "720p" -> 4_000_000_000.0
            "540p" -> 3_000_000_000.0
            "480p" -> 2_000_000_000.0
            "360p" -> 1_000_000_000.0
            else -> 500_000_000.0
        }
        if (preferredRes != "auto" && tags.resolution.lowercase() == preferredRes.lowercase()) {
            resScore += 10_000_000_000.0
        }
        val sourceScore = when (tags.source.uppercase()) {
            "REMUX" -> 80_000_000.0
            "BLURAY" -> 70_000_000.0
            "WEB-DL" -> 60_000_000.0
            "WEBRIP" -> 50_000_000.0
            "HDRIP" -> 40_000_000.0
            "HDTV" -> 30_000_000.0
            "DVDRIP" -> 20_000_000.0
            "CAM" -> 1_000_000.0
            else -> 30_000_000.0
        }
        var visualScore = 0.0
        if (tags.visual.contains("DV")) visualScore += 4_000_000.0
        if (tags.visual.contains("HDR")) visualScore += 2_000_000.0
        if (tags.visual.contains("10bit")) visualScore += 1_000_000.0
        val codecScore = when (tags.codec.uppercase()) {
            "AV1" -> 300_000.0
            "HEVC" -> 200_000.0
            "H.264" -> 100_000.0
            else -> 0.0
        }
        val sizeScore = minOf(99_999.0, maxOf(0.0, fileSize.toDouble() / (1024.0 * 1024.0)))
        return resScore + sourceScore + visualScore + codecScore + sizeScore
    }

    private data class SplitPartInfo(
        val isPart: Boolean,
        val baseKey: String,
        val cleanBase: String,
        val partNum: Int,
        val totalParts: Int
    )

    /** Port of backend.php fd_extract_split_part_info(). */
    private fun extractSplitPartInfo(filename: String, caption: String = ""): SplitPartInfo {
        val f = filename.trim()
        val cap = caption.trim()
        var partNum = 0
        var totalParts = 0
        var matched = false
        var cleanBase = f

        val rePart = Regex("[._\\s-]part[._\\s-]*0*(\\d{1,4})(?:[._\\s/-]+(?:of[._\\s-]+)?0*(\\d{1,4}))?", RegexOption.IGNORE_CASE)
        val reDotPart = Regex("[._\\s-]0*(\\d{1,3})\\.(mp4|mkv|avi|webm)$", RegexOption.IGNORE_CASE)
        val reExtPart = Regex("\\.(?:mp4|mkv|avi|webm)\\.0*(\\d{1,4})$", RegexOption.IGNORE_CASE)

        val m1 = rePart.find(f)
        val m2 = reDotPart.find(f)
        val m3 = reExtPart.find(f)
        if (m1 != null) {
            partNum = m1.groupValues[1].toIntOrNull() ?: 0
            if (m1.groupValues[2].isNotEmpty()) totalParts = m1.groupValues[2].toIntOrNull() ?: 0
            matched = true
            cleanBase = rePart.replace(f, "")
        } else if (m2 != null && !Regex("\\b(2160p|1080p|720p|480p|360p)\\b", RegexOption.IGNORE_CASE).containsMatchIn(m2.value)) {
            partNum = m2.groupValues[1].toIntOrNull() ?: 0
            matched = true
            val g1 = m2.groupValues[1]
            val g2 = m2.groupValues[2]
            cleanBase = Regex("[._\\s-]0*" + Regex.escape(g1) + "\\.$g2$", RegexOption.IGNORE_CASE).replace(f, ".$g2")
        } else if (m3 != null) {
            partNum = m3.groupValues[1].toIntOrNull() ?: 0
            matched = true
            cleanBase = Regex("\\.0*" + Regex.escape(m3.groupValues[1]) + "$", RegexOption.IGNORE_CASE).replace(f, "")
        }

        if (matched && totalParts == 0 && cap.isNotEmpty()) {
            val cm = Regex("part[._\\s-]*0*$partNum\\s*[/|of]\\s*0*(\\d{1,4})", RegexOption.IGNORE_CASE).find(cap)
            if (cm != null) totalParts = cm.groupValues[1].toIntOrNull() ?: 0
        }

        if (!matched || partNum <= 0) return SplitPartInfo(false, "", "", 0, 0)

        val baseKey = Regex("[^\\p{L}\\p{N}]+").replace(cleanBase, ".").trim('.').lowercase()
        return SplitPartInfo(true, baseKey, cleanBase, partNum, totalParts)
    }

    /** Port of backend.php fd_group_split_parts(): annotates split parts and orders them ascending. */
    private fun groupSplitParts(files: ArrayList<JSONObject>) {
        files.sortWith(Comparator { a, b ->
            val aInfo = extractSplitPartInfo(a.optString("title", ""), a.optString("caption", ""))
            val bInfo = extractSplitPartInfo(b.optString("title", ""), b.optString("caption", ""))
            if (aInfo.isPart && bInfo.isPart && aInfo.baseKey == bInfo.baseKey) {
                aInfo.partNum.compareTo(bInfo.partNum)
            } else {
                0
            }
        })
        for (f in files) {
            val info = extractSplitPartInfo(f.optString("title", ""), f.optString("caption", ""))
            if (info.isPart) {
                f.put("is_split_part", true)
                f.put("part_num", info.partNum)
                f.put("total_parts", info.totalParts)
                f.put("clean_base", info.cleanBase)
            }
        }
    }

    /** Port of backend.php $categorizeResolution(). */
    private fun categorizeResolutionKey(title: String, caption: String): String =
        when (extractMediaTags(title, caption).resolution.lowercase()) {
            "4k" -> "4k"
            "1080p" -> "1080p"
            "720p" -> "720p"
            "540p", "480p", "360p" -> "sd"
            else -> "unknown"
        }

    /** Port of backend.php $categorizeQuality() (note: webrip intentionally maps to unknown). */
    private fun categorizeQualityKey(title: String, caption: String): String =
        when (extractMediaTags(title, caption).source.lowercase()) {
            "remux" -> "remux"
            "bluray" -> "bluray"
            "web-dl" -> "webdl"
            "hdrip" -> "webrip"
            "hdtv" -> "hdtv"
            "cam" -> "cam"
            else -> "unknown"
        }

    /** Port of backend.php's AIOStreams-style stream filter block. */
    private fun applyStreamFilters(files: List<JSONObject>, cfg: JSONObject): ArrayList<JSONObject> {
        val resFilter = cfg.optJSONObject("resolutions")
        val qualityFilter = cfg.optJSONObject("qualities")
        val encodeFilter = cfg.optJSONObject("encodes")
        val visualFilter = cfg.optJSONObject("visual_tags")
        val excludeCam = cfg.optBoolean("exclude_cam", false)
        val excludeUnplayable = if (cfg.has("exclude_unplayable")) cfg.optBoolean("exclude_unplayable", true) else true
        val minSizeMb = maxOf(0, cfg.optInt("min_size_mb", 0))
        val maxSizeGb = maxOf(0, cfg.optInt("max_size_gb", 0))
        val excludedKeywords = cfg.optString("excluded_keywords", "").lowercase().split(",").map { it.trim() }.filter { it.isNotEmpty() }
        val requiredKeywords = cfg.optString("required_keywords", "").lowercase().split(",").map { it.trim() }.filter { it.isNotEmpty() }
        val reGeneric = Regex("^(?:video(?:\\.\\d+)*|\\d+|document|file)\\.(?:mp4|mkv|avi|mov|ts|flv)$", RegexOption.IGNORE_CASE)
        val rePlayable = Regex("\\.(mp4|m4v|mkv|webm|avi|mov|ts|m2ts|flv|wmv|3gp|mpg|mpeg|mp3|m4a|flac|wav|ogg|opus|aac)$", RegexOption.IGNORE_CASE)
        val reArchive = Regex("\\.(?:zip|rar|7z|tar|gz|bz2|xz|iso|bin|exe|apk|pdf|epub)$", RegexOption.IGNORE_CASE)
        val reRawSplit = Regex("\\.(?:0\\d{2,3}|\\d{3})$")

        val out = ArrayList<JSONObject>()
        for (fItem in files) {
            var fTitle = fItem.optString("title", "")
            val fCaption = fItem.optString("caption", "")
            val fSize = fItem.optLong("file_size", 0L)
            if (fCaption.isNotEmpty() && reGeneric.matches(fTitle.trim())) {
                val firstCap = fCaption.split("\n")[0].trim()
                if (firstCap.isNotEmpty()) fTitle = firstCap
            }
            val lowerTitle = fTitle.lowercase()
            val itemTags = extractMediaTags(fTitle, fCaption)

            // 1. Resolution filter
            val rKey = categorizeResolutionKey(fTitle, fCaption)
            if (resFilter != null && resFilter.has(rKey) && !resFilter.optBoolean(rKey, true)) continue

            // 2. Quality filter
            val qKey = categorizeQualityKey(fTitle, fCaption)
            if (excludeCam && qKey == "cam") continue
            if (qualityFilter != null && qualityFilter.has(qKey) && !qualityFilter.optBoolean(qKey, true)) continue

            // Unplayable filter (archives / raw split chunks)
            val hasPlayableExt = rePlayable.containsMatchIn(fTitle)
            val isArchive = reArchive.containsMatchIn(fTitle)
            val isRawSplit = !hasPlayableExt && reRawSplit.containsMatchIn(fTitle)
            val isUnplayableItem = !hasPlayableExt && (isRawSplit || isArchive)
            if (excludeUnplayable && isUnplayableItem) continue

            // 3. Encodes filter
            if (itemTags.codec == "HEVC" && encodeFilter != null && encodeFilter.has("hevc") && !encodeFilter.optBoolean("hevc", true)) continue
            if (itemTags.codec == "H.264" && encodeFilter != null && encodeFilter.has("avc") && !encodeFilter.optBoolean("avc", true)) continue
            if (itemTags.codec == "AV1" && encodeFilter != null && encodeFilter.has("av1") && !encodeFilter.optBoolean("av1", true)) continue

            // 4. Visual tags filter
            if (itemTags.visual.contains("HDR") && visualFilter != null && visualFilter.has("hdr") && !visualFilter.optBoolean("hdr", true)) continue
            if (itemTags.visual.contains("DV") && visualFilter != null && visualFilter.has("dv") && !visualFilter.optBoolean("dv", true)) continue

            // 5. Size range
            if (minSizeMb > 0 && fSize > 0 && fSize < minSizeMb.toLong() * 1048576L) continue
            if (maxSizeGb > 0 && fSize > 0 && fSize > maxSizeGb.toLong() * 1073741824L) continue

            // 6. Excluded keywords
            if (excludedKeywords.isNotEmpty() && excludedKeywords.any { lowerTitle.contains(it) }) continue
            // 7. Required keywords
            if (requiredKeywords.isNotEmpty() && requiredKeywords.none { lowerTitle.contains(it) }) continue

            fItem.put("_resKey", rKey)
            out.add(fItem)
        }
        return out
    }

    /** Appends a post's Telegram files (optionally filtered to a season/episode). */
    private fun addPostFiles(out: ArrayList<JSONObject>, postId: String, season: Int, episode: Int, filterEpisode: Boolean) {
        val json = JSONObject(wpGet(withBotId("$WP_API_BASE/post-files?post_id=$postId&limit=150"), 30))
        val files = json.optJSONObject("data")?.optJSONArray("files") ?: json.optJSONArray("files") ?: JSONArray()
        for (i in 0 until files.length()) {
            val f = files.getJSONObject(i)
            if (filterEpisode) {
                // Mirror backend.php fd_classify_season_episode + fd_file_matches_episode.
                val parsed = classifySeasonEpisode(
                    f.optString("title", ""), f.optInt("season_num", 0), f.optInt("episode_num", 0), f.optString("caption", "")
                )
                val ps = parsed.first
                val pe = parsed.second
                val peEnd = parsed.third
                val matches = when {
                    ps != season -> false
                    pe > 0 && peEnd >= pe -> episode in pe..peEnd
                    pe <= 0 -> false
                    else -> pe == episode
                }
                if (!matches) continue
            }
            out.add(f)
        }
    }

    /** Unwraps a WP REST response into a single object ({data:[{...}]} / {data:{...}} / plain). */
    private fun unwrapWpObject(body: String): JSONObject {
        val trimmed = body.trim()
        if (trimmed.startsWith("[")) return JSONArray(trimmed).optJSONObject(0) ?: JSONObject()
        val start = trimmed.indexOf('{')
        val json = try { JSONObject(if (start >= 0) trimmed.substring(start) else trimmed) } catch (e: Exception) { JSONObject() }
        json.optJSONArray("data")?.let { if (it.length() > 0) return it.optJSONObject(0) ?: json }
        json.optJSONObject("data")?.let { return it }
        return json
    }

    private fun wpGet(url: String, timeoutSec: Long = 15): String {
        val req = Request.Builder().url(url).get().addHeader("Accept", "application/json").build()
        val client = if (timeoutSec > 15) resolveClient else httpClient
        return client.newCall(req).execute().body?.string() ?: "{}"
    }

    /** backend.php's fd_fetch_stream_ajax always tags requests with the active bot id. */
    private fun withBotId(url: String): String {
        val bot = tdLibManager.botId
        if (bot.isEmpty()) return url
        return if (url.contains("?")) "$url&bot_id=$bot" else "$url?bot_id=$bot"
    }

    private fun wpArray(url: String): JSONArray {
        val body = wpGet(url)
        val json = JSONObject(if (body.contains("{")) body.substring(body.indexOf('{')) else body)
        return json.optJSONArray("data") ?: json.optJSONArray("items") ?: json.optJSONArray("files") ?: JSONArray()
    }

    /** Port of backend.php fd_clean_post_title() (entities + fd_clean_media_title). */
    private fun cleanPostTitle(title: String): String {
        val decoded = decodeHtmlEntities(title)
        val t = cleanMediaTitle(decoded).trim()
        return if (t.isNotEmpty()) t else decoded.trim()
    }

    /** Port of backend.php fd_clean_post_plot(): strip_tags -> decode entities -> trim. */
    private fun cleanPostPlot(plot: String): String {
        val stripped = plot.replace(Regex("<[^>]*>"), "")
        return decodeHtmlEntities(stripped).trim()
    }

    private fun decodeHtmlEntities(s: String): String {
        // Built by concatenation so the entity names are never interpreted as HTML.
        val amp = "&" + "amp;"
        val quot = "&" + "quot;"
        val apos = "&" + "apos;"
        val lt = "&" + "lt;"
        val gt = "&" + "gt;"
        val hellip = "&" + "hellip;"
        val nbsp = "&" + "nbsp;"
        val ndash = "&" + "ndash;"
        val t = s.replace(amp, "&").replace("&#038;", "&").replace("'", "'")
            .replace(apos, "'").replace(quot, "\"").replace("&#039;", "'")
            .replace(lt, "<").replace(gt, ">").replace(hellip, "…")
            .replace(nbsp, " ").replace("&#8217;", "’").replace(ndash, "–")
        return t.replace(Regex("&#(\\d+);")) { m -> try { String(Character.toChars(m.groupValues[1].toInt())) } catch (e: Exception) { "" } }
    }

    /** Port of backend.php fd_extract_post_genres() (tags + categories minus CMS labels). */
    private fun extractPostGenres(post: JSONObject): JSONArray {
        val tags = post.optJSONArray("tags")
        val cats = post.optJSONArray("categories")
        val raw = ArrayList<String>()
        if (tags != null) for (i in 0 until tags.length()) raw.add(tags.optString(i))
        if (cats != null) for (i in 0 until cats.length()) raw.add(cats.optString(i))
        val blacklist = setOf("telegram", "tv shows", "tv show", "tvseries", "movies", "movie", "uncategorized")
        val seen = HashSet<String>()
        val genres = ArrayList<String>()
        for (item in raw) {
            val clean = decodeHtmlEntities(item).trim()
            if (clean.isEmpty()) continue
            for (part in clean.split(Regex("\\s*&\\s*"))) {
                val p = part.trim()
                if (p.isEmpty()) continue
                val lower = p.lowercase()
                if (lower in blacklist || seen.contains(lower)) continue
                seen.add(lower)
                genres.add(p)
            }
        }
        if (genres.isEmpty() && cats != null) {
            for (i in 0 until cats.length()) {
                val clean = decodeHtmlEntities(cats.optString(i)).trim()
                if (clean.isNotEmpty() && !seen.contains(clean.lowercase())) {
                    seen.add(clean.lowercase())
                    genres.add(clean)
                }
            }
        }
        return if (genres.isNotEmpty()) JSONArray(genres) else JSONArray(listOf("Drama"))
    }

    /** Port of backend.php fd_extract_release_year(). */
    private fun extractReleaseYear(title: String, date: String = ""): String {
        Regex("\\b(19\\d{2}|20\\d{2})\\b").find(title)?.let { return it.groupValues[1] }
        if (date.isNotEmpty()) {
            Regex("\\b(19\\d{2}|20\\d{2})\\b").find(date)?.let { return it.groupValues[1] }
        }
        return ""
    }

    private val COUNTRY_NAMES = mapOf(
        "MY" to "Malaysia", "ID" to "Indonesia", "SG" to "Singapore", "TH" to "Thailand",
        "PH" to "Philippines", "VN" to "Vietnam", "KR" to "Korea", "JP" to "Japan",
        "CN" to "China", "HK" to "Hong Kong", "TW" to "Taiwan", "IN" to "India",
        "US" to "United States", "GB" to "United Kingdom", "AU" to "Australia",
        "DE" to "Germany", "NL" to "Netherlands", "FR" to "France", "CA" to "Canada"
    )
    private val AVAILABLE_COUNTRY_CODES = listOf(
        "MY", "ID", "SG", "TH", "PH", "VN", "KR", "JP", "CN", "HK", "TW", "IN", "US", "GB", "AU", "DE", "NL", "FR", "CA"
    )

    /** Port of backend.php fd_detect_country(): configured > CF header > default 'MY'. */
    private fun fdDetectCountry(): Triple<String, String, String> {
        val configured = try { loadCatalogSettings().optString("country", "").trim().uppercase() } catch (e: Exception) { "" }
        var code: String
        var source: String
        if (configured.isNotEmpty()) {
            code = configured; source = "configured"
        } else {
            code = ""; source = "cf-header"
        }
        if (code.isEmpty() || code == "XX" || code == "T1" || code.length != 2) {
            code = "MY"; source = "default"
        }
        return Triple(code, COUNTRY_NAMES[code] ?: code, source)
    }

    private fun detectCountryCode(): String = fdDetectCountry().first

    private fun buildPostMeta(post: JSONObject, type: String, releaseYear: String): JSONObject {
        val itemType = if (type == "series") "series" else "movie"
        return JSONObject().apply {
            put("id", "pm:post:" + post.optString("id", "0"))
            put("type", itemType)
            put("name", cleanPostTitle(post.optString("title", "")))
            put("poster", post.optString("thumbnail_url", ""))
            put("posterShape", "poster")
            put("description", cleanPostPlot(post.optString("excerpt", post.optString("content", ""))))
            put("genres", extractPostGenres(post))
            if (releaseYear.isNotEmpty()) put("releaseInfo", releaseYear)
        }
    }

    /** Movie/Series browse + search catalogs -> WordPress posts (mirrors backend.php). */
    private fun buildPostCatalogMetas(
        metas: JSONArray, type: String, catId: String, search: String,
        genre: String, year: String, skip: Int, limit: Int
    ) {
        val country = detectCountryCode()
        if (search.isNotEmpty()) {
            val u = StringBuilder("$WP_API_BASE/search?search=")
                .append(URLEncoder.encode(search, "UTF-8")).append("&limit=30&offset=").append(skip)
            if (country.isNotEmpty()) u.append("&country=").append(country)
            val arr = wpArray(withBotId(u.toString()))
            for (i in 0 until arr.length()) {
                val post = arr.optJSONObject(i) ?: continue
                if (post.optInt("id", 0) == 0) continue
                val title = post.optString("title", "")
                val cats = post.optJSONArray("categories")
                val catText = if (cats != null) (0 until cats.length()).joinToString(" ") { cats.optString(it) } else ""
                val isSeries = Regex("tvseries|series|season|episode|drama", RegexOption.IGNORE_CASE).containsMatchIn("$title $catText")
                if (type == "movie" && isSeries) continue
                if (type == "series" && !isSeries) continue
                val ry = extractReleaseYear(title, post.optString("date", ""))
                if (year.isNotEmpty() && ry.isNotEmpty() && ry != year) continue
                metas.put(buildPostMeta(post, type, ry))
            }
            return
        }

        val params = StringBuilder("limit=$limit&offset=$skip")
            .append("&media_type=").append(if (type == "series") "series" else "movie")
        val baseCatId = catId.replace(Regex("_(year|genre|latest|\\d{4}|older)$"), "")
        val countryMap = mapOf(
            "pm_movies_malay" to "malay", "pm_movies_indo" to "indonesian", "pm_movies_korean" to "korea",
            "pm_movies_japan" to "japan", "pm_movies_anime" to "anime", "pm_movies_chinese" to "china",
            "pm_movies_thai" to "thai", "pm_movies_bollywood" to "bollywood",
            "pm_movies_philippines" to "filipino", "pm_movies_pinoy" to "filipino",
            "pm_movies_english" to "english",
            "pm_series_kdrama" to "korea", "pm_series_anime" to "anime", "pm_series_japan" to "japan",
            "pm_series_malay" to "malay", "pm_series_cdrama" to "china", "pm_series_thai" to "thai",
            "pm_series_philippines" to "filipino", "pm_series_pinoy" to "filipino",
            "pm_series_english" to "english", "pm_series_indo" to "indonesian"
        )
        when {
            catId == "top" || catId == "pm_series_top" -> {
                if (year.isEmpty()) params.append("&category=popular")
            }
            catId == "year" || catId == "pm_movies_latest" || catId == "pm_series_year" || catId == "pm_series_latest" -> {}
            countryMap.containsKey(baseCatId) -> params.append("&category=").append(countryMap[baseCatId])
            countryMap.containsKey(catId) -> params.append("&category=").append(countryMap[catId])
            catId.startsWith("pm_cat_") -> {
                var slug = catId.removePrefix("pm_cat_")
                if (slug == "indo") slug = "indonesian"
                params.append("&category=").append(slug)
            }
        }
        if (country.isNotEmpty()) params.append("&country=").append(country)
        if (genre.isNotEmpty()) params.append("&genre=").append(URLEncoder.encode(genre, "UTF-8"))
        if (year.isNotEmpty()) params.append("&year=").append(year)

        val arr = wpArray(withBotId("$WP_API_BASE/posts?$params"))
        for (i in 0 until arr.length()) {
            val post = arr.optJSONObject(i) ?: continue
            if (post.optInt("id", 0) == 0) continue
            val ry = extractReleaseYear(post.optString("title", ""), post.optString("date", ""))
            metas.put(buildPostMeta(post, type, ry))
        }
    }

    /** Telegram-files catalogs (other/top, year, trending keywords) -> direct files. */
    private fun buildFileCatalogMetas(
        metas: JSONArray, type: String, catId: String, search: String,
        genre: String, year: String, skip: Int
    ) {
        val term = when {
            search.isNotEmpty() -> search
            catId.startsWith("pm_topkw_") -> trendingKeywordFor(catId)
            else -> if (year.isNotEmpty()) year else "__latest__"
        }
        if (term.isEmpty()) return
        val url = withBotId("$WP_API_BASE/search-files?search=" + URLEncoder.encode(term, "UTF-8") + "&limit=100&offset=$skip")
        val j = JSONObject(wpGet(url))
        val filesArr = j.optJSONObject("data")?.optJSONArray("files") ?: j.optJSONArray("files") ?: JSONArray()
        for (i in 0 until filesArr.length()) {
            val f = filesArr.getJSONObject(i)
            val code = f.optString("short_code", "")
            if (code.isEmpty()) continue
            val title = decodeHtmlEntities(f.optString("title", "Telegram File"))
            val size = f.optLong("file_size", 0L)
            val t = title.lowercase()
            if (type != "other") {
                val isEp = Regex("\\b(e\\d+|ep\\d+|episod\\w*|s\\d+e\\d+|part\\d+)\\b").containsMatchIn(t)
                if (type == "movie" && isEp && catId != "pm_search_files") continue
                if (type == "series" && !isEp && catId != "pm_search_files") continue
            }
            val pills = ArrayList<String>()
            when {
                Regex("\\b(2160p|4k|uhd)\\b").containsMatchIn(t) -> pills.add("4K")
                Regex("\\b(1080p|fhd)\\b").containsMatchIn(t) -> pills.add("1080p")
                Regex("\\b(720p|hd)\\b").containsMatchIn(t) -> pills.add("720p")
                Regex("\\b(480p|360p|sd)\\b").containsMatchIn(t) -> pills.add("SD")
            }
            when {
                Regex("\\b(bluray|blu-ray|remux)\\b").containsMatchIn(t) -> pills.add("BluRay")
                Regex("\\b(web-?dl|webrip)\\b").containsMatchIn(t) -> pills.add("WEB-DL")
            }
            if (Regex("\\b(hevc|x265|h265)\\b").containsMatchIn(t)) pills.add("HEVC")
            if (size > 0) pills.add(fdFormatBytes(size))
            val pillLine = if (pills.isNotEmpty()) pills.joinToString(" · ") else "Ready to stream"
            val primary = if (catId == "pm_files_year" || catId == "pm_files_latest") "Year" else "Trending File"
            val genres = (listOf(primary) + pills).distinct()
            if (genre.isNotEmpty() && genre !in genres) continue
            if (year.isNotEmpty() && !title.contains(year)) continue
            val prefix = when {
                catId == "pm_files_year" || catId == "pm_files_latest" -> "📅 File"
                catId.startsWith("pm_topkw_") -> "Trending File"
                else -> "⚡ Direct Telegram File"
            }
            metas.put(JSONObject().apply {
                put("id", "pm_file_$code")
                put("type", "other")
                put("name", title)
                put("poster", f.optString("thumbnail_url", ""))
                put("posterShape", "poster")
                put("description", "$prefix · $pillLine\n\n$title")
                put("genres", JSONArray(genres))
            })
        }
    }

    /** Reverse of the manifest's pm_topkw_<md5[:10]> id mapping. */
    private fun trendingKeywordFor(catId: String): String {
        return try {
            val j = JSONObject(wpGet("$WP_API_BASE/trending?limit=15"))
            val arr = j.optJSONArray("data") ?: j.optJSONArray("items") ?: JSONArray()
            for (i in 0 until arr.length()) {
                val kw = arr.optJSONObject(i)?.optString("keyword", "")?.trim() ?: ""
                if (kw.isEmpty()) continue
                if ("pm_topkw_" + md5Hex(kw.lowercase()).take(10) == catId) return kw
            }
            ""
        } catch (e: Exception) { "" }
    }

    private fun handleStremioMeta(uri: String, session: IHTTPSession): Response {
        val cleanPath = uri.removePrefix("/meta/").removeSuffix(".json")
        val slashIdx = cleanPath.indexOf('/')
        if (slashIdx == -1) {
            return jsonResponse(JSONObject().put("meta", JSONObject()))
        }
        val type = cleanPath.substring(0, slashIdx)
        val id = URLDecoder.decode(cleanPath.substring(slashIdx + 1), "UTF-8")
        DebugLog.log("Meta", "REQUEST type=$type id=$id")

        val meta = JSONObject()
        try {
            if (id.startsWith("pm_file_") || id.startsWith("pm:file_") || id.startsWith("pm:file:")) {
                val shortCode = id.substringAfterLast("file_").substringAfterLast("file:")
                DebugLog.log("Meta", "file meta shortCode=$shortCode")
                val url = "$WP_API_BASE/search-files?search=$shortCode&limit=1"
                val req = Request.Builder().url(url).get().build()
                val resp = httpClient.newCall(req).execute()
                val json = JSONObject(resp.body?.string() ?: "{}")
                val filesArr = json.optJSONArray("files")
                    ?: json.optJSONObject("data")?.optJSONArray("files")
                val f = if (filesArr != null && filesArr.length() > 0) filesArr.getJSONObject(0) else null
                val title = f?.optString("title", "")?.takeIf { it.isNotEmpty() } ?: "Telegram File ($shortCode)"
                val size = f?.optLong("file_size", 0L) ?: 0L
                meta.put("id", id)
                meta.put("type", type)
                meta.put("name", title)
                meta.put("poster", f?.optString("thumbnail_url", "") ?: "")
                meta.put("posterShape", "poster")
                meta.put("description", formatStreamDesc(title, size))
                DebugLog.log("Meta", "file meta built name=${title.take(60)}")
            } else if (id.startsWith("pm:post:") || id.startsWith("pm_post_")) {
                // Post IDs can exceed Int range (e.g. 9000009680), so keep them as a digit string.
                val postId = id.substringAfter("post:").substringAfter("post_").filter { it.isDigit() }.ifEmpty { "0" }
                val gpUrl = "$WP_API_BASE/get-post?post_id=$postId"
                DebugLog.log("Meta", "get-post url=$gpUrl")
                val gp = wpGet(gpUrl, 30)
                DebugLog.log("Meta", "get-post len=${gp.length} head=${gp.take(60)}")
                val json = unwrapWpObject(gp)

                meta.put("id", id)
                meta.put("type", type)
                meta.put("name", cleanPostTitle(json.optString("title", "Title")))
                meta.put("poster", json.optString("thumbnail_url", ""))
                meta.put("posterShape", "poster")
                meta.put("description", cleanPostPlot(json.optString("excerpt", json.optString("content", ""))))
                meta.put("genres", extractPostGenres(json))
                val ry = extractReleaseYear(json.optString("title", ""), json.optString("date", ""))
                if (ry.isNotEmpty()) meta.put("releaseInfo", ry)

                if (type == "series") {
                    val thumb = json.optString("thumbnail_url", "")
                    val fBody = wpGet("$WP_API_BASE/post-files?post_id=$postId&limit=200", 30)
                    val fJson = JSONObject(fBody)
                    val filesArr = fJson.optJSONObject("data")?.optJSONArray("files")
                        ?: fJson.optJSONArray("files")
                    val grouped = LinkedHashMap<String, JSONObject>()
                    var epIndex = 1
                    var hasExplicitEp = false
                    if (filesArr != null) {
                        for (i in 0 until filesArr.length()) {
                            val f = filesArr.getJSONObject(i)
                            val fCode = f.optString("short_code", "")
                            if (fCode.isEmpty()) continue
                            val fTitle = f.optString("title", "Episode $epIndex")
                            val fThumb = f.optString("thumbnail_url", thumb)
                            val fCaption = f.optString("caption", "")
                            val parsed = classifySeasonEpisode(fTitle, f.optInt("season_num", 0), f.optInt("episode_num", 0), fCaption)
                            val s = parsed.first
                            val e = parsed.second
                            if (e > 0) hasExplicitEp = true
                            val key = if (e <= 0) "${s}_0" else "${s}_$e"
                            if (!grouped.containsKey(key)) {
                                val epTitle = if (e > 0) "S${s}E$e" else "Episode $epIndex"
                                grouped[key] = JSONObject().apply {
                                    put("season", s)
                                    put("episode", e)
                                    put("title", epTitle)
                                    put("thumbnail", fThumb)
                                    put("added_date", f.optLong("added_date", 0L))
                                    put("raw_index", epIndex)
                                }
                            }
                            epIndex++
                        }
                    }
                    // Combined packs with no explicit episodes: keep one entry per distinct season.
                    if (!hasExplicitEp && grouped.size > 1) {
                        val newGrouped = LinkedHashMap<String, JSONObject>()
                        var idx = 1
                        for ((_, item) in grouped) {
                            val s = maxOf(1, item.optInt("season", 1))
                            newGrouped["${s}_1"] = JSONObject().apply {
                                put("season", s); put("episode", 1); put("title", "S${s}E1")
                                put("thumbnail", item.optString("thumbnail", thumb))
                                put("added_date", item.optLong("added_date", 0L))
                                put("raw_index", idx)
                            }
                            idx++
                        }
                        grouped.clear()
                        grouped.putAll(newGrouped)
                    }
                    if (hasExplicitEp) {
                        grouped.keys.filter { it.endsWith("_0") }.forEach { grouped.remove(it) }
                    }
                    val videos = JSONArray()
                    val seenIds = HashSet<String>()
                    for ((_, epInfo) in grouped) {
                        val s = maxOf(1, epInfo.optInt("season", 1))
                        val e = if (epInfo.optInt("episode", 0) > 0) epInfo.optInt("episode", 0) else 1
                        val videoId = "pm:post:$postId:$s:$e"
                        if (!seenIds.add(videoId)) continue
                        val epTitle = epInfo.optString("title", "").ifEmpty { "S${s}E$e" }
                        videos.put(JSONObject().apply {
                            put("id", videoId)
                            put("name", epTitle)
                            put("season", s)
                            put("episode", e)
                            put("number", e)
                            put("thumbnail", epInfo.optString("thumbnail", "").ifEmpty { thumb })
                            put("raw_index", epInfo.optInt("raw_index", 0))
                        })
                    }
                    // Natural ascending order: season, episode, raw_index.
                    val list = ArrayList<JSONObject>()
                    for (i in 0 until videos.length()) list.add(videos.getJSONObject(i))
                    list.sortWith(compareBy({ it.optInt("season", 0) }, { it.optInt("episode", 0) }, { it.optInt("raw_index", 0) }))
                    val outVideos = JSONArray()
                    for (v in list) { v.remove("raw_index"); outVideos.put(v) }
                    if (outVideos.length() == 0) {
                        outVideos.put(JSONObject().apply {
                            put("id", "pm:post:$postId:1:1"); put("name", "Episode 1")
                            put("season", 1); put("episode", 1); put("number", 1)
                            put("thumbnail", thumb)
                        })
                    }
                    meta.put("videos", outVideos)
                }
            }
        } catch (e: Exception) {
            DebugLog.log("Meta", "EXCEPTION: ${e.message}")
            Log.e(TAG, "handleStremioMeta error: ${e.message}", e)
        }

        DebugLog.log("Meta", "RESPONSE keys=${meta.keys().asSequence().toList()}")
        return jsonResponse(JSONObject().put("meta", meta))
    }

    /**
     * Zero-disk on-the-fly streaming handler.
     * Decodes payload, resolves TDLib file if needed, reads chunk via TdLibManager in RAM,
     * and streams byte chunk straight to socket with HTTP 206 Partial Content.
     */
    private fun handleDownload(uri: String, session: IHTTPSession): Response {
        val rangeHeader = session.headers["range"] ?: session.headers["Range"]
        val params = session.parameters
        // Round-robin a pooled bot for this whole request (resolve → read) so the
        // file id stays bound to a single TDLib client.
        val mgr = botPool.pickManager()
        DebugLog.log("Download", "BEGIN uri=$uri range=${rangeHeader ?: "none"} bot=${mgr.botId}")

        var payloadB64 = ""
        val parts = uri.split("/")
        val dlIdx = parts.indexOf("download")
        if (dlIdx != -1 && dlIdx + 1 < parts.size) {
            payloadB64 = parts[dlIdx + 1]
        }
        if (payloadB64.isEmpty()) {
            payloadB64 = params["d"]?.firstOrNull() ?: ""
        }

        var shortCode = ""
        var fileIdMt = ""
        var totalSize = 0L
        var targetFileId = params["file_id"]?.firstOrNull()?.toIntOrNull() ?: 0

        if (payloadB64.isNotEmpty()) {
            try {
                val padded = when (payloadB64.length % 4) {
                    2 -> "$payloadB64=="
                    3 -> "$payloadB64="
                    else -> payloadB64
                }
                val decodedBytes = Base64.decode(padded, Base64.URL_SAFE)
                val json = JSONObject(String(decodedBytes, Charsets.UTF_8))
                shortCode = json.optString("short_code", "")
                fileIdMt = json.optString("file_id_mt", "")
                val payloadSize = json.optLong("file_size", 0L)
                if (payloadSize > 0) totalSize = payloadSize
                DebugLog.log("Download", "payload shortCode=$shortCode hasFileIdMt=${fileIdMt.isNotEmpty()} payloadSize=$payloadSize")
            } catch (e: Exception) {
                DebugLog.log("Download", "payload decode FAILED: ${e.message}")
                Log.w(TAG, "Failed to decode payload: ${e.message}")
            }
        } else {
            DebugLog.log("Download", "no payload; raw file_id mode")
        }

        if (targetFileId == 0 && shortCode.isNotEmpty()) {
            targetFileId = shortCodeToFileCache[shortCode] ?: 0
            if (targetFileId != 0 && totalSize <= 0) {
                totalSize = shortCodeToSizeCache[shortCode] ?: 0
            }
            DebugLog.log("Download", "cache lookup shortCode=$shortCode -> fileId=$targetFileId size=$totalSize")
        }

        if (targetFileId == 0) {
            runBlocking {
                if (fileIdMt.isNotEmpty()) {
                    DebugLog.log("Download", "resolve via TDLib getRemoteFile(file_id_mt)")
                    val file = mgr.getRemoteFile(fileIdMt)
                    if (file != null) {
                        targetFileId = file.id
                        if (file.size > 0) totalSize = file.size
                        if (shortCode.isNotEmpty()) shortCodeToFileCache[shortCode] = targetFileId
                        DebugLog.log("Download", "getRemoteFile ok fileId=${file.id} size=${file.size}")
                    } else {
                        DebugLog.log("Download", "getRemoteFile returned NULL")
                    }
                } else if (shortCode.isNotEmpty()) {
                    // Resolve file_id_mt via WordPress with retry passes (matching backend.php)
                    try {
                        var remoteId = ""
                        var resolvedSize = 0L
                        val delays = listOf(0L, 1200L, 1500L, 1500L)
                        for ((retryIdx, delayMs) in delays.withIndex()) {
                            if (delayMs > 0) kotlinx.coroutines.delay(delayMs)
                            val forceParam = if (retryIdx > 0) "&force=1&nocache=1" else ""
                            val resUrl = "$WP_API_BASE/resolve-file?short_code=$shortCode&bot_id=${mgr.botId}$forceParam"
                            try {
                                DebugLog.log("Download", "resolve-file attempt=${retryIdx + 1} secret=${mgr.apiSecret.isNotEmpty()} botId=${mgr.botId}")
                                val reqBuilder = Request.Builder().url(resUrl).get()
                                    .addHeader("Accept", "application/json")
                                if (mgr.apiSecret.isNotEmpty()) {
                                    reqBuilder.addHeader("X-API-Secret", mgr.apiSecret)
                                }
                                val resp = resolveClient.newCall(reqBuilder.build()).execute()
                                val rawBody = resp.body?.string() ?: "{}"
                                DebugLog.log("Download", "resolve-file http=${resp.code} bodyLen=${rawBody.length}")
                                val cleanJson = if (rawBody.contains("{")) rawBody.substring(rawBody.indexOf('{')) else rawBody
                                val resJson = JSONObject(cleanJson)
                                if (resJson.optInt("ok", 0) == 1) {
                                    remoteId = resJson.optString("file_id_mt", resJson.optString("file_id", ""))
                                    resolvedSize = resJson.optLong("file_size", 0L)
                                    DebugLog.log("Download", "resolve-file ok remoteIdLen=${remoteId.length} size=$resolvedSize")
                                    if (remoteId.isNotEmpty()) break
                                } else {
                                    DebugLog.log("Download", "resolve-file NOT ok: ${cleanJson.take(200)}")
                                }
                            } catch (attemptErr: Exception) {
                                DebugLog.log("Download", "resolve-file attempt=${retryIdx + 1} FAILED: ${attemptErr.message}; retrying")
                            }
                        }
                        if (resolvedSize > 0) {
                            totalSize = resolvedSize
                            if (shortCode.isNotEmpty()) shortCodeToSizeCache[shortCode] = resolvedSize
                        }
                        if (remoteId.isNotEmpty()) {
                            val file = mgr.getRemoteFile(remoteId)
                            if (file != null) {
                                targetFileId = file.id
                                if (file.size > 0) totalSize = file.size
                                shortCodeToFileCache[shortCode] = targetFileId
                                DebugLog.log("Download", "resolved shortCode=$shortCode fileId=${file.id} size=${file.size}")
                            } else {
                                DebugLog.log("Download", "getRemoteFile NULL after resolve")
                            }
                        }
                    } catch (e: Exception) {
                        DebugLog.log("Download", "resolve EXCEPTION: ${e.message}")
                        Log.e(TAG, "Failed to resolve shortCode $shortCode: ${e.message}")
                    }
                }
            }
        }

        if (targetFileId == 0) {
            DebugLog.log("Download", "FAILED resolve (shortCode=$shortCode)")
            return newFixedLengthResponse(Response.Status.NOT_FOUND, MIME_PLAINTEXT, "File could not be resolved").apply {
                addCorsHeaders(this)
            }
        }

        // Backfill total size from TDLib when the payload did not carry it.
        // Fixes "Content-Range: bytes X-Y/0" on repeat / cached-file requests.
        if (totalSize <= 0 && shortCode.isNotEmpty()) {
            totalSize = shortCodeToSizeCache[shortCode] ?: 0
        }
        if (totalSize <= 0) {
            val sz = runBlocking { mgr.getFileSize(targetFileId) }
            if (sz > 0) totalSize = sz
            DebugLog.log("Download", "backfilled totalSize=$totalSize fileId=$targetFileId")
        }

        var startOffset = 0L
        var endOffset = if (totalSize > 0) totalSize - 1 else Long.MAX_VALUE

        if (!rangeHeader.isNullOrEmpty() && rangeHeader.startsWith("bytes=")) {
            val rangeVal = rangeHeader.removePrefix("bytes=").trim()
            val rParts = rangeVal.split("-")
            startOffset = rParts[0].toLongOrNull() ?: 0L
            if (rParts.size > 1 && rParts[1].isNotEmpty()) {
                endOffset = rParts[1].toLongOrNull() ?: endOffset
            }
        }
        if (totalSize > 0 && endOffset > totalSize - 1) endOffset = totalSize - 1
        if (endOffset < startOffset) endOffset = startOffset

        DebugLog.log("Download", "stream fileId=$targetFileId start=$startOffset end=$endOffset total=$totalSize")

        // Stream the whole requested byte range straight from TDLib to the socket
        // (MadelineProto downloadToBrowser style) instead of buffering to storage.
        return if (totalSize > 0) {
            val length = endOffset - startOffset + 1
            val status = if (rangeHeader != null) Response.Status.PARTIAL_CONTENT else Response.Status.OK
            DebugLog.log("Download", "RESPONSE ${if (rangeHeader != null) "206" else "200"} len=$length total=$totalSize")
            newFixedLengthResponse(status, "video/mp4", TdLibStream(mgr, targetFileId, startOffset, endOffset), length).apply {
                addHeader("Accept-Ranges", "bytes")
                addHeader("Content-Range", "bytes $startOffset-$endOffset/$totalSize")
                addHeader("Content-Length", length.toString())
                addCorsHeaders(this)
            }
        } else {
            newChunkedResponse(Response.Status.OK, "video/mp4", TdLibStream(mgr, targetFileId, startOffset, Long.MAX_VALUE)).apply {
                addHeader("Accept-Ranges", "bytes")
                addCorsHeaders(this)
            }
        }
    }

    /**
     * Lazily streams a TDLib file's byte range to the HTTP socket in 1 MiB chunks
     * (MadelineProto downloadToBrowser style) — nothing is buffered to storage and
     * the whole file is never materialised. TDLib's own LRU cache handles eviction
     * (see storage_max_files_size in TdLibManager), so we never churn the flash.
     */
    private inner class TdLibStream(
        private val mgr: TdLibManager,
        private val fileId: Int,
        startOffset: Long,
        private val endOffsetInclusive: Long
    ) : InputStream() {
        private var pos: Long = startOffset
        private var buf: ByteArray? = null
        private var bufPos = 0
        private var closed = false

        private fun fill(): Boolean {
            val cur = buf
            if (cur != null && bufPos < cur.size) return true
            if (pos > endOffsetInclusive) return false
            val want = minOf(1024L * 1024L, endOffsetInclusive - pos + 1).toInt()
            val data = runBlocking { mgr.readChunk(fileId, pos, want) }
            if (data == null || data.isEmpty()) return false
            buf = data
            bufPos = 0
            pos += data.size
            return true
        }

        override fun read(): Int {
            if (!fill()) return -1
            return buf!![bufPos++].toInt() and 0xFF
        }

        override fun read(b: ByteArray, off: Int, len: Int): Int {
            if (len == 0) return 0
            if (!fill()) return -1
            val cur = buf!!
            val n = minOf(cur.size - bufPos, len)
            System.arraycopy(cur, bufPos, b, off, n)
            bufPos += n
            return n
        }

        override fun close() {
            closed = true
            try { super.close() } catch (_: Exception) {}
        }
    }

    private fun extractResolution(title: String): String {
        val lower = title.lowercase()
        return when {
            lower.contains("2160p") || lower.contains("4k") || lower.contains("uhd") -> "4K"
            lower.contains("1080p") || lower.contains("fhd") -> "1080p"
            lower.contains("720p") || lower.contains("hd") -> "720p"
            lower.contains("480p") || lower.contains("sd") -> "480p"
            else -> "720p"
        }
    }

    private fun formatStreamDesc(title: String, sizeBytes: Long): String {
        val sizeFormatted = formatBytes(sizeBytes)
        val lower = title.lowercase()
        val pills = ArrayList<String>()
        if (sizeBytes > 0) pills.add("💾 $sizeFormatted")
        pills.add("⚡ Telegram")
        if (lower.contains("bluray") || lower.contains("blu-ray") || lower.contains("remux")) pills.add("BluRay")
        else if (lower.contains("web-dl") || lower.contains("webdl") || lower.contains("webrip")) pills.add("WEB-DL")
        if (lower.contains("hevc") || lower.contains("x265") || lower.contains("h265")) pills.add("HEVC")
        else if (lower.contains("avc") || lower.contains("x264") || lower.contains("h264")) pills.add("H.264")
        return "$title\n" + pills.joinToString(" • ")
    }

    private fun formatBytes(bytes: Long): String {
        if (bytes <= 0) return ""
        val kb = bytes / 1024.0
        val mb = kb / 1024.0
        val gb = mb / 1024.0
        return if (gb >= 1.0) {
            String.format(java.util.Locale.US, "%.1f GB", gb)
        } else {
            String.format(java.util.Locale.US, "%.0f MB", mb)
        }
    }

    /**
     * Serves static assets from assets/web/ (ported from public/).
     */
    private fun serveStaticAsset(uri: String): Response {
        val cleanPath = uri.trimStart('/').let { if (it.isEmpty()) "index.html" else it }
        val assetPath = "web/$cleanPath"

        return try {
            val inputStream = context.assets.open(assetPath)
            val mimeType = getMimeTypeForPath(cleanPath)
            // The HTML shell, stylesheets and scripts must always be revalidated:
            // they carry the inline polyfills / UI fixes and a stale cached copy
            // would pin the WebView to the old build after an app update. Only
            // truly static media (images) is worth caching.
            val revalidate = mimeType.startsWith("text/html") ||
                mimeType.startsWith("text/css") ||
                mimeType.contains("javascript")
            val cacheControl = if (revalidate) {
                "no-cache, no-store, must-revalidate"
            } else {
                "public, max-age=3600"
            }
            newChunkedResponse(Response.Status.OK, mimeType, inputStream).apply {
                addHeader("Cache-Control", cacheControl)
                addCorsHeaders(this)
            }
        } catch (e: Exception) {
            // Fallback: if not found and HTML request, serve index.html (SPA routing)
            try {
                val indexStream = context.assets.open("web/index.html")
                newChunkedResponse(Response.Status.OK, "text/html; charset=utf-8", indexStream).apply {
                    addCorsHeaders(this)
                }
            } catch (ignored: Exception) {
                newFixedLengthResponse(Response.Status.NOT_FOUND, MIME_PLAINTEXT, "Asset not found: $uri").apply {
                    addCorsHeaders(this)
                }
            }
        }
    }

    private fun getMimeTypeForPath(path: String): String {
        return when {
            path.endsWith(".html") -> "text/html; charset=utf-8"
            path.endsWith(".js") -> "application/javascript; charset=utf-8"
            path.endsWith(".css") -> "text/css; charset=utf-8"
            path.endsWith(".png") -> "image/png"
            path.endsWith(".jpg") || path.endsWith(".jpeg") -> "image/jpeg"
            path.endsWith(".svg") -> "image/svg+xml"
            path.endsWith(".json") -> "application/json; charset=utf-8"
            path.endsWith(".ico") -> "image/x-icon"
            path.endsWith(".mp4") -> "video/mp4"
            else -> "application/octet-stream"
        }
    }

    private fun jsonResponse(json: JSONObject, status: Response.IStatus = Response.Status.OK): Response {
        return newFixedLengthResponse(status, "application/json; charset=utf-8", json.toString()).apply {
            addCorsHeaders(this)
        }
    }

    private fun addCorsHeaders(response: Response) {
        response.addHeader("Access-Control-Allow-Origin", "*")
        response.addHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS, HEAD")
        response.addHeader("Access-Control-Allow-Headers", "*")
    }

    private fun getRequestBody(session: IHTTPSession): String {
        val map = HashMap<String, String>()
        session.parseBody(map)
        return map["postData"] ?: ""
    }
}
