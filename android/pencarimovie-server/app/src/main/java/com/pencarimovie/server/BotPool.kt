package com.pencarimovie.server

import android.content.Context
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.util.concurrent.CopyOnWriteArrayList
import java.util.concurrent.atomic.AtomicInteger

/**
 * Multi-bot pool. Each pooled bot is a namespaced [TdLibManager] with its own
 * token/session/database, so several bots stay logged in at once. Mirrors
 * backend.php's fd_get_bot_pool() and the /api/bots, /api/bots/add,
 * /api/bots/remove, /api/bots/set-active shapes. Downloads and streams
 * round-robin across the ready bots.
 */
class BotPool(private val context: Context) {

    class PoolBot(val slot: String, val token: String, val mgr: TdLibManager) {
        var botId: String = ""
        var username: String = ""
        var name: String = ""
        var apiSecret: String = ""
    }

    private val prefs = context.getSharedPreferences("bot_pool", Context.MODE_PRIVATE)
    private val extras = CopyOnWriteArrayList<PoolBot>()
    private val rr = AtomicInteger(0)
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    /** Always-present primary manager (guest auto-provision + fallback). */
    val primary: TdLibManager = TdLibManager(context, "primary")

    @Volatile private var activeId: String = ""

    fun initialize() {
        // Guest leasing is for an EMPTY pool only. Pooled bots restore their own
        // sessions a few seconds after boot, so a "no live session yet" check races
        // them: the primary reaches WaitPhoneNumber first and leases a guest even
        // though a pool is saved — a wasted lease plus a bot the user must remove.
        // Read the PERSISTED pool, not `extras`, so the answer cannot depend on
        // loadExtras() having run or on how fast each bot authenticates.
        primary.shouldAutoProvision = { !poolConfigured() }
        primary.initialize()
        loadExtras()
    }

    /** True when the persisted pool holds at least one configured bot token. */
    private fun poolConfigured(): Boolean {
        val raw = prefs.getString("bots", null) ?: return false
        return try {
            JSONArray(raw).length() > 0
        } catch (e: Exception) {
            false
        }
    }

    private fun loadExtras() {
        val raw = prefs.getString("bots", null) ?: return
        try {
            val arr = JSONArray(raw)
            for (i in 0 until arr.length()) {
                val o = arr.optJSONObject(i) ?: continue
                val slot = o.optString("slot")
                val token = o.optString("token")
                if (slot.isEmpty() || token.isEmpty()) continue
                if (extras.any { it.slot == slot }) continue
                val mgr = TdLibManager(context, slot)
                // Refresh this bot's PencariMovie credentials before its client is
                // created: a pooled bot without the api_secret gets 401 from the
                // WordPress /resolve-file bridge, which broke downloads for every
                // bot already in the pool before this fix (see prefetchCredentials).
                scope.launch {
                    mgr.prefetchCredentials(token)
                    mgr.initialize()
                }
                extras.add(PoolBot(slot, token, mgr))
            }
            DebugLog.log("BotPool", "loaded ${extras.size} pooled bot(s)")
        } catch (e: Exception) {
            DebugLog.log("BotPool", "load failed: ${e.message}")
        }
    }

    private fun save() {
        val arr = JSONArray()
        for (b in extras) arr.put(JSONObject().put("slot", b.slot).put("token", b.token))
        prefs.edit().putString("bots", arr.toString()).apply()
    }

    private fun refresh(b: PoolBot) {
        b.botId = b.mgr.botId
        b.username = b.mgr.botUsername
        b.name = b.mgr.botName
        b.apiSecret = b.mgr.apiSecret
    }

    private fun readyManagers(): List<TdLibManager> {
        val out = ArrayList<TdLibManager>()
        if (primary.hasSession) out.add(primary)
        for (b in extras) { refresh(b); if (b.mgr.hasSession) out.add(b.mgr) }
        return out
    }

