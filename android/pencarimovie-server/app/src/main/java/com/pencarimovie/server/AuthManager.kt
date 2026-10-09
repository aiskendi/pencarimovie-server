package com.pencarimovie.server

import android.content.Context
import fi.iki.elonen.NanoHTTPD.IHTTPSession
import java.security.SecureRandom
import javax.crypto.SecretKeyFactory
import javax.crypto.spec.PBEKeySpec

/**
 * Native port of backend.php's server-password auth (fd_auth_*).
 *
 * A single password (default `123456`) gates remote access; localhost / private-LAN
 * requests bypass it. Remote clients present an access token (32 hex) — as a
 * `/<token>/…` or `/t/<token>/…` path segment, `?token=`, `X-Auth-Token`, or the
 * `pm_auth` cookie. Changing the password rotates the token, exactly like
 * fd_auth_set_password().
 */
class AuthManager(private val context: Context) {

    companion object {
        private const val PREFS = "auth_state"
        private const val DEFAULT_PASSWORD = "123456"
        private const val ITERATIONS = 100_000
        private const val KEY_BITS = 256

        // Same shapes as backend.php fd_auth_token_from_request()/fd_strip_token_prefix().
        private val TOKEN_PATH_RE = Regex("^/([0-9a-fA-F]{32,})(/.*)?$")
        private val T_PATH_RE = Regex("^/t/([A-Za-z0-9]+)(/.*)?$")
    }

    private val prefs = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
    private val random = SecureRandom()

    private fun initIfNeeded() {
        if (prefs.getBoolean("init", false)) return
        val salt = randomHex(16)
        prefs.edit()
            .putString("salt", salt)
            .putString("hash", hashPassword(DEFAULT_PASSWORD, salt))
            .putString("token", randomHex(16))
            .putBoolean("enabled", true)
            .putBoolean("init", true)
            .apply()
        DebugLog.log("Auth", "initialised (default password, token ${token().take(6)}…)")
    }

    fun enabled(): Boolean { initIfNeeded(); return prefs.getBoolean("enabled", true) }

    fun token(): String { initIfNeeded(); return prefs.getString("token", "") ?: "" }

    fun rotateToken(): String {
        initIfNeeded()
        val t = randomHex(16)
        prefs.edit().putString("token", t).apply()
        return t
    }

    /** Change the password and rotate the token (mirrors fd_auth_set_password). */
    fun setPassword(password: String): String {
        initIfNeeded()
        val salt = randomHex(16)
        val t = randomHex(16)
        prefs.edit()
            .putString("salt", salt)
            .putString("hash", hashPassword(password, salt))
            .putString("token", t)
            .putBoolean("enabled", true)
            .apply()
        return t
    }

    fun verifyPassword(password: String): Boolean {
        initIfNeeded()
        val salt = prefs.getString("salt", "") ?: ""
        val hash = prefs.getString("hash", "") ?: ""
        if (hash.isEmpty()) return false
        return constantTimeEquals(hash, hashPassword(password, salt))
    }

    /** Token embedded in a `/<token>/…` or `/t/<token>/…` path, if any. */
    fun tokenFromPath(path: String): String {
        TOKEN_PATH_RE.find(path)?.let { return it.groupValues[1].lowercase() }
        T_PATH_RE.find(path)?.let { return it.groupValues[1] }
        return ""
    }

    /** Strip a leading `/<token>` or `/t/<token>` so normal route matching applies. */
    fun stripTokenPrefix(path: String): String {
        TOKEN_PATH_RE.find(path)?.let { return it.groupValues[2].ifEmpty { "/" } }
        T_PATH_RE.find(path)?.let { return it.groupValues[2].ifEmpty { "/" } }
        return path
    }

    fun isValidToken(candidate: String): Boolean {
        if (candidate.isEmpty()) return false
        return constantTimeEquals(token().lowercase(), candidate.lowercase())
    }

