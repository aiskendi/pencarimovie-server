package com.pencarimovie.server

import android.content.Context
import android.os.Build
import android.util.Base64
import android.util.Log
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import org.drinkless.tdlib.Client
import org.drinkless.tdlib.TdApi
import org.json.JSONObject
import java.io.File
import java.security.MessageDigest
import java.security.SecureRandom
import java.util.concurrent.ConcurrentHashMap
import java.util.concurrent.TimeUnit
import javax.crypto.Cipher
import javax.crypto.spec.IvParameterSpec
import javax.crypto.spec.SecretKeySpec
import kotlin.coroutines.resume

/**
 * Manages TDLib (Telegram Database Library) client lifecycle, authentication,
 * guest auto-provisioning, and zero-disk on-the-fly streaming.
 */
class TdLibManager(private val context: Context, private val slot: String = "primary") : Client.ResultHandler {

    companion object {
        private const val TAG = "TdLibManager"
        private const val WP_API_BASE = "https://pencarimovie.com/wp-json/pencarimovie-server/v1"
        private const val PREFS_NAME = "tdlib_session"
        private const val KEY_DB_ENCRYPTION = "db_encryption_key"
        private const val KEY_BOT_TOKEN = "bot_token"
        private const val KEY_BOT_ID = "bot_id"
        private const val KEY_API_ID = "api_id"
        private const val KEY_API_HASH = "api_hash"

        /** True once libtdjni loaded. Used by the /api/session runtime preflight. */
        @Volatile
        var nativeLoaded: Boolean = false
            private set

        init {
            try {
                System.loadLibrary("tdjni")
                nativeLoaded = true
                Log.i(TAG, "Loaded libtdjni native library successfully")
            } catch (e: UnsatisfiedLinkError) {
                nativeLoaded = false
                Log.e(TAG, "Failed to load libtdjni: ${e.message}")
            }
        }
    }

    private var client: Client? = null

    /** Namespaced prefs so each pooled bot keeps its own token/session. */
    private val prefsName = "tdlib_session_$slot"

    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val httpClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(15, TimeUnit.SECONDS)
        .build()

    init {
        DebugLog.init(context)
    }

    private val _authState = MutableStateFlow<TdApi.AuthorizationState?>(null)
    val authState: StateFlow<TdApi.AuthorizationState?> = _authState

    private val _isReady = MutableStateFlow(false)
    val isReady: StateFlow<Boolean> = _isReady
    val hasSession: Boolean get() = _isReady.value && activeBotId.isNotEmpty()

    private val fileUpdates = MutableSharedFlow<TdApi.File>(replay = 10, extraBufferCapacity = 50)
    private val fileCache = ConcurrentHashMap<Int, TdApi.File>()
    private val initMutex = Mutex()

    private var activeApiId: Int = 0
    private var activeApiHash: String = ""
    private var activeBotToken: String = ""
    private var activeBotId: String = ""
    private var activeBotUsername: String = ""
    private var activeBotName: String = ""
    private var activeApiSecret: String = ""
    private var isProvisioning: Boolean = false

    /** Last raw TDLib/network error, surfaced to the UI as a friendly message. */
    @Volatile
    var lastError: String = ""
        private set

    /**
     * Set by [BotPool]. Returning false blocks guest auto-provisioning, so removing
     * the built-in primary bot does not immediately lease a replacement while other
     * pooled bots are still logged in (matches backend.php /api/bots/remove).
     */
    @Volatile
    var shouldAutoProvision: (() -> Boolean)? = null

    val botId: String get() = activeBotId
    val botToken: String get() = activeBotToken
    val botUsername: String get() = activeBotUsername
    val botName: String get() = activeBotName
    val apiSecret: String get() = activeApiSecret
    val isProvisioningInProgress: Boolean get() = isProvisioning

    fun initialize() {
        if (client != null) return
        scope.launch { ensureClient() }
    }

