package com.pencarimovie.server

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.graphics.Bitmap
import android.net.Uri
import android.os.Bundle
import android.util.Log
import android.view.Menu
import android.view.MenuItem
import android.view.View
import android.webkit.ConsoleMessage
import android.webkit.WebChromeClient
import android.webkit.WebResourceRequest
import android.webkit.WebSettings
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.ProgressBar
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.appbar.MaterialToolbar

class WebActivity : AppCompatActivity() {

    companion object {
        const val EXTRA_URL = "extra_url"
        const val EXTRA_TITLE = "extra_title"

        fun start(context: Context, url: String, title: String? = null) {
            val intent = Intent(context, WebActivity::class.java).apply {
                putExtra(EXTRA_URL, url)
                if (title != null) putExtra(EXTRA_TITLE, title)
            }
            context.startActivity(intent)
        }
    }

    private lateinit var toolbar: MaterialToolbar
    private lateinit var progressBar: ProgressBar
    private lateinit var webView: WebView
    private var initialUrl: String = "http://127.0.0.1:8088/"

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Debug builds expose the WebView over chrome://inspect (adb forward +
        // DevTools protocol) so the served UI can be inspected live. Release
        // builds stay locked down.
        if (BuildConfig.DEBUG) {
            WebView.setWebContentsDebuggingEnabled(true)
        }
        setContentView(R.layout.activity_web)

        toolbar = findViewById(R.id.webToolbar)
        progressBar = findViewById(R.id.webProgressBar)
        webView = findViewById(R.id.webView)

        initialUrl = intent.getStringExtra(EXTRA_URL) ?: "http://127.0.0.1:8088/"
        val customTitle = intent.getStringExtra(EXTRA_TITLE)

        toolbar.title = customTitle ?: getString(R.string.app_name)
        toolbar.subtitle = initialUrl
        toolbar.setNavigationOnClickListener {
            finish()
        }

        setupToolbarMenu()
        setupWebView()
        setupBackNavigation()