    /**
     * Localhost and private-LAN requests bypass auth (mirrors fd_is_local_request).
     *
     * The socket peer is authoritative: the Host header is client-controlled, so a
     * tunnelled request sending `Host: 127.0.0.1` must NOT earn the localhost
     * bypass. Tunnel/public-proxy hostnames, Cloudflare headers and a forwarded
     * public client IP all mark the request remote.
     */
    fun isLocalRequest(session: IHTTPSession): Boolean {
        val headers = session.headers
        val host = (headers["host"] ?: "").lowercase()
        val forwardedHost = (headers["x-forwarded-host"] ?: "").lowercase()
        for (raw in listOf(host, forwardedHost)) {
            for (part in raw.split(",")) {
                val h = part.trim().substringBefore(":")
                if (h.isEmpty()) continue
                if (h == "trycloudflare.com" || h.endsWith(".trycloudflare.com") ||
                    h == "tunnel.pencarimovie.com" || h.endsWith(".tunnel.pencarimovie.com") ||
                    h.endsWith("-tunnel.pencarimovie.com") ||
                    h == "herokuapp.com" || h.endsWith(".herokuapp.com") ||
                    h.endsWith(".up.railway.app") || h.endsWith(".onrender.com") || h.endsWith(".fly.dev")
                ) return false
            }
        }
        if (!headers["cf-connecting-ip"].isNullOrEmpty() ||
            !headers["cf-ray"].isNullOrEmpty() ||
            !headers["cf-visitor"].isNullOrEmpty()
        ) return false

        val clientIp = (headers["x-forwarded-for"] ?: "").split(",").firstOrNull()?.trim() ?: ""
        if (clientIp.isNotEmpty() && !isPrivateOrLoopback(clientIp)) return false

        val peer = try { session.remoteIpAddress } catch (e: Exception) { null }
        if (!peer.isNullOrEmpty()) return isPrivateOrLoopback(peer)

        return isPrivateOrLoopback(host)
    }

    /** Loopback / RFC1918 / link-local / carrier-grade NAT address. */
    private fun isPrivateOrLoopback(rawIp: String): Boolean {
        val ip = rawIp.substringBefore('%').lowercase().removePrefix("::ffff:")
        if (ip.isEmpty()) return false
        if (ip == "::1" || ip == "::") return true
        if (ip.contains(":")) return ip.startsWith("fe80:") || ip.startsWith("fc") || ip.startsWith("fd")
        if (ip == "localhost") return true
        val p = ip.split(".")
        if (p.size != 4) return false
        val a = p[0].toIntOrNull() ?: return false
        val b = p[1].toIntOrNull() ?: return false
        return a == 127 || a == 10 || (a == 172 && b in 16..31) ||
            (a == 192 && b == 168) || (a == 100 && b in 64..127) || (a == 169 && b == 254)
    }

    private fun hashPassword(password: String, saltHex: String): String {
        return try {
            val spec = PBEKeySpec(password.toCharArray(), hexToBytes(saltHex), ITERATIONS, KEY_BITS)
            val factory = SecretKeyFactory.getInstance("PBKDF2WithHmacSHA1")
            bytesToHex(factory.generateSecret(spec).encoded)
        } catch (e: Exception) {
            DebugLog.log("Auth", "hash failed: ${e.message}")
            ""
        }
    }

    private fun randomHex(bytes: Int): String {
        val b = ByteArray(bytes)
        random.nextBytes(b)
        return bytesToHex(b)
    }

    private fun bytesToHex(b: ByteArray): String {
        val sb = StringBuilder(b.size * 2)
        for (x in b) { sb.append(Character.forDigit((x.toInt() shr 4) and 0xF, 16)); sb.append(Character.forDigit(x.toInt() and 0xF, 16)) }
        return sb.toString()
    }

    private fun hexToBytes(hex: String): ByteArray {
        val out = ByteArray(hex.length / 2)
        var i = 0
        while (i + 1 < hex.length) {
            out[i / 2] = ((Character.digit(hex[i], 16) shl 4) + Character.digit(hex[i + 1], 16)).toByte()
            i += 2
        }
        return out
    }

    private fun constantTimeEquals(a: String, b: String): Boolean {
        if (a.length != b.length) return false
        var diff = 0
        for (i in a.indices) diff = diff or (a[i].code xor b[i].code)
        return diff == 0
    }
}