    /** Create the TDLib client if it does not exist (serialized by initMutex). */
    suspend fun ensureClient() {
        initMutex.withLock {
            if (client != null) return@withLock
            loadSavedCredentials()
            Log.i(TAG, "Creating TDLib client instance...")
            client = Client.create(this@TdLibManager, null, null)
        }
    }

    /** Wait until the client can accept a bot token (WaitPhoneNumber or Ready). */
    private suspend fun awaitAuthReady(timeoutMs: Long = 15_000L) {
        kotlinx.coroutines.withTimeoutOrNull(timeoutMs) {
            authState.first {
                it is TdApi.AuthorizationStateWaitPhoneNumber || it is TdApi.AuthorizationStateReady
            }
        }
    }

    private fun loadSavedCredentials() {
        val prefs = context.getSharedPreferences(prefsName, Context.MODE_PRIVATE)
        activeApiId = prefs.getInt(KEY_API_ID, 0)
        activeApiHash = prefs.getString(KEY_API_HASH, "") ?: ""
        activeBotToken = prefs.getString(KEY_BOT_TOKEN, "") ?: ""
        activeBotId = prefs.getString(KEY_BOT_ID, "") ?: ""
        activeBotUsername = prefs.getString("bot_username", "") ?: ""
        activeBotName = prefs.getString("bot_name", "") ?: ""
        activeApiSecret = prefs.getString("api_secret", "") ?: ""
    }

    private fun saveCredentials(apiId: Int, apiHash: String, botToken: String, botId: String, secret: String = "") {
        activeApiId = apiId
        activeApiHash = apiHash
        activeBotToken = botToken
        activeBotId = botId
        if (secret.isNotEmpty()) activeApiSecret = secret
        context.getSharedPreferences(prefsName, Context.MODE_PRIVATE).edit()
            .putInt(KEY_API_ID, apiId)
            .putString(KEY_API_HASH, apiHash)
            .putString(KEY_BOT_TOKEN, botToken)
            .putString(KEY_BOT_ID, botId)
            .putString("api_secret", activeApiSecret)
            .apply()
    }

    override fun onResult(obj: TdApi.Object?) {
        if (obj == null) return
        when (obj.constructor) {
            TdApi.UpdateAuthorizationState.CONSTRUCTOR -> {
                val update = obj as TdApi.UpdateAuthorizationState
                handleAuthState(update.authorizationState)
            }
            TdApi.UpdateFile.CONSTRUCTOR -> {
                val update = obj as TdApi.UpdateFile
                fileCache[update.file.id] = update.file
                fileUpdates.tryEmit(update.file)
            }
        }
    }

    private fun handleAuthState(state: TdApi.AuthorizationState) {
        _authState.value = state
        Log.i(TAG, "AuthorizationState changed to: ${state.javaClass.simpleName}")

        when (state) {
            is TdApi.AuthorizationStateWaitTdlibParameters -> {
                sendTdlibParameters()
            }
            is TdApi.AuthorizationStateWaitPhoneNumber -> {
                _isReady.value = false
                // An explicit provision/login may already be driving the flow
                // (e.g. right after logout) — don't race a second provision.
                if (isProvisioning) return
                scope.launch {
                    if (activeBotToken.isNotEmpty()) {
                        Log.i(TAG, "Authenticating with saved bot token...")
                        checkBotToken(activeBotToken)
                    } else if (shouldAutoProvision?.invoke() == false) {
                        Log.i(TAG, "No bot token saved, but another pooled bot is active — skipping guest provision")
                    } else {
                        Log.i(TAG, "No bot token saved, attempting auto-provisioning...")
                        autoProvisionGuest()
                    }
                }
            }
            is TdApi.AuthorizationStateReady -> {
                _isReady.value = true
                Log.i(TAG, "TDLib authorization is READY")
                scope.launch {
                    fetchMe()
                }
            }
            is TdApi.AuthorizationStateClosing,
            is TdApi.AuthorizationStateClosed -> {
                _isReady.value = false
                // A closed TDLib client cannot be reused, so drop the reference;
                // the next provision/login creates a fresh one. Without this,
                // provisioning after logout kept timing out.
                if (state is TdApi.AuthorizationStateClosed) client = null
                Log.i(TAG, "TDLib client closed")
            }
        }
    }

