package com.pencarimovie.downloader

import android.Manifest
import android.content.ClipData
import android.content.ClipboardManager
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.content.pm.PackageManager
import android.content.res.ColorStateList
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.os.IBinder
import android.os.PowerManager
import android.provider.Settings
import android.util.Log
import android.view.View
import android.widget.LinearLayout
import android.widget.ProgressBar
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import androidx.lifecycle.lifecycleScope
import com.google.android.material.button.MaterialButton
import com.google.android.material.dialog.MaterialAlertDialogBuilder
import com.google.android.material.switchmaterial.SwitchMaterial
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.json.JSONObject
import java.io.File
import java.net.HttpURLConnection
import java.net.URL

class MainActivity : AppCompatActivity() {

    companion object {
        private const val TAG = "MainActivity"
        private const val KEY_BACKGROUND_GUIDE_SHOWN = "background_guide_shown"
    }

    private var serverService: ServerService? = null
    private var isBound = false

    private lateinit var viewStatusDot: View
    private lateinit var tvStatus: TextView
    private lateinit var progressBarStatus: ProgressBar
    private lateinit var tvStatusLan: TextView
    private lateinit var tvStatusTunnel: TextView

    private lateinit var btnStartStop: MaterialButton
    private lateinit var btnOpenWeb: MaterialButton
    private lateinit var btnAddon: MaterialButton
    private lateinit var btnSettings: MaterialButton

    private lateinit var switchTunnel: SwitchMaterial
    private lateinit var tvTunnelStatus: TextView

    private lateinit var switchAutoStart: SwitchMaterial
    private lateinit var switchBatterySaver: SwitchMaterial
    private lateinit var btnBackgroundGuide: MaterialButton

    private lateinit var updateBanner: LinearLayout
    private lateinit var tvUpdateText: TextView
    private lateinit var btnUpdateDownload: MaterialButton
    private lateinit var updateProgress: ProgressBar

    private var currentState: ServerState = ServerState.Idle
    private var serverPort: Int = 8088
    private var lanIp: String? = null
    private var currentTunnelUrl: String? = null

    private var stateObserver: ((ServerState) -> Unit)? = null
    private var isDownloadingUpdate = false
    private var tunnelPollJob: Job? = null
    private var isTunnelToggleInProgress = false

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
            serverService = (service as? ServerService.LocalBinder)?.getService()
            val svc = serverService
            stateObserver?.let { svc?.state?.removeObserver(it) }
            stateObserver = { state ->
                runOnUiThread { onStateChanged(state) }
            }
            svc?.state?.observe(stateObserver!!)

            // Sync battery saver state with the service
            val prefs = getSharedPreferences(BootReceiver.PREFS_NAME, Context.MODE_PRIVATE)
            svc?.setBatterySaver(prefs.getBoolean(ServerService.KEY_BATTERY_SAVER, true))
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            stateObserver?.let { serverService?.state?.removeObserver(it) }
            stateObserver = null
            serverService = null
            isBound = false
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        viewStatusDot = findViewById(R.id.viewStatusDot)
        tvStatus = findViewById(R.id.tvStatus)
        progressBarStatus = findViewById(R.id.progressBarStatus)
        tvStatusLan = findViewById(R.id.tvStatusLan)
        tvStatusTunnel = findViewById(R.id.tvStatusTunnel)

        btnStartStop = findViewById(R.id.btnStartStop)
        btnOpenWeb = findViewById(R.id.btnOpenWeb)
        btnAddon = findViewById(R.id.btnAddon)
        btnSettings = findViewById(R.id.btnSettings)

        switchTunnel = findViewById(R.id.switchTunnel)
        tvTunnelStatus = findViewById(R.id.tvTunnelStatus)

        switchAutoStart = findViewById(R.id.switchAutoStart)
        switchBatterySaver = findViewById(R.id.switchBatterySaver)
        btnBackgroundGuide = findViewById(R.id.btnBackgroundGuide)

        updateBanner = findViewById(R.id.updateBanner)
        tvUpdateText = findViewById(R.id.tvUpdateText)
        btnUpdateDownload = findViewById(R.id.btnUpdateDownload)
        updateProgress = findViewById(R.id.updateProgress)

