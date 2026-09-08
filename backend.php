<?php

declare(strict_types=1);

// Suppress PHP error output to prevent HTML warnings from breaking JSON/header responses.
// Errors are still logged via error_log for debugging.
ini_set('display_errors', '0');
ini_set('html_errors', '0');
error_reporting(E_ALL);

// Ensure bundled bin/ directory is added to PATH so MadelineProto ProcessRunner can locate PHP/FrankenPHP for IPC
$fdBinDir = __DIR__ . DIRECTORY_SEPARATOR . 'bin';
if (is_dir($fdBinDir)) {
    $existingPath = (string) ($_SERVER['PATH'] ?? ($_ENV['PATH'] ?? ''));
    if (!str_contains($existingPath, $fdBinDir)) {
        if (\function_exists('putenv')) {
            @putenv('PATH=' . $fdBinDir . PATH_SEPARATOR . $existingPath);
        }
        $_SERVER['PATH'] = $fdBinDir . PATH_SEPARATOR . $existingPath;
    }
}

/**
 * API credentials are no longer hardcoded here.
 * They are fetched from the WordPress REST API endpoint (/save-bot-token)
 * after successful bot token validation, encrypted with the token as key.
 */
function fd_is_temp_app_dir(?string $dir): bool
{
    if ($dir === null || $dir === '') {
        return true;
    }
    $real = realpath($dir) ?: $dir;
    $tempDir = realpath(sys_get_temp_dir());
    if ($tempDir !== false && str_starts_with($real, $tempDir)) {
        return true;
    }
    return str_contains($real, 'frankenphp_');
}

function fd_storage_has_session(string $storageDir): bool
{
    $session = $storageDir . DIRECTORY_SEPARATOR . 'session.madeline';
    return is_dir($session)
        || is_file($session)
        || is_file($storageDir . DIRECTORY_SEPARATOR . 'bot_id.txt')
        || is_file($storageDir . DIRECTORY_SEPARATOR . 'session_meta.json');
}

function fd_get_storage_dir(): string
{
    static $storageDir = null;
    if ($storageDir !== null && is_dir($storageDir) && is_writable($storageDir)) {
        return $storageDir;
    }

    // Prefer the served project root, not __DIR__.
    // FrankenPHP can extract PHP into a temp folder, so __DIR__/storage
    // would lose the Madeline session on restart / next worker.

    $candidates = [];
    $docRoot = (string) ($_SERVER['DOCUMENT_ROOT'] ?? '');
    if ($docRoot !== '') {
        $candidates[] = rtrim($docRoot, '/\\') . DIRECTORY_SEPARATOR . 'storage';
    }
    $cwd = getcwd();
    if ($cwd !== false) {
        $candidates[] = $cwd . DIRECTORY_SEPARATOR . 'storage';
    }
    $script = (string) ($_SERVER['SCRIPT_FILENAME'] ?? '');
    if ($script !== '') {
        $candidates[] = dirname($script) . DIRECTORY_SEPARATOR . 'storage';
    }

    $unique = [];
    foreach ($candidates as $candidate) {
        if ($candidate === '' || fd_is_temp_app_dir(dirname($candidate))) {
            continue;
        }
        $unique[$candidate] = true;
    }
    $candidates = array_keys($unique);

    foreach ($candidates as $candidate) {
        if (is_dir($candidate) && is_writable($candidate) && fd_storage_has_session($candidate)) {
            return $storageDir = $candidate;
        }
    }

    foreach ($candidates as $candidate) {
        if (is_dir($candidate) && is_writable($candidate)) {
            return $storageDir = $candidate;
        }
        $parent = dirname($candidate);
        if (!file_exists($candidate) && is_dir($parent) && is_writable($parent)) {
            return $storageDir = $candidate;
        }
    }

    $appStorage = __DIR__ . DIRECTORY_SEPARATOR . 'storage';
    if (!fd_is_temp_app_dir(__DIR__) && is_dir($appStorage) && is_writable($appStorage)) {
        return $storageDir = $appStorage;
    }

    if ($candidates !== []) {
        return $storageDir = $candidates[0];
    }

    return $storageDir = $appStorage;
}

function fd_storage_path(string $file): string
{
    $file = ltrim($file, '/\\');
    if (str_starts_with($file, 'storage/') || str_starts_with($file, 'storage\\')) {
        $subPath = substr($file, 7);
    } else {
        $subPath = $file;
    }
    $subPath = ltrim($subPath, '/\\');
    $storageDir = fd_get_storage_dir();

    if (!is_dir($storageDir)) {
        @mkdir($storageDir, 0777, true);
    }

    $fullPath = $storageDir . DIRECTORY_SEPARATOR . $subPath;
    $parent = dirname($fullPath);
    if (!is_dir($parent)) {
        @mkdir($parent, 0777, true);
    }
    return $fullPath;
}

define('FD_SESSION_PATH', fd_storage_path('storage/session.madeline'));
define('FD_WP_API_BASE', 'https://pencarimovie.com/wp-json/pencarimovie-server/v1');
define('FD_WP_AJAX_URL', 'https://pencarimovie.com/wp-admin/admin-ajax.php');
define('FD_APP_VERSION', '1.7.0');
define('FD_WP_VERSION_URL', FD_WP_API_BASE . '/version');
define('FD_API_SECRET_PATH', fd_storage_path('storage/api_secret.key'));
define('FD_BOT_ID_CACHE_PATH', fd_storage_path('storage/bot_id.txt'));
define('FD_DEVICE_ID_PATH', fd_storage_path('storage/device_id.txt'));
define('FD_SESSION_META_PATH', fd_storage_path('storage/session_meta.json'));
define('FD_BOT_POOL_PATH', fd_storage_path('storage/bot_pool.json'));
define('FD_CATALOG_SETTINGS_PATH', fd_storage_path('storage/catalog_settings.json'));

/**
 * Optional DNS resolution mapping for curl (e.g. "example.com:443:1.2.3.4").
 */