    private fun sendTdlibParameters() {
        val dbDir = File(context.filesDir, "tdlib/$slot").apply { mkdirs() }
        val filesDir = File(context.cacheDir, "tdlib_files/$slot").apply { mkdirs() }

        val params = TdApi.SetTdlibParameters().apply {
            databaseDirectory = dbDir.absolutePath
            this.filesDirectory = filesDir.absolutePath
            databaseEncryptionKey = getOrGenerateDbKey()
            useFileDatabase = false
            useChatInfoDatabase = false
            useMessageDatabase = true
            useSecretChats = false
            apiId = if (activeApiId != 0) activeApiId else 94575
            apiHash = if (activeApiHash.isNotEmpty()) activeApiHash else "a3406de8d171bb422bb6ddf3bbd800e2"
            systemLanguageCode = "en"
            deviceModel = Build.MODEL ?: "Android Device"
            systemVersion = "Android " + Build.VERSION.RELEASE
            applicationVersion = BuildConfig.VERSION_NAME
        }

        sendRequest(params) { result ->
            if (result is TdApi.Error) {
                Log.e(TAG, "Failed to set TDLib parameters: ${result.message} (${result.code})")
            } else {
                Log.i(TAG, "Set TDLib parameters successfully")
                // Bound TDLib's on-disk file cache so streamed parts are evicted by
                // TDLib's own LRU instead of accumulating forever or being churned by
                // per-request deletes (which wears flash). TDLib trims the least
                // recently used files once this size is exceeded.
                sendRequest(
                    TdApi.SetOption("storage_max_files_size", TdApi.OptionValueInteger(3L * 1024 * 1024 * 1024))
                ) { r ->
                    if (r is TdApi.Error) {
                        Log.w(TAG, "SetOption storage_max_files_size failed: ${r.message} (${r.code})")
                    } else {
                        Log.i(TAG, "TDLib storage_max_files_size set to 3 GB (LRU-managed cache)")
                    }
                }
            }
        }
    }