        btnBackgroundGuide.setOnClickListener {
            checkAndPromptBatteryOptimization(forcePrompt = true)
        }

        val prefs = getSharedPreferences(BootReceiver.PREFS_NAME, Context.MODE_PRIVATE)
        switchAutoStart.isChecked = prefs.getBoolean(BootReceiver.KEY_AUTO_START, false)
        switchAutoStart.setOnCheckedChangeListener { _, isChecked ->
            prefs.edit().putBoolean(BootReceiver.KEY_AUTO_START, isChecked).apply()
        }

        switchBatterySaver.isChecked = prefs.getBoolean(ServerService.KEY_BATTERY_SAVER, true)
        switchBatterySaver.setOnCheckedChangeListener { _, isChecked ->
            prefs.edit().putBoolean(ServerService.KEY_BATTERY_SAVER, isChecked).apply()
            serverService?.setBatterySaver(isChecked)
        }

        setupListeners()
        updateUiForState(ServerState.Idle)

        checkForAppUpdate()

        // Disable Start button until bootstrap check completes
        btnStartStop.isEnabled = false

        // Check and extract bootstrap on first launch
        val prefixDir = File(TermuxInstaller.PREFIX_DIR_PATH)
        val isFirstLaunch = !prefixDir.isDirectory || (prefixDir.list()?.isEmpty() == true)