define('FD_CURL_RESOLVE', '');
function fd_json(array $data, int $status = 200): never
{
    $body = json_encode($data, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
    if (!headers_sent()) {
        http_response_code($status);
        header('Content-Type: application/json; charset=utf-8');
        header('Cache-Control: no-store, no-cache, must-revalidate');
        header('Access-Control-Allow-Origin: *');
        header('Access-Control-Allow-Methods: GET, POST, OPTIONS');
        header('Access-Control-Allow-Headers: Content-Type, Authorization, X-API-Secret');
        if ($status >= 400) {
            header('Connection: close');
        }
        header('Content-Length: ' . strlen($body));
    }
    echo $body;
    if (function_exists('fastcgi_finish_request')) {
        fastcgi_finish_request();
    }
    exit;
}

function fd_log(string $message, array $context = []): void
{
    $suffix = '';
    if ($context !== []) {
        $suffix = ' ' . json_encode($context, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
    }

    $logLine = '[' . date('Y-m-d H:i:s') . '] [PencariMovie Downloader] ' . $message . $suffix . "\n";
    $logPath = fd_storage_path('storage/debug.log');
    $dir = dirname($logPath);
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    @file_put_contents($logPath, $logLine, FILE_APPEND | LOCK_EX);
    error_log('[PencariMovie Downloader] ' . $message . $suffix);
}

/**
 * Get the stored API secret (used for X-API-Secret header on WordPress requests).
 * Returns empty string if no secret has been saved yet.
 */
function fd_get_api_secret(): string
{
    $path = FD_API_SECRET_PATH;
    if (!is_file($path)) {
        return '';
    }
    $secret = trim((string) file_get_contents($path));
    return $secret;
}

/**
 * Save the API secret received from WordPress during bot login.
 */
function fd_save_api_secret(string $secret): void
{
    $path = FD_API_SECRET_PATH;
    $dir = dirname($path);
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    @file_put_contents($path, $secret, LOCK_EX);
}

/**
 * Clear the stored API secret (called during logout / session clear).
 */
function fd_clear_api_secret(): void
{
    $path = FD_API_SECRET_PATH;
    if (is_file($path)) {
        @unlink($path);
    }
}

/**
 * Get the cached Bot ID (avoids booting full MadelineProto on lightweight requests).
 * Source of truth is session_meta.json. bot_id.txt is a legacy fallback only.
 */
function fd_get_bot_id(): string
{
    $meta = fd_load_session_meta();
    $fromMeta = trim((string) ($meta['bot_id'] ?? ''));
    if ($fromMeta !== '') {
        return $fromMeta;
    }

    $path = FD_BOT_ID_CACHE_PATH;
    if (is_file($path)) {
        $val = trim((string) file_get_contents($path));
        if ($val !== '') {
            return $val;
        }
    }
    return '';
}

/**
 * Clear leftover bot_id.txt from older installs.
 */
function fd_clear_bot_id(): void
{
    $path = FD_BOT_ID_CACHE_PATH;
    if (is_file($path)) {
        @unlink($path);
    }
}

function fd_get_bot_session_path(string $botId = ''): string
{
    $botId = trim($botId);
    if ($botId === '') {
        return FD_SESSION_PATH;
    }
    return fd_storage_path('storage/sessions/' . preg_replace('/[^a-zA-Z0-9_\-]/', '', $botId) . '/session.madeline');
}

function fd_has_local_session(string $botId = ''): bool
{
    $botId = trim($botId);
    if ($botId !== '') {
        $sess = fd_get_bot_session_path($botId);
        return is_file($sess) || is_dir($sess);
    }
    if (is_file(FD_SESSION_PATH) || is_dir(FD_SESSION_PATH)) {
        return true;
    }
    // Check if any pool bot has a valid session
    $pool = fd_get_bot_pool();
    foreach ($pool as $b) {
        $bId = trim((string)($b['bot_id'] ?? ''));
        if ($bId !== '') {
            $sess = fd_get_bot_session_path($bId);
            if (is_file($sess) || is_dir($sess)) {
                return true;
            }
        }
    }
    return false;
}

function fd_load_session_meta(): array
{
    $path = FD_SESSION_META_PATH;
    if (!is_file($path)) {
        return [];
    }
    $data = @json_decode((string) @file_get_contents($path), true);
    return is_array($data) ? $data : [];
}

function fd_save_session_meta(string $botId, string $botUsername = '', string $botName = ''): void
{
    $dir = dirname(FD_SESSION_META_PATH);
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    @file_put_contents(
        FD_SESSION_META_PATH,
        json_encode([
            'bot_id' => $botId,
            'bot_username' => $botUsername,
            'bot_name' => $botName,
        ], JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES),
        LOCK_EX
    );
    // Sync into bot pool
    fd_add_pool_bot([
        'bot_id' => $botId,
        'bot_username' => $botUsername,
        'bot_name' => $botName,
        'is_active' => true,
        'status' => 'online',
        'updated_at' => time(),
    ]);
    // Remove the old one-line cache so bot identity lives in one file.
    fd_clear_bot_id();
}

function fd_clear_session_meta(): void
{
    if (is_file(FD_SESSION_META_PATH)) {
        @unlink(FD_SESSION_META_PATH);
    }
}

/**
 * ── Bot Pool Management Helpers ──────────────────────────────────────────
 */
function fd_get_bot_pool(): array
{
    $path = FD_BOT_POOL_PATH;
    $pool = [];
    if (is_file($path)) {
        $data = @json_decode((string) @file_get_contents($path), true);
        if (is_array($data)) {
            $pool = $data;
        }
    }
    // Ensure the primary/active bot is always in the pool if session meta exists
    $activeId = fd_get_bot_id();
    if ($activeId !== '') {
        $found = false;
        foreach ($pool as $b) {
            if ((string) ($b['bot_id'] ?? '') === $activeId) {
                $found = true;
                break;
            }
        }
        if (!$found) {
            $meta = fd_load_session_meta();
            $pool[] = [
                'bot_id' => $activeId,
                'bot_username' => (string) ($meta['bot_username'] ?? ''),
                'bot_name' => (string) ($meta['bot_name'] ?? $activeId),
                'status' => 'online',
                'added_at' => time(),
                'updated_at' => time(),
                'is_active' => true,
            ];
            fd_save_bot_pool($pool);
        }
    }
    return $pool;
}

function fd_save_bot_pool(array $pool): void
{
    $dir = dirname(FD_BOT_POOL_PATH);
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    @file_put_contents(
        FD_BOT_POOL_PATH,
        json_encode(array_values($pool), JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES | JSON_PRETTY_PRINT),
        LOCK_EX
    );
}

function fd_add_pool_bot(array $botData): array
{
    $botId = trim((string) ($botData['bot_id'] ?? ''));
    if ($botId === '') {
        return [];
    }
    $pool = fd_get_bot_pool();
    $found = false;
    foreach ($pool as $idx => $item) {
        if ((string) ($item['bot_id'] ?? '') === $botId) {
            $pool[$idx] = array_merge($item, $botData);
            $found = true;
            break;
        }
    }
    if (!$found) {
        $pool[] = array_merge([
            'bot_id' => $botId,
            'bot_username' => '',
            'bot_name' => '',
            'status' => 'online',
            'added_at' => time(),
            'updated_at' => time(),
        ], $botData);
    }
    fd_save_bot_pool($pool);
    return $pool;
}

function fd_remove_pool_bot(string $botId): array
{
    $botId = trim($botId);
    if ($botId === '') {
        return fd_get_bot_pool();
    }
    $pool = fd_get_bot_pool();
    $newPool = [];
    foreach ($pool as $item) {
        if ((string) ($item['bot_id'] ?? '') !== $botId) {
            $newPool[] = $item;
        }
    }
    fd_save_bot_pool($newPool);
    // Delete session files for this bot
    $botSessDir = dirname(fd_get_bot_session_path($botId));
    if (is_dir($botSessDir)) {
        fd_clear_session_directory($botSessDir);
    }
    // If active bot was removed, update session meta to another available bot
    $currentActive = fd_get_bot_id();
    if ($currentActive === $botId) {
        if (!empty($newPool)) {
            $first = $newPool[0];
            fd_save_session_meta(
                (string) ($first['bot_id'] ?? ''),
                (string) ($first['bot_username'] ?? ''),
                (string) ($first['bot_name'] ?? '')
            );
        } else {
            fd_clear_session_meta();
            fd_clear_session();
        }
    }
    if (empty($newPool)) {
        if (is_file(FD_BOT_POOL_PATH)) {
            @unlink(FD_BOT_POOL_PATH);
        }
        fd_clear_session_meta();
        fd_clear_session();
    }
    return $newPool;
}

/**
 * Pick an available bot from the pool using round-robin / least-busy / cooldown strategy.
 */
function fd_pick_pool_bot(): array
{
    $pool = fd_get_bot_pool();
    $activeBotId = fd_get_bot_id();
    $validBots = [];

    foreach ($pool as $b) {
        $bId = trim((string) ($b['bot_id'] ?? ''));
        if ($bId === '') {
            continue;
        }
        $cooldownUntil = (int) ($b['cooldown_until'] ?? 0);
        if ($cooldownUntil > time()) {
            continue; // Skip flooded / cooldown bots
        }
        $hasSess = fd_has_local_session($bId) || ($bId === $activeBotId && fd_has_local_session());
        if ($hasSess) {
            $validBots[] = $b;
        }
    }

    if (empty($validBots)) {
        // Fallback to active bot if available
        if ($activeBotId !== '') {
            $meta = fd_load_session_meta();
            return [
                'bot_id' => $activeBotId,
                'bot_username' => (string) ($meta['bot_username'] ?? ''),
                'bot_name' => (string) ($meta['bot_name'] ?? ''),
            ];
        }
        return [];
    }

    // Round-robin selection using persistent tracker file across requests/workers
    $rrFile = fd_storage_path('storage/rr_bot_index.txt');
    $rrIndex = 0;
    if (is_file($rrFile)) {
        $rrIndex = (int) @file_get_contents($rrFile);
    }
    $picked = $validBots[$rrIndex % count($validBots)];
    @file_put_contents($rrFile, (string) (($rrIndex + 1) % count($validBots)), LOCK_EX);
    return $picked;
}

function fd_auto_provision_guest(): ?array
{
    fd_ensure_autoload();
    $provisionUrl = FD_WP_API_BASE . '/provision-session?_nocache=' . time();
    $resp = fd_http_json($provisionUrl, [], 'GET', 15);

    if (empty($resp['ok']) || empty($resp['bot_token'])) {
        fd_log('provision endpoint returned error or incomplete payload', ['resp' => $resp]);
        return null;
    }

    $botToken = trim((string) $resp['bot_token']);

    if (!empty($resp['api_secret'])) {
        fd_save_api_secret((string) $resp['api_secret']);
    }

    // Forward the encrypted API credentials so fd_boot_madeline decrypts them using $botToken
    $overrides = [];
    if (!empty($resp['encrypted_credentials']) && !empty($resp['credentials_iv'])) {
        $overrides['encrypted_credentials'] = $resp['encrypted_credentials'];
        $overrides['encryption_iv'] = $resp['credentials_iv'];
    }

    $targetBotId = !empty($resp['bot_id']) ? (string) $resp['bot_id'] : '';

    // Clean any stale session for this bot before initial login
    if ($targetBotId !== '') {
        fd_clear_session($targetBotId);
    }

    // Capture stray output before boot
    $diagObLevel = ob_get_level();
    while (ob_get_level() > 0) {
        ob_get_clean();
    }
    while (ob_get_level() < $diagObLevel) {
        ob_start();
    }

    [$madeline, $error] = fd_boot_madeline($botToken, $overrides, $targetBotId);

    if (!$madeline) {
        fd_log('auto provision fd_boot_madeline failed', ['error' => $error]);
        return null;
    }

    try {
        $self = $madeline->getSelf();
        $botId = (string) ($self['id'] ?? $targetBotId);
        $botUsername = (string) ($self['username'] ?? '');
        $botName = (string) ($self['first_name'] ?? '');

        fd_save_session_meta($botId, $botUsername, $botName);
        fd_add_pool_bot([
            'bot_id' => $botId,
            'bot_username' => $botUsername,
            'bot_name' => $botName,
        ]);

        return [
            'bot_id' => $botId,
            'bot_username' => $botUsername,
            'bot_name' => $botName,
            'madeline' => $madeline,
        ];
    } catch (Throwable $e) {
        fd_log('auto provision getSelf failed', ['error' => $e->getMessage()]);
        return null;
    }
}

function fd_is_cloudflare_tunnel_request(): bool
{
    $hosts = [
        (string) ($_SERVER['HTTP_HOST'] ?? ''),
        (string) ($_SERVER['HTTP_X_FORWARDED_HOST'] ?? ''),
    ];
    foreach ($hosts as $raw) {
        foreach (explode(',', $raw) as $part) {
            $hostOnly = strtolower(explode(':', trim($part))[0]);
            if ($hostOnly !== '' && (
                str_ends_with($hostOnly, '.trycloudflare.com')
                || $hostOnly === 'trycloudflare.com'
                || str_ends_with($hostOnly, '.tunnel.pencarimovie.com')
                || str_ends_with($hostOnly, '-tunnel.pencarimovie.com')
                || $hostOnly === 'tunnel.pencarimovie.com'
            )) {
                return true;
            }
        }
    }

    return !empty($_SERVER['HTTP_CF_CONNECTING_IP'])
        || !empty($_SERVER['HTTP_CF_RAY'])
        || !empty($_SERVER['HTTP_CF_VISITOR']);
}

function fd_is_local_request(): bool
{
    // cloudflared proxies as 127.0.0.1. Treat TryCloudflare / CF headers as remote
    // so enable/disable/bot-login stay on the real local dashboard.
    if (fd_is_cloudflare_tunnel_request()) {
        return false;
    }

    $remoteAddr = (string) ($_SERVER['REMOTE_ADDR'] ?? '');
    if ($remoteAddr === '') {
        return true;
    }

    if (in_array($remoteAddr, ['127.0.0.1', '::1'], true) || str_starts_with($remoteAddr, '::ffff:127.0.0.1')) {
        return true;
    }

    // If client IP matches the server IP (same device/host making request to itself)
    $serverAddr = (string) ($_SERVER['SERVER_ADDR'] ?? '');
    if ($serverAddr !== '' && $remoteAddr === $serverAddr) {
        return true;
    }

    // Check private & reserved IPv4/IPv6 ranges
    if (filter_var($remoteAddr, FILTER_VALIDATE_IP, FILTER_FLAG_NO_PRIV_RANGE | FILTER_FLAG_NO_RES_RANGE) === false) {
        return true;
    }

    // Carrier-Grade NAT (CGNAT, RFC 6598: 100.64.0.0/10) used by mobile ISPs
    $long = ip2long($remoteAddr);
    if ($long !== false) {
        $cgnatStart = ip2long('100.64.0.0');
        $cgnatEnd = ip2long('100.127.255.255');
        if ($long >= $cgnatStart && $long <= $cgnatEnd) {
            return true;
        }
    }

    // Host header is localhost or loopback
    $httpHost = (string) ($_SERVER['HTTP_HOST'] ?? '');
    $hostOnly = explode(':', $httpHost)[0];
    if (in_array($hostOnly, ['localhost', '127.0.0.1', '::1'], true)) {
        return true;
    }

    return false;
}

function fd_require_local_request(): void
{
    if (!fd_is_local_request()) {
        fd_json(['ok' => 0, 'message' => 'This endpoint is restricted to local requests only.'], 403);
    }
}

function fd_decode_download_payload(string $payload): array
{
    $payload = strtr($payload, '-_', '+/');
    $padding = strlen($payload) % 4;
    if ($padding > 0) {
        $payload .= str_repeat('=', 4 - $padding);
    }

    $decoded = base64_decode($payload, true);
    if ($decoded === false) {
        return [];
    }

    $json = json_decode($decoded, true);
    return is_array($json) ? $json : [];
}

function fd_http_json(string $url, array $payload = [], string $method = 'GET', int $timeout = 20): array
{
    $query = '';
    if ($method === 'GET' && $payload) {
        $query = '?' . http_build_query($payload);
    }

    // Build header array
    $headers = [
        'Accept: application/json',
        'X-App-Version: ' . FD_APP_VERSION,
    ];

    // Add API secret header for authenticating with WordPress endpoints
    $apiSecret = fd_get_api_secret();
    if ($apiSecret !== '') {
        $headers[] = "X-API-Secret: $apiSecret";
    }

    $body = '';
    if ($method !== 'GET' && $payload) {
        $body = json_encode($payload, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
        $headers[] = 'Content-Type: application/json';
    }

    $response = fd_http_get_contents($url . $query, [
        'method' => $method,
        'headers' => $headers,
        'body' => $body,
        'timeout' => $timeout,
    ]);
    $decoded = json_decode((string) $response, true);
    return is_array($decoded) ? $decoded : [];
}

/**
 * Low-level HTTP fetch using curl (preferred) or file_get_contents (fallback).
 *
 * Uses curl when available because bundled PHP 8.5 on Windows has unreliable
 * file_get_contents SSL handling (OpenSSL CA bundle issues). Curl works around
 * this by using its own CA store or accepting verify_peer=false when needed.
 *
 * @param string $url     The full URL to request.
 * @param array  $options Optional. {
 *     @var string   $method  HTTP method (GET, POST, etc.). Default 'GET'.
 *     @var string[] $headers Array of raw header strings, e.g. ["Accept: application/json"].
 *     @var string   $body    Request body for POST/PUT requests.
 *     @var int      $timeout Connection timeout in seconds. Default 15.
 * }
 * @return string|false Response body on success, false on failure.
 */
function fd_http_get_contents(string $url, array $options = []): string|false
{
    $method = strtoupper($options['method'] ?? 'GET');
    $headers = $options['headers'] ?? [];
    $body = $options['body'] ?? '';
    $timeout = (int) ($options['timeout'] ?? 15);

    // Ensure X-App-Version and User-Agent headers are sent on all requests
    $hasVersionHeader = false;
    $hasUserAgentHeader = false;
    foreach ($headers as $h) {
        if (stripos($h, 'X-App-Version:') === 0) {
            $hasVersionHeader = true;
        }
        if (stripos($h, 'User-Agent:') === 0) {
            $hasUserAgentHeader = true;
        }
    }
    if (!$hasVersionHeader) {
        $headers[] = 'X-App-Version: ' . FD_APP_VERSION;
    }
    if (!$hasUserAgentHeader) {
        $headers[] = 'User-Agent: pencarimovie-server/' . FD_APP_VERSION;
    }

    // Direct cURL fetch
    if (function_exists('curl_version')) {
        $ch = curl_init();
        // Build base curl options
        $curlOpts = [
            CURLOPT_URL => $url,
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT => $timeout,
            CURLOPT_CONNECTTIMEOUT => max(5, $timeout - 5),
            CURLOPT_FOLLOWLOCATION => true,
            CURLOPT_SSL_VERIFYPEER => false,
            CURLOPT_SSL_VERIFYHOST => 2,
            CURLOPT_HTTPHEADER => $headers,
            CURLOPT_HEADERFUNCTION => function ($curl, $headerLine) {
                $len = strlen($headerLine);
                $parts = explode(':', $headerLine, 2);
                if (count($parts) === 2) {
                    $name = strtolower(trim($parts[0]));
                    $val = trim($parts[1]);
                    if ($name === 'x-min-version' || $name === 'x-update-url' || $name === 'x-update-required' || $name === 'x-sponsor-name' || $name === 'x-sponsor-desc' || $name === 'x-sponsor-url') {
                        fd_update_version_state([$name => $val]);
                    }
                }
                return $len;
            },
        ];

        // Fallback DNS / direct Cloudflare IP resolution for pencarimovie.com & Cinemeta
        // Essential in Android / Termux proot where /etc/resolv.conf is missing or blocked by mobile carrier DNS
        $resolveEntries = [];
        if (defined('FD_CURL_RESOLVE') && FD_CURL_RESOLVE !== '') {
            $resolveEntries[] = FD_CURL_RESOLVE;
        } else {
            // Default Cloudflare Anycast IPs for pencarimovie.com
            $resolveEntries[] = 'pencarimovie.com:443:104.21.47.164';
            $resolveEntries[] = 'pencarimovie.com:443:172.67.149.53';
            $resolveEntries[] = 'pencarimovie.com:80:104.21.47.164';
            $resolveEntries[] = 'pencarimovie.com:80:172.67.149.53';
            // Default Cloudflare Anycast IPs for v3-cinemeta.strem.io
            $resolveEntries[] = 'v3-cinemeta.strem.io:443:104.17.88.107';
            $resolveEntries[] = 'v3-cinemeta.strem.io:443:104.17.89.107';
        }
        $curlOpts[CURLOPT_RESOLVE] = $resolveEntries;

        curl_setopt_array($ch, $curlOpts);

        if ($method === 'POST') {
            curl_setopt($ch, CURLOPT_POST, true);
            if ($body !== '') {
                curl_setopt($ch, CURLOPT_POSTFIELDS, $body);
            }
        } elseif ($method !== 'GET') {
            curl_setopt($ch, CURLOPT_CUSTOMREQUEST, $method);
            if ($body !== '') {
                curl_setopt($ch, CURLOPT_POSTFIELDS, $body);
            }
        }

        $cStart = microtime(true);
        $response = curl_exec($ch);
        $cDuration = round(microtime(true) - $cStart, 3);
        $httpCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);

        if ($response === false) {
            fd_log('curl request failed', [
                'url' => $url,
                'http_code' => $httpCode,
                'duration_seconds' => $cDuration,
                'errno' => curl_errno($ch),
                'error' => curl_error($ch),
            ]);
        } else {
            fd_log('curl request completed', [
                'url' => $url,
                'http_code' => $httpCode,
                'duration_seconds' => $cDuration,
                'bytes' => strlen($response),
            ]);
        }

        // In PHP 8.5+ curl_close() is a no-op, just here for readability
        if (PHP_VERSION_ID < 80500) {
            curl_close($ch);
        }
        unset($ch);

        if ($response === false || $response === '') {
            return false;
        }

        return $response;
    }

    // Fallback: file_get_contents with SSL verification disabled
    // (Windows bundled PHP has no valid CA bundle by default)
    $headerStr = '';
    foreach ($headers as $h) {
        $headerStr .= $h . "\r\n";
    }

    $ctx = stream_context_create([
        'http' => [
            'method' => $method,
            'timeout' => $timeout,
            'ignore_errors' => true,
            'header' => $headerStr,
        ],
        'ssl' => [
            'verify_peer' => false,
            'verify_peer_name' => false,
        ],
    ]);

    if ($body !== '') {
        $ctx = stream_context_create([
            'http' => [
                'method' => $method,
                'timeout' => $timeout,
                'ignore_errors' => true,
                'header' => $headerStr,
                'content' => $body,
            ],
            'ssl' => [
                'verify_peer' => false,
                'verify_peer_name' => false,
            ],
        ]);
    }

    $res = @file_get_contents($url, false, $ctx);
    $responseHeaders = function_exists('http_get_last_response_headers')
        ? (http_get_last_response_headers() ?? [])
        : ($GLOBALS['http_response_header'] ?? []);

    if (is_array($responseHeaders)) {
        foreach ($responseHeaders as $line) {
            $parts = explode(':', $line, 2);
            if (count($parts) === 2) {
                $name = strtolower(trim($parts[0]));
                $val = trim($parts[1]);
                if ($name === 'x-min-version' || $name === 'x-update-url' || $name === 'x-update-required' || $name === 'x-sponsor-name' || $name === 'x-sponsor-desc' || $name === 'x-sponsor-url') {
                    fd_update_version_state([$name => $val]);
                }
            }
        }
    }

    return $res;
}

function fd_resolve_shortcode_cached(string $shortCode, string $botId): ?array
{
    static $memoryCache = [];
    $cacheKey = $shortCode . ':' . $botId;
    if (isset($memoryCache[$cacheKey])) {
        return $memoryCache[$cacheKey];
    }

    $diskCacheFile = fd_storage_path('storage/resolve_cache_' . md5($cacheKey) . '.json');
    if (is_file($diskCacheFile)) {
        $mtime = (int) filemtime($diskCacheFile);
        if ((time() - $mtime) < 86400) {
            $cachedRaw = @file_get_contents($diskCacheFile);
            if ($cachedRaw) {
                $cachedJson = json_decode($cachedRaw, true);
                if (is_array($cachedJson) && (!empty($cachedJson['file_id_mt']) || !empty($cachedJson['file_id']))) {
                    $memoryCache[$cacheKey] = $cachedJson;
                    return $cachedJson;
                }
            }
        } else {
            @unlink($diskCacheFile);
        }
    }
    return null;
}

function fd_save_resolve_cache(string $shortCode, string $botId, array $data): void
{
    $cacheKey = $shortCode . ':' . $botId;
    $diskCacheFile = fd_storage_path('storage/resolve_cache_' . md5($cacheKey) . '.json');
    @file_put_contents($diskCacheFile, json_encode($data, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE), LOCK_EX);
}

/**
 * Concurrently resolves a shortCode across all available bots in parallel using curl_multi.
 * Returns immediately as soon as ANY bot successfully resolves a file ID.
 */
function fd_resolve_shortcode_concurrent(string $shortCode, array $candidateBots = []): array
{
    if (empty($candidateBots)) {
        // Rotate the starting bot candidate so resolutions and download requests round-robin across bots
        $picked = fd_pick_pool_bot();
        $startBotId = !empty($picked['bot_id']) ? (string) $picked['bot_id'] : '';
        if ($startBotId !== '') {
            $candidateBots[] = $startBotId;
        }

        $pool = fd_get_bot_pool();
        foreach ($pool as $pBot) {
            $pId = (string) ($pBot['bot_id'] ?? '');
            if ($pId !== '' && !in_array($pId, $candidateBots, true)) {
                $candidateBots[] = $pId;
            }
        }
        $activeBotId = fd_get_bot_id();
        if ($activeBotId !== '' && !in_array($activeBotId, $candidateBots, true)) {
            $candidateBots[] = $activeBotId;
        }
    }

    if (empty($candidateBots)) {
        return ['ok' => 0, 'message' => 'No active bots available for resolution.'];
    }

    // Check fast local cache across candidate bots first (0ms lookup)
    foreach ($candidateBots as $bId) {
        $cached = fd_resolve_shortcode_cached($shortCode, $bId);
        if ($cached !== null) {
            if (empty($cached['bot_id'])) {
                $cached['bot_id'] = $bId;
            }
            return $cached;
        }
    }

    // Single bot fast path
    if (count($candidateBots) === 1) {
        $bId = (string) reset($candidateBots);
        return fd_resolve_shortcode($shortCode, $bId);
    }

    // Race all candidate bots simultaneously via curl_multi
    $secret = fd_get_api_secret();
    $mh = curl_multi_init();
    $handles = [];

    $headers = [
        'Accept: application/json',
        'User-Agent: pencarimovie-server/' . FD_APP_VERSION,
        'X-App-Version: ' . FD_APP_VERSION,
    ];
    if ($secret !== '') {
        $headers[] = 'X-API-Secret: ' . $secret;
    }

    fd_log('resolve_shortcode_concurrent starting', [
        'short_code' => $shortCode,
        'bot_count' => count($candidateBots),
        'bots' => $candidateBots,
    ]);

    foreach ($candidateBots as $bId) {
        $targetUrl = FD_WP_API_BASE . '/resolve-file?' . http_build_query([
            'short_code' => $shortCode,
            'bot_id' => $bId,
        ]);

        $ch = curl_init($targetUrl);
        $resOpts = [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT => 12,
            CURLOPT_CONNECTTIMEOUT => 5,
            CURLOPT_HTTPHEADER => $headers,
            CURLOPT_SSL_VERIFYPEER => false,
            CURLOPT_SSL_VERIFYHOST => 2,
            CURLOPT_RESOLVE => [
                'pencarimovie.com:443:104.21.47.164',
                'pencarimovie.com:443:172.67.149.53',
            ],
        ];
        curl_setopt_array($ch, $resOpts);
        curl_multi_add_handle($mh, $ch);
        $handles[$bId] = $ch;
    }

    $running = null;
    $winner = null;
    $winnerBotId = null;
    $lastErrorResult = null;
    $botStatuses = [];

    do {
        $status = curl_multi_exec($mh, $running);
        if ($status > 0) {
            break;
        }

        while ($info = curl_multi_info_read($mh)) {
            $ch = $info['handle'];
            $bId = (string) array_search($ch, $handles, true);
            $raw = curl_multi_getcontent($ch);
            $httpCode = (int) curl_getinfo($ch, CURLINFO_HTTP_CODE);
            $curlErr = curl_error($ch);

            $botStatuses[$bId] = [
                'http' => $httpCode,
                'err' => $curlErr ?: null,
            ];

            if ($httpCode >= 200 && $httpCode < 300 && is_string($raw) && $raw !== '') {
                $json = json_decode($raw, true);
                if (is_array($json)) {
                    if (!empty($json['file_id_mt']) || !empty($json['file_id'])) {
                        $winner = $json;
                        $winnerBotId = $bId;
                        break 2; // Found first winning resolution!
                    }
                    $lastErrorResult = $json;
                }
            } elseif (is_string($raw) && $raw !== '') {
                $json = json_decode($raw, true);
                if (is_array($json)) {
                    $lastErrorResult = $json;
                }
            }
        }

        if ($running > 0) {
            curl_multi_select($mh, 0.02);
        }
    } while ($running > 0);

    foreach ($handles as $ch) {
        curl_multi_remove_handle($mh, $ch);
    }
    curl_multi_close($mh);

    fd_log('concurrent resolve multi-curl complete', [
        'short_code' => $shortCode,
        'winner_bot' => $winnerBotId,
        'statuses' => $botStatuses,
    ]);

    if ($winner !== null && $winnerBotId !== null) {
        $winner['bot_id'] = $winnerBotId;
        fd_save_resolve_cache($shortCode, $winnerBotId, $winner);
        fd_log('resolve_shortcode_concurrent succeeded', [
            'short_code' => $shortCode,
            'winner_bot' => $winnerBotId,
        ]);
        return $winner;
    }

    // If all bots returned 404 on the initial race, WordPress may have just triggered a background
    // bot relay/forward. Wait 1.2s and do a second pass before giving up.
    $all404 = !empty($botStatuses) && count(array_filter($botStatuses, fn($s) => ($s['http'] ?? 0) === 404)) === count($botStatuses);
    if ($all404) {
        usleep(1200000); // 1.2 seconds wait for Telegram relay to complete

        // Re-race candidate bots
        $mhRetry = curl_multi_init();
        $retryHandles = [];
        foreach ($candidateBots as $bId) {
            $targetUrl = FD_WP_API_BASE . '/resolve-file?' . http_build_query([
                'short_code' => $shortCode,
                'bot_id' => $bId,
            ]);

            $ch = curl_init($targetUrl);
            curl_setopt_array($ch, [
                CURLOPT_RETURNTRANSFER => true,
                CURLOPT_TIMEOUT => 6,
                CURLOPT_CONNECTTIMEOUT => 3,
                CURLOPT_HTTPHEADER => $headers,
                CURLOPT_SSL_VERIFYPEER => false,
                CURLOPT_SSL_VERIFYHOST => 2,
                CURLOPT_RESOLVE => [
                    'pencarimovie.com:443:104.21.47.164',
                    'pencarimovie.com:443:172.67.149.53',
                ],
            ]);
            curl_multi_add_handle($mhRetry, $ch);
            $retryHandles[$bId] = $ch;
        }

        $rRunning = null;
        do {
            $status = curl_multi_exec($mhRetry, $rRunning);
            if ($status > 0) break;

            while ($info = curl_multi_info_read($mhRetry)) {
                $ch = $info['handle'];
                $bId = (string) array_search($ch, $retryHandles, true);
                $raw = curl_multi_getcontent($ch);
                $httpCode = (int) curl_getinfo($ch, CURLINFO_HTTP_CODE);

                if ($httpCode >= 200 && $httpCode < 300 && is_string($raw) && $raw !== '') {
                    $json = json_decode($raw, true);
                    if (is_array($json) && (!empty($json['file_id_mt']) || !empty($json['file_id']))) {
                        $winner = $json;
                        $winnerBotId = $bId;
                        break 2;
                    }
                }
            }
            if ($rRunning > 0) {
                curl_multi_select($mhRetry, 0.02);
            }
        } while ($rRunning > 0);

        foreach ($retryHandles as $ch) {
            curl_multi_remove_handle($mhRetry, $ch);
        }
        curl_multi_close($mhRetry);

        if ($winner !== null && $winnerBotId !== null) {
            $winner['bot_id'] = $winnerBotId;
            fd_save_resolve_cache($shortCode, $winnerBotId, $winner);
            fd_log('resolve_shortcode_concurrent succeeded on relay retry', [
                'short_code' => $shortCode,
                'winner_bot' => $winnerBotId,
            ]);
            return $winner;
        }
    }

    fd_log('resolve_shortcode_concurrent failed across all bots', [
        'short_code' => $shortCode,
        'bot_statuses' => $botStatuses,
        'last_error' => $lastErrorResult,
    ]);

    return is_array($lastErrorResult) ? $lastErrorResult : ['ok' => 0, 'message' => 'Failed to resolve short code across all bots.'];
}

function fd_resolve_shortcode(string $shortCode, string $botId = ''): array
{
    if ($botId === '') {
        return fd_resolve_shortcode_concurrent($shortCode);
    }

    $cached = fd_resolve_shortcode_cached($shortCode, $botId);
    if ($cached !== null) {
        return $cached;
    }

    // Probabilistic cleanup (1 in 50 calls) to prune stale resolve cache files
    if (mt_rand(1, 50) === 1) {
        $storageDir = fd_get_storage_dir();
        $staleFiles = glob($storageDir . '/resolve_cache_*.json');
        if ($staleFiles) {
            $now = time();
            foreach ($staleFiles as $sf) {
                if (($now - (int) filemtime($sf)) > 86400) {
                    @unlink($sf);
                }
            }
        }
    }

    $url = FD_WP_API_BASE . '/resolve-file';
    $params = ['short_code' => $shortCode, 'bot_id' => $botId];

    // Fast 5-second timeout so stream attempts fail quickly instead of hanging
    $res = fd_http_json($url, $params, 'GET', 5);
    if (!empty($res['file_id_mt']) || !empty($res['file_id'])) {
        fd_save_resolve_cache($shortCode, $botId, $res);
    }

    return $res;
}

function fd_load_madeline_autoload(): ?string
{
    $candidates = [
        __DIR__ . DIRECTORY_SEPARATOR . 'vendor' . DIRECTORY_SEPARATOR . 'autoload.php',
        fd_storage_path('vendor/autoload.php'),
        fd_storage_path('vendor/madelineproto/autoload.php'),
        fd_storage_path('storage/vendor/autoload.php'),
    ];

    foreach ($candidates as $candidate) {
        if (is_file($candidate)) {
            require_once $candidate;
            return $candidate;
        }
    }

    return null;
}

/**
 * Ensure the Composer/MadelineProto autoloader is loaded.
 * Uses output buffering to suppress the polyfill.php echo warning on Windows.
 * Safe to call multiple times — uses require_once internally.
 */
function fd_ensure_autoload(): bool
{
    if (class_exists('\\danog\\MadelineProto\\API', false)) {
        return true;
    }
    $level = ob_get_level();
    ob_start();
    $result = fd_load_madeline_autoload();
    while (ob_get_level() > $level) {
        ob_end_clean();
    }

    return $result !== null;
}

function fd_require_fileinfo(): bool
{
    if (extension_loaded('fileinfo')) {
        return true;
    }

    if (!headers_sent()) {
        http_response_code(501);
        header('Content-Type: application/json; charset=utf-8');
        header('Cache-Control: no-store');
    }
    echo json_encode([
        'ok' => 0,
        'message' => 'MadelineProto requires the fileinfo extension to run. Try running sudo apt-get install php8.5-fileinfo.',
        'hint' => 'Install MadelineProto dependencies and ensure a bot session is configured.',
    ], JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
    exit;
}

/**
 * Decrypt api_id/api_hash that were encrypted by WordPress using the bot token.
 *
 * @param string $encryptedB64 Base64-encoded ciphertext.
 * @param string $ivB64       Base64-encoded initialization vector.
 * @param string $token       The bot token (used as AES-256-CBC key material).
 * @return array [int|null api_id, string|null error]
 */
function fd_decrypt_credentials(string $encryptedB64, string $ivB64, string $token): array
{
    if ($encryptedB64 === '' || $ivB64 === '') {
        return [null, 'Empty encrypted credentials or IV from WordPress.'];
    }
    if (!function_exists('openssl_decrypt')) {
        return [null, 'openssl extension is required to decrypt credentials.'];
    }

    $key = hash('sha256', $token, true);
    $iv = base64_decode($ivB64, true);
    $ciphertext = base64_decode($encryptedB64, true);
    if ($iv === false || $ciphertext === false) {
        return [null, 'Invalid base64 in encrypted credentials or IV.'];
    }

    $decrypted = @openssl_decrypt($ciphertext, 'aes-256-cbc', $key, OPENSSL_RAW_DATA, $iv);
    if ($decrypted === false) {
        return [null, 'Failed to decrypt credentials from WordPress.'];
    }

    $data = json_decode($decrypted, true);
    if (!is_array($data) || empty($data['api_id']) || empty($data['api_hash'])) {
        return [null, 'Decrypted credentials have invalid structure.'];
    }

    return [(int) $data['api_id'], (string) $data['api_hash']];
}

/**
 * Fetch encrypted api_id/api_hash from the WordPress REST API.
 *
 * Calls POST /save-bot-token with the bot token. WordPress validates the token
 * via Telegram getMe, then returns AES-256-CBC encrypted credentials.
 *
 * @param string $botToken The bot token to authenticate with WordPress.
 * @return array [int|null api_id, string|null error]
 */
function fd_fetch_credentials_from_wordpress(string $botToken): array
{
    try {
        $payload = json_encode(['bot_token' => $botToken], JSON_UNESCAPED_SLASHES);

        $body = fd_http_get_contents(FD_WP_API_BASE . '/save-bot-token', [
            'method' => 'POST',
            'headers' => [
                'Content-Type: application/json',
                'User-Agent: pencarimovie-server/' . FD_APP_VERSION,
            ],
            'body' => $payload,
            'timeout' => 30,
        ]);
        if ($body === false) {
            fd_log('wordpress raw response body: false');
            return [null, 'WordPress connection failed (HTTP request failed).'];
        }

        fd_log('wordpress raw response body', ['body' => $body]);

        // Strip UTF-8 Byte Order Mark (BOM) if present at the start of the response
        if (str_starts_with($body, "\xEF\xBB\xBF")) {
            $body = substr($body, 3);
        }
        $body = trim($body);

        $data = json_decode($body, true);
        if (!is_array($data) || empty($data['ok'])) {
            $msg = $data['message'] ?? 'WordPress rejected the bot token.';
            fd_log('wordpress rejected token', ['message' => $msg, 'response_decoded' => $data]);
            return [null, $msg];
        }
        if (empty($data['encrypted_credentials']) || empty($data['encryption_iv'])) {
            return [null, 'WordPress response missing encrypted credentials.'];
        }

        // Save the API secret returned by WordPress for authenticating future requests
        if (!empty($data['api_secret'])) {
            fd_save_api_secret((string) $data['api_secret']);
            fd_log('api secret saved from wordpress');
        }

        return fd_decrypt_credentials(
            $data['encrypted_credentials'],
            $data['encryption_iv'],
            $botToken
        );
    } catch (\Throwable $e) {
        return [null, 'WordPress connection failed: ' . $e->getMessage()];
    }
}

/**
 * Load cached Telegram API credentials from local storage.
 * Created at runtime after first successful WordPress fetch.
 *
 * @return array [int|null api_id, string|null api_hash]
 */
function fd_load_cached_api_credentials(): array
{
    $path = fd_storage_path('storage/api_credentials.json');
    if (!is_file($path)) {
        return [null, null];
    }
    $data = @json_decode((string) file_get_contents($path), true);
    if (!is_array($data) || empty($data['api_id']) || empty($data['api_hash'])) {
        @unlink($path);
        return [null, null];
    }
    return [(int) $data['api_id'], (string) $data['api_hash']];
}

/**
 * Persist decrypted api_id/api_hash to local cache.
 * This avoids calling WordPress on every request during session resume.
 */
function fd_save_cached_api_credentials(int $apiId, string $apiHash): void
{
    $path = fd_storage_path('storage/api_credentials.json');
    $dir = dirname($path);
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    @file_put_contents(
        $path,
        json_encode(['api_id' => $apiId, 'api_hash' => $apiHash], JSON_UNESCAPED_SLASHES),
        LOCK_EX
    );
}

/**
 * Check whether an IPC worker is currently listening for the given session dir.
 *
 * MadelineProto stores the IPC endpoint in a file named "ipc" inside the session
 * directory (on Windows it contains "tcp://127.0.0.1:PORT"). If that file exists
 * and the endpoint is connectable, a worker is already running.
 */
function fd_ipc_worker_running(string $sessionDir): bool
{
    $ipcFile = rtrim($sessionDir, '/\\') . DIRECTORY_SEPARATOR . 'ipc';
    if (!is_file($ipcFile)) {
        return false;
    }
    $endpoint = trim((string) @file_get_contents($ipcFile));
    if ($endpoint === '') {
        return false;
    }
    if (str_starts_with($endpoint, 'tcp://')) {
        $parts = parse_url($endpoint);
        $host = $parts['host'] ?? '127.0.0.1';
        $port = (int) ($parts['port'] ?? 0);
        if ($port <= 0) {
            return false;
        }
        $conn = @fsockopen($host, $port, $errno, $errstr, 0.5);
        if ($conn) {
            fclose($conn);
            return true;
        }
        return false;
    }
    // Unix socket or FIFO — treat file existence as a weak signal.
    return true;
}

/**
 * Ensure a persistent MadelineProto IPC worker is running for a session directory.
 *
 * Under FrankenPHP each web request is a short-lived process, so workers spawned
 * via MadelineProto's internal proc_open() die when the request ends. To keep IPC
 * reliable we spawn the worker as a DETACHED background process that survives the
 * request, then let fd_boot_madeline() connect to it as an IPC client (~40-60ms).
 *
 * @param string $sessionDir Absolute session directory (e.g. .../session.madeline)
 * @return bool True if a worker is running (or was just started).
 */
function fd_ensure_ipc_worker(string $sessionDir): bool
{
    if (fd_ipc_worker_running($sessionDir)) {
        return true;
    }

    // If an IPC daemon is currently starting and holding the session lock, do not spawn another duplicate.
    $normalizedDir = rtrim(str_replace('\\', '/', $sessionDir), '/');
    $lockPath = $normalizedDir . '/lock';
    if (file_exists($lockPath)) {
        $fp = @fopen($lockPath, 'c');
        if ($fp) {
            $canLock = @flock($fp, LOCK_EX | LOCK_NB);
            if (!$canLock) {
                // Another process/worker is already holding the exclusive lock (initializing or running).
                @fclose($fp);
                // Wait briefly for it to finish socket binding
                for ($i = 0; $i < 20; $i++) {
                    if (fd_ipc_worker_running($sessionDir)) {
                        return true;
                    }
                    usleep(100000); // 100ms
                }
                return fd_ipc_worker_running($sessionDir);
            }
            @flock($fp, LOCK_UN);
            @fclose($fp);
        }
    }

    $root = fd_get_app_root();
    $phpBin = PHP_BINARY;
    if (PHP_SAPI !== 'cli' && PHP_SAPI !== 'phpdbg') {
        $prefix = $_SERVER['PREFIX'] ?? ($_ENV['PREFIX'] ?? '');
        if ($prefix !== '' && is_file($prefix . '/bin/php')) {
            $phpBin = $prefix . '/bin/php';
        } else {
            $candidate = $root . DIRECTORY_SEPARATOR . 'bin' . DIRECTORY_SEPARATOR . (fd_is_windows() ? 'php.exe' : 'php');
            if (is_file($candidate)) {
                $phpBin = $candidate;
            }
        }
    }
    $entry = $root . DIRECTORY_SEPARATOR . 'vendor' . DIRECTORY_SEPARATOR . 'danog' . DIRECTORY_SEPARATOR . 'madelineproto' . DIRECTORY_SEPARATOR . 'src' . DIRECTORY_SEPARATOR . 'Ipc' . DIRECTORY_SEPARATOR . 'Runner' . DIRECTORY_SEPARATOR . 'entry.php';
    if (!is_file($entry)) {
        return false;
    }

    $startupId = random_int(100000000, 2000000000);
    $sessionDir = str_replace('/', DIRECTORY_SEPARATOR, $sessionDir);

    $cmd = '"' . $phpBin . '"'
        . ' -dhtml_errors=0 -ddisplay_errors=0 -dlog_errors=1'
        . ' "' . $entry . '"'
        . ' madeline-ipc'
        . ' "' . $sessionDir . '"'
        . ' ' . $startupId;

    if (fd_is_windows()) {
        // Spawn a detached worker that survives the calling web request.
        // PowerShell Start-Process keeps the child inside the FrankenPHP job
        // object (killed when the request ends), but `start "" /b` via
        // pclose(popen()) detaches it so it persists. Verified under FrankenPHP.
        @pclose(@popen('start "" /b ' . $cmd . ' > NUL 2>&1', 'r'));
    } else {
        @shell_exec('nohup ' . $cmd . ' > /dev/null 2>&1 &');
    }

    // Wait up to ~8s for the worker to come up.
    for ($i = 0; $i < 40; $i++) {
        if (fd_ipc_worker_running($sessionDir)) {
            return true;
        }
        usleep(200000); // 200ms
    }
    return fd_ipc_worker_running($sessionDir);
}

/**
 * Boot MadelineProto using session-based auth.
 *
 * - If a session file exists and is valid, returns the instance directly.
 * - If session is invalid or missing and $botToken is provided, does botLogin().
 * - If no session and no token, returns an error.
 *
 * API credentials are resolved in this priority:
 *   1. $overrides['api_id'] / $overrides['api_hash'] (emergency manual override)
 *   2. Local cache (storage/api_credentials.json, created after first WordPress fetch)
 *   3. WordPress REST API fetch (only when $botToken is provided for new login)
 *
 * Output buffering suppresses MadelineProto's direct-echo warnings (polyfill.php).
 * Stale lock files are cleaned before construction to prevent 30-second lock contention.
 *
 * @param string|null $botToken Optional bot token for initial login / re-login.
 * @param array       $overrides Optional api_id/api_hash overrides (emergency only).
 * @param string      $targetBotId Optional target bot_id to select dedicated session directory.
 * @return array [MadelineProto|null, string|null error]
 */
function fd_boot_madeline(?string $botToken = null, array $overrides = [], string $targetBotId = ''): array
{
    // ── Suppress direct echo from polyfill.php ──────────────────────────────
    // polyfill.php is loaded as a Composer autoload file (autoload_files.php),
    // which means it executes during vendor/autoload.php require. Its line 10
    // echoes "WARNING: MadelineProto runs around 10x slower on windows..."
    // directly to stdout on every request. We must capture output buffering
    // BEFORE any autoload-triggering call to prevent this from corrupting JSON.
    unset($_GET['MadelineSelfRestart']);

    // Hook global error handler to safely intercept Windows stream_socket_server() notices
    // without requiring manual modifications to vendor/ files across Composer updates.
    $prevErrorHandler = set_error_handler(static function (int $errno, string $errstr, ?string $errfile = null, ?int $errline = null) use (&$prevErrorHandler): bool {
        if (
            str_contains($errstr, 'The operation completed successfully')
            || (is_string($errfile) && str_contains($errfile, DIRECTORY_SEPARATOR . 'danog' . DIRECTORY_SEPARATOR . 'ipc'))
        ) {
            return true; // Suppress false-positive Windows socket notices
        }
        if (is_callable($prevErrorHandler)) {
            return (bool) $prevErrorHandler($errno, $errstr, $errfile, $errline);
        }
        return false;
    });

    $sessionPath = fd_get_bot_session_path($targetBotId);
    $sessionDir = dirname($sessionPath);

    // During a fresh bot login (token provided, no session yet) force a full
    // MadelineProto boot instead of trying to start an IPC server, which fails
    // under FrankenPHP's short-lived requests. getSlow() (patched) checks this
    // global. After login the session exists and IPC client connect is used.
    $GLOBALS['FD_FORCE_FULL_BOOT'] = ($botToken !== null && $botToken !== '')
        && !(is_dir($sessionPath) || is_file($sessionPath));

    $bootObLevel = ob_get_level();
    $bootEntryObLevel = $bootObLevel;
    ob_start();
    fd_log('fd_boot_madeline entry', [
        'ob_level' => $bootObLevel,
        'bot_token_provided' => $botToken !== null && $botToken !== '',
        'target_bot_id' => $targetBotId,
        'session_path' => $sessionPath,
        'session_exists' => is_dir($sessionPath) || is_file($sessionPath),
    ]);

    // Check if MadelineProto is already loaded (e.g., via routing-level pre-load).
    // We use class_exists without the second parameter here to trigger the
    // Composer autoloader to actually load the class file if needed.
    if (!class_exists('\\danog\\MadelineProto\\API')) {
        // Try loading the autoloader if not already done
        if (!fd_ensure_autoload()) {
            while (ob_get_level() > $bootObLevel) {
                ob_end_clean();
            }
            return [null, 'MadelineProto autoload file was not found. Run composer install first.'];
        }

        // After loading autoload, check again — this time class_exists triggers
        // the freshly registered Composer autoloader to load the class.
        if (!class_exists('\\danog\\MadelineProto\\API')) {
            while (ob_get_level() > $bootObLevel) {
                ob_end_clean();
            }
            return [null, 'MadelineProto class is not available after loading autoload.'];
        }
    }

    // ── Resolve api_id / api_hash ─────────────────────────────────────────
    // Priority: 1) POST body overrides, 2) local cache, 3) WordPress (new login)
    $apiId = (int) ($overrides['api_id'] ?? 0);
    $apiHash = trim((string) ($overrides['api_hash'] ?? ''));

    if ($apiId === 0 || $apiHash === '') {
        // Check if caller supplied encrypted_credentials from browser-direct handshake
        if (!empty($overrides['encrypted_credentials']) && !empty($overrides['encryption_iv']) && $botToken !== null && $botToken !== '') {
            [$decId, $decHashOrErr] = fd_decrypt_credentials(
                (string) $overrides['encrypted_credentials'],
                (string) $overrides['encryption_iv'],
                $botToken
            );
            if ($decId !== null && $decHashOrErr !== null) {
                $apiId = $decId;
                $apiHash = $decHashOrErr;
                fd_save_cached_api_credentials($apiId, $apiHash);
                fd_log('api credentials decrypted from browser-direct payload', ['api_id' => $apiId]);
            }
        }
    }

    if ($apiId === 0 || $apiHash === '') {
        [$cachedId, $cachedHash] = fd_load_cached_api_credentials();
        if ($cachedId !== null && $cachedHash !== null) {
            $apiId = $cachedId;
            $apiHash = $cachedHash;
        }
    }

    // No overrides and no cache — fetch from WordPress backend directly as fallback
    if (($apiId === 0 || $apiHash === '') && $botToken !== null && $botToken !== '') {
        fd_log('fetching api credentials from wordpress (backend fallback)', []);
        [$wpId, $wpHashOrError] = fd_fetch_credentials_from_wordpress($botToken);
        if ($wpId === null) {
            while (ob_get_level() > $bootObLevel) {
                ob_end_clean();
            }
            return [null, $wpHashOrError];
        }
        $apiId = $wpId;
        $apiHash = $wpHashOrError;
        fd_save_cached_api_credentials($apiId, $apiHash);
        fd_log('api credentials cached from wordpress', ['api_id' => $apiId]);
    }

    if ($apiId === 0 || $apiHash === '') {
        while (ob_get_level() > $bootObLevel) {
            ob_end_clean();
        }
        return [null, 'No API credentials available. Login via the settings page to fetch from WordPress.'];
    }

    if (!is_dir($sessionDir)) {
        @mkdir($sessionDir, 0777, true);
    }

    // ── Ensure a persistent IPC worker is running ────────────────────────────
    // Under FrankenPHP, workers spawned from within a request die when the
    // request ends, so MadelineProto's internal auto-spawn is unreliable. If a
    // session already exists, spawn a detached background IPC worker now so the
    // new API() below connects to it as an IPC client (~40-60ms) instead of doing
    // a slow full direct-mode boot. Skip during fresh login (no session yet).
    if (($botToken === null || $botToken === '') && (is_dir($sessionPath) || is_file($sessionPath))) {
        fd_ensure_ipc_worker($sessionPath);
    }

    $settings = new \danog\MadelineProto\Settings();
    $settings->getAppInfo()
        ->setApiId($apiId)
        ->setApiHash($apiHash);
    $settings->getLogger()->setLevel(\danog\MadelineProto\Logger::NOTICE);

    // ── Retry construction loop ───────────────────────────────────────────────
    // Under FrankenPHP, multiple workers service requests concurrently.
    // MadelineProto's AsyncTools::flock() uses touch() to create the lock file
    // only when it doesn't exist. If we delete the lock file in cleanup, every
    // worker races to touch() it, causing "Permission denied" on Windows.
    //
    // The correct approach: NEVER delete the lock file. Let it exist permanently.
    // MadelineProto's flock() with LOCK_NB + polling handles contention between
    // workers naturally (100ms poll intervals, up to 30-second timeout).
    //
    // We still clean /lightState.php.lock and /safe.php.lock (non-lock artifacts
    // from stale IPC sessions), but /lock is left alone.
    $lastError = null;
    for ($bootAttempt = 0; $bootAttempt < 3; $bootAttempt++) {
        try {
            // Clean stale state files (NOT /lock — that should persist to
            // prevent concurrent touch() races under FrankenPHP).
            // Also avoid deleting active .lock files during active downloads.
            if (is_dir($sessionPath)) {
                foreach (['/lightState.php.lock', '/safe.php.lock'] as $lockName) {
                    $lockPath = $sessionPath . $lockName;
                    if (is_file($lockPath)) {
                        $mtime = (int) filemtime($lockPath);
                        // Only remove if stale for more than 45 seconds to not break concurrent operations
                        if ((time() - $mtime) > 45) {
                            @unlink($lockPath);
                        }
                    }
                }
            }

            $t0 = microtime(true);
            $madeline = new \danog\MadelineProto\API($sessionPath, $settings);
            $t1 = microtime(true);
            fd_log('madeline construction', ['ms' => round(($t1 - $t0) * 1000), 'attempt' => $bootAttempt + 1]);

            // Try to resume existing session first.
            // The constructor already deserializes the session if present and logs
            // in via connectToMadelineProto(). We only need getSelf() to verify.
            // MadelineProto v8+ stores sessions as directories, so check both.
            if (is_dir($sessionPath) || is_file($sessionPath)) {
                try {
                    $self = $madeline->getSelf();
                    if ($self && !empty($self['id'])) {
                        fd_save_session_meta(
                            (string) $self['id'],
                            (string) ($self['username'] ?? ''),
                            (string) ($self['first_name'] ?? '')
                        );
                        fd_log('session resumed', [
                            'bot_id' => $self['id'],
                            'elapsed_ms' => round((microtime(true) - $t0) * 1000),
                            'attempt' => $bootAttempt + 1,
                        ]);
                        // Disable background update polling / event handling since we only stream media
                        if (method_exists($madeline, 'setNoop')) {
                            try {
                                $madeline->setNoop();
                            } catch (Throwable $_t) {
                            }
                        }
                        while (ob_get_level() > $bootObLevel) {
                            ob_end_clean();
                        }
                        return [$madeline, null];
                    }
                } catch (Throwable $throwable) {
                    fd_log('existing session invalid, will re-login', [
                        'error' => $throwable->getMessage(),
                        'attempt' => $bootAttempt + 1,
                    ]);
                }
            }

            // No valid session — attempt bot login if token is provided
            if ($botToken !== null && $botToken !== '') {
                $t2 = microtime(true);
                $madeline->botLogin($botToken);
                $t3 = microtime(true);
                fd_log('botLogin completed', ['ms' => round(($t3 - $t2) * 1000)]);

                // Verify login — start() is redundant after constructor + botLogin
                $self = $madeline->getSelf();
                $t4 = microtime(true);
                fd_log('getSelf after login', ['ms' => round(($t4 - $t3) * 1000)]);

                if ($self && !empty($self['id'])) {
                    fd_save_session_meta(
                        (string) $self['id'],
                        (string) ($self['username'] ?? ''),
                        (string) ($self['first_name'] ?? '')
                    );
                    // Disable background update polling / event handling since we only stream media
                    if (method_exists($madeline, 'setNoop')) {
                        try {
                            $madeline->setNoop();
                        } catch (Throwable $_t) {
                        }
                    }
                    // Login succeeded — spawn a detached IPC worker for this bot so
                    // subsequent requests connect via IPC instead of full boots.
                    $newSessionPath = fd_get_bot_session_path((string) $self['id']);
                    if (is_dir($newSessionPath) || is_file($newSessionPath)) {
                        fd_ensure_ipc_worker($newSessionPath);
                    }
                    while (ob_get_level() > $bootObLevel) {
                        ob_end_clean();
                    }
                    return [$madeline, null];
                }

                while (ob_get_level() > $bootObLevel) {
                    ob_end_clean();
                }
                return [null, 'botLogin completed but getSelf returned no valid identity.'];
            }

            // No session and no token — discard buffered output
            while (ob_get_level() > $bootObLevel) {
                ob_end_clean();
            }
            return [null, 'No valid session. Call /api/botlogin to authenticate.'];
        } catch (Throwable $throwable) {
            // Clean up output buffer on exception
            while (ob_get_level() > $bootObLevel) {
                ob_end_clean();
            }
            // Log the error and retry if this wasn't the last attempt
            $lastError = $throwable->getMessage();
            fd_log('madeline boot attempt failed', [
                'error' => $lastError,
                'attempt' => $bootAttempt + 1,
            ]);
            // Small delay before retrying
            if ($bootAttempt < 2) {
                usleep(500000); // 500ms
            }
            // Continue to next retry attempt
        }
    }

    // All retry attempts exhausted
    while (ob_get_level() > $bootObLevel) {
        ob_end_clean();
    }
    return [null, $lastError ?? 'Could not boot MadelineProto after 3 attempts.'];
}

/**
 * Safely delete a session directory with Windows lock-handling and retries.
 */
function fd_clear_session_directory(string $sessionPath): void
{
    if (is_dir($sessionPath)) {
        $deleted = false;
        for ($attempt = 0; $attempt < 3; $attempt++) {
            try {
                $files = new RecursiveIteratorIterator(
                    new RecursiveDirectoryIterator($sessionPath, RecursiveDirectoryIterator::SKIP_DOTS),
                    RecursiveIteratorIterator::CHILD_FIRST
                );
                foreach ($files as $fileinfo) {
                    if ($fileinfo->isDir()) {
                        @rmdir($fileinfo->getRealPath());
                    } else {
                        @unlink($fileinfo->getRealPath());
                    }
                }
                if (@rmdir($sessionPath)) {
                    $deleted = true;
                    break;
                }
            } catch (\Throwable $e) {
            }
            if ($attempt < 2) {
                usleep(200000); // 200ms
            }
        }

        if (!$deleted) {
            $tempName = $sessionPath . '.obsolete.' . getmypid() . '.' . time();
            $renamed = false;
            try {
                $renamed = @rename($sessionPath, $tempName);
            } catch (\Throwable $e) {
                $renamed = false;
            }
            if ($renamed) {
                try {
                    $files = new RecursiveIteratorIterator(
                        new RecursiveDirectoryIterator($tempName, RecursiveDirectoryIterator::SKIP_DOTS),
                        RecursiveIteratorIterator::CHILD_FIRST
                    );
                    foreach ($files as $fileinfo) {
                        if ($fileinfo->isDir()) {
                            @rmdir($fileinfo->getRealPath());
                        } else {
                            @unlink($fileinfo->getRealPath());
                        }
                    }
                    @rmdir($tempName);
                } catch (\Throwable $e) {
                }
            }
        }
    } elseif (is_file($sessionPath)) {
        @unlink($sessionPath);
    }

    $staleLock = $sessionPath . '.lock';
    if (is_file($staleLock)) {
        @unlink($staleLock);
    }
}

/**
 * Clear MadelineProto session files from storage.
 */
function fd_clear_session(string $botId = ''): void
{
    if ($botId !== '') {
        $path = fd_get_bot_session_path($botId);
        fd_clear_session_directory(is_dir($path) ? $path : dirname($path));
        return;
    }

    $sessionPath = FD_SESSION_PATH;
    fd_clear_session_directory($sessionPath);

    // Clear all partitioned bot sessions in storage/sessions/
    $sessionsDir = fd_storage_path('storage/sessions');
    if (is_dir($sessionsDir)) {
        fd_clear_session_directory($sessionsDir);
    }

    // Clear bot pool file
    if (is_file(FD_BOT_POOL_PATH)) {
        @unlink(FD_BOT_POOL_PATH);
    }

    // Clear cached API credentials — forces re-fetch from WordPress on next login
    $credsCache = fd_storage_path('storage/api_credentials.json');
    if (is_file($credsCache)) {
        @unlink($credsCache);
    }

    // Clear the stored API secret — forces fresh secret on next login
    fd_clear_api_secret();
    fd_clear_bot_id();
    fd_clear_session_meta();

    // Clear resolve cache files
    $storageDir = fd_get_storage_dir();
    $cacheFiles = glob($storageDir . '/resolve_cache_*.json');
    if ($cacheFiles) {
        foreach ($cacheFiles as $cf) {
            @unlink($cf);
        }
    }
}

/**
 * Check the application version against the WordPress minimum required version.
 *
 * Fetches min_version from FD_WP_VERSION_URL and caches the result for
 * FD_VERSION_CACHE_TTL seconds. Uses fd_http_json() which automatically
 * includes the X-API-Secret header. On failure, returns a safe default
 * (update_needed=false) so the app continues to work if WordPress is unreachable.
 *
 * @return array{ok:bool,update_needed:bool,current_version:string,minimum_version:string,update_url:string,release_notes:string}
 */
/**
 * In-memory version state updated from response headers during requests.
 */
function fd_update_version_state(array $headers): void
{
    global $fd_version_state;
    if (!is_array($fd_version_state)) {
        $fd_version_state = [
            'min_version' => '',
            'update_url' => '',
            'update_required' => false,
        ];
    }
    if (isset($headers['x-min-version'])) {
        $fd_version_state['min_version'] = (string) $headers['x-min-version'];
    }
    if (isset($headers['x-update-url'])) {
        $fd_version_state['update_url'] = (string) $headers['x-update-url'];
    }
    if (isset($headers['x-update-required'])) {
        $fd_version_state['update_required'] = (string) $headers['x-update-required'] === '1';
    }
    if (isset($headers['x-sponsor-name'])) {
        $fd_version_state['sponsor_name'] = rawurldecode((string) $headers['x-sponsor-name']);
    }
    if (isset($headers['x-sponsor-desc'])) {
        $fd_version_state['sponsor_desc'] = rawurldecode((string) $headers['x-sponsor-desc']);
    }
    if (isset($headers['x-sponsor-url'])) {
        $fd_version_state['sponsor_url'] = (string) $headers['x-sponsor-url'];
    }
}

/**
 * Check the application version against the WordPress minimum required version.
 *
 * Uses the in-memory response header state captured on live HTTP requests,
 * with a fallback to the /version endpoint if not yet initialized.
 *
 * @return array{ok:bool,update_needed:bool,current_version:string,minimum_version:string,update_url:string,release_notes:string}
 */
function fd_check_version(): array
{
    global $fd_version_state;

    $current = FD_APP_VERSION;

    // If we haven't received version headers yet from a previous request, fetch once
    if (empty($fd_version_state['min_version']) || !isset($fd_version_state['sponsor_url'])) {
        $response = fd_http_json(FD_WP_VERSION_URL . '?t=' . time(), [], 'GET', 3);
        if (!empty($response['ok'])) {
            $minVersion = (string) ($response['min_version'] ?? '');
            $updateUrl = (string) ($response['update_url'] ?? '');
            $updateNeeded = $minVersion !== '' && version_compare($current, $minVersion, '<');
            $fd_version_state['min_version'] = $minVersion;
            $fd_version_state['update_url'] = $updateUrl;
            $fd_version_state['update_required'] = $updateNeeded;
            if (isset($response['sponsor']) && is_array($response['sponsor'])) {
                $fd_version_state['sponsor_name'] = (string) ($response['sponsor']['name'] ?? '');
                $fd_version_state['sponsor_desc'] = (string) ($response['sponsor']['description'] ?? '');
                $fd_version_state['sponsor_url'] = (string) ($response['sponsor']['url'] ?? '');
            }
            return [
                'ok' => true,
                'update_needed' => $updateNeeded,
                'current_version' => $current,
                'minimum_version' => $minVersion,
                'update_url' => $updateUrl,
                'release_notes' => (string) ($response['release_notes'] ?? ''),
                'sponsor' => [
                    'name' => (string) ($fd_version_state['sponsor_name'] ?? ''),
                    'description' => (string) ($fd_version_state['sponsor_desc'] ?? ''),
                    'url' => (string) ($fd_version_state['sponsor_url'] ?? ''),
                ],
            ];
        }
    }

    $minVersion = (string) ($fd_version_state['min_version'] ?? '');
    $updateUrl = (string) ($fd_version_state['update_url'] ?? '');
    $updateNeeded = !empty($fd_version_state['update_required']) || ($minVersion !== '' && version_compare($current, $minVersion, '<'));

    return [
        'ok' => true,
        'update_needed' => $updateNeeded,
        'current_version' => $current,
        'minimum_version' => $minVersion,
        'update_url' => $updateUrl,
        'release_notes' => '',
        'sponsor' => [
            'name' => (string) ($fd_version_state['sponsor_name'] ?? ''),
            'description' => (string) ($fd_version_state['sponsor_desc'] ?? ''),
            'url' => (string) ($fd_version_state['sponsor_url'] ?? ''),
        ],
    ];
}

// ─── Stremio & Nuvio Helpers ─────────────────────────────────────────────────

function fd_format_bytes(int $bytes, int $precision = 1): string
{
    if ($bytes <= 0) return '0 B';
    $units = ['B', 'KB', 'MB', 'GB', 'TB'];
    $pow = min((int) floor(log($bytes, 1024)), count($units) - 1);
    return round($bytes / pow(1024, $pow), $precision) . ' ' . $units[$pow];
}

/**
 * Clean Telegram file title from spam prefixes, channel promos, and bot forwarding artifacts.
 */
function fd_clean_media_title(string $title): string
{
    if ($title === '') return '';
    $t = $title;
    // Strip emojis
    $t = preg_replace('/[\x{1F300}-\x{1F9FF}\x{2600}-\x{26FF}\x{2700}-\x{27BF}]/u', ' ', $t);
    // Strip website / streaming host prefixes
    $t = preg_replace('/^(?:on9[._\s]stream[._\s]+|stream[._\s]+|www\.[a-z0-9.-]+\.[a-z]{2,}[._\s]+)/i', '', $t);
    // Strip forwarded and join channel spam
    $t = preg_replace('/forwarded[._\s]from.*$/i', '', $t);
    $t = preg_replace('/(?:Join[._\s]Channel|Join[._\s]Group|Join[._\s]us|Join[._\s]@).*$/i', '', $t);
    $t = preg_replace('/kumpulan[._\s]drama.*$/i', '', $t);
    $t = preg_replace('/Please[.\s]Don[\x27\x22.]?t[.\s]Forward.*$/i', '', $t);
    $t = preg_replace('/(?:Req\.By|Request\.By|File\.Request\.By|Requested\.By).*$/i', '', $t);
    $t = preg_replace('/(?:Channel\.Terbaik\.Anda|Filemku\.bot|LayarAsiaBot|filembot).*$/i', '', $t);
    $t = preg_replace('/(?:https?:\/\/|httpst\.me|https?\.?t\.me|\bt\.me\/)[\w.\/\?=&_-]*/i', '', $t);
    $t = preg_replace('/[._\s]+Watch[._\s]Hd[._\s]Video[._\s]Online.*$/i', '', $t);
    $t = preg_replace('/(?:^|[.\s_#-]+)Open[.\s_-]*Mini[.\s_-]*App.*$/iu', '', $t);
    $t = preg_replace('/(?:[.\s_-]*\d+(?:[.,]\d+)?[.\s_-]*(?:MB|GB|KB|TB))+(?:[.\s_-]*https)?(?:[.\s_-]*Open[.\s_-]*Mini[.\s_-]*App)?$/iu', '', $t);

    // Trim trailing and leading punctuation/whitespace
    $t = trim($t, " ._-=\t\n\r\0\x0B");
    return $t;
}

/**
 * Detect if a media filename / title represents a split file part (e.g. part001, part01, .001, etc.).
 * Returns array: ['is_part' => bool, 'base_key' => string, 'part_num' => int, 'total_parts' => int]
 */
function fd_extract_split_part_info(string $filename, string $caption = ''): array
{
    $f = trim($filename);
    $cap = trim($caption);

    // Common patterns:
    // 1) .part001.mkv, .part01.mp4, -part.001/005, part01.mp4, part 001, etc.
    // 2) .001, .002 (raw split chunk extensions)
    // 3) Title.mkv.001 or Title.mp4.001
    // 4) part001 of 005 or part.001/005 in caption
    $partNum = 0;
    $totalParts = 0;
    $matched = false;
    $cleanBase = $f;

    if (preg_match('/[._\s-]part[._\s-]*0*(\d{1,4})(?:[._\s\/-]+(?:of[._\s-]+)?0*(\d{1,4}))?/i', $f, $m)) {
        $partNum = (int) $m[1];
        if (!empty($m[2])) $totalParts = (int) $m[2];
        $matched = true;
        // Strip the part marker from base name
        $cleanBase = preg_replace('/[._\s-]part[._\s-]*0*\d{1,4}(?:[._\s\/-]+(?:of[._\s-]+)?0*\d{1,4})?/i', '', $f);
    } elseif (preg_match('/[._\s-]0*(\d{1,3})\.(mp4|mkv|avi|webm)$/i', $f, $m) && !preg_match('/\b(2160p|1080p|720p|480p|360p)\b/i', $m[0])) {
        // e.g. Something.001.mp4 or Something-001.mkv
        $partNum = (int) $m[1];
        $matched = true;
        $cleanBase = preg_replace('/[._\s-]0*' . $m[1] . '\.' . $m[2] . '$/i', '.' . $m[2], $f);
    } elseif (preg_match('/\.(?:mp4|mkv|avi|webm)\.0*(\d{1,4})$/i', $f, $m)) {
        // e.g. movie.mkv.001
        $partNum = (int) $m[1];
        $matched = true;
        $cleanBase = preg_replace('/\.0*' . $m[1] . '$/i', '', $f);
    }

    // Check caption for part total e.g. "part.001/005"
    if ($matched && $totalParts === 0 && $cap !== '') {
        if (preg_match('/part[._\s-]*0*' . $partNum . '\s*[\/|of]\s*0*(\d{1,4})/i', $cap, $cm)) {
            $totalParts = (int) $cm[1];
        }
    }

    if (!$matched || $partNum <= 0) {
        return ['is_part' => false, 'base_key' => '', 'part_num' => 0, 'total_parts' => 0];
    }

    // Normalize base key for matching parts belonging to the same split group
    $baseKey = strtolower(trim(preg_replace('/[^\p{L}\p{N}]+/u', '.', $cleanBase), '.'));

    return [
        'is_part' => true,
        'base_key' => $baseKey,
        'clean_base' => $cleanBase,
        'part_num' => $partNum,
        'total_parts' => $totalParts,
    ];
}

/**
 * Inspect a list of file items and sort/label split parts (part001, part002, ...)
 * as sequential separate streams with clean titles.
 */
function fd_group_split_parts(array $files): array
{
    // Sort files so that if multi-part files exist, they appear in ascending part order
    usort($files, function ($a, $b) {
        $aTitle = (string) ($a['title'] ?? '');
        $bTitle = (string) ($b['title'] ?? '');
        $aInfo = fd_extract_split_part_info($aTitle, (string) ($a['caption'] ?? ''));
        $bInfo = fd_extract_split_part_info($bTitle, (string) ($b['caption'] ?? ''));

        if ($aInfo['is_part'] && $bInfo['is_part'] && $aInfo['base_key'] === $bInfo['base_key']) {
            return $aInfo['part_num'] <=> $bInfo['part_num'];
        }
        return 0;
    });

    foreach ($files as &$f) {
        $info = fd_extract_split_part_info((string) ($f['title'] ?? ''), (string) ($f['caption'] ?? ''));
        if ($info['is_part']) {
            $f['is_split_part'] = true;
            $f['part_num'] = $info['part_num'];
            $f['total_parts'] = $info['total_parts'];
            $f['clean_base'] = $info['clean_base'];
        }
    }
    unset($f);

    return $files;
}


/**
 * Format clean title for Stremio / Nuvio metadata.
 * Keeps the WordPress "• TvSeries" / "• Movie" suffix intact (reverted per user
 * request) but still strips other media spam via fd_clean_media_title().
 */
function fd_clean_post_title(string $title): string
{
    $t = fd_clean_media_title($title);
    $t = trim($t, " \t\n\r\0\x0B");
    return $t !== '' ? $t : $title;
}

/**
 * Extract 4-digit release year from post title, date, or content.
 */
function fd_extract_release_year(string $title, string $date = ''): string
{
    if (preg_match('/\b(19\d{2}|20\d{2})\b/', $title, $m)) {
        return $m[1];
    }
    if ($date !== '' && preg_match('/\b(19\d{2}|20\d{2})\b/', $date, $m)) {
        return $m[1];
    }
    return '';
}

/**
 * Extract clean display genres from WordPress post data.
 * Filters out internal CMS categories like "Telegram", "TV Shows", "Movies"
 * and prioritizes real genre tags (e.g. "Comedy", "Drama", "Action", "Romance").
 */
function fd_extract_post_genres(array $post): array
{
    $tags = (array) ($post['tags'] ?? []);
    $cats = (array) ($post['categories'] ?? []);
    $rawList = array_merge($tags, $cats);

    $cmsBlacklist = ['telegram', 'tv shows', 'tv show', 'tvseries', 'movies', 'movie', 'uncategorized'];
    $seen = [];
    $genres = [];

    foreach ($rawList as $item) {
        $clean = trim(html_entity_decode((string) $item, ENT_QUOTES | ENT_HTML5, 'UTF-8'));
        if ($clean === '') continue;
        // Split combined genres on "&" (e.g. "Action & Adventure" -> "Action", "Adventure")
        $parts = preg_split('/\s*&\s*/', $clean);
        foreach ($parts as $part) {
            $part = trim($part);
            if ($part === '') continue;
            $lower = strtolower($part);
            if (in_array($lower, $cmsBlacklist, true)) continue;
            if (isset($seen[$lower])) continue;
            $seen[$lower] = true;
            $genres[] = $part;
        }
    }

    // If all tags were filtered out, fallback to cleaned categories or general default
    if (empty($genres)) {
        foreach ($cats as $c) {
            $clean = trim(html_entity_decode((string) $c, ENT_QUOTES | ENT_HTML5, 'UTF-8'));
            if ($clean !== '' && !isset($seen[strtolower($clean)])) {
                $seen[strtolower($clean)] = true;
                $genres[] = $clean;
            }
        }
    }

    return !empty($genres) ? $genres : ['Drama'];
}

function fd_classify_season_episode(string $title, int $seasonNum = 0, int $episodeNum = 0, string $caption = ''): array
{
    $season = 0;
    $episode = 0;
    $episodeEnd = 0;

    // Clean title from forwarded spam / suffixes before matching
    $title = fd_clean_media_title($title);

    // If filename is generic (e.g. video.2022.08.09... or video.mp4), fallback to caption for parsing
    if ($caption !== '' && preg_match('/^(?:video(?:\.\d+)*|\d+|document|file)\.(?:mp4|mkv|avi|mov|ts|flv)$/i', trim($title))) {
        $firstCaptionLine = trim(explode("\n", $caption)[0]);
        if ($firstCaptionLine !== '') {
            $title = fd_clean_media_title($firstCaptionLine);
        }
    }

    // 1. Explicit SxxExx.Exx (range like S01.E01.E14 or S01E01-E14 or S01E01-14)
    if (preg_match('/(?:^|[^a-z0-9])S(\d{1,2})\s*[ ._-]*E(?:P|PS|PISODE)?\s*[ ._-]*(\d{1,4})\s*(?:[ ._-]+E(?:P|PS|PISODE)?|\s*[-~–—]\s*|\s+(?:to|hingga|sampai)\s+)\s*(\d{1,4})(?:[^a-z0-9]|$)/i', $title, $m)) {
        $season = (int) $m[1];
        $episode = (int) $m[2];
        $episodeEnd = (int) $m[3];
    }
    // 1b. Single SxxExx or Sxx.Exx / SxxEPxx / SxxEpxx in title
    elseif (preg_match('/(?:^|[^a-z0-9])S(\d{1,2})\s*[ ._-]*E(?:P|PS|PISODE)?\s*[ ._-]*(\d{1,4})(?:[^a-z0-9]|$)/i', $title, $m)) {
        $season = (int) $m[1];
        $episode = (int) $m[2];
    }
    // 2. Explicit 1x05 / 01x12
    elseif (preg_match('/(?:^|[^a-z0-9])(\d{1,2})\s*[xX]\s*(\d{1,4})(?![0-9])(?:[^a-z0-9]|$)/', $title, $m)) {
        $season = (int) $m[1];
        $episode = (int) $m[2];
    }
    // 3. Part / Vol / Cour followed by season and episode (e.g. Part 3.01, Vol 2 - 05, Part 1 E27)
    elseif (preg_match('/(?:^|[^a-z0-9])(?:PART|VOL|VOLUME|COUR)\s*[ ._-]*0*(\d{1,2})[ ._-]+(?:E(?:P|PS|PISODE)?\s*[ ._-]*)?0*(\d{1,4})(?=[ ._\-\]\)]|$)/i', $title, $m)) {
        $n2 = (int) $m[2];
        if ($n2 < 1900 || $n2 > 2100) {
            $season = (int) $m[1];
            $episode = $n2;
        }
    }

    // 4. Season / Musim / Part / Cour / Vol keywords
    if ($season === 0) {
        if (preg_match('/(?:^|[^a-z0-9])(?:season|musim)\s*[ ._-]*0*(\d{1,2})(?:[^a-z0-9]|$)/i', $title, $m)) {
            $season = (int) $m[1];
        } elseif (preg_match('/(?:^|[^a-z0-9])(?:PART|VOL|VOLUME|COUR)\s*[ ._-]*0*(\d{1,2})(?=[ ._-]+(?:EP|E|\d))/i', $title, $m)) {
            $season = (int) $m[1];
        } elseif (preg_match('/(?:^|[^a-z0-9])S(\d{1,2})(?=[^a-z0-9]|$)/i', $title, $m)) {
            $season = (int) $m[1];
        }
    }

    // 5. Check explicit EP / Episode range tokens (e.g. EP01-EP14, EP01-14, E01.E14, E01-E14)
    if ($episode === 0) {
        if (preg_match('/(?:^|[^a-z0-9])(?:EP|EPS|EPISODE|EPISOD|E)\s*[ ._-]*0*(\d{1,4})\s*(?:[ ._-]+(?:EP|EPS|EPISODE|EPISOD|E)|\s*[-~–—]\s*|\s+(?:to|hingga|sampai)\s+)\s*0*(\d{1,4})(?:[^a-z0-9]|$)/i', $title, $m)) {
            $n1 = (int) $m[1];
            $n2 = (int) $m[2];
            if ($n1 > 0 && ($n1 < 1900 || $n1 > 2100) && $n2 > 0 && ($n2 < 1900 || $n2 > 2100) && $n2 >= $n1 && ($n2 - $n1) <= 150) {
                $episode = $n1;
                $episodeEnd = $n2;
            }
        }
    }

    // 6. Check explicit EP / Episode / Bahagian tokens in title (e.g. EP27, Episode 05)
    if ($episode === 0) {
        if (preg_match('/(?:^|[^a-z0-9])(?:EP|EPS|EPISODE|EPISOD|BAHAGIAN|BABAK)\s*[ ._-]*0*(\d{1,4})(?:[^a-z0-9]|$)/i', $title, $m)) {
            $n = (int) $m[1];
            if ($n > 0 && ($n < 1900 || $n > 2100)) {
                $episode = $n;
            }
        }
    }

    // 7. Token starting with E followed by digits (e.g. kdg.E01, OLD.E32, e27.end.mp4, DramaDaily.720p...E30.mp4)
    if ($episode === 0) {
        if (preg_match('/(?:^|[^a-z0-9])E[ ._-]*0*(\d{1,4})(?:[^a-z0-9]|$)/i', $title, $m)) {
            $n = (int) $m[1];
            if ($n > 0 && ($n < 1900 || $n > 2100)) {
                $episode = $n;
            }
        }
    }

    // 7. Part / Vol as episode fallback ONLY if season wasn't detected from it
    if ($episode === 0 && $season === 0) {
        if (preg_match('/(?:^|[^a-z0-9])(?:PART|VOL|VOLUME)\s*[ ._-]*0*(\d{1,4})(?:[^a-z0-9]|$)/i', $title, $m)) {
            $n = (int) $m[1];
            if ($n > 0 && ($n < 1900 || $n > 2100)) {
                $episode = $n;
            }
        }
    }

    // 8. Bare numbers without E/EP prefix (e.g. Flying.Up.Without.Disturb.32.480p.mp4, Title.06.720p.mp4, Title - 05.mkv)
    if ($episode === 0) {
        $clean = preg_replace('/\.(mp4|mkv|avi|mov|ts|flv|webm)$/i', '', $title);
        // Strip 4-digit release years (1900-2099) so they don't get misidentified as bare episode numbers
        $cleanWithoutYears = (string) preg_replace('/\b(?:19|20)\d{2}\b/', ' ', $clean);
        // Negative lookbehind (?<![0-9]) prevents matching numbers that are part
        // of audio codecs like "DD5.1.x264" (the "1" in "5.1" must not be treated
        // as an episode number).
        if (preg_match('/(?<![0-9])[ ._\[\(-](\d{1,3})[ ._\]\)-]+(?:2160p|1080p|720p|480p|360p|4k|uhd|fhd|hd|sd|web|bluray|hdtv|malaysub|end|final|x264|x265|hevc|aac)/i', $cleanWithoutYears, $m)) {
            $n = (int) $m[1];
            if ($n > 0 && ($n < 1900 || $n > 2100)) {
                $episode = $n;
            }
        } elseif (preg_match('/(?<![0-9])[ ._\[\(-](\d{1,3})[ ._\]\)]*$/', $cleanWithoutYears, $m)) {
            $n = (int) $m[1];
            if ($n > 0 && ($n < 1900 || $n > 2100)) {
                $episode = $n;
            }
        }
    }

    // Fallbacks to DB media-rank columns if not found in title
    if ($season === 0 && $seasonNum > 0) {
        // Only trust DB season if title didn't find a conflicting uncorroborated episode
        if ($episode === 0 || $episodeNum === $episode) {
            $season = $seasonNum;
        }
    }
    if ($episode === 0 && $episodeNum > 0) {
        $episode = $episodeNum;
    }

    if ($season === 0) {
        $season = 1;
    }

    return ['season' => $season, 'episode' => $episode, 'episode_end' => $episodeEnd];
}

function fd_file_matches_episode(array $parsed, int $targetSeason, int $targetEpisode): bool
{
    $s = (int) ($parsed['season'] ?? 0);
    $e = (int) ($parsed['episode'] ?? 0);
    $eEnd = (int) ($parsed['episode_end'] ?? 0);

    if ($s !== $targetSeason) {
        return false;
    }

    // Combined pack (E01-E14): keep it on episodes inside the range.
    if ($e > 0 && $eEnd >= $e) {
        return $targetEpisode >= $e && $targetEpisode <= $eEnd;
    }

    // Unclassified / E0 files must not leak onto episode 1.
    if ($e <= 0) {
        return false;
    }

    return $e === $targetEpisode;
}

function fd_fetch_stream_ajax(string $action, array $params = []): array
{
    $streamAction = 'stream_' . $action;
    $wpUrl = defined('FD_WP_AJAX_URL') ? FD_WP_AJAX_URL : 'https://pencarimovie.com/wp-admin/admin-ajax.php';
    $queryParams = $params;
    $queryParams['action'] = $streamAction;
    if (empty($queryParams['bot_id'])) {
        $activeBotId = fd_get_bot_id();
        if ($activeBotId !== '') {
            $queryParams['bot_id'] = $activeBotId;
        }
    }
    // Cloudflare country header detection: pencarimovie.com automatically receives
    // $_SERVER['HTTP_CF_IPCOUNTRY'] from Cloudflare. If client passes ?country= or CF header exists, forward it.
    if (empty($queryParams['country']) && !empty($_SERVER['HTTP_CF_IPCOUNTRY'])) {
        $queryParams['country'] = sanitize_text_field($_SERVER['HTTP_CF_IPCOUNTRY']);
    }
    $fullWpUrl = $wpUrl . '?' . http_build_query($queryParams);

    // Direct cURL fetch
    try {
        $body = fd_http_get_contents($fullWpUrl, [
            'method' => 'GET',
            'headers' => ['X-Requested-With: XMLHttpRequest'],
            'timeout' => 12,
        ]);
        if (is_string($body) && $body !== '') {
            $decoded = json_decode($body, true);
            if (is_array($decoded) && isset($decoded['success']) && $decoded['success']) {
                return (array) ($decoded['data'] ?? []);
            }
            if (is_array($decoded)) {
                return $decoded;
            }
        }
        return [];
    } catch (\Throwable $e) {
        fd_log('stremio wp ajax fetch failed', ['action' => $streamAction, 'error' => $e->getMessage()]);
        return [];
    }
}

/**
 * Page post_files through Manticore OPTION scroll instead of one 5000-row dump.
 * Each WP request is a small page; we stop at $maxFiles or when a page is short.
 *
 * Optional $opts['until_unique_episodes'] keeps one file per S/E and keeps
 * paging past quality-duplicate pages so earlier seasons are not truncated.
 */
function fd_fetch_post_files_paged(int $postId, array $opts = []): array
{
    $pageSize = max(1, min((int) ($opts['page_size'] ?? 100), 200));
    $maxFiles = max($pageSize, (int) ($opts['max_files'] ?? 300));
    $season = max(0, (int) ($opts['season'] ?? 0));
    $episode = max(0, (int) ($opts['episode'] ?? 0));
    $search = trim((string) ($opts['search'] ?? ''));
    $filter = $opts['filter'] ?? null;
    $untilUnique = !empty($opts['until_unique_episodes']);
    $maxUnique = max(1, (int) ($opts['max_unique_episodes'] ?? 400));
    $limit = $untilUnique ? $maxUnique : $maxFiles;
    $defaultPages = $untilUnique ? 8 : ((int) ceil($maxFiles / $pageSize) + 2);
    $maxPages = max(1, (int) ($opts['max_pages'] ?? $defaultPages));
    $staleLimit = max(1, (int) ($opts['stale_pages'] ?? ($untilUnique ? 4 : 2)));

    $all = [];
    $seen = [];
    $seenEps = [];
    $stalePages = 0;
    $offset = 0;

    for ($page = 0; $page < $maxPages; $page++) {
        if ($search !== '') {
            $params = [
                'search' => $search,
                'limit' => $pageSize,
                'offset' => $offset,
            ];
            $res = fd_fetch_stream_ajax('search_files', $params);
        } else {
            $params = [
                'post_id' => $postId,
                'limit' => $pageSize,
                'offset' => $offset,
            ];
            if ($season > 0) {
                $params['season'] = $season;
            }
            if ($episode > 0) {
                $params['episode'] = $episode;
            }
            $res = fd_fetch_stream_ajax('post_files', $params);
        }
        $files = (array) ($res['files'] ?? []);
        if ($files === []) {
            break;
        }

        $newEpsThisPage = 0;
        foreach ($files as $file) {
            $code = (string) ($file['short_code'] ?? '');
            if ($code === '' || isset($seen[$code])) {
                continue;
            }
            if (is_callable($filter) && !$filter($file)) {
                continue;
            }

            if ($untilUnique) {
                $parsed = fd_classify_season_episode(
                    (string) ($file['title'] ?? ''),
                    (int) ($file['season_num'] ?? 0),
                    (int) ($file['episode_num'] ?? 0),
                    (string) ($file['caption'] ?? '')
                );
                $epKey = $parsed['season'] . '_' . $parsed['episode'];
                if (isset($seenEps[$epKey])) {
                    $seen[$code] = true;
                    continue;
                }
                $seenEps[$epKey] = true;
                $newEpsThisPage++;
            }

            $seen[$code] = true;
            $all[] = $file;

            if (count($all) >= $limit) {
                return $all;
            }
        }

        if ($untilUnique) {
            if ($newEpsThisPage === 0) {
                $stalePages++;
            } else {
                $stalePages = 0;
            }
            if ($stalePages >= $staleLimit) {
                break;
            }
        }

        $returned = count($files);
        $hasMore = !empty($res['has_more']) || $returned >= $pageSize;
        if (!$hasMore) {
            break;
        }
        $offset += $pageSize;
    }

    return $all;
}

/**
 * One representative file for an exact SxxExx hit.
 * Live WP MATCH only appends SxxExx when both season and episode are set.
 */
function fd_fetch_one_episode_file(int $postId, int $season, int $episode): ?array
{
    if ($season <= 0 || $episode <= 0) {
        return null;
    }

    $res = fd_fetch_stream_ajax('post_files', [
        'post_id' => $postId,
        'limit' => 8,
        'offset' => 0,
        'season' => $season,
        'episode' => $episode,
    ]);

    foreach ((array) ($res['files'] ?? []) as $file) {
        $parsed = fd_classify_season_episode(
            (string) ($file['title'] ?? ''),
            (int) ($file['season_num'] ?? 0),
            (int) ($file['episode_num'] ?? 0),
            (string) ($file['caption'] ?? '')
        );
        if ((int) $parsed['season'] === $season && (int) $parsed['episode'] === $episode) {
            return $file;
        }
    }

    return null;
}

function fd_stream_keyword_from_post_title(string $title): string
{
    $keyword = trim((string) preg_replace('/[\x00-\x1F]+/u', ' ', $title));
    $keyword = trim((string) preg_replace('/\s*[•·]\s*.+$/u', '', $keyword));
    $keyword = trim((string) preg_replace('/\s*\(\d{4}\)\s*$/u', '', $keyword));
    $keyword = trim((string) preg_replace('/\s+\d{4}\s*$/u', '', $keyword));
    $keyword = trim((string) preg_replace('/\b(?:tvseries|tv\s*series)\b/iu', '', $keyword));
    return trim((string) preg_replace('/\s+/', ' ', $keyword));
}

/**
 * Generate search keyword variants for a title (handling apostrophe-s vs s vs omitted s, e.g. "Princess's" vs "Princess's" vs "Princess s" vs "Princess").
 */
function fd_stream_keyword_variants(string $keyword): array
{
    $variants = [$keyword];

    // 1. Replace apostrophe with space ("Princess's" -> "Princess s", "Grey's" -> "Grey s")
    $withSpace = trim((string) preg_replace("/['’`]/u", ' ', $keyword));
    $withSpace = trim((string) preg_replace('/\s+/', ' ', $withSpace));
    if ($withSpace !== '' && $withSpace !== $keyword) {
        $variants[] = $withSpace;
    }

    // 2. Remove apostrophe completely ("Princess's" -> "Princesss", "Grey's" -> "Greys")
    $noApos = trim((string) preg_replace("/['’`]/u", '', $keyword));
    $noApos = trim((string) preg_replace('/\s+/', ' ', $noApos));
    if ($noApos !== '' && !in_array($noApos, $variants, true)) {
        $variants[] = $noApos;
    }

    // 3. Remove 's / s' possessive altogether ("Princess's" -> "Princess", "Grey's" -> "Grey")
    $noPossessive = trim((string) preg_replace("/(?:['’`]s|s['’`]|\\bs\\b)/iu", '', $keyword));
    $noPossessive = trim((string) preg_replace('/\s+/', ' ', $noPossessive));
    if ($noPossessive !== '' && !in_array($noPossessive, $variants, true)) {
        $variants[] = $noPossessive;
    }

    return array_values(array_unique($variants));
}

function fd_episode_stream_filter(int $season, int $episode): callable
{
    return static function (array $pf) use ($season, $episode): bool {
        if (empty($pf['short_code'])) {
            return false;
        }
        $parsed = fd_classify_season_episode(
            (string) ($pf['title'] ?? ''),
            (int) ($pf['season_num'] ?? 0),
            (int) ($pf['episode_num'] ?? 0),
            (string) ($pf['caption'] ?? '')
        );
        return fd_file_matches_episode($parsed, $season, $episode);
    };
}

/**
 * Playable files for one series episode only.
 * SxxExx MATCH misses E01-style names; search_files backfills those.
 * Never dump mixed/unfiltered post files onto an episode page.
 */
function fd_fetch_episode_stream_files(int $postId, int $season, int $episode, int $maxFiles = 40): array
{
    if ($postId <= 0 || $season <= 0 || $episode <= 0) {
        return [];
    }

    $filter = fd_episode_stream_filter($season, $episode);
    $all = [];
    $seen = [];
    $add = static function (array $files) use (&$all, &$seen, $filter, $maxFiles): void {
        foreach ($files as $file) {
            if (count($all) >= $maxFiles) {
                return;
            }
            $code = (string) ($file['short_code'] ?? '');
            if ($code === '' || isset($seen[$code])) {
                continue;
            }
            if (!$filter($file)) {
                continue;
            }
            $seen[$code] = true;
            $all[] = $file;
        }
    };

    // 1. First probe post files using exact season & episode parameters (fast MATCH)
    $add(fd_fetch_post_files_paged($postId, [
        'page_size' => 50,
        'max_files' => $maxFiles,
        'season' => $season,
        'episode' => $episode,
        'filter' => $filter,
        'max_pages' => 2,
    ]));

    // 2. Scan all files in the post up to 1000 items with the episode filter
    // (covers posts where episodes lack Sxx or have custom tags like E01 / Ep.1 / nunadrama)
    if (count($all) < $maxFiles) {
        $add(fd_fetch_post_files_paged($postId, [
            'page_size' => 100,
            'max_files' => 1000,
            'max_pages' => 10,
            'filter' => $filter,
        ]));
    }

    // 3. If still needed, probe search_files by title keywords
    if (count($all) < $maxFiles) {
        $postData = fd_fetch_stream_ajax('get_post', ['post_id' => $postId]);
        $post = !empty($postData) && is_array($postData) ? ($postData[0] ?? $postData) : [];
        $keyword = fd_stream_keyword_from_post_title((string) ($post['title'] ?? ''));

        if ($keyword !== '') {
            $kwVariants = fd_stream_keyword_variants($keyword);
            $queries = [];
            foreach ($kwVariants as $kwVar) {
                $queries[] = sprintf('%s S%02dE%02d', $kwVar, $season, $episode);
                $queries[] = sprintf('%s E%02d', $kwVar, $episode);
                $queries[] = sprintf('%s EP%02d', $kwVar, $episode);
                $queries[] = sprintf('E%02d %s', $episode, $kwVar);
                $queries[] = sprintf('EP%02d %s', $episode, $kwVar);
                $queries[] = sprintf('S%02dE%02d %s', $season, $episode, $kwVar);
                if ($episode < 10) {
                    $queries[] = sprintf('%s E%d', $kwVar, $episode);
                    $queries[] = sprintf('E%d %s', $episode, $kwVar);
                }
            }

            foreach (array_values(array_unique($queries)) as $query) {
                if (count($all) >= $maxFiles) {
                    break;
                }
                $res = fd_fetch_stream_ajax('search_files', [
                    'search' => $query,
                    'limit' => 50,
                    'offset' => 0,
                ]);
                $add((array) ($res['files'] ?? []));
            }
        }
    }

    return $all;
}

/**
 * Build a series episode file list without dumping thousands of quality
 * variants. Scans post_files with until_unique_episodes to discover all
 * distinct seasons and episodes, then backfills individual episode files
 * found via keyword search (covers posts whose own files are only
 * "COMBINED" season packs while the real per-episode files live elsewhere
 * in the Telegram database).
 */
function fd_fetch_series_episode_files(int $postId): array
{
    $all = fd_fetch_post_files_paged($postId, [
        'page_size' => 200,
        'max_files' => 600,
        'max_pages' => 3,
        'until_unique_episodes' => true,
        'max_unique_episodes' => 300,
        'stale_pages' => 1,
    ]);

    // If the post's own files are all combined packs (no explicit episode
    // numbers), backfill individual episode files via keyword search so the
    // series shows real per-episode entries (S01E01, S01E02, ...).
    $hasExplicitEp = false;
    foreach ($all as $f) {
        $parsed = fd_classify_season_episode(
            (string) ($f['title'] ?? ''),
            (int) ($f['season_num'] ?? 0),
            (int) ($f['episode_num'] ?? 0),
            (string) ($f['caption'] ?? '')
        );
        if (($parsed['episode'] ?? 0) > 0) {
            $hasExplicitEp = true;
            break;
        }
    }

    if ($hasExplicitEp) {
        return $all;
    }

    // No explicit episodes in the post's own files — search for individual
    // episode files by title keyword (same strategy as the stream handler).
    $postData = fd_fetch_stream_ajax('get_post', ['post_id' => $postId]);
    $post = !empty($postData) && is_array($postData) ? ($postData[0] ?? $postData) : [];
    $keyword = fd_stream_keyword_from_post_title((string) ($post['title'] ?? ''));
    if ($keyword === '') {
        return $all;
    }

    $seen = [];
    foreach ($all as $f) {
        $seen[(string) ($f['short_code'] ?? '')] = true;
    }

    // Single broad Manticore search (limit=1000) then filter locally by
    // season/episode. This finds ALL seasons (including later seasons labeled
    // with a newer year, e.g. "Weak Hero 2025 S02E01") in one request instead
    // of dozens of per-season probes.
    $backfill = [];
    $kwVariants = fd_stream_keyword_variants($keyword);
    foreach ($kwVariants as $kwVar) {
        if (count($backfill) >= 300) {
            break;
        }
        $res = fd_fetch_stream_ajax('search_files', [
            'search' => $kwVar,
            'limit' => 1000,
            'offset' => 0,
        ]);
        $files = (array) ($res['files'] ?? []);
        foreach ($files as $file) {
            if (count($backfill) >= 300) {
                break 2;
            }
            $code = (string) ($file['short_code'] ?? '');
            if ($code === '' || isset($seen[$code])) {
                continue;
            }
            $parsed = fd_classify_season_episode(
                (string) ($file['title'] ?? ''),
                (int) ($file['season_num'] ?? 0),
                (int) ($file['episode_num'] ?? 0),
                (string) ($file['caption'] ?? '')
            );
            if (($parsed['episode'] ?? 0) <= 0) {
                continue;
            }
            $seen[$code] = true;
            $backfill[] = $file;
        }
    }

    return array_merge($all, $backfill);
}

function fd_is_usable_lan_ipv4(string $ip): bool
{
    if (!filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV4)) {
        return false;
    }
    // Never treat loopback, link-local, Docker, or VirtualBox host-only as LAN.
    if (
        str_starts_with($ip, '127.') ||
        str_starts_with($ip, '169.254.') ||
        str_starts_with($ip, '172.17.') ||
        str_starts_with($ip, '192.168.56.') ||
        $ip === '0.0.0.0'
    ) {
        return false;
    }
    $parts = array_map('intval', explode('.', $ip));
    $a = $parts[0] ?? 0;
    $b = $parts[1] ?? 0;
    // RFC1918 only — public/ISP addresses (e.g. rmnet 21.x) are not LAN.
    if ($a === 10) {
        return true;
    }
    if ($a === 172 && $b >= 16 && $b <= 31) {
        return true;
    }
    if ($a === 192 && $b === 168) {
        return true;
    }
    return false;
}

function fd_is_skipped_lan_iface(string $iface): bool
{
    $iface = strtolower($iface);
    if ($iface === 'lo' || $iface === 'lo0') {
        return true;
    }
    $prefixes = [
        'rmnet',
        'ccmni',
        'pdp',
        'ccinet',
        'clat',
        'dummy',
        'docker',
        'br-',
        'veth',
        'cni',
        'flannel',
        'virbr',
        'tun',
        'wg',
        'ppp',
        'ipsec',
        'tailscale',
        'utun',
        'orichi',
    ];
    foreach ($prefixes as $prefix) {
        if (str_starts_with($iface, $prefix)) {
            return true;
        }
    }
    return false;
}

function fd_lan_iface_score(string $iface): int
{
    $iface = strtolower($iface);
    // Android hotspot / soft AP first.
    if (preg_match('/^(ap\d*|wlan\d*_ap|softap\d*)$/', $iface)) {
        return 100;
    }
    // Wi-Fi client or AP (wlan0, wlan1, wlan2, ...) — real shared LAN.
    if (preg_match('/^wlan\d+/', $iface)) {
        return 90;
    }
    if (preg_match('/^(rndis\d*|usb\d*|eth\d*|bnep\d*|bt-pan)$/', $iface)) {
        return 70;
    }
    // Vendor virtual gateway (vgate0 is POINTOPOINT /32 — last-resort LAN).
    if (str_starts_with($iface, 'vgate')) {
        return 20;
    }
    return 40;
}

function fd_pick_lan_ip_from_text(string $output): string
{
    $currentIface = '';
    $bestIp = '';
    $bestScore = -1;
    foreach (preg_split('/\r\n|\r|\n/', $output) as $line) {
        // `ip -4 addr show`: "2: wlan2: <BROADCAST,MULTICAST,UP,LOWER_UP> ..."
        // `ifconfig` (standard): "wlan2: flags=4163<UP,BROADCAST,RUNNING,MULTICAST> ..."
        // `ifconfig` (busybox):  "wlan2     Link encap:Ethernet  HWaddr ..."
        if (
            preg_match('/^\d+:\s+([^:@\s]+)/', $line, $m) ||
            preg_match('/^([A-Za-z0-9_.-]+)[:\s]/', $line, $m)
        ) {
            $currentIface = $m[1];
            continue;
        }
        if ($currentIface === '' || fd_is_skipped_lan_iface($currentIface)) {
            continue;
        }
        if (!preg_match('/\binet(?:\s+addr)?:?\s*(\d+\.\d+\.\d+\.\d+)/', $line, $m)) {
            continue;
        }
        $candidate = $m[1];
        if (!fd_is_usable_lan_ipv4($candidate)) {
            continue;
        }
        $score = fd_lan_iface_score($currentIface);
        if ($score > $bestScore) {
            $bestScore = $score;
            $bestIp = $candidate;
        }
    }
    return $bestIp;
}

function fd_lan_ip_from_php_ifaces(): string
{
    if (!function_exists('net_get_interfaces')) {
        return '';
    }
    try {
        $ifaces = @net_get_interfaces();
    } catch (Throwable $e) {
        return '';
    }
    if (!is_array($ifaces)) {
        return '';
    }
    $bestIp = '';
    $bestScore = -1;
    foreach ($ifaces as $name => $info) {
        $iface = explode(':', (string) $name, 2)[0];
        if (fd_is_skipped_lan_iface($iface)) {
            continue;
        }
        foreach (($info['unicast'] ?? []) as $addr) {
            $ip = (string) ($addr['address'] ?? '');
            if (!fd_is_usable_lan_ipv4($ip)) {
                continue;
            }
            $score = fd_lan_iface_score($iface);
            if ($score > $bestScore) {
                $bestScore = $score;
                $bestIp = $ip;
            }
        }
    }
    return $bestIp;
}

function fd_is_android_runtime(): bool
{
    $prefix = (string) ($_SERVER['PREFIX'] ?? $_ENV['PREFIX'] ?? '');
    return is_file('/system/bin/getprop')
        || isset($_SERVER['ANDROID_ROOT'])
        || isset($_ENV['ANDROID_ROOT'])
        || str_contains($prefix, 'com.termux')
        || str_contains($prefix, 'com.pencarimovie')
        || is_dir('/data/data/com.pencarimovie.downloader')
        || is_dir('/data/data/com.termux');
}

function fd_cached_lan_ip(): string
{
    // Do not use getenv() — it was removed from this file because it can
    // fatal on the Android/proot FrankenPHP build (disabled or missing).
    $env = trim((string) ($_SERVER['LAN_IP'] ?? $_ENV['LAN_IP'] ?? ''));
    if (fd_is_usable_lan_ipv4($env)) {
        return $env;
    }
    try {
        $path = fd_storage_path('storage/lan_ip.txt');
        if (is_file($path)) {
            $cached = trim((string) @file_get_contents($path));
            if (fd_is_usable_lan_ipv4($cached)) {
                return $cached;
            }
        }
    } catch (Throwable $e) {
        return '';
    }
    return '';
}

/**
 * Safe LAN IP for JSON APIs. Never shells out and never calls getenv().
 * Empty LAN_IP on old APKs must not fail session/auth.
 */
function fd_get_lan_ip_fast(): string
{
    try {
        $cached = fd_cached_lan_ip();
        if ($cached !== '') {
            return $cached;
        }
        $serverAddr = (string) ($_SERVER['SERVER_ADDR'] ?? '');
        if ($serverAddr !== '' && fd_is_usable_lan_ipv4($serverAddr)) {
            return $serverAddr;
        }
    } catch (Throwable $e) {
        return '';
    }
    return '';
}

function fd_get_lan_ip(): string
{
    try {
        $fast = fd_get_lan_ip_fast();
        if ($fast !== '') {
            return $fast;
        }

        // Old APK / Termux / proot: shell_exec(ifconfig/getprop/ip) and
        // net_get_interfaces() can hang or fatal. Skip live probes there.
        if (fd_is_android_runtime()) {
            return '';
        }

        $fromPhp = fd_lan_ip_from_php_ifaces();
        if ($fromPhp !== '') {
            return $fromPhp;
        }

        if (stripos(PHP_OS, 'WIN') === 0) {
            $lines = [];
            if (function_exists('exec')) {
                @exec('route print -4 0.0.0.0', $lines);
            }
            $bestIp = '';
            $bestMetric = 999999;
            foreach ($lines as $line) {
                if (preg_match('/0\.0\.0\.0\s+0\.0\.0\.0\s+(\S+)\s+(\d+\.\d+\.\d+\.\d+)\s+(\d+)/', $line, $m)) {
                    $ip = $m[2];
                    $metric = (int) $m[3];
                    if (fd_is_usable_lan_ipv4($ip) && $metric < $bestMetric) {
                        $bestMetric = $metric;
                        $bestIp = $ip;
                    }
                }
            }
            if ($bestIp !== '') {
                return $bestIp;
            }
        } elseif (function_exists('shell_exec')) {
            foreach (['ip -4 addr show', 'ifconfig', 'busybox ifconfig'] as $cmd) {
                $output = (string) @shell_exec($cmd . ' 2>/dev/null');
                if ($output === '') {
                    continue;
                }
                $candidate = fd_pick_lan_ip_from_text($output);
                if ($candidate !== '') {
                    return $candidate;
                }
            }

            $hostIps = @shell_exec('hostname -I 2>/dev/null');
            if ($hostIps) {
                $parts = preg_split('/\s+/', trim($hostIps));
                foreach ($parts as $part) {
                    if ($part !== '' && fd_is_usable_lan_ipv4($part)) {
                        return $part;
                    }
                }
            }
        }

        $serverAddr = $_SERVER['SERVER_ADDR'] ?? '';
        if ($serverAddr !== '' && fd_is_usable_lan_ipv4($serverAddr)) {
            return $serverAddr;
        }
    } catch (Throwable $e) {
        return '';
    }
    return '';
}

function fd_get_live_tunnel_https_origin(): string
{
    $state = fd_load_tunnel_state();
    $pid = (int) ($state['pid'] ?? 0);
    if ($pid < 2) {
        $pid = fd_tunnel_read_pid();
    }
    if ($pid < 2 || !fd_tunnel_pid_alive($pid)) {
        return '';
    }

    $url = fd_tunnel_normalize_url((string) ($state['tunnel_url'] ?? ''));
    if ($url === '' || !str_starts_with($url, 'https://')) {
        $url = fd_tunnel_normalize_url(fd_tunnel_read_quicktunnel_url($pid, (int) ($state['metrics_port'] ?? 0)));
    }
    if ($url === '' || !str_starts_with($url, 'https://')) {
        return '';
    }

    // Always prefer the stable custom subdomain if available
    $subdomain = fd_tunnel_subdomain();
    if ($subdomain !== '') {
        return 'https://' . $subdomain . '-tunnel.pencarimovie.com';
    }

    return rtrim($url, '/');
}

/**
 * Label the addon by the address the client used to fetch /manifest.json
 * so Stremio/Nuvio show Localhost vs Wi-Fi/LAN vs Cloudflare as separate addons.
 *
 * Optional ?mode=lan|localhost|tunnel forces the label so a tunneled HTTPS
 * page can still install an HTTP transport URL via Stremio API sync.
 *
 * @return array{mode: string, id: string, name: string, description: string}
 */
function fd_stremio_manifest_identity(): array
{
    $forwardedHost = trim((string) ($_SERVER['HTTP_X_FORWARDED_HOST'] ?? ''));
    $hostHeader = $forwardedHost !== '' ? $forwardedHost : trim((string) ($_SERVER['HTTP_HOST'] ?? ''));
    $hostHeader = trim(explode(',', $hostHeader)[0]);
    if ($hostHeader === '') {
        $hostHeader = '127.0.0.1:8088';
    }

    $hostName = strtolower((string) (parse_url('http://' . $hostHeader, PHP_URL_HOST) ?: $hostHeader));
    $isTunnel = fd_is_cloudflare_tunnel_request()
        || str_ends_with($hostName, '.trycloudflare.com')
        || $hostName === 'trycloudflare.com'
        || str_ends_with($hostName, '.tunnel.pencarimovie.com')
        || str_ends_with($hostName, '-tunnel.pencarimovie.com')
        || $hostName === 'tunnel.pencarimovie.com';
    $tunnelOrigin = fd_get_live_tunnel_https_origin();
    $scheme = $isTunnel ? 'https' : 'http';
    $origin = ($isTunnel && $tunnelOrigin !== '') ? $tunnelOrigin : ($scheme . '://' . $hostHeader);
    $listenPort = fd_get_listen_port();
    $modeOverride = strtolower(trim((string) ($_GET['mode'] ?? '')));

    if ($modeOverride === 'tunnel') {
        return [
            'mode' => 'tunnel',
            'id' => 'org.pencarimovie.addon.tunnel',
            'name' => 'PencariMovie (Cloudflare)',
            'description' => 'Stream movies and series from Telegram via Cloudflare Tunnel HTTPS. Address: ' . ($tunnelOrigin !== '' ? $tunnelOrigin : $origin),
        ];
    }
    if ($modeOverride === 'localhost') {
        $localOrigin = 'http://127.0.0.1:' . $listenPort;
        return [
            'mode' => 'localhost',
            'id' => 'org.pencarimovie.addon.local',
            'name' => 'PencariMovie (Localhost)',
            'description' => 'Stream movies and series from Telegram on this device only. Address: ' . $localOrigin,
        ];
    }
    if ($modeOverride === 'lan') {
        $lanIp = fd_get_lan_ip();
        $lanOrigin = $lanIp !== ''
            ? ('http://' . $lanIp . ':' . $listenPort)
            : $origin;
        return [
            'mode' => 'lan',
            'id' => 'org.pencarimovie.addon.lan',
            'name' => 'PencariMovie (Wi-Fi / LAN)',
            'description' => 'Stream movies and series from Telegram on your Wi-Fi / LAN. Address: ' . $lanOrigin,
        ];
    }

    if ($isTunnel) {
        return [
            'mode' => 'tunnel',
            'id' => 'org.pencarimovie.addon.tunnel',
            'name' => 'PencariMovie (Cloudflare)',
            'description' => 'Stream movies and series from Telegram via Cloudflare Tunnel HTTPS. Address: ' . ($tunnelOrigin !== '' ? $tunnelOrigin : $origin),
        ];
    }

    if (in_array($hostName, ['127.0.0.1', 'localhost', '::1'], true)) {
        return [
            'mode' => 'localhost',
            'id' => 'org.pencarimovie.addon.local',
            'name' => 'PencariMovie (Localhost)',
            'description' => 'Stream movies and series from Telegram on this device only. Address: ' . $origin,
        ];
    }

    if (fd_is_usable_lan_ipv4($hostName)) {
        return [
            'mode' => 'lan',
            'id' => 'org.pencarimovie.addon.lan',
            'name' => 'PencariMovie (Wi-Fi / LAN)',
            'description' => 'Stream movies and series from Telegram on your Wi-Fi / LAN. Address: ' . $origin,
        ];
    }

    return [
        'mode' => 'default',
        'id' => 'org.pencarimovie.addon',
        'name' => 'PencariMovie',
        'description' => 'Stream movies and series from Telegram. Address: ' . $origin,
    ];
}

function fd_get_stremio_base_url(): string
{
    $forwardedHost = trim((string) ($_SERVER['HTTP_X_FORWARDED_HOST'] ?? ''));
    $host = $forwardedHost !== '' ? $forwardedHost : trim((string) ($_SERVER['HTTP_HOST'] ?? ''));
    $host = trim(explode(',', $host)[0]);
    if ($host === '') {
        $host = '127.0.0.1:8088';
    }

    $forwardedProto = strtolower(trim(explode(',', (string) ($_SERVER['HTTP_X_FORWARDED_PROTO'] ?? ''))[0]));
    $cfVisitor = (string) ($_SERVER['HTTP_CF_VISITOR'] ?? '');
    $isTunnel = fd_is_cloudflare_tunnel_request();
    $isHttps = (isset($_SERVER['HTTPS']) && $_SERVER['HTTPS'] !== '' && $_SERVER['HTTPS'] !== 'off')
        || $forwardedProto === 'https'
        || str_contains($cfVisitor, '"scheme":"https"')
        || $isTunnel;
    $scheme = $isHttps ? 'https' : 'http';
    $origin = $scheme . '://' . $host;

    // If request comes through tunnel or localhost with tunnel running, use custom subdomain
    $tunnelOrigin = fd_get_live_tunnel_https_origin();
    if ($tunnelOrigin !== '') {
        $hostName = strtolower((string) (parse_url('http://' . $host, PHP_URL_HOST) ?: $host));
        if ($isTunnel || in_array($hostName, ['127.0.0.1', 'localhost', '::1'], true)) {
            return $tunnelOrigin;
        }
    }

    return $origin;
}

/**
 * Telegram file_type is often "document". Stremio HTML5 playback needs a real video MIME.
 */
function fd_guess_video_mime(string $fileName, string $mime = ''): string
{
    $mime = strtolower(trim($mime));
    if (
        $mime !== ''
        && str_contains($mime, '/')
        && !in_array($mime, ['document', 'application/document', 'application/octet-stream', 'binary/octet-stream'], true)
    ) {
        return $mime;
    }

    $ext = strtolower(pathinfo($fileName, PATHINFO_EXTENSION));
    return match ($ext) {
        'mkv' => 'video/x-matroska',
        'webm' => 'video/webm',
        'avi' => 'video/x-msvideo',
        'mov' => 'video/quicktime',
        'ts', 'm2ts' => 'video/mp2t',
        'm4v' => 'video/mp4',
        default => 'video/mp4',
    };
}

function fd_stremio_stream_filename(string $fileName, string $mime = ''): string
{
    $name = trim($fileName);
    if ($name === '') {
        $name = 'video.mp4';
    }
    $name = preg_replace('/[^\w.\-]+/', '_', $name) ?: 'video.mp4';
    $name = trim($name, '._-');
    if ($name === '') {
        $name = 'video.mp4';
    }

    $ext = strtolower(pathinfo($name, PATHINFO_EXTENSION));
    $videoExts = ['mp4', 'm4v', 'mkv', 'webm', 'avi', 'mov', 'ts', 'm2ts'];
    if (!in_array($ext, $videoExts, true)) {
        $fromMime = strtolower($mime);
        $suffix = match (true) {
            str_contains($fromMime, 'matroska') => 'mkv',
            str_contains($fromMime, 'webm') => 'webm',
            str_contains($fromMime, 'quicktime') => 'mov',
            default => 'mp4',
        };
        $name .= '.' . $suffix;
    }

    return $name;
}

/**
 * Official Stremio addon-sdk: HTML5 playback only for HTTPS MP4 URLs.
 * Stremio Web's HTML5 check is url.endsWith('.mp4') on the FULL URL, not pathname.
 * Query-string ?d= after .mp4 fails that check, so the payload lives in the path.
 */
function fd_build_stremio_stream_url(string $baseUrl, string $payloadB64, string $fileName, string $mime = ''): string
{
    $safe = fd_stremio_stream_filename($fileName, $mime);
    return rtrim($baseUrl, '/') . '/api/download/' . rawurlencode($payloadB64) . '/' . rawurlencode($safe);
}

function fd_is_public_download_path(string $path): bool
{
    return $path === '/api/download' || str_starts_with($path, '/api/download/');
}

function fd_extract_download_payload_from_path(string $path): string
{
    if (!preg_match('#^/api/download/([^/]+)(?:/|$)#', $path, $m)) {
        return '';
    }

    $segment = rawurldecode($m[1]);
    $ext = strtolower(pathinfo($segment, PATHINFO_EXTENSION));
    $videoExts = ['mp4', 'm4v', 'mkv', 'webm', 'avi', 'mov', 'ts', 'm2ts'];
    if (in_array($ext, $videoExts, true)) {
        return '';
    }

    return $segment;
}

function fd_get_app_root(): string
{
    $storage = fd_get_storage_dir();
    $root = dirname($storage);
    if ($root !== '' && $root !== '.' && is_dir($root) && !fd_is_temp_app_dir($root)) {
        return $root;
    }
    $docRoot = (string) ($_SERVER['DOCUMENT_ROOT'] ?? '');
    if ($docRoot !== '' && !fd_is_temp_app_dir($docRoot)) {
        return rtrim($docRoot, '/\\');
    }
    $cwd = getcwd();
    if (is_string($cwd) && $cwd !== '' && !fd_is_temp_app_dir($cwd)) {
        return $cwd;
    }
    return __DIR__;
}

function fd_get_listen_port(): int
{
    $port = (int) ($_SERVER['SERVER_PORT'] ?? 0);
    if ($port > 0 && $port < 65536) {
        return $port;
    }
    $env = (int) ($_SERVER['PORT'] ?? $_ENV['PORT'] ?? 0);
    if ($env > 0 && $env < 65536) {
        return $env;
    }
    return 8088;
}

function fd_is_windows(): bool
{
    return stripos(PHP_OS, 'WIN') === 0
        || defined('PHP_OS_FAMILY') && PHP_OS_FAMILY === 'Windows';
}

function fd_tunnel_dir(): string
{
    $dir = fd_storage_path('storage/tunnel');
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    return $dir;
}

function fd_tunnel_state_path(): string
{
    return fd_tunnel_dir() . DIRECTORY_SEPARATOR . 'state.json';
}

function fd_tunnel_pid_path(): string
{
    return fd_tunnel_dir() . DIRECTORY_SEPARATOR . 'cloudflared.pid';
}

function fd_tunnel_log_path(): string
{
    return fd_tunnel_dir() . DIRECTORY_SEPARATOR . 'cloudflared.log';
}

function fd_tunnel_err_log_path(): string
{
    return fd_tunnel_dir() . DIRECTORY_SEPARATOR . 'cloudflared.err.log';
}

function fd_tunnel_config_path(): string
{
    return fd_tunnel_dir() . DIRECTORY_SEPARATOR . 'config.yml';
}

function fd_tunnel_bin_dir(): string
{
    $dir = fd_storage_path('storage/bin');
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    return $dir;
}


function fd_get_device_id(): string
{
    $path = FD_DEVICE_ID_PATH;
    if (is_file($path)) {
        $id = strtolower(trim((string) @file_get_contents($path)));
        if (preg_match('/^[a-z0-9]{4,12}$/', $id)) {
            return $id;
        }
    }

    try {
        $id = substr(bin2hex(random_bytes(3)), 0, 6);
    } catch (\Throwable $e) {
        $id = substr(md5(uniqid((string) mt_rand(), true)), 0, 6);
    }

    @file_put_contents($path, $id, LOCK_EX);
    return $id;
}

/**
 * Check if a topkeyword is valid for catalog display (rejecting filler and NSFW words like 'new', 'movie', 'sex', etc.).
 */
function fd_is_topkeyword_valid(string $keyword): bool
{
    $kw = strtolower(trim($keyword));
    if ($kw === '' || mb_strlen($kw, 'UTF-8') < 3) {
        return false;
    }

    static $banned = [
        // Generic filler words
        'new',
        'movie',
        'movies',
        'film',
        'filem',
        'music',
        'song',
        'songs',
        'mp3',
        'latest',
        'baru',
        'naya',
        'putiya',
        'list',
        'lists',
        'start',
        'search',
        'audio',
        'video',
        'videos',
        'download',
        'free',
        'full',
        'hd',
        'online',
        'watch',
        'streaming',
        'series',
        'episode',
        'season',
        'part',
        'chapter',
        'test',
        'bot',
        'admin',
        'help',
        'hi',
        'hello',
        'hey',
        'hai',
        'helo',
        'ok',
        // Adult / NSFW tokens
        'sex',
        'sexy',
        'porn',
        'bokep',
        'lucah',
        'ngentot',
        'xxx',
        'hentai',
        'jav',
        'adult',
        'nsfw',
        'naked',
        'nude',
        'colmek',
        'sange',
        'tetek',
    ];

    if (in_array($kw, $banned, true)) {
        return false;
    }

    // Also reject if keyword matches single banned token exactly
    foreach ($banned as $b) {
        if ($kw === $b) {
            return false;
        }
    }

    return true;
}

/**
 * Default catalog definitions that can be toggled on/off or customized.
 */
function fd_get_default_catalog_options(): array
{
    $options = [
        // Special catalogs: Popular, Search, Telegram Files enabled by default
        'top' => ['type' => 'movie', 'group' => 'special', 'name' => 'Popular (Movies)', 'default' => true],
        'year' => ['type' => 'movie', 'group' => 'special', 'name' => 'New (Movies)', 'default' => false],
        'pm_search_movie' => ['type' => 'movie', 'group' => 'special', 'name' => 'Search Movies', 'default' => true],
        'pm_series_top' => ['type' => 'series', 'group' => 'special', 'name' => 'Popular (Series)', 'default' => true],
        'pm_series_year' => ['type' => 'series', 'group' => 'special', 'name' => 'New (Series)', 'default' => false],
        'pm_search_series' => ['type' => 'series', 'group' => 'special', 'name' => 'Search Series', 'default' => true],
        'pm_files_year' => ['type' => 'other', 'group' => 'special', 'name' => 'New (Telegram Files)', 'default' => true],
        'pm_search_files' => ['type' => 'other', 'group' => 'special', 'name' => 'Telegram Files (Search)', 'default' => true],
        'pm_trending_keywords' => ['type' => 'other', 'group' => 'special', 'name' => 'Trending Keywords (Files)', 'default' => true],
        // Movies country/category (default disabled)
        'pm_movies_malay' => ['type' => 'movie', 'group' => 'country', 'name' => 'Malaysia (Movie)', 'default' => false],
        'pm_movies_indo' => ['type' => 'movie', 'group' => 'country', 'name' => 'Indonesia (Movie)', 'default' => false],
        'pm_movies_korean' => ['type' => 'movie', 'group' => 'country', 'name' => 'Korea (Movie)', 'default' => false],
        'pm_movies_japan' => ['type' => 'movie', 'group' => 'country', 'name' => 'Japan (Movie)', 'default' => false],
        'pm_movies_anime' => ['type' => 'movie', 'group' => 'country', 'name' => 'Anime (Movie)', 'default' => false],
        'pm_movies_chinese' => ['type' => 'movie', 'group' => 'country', 'name' => 'China / HK (Movie)', 'default' => false],
        'pm_movies_thai' => ['type' => 'movie', 'group' => 'country', 'name' => 'Thailand (Movie)', 'default' => false],
        'pm_movies_bollywood' => ['type' => 'movie', 'group' => 'country', 'name' => 'Bollywood (Movie)', 'default' => false],
        'pm_movies_philippines' => ['type' => 'movie', 'group' => 'country', 'name' => 'Philippines (Movie)', 'default' => false],
        'pm_movies_english' => ['type' => 'movie', 'group' => 'country', 'name' => 'English (Movie)', 'default' => false],
        // Series country/category (default disabled)
        'pm_series_kdrama' => ['type' => 'series', 'group' => 'country', 'name' => 'K-Drama (Series)', 'default' => false],
        'pm_series_anime' => ['type' => 'series', 'group' => 'country', 'name' => 'Anime (Series)', 'default' => false],
        'pm_series_japan' => ['type' => 'series', 'group' => 'country', 'name' => 'J-Drama (Series)', 'default' => false],
        'pm_series_malay' => ['type' => 'series', 'group' => 'country', 'name' => 'Malaysia (Series)', 'default' => false],
        'pm_series_cdrama' => ['type' => 'series', 'group' => 'country', 'name' => 'C-Drama (Series)', 'default' => false],
        'pm_series_thai' => ['type' => 'series', 'group' => 'country', 'name' => 'Thailand (Series)', 'default' => false],
        'pm_series_philippines' => ['type' => 'series', 'group' => 'country', 'name' => 'Philippines (Series)', 'default' => false],
        'pm_series_english' => ['type' => 'series', 'group' => 'country', 'name' => 'English (Series)', 'default' => false],
        'pm_series_indo' => ['type' => 'series', 'group' => 'country', 'name' => 'Indonesia (Series)', 'default' => false],
    ];

    return $options;
}

/**
 * Load catalog settings from disk.
 * Returns:
 * [
 *   'catalogs_enabled' => bool (true = catalogs active, false = disable catalogs, streams list only from tt),
 *   'enabled_types' => ['movie' => true, 'series' => true],
 *   'enabled_catalogs' => ['pm_movies_latest' => true, ...]
 * ]
 */
function fd_load_catalog_settings(): array
{
    $defaults = [
        'catalogs_enabled' => true,
        'enabled_types' => [
            'movie' => true,
            'series' => true,
            'other' => true,
        ],
        'enabled_catalogs' => [],
    ];
    $catalogOptions = fd_get_default_catalog_options();
    foreach ($catalogOptions as $id => $info) {
        $defaults['enabled_catalogs'][$id] = !empty($info['default']);
    }

    $path = FD_CATALOG_SETTINGS_PATH;
    if (is_file($path)) {
        $data = json_decode((string) @file_get_contents($path), true);
        if (is_array($data)) {
            if (isset($data['catalogs_enabled'])) {
                $defaults['catalogs_enabled'] = (bool) $data['catalogs_enabled'];
            }
            if (!empty($data['enabled_types']) && is_array($data['enabled_types'])) {
                if (isset($data['enabled_types']['movie'])) {
                    $defaults['enabled_types']['movie'] = (bool) $data['enabled_types']['movie'];
                }
                if (isset($data['enabled_types']['series'])) {
                    $defaults['enabled_types']['series'] = (bool) $data['enabled_types']['series'];
                }
                if (isset($data['enabled_types']['other'])) {
                    $defaults['enabled_types']['other'] = (bool) $data['enabled_types']['other'];
                }
            }
            if (isset($data['enabled_catalogs']) && is_array($data['enabled_catalogs'])) {
                foreach ($data['enabled_catalogs'] as $cid => $val) {
                    $defaults['enabled_catalogs'][$cid] = (bool) $val;
                }
            }
        }
    }

    return $defaults;
}

/**
 * Save catalog settings to disk.
 */
function fd_save_catalog_settings(array $settings): bool
{
    $path = FD_CATALOG_SETTINGS_PATH;
    $json = json_encode($settings, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES);
    return @file_put_contents($path, $json, LOCK_EX) !== false;
}

/**
 * Detect client/server country from Cloudflare headers in memory.
 * No local json files or user_id required.
 *
 * @return array{country_code: string, country_name: string, source: string}
 */
function fd_detect_country(): array
{
    $code = strtoupper(trim((string) ($_SERVER['HTTP_CF_IPCOUNTRY'] ?? '')));
    $source = 'cf-header';

    if ($code === '' || $code === 'XX' || $code === 'T1' || strlen($code) !== 2) {
        $code = 'MY';
        $source = 'default';
    }

    $countryMap = [
        'MY' => 'Malaysia',
        'ID' => 'Indonesia',
        'SG' => 'Singapore',
        'TH' => 'Thailand',
        'PH' => 'Philippines',
        'VN' => 'Vietnam',
        'KR' => 'Korea',
        'JP' => 'Japan',
        'CN' => 'China',
        'HK' => 'Hong Kong',
        'TW' => 'Taiwan',
        'IN' => 'India',
        'US' => 'United States',
        'GB' => 'United Kingdom',
        'AU' => 'Australia',
        'DE' => 'Germany',
        'NL' => 'Netherlands',
        'FR' => 'France',
        'CA' => 'Canada',
    ];

    return [
        'country_code' => $code,
        'country_name' => $countryMap[$code] ?? $code,
        'source' => $source,
    ];
}

function fd_tunnel_device_id(): string
{
    return fd_get_device_id();
}

function fd_tunnel_subdomain(): string
{
    return fd_get_device_id();
}

function fd_tunnel_bot_subdomain(): string
{
    return fd_tunnel_subdomain();
}

function fd_tunnel_register_worker(string $tunnelUrl): array
{
    $shortId = fd_tunnel_device_id();
    $payload = [
        'shortId' => $shortId,
        'tunnelUrl' => $tunnelUrl,
    ];

    // 1. Primary registration: Contabo VPS endpoint (stores in Redis, 0 Worker quota)
    $vpsEndpoint = (defined('FD_WP_API_BASE') ? FD_WP_API_BASE : 'https://pencarimovie.com/wp-json/pencarimovie-server/v1') . '/tunnel/register';
    $vpsRes = fd_http_json($vpsEndpoint, $payload, 'POST', 10);

    // 2. Fallback registration: Cloudflare Worker relay
    $cfEndpoint = 'https://tunnel.pencarimovie.com/api/tunnel/register';
    @fd_http_json($cfEndpoint, $payload, 'POST', 5);

    return $vpsRes;
}

function fd_load_tunnel_state(): array
{
    $path = fd_tunnel_state_path();
    if (!is_file($path)) {
        return [];
    }
    $raw = @file_get_contents($path);
    if (!is_string($raw) || $raw === '') {
        return [];
    }
    $data = json_decode($raw, true);
    return is_array($data) ? $data : [];
}

function fd_save_tunnel_state(array $state): void
{
    $path = fd_tunnel_state_path();
    $dir = dirname($path);
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    @file_put_contents(
        $path,
        json_encode($state, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE | JSON_PRETTY_PRINT),
        LOCK_EX
    );
}

function fd_clear_tunnel_state(): void
{
    $path = fd_tunnel_state_path();
    if (is_file($path)) {
        @unlink($path);
    }
    $pidPath = fd_tunnel_pid_path();
    if (is_file($pidPath)) {
        @unlink($pidPath);
    }
}

function fd_tunnel_read_pid(): int
{
    $path = fd_tunnel_pid_path();
    if (!is_file($path)) {
        $state = fd_load_tunnel_state();
        return (int) ($state['pid'] ?? 0);
    }
    return (int) trim((string) @file_get_contents($path));
}

function fd_tunnel_write_pid(int $pid): void
{
    @file_put_contents(fd_tunnel_pid_path(), (string) $pid, LOCK_EX);
}

function fd_tunnel_pid_alive(int $pid): bool
{
    if ($pid <= 1) {
        return false;
    }
    if (fd_is_windows()) {
        $tasklist = is_file('C:\\Windows\\System32\\tasklist.exe') ? 'C:\\Windows\\System32\\tasklist.exe' : 'tasklist';
        $out = [];
        @exec($tasklist . ' /FI "PID eq ' . $pid . '" /NH /FO CSV 2>nul', $out);
        $joined = strtolower(implode("\n", $out));
        return str_contains($joined, 'cloudflared') && str_contains($joined, (string) $pid);
    }
    if (function_exists('posix_kill')) {
        return @posix_kill($pid, 0);
    }
    return is_dir('/proc/' . $pid);
}

function fd_tunnel_kill_pid(int $pid): void
{
    if ($pid <= 1) {
        return;
    }
    if (fd_is_windows()) {
        $taskkill = is_file('C:\\Windows\\System32\\taskkill.exe') ? 'C:\\Windows\\System32\\taskkill.exe' : 'taskkill';
        @exec($taskkill . ' /PID ' . $pid . ' /T /F >nul 2>nul');
        return;
    }
    if (function_exists('posix_kill')) {
        @posix_kill($pid, 15);
        usleep(250000);
        if (fd_tunnel_pid_alive($pid)) {
            @posix_kill($pid, 9);
        }
        return;
    }
    @exec('kill ' . $pid . ' 2>/dev/null');
    usleep(250000);
    if (fd_tunnel_pid_alive($pid)) {
        @exec('kill -9 ' . $pid . ' 2>/dev/null');
    }
}

function fd_tunnel_kill_leftovers(): void
{
    $marker = fd_tunnel_config_path();
    $pid = fd_tunnel_read_pid();
    if ($pid > 1) {
        fd_tunnel_kill_pid($pid);
    }

    if (fd_is_windows()) {
        $ps = 'Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |'
            . ' Where-Object { $_.Name -match \'cloudflared\' -and $_.CommandLine -and'
            . ' ($_.CommandLine -match [regex]::Escape(\'' . str_replace('\'', '\'\'', $marker) . '\')'
            . ' -or $_.CommandLine -match \'storage[\\\\/]tunnel\') } |'
            . ' ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }';
        $psExe = 'powershell.exe';
        if (is_file('C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe')) {
            $psExe = 'C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe';
        }
        @exec(escapeshellarg($psExe) . ' -NoProfile -Command ' . escapeshellarg($ps) . ' >nul 2>nul');
        return;
    }

    @exec('pkill -f ' . escapeshellarg($marker) . ' 2>/dev/null');
    @exec('kill $(pgrep -f ' . escapeshellarg($marker) . ') 2>/dev/null');
}

function fd_tunnel_normalize_url(string $value): string
{
    $value = trim($value);
    if ($value === '') {
        return '';
    }
    if (!preg_match('#^https?://#i', $value)) {
        $value = 'https://' . ltrim($value, '/');
    }
    $value = rtrim($value, '/');
    if (!preg_match('#^https://([a-z0-9-]+)\.trycloudflare\.com$#i', $value, $m)) {
        return '';
    }
    if (strtolower($m[1]) === 'api') {
        return '';
    }
    return 'https://' . strtolower($m[1]) . '.trycloudflare.com';
}

function fd_tunnel_parse_url_from_text(string $text): string
{
    if ($text === '' || !preg_match_all('#https://([a-z0-9-]+)\.trycloudflare\.com#i', $text, $matches)) {
        return '';
    }
    $found = '';
    foreach ($matches[1] as $i => $host) {
        if (strtolower($host) === 'api') {
            continue;
        }
        $found = fd_tunnel_normalize_url($matches[0][$i]);
    }
    return $found;
}

function fd_tunnel_read_logs(): string
{
    $chunks = [];
    foreach ([fd_tunnel_log_path(), fd_tunnel_err_log_path()] as $path) {
        if (is_file($path)) {
            $chunks[] = (string) @file_get_contents($path);
        }
    }
    return implode("\n", $chunks);
}

function fd_tunnel_pick_metrics_port(): int
{
    $sock = @stream_socket_server('tcp://127.0.0.1:0');
    if (is_resource($sock)) {
        $name = @stream_socket_get_name($sock, false);
        fclose($sock);
        if (is_string($name) && preg_match('/:(\d+)$/', $name, $m)) {
            $port = (int) $m[1];
            if ($port > 0) {
                return $port;
            }
        }
    }
    return 20241;
}

function fd_tunnel_local_http_get(string $url, int $timeoutSec = 1): string
{
    if (function_exists('curl_version')) {
        $ch = curl_init($url);
        curl_setopt_array($ch, [
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT => $timeoutSec,
            CURLOPT_CONNECTTIMEOUT => $timeoutSec,
            CURLOPT_FOLLOWLOCATION => false,
            CURLOPT_NOSIGNAL => true,
            CURLOPT_HTTPHEADER => [
                'Accept: application/json',
                'User-Agent: pencarimovie-server/' . FD_APP_VERSION,
            ],
        ]);
        $cStart = microtime(true);
        $body = curl_exec($ch);
        $cDuration = round(microtime(true) - $cStart, 3);
        $httpCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);

        if ($body !== false) {
            fd_log('tunnel curl request completed', [
                'url' => $url,
                'http_code' => $httpCode,
                'duration_seconds' => $cDuration,
            ]);
        } else {
            fd_log('tunnel curl request failed', [
                'url' => $url,
                'http_code' => $httpCode,
                'duration_seconds' => $cDuration,
                'errno' => curl_errno($ch),
                'error' => curl_error($ch),
            ]);
        }

        if (PHP_VERSION_ID < 80500) {
            curl_close($ch);
        }
        unset($ch);
        return is_string($body) ? $body : '';
    }
    $ctx = stream_context_create([
        'http' => [
            'method' => 'GET',
            'timeout' => $timeoutSec,
            'ignore_errors' => true,
            'header' => "Accept: application/json\r\n",
        ],
    ]);
    $body = @file_get_contents($url, false, $ctx);
    return is_string($body) ? $body : '';
}

function fd_tunnel_metrics_ports(int $pid = 0, int $preferred = 0): array
{
    $ports = [];
    if ($preferred > 0) {
        $ports[] = $preferred;
    }
    $fromState = (int) (fd_load_tunnel_state()['metrics_port'] ?? 0);
    if ($fromState > 0) {
        $ports[] = $fromState;
    }
    $ports[] = 20241;

    if ($pid > 1) {
        if (fd_is_windows()) {
            $out = [];
            @exec('netstat -ano -p tcp 2>nul', $out);
            foreach ($out as $line) {
                $line = (string) $line;
                if (!str_contains($line, (string) $pid) || stripos($line, 'LISTENING') === false) {
                    continue;
                }
                if (preg_match('/127\.0\.0\.1:(\d+)/', $line, $m)) {
                    $ports[] = (int) $m[1];
                }
            }
        } else {
            $out = (string) @shell_exec('ss -lntp 2>/dev/null || netstat -lntp 2>/dev/null');
            foreach (preg_split('/\r\n|\n|\r/', $out) as $line) {
                if (!str_contains((string) $line, (string) $pid)) {
                    continue;
                }
                if (preg_match('/127\.0\.0\.1:(\d+)/', (string) $line, $m)) {
                    $ports[] = (int) $m[1];
                }
            }
        }
    }

    return array_values(array_unique(array_filter($ports, static fn($port) => (int) $port > 0)));
}

function fd_tunnel_parse_quicktunnel_body(string $body): string
{
    $body = trim($body);
    if ($body === '') {
        return '';
    }
    $json = json_decode($body, true);
    if (is_array($json)) {
        foreach (['hostname', 'Hostname', 'url', 'URL'] as $key) {
            if (!empty($json[$key]) && is_string($json[$key])) {
                $url = fd_tunnel_normalize_url($json[$key]);
                if ($url !== '') {
                    return $url;
                }
            }
        }
    }
    if (preg_match('/userHostname="(https:\/\/[a-z0-9-]+\.trycloudflare\.com)"/i', $body, $m)) {
        return fd_tunnel_normalize_url($m[1]);
    }
    return fd_tunnel_parse_url_from_text($body);
}

function fd_tunnel_read_quicktunnel_url(int $pid = 0, int $metricsPort = 0): string
{
    foreach (fd_tunnel_metrics_ports($pid, $metricsPort) as $port) {
        $base = 'http://127.0.0.1:' . $port;
        $url = fd_tunnel_parse_quicktunnel_body(fd_tunnel_local_http_get($base . '/quicktunnel', 1));
        if ($url !== '') {
            return $url;
        }
        $url = fd_tunnel_parse_quicktunnel_body(fd_tunnel_local_http_get($base . '/metrics', 1));
        if ($url !== '') {
            return $url;
        }
    }
    return '';
}

function fd_tunnel_asset_name(): string
{
    $arch = strtolower(php_uname('m'));
    $isArm = (bool) preg_match('/arm|aarch/i', $arch);
    $isArm32 = (bool) preg_match('/armv7|armhf|armv6|armel/i', $arch);

    if (fd_is_windows()) {
        // Cloudflare does not publish native Windows ARM binaries; Windows 11 ARM runs amd64 via x64 emulation
        return 'cloudflared-windows-amd64.exe';
    }
    if (stripos(PHP_OS, 'Darwin') === 0) {
        return $isArm ? 'cloudflared-darwin-arm64.tgz' : 'cloudflared-darwin-amd64.tgz';
    }
    if ($isArm32) {
        return 'cloudflared-linux-arm';
    }
    return $isArm ? 'cloudflared-linux-arm64' : 'cloudflared-linux-amd64';
}

function fd_tunnel_bin_path(): string
{
    $name = fd_is_windows() ? 'cloudflared.exe' : 'cloudflared';
    return fd_tunnel_bin_dir() . DIRECTORY_SEPARATOR . $name;
}

function fd_tunnel_which(): string
{
    if (fd_is_windows()) {
        $out = [];
        @exec('where cloudflared 2>nul', $out);
        foreach ($out as $line) {
            $line = trim((string) $line);
            if ($line !== '' && is_file($line)) {
                return $line;
            }
        }
        return '';
    }
    $out = trim((string) @shell_exec('command -v cloudflared 2>/dev/null'));
    return ($out !== '' && is_file($out)) ? $out : '';
}

function fd_http_download_file(string $url, string $dest, int $timeout = 120): bool
{
    $dir = dirname($dest);
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    $tmp = $dest . '.part';
    if (is_file($tmp)) {
        @unlink($tmp);
    }

    $headers = [
        'User-Agent: pencarimovie-server/' . FD_APP_VERSION,
        'Accept: application/octet-stream',
    ];

    if (function_exists('curl_version')) {
        $downloadWithCurl = function (array $extraOpts = []) use ($url, $tmp, $timeout, $headers): array {
            $fp = @fopen($tmp, 'wb');
            if ($fp === false) {
                return [false, 'Failed to open temp file for writing.'];
            }
            $ch = curl_init();
            $curlOpts = [
                CURLOPT_URL => $url,
                CURLOPT_FILE => $fp,
                CURLOPT_FOLLOWLOCATION => true,
                CURLOPT_TIMEOUT => $timeout,
                CURLOPT_CONNECTTIMEOUT => 20,
                CURLOPT_SSL_VERIFYPEER => false,
                CURLOPT_SSL_VERIFYHOST => 0,
                CURLOPT_HTTPHEADER => $headers,
                CURLOPT_FAILONERROR => true,
            ];
            foreach ($extraOpts as $k => $v) {
                $curlOpts[$k] = $v;
            }
            curl_setopt_array($ch, $curlOpts);
            $cStart = microtime(true);
            $ok = curl_exec($ch) === true;
            $err = $ok ? '' : curl_error($ch);
            $cDuration = round(microtime(true) - $cStart, 3);
            $httpCode = curl_getinfo($ch, CURLINFO_HTTP_CODE);

            if ($ok) {
                fd_log('curl file download completed', [
                    'url' => $url,
                    'http_code' => $httpCode,
                    'duration_seconds' => $cDuration,
                ]);
            } else {
                fd_log('curl file download failed', [
                    'url' => $url,
                    'http_code' => $httpCode,
                    'duration_seconds' => $cDuration,
                    'errno' => curl_errno($ch),
                    'error' => $err,
                ]);
            }

            if (PHP_VERSION_ID < 80500) {
                curl_close($ch);
            }
            fclose($fp);
            unset($ch);
            return [$ok, $err];
        };

        [$ok, $err] = $downloadWithCurl();

        // If DNS resolution fails (common in Android / Termux proot containers), retry with fallback DNS / resolve mapping
        if (!$ok && (stripos($err, 'Could not resolve') !== false || stripos($err, 'Couldn\'t resolve') !== false || stripos($err, 'name lookup') !== false)) {
            fd_log('cloudflared download DNS failed, attempting DNS-over-HTTPS / direct resolution fallback', ['url' => $url, 'error' => $err]);

            // Try resolving github.com, objects.githubusercontent.com, and release-assets.githubusercontent.com via known IPs
            $fallbackResolves = [
                'github.com:443:140.82.121.3',
                'github.com:443:140.82.121.4',
                'objects.githubusercontent.com:443:185.199.108.133',
                'objects.githubusercontent.com:443:185.199.109.133',
                'objects.githubusercontent.com:443:185.199.110.133',
                'objects.githubusercontent.com:443:185.199.111.133',
                'release-assets.githubusercontent.com:443:185.199.108.133',
                'release-assets.githubusercontent.com:443:185.199.109.133',
                'release-assets.githubusercontent.com:443:185.199.110.133',
                'release-assets.githubusercontent.com:443:185.199.111.133',
                'raw.githubusercontent.com:443:185.199.108.133',
                'raw.githubusercontent.com:443:185.199.109.133',
                'raw.githubusercontent.com:443:185.199.110.133',
                'raw.githubusercontent.com:443:185.199.111.133',
            ];

            $retryOpts = [CURLOPT_RESOLVE => $fallbackResolves];
            if (defined('CURLOPT_DNS_SERVERS')) {
                $retryOpts[CURLOPT_DNS_SERVERS] = '1.1.1.1,8.8.8.8,1.0.0.1,8.8.4.4';
            }

            [$ok, $err] = $downloadWithCurl($retryOpts);
        }

        if (!$ok) {
            fd_log('cloudflared download failed', ['url' => $url, 'error' => $err]);
            @unlink($tmp);
            return false;
        }
    } else {
        $body = fd_http_get_contents($url, ['timeout' => $timeout, 'headers' => $headers]);
        if (!is_string($body) || $body === '') {
            return false;
        }
        if (@file_put_contents($tmp, $body) === false) {
            return false;
        }
    }

    if (!is_file($tmp) || filesize($tmp) < 1024) {
        @unlink($tmp);
        return false;
    }
    if (is_file($dest)) {
        @unlink($dest);
    }
    return @rename($tmp, $dest);
}

function fd_tunnel_extract_tgz(string $tgz, string $destBin): bool
{
    $dir = dirname($destBin);
    if (class_exists(PharData::class)) {
        try {
            $phar = new PharData($tgz);
            $phar->extractTo($dir, null, true);
            unset($phar);
        } catch (Throwable $e) {
            fd_log('cloudflared tgz extract failed', ['error' => $e->getMessage()]);
        }
    }
    if (!is_file($destBin)) {
        @exec('tar -xzf ' . escapeshellarg($tgz) . ' -C ' . escapeshellarg($dir) . ' 2>/dev/null');
    }
    $found = $destBin;
    if (!is_file($found)) {
        $matches = glob($dir . DIRECTORY_SEPARATOR . 'cloudflared*') ?: [];
        foreach ($matches as $match) {
            if (is_file($match) && !str_ends_with(strtolower($match), '.tgz')) {
                $found = $match;
                break;
            }
        }
    }
    if (!is_file($found)) {
        return false;
    }
    if ($found !== $destBin) {
        @rename($found, $destBin);
    }
    @chmod($destBin, 0755);
    return is_file($destBin);
}

function fd_ensure_cloudflared(): array
{
    // First check system PATH or Program Files
    $which = fd_tunnel_which();
    if ($which !== '' && is_file($which)) {
        return [$which, ''];
    }

    $commonWindowsPaths = [
        'C:\\Program Files (x86)\\cloudflared\\cloudflared.exe',
        'C:\\Program Files\\cloudflared\\cloudflared.exe',
    ];
    if (fd_is_windows()) {
        foreach ($commonWindowsPaths as $cp) {
            if (is_file($cp)) {
                return [$cp, ''];
            }
        }
    }

    $local = fd_tunnel_bin_path();
    if (is_file($local) && filesize($local) > 1024) {
        if (!fd_is_windows()) {
            @chmod($local, 0755);
        }
        return [$local, ''];
    }

    $asset = fd_tunnel_asset_name();
    $url = 'https://github.com/cloudflare/cloudflared/releases/latest/download/' . $asset;
    $downloadTo = str_ends_with($asset, '.tgz') ? ($local . '.tgz') : $local;
    fd_log('downloading cloudflared', ['url' => $url]);
    if (fd_http_download_file($url, $downloadTo, 180)) {
        if (str_ends_with($asset, '.tgz')) {
            if (!fd_tunnel_extract_tgz($downloadTo, $local)) {
                return ['', 'Downloaded cloudflared archive but failed to extract the binary.'];
            }
            @unlink($downloadTo);
        } elseif (!fd_is_windows()) {
            @chmod($local, 0755);
        }
        if (is_file($local)) {
            return [$local, ''];
        }
    }

    return ['', 'Failed to download cloudflared from GitHub. Check internet access and try again.'];
}

function fd_tunnel_write_dummy_config(string $localUrl = 'http://127.0.0.1:8088'): string
{
    $path = fd_tunnel_config_path();
    $body = "ingress:\n"
        . "  - service: " . $localUrl . "\n";
    @file_put_contents($path, $body);
    return $path;
}

function fd_tunnel_spawn(string $bin, string $localUrl, int $metricsPort = 20241): array
{
    $config = fd_tunnel_write_dummy_config($localUrl);
    $logFile = fd_tunnel_log_path();
    $errLog = fd_tunnel_err_log_path();
    $pidFile = fd_tunnel_pid_path();
    $metrics = '127.0.0.1:' . $metricsPort;
    foreach ([$logFile, $errLog, $pidFile] as $path) {
        if (is_file($path)) {
            @unlink($path);
        }
    }

    if (fd_is_windows()) {
        $binReal = realpath($bin) ?: $bin;
        $configReal = realpath($config) ?: $config;
        $logReal = realpath(dirname($logFile)) ? (realpath(dirname($logFile)) . DIRECTORY_SEPARATOR . basename($logFile)) : $logFile;

        // Use proc_open with bypass_shell = true so Windows executes the binary directly without cmd.exe or powershell dependency
        $cmdLine = escapeshellarg($binReal)
            . ' tunnel --url ' . escapeshellarg($localUrl)
            . ' --config ' . escapeshellarg($configReal)
            . ' --logfile ' . escapeshellarg($logReal)
            . ' --metrics ' . escapeshellarg($metrics)
            . ' --no-autoupdate --retries 99';

        $descriptors = [
            0 => ['pipe', 'r'],
            1 => ['pipe', 'w'],
            2 => ['pipe', 'w'],
        ];

        // Minimal safe environment with TUNNEL_TRANSPORT_PROTOCOL=http2 and system root
        $env = [
            'SystemRoot' => getenv('SystemRoot') ?: 'C:\\Windows',
            'WINDIR' => getenv('WINDIR') ?: 'C:\\Windows',
            'PATH' => getenv('PATH') ?: 'C:\\Windows\\System32;C:\\Windows',
            'TUNNEL_TRANSPORT_PROTOCOL' => 'http2',
            'USERPROFILE' => getenv('USERPROFILE') ?: 'C:\\Users\\ewangtlex',
            'LOCALAPPDATA' => getenv('LOCALAPPDATA') ?: 'C:\\Users\\ewangtlex\\AppData\\Local',
            'APPDATA' => getenv('APPDATA') ?: 'C:\\Users\\ewangtlex\\AppData\\Roaming',
            'TEMP' => sys_get_temp_dir(),
            'TMP' => sys_get_temp_dir(),
        ];

        $pipes = [];
        $proc = @proc_open($cmdLine, $descriptors, $pipes, dirname($binReal), $env, [
            'bypass_shell' => true,
            'suppress_errors' => true,
        ]);

        if (!is_resource($proc)) {
            return [0, 'Failed to spawn cloudflared via proc_open.'];
        }

        $status = proc_get_status($proc);
        $pid = (int) ($status['pid'] ?? 0);

        // Close pipes so the child process is detached and doesn't block
        foreach ($pipes as $p) {
            if (is_resource($p)) {
                @fclose($p);
            }
        }

        if ($pid <= 1) {
            return [0, 'Failed to obtain cloudflared PID.'];
        }

        fd_tunnel_write_pid($pid);
        return [$pid, ''];
    }

    $hasNohup = trim((string) @shell_exec('command -v nohup 2>/dev/null')) !== '';
    $nohupPrefix = $hasNohup ? 'nohup ' : '';
    // setsid fully detaches the child into a new session/process group so it is
    // NOT killed when the spawning PHP request (short-lived under FrankenPHP)
    // ends. This is the key fix for cloudflared dying on Termux/Android.
    $hasSetsid = trim((string) @shell_exec('command -v setsid 2>/dev/null')) !== '';
    $detachPrefix = $hasSetsid ? 'setsid ' : $nohupPrefix;

    if (fd_is_android_runtime() || (!fd_is_windows() && (!is_file('/etc/resolv.conf') || !is_file('/etc/ssl/certs/ca-certificates.crt')))) {
        $resolvBody = "nameserver 1.1.1.1\nnameserver 8.8.8.8\nnameserver 1.0.0.1\nnameserver 8.8.4.4\n";
        $prefixCandidates = array_filter([
            (string) ($_SERVER['PREFIX'] ?? $_ENV['PREFIX'] ?? ''),
            '/data/data/com.pencarimovie.downloader/files/usr',
            '/data/data/com.termux/files/usr',
        ]);
        foreach ($prefixCandidates as $pfx) {
            if (is_dir($pfx . '/etc') && !is_file($pfx . '/etc/resolv.conf')) {
                @file_put_contents($pfx . '/etc/resolv.conf', $resolvBody);
            }
        }
        $appResolv = fd_tunnel_dir() . DIRECTORY_SEPARATOR . 'resolv.conf';
        @file_put_contents($appResolv, $resolvBody);

        // Locate CA certificates bundle in Android / Termux / system paths
        $caCertPath = '';
        $caCandidates = [
            '/data/data/com.termux/files/usr/etc/tls/cert.pem',
            '/data/data/com.pencarimovie.downloader/files/usr/etc/tls/cert.pem',
            '/data/data/com.termux/files/usr/etc/ssl/certs/ca-certificates.crt',
            '/data/data/com.pencarimovie.downloader/files/usr/etc/ssl/certs/ca-certificates.crt',
            '/system/etc/security/cacerts',
            '/etc/ssl/certs/ca-certificates.crt',
            '/etc/ssl/cert.pem',
        ];
        foreach ($prefixCandidates as $pfx) {
            $caCandidates[] = $pfx . '/etc/tls/cert.pem';
            $caCandidates[] = $pfx . '/etc/ssl/certs/ca-certificates.crt';
        }
        foreach ($caCandidates as $cand) {
            if (is_file($cand) && filesize($cand) > 1024) {
                $caCertPath = $cand;
                break;
            }
        }

        $prootBin = '';
        foreach (['/data/data/com.pencarimovie.downloader/files/usr/bin/proot', '/data/data/com.termux/files/usr/bin/proot'] as $pb) {
            if (is_file($pb) && is_executable($pb)) {
                $prootBin = $pb;
                break;
            }
        }
        if ($prootBin === '') {
            $whichProot = trim((string) @shell_exec('command -v proot 2>/dev/null'));
            if ($whichProot !== '' && is_file($whichProot)) {
                $prootBin = $whichProot;
            }
        }

        if ($prootBin !== '') {
            $prootBinds = '-b ' . escapeshellarg($appResolv . ':/etc/resolv.conf');
            if ($caCertPath !== '') {
                $prootBinds .= ' -b ' . escapeshellarg($caCertPath . ':/etc/ssl/certs/ca-certificates.crt')
                    . ' -b ' . escapeshellarg($caCertPath . ':/etc/ssl/cert.pem')
                    . ' -b ' . escapeshellarg($caCertPath . ':/etc/pki/tls/certs/ca-bundle.crt');
            }
            if (is_dir('/system/etc/security/cacerts')) {
                $prootBinds .= ' -b /system/etc/security/cacerts:/system/etc/security/cacerts';
            }

            $sslEnv = '';
            if ($caCertPath !== '') {
                $sslEnv = 'SSL_CERT_FILE=' . escapeshellarg($caCertPath) . ' SSL_CERT_DIR=' . escapeshellarg(dirname($caCertPath)) . ' ';
            }

            $cmd = 'TUNNEL_TRANSPORT_PROTOCOL=http2 ' . $sslEnv . $detachPrefix . escapeshellarg($prootBin) . ' --link2symlink -0 '
                . $prootBinds . ' '
                . escapeshellarg($bin)
                . ' tunnel --url ' . escapeshellarg($localUrl)
                . ' --config ' . escapeshellarg($config)
                . ' --logfile ' . escapeshellarg($logFile)
                . ' --metrics ' . escapeshellarg($metrics)
                . ' --edge-ip-version 4'
                . ' --no-autoupdate --retries 99'
                . ' >> ' . escapeshellarg($errLog)
                . ' 2>&1 & echo $!';
            $pid = (int) trim((string) @shell_exec($cmd));
            if ($pid > 1) {
                fd_tunnel_write_pid($pid);
                return [$pid, ''];
            }
        }
    }

    $sslEnv = '';
    if (!empty($caCertPath) && is_file($caCertPath)) {
        $sslEnv = 'SSL_CERT_FILE=' . escapeshellarg($caCertPath) . ' SSL_CERT_DIR=' . escapeshellarg(dirname($caCertPath)) . ' ';
    }

    $cmd = 'TUNNEL_TRANSPORT_PROTOCOL=http2 ' . $sslEnv . $detachPrefix . escapeshellarg($bin)
        . ' tunnel --url ' . escapeshellarg($localUrl)
        . ' --config ' . escapeshellarg($config)
        . ' --logfile ' . escapeshellarg($logFile)
        . ' --metrics ' . escapeshellarg($metrics)
        . ' --edge-ip-version 4'
        . ' --no-autoupdate --retries 99'
        . ' >> ' . escapeshellarg($errLog)
        . ' 2>&1 & echo $!';
    $pid = (int) trim((string) @shell_exec($cmd));
    if ($pid <= 1) {
        return [0, 'Failed to start cloudflared. proc/shell may be disabled on this runtime.'];
    }
    fd_tunnel_write_pid($pid);
    return [$pid, ''];
}

function fd_tunnel_wait_for_url(int $timeoutSec = 90, int $metricsPort = 0, int $pid = 0): string
{
    $deadline = microtime(true) + $timeoutSec;
    $fromLogs = '';
    $logSeenAt = 0.0;
    while (microtime(true) < $deadline) {
        $live = fd_tunnel_read_quicktunnel_url($pid, $metricsPort);
        if ($live !== '') {
            return $live;
        }
        $fromLogs = fd_tunnel_parse_url_from_text(fd_tunnel_read_logs());
        if ($fromLogs !== '' && $logSeenAt === 0.0) {
            $logSeenAt = microtime(true);
        }
        if ($fromLogs !== '' && (microtime(true) - $logSeenAt) >= 8) {
            return $fromLogs;
        }
        usleep(250000);
    }
    $live = fd_tunnel_read_quicktunnel_url($pid, $metricsPort);
    return $live !== '' ? $live : $fromLogs;
}

/**
 * Watchdog: if the tunnel state says it should be enabled but cloudflared is no
 * longer running (e.g. killed by Android/Termux process management, network
 * drop, or the spawning request ending), re-spawn it automatically.
 *
 * A cooldown (default 20s) prevents restart loops when cloudflared cannot
 * actually start. Returns true if a restart was attempted.
 */
function fd_tunnel_auto_restart(): bool
{
    $state = fd_load_tunnel_state();
    if (empty($state['enabled'])) {
        return false;
    }

    $pid = (int) ($state['pid'] ?? 0);
    if ($pid > 1 && fd_tunnel_pid_alive($pid)) {
        return false;
    }

    // Cooldown to avoid tight restart loops.
    $lastAttempt = (int) ($state['restart_attempt_at'] ?? 0);
    if ($lastAttempt > 0 && (time() - $lastAttempt) < 20) {
        return false;
    }

    $bin = (string) ($state['bin'] ?? '');
    if ($bin === '' || !is_file($bin)) {
        [$bin,] = fd_ensure_cloudflared();
    }
    if ($bin === '') {
        return false;
    }

    $port = (int) ($state['local_port'] ?? fd_get_listen_port());
    $metricsPort = (int) ($state['metrics_port'] ?? 0);
    if ($metricsPort <= 0) {
        $metricsPort = fd_tunnel_pick_metrics_port();
    }

    $localUrl = 'http://127.0.0.1:' . $port;
    fd_tunnel_kill_leftovers();

    [$newPid, $spawnError] = fd_tunnel_spawn($bin, $localUrl, $metricsPort);
    if ($newPid <= 1) {
        $state['restart_attempt_at'] = time();
        fd_save_tunnel_state($state);
        fd_log('tunnel auto-restart failed', ['error' => $spawnError]);
        return false;
    }

    $url = fd_tunnel_wait_for_url(60, $metricsPort, $newPid);
    if ($url === '') {
        $state['restart_attempt_at'] = time();
        fd_save_tunnel_state($state);
        fd_tunnel_kill_leftovers();
        fd_log('tunnel auto-restart: no URL appeared');
        return false;
    }

    $live = fd_tunnel_read_quicktunnel_url($newPid, $metricsPort);
    if ($live !== '') {
        $url = $live;
    }

    // Re-register with the relay worker (new quick-tunnel URL).
    fd_tunnel_register_worker($url);

    $state['enabled'] = true;
    $state['pid'] = $newPid;
    $state['tunnel_url'] = $url;
    $state['metrics_port'] = $metricsPort;
    $state['started_at'] = time();
    $state['restart_attempt_at'] = time();
    fd_save_tunnel_state($state);
    fd_tunnel_write_pid($newPid);

    fd_log('tunnel auto-restarted', ['pid' => $newPid, 'url' => $url]);
    return true;
}

function fd_get_tunnel_status(): array
{
    $state = fd_load_tunnel_state();
    $pid = fd_tunnel_read_pid();
    $alive = $pid > 1 && fd_tunnel_pid_alive($pid);

    // Watchdog: if the state says enabled but cloudflared died, revive it.
    if (!$alive && !empty($state['enabled'])) {
        fd_tunnel_auto_restart();
        $state = fd_load_tunnel_state();
        $pid = fd_tunnel_read_pid();
        $alive = $pid > 1 && fd_tunnel_pid_alive($pid);
    }

    $url = trim((string) ($state['tunnel_url'] ?? ''));
    $metricsPort = (int) ($state['metrics_port'] ?? 0);

    if ($alive) {
        $live = fd_tunnel_read_quicktunnel_url($pid, $metricsPort);
        if ($live !== '') {
            $url = $live;
        } elseif ($url === '') {
            $url = fd_tunnel_parse_url_from_text(fd_tunnel_read_logs());
        }

        if ($url !== '' && (($state['tunnel_url'] ?? '') !== $url || (int) ($state['pid'] ?? 0) !== $pid || empty($state['enabled']))) {
            $state['enabled'] = true;
            $state['pid'] = $pid;
            $state['tunnel_url'] = $url;
            $state['local_port'] = (int) ($state['local_port'] ?? fd_get_listen_port());
            if ($metricsPort > 0) {
                $state['metrics_port'] = $metricsPort;
            }
            if (empty($state['started_at'])) {
                $state['started_at'] = time();
            }
            fd_save_tunnel_state($state);

            // Ensure the new URL is immediately registered with VPS Redis / relay
            fd_tunnel_register_worker($url);
        }
    }

    $enabled = $alive && $url !== '';
    $subdomain = fd_tunnel_subdomain();
    $publicDomainUrl = $enabled ? ('https://' . $subdomain . '-tunnel.pencarimovie.com') : '';
    $activeUrl = ($publicDomainUrl !== '') ? $publicDomainUrl : $url;
    $manifestUrl = $enabled ? (rtrim($activeUrl, '/') . '/manifest.json') : '';

    return [
        'ok' => 1,
        'enabled' => $enabled,
        'running' => $alive,
        'pid' => $alive ? $pid : 0,
        'tunnel_url' => $enabled ? $url : '',
        'public_url' => $publicDomainUrl,
        'device_id' => fd_get_device_id(),
        'manifest_url' => $manifestUrl,
        'local_port' => (int) ($state['local_port'] ?? fd_get_listen_port()),
        'started_at' => (int) ($state['started_at'] ?? 0),
        'message' => $enabled
            ? ('Live on custom subdomain: ' . $publicDomainUrl)
            : ($alive ? 'cloudflared is running but the public URL is not ready yet.' : 'Cloudflare tunnel is off.'),
    ];
}

function fd_disable_tunnel(): array
{
    // Invalidate registration on VPS Redis
    @fd_tunnel_register_worker('');
    fd_tunnel_kill_leftovers();
    fd_clear_tunnel_state();
    return [
        'ok' => 1,
        'enabled' => false,
        'running' => false,
        'pid' => 0,
        'tunnel_url' => '',
        'manifest_url' => '',
        'message' => 'Cloudflare tunnel stopped.',
    ];
}

function fd_enable_tunnel(): array
{
    @set_time_limit(180);
    if (function_exists('ignore_user_abort')) {
        ignore_user_abort(true);
    }

    // Always kill leftovers and clear previous state files before starting a new tunnel
    // so old trycloudflare.com log entries / dead sockets are never reused.
    fd_tunnel_kill_leftovers();
    fd_clear_tunnel_state();

    [$bin, $error] = fd_ensure_cloudflared();
    if ($bin === '') {
        return ['ok' => 0, 'enabled' => false, 'message' => $error ?: 'cloudflared is not available.'];
    }

    $port = fd_get_listen_port();
    $localUrl = 'http://127.0.0.1:' . $port;
    $metricsPort = fd_tunnel_pick_metrics_port();
    [$pid, $spawnError] = fd_tunnel_spawn($bin, $localUrl, $metricsPort);
    if ($pid <= 1) {
        return ['ok' => 0, 'enabled' => false, 'message' => $spawnError ?: 'Failed to start cloudflared.'];
    }

    $url = fd_tunnel_wait_for_url(90, $metricsPort, $pid);
    if ($url === '') {
        $tail = substr(fd_tunnel_read_logs(), -1200);
        fd_tunnel_kill_leftovers();
        fd_clear_tunnel_state();
        $hint = $tail !== '' ? ' Log: ' . trim(preg_replace('/\s+/', ' ', $tail)) : '';
        return [
            'ok' => 0,
            'enabled' => false,
            'message' => 'cloudflared started but no trycloudflare.com URL appeared.' . $hint,
        ];
    }

    $live = fd_tunnel_read_quicktunnel_url($pid, $metricsPort);
    if ($live !== '') {
        $url = $live;
    }

    // Register with the *.pencarimovie.com relay worker
    fd_tunnel_register_worker($url);

    $subdomain = fd_tunnel_subdomain();
    $publicDomainUrl = 'https://' . $subdomain . '-tunnel.pencarimovie.com';

    $state = [
        'enabled' => true,
        'pid' => $pid,
        'tunnel_url' => $url,
        'public_url' => $publicDomainUrl,
        'local_port' => $port,
        'metrics_port' => $metricsPort,
        'started_at' => time(),
        'bin' => $bin,
    ];
    fd_save_tunnel_state($state);
    fd_tunnel_write_pid($pid);

    return [
        'ok' => 1,
        'enabled' => true,
        'running' => fd_tunnel_pid_alive($pid),
        'pid' => $pid,
        'tunnel_url' => $url,
        'public_url' => $publicDomainUrl,
        'manifest_url' => rtrim($publicDomainUrl, '/') . '/manifest.json',
        'local_port' => $port,
        'started_at' => $state['started_at'],
        'message' => 'Tunnel is live at ' . $publicDomainUrl,
    ];
}

function fd_stremio_json(array $data, int $status = 200, ?string $cacheControl = null): never
{
    if (!headers_sent()) {
        http_response_code($status);
        header('Content-Type: application/json; charset=utf-8');
        header('Access-Control-Allow-Origin: *');
        header('Access-Control-Allow-Methods: GET, OPTIONS');
        header('Access-Control-Allow-Headers: *');
        if ($cacheControl !== null) {
            header('Cache-Control: ' . $cacheControl);
        } else {
            header('Cache-Control: no-cache, no-store, must-revalidate');
        }
    }
    echo json_encode($data, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
    exit;
}

// ─── Routing ─────────────────────────────────────────────────────────────────

$path = parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH) ?: '/';
$method = $_SERVER['REQUEST_METHOD'] ?? 'GET';

// ─── Nuvio Addon Routes ──────────────────────────────────────────────────────
// Support /nuvio, /stremio (alias redirect), /configure, and root level (/manifest.json, /catalog/..., /meta/..., /stream/...)
$isNuvioRoute = ($path === '/nuvio' || str_starts_with($path, '/nuvio/')) ||
    ($path === '/stremio' || str_starts_with($path, '/stremio/')) ||
    $path === '/configure' || $path === '/configure/' ||
    $path === '/manifest.json' ||
    preg_match('#^/(catalog|meta|stream)/#', $path);

if ($isNuvioRoute) {
    // Handle redirect for legacy /stremio to /nuvio
    if ($path === '/stremio' || $path === '/stremio/') {
        header('Location: /nuvio', true, 301);
        exit;
    }

    // Handle Stremio's standard /configure route -> redirects directly to dashboard with #configure
    if ($path === '/configure' || $path === '/configure/' || $addonPath === '/configure' || $addonPath === '/configure/') {
        header('Location: /#configure', true, 302);
        exit;
    }

    // Normalize path by stripping /nuvio or /stremio prefix if present so internal matching is uniform
    $addonPath = preg_replace('#^/(nuvio|stremio)#', '', $path);
    if ($addonPath === '') {
        $addonPath = '/';
    }
    if ($method === 'OPTIONS') {
        header('Access-Control-Allow-Origin: *');
        header('Access-Control-Allow-Methods: GET, OPTIONS');
        header('Access-Control-Allow-Headers: *');
        exit;
    }

    $baseUrl = fd_get_stremio_base_url();

    // ── Nuvio Addon Installation / Landing Page ──
    if ($addonPath === '/' && ($path === '/nuvio' || $path === '/nuvio/')) {
        header('Content-Type: text/html; charset=utf-8');
        $randSuffix = '?r=' . random_int(100000, 999999);
        $manifestUrl = $baseUrl . '/manifest.json' . $randSuffix;
        $lanIp = fd_get_lan_ip();
        $requestHost = (string) (parse_url($baseUrl, PHP_URL_HOST) ?? ($_SERVER['SERVER_ADDR'] ?? '127.0.0.1'));
        $openedViaLan = fd_is_usable_lan_ipv4($requestHost);
        $parsedPort = parse_url($baseUrl, PHP_URL_PORT) ?? ($_SERVER['SERVER_PORT'] ?? '');
        $portSuffix = ($parsedPort !== '' && $parsedPort !== '80' && $parsedPort !== '443') ? (':' . $parsedPort) : '';
        $lanManifestUrl = ($lanIp !== '127.0.0.1') ? preg_replace('#://[^/]+#', '://' . $lanIp . $portSuffix, $manifestUrl) : $manifestUrl;
        $versionCheck = fd_check_version();
        $isOutdated = !empty($versionCheck['update_needed']);
        $updateUrl = $versionCheck['update_url'] ?? 'https://github.com/aiskendi/pencarimovie-server';
        $minVersion = $versionCheck['minimum_version'] ?? '';
        $currentVersion = $versionCheck['current_version'] ?? FD_APP_VERSION;
?>
        <!DOCTYPE html>
        <html lang="en">

        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>PencariMovie Nuvio Addon</title>
            <style>
                * {
                    box-sizing: border-box;
                    margin: 0;
                    padding: 0;
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
                }

                body {
                    background: #141414;
                    color: #fff;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    min-height: 100vh;
                    padding: 20px;
                }

                .card {
                    background: #1f1f1f;
                    border-radius: 14px;
                    padding: 40px;
                    max-width: 540px;
                    width: 100%;
                    text-align: center;
                    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.6);
                    border: 1px solid #2e2e2e;
                }

                .logo {
                    font-size: 2.2rem;
                    font-weight: 800;
                    color: #ff6b35;
                    margin-bottom: 10px;
                    letter-spacing: -0.5px;
                }

                .badge {
                    display: inline-block;
                    background: rgba(255, 107, 53, 0.15);
                    color: #ff6b35;
                    font-size: 0.8rem;
                    font-weight: 700;
                    padding: 4px 12px;
                    border-radius: 20px;
                    margin-bottom: 16px;
                    border: 1px solid rgba(255, 107, 53, 0.3);
                }

                .tagline {
                    color: #b0b0b0;
                    font-size: 1rem;
                    margin-bottom: 24px;
                    line-height: 1.5;
                }

                .manifest-label {
                    text-align: left;
                    font-size: 0.85rem;
                    color: #888;
                    margin-bottom: 6px;
                    font-weight: 600;
                }

                .manifest-box {
                    background: #121212;
                    padding: 14px;
                    border-radius: 8px;
                    font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
                    font-size: 0.9rem;
                    color: #00d26a;
                    word-break: break-all;
                    margin-bottom: 14px;
                    border: 1px solid #2a2a2a;
                    text-align: left;
                    user-select: all;
                }

                .btn {
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                    gap: 10px;
                    background: #ff6b35;
                    color: #fff;
                    text-decoration: none;
                    padding: 14px 24px;
                    border-radius: 8px;
                    font-weight: 700;
                    font-size: 1.05rem;
                    transition: all 0.2s;
                    margin-bottom: 20px;
                    width: 100%;
                    border: none;
                    cursor: pointer;
                }

                .btn:hover {
                    background: #ff824d;
                    transform: translateY(-1px);
                    box-shadow: 0 4px 14px rgba(255, 107, 53, 0.35);
                }

                .instructions {
                    background: #181818;
                    border-radius: 10px;
                    padding: 20px;
                    text-align: left;
                    border: 1px solid #282828;
                }

                .instructions-title {
                    font-size: 0.95rem;
                    font-weight: 700;
                    color: #fff;
                    margin-bottom: 12px;
                    display: flex;
                    align-items: center;
                    gap: 8px;
                }

                .instructions ol {
                    margin-left: 20px;
                    color: #aaa;
                    font-size: 0.88rem;
                    line-height: 1.6;
                }

                .instructions li {
                    margin-bottom: 8px;
                }

                .instructions li strong {
                    color: #eee;
                }

                .status-copied {
                    display: none;
                    background: rgba(46, 204, 113, 0.15);
                    color: #2ecc71;
                    padding: 8px 12px;
                    border-radius: 6px;
                    font-size: 0.85rem;
                    margin-bottom: 16px;
                    border: 1px solid rgba(46, 204, 113, 0.3);
                }
            </style>
        </head>

        <body>
            <div class="card">
                <div class="logo">PencariMovie</div>
                <div class="badge">NUVIO ADDON</div>

                <?php if ($isOutdated): ?>
                    <div style="background: rgba(231, 76, 60, 0.15); border: 1px solid rgba(231, 76, 60, 0.4); border-radius: 8px; padding: 14px; margin-bottom: 20px; text-align: left;">
                        <div style="font-weight: 700; color: #ff6b6b; margin-bottom: 6px; font-size: 0.95rem; display: flex; align-items: center; gap: 6px;">
                            ⚠️ Update Required
                        </div>
                        <div style="color: #ddd; font-size: 0.85rem; line-height: 1.4; margin-bottom: 10px;">
                            Your app version (<strong>v<?= htmlspecialchars($currentVersion) ?></strong>) is outdated. Minimum version is <strong>v<?= htmlspecialchars($minVersion) ?></strong>.
                        </div>
                        <a href="<?= htmlspecialchars($updateUrl) ?>" target="_blank" rel="noopener noreferrer" style="display: inline-block; background: #e74c3c; color: #fff; text-decoration: none; padding: 8px 14px; border-radius: 6px; font-size: 0.85rem; font-weight: 700;">
                            ⬇️ Download Update
                        </a>
                    </div>
                <?php endif; ?>

                <div class="tagline">Stream movies, series from Telegram server directly in Nuvio</div>

                <div class="manifest-label" style="display: flex; justify-content: space-between; align-items: center;">
                    <span>📡 Wi-Fi / LAN Manifest URL (For TV, Phone, Tablet):</span>
                    <span style="font-size: 0.72rem; background: rgba(0, 210, 106, 0.15); color: #00d26a; padding: 2px 8px; border-radius: 10px; font-weight: 600;">Recommended</span>
                </div>
                <div class="manifest-box" id="mUrl"><?= htmlspecialchars($lanManifestUrl) ?></div>

                <div id="copiedNotice" class="status-copied">✓ Copied Wi-Fi / LAN URL to clipboard!</div>

                <button class="btn" onclick="copyManifestUrl('mUrl', '✓ Copied Wi-Fi / LAN URL to clipboard!')">
                    📋 Copy Wi-Fi / LAN Manifest URL
                </button>

                <?php if (!$openedViaLan): ?>
                    <div class="manifest-label" style="display: flex; justify-content: space-between; align-items: center; margin-top: 18px;">
                        <span>💻 Localhost Manifest URL (This Device Only):</span>
                        <span style="font-size: 0.72rem; background: rgba(255, 255, 255, 0.08); color: #aaa; padding: 2px 8px; border-radius: 10px; font-weight: 600;">Localhost</span>
                    </div>
                    <div class="manifest-box" id="mUrlLocal" style="color: #bbb; border-color: #333;"><?= htmlspecialchars($manifestUrl) ?></div>
                    <button class="btn" style="background: #2a2a2a; border: 1px solid #3a3a3a; margin-bottom: 20px;" onclick="copyManifestUrl('mUrlLocal', '✓ Copied Local URL to clipboard!')">
                        📋 Copy Localhost Manifest URL
                    </button>
                <?php endif; ?>

                <div class="instructions">
                    <div class="instructions-title">🚀 How to install in Nuvio:</div>
                    <ol>
                        <li>Connect your Android TV, phone, or tablet to the <strong>same Wi-Fi network</strong> as this server.</li>
                        <li>Open the <strong>Nuvio</strong> app on your device.</li>
                        <li>Go to <strong>profile</strong> &rarr; <strong>content & discovery</strong> &rarr; <strong>addons</strong>.</li>
                        <li>Paste the <strong>Wi-Fi / LAN Manifest URL</strong> and click <strong>Install addon</strong>.</li>
                    </ol>
                </div>
            </div>

            <script>
                function copyManifestUrl(elementId = 'mUrl', msg = '✓ Copied to clipboard!') {
                    const val = document.getElementById(elementId).textContent.trim();
                    navigator.clipboard.writeText(val);
                    const notice = document.getElementById('copiedNotice');
                    notice.textContent = msg;
                    notice.style.display = 'block';
                    setTimeout(() => {
                        notice.style.display = 'none';
                    }, 3000);
                }
            </script>
        </body>

        </html>
<?php
        exit;
    }

    // ── Nuvio / Stremio Manifest ──
    if ($addonPath === '/manifest.json') {
        // Fetch genre list from WordPress (matching public/app.js)
        $categories = fd_fetch_stream_ajax('categories');
        $defaultCategories = [
            ['name' => 'Animation', 'slug' => 'animation'],
            ['name' => 'Action', 'slug' => 'action'],
            ['name' => 'Comedy', 'slug' => 'comedy'],
            ['name' => 'Drama', 'slug' => 'drama'],
            ['name' => 'Horror', 'slug' => 'horror'],
            ['name' => 'Sci-Fi', 'slug' => 'sci-fi'],
            ['name' => 'Thriller', 'slug' => 'thriller'],
            ['name' => 'Malay', 'slug' => 'malay'],
            ['name' => 'Indo', 'slug' => 'indonesian'],
            ['name' => 'Korean', 'slug' => 'korean'],
        ];

        $categoryList = (!empty($categories) && is_array($categories)) ? $categories : $defaultCategories;

        // Pure Film & TV Genres for Stremio filter dropdown
        $allGenreOptions = [
            'Action',
            'Adventure',
            'Animation',
            'Anime',
            'Biography',
            'Comedy',
            'Crime',
            'Documentary',
            'Drama',
            'Family',
            'Fantasy',
            'History',
            'Horror',
            'Music',
            'Musical',
            'Mystery',
            'Romance',
            'Sci-Fi',
            'Sport',
            'Thriller',
            'War',
            'Western',
        ];

        // Release Years for Stremio / Nuvio Discover filter dropdown
        $currentYear = (int) date('Y');
        $allYearOptions = [];
        for ($y = $currentYear; $y >= 2000; $y--) {
            $allYearOptions[] = (string) $y;
        }

        // Catalog order is the Nuvio/Stremio home-row order.
        // Latest Releases is first so it appears at the top of each type.
        $manifestCatalogs = [
            // Movies Catalogs
            [
                'type' => 'movie',
                'id' => 'top',
                'name' => 'Popular',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'year',
                'name' => 'New',
                'genres' => $allYearOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allYearOptions, 'isRequired' => true],
                    ['name' => 'skip', 'isRequired' => false],
                ],
                'extraSupported' => ['genre', 'skip'],
                'extraRequired' => ['genre'],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_search_movie',
                'name' => 'Search Movies',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'search', 'isRequired' => true],
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'other',
                'id' => 'pm_files_year',
                'name' => 'New',
                'genres' => $allYearOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allYearOptions, 'isRequired' => true],
                    ['name' => 'skip', 'isRequired' => false],
                ],
                'extraSupported' => ['genre', 'skip'],
                'extraRequired' => ['genre'],
            ],
            [
                'type' => 'other',
                'id' => 'pm_search_files',
                'name' => 'Telegram Files',
                'genres' => ['4K', '1080p', '720p', 'BluRay', 'WEB-DL', 'HEVC'],
                'extra' => [
                    ['name' => 'search', 'isRequired' => true],
                    ['name' => 'genre', 'options' => ['4K', '1080p', '720p', 'BluRay', 'WEB-DL', 'HEVC'], 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_malay',
                'name' => 'Malaysia',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_indo',
                'name' => 'Indonesia',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_korean',
                'name' => 'Korea',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_japan',
                'name' => 'Japan',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_anime',
                'name' => 'Anime',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_chinese',
                'name' => 'China / HK',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_thai',
                'name' => 'Thailand',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_bollywood',
                'name' => 'Bollywood',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_philippines',
                'name' => 'Philippines',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'movie',
                'id' => 'pm_movies_english',
                'name' => 'English',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],

            // Series Catalogs
            [
                'type' => 'series',
                'id' => 'pm_series_top',
                'name' => 'Popular',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_year',
                'name' => 'New',
                'genres' => $allYearOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allYearOptions, 'isRequired' => true],
                    ['name' => 'skip', 'isRequired' => false],
                ],
                'extraSupported' => ['genre', 'skip'],
                'extraRequired' => ['genre'],
            ],
            [
                'type' => 'series',
                'id' => 'pm_search_series',
                'name' => 'Search Series',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'search', 'isRequired' => true],
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_kdrama',
                'name' => 'K-Drama',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_anime',
                'name' => 'Anime',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_japan',
                'name' => 'J-Drama',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_malay',
                'name' => 'Malaysia',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_cdrama',
                'name' => 'C-Drama',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_thai',
                'name' => 'Thailand',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_philippines',
                'name' => 'Philippines',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_english',
                'name' => 'English',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
            [
                'type' => 'series',
                'id' => 'pm_series_indo',
                'name' => 'Indonesia',
                'genres' => $allGenreOptions,
                'extra' => [
                    ['name' => 'genre', 'options' => $allGenreOptions, 'isRequired' => false],
                    ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                    ['name' => 'skip', 'isRequired' => false],
                ],
            ],
        ];

        // Add top keyword catalogs to $manifestCatalogs if trending keywords are available
        try {
            $trending = fd_fetch_stream_ajax('trending', ['limit' => 15]);
            if (is_array($trending)) {
                foreach ($trending as $item) {
                    $kw = trim((string) ($item['keyword'] ?? ''));
                    if (!fd_is_topkeyword_valid($kw)) continue;
                    $catId = 'pm_topkw_' . substr(md5(strtolower($kw)), 0, 10);
                    $manifestCatalogs[] = [
                        'type' => 'other',
                        'id' => $catId,
                        'name' => ucwords($kw),
                        'genres' => ['4K', '1080p', '720p', 'BluRay', 'WEB-DL', 'HEVC'],
                        'extra' => [
                            ['name' => 'genre', 'options' => ['4K', '1080p', '720p', 'BluRay', 'WEB-DL', 'HEVC'], 'isRequired' => false],
                            ['name' => 'year', 'options' => $allYearOptions, 'isRequired' => false],
                            ['name' => 'skip', 'isRequired' => false],
                        ],
                    ];
                }
            }
        } catch (\Throwable $e) {
            // Silently skip if trending fetch fails
        }

        $identity = fd_stremio_manifest_identity();

        // Apply Catalog Settings:
        // 1. If catalogs are disabled: empty catalogs array, remove catalog resource (streams list only from tt).
        // 2. If enabled: filter catalogs by enabled types (movie/series) and individual category/country selections.
        $catSettings = fd_load_catalog_settings();
        $filteredCatalogs = [];
        $resources = [];

        if (!empty($catSettings['catalogs_enabled'])) {
            $enabledTypes = $catSettings['enabled_types'] ?? ['movie' => true, 'series' => true, 'other' => true];
            $enabledCatalogMap = $catSettings['enabled_catalogs'] ?? [];

            $topkwEnabled = !isset($enabledCatalogMap['pm_trending_keywords']) || !empty($enabledCatalogMap['pm_trending_keywords']);

            foreach ($manifestCatalogs as $cat) {
                $cType = $cat['type'] ?? '';
                $cId = $cat['id'] ?? '';

                // If this is a top-keyword catalog, check the master toggle 'pm_trending_keywords'
                if (str_starts_with($cId, 'pm_topkw_')) {
                    if (!$topkwEnabled) {
                        continue;
                    }
                }

                // Check if media type (movie or series or other) is enabled
                if (!empty($enabledTypes[$cType])) {
                    // Check if specific catalog is enabled (defaulting to true if not set)
                    if (!isset($enabledCatalogMap[$cId]) || !empty($enabledCatalogMap[$cId])) {
                        $filteredCatalogs[] = $cat;
                    }
                }
            }

            // Determine active types based on enabled catalogs/types
            $activeTypes = [];
            if (!empty($enabledTypes['movie'])) $activeTypes[] = 'movie';
            if (!empty($enabledTypes['series'])) $activeTypes[] = 'series';
            if (!empty($enabledTypes['other'])) $activeTypes[] = 'other';
            if (empty($activeTypes)) $activeTypes = ['movie', 'series', 'other'];

            if (!empty($filteredCatalogs)) {
                $resources[] = [
                    'name' => 'catalog',
                    'types' => $activeTypes,
                ];
            }
            $resources[] = [
                'name' => 'meta',
                'types' => $activeTypes,
                'idPrefixes' => ['pm_', 'pm:'],
            ];
            $resources[] = [
                'name' => 'stream',
                'types' => ['movie', 'series', 'other'],
                'idPrefixes' => ['pm_', 'pm:', 'tt'],
            ];
        } else {
            // Catalogs disabled: only stream resource remains for tt IDs (and pm_ if directly linked)
            $filteredCatalogs = [];
            $resources[] = [
                'name' => 'stream',
                'types' => ['movie', 'series', 'other'],
                'idPrefixes' => ['pm_', 'pm:', 'tt'],
            ];
        }

        $manifest = [
            'id' => $identity['id'],
            'version' => FD_APP_VERSION,
            'name' => $identity['name'],
            'description' => $identity['description'],
            'resources' => $resources,
            'types' => ['movie', 'series', 'other'],
            'idPrefixes' => ['pm_', 'pm:', 'tt'],
            'catalogs' => $filteredCatalogs,
            'behaviorHints' => [
                'configurable' => true,
                'configurationRequired' => false,
                'adult' => false,
                'p2p' => false,
            ],
        ];

        fd_stremio_json($manifest, 200, 'max-age=3600, public');
    }

    // ── Nuvio Catalog: /catalog/:type/:id[/:extra].json ──
    if (preg_match('#^/catalog/([^/]+)/([^/]+?)(?:/(.*))?\.json$#', $addonPath, $matches)) {
        $catalogType = $matches[1];
        $catalogId = $matches[2];
        $extraStr = $matches[3] ?? '';

        $extra = [];
        if ($extraStr !== '') {
            $decodedExtra = urldecode($extraStr);
            $parsedJson = json_decode($decodedExtra, true);
            if (is_array($parsedJson)) {
                $extra = $parsedJson;
            } else {
                $pairs = explode('&', $decodedExtra);
                foreach ($pairs as $p) {
                    $kv = explode('=', $p, 2);
                    if (count($kv) === 2) {
                        $extra[$kv[0]] = $kv[1];
                    }
                }
            }
        }
        // Also support query string parameters (?skip=...&genre=...&year=...&search=...)
        if (isset($_GET['search'])) $extra['search'] = (string)$_GET['search'];
        if (isset($_GET['genre'])) $extra['genre'] = (string)$_GET['genre'];
        if (isset($_GET['year'])) $extra['year'] = (string)$_GET['year'];
        if (isset($_GET['skip'])) $extra['skip'] = (int)$_GET['skip'];

        $searchQuery = $extra['search'] ?? '';
        $genre = $extra['genre'] ?? '';
        $year = $extra['year'] ?? '';

        // If catalog is a Year catalog (like Cinemeta's year catalog) where 'genre' is a 4-digit year,
        // map it to $year and default to the current year (2026) when not specified.
        $isYearCatalog = ($catalogId === 'year' || $catalogId === 'pm_series_year' || $catalogId === 'pm_files_year');
        if ($isYearCatalog) {
            if ($year === '' && preg_match('/^\d{4}$/', $genre)) {
                $year = $genre;
                $genre = '';
            } elseif ($year === '') {
                $year = (string) date('Y');
            }
        }
        $skip = (int) ($extra['skip'] ?? 0);
        $limit = ($searchQuery !== '') ? 60 : 24;

        $metas = [];

        if ($searchQuery !== '') {
            // Search mode - strictly segregate Movie search vs Series search

            // 1. Search posts
            $searchPosts = fd_fetch_stream_ajax('search', [
                'search' => $searchQuery,
                'limit' => 30,
                'offset' => $skip,
            ]);

            if (is_array($searchPosts)) {
                foreach ($searchPosts as $post) {
                    $pId = $post['id'] ?? 0;
                    if (!$pId) continue;
                    $pTitle = $post['title'] ?? '';
                    $pThumb = $post['thumbnail_url'] ?? '';
                    $pExcerpt = $post['excerpt'] ?? '';
                    $pCats = (array) ($post['categories'] ?? []);

                    $isSeries = preg_match('/tvseries|series|season|episode|drama/i', $pTitle . ' ' . implode(' ', $pCats));

                    // Strict filtering: Movies search only shows movies; Series search only shows series
                    if ($catalogType === 'movie' && $isSeries) {
                        continue;
                    }
                    if ($catalogType === 'series' && !$isSeries) {
                        continue;
                    }

                    $itemType = ($catalogType === 'series' || $isSeries) ? 'series' : 'movie';
                    $cleanName = fd_clean_post_title($pTitle);
                    $releaseYear = fd_extract_release_year($pTitle, (string)($post['date'] ?? ''));
                    $itemGenres = fd_extract_post_genres($post);

                    $metaItem = [
                        'id' => 'pm:post:' . $pId,
                        'type' => $itemType,
                        'name' => $cleanName,
                        'poster' => $pThumb,
                        'posterShape' => 'poster',
                        'description' => $pExcerpt,
                        'genres' => $itemGenres,
                    ];
                    if ($releaseYear !== '') {
                        $metaItem['releaseInfo'] = $releaseYear;
                    }
                    $metas[] = $metaItem;
                }
            }

            // 2. Search direct Telegram files (included in separate pm_search_files catalog)
            if ($catalogId === 'pm_search_files') {
                $metas = []; // Reset metas to ensure direct Telegram files only
                $searchFiles = fd_fetch_stream_ajax('search_files', [
                    'search' => $searchQuery,
                    'limit' => 50,
                    'offset' => $skip,
                ]);

                if (is_array($searchFiles) && isset($searchFiles['files']) && is_array($searchFiles['files'])) {
                    foreach ($searchFiles['files'] as $file) {
                        $fCode = $file['short_code'] ?? '';
                        if ($fCode === '') continue;
                        $fTitle = $file['title'] ?? 'Telegram File';
                        $fThumb = $file['thumbnail_url'] ?? '';
                        $fSize = (int) ($file['file_size'] ?? 0);

                        // Extract resolution & format tags
                        $pills = [];
                        if (preg_match('/\b(2160p|4[kK]|uhd)\b/i', $fTitle)) $pills[] = '4K';
                        elseif (preg_match('/\b(1080p|fhd)\b/i', $fTitle)) $pills[] = '1080p';
                        elseif (preg_match('/\b(720p|hd)\b/i', $fTitle)) $pills[] = '720p';
                        elseif (preg_match('/\b(480p|360p|sd)\b/i', $fTitle)) $pills[] = 'SD';

                        if (preg_match('/\b(bluray|blu-ray|remux)\b/i', $fTitle)) $pills[] = 'BluRay';
                        elseif (preg_match('/\b(web-?dl|webrip)\b/i', $fTitle)) $pills[] = 'WEB-DL';
                        if (preg_match('/\b(hevc|x265|h265)\b/i', $fTitle)) $pills[] = 'HEVC';

                        if ($fSize > 0) $pills[] = fd_format_bytes($fSize);

                        $pillLine = !empty($pills) ? implode(' · ', $pills) : 'Ready to stream';
                        $genres = array_values(array_unique(array_merge(['Direct File'], $pills)));

                        // Apply genre/quality filter if selected (e.g. 4K, 1080p, 720p, BluRay, WEB-DL, HEVC)
                        if ($genre !== '' && !in_array($genre, $genres, true)) {
                            continue;
                        }

                        // Apply year filter if selected (e.g. 2026)
                        if ($year !== '' && !str_contains($fTitle, $year)) {
                            continue;
                        }

                        $metas[] = [
                            'id' => 'pm_file_' . $fCode,
                            'type' => 'other',
                            'name' => $fTitle,
                            'poster' => $fThumb,
                            'posterShape' => 'poster',
                            'description' => "⚡ Direct Telegram File · {$pillLine}\n\n{$fTitle}",
                            'genres' => $genres,
                        ];
                    }
                }
            }
        } elseif ($catalogId === 'pm_files_year' || $catalogId === 'pm_files_latest' || str_starts_with($catalogId, 'pm_topkw_')) {
            // ── Telegram Files Catalogs (Year Files / Top Keywords) ──
            $searchFiles = [];
            // When a quality or year filter is active, fetch extra candidate files to filter down
            $fileFetchLimit = ($genre !== '' || $year !== '') ? 100 : 50;

            if ($catalogId === 'pm_files_year' || $catalogId === 'pm_files_latest') {
                // Fetch newest files from tg_file_new (or search by year if selected)
                $latestSearchTerm = ($year !== '') ? $year : '__latest__';
                $searchFiles = fd_fetch_stream_ajax('search_files', [
                    'search' => $latestSearchTerm,
                    'limit' => $fileFetchLimit,
                    'offset' => $skip,
                ]);
            } else {
                $keywordToSearch = '';
                if (str_starts_with($catalogId, 'pm_topkw_')) {
                    // Look up keyword from options
                    $catOptions = fd_get_default_catalog_options();
                    if (isset($catOptions[$catalogId]['keyword'])) {
                        $keywordToSearch = $catOptions[$catalogId]['keyword'];
                    }
                }

                if ($keywordToSearch !== '') {
                    // Fetch files for this specific keyword
                    $queryTerm = $keywordToSearch;
                    $searchFiles = fd_fetch_stream_ajax('search_files', [
                        'search' => $queryTerm,
                        'limit' => $fileFetchLimit,
                        'offset' => $skip,
                    ]);
                }
            }

            if (is_array($searchFiles) && isset($searchFiles['files']) && is_array($searchFiles['files'])) {
                foreach ($searchFiles['files'] as $file) {
                    $fCode = $file['short_code'] ?? '';
                    if ($fCode === '') continue;
                    $fTitle = $file['title'] ?? 'Telegram File';
                    $fThumb = $file['thumbnail_url'] ?? '';
                    $fSize = (int) ($file['file_size'] ?? 0);

                    // Extract resolution & format tags
                    $pills = [];
                    if (preg_match('/\b(2160p|4[kK]|uhd)\b/i', $fTitle)) $pills[] = '4K';
                    elseif (preg_match('/\b(1080p|fhd)\b/i', $fTitle)) $pills[] = '1080p';
                    elseif (preg_match('/\b(720p|hd)\b/i', $fTitle)) $pills[] = '720p';
                    elseif (preg_match('/\b(480p|360p|sd)\b/i', $fTitle)) $pills[] = 'SD';

                    if (preg_match('/\b(bluray|blu-ray|remux)\b/i', $fTitle)) $pills[] = 'BluRay';
                    elseif (preg_match('/\b(web-?dl|webrip)\b/i', $fTitle)) $pills[] = 'WEB-DL';
                    if (preg_match('/\b(hevc|x265|h265)\b/i', $fTitle)) $pills[] = 'HEVC';

                    if ($fSize > 0) $pills[] = fd_format_bytes($fSize);

                    $pillLine = !empty($pills) ? implode(' · ', $pills) : 'Ready to stream';
                    $primaryLabel = ($catalogId === 'pm_files_year' || $catalogId === 'pm_files_latest') ? 'Year' : 'Trending File';
                    $genres = array_values(array_unique(array_merge([$primaryLabel], $pills)));

                    // Apply genre/quality filter if selected (e.g. 4K, 1080p, 720p, BluRay, WEB-DL, HEVC)
                    if ($genre !== '' && !in_array($genre, $genres, true)) {
                        continue;
                    }

                    // Apply year filter if selected (e.g. 2026)
                    if ($year !== '' && !str_contains($fTitle, $year)) {
                        continue;
                    }

                    $descriptionPrefix = match (true) {
                        $catalogId === 'pm_files_year' || $catalogId === 'pm_files_latest' => "📅 File",
                        str_starts_with($catalogId, 'pm_topkw_') => "Trending File",
                        default => "⚡ Direct Telegram File",
                    };

                    $metas[] = [
                        'id' => 'pm_file_' . $fCode,
                        'type' => 'other',
                        'name' => $fTitle,
                        'poster' => $fThumb,
                        'posterShape' => 'poster',
                        'description' => "{$descriptionPrefix} · {$pillLine}\n\n{$fTitle}",
                        'genres' => $genres,
                    ];
                }
            }
        } else {
            // Browse catalogs. Country + Discover genre + movie/series type are ANDed
            // in WordPress so titles are not dropped after fetch.
            $fetchLimit = min(max($limit, 24), 100);
            $params = [
                'limit' => $fetchLimit,
                'offset' => $skip,
            ];
            if ($catalogType === 'movie' || $catalogType === 'series') {
                $params['media_type'] = $catalogType;
            }

            $catalogCategoryMap = [
                'pm_movies_malay' => 'malay',
                'pm_movies_indo' => 'indonesian',
                'pm_movies_korean' => 'korea',
                'pm_movies_japan' => 'japan',
                'pm_movies_anime' => 'anime',
                'pm_movies_chinese' => 'china',
                'pm_movies_thai' => 'thai',
                'pm_movies_bollywood' => 'bollywood',
                'pm_movies_philippines' => 'filipino',
                'pm_movies_pinoy' => 'filipino',
                'pm_movies_english' => 'english',
                'pm_series_kdrama' => 'korea',
                'pm_series_anime' => 'anime',
                'pm_series_japan' => 'japan',
                'pm_series_malay' => 'malay',
                'pm_series_cdrama' => 'china',
                'pm_series_thai' => 'thai',
                'pm_series_philippines' => 'filipino',
                'pm_series_pinoy' => 'filipino',
                'pm_series_english' => 'english',
                'pm_series_indo' => 'indonesian',
            ];

            if ($catalogId === 'top' || $catalogId === 'pm_series_top') {
                // Popular releases - derived from top search keywords filtered by user country
                $params['category'] = 'popular';
                $detectedCountry = fd_detect_country();
                if (!empty($detectedCountry['country_code'])) {
                    $params['country'] = $detectedCountry['country_code'];
                }
            } elseif ($catalogId === 'year' || $catalogId === 'pm_movies_latest' || $catalogId === 'pm_series_year' || $catalogId === 'pm_series_latest') {
                // Latest releases - no country filter; genre extra still applies.
                $params['category'] = '';
            } elseif (isset($catalogCategoryMap[$catalogId])) {
                $params['category'] = $catalogCategoryMap[$catalogId];
            } elseif (str_starts_with($catalogId, 'pm_cat_')) {
                $catSlug = substr($catalogId, strlen('pm_cat_'));
                if ($catSlug === 'indo') $catSlug = 'indonesian';
                $params['category'] = $catSlug;
            }

            if ($genre !== '') {
                $params['genre'] = $genre;
            }
            if ($year !== '') {
                $params['year'] = $year;
            }

            $posts = fd_fetch_stream_ajax('posts', $params);
            if (is_array($posts)) {
                foreach ($posts as $post) {
                    $pId = $post['id'] ?? 0;
                    if (!$pId) {
                        continue;
                    }
                    $pTitle = $post['title'] ?? '';
                    $pThumb = $post['thumbnail_url'] ?? '';
                    $pExcerpt = $post['excerpt'] ?? '';
                    $itemType = ($catalogType === 'series') ? 'series' : 'movie';
                    $cleanName = fd_clean_post_title($pTitle);
                    $releaseYear = fd_extract_release_year($pTitle, (string)($post['date'] ?? ''));
                    $itemGenres = fd_extract_post_genres($post);

                    $metaItem = [
                        'id' => 'pm:post:' . $pId,
                        'type' => $itemType,
                        'name' => $cleanName,
                        'poster' => $pThumb,
                        'posterShape' => 'poster',
                        'description' => $pExcerpt,
                        'genres' => $itemGenres,
                    ];
                    if ($releaseYear !== '') {
                        $metaItem['releaseInfo'] = $releaseYear;
                    }
                    $metas[] = $metaItem;
                }
            }
        }

        fd_stremio_json(['metas' => $metas], 200, 'max-age=600, public');
    }

    // ── Nuvio Meta: /meta/:type/:id.json ──
    if (preg_match('#^/meta/([^/]+)/([^/]+?)(?:\.json)?$#', $addonPath, $matches)) {
        $itemType = urldecode($matches[1]);
        $itemId = urldecode(urldecode($matches[2])); // Handle double-encoded IDs from web clients

        fd_log('stremio meta request received', [
            'itemType' => $itemType,
            'itemId' => $itemId,
            'clientIp' => $_SERVER['REMOTE_ADDR'] ?? '',
            'userAgent' => $_SERVER['HTTP_USER_AGENT'] ?? '',
        ]);

        // Format 1: Direct tg_file_new (pm_file_SHORT_CODE, pm:file_SHORT_CODE, or pm:file:SHORT_CODE)
        if (str_starts_with($itemId, 'pm_file_') || str_starts_with($itemId, 'pm:file_') || str_starts_with($itemId, 'pm:file:')) {
            if (str_starts_with($itemId, 'pm_file_')) {
                $shortCode = substr($itemId, strlen('pm_file_'));
            } elseif (str_starts_with($itemId, 'pm:file_')) {
                $shortCode = substr($itemId, strlen('pm:file_'));
            } else {
                $shortCode = substr($itemId, strlen('pm:file:'));
            }
            $botId = fd_get_bot_id();

            $title = '';
            $thumb = '';
            $size = 0;
            $fileType = '';

            // First check search_files directly (now handles exact short_code lookup via Manticore)
            $sf = fd_fetch_stream_ajax('search_files', ['search' => $shortCode, 'limit' => 1]);
            if (is_array($sf) && !empty($sf['files'])) {
                foreach ($sf['files'] as $f) {
                    if (($f['short_code'] ?? '') === $shortCode || count($sf['files']) === 1) {
                        $title = (string) ($f['title'] ?? '');
                        $thumb = (string) ($f['thumbnail_url'] ?? '');
                        $size = (int) ($f['file_size'] ?? 0);
                        $fileType = (string) ($f['file_type'] ?? ($f['extension'] ?? ''));
                        break;
                    }
                }
            }

            // If missing metadata or thumbnail, call resolve_shortcode
            if ($title === '' || $thumb === '' || $size === 0) {
                $res = fd_resolve_shortcode($shortCode, $botId);
                if ($title === '' && !empty($res['title'])) $title = (string) $res['title'];
                if ($thumb === '' && !empty($res['thumbnail_url'])) $thumb = (string) $res['thumbnail_url'];
                if ($size === 0 && !empty($res['file_size'])) $size = (int) $res['file_size'];
                if ($fileType === '' && !empty($res['file_type'])) $fileType = (string) $res['file_type'];
            }

            if ($title === '') {
                $title = 'File ' . $shortCode;
            }

            $cleanTitle = fd_clean_media_title($title);
            if ($cleanTitle === '') $cleanTitle = $title;

            // Extract tags & details for clean description
            $pills = [];
            if (preg_match('/\b(2160p|4[kK]|uhd)\b/i', $title)) $pills[] = '4K UHD';
            elseif (preg_match('/\b(1080p|fhd)\b/i', $title)) $pills[] = '1080p';
            elseif (preg_match('/\b(720p|hd)\b/i', $title)) $pills[] = '720p';
            elseif (preg_match('/\b(480p|360p|sd)\b/i', $title)) $pills[] = 'SD';

            if (preg_match('/\b(bluray|blu-ray|remux)\b/i', $title)) $pills[] = 'BluRay';
            elseif (preg_match('/\b(web-?dl|webrip)\b/i', $title)) $pills[] = 'WEB-DL';
            elseif (preg_match('/\b(hdtv|tvrip)\b/i', $title)) $pills[] = 'HDTV';

            if (preg_match('/\b(hdr10\+|hdr10|hdr|dolby\s*vision|dovi|dv)\b/i', $title)) $pills[] = 'HDR';
            if (preg_match('/\b(hevc|x265|h265)\b/i', $title)) $pills[] = 'HEVC';
            elseif (preg_match('/\b(avc|x264|h264)\b/i', $title)) $pills[] = 'AVC';

            if ($size > 0) $pills[] = fd_format_bytes($size);

            $pillLine = !empty($pills) ? implode('  •  ', $pills) : 'Ready to stream';
            $genres = array_values(array_unique(array_merge(['Direct Telegram File'], $pills)));

            $meta = [
                'id' => $itemId,
                'type' => 'other',
                'name' => $cleanTitle,
                'poster' => $thumb,
                'posterShape' => 'poster',
                'background' => $thumb,
                'logo' => $thumb,
                'description' => "⚡ Direct Telegram Cloud File\n" . $pillLine . "\n\n📄 File: " . $cleanTitle,
                'genres' => $genres,
            ];

            fd_stremio_json(['meta' => $meta], 200, 'max-age=600, public');
        }

        // Format 2: WordPress Post (pm_post_POST_ID, pm:post_POST_ID, or pm:post:POST_ID)
        if (str_starts_with($itemId, 'pm_post_') || str_starts_with($itemId, 'pm:post_') || str_starts_with($itemId, 'pm:post:')) {
            if (str_starts_with($itemId, 'pm_post_')) {
                $postId = (int) substr($itemId, strlen('pm_post_'));
            } elseif (str_starts_with($itemId, 'pm:post_')) {
                $postId = (int) substr($itemId, strlen('pm:post_'));
            } else {
                $postId = (int) substr($itemId, strlen('pm:post:'));
            }
            $postData = fd_fetch_stream_ajax('get_post', ['post_id' => $postId]);
            $post = !empty($postData) && is_array($postData) ? ($postData[0] ?? $postData) : [];

            $title = $post['title'] ?? 'PencariMovie Media';
            $thumb = $post['thumbnail_url'] ?? '';
            $excerpt = $post['excerpt'] ?? ($post['content'] ?? '');
            $cats = (array) ($post['categories'] ?? []);
            $tags = (array) ($post['tags'] ?? []);

            // Probe S01, S02, ... so later-season ranker boost cannot hide S1.
            $files = $itemType === 'series'
                ? fd_fetch_series_episode_files($postId)
                : fd_fetch_post_files_paged($postId, [
                    'page_size' => 50,
                    'max_files' => 80,
                ]);

            $isSeries = ($itemType === 'series') || preg_match('/tvseries|series|season|episode|drama/i', $title . ' ' . implode(' ', $cats));
            $resolvedType = ($itemType === 'series' || $isSeries) ? 'series' : 'movie';

            fd_log('stremio meta post resolved', [
                'postId' => $postId,
                'title' => $title,
                'resolvedType' => $resolvedType,
                'categories' => $cats,
                'tags' => $tags,
                'filesCount' => count($files),
            ]);

            $videos = [];
            if ($resolvedType === 'series' && !empty($files)) {
                $groupedEpisodes = [];
                $epIndex = 1;
                $hasExplicitEp = false;

                // First pass: classify season & episode for all files
                foreach ($files as $f) {
                    $fCode = $f['short_code'] ?? '';
                    if ($fCode === '') continue;
                    $fTitle = $f['title'] ?? ('Episode ' . $epIndex);
                    $fThumb = $f['thumbnail_url'] ?? $thumb;
                    $fCaption = $f['caption'] ?? '';
                    $parsed = fd_classify_season_episode($fTitle, (int)($f['season_num'] ?? 0), (int)($f['episode_num'] ?? 0), $fCaption);
                    $s = $parsed['season'];
                    $e = $parsed['episode'];
                    $eEnd = $parsed['episode_end'] ?? 0;

                    if ($e > 0) {
                        $hasExplicitEp = true;
                    }

                    // Combined packs (E01-E14) stay as one list item.
                    $epList = [$e];

                    foreach ($epList as $targetEp) {
                        // Keep unclassified E0 out of the numbered list until
                        // we know the series has no explicit episodes at all.
                        if ($targetEp <= 0) {
                            $key = "{$s}_0";
                        } else {
                            $key = "{$s}_{$targetEp}";
                        }
                        if (!isset($groupedEpisodes[$key])) {
                            $epTitle = $targetEp > 0 ? ('S' . $s . 'E' . $targetEp) : ('Episode ' . $epIndex);
                            $groupedEpisodes[$key] = [
                                'season' => $s,
                                'episode' => $targetEp,
                                'title' => $epTitle,
                                'thumbnail' => $fThumb,
                                'added_date' => $f['added_date'] ?? null,
                                'raw_index' => $epIndex,
                            ];
                        }
                    }
                    $epIndex++;
                }

                // If no file had explicit episode numbers (e.g. telefilm with multiple qualities,
                // or "COMBINED" season packs like S01.COMBINED / S02.COMBINED), keep ONE episode
                // per distinct season instead of collapsing everything into a single Episode 1.
                if (!$hasExplicitEp && count($groupedEpisodes) > 1) {
                    $newGrouped = [];
                    $idx = 1;
                    foreach ($groupedEpisodes as $key => $item) {
                        $s = max(1, (int) $item['season']);
                        $newGrouped["{$s}_1"] = [
                            'season' => $s,
                            'episode' => 1,
                            'title' => 'S' . $s . 'E1',
                            'thumbnail' => $item['thumbnail'] ?? $thumb,
                            'added_date' => $item['added_date'] ?? null,
                            'raw_index' => $idx,
                        ];
                        $idx++;
                    }
                    $groupedEpisodes = $newGrouped;
                }

                // Drop leftover E0 placeholders once real episode numbers exist
                // for that season (S2_0 used to collide with S2E1 as id :2:1).
                if ($hasExplicitEp) {
                    foreach (array_keys($groupedEpisodes) as $key) {
                        if (str_ends_with($key, '_0')) {
                            unset($groupedEpisodes[$key]);
                        }
                    }
                }

                // Convert grouped episodes to Stremio/Nuvio videos list
                $seenVideoIds = [];
                foreach ($groupedEpisodes as $epInfo) {
                    $s = max(1, (int) $epInfo['season']);
                    $e = $epInfo['episode'] > 0 ? (int) $epInfo['episode'] : 1;
                    $videoId = "pm:post:{$postId}:{$s}:{$e}";
                    if (isset($seenVideoIds[$videoId])) {
                        continue;
                    }
                    $seenVideoIds[$videoId] = true;
                    $relDate = !empty($epInfo['added_date']) && is_numeric($epInfo['added_date'])
                        ? date('Y-m-d\TH:i:s\Z', (int)$epInfo['added_date'])
                        : date('Y-m-d\TH:i:s\Z');
                    $epTitle = $epInfo['title'] !== '' ? $epInfo['title'] : ('S' . $s . 'E' . $e);

                    $videos[] = [
                        'id' => $videoId,
                        'name' => $epTitle,
                        'season' => $s,
                        'episode' => $e,
                        'number' => $e,
                        'released' => $relDate,
                        'thumbnail' => $epInfo['thumbnail'] ?: $thumb,
                        'raw_index' => $epInfo['raw_index'],
                    ];
                }

                // Sort series videos in natural ascending order: Season ASC, Episode ASC
                usort($videos, function ($a, $b) {
                    if ($a['season'] !== $b['season']) {
                        return $a['season'] <=> $b['season'];
                    }
                    if ($a['episode'] !== $b['episode']) {
                        return $a['episode'] <=> $b['episode'];
                    }
                    return $a['raw_index'] <=> $b['raw_index'];
                });

                // Remove temporary raw_index key
                foreach ($videos as &$v) {
                    unset($v['raw_index']);
                }
                unset($v);
            }

            // Ensure series always has at least a fallback episode so Stremio doesn't reject it
            if ($resolvedType === 'series' && empty($videos)) {
                $videos[] = [
                    'id' => "pm:post:{$postId}:1:1",
                    'name' => 'Episode 1',
                    'season' => 1,
                    'episode' => 1,
                    'number' => 1,
                    'released' => date('Y-m-d\TH:i:s\Z'),
                    'thumbnail' => $thumb,
                ];
            }

            $cleanPostTitle = fd_clean_post_title($title);
            $releaseYear = fd_extract_release_year($title, (string)($post['date'] ?? ''));
            $postGenres = fd_extract_post_genres($post);

            $meta = [
                'id' => $itemId,
                'type' => $resolvedType,
                'name' => $cleanPostTitle,
                'poster' => $thumb,
                'posterShape' => 'poster',
                'background' => $thumb,
                'description' => strip_tags((string) $excerpt),
                'genres' => $postGenres,
            ];

            if ($releaseYear !== '') {
                $meta['releaseInfo'] = $releaseYear;
                $meta['year'] = $releaseYear;
            }

            // For Stremio protocol: 'videos' is ONLY provided for 'series'.
            // For 'movie', no 'videos' array is provided, so Stremio shows a single direct Play button without seasons/episodes.
            if ($resolvedType === 'series' && !empty($videos)) {
                $meta['videos'] = $videos;
            }

            fd_log('stremio meta response prepared', [
                'itemId' => $itemId,
                'resolvedType' => $resolvedType,
                'videoCount' => count($meta['videos'] ?? []),
                'genres' => $meta['genres'] ?? [],
            ]);

            fd_stremio_json(['meta' => $meta], 200, 'max-age=600, public');
        }

        fd_log('stremio meta not found', [
            'itemType' => $itemType,
            'itemId' => $itemId,
        ]);
        fd_stremio_json(['meta' => null], 404);
    }

    // ── Nuvio Stream: /stream/:type/:id.json ──
    if (preg_match('#^/stream/([^/]+)/([^/]+?)(?:\.json)?$#', $addonPath, $matches)) {
        $itemType = urldecode($matches[1]);
        $itemId = urldecode(urldecode($matches[2])); // Handle double-encoded IDs from web clients
        $streams = [];

        $botIdStr = fd_get_bot_id();
        $hasSession = fd_has_local_session();

        // Check if an app update is required
        $versionCheck = fd_check_version();
        if (!empty($versionCheck['update_needed'])) {
            $minV = $versionCheck['minimum_version'] ?? '';
            $curV = $versionCheck['current_version'] ?? FD_APP_VERSION;
            $upUrl = $versionCheck['update_url'] ?? 'https://github.com/aiskendi/pencarimovie-server';
            $streams[] = [
                'name' => 'PencariMovie',
                'description' => "Update required (v{$curV} < v{$minV})\nOpen the download page to continue",
                'externalUrl' => $upUrl,
            ];
            fd_stremio_json(['streams' => $streams]);
        }

        // If no bot is connected / bot is disconnected, attempt auto-provisioning first
        if (!$hasSession || $botIdStr === '') {
            $autoProv = fd_auto_provision_guest();
            if ($autoProv && !empty($autoProv['bot_id'])) {
                $hasSession = true;
                $botIdStr = (string) $autoProv['bot_id'];
            } else {
                $streams[] = [
                    'name' => 'PencariMovie',
                    'description' => "Telegram bot not connected\nOpen the dashboard and paste a bot token",
                    'externalUrl' => $baseUrl . '/#settings',
                ];
                fd_stremio_json(['streams' => $streams]);
            }
        }

        // Collect all target files to stream
        $filesToStream = [];

        if (str_starts_with($itemId, 'pm_file_') || str_starts_with($itemId, 'pm:file_') || str_starts_with($itemId, 'pm:file:')) {
            if (str_starts_with($itemId, 'pm_file_')) {
                $fCode = substr($itemId, strlen('pm_file_'));
            } elseif (str_starts_with($itemId, 'pm:file_')) {
                $fCode = substr($itemId, strlen('pm:file_'));
            } else {
                $fCode = substr($itemId, strlen('pm:file:'));
            }
            $fileObj = ['short_code' => $fCode];

            // Resolve file details for instant direct playback with rich metadata
            $sf = fd_fetch_stream_ajax('search_files', ['search' => $fCode, 'limit' => 1]);
            if (is_array($sf) && !empty($sf['files'])) {
                foreach ($sf['files'] as $f) {
                    if (($f['short_code'] ?? '') === $fCode || count($sf['files']) === 1) {
                        $fileObj['title'] = (string) ($f['title'] ?? '');
                        $fileObj['file_size'] = (int) ($f['file_size'] ?? 0);
                        $fileObj['mime'] = (string) ($f['file_type'] ?? 'video/mp4');
                        break;
                    }
                }
            }

            if (empty($fileObj['title']) || empty($fileObj['file_size'])) {
                $res = fd_resolve_shortcode($fCode, $botIdStr);
                if (!empty($res['title'])) $fileObj['title'] = (string) $res['title'];
                if (!empty($res['file_size'])) $fileObj['file_size'] = (int) $res['file_size'];
                if (!empty($res['file_type'])) $fileObj['mime'] = (string) $res['file_type'];
            }

            $filesToStream[] = $fileObj;
        } elseif (preg_match('/^pm[_:]post[_:](\d+):(\d+):(\d+)$/', $itemId, $m)) {
            // Series Episode requested: pm_post_POST_ID:SEASON:EPISODE or pm:post_... or pm:post:...
            $postId = (int) $m[1];
            $targetSeason = (int) $m[2];
            $targetEpisode = (int) $m[3];

            $filesToStream = fd_fetch_episode_stream_files(
                $postId,
                $targetSeason,
                $targetEpisode,
                40
            );

            // Sort episode streams by quality: 4K UHD -> 1080p -> 720p -> SD -> file_size DESC
            usort($filesToStream, function ($a, $b) {
                $getScore = function ($title, $size) {
                    if (preg_match('/\b(2160p|4[kK]|uhd)\b/i', $title)) return 4000000000 + $size;
                    if (preg_match('/\b(1080p|fhd)\b/i', $title)) return 3000000000 + $size;
                    if (preg_match('/\b(720p|hd)\b/i', $title)) return 2000000000 + $size;
                    if (preg_match('/\b(480p|360p|sd)\b/i', $title)) return 1000000000 + $size;
                    return $size;
                };
                return $getScore($b['title'] ?? '', (int)($b['file_size'] ?? 0)) <=> $getScore($a['title'] ?? '', (int)($a['file_size'] ?? 0));
            });
        } elseif (preg_match('/^pm[_:]post[_:](\d+):([a-zA-Z0-9_-]+)$/', $itemId, $m)) {
            // Legacy / direct file short code within post
            $fCode = $m[2];
            $filesToStream[] = ['short_code' => $fCode];
        } elseif (str_starts_with($itemId, 'pm_post_') || str_starts_with($itemId, 'pm:post_') || str_starts_with($itemId, 'pm:post:')) {
            // Whole post requested (e.g. movie post with multiple qualities or video files)
            if (str_starts_with($itemId, 'pm_post_')) {
                $postId = (int) substr($itemId, strlen('pm_post_'));
            } elseif (str_starts_with($itemId, 'pm:post_')) {
                $postId = (int) substr($itemId, strlen('pm:post_'));
            } else {
                $postId = (int) substr($itemId, strlen('pm:post:'));
            }
            $postData = fd_fetch_stream_ajax('get_post', ['post_id' => $postId]);
            $post = !empty($postData) && is_array($postData) ? ($postData[0] ?? $postData) : [];
            $postTitle = $post['title'] ?? '';

            $postFiles = fd_fetch_post_files_paged($postId, [
                'page_size' => 50,
                'max_files' => 80,
            ]);

            // For movie streams, strictly filter files to match post title and year
            if ($itemType === 'movie' || (!empty($postTitle) && !preg_match('/tvseries|series|season|episode|drama/i', $postTitle))) {
                $postYear = null;
                if (preg_match('/\b(19\d\d|20\d\d)\b/', $postTitle, $ym)) {
                    $postYear = $ym[1];
                }

                $cleanTitle = preg_replace('/\s*[•··]\s*.+$/u', '', $postTitle);
                if ($postYear) {
                    $cleanTitle = preg_replace('/\b' . $postYear . '\b/', '', $cleanTitle);
                }
                $cleanTitle = trim(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', $cleanTitle));
                $cleanTitle = trim(preg_replace('/\s+/', ' ', $cleanTitle));
                $postWords = array_values(array_filter(explode(' ', strtolower($cleanTitle)), fn($w) => strlen($w) > 1));

                $matchedFiles = [];
                foreach ($postFiles as $pf) {
                    if (empty($pf['short_code'])) continue;
                    $fTitle = $pf['title'] ?? '';

                    // Exclude series episodes from movie streams
                    if (preg_match('/[sS]\d{1,2}\s*[eE]\d{1,2}|(?:season|episod|episode|ep\.)\s*\d+/i', $fTitle)) {
                        continue;
                    }

                    // Strict year match if both post and file specify a year
                    if ($postYear !== null && preg_match('/\b(19\d\d|20\d\d)\b/', $fTitle, $fym)) {
                        if ($fym[1] !== $postYear) {
                            continue;
                        }
                    }

                    // Strict title word match
                    $cleanFTitle = strtolower(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', $fTitle));
                    $wordsMatch = true;
                    foreach ($postWords as $pw) {
                        if (!str_contains($cleanFTitle, $pw)) {
                            $wordsMatch = false;
                            break;
                        }
                    }
                    if (!$wordsMatch) {
                        continue;
                    }

                    // Exclude sequels / franchise parts unless this file is a split chunk (e.g. part001/005, .001)
                    if (!preg_match('/\b(part\s*\d+|part\s*[ivx]+|\d+)\b/i', $cleanTitle)) {
                        $isSplitPart = preg_match('/[._\s-]part[._\s-]*0*\d{1,4}/i', $fTitle) || preg_match('/\.(?:mp4|mkv)\.0*\d{1,4}$/i', $fTitle);
                        if (!$isSplitPart && preg_match('/\b(part\s*\d+|part\s*[ivx]+)\b/i', $fTitle)) {
                            continue;
                        }
                    }

                    $matchedFiles[] = $pf;
                }

                // If matched files found, use them; otherwise fallback to postFiles
                $postFilesToUse = !empty($matchedFiles) ? $matchedFiles : $postFiles;
                foreach ($postFilesToUse as $pf) {
                    if (!empty($pf['short_code'])) {
                        $filesToStream[] = $pf;
                    }
                }
            } else {
                foreach ($postFiles as $pf) {
                    if (!empty($pf['short_code'])) {
                        $filesToStream[] = $pf;
                    }
                }
            }
        } elseif (preg_match('/^(tt\d{6,10})(?::(\d+):(\d+))?$/i', $itemId, $m)) {
            // External Stremio standard IMDb ID requested (e.g. tt1234567 or tt1234567:1:1 for series)
            $imdbId = $m[1] ?? '';
            $targetSeason = isset($m[2]) ? (int)$m[2] : null;
            $targetEpisode = isset($m[3]) ? (int)$m[3] : null;

            // Resolve title name from Cinemeta (standard Stremio metadata)
            $searchedTitle = '';
            $searchedYear = '';

            if ($imdbId !== '') {
                $cinemetaType = ($targetSeason !== null || $itemType === 'series') ? 'series' : 'movie';
                $cinemetaUrl = "https://v3-cinemeta.strem.io/meta/{$cinemetaType}/{$imdbId}.json";
                $cinemetaJson = fd_http_json($cinemetaUrl, [], 'GET', 5);
                if (!empty($cinemetaJson['meta']['name'])) {
                    $searchedTitle = (string) $cinemetaJson['meta']['name'];
                    $searchedYear = (string) ($cinemetaJson['meta']['year'] ?? '');
                }
            }

            // Query search index with resolved title
            $searchQuery = $searchedTitle !== '' ? $searchedTitle : $itemId;

            if ($searchQuery !== '') {
                if ($targetSeason !== null && $targetEpisode !== null) {
                    $imdbEpisodeFilter = fd_episode_stream_filter($targetSeason, $targetEpisode);

                    // For Series Episode: search title with Season/Episode tokens
                    $epQuery = sprintf('%s S%02dE%02d', $searchQuery, $targetSeason, $targetEpisode);
                    $sf = fd_fetch_stream_ajax('search_files', ['search' => $epQuery, 'limit' => 30]);
                    if (is_array($sf) && !empty($sf['files'])) {
                        foreach ($sf['files'] as $f) {
                            if ($imdbEpisodeFilter($f)) {
                                $filesToStream[] = $f;
                            }
                        }
                    }

                    // Fallback to searching without SxxExx if few files found
                    if (count($filesToStream) === 0) {
                        $epQueryAlt = sprintf('%s E%02d', $searchQuery, $targetEpisode);
                        $sfAlt = fd_fetch_stream_ajax('search_files', ['search' => $epQueryAlt, 'limit' => 30]);
                        if (is_array($sfAlt) && !empty($sfAlt['files'])) {
                            foreach ($sfAlt['files'] as $f) {
                                if ($imdbEpisodeFilter($f)) {
                                    $filesToStream[] = $f;
                                }
                            }
                        }
                    }

                    // Also search post files, still locked to this S/E
                    $sp = fd_fetch_stream_ajax('search', ['search' => $searchQuery, 'limit' => 5]);
                    if (is_array($sp)) {
                        foreach ($sp as $p) {
                            $pId = $p['id'] ?? 0;
                            if (!$pId) continue;
                            $pFiles = fd_fetch_episode_stream_files((int) $pId, $targetSeason, $targetEpisode, 40);
                            foreach ($pFiles as $pf) {
                                $filesToStream[] = $pf;
                            }
                        }
                    }

                    // Fallback: If no files found yet, fetch direct search_files with broad search query and classify
                    if (count($filesToStream) === 0) {
                        for ($sfOffset = 0; $sfOffset < 500; $sfOffset += 100) {
                            $sfBroad = fd_fetch_stream_ajax('search_files', ['search' => $searchQuery, 'limit' => 100, 'offset' => $sfOffset]);
                            $broadFiles = (array) ($sfBroad['files'] ?? []);
                            if (empty($broadFiles)) break;
                            foreach ($broadFiles as $bf) {
                                if ($imdbEpisodeFilter($bf)) {
                                    $filesToStream[] = $bf;
                                }
                            }
                            if (count($filesToStream) > 0 || count($broadFiles) < 100) break;
                        }
                    }
                } else {
                    // For Movie: search direct files and posts (try with year first, fallback to title only)
                    $queriesToTry = [];
                    if ($searchedTitle !== '' && $searchedYear !== '') {
                        $queriesToTry[] = "{$searchedTitle} {$searchedYear}";
                    }
                    $queriesToTry[] = $searchQuery;

                    foreach ($queriesToTry as $mQuery) {
                        $sf = fd_fetch_stream_ajax('search_files', ['search' => $mQuery, 'limit' => 30]);
                        if (is_array($sf) && !empty($sf['files'])) {
                            foreach ($sf['files'] as $f) {
                                $filesToStream[] = $f;
                            }
                        }

                        // If files found directly in search_files, no need to make additional slow WP post queries
                        if (count($filesToStream) > 0) {
                            break;
                        }

                        $sp = fd_fetch_stream_ajax('search', ['search' => $mQuery, 'limit' => 5]);
                        if (is_array($sp)) {
                            foreach ($sp as $p) {
                                $pId = $p['id'] ?? 0;
                                if (!$pId) continue;
                                $pFilesRes = fd_fetch_stream_ajax('post_files', ['post_id' => $pId, 'limit' => 20]);
                                $pFiles = (array) ($pFilesRes['files'] ?? []);
                                foreach ($pFiles as $pf) {
                                    if (!empty($pf['short_code'])) {
                                        $filesToStream[] = $pf;
                                    }
                                }
                            }
                        }

                        if (count($filesToStream) > 0) {
                            break;
                        }
                    }
                }
            }
        }

        // Automatically group and combine multi-part split videos (part001, part002, ...)
        $filesToStream = fd_group_split_parts($filesToStream);

        // Keep the playable stream list small: quality variants, not every file in a series.
        $maxStreamFiles = 100;
        $filesToStream = array_slice($filesToStream, 0, $maxStreamFiles);

        foreach ($filesToStream as $fItem) {
            $fCode = $fItem['short_code'] ?? '';
            if ($fCode === '') continue;

            $fTitle = $fItem['title'] ?? '';
            $fCaption = $fItem['caption'] ?? '';
            $fSize = (int) ($fItem['file_size'] ?? 0);
            $fMime = $fItem['mime'] ?? 'video/mp4';

            // If title is generic video filename, use caption title for display
            if ($fCaption !== '' && preg_match('/^(?:video(?:\.\d+)*|\d+|document|file)\.(?:mp4|mkv|avi|mov|ts|flv)$/i', trim($fTitle))) {
                $firstCap = trim(explode("\n", $fCaption)[0]);
                if ($firstCap !== '') {
                    $fTitle = $firstCap;
                }
            }

            // Extract resolution / quality / release / codec tags from filename
            $qualityTag = '';
            if (preg_match('/\b(2160p|4[kK]|uhd)\b/i', $fTitle)) {
                $qualityTag = '4K';
            } elseif (preg_match('/\b(1080p|fhd)\b/i', $fTitle)) {
                $qualityTag = '1080p';
            } elseif (preg_match('/\b(720p|hd)\b/i', $fTitle)) {
                $qualityTag = '720p';
            } elseif (preg_match('/\b(480p|360p|sd)\b/i', $fTitle)) {
                $qualityTag = 'SD';
            }

            // Detect source type (BluRay, WEB-DL, HDR, etc.)
            $metaPills = [];
            if (preg_match('/\b(bluray|blu-ray|remux)\b/i', $fTitle)) {
                $metaPills[] = 'BluRay';
            } elseif (preg_match('/\b(web-?dl|webrip)\b/i', $fTitle)) {
                $metaPills[] = 'WEB-DL';
            } elseif (preg_match('/\b(hdtv|tvrip)\b/i', $fTitle)) {
                $metaPills[] = 'HDTV';
            }
            if (preg_match('/\b(hdr10\+|hdr10|hdr|dolby\s*vision|dovi|dv)\b/i', $fTitle)) {
                $metaPills[] = 'HDR';
            }
            if (preg_match('/\b(hevc|x265|h265)\b/i', $fTitle)) {
                $metaPills[] = 'HEVC';
            } elseif (preg_match('/\b(avc|x264|h264)\b/i', $fTitle)) {
                $metaPills[] = 'AVC';
            }
            if (preg_match('/\b(aac|ac3|eac3|dts|dolby|atmos|5\.1|7\.1)\b/i', $fTitle, $am)) {
                $metaPills[] = strtoupper($am[1]);
            }

            // AIOStreams transformer + Torrentio: name is addon + quality, no title field.
            $streamName = 'PencariMovie' . "\n" . ($qualityTag !== '' ? $qualityTag : 'Direct');

            $cleanFTitle = fd_clean_media_title($fTitle);
            $fileName = $cleanFTitle !== '' ? $cleanFTitle : ($fTitle !== '' ? $fTitle : ($fCode . '.mp4'));
            $streamFileName = fd_stremio_stream_filename($fileName, $fMime);
            $streamMime = fd_guess_video_mime($streamFileName, $fMime);
            $streamBot = fd_pick_pool_bot();
            $streamBotId = !empty($streamBot['bot_id']) ? (string) $streamBot['bot_id'] : $botIdStr;

            $payload = [
                'short_code' => $fCode,
                'bot_id' => $streamBotId,
                'file_size' => $fSize,
                'file_name' => $streamFileName,
                'mime' => $streamMime,
            ];
            if (!empty($fItem['is_split_part'])) {
                $pNumStr = sprintf('%02d', (int) $fItem['part_num']);
                $totalStr = !empty($fItem['total_parts']) ? sprintf('/%02d', (int) $fItem['total_parts']) : '';
                $metaPills[] = "Part {$pNumStr}{$totalStr}";
            }

            $d = rtrim(strtr(base64_encode(json_encode($payload, JSON_UNESCAPED_SLASHES)), '+/', '-_'), '=');
            $localStreamUrl = fd_build_stremio_stream_url($baseUrl, $d, $streamFileName, $streamMime);
            $displayName = $cleanFTitle !== '' ? $cleanFTitle : ($fTitle !== '' ? $fTitle : ('File ShortCode: ' . $fCode));

            $pillsLine = !empty($metaPills) ? implode(' • ', $metaPills) : '';
            $sizeBit = $fSize > 0 ? fd_format_bytes($fSize) : '';
            $descBits = array_values(array_filter([$pillsLine, $sizeBit !== '' ? $sizeBit . ' • Telegram' : 'Telegram']));
            $streamDesc = $displayName . "\n" . implode(' • ', $descBits);

            // Official SDK + Stremio Web: HTML5 only for HTTPS URLs that literally
            // end with .mp4 (client check is url.endsWith('.mp4'), not pathname).
            // Helloworld / AIOStreams omit notWebReady on web-ready HTTP MP4; setting
            // it to false can still make some Stremio Web builds skip the list.
            $streamExt = strtolower(pathinfo(parse_url($localStreamUrl, PHP_URL_PATH) ?: '', PATHINFO_EXTENSION));
            $isHttpsMp4 = str_starts_with($localStreamUrl, 'https://')
                && str_ends_with($localStreamUrl, '.mp4')
                && $streamExt === 'mp4'
                && str_starts_with($streamMime, 'video/mp4');
            $behaviorHints = [
                'filename' => $streamFileName,
            ];
            if (!$isHttpsMp4) {
                $behaviorHints['notWebReady'] = true;
            }
            if ($fSize > 0) {
                $behaviorHints['videoSize'] = $fSize;
            }
            if ($itemType === 'series') {
                $groupTokens = ['pencarimovie'];
                $fullText = strtolower($fTitle . ' ' . $fCaption);

                // 1. Release source / group / encoder
                $groups = [
                    'myfilm4u',
                    'dramaost',
                    'nodrakor',
                    'nodrafilm',
                    'mkvdrama',
                    'ydf',
                    'fanszz',
                    'cdl',
                    'mkvking',
                    'dramadaily',
                    'kdg',
                    'pahe',
                    'psa',
                    'galaxyrg',
                    'ember',
                    'megusta',
                    'ion10',
                    'flux',
                    'ntb',
                    'syncopy',
                    'yts',
                    'yify',
                    'tgx',
                    'bone',
                    'playweb',
                    'nby',
                    'naz',
                    'kaki',
                    'dramaviral',
                    'dfm',
                    'kt',
                    'tvalhijrah',
                    'melia',
                    'mk',
                    'flx',
                    'mkvcinemas'
                ];
                $matchedGroups = [];
                foreach ($groups as $g) {
                    if (preg_match('/(?:^|[._\-\s\[\(])' . preg_quote($g, '/') . '(?:[._\-\s\]\)]|$)/i', $fTitle . ' ' . $fCaption)) {
                        $matchedGroups[] = $g;
                    }
                }
                if (!empty($matchedGroups)) {
                    $groupTokens[] = implode('.', $matchedGroups);
                }

                // 2. Language / subtitle flavor
                if (preg_match('/\b(malaysub|malay\.?sub|sub\.?malay)\b/i', $fullText)) {
                    $groupTokens[] = 'malaysub';
                } elseif (preg_match('/\b(indosub|indo\.?sub|sub\.?indo|indonesian)\b/i', $fullText)) {
                    $groupTokens[] = 'indosub';
                } elseif (preg_match('/\b(engsub|eng\.?sub|sub\.?eng|english)\b/i', $fullText)) {
                    $groupTokens[] = 'engsub';
                } elseif (preg_match('/\b(chinsub|sub\.?chin|chinese)\b/i', $fullText)) {
                    $groupTokens[] = 'chinsub';
                } elseif (preg_match('/\b(multisub|multi\.?sub)\b/i', $fullText)) {
                    $groupTokens[] = 'multisub';
                } elseif (preg_match('/\b(hardsub)\b/i', $fullText)) {
                    $groupTokens[] = 'hardsub';
                } elseif (preg_match('/\b(softsub)\b/i', $fullText)) {
                    $groupTokens[] = 'softsub';
                } elseif (preg_match('/\b(raw)\b/i', $fullText)) {
                    $groupTokens[] = 'raw';
                }

                // 3. Source / Medium
                if (preg_match('/\b(bluray|blu-ray|bdrip|remux)\b/i', $fullText)) {
                    $groupTokens[] = 'bluray';
                } elseif (preg_match('/\b(web-?dl|webrip)\b/i', $fullText)) {
                    $groupTokens[] = 'webdl';
                } elseif (preg_match('/\b(hdtv|tvrip|pdtv)\b/i', $fullText)) {
                    $groupTokens[] = 'hdtv';
                }

                // 4. Resolution / quality
                if ($qualityTag !== '') {
                    $groupTokens[] = strtolower(str_replace(' ', '-', $qualityTag));
                }

                // 5. Codec
                if (preg_match('/\b(hevc|x265|h265)\b/i', $fullText)) {
                    $groupTokens[] = 'x265';
                } elseif (preg_match('/\b(avc|x264|h264)\b/i', $fullText)) {
                    $groupTokens[] = 'x264';
                }

                // 6. Clean title signature fallback to group consistent title releases together
                $sig = preg_replace('/\.(mp4|mkv|avi|ts|flv)$/i', '', $cleanFTitle);
                $sig = preg_replace('/\b(?:19\d\d|20\d\d)\b/', '', $sig);
                $sig = preg_replace('/(?:^|[^a-z0-9])(?:S\d{1,2})?[ ._-]*(?:EP|EPS|EPISODE|EPISOD|E|PART|VOL|BAHAGIAN)[ ._-]*\d{1,4}(?:[^a-z0-9]|$)/i', ' ', $sig);
                $sig = preg_replace('/\b(akhir|final|end)\b/i', '', $sig);
                $sig = preg_replace('/[^a-z0-9]+/i', '-', trim($sig));
                $sig = strtolower(trim($sig, '-'));
                if ($sig !== '') {
                    $groupTokens[] = substr($sig, 0, 30);
                }

                $behaviorHints['bingeGroup'] = implode('-', array_unique($groupTokens));
            }

            if (empty($behaviorHints['bingeGroup'])) {
                $behaviorHints['bingeGroup'] = 'pencarimovie-' . ($qualityTag !== '' ? strtolower(str_replace(' ', '-', $qualityTag)) : 'direct');
            }

            // AIOStreams convertParsedStreamToStream: name + description + url +
            // behaviorHints only.
            $streams[] = [
                'name' => $streamName,
                'description' => $streamDesc,
                'url' => $localStreamUrl,
                'behaviorHints' => $behaviorHints,
            ];
        }

        // Add sponsored / ad stream link with externalUrl on top if configured and not empty
        if (!empty($streams)) {
            $sponsorInfo = $versionCheck['sponsor'] ?? [];
            $adUrl = trim((string) ($sponsorInfo['url'] ?? ''));
            if ($adUrl !== '') {
                $adName = trim((string) ($sponsorInfo['name'] ?? ''));
                $adDesc = trim((string) ($sponsorInfo['description'] ?? ''));
                array_unshift($streams, [
                    'name' => $adName,
                    'description' => $adDesc,
                    'externalUrl' => $adUrl,
                ]);
            }
        }

        fd_stremio_json(['streams' => $streams]);
    }

    fd_stremio_json(['ok' => 0, 'message' => 'Unknown Nuvio addon route'], 404);
}


if (str_starts_with($path, '/api/')) {
    if (!headers_sent()) {
        header('Access-Control-Allow-Origin: *');
        header('Access-Control-Allow-Methods: GET, HEAD, POST, OPTIONS');
        header('Access-Control-Allow-Headers: Content-Type, Authorization, X-API-Secret, Range');
        header('Access-Control-Expose-Headers: Accept-Ranges, Content-Range, Content-Length, Content-Type');
    }

    if ($method === 'OPTIONS') {
        http_response_code(204);
        exit;
    }

    // Public player/catalog routes. Everything else is local-only, except a
    // small read-only set that the TryCloudflare web UI needs.
    // Enable/disable and bot login stay behind fd_require_local_request();
    // cloudflared connections are treated as remote via Host/CF headers.
    $alwaysPublicApi = [
        '/api/download',
        '/api/version',
        '/api/proxy-stream',
        '/api/resolve-shortcode',
        '/api/provision',
    ];
    if (fd_is_public_download_path($path) && !in_array($path, $alwaysPublicApi, true)) {
        $alwaysPublicApi[] = $path;
    }
    $tunnelReadableApi = [
        '/api/session',
        '/api/lan-ip',
        '/api/bots',
        '/api/tunnel/status',
        '/api/provision',
        '/api/catalog-settings',
        '/api/country',
    ];
    if (!in_array($path, $alwaysPublicApi, true)) {
        $allowViaTunnel = fd_is_cloudflare_tunnel_request() && in_array($path, $tunnelReadableApi, true);
        if (!$allowViaTunnel) {
            fd_require_local_request();
        }
    }

    // Lightweight routes must not load Composer/Madeline or hit the version
    // gate. Refreshing the page calls /api/session; autoload or a 426 there
    // is treated as logout by the frontend.
    if ($path === '/api/version') {
        fd_json(fd_check_version());
    }

    if ($path === '/api/lan-ip') {
        fd_json([
            'ok' => 1,
            'lan_ip' => fd_get_lan_ip(),
            'port' => fd_get_listen_port(),
        ]);
    }

    if ($path === '/api/tunnel/status' && $method === 'GET') {
        fd_json(fd_get_tunnel_status());
    }

    if ($path === '/api/tunnel/enable' && $method === 'POST') {
        $result = fd_enable_tunnel();
        fd_json($result, !empty($result['ok']) ? 200 : 500);
    }

    if ($path === '/api/tunnel/disable' && $method === 'POST') {
        fd_json(fd_disable_tunnel());
    }

    // ── Catalog settings API ──
    if ($path === '/api/catalog-settings' && $method === 'GET') {
        $settings = fd_load_catalog_settings();
        $catalogOptions = fd_get_default_catalog_options();
        fd_json([
            'ok' => 1,
            'settings' => $settings,
            'catalog_options' => $catalogOptions,
        ]);
    }

    if ($path === '/api/catalog-settings' && $method === 'POST') {
        $input = json_decode((string) file_get_contents('php://input'), true);
        if (!is_array($input)) {
            fd_json(['ok' => 0, 'error' => 'Invalid JSON input'], 400);
        }

        $current = fd_load_catalog_settings();
        if (isset($input['catalogs_enabled'])) {
            $current['catalogs_enabled'] = (bool) $input['catalogs_enabled'];
        }
        if (isset($input['enabled_types']) && is_array($input['enabled_types'])) {
            if (isset($input['enabled_types']['movie'])) {
                $current['enabled_types']['movie'] = (bool) $input['enabled_types']['movie'];
            }
            if (isset($input['enabled_types']['series'])) {
                $current['enabled_types']['series'] = (bool) $input['enabled_types']['series'];
            }
            if (isset($input['enabled_types']['other'])) {
                $current['enabled_types']['other'] = (bool) $input['enabled_types']['other'];
            }
        }
        if (isset($input['enabled_catalogs']) && is_array($input['enabled_catalogs'])) {
            foreach ($input['enabled_catalogs'] as $cid => $val) {
                $current['enabled_catalogs'][$cid] = (bool) $val;
            }
        }

        $saved = fd_save_catalog_settings($current);
        fd_json([
            'ok' => $saved ? 1 : 0,
            'settings' => $current,
            'message' => $saved ? 'Catalog settings updated successfully' : 'Failed to save settings',
        ]);
    }


    // ── Country Detection API (reads Cloudflare header in-memory, zero disk footprint) ──
    if ($path === '/api/country' && $method === 'GET') {
        fd_json([
            'ok' => 1,
            'country' => fd_detect_country(),
        ]);
    }

    if ($path === '/api/session') {
        $botId = fd_get_bot_id();
        $meta = fd_load_session_meta();
        // Leftover session.madeline without bot_id still lets the catalog load,
        // but WordPress resolve-file fails with "bot_id not found". Treat that
        // as incomplete login so the frontend shows the token prompt.
        $hasSession = fd_has_local_session() && $botId !== '';
        $pool = fd_get_bot_pool();
        $viaTunnel = fd_is_cloudflare_tunnel_request();

        fd_json([
            'ok' => 1,
            'version' => FD_APP_VERSION,
            'has_session' => $hasSession,
            'bot_id' => $hasSession ? $botId : '',
            'bot_username' => $hasSession ? (string) ($meta['bot_username'] ?? '') : '',
            'bot_name' => $hasSession ? (string) ($meta['bot_name'] ?? '') : '',
            'api_secret' => ($hasSession && !$viaTunnel) ? fd_get_api_secret() : '',
            'device_id' => fd_get_device_id(),
            'bot_count' => count($pool),
            'bot_pool' => $pool,
        ]);
    }

    // ── POST /api/provision — auto-provision a guest bot session on the fly ───
    if ($path === '/api/provision' && in_array($method, ['GET', 'POST'], true)) {
        $provisioned = fd_auto_provision_guest();
        if (!$provisioned) {
            fd_json([
                'ok' => 0,
                'message' => 'Could not obtain guest bot session from server.',
            ], 500);
        }

        fd_json([
            'ok' => 1,
            'message' => 'Guest bot session initialized successfully.',
            'bot_id' => $provisioned['bot_id'],
            'bot_username' => $provisioned['bot_username'],
            'bot_name' => $provisioned['bot_name'],
            'api_secret' => fd_get_api_secret(),
            'pool' => fd_get_bot_pool(),
        ]);
    }

    // ── GET /api/bots — list all configured bots in the pool ─────────────────
    if ($path === '/api/bots' && $method === 'GET') {
        $pool = fd_get_bot_pool();
        $activeId = fd_get_bot_id();
        $list = [];
        foreach ($pool as $b) {
            $bId = (string)($b['bot_id'] ?? '');
            $hasSess = fd_has_local_session($bId) || ($bId === $activeId && fd_has_local_session());
            $list[] = array_merge($b, [
                'has_session' => $hasSess,
                'is_active' => $bId === $activeId,
            ]);
        }
        fd_json([
            'ok' => 1,
            'active_bot_id' => $activeId,
            'total_bots' => count($list),
            'bots' => $list,
        ]);
    }

    // ── POST /api/bots/add — add one or multiple bot tokens to the pool ──────
    if ($path === '/api/bots/add' && $method === 'POST') {
        $input = json_decode((string) file_get_contents('php://input'), true);
        if (!is_array($input)) {
            fd_json(['ok' => 0, 'message' => 'Invalid JSON body'], 400);
        }

        $tokens = [];
        if (!empty($input['bot_token'])) {
            $tokens[] = trim((string) $input['bot_token']);
        } elseif (!empty($input['tokens']) && is_array($input['tokens'])) {
            foreach ($input['tokens'] as $t) {
                $t = trim((string) $t);
                if ($t !== '') {
                    $tokens[] = $t;
                }
            }
        } elseif (!empty($input['tokens_text'])) {
            // Support newline / space separated tokens
            $parts = preg_split('/[\r\n\s,]+/', (string) $input['tokens_text']);
            foreach ($parts as $p) {
                $p = trim($p);
                if ($p !== '') {
                    $tokens[] = $p;
                }
            }
        }

        if (empty($tokens)) {
            fd_json(['ok' => 0, 'message' => 'bot_token or tokens array is required.'], 400);
        }

        $results = [];
        $addedCount = 0;

        foreach ($tokens as $token) {
            // Extract numeric prefix for quick bot_id estimation
            $tempBotId = '';
            if (preg_match('/^(\d+):/', $token, $m)) {
                $tempBotId = $m[1];
            }

            fd_log('adding bot to pool', ['token_prefix' => substr($token, 0, 8) . '...']);

            // Boot MadelineProto for this specific bot session
            [$madeline, $error] = fd_boot_madeline($token, [], $tempBotId);

            if (!$madeline) {
                $results[] = [
                    'token_prefix' => substr($token, 0, 8) . '...',
                    'ok' => 0,
                    'error' => $error ?: 'Failed to login',
                ];
                continue;
            }

            try {
                $self = $madeline->getSelf();
                $bId = (string) ($self['id'] ?? $tempBotId);
                $bUser = (string) ($self['username'] ?? '');
                $bName = (string) ($self['first_name'] ?? '');

                // Ensure session directory is moved to specific bot_id folder if needed
                if ($tempBotId === '' || $tempBotId !== $bId) {
                    $oldPath = fd_get_bot_session_path($tempBotId);
                    $newPath = fd_get_bot_session_path($bId);
                    if ($oldPath !== $newPath && (is_dir($oldPath) || is_file($oldPath))) {
                        @rename($oldPath, $newPath);
                    }
                }

                $botEntry = [
                    'bot_id' => $bId,
                    'bot_username' => $bUser,
                    'bot_name' => $bName,
                    'status' => 'online',
                    'updated_at' => time(),
                ];

                fd_add_pool_bot($botEntry);

                // If no active bot is set, set this as primary active bot
                if (fd_get_bot_id() === '') {
                    fd_save_session_meta($bId, $bUser, $bName);
                }

                $results[] = [
                    'ok' => 1,
                    'bot_id' => $bId,
                    'bot_username' => $bUser,
                    'bot_name' => $bName,
                ];
                $addedCount++;
            } catch (\Throwable $e) {
                $results[] = [
                    'token_prefix' => substr($token, 0, 8) . '...',
                    'ok' => 0,
                    'error' => $e->getMessage(),
                ];
            }
        }

        fd_json([
            'ok' => $addedCount > 0 ? 1 : 0,
            'added_count' => $addedCount,
            'results' => $results,
            'pool' => fd_get_bot_pool(),
        ], $addedCount > 0 ? 200 : 400);
    }

    // ── POST /api/bots/remove — remove a bot from the pool ───────────────────
    if ($path === '/api/bots/remove' && $method === 'POST') {
        $input = json_decode((string) file_get_contents('php://input'), true);
        if (!is_array($input)) {
            fd_json(['ok' => 0, 'message' => 'Invalid JSON body'], 400);
        }
        $botId = trim((string) ($input['bot_id'] ?? ''));
        if ($botId === '') {
            fd_json(['ok' => 0, 'message' => 'bot_id is required.'], 400);
        }
        $updatedPool = fd_remove_pool_bot($botId);
        fd_json([
            'ok' => 1,
            'message' => 'Bot removed from pool.',
            'pool' => $updatedPool,
            'active_bot_id' => fd_get_bot_id(),
        ]);
    }

    // ── POST /api/bots/set-active — set primary active bot in pool ───────────
    if ($path === '/api/bots/set-active' && $method === 'POST') {
        $input = json_decode((string) file_get_contents('php://input'), true);
        if (!is_array($input)) {
            fd_json(['ok' => 0, 'message' => 'Invalid JSON body'], 400);
        }
        $botId = trim((string) ($input['bot_id'] ?? ''));
        if ($botId === '') {
            fd_json(['ok' => 0, 'message' => 'bot_id is required.'], 400);
        }
        $pool = fd_get_bot_pool();
        $target = null;
        foreach ($pool as $b) {
            if ((string)($b['bot_id'] ?? '') === $botId) {
                $target = $b;
                break;
            }
        }
        if (!$target) {
            fd_json(['ok' => 0, 'message' => 'Bot ID not found in pool.'], 404);
        }

        fd_save_session_meta(
            (string)($target['bot_id'] ?? ''),
            (string)($target['bot_username'] ?? ''),
            (string)($target['bot_name'] ?? '')
        );

        fd_json([
            'ok' => 1,
            'active_bot_id' => $botId,
            'bot_username' => (string)($target['bot_username'] ?? ''),
            'bot_name' => (string)($target['bot_name'] ?? ''),
            'tunnel_restarted' => $tunnelRestarted,
        ]);
    }

    fd_require_fileinfo();

    // Pre-load Composer autoloader so amphp classes (HttpClient, etc.) are
    // available for all API routes. fd_ensure_autoload() handles the output
    // buffering needed to suppress the polyfill.php echo warning on Windows.
    fd_ensure_autoload();

    // ── Version gate — block all other endpoints if update is required ──────
    $v0 = microtime(true);
    $versionCheck = fd_check_version();
    $v1 = microtime(true);
    fd_log('version check timing', ['ms' => round(($v1 - $v0) * 1000), 'path' => $path]);
    if (!empty($versionCheck['update_needed'])) {
        $minVersion = $versionCheck['minimum_version'] ?? '';
        $currentVersion = $versionCheck['current_version'] ?? FD_APP_VERSION;
        $updateUrl = $versionCheck['update_url'] ?? '';

        fd_log('version gate blocked request', [
            'current' => $currentVersion,
            'minimum' => $minVersion,
            'path' => $path,
        ]);

        fd_json([
            'ok' => 0,
            'message' => 'Update Required. Your version (' . $currentVersion . ') is below the minimum required version (' . $minVersion . ').',
            'update_needed' => true,
            'current_version' => $currentVersion,
            'minimum_version' => $minVersion,
            'update_url' => $updateUrl,
        ], 426);
    }

    // ── GET /api/resolve-shortcode — proxy short_code resolution through
    //     the local backend so the API secret (X-API-Secret header) is
    //     automatically sent to WordPress. The frontend should call this
    //     instead of hitting WordPress directly. ────────────────────────────
    if ($path === '/api/resolve-shortcode' && $method === 'GET') {
        $shortCode = trim((string) ($_GET['short_code'] ?? ''));
        $botId = trim((string) ($_GET['bot_id'] ?? ''));
        if ($shortCode === '') {
            fd_json(['ok' => 0, 'message' => 'short_code is required.'], 400);
        }

        if ($botId === '') {
            $picked = fd_pick_pool_bot();
            if (!empty($picked['bot_id'])) {
                $botId = (string) $picked['bot_id'];
            }
        }

        // Use concurrent multi-bot resolution across all pool bots for instantaneous resolution
        $result = fd_resolve_shortcode_concurrent($shortCode, $botId !== '' ? [$botId] : []);
        if (empty($result['ok']) && $botId !== '') {
            $result = fd_resolve_shortcode_concurrent($shortCode);
        }
        if (empty($result['ok'])) {
            fd_json($result, 200);
        }
        fd_json($result);
    }

    // ── POST /api/botlogin — one-time bot token login ───────────────────────
    if ($path === '/api/botlogin' && $method === 'POST') {
        $input = json_decode((string) file_get_contents('php://input'), true);
        if (!is_array($input)) {
            fd_json(['ok' => 0, 'message' => 'Invalid JSON body'], 400);
        }

        $botToken = trim((string) ($input['bot_token'] ?? ''));
        if ($botToken === '') {
            fd_json(['ok' => 0, 'message' => 'bot_token is required.'], 400);
        }

        fd_log('botlogin start', ['token_prefix' => substr($botToken, 0, 8) . '...']);

        $tempBotId = '';
        if (preg_match('/^(\d+):/', $botToken, $m)) {
            $tempBotId = $m[1];
        }

        // Clean only this bot's existing partitioned session to force fresh login
        if ($tempBotId !== '') {
            fd_clear_session($tempBotId);
        } else {
            fd_clear_session();
        }

        // Save api_secret if forwarded from browser-direct WordPress call
        if (!empty($input['api_secret'])) {
            fd_save_api_secret((string) $input['api_secret']);
            fd_log('api secret saved from browser login payload');
        }

        // ═══ [DIAGNOSTIC] Capture any stray output before fd_boot_madeline ═══
        $diagObLevel = ob_get_level();
        $preBootOutput = '';
        while (ob_get_level() > 0) {
            $preBootOutput .= ob_get_clean();
        }
        while (ob_get_level() < $diagObLevel) {
            ob_start();
        }
        if ($preBootOutput !== '') {
            fd_log('⚠️  stray output detected BEFORE fd_boot_madeline', [
                'length' => strlen($preBootOutput),
                'preview' => substr($preBootOutput, 0, 500),
            ]);
        }

        // Forward any browser-provided encrypted credentials to fd_boot_madeline
        $bootOverrides = [];
        if (!empty($input['encrypted_credentials']) && !empty($input['encryption_iv'])) {
            $bootOverrides['encrypted_credentials'] = (string) $input['encrypted_credentials'];
            $bootOverrides['encryption_iv'] = (string) $input['encryption_iv'];
        }

        // API credentials are resolved internally by fd_boot_madeline()
        $tBoot0 = microtime(true);
        [$madeline, $error] = fd_boot_madeline($botToken, $bootOverrides, $tempBotId);
        $tBoot1 = microtime(true);
        fd_log('fd_boot_madeline timing', ['ms' => round(($tBoot1 - $tBoot0) * 1000)]);

        if (!$madeline) {
            fd_log('botlogin failed', ['error' => $error]);
            fd_json(['ok' => 0, 'message' => $error ?: 'Login failed.'], 401);
        }

        // ═══ [DIAGNOSTIC] Check for stray output after fd_boot_madeline ═══
        $postBootOutput = '';
        if (ob_get_level() > $diagObLevel) {
            while (ob_get_level() > $diagObLevel) {
                $postBootOutput .= ob_get_clean();
            }
            if ($postBootOutput !== '') {
                fd_log('⚠️  stray output detected AFTER fd_boot_madeline', [
                    'length' => strlen($postBootOutput),
                    'preview' => substr($postBootOutput, 0, 500),
                ]);
            }
        }

        try {
            $self = $madeline->getSelf();
            $botId = (string) ($self['id'] ?? $tempBotId);
            if ($botId === '') {
                fd_json(['ok' => 0, 'message' => 'Login succeeded but bot ID is empty.'], 500);
            }

            // Ensure session directory is organized under specific bot_id folder
            if ($tempBotId === '' || $tempBotId !== $botId) {
                $oldPath = fd_get_bot_session_path($tempBotId);
                $newPath = fd_get_bot_session_path($botId);
                if ($oldPath !== $newPath && (is_dir($oldPath) || is_file($oldPath))) {
                    @rename($oldPath, $newPath);
                }
            }

            $botUsername = (string) ($self['username'] ?? '');
            $botName = (string) ($self['first_name'] ?? '');

            // Register into bot pool
            fd_add_pool_bot([
                'bot_id' => $botId,
                'bot_username' => $botUsername,
                'bot_name' => $botName,
                'status' => 'online',
                'updated_at' => time(),
            ]);

            // Save primary active bot meta
            fd_save_session_meta($botId, $botUsername, $botName);

            // ═══ [DIAGNOSTIC] Check output buffer state before fd_json ═══
            $bufBeforeJson = '';
            while (ob_get_level() > 0) {
                $bufBeforeJson .= ob_get_clean();
            }
            if ($bufBeforeJson !== '') {
                fd_log('⚠️  stray output before botlogin success fd_json', [
                    'length' => strlen($bufBeforeJson),
                    'preview' => substr($bufBeforeJson, 0, 500),
                ]);
            }
            // Restore output buffering so fd_json can set headers
            ob_start();

            fd_log('botlogin successful', ['bot_id' => $botId, 'bot_username' => $botUsername]);
            fd_json([
                'ok' => 1,
                'bot_id' => $botId,
                'bot_username' => $botUsername,
                'bot_name' => $botName,
                'api_secret' => fd_get_api_secret(),
                'pool' => fd_get_bot_pool(),
            ]);
        } catch (Throwable $throwable) {
            fd_log('botlogin getSelf failed', ['error' => $throwable->getMessage()]);
            fd_json(['ok' => 0, 'message' => 'Session validation failed: ' . $throwable->getMessage()], 500);
        }
    }

    // ── POST /api/botlogout — terminate session via Telegram API then clean up ─
    if ($path === '/api/botlogout' && $method === 'POST') {
        // Attempt to boot MadelineProto from the existing session and call
        // logout() which sends auth.logOut to Telegram, properly invalidating
        // the authorization key on Telegram's servers (not just locally).
        $pool = fd_get_bot_pool();
        foreach ($pool as $b) {
            $bId = (string) ($b['bot_id'] ?? '');
            if ($bId !== '') {
                try {
                    [$mBot, $eBot] = fd_boot_madeline(null, [], $bId);
                    if ($mBot) {
                        $mBot->logout();
                    }
                    unset($mBot);
                } catch (Throwable $t) {
                }
            }
        }

        try {
            [$madeline, $error] = fd_boot_madeline();
            if ($madeline) {
                $madeline->logout();
                fd_log('madeline logout() completed');
            }
        } catch (Throwable $throwable) {
            fd_log('madeline logout() threw', ['error' => $throwable->getMessage()]);
        }

        unset($madeline);

        // Always clean up all local sessions and pool files
        fd_clear_session();
        fd_json(['ok' => 1, 'message' => 'Session cleared.']);
    }


    // ── GET /api/proxy-stream — proxy streaming data from WordPress AJAX ────
    if ($path === '/api/proxy-stream' && $method === 'GET') {
        $action = trim((string) ($_GET['action'] ?? ''));
        if ($action === '') {
            fd_json(['ok' => 0, 'message' => 'action parameter is required.'], 400);
        }

        // Map short action names to stream_* WordPress AJAX actions
        // e.g., "trending" → "stream_trending", "search_files" → "stream_search_files"
        $streamAction = 'stream_' . $action;

        // Build the WordPress admin-ajax.php URL, forwarding all GET params
        // with the action replaced by the stream_* prefixed version
        $wpUrl = defined('FD_WP_AJAX_URL') ? FD_WP_AJAX_URL : 'https://pencarimovie.com/wp-admin/admin-ajax.php';
        $queryParams = $_GET;
        $queryParams['action'] = $streamAction;
        if (empty($queryParams['bot_id'])) {
            $activeBotId = fd_get_bot_id();
            if ($activeBotId !== '') {
                $queryParams['bot_id'] = $activeBotId;
            }
        }
        if (empty($queryParams['country'])) {
            $c = fd_detect_country();
            if (!empty($c['country_code'])) {
                $queryParams['country'] = $c['country_code'];
            }
        }
        $wpUrl .= '?' . http_build_query($queryParams);

        try {
            $body = fd_http_get_contents($wpUrl, [
                'method' => 'GET',
                'headers' => ['X-Requested-With: XMLHttpRequest'],
                'timeout' => 15,
            ]);
            if ($body === false) {
                throw new \RuntimeException('fd_http_get_contents failed');
            }
        } catch (\Throwable $e) {
            fd_log('proxy-stream failed', ['action' => $streamAction, 'error' => $e->getMessage()]);
            fd_json(['ok' => 0, 'message' => 'Failed to fetch data from WordPress.'], 502);
        }

        // Try to decode as JSON to return proper Content-Type
        $decoded = json_decode($body, true);
        if (is_array($decoded)) {
            fd_json($decoded);
        }

        // If not JSON, return raw with correct content type
        header('Content-Type: application/json; charset=utf-8');
        echo $body;
        return true;
    }

    // ── GET /api/download[/:payload/:filename] — stream Telegram file via MadelineProto ──
    // Stremio Web requires the full URL to end with .mp4, so the payload is a path
    // segment: /api/download/<base64url>/<filename>.mp4. Query ?d= remains supported.
    if (fd_is_public_download_path($path)) {
        $encoded = trim((string) ($_GET['d'] ?? $_POST['d'] ?? ''));
        if ($encoded === '') {
            $encoded = fd_extract_download_payload_from_path($path);
        }
        $decodedPayload = $encoded !== '' ? fd_decode_download_payload($encoded) : [];
        $fileId = trim((string) ($decodedPayload['file_id'] ?? ($_GET['file_id'] ?? $_POST['file_id'] ?? '')));
        $shortCode = trim((string) ($decodedPayload['short_code'] ?? ($_GET['short_code'] ?? $_POST['short_code'] ?? $_GET['sc'] ?? '')));
        if ($shortCode === '' && $fileId === '' && $encoded !== '' && empty($decodedPayload)) {
            $shortCode = $encoded;
        }
        $fileSize = (int) ($decodedPayload['file_size'] ?? ($_GET['file_size'] ?? $_POST['file_size'] ?? $_GET['size'] ?? $_POST['size'] ?? 0));
        $fileName = trim((string) ($decodedPayload['file_name'] ?? ($_GET['file_name'] ?? $_POST['file_name'] ?? $_GET['name'] ?? $_POST['name'] ?? '')));
        $fileMime = trim((string) ($decodedPayload['mime'] ?? ($_GET['mime'] ?? $_POST['mime'] ?? $_GET['mime_type'] ?? $_POST['mime_type'] ?? '')));
        $botId = trim((string) ($decodedPayload['bot_id'] ?? ($_GET['bot_id'] ?? $_POST['bot_id'] ?? '')));

        ini_set('display_errors', '0');
        ini_set('log_errors', '1');

        fd_log('download request received', [
            'file_id_present' => $fileId !== '',
            'short_code_present' => $shortCode !== '',
            'encoded_present' => $encoded !== '',
            'bot_id_present' => $botId !== '',
            'file_size' => $fileSize,
            'file_name' => $fileName,
            'mime' => $fileMime,
            'ob_level' => ob_get_level(),
        ]);

        // If Telegram Bot is not connected or session missing, attempt auto-provisioning
        $activeBotId = fd_get_bot_id();
        if (!fd_has_local_session()) {
            $autoProv = fd_auto_provision_guest();
            if ($autoProv && !empty($autoProv['bot_id'])) {
                $activeBotId = (string) $autoProv['bot_id'];
                if ($botId === '') {
                    $botId = $activeBotId;
                }
            } else {
                header('Cache-Control: no-cache, no-store, must-revalidate');
                header('Connection: close');
                fd_json([
                    'ok' => 0,
                    'message' => 'Telegram Bot is not connected. Please connect your bot token in dashboard settings to stream.',
                    'hint' => 'Open dashboard settings and connect your bot token.',
                    'short_code' => $shortCode,
                    'bot_id' => $botId,
                ], 403);
            }
        }

        // Build candidate bot list from pool and active bot
        $candidateBots = [];
        if ($shortCode !== '') {
            // When short_code is present, pick a rotated bot from the pool for each download request
            $pickedBot = fd_pick_pool_bot();
            if (!empty($pickedBot['bot_id'])) {
                $candidateBots[] = (string) $pickedBot['bot_id'];
            }
        }

        if ($botId !== '' && !in_array($botId, $candidateBots, true)) {
            $candidateBots[] = $botId;
        }

        $botPool = fd_get_bot_pool();
        foreach ($botPool as $pBot) {
            $pId = (string) ($pBot['bot_id'] ?? '');
            if ($pId !== '' && !in_array($pId, $candidateBots, true)) {
                $candidateBots[] = $pId;
            }
        }
        if ($activeBotId !== '' && !in_array($activeBotId, $candidateBots, true)) {
            $candidateBots[] = $activeBotId;
        }

        if (empty($candidateBots)) {
            $candidateBots[] = $activeBotId;
        }

        $botId = (string) $candidateBots[0];
        $madeline = null;
        $error = null;


        // If short_code is provided, concurrently resolve across all candidate bots simultaneously
        if ($shortCode !== '') {
            $res = fd_resolve_shortcode_concurrent($shortCode, $candidateBots);
            $candidateFileId = trim((string) ($res['file_id_mt'] ?? $res['file_id'] ?? ''));
            $cBotId = !empty($res['bot_id']) ? (string) $res['bot_id'] : $botId;
            $resolved = $res;

            if ($candidateFileId !== '') {
                // Try booting MadelineProto for this winning bot
                [$bootedMadeline, $bootErr] = fd_boot_madeline(null, [], $cBotId);
                if (!$bootedMadeline && $cBotId !== $activeBotId) {
                    [$bootedMadeline, $bootErr] = fd_boot_madeline(null, [], '');
                }

                if ($bootedMadeline) {
                    $madeline = $bootedMadeline;
                    $fileId = $candidateFileId;
                    $fileSize = (int) ($res['file_size'] ?? $fileSize);
                    $resolvedName = trim((string) ($res['title'] ?? $res['file_name'] ?? ''));
                    if ($resolvedName !== '') {
                        $fileName = $resolvedName;
                    }
                    $resolvedMime = trim((string) ($res['mime'] ?? ''));
                    $resolvedType = trim((string) ($res['file_type'] ?? ''));
                    // Telegram file_type is often "document"; do not overwrite a real video MIME with that.
                    $fileMime = fd_guess_video_mime(
                        $fileName,
                        $resolvedMime !== '' ? $resolvedMime : ($fileMime !== '' ? $fileMime : $resolvedType)
                    );
                    $botId = $cBotId;
                } else {
                    $error = $bootErr;
                }
            }

            // If resolve failed for current candidates, retry with a fresh auto-provisioned guest bot
            if ($fileId === '' && empty($madeline)) {
                $retryProv = fd_auto_provision_guest();
                if ($retryProv && !empty($retryProv['bot_id'])) {
                    $retryBotId = (string) $retryProv['bot_id'];
                    $retryRes = fd_resolve_shortcode($shortCode, $retryBotId);
                    $retryFileId = trim((string) ($retryRes['file_id_mt'] ?? $retryRes['file_id'] ?? ''));
                    if ($retryFileId !== '') {
                        [$retryBooted, $retryErr] = fd_boot_madeline(null, [], $retryBotId);
                        if ($retryBooted) {
                            $madeline = $retryBooted;
                            $fileId = $retryFileId;
                            $fileSize = (int) ($retryRes['file_size'] ?? $fileSize);
                            $resolvedName = trim((string) ($retryRes['title'] ?? $retryRes['file_name'] ?? ''));
                            if ($resolvedName !== '') {
                                $fileName = $resolvedName;
                            }
                            $resolvedMime = trim((string) ($retryRes['mime'] ?? ''));
                            $resolvedType = trim((string) ($retryRes['file_type'] ?? ''));
                            $fileMime = fd_guess_video_mime(
                                $fileName,
                                $resolvedMime !== '' ? $resolvedMime : ($fileMime !== '' ? $fileMime : $resolvedType)
                            );
                            $botId = $retryBotId;
                        }
                    }
                }
            }

            if ($fileId === '' && empty($madeline)) {
                $errMsg = !empty($resolved['message'])
                    ? (string) $resolved['message']
                    : (!empty($resolved['description']) ? (string) $resolved['description'] : 'File source is unpopulated, expired, or missing.');

                $statusCode = 404;
                if (stripos($errMsg, 'not allowed') !== false || stripos($errMsg, 'unauthorized') !== false || stripos($errMsg, 'forbidden') !== false) {
                    $statusCode = 403;
                }

                header('Cache-Control: no-cache, no-store, must-revalidate');
                header('Connection: close');
                fd_json([
                    'ok' => 0,
                    'message' => $errMsg,
                    'short_code' => $shortCode,
                    'bot_id' => $botId,
                    'retried_bots' => count($candidateBots),
                ], $statusCode);
            }
        }

        if ($fileId === '') {
            header('Cache-Control: no-cache, no-store, must-revalidate');
            header('Connection: close');
            fd_json([
                'ok' => 0,
                'message' => 'file_id or valid short_code is required for local browser download.',
                'hint' => 'Pass a Bot API file id or shortcode to resolve the file.',
            ], 400);
        }

        if ($fileSize <= 0 || $fileName === '' || $fileMime === '') {
            header('Cache-Control: no-cache, no-store, must-revalidate');
            header('Connection: close');
            fd_json([
                'ok' => 0,
                'message' => 'For Bot API file_id download, file_size, file_name, and mime are required.',
                'hint' => 'Include file_size, file_name, and mime from your WordPress metadata response.',
            ], 400);
        }

        // If not already booted during short_code resolution loop, boot MadelineProto now
        if (!$madeline) {
            [$madeline, $error] = fd_boot_madeline(null, [], $botId);
            if (!$madeline && $botId !== $activeBotId) {
                [$madeline, $error] = fd_boot_madeline(null, [], '');
            }
            if (!$madeline) {
                // Retry by provisioning a fresh guest session
                $retryProv = fd_auto_provision_guest();
                if ($retryProv && !empty($retryProv['bot_id'])) {
                    $botId = (string) $retryProv['bot_id'];
                    [$madeline, $error] = fd_boot_madeline(null, [], $botId);
                }
            }
        }

        if (!$madeline) {
            fd_log('download failed — no valid session', [
                'error' => $error,
            ]);
            header('Cache-Control: no-cache, no-store, must-revalidate');
            header('Connection: close');
            fd_json([
                'ok' => 0,
                'message' => $error ?: 'No valid MadelineProto session.',
                'hint' => 'Call /api/botlogin first to authenticate.',
            ], 403);
        }

        if (!method_exists($madeline, 'downloadToBrowser')) {
            fd_log('downloadToBrowser unavailable on madeline instance');
            fd_json([
                'ok' => 0,
                'message' => 'MadelineProto downloadToBrowser() is not available.',
            ], 501);
        }

        try {
            $fileName = fd_stremio_stream_filename($fileName, $fileMime);
            $fileMime = fd_guess_video_mime($fileName, $fileMime);
            if (!headers_sent()) {
                header('Access-Control-Allow-Origin: *');
                header('Access-Control-Allow-Methods: GET, HEAD, OPTIONS');
                header('Access-Control-Expose-Headers: Accept-Ranges, Content-Range, Content-Length, Content-Type');
                header('Accept-Ranges: bytes');
            }
            fd_log('starting downloadToBrowser', [
                'file_id' => $fileId,
                'file_size' => $fileSize,
                'file_name' => $fileName,
                'mime' => $fileMime,
            ]);
            $madeline->downloadToBrowser($fileId, null, $fileSize, $fileName, $fileMime);
        } catch (Throwable $throwable) {
            fd_log('downloadToBrowser failed', [
                'error' => $throwable->getMessage(),
            ]);
            fd_json([
                'ok' => 0,
                'message' => 'File download failed.',
            ], 500);
        }

        return true;
    }

    fd_json(['ok' => 0, 'message' => 'Unknown API endpoint'], 404);
}

// ─── Static file serving ────────────────────────────────────────────────────

$publicDir = __DIR__ . '/public';
$file = $path === '/' ? '/index.html' : $path;
$full = realpath($publicDir . $file);

if ($full && str_starts_with($full, realpath($publicDir)) && is_file($full)) {
    $ext = strtolower(pathinfo($full, PATHINFO_EXTENSION));
    $types = [
        'html' => 'text/html; charset=utf-8',
        'css' => 'text/css; charset=utf-8',
        'js' => 'application/javascript; charset=utf-8',
        'json' => 'application/json; charset=utf-8',
        'svg' => 'image/svg+xml; charset=utf-8',
        'ico' => 'image/x-icon',
    ];
    header('Content-Type: ' . ($types[$ext] ?? 'application/octet-stream'));
    header('Cache-Control: no-store');
    readfile($full);
    return true;
}

http_response_code(404);
echo 'Not found';
