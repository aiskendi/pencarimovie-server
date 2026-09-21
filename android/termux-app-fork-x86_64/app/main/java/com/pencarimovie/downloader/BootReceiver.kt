package com.pencarimovie.downloader

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import android.util.Log
import androidx.core.content.ContextCompat

class BootReceiver : BroadcastReceiver() {
    companion object {
        private const val TAG = "BootReceiver"
        const val PREFS_NAME = "pencarimovie_settings"
        const val KEY_AUTO_START = "auto_start_on_boot"
    }

    override fun onReceive(context: Context, intent: Intent) {
        val action = intent.action
        Log.i(TAG, "Received broadcast intent action: $action")
        if (Intent.ACTION_BOOT_COMPLETED == action ||
            "android.intent.action.QUICKBOOT_POWERON" == action ||
            "com.htc.intent.action.QUICKBOOT_POWERON" == action
        ) {
            val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            val autoStart = prefs.getBoolean(KEY_AUTO_START, false)
            Log.i(TAG, "Auto start on boot setting is: $autoStart")
            if (autoStart) {
                Log.i(TAG, "Starting ServerService from boot broadcast")
                val startIntent = ServerService.startIntent(context)
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    ContextCompat.startForegroundService(context, startIntent)
                } else {
                    context.startService(startIntent)
                }
            }
        }
    }
}