    /**
     * Auto-provisions a guest bot session by calling the WordPress REST API,
     * decrypting the encrypted credentials using AES-256-CBC with SHA-256(botToken),
     * and authenticating with TDLib.
     */
    suspend fun autoProvisionGuest(): Boolean = withContext(Dispatchers.IO) {
        isProvisioning = true
        lastError = ""
        try {
            // After a logout the previous client is gone: recreate it and wait
            // until it can accept a bot token before requesting a guest session.
            ensureClient()
            awaitAuthReady()
            Log.i(TAG, "Requesting guest provision from WordPress...")
            val url = "$WP_API_BASE/provision-session?_nocache=${System.currentTimeMillis()}"
            val req = Request.Builder().url(url).get().build()
            val resp = httpClient.newCall(req).execute()

            if (!resp.isSuccessful) {
                lastError = "Could not reach the PencariMovie API to get a guest session (HTTP ${resp.code})."
                Log.e(TAG, "Provision HTTP request failed with code ${resp.code}")
                return@withContext false
            }

            val body = resp.body?.string()
            if (body == null) {
                lastError = "The PencariMovie API returned an empty response."
                return@withContext false
            }
            val json = JSONObject(body)
            if (json.optInt("ok", 0) != 1) {
                lastError = json.optString("message", "").ifEmpty {
                    "The PencariMovie API did not return a guest session."
                }
                Log.e(TAG, "Provision response not ok: $body")
                return@withContext false
            }

            val botToken = json.getString("bot_token")
            val botId = json.optString("bot_id", "")
            val encryptedCreds = json.optString("encrypted_credentials", "")
            val ivB64 = json.optString("credentials_iv", "")
            val secret = json.optString("api_secret", "")

            var apiId = 0
            var apiHash = ""
            if (encryptedCreds.isNotEmpty() && ivB64.isNotEmpty()) {
                val creds = decryptCredentials(encryptedCreds, ivB64, botToken)
                if (creds != null) {
                    apiId = creds.first
                    apiHash = creds.second
                    Log.i(TAG, "Decrypted credentials from WordPress successfully: apiId=$apiId")
                }
            }

            saveCredentials(apiId, apiHash, botToken, botId, secret)
            checkBotToken(botToken)
            // Wait up to 15 seconds for TDLib to transition to Ready state and fetchMe() to complete
            val ready = kotlinx.coroutines.withTimeoutOrNull(15_000L) {
                isReady.first { it }
            } ?: false
            if (!ready) {
                // Keep the real TDLib error (e.g. ACCESS_TOKEN_EXPIRED from
                // checkBotToken) — overwriting it hid the actual reason behind a
                // misleading "token is correct?" hint.
                if (lastError.isEmpty()) {
                    lastError = "Guest session was obtained but TDLib did not connect in time."
                }
                Log.w(TAG, "TDLib did not reach Ready state within 15 seconds")
            } else {
                // fetchMe() fills username/name just after Ready; wait briefly so
                // the API response and bot pool see them (matches backend session meta).
                var waited = 0
                while (activeBotUsername.isEmpty() && waited < 3_000) { delay(150); waited += 150 }
            }
            ready
        } catch (e: Exception) {
            lastError = e.message ?: "Could not reach the PencariMovie API."
            Log.e(TAG, "autoProvisionGuest error: ${e.message}", e)
            false
        } finally {
            isProvisioning = false
        }
    }

    /**
     * Decrypts credentials from WordPress:
     * Key = SHA-256(botToken) (raw 32 bytes)
     * Cipher = AES-256-CBC with PKCS5 padding
     */
    private fun decryptCredentials(encryptedB64: String, ivB64: String, token: String): Pair<Int, String>? {
        return try {
            val keyBytes = MessageDigest.getInstance("SHA-256").digest(token.toByteArray(Charsets.UTF_8))
            val ivBytes = Base64.decode(ivB64, Base64.DEFAULT)
            val cipherText = Base64.decode(encryptedB64, Base64.DEFAULT)

            val cipher = Cipher.getInstance("AES/CBC/PKCS5Padding")
            cipher.init(Cipher.DECRYPT_MODE, SecretKeySpec(keyBytes, "AES"), IvParameterSpec(ivBytes))
            val plainBytes = cipher.doFinal(cipherText)
            val json = JSONObject(String(plainBytes, Charsets.UTF_8))
            val apiId = json.getInt("api_id")
            val apiHash = json.getString("api_hash")
            Pair(apiId, apiHash)
        } catch (e: Exception) {
            Log.e(TAG, "decryptCredentials error: ${e.message}")
            null
        }
    }