    fun activeManager(): TdLibManager {
        val id = activeId
        if (id.isNotEmpty()) {
            for (b in extras) { refresh(b); if (b.botId == id) return b.mgr }
            if (primary.botId == id) return primary
        }
        // Active id empty or stale (its bot was just removed) — fall back to any
        // ready bot so requests don't land on a session-less primary.
        return readyManagers().firstOrNull() ?: primary
    }

    /** Round-robin a ready bot for one download/stream request. */
    fun pickManager(): TdLibManager {
        val ready = readyManagers()
        if (ready.isEmpty()) return activeManager()
        val i = (rr.getAndIncrement() and 0x7fffffff) % ready.size
        return ready[i]
    }

    fun activeId(): String {
        if (activeId.isNotEmpty()) return activeId
        if (primary.hasSession) return primary.botId
        for (b in extras) { refresh(b); if (b.mgr.hasSession || b.botId.isNotEmpty()) return b.botId }
        return ""
    }

    fun count(): Int = readyManagers().size

    fun list(): JSONArray {
        val arr = JSONArray()
        val active = activeId()
        if (primary.hasSession) {
            arr.put(botJson(primary.botId, primary.botUsername, primary.botName, active.isEmpty() || primary.botId == active))
        }
        for (b in extras) {
            refresh(b)
            arr.put(botJson(b.botId, b.username, b.name, b.botId == active))
        }
        return arr
    }

    private fun botJson(id: String, username: String, name: String, isActive: Boolean): JSONObject =
        JSONObject().apply {
            put("bot_id", id)
            put("bot_username", username)
            put("bot_name", name)
            put("is_active", isActive)
            put("session_exists", id.isNotEmpty())
        }

    /** Per-token result row, mirroring backend.php's /api/bots/add results[]. */
    class AddResult(
        val ok: Boolean,
        val token: String,
        val botId: String = "",
        val username: String = "",
        val name: String = "",
        val error: String = ""
    )

    /**
     * Add a bot token. On success the bot joins the pool; if nothing is active yet the
     * first successful bot becomes the active bot (matches backend.php's behaviour of
     * calling fd_save_session_meta() when fd_get_bot_id() is empty).
     */
    suspend fun add(rawToken: String): AddResult {
        val token = rawToken.trim()
        if (token.isEmpty() || !token.contains(":")) return AddResult(false, token, error = "Invalid bot token.")
        if (token == primary.botToken || extras.any { it.token == token }) {
            return AddResult(false, token, error = "Bot is already in the pool.")
        }

        val slot = "b" + Integer.toHexString(token.hashCode()).take(8) + System.currentTimeMillis().toString(16).takeLast(4)
        // Seed the namespaced prefs so the manager picks the token up on boot.
        context.getSharedPreferences("tdlib_session_$slot", Context.MODE_PRIVATE).edit()
            .putString("bot_token", token).apply()

        val mgr = TdLibManager(context, slot)
        // Every bot is booted with the credentials PencariMovie issued for ITS token
        // (api_id + api_secret), exactly like backend.php fd_boot_madeline(). Skipping
        // this left pooled bots on the fallback api id with no secret, so downloads
        // died the moment the guest/primary bot left the pool.
        mgr.prefetchCredentials(token)
        mgr.initialize()
        val ready = withTimeoutOrNull(30_000L) { mgr.isReady.first { it } } ?: false
        if (!ready) {
            try { mgr.close() } catch (e: Exception) {}
            context.getSharedPreferences("tdlib_session_$slot", Context.MODE_PRIVATE).edit().clear().apply()
            File(context.filesDir, "tdlib/$slot").deleteRecursively()
            return AddResult(
                false, token,
                error = mgr.lastError.ifEmpty { "Bot did not connect (invalid token or no network)." }
            )
        }
        // fetchMe() runs right after Ready; wait briefly for the bot id.
        var waited = 0
        while (mgr.botId.isEmpty() && waited < 5_000) { delay(150); waited += 150 }

        val bot = PoolBot(slot, token, mgr)
        refresh(bot)
        extras.add(bot)
        save()
        // First bot added while nothing is active becomes the active bot.
        if (activeId().isEmpty() && bot.botId.isNotEmpty()) setActive(bot.botId)
        val label = if (bot.username.isNotEmpty()) "@${bot.username}" else bot.botId.ifEmpty { "bot" }
        DebugLog.log("BotPool", "added $label (slot=$slot)")
        return AddResult(true, token, bot.botId, bot.username, bot.name)
    }

