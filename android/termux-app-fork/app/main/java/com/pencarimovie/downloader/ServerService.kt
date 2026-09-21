package com.pencarimovie.downloader

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Binder
import android.os.Build
import android.net.wifi.WifiManager
import android.os.IBinder
import android.os.PowerManager
import android.util.Log
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import java.net.Inet4Address
import java.net.NetworkInterface
import kotlin.concurrent.thread

class ServerService : Service() {

    companion object {
        private const val TAG = "ServerService"
        private const val NOTIFICATION_ID = 1001
        private const val CHANNEL_ID = "pencarimovie-server"
        /** Wake lock refresh interval (4 hours). */
        private const val WAKE_LOCK_REFRESH_MS = 4 * 60 * 60 * 1000L

        const val KEY_BATTERY_SAVER = "battery_saver_mode"
        /** Timeout to hold wake lock after active streaming/downloading activity ceases (30s). */
        private const val ACTIVE_LOCK_TIMEOUT_MS = 30_000L

        private const val ACTION_START = "com.pencarimovie.downloader.action.START"
        private const val ACTION_STOP = "com.pencarimovie.downloader.action.STOP"

        fun startIntent(context: Context): Intent =
            Intent(context, ServerService::class.java).setAction(ACTION_START)

        fun stopIntent(context: Context): Intent =
            Intent(context, ServerService::class.java).setAction(ACTION_STOP)
    }

    /** Binder class for activity-service communication. */
    inner class LocalBinder : Binder() {
        fun getService(): ServerService = this@ServerService
    }