    /**
     * Fetch + decrypt the PencariMovie credentials for a bot token and persist them
     * into this slot's prefs BEFORE the client is created — the pooled-bot equivalent
     * of backend.php fd_fetch_credentials_from_wordpress(), which is called for every
     * bot it boots.
     *
     * Without it a pooled bot authenticates with the fallback api_id and has no
     * api_secret, so the WordPress /resolve-file bridge answers 401 and every
     * download through that bot fails with "File could not be resolved" — which is
     * exactly what happened once the guest/primary bot was removed from the pool.
     */
    suspend fun prefetchCredentials(token: String): Boolean = withContext(Dispatchers.IO) {
        try {
            val payload = JSONObject().put("bot_token", token).toString()
            val req = Request.Builder()
                .url("$WP_API_BASE/save-bot-token")
                .post(payload.toRequestBody("application/json".toMediaType()))
                .header("User-Agent", "pencarimovie-server/${BuildConfig.VERSION_NAME}")
                .build()
            httpClient.newCall(req).execute().use { resp ->
                val raw = resp.body?.string() ?: return@withContext false
                val clean = raw.removePrefix("\uFEFF").trim()
                val json = JSONObject(clean)
                if (json.optInt("ok", 0) != 1) {
                    Log.w(TAG, "save-bot-token rejected for slot=$slot: ${clean.take(160)}")
                    return@withContext false
                }
                val enc = json.optString("encrypted_credentials", "")
                val iv = json.optString("encryption_iv", json.optString("credentials_iv", ""))
                val creds = if (enc.isNotEmpty() && iv.isNotEmpty()) {
                    decryptCredentials(enc, iv, token)
                } else null
                if (creds == null) {
                    Log.w(TAG, "save-bot-token returned no usable credentials for slot=$slot")
                    return@withContext false
                }
                saveCredentials(
                    creds.first, creds.second, token,
                    json.optString("bot_id", ""), json.optString("api_secret", "")
                )
                Log.i(TAG, "Prefetched WordPress credentials for slot=$slot apiId=${creds.first}")
                true
            }
        } catch (e: Exception) {
            Log.w(TAG, "prefetchCredentials failed for slot=$slot: ${e.message}")
            false
        }
    }

    fun checkBotToken(token: String) {
        val check = TdApi.CheckAuthenticationBotToken(token)
        sendRequest(check) { result ->
            if (result is TdApi.Error) {
                lastError = result.message
                Log.e(TAG, "CheckAuthenticationBotToken error: ${result.message} (${result.code})")
            } else {
                lastError = ""
                Log.i(TAG, "Bot token accepted by TDLib")
            }
        }
    }

    private suspend fun fetchMe() {
        val me = sendRequestSuspend(TdApi.GetMe())
        if (me is TdApi.User) {
            activeBotId = me.id.toString()
            activeBotName = me.firstName ?: "PencariMovie Bot"
            activeBotUsername = me.usernames?.activeUsernames?.firstOrNull() ?: ""
            Log.i(TAG, "Logged in as bot: id=${me.id}, firstName=${me.firstName}, username=$activeBotUsername")
            context.getSharedPreferences(prefsName, Context.MODE_PRIVATE).edit()
                .putString(KEY_BOT_ID, activeBotId)
                .putString("bot_username", activeBotUsername)
                .putString("bot_name", activeBotName)
                .apply()
        }
    }

    fun logout() {
        sendRequest(TdApi.LogOut()) {}
        activeBotToken = ""
        activeBotId = ""
        activeBotUsername = ""
        activeBotName = ""
        context.getSharedPreferences(prefsName, Context.MODE_PRIVATE).edit()
            .remove(KEY_BOT_TOKEN)
            .remove(KEY_BOT_ID)
            .remove("bot_username")
            .remove("bot_name")
            .apply()
        _isReady.value = false
    }