        webView.loadUrl(initialUrl)
    }

    override fun onNewIntent(intent: Intent?) {
        super.onNewIntent(intent)
        setIntent(intent)
        val newUrl = intent?.getStringExtra(EXTRA_URL) ?: "http://127.0.0.1:8088/"
        val customTitle = intent?.getStringExtra(EXTRA_TITLE)
        toolbar.subtitle = newUrl
        if (customTitle != null) {
            toolbar.title = customTitle
        }
        webView.loadUrl(newUrl)
    }

    private fun setupToolbarMenu() {
        toolbar.inflateMenu(R.menu.menu_web)
        toolbar.setOnMenuItemClickListener { item ->
            when (item.itemId) {
                R.id.action_refresh -> {
                    webView.reload()
                    true
                }
                R.id.action_open_external -> {
                    val currentUrl = webView.url ?: initialUrl
                    try {
                        val intent = Intent(Intent.ACTION_VIEW, Uri.parse(currentUrl))
                        startActivity(intent)
                    } catch (e: Exception) {
                        Toast.makeText(this, "Could not open external browser", Toast.LENGTH_SHORT).show()
                    }
                    true
                }
                else -> false
            }
        }
    }

    @SuppressLint("SetJavaScriptEnabled")
    private fun setupWebView() {
        val settings = webView.settings
        settings.javaScriptEnabled = true
        settings.domStorageEnabled = true
        settings.databaseEnabled = true
        settings.loadWithOverviewMode = true
        settings.useWideViewPort = true
        settings.builtInZoomControls = true
        settings.displayZoomControls = false
        settings.allowFileAccess = true
        settings.allowContentAccess = true

        // Take focus so keyboard / D-pad / wheel input reaches the page
        // (the WebView subclass turns Up/Down/Page keys into list scrolling).
        webView.isFocusable = true
        webView.isFocusableInTouchMode = true
        webView.requestFocus()

        // Ensure audio/video and local streams work seamlessly
        settings.mediaPlaybackRequiresUserGesture = false
        settings.mixedContentMode = WebSettings.MIXED_CONTENT_ALWAYS_ALLOW

        webView.webChromeClient = object : WebChromeClient() {
            override fun onConsoleMessage(msg: ConsoleMessage): Boolean {
                // Only surface page console output in debug builds; release
                // builds stay quiet.
                if (BuildConfig.DEBUG) {
                    Log.d("WebViewConsole", msg.message() + " @" + msg.lineNumber() + " " + msg.sourceId())
                }
                return BuildConfig.DEBUG
            }

            override fun onProgressChanged(view: WebView?, newProgress: Int) {
                if (newProgress in 1..99) {
                    progressBar.visibility = View.VISIBLE
                    progressBar.progress = newProgress
                } else {
                    progressBar.visibility = View.GONE
                }
            }

            override fun onReceivedTitle(view: WebView?, title: String?) {
                super.onReceivedTitle(view, title)
                if (!title.isNullOrBlank() && !title.startsWith("http")) {
                    toolbar.title = title
                }
            }
        }

        webView.webViewClient = object : WebViewClient() {
            override fun shouldOverrideUrlLoading(view: WebView?, request: WebResourceRequest?): Boolean {
                val uri = request?.url ?: return false
                return handleUrlScheme(uri)
            }

            @Deprecated("Deprecated in Java")
            override fun shouldOverrideUrlLoading(view: WebView?, url: String?): Boolean {
                val uri = if (url != null) Uri.parse(url) else return false
                return handleUrlScheme(uri)
            }

            override fun onPageStarted(view: WebView?, url: String?, favicon: Bitmap?) {
                super.onPageStarted(view, url, favicon)
                progressBar.visibility = View.VISIBLE
                if (url != null) {
                    toolbar.subtitle = url
                }
            }

            override fun onPageFinished(view: WebView?, url: String?) {
                super.onPageFinished(view, url)
                progressBar.visibility = View.GONE
                if (url != null) {
                    toolbar.subtitle = url
                }
            }
        }
    }

    private fun handleUrlScheme(uri: Uri): Boolean {
        val scheme = uri.scheme?.lowercase() ?: ""
        val urlString = uri.toString()

        // Handle custom protocols like Stremio, Telegram, Magnet, Intents
        if (scheme in listOf("stremio", "org-stremio-addon", "magnet", "tg", "intent", "market")) {
            try {
                val intent = Intent(Intent.ACTION_VIEW, uri).apply {
                    addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
                }
                startActivity(intent)
                return true
            } catch (e: Exception) {
                if (scheme == "stremio" || scheme == "org-stremio-addon") {
                    Toast.makeText(this, "Stremio app not found. Install Stremio to open addon links.", Toast.LENGTH_LONG).show()
                } else {
                    Toast.makeText(this, "No app available to open this link: $urlString", Toast.LENGTH_SHORT).show()
                }
                return true
            }
        }

        // External non-local links (e.g. github.com, t.me) opened via external browser
        val host = uri.host?.lowercase() ?: ""
        val isLocalHost = host == "localhost" || host == "127.0.0.1" || host.startsWith("192.168.") || host.startsWith("10.") || host.contains("trycloudflare.com") || host.contains("pencarimovie.com")

        if (!isLocalHost && (scheme == "http" || scheme == "https")) {
            try {
                val intent = Intent(Intent.ACTION_VIEW, uri)
                startActivity(intent)
                return true
            } catch (_: Exception) {
                return false
            }
        }

        // Keep local and app pages inside the WebView
        return false
    }

    private fun setupBackNavigation() {
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                if (webView.canGoBack()) {
                    webView.goBack()
                } else {
                    finish()
                }
            }
        })
    }

    override fun onResume() {
        super.onResume()
        webView.onResume()
    }

    override fun onPause() {
        webView.onPause()
        super.onPause()
    }

    override fun onDestroy() {
        webView.destroy()
        super.onDestroy()
    }
}