        TermuxInstaller.setupBootstrapIfNeeded(this, Runnable {
            runOnUiThread {
                btnStartStop.isEnabled = true
                if (isFirstLaunch) {
                    checkAndPromptBatteryOptimization()
                }
            }
        })
    }

    override fun onStart() {
        super.onStart()
        bindService()
    }

    override fun onResume() {
        super.onResume()
        if (currentState is ServerState.Running) {
            fetchTunnelStatus()
            startTunnelPolling()
        }
    }

    override fun onStop() {
        super.onStop()
        stopTunnelPolling()
        if (isBound) {
            unbindService(serviceConnection)
            isBound = false
        }
    }

    private fun bindService() {
        val intent = Intent(this, ServerService::class.java)
        bindService(intent, serviceConnection, Context.BIND_AUTO_CREATE)
        isBound = true
    }

    private fun setupListeners() {
        btnStartStop.setOnClickListener {
            when (currentState) {
                is ServerState.Idle, is ServerState.Error -> {
                    requestNotificationPermissionAndStart()
                }
                is ServerState.Running -> {
                    stopServer()
                }
                else -> { /* ignore */ }
            }
        }

        btnOpenWeb.setOnClickListener {
            WebActivity.start(this, "http://localhost:$serverPort/", getString(R.string.app_name))
        }

        btnAddon.setOnClickListener {
            WebActivity.start(this, "http://localhost:$serverPort/#addon", getString(R.string.addon_setup))
        }

        btnSettings.setOnClickListener {
            WebActivity.start(this, "http://localhost:$serverPort/#settings", getString(R.string.server_settings))
        }

        tvStatusLan.setOnClickListener {
            val lanUrl = lanIp?.let { ip -> "http://$ip:$serverPort" }
            if (!lanUrl.isNullOrEmpty()) {
                copyToClipboard(lanUrl, "LAN URL copied to clipboard")
            }
        }

        tvStatusTunnel.setOnClickListener {
            val tunnel = currentTunnelUrl
            if (!tunnel.isNullOrEmpty()) {
                copyToClipboard(tunnel, "Tunnel URL copied to clipboard")
            }
        }

        switchTunnel.setOnClickListener {
            val isChecked = switchTunnel.isChecked
            toggleTunnel(isChecked)
        }
    }

    private fun copyToClipboard(text: String, message: String) {
        val clipboard = getSystemService(Context.CLIPBOARD_SERVICE) as? ClipboardManager
        val clip = ClipData.newPlainText("PencariMovie Server URL", text)
        clipboard?.setPrimaryClip(clip)
        Toast.makeText(this, message, Toast.LENGTH_SHORT).show()
    }

    private fun toggleTunnel(enable: Boolean) {
        if (currentState !is ServerState.Running) return
        isTunnelToggleInProgress = true
        switchTunnel.isEnabled = false
        tvTunnelStatus.text = if (enable) getString(R.string.tunnel_starting) else "Disconnecting tunnel…"

        // Fast poll loop while connecting
        val fastPollJob = if (enable) {
            lifecycleScope.launch(Dispatchers.IO) {
                while (isActive && isTunnelToggleInProgress) {
                    delay(2000)
                    fetchTunnelStatusInternal()
                }
            }
        } else null

        lifecycleScope.launch(Dispatchers.IO) {
            try {
                val endpoint = if (enable) "enable" else "disable"
                val url = URL("http://127.0.0.1:$serverPort/api/tunnel/$endpoint")
                val conn = (url.openConnection() as HttpURLConnection).apply {
                    requestMethod = "POST"
                    connectTimeout = 10_000
                    readTimeout = 180_000 // Cloudflared download & setup can take up to 90s
                    doOutput = true
                    setRequestProperty("Content-Type", "application/json")
                    setRequestProperty("Accept", "application/json")
                }
                val payload = if (enable) "{\"tunnel_token\":\"\"}" else "{}"
                conn.outputStream.use { it.write(payload.toByteArray()) }

                val responseCode = conn.responseCode
                val stream = if (responseCode in 200..299) conn.inputStream else conn.errorStream
                val responseBody = stream?.bufferedReader()?.use { it.readText() }.orEmpty()
                conn.disconnect()

                var parsedError: String? = null
                if (responseBody.isNotEmpty()) {
                    try {
                        val json = JSONObject(responseBody)
                        if (json.optInt("ok", 1) == 0) {
                            parsedError = json.optString("message", "Failed to start tunnel")
                        }
                    } catch (_: Exception) {}
                }

                withContext(Dispatchers.Main) {
                    fastPollJob?.cancel()
                    isTunnelToggleInProgress = false
                    if (parsedError != null) {
                        tvTunnelStatus.text = parsedError
                        switchTunnel.isChecked = false
                        switchTunnel.isEnabled = true
                    } else {
                        fetchTunnelStatus()
                    }
                }
            } catch (e: Exception) {
                Log.w(TAG, "Failed to toggle tunnel: ${e.message}")
                withContext(Dispatchers.Main) {
                    fastPollJob?.cancel()
                    isTunnelToggleInProgress = false
                    tvTunnelStatus.text = "Tunnel error: ${e.message}"
                    switchTunnel.isEnabled = true
                    fetchTunnelStatus()
                }
            }
        }
    }

    private fun startTunnelPolling() {
        stopTunnelPolling()
        tunnelPollJob = lifecycleScope.launch(Dispatchers.IO) {
            while (isActive && currentState is ServerState.Running) {
                fetchTunnelStatusInternal()
                delay(6000)
            }
        }
    }

    private fun stopTunnelPolling() {
        tunnelPollJob?.cancel()
        tunnelPollJob = null
    }

    private fun fetchTunnelStatus() {
        lifecycleScope.launch(Dispatchers.IO) {
            fetchTunnelStatusInternal()
        }
    }

    private suspend fun fetchTunnelStatusInternal() {
        if (currentState !is ServerState.Running) return
        try {
            val url = URL("http://127.0.0.1:$serverPort/api/tunnel/status")
            val conn = url.openConnection() as HttpURLConnection
            conn.requestMethod = "GET"
            conn.connectTimeout = 3000
            conn.readTimeout = 4000

            if (conn.responseCode == 200) {
                val body = conn.inputStream.bufferedReader().use { it.readText() }
                val json = JSONObject(body)
                val enabled = json.optBoolean("enabled", false)
                val running = json.optBoolean("running", false)
                val publicUrl = json.optString("public_url", "").ifEmpty {
                    json.optString("public_domain_url", "").ifEmpty {
                        json.optString("tunnel_url", "")
                    }
                }

                val isTunnelActive = enabled || running || publicUrl.isNotEmpty()

                withContext(Dispatchers.Main) {
                    if (!isTunnelToggleInProgress) {
                        switchTunnel.isChecked = isTunnelActive
                        switchTunnel.isEnabled = (currentState is ServerState.Running)
                    }

                    if (isTunnelActive && publicUrl.isNotEmpty()) {
                        currentTunnelUrl = publicUrl
                        tvTunnelStatus.text = "Connected: $publicUrl"
                        tvStatusTunnel.text = "Tunnel: $publicUrl (Tap to copy)"
                        tvStatusTunnel.visibility = View.VISIBLE
                    } else if (isTunnelActive) {
                        currentTunnelUrl = null
                        tvTunnelStatus.text = getString(R.string.tunnel_starting)
                        tvStatusTunnel.visibility = View.GONE
                    } else {
                        currentTunnelUrl = null
                        tvTunnelStatus.text = getString(R.string.tunnel_off)
                        tvStatusTunnel.visibility = View.GONE
                    }
                }
            }
            conn.disconnect()
        } catch (_: Exception) {
            withContext(Dispatchers.Main) {
                if (!isTunnelToggleInProgress) {
                    switchTunnel.isEnabled = (currentState is ServerState.Running)
                }
            }
        }
    }

    private fun checkForAppUpdate() {
        lifecycleScope.launch {
            val info = UpdateChecker.checkForUpdate(BuildConfig.VERSION_CODE) ?: return@launch

            tvUpdateText.text = getString(
                R.string.update_available,
                info.versionName,
                BuildConfig.VERSION_NAME
            )
            updateBanner.visibility = View.VISIBLE

            btnUpdateDownload.setOnClickListener { downloadAndInstallUpdate(info) }
        }
    }

    private fun downloadAndInstallUpdate(info: UpdateChecker.UpdateInfo) {
        if (isDownloadingUpdate) return
        isDownloadingUpdate = true
        btnUpdateDownload.isEnabled = false

        updateProgress.visibility = View.VISIBLE
        updateProgress.isIndeterminate = true
        updateProgress.progress = 0
        btnUpdateDownload.text = getString(R.string.update_downloading)

        lifecycleScope.launch {
            try {
                val apk = UpdateChecker.downloadApk(this@MainActivity, info) { done, total ->
                    runOnUiThread {
                        if (total > 0) {
                            val pct = (done * 100 / total).toInt()
                            updateProgress.isIndeterminate = false
                            updateProgress.progress = pct
                            btnUpdateDownload.text =
                                getString(R.string.update_downloading_pct, pct)
                        } else {
                            updateProgress.isIndeterminate = true
                            btnUpdateDownload.text = getString(R.string.update_downloading)
                        }
                    }
                }

                if (apk == null) {
                    Toast.makeText(
                        this@MainActivity,
                        R.string.update_download_failed,
                        Toast.LENGTH_LONG
                    ).show()
                    openReleasePage()
                    return@launch
                }

                updateProgress.isIndeterminate = true
                btnUpdateDownload.text = getString(R.string.update_installing)

                if (!UpdateChecker.installApk(this@MainActivity, apk)) {
                    Toast.makeText(
                        this@MainActivity,
                        R.string.update_install_failed,
                        Toast.LENGTH_LONG
                    ).show()
                    openReleasePage()
                }
            } finally {
                isDownloadingUpdate = false
                btnUpdateDownload.isEnabled = true
                btnUpdateDownload.text = getString(R.string.update_download)
                updateProgress.visibility = View.GONE
                updateProgress.isIndeterminate = false
            }
        }
    }

    private fun openReleasePage() {
        try {
            startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(UpdateChecker.RELEASE_PAGE_URL)))
        } catch (_: Exception) {
            Toast.makeText(this, UpdateChecker.RELEASE_PAGE_URL, Toast.LENGTH_LONG).show()
        }
    }

    private fun onStateChanged(state: ServerState) {
        currentState = state
        when (state) {
            is ServerState.Running -> {
                serverPort = state.port
                lanIp = state.lanIp
            }
            else -> {}
        }
        updateUiForState(state)
    }

    private fun updateUiForState(state: ServerState) {
        when (state) {
            is ServerState.Idle -> {
                stopTunnelPolling()
                btnStartStop.text = getString(R.string.start_server)
                btnStartStop.isEnabled = true
                btnOpenWeb.isEnabled = false
                btnAddon.isEnabled = false
                btnSettings.isEnabled = false
                switchTunnel.isEnabled = false
                switchTunnel.isChecked = false
                tvTunnelStatus.text = getString(R.string.tunnel_off)

                viewStatusDot.backgroundTintList = ColorStateList.valueOf(getColor(R.color.glass_text_dim))
                tvStatus.text = getString(R.string.status_idle)
                tvStatus.setTextColor(getColor(R.color.glass_text_dim))
                progressBarStatus.visibility = View.GONE
                tvStatusLan.visibility = View.GONE
                tvStatusTunnel.visibility = View.GONE
            }
            is ServerState.Starting -> {
                btnStartStop.text = "Starting..."
                btnStartStop.isEnabled = false
                btnOpenWeb.isEnabled = false
                btnAddon.isEnabled = false
                btnSettings.isEnabled = false
                switchTunnel.isEnabled = false

                viewStatusDot.backgroundTintList = ColorStateList.valueOf(getColor(R.color.glass_warn))
                tvStatus.text = getString(R.string.status_starting)
                tvStatus.setTextColor(getColor(R.color.glass_warn))
                progressBarStatus.visibility = View.VISIBLE
                tvStatusLan.visibility = View.GONE
                tvStatusTunnel.visibility = View.GONE
            }
            is ServerState.SetupProgress -> {
                btnStartStop.text = "Starting..."
                btnStartStop.isEnabled = false
                btnOpenWeb.isEnabled = false
                btnAddon.isEnabled = false
                btnSettings.isEnabled = false
                switchTunnel.isEnabled = false

                viewStatusDot.backgroundTintList = ColorStateList.valueOf(getColor(R.color.glass_warn))
                tvStatus.text = "Status: Setting up...\n${state.message}"
                tvStatus.setTextColor(getColor(R.color.glass_warn))
                progressBarStatus.visibility = View.VISIBLE
                tvStatusLan.visibility = View.GONE
                tvStatusTunnel.visibility = View.GONE
            }
            is ServerState.Running -> {
                btnStartStop.text = getString(R.string.stop_server)
                btnStartStop.isEnabled = true
                btnOpenWeb.isEnabled = true
                btnAddon.isEnabled = true
                btnSettings.isEnabled = true
                switchTunnel.isEnabled = true

                viewStatusDot.backgroundTintList = ColorStateList.valueOf(getColor(R.color.glass_ok))
                tvStatus.text = "Status: Running (port ${state.port})"
                tvStatus.setTextColor(getColor(R.color.glass_ok))
                progressBarStatus.visibility = View.GONE

                if (!state.lanIp.isNullOrEmpty()) {
                    tvStatusLan.text = "LAN: http://${state.lanIp}:${state.port} (Tap to copy)"
                    tvStatusLan.visibility = View.VISIBLE
                } else {
                    tvStatusLan.visibility = View.GONE
                }

                startTunnelPolling()
            }
            is ServerState.Error -> {
                stopTunnelPolling()
                btnStartStop.text = getString(R.string.start_server)
                btnStartStop.isEnabled = true
                btnOpenWeb.isEnabled = false
                btnAddon.isEnabled = false
                btnSettings.isEnabled = false
                switchTunnel.isEnabled = false
                switchTunnel.isChecked = false

                viewStatusDot.backgroundTintList = ColorStateList.valueOf(getColor(R.color.glass_error))
                tvStatus.text = "Status: Error\n${state.message}"
                tvStatus.setTextColor(getColor(R.color.glass_error))
                progressBarStatus.visibility = View.GONE
                tvStatusLan.visibility = View.GONE
                tvStatusTunnel.visibility = View.GONE
            }
            is ServerState.Stopping -> {
                stopTunnelPolling()
                btnStartStop.text = getString(R.string.status_stopping)
                btnStartStop.isEnabled = false
                btnOpenWeb.isEnabled = false
                btnAddon.isEnabled = false
                btnSettings.isEnabled = false
                switchTunnel.isEnabled = false

                viewStatusDot.backgroundTintList = ColorStateList.valueOf(getColor(R.color.glass_error))
                tvStatus.text = getString(R.string.status_stopping)
                tvStatus.setTextColor(getColor(R.color.glass_error))
                progressBarStatus.visibility = View.VISIBLE
                tvStatusLan.visibility = View.GONE
                tvStatusTunnel.visibility = View.GONE
            }
        }
    }

    private fun requestNotificationPermissionAndStart() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            if (ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS)
                != PackageManager.PERMISSION_GRANTED
            ) {
                ActivityCompat.requestPermissions(
                    this,
                    arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                    100
                )
                return
            }
        }
        startServer()
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == 100) {
            if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                startServer()
            } else {
                Toast.makeText(this, "Notification permission required for background server", Toast.LENGTH_LONG).show()
            }
        }
    }

    private fun startServer() {
        val intent = ServerService.startIntent(this)
        ContextCompat.startForegroundService(this, intent)
    }

    private fun stopServer() {
        val intent = ServerService.stopIntent(this)
        startService(intent)
    }

    private fun openAppSettings() {
        try {
            val intent = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
                data = Uri.parse("package:$packageName")
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            startActivity(intent)
        } catch (_: Exception) {
            try {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                    val intent = Intent(
                        Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS,
                        Uri.parse("package:$packageName")
                    ).apply {
                        addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                    }
                    startActivity(intent)
                }
            } catch (_: Exception) {}
        }
    }

    private fun checkAndPromptBatteryOptimization(forcePrompt: Boolean = false) {
        val prefs = getSharedPreferences(BootReceiver.PREFS_NAME, Context.MODE_PRIVATE)
        if (!forcePrompt) {
            if (prefs.getBoolean(KEY_BACKGROUND_GUIDE_SHOWN, false)) return
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                val pm = getSystemService(Context.POWER_SERVICE) as? PowerManager
                if (pm?.isIgnoringBatteryOptimizations(packageName) == true) {
                    prefs.edit().putBoolean(KEY_BACKGROUND_GUIDE_SHOWN, true).apply()
                    return
                }
            }
        }
        prefs.edit().putBoolean(KEY_BACKGROUND_GUIDE_SHOWN, true).apply()

        val message = StringBuilder().apply {
            append("⚠️ IMPORTANT: Android may stop PencariMovie Server if background activity is restricted!\n\n")
            append("Recommended steps to keep the server running in the background:\n\n")
            append("1. Allow Background Activity:\n")
            append("   Allow the app to run in the background.\n\n")
            append("2. Lock in Recents App:\n")
            append("   Open the Recent Apps screen, long-press (or open the menu on) PencariMovie, and choose Lock (padlock/pin icon).\n\n")
            append("3. Battery > Unrestricted:\n")
            append("   Go to App Info > Battery / Battery usage > set to 'Unrestricted' / 'No restrictions', and turn OFF Smart mode / Optimized.\n")
            append("   (Samsung/Xiaomi/Oppo/Vivo users: also turn OFF 'Deep sleeping apps' and turn ON 'Auto-start' / 'Background launch').\n\n")
            append("4. Turn ON 'Auto start on boot' in the app:\n")
            append("   Enable the toggle inside this app so the server automatically restarts after the phone reboots.\n\n")
            append("5. Do NOT Swipe Clear:\n")
            append("   After locking in recents, keep the server notification active. Android kills background processes that have no foreground notification.\n\n")
            append("6. Check Cleaner / RAM Booster:\n")
            append("   If you use a cleaner or battery booster, exclude/whitelist PencariMovie from its clean list.")
        }.toString()

        MaterialAlertDialogBuilder(this)
            .setTitle("Keep Server Running in Background")
            .setMessage(message)
            .setCancelable(true)
            .setPositiveButton("Open Settings") { _, _ ->
                openAppSettings()
            }
            .setNegativeButton("Close", null)
            .show()
    }
}