    /**
     * Reads a slice of bytes on the fly via TDLib without permanent storage.
     * Uses TdApi.ReadFilePart to read chunk from TDLib cache into memory,
     * triggering TdApi.DownloadFile if the part is not yet available.
     * Automatically purges the cached file part after reading.
     */
    suspend fun readChunk(fileId: Int, offset: Long, limit: Int): ByteArray? = withContext(Dispatchers.IO) {
        DebugLog.log("TDLib", "readChunk BEGIN fileId=$fileId offset=$offset limit=$limit")
        try {
            // First attempt to read directly
            val readPart = TdApi.ReadFilePart(fileId, offset, limit.toLong())
            val directResult = sendRequestSuspend(readPart)
            when {
                directResult is TdApi.Data && directResult.data.isNotEmpty() -> {
                    DebugLog.log("TDLib", "readChunk DIRECT hit fileId=$fileId bytes=${directResult.data.size}")
                    return@withContext directResult.data
                }
                directResult is TdApi.Error -> {
                    DebugLog.log("TDLib", "readChunk direct ReadFilePart ERROR code=${directResult.code} msg=${directResult.message}")
                }
                else -> {
                    DebugLog.log("TDLib", "readChunk direct ReadFilePart empty/incomplete -> triggering DownloadFile")
                }
            }

            // If part not available, trigger download with high priority
            val download = TdApi.DownloadFile(fileId, 32, offset, limit.toLong(), false)
            sendRequest(download) {}

            // Wait up to 15 seconds for part to arrive
            val deadline = System.currentTimeMillis() + 15_000L
            var attempts = 0
            while (System.currentTimeMillis() < deadline) {
                delay(100)
                attempts++
                val retryResult = sendRequestSuspend(TdApi.ReadFilePart(fileId, offset, limit.toLong()))
                if (retryResult is TdApi.Data && retryResult.data.isNotEmpty()) {
                    DebugLog.log("TDLib", "readChunk PARTS arrived after ${attempts} polls, bytes=${retryResult.data.size}")
                    return@withContext retryResult.data
                }
                if (retryResult is TdApi.Error && retryResult.code != 400) {
                    DebugLog.log("TDLib", "readChunk retry ReadFilePart ERROR code=${retryResult.code} msg=${retryResult.message}")
                }
            }
            DebugLog.log("TDLib", "readChunk TIMEOUT after ${attempts} polls fileId=$fileId offset=$offset")
            null
        } catch (e: Exception) {
            DebugLog.log("TDLib", "readChunk EXCEPTION fileId=$fileId offset=$offset: ${e.message}")
            Log.w(TAG, "readChunk error (fileId=$fileId, offset=$offset): ${e.message}")
            null
        }
    }

    /**
     * Resolves a Telegram remote file ID (e.g. from Bot API or MTProto file_id_mt)
     * into a TDLib File object containing the internal integer fileId.
     */
    /**
     * Normalizes a Bot API remote file ID version byte to be compatible with TDLib.
     * Newer Bot API servers encode version 61+, while client TDLib builds require version < Version::Next.
     * Rewriting the trailing subversion byte to 30 allows TDLib to deserialize the FileId structure.
     */
    private fun normalizeRemoteFileId(rawId: String): String {
        try {
            val padded = when (rawId.length % 4) {
                2 -> "$rawId=="
                3 -> "$rawId="
                else -> rawId
            }
            val bytes = Base64.decode(padded, Base64.URL_SAFE)
            if (bytes.size > 2 && bytes[bytes.size - 1] == 4.toByte()) {
                val version = bytes[bytes.size - 2].toInt() and 0xFF
                if (version > 30) {
                    bytes[bytes.size - 2] = 30.toByte()
                    val patched = Base64.encodeToString(bytes, Base64.URL_SAFE or Base64.NO_WRAP or Base64.NO_PADDING)
                    Log.i(TAG, "Normalized remote file ID version from $version to 30")
                    return patched
                }
            }
        } catch (e: Exception) {
            Log.w(TAG, "Could not normalize remote file ID: ${e.message}")
        }
        return rawId
    }

