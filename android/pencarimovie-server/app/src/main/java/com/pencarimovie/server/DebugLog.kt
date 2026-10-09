package com.pencarimovie.server

import android.content.Context
import android.util.Log
import org.json.JSONArray
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale

/**
 * Toggle-able step logger for tracing the streaming / resolve / TDLib download path.
 *
 * - Enabled state persists in SharedPreferences so it survives restarts.
 * - When enabled, every line goes to logcat (tag "PMDBG") AND an in-memory ring buffer.
 * - The buffer is retrievable over HTTP via /api/debug-logs for on-device inspection
 *   without needing a logcat cable.
 *
 * Toggle via:
 *   GET  /api/debug-mode            -> {"ok":1,"enabled":<bool>}
 *   POST /api/debug-mode?enabled=1  -> enable   (also ?enabled=0 to disable, ?toggle=1 to flip)
 *   GET  /api/debug-logs            -> {"ok":1,"enabled":..,"count":..,"logs":[...]}
 *   GET  /api/debug-logs?clear=1    -> fetch then clear
 */
object DebugLog {

    private const val TAG = "PMDBG"
    private const val PREFS = "debug_prefs"
    private const val KEY_ENABLED = "debug_logging_enabled"
    private const val MAX_LINES = 1000

    @Volatile
    private var enabled: Boolean = false

    @Volatile
    private var initialized: Boolean = false

    private val buffer = ArrayDeque<String>()
    private val lock = Any()
    private val timeFmt = ThreadLocal.withInitial {
        SimpleDateFormat("HH:mm:ss.SSS", Locale.US)
    }

    fun init(context: Context) {
        if (initialized) return
        synchronized(lock) {
            if (initialized) return
            enabled = context.getSharedPreferences(PREFS, Context.MODE_PRIVATE)
                .getBoolean(KEY_ENABLED, false)
            initialized = true
        }
    }

    val isEnabled: Boolean get() = enabled

    fun setEnabled(context: Context, value: Boolean) {
        enabled = value
        context.getSharedPreferences(PREFS, Context.MODE_PRIVATE).edit()
            .putBoolean(KEY_ENABLED, value)
            .apply()
        // Force the toggle line into the buffer regardless of prior state.
        record("DebugLog", "Debug logging ${if (value) "ENABLED" else "DISABLED"}")
    }

    fun clear() {
        synchronized(lock) { buffer.clear() }
    }

    fun count(): Int = synchronized(lock) { buffer.size }

    fun dump(): String = synchronized(lock) { buffer.joinToString("\n") }

    fun dumpArray(): JSONArray = synchronized(lock) {
        JSONArray().apply { buffer.forEach { put(it) } }
    }

    fun log(tag: String, message: String) {
        if (!enabled) return
        record(tag, message)
    }

    private fun record(tag: String, message: String) {
        val line = "${timeFmt.get()!!.format(Date())} [$tag] $message"
        Log.i(TAG, line)
        synchronized(lock) {
            buffer.addLast(line)
            while (buffer.size > MAX_LINES) buffer.removeFirst()
        }
    }
}