    private val binder = LocalBinder()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)

    /** Handler for dispatching state changes to the main thread. */
    private val mainHandler = android.os.Handler(android.os.Looper.getMainLooper())

    /** NativeRunner configured for bootstrap prefix paths (standalone APK). */
    private val nativeRunner = NativeRunner().also { runner ->
        // The onProcessDied callback is set before start() is called, ensuring
        // the process death callback is always available. It will be reassigned
        // inside startServer() to capture the latest coroutine context.
        runner.onProcessDied = null // placeholder; real callback set in startServer()
    }

    private var wakeLock: PowerManager.WakeLock? = null
    private var wifiLock: WifiManager.WifiLock? = null
    private var wakeLockRefreshJob: kotlinx.coroutines.Job? = null
    private var activeLockTimeoutJob: kotlinx.coroutines.Job? = null

    private var isBatterySaverEnabled = true

    /** Atomic flag to prevent double-completion of stop (race with rapid restart). */
    private val stopRequested = java.util.concurrent.atomic.AtomicBoolean(false)

    /**
     * Runtime log line callback — delivers ALL server process output to the UI
     * regardless of server state. Set by MainActivity during onServiceConnected
     * to receive runtime output (download requests, errors, PHP warnings, etc.)
     * that is not tied to state machine transitions.
     *
     * This callback is invoked on the main thread (dispatched via mainHandler
     * from the progress callback), so it can safely modify View elements.
     */
    var onServerLogLine: ((String) -> Unit)? = null

    private val _state = ServerStateObservable(ServerState.Idle)
    val state: ServerStateObservable get() = _state

    override fun onCreate() {
        super.onCreate()
        Log.i(TAG, "Service created")
        createNotificationChannel()
        val prefs = getSharedPreferences(BootReceiver.PREFS_NAME, Context.MODE_PRIVATE)
        isBatterySaverEnabled = prefs.getBoolean(KEY_BATTERY_SAVER, true)
    }

    fun setBatterySaver(enabled: Boolean) {
        isBatterySaverEnabled = enabled
        Log.i(TAG, "Battery saver mode changed to: $enabled")
        if (enabled) {
            stopWakeLockRefresh()
            if (activeLockTimeoutJob == null) {
                releaseWakeLock()
                releaseWifiLock()
            }
        } else {
            if (_state.value is ServerState.Running || _state.value is ServerState.Starting) {
                acquireWakeLock()
                startWakeLockRefresh()
            }
        }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        when (intent?.action) {
            ACTION_START -> {
                _state.value = ServerState.Starting
                startForeground(NOTIFICATION_ID, buildNotification("Starting...", true))
                if (!isBatterySaverEnabled) {
                    acquireWakeLock()
                    startWakeLockRefresh()
                } else {
                    // Temporarily acquire wake lock during bootstrap/startup
                    acquireWakeLock()
                }
                startServer()
            }
            ACTION_STOP -> {
                stopServer()
                // Detach the notification from the foreground service but KEEP it
                // on screen, demoted to a plain swipeable notification that offers
                // a Start action. STOP_FOREGROUND_DETACH (not _REMOVE) is what
                // leaves it behind; the follow-up notify() re-posts it with
                // setOngoing(false) so the user can swipe it away.
                //
                // STOP_FOREGROUND_DETACH is API 24+; on older devices fall back to
                // the deprecated boolean overload, which also detaches.
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N) {
                    stopForeground(STOP_FOREGROUND_DETACH)
                } else {
                    @Suppress("DEPRECATION")
                    stopForeground(false)
                }
                updateNotification("Stopped", false)
                stopWakeLockRefresh()
                activeLockTimeoutJob?.cancel()
                activeLockTimeoutJob = null
                releaseWakeLock()
                releaseWifiLock()
                // stopSelf() is called by the stopServer() coroutine after
                // nativeRunner.stop() completes, ensuring the state transitions
                // properly to Idle before the service is destroyed.
            }
            null -> {
                // System restarted the service after process kill without intent
                Log.d(TAG, "Service restarted with null intent")
            }
        }
        return START_NOT_STICKY
    }

    override fun onDestroy() {
        // Ensure clean stop — nativeRunner.stop() is idempotent
        nativeRunner.stop()
        scope.cancel()
        stopWakeLockRefresh()
        activeLockTimeoutJob?.cancel()
        activeLockTimeoutJob = null
        releaseWakeLock()
        releaseWifiLock()
        Log.i(TAG, "Service destroyed")
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder = binder

    private fun startServer() {
        // Clear any pending stop flag — if the user pressed Stop then immediately
        // Start, the old stop thread's AtomicBoolean check will fail and not
        // interfere with this new start.
        stopRequested.set(false)
        scope.launch(Dispatchers.IO) {
            try {
                // Starting state already emitted by onStartCommand before calling this
                updateNotification("Starting...", true)

                // Clear any previous death callback during startup/fallback phases
                nativeRunner.onProcessDied = null

                // Start native runner with progress callback.
                // The progress callback serves TWO purposes:
                //   1. It drives the state machine during startup (SetupProgress
                //      state), but is guarded to NOT overwrite Running/Error.
                //   2. It forwards EVERY line to the runtime log callback
                //      (onServerLogLine) regardless of state, so the UI log
                //      continues to show useful output like download requests
                //      and errors even while the server is running.
                val lanIp = detectLanIp()
                val port = nativeRunner.start(lanIp = lanIp) { progressLine ->
                    // Inspect log stream for active download/stream/transfers to opportunistically keep CPU & Wi-Fi awake
                    if (isBatterySaverEnabled && isStreamingActivity(progressLine)) {
                        extendActiveLocks()
                    }

                    mainHandler.post {
                        // [1] Forward to runtime log — delivers ALL output to UI
                        // regardless of current state (unfiltered by state guard).
                        onServerLogLine?.invoke(progressLine)

                        // [2] State machine update — guarded to prevent overwriting
                        // Running/Error/Stopping with SetupProgress.
                        val current = _state.value
                        if (current is ServerState.Starting || current is ServerState.SetupProgress) {
                            _state.value = ServerState.SetupProgress(progressLine)
                        }
                    }
                }

                Log.i(TAG, "Server running on port $port, LAN IP: $lanIp")

                // Only arm the unexpected process death callback AFTER start has succeeded
                nativeRunner.onProcessDied = {
                    mainHandler.post {
                        Log.w(TAG, "Server process died unexpectedly — transitioning to Error state")
                        _state.value = ServerState.Error("Server process died unexpectedly")
                        updateNotification("Server crashed", false)
                    }
                }

                // Dispatch Running state to main thread for UI listener safety
                mainHandler.post {
                    _state.value = ServerState.Running(
                        host = "0.0.0.0",
                        port = port,
                        lanIp = lanIp
                    )
                    updateNotification("Running on port $port", true)
                    // If battery saver is active, release the temporary startup wakelock now that server is idle
                    if (isBatterySaverEnabled) {
                        releaseWakeLock()
                    }
                }
            } catch (e: Exception) {
                Log.e(TAG, "Failed to start server", e)
                mainHandler.post {
                    _state.value = ServerState.Error(e.message ?: "Unknown error")
                    updateNotification("Error: ${e.message}", false)
                }
            }
        }
    }

    private fun stopServer() {
        // Run stop on background thread — nativeRunner.stop() now waits for
        // pkill + port release which would block the main thread.
        _state.value = ServerState.Stopping
        stopRequested.set(true)
        Log.d(TAG, "stopServer: state set to Stopping, stopRequested=true, spawning background stop thread")
        // Use a plain Thread instead of coroutine to avoid any scope cancellation
        // issues — the coroutine scope may be cancelled by onDestroy() before the
        // mainHandler.post callback has a chance to execute.
        thread(start = true) {
            Log.d(TAG, "stopServer: background thread started, calling nativeRunner.stop()")
            try {
                nativeRunner.stop()
            } catch (e: Exception) {
                Log.e(TAG, "stopServer: nativeRunner.stop() threw", e)
            }
            Log.d(TAG, "stopServer: nativeRunner.stop() completed, posting Idle state via Handler")
            // Use AtomicBoolean.compareAndSet to atomically claim the "stop complete"
            // action. If this returns true, we're the first and only thread to do so.
            // If false, a restart already cleared the flag (via startServer()).
            mainHandler.post {
                if (stopRequested.compareAndSet(true, false)) {
                    Log.d(TAG, "stopServer: Handler executing — setting state to Idle")
                    _state.value = ServerState.Idle
                    Log.d(TAG, "stopServer: Idle state set, calling stopSelf()")
                    try {
                        stopSelf()
                    } catch (e: Exception) {
                        Log.e(TAG, "stopServer: stopSelf() threw", e)
                    }
                } else {
                    Log.d(TAG, "stopServer: stopRequested was already false (restart?), skipping Idle")
                }
            }
        }
    }

    private fun createNotificationChannel() {
        val channel = NotificationChannel(
            CHANNEL_ID,
            "PencariMovie Server",
            NotificationManager.IMPORTANCE_LOW
        ).apply {
            description = "Notification for PencariMovie Server status"
            setShowBadge(false)
        }
        val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.createNotificationChannel(channel)
    }

    /**
     * Build the status notification.
     *
     * @param content status line shown under the title
     * @param onGoing true while the server is running/starting. An ongoing
     *   notification cannot be swiped away and is required for the foreground
     *   service. When false (server stopped) the notification becomes a plain
     *   swipeable one so the user can dismiss it, and it offers a Start action
     *   instead of Stop.
     */
    private fun buildNotification(content: String, onGoing: Boolean): Notification {
        val openIntent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
        }
        val openPendingIntent = PendingIntent.getActivity(
            this, 0, openIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        // Distinct request codes so the two service PendingIntents never collide.
        val stopPendingIntent = PendingIntent.getService(
            this, 1, stopIntent(this),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        val startPendingIntent = PendingIntent.getService(
            this, 2, startIntent(this),
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )

        val builder = NotificationCompat.Builder(this, CHANNEL_ID)
            .setContentTitle("PencariMovie Server")
            .setContentText(content)
            .setSmallIcon(android.R.drawable.ic_menu_compass)
            .setOngoing(onGoing)
            .setContentIntent(openPendingIntent)

        if (onGoing) {
            builder.addAction(android.R.drawable.ic_media_pause, "Stop", stopPendingIntent)
        } else {
            builder.addAction(android.R.drawable.ic_media_play, "Start", startPendingIntent)
        }
        return builder.build()
    }

    private fun updateNotification(content: String, onGoing: Boolean) {
        val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        manager.notify(NOTIFICATION_ID, buildNotification(content, onGoing))
    }

    private fun acquireWakeLock() {
        val pm = getSystemService(Context.POWER_SERVICE) as PowerManager
        wakeLock = pm.newWakeLock(
            PowerManager.PARTIAL_WAKE_LOCK,
            "pencarimovie:server_wakelock"
        ).apply {
            // Indefinite acquisition — keeps CPU awake while server runs.
            // A periodic refresh job (startWakeLockRefresh) re-acquires every 4h
            // to comply with Android's internal timeout for partial wake locks
            // on some OEM devices.
            acquire()
        }
        Log.i(TAG, "Wake lock acquired (indefinite)")
    }

    /**
     * Periodically refreshes the wake lock every [WAKE_LOCK_REFRESH_MS].
     *
     * Some Android OEMs silently release partial wake locks after a timeout.
     * This coroutine re-acquires the wake lock every 4 hours to prevent the
     * device from sleeping while the server is running.
     */
    private fun startWakeLockRefresh() {
        stopWakeLockRefresh()
        wakeLockRefreshJob = scope.launch {
            while (true) {
                delay(WAKE_LOCK_REFRESH_MS)
                val wl = wakeLock
                if (wl != null && !wl.isHeld) {
                    Log.w(TAG, "Wake lock was released — re-acquiring")
                    wl.acquire()
                }
            }
        }
        Log.i(TAG, "Wake lock refresh started (every ${WAKE_LOCK_REFRESH_MS / 60000} min)")
    }

    private fun stopWakeLockRefresh() {
        wakeLockRefreshJob?.cancel()
        wakeLockRefreshJob = null
    }

    private fun releaseWakeLock() {
        wakeLock?.let {
            if (it.isHeld) it.release()
        }
        wakeLock = null
        Log.i(TAG, "Wake lock released")
    }

    private fun acquireWifiLock() {
        if (wifiLock == null) {
            val wm = applicationContext.getSystemService(Context.WIFI_SERVICE) as? WifiManager
            val mode = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                WifiManager.WIFI_MODE_FULL_HIGH_PERF
            } else {
                WifiManager.WIFI_MODE_FULL
            }
            wifiLock = wm?.createWifiLock(mode, "pencarimovie:streaming_wifilock")?.apply {
                setReferenceCounted(false)
            }
        }
        wifiLock?.let {
            if (!it.isHeld) {
                it.acquire()
                Log.i(TAG, "Wi-Fi high-performance lock acquired")
            }
        }
    }

    private fun releaseWifiLock() {
        wifiLock?.let {
            if (it.isHeld) it.release()
        }
        wifiLock = null
        Log.i(TAG, "Wi-Fi lock released")
    }

    private fun isStreamingActivity(line: String): Boolean {
        val l = line.lowercase()
        return l.contains("download request received") ||
               l.contains("starting downloadtobrowser") ||
               l.contains("stremio stream request") ||
               l.contains("stream_posts") ||
               l.contains("stream_search") ||
               l.contains("resolve-file") ||
               l.contains("proxy-stream") ||
               l.contains("/api/download")
    }

    /**
     * Temporarily holds wake lock and Wi-Fi lock while actively downloading or streaming.
     * Automatically releases locks 30s after the last streaming packet/request.
     */
    private fun extendActiveLocks() {
        acquireWakeLock()
        acquireWifiLock()
        activeLockTimeoutJob?.cancel()
        activeLockTimeoutJob = scope.launch {
            delay(ACTIVE_LOCK_TIMEOUT_MS)
            if (isBatterySaverEnabled) {
                releaseWakeLock()
                releaseWifiLock()
                Log.i(TAG, "Active transfer timeout reached: smart locks released to save battery")
            }
            activeLockTimeoutJob = null
        }
    }

    private fun isPrivateIPv4(ip: String): Boolean {
        val parts = ip.split(".")
        if (parts.size != 4) return false
        val a = parts[0].toIntOrNull() ?: return false
        val b = parts[1].toIntOrNull() ?: return false
        val c = parts[2].toIntOrNull() ?: return false
        val d = parts[3].toIntOrNull() ?: return false
        if (a !in 0..255 || b !in 0..255 || c !in 0..255 || d !in 0..255) return false

        // Exclude loopback (127.0.0.0/8) and link-local (169.254.0.0/16)
        if (a == 127 || (a == 169 && b == 254)) return false

        // 10.0.0.0/8
        if (a == 10) return true

        // 172.16.0.0/12 (172.16.0.0 - 172.31.255.255)
        if (a == 172 && b in 16..31) return true

        // 192.168.0.0/16
        if (a == 192 && b == 168) return true

        return false
    }

    private fun isSkippedInterface(name: String): Boolean {
        val lower = name.lowercase()
        return lower.startsWith("rmnet") ||
               lower.startsWith("ccmni") ||
               lower.startsWith("pdp") ||
               lower.startsWith("wwan") ||
               lower.startsWith("clat") ||
               lower.startsWith("dummy") ||
               lower.startsWith("sit") ||
               lower.startsWith("tun") ||
               lower.startsWith("tap") ||
               lower.startsWith("wg") ||
               lower.startsWith("ppp") ||
               lower.startsWith("ipsec") ||
               lower.startsWith("tailscale")
    }

    private fun getInterfacePriority(name: String): Int {
        val lower = name.lowercase()
        return when {
            // Wi-Fi hotspot / SoftAP (swlan, ap, softap, tether_wlan)
            lower.startsWith("ap") || lower.startsWith("softap") || lower.startsWith("swlan") || lower.startsWith("tether_wlan") -> 100
            // Wi-Fi client / Wi-Fi direct (wlan0, wlan1, etc.)
            lower.startsWith("wlan") -> 90
            // Ethernet / USB tethering (eth, rndis, usb)
            lower.startsWith("eth") || lower.startsWith("rndis") || lower.startsWith("usb") -> 80
            // Virtual gateway / LAN bridge / Bluetooth PAN
            lower.startsWith("vgate") || lower.startsWith("br") || lower.startsWith("bridge") ||
            lower.startsWith("bnep") || lower.startsWith("bt-pan") -> 70
            // Any other interface
            else -> 50
        }
    }

    private fun detectLanIp(): String? {
        // Method 1: ConnectivityManager (Official Android API, works reliably on Android 10+)
        try {
            val cm = getSystemService(Context.CONNECTIVITY_SERVICE) as? android.net.ConnectivityManager
            if (cm != null) {
                for (network in cm.allNetworks) {
                    val linkProperties = cm.getLinkProperties(network) ?: continue
                    val ifaceName = linkProperties.interfaceName ?: ""
                    if (isSkippedInterface(ifaceName)) continue

                    for (linkAddress in linkProperties.linkAddresses) {
                        val address = linkAddress.address
                        if (address is Inet4Address && !address.isLoopbackAddress) {
                            val ip = address.hostAddress ?: continue
                            if (isPrivateIPv4(ip)) {
                                Log.i(TAG, "Detected LAN IP via ConnectivityManager on $ifaceName: $ip")
                                return ip
                            }
                        }
                    }
                }
            }
        } catch (e: Exception) {
            Log.w(TAG, "ConnectivityManager LAN IP detection failed, falling back to NetworkInterface", e)
        }

        // Method 2: NetworkInterface traversal (Fallback & Hotspot/AP interfaces)
        try {
            val interfaces = NetworkInterface.getNetworkInterfaces() ?: return null
            var bestIp: String? = null
            var bestPriority = -1

            while (interfaces.hasMoreElements()) {
                val networkInterface = interfaces.nextElement()
                if (networkInterface.isLoopback) continue
                val name = networkInterface.name

                // Skip cellular WAN / VPN interfaces (rmnet, tun, wg, ...)
                if (isSkippedInterface(name)) continue

                val priority = getInterfacePriority(name)
                val addresses = networkInterface.inetAddresses
                while (addresses.hasMoreElements()) {
                    val address = addresses.nextElement()
                    if (address is Inet4Address && !address.isLoopbackAddress) {
                        val ip = address.hostAddress ?: continue
                        if (isPrivateIPv4(ip)) {
                            if (priority > bestPriority) {
                                bestPriority = priority
                                bestIp = ip
                            }
                        }
                    }
                }
            }
            if (bestIp != null) {
                Log.i(TAG, "Detected LAN IP via NetworkInterface: $bestIp")
                return bestIp
            }
        } catch (e: Exception) {
            Log.w(TAG, "Failed to detect LAN IP via NetworkInterface", e)
        }

        // Method 3: Android system properties (Hotspot IP fallback)
        val hotspotProps = listOf("dhcp.wlan0.ipaddress", "dhcp.wlan1.ipaddress", "dhcp.ap0.ipaddress", "dhcp.softap0.ipaddress")
        for (prop in hotspotProps) {
            try {
                val p = Runtime.getRuntime().exec(arrayOf("getprop", prop))
                val ip = p.inputStream.bufferedReader().readLine()?.trim().orEmpty()
                if (isPrivateIPv4(ip)) {
                    Log.i(TAG, "Detected LAN IP via getprop ($prop): $ip")
                    return ip
                }
            } catch (_: Exception) {}
        }

        return null
    }
}

class ServerStateObservable(initial: ServerState) {
    @Volatile
    var value: ServerState = initial
        set(newValue) {
            field = newValue
            listeners.forEach { it(newValue) }
        }

    private val listeners = mutableListOf<(ServerState) -> Unit>()

    fun observe(listener: (ServerState) -> Unit) {
        listeners.add(listener)
        listener(value)
    }

    fun removeObserver(listener: (ServerState) -> Unit) {
        listeners.remove(listener)
    }
}