    fun remove(botId: String): Boolean {
        if (botId.isEmpty()) return false

        // Pooled (explicitly added) bot.
        val b = extras.firstOrNull { refresh(it); it.botId == botId }
        if (b != null) {
            try { b.mgr.close() } catch (e: Exception) {}
            File(context.filesDir, "tdlib/${b.slot}").deleteRecursively()
            context.getSharedPreferences("tdlib_session_${b.slot}", Context.MODE_PRIVATE).edit().clear().apply()
            extras.remove(b)
            if (activeId == botId) activeId = ""
            save()
            DebugLog.log("BotPool", "removed $botId")
            return true
        }

        // The single/primary (guest) bot: clear its session so it leaves the pool.
        // Previously only `extras` was checked, so the Remove button did nothing
        // for the built-in single bot.
        if (primary.botId.isNotEmpty() && primary.botId == botId) {
            try { primary.close() } catch (e: Exception) {}
            try { File(context.filesDir, "tdlib/primary").deleteRecursively() } catch (e: Exception) {}
            context.getSharedPreferences("tdlib_session_primary", Context.MODE_PRIVATE).edit().clear().apply()
            context.getSharedPreferences("tdlib_session", Context.MODE_PRIVATE).edit().clear().apply()
            if (activeId == botId) activeId = ""
            save()
            DebugLog.log("BotPool", "removed primary bot $botId")
            return true
        }

        return false
    }

    fun setActive(botId: String): Boolean {
        if (botId.isEmpty()) return false
        for (b in extras) { refresh(b); if (b.botId == botId) { activeId = botId; return true } }
        if (primary.botId == botId) { activeId = botId; return true }
        return false
    }

    /** Return the bot id for an already-pooled token, or "" when unknown. */
    fun findIdByToken(token: String): String {
        val t = token.trim()
        if (t.isEmpty()) return ""
        if (primary.botToken == t) return primary.botId
        for (b in extras) { refresh(b); if (b.token == t) return b.botId }
        return ""
    }

    /**
     * Clear EVERY bot session locally — primary + all pooled bots — and empty the
     * pool. Mirrors backend.php fd_clear_session() called by /api/botlogout:
     * sessions are torn down locally (Close(), never a server-side LogOut) so leased
     * guest bot keys are not revoked and do not hit AUTH_KEY_UNREGISTERED.
     */
    fun clearAll() {
        for (b in extras) {
            try { b.mgr.close() } catch (e: Exception) {}
            try { File(context.filesDir, "tdlib/${b.slot}").deleteRecursively() } catch (e: Exception) {}
            context.getSharedPreferences("tdlib_session_${b.slot}", Context.MODE_PRIVATE).edit().clear().apply()
        }
        extras.clear()
        try { primary.close() } catch (e: Exception) {}
        try { File(context.filesDir, "tdlib/primary").deleteRecursively() } catch (e: Exception) {}
        context.getSharedPreferences("tdlib_session_primary", Context.MODE_PRIVATE).edit().clear().apply()
        // Legacy single-session prefs (older builds) too.
        context.getSharedPreferences("tdlib_session", Context.MODE_PRIVATE).edit().clear().apply()
        activeId = ""
        prefs.edit().remove("bots").apply()
        DebugLog.log("BotPool", "cleared all sessions (primary + ${extras.size} pooled)")
    }

    fun close() {
        for (b in extras) try { b.mgr.close() } catch (e: Exception) {}
        try { primary.close() } catch (e: Exception) {}
    }
}