    suspend fun getRemoteFile(remoteFileId: String): TdApi.File? = withContext(Dispatchers.IO) {
        DebugLog.log("TDLib", "getRemoteFile BEGIN idLen=${remoteFileId.length}")
        try {
            val normalizedId = normalizeRemoteFileId(remoteFileId)
            if (normalizedId != remoteFileId) {
                DebugLog.log("TDLib", "getRemoteFile normalized file id version byte (61->30)")
            }
            var res = sendRequestSuspend(TdApi.GetRemoteFile(normalizedId, null))
            if (res is TdApi.Error) {
                DebugLog.log("TDLib", "getRemoteFile null-type failed code=${res.code} msg=${res.message}; retry FileTypeVideo")
                res = sendRequestSuspend(TdApi.GetRemoteFile(normalizedId, TdApi.FileTypeVideo()))
            }
            if (res is TdApi.Error) {
                DebugLog.log("TDLib", "getRemoteFile FileTypeVideo failed code=${res.code} msg=${res.message}; retry FileTypeDocument")
                res = sendRequestSuspend(TdApi.GetRemoteFile(normalizedId, TdApi.FileTypeDocument()))
            }
            if (res is TdApi.File) {
                fileCache[res.id] = res
                DebugLog.log("TDLib", "getRemoteFile SUCCESS fileId=${res.id} size=${res.size}")
                Log.i(TAG, "getRemoteFile success: fileId=${res.id}, size=${res.size}")
                res
            } else if (res is TdApi.Error) {
                DebugLog.log("TDLib", "getRemoteFile FAILED TdApi.Error code=${res.code} message=${res.message}")
                Log.e(TAG, "getRemoteFile TdApi.Error: code=${res.code}, message=${res.message}")
                null
            } else {
                DebugLog.log("TDLib", "getRemoteFile unexpected type: ${res?.javaClass?.simpleName}")
                Log.w(TAG, "getRemoteFile returned unexpected type: ${res?.javaClass?.simpleName}")
                null
            }
        } catch (e: Exception) {
            DebugLog.log("TDLib", "getRemoteFile EXCEPTION: ${e.message}")
            Log.e(TAG, "getRemoteFile error for $remoteFileId: ${e.message}")
            null
        }
    }

    /**
     * Returns the known size (in bytes) of a local TDLib file, or 0 if unknown.
     * Used to backfill Content-Range total when the payload had no file_size.
     */
    suspend fun getFileSize(fileId: Int): Long = withContext(Dispatchers.IO) {
        try {
            val res = sendRequestSuspend(TdApi.GetFile(fileId))
            if (res is TdApi.File) {
                fileCache[res.id] = res
                DebugLog.log("TDLib", "getFileSize fileId=$fileId size=${res.size}")
                res.size
            } else {
                DebugLog.log("TDLib", "getFileSize failed for fileId=$fileId -> ${res?.javaClass?.simpleName}")
                0L
            }
        } catch (e: Exception) {
            DebugLog.log("TDLib", "getFileSize EXCEPTION fileId=$fileId: ${e.message}")
            0L
        }
    }

    /**
     * Deletes temporary file parts from disk cache immediately to ensure zero storage overhead.
     */
    fun cleanupFile(fileId: Int) {
        sendRequest(TdApi.DeleteFile(fileId)) {}
    }

    /**
     * Sends an asynchronous TDLib request with callback.
     */
    fun sendRequest(function: TdApi.Function<*>, callback: (TdApi.Object) -> Unit) {
        val c = client
        if (c != null) {
            c.send(function) { result ->
                callback(result)
            }
        } else {
            callback(TdApi.Error(500, "TDLib client not initialized"))
        }
    }

    /**
     * Sends a TDLib request and suspends until result is returned.
     */
    suspend fun sendRequestSuspend(function: TdApi.Function<*>): TdApi.Object = suspendCancellableCoroutine { cont ->
        sendRequest(function) { result ->
            if (cont.isActive) {
                cont.resume(result)
            }
        }
    }

    private fun getOrGenerateDbKey(): ByteArray {
        val prefs = context.getSharedPreferences(prefsName, Context.MODE_PRIVATE)
        var keyB64 = prefs.getString(KEY_DB_ENCRYPTION, null)
        if (keyB64 == null) {
            val key = ByteArray(32)
            SecureRandom().nextBytes(key)
            keyB64 = Base64.encodeToString(key, Base64.NO_WRAP)
            prefs.edit().putString(KEY_DB_ENCRYPTION, keyB64).apply()
            return key
        }
        return Base64.decode(keyB64, Base64.NO_WRAP)
    }

    fun close() {
        client?.send(TdApi.Close()) {}
        client = null
        _isReady.value = false
    }
}
