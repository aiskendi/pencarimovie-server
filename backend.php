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
        || is_file($storageDir . DIRECTORY_SEPARATOR . 'session_meta.json')
        || is_dir($storageDir . DIRECTORY_SEPARATOR . 'sessions');
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
// Fallback host used when the primary domain is blocked by an in-path DPI
// firewall (corporate / campus / hospital networks reset the TLS handshake on
// the "pencarimovie.com" SNI). telegra.my is a Cloudflare Worker that reverse
// proxies /wp-json/* and /wp-admin/admin-ajax.php back to pencarimovie.com over
// Cloudflare's internal backbone, so the local SNI is a benign hostname.
define('FD_WP_FALLBACK_HOST', 'telegra.my');
define('FD_WP_API_BASE_FALLBACK', 'https://' . FD_WP_FALLBACK_HOST . '/wp-json/pencarimovie-server/v1');
define('FD_WP_AJAX_URL_FALLBACK', 'https://' . FD_WP_FALLBACK_HOST . '/wp-admin/admin-ajax.php');
define('FD_APP_VERSION', '2.2.1');
define('FD_WP_VERSION_URL', FD_WP_API_BASE . '/version');
define('FD_API_SECRET_PATH', fd_storage_path('storage/api_secret.key'));
define('FD_BOT_ID_CACHE_PATH', fd_storage_path('storage/bot_id.txt'));
define('FD_DEVICE_ID_PATH', fd_storage_path('storage/device_id.txt'));
define('FD_SESSION_META_PATH', fd_storage_path('storage/session_meta.json'));
define('FD_BOT_POOL_PATH', fd_storage_path('storage/bot_pool.json'));
define('FD_CATALOG_SETTINGS_PATH', fd_storage_path('storage/catalog_settings.json'));
define('FD_AUTH_PATH', fd_storage_path('storage/auth.json'));
define('FD_AUTH_DEFAULT_PASSWORD', '123456');
define('FD_AUTH_COOKIE', 'pm_auth');
define('FD_DEBUG_LOG_PATH', fd_storage_path('storage/debug.log'));
define('FD_DEBUG_TOGGLE_PATH', fd_storage_path('storage/debug_mode.txt'));
define('FD_CACHE_DIR', fd_storage_path('storage/cache'));
define('FD_MAX_LOG_SIZE', 5 * 1024 * 1024); // 5 MB max per log file

function fd_cache_path(string $file): string
{
    $dir = FD_CACHE_DIR;
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    return $dir . DIRECTORY_SEPARATOR . ltrim($file, '/\\');
}

/**
 * Periodically purge expired cache files in storage/cache and legacy root cache files.
 */
function fd_prune_cache_files(bool $force = false): void
{
    static $lastPrune = 0;
    $now = time();
    if (!$force && ($now - $lastPrune) < 300) {
        return;
    }
    $lastPrune = $now;

    $cacheDir = FD_CACHE_DIR;
    $storageDir = fd_get_storage_dir();

    // Map prefix to TTL in seconds
    $ttls = [
        'resolve_cache_' => 7200,   // 2h
        'stream_cache_'  => 300,    // 5m
        'up_cat_'        => 600,    // 10m
        'up_meta_'       => 3600,   // 1h
        'sub_cache_'     => 1800,   // 30m
        'cinemeta_'      => 86400,  // 24h
        'upstream_manifest_' => 7200, // 2h
    ];

    // 1. Move or purge legacy cache files scattered in storage/ root
    foreach ($ttls as $prefix => $ttl) {
        $legacyFiles = glob($storageDir . '/' . $prefix . '*.json');
        if ($legacyFiles) {
            foreach ($legacyFiles as $lf) {
                if (is_file($lf)) {
                    $age = $now - (int) @filemtime($lf);
                    if ($age >= $ttl) {
                        @unlink($lf);
                    } else {
                        // Move active cache file to storage/cache/
                        $dest = fd_cache_path(basename($lf));
                        @rename($lf, $dest);
                    }
                }
            }
        }
    }

    // 2. Prune expired cache files in storage/cache/
    if (is_dir($cacheDir)) {
        foreach ($ttls as $prefix => $ttl) {
            $files = glob($cacheDir . '/' . $prefix . '*.json');
            if ($files) {
                foreach ($files as $f) {
                    if (is_file($f) && ($now - (int) @filemtime($f)) >= $ttl) {
                        @unlink($f);
                    }
                }
            }
        }
    }
}

/**
 * Optional DNS resolution mapping for curl (e.g. "example.com:443:1.2.3.4").
 *
 * Overridable via the FD_CURL_RESOLVE environment variable so a blocked
 * primary host can be pinned to a black-hole IP for fallback testing.
 */
define('FD_CURL_RESOLVE', (string) (getenv('FD_CURL_RESOLVE') ?: ''));

function fd_is_debug_enabled(): bool
{
    $file = FD_DEBUG_TOGGLE_PATH;
    if (is_file($file)) {
        $val = trim((string) @file_get_contents($file));
        return $val === '1' || strtolower($val) === 'true' || strtolower($val) === 'on';
    }
    // Check fallback locations for debug_mode.txt
    $candidates = [
        fd_storage_path('storage/debug_mode.txt'),
        fd_storage_path('debug_mode.txt'),
        fd_get_app_root() . DIRECTORY_SEPARATOR . 'storage' . DIRECTORY_SEPARATOR . 'debug_mode.txt',
        fd_get_app_root() . DIRECTORY_SEPARATOR . 'debug_mode.txt',
    ];
    foreach ($candidates as $cand) {
        if (is_file($cand)) {
            $val = trim((string) @file_get_contents($cand));
            if ($val === '1' || strtolower($val) === 'true' || strtolower($val) === 'on') {
                return true;
            }
        }
    }
    return false;
}

function fd_set_debug_enabled(bool $enabled): void
{
    $file = FD_DEBUG_TOGGLE_PATH;
    $dir = dirname($file);
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    if ($enabled) {
        @file_put_contents($file, '1', LOCK_EX);
    } else {
        @unlink($file);
    }
}

function fd_limit_file_size(string $filePath, int $maxBytes = FD_MAX_LOG_SIZE): void
{
    if (!is_file($filePath)) {
        return;
    }
    $size = @filesize($filePath);
    if ($size === false || $size <= $maxBytes) {
        return;
    }

    // Keep the most recent half of the file ($maxBytes / 2)
    $keepBytes = (int) ($maxBytes / 2);
    $fp = @fopen($filePath, 'r+');
    if (!$fp) {
        return;
    }

    if (@flock($fp, LOCK_EX)) {
        @fseek($fp, -$keepBytes, SEEK_END);
        // Advance to next newline to avoid truncated line
        @fgets($fp);
        $remaining = '';
        while (!feof($fp)) {
            $remaining .= fread($fp, 65536);
        }
        @ftruncate($fp, 0);
        @rewind($fp);
        @fwrite($fp, "[... truncated old log entries ...]\n" . $remaining);
        @fflush($fp);
        @flock($fp, LOCK_UN);
    }
    @fclose($fp);
}

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
    if (!fd_is_debug_enabled()) {
        return;
    }

    $suffix = '';
    if ($context !== []) {
        $suffix = ' ' . json_encode($context, JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES);
    }

    $logLine = '[' . date('Y-m-d H:i:s') . '] [PencariMovie Downloader] ' . $message . $suffix . "\n";
    $logPath = FD_DEBUG_LOG_PATH;
    $dir = dirname($logPath);
    if (!is_dir($dir)) {
        @mkdir($dir, 0777, true);
    }
    @file_put_contents($logPath, $logLine, FILE_APPEND | LOCK_EX);
    error_log('[PencariMovie Downloader] ' . $message . $suffix);

    // Limit debug.log size if it grows past 5MB
    fd_limit_file_size($logPath, FD_MAX_LOG_SIZE);
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

    // Fallback: pick the first active bot from bot_pool.json file directly (without calling fd_get_bot_pool to avoid recursion)
    $poolPath = FD_BOT_POOL_PATH;
    if (is_file($poolPath)) {
        $poolData = @json_decode((string) @file_get_contents($poolPath), true);
        if (is_array($poolData)) {
            foreach ($poolData as $b) {
                if (!empty($b['is_active']) && !empty($b['bot_id'])) {
                    return (string) $b['bot_id'];
                }
            }
            if (!empty($poolData[0]['bot_id'])) {
                return (string) $poolData[0]['bot_id'];
            }
        }
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

function fd_is_guest_provision_in_progress(): bool
{
    $lockFile = fd_storage_path('storage/guest_provision.lock');
    if (!file_exists($lockFile)) {
        return false;
    }

    // Staleness guard: a provision that crashed or was killed leaves the lock
    // file behind with no holder. Without this check the frontend polls
    // /api/session forever and shows "Provisioning guest bot..." in an endless
    // loop (observed: a 4-day-old 0-byte lock kept is_provisioning=true).
    //
    // IMPORTANT: on Windows a file held open by another process makes
    // filemtime() return false. Falling back to 0 would skip the guard and
    // reproduce the loop, so we use a SEPARATE heartbeat file that is never
    // flock()ed. The heartbeat is written when a provision starts and refreshed
    // while it runs; if it is missing or older than 300s the lock is stale.
    $heartbeatFile = fd_storage_path('storage/guest_provision.heartbeat');
    $hb = @filemtime($heartbeatFile);
    if ($hb === false) {
        // No heartbeat at all. Fall back to the lock's own mtime, and if even
        // that is unreadable treat the lock as stale (a live provision always
        // writes a heartbeat first).
        $lm = @filemtime($lockFile);
        if ($lm === false) {
            @unlink($lockFile);
            fd_log('reclaimed guest_provision.lock (no heartbeat, unreadable mtime)');
            return false;
        }
        $hb = $lm;
    }
    if ((time() - (int) $hb) > 300) {
        @unlink($lockFile);
        @unlink($heartbeatFile);
        fd_log('reclaimed stale guest_provision.lock', ['age_seconds' => time() - (int) $hb]);
        return false;
    }

    $fp = @fopen($lockFile, 'c+');
    if (!$fp) {
        return false;
    }
    $canLock = @flock($fp, LOCK_EX | LOCK_NB);
    if ($canLock) {
        @flock($fp, LOCK_UN);
        @fclose($fp);
        return false; // Not in progress
    }
    @fclose($fp);
    return true; // Another process is currently holding the provision lock
}

function fd_auto_provision_guest(): ?array
{
    fd_ensure_autoload();

    // ── Clock pre-check ───────────────────────────────────────────────────────
    // A skewed clock makes the MTProto auth key exchange fail with
    // ENCRYPTED_NOT_BOUND cancellation. Provisioning a new guest bot cannot fix
    // that, so refuse up front and tell the user the exact offset. Without this
    // guard every page load provisions yet another bot (observed: 8 bots in
    // 12 minutes with a 48s skew).
    $clockProbe = fd_measure_clock_offset();
    if ($clockProbe['ok'] && abs((int) $clockProbe['offset']) > 30) {
        $off = abs((int) $clockProbe['offset']);
        $mins = intdiv($off, 60);
        $secs = $off % 60;
        $human = $mins > 0 ? "{$mins}m {$secs}s" : "{$secs}s";
        $dir = (int) $clockProbe['offset'] > 0 ? 'ahead of' : 'behind';
        fd_log('auto provision aborted: clock skew too large', [
            'offset_seconds' => (int) $clockProbe['offset'],
        ]);
        return ['error' => "Device clock out of sync!\nYour clock is {$human} {$dir} Telegram's server time.\n\n"
            . "Enable 'Set time automatically' (Automatic date and time / NTP) in your device Settings, then restart the PencariMovie Server."];
    }

    // Check if an existing valid guest session is already active or in the pool.
    // Avoid re-provisioning and duplicating guest bots if one is already functioning.
    $existingBotId = fd_get_bot_id();
    if ($existingBotId !== '' && fd_has_local_session($existingBotId)) {
        $meta = fd_load_session_meta();
        [$madeline, $error] = fd_boot_madeline(null, [], $existingBotId);
        if ($madeline) {
            return [
                'bot_id' => $existingBotId,
                'bot_username' => (string) ($meta['bot_username'] ?? ''),
                'bot_name' => (string) ($meta['bot_name'] ?? ''),
                'madeline' => $madeline,
            ];
        }

        // A session directory exists but the boot failed. If the failure is a
        // clock-skew / auth-key problem, provisioning a NEW guest bot will not
        // help — it will fail the same way and create an endless loop of new
        // bots (observed: 8063426232 -> 8188066637 -> ...). Return the error so
        // the user fixes their clock instead.
        if ($error !== null && fd_is_clock_or_authkey_error((string) $error)) {
            fd_log('auto provision aborted: existing session failed with clock/auth-key error', [
                'bot_id' => $existingBotId,
                'error' => $error,
            ]);
            return ['error' => fd_friendly_login_error((string) $error)];
        }
    }

    // Also guard against a session directory that exists on disk but has no
    // bot_id recorded (e.g. a previous provision died mid-handshake). Without
    // this, every page load provisions yet another guest bot.
    if ($existingBotId === '') {
        $orphan = fd_find_orphan_session_bot_id();
        if ($orphan !== '') {
            fd_log('auto provision aborted: orphan session dir present without bot_id', ['orphan_bot_id' => $orphan]);
            return ['error' => 'A previous login did not finish. Please restart the PencariMovie Server and try again.'];
        }
    }

    // Use a non-blocking process file lock to prevent concurrent requests
    // (e.g. parallel Stremio stream calls, page reloads, or retries) from triggering
    // simultaneous guest bot provisions and creating multiple/duplicate guest bots.
    $lockDir = fd_storage_path('storage');
    if (!is_dir($lockDir)) {
        @mkdir($lockDir, 0777, true);
    }
    $lockFp = @fopen(fd_storage_path('storage/guest_provision.lock'), 'c+');
    if ($lockFp) {
        $locked = false;
        // Wait up to 30 seconds for any ongoing provision to finish
        $lockWaitStart = microtime(true);
        while (microtime(true) - $lockWaitStart < 30) {
            if (@flock($lockFp, LOCK_EX | LOCK_NB)) {
                $locked = true;
                // Heartbeat: write a SEPARATE file (never flock()ed) so the
                // staleness guard in fd_is_guest_provision_in_progress() can
                // read a fresh mtime even while this process holds the lock
                // open. On Windows filemtime() on a locked file returns false,
                // which would otherwise skip the guard and loop forever.
                @file_put_contents(fd_storage_path('storage/guest_provision.heartbeat'), (string) time(), LOCK_EX);
                break;
            }
            // During wait, check if session was successfully provisioned by another worker
            $midWaitBotId = fd_get_bot_id();
            if ($midWaitBotId !== '' && fd_has_local_session($midWaitBotId)) {
                $meta = fd_load_session_meta();
                [$madeline, $error] = fd_boot_madeline(null, [], $midWaitBotId);
                if ($madeline) {
                    @fclose($lockFp);
                    return [
                        'bot_id' => $midWaitBotId,
                        'bot_username' => (string) ($meta['bot_username'] ?? ''),
                        'bot_name' => (string) ($meta['bot_name'] ?? ''),
                        'madeline' => $madeline,
                    ];
                }
            }
            usleep(250000); // 250ms sleep
        }

        if (!$locked) {
            @fclose($lockFp);
            // Before returning null, check one final time if provision succeeded
            $finalBotId = fd_get_bot_id();
            if ($finalBotId !== '' && fd_has_local_session($finalBotId)) {
                $meta = fd_load_session_meta();
                [$madeline, $error] = fd_boot_madeline(null, [], $finalBotId);
                if ($madeline) {
                    return [
                        'bot_id' => $finalBotId,
                        'bot_username' => (string) ($meta['bot_username'] ?? ''),
                        'bot_name' => (string) ($meta['bot_name'] ?? ''),
                        'madeline' => $madeline,
                    ];
                }
            }
            fd_log('auto provision skipped: another provision is currently in progress');
            return null;
        }

        // Once lock is acquired, re-check if another process finished provisioning while we waited
        $recheckBotId = fd_get_bot_id();
        if ($recheckBotId !== '' && fd_has_local_session($recheckBotId)) {
            $meta = fd_load_session_meta();
            [$madeline, $error] = fd_boot_madeline(null, [], $recheckBotId);
            @flock($lockFp, LOCK_UN);
            @fclose($lockFp);
            if ($madeline) {
                return [
                    'bot_id' => $recheckBotId,
                    'bot_username' => (string) ($meta['bot_username'] ?? ''),
                    'bot_name' => (string) ($meta['bot_name'] ?? ''),
                    'madeline' => $madeline,
                ];
            }
        }
    }

    $lastError = null;
    try {
        // Try up to 2 times with a fast 6s timeout so page load doesn't hang
        for ($pAttempt = 0; $pAttempt < 2; $pAttempt++) {
            $provisionUrl = FD_WP_API_BASE . '/provision-session?_nocache=' . time() . '_' . mt_rand(1000, 9999);
            $resp = fd_http_json($provisionUrl, [], 'GET', 6);

            if (empty($resp['ok']) || empty($resp['bot_token'])) {
                fd_log('provision endpoint returned error or incomplete payload', ['resp' => $resp, 'attempt' => $pAttempt + 1]);
                continue;
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

            // Use numeric bot_id directly from the response so no directory rename is needed,
            // preventing Windows file lock contention on rename while session is active.
            $targetBotId = !empty($resp['bot_id']) ? (string) $resp['bot_id'] : '';
            if ($targetBotId === '') {
                $targetBotId = 'prov_' . substr(hash('sha256', $botToken), 0, 10);
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
                $lastError = $error;
                fd_log('auto provision fd_boot_madeline failed', ['error' => $error, 'bot_id' => $targetBotId, 'attempt' => $pAttempt + 1]);
                // If clock is out of sync, stop retrying and return immediately so user sees clock error
                if ($error && str_contains(strtolower($error), 'clock')) {
                    return ['error' => $error];
                }
                continue;
            }

            try {
                $self = $madeline->getSelf();
                $botId = (string) ($self['id'] ?? $targetBotId);
                if ($botId === '') {
                    throw new \RuntimeException('Bot ID is empty after getSelf');
                }

                // If targetBotId was temporary ('prov_...'), move files to real numeric botId
                if ($targetBotId !== '' && $targetBotId !== $botId) {
                    $oldPath = dirname(fd_get_bot_session_path($targetBotId));
                    $newPath = dirname(fd_get_bot_session_path($botId));
                    if ($oldPath !== $newPath && is_dir($oldPath)) {
                        if (!is_dir($newPath)) {
                            @mkdir($newPath, 0777, true);
                        }
                        // Copy/move all files inside $oldPath into $newPath so opened file locks on Windows don't break directory rename
                        $srcFiles = @scandir($oldPath);
                        if ($srcFiles) {
                            foreach ($srcFiles as $sf) {
                                if ($sf === '.' || $sf === '..') continue;
                                $srcFile = $oldPath . DIRECTORY_SEPARATOR . $sf;
                                $dstFile = $newPath . DIRECTORY_SEPARATOR . $sf;
                                if (is_file($srcFile)) {
                                    @copy($srcFile, $dstFile);
                                }
                            }
                        }
                        @rename($oldPath, $newPath);
                    }
                }

                $botUsername = (string) ($self['username'] ?? '');
                $botName = (string) ($self['first_name'] ?? '');

                fd_save_session_meta($botId, $botUsername, $botName);
                fd_add_pool_bot([
                    'bot_id' => $botId,
                    'bot_username' => $botUsername,
                    'bot_name' => $botName,
                    'status' => 'online',
                    'is_active' => true,
                ]);

                return [
                    'bot_id' => $botId,
                    'bot_username' => $botUsername,
                    'bot_name' => $botName,
                    'madeline' => $madeline,
                ];
            } catch (Throwable $e) {
                fd_log('auto provision getSelf failed', ['error' => $e->getMessage(), 'attempt' => $pAttempt + 1]);
                continue;
            }
        }

        if ($lastError) {
            return ['error' => $lastError];
        }
        return null;
    } finally {
        if (is_resource($lockFp)) {
            @flock($lockFp, LOCK_UN);
            @fclose($lockFp);
        }
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
    // cloudflared proxies as 127.0.0.1. Treat TryCloudflare / CF headers as remote.
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

    return false;
}

function fd_require_local_request(): void
{
    if (!fd_is_local_request()) {
        fd_json(['ok' => 0, 'message' => 'This endpoint is restricted to local requests only.'], 403);
    }
}

// ─── Server password auth ───────────────────────────────────────────────────
// A single password (default 123456) gates the admin surface and the streams
// list. Localhost / private-LAN requests bypass it so local installs are
// unaffected. Remote clients pass a token as a /t/<token>/ path segment.

function fd_auth_load(): array
{
    $path = FD_AUTH_PATH;
    $data = [];
    if (is_file($path)) {
        $decoded = json_decode((string) @file_get_contents($path), true);
        if (is_array($decoded)) {
            $data = $decoded;
        }
    }
    if (empty($data['password_hash'])) {
        $data['password_hash'] = password_hash(FD_AUTH_DEFAULT_PASSWORD, PASSWORD_DEFAULT);
    }
    if (empty($data['token'])) {
        $data['token'] = bin2hex(random_bytes(16));
    }
    if (!isset($data['enabled'])) {
        $data['enabled'] = true;
    }
    if (!is_file($path)) {
        fd_auth_save($data);
    }
    return $data;
}

function fd_auth_save(array $data): void
{
    @file_put_contents(FD_AUTH_PATH, json_encode($data, JSON_UNESCAPED_SLASHES), LOCK_EX);
}

function fd_auth_enabled(): bool
{
    return !empty(fd_auth_load()['enabled']);
}

function fd_auth_token(): string
{
    return (string) (fd_auth_load()['token'] ?? '');
}

function fd_auth_rotate_token(): string
{
    $data = fd_auth_load();
    $data['token'] = bin2hex(random_bytes(16));
    fd_auth_save($data);
    return $data['token'];
}

function fd_auth_verify_password(string $pw): bool
{
    $hash = (string) (fd_auth_load()['password_hash'] ?? '');
    return $hash !== '' && password_verify($pw, $hash);
}

function fd_auth_set_password(string $pw): void
{
    $data = fd_auth_load();
    $data['password_hash'] = password_hash($pw, PASSWORD_DEFAULT);
    $data['token'] = bin2hex(random_bytes(16));
    $data['enabled'] = true;
    fd_auth_save($data);
}

function fd_auth_client_ip(): string
{
    if (fd_is_cloudflare_tunnel_request() && !empty($_SERVER['HTTP_CF_CONNECTING_IP'])) {
        return (string) $_SERVER['HTTP_CF_CONNECTING_IP'];
    }
    return (string) ($_SERVER['REMOTE_ADDR'] ?? '127.0.0.1');
}

/**
 * Progressive lockout ladder (same as 9router):
 * Tier 0: 30s, Tier 1: 120s (2m), Tier 2: 600s (10m), Tier 3+: 1800s (30m)
 * Locked out after 5 consecutive failed attempts.
 */
function fd_auth_lockout_check(string $ip): ?int
{
    $lockFile = fd_storage_path('storage/cache/auth_lockout.json');
    if (!is_file($lockFile)) {
        return null;
    }
    $state = json_decode((string) @file_get_contents($lockFile), true) ?: [];
    $entry = $state[$ip] ?? null;
    if (!$entry || empty($entry['lockUntil'])) {
        return null;
    }
    $now = time();
    if ($entry['lockUntil'] > $now) {
        return (int) ($entry['lockUntil'] - $now);
    }
    return null;
}

function fd_auth_record_failure(string $ip): array
{
    $lockFile = fd_storage_path('storage/cache/auth_lockout.json');
    $dir = dirname($lockFile);
    if (!is_dir($dir)) {
        @mkdir($dir, 0755, true);
    }
    $state = is_file($lockFile) ? (json_decode((string) @file_get_contents($lockFile), true) ?: []) : [];
    $now = time();

    // Prune entries older than 1 hour
    foreach ($state as $k => $v) {
        if (!empty($v['lastFailAt']) && ($now - $v['lastFailAt'] > 3600) && empty($v['lockUntil'])) {
            unset($state[$k]);
        }
    }

    $tiers = [30, 120, 600, 1800];
    $entry = $state[$ip] ?? ['fails' => 0, 'lockUntil' => 0, 'lockLevel' => 0, 'lastFailAt' => 0];
    $entry['fails'] = ($entry['fails'] ?? 0) + 1;
    $entry['lastFailAt'] = $now;

    $retryAfter = 0;
    if ($entry['fails'] >= 5) {
        $level = min($entry['lockLevel'] ?? 0, count($tiers) - 1);
        $duration = $tiers[$level];
        $entry['lockUntil'] = $now + $duration;
        $entry['lockLevel'] = $level + 1;
        $entry['fails'] = 0;
        $retryAfter = $duration;
    }

    $state[$ip] = $entry;
    @file_put_contents($lockFile, json_encode($state, JSON_UNESCAPED_SLASHES), LOCK_EX);

    return [
        'locked' => $retryAfter > 0,
        'retryAfter' => $retryAfter,
        'remaining' => max(0, 5 - ($entry['fails'] ?? 0)),
    ];
}

function fd_auth_record_success(string $ip): void
{
    $lockFile = fd_storage_path('storage/cache/auth_lockout.json');
    if (!is_file($lockFile)) {
        return;
    }
    $state = json_decode((string) @file_get_contents($lockFile), true) ?: [];
    if (isset($state[$ip])) {
        unset($state[$ip]);
        @file_put_contents($lockFile, json_encode($state, JSON_UNESCAPED_SLASHES), LOCK_EX);
    }
}

/**
 * Extract the auth token from the request: /t/<token>/ path prefix, ?token=,
 * X-Auth-Token header, or the pm_auth cookie.
 */
function fd_auth_token_from_request(): string
{
    $path = (string) parse_url((string) ($_SERVER['REQUEST_URI'] ?? ''), PHP_URL_PATH);
    // Support clean /<token>/manifest.json or /<token>/stream/... (32-char hex token)
    if (preg_match('#^/([0-9a-fA-F]{32})(?:/|$)#', $path, $m)) {
        return strtolower($m[1]);
    }
    if (preg_match('#^/t/([A-Za-z0-9]+)(?:/|$)#', $path, $m)) {
        return $m[1];
    }
    $q = trim((string) ($_GET['token'] ?? ''));
    if ($q !== '') {
        return $q;
    }
    $h = trim((string) ($_SERVER['HTTP_X_AUTH_TOKEN'] ?? ''));
    if ($h !== '') {
        return $h;
    }
    return trim((string) ($_COOKIE[FD_AUTH_COOKIE] ?? ''));
}

/**
 * True when the request may access protected routes. Localhost and private
 * LAN requests always pass so local installs never see a password prompt.
 */
function fd_is_authenticated(): bool
{
    if (!fd_auth_enabled()) {
        return true;
    }
    if (fd_is_local_request()) {
        return true;
    }
    $token = fd_auth_token_from_request();
    return $token !== '' && hash_equals(fd_auth_token(), $token);
}

function fd_require_auth(): void
{
    if (!fd_is_authenticated()) {
        fd_json(['ok' => 0, 'message' => 'Password required', 'auth_required' => true], 401);
    }
}

/**
 * Strip a leading /t/<token> segment so existing route matching still works.
 */
function fd_strip_token_prefix(string $path): string
{
    if (preg_match('#^/[0-9a-fA-F]{32}(/.*)?$#', $path, $m)) {
        return (!empty($m[1])) ? $m[1] : '/';
    }
    if (preg_match('#^/t/[A-Za-z0-9]+(/.*)?$#', $path, $m)) {
        return $m[1] ?? '/';
    }
    return $path;
}

/**
 * Stremio-shaped locked stream: tells the user to re-install from #addon.
 */
function fd_stremio_locked_stream(string $baseUrl): never
{
    fd_stremio_json([
        'streams' => [[
            'name' => 'PencariMovie',
            'description' => "Addon URL changed or password required.\nOpen PencariMovie and re-install the addon from #addon.",
            'externalUrl' => rtrim($baseUrl, '/') . '/#addon',
        ]],
    ], 200, 'no-cache, no-store, must-revalidate');
}

/**
 * Eclipse-shaped locked response (Eclipse expects {error}, not {streams}).
 */
function fd_eclipse_locked_response(string $baseUrl): never
{
    fd_stremio_json([
        'error' => "Addon URL changed or password required. Open PencariMovie and re-install the addon from #addon.",
        'externalUrl' => rtrim($baseUrl, '/') . '/#addon',
    ], 200, 'no-cache, no-store, must-revalidate');
}

/**
 * Minimal standalone password page. Deliberately does NOT load app.js or the
 * dashboard CSS so no dashboard markup leaks to unauthenticated visitors.
 */
function fd_serve_auth_gate_html(): void
{
    if (!headers_sent()) {
        header('Content-Type: text/html; charset=utf-8');
        header('Cache-Control: no-store, no-cache, must-revalidate');
    }
    echo <<<'HTML'
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
</div>
<script>
document.getElementById('f').addEventListener('submit', async function(ev){
  ev.preventDefault();
  var b=document.getElementById('b'), e=document.getElementById('e');
  b.disabled=true; e.textContent='';
  try{
    var r=await fetch('/api/auth/login',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({password:document.getElementById('pw').value})});
    var d=await r.json();
    if(d.ok){location.reload();return;}
    if(r.status===429&&d.retryAfter){
      var rem=d.retryAfter;
      e.textContent='Too many attempts. Locked for '+rem+'s.';
      var t=setInterval(function(){
        rem--;
        if(rem<=0){clearInterval(t);b.disabled=false;e.textContent='You can try again now.';}
        else{e.textContent='Too many attempts. Locked for '+rem+'s.';}
      },1000);
      return;
    }
    e.textContent=d.message||'Wrong password';
  }catch(err){e.textContent='Connection failed';}
  b.disabled=false;
});
</script></body></html>
HTML;
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

/**
 * Build `CURLOPT_RESOLVE` entries for every anycast-pinned host.
 *
 * @return string[] e.g. ['pencarimovie.com:443:104.21.47.164', ...]
 */
function fd_curl_resolve_entries(): array
{
    $entries = [];
    foreach (fd_get_upstream_manifest_hosts() as $host => $ips) {
        foreach ($ips as $ip) {
            $entries[] = "{$host}:443:{$ip}";
            $entries[] = "{$host}:80:{$ip}";
        }
    }
    return $entries;
}

/**
 * Resolve a hostname to a list of IPv4 addresses using the system resolver.
 *
 * @return string[] Unique IPv4 addresses (empty when resolution fails).
 */
function fd_resolve_host_ips(string $host): array
{
    $host = strtolower(trim($host));
    if ($host === '') {
        return [];
    }

    $ips = [];
    $records = @dns_get_record($host, DNS_A);
    if (is_array($records)) {
        foreach ($records as $rec) {
            if (!empty($rec['ip']) && filter_var($rec['ip'], FILTER_VALIDATE_IP, FILTER_FLAG_IPV4)) {
                $ips[] = (string) $rec['ip'];
            }
        }
    }
    if (empty($ips)) {
        $ip = @gethostbyname($host);
        if (is_string($ip) && $ip !== $host && filter_var($ip, FILTER_VALIDATE_IP, FILTER_FLAG_IPV4)) {
            $ips[] = $ip;
        }
    }

    return array_values(array_unique($ips));
}

/**
 * Resolve the hosts that must be anycast-pinned for Android/Termux.
 *
 * Covers the app's own API host, Cinemeta, and every configured upstream
 * Stremio manifest host. Results are cached on disk for 24h to avoid a DNS
 * lookup on every request. Hardcoded Cloudflare IPs are used as a fallback
 * when resolution fails (e.g. on Termux with no system DNS).
 *
 * @return array<string,string[]> host => list of IPs
 */
function fd_get_upstream_manifest_hosts(): array
{
    static $cached = null;
    if ($cached !== null) {
        return $cached;
    }
    $cached = [];

    $cacheFile = fd_storage_path('storage/manifest_hosts.json');
    if (is_file($cacheFile) && (time() - (int) @filemtime($cacheFile)) < 86400) {
        $data = json_decode((string) @file_get_contents($cacheFile), true);
        if (is_array($data) && !empty($data)) {
            // Self-heal: a cache written before the telegra.my fallback existed
            // would omit the fallback host, breaking the DPI bypass on
            // Android/Termux (no DNS). Merge it in without a full refresh.
            if (
                defined('FD_WP_FALLBACK_HOST') && FD_WP_FALLBACK_HOST !== ''
                && empty($data[FD_WP_FALLBACK_HOST])
            ) {
                $fbIps = fd_resolve_host_ips(FD_WP_FALLBACK_HOST);
                if (empty($fbIps)) {
                    $fbIps = ['104.21.15.194', '172.67.163.205'];
                }
                $data[FD_WP_FALLBACK_HOST] = $fbIps;
                @file_put_contents($cacheFile, json_encode($data, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES), LOCK_EX);
            }
            return $cached = $data;
        }
    }

    // Known-good Cloudflare anycast IPs used when live resolution fails.
    $fallbacks = [
        'pencarimovie.com' => ['104.21.47.164', '172.67.149.53'],
        // telegra.my is the DPI-firewall fallback host (see fd_fallback_url()).
        // Pinned so the fallback also works on Android/Termux with no DNS.
        'telegra.my' => ['104.21.15.194', '172.67.163.205'],
        'v3-cinemeta.strem.io' => ['104.17.88.107', '104.17.89.107'],
        'opensubtitles-v3.strem.io' => ['104.17.88.107', '104.17.89.107'],
        'subs5.strem.io' => ['104.17.88.107', '104.17.89.107'],
        'opensubtitlesv3-pro.dexter21767.com' => ['104.21.21.22', '172.67.195.253'],
    ];

    // Hosts to pin: app API + Cinemeta + every configured upstream manifest.
    $hostsToResolve = array_keys($fallbacks);

    $catSettings = fd_load_catalog_settings();
    $upstreams = (array) ($catSettings['upstream_manifests'] ?? []);
    foreach ($upstreams as $upstream) {
        $manifestUrl = trim((string) ($upstream['url'] ?? ''));
        if ($manifestUrl === '') {
            continue;
        }
        $host = strtolower((string) parse_url($manifestUrl, PHP_URL_HOST));
        if ($host !== '') {
            $hostsToResolve[] = $host;
        }
    }
    $hostsToResolve = array_values(array_unique($hostsToResolve));

    $hosts = [];
    foreach ($hostsToResolve as $host) {
        $ips = fd_resolve_host_ips($host);
        if (empty($ips) && isset($fallbacks[$host])) {
            // Resolution failed (Termux) — fall back to the known Cloudflare IPs.
            $ips = $fallbacks[$host];
        }
        if (!empty($ips)) {
            $hosts[$host] = $ips;
        }
    }

    if (!empty($hosts)) {
        @file_put_contents($cacheFile, json_encode($hosts, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES), LOCK_EX);
    }

    return $cached = $hosts;
}

/**
 * Singleton Amp HTTP client configured with an Anycast DNS Resolver.
 */
function fd_get_amphp_client(): ?\Amp\Http\Client\HttpClient
{
    static $client = null;
    if ($client === null) {
        if (!fd_ensure_autoload() || !interface_exists('Amp\\Dns\\DnsResolver') || !class_exists('Amp\\Http\\Client\\HttpClientBuilder')) {
            return null;
        }

        if (!class_exists('FdAnycastDnsResolver', false)) {
            class FdAnycastDnsResolver implements \Amp\Dns\DnsResolver
            {
                private array $staticHosts;
                private ?\Amp\Dns\DnsResolver $fallback;

                public function __construct(array $staticHosts = [], ?\Amp\Dns\DnsResolver $fallback = null)
                {
                    $this->staticHosts = $staticHosts;
                    $this->fallback = $fallback;
                }

                public function resolve(string $name, ?int $typeRestriction = null, ?\Amp\Cancellation $cancellation = null): array
                {
                    $lower = strtolower($name);
                    if (isset($this->staticHosts[$lower])) {
                        $records = [];
                        foreach ((array) $this->staticHosts[$lower] as $ip) {
                            $type = str_contains($ip, ':') ? \Amp\Dns\DnsRecord::AAAA : \Amp\Dns\DnsRecord::A;
                            if ($typeRestriction === null || $typeRestriction === $type) {
                                $records[] = new \Amp\Dns\DnsRecord($ip, $type, 3600);
                            }
                        }
                        if (!empty($records)) {
                            return $records;
                        }
                    }
                    $fallback = $this->fallback ?? \Amp\Dns\createDefaultResolver();
                    return $fallback->resolve($name, $typeRestriction, $cancellation);
                }

                public function query(string $name, int $type, ?\Amp\Cancellation $cancellation = null): array
                {
                    return $this->resolve($name, $type, $cancellation);
                }
            }
        }

        // Dynamically resolved hosts (app API + Cinemeta + configured manifests),
        // with hardcoded Cloudflare IPs as fallback when DNS is unavailable.
        $staticHosts = fd_get_upstream_manifest_hosts();

        if (defined('FD_CURL_RESOLVE') && FD_CURL_RESOLVE !== '') {
            $parts = explode(':', FD_CURL_RESOLVE);
            if (count($parts) >= 3) {
                $staticHosts[strtolower($parts[0])] = [$parts[2]];
            }
        }

        $customResolver = new FdAnycastDnsResolver($staticHosts);
        \Amp\Dns\dnsResolver($customResolver);

        // Disable TLS certificate peer verification on Amp to prevent handshake errors
        // on Android Termux / PRoot where system CA bundles are missing or unbundled
        $tlsContext = (new \Amp\Socket\ClientTlsContext(''))->withoutPeerVerification();
        $connectContext = (new \Amp\Socket\ConnectContext())->withTlsContext($tlsContext);
        $factory = new \Amp\Http\Client\Connection\DefaultConnectionFactory(null, $connectContext);
        $pool = new \Amp\Http\Client\Connection\UnlimitedConnectionPool($factory);

        $client = (new \Amp\Http\Client\HttpClientBuilder())
            ->usingPool($pool)
            ->build();
    }
    return $client;
}

/**
 * Execute an HTTP request via Amp\Http\Client with Anycast DNS resolution.
 */
function fd_amphp_http_request(string $url, string $method, array $headers, string $body, int $timeout): string|false
{
    $client = fd_get_amphp_client();
    if ($client === null) {
        return false;
    }
    $request = new \Amp\Http\Client\Request($url, $method);

    foreach ($headers as $h) {
        $parts = explode(':', $h, 2);
        if (count($parts) === 2) {
            $request->setHeader(trim($parts[0]), trim($parts[1]));
        }
    }
    if ($body !== '') {
        $request->setBody($body);
    }

    $tStart = microtime(true);
    // Ensure a safe timeout of at least 10 seconds to accommodate TLS handshakes
    $effectiveTimeout = max(10, $timeout);
    $cancellation = new \Amp\TimeoutCancellation($effectiveTimeout);
    $response = $client->request($request, $cancellation);
    $status = $response->getStatus();

    // Capture version headers if present
    foreach ($response->getHeaders() as $name => $values) {
        $lName = strtolower($name);
        if (in_array($lName, ['x-min-version', 'x-update-url', 'x-update-required', 'x-sponsor-name', 'x-sponsor-desc', 'x-sponsor-url'], true)) {
            $val = is_array($values) ? end($values) : (string) $values;
            fd_update_version_state([$lName => $val]);
        }
    }

    $resBody = $response->getBody()->buffer($cancellation);
    $tDuration = round(microtime(true) - $tStart, 3);

    if ($status < 200 || $status >= 400) {
        fd_log('amphp request failed status', [
            'url' => $url,
            'http_code' => $status,
            'duration_seconds' => $tDuration,
        ]);
        if ($resBody === '') {
            return false;
        }
    } else {
        fd_log('amphp request completed', [
            'url' => $url,
            'http_code' => $status,
            'duration_seconds' => $tDuration,
            'bytes' => strlen($resBody),
        ]);
    }

    return $resBody;
}

/**
 * Rewrite a pencarimovie.com URL to the telegra.my Cloudflare Worker fallback.
 *
 * Used when an in-path DPI firewall resets the TLS handshake on the
 * "pencarimovie.com" SNI. Returns null when the URL is not a primary-domain URL
 * (so callers can skip the retry).
 */
function fd_fallback_url(string $url): ?string
{
    if (!defined('FD_WP_FALLBACK_HOST') || FD_WP_FALLBACK_HOST === '') {
        return null;
    }
    if (stripos($url, 'pencarimovie.com') === false) {
        return null;
    }
    return str_ireplace('pencarimovie.com', FD_WP_FALLBACK_HOST, $url);
}

/**
 * True when a failed request looks like an in-path firewall TLS reset rather
 * than a genuine upstream error. cURL reports errno 35 (SSL connect error) /
 * 56 (recv failure) / 7 (couldn't connect) with an empty HTTP code.
 */
function fd_is_connection_reset(?int $errno, int $httpCode, string $error): bool
{
    if ($httpCode > 0) {
        return false;
    }
    if (in_array($errno, [7, 35, 52, 56], true)) {
        return true;
    }
    $needle = strtolower($error);
    foreach (['reset by peer', 'tls negotiation failed', 'ssl connect error', 'connection refused', 'timed out'] as $frag) {
        if ($needle !== '' && str_contains($needle, $frag)) {
            return true;
        }
    }
    return false;
}

function fd_http_get_contents(string $url, array $options = []): string|false
{
    $method = strtoupper($options['method'] ?? 'GET');
    $headers = $options['headers'] ?? [];
    $body = $options['body'] ?? '';
    $timeout = max(10, (int) ($options['timeout'] ?? 15));
    // Set by the fallback retry so a failed telegra.my request cannot recurse.
    $noFallback = !empty($options['_fd_no_fallback']);

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

    // 1. Preferred async/HTTP client: Amp\Http\Client with Custom Anycast DNS Resolver
    if (fd_ensure_autoload() && class_exists('Amp\\Http\\Client\\HttpClientBuilder')) {
        // The Amp client is fast when warm (~155ms) but a slow WordPress response
        // can exceed the timeout and raise "The operation was cancelled". A single
        // retry is nearly free (the connection is already pooled) and recovers the
        // request instead of falling through to the slower cURL path.
        $ampAttempts = 2;
        for ($ampTry = 1; $ampTry <= $ampAttempts; $ampTry++) {
            try {
                $response = fd_amphp_http_request($url, $method, $headers, $body, $timeout);
                if ($response !== false && $response !== '') {
                    return $response;
                }
                // Empty body with a 2xx is a valid (if unusual) response — stop.
                break;
            } catch (\Throwable $e) {
                $msg = $e->getMessage();
                $isCancelled = stripos($msg, 'cancelled') !== false
                    || stripos($msg, 'timeout') !== false;
                // In-path DPI firewall resets the TLS handshake on the
                // "pencarimovie.com" SNI. Amp surfaces this as a socket/TLS
                // error, not a timeout — retry once via the telegra.my Worker.
                $isReset = !$isCancelled && (
                    stripos($msg, 'reset') !== false
                    || stripos($msg, 'tls') !== false
                    || stripos($msg, 'handshake') !== false
                    || stripos($msg, 'socket') !== false
                    || stripos($msg, 'connection') !== false
                );
                fd_log('amphp request exception', [
                    'url' => $url,
                    'error' => $msg,
                    'attempt' => $ampTry,
                    'will_retry' => $isCancelled && $ampTry < $ampAttempts,
                ]);
                if ($isReset && !$noFallback) {
                    $fallbackUrl = fd_fallback_url($url);
                    if ($fallbackUrl !== null) {
                        fd_log('amphp fallback to telegra.my', [
                            'url' => $url,
                            'fallback_url' => $fallbackUrl,
                            'error' => $msg,
                        ]);
                        $fbOptions = $options;
                        $fbOptions['_fd_no_fallback'] = true;
                        return fd_http_get_contents($fallbackUrl, $fbOptions);
                    }
                }
                if (!$isCancelled || $ampTry >= $ampAttempts) {
                    break;
                }
                // Brief pause before the retry so a transient backend stall clears.
                usleep(150000);
            }
        }
    }

    // 2. Direct cURL fetch (fallback) with Anycast DNS resolution
    if (function_exists('curl_version')) {
        $ch = curl_init();
        // Build base curl options
        $curlOpts = [
            CURLOPT_URL => $url,
            CURLOPT_RETURNTRANSFER => true,
            CURLOPT_TIMEOUT => $timeout,
            // Give the connect phase most of the budget. A 5s connect timeout
            // failed on cold TLS handshakes to Cloudflare; 10s is reliable.
            CURLOPT_CONNECTTIMEOUT => max(10, (int) ($timeout * 0.6)),
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
            // Anycast-pin the app API, Cinemeta, and every configured upstream
            // manifest host so Android / Termux can reach them without working
            // system DNS. IPs are resolved dynamically (24h cache) with
            // hardcoded Cloudflare IPs as fallback.
            foreach (fd_get_upstream_manifest_hosts() as $host => $ips) {
                foreach ($ips as $ip) {
                    $resolveEntries[] = "{$host}:443:{$ip}";
                    $resolveEntries[] = "{$host}:80:{$ip}";
                }
            }
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

        $cErrno = curl_errno($ch);
        $cError = curl_error($ch);

        if ($response === false) {
            fd_log('curl request failed', [
                'url' => $url,
                'http_code' => $httpCode,
                'duration_seconds' => $cDuration,
                'errno' => $cErrno,
                'error' => $cError,
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
            // In-path DPI firewall (corporate / campus / hospital) resets the
            // TLS handshake on the "pencarimovie.com" SNI. Retry once through
            // the telegra.my Cloudflare Worker, which proxies back to the
            // origin over Cloudflare's internal backbone.
            $fallbackUrl = $noFallback ? null : fd_fallback_url($url);
            if ($fallbackUrl !== null && fd_is_connection_reset($cErrno, $httpCode, $cError)) {
                fd_log('curl fallback to telegra.my', [
                    'url' => $url,
                    'fallback_url' => $fallbackUrl,
                    'errno' => $cErrno,
                    'error' => $cError,
                ]);
                $fbOptions = $options;
                $fbOptions['_fd_no_fallback'] = true;
                return fd_http_get_contents($fallbackUrl, $fbOptions);
            }
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

/**
 * Resolve external media ID (IMDb tt..., TMDB tmdb:..., Kitsu kitsu:..., etc.)
 * to metadata: { title, year, season, episode, imdb_id }.
 *
 * Uses multi-tier resolution:
 * 1. 24-hour disk cache in storage/cinemeta_{md5}.json
 * 2. Cinemeta for IMDb tt... IDs
 * 3. AIOStreams meta proxy for TMDB tmdb:... IDs (fast and reliable)
 * 4. Kitsu API for kitsu:... anime IDs
 */
/**
 * Look up a stored external ID via the standalone `media_ids_idx` table.
 *
 * `media_ids_idx` is self-contained (title, year, imdb_id, media_type are
 * stored on the row), so this works WITHOUT wp_posts or posts_idx. This is the
 * primary resolver used to serve streams for other addons.
 *
 * Uses a GENERIC id map (prefix + value), so it works for the standard
 * Stremio prefixes (tt, tmdb, kitsu, mal, anilist, tvdb, anidb) AND any
 * custom prefix a user's manifest.json declares — no schema change needed.
 *
 * @param string $prefix  Lowercase prefix (tmdb, kitsu, mal, anilist, tvdb, anidb, custom...)
 * @param string $value   ID value (numeric or string, e.g. "1108427" or "tt0111161")
 * @return array{title:string,year:string,imdb_id:string,media_type:string}
 */
function fd_lookup_local_catalog_by_prefix(string $prefix, string $value): array
{
    $empty = ['title' => '', 'year' => '', 'imdb_id' => '', 'media_type' => ''];
    $prefix = strtolower(trim($prefix));
    $value = trim($value);
    if ($prefix === '' || $value === '') {
        return $empty;
    }

    // 1. Standalone local Manticore media_ids_idx lookup (fastest).
    $hit = fd_manticore_lookup_by_id($prefix, $value);
    if (!empty($hit['title'])) {
        return $hit;
    }

    // 2. Fallback: Query remote WordPress /lookup-id (which checks server's media_ids_idx & postmeta).
    if (defined('FD_WP_API_BASE')) {
        $url = FD_WP_API_BASE . '/lookup-id?' . http_build_query(['prefix' => $prefix, 'id' => $value]);
        $res = fd_http_json($url, [], 'GET', 3);
        if (!empty($res['title'])) {
            $resolved = [
                'title'      => (string) $res['title'],
                'year'       => (string) ($res['year'] ?? ''),
                'imdb_id'    => (string) ($res['imdb_id'] ?? ''),
                'media_type' => (string) ($res['media_type'] ?? 'movie'),
            ];
            // Cache locally into media_ids_idx if local Manticore is reachable
            $postId = (int) ($res['post_id'] ?? 0);
            if ($postId > 0) {
                fd_media_ids_upsert($postId, [$prefix => $value], $resolved['title'], (int)$resolved['year'], $resolved['imdb_id'], $resolved['media_type']);
            }
            return $resolved;
        }
    }

    return $empty;
}

/**
 * Open a connection to the local Manticore instance (MySQL protocol, 127.0.0.1:9306).
 *
 * @return \mysqli|null
 */
function fd_manticore_connect(): ?\mysqli
{
    if (!function_exists('mysqli_connect')) {
        return null;
    }
    // PHP 8.1+ throws mysqli_sql_exception by default; suppress so a missing
    // local Manticore just returns null instead of fataling the request.
    if (function_exists('mysqli_report')) {
        @mysqli_report(MYSQLI_REPORT_OFF);
    }
    $conn = @mysqli_connect('127.0.0.1', '', '', '', 9306);
    return $conn instanceof \mysqli ? $conn : null;
}

/**
 * Standalone lookup for a stored external ID via the generic `media_ids_idx`
 * table (prefix + value attributes).
 *
 * `media_ids_idx` is self-contained: it stores title, year, imdb_id and
 * media_type alongside the (prefix, value) pair, so stream resolution for
 * other addons works WITHOUT needing wp_posts or posts_idx.
 *
 * @return array{title:string,year:string,imdb_id:string,media_type:string}
 */
function fd_manticore_lookup_by_id(string $prefix, string $value): array
{
    $empty = ['title' => '', 'year' => '', 'imdb_id' => '', 'media_type' => ''];
    $prefix = strtolower(trim($prefix));
    $value = trim($value);
    if ($prefix === '' || $value === '') {
        return $empty;
    }

    $conn = fd_manticore_connect();
    if ($conn === null) {
        return $empty;
    }

    $escPrefix = @mysqli_real_escape_string($conn, $prefix);
    $escValue = @mysqli_real_escape_string($conn, $value);
    $sql = "SELECT post_id, title, year, imdb_id, media_type FROM media_ids_idx "
        . "WHERE prefix = '{$escPrefix}' AND value = '{$escValue}' LIMIT 1";
    $res = @mysqli_query($conn, $sql);
    $row = $res ? @mysqli_fetch_assoc($res) : null;
    @mysqli_close($conn);

    if (empty($row['title'])) {
        return $empty;
    }

    return [
        'title'      => (string) $row['title'],
        'year'       => !empty($row['year']) ? (string) $row['year'] : '',
        'imdb_id'    => (string) ($row['imdb_id'] ?? ''),
        'media_type' => (string) ($row['media_type'] ?? ''),
    ];
}

/**
 * Upsert one or more (prefix => value) ID rows into the standalone
 * `media_ids_idx` table.
 *
 * This is the ONLY write path needed to make external IDs resolvable for
 * other addons. It does not require wp_posts or posts_idx to exist.
 *
 * @param int                  $postId    Catalog post id (used as the row key)
 * @param array<string,string> $mediaIds  Map of prefix => value
 * @param string               $title     Display title
 * @param int                  $year      Release year
 * @param string               $imdbId    IMDb id (tt...) when known
 * @param string               $mediaType 'movie' | 'tvseries'
 * @return int Number of rows written (0 on failure)
 */
function fd_media_ids_upsert(int $postId, array $mediaIds, string $title, int $year = 0, string $imdbId = '', string $mediaType = 'movie'): int
{
    if ($postId <= 0 || empty($mediaIds)) {
        return 0;
    }

    $conn = fd_manticore_connect();
    if ($conn === null) {
        return 0;
    }

    $escTitle = @mysqli_real_escape_string($conn, $title);
    $escImdb = @mysqli_real_escape_string($conn, $imdbId);
    $escType = @mysqli_real_escape_string($conn, $mediaType);

    $rows = [];
    foreach ($mediaIds as $prefix => $value) {
        $prefix = strtolower(trim((string) $prefix));
        $value = trim((string) $value);
        if ($prefix === '' || $value === '') {
            continue;
        }
        $escPrefix = @mysqli_real_escape_string($conn, $prefix);
        $escValue = @mysqli_real_escape_string($conn, $value);
        $rows[] = "({$postId}, '{$escPrefix}', '{$escValue}', '{$escTitle}', {$year}, '{$escImdb}', '{$escType}')";
    }

    if (empty($rows)) {
        @mysqli_close($conn);
        return 0;
    }

    $sql = "REPLACE INTO media_ids_idx(post_id, prefix, value, title, year, imdb_id, media_type) VALUES "
        . implode(', ', $rows);
    $ok = @mysqli_query($conn, $sql);
    @mysqli_close($conn);

    return $ok ? count($rows) : 0;
}

/**
 * Delete all `media_ids_idx` rows for a post id.
 */
function fd_media_ids_delete(int $postId): void
{
    if ($postId <= 0) {
        return;
    }
    $conn = fd_manticore_connect();
    if ($conn === null) {
        return;
    }
    @mysqli_query($conn, "DELETE FROM media_ids_idx WHERE post_id = {$postId}");
    @mysqli_close($conn);
}

function fd_resolve_external_media_metadata(string $itemId, string $itemType = 'movie'): array
{
    $title = '';
    $year = '';
    $season = null;
    $episode = null;
    $imdbId = '';

    $rawId = trim($itemId);
    if ($rawId === '') {
        return ['title' => '', 'year' => '', 'season' => null, 'episode' => null, 'imdb_id' => ''];
    }

    // 1. IMDb format: tt1234567 or tt1234567:1:1
    if (preg_match('/^(tt\d{6,10})(?::(\d+):(\d+))?$/i', $rawId, $m)) {
        $imdbId = $m[1];
        $season = isset($m[2]) ? (int)$m[2] : null;
        $episode = isset($m[3]) ? (int)$m[3] : null;

        $cacheFile = fd_storage_path('storage/cinemeta_' . md5($imdbId) . '.json');
        if (is_file($cacheFile) && (time() - (int)filemtime($cacheFile)) < 86400) {
            $cData = json_decode((string)@file_get_contents($cacheFile), true);
            if (is_array($cData) && !empty($cData['name'])) {
                $title = (string) ($cData['name'] ?? '');
                $year = (string) ($cData['year'] ?? '');
            }
        }

        if ($title === '' || ($year === '' && ($season !== null || $itemType === 'series'))) {
            $cinemetaType = ($season !== null || $itemType === 'series') ? 'series' : 'movie';
            $cinemetaUrl = "https://v3-cinemeta.strem.io/meta/{$cinemetaType}/{$imdbId}.json";
            $cinemetaJson = fd_http_json($cinemetaUrl, [], 'GET', 5);
            if (!empty($cinemetaJson['meta']['name'])) {
                $title = (string) $cinemetaJson['meta']['name'];
                $relInfo = (string) ($cinemetaJson['meta']['releaseInfo'] ?? ($cinemetaJson['meta']['year'] ?? ''));
                if (preg_match('/\b(19\d\d|20\d\d)\b/', $relInfo, $ym)) {
                    $year = $ym[1];
                } elseif (!empty($cinemetaJson['meta']['videos'][0]['released']) && preg_match('/\b(19\d\d|20\d\d)\b/', (string)$cinemetaJson['meta']['videos'][0]['released'], $ym)) {
                    $year = $ym[1];
                }
                @file_put_contents($cacheFile, json_encode(['name' => $title, 'year' => $year]), LOCK_EX);
            }
        }

        if ($title !== '') {
            // Auto-populate media_ids_idx for IMDb IDs so they are independently stored
            $numericImdb = (int) preg_replace('/\D/', '', $imdbId);
            $pseudoId = 9000000000 + ($numericImdb % 1000000000);
            fd_media_ids_upsert($pseudoId, ['imdb' => $imdbId, 'tt' => $imdbId], $title, (int)$year, $imdbId, ($season !== null || $itemType === 'series') ? 'tvseries' : 'movie');
        }

        return [
            'title' => $title,
            'year' => $year,
            'season' => $season,
            'episode' => $episode,
            'imdb_id' => $imdbId,
        ];
    }

    // Check if user has configured custom upstream manifests in catalog settings
    $catSettings = fd_load_catalog_settings();
    $configuredUpstreams = (array) ($catSettings['upstream_manifests'] ?? []);

    // 1b. Local catalog lookup for ANY prefixed ID (tmdb:, kitsu:, mal:, anilist:,
    // tvdb:, anidb:, or a custom prefix from a user's manifest.json).
    // This lets future users resolve these IDs WITHOUT configuring upstream
    // Stremio addons, because the scraper already stored the ID + title in the
    // generic `_media_ids` postmeta map / `media_ids_idx`.
    if (preg_match('/^([a-zA-Z0-9_-]+):([^:]+)(?::(\d+):(\d+))?$/', $rawId, $pm)) {
        $prefix = strtolower($pm[1]);
        $value = (string) $pm[2];
        $season = isset($pm[3]) ? (int) $pm[3] : null;
        $episode = isset($pm[4]) ? (int) $pm[4] : null;

        $local = fd_lookup_local_catalog_by_prefix($prefix, $value);
        if (!empty($local['title'])) {
            return [
                'title' => (string) $local['title'],
                'year' => (string) ($local['year'] ?? ''),
                'season' => $season,
                'episode' => $episode,
                'imdb_id' => (string) ($local['imdb_id'] ?? ''),
            ];
        }
    }

    // 2. TMDB format: tmdb:1108427 or tmdb:1399:1:1
    if (preg_match('/^tmdb:(\d+)(?::(\d+):(\d+))?$/i', $rawId, $m)) {
        $tmdbNumeric = $m[1];
        $season = isset($m[2]) ? (int)$m[2] : null;
        $episode = isset($m[3]) ? (int)$m[3] : null;
        $resolvedType = ($season !== null || $itemType === 'series') ? 'series' : 'movie';

        $cacheFile = fd_storage_path('storage/cinemeta_' . md5('tmdb:' . $tmdbNumeric) . '.json');
        if (is_file($cacheFile) && (time() - (int)filemtime($cacheFile)) < 86400) {
            $cData = json_decode((string)@file_get_contents($cacheFile), true);
            if (is_array($cData) && !empty($cData['name'])) {
                $title = (string) ($cData['name'] ?? '');
                $year = (string) ($cData['year'] ?? '');
                $imdbId = (string) ($cData['imdb_id'] ?? '');
            }
        }

        if ($title === '') {
            // Check configured upstream addons that declare 'meta' support
            foreach ($configuredUpstreams as $upstream) {
                $manifestUrl = trim((string)($upstream['url'] ?? ''));
                if ($manifestUrl === '') continue;
                $baseAddonUrl = preg_replace('#/manifest\.json(\?.*)?$#i', '', $manifestUrl);
                $metaUrl = rtrim($baseAddonUrl, '/') . "/meta/{$resolvedType}/tmdb:{$tmdbNumeric}.json";
                $res = fd_http_json($metaUrl, [], 'GET', 4);
                if (!empty($res['meta']['name'])) {
                    $title = (string) $res['meta']['name'];
                    $year = (string) ($res['meta']['year'] ?? ($res['meta']['releaseInfo'] ?? ''));
                    if (preg_match('/\b(19\d\d|20\d\d)\b/', $year, $ym)) {
                        $year = $ym[1];
                    }
                    $imdbId = (string) ($res['meta']['imdb_id'] ?? '');
                    break;
                }
            }

            if ($title !== '') {
                @file_put_contents($cacheFile, json_encode(['name' => $title, 'year' => $year, 'imdb_id' => $imdbId]), LOCK_EX);
                // Make media_ids_idx independent by populating on-the-fly from upstream
                $mediaIdsToSave = ['tmdb' => $tmdbNumeric];
                if (!empty($imdbId)) {
                    $mediaIdsToSave['imdb'] = $imdbId;
                }
                $pseudoId = 8000000000 + (int)$tmdbNumeric;
                fd_media_ids_upsert($pseudoId, $mediaIdsToSave, $title, (int)$year, $imdbId, $resolvedType === 'series' ? 'tvseries' : 'movie');
            }
        }

        return [
            'title' => $title,
            'year' => $year,
            'season' => $season,
            'episode' => $episode,
            'imdb_id' => $imdbId,
        ];
    }

    // 3. Kitsu anime format: kitsu:1 or kitsu:1:1
    if (preg_match('/^kitsu:(\d+)(?::(\d+))?$/i', $rawId, $m)) {
        $kitsuId = $m[1];
        $episode = isset($m[2]) ? (int)$m[2] : null;
        $season = 1;

        $cacheFile = fd_storage_path('storage/cinemeta_' . md5('kitsu:' . $kitsuId) . '.json');
        if (is_file($cacheFile) && (time() - (int)filemtime($cacheFile)) < 86400) {
            $cData = json_decode((string)@file_get_contents($cacheFile), true);
            if (is_array($cData) && !empty($cData['name'])) {
                $title = (string) ($cData['name'] ?? '');
                $year = (string) ($cData['year'] ?? '');
                $imdbId = (string) ($cData['imdb_id'] ?? '');
            }
        }

        if ($title === '') {
            $kitsuUrl = "https://anime-kitsu.strem.fun/meta/anime/kitsu:{$kitsuId}.json";
            $res = fd_http_json($kitsuUrl, [], 'GET', 5);
            if (!empty($res['meta']['name'])) {
                $title = (string) $res['meta']['name'];
                $year = (string) ($res['meta']['year'] ?? ($res['meta']['releaseInfo'] ?? ''));
                if (preg_match('/\b(19\d\d|20\d\d)\b/', $year, $ym)) {
                    $year = $ym[1];
                }
                $imdbId = (string) ($res['meta']['imdb_id'] ?? '');
                @file_put_contents($cacheFile, json_encode(['name' => $title, 'year' => $year, 'imdb_id' => $imdbId]), LOCK_EX);
                // Make media_ids_idx independent by populating on-the-fly from upstream
                $mediaIdsToSave = ['kitsu' => $kitsuId];
                if (!empty($imdbId)) {
                    $mediaIdsToSave['imdb'] = $imdbId;
                }
                $pseudoId = 7000000000 + (int)$kitsuId;
                fd_media_ids_upsert($pseudoId, $mediaIdsToSave, $title, (int)$year, $imdbId, 'tvseries');
            }
        }

        return [
            'title' => $title,
            'year' => $year,
            'season' => $season,
            'episode' => $episode,
            'imdb_id' => $imdbId,
        ];
    }

    // 4. Other prefixed IDs (e.g. mal:..., anilist:..., tvdb:...)
    if (preg_match('/^([a-zA-Z0-9_-]+):(\d+)(?::(\d+):?(\d+)?)?$/i', $rawId, $m)) {
        $prefix = strtolower($m[1]);
        $val = $m[2];
        $season = isset($m[3]) ? (int)$m[3] : null;
        $episode = isset($m[4]) ? (int)$m[4] : null;
        $resolvedType = ($season !== null || $itemType === 'series') ? 'series' : 'movie';

        // Check configured upstream addons that declare 'meta' support
        foreach ($configuredUpstreams as $upstream) {
            $manifestUrl = trim((string)($upstream['url'] ?? ''));
            if ($manifestUrl === '') continue;
            $baseAddonUrl = preg_replace('#/manifest\.json(\?.*)?$#i', '', $manifestUrl);
            $metaUrl = rtrim($baseAddonUrl, '/') . "/meta/{$resolvedType}/{$prefix}:{$val}.json";
            $res = fd_http_json($metaUrl, [], 'GET', 4);
            if (!empty($res['meta']['name'])) {
                $title = (string) $res['meta']['name'];
                $year = (string) ($res['meta']['year'] ?? ($res['meta']['releaseInfo'] ?? ''));
                if (preg_match('/\b(19\d\d|20\d\d)\b/', $year, $ym)) {
                    $year = $ym[1];
                }
                $imdbId = (string) ($res['meta']['imdb_id'] ?? '');
                break;
            }
        }

        if ($title !== '') {
            // Make media_ids_idx independent by populating on-the-fly from upstream
            $mediaIdsToSave = [$prefix => $val];
            if (!empty($imdbId)) {
                $mediaIdsToSave['imdb'] = $imdbId;
            }
            $pseudoId = (int) sprintf('%u', crc32($prefix . ':' . $val));
            fd_media_ids_upsert($pseudoId, $mediaIdsToSave, $title, (int)$year, $imdbId, $resolvedType === 'series' ? 'tvseries' : 'movie');
        }

        return [
            'title' => $title,
            'year' => $year,
            'season' => $season,
            'episode' => $episode,
            'imdb_id' => $imdbId,
        ];
    }

    return ['title' => '', 'year' => '', 'season' => null, 'episode' => null, 'imdb_id' => ''];
}


function fd_resolve_shortcode_cached(string $shortCode, string $botId): ?array
{
    static $memoryCache = [];
    $cacheKey = $shortCode . ':' . $botId;
    if (isset($memoryCache[$cacheKey])) {
        return $memoryCache[$cacheKey];
    }

    $diskCacheFile = fd_cache_path('resolve_cache_' . md5($cacheKey) . '.json');
    if (is_file($diskCacheFile)) {
        $mtime = (int) filemtime($diskCacheFile);
        // Expire file_id_mt after 2 hours because Telegram file_reference tokens expire
        if ((time() - $mtime) < 7200) {
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
    $diskCacheFile = fd_cache_path('resolve_cache_' . md5($cacheKey) . '.json');
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
            CURLOPT_RESOLVE => fd_curl_resolve_entries(),
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

    // If all bots returned 404 on the initial race, WordPress has just triggered a background
    // Telegram forward/copy relay. Telegram takes 1.5s - 4.5s to finish relaying and populate tg_file_id.
    // Poll up to 4 passes (1.2s, 1.5s, 1.5s, 1.5s) so the stream or player plays directly on first click.
    $all404 = !empty($botStatuses) && count(array_filter($botStatuses, fn($s) => ($s['http'] ?? 0) === 404)) === count($botStatuses);
    if ($all404) {
        $delays = [1200000, 1500000, 1500000, 1500000]; // in microseconds (total ~5.7s max)

        foreach ($delays as $retryIdx => $sleepUs) {
            usleep($sleepUs);

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
                    CURLOPT_RESOLVE => fd_curl_resolve_entries(),
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
                fd_log('resolve_shortcode_concurrent succeeded on relay retry pass ' . ($retryIdx + 1), [
                    'short_code' => $shortCode,
                    'winner_bot' => $winnerBotId,
                ]);
                // Look up or attach IMDb ID if missing so the web player can load OpenSubtitles automatically
                if (empty($winner['imdb_id']) && !empty($winner['title'])) {
                    $winner['imdb_id'] = fd_find_imdb_id_for_title((string) $winner['title']);
                }
                return $winner;
            }
        }
    }

    fd_log('resolve_shortcode_concurrent failed across all bots', [
        'short_code' => $shortCode,
        'bot_statuses' => $botStatuses,
        'last_error' => $lastErrorResult,
    ]);

    return is_array($lastErrorResult) ? $lastErrorResult : ['ok' => 0, 'message' => 'Failed to resolve short code across all bots.'];
}

function fd_resolve_shortcode(string $shortCode, string $botId = '', bool $bypassCache = false): array
{
    if ($botId === '') {
        $botId = fd_get_bot_id();
        if ($botId === '') {
            return fd_resolve_shortcode_concurrent($shortCode);
        }
    }

    if (!$bypassCache) {
        $cached = fd_resolve_shortcode_cached($shortCode, $botId);
        if ($cached !== null) {
            if (empty($cached['imdb_id']) && !empty($cached['title'])) {
                $cached['imdb_id'] = fd_find_imdb_id_for_title((string) $cached['title']);
            }
            return $cached;
        }
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
    if ($bypassCache) {
        $params['force'] = 1;
        $params['nocache'] = 1;
    }

    $res = fd_http_json($url, $params, 'GET', 6);
    if (!empty($res['file_id_mt']) || !empty($res['file_id'])) {
        if (empty($res['imdb_id']) && !empty($res['title'])) {
            $res['imdb_id'] = fd_find_imdb_id_for_title((string) $res['title']);
        }
        fd_save_resolve_cache($shortCode, $botId, $res);
        return $res;
    }

    // If 404, Telegram background relay was just triggered; poll up to 3 times (1.2s, 1.5s, 1.5s)
    $delays = [1200000, 1500000, 1500000];
    foreach ($delays as $delayUs) {
        usleep($delayUs);
        $res = fd_http_json($url, $params, 'GET', 6);
        if (!empty($res['file_id_mt']) || !empty($res['file_id'])) {
            if (empty($res['imdb_id']) && !empty($res['title'])) {
                $res['imdb_id'] = fd_find_imdb_id_for_title((string) $res['title']);
            }
            fd_save_resolve_cache($shortCode, $botId, $res);
            return $res;
        }
    }

    return $res;
}

/**
 * Resolve multiple short_codes at once using the batch endpoint with fallback to single concurrent resolution.
 * Automatically saves all resolved file_id_mt records into local resolve_cache.
 *
 * @param array $shortCodes List of short code strings
 * @param string $botId Optional specific bot ID (defaults to active pool bot)
 * @return array Map of [shortCode => resolvedData]
 */
/**
 * Fire-and-forget asynchronous HTTP request to hit and trigger warmup without blocking.
 * Opens a non-blocking TCP socket, sends the HTTP request, and closes immediately.
 */
function fd_http_fire_and_forget(string $url, array $payload = [], string $method = 'POST'): void
{
    $parts = parse_url($url);
    if (!$parts || empty($parts['host'])) {
        return;
    }

    $scheme = strtolower($parts['scheme'] ?? 'http');
    $isSsl = ($scheme === 'https');
    $port = $parts['port'] ?? ($isSsl ? 443 : 80);
    $host = $parts['host'];
    $path = ($parts['path'] ?? '/') . (!empty($parts['query']) ? '?' . $parts['query'] : '');

    $secret = fd_get_api_secret();
    $body = !empty($payload) ? http_build_query($payload) : '';
    if ($method === 'GET' && !empty($body)) {
        $path .= (str_contains($path, '?') ? '&' : '?') . $body;
        $body = '';
    }

    $req = "{$method} {$path} HTTP/1.1\r\n";
    $req .= "Host: {$host}\r\n";
    $req .= "User-Agent: pencarimovie-server/" . FD_APP_VERSION . "\r\n";
    $req .= "Accept: application/json\r\n";
    if ($secret !== '') {
        $req .= "X-API-Secret: {$secret}\r\n";
    }
    if ($method === 'POST') {
        $req .= "Content-Type: application/x-www-form-urlencoded\r\n";
        $req .= "Content-Length: " . strlen($body) . "\r\n";
    }
    $req .= "Connection: Close\r\n\r\n";
    if ($body !== '') {
        $req .= $body;
    }

    $target = ($isSsl ? 'ssl://' : 'tcp://') . $host . ':' . $port;
    $ctx = stream_context_create([
        'ssl' => [
            'verify_peer' => false,
            'verify_peer_name' => false,
        ],
    ]);

    $errno = 0;
    $errstr = '';
    $fp = @stream_socket_client($target, $errno, $errstr, 2, STREAM_CLIENT_CONNECT, $ctx);
    if ($fp) {
        stream_set_blocking($fp, false);
        @fwrite($fp, $req);
        @fclose($fp);
    }
}

function fd_resolve_shortcodes_batch(array $shortCodes, string $botId = ''): array
{
    $shortCodes = array_values(array_unique(array_filter(array_map('trim', $shortCodes))));
    if (empty($shortCodes)) {
        return [];
    }

    if ($botId === '') {
        $picked = fd_pick_pool_bot();
        $botId = !empty($picked['bot_id']) ? (string) $picked['bot_id'] : fd_get_bot_id();
    }

    $results = [];
    $missingCodes = [];

    // Check fast local cache first (0ms)
    foreach ($shortCodes as $sc) {
        $cached = fd_resolve_shortcode_cached($sc, $botId);
        if ($cached !== null) {
            $results[$sc] = $cached;
        } else {
            $missingCodes[] = $sc;
        }
    }

    if (empty($missingCodes)) {
        return $results;
    }

    // Trigger non-blocking fire-and-forget warmup on WordPress in chunks of 40
    $chunks = array_chunk($missingCodes, 40);
    foreach ($chunks as $chunk) {
        $url = FD_WP_API_BASE . '/resolve-files';
        fd_http_fire_and_forget($url, [
            'short_codes' => implode(',', $chunk),
            'bot_id' => $botId,
        ], 'POST');
    }

    return $results;
}

/**
 * Batch warm-up / repopulate the local resolve cache.
 *
 * Unlike fd_resolve_shortcodes_batch() (cache-only + fire-and-forget), this
 * actually calls WordPress /resolve-files, waits for the response, and writes
 * every resolved file_id_mt into the local resolve cache so subsequent
 * playback is instant. Codes the batch endpoint misses are retried
 * individually via fd_resolve_shortcode().
 *
 * @param array  $shortCodes List of short codes to warm up
 * @param string $botId      Bot to resolve against (defaults to active pool bot)
 * @param int    $chunkSize  Codes per batch request (WordPress caps at ~40)
 * @return array{ok:int,bot_id:string,total:int,resolved:int,failed:int,results:array,failed_codes:array}
 */
function fd_warmup_resolve_batch(array $shortCodes, string $botId = '', int $chunkSize = 40): array
{
    $shortCodes = array_values(array_unique(array_filter(array_map('trim', $shortCodes))));
    if (empty($shortCodes)) {
        return ['ok' => 0, 'message' => 'No short_codes provided.', 'total' => 0, 'resolved' => 0, 'failed' => 0, 'results' => [], 'failed_codes' => []];
    }

    if ($botId === '') {
        $picked = fd_pick_pool_bot();
        $botId = !empty($picked['bot_id']) ? (string) $picked['bot_id'] : fd_get_bot_id();
    }

    $chunkSize = max(1, min(40, $chunkSize));
    $results = [];
    $failedCodes = [];

    // 1. Serve anything already cached without a network round-trip.
    $missingCodes = [];
    foreach ($shortCodes as $sc) {
        $cached = fd_resolve_shortcode_cached($sc, $botId);
        if ($cached !== null) {
            $results[$sc] = $cached;
        } else {
            $missingCodes[] = $sc;
        }
    }

    // 2. Batch-resolve the misses against WordPress and persist the results.
    if (!empty($missingCodes)) {
        $chunks = array_chunk($missingCodes, $chunkSize);
        foreach ($chunks as $chunk) {
            $url = FD_WP_API_BASE . '/resolve-files';
            $res = fd_http_json($url, [
                'short_codes' => implode(',', $chunk),
                'bot_id' => $botId,
            ], 'POST', 30);

            // WordPress may return {results:{code:{...}}} or {data:{results:{...}}}
            $map = [];
            if (is_array($res)) {
                if (isset($res['results']) && is_array($res['results'])) {
                    $map = $res['results'];
                } elseif (isset($res['data']['results']) && is_array($res['data']['results'])) {
                    $map = $res['data']['results'];
                } elseif (isset($res['files']) && is_array($res['files'])) {
                    foreach ($res['files'] as $row) {
                        $code = (string) ($row['short_code'] ?? '');
                        if ($code !== '') {
                            $map[$code] = $row;
                        }
                    }
                }
            }

            foreach ($chunk as $sc) {
                $row = $map[$sc] ?? null;
                if (is_array($row) && (!empty($row['file_id_mt']) || !empty($row['file_id']))) {
                    if (empty($row['bot_id'])) {
                        $row['bot_id'] = $botId;
                    }
                    fd_save_resolve_cache($sc, $botId, $row);
                    $results[$sc] = $row;
                } else {
                    $failedCodes[] = $sc;
                }
            }
        }
    }

    // 3. Retry the batch misses individually (the batch endpoint can skip codes
    //    whose Telegram relay was not yet ready). Run these in parallel via
    //    curl_multi so a large miss set does not serialize into minutes.
    $stillFailed = [];
    if (!empty($failedCodes)) {
        $retryChunks = array_chunk($failedCodes, 12);
        foreach ($retryChunks as $retryChunk) {
            $multi = curl_multi_init();
            $handles = [];
            $secret = fd_get_api_secret();
            $headers = [
                'Accept: application/json',
                'User-Agent: pencarimovie-server/' . FD_APP_VERSION,
                'X-App-Version: ' . FD_APP_VERSION,
            ];
            if ($secret !== '') {
                $headers[] = 'X-API-Secret: ' . $secret;
            }
            foreach ($retryChunk as $sc) {
                $targetUrl = FD_WP_API_BASE . '/resolve-file?' . http_build_query([
                    'short_code' => $sc,
                    'bot_id' => $botId,
                    'force' => 1,
                    'nocache' => 1,
                ]);
                $ch = curl_init($targetUrl);
                curl_setopt_array($ch, [
                    CURLOPT_RETURNTRANSFER => true,
                    CURLOPT_TIMEOUT => 15,
                    CURLOPT_CONNECTTIMEOUT => 5,
                    CURLOPT_HTTPHEADER => $headers,
                    CURLOPT_SSL_VERIFYPEER => false,
                    CURLOPT_SSL_VERIFYHOST => 2,
                    CURLOPT_RESOLVE => fd_curl_resolve_entries(),
                ]);
                curl_multi_add_handle($multi, $ch);
                $handles[$sc] = $ch;
            }

            $running = null;
            do {
                curl_multi_exec($multi, $running);
                if ($running > 0) {
                    curl_multi_select($multi, 0.5);
                }
            } while ($running > 0);

            foreach ($handles as $sc => $ch) {
                $body = (string) curl_multi_getcontent($ch);
                curl_multi_remove_handle($multi, $ch);
                curl_close($ch);
                $row = json_decode($body, true);
                if (is_array($row) && (!empty($row['file_id_mt']) || !empty($row['file_id']))) {
                    if (empty($row['bot_id'])) {
                        $row['bot_id'] = $botId;
                    }
                    fd_save_resolve_cache($sc, $botId, $row);
                    $results[$sc] = $row;
                } else {
                    $stillFailed[] = $sc;
                }
            }
            curl_multi_close($multi);
        }
    }

    fd_log('warmup resolve batch completed', [
        'bot_id' => $botId,
        'total' => count($shortCodes),
        'resolved' => count($results),
        'failed' => count($stillFailed),
    ]);

    return [
        'ok' => 1,
        'bot_id' => $botId,
        'total' => count($shortCodes),
        'resolved' => count($results),
        'failed' => count($stillFailed),
        'results' => $results,
        'failed_codes' => $stillFailed,
    ];
}

/**
 * Prewarm / resolve all streams in the background (fire-and-forget) before playback/download.
 * Never blocks stream list delivery or user navigation.
 *
 * @param array $files List of media file rows from search_files or post_files
 * @param string $botId Destination bot ID
 * @return array Enriched files array with cached file_id_mt populated where already warm
 */
function fd_prewarm_streams_batch(array $files, string $botId = ''): array
{
    if (empty($files)) {
        return [];
    }

    $shortCodes = [];
    foreach ($files as $f) {
        $sc = $f['short_code'] ?? '';
        if ($sc !== '') {
            $shortCodes[] = $sc;
        }
    }

    if (empty($shortCodes)) {
        return $files;
    }

    // Hit-and-forget: checks local cache for instant hits, fires async background warmup for misses
    $resolvedMap = fd_resolve_shortcodes_batch($shortCodes, $botId);

    // Merge resolved data into the file list if already in cache
    foreach ($files as &$f) {
        $sc = $f['short_code'] ?? '';
        if ($sc !== '' && isset($resolvedMap[$sc])) {
            $r = $resolvedMap[$sc];
            if (!empty($r['file_id_mt'])) {
                $f['file_id_mt'] = $r['file_id_mt'];
            }
            if (!empty($r['file_id'])) {
                $f['file_id'] = $r['file_id'];
            }
            if (empty($f['file_size']) && !empty($r['file_size'])) {
                $f['file_size'] = (int) $r['file_size'];
            }
            if (!empty($r['bot_id'])) {
                $f['bot_id'] = (string) $r['bot_id'];
            }
        }
    }
    unset($f);

    return $files;
}

/**
 * Helper to look up an IMDb ID for a media title via Cinemeta
 * so OpenSubtitles can be fetched in the web player even when files don't have an explicit IMDb ID.
 */
/**
 * Retrieve subtitles for a media item from OpenSubtitles and upstream manifests,
 * with caching, language normalization, and automatic ID resolution.
 */
function fd_get_item_subtitles(string $itemType, string $itemId): array
{
    // Internal pm:post: IDs cannot be resolved by external subtitle providers (OpenSubtitles)
    // Avoid slow Cinemeta fallback queries on internal synthetic IDs
    if (str_starts_with($itemId, 'pm:post:') || str_starts_with($itemId, 'pm_post_') || str_starts_with($itemId, 'pm:file:') || str_starts_with($itemId, 'pm_file_')) {
        return [];
    }

    // If not starting with 'tt' (e.g. searching with title text or short code), resolve to IMDb ID
    if (!preg_match('/^tt\d{6,10}/i', $itemId)) {
        $resolvedImdb = fd_find_imdb_id_for_title($itemId);
        if ($resolvedImdb !== '') {
            $itemId = $resolvedImdb;
        } else {
            return [];
        }
    }

    // Check local disk cache (30 min TTL)
    $subCacheFile = fd_storage_path('storage/sub_cache_' . md5($itemType . '_' . $itemId) . '.json');
    if (is_file($subCacheFile) && (time() - (int)filemtime($subCacheFile)) < 1800) {
        $cachedSubs = json_decode((string)@file_get_contents($subCacheFile), true);
        if (is_array($cachedSubs) && !empty($cachedSubs['subtitles'])) {
            return $cachedSubs['subtitles'];
        }
    }

    $subtitles = [];
    $seenSubIds = [];

    // Build list of subtitle endpoints: OpenSubtitles v3 & OpenSubtitlesv3 PRO
    $proCfg = base64_encode(json_encode([
        'langs' => ['en', 'id', 'ms', 'ar', 'es', 'zh', 'ja', 'ko', 'fr', 'de'],
        'source' => 'all',
        'aiTranslated' => true,
        'autoAdjustment' => false,
    ]));

    $subEndpoints = [
        "https://opensubtitles-v3.strem.io/subtitles/{$itemType}/" . urlencode($itemId) . ".json",
    ];

    $catSettings = fd_load_catalog_settings();
    $configuredUpstreams = (array) ($catSettings['upstream_manifests'] ?? []);
    foreach ($configuredUpstreams as $upstream) {
        $manifestUrl = trim((string)($upstream['url'] ?? ''));
        if ($manifestUrl === '') continue;
        $baseAddonUrl = preg_replace('#/manifest\.json(\?.*)?$#i', '', $manifestUrl);
        $subEndpoints[] = rtrim($baseAddonUrl, '/') . "/subtitles/{$itemType}/" . urlencode($itemId) . ".json";
    }

    foreach (array_unique($subEndpoints) as $upstreamSubUrl) {
        $subRes = fd_http_json($upstreamSubUrl, [], 'GET', 5);
        if (!empty($subRes['subtitles']) && is_array($subRes['subtitles'])) {
            foreach ($subRes['subtitles'] as $sub) {
                if (is_array($sub) && !empty($sub['url'])) {
                    $sId = (string)($sub['id'] ?? $sub['url']);
                    if (!isset($seenSubIds[$sId])) {
                        $seenSubIds[$sId] = true;
                        $subtitles[] = $sub;
                    }
                }
            }
        }
    }

    // Cache result for 30 minutes
    if (!empty($subCacheFile)) {
        @file_put_contents($subCacheFile, json_encode(['subtitles' => $subtitles], JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE), LOCK_EX);
    }

    return $subtitles;
}

function fd_find_imdb_id_for_title(string $rawTitle): string
{
    // If raw title is an internal pm ID, it is not a real search title
    if (str_starts_with($rawTitle, 'pm:post:') || str_starts_with($rawTitle, 'pm_post_') || str_starts_with($rawTitle, 'pm:file:') || str_starts_with($rawTitle, 'pm_file_')) {
        return '';
    }

    // Skip Cinemeta search for audio/music files (mp3, m4a, flac, etc.)
    if (preg_match('/\.(?:mp3|m4a|flac|wav|ogg|opus|aac)$/i', $rawTitle)) {
        return '';
    }

    // Clean spam / quality tags / extensions
    $clean = fd_clean_media_title($rawTitle);
    if ($clean === '') {
        $clean = $rawTitle;
    }

    // Extract year if present
    $year = '';
    if (preg_match('/\b(19\d\d|20\d\d)\b/', $clean, $ym)) {
        $year = $ym[1];
        $clean = trim(preg_replace('/\b(19\d\d|20\d\d)\b.*$/', '', $clean));
    }

    $clean = trim(preg_replace('/[._\-]/', ' ', $clean));
    if ($clean === '') {
        return '';
    }

    $cacheKey = 'title_imdb_' . md5(strtolower($clean . '_' . $year));
    $cacheFile = fd_storage_path('storage/' . $cacheKey . '.txt');
    if (is_file($cacheFile) && (time() - (int) @filemtime($cacheFile)) < 604800) {
        return trim((string) @file_get_contents($cacheFile));
    }

    // Try Cinemeta search (fast 2s timeout)
    $searchQuery = urlencode($clean);
    $url = "https://v3-cinemeta.strem.io/catalog/movie/top/search={$searchQuery}.json";
    $json = fd_http_json($url, [], 'GET', 2);
    $imdbId = '';

    if (!empty($json['metas'][0]['id']) && str_starts_with($json['metas'][0]['id'], 'tt')) {
        $imdbId = (string) $json['metas'][0]['id'];
    } else {
        // Try series search (fast 2s timeout)
        $urlSeries = "https://v3-cinemeta.strem.io/catalog/series/top/search={$searchQuery}.json";
        $jsonSeries = fd_http_json($urlSeries, [], 'GET', 2);
        if (!empty($jsonSeries['metas'][0]['id']) && str_starts_with($jsonSeries['metas'][0]['id'], 'tt')) {
            $imdbId = (string) $jsonSeries['metas'][0]['id'];
        }
    }

    if ($imdbId !== '') {
        @file_put_contents($cacheFile, $imdbId, LOCK_EX);
    }
    return $imdbId;
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
            fd_log('pencarimovie raw response body: false');
            return [null, 'PencariMovie connection failed (HTTP request failed).'];
        }

        fd_log('pencarimovie raw response body', ['body' => $body]);

        // Strip UTF-8 Byte Order Mark (BOM) if present at the start of the response
        if (str_starts_with($body, "\xEF\xBB\xBF")) {
            $body = substr($body, 3);
        }
        $body = trim($body);

        $data = json_decode($body, true);
        if (!is_array($data) || empty($data['ok'])) {
            $msg = $data['message'] ?? 'PencariMovie server rejected the bot token.';
            fd_log('pencarimovie rejected token', ['message' => $msg, 'response_decoded' => $data]);
            return [null, $msg];
        }
        if (empty($data['encrypted_credentials']) || empty($data['encryption_iv'])) {
            return [null, 'PencariMovie response missing encrypted credentials.'];
        }

        // Save the API secret returned by PencariMovie for authenticating future requests
        if (!empty($data['api_secret'])) {
            fd_save_api_secret((string) $data['api_secret']);
            fd_log('api secret saved from pencarimovie');
        }

        return fd_decrypt_credentials(
            $data['encrypted_credentials'],
            $data['encryption_iv'],
            $botToken
        );
    } catch (\Throwable $e) {
        return [null, 'PencariMovie connection failed: ' . $e->getMessage()];
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
        return [null, 'No API credentials available. Login via the settings page to fetch from PencariMovie.'];
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
    if (is_dir($sessionPath) || is_file($sessionPath)) {
        $lockFile = rtrim($sessionPath, '/\\') . DIRECTORY_SEPARATOR . 'lock';
        if (!file_exists($lockFile)) {
            @touch($lockFile);
        }
        if ($botToken === null || $botToken === '') {
            fd_ensure_ipc_worker($sessionPath);
        }
    }

    $settings = new \danog\MadelineProto\Settings();
    $settings->getAppInfo()
        ->setApiId($apiId)
        ->setApiHash($apiHash);
    // Disable IPv6 and DoH on mobile/Android environments.
    // DoH (DNS-over-HTTPS) tries to connect to mozilla.cloudflare-dns.com at startup,
    // which hangs for 20+ seconds if DNS is cold or blocked on Android.
    // Disabling DoH and IPv6 forces direct IPv4 connections to Telegram DCs (149.154.*.*)
    // without any mozilla.cloudflare-dns.com lookups or flora/venus web subdomain hops.
    //
    // Obfuscated transport (MTProto Obfuscated2) is used as a FALLBACK, not by
    // default. Some ISP DPI middleboxes (reported: TIME Fibre Home, Malaysia)
    // fingerprint the plain MTProto handshake (first byte 0xef) and inject a
    // TCP RST on port 443. The symptom is a successful TCP connect followed
    // immediately by NothingInTheSocketException, then the auth key transition
    // to ENCRYPTED_NOT_BOUND is cancelled ("The operation was cancelled").
    //
    // MadelineProto does NOT auto-retry with obfuscation: DataCenter::getCtxs()
    // only inserts ObfuscatedStream when Connection::getObfuscated() is true,
    // and the reconnect loop reuses the same stream stack. So we implement the
    // fallback ourselves in the retry loop below: attempt 1 uses the plain
    // transport (fast path, no AES-CTR overhead), and if it fails with a
    // DPI-style error we flip $useObfuscated and retry.
    //
    // ObfuscatedStream is supported by every Telegram DC and is what official
    // Telegram clients use, so the fallback is safe (no session change,
    // negligible CPU cost via AES-NI). Verified: the stack becomes
    // AbridgedStream => ObfuscatedStream => BufferedRawStream => DefaultStream
    // and the full auth key transition completes.
    $useObfuscated = false;
    $settings->getConnection()
        ->setIpv6(false)
        ->setTimeout(15.0)
        ->setObfuscated($useObfuscated)
        ->setUseDoH(false);
    if (fd_is_debug_enabled()) {
        $settings->getLogger()->setLevel(\danog\MadelineProto\Logger::NOTICE);
    } else {
        $settings->getLogger()->setLevel(\danog\MadelineProto\Logger::FATAL_ERROR);
    }
    $settings->getLogger()->setMaxSize(FD_MAX_LOG_SIZE);

    // Increase RPC timeouts and parallel chunk tuning for Telegram file downloads
    // Default rpcDropTimeout is 60s; rpcResendTimeout defaults to 4s.
    // On high-latency mobile networks or media DCs (DC -2 / DC -4), 4s is too aggressive
    // and causes constant "Still missing upload.getFile ... sending state request" spam
    // while the socket is busy receiving previous chunks. Raising resend timeout to 12s
    // and drop timeout to 180s stops unnecessary state request stalls.
    $settings->getRpc()->setRpcDropTimeout(180);
    $settings->getRpc()->setRpcResendTimeout(12);
    // Reduce parallel download chunks from 20 to 8 on mobile/Android to prevent socket saturation
    $settings->getFiles()->setDownloadParallelChunks(8);

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
        // ── Obfuscated-transport fallback ─────────────────────────────────────
        // Attempt 1 uses the plain transport. If it fails with a DPI-style
        // error (socket reset right after connect, or a cancelled auth key
        // transition), flip to ObfuscatedStream and retry. This keeps the fast
        // plain path for the ~99% of users whose ISP does not fingerprint
        // MTProto, while still recovering automatically on TIME Fibre Home and
        // similar DPI networks.
        if ($bootAttempt > 0 && !$useObfuscated && fd_is_dpi_transport_error((string) $lastError)) {
            $useObfuscated = true;
            $settings->getConnection()->setObfuscated(true);
            fd_log('retrying with obfuscated transport after DPI-style failure', [
                'attempt' => $bootAttempt + 1,
                'previous_error' => $lastError,
            ]);
        }
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

            // Reset the full-boot flag immediately after construction.
            //
            // Ipc::getSlow() consults FD_FORCE_FULL_BOOT during construction to
            // decide between a full session deserialize and an IPC client
            // connect. Leaving it set for the rest of the request means any
            // later API construction in the same request also does a full boot,
            // and a full-mode instance SAVES THE SESSION on shutdown
            // (APIWrapper::serialize() returns early only for IPC clients).
            //
            // Each save takes an exclusive flock on safe.php.lock and
            // lightState.php.lock. Under concurrency those locks serialize
            // every request and can wedge the whole server. Clearing the flag
            // here keeps it scoped to the single fresh-login boot it was
            // intended for, so every other boot takes the IPC client path and
            // performs no session save.
            $GLOBALS['FD_FORCE_FULL_BOOT'] = false;

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
            // If session is busy or locked, clear only transient .lock files (NOT the entire session data directory)
            if (str_contains(strtolower($lastError), 'busy') || str_contains(strtolower($lastError), 'lock') || str_contains(strtolower($lastError), 'could not connect')) {
                fd_log('clearing session transient locks due to lock/busy error', ['target_bot_id' => $targetBotId]);
                if (is_dir($sessionPath)) {
                    foreach (['/lightState.php.lock', '/safe.php.lock', '/ipcState.php.lock', '/lock'] as $lockName) {
                        $lockPath = $sessionPath . $lockName;
                        if (is_file($lockPath)) {
                            @unlink($lockPath);
                        }
                    }
                }
            }
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
    $rawError = $lastError ?? 'Could not boot MadelineProto after 3 attempts.';
    $friendlyError = fd_friendly_login_error($rawError);
    return [null, $friendlyError];
}

/**
 * Decide whether a MadelineProto boot error looks like ISP DPI interference on
 * the plain MTProto transport, so the caller can retry with ObfuscatedStream.
 *
 * Reported signature (TIME Fibre Home, Malaysia): the TCP connect succeeds,
 * then the socket is reset before the auth key transition completes.
 *
 *   ReadLoop: danog\MadelineProto\NothingInTheSocketException
 *   Got exception in DC 2.0, reconnecting...
 *   SecurityException ... An error occurred while handling state transition
 *   to ENCRYPTED_NOT_BOUND in DC 2: The operation was cancelled
 *
 * Deliberately narrow: a generic "The operation was cancelled" alone is NOT
 * enough (it also happens on slow cold Diffie-Hellman key generation), so we
 * require a socket-level reset marker or the ENCRYPTED_NOT_BOUND transition.
 */
function fd_is_dpi_transport_error(string $error): bool
{
    if ($error === '') {
        return false;
    }
    $lower = strtolower($error);

    // A clock-skew error also surfaces as an ENCRYPTED_NOT_BOUND cancellation,
    // but obfuscation cannot fix it. Exclude it so we do not waste a retry.
    if (fd_is_clock_or_authkey_error($error)) {
        return false;
    }

    // Socket reset / immediate EOF right after connect.
    if (
        str_contains($lower, 'nothinginthesocketexception')
        || str_contains($lower, 'nothing in the socket')
        || str_contains($lower, 'connection reset by peer')
        || str_contains($lower, 'reset by peer')
        || str_contains($lower, 'broken pipe')
        || str_contains($lower, 'unexpected eof')
    ) {
        return true;
    }

    // Auth key transition cancelled while binding to the DC.
    if (
        str_contains($lower, 'encrypted_not_bound')
        || str_contains($lower, 'encrypted_not_inited')
    ) {
        return true;
    }

    return false;
}

/**
 * Decide whether a boot error is caused by clock skew or a broken auth-key
 * exchange. Provisioning a fresh guest bot cannot fix either, so the caller
 * must stop instead of looping.
 *
 * Observed chain: clock ~49s ahead -> MsgIdHandler nudges time_delta -> Telegram
 * replies bad_msg_notification "msg_id too high" -> ResponseHandler resets the
 * MTProto session mid-handshake -> SecurityException "wrong new_nonce_hash1".
 */
function fd_is_clock_or_authkey_error(string $error): bool
{
    if ($error === '') {
        return false;
    }
    $lower = strtolower($error);

    return str_contains($lower, 'clock')
        || str_contains($lower, 'sync your date')
        || str_contains($lower, 'too new compared')
        || str_contains($lower, 'too old compared')
        || str_contains($lower, 'msg_id too high')
        || str_contains($lower, 'message id')
        || str_contains($lower, 'new_nonce_hash')
        || str_contains($lower, 'time delta')
        || str_contains($lower, 'time_delta');
}

/**
 * Find a session directory that exists on disk but has no bot_id recorded in
 * session_meta.json / bot_pool.json. This happens when a previous provision
 * died mid-handshake, and it is what makes every page load provision yet
 * another guest bot.
 *
 * @return string The orphan bot id, or '' when none is found.
 */
function fd_find_orphan_session_bot_id(): string
{
    $sessionsDir = fd_storage_path('storage/sessions');
    if (!is_dir($sessionsDir)) {
        return '';
    }
    $entries = @scandir($sessionsDir);
    if (!$entries) {
        return '';
    }
    foreach ($entries as $entry) {
        if ($entry === '.' || $entry === '..') {
            continue;
        }
        $dir = $sessionsDir . DIRECTORY_SEPARATOR . $entry;
        if (!is_dir($dir)) {
            continue;
        }
        // A session dir counts as orphaned only if it has no usable session file.
        $sess = $dir . DIRECTORY_SEPARATOR . 'session.madeline';
        if (is_file($sess) || is_dir($sess)) {
            return '';
        }
        return (string) $entry;
    }
    return '';
}

/**
 * Measure the local clock offset against Telegram's server time.
 *
 * Telegram rejects MTProto handshakes when the client clock is off by more
 * than ~300 seconds ("message ID too new/old compared to the max/min value").
 * This probes a Telegram DC over plain HTTP (no auth, no session) and compares
 * the `Date` response header to local time, so we can tell the user the exact
 * number of seconds their clock is wrong instead of a generic message.
 *
 * @return array{ok: bool, offset: int, server_time: int, local_time: int, error: string}
 */
function fd_measure_clock_offset(): array
{
    $localTime = time();
    $serverTime = 0;

    // Telegram DCs answer plain HTTP on port 80 with a Date header.
    $hosts = ['149.154.167.51', '149.154.175.50', '149.154.167.91'];
    foreach ($hosts as $host) {
        $ctx = stream_context_create([
            'http' => [
                'method' => 'HEAD',
                'timeout' => 5,
                'ignore_errors' => true,
                'header' => "Host: telegram.org\r\nConnection: close\r\n",
            ],
            'ssl' => ['verify_peer' => false, 'verify_peer_name' => false],
        ]);
        $headers = @get_headers('http://' . $host . '/', true, $ctx);
        if (is_array($headers)) {
            // get_headers() returns a flat array for a single response.
            $dateHeader = '';
            foreach ($headers as $k => $v) {
                if (is_string($k) && strtolower($k) === 'date' && is_string($v)) {
                    $dateHeader = $v;
                    break;
                }
            }
            if ($dateHeader !== '') {
                $parsed = strtotime($dateHeader);
                if ($parsed !== false && $parsed > 0) {
                    $serverTime = $parsed;
                    break;
                }
            }
        }
    }

    if ($serverTime === 0) {
        return [
            'ok' => false,
            'offset' => 0,
            'server_time' => 0,
            'local_time' => $localTime,
            'error' => 'Could not read Telegram server time.',
        ];
    }

    return [
        'ok' => true,
        'offset' => $localTime - $serverTime,
        'server_time' => $serverTime,
        'local_time' => $localTime,
        'error' => '',
    ];
}

/**
 * Build a precise, actionable clock-skew message including the measured offset.
 */
function fd_clock_skew_message(): string
{
    $probe = fd_measure_clock_offset();
    $base = "Device clock out of sync!\nTelegram requires accurate device time (within ~5 minutes).";

    if (!$probe['ok']) {
        return $base . "\n\nPlease enable 'Set time automatically' (Automatic date and time / NTP) in your device Settings, then restart the server.";
    }

    $offset = (int) $probe['offset'];
    $abs = abs($offset);
    $direction = $offset > 0 ? 'ahead of' : 'behind';
    $minutes = floor($abs / 60);
    $seconds = $abs % 60;
    $human = $minutes > 0 ? "{$minutes}m {$seconds}s" : "{$seconds}s";

    return $base
        . "\n\nYour clock is {$human} {$direction} Telegram's server time."
        . "\n\nFix: enable 'Set time automatically' (Automatic date and time / NTP) in your device Settings, then restart the PencariMovie Server.";
}

/**
 * Translate internal MadelineProto/system boot errors into clear, actionable advice for users.
 */
function fd_friendly_login_error(string $rawError): string
{
    $lower = strtolower($rawError);

    // 1. Clock skew / NTP / message ID too new or old
    if (
        str_contains($lower, 'sync your date')
        || str_contains($lower, 'too new compared to the max value')
        || str_contains($lower, 'too old compared to the min value')
        || str_contains($lower, 'message id')
    ) {
        return fd_clock_skew_message();
    }

    // 2. DNS / Network connectivity / IP resolution issues
    if (
        str_contains($lower, 'could not resolve host')
        || str_contains($lower, 'name or service not known')
        || str_contains($lower, 'connection refused')
        || str_contains($lower, 'network is unreachable')
        || str_contains($lower, 'failed to connect')
    ) {
        return "Network connection failed!\nCannot reach Telegram or PencariMovie API servers. Check your internet connection or private DNS settings.";
    }

    // 3. Invalid bot token
    if (
        str_contains($lower, 'unauthorized')
        || str_contains($lower, 'bot_token_invalid')
        || str_contains($lower, 'token is invalid')
    ) {
        return "Invalid Bot Token!\nPlease double check the bot token from @BotFather. Ensure there are no extra spaces or missing characters.";
    }

    // 4. Session locked or busy
    if (
        str_contains($lower, 'busy')
        || str_contains($lower, 'lock')
        || str_contains($lower, 'could not connect to madelineproto')
    ) {
        return "Session busy or locked by another process.\nPlease wait a moment or restart the PencariMovie Server.";
    }

    // 5. Uncaught Revolt EventLoop exception
    if (preg_match('/Uncaught (.+?) thrown in event loop callback/i', $rawError, $m)) {
        // Strip out the noisy stack callback boilerplate to show the core problem
        $core = trim($m[1]);
        if (str_contains($lower, 'sync your date') || str_contains($lower, 'too new compared')) {
            return fd_clock_skew_message();
        }
        return "Login failed: " . $core;
    }

    return $rawError;
}

/**
 * Safely delete a session directory with Windows lock-handling and retries.
 */
function fd_clear_session_directory(string $sessionPath): void
{
    if (is_dir($sessionPath)) {
        // Terminate any background process locking this session path
        if (PHP_OS_FAMILY !== 'Windows') {
            @exec('pkill -9 -f ' . escapeshellarg($sessionPath) . ' 2>/dev/null');
        } else {
            // On Windows, kill any background PHP IPC worker process holding locks on this session path
            // Query running processes using PowerShell to find processes running entry.php or matching session path
            $sessBase = basename(rtrim($sessionPath, '/\\'));
            $psKill = 'powershell -NoProfile -Command "Get-CimInstance Win32_Process | Where-Object { ($_.Name -eq \'php.exe\') -and ($_.CommandLine -like \'*' . addcslashes($sessBase, "'\"") . '*\') } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }" >nul 2>nul';
            @exec($psKill);
            usleep(150000); // 150ms for OS to release file handles
        }
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
        // Delete the exact bot session directory as well as any parent folder under storage/sessions/
        $botDir = fd_storage_path('storage/sessions/' . preg_replace('/[^a-zA-Z0-9_\-]/', '', $botId));
        if (is_dir($botDir)) {
            fd_clear_session_directory($botDir);
        }
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

    // Clear resolve cache files and stream cache files
    fd_prune_cache_files(true);
    $cacheDir = FD_CACHE_DIR;
    if (is_dir($cacheDir)) {
        $allCacheFiles = glob($cacheDir . '/*.json');
        if ($allCacheFiles) {
            foreach ($allCacheFiles as $cf) {
                @unlink($cf);
            }
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
    $cacheFile = fd_storage_path('storage/version_cache.json');

    // 1. If in-memory state is empty, try loading disk cache first (persists across worker processes)
    if (empty($fd_version_state['min_version']) || !isset($fd_version_state['sponsor_url'])) {
        if (is_file($cacheFile)) {
            $cached = @json_decode((string) @file_get_contents($cacheFile), true);
            if (is_array($cached) && !empty($cached['time']) && (time() - $cached['time'] < 3600)) {
                $fd_version_state = $cached['data'] ?? [];
            }
        }
    }

    // 2. If still empty, fetch once with a quick timeout and write to disk cache
    if (empty($fd_version_state['min_version']) || !isset($fd_version_state['sponsor_url'])) {
        $response = fd_http_json(FD_WP_VERSION_URL . '?t=' . time(), [], 'GET', 4);
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
            $result = [
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
            @file_put_contents($cacheFile, json_encode([
                'time' => time(),
                'data' => $fd_version_state,
                'result' => $result,
            ], JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES), LOCK_EX);
            return $result;
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
 * Clean HTML entities and unclosed entity artifacts (e.g. &amp, &quot, &apos).
 */
function fd_clean_html_entities(string $text): string
{
    if ($text === '') return '';
    // 1. First standard HTML entity decode
    $text = html_entity_decode($text, ENT_QUOTES | ENT_HTML5, 'UTF-8');
    // 2. Fix unclosed/malformed entities like "&amp gincu", "&", "&quot foo", "&; "
    $text = preg_replace('/&amp(?:;|\b(?=[^\w;]|$))/i', '&', $text);
    $text = preg_replace('/&quot(?:;|\b(?=[^\w;]|$))/i', '"', $text);
    $text = preg_replace('/&apos(?:;|\b(?=[^\w;]|$))/i', "'", $text);
    $text = preg_replace('/&lt(?:;|\b(?=[^\w;]|$))/i', '<', $text);
    $text = preg_replace('/&gt(?:;|\b(?=[^\w;]|$))/i', '>', $text);
    $text = preg_replace('/&\s*;\s*/', '& ', $text);
    // Double decode in case of double escaping like &amp;
    $text = html_entity_decode($text, ENT_QUOTES | ENT_HTML5, 'UTF-8');
    return trim($text);
}

/**
 * Generate clean search query variants for Manticore full-text search.
 * Handles & vs dan vs and vs space, removes HTML entities and noise suffixes.
 */
function fd_build_search_query_variants(string $title, string $year = ''): array
{
    $variants = [];
    $title = trim($title);
    if ($title === '') return [];

    // 1. Decode entities
    $clean = fd_clean_html_entities($title);
    // Strip trailing suffixes like • Movie / • TvSeries
    $clean = preg_replace('/\s*[•··]\s*.+$/u', '', $clean);

    // Strip common release/quality/size noise from catalog titles (e.g. 1080p, HDTV, 2.3GB, NF, WEB-DL)
    $clean = preg_replace('/\b(?:2160p|1080p|720p|480p|360p|uhd|fhd|hd|sd|hdtv|web-?dl|webrip|bluray|blu-ray|remux|dvdrip|hevc|x264|x265|h264|h265|\d+(?:\.\d+)?\s*(?:gb|mb))\b/i', ' ', $clean);

    // Strip trailing year if already part of clean string
    if ($year !== '') {
        $clean = preg_replace('/\b' . preg_quote($year, '/') . '\b/', '', $clean);
    }
    $clean = trim(preg_replace('/\s+/', ' ', $clean));

    // Handle apostrophe-s and possessive variants (e.g. "Selina's Gold" -> "Selinas Gold", "Selina s Gold", "Selina Gold")
    $apostropheForms = [$clean];
    if (preg_match("/\b(\w+)['’`](\w+)\b/u", $clean)) {
        $apostropheForms[] = preg_replace("/\b(\w+)['’`](\w+)\b/u", '$1$2', $clean);
        $apostropheForms[] = preg_replace("/\b(\w+)['’`](\w+)\b/u", '$1 $2', $clean);
        $apostropheForms[] = preg_replace("/\b(\w+)['’`]s\b/iu", '$1', $clean);
    } elseif (preg_match("/\b(\w+)s\b/i", $clean)) {
        $apostropheForms[] = preg_replace("/\b(\w+)s\b/i", '$1 s', $clean);
        $apostropheForms[] = preg_replace("/\b(\w+)s\b/i", '$1', $clean);
    }

    // Variants with & replaced by 'dan' (Malay/Indo) or 'and' or space
    $baseForms = [];
    foreach ($apostropheForms as $form) {
        $form = trim(preg_replace('/\s+/', ' ', $form));
        $baseForms[] = $form;
        if (str_contains($form, '&')) {
            $baseForms[] = trim(preg_replace('/\s+/', ' ', str_replace('&', ' dan ', $form)));
            $baseForms[] = trim(preg_replace('/\s+/', ' ', str_replace('&', ' and ', $form)));
            $baseForms[] = trim(preg_replace('/\s+/', ' ', str_replace('&', ' ', $form)));
        } elseif (preg_match('/\b(?:dan|and)\b/i', $form)) {
            $baseForms[] = trim(preg_replace('/\s+/', ' ', preg_replace('/\b(?:dan|and)\b/i', '&', $form)));
            $baseForms[] = trim(preg_replace('/\s+/', ' ', preg_replace('/\b(?:dan|and)\b/i', ' ', $form)));
        }
    }

    $baseForms = array_values(array_unique(array_filter($baseForms)));

    foreach ($baseForms as $bf) {
        if ($year !== '') {
            $variants[] = "{$bf} {$year}";
        }
        $variants[] = $bf;
    }

    return array_values(array_unique(array_filter($variants)));
}

/**
 * Clean Telegram file title from spam prefixes, channel promos, and bot forwarding artifacts.
 */
function fd_clean_media_title(string $title): string
{
    if ($title === '') return '';
    $t = fd_clean_html_entities($title);
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
 * Extract comprehensive media release tags (Resolution, Source, Platform, Codec, Audio, Visual)
 * from filename, caption, and title metadata.
 */
function fd_extract_media_tags(string $title, string $caption = ''): array
{
    $text = $title . ' ' . $caption;

    // 1. Resolution
    $res = '';
    if (preg_match('/\b(2160p|4k|uhd)\b/i', $text)) {
        $res = '4K';
    } elseif (preg_match('/\b(1080p|fhd)\b/i', $text)) {
        $res = '1080p';
    } elseif (preg_match('/\b(720p|hd)\b/i', $text)) {
        $res = '720p';
    } elseif (preg_match('/\b(540p)\b/i', $text)) {
        $res = '540p';
    } elseif (preg_match('/\b(480p|sd)\b/i', $text)) {
        $res = '480p';
    } elseif (preg_match('/\b(360p)\b/i', $text)) {
        $res = '360p';
    }

    // 2. Source / Quality
    $source = '';
    if (preg_match('/\b(remux)\b/i', $text)) {
        $source = 'REMUX';
    } elseif (preg_match('/\b(bluray|blu-ray|bdrip|bbrip|brrip)\b/i', $text)) {
        $source = 'BluRay';
    } elseif (preg_match('/\b(web-?dl)\b/i', $text)) {
        $source = 'WEB-DL';
    } elseif (preg_match('/\b(webrip|web)\b/i', $text)) {
        $source = 'WEBRip';
    } elseif (preg_match('/\b(hdrip)\b/i', $text)) {
        $source = 'HDRip';
    } elseif (preg_match('/\b(hdtv|tvrip|pdtv)\b/i', $text)) {
        $source = 'HDTV';
    } elseif (preg_match('/\b(dvdrip|dvd)\b/i', $text)) {
        $source = 'DVDRip';
    } elseif (preg_match('/\b(hdcam|camrip|cam|telesync|ts|tc)\b/i', $text)) {
        $source = 'CAM';
    }

    // 3. Platform / Streaming Provider
    $platform = '';
    if (preg_match('/\b(nf|netflix)\b/i', $text)) {
        $platform = 'NF';
    } elseif (preg_match('/\b(amzn|primevideo|prime)\b/i', $text)) {
        $platform = 'AMZN';
    } elseif (preg_match('/\b(dsnp|disney\+?|disney)\b/i', $text)) {
        $platform = 'DSNP';
    } elseif (preg_match('/\b(atvp|apple\s*tv\+?)\b/i', $text)) {
        $platform = 'ATVP';
    } elseif (preg_match('/\b(hmax|hbo\s*max)\b/i', $text)) {
        $platform = 'HMAX';
    } elseif (preg_match('/\b(zee5)\b/i', $text)) {
        $platform = 'ZEE5';
    } elseif (preg_match('/\b(hotstar)\b/i', $text)) {
        $platform = 'Hotstar';
    } elseif (preg_match('/\b(viki)\b/i', $text)) {
        $platform = 'Viki';
    } elseif (preg_match('/\b(wetv)\b/i', $text)) {
        $platform = 'WeTV';
    } elseif (preg_match('/\b(iqiyi)\b/i', $text)) {
        $platform = 'iQIYI';
    } elseif (preg_match('/\b(starzplay)\b/i', $text)) {
        $platform = 'StarzPlay';
    }

    // 4. Video Codec
    $codec = '';
    if (preg_match('/\b(hevc|x265|h\.?265)\b/i', $text)) {
        $codec = 'HEVC';
    } elseif (preg_match('/\b(avc|x264|h\.?264)\b/i', $text)) {
        $codec = 'H.264';
    } elseif (preg_match('/\b(av1)\b/i', $text)) {
        $codec = 'AV1';
    } elseif (preg_match('/\b(xvid|divx)\b/i', $text)) {
        $codec = 'XviD';
    }

    // 5. Audio Codec & Channels
    $audio = '';
    if (preg_match('/\b(atmos)\b/i', $text)) {
        $audio = 'Atmos';
    } elseif (preg_match('/\b(ddp\s*5\.1|dd\+\s*5\.1|eac3\s*5\.1)\b/i', $text)) {
        $audio = 'DDP5.1';
    } elseif (preg_match('/\b(ddp\s*2\.0|dd\+\s*2\.0|eac3\s*2\.0)\b/i', $text)) {
        $audio = 'DDP2.0';
    } elseif (preg_match('/\b(ddp|dd\+|eac3)\b/i', $text)) {
        $audio = 'DDP';
    } elseif (preg_match('/\b(dd\s*5\.1|ac3\s*5\.1)\b/i', $text)) {
        $audio = 'DD5.1';
    } elseif (preg_match('/\b(ac3|dd)\b/i', $text)) {
        $audio = 'AC3';
    } elseif (preg_match('/\b(dts-hd\s*ma)\b/i', $text)) {
        $audio = 'DTS-HD MA';
    } elseif (preg_match('/\b(dts-hd)\b/i', $text)) {
        $audio = 'DTS-HD';
    } elseif (preg_match('/\b(dts)\b/i', $text)) {
        $audio = 'DTS';
    } elseif (preg_match('/\b(truehd)\b/i', $text)) {
        $audio = 'TrueHD';
    } elseif (preg_match('/\b(aac\s*5\.1|5\.1\s*aac)\b/i', $text)) {
        $audio = 'AAC5.1';
    } elseif (preg_match('/\b(aac\s*2\.0|2\.0\s*aac|aac2)\b/i', $text)) {
        $audio = 'AAC2.0';
    } elseif (preg_match('/\b(aac)\b/i', $text)) {
        $audio = 'AAC';
    } elseif (preg_match('/\b(flac)\b/i', $text)) {
        $audio = 'FLAC';
    } elseif (preg_match('/\b(opus)\b/i', $text)) {
        $audio = 'Opus';
    }

    // 6. Visual Enhancements (HDR, DV, 10-bit)
    $visual = [];
    if (preg_match('/\b(hdr10\+|hdr10|hdr)\b/i', $text)) {
        $visual[] = 'HDR';
    }
    if (preg_match('/\b(dolby\s*vision|dovi|dv)\b/i', $text)) {
        $visual[] = 'DV';
    }
    if (preg_match('/\b(10bit|10-bit|hi10p?)\b/i', $text)) {
        $visual[] = '10bit';
    }
    if (preg_match('/\b(imax)\b/i', $text)) {
        $visual[] = 'IMAX';
    }

    // 7. Edition / Cuts
    $edition = '';
    if (preg_match('/\b(remastered)\b/i', $text)) {
        $edition = 'Remastered';
    } elseif (preg_match('/\b(extended)\b/i', $text)) {
        $edition = 'Extended';
    } elseif (preg_match('/\b(uncut)\b/i', $text)) {
        $edition = 'Uncut';
    } elseif (preg_match('/\b(repack|proper)\b/i', $text)) {
        $edition = 'Proper';
    }

    return [
        'resolution' => $res,
        'source' => $source,
        'platform' => $platform,
        'codec' => $codec,
        'audio' => $audio,
        'visual' => $visual,
        'edition' => $edition,
    ];
}

/**
 * Calculate a comprehensive sorting score for streams based on:
 * 1) Quality / Source: REMUX > BluRay/BDRip/BBRip > WEB-DL > WEBRip > HDRip > HDTV > DVDRip > CAM/TS
 * 2) Resolution: 4K (2160p) > 1080p > 720p > 540p > 480p > 360p
 * 3) Visual Enhancements: DV / HDR / 10bit
 * 4) Codec: AV1 > HEVC (x265) > H.264 (x264)
 * 5) File size (higher bitrate/quality preferred as tie-breaker)
 */
function fd_calculate_stream_sort_score(string $title, string $caption = '', int $fileSize = 0, string $preferredRes = 'auto'): float
{
    $tags = fd_extract_media_tags($title, $caption);

    // 1. Resolution score (base weight: 1,000,000,000)
    $resScore = match (strtolower($tags['resolution'])) {
        '4k' => 6000000000.0,
        '1080p' => 5000000000.0,
        '720p' => 4000000000.0,
        '540p' => 3000000000.0,
        '480p' => 2000000000.0,
        '360p' => 1000000000.0,
        default => 500000000.0,
    };

    // If user explicitly configured a preferred resolution, boost that resolution to the very top
    if ($preferredRes !== 'auto' && strtolower($tags['resolution']) === strtolower($preferredRes)) {
        $resScore += 10000000000.0;
    }

    // 2. Quality / Source score (weight: 10,000,000)
    // Hierarchy: REMUX > BluRay (BDRip, BBRip, BRRip) > WEB-DL > WEBRip > HDRip > HDTV > DVDRip > CAM/TS/Telesync
    $sourceScore = match (strtoupper($tags['source'])) {
        'REMUX' => 80000000.0,
        'BLURAY' => 70000000.0,
        'WEB-DL' => 60000000.0,
        'WEBRIP' => 50000000.0,
        'HDRIP' => 40000000.0,
        'HDTV' => 30000000.0,
        'DVDRIP' => 20000000.0,
        'CAM' => 1000000.0,
        default => 30000000.0, // unknown default between DVDRip & HDTV
    };

    // 3. Visual tags score (DV, HDR, 10bit)
    $visualScore = 0.0;
    if (!empty($tags['visual'])) {
        if (in_array('DV', $tags['visual'], true)) $visualScore += 4000000.0;
        if (in_array('HDR', $tags['visual'], true)) $visualScore += 2000000.0;
        if (in_array('10bit', $tags['visual'], true)) $visualScore += 1000000.0;
    }

    // 4. Codec score (AV1 > HEVC > H.264)
    $codecScore = match (strtoupper($tags['codec'])) {
        'AV1' => 300000.0,
        'HEVC' => 200000.0,
        'H.264' => 100000.0,
        default => 0.0,
    };

    // 5. File size tie-breaker (normalised so larger high-bitrate files rank higher within same tier)
    $sizeScore = min(99999.0, max(0.0, (float) $fileSize / (1024 * 1024)));

    return $resScore + $sourceScore + $visualScore + $codecScore + $sizeScore;
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
    $t = fd_clean_html_entities($title);
    $t = fd_clean_media_title($t);
    $t = trim($t, " \t\n\r\0\x0B");
    return $t !== '' ? $t : fd_clean_html_entities($title);
}

/**
 * Clean post excerpt or plot description for display.
 */
function fd_clean_post_plot(string $plot): string
{
    if ($plot === '') return '';
    $clean = strip_tags($plot);
    $clean = fd_clean_html_entities($clean);
    return trim($clean);
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
    if ($action === 'trending') {
        unset($queryParams['bot_id']);
    } elseif (empty($queryParams['bot_id'])) {
        $activeBotId = fd_get_bot_id();
        if ($activeBotId !== '') {
            $queryParams['bot_id'] = $activeBotId;
        }
    }
    // Ensure region follows user configuration first, then CF header
    if (empty($queryParams['country'])) {
        $detected = fd_detect_country();
        if (!empty($detected['country_code'])) {
            $queryParams['country'] = $detected['country_code'];
        }
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
 * Cached genre list for /manifest.json.
 *
 * The manifest is fetched by every client on install/refresh, and the genre
 * list is a blocking remote WordPress call (12s timeout). Caching it to disk
 * means only the first request pays that latency; concurrent manifest requests
 * read the cache instead of each opening their own WordPress connection.
 *
 * Single-flight: when the cache is cold, only ONE request performs the remote
 * fetch. Every other concurrent request immediately gets the stale cache (or
 * [] so the caller uses its hardcoded defaults) instead of piling onto
 * WordPress. Without this, a burst of N cold requests each opened their own
 * WordPress connection and timed out together (measured: 71/400 timeouts at
 * C=100).
 *
 * Returns [] on failure so the caller falls back to its hardcoded defaults.
 */
function fd_manifest_categories_cached(): array
{
    $cacheFile = fd_cache_path('manifest_categories.json');
    $ttl = 3600;

    $readCache = static function () use ($cacheFile): array {
        if (!is_file($cacheFile)) {
            return [];
        }
        $data = json_decode((string) @file_get_contents($cacheFile), true);
        return (is_array($data) && !empty($data)) ? $data : [];
    };

    // Fresh cache: serve it.
    if (is_file($cacheFile) && (time() - (int) filemtime($cacheFile)) < $ttl) {
        $cached = $readCache();
        if (!empty($cached)) {
            return $cached;
        }
    }

    // Cold/stale cache: only one request may refresh. Others serve stale now.
    $lockFile = fd_cache_path('manifest_categories.lock');
    $lockFp = @fopen($lockFile, 'c');
    $isRefresher = false;
    if ($lockFp) {
        $isRefresher = @flock($lockFp, LOCK_EX | LOCK_NB);
    }

    if (!$isRefresher) {
        // Someone else is refreshing: serve stale immediately, never block.
        if ($lockFp) {
            @fclose($lockFp);
        }
        return $readCache();
    }

    try {
        // Re-check: the refresher ahead of us may have just published.
        if (is_file($cacheFile) && (time() - (int) filemtime($cacheFile)) < $ttl) {
            $cached = $readCache();
            if (!empty($cached)) {
                return $cached;
            }
        }

        $categories = fd_fetch_stream_ajax('categories');
        if (!empty($categories) && is_array($categories)) {
            @file_put_contents(
                $cacheFile,
                json_encode($categories, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE),
                LOCK_EX
            );
            return $categories;
        }

        // Remote fetch failed: serve stale rather than blocking on a dead upstream.
        return $readCache();
    } finally {
        @flock($lockFp, LOCK_UN);
        @fclose($lockFp);
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
function fd_fetch_episode_stream_files(int $postId, int $season, int $episode, int $maxFiles = 40, ?array $preloadedPost = null): array
{
    $tStart = microtime(true);
    if ($postId <= 0 || $season <= 0 || $episode <= 0) {
        return [];
    }

    if ($preloadedPost !== null && !empty($preloadedPost['title'])) {
        $post = $preloadedPost;
    } else {
        $postData = fd_fetch_stream_ajax('get_post', ['post_id' => $postId]);
        $post = !empty($postData) && is_array($postData) ? ($postData[0] ?? $postData) : [];
    }

    $fullTitle = (string) ($post['title'] ?? '');
    $keyword = fd_stream_keyword_from_post_title($fullTitle);
    $postYear = null;
    if (preg_match('/\b(19\d\d|20\d\d)\b/', $fullTitle, $ym)) {
        $postYear = $ym[1];
    }

    $filter = fd_episode_stream_filter($season, $episode);
    $all = [];
    $seen = [];
    $add = static function (array $files) use (&$all, &$seen, $filter, $maxFiles, $postYear, $keyword, $fullTitle): void {
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

            $fTitle = (string) ($file['title'] ?? '');

            // 1. Strict Year Guard: If file specifies a 4-digit year that contradicts post year, skip!
            // E.g. Post is Glory 2025, but file has 2022 -> reject. Post is The Glory 2022, file has 2025 -> reject.
            // Exception: Allow +/-1 year tolerance if the entire title keywords match (e.g. series released Jan 2026 where uploaders labeled E01/E02 as 2025).
            if ($postYear !== null && preg_match('/\b(19\d\d|20\d\d)\b/', $fTitle, $fym)) {
                $fileYear = (int) $fym[1];
                $targetYear = (int) $postYear;
                if ($fileYear !== $targetYear) {
                    $yearDiff = abs($fileYear - $targetYear);
                    $cleanFTitle = strtolower(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', $fTitle));
                    $cleanPostWords = array_values(array_filter(explode(' ', strtolower(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', (string)$keyword))), fn($w) => strlen($w) > 1));
                    $allWordsMatch = !empty($cleanPostWords);
                    foreach ($cleanPostWords as $pw) {
                        if (!str_contains($cleanFTitle, $pw)) {
                            $allWordsMatch = false;
                            break;
                        }
                    }
                    if ($yearDiff > 1 || !$allWordsMatch) {
                        continue;
                    }
                }
            }

            // 2. Strict Franchise / Title Guard for short titles:
            // Prevents "Glory" from matching unrelated titles like "You Are My Glory", "Gold Rush Our Race to Olympic Glory", etc.
            $cleanKw = strtolower(trim($keyword));
            if (strlen($cleanKw) >= 2) {
                $normFTitle = strtolower(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', $fTitle));
                $normFTitle = preg_replace('/\b(on9|stream|rilisanfilm|dailymaci|kdramashubs|wetv|pahe|galaxy|fanszz|dramaost|mkvdrama|dusklight|primefix|moviesmod|vegamovies|mkvcinemas|nodrakorid)\b/', ' ', $normFTitle);
                $normFTitle = trim(preg_replace('/\s+/', ' ', $normFTitle));

                if ($cleanKw === 'glory') {
                    if (preg_match('/\b(?:my|olympic|special forces|cage of|song of|road to|power and the|greater|for the|tunes of|morning|race for|mad dog and)\s+glory\b/i', $normFTitle)) {
                        continue;
                    }
                    // Differentiate Glory 2025 (Chinese series) from The Glory 2022 (Korean Netflix series)
                    if ($postYear === '2025') {
                        if (preg_match('/\b(2022|korean|korea|k\.o|netflix|\bnf\b|rarbg)\b/i', $fTitle) && !str_contains($fTitle, '2025')) {
                            continue;
                        }
                    } elseif ($postYear === '2022') {
                        if (preg_match('/\b(2025|wetv|dusklight|layarkeren)\b/i', $fTitle) && !str_contains($fTitle, '2022')) {
                            continue;
                        }
                    }
                }
                if (!str_starts_with(strtolower($fullTitle), 'the ') && str_starts_with($normFTitle, 'the ') && $postYear !== null) {
                    if (preg_match('/\b(2022|korean|korea|k\.o|netflix|\bnf\b|rarbg)\b/i', $fTitle) && !str_contains($fTitle, (string)$postYear)) {
                        continue;
                    }
                }
            }

            $seen[$code] = true;
            $all[] = $file;
        }
    };

    // 1. Probe post files for exact SxxExx MATCH (1 page, 50 items)
    $exactFiles = fd_fetch_post_files_paged($postId, [
        'page_size' => 50,
        'max_files' => $maxFiles,
        'season' => $season,
        'episode' => $episode,
        'filter' => $filter,
        'max_pages' => 1,
    ]);
    $add($exactFiles);
    $exactCount = count($all);

    // 2. Fast keyword probe to discover external/Telegram channel releases (MalaySub, Fanszz, DramaOST, etc.)
    if (count($all) < $maxFiles && $keyword !== '') {
        $tokens = sprintf('(E%02d | S%02dE%02d | EP%02d', $episode, $season, $episode, $episode);
        if ($episode < 10) {
            $tokens .= sprintf(' | EP%d | E%d', $episode, $episode);
        }
        $tokens .= ')';

        $q = "{$keyword} {$tokens}";
        $res = fd_fetch_stream_ajax('search_files', [
            'search' => $q,
            'limit' => 50,
            'offset' => 0,
        ]);
        $add((array) ($res['files'] ?? []));

        // Also query with episode token first (e.g. "(E01 | ...) Keyword") to find releases
        // that place the episode tag before the title (e.g. "OLD.E01.To.My.Beloved.Thief...").
        if (count($all) < $maxFiles) {
            $qLeading = "{$tokens} {$keyword}";
            $resLeading = fd_fetch_stream_ajax('search_files', [
                'search' => $qLeading,
                'limit' => 50,
                'offset' => 0,
            ]);
            $add((array) ($resLeading['files'] ?? []));
        }

        // If very few files and postYear is known, try with postYear
        if (count($all) < 15 && $postYear !== null) {
            $qYear = "{$keyword} {$postYear} {$tokens}";
            $resYear = fd_fetch_stream_ajax('search_files', [
                'search' => $qYear,
                'limit' => 50,
                'offset' => 0,
            ]);
            $add((array) ($resYear['files'] ?? []));
        }
    }

    // 3. Fallback scan of post files only if we still have very few files
    if (count($all) < 10) {
        $pagedFiles = fd_fetch_post_files_paged($postId, [
            'page_size' => 150,
            'max_files' => 150,
            'filter' => $filter,
            'max_pages' => 1,
        ]);
        $add($pagedFiles);
    }
    $postCount = count($all);

    $elapsed = round(microtime(true) - $tStart, 3);
    fd_log('stremio episode streams resolved', [
        'postId' => $postId,
        'title' => $fullTitle,
        'season' => $season,
        'episode' => $episode,
        'exactCount' => $exactCount,
        'postCount' => $postCount,
        'totalCount' => count($all),
        'duration_seconds' => $elapsed,
    ]);

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
    $seenEpsBySeason = [];
    $maxEpBySeason = [];
    foreach ($all as $f) {
        $parsed = fd_classify_season_episode(
            (string) ($f['title'] ?? ''),
            (int) ($f['season_num'] ?? 0),
            (int) ($f['episode_num'] ?? 0),
            (string) ($f['caption'] ?? '')
        );
        $ep = (int) ($parsed['episode'] ?? 0);
        $s = max(1, (int) ($parsed['season'] ?? 1));
        if ($ep > 0) {
            $hasExplicitEp = true;
            $seenEpsBySeason[$s][$ep] = true;
            if (!isset($maxEpBySeason[$s]) || $ep > $maxEpBySeason[$s]) {
                $maxEpBySeason[$s] = $ep;
            }
        }
    }

    // Check if there are missing episodes (gaps) in any season (e.g. S1 has E03-E16 but missing E01, E02).
    $hasGaps = false;
    if ($hasExplicitEp) {
        foreach ($maxEpBySeason as $s => $maxEp) {
            for ($checkEp = 1; $checkEp <= $maxEp; $checkEp++) {
                if (empty($seenEpsBySeason[$s][$checkEp])) {
                    $hasGaps = true;
                    break 2;
                }
            }
        }
    }

    // If explicit episodes exist and there are no missing episode gaps, post files are complete.
    if ($hasExplicitEp && !$hasGaps) {
        return $all;
    }

    // Either no explicit episodes (combined packs) or missing episode gaps in the post's files:
    // search for individual episode files by title keyword to backfill missing episodes.
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
        str_starts_with($ip, '172.18.') ||
        str_starts_with($ip, '172.19.') ||
        str_starts_with($ip, '172.20.') ||
        str_starts_with($ip, '172.21.') ||
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
    if ($modeOverride === 'server') {
        return [
            'mode' => 'server',
            'id' => 'org.pencarimovie.addon.server',
            'name' => 'PencariMovie (Server)',
            'description' => 'Stream movies and series from Telegram. Address: ' . $origin,
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

    // If request actually comes through tunnel, use custom subdomain or tunnel origin
    if ($isTunnel) {
        $tunnelOrigin = fd_get_live_tunnel_https_origin();
        if ($tunnelOrigin !== '') {
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
        'mp3' => 'audio/mpeg',
        'm4a' => 'audio/mp4',
        'flac' => 'audio/flac',
        'wav' => 'audio/wav',
        'ogg', 'opus' => 'audio/ogg',
        'aac' => 'audio/aac',
        'mkv' => 'video/x-matroska',
        'webm' => 'video/webm',
        'avi' => 'video/x-msvideo',
        'mov' => 'video/quicktime',
        'ts', 'm2ts' => 'video/mp2t',
        'm4v' => 'video/mp4',
        default => 'video/mp4',
    };
}

function fd_mime_to_extension(string $mime, string $fallback = 'mp4'): string
{
    $m = strtolower(trim($mime));
    if ($m === '') {
        return $fallback;
    }
    return match (true) {
        str_contains($m, 'matroska') => 'mkv',
        str_contains($m, 'webm') => 'webm',
        str_contains($m, 'quicktime') => 'mov',
        str_contains($m, 'x-msvideo'), str_contains($m, 'avi') => 'avi',
        str_contains($m, 'mp2t'), str_contains($m, 'm2ts') => 'ts',
        str_contains($m, 'flv') => 'flv',
        str_contains($m, 'wmv') => 'wmv',
        str_contains($m, '3gp') => '3gp',
        str_contains($m, 'audio/mpeg'), str_contains($m, 'mp3') => 'mp3',
        str_contains($m, 'audio/mp4'), str_contains($m, 'm4a') => 'm4a',
        str_contains($m, 'audio/flac'), str_contains($m, 'flac') => 'flac',
        str_contains($m, 'audio/wav'), str_contains($m, 'wave') => 'wav',
        str_contains($m, 'audio/ogg'), str_contains($m, 'opus') => 'ogg',
        str_contains($m, 'audio/aac'), str_contains($m, 'aac') => 'aac',
        str_contains($m, 'mp4'), str_contains($m, 'm4v') => 'mp4',
        str_contains($m, 'video/mpeg'), str_contains($m, 'mpeg'), str_contains($m, 'mpg') => 'mpg',
        str_contains($m, 'audio/flac'), str_contains($m, 'flac') => 'flac',
        str_contains($m, 'audio/wav'), str_contains($m, 'wave') => 'wav',
        str_contains($m, 'audio/ogg'), str_contains($m, 'opus') => 'ogg',
        str_contains($m, 'zip') => 'zip',
        str_contains($m, 'x-rar'), str_contains($m, 'rar') => 'rar',
        str_contains($m, '7z') => '7z',
        str_contains($m, 'tar') => 'tar',
        str_contains($m, 'gzip') => 'gz',
        default => $fallback,
    };
}

function fd_stremio_stream_filename(string $fileName, string $mime = ''): string
{
    $name = trim($fileName);
    if ($name === '') {
        $name = 'file';
    }

    // Preserve existing extension if it's already a raw split chunk (.001, .002) or archive
    if (preg_match('/\.(?:0\d{2,3}|\d{3}|zip|rar|7z|tar|gz|bz2|xz|iso|bin|exe|apk|pdf|epub)$/i', $name)) {
        $name = preg_replace('/[^\w.\-]+/', '_', $name) ?: 'file';
        return trim($name, '._-');
    }

    $origExt = strtolower(pathinfo($fileName, PATHINFO_EXTENSION));
    $isAudio = str_starts_with(strtolower($mime), 'audio/') || in_array($origExt, ['mp3', 'm4a', 'flac', 'wav', 'ogg', 'opus', 'aac'], true);
    $targetExt = in_array($origExt, ['mp3', 'm4a', 'flac', 'wav', 'ogg', 'opus', 'aac'], true)
        ? $origExt
        : fd_mime_to_extension($mime, $isAudio ? 'mp3' : 'mp4');

    // Strip duplicate/nested media extensions (e.g. filename.mp4.mkv -> filename.mkv)
    $mediaPattern = '/\.(mp4|m4v|mkv|webm|avi|mov|ts|m2ts|flv|wmv|3gp|mpg|mpeg|mp3|m4a|flac|wav|ogg|opus|aac)$/i';
    while (preg_match($mediaPattern, $name)) {
        $name = preg_replace($mediaPattern, '', $name);
    }

    $defaultBase = $isAudio ? 'audio' : 'video';
    $name = preg_replace('/[^\w.\-]+/', '_', $name) ?: $defaultBase;
    $name = trim($name, '._-');
    if ($name === '') {
        $name = $defaultBase;
    }

    return $name . '.' . $targetExt;
}

/**
 * Stream URL builder using path format: /api/download/<payload>/<filename>
 */
function fd_build_stremio_stream_url(string $baseUrl, string $payloadB64, string $fileName, string $mime = ''): string
{
    $safe = fd_stremio_stream_filename($fileName, $mime);
    // Carry the auth token as a query param so remote players can fetch the
    // file. Localhost/LAN requests ignore it. The real filename + extension
    // from fd_stremio_stream_filename() are preserved untouched.
    $token = fd_auth_enabled() ? fd_auth_token() : '';
    $query = $token !== '' ? '?token=' . rawurlencode($token) : '';
    return rtrim($baseUrl, '/') . '/api/download/' . rawurlencode($payloadB64) . '/' . rawurlencode($safe) . $query;
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
    $mediaExts = ['mp4', 'm4v', 'mkv', 'webm', 'avi', 'mov', 'ts', 'm2ts', 'flv', 'wmv', '3gp', 'mpg', 'mpeg', 'mp3', 'm4a', 'flac', 'wav', 'ogg', 'opus'];
    if (in_array($ext, $mediaExts, true)) {
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

function fd_tunnel_token_path(): string
{
    return fd_tunnel_dir() . DIRECTORY_SEPARATOR . 'token.txt';
}

function fd_load_saved_tunnel_token(): string
{
    $path = fd_tunnel_token_path();
    if (!is_file($path)) {
        return '';
    }
    return trim((string) @file_get_contents($path));
}

function fd_save_tunnel_token(string $token): void
{
    $token = trim($token);
    if ($token !== '') {
        @file_put_contents(fd_tunnel_token_path(), $token, LOCK_EX);
    }
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
 * Map of trending keyword catalog IDs to their search keyword strings.
 * Shared by manifest building and catalog routing so both resolve the same
 * top-keyword catalogs (limited to the top 5).
 */
function fd_get_trending_keywords_map(): array
{
    static $cached = null;
    if ($cached !== null) {
        return $cached;
    }
    $cached = [];
    try {
        $trending = fd_fetch_stream_ajax('trending', ['limit' => 15]);
        if (is_array($trending)) {
            foreach ($trending as $item) {
                $kw = trim((string) ($item['keyword'] ?? ''));
                if (!fd_is_topkeyword_valid($kw)) continue;
                $catId = 'pm_topkw_' . substr(md5(strtolower($kw)), 0, 10);
                $cached[$catId] = $kw;
                if (count($cached) >= 5) {
                    break;
                }
            }
        }
    } catch (\Throwable $e) {
    }
    return $cached;
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
        'country' => '',
        'enabled_types' => [
            'movie' => true,
            'series' => true,
            'other' => true,
        ],
        'enabled_catalogs' => [],
        'upstream_manifests' => [],
        'stream_config' => [
            'resolutions' => [
                '4k' => true,
                '1080p' => true,
                '720p' => true,
                'sd' => true,
                'unknown' => true,
            ],
            'qualities' => [
                'remux' => true,
                'bluray' => true,
                'webdl' => true,
                'webrip' => true,
                'hdtv' => true,
                'cam' => true,
                'unknown' => true,
            ],
            'encodes' => [
                'hevc' => true,
                'avc' => true,
                'av1' => true,
            ],
            'visual_tags' => [
                'hdr' => true,
                'dv' => true,
            ],
            'preferred_resolution' => 'auto',
            'max_streams_per_resolution' => 0,
            'max_streams_total' => 0,
            'min_size_mb' => 0,
            'max_size_gb' => 0,
            'exclude_cam' => false,
            'exclude_unplayable' => true,
            'excluded_keywords' => '',
            'required_keywords' => '',
        ],
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
            if (isset($data['country']) && is_string($data['country'])) {
                $defaults['country'] = strtoupper(trim(preg_replace('/[^a-zA-Z0-9_-]/', '', $data['country'])));
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
            if (isset($data['upstream_manifests']) && is_array($data['upstream_manifests'])) {
                $defaults['upstream_manifests'] = array_values(array_filter($data['upstream_manifests'], function ($m) {
                    return is_array($m) && !empty($m['url']);
                }));
            }
            if (isset($data['stream_config']) && is_array($data['stream_config'])) {
                $sc = $data['stream_config'];
                if (isset($sc['resolutions']) && is_array($sc['resolutions'])) {
                    foreach (['4k', '1080p', '720p', 'sd', 'unknown'] as $rKey) {
                        if (isset($sc['resolutions'][$rKey])) {
                            $defaults['stream_config']['resolutions'][$rKey] = (bool) $sc['resolutions'][$rKey];
                        }
                    }
                }
                if (isset($sc['qualities']) && is_array($sc['qualities'])) {
                    foreach (['remux', 'bluray', 'webdl', 'webrip', 'hdtv', 'cam', 'unknown'] as $qKey) {
                        if (isset($sc['qualities'][$qKey])) {
                            $defaults['stream_config']['qualities'][$qKey] = (bool) $sc['qualities'][$qKey];
                        }
                    }
                }
                if (isset($sc['encodes']) && is_array($sc['encodes'])) {
                    foreach (['hevc', 'avc', 'av1'] as $eKey) {
                        if (isset($sc['encodes'][$eKey])) {
                            $defaults['stream_config']['encodes'][$eKey] = (bool) $sc['encodes'][$eKey];
                        }
                    }
                }
                if (isset($sc['visual_tags']) && is_array($sc['visual_tags'])) {
                    foreach (['hdr', 'dv'] as $vKey) {
                        if (isset($sc['visual_tags'][$vKey])) {
                            $defaults['stream_config']['visual_tags'][$vKey] = (bool) $sc['visual_tags'][$vKey];
                        }
                    }
                }
                if (isset($sc['preferred_resolution'])) {
                    $defaults['stream_config']['preferred_resolution'] = (string) $sc['preferred_resolution'];
                }
                if (isset($sc['max_streams_per_resolution'])) {
                    $defaults['stream_config']['max_streams_per_resolution'] = (int) $sc['max_streams_per_resolution'];
                }
                if (isset($sc['max_streams_total'])) {
                    $defaults['stream_config']['max_streams_total'] = (int) $sc['max_streams_total'];
                }
                if (isset($sc['min_size_mb'])) {
                    $defaults['stream_config']['min_size_mb'] = (int) $sc['min_size_mb'];
                }
                if (isset($sc['max_size_gb'])) {
                    $defaults['stream_config']['max_size_gb'] = (int) $sc['max_size_gb'];
                }
                if (isset($sc['exclude_cam'])) {
                    $defaults['stream_config']['exclude_cam'] = (bool) $sc['exclude_cam'];
                }
                if (isset($sc['exclude_unplayable'])) {
                    $defaults['stream_config']['exclude_unplayable'] = (bool) $sc['exclude_unplayable'];
                }
                if (isset($sc['excluded_keywords'])) {
                    $defaults['stream_config']['excluded_keywords'] = trim((string) $sc['excluded_keywords']);
                }
                if (isset($sc['required_keywords'])) {
                    $defaults['stream_config']['required_keywords'] = trim((string) $sc['required_keywords']);
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
    // Check if user manually configured a preferred country in catalog settings
    $catSettings = fd_load_catalog_settings();
    if (!empty($catSettings['country'])) {
        $code = strtoupper(trim((string) $catSettings['country']));
        $source = 'configured';
    } else {
        $code = strtoupper(trim((string) ($_SERVER['HTTP_CF_IPCOUNTRY'] ?? '')));
        $source = 'cf-header';
    }

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
    $data = [];
    if (is_file($path)) {
        $raw = @file_get_contents($path);
        if (is_string($raw) && $raw !== '') {
            $decoded = json_decode($raw, true);
            if (is_array($decoded)) {
                $data = $decoded;
            }
        }
    }
    if (empty($data['tunnel_token'])) {
        $saved = fd_load_saved_tunnel_token();
        if ($saved !== '') {
            $data['tunnel_token'] = $saved;
        }
    }
    return $data;
}

function fd_save_tunnel_state(array $state): void
{
    if (!empty($state['tunnel_token'])) {
        fd_save_tunnel_token((string) $state['tunnel_token']);
    }
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

function fd_clear_tunnel_state(bool $preserveToken = true): void
{
    $savedToken = $preserveToken ? fd_load_saved_tunnel_token() : '';
    $path = fd_tunnel_state_path();
    if (is_file($path)) {
        @unlink($path);
    }
    $pidPath = fd_tunnel_pid_path();
    if (is_file($pidPath)) {
        @unlink($pidPath);
    }
    if ($preserveToken && $savedToken !== '') {
        fd_save_tunnel_token($savedToken);
    } elseif (!$preserveToken) {
        @unlink(fd_tunnel_token_path());
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

function fd_tunnel_parse_ingress_domains(string $logs, int $port = 8088): array
{
    if ($logs === '' || !str_contains($logs, 'ingress')) {
        return [];
    }
    $matched = [];
    $allHosts = [];
    foreach (explode("\n", $logs) as $line) {
        $t = trim($line);
        if ($t === '' || !str_contains($t, 'ingress')) {
            continue;
        }
        $d = json_decode($t, true);
        if (!is_array($d) || empty($d['config'])) {
            continue;
        }
        $cfg = json_decode((string) $d['config'], true);
        if (!is_array($cfg) || empty($cfg['ingress'])) {
            continue;
        }
        foreach ($cfg['ingress'] as $ing) {
            $host = strtolower(trim((string) ($ing['hostname'] ?? '')));
            $svc = strtolower(trim((string) ($ing['service'] ?? '')));
            if ($host === '' || str_starts_with($svc, 'http_status:')) {
                continue;
            }
            $allHosts[$host] = true;
            if (str_contains($svc, (string) $port)) {
                $matched[$host] = true;
            }
        }
    }
    return array_values(array_keys(!empty($matched) ? $matched : $allHosts));
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
        $tarExe = (fd_is_windows() && is_file('C:\\Windows\\System32\\tar.exe')) ? 'C:\\Windows\\System32\\tar.exe' : 'tar';
        @exec(escapeshellarg($tarExe) . ' -xzf ' . escapeshellarg($tgz) . ' -C ' . escapeshellarg($dir) . ' 2>&1');
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

function fd_tunnel_spawn(string $bin, string $localUrl, int $metricsPort = 20241, string $tunnelToken = ''): array
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

    $isNamedTunnel = trim($tunnelToken) !== '';

    if (fd_is_windows()) {
        $binReal = realpath($bin) ?: $bin;
        $configReal = realpath($config) ?: $config;
        $logReal = realpath(dirname($logFile)) ? (realpath(dirname($logFile)) . DIRECTORY_SEPARATOR . basename($logFile)) : $logFile;

        // Use proc_open with bypass_shell = true so Windows executes the binary directly without cmd.exe or powershell dependency
        if ($isNamedTunnel) {
            $cmdLine = escapeshellarg($binReal)
                . ' tunnel --logfile ' . escapeshellarg($logReal)
                . ' --metrics ' . escapeshellarg($metrics)
                . ' --no-autoupdate --retries 99 run --token ' . escapeshellarg(trim($tunnelToken));
        } else {
            $cmdLine = escapeshellarg($binReal)
                . ' tunnel --url ' . escapeshellarg($localUrl)
                . ' --config ' . escapeshellarg($configReal)
                . ' --logfile ' . escapeshellarg($logReal)
                . ' --metrics ' . escapeshellarg($metrics)
                . ' --no-autoupdate --retries 99';
        }

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

            if ($isNamedTunnel) {
                $cmd = 'TUNNEL_TRANSPORT_PROTOCOL=http2 ' . $sslEnv . $detachPrefix . escapeshellarg($prootBin) . ' --link2symlink -0 '
                    . $prootBinds . ' '
                    . escapeshellarg($bin)
                    . ' tunnel --logfile ' . escapeshellarg($logFile)
                    . ' --metrics ' . escapeshellarg($metrics)
                    . ' --edge-ip-version 4'
                    . ' --no-autoupdate --retries 99 run --token ' . escapeshellarg(trim($tunnelToken))
                    . ' >> ' . escapeshellarg($errLog)
                    . ' 2>&1 & echo $!';
            } else {
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
            }
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

    if ($isNamedTunnel) {
        $cmd = 'TUNNEL_TRANSPORT_PROTOCOL=http2 ' . $sslEnv . $detachPrefix . escapeshellarg($bin)
            . ' tunnel --logfile ' . escapeshellarg($logFile)
            . ' --metrics ' . escapeshellarg($metrics)
            . ' --edge-ip-version 4'
            . ' --no-autoupdate --retries 99 run --token ' . escapeshellarg(trim($tunnelToken))
            . ' >> ' . escapeshellarg($errLog)
            . ' 2>&1 & echo $!';
    } else {
        $cmd = 'TUNNEL_TRANSPORT_PROTOCOL=http2 ' . $sslEnv . $detachPrefix . escapeshellarg($bin)
            . ' tunnel --url ' . escapeshellarg($localUrl)
            . ' --config ' . escapeshellarg($config)
            . ' --logfile ' . escapeshellarg($logFile)
            . ' --metrics ' . escapeshellarg($metrics)
            . ' --edge-ip-version 4'
            . ' --no-autoupdate --retries 99'
            . ' >> ' . escapeshellarg($errLog)
            . ' 2>&1 & echo $!';
    }
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

    $tunnelToken = trim((string) ($state['tunnel_token'] ?? ''));
    $localUrl = 'http://127.0.0.1:' . $port;
    fd_tunnel_kill_leftovers();

    [$newPid, $spawnError] = fd_tunnel_spawn($bin, $localUrl, $metricsPort, $tunnelToken);
    if ($newPid <= 1) {
        $state['restart_attempt_at'] = time();
        fd_save_tunnel_state($state);
        fd_log('tunnel auto-restart failed', ['error' => $spawnError]);
        return false;
    }

    if ($tunnelToken !== '') {
        // Named tunnel with custom token
        $state['enabled'] = true;
        $state['pid'] = $newPid;
        $state['metrics_port'] = $metricsPort;
        $state['started_at'] = time();
        $state['restart_attempt_at'] = time();
        fd_save_tunnel_state($state);
        fd_tunnel_write_pid($newPid);
        fd_log('named tunnel auto-restarted', ['pid' => $newPid]);
        return true;
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

function fd_tunnel_extract_token(string $raw): string
{
    $raw = trim($raw);
    if ($raw === '') {
        return '';
    }
    // Handle full service install command or raw token
    if (preg_match('/eyJh[A-Za-z0-9_=-]+/', $raw, $m)) {
        return $m[0];
    }
    return $raw;
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

    $tunnelToken = trim((string) ($state['tunnel_token'] ?? ''));
    $customDomain = trim((string) ($state['custom_domain'] ?? ''));
    $metricsPort = (int) ($state['metrics_port'] ?? 0);
    $url = trim((string) ($state['tunnel_url'] ?? ''));

    if ($alive && $tunnelToken !== '') {
        $port = (int) ($state['local_port'] ?? fd_get_listen_port());
        $logs = fd_tunnel_read_logs();
        $ingressDomains = fd_tunnel_parse_ingress_domains($logs, $port);
        if (!empty($ingressDomains)) {
            $customDomain = $ingressDomains[0];
            $state['custom_domain'] = $customDomain;
            $state['custom_domains'] = $ingressDomains;
            fd_save_tunnel_state($state);
        }

        $publicDomainUrl = $customDomain !== '' ? ('https://' . $customDomain) : '';
        $activeUrl = $publicDomainUrl;
        $manifestUrl = $activeUrl !== '' ? (rtrim($activeUrl, '/') . '/manifest.json') : '';

        $domainMsg = !empty($ingressDomains)
            ? ('Live on ' . implode(', ', array_map(fn($d) => 'https://' . $d, $ingressDomains)))
            : ($publicDomainUrl !== '' ? ('Live on ' . $publicDomainUrl) : 'Named Cloudflare Tunnel connected.');

        return [
            'ok' => 1,
            'enabled' => true,
            'running' => true,
            'pid' => $pid,
            'tunnel_token' => $tunnelToken,
            'tunnel_url' => $publicDomainUrl,
            'public_url' => $publicDomainUrl,
            'custom_domain' => $customDomain,
            'custom_domains' => $ingressDomains,
            'device_id' => fd_get_device_id(),
            'manifest_url' => $manifestUrl,
            'local_port' => $port,
            'started_at' => (int) ($state['started_at'] ?? 0),
            'message' => $domainMsg,
        ];
    }

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
        'tunnel_token' => $tunnelToken,
        'tunnel_url' => $enabled ? $url : '',
        'public_url' => $publicDomainUrl,
        'custom_domain' => '',
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
    fd_clear_tunnel_state(true);
    return [
        'ok' => 1,
        'enabled' => false,
        'running' => false,
        'pid' => 0,
        'tunnel_token' => fd_load_saved_tunnel_token(),
        'tunnel_url' => '',
        'manifest_url' => '',
        'message' => 'Cloudflare tunnel stopped.',
    ];
}

function fd_enable_tunnel(string $rawToken = ''): array
{
    @set_time_limit(180);
    if (function_exists('ignore_user_abort')) {
        ignore_user_abort(true);
    }

    $tunnelToken = fd_tunnel_extract_token($rawToken);
    if ($tunnelToken === '') {
        $tunnelToken = fd_load_saved_tunnel_token();
    } else {
        fd_save_tunnel_token($tunnelToken);
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
    [$pid, $spawnError] = fd_tunnel_spawn($bin, $localUrl, $metricsPort, $tunnelToken);
    if ($pid <= 1) {
        return ['ok' => 0, 'enabled' => false, 'message' => $spawnError ?: 'Failed to start cloudflared.'];
    }

    if ($tunnelToken !== '') {
        // Wait up to 10s for the tunnel to register and retrieve connector ready status
        $ready = false;
        $customDomain = '';
        $deadline = microtime(true) + 12;
        while (microtime(true) < $deadline) {
            $resp = @fd_tunnel_local_http_get('http://127.0.0.1:' . $metricsPort . '/ready', 1);
            if ($resp !== '' && stripos($resp, '"status":200') !== false) {
                $ready = true;
                break;
            }
            usleep(250000);
        }

        $logs = fd_tunnel_read_logs();
        $ingressDomains = fd_tunnel_parse_ingress_domains($logs, $port);
        if (!empty($ingressDomains)) {
            $customDomain = $ingressDomains[0];
        }

        $publicUrl = $customDomain !== '' ? ('https://' . $customDomain) : '';
        $state = [
            'enabled' => true,
            'pid' => $pid,
            'tunnel_token' => $tunnelToken,
            'custom_domain' => $customDomain,
            'custom_domains' => $ingressDomains,
            'tunnel_url' => $publicUrl,
            'public_url' => $publicUrl,
            'local_port' => $port,
            'metrics_port' => $metricsPort,
            'started_at' => time(),
            'bin' => $bin,
        ];
        fd_save_tunnel_state($state);
        fd_tunnel_write_pid($pid);

        $domainMsg = !empty($ingressDomains)
            ? ('Connected to ' . implode(', ', array_map(fn($d) => 'https://' . $d, $ingressDomains)))
            : ($publicUrl !== '' ? ('Connected to ' . $publicUrl) : 'Named Cloudflare Tunnel connected.');

        return [
            'ok' => 1,
            'enabled' => true,
            'running' => fd_tunnel_pid_alive($pid),
            'pid' => $pid,
            'tunnel_token' => $tunnelToken,
            'custom_domain' => $customDomain,
            'custom_domains' => $ingressDomains,
            'tunnel_url' => $publicUrl,
            'public_url' => $publicUrl,
            'manifest_url' => $publicUrl !== '' ? (rtrim($publicUrl, '/') . '/manifest.json') : '',
            'local_port' => $port,
            'started_at' => $state['started_at'],
            'message' => $domainMsg,
        ];
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

/**
 * Helper to clean music track titles and split artist - title.
 */
function fd_eclipse_parse_audio_title(string $rawTitle, string $caption = ''): array
{
    // Known artist names used for "Artist Title" splitting when no separator
    // exists in the filename or caption.
    static $knownArtists = [
        'wali',
        'st12',
        'tulus',
        'armada',
        'dmasiv',
        "d'masiv",
        'cokelat',
        'judika',
        'tiara andini',
        'tiara',
        'lyodra',
        'sheila on 7',
        'sheila',
        'dewa 19',
        'dewa',
        'peterpan',
        'noah',
        'rossa',
        'afgan',
        'raisa',
        'yovie & nuno',
        'yovie',
        'amuk',
        'adele',
        'rihanna',
        'eminem',
        'drake',
        'beyonce',
        'shakira',
        'sienna spiro',
        'billie eilish',
        'ariana grande',
        'taylor swift',
        'luke combs',
        'ed sheeran',
        'bruno mars',
        'olivia rodrigo',
        'justin bieber',
        'dua lipa',
        'lady gaga',
        'kelly clarkson',
        'selena gomez',
        'camila cabello',
        'doja cat',
        'katy perry',
        'miley cyrus',
        'avril lavigne',
        'mariah carey',
        'whitney houston',
        'britney spears',
        'michael jackson',
        'the weeknd',
        'post malone',
        'harry styles',
        'charlie puth',
        'shawn mendes',
        'sam smith',
        'sia',
        'pink',
        'coldplay',
        'maroon 5',
        'imagine dragons',
        'playboi carti',
        'doechii',
        'kendrick lamar',
        'travis scott',
        'metro boomin',
        'future',
        '21 savage',
        'sza',
        'frank ocean',
        'tyler the creator',
        'asap rocky',
        'lil baby',
        'gunna',
        'young thug',
        'nicki minaj',
        'cardi b',
        'megan thee stallion',
    ];

    $name = trim($rawTitle);
    // Strip leading track numbers like "01.", "08.", "4.", "01 - "
    $name = preg_replace('/^\d+[\s.\-_]+/', '', $name);
    // Strip media extensions
    $mediaPattern = '/\.(mp3|m4a|flac|wav|ogg|opus|aac|mp4|mkv|mpg)$/i';
    while (preg_match($mediaPattern, $name)) {
        $name = preg_replace($mediaPattern, '', $name);
    }

    // Preserve apostrophe variants before replacing dots
    $name = str_replace(['‘', '’', '`'], "'", $name);
    $name = preg_replace('/\bd\.masiv\b/i', "D'MASIV", $name);
    $name = preg_replace('/\bd\s+masiv\b/i', "D'MASIV", $name);

    // Strip bracketed clutter e.g. [128kbps], (Official Lyric Video), (Lyrics)
    $name = preg_replace('/\s*\[[^\]]*\]|\s*\([^\)]*\)/', ' ', $name);

    // Strip common YouTube/clutter tokens
    $name = preg_replace('/\b(?:1080p|720p|480p|140|251|599|vc\s*trinity|nagaswara|youtube|official\s*(?:video|audio|lyric|lyrics)?|audio|lyrics?|lyrics\s*video|hd|hq|live|remix|cover|karaoke|acoustic|video)\b/i', ' ', $name);

    // Replace dots and underscores with spaces
    $name = str_replace(['.', '_'], ' ', $name);
    $name = trim(preg_replace('/\s+/', ' ', $name));

    $artist = '';
    $title = $name;
    $isrc = '';

    // First check caption for "Artist - Title [ISRC]"
    if ($caption !== '') {
        if (preg_match('/\[([A-Z]{2}[A-Z0-9]{3}[0-9]{7})\]/', $caption, $isrcM)) {
            $isrc = $isrcM[1];
        }

        // Caption format "<short_code> <Title> <Artist>" (no separator), e.g.
        // "AgADMG4AAqklwUs Timeless (Tasty Or Not Afro House Remix) The Weeknd, Playboi Carti".
        // The leading token is a Telegram file code, the trailing words are the
        // artist. Split at the last known artist name found in the caption.
        if ($artist === '' && preg_match('/^(?:AgAD|AQAD)\S+\s+(.+)$/u', trim($caption), $codeM)) {
            $afterCode = trim($codeM[1]);
            // Strip a trailing bracketed/parenthesised qualifier for matching.
            $afterCodeNorm = trim(preg_replace('/\s*[\(\[][^\)\]]*[\)\]]/', ' ', $afterCode));
            $afterCodeNorm = trim(preg_replace('/\s+/', ' ', $afterCodeNorm));
            $lowerAfter = strtolower($afterCodeNorm);
            // Collect every known-artist match position (word-boundary checked).
            $artistHits = [];
            foreach ($knownArtists as $ka) {
                $pos = strpos($lowerAfter, $ka);
                while ($pos !== false) {
                    $before = $pos > 0 ? $lowerAfter[$pos - 1] : ' ';
                    $after = ($pos + strlen($ka)) < strlen($lowerAfter) ? $lowerAfter[$pos + strlen($ka)] : ' ';
                    if (($before === ' ' || $before === ',') && ($after === ' ' || $after === ',')) {
                        $artistHits[] = ['start' => $pos, 'end' => $pos + strlen($ka)];
                    }
                    $pos = strpos($lowerAfter, $ka, $pos + 1);
                }
            }
            // The artist is the trailing run of artist names. Walk backwards from
            // the end: keep extending the artist span while the gap between the
            // previous hit and the current span is only separators (", ", " & ",
            // " and ", " ft. ", " feat. "). This turns
            // "Timeless The Weeknd, Playboi Carti" into artist
            // "The Weeknd, Playboi Carti" and title "Timeless".
            if (!empty($artistHits)) {
                usort($artistHits, static fn(array $a, array $b): int => $a['start'] <=> $b['start']);
                $spanStart = $artistHits[count($artistHits) - 1]['start'];
                for ($i = count($artistHits) - 2; $i >= 0; $i--) {
                    $gap = substr($lowerAfter, $artistHits[$i]['end'], $spanStart - $artistHits[$i]['end']);
                    if (preg_match('/^\s*(?:,|&|and|ft\.?|feat\.?|x|with)\s*$/i', $gap)) {
                        $spanStart = $artistHits[$i]['start'];
                    } else {
                        break;
                    }
                }
                if ($spanStart > 0) {
                    $titlePart = trim(substr($afterCodeNorm, 0, $spanStart));
                    $artistPart = trim(substr($afterCodeNorm, $spanStart));
                    // Normalise "The Weeknd, Playboi Carti" → keep as-is.
                    $artistPart = trim(preg_replace('/\s*,\s*/', ', ', $artistPart));
                    // Strip any known-artist names that leaked into the title
                    // (e.g. "Playboi Carti Timeless" → "Timeless").
                    foreach ($knownArtists as $ka) {
                        $titlePart = preg_replace('/\b' . preg_quote($ka, '/') . '\b/i', ' ', $titlePart);
                    }
                    $titlePart = trim(preg_replace('/\s+/', ' ', $titlePart));
                    // Drop dangling feature connectors left at the end, repeatedly
                    // ("Timeless Remix ft. &" → "Timeless Remix").
                    for ($pass = 0; $pass < 4; $pass++) {
                        $before = $titlePart;
                        $titlePart = preg_replace('/[\s,&]+(?:ft\.?|feat\.?|with|x|and)?[\s,&]*$/i', '', $titlePart);
                        $titlePart = trim($titlePart);
                        if ($titlePart === $before) {
                            break;
                        }
                    }
                    $titlePart = trim($titlePart, " \t\n\r\0\x0B,-&");
                    if ($titlePart !== '' && $artistPart !== '') {
                        $artist = ucwords($artistPart);
                        $title = $titlePart;
                    }
                }
            }
        }

        $lines = explode("\n", $caption);
        foreach ($lines as $line) {
            if ($artist !== '') {
                break;
            }
            $line = trim($line);
            // Skip Telegram forward/relay noise: "forwarded from -1001064830073 ..."
            // would otherwise match the "Artist - Title" pattern on the channel id
            // hyphen and produce artist="forwarded from" with a numeric title.
            if (preg_match('/^(?:forwarded\s+from|forwarded|via|from)\b/i', $line)) {
                continue;
            }
            if (preg_match('/^([^\-]+?)\s*[\-–—]\s*([^\[]+)/u', $line, $m)) {
                $candArtist = trim($m[1]);
                $candTitle = trim($m[2]);
                if ($candArtist !== '' && $candTitle !== '' && !str_starts_with($candArtist, '#') && !str_starts_with($candArtist, '@') && !preg_match('/^(?:AgAD|AQAD)/', $candArtist)) {
                    $artist = $candArtist;
                    $title = $candTitle;
                    break;
                }
            }
        }
    }

    if ($artist === '') {
        // 1. Check for "Artist - Title" or "Artist – Title"
        if (preg_match('/^([^\-]+?)\s*[\-–—]\s*(.+)$/u', $name, $m)) {
            $artist = trim($m[1]);
            $title = trim($m[2]);
        }
        // 2. Check for "Artist by Title" or "Title by Artist"
        elseif (preg_match('/^(.+?)\s+by\s+(.+)$/i', $name, $m)) {
            $title = trim($m[1]);
            $artist = trim($m[2]);
        }
        // 3. Known artist prefix matching
        else {
            $words = explode(' ', $name);
            if (count($words) >= 2) {
                if (count($words) >= 3) {
                    $cand2 = strtolower($words[0] . ' ' . $words[1]);
                    if (in_array($cand2, $knownArtists, true)) {
                        $artist = $words[0] . ' ' . $words[1];
                        $title = implode(' ', array_slice($words, 2));
                    }
                }
                if ($artist === '') {
                    $cand1 = strtolower($words[0]);
                    if (in_array($cand1, $knownArtists, true)) {
                        $artist = $words[0];
                        $title = implode(' ', array_slice($words, 1));
                    }
                }
            }
        }
    }

    // Normalize hyphen variants in title ("Hati-Hati" vs "Hati Hati")
    $title = preg_replace('/[^\p{L}\p{N}\s\']+/u', ' ', $title);
    $title = trim(preg_replace('/\s+/', ' ', $title));

    if ($artist === '') {
        $artist = $name;
    }

    return [
        'artist' => $artist,
        'title'  => $title,
        'isrc'   => $isrc,
    ];
}

/**
 * Format a tg_file_new music record into Eclipse Music Track format.
 */
function fd_eclipse_format_track(array $item, string $baseUrl, string $searchQuery = ''): array
{
    // Known artist names for "Title Artist" reversed-order splitting.
    static $knownArtists = [
        'wali',
        'st12',
        'tulus',
        'armada',
        'dmasiv',
        "d'masiv",
        'cokelat',
        'judika',
        'tiara andini',
        'tiara',
        'lyodra',
        'sheila on 7',
        'sheila',
        'dewa 19',
        'dewa',
        'peterpan',
        'noah',
        'rossa',
        'afgan',
        'raisa',
        'yovie & nuno',
        'yovie',
        'amuk',
        'adele',
        'rihanna',
        'eminem',
        'drake',
        'beyonce',
        'shakira',
        'sienna spiro',
        'billie eilish',
        'ariana grande',
        'taylor swift',
        'luke combs',
        'ed sheeran',
        'bruno mars',
        'olivia rodrigo',
        'justin bieber',
        'dua lipa',
        'lady gaga',
        'kelly clarkson',
        'selena gomez',
        'camila cabello',
        'doja cat',
        'katy perry',
        'miley cyrus',
        'avril lavigne',
        'mariah carey',
        'whitney houston',
        'britney spears',
        'michael jackson',
        'the weeknd',
        'post malone',
        'harry styles',
        'charlie puth',
        'shawn mendes',
        'sam smith',
        'sia',
        'pink',
        'coldplay',
        'maroon 5',
        'imagine dragons',
    ];

    $sc = (string) ($item['short_code'] ?? '');
    $rawTitle = (string) ($item['title'] ?? '');
    $caption = (string) ($item['caption'] ?? '');
    $itemPerformer = trim((string) ($item['performer'] ?? ''));
    $parsed = fd_eclipse_parse_audio_title($rawTitle, $caption);

    // Fallback if performer was empty: extract from caption @botname Title Artist
    // e.g. "@inputfilebot Asal Kau Bahagia Armada" -> Artist: Armada
    if ($itemPerformer === '' && $caption !== '') {
        if (preg_match('/@\w+\s+(.+)$/um', $caption, $botM)) {
            $botLine = trim($botM[1]);
            $botWords = preg_split('/\s+/', $botLine);
            if (count($botWords) >= 2) {
                $candArtist = end($botWords);
                if (mb_strlen($candArtist) >= 3 && !preg_match('/^(?:m4a|mp3|flac|wav|ogg|opus)$/i', $candArtist)) {
                    $itemPerformer = $candArtist;
                }
            }
        }
    }

    if ($itemPerformer !== '') {
        $parsed['artist'] = $itemPerformer;
        // Strip artist name from title if title starts with artist
        $artRegex = '/^' . preg_quote($itemPerformer, '/') . '[\s\-–—:]+/i';
        $parsed['title'] = trim(preg_replace($artRegex, '', $parsed['title']));
    }

    // Safety fallback: if artist still equals title or is empty, try extracting from rawTitle or caption
    if ($parsed['artist'] === '' || strcasecmp($parsed['artist'], $parsed['title']) === 0) {
        if (preg_match('/@\w+\s+(.+)$/um', $caption, $botM)) {
            $botLine = trim($botM[1]);
            $botWords = preg_split('/\s+/', $botLine);
            if (count($botWords) >= 2) {
                $parsed['artist'] = end($botWords);
            }
        }
    }

    // Query-aware split: many Telegram music files are named "Artist Title" with
    // no separator and have an empty performer + a caption that is only a short
    // code. When the search query starts with the parsed title, split the query
    // into artist + title (e.g. query "Michael Jackson Black or White" with
    // title "Michael Jackson Black or White" -> artist "Michael Jackson",
    // title "Black or White").
    if ($searchQuery !== '' && ($parsed['artist'] === '' || strcasecmp($parsed['artist'], $parsed['title']) === 0)) {
        $normTitle = strtolower(preg_replace('/[^\p{L}\p{N}\s]+/u', ' ', $parsed['title']));
        $normTitle = trim(preg_replace('/\s+/', ' ', $normTitle));
        $normQuery = strtolower(preg_replace('/[^\p{L}\p{N}\s]+/u', ' ', $searchQuery));
        $normQuery = trim(preg_replace('/\s+/', ' ', $normQuery));

        if ($normTitle !== '' && $normQuery !== '' && str_starts_with($normQuery, $normTitle)) {
            // The query is "title + extra words" — the extra words are the real
            // title, and the matched prefix is the artist.
            $extra = trim(substr($normQuery, strlen($normTitle)));
            if ($extra !== '') {
                $parsed['artist'] = $parsed['title'];
                $parsed['title'] = ucwords($extra);
            }
        } elseif ($normTitle !== '' && str_starts_with($normTitle, $normQuery)) {
            // The title is "query + extra words" — the query is the artist.
            $extra = trim(substr($normTitle, strlen($normQuery)));
            if ($extra !== '') {
                $parsed['artist'] = $searchQuery;
                $parsed['title'] = ucwords($extra);
            }
        } elseif ($normTitle !== '' && $normTitle === $normQuery) {
            // Title and query are identical ("Michael Jackson Black or White").
            // Split the query itself: the leading words that form a known artist
            // name become the artist, the rest is the title.
            $qWords = preg_split('/\s+/', $normQuery);
            $splitAt = 0;
            // Try the longest leading prefix (up to 3 words) that looks like an
            // artist: it must be followed by at least one more word.
            for ($n = min(3, count($qWords) - 1); $n >= 1; $n--) {
                $cand = implode(' ', array_slice($qWords, 0, $n));
                if (in_array($cand, $knownArtists, true)) {
                    $splitAt = $n;
                    break;
                }
            }
            // Fallback: assume a 2-word artist when the query has 4+ words.
            if ($splitAt === 0 && count($qWords) >= 4) {
                $splitAt = 2;
            }
            if ($splitAt > 0 && $splitAt < count($qWords)) {
                $parsed['artist'] = ucwords(implode(' ', array_slice($qWords, 0, $splitAt)));
                $parsed['title'] = ucwords(implode(' ', array_slice($qWords, $splitAt)));
            }
        }
    }

    // Reversed order: the filename is "Title Artist" (e.g.
    // "Black.or.White.Michael.Jackson.mp3"). Detect a known artist name at the
    // END of the title and move it to the artist field.
    if ($parsed['artist'] === '' || strcasecmp($parsed['artist'], $parsed['title']) === 0) {
        $titleStr = trim((string) $parsed['title']);
        $tWords = $titleStr === '' ? [] : preg_split('/\s+/', $titleStr);
        if (!is_array($tWords)) {
            $tWords = [];
        }
        $tCount = count($tWords);
        if ($tCount >= 3) {
            for ($n = min(3, $tCount - 1); $n >= 1; $n--) {
                $tailWords = array_slice($tWords, $tCount - $n);
                $tail = strtolower(implode(' ', $tailWords));
                if (in_array($tail, $knownArtists, true)) {
                    $parsed['artist'] = ucwords(implode(' ', $tailWords));
                    $parsed['title'] = ucwords(implode(' ', array_slice($tWords, 0, $tCount - $n)));
                    break;
                }
            }
        }
    }
    $duration = (int) ($item['duration'] ?? 0);
    $ext = strtolower((string) ($item['extension'] ?? 'mp3'));
    if ($ext === '' || $ext === 'noext') {
        $ext = 'mp3';
    }

    $track = [
        'id'         => $sc,
        'title'      => $parsed['title'],
        'artist'     => $parsed['artist'],
        'duration'   => $duration,
        'durationMs' => $duration > 0 ? ($duration * 1000) : 180000,
        'format'     => in_array($ext, ['m4a', 'alac'], true) ? 'flac' : $ext,
    ];

    if (!empty($parsed['isrc'])) {
        $track['isrc'] = $parsed['isrc'];
    }

    $thumb = (string) ($item['thumbnail_url'] ?? '');
    if ($thumb !== '') {
        $track['artworkURL'] = $thumb;
    }

    return $track;
}

/**
 * Score audio quality for Eclipse sorting:
 * Prefers lossless (FLAC, WAV) and high bitrate AAC/M4A, plus larger file size.
 */
function fd_eclipse_audio_quality_score(string $ext, int $sizeBytes): int
{
    $extWeight = match (strtolower($ext)) {
        'flac', 'wav' => 2000,
        'm4a', 'alac' => 1200,
        'aac', 'opus' => 800,
        'mp3'         => 400,
        default       => 100,
    };

    // Add up to 500 points for larger files (e.g. 50MB flac gets 500, 4MB mp3 gets 40)
    $sizeBonus = min(500, (int) round($sizeBytes / (100 * 1024)));

    return $extWeight + $sizeBonus;
}

// ─── Routing ─────────────────────────────────────────────────────────────────

$path = parse_url($_SERVER['REQUEST_URI'] ?? '/', PHP_URL_PATH) ?: '/';
$method = $_SERVER['REQUEST_METHOD'] ?? 'GET';

// Strip a /t/<token> prefix so tokenized Stremio/Eclipse URLs match the same
// routes as untokenized ones. The token itself is read by fd_auth_token_from_request().
//
// NOTE: this path form is a fallback only. Caddy's php_server rewrites the URI
// to index.php before PHP runs, so REQUEST_URI is already /index.php by the
// time this executes — the /t/<token> segment is gone. The dashboard therefore
// emits the ?token= query form, which survives the rewrite. Keep this strip for
// non-Caddy runtimes (php -S via router.php) where REQUEST_URI is preserved.
$path = fd_strip_token_prefix($path);

// Run non-blocking cache pruning (cleans expired files, throttled to once every 5 minutes)
fd_prune_cache_files();

// ─── Eclipse Music Addon Routes ───────────────────────────────────────────────
// Supports /eclipse, /eclipse/manifest.json, /eclipse/search, /eclipse/stream/{id}, /eclipse/resolve, /eclipse/resolve-isrc, /eclipse/catalog/{id}
$isEclipseRoute = ($path === '/eclipse' || str_starts_with($path, '/eclipse/'));

if ($isEclipseRoute) {
    fd_log('eclipse route requested', [
        'method' => $method,
        'path'   => $path,
        'uri'    => $_SERVER['REQUEST_URI'] ?? '',
        'query'  => $_GET,
        'ip'     => $_SERVER['REMOTE_ADDR'] ?? '',
        'ua'     => $_SERVER['HTTP_USER_AGENT'] ?? '',
    ]);

    if ($method === 'OPTIONS') {
        header('Access-Control-Allow-Origin: *');
        header('Access-Control-Allow-Methods: GET, OPTIONS');
        header('Access-Control-Allow-Headers: *');
        exit;
    }

    $addonPath = preg_replace('#^/eclipse#', '', $path);
    if ($addonPath === '') {
        $addonPath = '/';
    }

    $baseUrl = fd_get_stremio_base_url();

    // ── Eclipse Addon Installation / Landing Page ──
    if ($addonPath === '/' || $addonPath === '') {
        header('Content-Type: text/html; charset=utf-8');
        $parsedPort = parse_url($baseUrl, PHP_URL_PORT) ?? ($_SERVER['SERVER_PORT'] ?? '8088');
        $portSuffix = ($parsedPort !== '' && $parsedPort !== '80' && $parsedPort !== '443') ? (':' . $parsedPort) : '';
        $localManifestUrl = rtrim($baseUrl, '/') . '/eclipse/manifest.json';
        $lanIp = fd_get_lan_ip();
        $lanManifestUrl = ($lanIp !== '127.0.0.1') ? "http://{$lanIp}{$portSuffix}/eclipse/manifest.json" : $localManifestUrl;
?>
        <!DOCTYPE html>
        <html lang="en">

        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>PencariMovie Eclipse Music Addon</title>
            <style>
                * {
                    box-sizing: border-box;
                    margin: 0;
                    padding: 0;
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
                }

                body {
                    background: #0e0e10;
                    color: #fff;
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    min-height: 100vh;
                    padding: 20px;
                }

                .card {
                    background: #18181b;
                    border: 1px solid #27272a;
                    border-radius: 16px;
                    padding: 32px;
                    max-width: 540px;
                    width: 100%;
                    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.5);
                    text-align: center;
                }

                .badge {
                    display: inline-block;
                    background: #e11d48;
                    color: #fff;
                    font-size: 11px;
                    font-weight: 700;
                    text-transform: uppercase;
                    letter-spacing: 0.05em;
                    padding: 4px 10px;
                    border-radius: 20px;
                    margin-bottom: 16px;
                }

                h1 {
                    font-size: 24px;
                    margin-bottom: 12px;
                    font-weight: 700;
                    color: #fafafa;
                }

                p {
                    color: #a1a1aa;
                    font-size: 14px;
                    line-height: 1.5;
                    margin-bottom: 24px;
                }

                .url-box {
                    background: #09090b;
                    border: 1px solid #27272a;
                    border-radius: 8px;
                    padding: 12px 14px;
                    display: flex;
                    align-items: center;
                    gap: 8px;
                    margin-bottom: 20px;
                    word-break: break-all;
                    text-align: left;
                }

                .url-text {
                    flex: 1;
                    font-family: monospace;
                    font-size: 13px;
                    color: #e4e4e7;
                }

                .btn {
                    background: #e11d48;
                    color: #fff;
                    border: none;
                    padding: 10px 18px;
                    border-radius: 8px;
                    font-weight: 600;
                    font-size: 14px;
                    cursor: pointer;
                    transition: background 0.2s;
                    text-decoration: none;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                }

                .btn:hover {
                    background: #be123c;
                }

                .btn-secondary {
                    background: #27272a;
                    color: #e4e4e7;
                    margin-left: 8px;
                }

                .btn-secondary:hover {
                    background: #3f3f46;
                }

                .instructions {
                    text-align: left;
                    background: #121215;
                    border-radius: 10px;
                    padding: 16px;
                    margin-top: 24px;
                    border: 1px solid #27272a;
                }

                .instructions h3 {
                    font-size: 13px;
                    font-weight: 600;
                    text-transform: uppercase;
                    color: #71717a;
                    margin-bottom: 8px;
                }

                .instructions ol {
                    padding-left: 20px;
                    color: #a1a1aa;
                    font-size: 13px;
                    line-height: 1.6;
                }
            </style>
        </head>

        <body>
            <div class="card">
                <span class="badge">Eclipse Music Addon</span>
                <h1>PencariMusic</h1>
                <p>Stream Telegram music and audio directly inside Eclipse Music.</p>

                <div class="url-box">
                    <span class="url-text" id="manifestUrl"><?php echo htmlspecialchars($localManifestUrl); ?></span>
                    <button class="btn btn-secondary" onclick="navigator.clipboard.writeText(document.getElementById('manifestUrl').innerText); alert('Copied!');">Copy</button>
                </div>

                <div class="instructions">
                    <h3>How to Install in Eclipse</h3>
                    <ol>
                        <li>Open <strong>Eclipse Music</strong> app</li>
                        <li>Go to <strong>Settings</strong> ➔ <strong>Connections</strong> ➔ <strong>Add Connection</strong> ➔ <strong>Addon</strong></li>
                        <li>Paste your Manifest URL above</li>
                        <li>Tap <strong>Install</strong></li>
                    </ol>
                </div>
            </div>
        </body>

        </html>
    <?php
        exit;
    }

    // ── Manifest: /eclipse/manifest.json ──
    if ($addonPath === '/manifest.json') {
        $eclipseManifest = [
            'id'          => 'org.pencarimovie.eclipsemusic',
            'name'        => 'PencariMusic',
            'version'     => '1.0.0',
            'description' => 'Stream Telegram music and audio files via PencariMovie Server',
            'resources'   => ['search', 'stream', 'catalog'],
            'types'       => ['track', 'album', 'artist'],
            'contentType' => 'music',
            'icon'        => 'https://pencarimovie.com/wp-content/uploads/tg-placeholder/document-bl.png?v=1',
            'catalogs'    => [
                [
                    'id'   => 'latest',
                    'type' => 'track',
                    'name' => 'Latest Telegram Tracks',
                ]
            ],
            'settings'    => [
                [
                    'key'     => 'quality',
                    'type'    => 'select',
                    'label'   => 'Audio quality',
                    'default' => 'high',
                    'options' => [
                        ['value' => 'high',   'label' => 'Original / High Quality'],
                        ['value' => 'normal', 'label' => 'Normal Quality'],
                    ]
                ]
            ]
        ];
        fd_stremio_json($eclipseManifest, 200, 'max-age=3600');
    }

    // ── Search: /eclipse/search?q={query} ──
    if ($addonPath === '/search') {
        $rawQ = trim((string) ($_GET['q'] ?? ''));
        // If query is formatted like isrc:USUM71805289, strip prefix
        $q = preg_replace('/^isrc\s*:\s*/i', '', $rawQ);
        $cleanQ = trim($q);
        $tracks = [];
        $seenIds = [];

        $url = FD_WP_API_BASE . '/search-music';
        $params = [
            'search' => $cleanQ,
            'limit'  => 50,
            'offset' => 0,
            // Cache-buster: Cloudflare edge nodes can serve a stale result set
            // for a few minutes after a new track is indexed, which made freshly
            // added tracks "not match" until the edge cache expired.
            't'      => time(),
        ];
        // 15s: the WordPress search-music endpoint regularly takes 4-9s for
        // multi-word queries. A 5s timeout returned an empty result set and the
        // track silently "did not match".
        $res = fd_http_json($url, $params, 'GET', 15);

        $rawItems = [];
        if (!empty($res['items']) && is_array($res['items'])) {
            foreach ($res['items'] as $item) {
                $sc = (string) ($item['short_code'] ?? '');
                if ($sc !== '' && empty($seenIds[$sc])) {
                    $seenIds[$sc] = true;
                    $rawItems[] = $item;
                }
            }
        }

        // Fallback: many Telegram music files are named "Artist - Title" but the
        // indexed caption may only contain the title (older files have no artist
        // line). If the exact query returned few results, retry with progressively
        // narrower title-only queries so the track is still found.
        $queryWords = preg_split('/\s+/', $cleanQ);
        if (count($queryWords) >= 2 && count($rawItems) < 5) {
            $fallbackQueries = [];

            // 1. "Artist - Title" / "Artist – Title" → search the title part only.
            if (preg_match('/^(.+?)\s*[\-–—]\s*(.+)$/u', $cleanQ, $sepM)) {
                $titlePart = trim($sepM[2]);
                if (mb_strlen($titlePart) >= 3) {
                    $fallbackQueries[] = $titlePart;
                }
            }

            // 2. Drop leading words one at a time (artist is usually 1-3 words).
            for ($drop = 1; $drop <= min(3, count($queryWords) - 1); $drop++) {
                $cand = trim(implode(' ', array_slice($queryWords, $drop)));
                // Strip a leading separator left behind by the drop.
                $cand = trim(preg_replace('/^[\-–—]\s*/u', '', $cand));
                if (mb_strlen($cand) >= 3) {
                    $fallbackQueries[] = $cand;
                }
            }

            // 3. Last resort: the longest trailing run of words (the title).
            if (count($queryWords) >= 3) {
                $tail = trim(implode(' ', array_slice($queryWords, -2)));
                if (mb_strlen($tail) >= 3) {
                    $fallbackQueries[] = $tail;
                }
            }

            $fallbackQueries = array_values(array_unique($fallbackQueries));
            foreach ($fallbackQueries as $fq) {
                if (count($rawItems) >= 5) {
                    break;
                }
                $fallbackRes = fd_http_json($url, ['search' => $fq, 'limit' => 20, 'offset' => 0], 'GET', 10);
                if (!empty($fallbackRes['items']) && is_array($fallbackRes['items'])) {
                    foreach ($fallbackRes['items'] as $item) {
                        $sc = (string) ($item['short_code'] ?? '');
                        if ($sc !== '' && empty($seenIds[$sc])) {
                            $seenIds[$sc] = true;
                            $rawItems[] = $item;
                        }
                    }
                }
            }
        }

        // Sort items so highest audio quality (FLAC, M4A, higher bitrate) appears first
        usort($rawItems, function (array $a, array $b) {
            $scoreA = fd_eclipse_audio_quality_score((string) ($a['extension'] ?? 'mp3'), (int) ($a['file_size'] ?? 0));
            $scoreB = fd_eclipse_audio_quality_score((string) ($b['extension'] ?? 'mp3'), (int) ($b['file_size'] ?? 0));
            return $scoreB <=> $scoreA;
        });

        foreach ($rawItems as $item) {
            $tracks[] = fd_eclipse_format_track($item, $baseUrl, $cleanQ);
        }

        fd_stremio_json(['tracks' => $tracks], 200, 'no-cache, no-store, must-revalidate');
    }

    // ── Stream Resolution: /eclipse/stream/{id} ──
    if (preg_match('#^/stream/([^/]+)$#', $addonPath, $m)) {
        if (!fd_is_authenticated()) {
            fd_eclipse_locked_response($baseUrl);
        }
        $shortCode = urldecode($m[1]);
        fd_log('eclipse /stream requested', ['short_code' => $shortCode]);
        // Resolve with active bot
        $activeBotId = fd_get_bot_id();
        $resolved = fd_resolve_shortcode($shortCode, $activeBotId);

        if (empty($resolved['file_id_mt']) && empty($resolved['file_id'])) {
            fd_log('eclipse /stream initial resolve failed, retrying without cache', ['short_code' => $shortCode]);
            $resolved = fd_resolve_shortcode($shortCode, $activeBotId, true);
        }

        if (empty($resolved['file_id_mt']) && empty($resolved['file_id'])) {
            fd_log('eclipse /stream resolve permanently failed', ['short_code' => $shortCode, 'resolved' => $resolved]);
            fd_stremio_json(['error' => 'Unable to resolve file ID for this track.'], 404);
        }

        $fileId = (string) ($resolved['file_id_mt'] ?? $resolved['file_id']);
        $fileSize = (int) ($resolved['file_size'] ?? 0);
        $fileName = (string) ($resolved['title'] ?? 'track.mp3');
        $mime = (string) ($resolved['mime'] ?? 'audio/mpeg');
        $ext = strtolower((string) ($resolved['extension'] ?? 'mp3'));

        $botId = (string) ($resolved['bot_id'] ?? fd_get_bot_id());
        $payload = [
            'short_code' => $shortCode,
            'bot_id'     => $botId,
            'file_id'    => $fileId,
            'file_size'  => $fileSize,
            'file_name'  => $fileName,
            'mime'       => $mime,
        ];
        $payloadB64 = rtrim(strtr(base64_encode(json_encode($payload, JSON_UNESCAPED_SLASHES)), '+/', '-_'), '=');

        // No transcoding: the file is served exactly as stored, so the
        // advertised format and the URL extension must match the source.
        $format = match ($ext) {
            'm4a', 'alac' => 'm4a',
            'aac'         => 'aac',
            'flac'        => 'flac',
            'wav'         => 'wav',
            'ogg', 'opus' => 'ogg',
            default       => 'mp3',
        };

        $safeName = fd_stremio_stream_filename($fileName, $mime);
        // Force audio extension if not set
        if (!preg_match('/\.(mp3|flac|m4a|aac|wav|ogg|opus)$/i', $safeName)) {
            $safeName .= '.' . ($ext !== '' ? $ext : 'mp3');
        }

        // Stream URL should preserve the base host Eclipse used to contact the server (e.g. LAN IP
        // when playing from an Android client on Wi-Fi, or 127.0.0.1 when running locally).
        $streamUrl = fd_build_stremio_stream_url($baseUrl, $payloadB64, $safeName, $mime);

        $resp = [
            'url'           => $streamUrl,
            'format'        => $format,
            'quality'       => in_array($format, ['flac', 'wav'], true) ? 'LOSSLESS' : '320kbps',
            'streamQuality' => in_array($format, ['flac', 'wav'], true) ? 'LOSSLESS' : 'HIGH',
            'audioQuality'  => in_array($format, ['flac', 'wav'], true) ? 'LOSSLESS' : 'HIGH',
            'bitrate'       => in_array($format, ['flac', 'wav'], true) ? 1411200 : 320000,
            'sampleRate'    => 48000,
            'bitDepth'      => 24,
        ];

        fd_stremio_json($resp, 200, 'max-age=300');
    }

    // ── Direct Audio Stream Link: /eclipse/play/{id} ──
    if (preg_match('#^/play/([^/]+)$#', $addonPath, $m)) {
        $shortCode = urldecode($m[1]);
        $resolved = fd_resolve_shortcode($shortCode);

        if (empty($resolved['file_id_mt']) && empty($resolved['file_id'])) {
            http_response_code(404);
            echo "Track not found";
            exit;
        }

        $fileId = (string) ($resolved['file_id_mt'] ?? $resolved['file_id']);
        $fileSize = (int) ($resolved['file_size'] ?? 0);
        $fileName = (string) ($resolved['title'] ?? 'track.mp3');
        $mime = (string) ($resolved['mime'] ?? 'audio/mpeg');
        $ext = strtolower((string) ($resolved['extension'] ?? 'mp3'));

        $botId = (string) ($resolved['bot_id'] ?? fd_get_bot_id());
        $payload = [
            'short_code' => $shortCode,
            'bot_id'     => $botId,
            'file_id'    => $fileId,
            'file_size'  => $fileSize,
            'file_name'  => $fileName,
            'mime'       => $mime,
        ];
        $payloadB64 = rtrim(strtr(base64_encode(json_encode($payload, JSON_UNESCAPED_SLASHES)), '+/', '-_'), '=');
        $safeName = fd_stremio_stream_filename($fileName, $mime);
        if (!preg_match('/\.(mp3|flac|m4a|aac|wav|ogg|opus)$/i', $safeName)) {
            $safeName .= '.' . ($ext !== '' ? $ext : 'mp3');
        }
        $effectivePlayBase = $baseUrl;
        if (preg_match('#^https?://(?:192\.168\.|10\.|172\.(?:1[6-9]|2\d|3[01])\.)#', $baseUrl)) {
            $parsedPort = parse_url($baseUrl, PHP_URL_PORT) ?? ($_SERVER['SERVER_PORT'] ?? '8088');
            $portSuffix = ($parsedPort !== '' && $parsedPort !== '80' && $parsedPort !== '443') ? (':' . $parsedPort) : ':8088';
            $effectivePlayBase = "http://127.0.0.1{$portSuffix}";
        }
        $targetUrl = fd_build_stremio_stream_url($effectivePlayBase, $payloadB64, $safeName, $mime);
        header('Location: ' . $targetUrl, true, 302);
        exit;
    }

    // ── Catalog: /eclipse/catalog/{id} ──
    if (preg_match('#^/catalog/([^/]+)$#', $addonPath, $m)) {
        $catId = $m[1];
        $skip = max(0, (int) ($_GET['skip'] ?? 0));

        $url = FD_WP_API_BASE . '/search-music';
        $params = [
            'search' => '',
            'limit'  => 50,
            'offset' => $skip,
        ];
        $res = fd_http_json($url, $params, 'GET', 5);

        $items = [];
        if (!empty($res['items']) && is_array($res['items'])) {
            foreach ($res['items'] as $item) {
                $t = fd_eclipse_format_track($item, $baseUrl);
                $durationMs = ($t['duration'] ?? 0) * 1000;
                $items[] = [
                    'id'         => $t['id'],
                    'type'       => 'track',
                    'title'      => $t['title'],
                    'artist'     => $t['artist'],
                    'durationMs' => $durationMs,
                    'artworkURL' => $t['artworkURL'] ?? '',
                ];
            }
        }

        fd_stremio_json(['items' => $items], 200, 'max-age=120');
    }

    // ── Resolve (Smart Shuffle / Radio): /eclipse/resolve ──
    if ($addonPath === '/resolve') {
        $title = trim((string) ($_GET['title'] ?? ''));
        $artist = trim((string) ($_GET['artist'] ?? ''));
        fd_log('eclipse /resolve requested', ['title' => $title, 'artist' => $artist]);

        // Clean query terms
        $cleanArtist = trim((string) preg_replace('/\b(?:unknown\s*artist)\b/i', '', $artist));
        $cleanTitle = trim((string) preg_replace('/[^\p{L}\p{N}\s]+/u', ' ', $title));
        $cleanTitle = trim((string) preg_replace('/\s+/', ' ', $cleanTitle));

        $queries = [];
        if ($cleanArtist !== '' && $cleanTitle !== '') {
            $queries[] = "{$cleanArtist} {$cleanTitle}";
        }
        if ($cleanTitle !== '') {
            $queries[] = $cleanTitle;
        }

        foreach ($queries as $q) {
            $url = FD_WP_API_BASE . '/search-music';
            $params = [
                'search' => $q,
                'limit'  => 20,
                'offset' => 0,
            ];
            $res = fd_http_json($url, $params, 'GET', 5);
            if (!empty($res['items']) && is_array($res['items'])) {
                $candidates = $res['items'];
                // Sort candidates so highest quality files (FLAC, M4A, higher bitrate) come first
                usort($candidates, function (array $a, array $b) {
                    $scoreA = fd_eclipse_audio_quality_score((string) ($a['extension'] ?? 'mp3'), (int) ($a['file_size'] ?? 0));
                    $scoreB = fd_eclipse_audio_quality_score((string) ($b['extension'] ?? 'mp3'), (int) ($b['file_size'] ?? 0));
                    return $scoreB <=> $scoreA;
                });

                // Find best matching item
                foreach ($candidates as $cand) {
                    $candTitle = (string) ($cand['title'] ?? '');
                    // Basic sanity check: ensure title words match
                    if ($cleanTitle !== '') {
                        $titleWords = explode(' ', strtolower($cleanTitle));
                        $candLower = strtolower($candTitle);
                        $matchedWords = 0;
                        foreach ($titleWords as $tw) {
                            if (strlen($tw) > 2 && str_contains($candLower, $tw)) {
                                $matchedWords++;
                            }
                        }
                        if ($matchedWords === 0 && count($titleWords) > 1) {
                            continue;
                        }
                    }

                    $t = fd_eclipse_format_track($cand, $baseUrl);
                    fd_log('eclipse /resolve found item', ['short_code' => $t['id'], 'title' => $t['title'], 'artist' => $t['artist']]);
                    fd_stremio_json([
                        'item' => [
                            'id'     => $t['id'],
                            'type'   => 'track',
                            'title'  => $t['title'],
                            'artist' => $t['artist'],
                        ]
                    ]);
                }
            }
        }

        fd_log('eclipse /resolve returned null', ['title' => $title, 'artist' => $artist]);
        fd_stremio_json(['item' => null], 200);
    }

    // ── Resolve ISRC: /eclipse/resolve-isrc ──
    if ($addonPath === '/resolve-isrc') {
        fd_log('eclipse /resolve-isrc requested', ['isrc' => $_GET['isrc'] ?? '']);
        // Our Telegram database doesn't have official ISRC columns, so return null
        fd_stremio_json(['trackId' => null], 200);
    }

    fd_stremio_json(['error' => 'Unknown Eclipse route'], 404);
}

// ─── Nuvio Addon Routes ──────────────────────────────────────────────────────
// Support /nuvio, /stremio (alias redirect), /configure, and root level (/manifest.json, /catalog/..., /meta/..., /stream/...)
$isNuvioRoute = ($path === '/nuvio' || str_starts_with($path, '/nuvio/')) ||
    ($path === '/stremio' || str_starts_with($path, '/stremio/')) ||
    $path === '/configure' || $path === '/configure/' ||
    $path === '/manifest.json' ||
    preg_match('#^/(catalog|meta|stream|subtitles)/#', $path);

if ($isNuvioRoute) {
    // Handle redirect for legacy /stremio to /nuvio
    if ($path === '/stremio' || $path === '/stremio/') {
        header('Location: /nuvio', true, 301);
        exit;
    }

    // Normalize path by stripping /nuvio or /stremio prefix if present so internal matching is uniform
    $addonPath = preg_replace('#^/(nuvio|stremio)#', '', $path);
    if ($addonPath === '') {
        $addonPath = '/';
    }

    // Handle Stremio's standard /configure route -> redirects directly to dashboard with #configure
    if ($path === '/configure' || $path === '/configure/' || $addonPath === '/configure' || $addonPath === '/configure/') {
        header('Location: /#addon', true, 302);
        exit;
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
        // $randSuffix = '?r=' . random_int(100000, 999999);
        $manifestUrl = $baseUrl . '/manifest.json';
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
        // Fetch genre list from WordPress (matching public/app.js).
        // Cached to disk: this is a blocking remote call (12s timeout) and
        // /manifest.json is fetched by every client on every install/refresh.
        // Without the cache, N concurrent manifest requests each open their own
        // WordPress connection and TTFB stacks linearly (measured 1.5s -> 10.3s).
        $categories = fd_manifest_categories_cached();
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
        $kwMap = fd_get_trending_keywords_map();
        foreach ($kwMap as $catId => $kw) {
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
        }

        // Determine active types based on enabled catalogs/types
        $enabledTypes = $catSettings['enabled_types'] ?? ['movie' => true, 'series' => true, 'other' => true];
        $activeTypes = [];
        if (!empty($enabledTypes['movie'])) $activeTypes[] = 'movie';
        if (!empty($enabledTypes['series'])) $activeTypes[] = 'series';
        if (!empty($enabledTypes['other'])) $activeTypes[] = 'other';
        if (empty($activeTypes)) $activeTypes = ['movie', 'series', 'other'];

        // Import and bridge catalogs from configured upstream manifests INDEPENDENTLY of local catalogs toggle
        $configuredUpstreams = (array) ($catSettings['upstream_manifests'] ?? []);
        foreach ($configuredUpstreams as $uIdx => $upstream) {
            $mUrl = trim((string)($upstream['url'] ?? ''));
            if ($mUrl === '') continue;
            $mId = trim((string)($upstream['id'] ?? ('up' . $uIdx)));
            $mName = trim((string)($upstream['name'] ?? 'Addon'));

            // Cache upstream manifest JSON on disk for 2 hours
            $mCacheFile = fd_storage_path('storage/upstream_manifest_' . md5($mUrl) . '.json');
            $mJson = null;
            if (is_file($mCacheFile) && (time() - (int)filemtime($mCacheFile)) < 7200) {
                $mJson = json_decode((string)@file_get_contents($mCacheFile), true);
            }
            if (!is_array($mJson) || empty($mJson['catalogs'])) {
                $mJson = fd_http_json($mUrl, [], 'GET', 6);
                if (is_array($mJson) && !empty($mJson['catalogs'])) {
                    @file_put_contents($mCacheFile, json_encode($mJson, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE), LOCK_EX);
                }
            }

            if (is_array($mJson) && !empty($mJson['catalogs']) && is_array($mJson['catalogs'])) {
                foreach ($mJson['catalogs'] as $uCat) {
                    if (!is_array($uCat) || empty($uCat['type']) || empty($uCat['id'])) continue;
                    $uType = $uCat['type'];
                    if (!in_array($uType, $activeTypes, true)) {
                        $activeTypes[] = $uType;
                    }

                    // Namespace catalog ID to avoid collisions: up_{index}_{id}
                    $bridgeCatId = 'up_' . $uIdx . '_' . $uCat['id'];
                    $bridgeCat = $uCat;
                    $bridgeCat['id'] = $bridgeCatId;
                    $bridgeCat['name'] = ($uCat['name'] ?? 'Catalog') . " ({$mName})";
                    $filteredCatalogs[] = $bridgeCat;
                }
            }
        }

        if (!empty($filteredCatalogs)) {
            $resources[] = [
                'name' => 'catalog',
                'types' => $activeTypes,
            ];
        }
        $resources[] = [
            'name' => 'meta',
            'types' => $activeTypes,
            'idPrefixes' => ['pm_', 'pm:', 'tt', 'tmdb:', 'kitsu:', 'kitsu', 'mal:', 'anilist:', 'tvdb:'],
        ];
        $resources[] = [
            'name' => 'stream',
            'types' => ['movie', 'series', 'other'],
            'idPrefixes' => ['pm_', 'pm:', 'tt', 'tmdb:', 'kitsu:', 'kitsu', 'mal:', 'anilist:', 'tvdb:'],
        ];
        $resources[] = [
            'name' => 'subtitles',
            'types' => ['movie', 'series'],
            'idPrefixes' => ['pm_', 'pm:', 'tt', 'tmdb:', 'kitsu:', 'kitsu', 'mal:', 'anilist:', 'tvdb:'],
        ];


        $manifest = [
            'id' => $identity['id'],
            'version' => FD_APP_VERSION,
            'name' => $identity['name'],
            'description' => $identity['description'],
            'resources' => $resources,
            'types' => ['movie', 'series', 'other'],
            'idPrefixes' => ['pm_', 'pm:', 'tt', 'tmdb:', 'kitsu:', 'kitsu', 'mal:', 'anilist:', 'tvdb:'],
            'catalogs' => $filteredCatalogs,
            'behaviorHints' => [
                'configurable' => true,
                'configurationRequired' => false,
                'adult' => false,
                'p2p' => false,
            ],
        ];

        fd_stremio_json($manifest, 200, 'no-cache, no-store, must-revalidate');
    }

    // ── Nuvio Catalog: /catalog/:type/:id[/:extra].json ──
    if (preg_match('#^/catalog/([^/]+)/([^/]+?)(?:/(.*))?\.json$#', $addonPath, $matches)) {
        $catalogType = $matches[1];
        $catalogId = $matches[2];
        $extraStr = $matches[3] ?? '';

        // Bridge: Check if this is an upstream bridged catalog (prefixed with up_{index}_)
        if (preg_match('/^up_(\d+)_(.+)$/', $catalogId, $upMatch)) {
            $uIdx = (int) $upMatch[1];
            $realCatId = $upMatch[2];
            $catSettings = fd_load_catalog_settings();
            $configuredUpstreams = (array) ($catSettings['upstream_manifests'] ?? []);
            if (isset($configuredUpstreams[$uIdx])) {
                $mUrl = trim((string)($configuredUpstreams[$uIdx]['url'] ?? ''));
                if ($mUrl !== '') {
                    $baseAddonUrl = preg_replace('#/manifest\.json(\?.*)?$#i', '', $mUrl);
                    $forwardPath = "/catalog/{$catalogType}/" . urlencode($realCatId);
                    if ($extraStr !== '') {
                        $forwardPath .= '/' . $extraStr;
                    }
                    $forwardUrl = rtrim($baseAddonUrl, '/') . $forwardPath . '.json';
                    if (!empty($_SERVER['QUERY_STRING'])) {
                        $forwardUrl .= '?' . $_SERVER['QUERY_STRING'];
                    }

                    $cacheKey = md5($forwardUrl);
                    $cacheFile = fd_storage_path('storage/up_cat_' . $cacheKey . '.json');
                    if (is_file($cacheFile) && (time() - (int)filemtime($cacheFile)) < 600) {
                        $cachedData = json_decode((string)@file_get_contents($cacheFile), true);
                        if (is_array($cachedData) && isset($cachedData['metas'])) {
                            fd_stremio_json($cachedData, 200, 'max-age=600, public');
                        }
                    }

                    $upRes = fd_http_json($forwardUrl, [], 'GET', 8);
                    if (is_array($upRes) && isset($upRes['metas'])) {
                        @file_put_contents($cacheFile, json_encode($upRes, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE), LOCK_EX);
                        fd_stremio_json($upRes, 200, 'max-age=600, public');
                    }
                }
            }
            fd_stremio_json(['metas' => []]);
        }

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
                        'description' => fd_clean_post_plot($pExcerpt),
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
                        $fTitle = fd_clean_html_entities((string) ($file['title'] ?? 'Telegram File'));
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
                    $kwMap = fd_get_trending_keywords_map();
                    if (isset($kwMap[$catalogId])) {
                        $keywordToSearch = $kwMap[$catalogId];
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
                    $fTitle = fd_clean_html_entities((string) ($file['title'] ?? 'Telegram File'));
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
                        'description' => fd_clean_post_plot($pExcerpt),
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
                'description' => fd_clean_post_plot((string) $excerpt),
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

        // ── Meta Bridge: If not a local pm_ ID, proxy from upstream manifests or Cinemeta ──
        if (!str_starts_with($itemId, 'pm_') && !str_starts_with($itemId, 'pm:')) {
            // Check meta disk cache (1 hour TTL)
            $metaCacheFile = fd_storage_path('storage/up_meta_' . md5($itemType . '_' . $itemId) . '.json');
            if (is_file($metaCacheFile) && (time() - (int)filemtime($metaCacheFile)) < 3600) {
                $cachedMeta = json_decode((string)@file_get_contents($metaCacheFile), true);
                if (is_array($cachedMeta) && isset($cachedMeta['meta'])) {
                    fd_stremio_json($cachedMeta, 200, 'max-age=3600, public');
                }
            }

            // 1. Try upstream manifests first
            $catSettings = fd_load_catalog_settings();
            $configuredUpstreams = (array) ($catSettings['upstream_manifests'] ?? []);
            foreach ($configuredUpstreams as $upstream) {
                $mUrl = trim((string)($upstream['url'] ?? ''));
                if ($mUrl === '') continue;
                $baseAddonUrl = preg_replace('#/manifest\.json(\?.*)?$#i', '', $mUrl);
                $upMetaUrl = rtrim($baseAddonUrl, '/') . "/meta/{$itemType}/" . rawurlencode($itemId) . ".json";
                $mRes = fd_http_json($upMetaUrl, [], 'GET', 6);
                if (!empty($mRes['meta']) && is_array($mRes['meta']) && !empty($mRes['meta']['name'])) {
                    @file_put_contents($metaCacheFile, json_encode($mRes, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE), LOCK_EX);
                    fd_stremio_json($mRes, 200, 'max-age=3600, public');
                }
            }

            // 2. Fallback to Cinemeta for IMDb IDs (tt...)
            if (preg_match('/^(tt\d{6,10})/i', $itemId, $tm)) {
                $ttId = $tm[1];
                $cineType = ($itemType === 'series') ? 'series' : 'movie';
                $cineUrl = "https://v3-cinemeta.strem.io/meta/{$cineType}/{$ttId}.json";
                $cRes = fd_http_json($cineUrl, [], 'GET', 6);
                if (!empty($cRes['meta']) && is_array($cRes['meta'])) {
                    @file_put_contents($metaCacheFile, json_encode($cRes, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE), LOCK_EX);
                    fd_stremio_json($cRes, 200, 'max-age=3600, public');
                }
            }

            // 3. Fallback to Kitsu for Anime (kitsu:...)
            if (preg_match('/^kitsu:(\d+)/i', $itemId, $km)) {
                $kId = $km[1];
                $kitsuUrl = "https://anime-kitsu.strem.fun/meta/anime/kitsu:{$kId}.json";
                $kRes = fd_http_json($kitsuUrl, [], 'GET', 6);
                if (!empty($kRes['meta']) && is_array($kRes['meta'])) {
                    @file_put_contents($metaCacheFile, json_encode($kRes, JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE), LOCK_EX);
                    fd_stremio_json($kRes, 200, 'max-age=3600, public');
                }
            }
        }

        fd_log('stremio meta not found', [
            'itemType' => $itemType,
            'itemId' => $itemId,
        ]);
        fd_stremio_json(['meta' => null], 404);
    }

    // ── Nuvio Stream: /stream/:type/:id.json ──
    if (preg_match('#^/stream/([^/]+)/([^/]+?)(?:\.json)?$#', $addonPath, $matches)) {
        if (!fd_is_authenticated()) {
            fd_stremio_locked_stream($baseUrl);
        }
        $streamStart = microtime(true);
        $itemType = urldecode($matches[1]);
        $itemId = urldecode(urldecode($matches[2])); // Handle double-encoded IDs from web clients

        fd_log('stremio stream request received', [
            'itemType' => $itemType,
            'itemId' => $itemId,
            'clientIp' => $_SERVER['REMOTE_ADDR'] ?? '',
            'userAgent' => $_SERVER['HTTP_USER_AGENT'] ?? '',
        ]);

        // Check 5-minute stream list cache (per itemId and baseUrl to differentiate tunnel vs local).
        // The auth state is part of the key so an unauthenticated request can never
        // be served a cached authenticated response (and vice versa).
        $streamCacheKey = md5($itemId . ':' . $itemType . ':' . $baseUrl . ':' . (fd_is_authenticated() ? 'auth' : 'anon'));
        $streamCacheFile = fd_cache_path('stream_cache_' . $streamCacheKey . '.json');
        if (is_file($streamCacheFile) && (time() - (int)filemtime($streamCacheFile)) < 300) {
            $cachedJson = @file_get_contents($streamCacheFile);
            if ($cachedJson) {
                $cachedData = json_decode($cachedJson, true);
                if (is_array($cachedData) && isset($cachedData['streams'])) {
                    fd_log('stremio stream served from cache', [
                        'itemId' => $itemId,
                        'streamCount' => count($cachedData['streams']),
                        'duration_seconds' => round(microtime(true) - $streamStart, 4),
                    ]);
                    fd_stremio_json($cachedData, 200, 'max-age=300, public');
                }
            }
        }

        // Pre-fetch subtitles for this item so they can be embedded directly in stream objects
        // and served to external Stremio players supporting stream.subtitles
        $subtitlesForStream = fd_get_item_subtitles($itemType, $itemId);

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
            if (fd_is_guest_provision_in_progress()) {
                // Server is actively provisioning a guest bot session in the background
                $streams[] = [
                    'name' => 'PencariMovie',
                    'description' => "Connecting guest bot in progress...\nPlease refresh or try again in a few seconds.",
                    'externalUrl' => $baseUrl . '/#settings'
                ];
                fd_stremio_json(['streams' => $streams], 200, 'no-cache, no-store, must-revalidate');
            }

            $autoProv = fd_auto_provision_guest();
            if ($autoProv && !empty($autoProv['bot_id'])) {
                $hasSession = true;
                $botIdStr = (string) $autoProv['bot_id'];
            } else {
                // Surface the real reason instead of a generic "not connected".
                // A clock-skew error is actionable and must be shown verbatim so
                // the user knows to enable NTP; otherwise they only see a vague
                // "Telegram bot not connected" and cannot fix anything.
                $provErr = trim((string) ($autoProv['error'] ?? ''));
                if ($provErr !== '') {
                    $streams[] = [
                        'name' => 'PencariMovie',
                        'description' => $provErr,
                        'externalUrl' => $baseUrl . '/#settings',
                    ];
                    fd_stremio_json(['streams' => $streams], 200, 'no-cache, no-store, must-revalidate');
                }
                if (fd_is_guest_provision_in_progress()) {
                    $streams[] = [
                        'name' => 'PencariMovie',
                        'description' => "Connecting guest bot in progress...\nPlease refresh or try again in a few seconds.",
                        'externalUrl' => $baseUrl . '/#settings'
                    ];
                    fd_stremio_json(['streams' => $streams], 200, 'no-cache, no-store, must-revalidate');
                } else {
                    $streams[] = [
                        'name' => 'PencariMovie',
                        'description' => "Telegram bot not connected\nOpen Settings and paste a bot token",
                        'externalUrl' => $baseUrl . '/#settings'
                    ];
                    fd_stremio_json(['streams' => $streams]);
                }
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
                        $fRawMime = (string) ($f['mime'] ?? ($f['mime_type'] ?? ''));
                        $fRawType = (string) ($f['file_type'] ?? '');
                        $fileObj['mime'] = fd_guess_video_mime($fileObj['title'], $fRawMime !== '' ? $fRawMime : $fRawType);
                        break;
                    }
                }
            }

            if (empty($fileObj['title']) || empty($fileObj['file_size'])) {
                $res = fd_resolve_shortcode($fCode, $botIdStr);
                if (!empty($res['title'])) $fileObj['title'] = (string) $res['title'];
                if (!empty($res['file_size'])) $fileObj['file_size'] = (int) $res['file_size'];
                $rRawMime = (string) ($res['mime'] ?? ($res['mime_type'] ?? ''));
                $rRawType = (string) ($res['file_type'] ?? '');
                $fileObj['mime'] = fd_guess_video_mime($fileObj['title'] ?? '', $rRawMime !== '' ? $rRawMime : $rRawType);
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

            // Sort episode streams using comprehensive media ranking:
            // Quality (REMUX > BluRay/BDRip/BBRip > WEB-DL > WEBRip > HDRip > HDTV > DVDRip > CAM/TS)
            // Resolution (4K > 1080p > 720p > 480p > 360p), Visual, Codec, and file size
            usort($filesToStream, function ($a, $b) {
                $sA = fd_calculate_stream_sort_score((string)($a['title'] ?? ''), (string)($a['caption'] ?? ''), (int)($a['file_size'] ?? 0));
                $sB = fd_calculate_stream_sort_score((string)($b['title'] ?? ''), (string)($b['caption'] ?? ''), (int)($b['file_size'] ?? 0));
                return $sB <=> $sA;
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

                $cleanTitle = preg_replace('/\s*[•··]\s*.+$/u', '', fd_clean_html_entities($postTitle));
                $cleanTitle = preg_replace('/\b(?:2160p|1080p|720p|480p|360p|uhd|fhd|hd|sd|hdtv|web-?dl|webrip|bluray|blu-ray|remux|dvdrip|hevc|x264|x265|h264|h265|\d+(?:\.\d+)?\s*(?:gb|mb))\b/i', ' ', $cleanTitle);
                if ($postYear) {
                    $cleanTitle = preg_replace('/\b' . $postYear . '\b/', '', $cleanTitle);
                }
                $cleanTitle = trim(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', $cleanTitle));
                $cleanTitle = trim(preg_replace('/\s+/', ' ', $cleanTitle));
                $postWords = array_values(array_filter(explode(' ', strtolower($cleanTitle)), fn($w) => strlen($w) > 1 && !in_array($w, ['dan', 'and', 'the'], true)));

                $matchedFiles = [];
                foreach ($postFiles as $pf) {
                    if (empty($pf['short_code'])) continue;
                    $fTitle = fd_clean_html_entities((string) ($pf['title'] ?? ''));

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

                    // Strict title word match (supports stem/plural variants e.g. selina vs selinas vs selina's)
                    $cleanFTitle = strtolower(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', $fTitle));
                    $cleanFTitleCompact = str_replace(' ', '', $cleanFTitle);
                    $wordsMatch = true;
                    foreach ($postWords as $pw) {
                        $pwStem = rtrim($pw, 's');
                        $hasMatch = str_contains($cleanFTitle, $pw)
                            || (strlen($pwStem) >= 4 && str_contains($cleanFTitle, $pwStem))
                            || str_contains($cleanFTitleCompact, $pw);
                        if (!$hasMatch) {
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
                $seenCodes = [];
                $postFilesToUse = !empty($matchedFiles) ? $matchedFiles : $postFiles;
                foreach ($postFilesToUse as $pf) {
                    if (!empty($pf['short_code']) && !isset($seenCodes[$pf['short_code']])) {
                        $seenCodes[$pf['short_code']] = true;
                        $filesToStream[] = $pf;
                    }
                }

                // Also supplement with search_files variants (e.g. "selina s gold" vs "selinas gold" or "gol & gincu" vs "gol dan gincu")
                // to make sure all available formats and release variants in Manticore are found!
                if (!empty($cleanTitle)) {
                    $searchVariants = fd_build_search_query_variants($postTitle, $postYear ?? '');
                    foreach ($searchVariants as $sv) {
                        $sf = fd_fetch_stream_ajax('search_files', ['search' => $sv, 'limit' => 30]);
                        if (is_array($sf) && !empty($sf['files'])) {
                            foreach ($sf['files'] as $f) {
                                if (empty($f['short_code']) || isset($seenCodes[$f['short_code']])) continue;
                                $fTitle = fd_clean_html_entities((string) ($f['title'] ?? ''));

                                if (preg_match('/[sS]\d{1,2}\s*[eE]\d{1,2}|(?:season|episod|episode|ep\.)\s*\d+/i', $fTitle)) {
                                    continue;
                                }

                                if ($postYear !== null && preg_match('/\b(19\d\d|20\d\d)\b/', $fTitle, $fym)) {
                                    if ($fym[1] !== $postYear) {
                                        continue;
                                    }
                                }

                                $cleanF = strtolower(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', $fTitle));
                                $cleanFCompact = str_replace(' ', '', $cleanF);
                                $allWords = true;
                                foreach ($postWords as $pw) {
                                    $pwStem = rtrim($pw, 's');
                                    $hasMatch = str_contains($cleanF, $pw)
                                        || (strlen($pwStem) >= 4 && str_contains($cleanF, $pwStem))
                                        || str_contains($cleanFCompact, $pw);
                                    if (!$hasMatch) {
                                        $allWords = false;
                                        break;
                                    }
                                }
                                if ($allWords) {
                                    $seenCodes[$f['short_code']] = true;
                                    $filesToStream[] = $f;
                                }
                            }
                        }
                    }
                }
            } else {
                foreach ($postFiles as $pf) {
                    if (!empty($pf['short_code'])) {
                        $filesToStream[] = $pf;
                    }
                }
            }
        } elseif (
            preg_match('/^tt\d{6,10}/i', $itemId) ||
            preg_match('/^(?:tmdb|kitsu|mal|anilist|tvdb):/i', $itemId) ||
            preg_match('/^[a-zA-Z0-9_-]+:\d+/i', $itemId)
        ) {
            // External standard Stremio catalog ID requested (e.g. IMDb tt1234567, TMDB tmdb:1108427, Kitsu kitsu:1, etc.)
            $resolvedMeta = fd_resolve_external_media_metadata($itemId, $itemType);
            $searchedTitle = (string) ($resolvedMeta['title'] ?? '');
            $searchedYear = (string) ($resolvedMeta['year'] ?? '');
            $targetSeason = $resolvedMeta['season'] ?? null;
            $targetEpisode = $resolvedMeta['episode'] ?? null;
            $imdbId = (string) ($resolvedMeta['imdb_id'] ?? '');

            // Query search index with resolved title
            $searchQuery = $searchedTitle !== '' ? $searchedTitle : $itemId;

            if ($searchQuery !== '') {
                if ($targetSeason !== null && $targetEpisode !== null) {
                    $imdbEpisodeFilter = fd_episode_stream_filter($targetSeason, $targetEpisode);

                    // 1. Exact Series Post Match: Try matching the title with year first (e.g. "Glory 2025")
                    // This targets the exact post in 1 query instead of looping over 5 unrelated posts!
                    $queriesToSearch = fd_build_search_query_variants($searchedTitle !== '' ? $searchedTitle : $searchQuery, $searchedYear);

                    $matchedPostId = null;
                    $matchedPostData = null;
                    foreach ($queriesToSearch as $sq) {
                        $sp = fd_fetch_stream_ajax('search', ['search' => $sq, 'limit' => 5]);
                        if (!is_array($sp) || empty($sp)) {
                            continue;
                        }

                        // Pass 1: exact year match
                        if ($searchedYear !== '') {
                            foreach ($sp as $p) {
                                $pTitle = (string) ($p['title'] ?? '');
                                $pId = $p['id'] ?? 0;
                                if (!$pId) continue;
                                if (preg_match('/\b(19\d\d|20\d\d)\b/', $pTitle, $ym) && $ym[1] === $searchedYear) {
                                    $matchedPostId = (int) $pId;
                                    $matchedPostData = $p;
                                    break 2;
                                }
                            }
                        }

                        // If no exact year match yet and we searched with year, try raw title query next
                        if ($matchedPostId === null && $sq === "{$searchedTitle} {$searchedYear}") {
                            continue;
                        }

                        // Pass 2: best candidate from search results
                        if ($matchedPostId === null) {
                            foreach ($sp as $p) {
                                $pTitle = (string) ($p['title'] ?? '');
                                $pId = $p['id'] ?? 0;
                                if (!$pId) continue;
                                // If searchedYear is known, avoid picking a post from a completely different year
                                if ($searchedYear !== '' && preg_match('/\b(19\d\d|20\d\d)\b/', $pTitle, $ym) && $ym[1] !== $searchedYear) {
                                    continue;
                                }
                                $matchedPostId = (int) $pId;
                                $matchedPostData = $p;
                                break 2;
                            }
                        }
                    }

                    if ($matchedPostId !== null) {
                        $filesToStream = fd_fetch_episode_stream_files($matchedPostId, $targetSeason, $targetEpisode, 40, $matchedPostData);
                    }

                    // Fallback: If no post matched or post returned 0 files, probe search_files directly
                    if (count($filesToStream) === 0) {
                        $epQuery = sprintf('%s S%02dE%02d', $searchQuery, $targetSeason, $targetEpisode);
                        $sf = fd_fetch_stream_ajax('search_files', ['search' => $epQuery, 'limit' => 30]);
                        if (is_array($sf) && !empty($sf['files'])) {
                            foreach ($sf['files'] as $f) {
                                if ($imdbEpisodeFilter($f)) {
                                    $filesToStream[] = $f;
                                }
                            }
                        }
                    }
                } else {
                    // For Movie: search direct files and posts using all query variants (handling &, dan, and, entities)
                    $queriesToTry = fd_build_search_query_variants($searchedTitle !== '' ? $searchedTitle : $searchQuery, $searchedYear);

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

                    // Strict Movie Title & Year Guard: prevent cross-matching different titles
                    // (e.g. "Runner 2026" vs "The Runner 2026", "Late Runner 2026", "Blade Runner 2049")
                    if (!empty($filesToStream) && $searchedTitle !== '') {
                        $cleanSearched = trim(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', $searchedTitle));
                        $cleanSearched = trim(preg_replace('/\s+/', ' ', $cleanSearched));
                        $searchedLower = strtolower($cleanSearched);
                        $hasLeadingThe = str_starts_with($searchedLower, 'the ');

                        $filteredMovieFiles = [];
                        foreach ($filesToStream as $mf) {
                            $fTitle = $mf['title'] ?? '';

                            // 1. Strict Year check if year is present in filename
                            if ($searchedYear !== '' && preg_match('/\b(19\d\d|20\d\d)\b/', $fTitle, $fym)) {
                                if ($fym[1] !== $searchedYear) {
                                    continue;
                                }
                            }

                            // 2. Normalize filename to extract the movie title portion before release tags/year
                            $cleanF = strtolower(preg_replace('/[^\p{L}\p{N}\s]/u', ' ', $fTitle));
                            $cleanF = trim(preg_replace('/\s+/', ' ', $cleanF));
                            // Strip release tags (1080p, bluray, etc.)
                            $baseF = preg_replace('/\b(2160p|1080p|720p|480p|360p|4k|fhd|hd|sd|bluray|web-?dl|webrip|hdrip|hdtv|cam|hevc|x264|x265|aac.*|lubokvideo|yts|lulustream)\b.*/i', '', $cleanF);
                            // Strip year and anything following
                            if ($searchedYear !== '') {
                                $baseF = preg_replace('/\b' . $searchedYear . '\b.*/', '', $baseF);
                            } else {
                                $baseF = preg_replace('/\b(19\d\d|20\d\d)\b.*/', '', $baseF);
                            }
                            $baseF = trim($baseF);

                            // Strip leading release channels/groups (e.g. "prakytv the runner" -> "the runner")
                            $baseF = preg_replace('/^(?:prakytv|ngefilm\s*store|runningmovieshd|kannadachallengers|mkvcinemas|vegamovies|moviesmod|pahe|galaxy|wetv)\s+/i', '', $baseF);

                            // If searchedTitle is "runner" and filename is "the runner", do NOT match (different movies)
                            if (!$hasLeadingThe && str_starts_with($baseF, 'the ')) {
                                continue;
                            }
                            // If searchedTitle is "the runner" and filename is "runner", do NOT match (different movies)
                            if ($hasLeadingThe && !str_starts_with($baseF, 'the ') && $baseF === substr($searchedLower, 4)) {
                                continue;
                            }

                            // Exclude titles with prefixes/suffixes (e.g. "late runner", "blade runner")
                            if ($baseF !== $searchedLower && $baseF !== "the {$searchedLower}") {
                                continue;
                            }

                            $filteredMovieFiles[] = $mf;
                        }

                        // If strict guard found exact matches, use them; otherwise fallback only if none found
                        if (!empty($filteredMovieFiles)) {
                            $filesToStream = $filteredMovieFiles;
                        }
                    }
                }
            }
        }

        // Automatically group and combine multi-part split videos (part001, part002, ...)
        $filesToStream = fd_group_split_parts($filesToStream);
        $totalFilesBeforeFilter = count($filesToStream);

        // Load stream configuration (AIOStreams style filters)
        $catSettings = fd_load_catalog_settings();
        $streamConfig = $catSettings['stream_config'] ?? [];
        $resFilter = $streamConfig['resolutions'] ?? ['4k' => true, '1080p' => true, '720p' => true, 'sd' => true, 'unknown' => true];
        $qualityFilter = $streamConfig['qualities'] ?? ['remux' => true, 'bluray' => true, 'webdl' => true, 'webrip' => true, 'hdtv' => true, 'cam' => true, 'unknown' => true];
        $encodeFilter = $streamConfig['encodes'] ?? ['hevc' => true, 'avc' => true, 'av1' => true];
        $visualFilter = $streamConfig['visual_tags'] ?? ['hdr' => true, 'dv' => true];
        $excludeCam = !empty($streamConfig['exclude_cam']);
        $excludeUnplayable = isset($streamConfig['exclude_unplayable']) ? !empty($streamConfig['exclude_unplayable']) : true;
        $preferredRes = $streamConfig['preferred_resolution'] ?? 'auto';
        $maxPerRes = max(0, (int)($streamConfig['max_streams_per_resolution'] ?? 0));
        $maxTotal = max(0, (int)($streamConfig['max_streams_total'] ?? 0));
        $minSizeMb = max(0, (int)($streamConfig['min_size_mb'] ?? 0));
        $maxSizeGb = max(0, (int)($streamConfig['max_size_gb'] ?? 0));
        $excludedKeywordsRaw = trim((string)($streamConfig['excluded_keywords'] ?? ''));
        $requiredKeywordsRaw = trim((string)($streamConfig['required_keywords'] ?? ''));

        $excludedKeywords = array_values(array_filter(array_map('trim', explode(',', strtolower($excludedKeywordsRaw))), fn($k) => $k !== ''));
        $requiredKeywords = array_values(array_filter(array_map('trim', explode(',', strtolower($requiredKeywordsRaw))), fn($k) => $k !== ''));

        // Helper to categorize resolution key matching AIOStreams
        $categorizeResolution = function (string $title, string $caption = ''): string {
            $tags = fd_extract_media_tags($title, $caption);
            return match (strtolower($tags['resolution'])) {
                '4k' => '4k',
                '1080p' => '1080p',
                '720p' => '720p',
                '540p', '480p', '360p' => 'sd',
                default => 'unknown',
            };
        };

        // Helper to categorize quality matching AIOStreams
        $categorizeQuality = function (string $title, string $caption = ''): string {
            $tags = fd_extract_media_tags($title, $caption);
            return match (strtolower($tags['source'])) {
                'remux' => 'remux',
                'bluray' => 'bluray',
                'web-dl' => 'webdl',
                'hdrip' => 'webrip',
                'hdtv' => 'hdtv',
                'cam' => 'cam',
                default => 'unknown',
            };
        };

        // Apply comprehensive AIOStreams filters
        $filteredFiles = [];
        foreach ($filesToStream as $fItem) {
            $fTitle = $fItem['title'] ?? '';
            $fCaption = $fItem['caption'] ?? '';
            $fSize = (int) ($fItem['file_size'] ?? 0);
            if ($fCaption !== '' && preg_match('/^(?:video(?:\.\d+)*|\d+|document|file)\.(?:mp4|mkv|avi|mov|ts|flv)$/i', trim($fTitle))) {
                $firstCap = trim(explode("\n", $fCaption)[0]);
                if ($firstCap !== '') $fTitle = $firstCap;
            }

            $lowerTitle = strtolower($fTitle);
            $itemTags = fd_extract_media_tags($fTitle, $fCaption);

            // 1. Resolution filter
            $rKey = $categorizeResolution($fTitle, $fCaption);
            if (isset($resFilter[$rKey]) && !$resFilter[$rKey]) {
                continue;
            }

            // 2. Quality filter
            $qKey = $categorizeQuality($fTitle, $fCaption);
            if ($excludeCam && $qKey === 'cam') {
                continue;
            }
            if (isset($qualityFilter[$qKey]) && !$qualityFilter[$qKey]) {
                continue;
            }

            // Check if file is an unplayable format. If it ends with a playable media extension (mp4, mkv, webm, etc.), it is playable!
            $hasPlayableExt = (bool) preg_match('/\.(mp4|m4v|mkv|webm|avi|mov|ts|m2ts|flv|wmv|3gp|mpg|mpeg|mp3|m4a|flac|wav|ogg|opus|aac)$/i', $fTitle);
            $isArchive = (bool) preg_match('/\.(?:zip|rar|7z|tar|gz|bz2|xz|iso|bin|exe|apk|pdf|epub)$/i', $fTitle);
            $isRawSplit = !$hasPlayableExt && (bool) preg_match('/\.(?:0\d{2,3}|\d{3})$/i', $fTitle);
            $isUnplayableItem = !$hasPlayableExt && ($isRawSplit || $isArchive);
            if ($excludeUnplayable && $isUnplayableItem) {
                continue;
            }

            // 3. Encodes filter (HEVC, AVC, AV1)
            $isHevc = $itemTags['codec'] === 'HEVC';
            $isAvc = $itemTags['codec'] === 'H.264';
            $isAv1 = $itemTags['codec'] === 'AV1';

            if ($isHevc && isset($encodeFilter['hevc']) && !$encodeFilter['hevc']) continue;
            if ($isAvc && isset($encodeFilter['avc']) && !$encodeFilter['avc']) continue;
            if ($isAv1 && isset($encodeFilter['av1']) && !$encodeFilter['av1']) continue;

            // 4. Visual tags filter (HDR, DV)
            $isHdr = in_array('HDR', $itemTags['visual'], true);
            $isDv = in_array('DV', $itemTags['visual'], true);

            if ($isHdr && isset($visualFilter['hdr']) && !$visualFilter['hdr']) continue;
            if ($isDv && isset($visualFilter['dv']) && !$visualFilter['dv']) continue;

            // 5. Size Range filters
            if ($minSizeMb > 0 && $fSize > 0 && $fSize < ($minSizeMb * 1048576)) {
                continue;
            }
            if ($maxSizeGb > 0 && $fSize > 0 && $fSize > ($maxSizeGb * 1073741824)) {
                continue;
            }

            // 6. Excluded Keywords filter
            if (!empty($excludedKeywords)) {
                $hasExcludedKw = false;
                foreach ($excludedKeywords as $ekw) {
                    if (str_contains($lowerTitle, $ekw)) {
                        $hasExcludedKw = true;
                        break;
                    }
                }
                if ($hasExcludedKw) continue;
            }

            // 7. Required Keywords filter
            if (!empty($requiredKeywords)) {
                $hasRequiredKw = false;
                foreach ($requiredKeywords as $rkw) {
                    if (str_contains($lowerTitle, $rkw)) {
                        $hasRequiredKw = true;
                        break;
                    }
                }
                if (!$hasRequiredKw) continue;
            }

            $fItem['_resKey'] = $rKey;
            $fItem['_qualKey'] = $qKey;
            $filteredFiles[] = $fItem;
        }

        // Sort streams using comprehensive media ranking:
        // Preferred resolution (if set) > Resolution (4K > 1080p > 720p > 480p > 360p) >
        // Quality (REMUX > BluRay/BDRip/BBRip > WEB-DL > WEBRip > HDRip > HDTV > DVDRip > CAM/TS) >
        // Visual enhancements (DV/HDR/10bit) > Codec (AV1 > HEVC > H.264) > File size
        usort($filteredFiles, function ($a, $b) use ($preferredRes) {
            $sA = fd_calculate_stream_sort_score(
                (string)($a['title'] ?? ''),
                (string)($a['caption'] ?? ''),
                (int)($a['file_size'] ?? 0),
                $preferredRes
            );
            $sB = fd_calculate_stream_sort_score(
                (string)($b['title'] ?? ''),
                (string)($b['caption'] ?? ''),
                (int)($b['file_size'] ?? 0),
                $preferredRes
            );
            return $sB <=> $sA;
        });

        // Apply per-resolution max limits if configured (> 0)
        $resCounts = [];
        if ($maxPerRes > 0) {
            $limitedFiles = [];
            foreach ($filteredFiles as $fItem) {
                $rKey = $fItem['_resKey'] ?? 'unknown';
                $cnt = $resCounts[$rKey] ?? 0;
                if ($cnt < $maxPerRes) {
                    $limitedFiles[] = $fItem;
                    $resCounts[$rKey] = $cnt + 1;
                }
            }
            $filesToStream = $limitedFiles;
        } else {
            $filesToStream = $filteredFiles;
        }

        // Keep the playable stream list small or user-capped
        $maxStreamFiles = ($maxTotal > 0) ? $maxTotal : 100;
        $filesToStream = array_slice($filesToStream, 0, $maxStreamFiles);

        // Pre-resolve all streams before play or download using batch warmup
        // Populates file_id_mt in memory and local resolve_cache for instant playback
        if (!empty($filesToStream)) {
            $filesToStream = fd_prewarm_streams_batch($filesToStream, $botIdStr);
        }

        foreach ($filesToStream as $fItem) {
            $fCode = $fItem['short_code'] ?? '';
            if ($fCode === '') continue;

            $fTitle = $fItem['title'] ?? '';
            $fCaption = $fItem['caption'] ?? '';
            $fSize = (int) ($fItem['file_size'] ?? 0);
            $fRawMime = (string) ($fItem['mime'] ?? ($fItem['mime_type'] ?? ($fItem['file_type'] ?? '')));
            $fMime = fd_guess_video_mime($fTitle, $fRawMime);

            // If title is generic video filename, use caption title for display
            if ($fCaption !== '' && preg_match('/^(?:video(?:\.\d+)*|\d+|document|file)\.(?:mp4|mkv|avi|mov|ts|flv)$/i', trim($fTitle))) {
                $firstCap = trim(explode("\n", $fCaption)[0]);
                if ($firstCap !== '') {
                    $fTitle = $firstCap;
                }
            }

            // Extract resolution / quality / release / codec / audio tags from filename & caption
            $mediaTags = fd_extract_media_tags($fTitle, $fCaption);
            $qualityTag = $mediaTags['resolution'];

            $metaPills = [];
            if ($mediaTags['platform'] !== '') {
                $metaPills[] = $mediaTags['platform'];
            }
            if ($mediaTags['source'] !== '') {
                $metaPills[] = $mediaTags['source'];
            }
            if (!empty($mediaTags['visual'])) {
                foreach ($mediaTags['visual'] as $v) {
                    $metaPills[] = $v;
                }
            }
            if ($mediaTags['codec'] !== '') {
                $metaPills[] = $mediaTags['codec'];
            }
            if ($mediaTags['audio'] !== '') {
                $metaPills[] = $mediaTags['audio'];
            }
            if ($mediaTags['edition'] !== '') {
                $metaPills[] = $mediaTags['edition'];
            }
            $metaPills = array_values(array_unique($metaPills));

            $cleanFTitle = fd_clean_media_title($fTitle);
            $fileName = $cleanFTitle !== '' ? $cleanFTitle : ($fTitle !== '' ? $fTitle : ($fCode . '.mp4'));

            // Check if file is an unplayable format (archive or raw split chunk).
            // If the filename ends with any standard playable video/audio extension, it is playable!
            $hasPlayableExt = (bool) preg_match('/\.(mp4|m4v|mkv|webm|avi|mov|ts|m2ts|flv|wmv|3gp|mpg|mpeg|mp3|m4a|flac|wav|ogg|opus|aac)$/i', $fTitle);
            $isArchive = (bool) preg_match('/\.(?:zip|rar|7z|tar|gz|bz2|xz|iso|bin|exe|apk|pdf|epub)$/i', $fTitle);
            $isRawSplit = !$hasPlayableExt && (bool) preg_match('/\.(?:0\d{2,3}|\d{3})$/i', $fTitle);
            $isUnplayable = !$hasPlayableExt && ($isRawSplit || $isArchive);

            // Torrentio / AIOStreams Hybrid styling
            $resBadge = $qualityTag !== '' ? $qualityTag : 'Direct';
            $streamBadgeLine1 = "PencariMovie {$resBadge}";

            $visualPills = [];
            if ($isUnplayable) {
                $visualPills[] = 'External';
            }
            if (in_array('HDR', $mediaTags['visual'], true)) $visualPills[] = 'HDR';
            if (in_array('DV', $mediaTags['visual'], true)) $visualPills[] = 'DV';
            if (in_array('10bit', $mediaTags['visual'], true)) $visualPills[] = '10bit';
            if (in_array('IMAX', $mediaTags['visual'], true)) $visualPills[] = 'IMAX';

            $streamName = $streamBadgeLine1 . (!empty($visualPills) ? "\n" . implode(' | ', $visualPills) : '');

            $streamBot = fd_pick_pool_bot();
            $streamBotId = !empty($fItem['bot_id']) ? (string) $fItem['bot_id'] : (!empty($streamBot['bot_id']) ? (string) $streamBot['bot_id'] : $botIdStr);

            $displayName = $cleanFTitle !== '' ? $cleanFTitle : ($fTitle !== '' ? $fTitle : ('File ShortCode: ' . $fCode));

            // Extract languages & subtitles for Torrentio-style flag representation
            $fullMetaText = $fTitle . ' ' . $fCaption;
            $langFlags = [];
            if (preg_match('/\b(malay|malaysub|sub\s*malay|msia|melayu)\b/i', $fullMetaText)) {
                $langFlags[] = '🇲🇾 Malay';
            }
            if (preg_match('/\b(eng|english|esub|sub\s*eng)\b/i', $fullMetaText)) {
                $langFlags[] = '🇬🇧 English';
            }
            if (preg_match('/\b(indo|indonesia|indosub)\b/i', $fullMetaText)) {
                $langFlags[] = '🇮🇩 Indo';
            }
            if (preg_match('/\b(hindi|hin|dub\s*hindi)\b/i', $fullMetaText)) {
                $langFlags[] = '🇮🇳 Hindi';
            }
            if (preg_match('/\b(korean|kor|kdrama)\b/i', $fullMetaText)) {
                $langFlags[] = '🇰🇷 Korean';
            }
            if (preg_match('/\b(japanese|jap|anime)\b/i', $fullMetaText)) {
                $langFlags[] = '🇯🇵 Japanese';
            }
            if (preg_match('/\b(chinese|mandarin|cantonese|c-drama)\b/i', $fullMetaText)) {
                $langFlags[] = '🇨🇳 Chinese';
            }
            if (preg_match('/\b(thai)\b/i', $fullMetaText)) {
                $langFlags[] = '🇹🇭 Thai';
            }
            $langFlags = array_values(array_unique($langFlags));

            // Format description:
            // Line 1: Clean Release Filename
            // Line 2: 💾 1.6 GB ⚡ Telegram • BluRay • HEVC • AAC5.1
            // Line 3: 💬 🇲🇾 Malay / 🇬🇧 English (if detected)
            $specBits = [];
            if ($fSize > 0) {
                $specBits[] = '💾 ' . fd_format_bytes($fSize);
            }
            $specBits[] = '⚡ Telegram';
            foreach ($metaPills as $mp) {
                if ($mp !== 'HDR') { // already on badge
                    $specBits[] = $mp;
                }
            }

            $descLines = [];
            $descLines[] = $displayName;
            $descLines[] = implode(' • ', $specBits);
            if (!empty($langFlags)) {
                $descLines[] = '💬 ' . implode(' / ', $langFlags);
            }
            $streamDesc = implode("\n", $descLines);

            $streamFileName = fd_stremio_stream_filename($fileName, $fMime);
            $streamMime = fd_guess_video_mime($streamFileName, $fMime);

            $payload = [
                'short_code' => $fCode,
                'bot_id' => $streamBotId,
                'file_size' => $fSize,
                'file_name' => $streamFileName,
                'mime' => $streamMime,
            ];
            if (!empty($fItem['file_id_mt'])) {
                $payload['file_id_mt'] = $fItem['file_id_mt'];
            }
            if (!empty($fItem['is_split_part'])) {
                $pNumStr = sprintf('%02d', (int) $fItem['part_num']);
                $totalStr = !empty($fItem['total_parts']) ? sprintf('/%02d', (int) $fItem['total_parts']) : '';
                $metaPills[] = "Part {$pNumStr}{$totalStr}";
            }

            $d = rtrim(strtr(base64_encode(json_encode($payload, JSON_UNESCAPED_SLASHES)), '+/', '-_'), '=');
            $localStreamUrl = fd_build_stremio_stream_url($baseUrl, $d, $streamFileName, $streamMime);

            if ($isUnplayable) {
                // For unplayable formats (split chunk .001 or archive), provide an externalUrl stream
                // pointing to the local downloader stream URL so Stremio/browser downloads the file directly
                $unplayableStreamObj = [
                    'name' => $streamName,
                    'description' => $streamDesc,
                    'externalUrl' => $localStreamUrl,
                    'behaviorHints' => [
                        'filename' => $streamFileName,
                        'notWebReady' => true,
                    ],
                ];
                if ($fSize > 0) {
                    $unplayableStreamObj['behaviorHints']['videoSize'] = $fSize;
                }
                $streams[] = $unplayableStreamObj;
                continue;
            }

            $streamExt = strtolower(pathinfo(parse_url($localStreamUrl, PHP_URL_PATH) ?: '', PATHINFO_EXTENSION));
            $isWebReady = (str_starts_with($localStreamUrl, 'https://') || str_starts_with($localStreamUrl, 'http://'))
                && in_array($streamExt, ['mp4', 'm4v', 'webm'], true);
            $behaviorHints = [
                'filename' => $streamFileName,
            ];
            if (!$isWebReady) {
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
            // behaviorHints + subtitles.
            $streamObj = [
                'name' => $streamName,
                'description' => $streamDesc,
                'url' => $localStreamUrl,
                'behaviorHints' => $behaviorHints,
            ];
            // Attach subtitles if available for this media item so players with stream.subtitles support get them inline
            if (!empty($subtitlesForStream)) {
                $streamObj['subtitles'] = $subtitlesForStream;
            }
            $streams[] = $streamObj;
        }

        // Fetch & merge streams from configured upstream addons
        $catSettings = fd_load_catalog_settings();
        $configuredUpstreams = (array) ($catSettings['upstream_manifests'] ?? []);
        foreach ($configuredUpstreams as $upstream) {
            $manifestUrl = trim((string)($upstream['url'] ?? ''));
            if ($manifestUrl === '') continue;
            $baseAddonUrl = preg_replace('#/manifest\.json(\?.*)?$#i', '', $manifestUrl);
            $upstreamStreamUrl = rtrim($baseAddonUrl, '/') . "/stream/{$itemType}/" . urlencode($itemId) . ".json";
            $uRes = fd_http_json($upstreamStreamUrl, [], 'GET', 6);
            if (!empty($uRes['streams']) && is_array($uRes['streams'])) {
                foreach ($uRes['streams'] as $uStream) {
                    if (is_array($uStream) && (!empty($uStream['url']) || !empty($uStream['infoHash']) || !empty($uStream['externalUrl']))) {
                        $streams[] = $uStream;
                    }
                }
            }
        }

        // Add sponsored / ad stream link with externalUrl on top if configured and not empty
        $sponsorInfo = $versionCheck['sponsor'] ?? [];
        $adUrl = trim((string) ($sponsorInfo['url'] ?? ''));
        $adName = trim((string) ($sponsorInfo['name'] ?? ''));
        $adDesc = trim((string) ($sponsorInfo['description'] ?? ''));

        if (!empty($streams)) {
            if ($adUrl !== '') {
                array_unshift($streams, [
                    'name' => $adName,
                    'description' => $adDesc,
                    'externalUrl' => $adUrl,
                ]);
            }
        } else {
            // No streams found: check if files existed but were all hidden by user's stream filters
            if (!empty($totalFilesBeforeFilter) && $totalFilesBeforeFilter > 0) {
                $noStreamTitle = '⚠️ Streams Filtered Out';
                $noStreamDesc = "🔍 {$totalFilesBeforeFilter} stream(s) found on PencariMovie, but hidden by your active filters (Resolution / Codec / Size / Keywords).\n👉 Tap to open settings & adjust filters.";
                $fallbackUrl = rtrim($baseUrl, '/') . '/#configure';
            } else {
                $noStreamTitle = 'No Streams Found';
                $noStreamDesc = "⚠️ No streams available for this title.";
                $fallbackUrl = $adUrl !== '' ? $adUrl : ($versionCheck['update_url'] ?? 'https://pencarimovie.com');
            }
            $streams[] = [
                'name' => $noStreamTitle,
                'description' => $noStreamDesc,
                'externalUrl' => $fallbackUrl,
            ];
        }

        // Save to stream cache for 5 minutes
        if (!empty($streamCacheFile)) {
            @file_put_contents($streamCacheFile, json_encode(['streams' => $streams], JSON_UNESCAPED_SLASHES | JSON_UNESCAPED_UNICODE), LOCK_EX);
        }

        fd_stremio_json(['streams' => $streams]);
    }

    // ── Nuvio Subtitles: /subtitles/:type/:id[/:extra].json ──
    if (preg_match('#^/subtitles/([^/]+)/([^/]+?)(?:/(.*))?\.json$#', $addonPath, $matches)) {
        $itemType = $matches[1];
        $itemId = urldecode($matches[2]);
        $subtitles = fd_get_item_subtitles($itemType, $itemId);
        fd_stremio_json(['subtitles' => $subtitles]);
    }

    fd_stremio_json(['ok' => 0, 'message' => 'Unknown Nuvio addon route'], 404);
}


if (str_starts_with($path, '/api/')) {
    if (!headers_sent()) {
        header('Access-Control-Allow-Origin: *');
        header('Access-Control-Allow-Methods: GET, HEAD, POST, OPTIONS');
        header('Access-Control-Allow-Headers: Content-Type, Authorization, X-API-Secret, Range');
        header('Access-Control-Expose-Headers: Accept-Ranges, Content-Range, Content-Length, Content-Type');
        header('X-Content-Type-Options: nosniff');
        header('X-Frame-Options: SAMEORIGIN');
        header('Referrer-Policy: strict-origin-when-cross-origin');
    }

    if ($method === 'OPTIONS') {
        http_response_code(204);
        exit;
    }

    // Public API routes: no auth, no local-only check. Auth endpoints must be
    // reachable so a remote user can log in; /api/download is public at the
    // route level but its handler calls fd_require_auth() itself.
    $alwaysPublicApi = [
        '/api/download',
        '/api/version',
        '/api/clock-check',
        '/api/auth/status',
        '/api/auth/login',
    ];
    if (fd_is_public_download_path($path) && !in_array($path, $alwaysPublicApi, true)) {
        $alwaysPublicApi[] = $path;
    }
    // Security note: /api/logs and /api/debug-mode must remain local-only
    // to prevent sensitive session traces or Telegram auth tokens leaking over tunnels.
    // They require BOTH a valid token AND a local request.
    if (!in_array($path, $alwaysPublicApi, true)) {
        fd_require_auth();
    }

    // ── GET /api/sub-proxy — proxy & convert subtitle (SRT -> WebVTT) for HTML5 video
    if ($path === '/api/sub-proxy' && $method === 'GET') {
        $subUrl = trim((string) ($_GET['url'] ?? ''));
        if ($subUrl === '' || !preg_match('#^https?://#i', $subUrl)) {
            http_response_code(400);
            echo "URL is required";
            exit;
        }

        $cacheKey = 'sub_vtt_' . md5($subUrl);
        $cacheFile = fd_storage_path('storage/' . $cacheKey . '.vtt');
        if (is_file($cacheFile) && (time() - (int)@filemtime($cacheFile)) < 86400) {
            header('Content-Type: text/vtt; charset=utf-8');
            header('Access-Control-Allow-Origin: *');
            readfile($cacheFile);
            exit;
        }

        $raw = fd_http_get_contents($subUrl, ['timeout' => 10]);
        if ($raw === false || $raw === '') {
            http_response_code(502);
            echo "Failed to fetch subtitle";
            exit;
        }

        // Convert to WebVTT if needed
        $vtt = $raw;
        if (!str_starts_with(trim($raw), 'WEBVTT')) {
            // Convert SRT to WebVTT
            $vtt = str_replace(["\r\n", "\r"], "\n", $raw);
            // Replace comma timestamps (00:00:00,000) with period (00:00:00.000)
            $vtt = preg_replace('/(\d{2}:\d{2}:\d{2}),(\d{3})/m', '$1.$2', $vtt);
            $vtt = "WEBVTT\n\n" . ltrim($vtt);
        }

        @file_put_contents($cacheFile, $vtt, LOCK_EX);
        header('Content-Type: text/vtt; charset=utf-8');
        header('Access-Control-Allow-Origin: *');
        echo $vtt;
        exit;
    }

    // ── Auth routes (public: must work before login and before the version gate) ──
    if ($path === '/api/auth/status' && $method === 'GET') {
        $authed = fd_is_authenticated();
        fd_json([
            'ok' => 1,
            'enabled' => fd_auth_enabled(),
            'authenticated' => $authed,
            'token' => $authed ? fd_auth_token() : '',
        ]);
    }

    if ($path === '/api/auth/login' && $method === 'POST') {
        $clientIp = fd_auth_client_ip();
        $retryAfter = fd_auth_lockout_check($clientIp);
        if ($retryAfter !== null && $retryAfter > 0) {
            header('Retry-After: ' . $retryAfter);
            fd_json([
                'ok' => 0,
                'message' => "Too many failed attempts. Try again in {$retryAfter}s. (Or reset via: pms reset-password)",
                'retryAfter' => $retryAfter,
                'locked' => true,
            ], 429);
        }

        $input = json_decode((string) file_get_contents('php://input'), true);
        $pw = (string) (is_array($input) ? ($input['password'] ?? '') : '');
        if (!fd_auth_verify_password($pw)) {
            $fail = fd_auth_record_failure($clientIp);
            if (!empty($fail['locked'])) {
                $wait = $fail['retryAfter'];
                header('Retry-After: ' . $wait);
                fd_json([
                    'ok' => 0,
                    'message' => "Too many failed attempts. Try again in {$wait}s. (Or reset via: pms reset-password)",
                    'retryAfter' => $wait,
                    'locked' => true,
                ], 429);
            }
            $left = $fail['remaining'];
            fd_json([
                'ok' => 0,
                'message' => "Invalid password. {$left} attempt(s) left before lockout.",
                'remainingBeforeLock' => $left,
            ], 401);
        }

        fd_auth_record_success($clientIp);
        $token = fd_auth_token();
        setcookie(FD_AUTH_COOKIE, $token, [
            'expires' => time() + 86400 * 365,
            'path' => '/',
            'httponly' => true,
            'samesite' => 'Lax',
        ]);
        fd_json(['ok' => 1, 'token' => $token]);
    }

    if ($path === '/api/auth/logout' && $method === 'POST') {
        setcookie(FD_AUTH_COOKIE, '', ['expires' => time() - 3600, 'path' => '/']);
        fd_json(['ok' => 1]);
    }

    if ($path === '/api/auth/password' && $method === 'POST') {
        $input = json_decode((string) file_get_contents('php://input'), true);
        $input = is_array($input) ? $input : [];
        if (!fd_auth_verify_password((string) ($input['current'] ?? ''))) {
            fd_json(['ok' => 0, 'message' => 'Current password is wrong'], 401);
        }
        $next = trim((string) ($input['next'] ?? ''));
        if (strlen($next) < 4) {
            fd_json(['ok' => 0, 'message' => 'Password must be at least 4 characters'], 400);
        }
        fd_auth_set_password($next);
        fd_json(['ok' => 1, 'token' => fd_auth_token()]);
    }

    if ($path === '/api/auth/token/rotate' && $method === 'POST') {
        fd_json(['ok' => 1, 'token' => fd_auth_rotate_token()]);
    }

    if ($path === '/api/auth/reset' && $method === 'POST') {
        fd_require_local_request();
        fd_auth_set_password(FD_AUTH_DEFAULT_PASSWORD);
        @unlink(fd_storage_path('storage/cache/auth_lockout.json'));
        fd_json(['ok' => 1, 'token' => fd_auth_token()]);
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
        $input = json_decode((string) file_get_contents('php://input'), true);
        $token = is_array($input) ? trim((string) ($input['tunnel_token'] ?? '')) : '';
        $result = fd_enable_tunnel($token);
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
            'is_tunnel' => false,
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
        if (array_key_exists('country', $input)) {
            $current['country'] = strtoupper(trim(preg_replace('/[^a-zA-Z0-9_-]/', '', (string)$input['country'])));
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

        if (isset($input['upstream_manifests']) && is_array($input['upstream_manifests'])) {
            $cleanedManifests = [];
            foreach ($input['upstream_manifests'] as $m) {
                if (is_array($m) && !empty($m['url'])) {
                    $cleanedManifests[] = [
                        'name' => trim((string)($m['name'] ?? 'Addon')),
                        'url' => trim((string)$m['url']),
                        'id' => trim((string)($m['id'] ?? '')),
                        'version' => trim((string)($m['version'] ?? '')),
                    ];
                }
            }
            $current['upstream_manifests'] = $cleanedManifests;
        }

        if (isset($input['stream_config']) && is_array($input['stream_config'])) {
            $inSc = $input['stream_config'];
            if (!isset($current['stream_config']) || !is_array($current['stream_config'])) {
                $current['stream_config'] = [];
            }
            if (isset($inSc['resolutions']) && is_array($inSc['resolutions'])) {
                $current['stream_config']['resolutions'] = [
                    '4k' => !empty($inSc['resolutions']['4k']),
                    '1080p' => !empty($inSc['resolutions']['1080p']),
                    '720p' => !empty($inSc['resolutions']['720p']),
                    'sd' => !empty($inSc['resolutions']['sd']),
                    'unknown' => !empty($inSc['resolutions']['unknown']),
                ];
            }
            if (isset($inSc['qualities']) && is_array($inSc['qualities'])) {
                $current['stream_config']['qualities'] = [
                    'remux' => !empty($inSc['qualities']['remux']),
                    'bluray' => !empty($inSc['qualities']['bluray']),
                    'webdl' => !empty($inSc['qualities']['webdl']),
                    'webrip' => !empty($inSc['qualities']['webrip']),
                    'hdtv' => !empty($inSc['qualities']['hdtv']),
                    'cam' => !empty($inSc['qualities']['cam']),
                    'unknown' => !empty($inSc['qualities']['unknown']),
                ];
            }
            if (isset($inSc['encodes']) && is_array($inSc['encodes'])) {
                $current['stream_config']['encodes'] = [
                    'hevc' => !empty($inSc['encodes']['hevc']),
                    'avc' => !empty($inSc['encodes']['avc']),
                    'av1' => !empty($inSc['encodes']['av1']),
                ];
            }
            if (isset($inSc['visual_tags']) && is_array($inSc['visual_tags'])) {
                $current['stream_config']['visual_tags'] = [
                    'hdr' => !empty($inSc['visual_tags']['hdr']),
                    'dv' => !empty($inSc['visual_tags']['dv']),
                ];
            }
            if (isset($inSc['preferred_resolution'])) {
                $pref = strtolower(trim((string)$inSc['preferred_resolution']));
                $current['stream_config']['preferred_resolution'] = in_array($pref, ['auto', '4k', '1080p', '720p', 'sd'], true) ? $pref : 'auto';
            }
            if (isset($inSc['max_streams_per_resolution'])) {
                $current['stream_config']['max_streams_per_resolution'] = max(0, min(50, (int)$inSc['max_streams_per_resolution']));
            }
            if (isset($inSc['max_streams_total'])) {
                $current['stream_config']['max_streams_total'] = max(0, min(200, (int)$inSc['max_streams_total']));
            }
            if (isset($inSc['min_size_mb'])) {
                $current['stream_config']['min_size_mb'] = max(0, min(100000, (int)$inSc['min_size_mb']));
            }
            if (isset($inSc['max_size_gb'])) {
                $current['stream_config']['max_size_gb'] = max(0, min(200, (int)$inSc['max_size_gb']));
            }
            if (isset($inSc['exclude_cam'])) {
                $current['stream_config']['exclude_cam'] = (bool) $inSc['exclude_cam'];
            }
            if (isset($inSc['exclude_unplayable'])) {
                $current['stream_config']['exclude_unplayable'] = (bool) $inSc['exclude_unplayable'];
            }
            if (isset($inSc['excluded_keywords'])) {
                $current['stream_config']['excluded_keywords'] = trim((string) $inSc['excluded_keywords']);
            }
            if (isset($inSc['required_keywords'])) {
                $current['stream_config']['required_keywords'] = trim((string) $inSc['required_keywords']);
            }
        }

        $saved = fd_save_catalog_settings($current);
        if ($saved) {
            // Invalidate stream cache so new stream configurations apply immediately
            $cacheDir = FD_CACHE_DIR;
            if (is_dir($cacheDir)) {
                $cachedStreamFiles = glob($cacheDir . '/stream_cache_*.json');
                if ($cachedStreamFiles) {
                    foreach ($cachedStreamFiles as $csf) {
                        @unlink($csf);
                    }
                }
            }
        }
        fd_json([
            'ok' => $saved ? 1 : 0,
            'settings' => $current,
            'message' => $saved ? 'Catalog settings updated successfully' : 'Failed to save settings',
        ]);
    }

    // ── Standalone media_ids_idx API ──
    // Local-only write path that populates the self-contained `media_ids_idx`
    // table. This is what makes external IDs resolvable for other addons
    // WITHOUT needing wp_posts or posts_idx.
    //
    // POST body: { post_id, media_ids: {prefix: value, ...}, title, year, imdb_id, media_type }
    // DELETE:    ?post_id=N  (removes all rows for that post)
    if ($path === '/api/media-ids' && in_array($method, ['POST', 'DELETE'], true)) {
        fd_require_auth();

        if ($method === 'DELETE') {
            $postId = (int) ($_REQUEST['post_id'] ?? 0);
            if ($postId <= 0) {
                fd_json(['ok' => 0, 'error' => 'Missing post_id'], 400);
            }
            fd_media_ids_delete($postId);
            fd_json(['ok' => 1, 'deleted' => $postId]);
        }

        $input = json_decode((string) file_get_contents('php://input'), true);
        if (!is_array($input)) {
            fd_json(['ok' => 0, 'error' => 'Invalid JSON body'], 400);
        }

        $postId = (int) ($input['post_id'] ?? 0);
        $mediaIds = (array) ($input['media_ids'] ?? []);
        $title = trim((string) ($input['title'] ?? ''));
        $year = (int) ($input['year'] ?? 0);
        $imdbId = trim((string) ($input['imdb_id'] ?? ''));
        $mediaType = trim((string) ($input['media_type'] ?? 'movie'));

        if ($postId <= 0 || empty($mediaIds) || $title === '') {
            fd_json(['ok' => 0, 'error' => 'post_id, media_ids and title are required'], 400);
        }

        $written = fd_media_ids_upsert($postId, $mediaIds, $title, $year, $imdbId, $mediaType);
        fd_json([
            'ok'      => $written > 0 ? 1 : 0,
            'post_id' => $postId,
            'written' => $written,
        ], $written > 0 ? 200 : 500);
    }

    // ── Upstream manifest validator API ──
    if ($path === '/api/validate-manifest' && $method === 'POST') {
        $input = json_decode((string) file_get_contents('php://input'), true);
        $manifestUrl = trim((string) ($input['url'] ?? ''));
        if ($manifestUrl === '') {
            fd_json(['ok' => 0, 'error' => 'Please provide a manifest URL.'], 400);
        }

        // Auto-fix stremio:// protocol to https://
        if (str_starts_with($manifestUrl, 'stremio://')) {
            $manifestUrl = 'https://' . substr($manifestUrl, strlen('stremio://'));
        }
        // Auto-append /manifest.json if omitted
        if (!preg_match('#/manifest\.json(\?.*)?$#i', $manifestUrl)) {
            $manifestUrl = rtrim($manifestUrl, '/') . '/manifest.json';
        }

        $manifestJson = fd_http_json($manifestUrl, [], 'GET', 8);
        if (empty($manifestJson['id']) || empty($manifestJson['name'])) {
            fd_json([
                'ok' => 0,
                'error' => 'Invalid Stremio manifest. Make sure the URL points to a valid manifest.json responding with id and name.',
            ], 400);
        }

        fd_json([
            'ok' => 1,
            'manifest' => [
                'id' => (string) $manifestJson['id'],
                'name' => (string) $manifestJson['name'],
                'version' => (string) ($manifestJson['version'] ?? '1.0.0'),
                'description' => (string) ($manifestJson['description'] ?? ''),
                'url' => $manifestUrl,
                'resources' => $manifestJson['resources'] ?? [],
            ],
            'message' => 'Manifest validated successfully.',
        ]);
    }


    // ── Country Detection API (reads Cloudflare header in-memory, zero disk footprint) ──
    if ($path === '/api/country' && $method === 'GET') {
        $countryList = [
            ['code' => '', 'name' => 'Auto (Detected)'],
            ['code' => 'MY', 'name' => 'Malaysia'],
            ['code' => 'ID', 'name' => 'Indonesia'],
            ['code' => 'SG', 'name' => 'Singapore'],
            ['code' => 'TH', 'name' => 'Thailand'],
            ['code' => 'PH', 'name' => 'Philippines'],
            ['code' => 'VN', 'name' => 'Vietnam'],
            ['code' => 'KR', 'name' => 'Korea'],
            ['code' => 'JP', 'name' => 'Japan'],
            ['code' => 'CN', 'name' => 'China'],
            ['code' => 'HK', 'name' => 'Hong Kong'],
            ['code' => 'TW', 'name' => 'Taiwan'],
            ['code' => 'IN', 'name' => 'India'],
            ['code' => 'US', 'name' => 'United States'],
            ['code' => 'GB', 'name' => 'United Kingdom'],
            ['code' => 'AU', 'name' => 'Australia'],
            ['code' => 'DE', 'name' => 'Germany'],
            ['code' => 'NL', 'name' => 'Netherlands'],
            ['code' => 'FR', 'name' => 'France'],
            ['code' => 'CA', 'name' => 'Canada'],
        ];
        $settings = fd_load_catalog_settings();
        fd_json([
            'ok' => 1,
            'country' => fd_detect_country(),
            'configured_country' => (string) ($settings['country'] ?? ''),
            'available_countries' => $countryList,
        ]);
    }

    if ($path === '/api/country' && $method === 'POST') {
        $input = json_decode((string) file_get_contents('php://input'), true);
        if (!is_array($input)) {
            fd_json(['ok' => 0, 'error' => 'Invalid JSON input'], 400);
        }
        $targetCode = strtoupper(trim(preg_replace('/[^a-zA-Z0-9_-]/', '', (string)($input['country'] ?? ''))));
        $settings = fd_load_catalog_settings();
        $settings['country'] = $targetCode;
        $saved = fd_save_catalog_settings($settings);
        fd_json([
            'ok' => $saved ? 1 : 0,
            'country' => fd_detect_country(),
            'configured_country' => $targetCode,
            'message' => $saved ? 'Country updated successfully' : 'Failed to save country',
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

        // If meta username/name is empty, fill from bot pool
        $botUsername = (string) ($meta['bot_username'] ?? '');
        $botName = (string) ($meta['bot_name'] ?? '');
        if (($botUsername === '' || $botName === '') && !empty($pool)) {
            foreach ($pool as $pb) {
                if ((string)($pb['bot_id'] ?? '') === $botId) {
                    if ($botUsername === '') $botUsername = (string)($pb['bot_username'] ?? '');
                    if ($botName === '') $botName = (string)($pb['bot_name'] ?? '');
                    break;
                }
            }
        }

        $isProvisioning = fd_is_guest_provision_in_progress();
        fd_json([
            'ok' => 1,
            'version' => FD_APP_VERSION,
            'has_session' => $hasSession,
            'is_provisioning' => $isProvisioning,
            'bot_id' => $hasSession ? $botId : '',
            'bot_username' => $hasSession ? $botUsername : '',
            'bot_name' => $hasSession ? $botName : '',
            'api_secret' => $hasSession ? fd_get_api_secret() : '',
            'device_id' => fd_get_device_id(),
            'bot_count' => count($pool),
            'bot_pool' => $pool,
        ]);
    }

    // ── GET /api/clock-check — measure local clock offset vs Telegram ─────────
    // Lets the frontend warn the user BEFORE attempting a bot login, since a
    // skewed clock makes every MTProto handshake fail with a confusing
    // "message ID too new/old" error.
    if ($path === '/api/clock-check' && $method === 'GET') {
        $probe = fd_measure_clock_offset();
        $abs = abs((int) $probe['offset']);
        fd_json([
            'ok' => $probe['ok'] ? 1 : 0,
            'offset_seconds' => (int) $probe['offset'],
            'server_time' => (int) $probe['server_time'],
            'local_time' => (int) $probe['local_time'],
            // Telegram's MTProto handshake is far less tolerant than the
            // documented ±300s: a measured 49s offset was enough to trigger
            // bad_msg_notification "msg_id too high" and a mid-handshake
            // session reset (SecurityException: wrong new_nonce_hash1).
            // Treat anything beyond 30s as skewed and beyond 60s as critical.
            'skewed' => $probe['ok'] && $abs > 30,
            'critical' => $probe['ok'] && $abs > 60,
            'message' => $probe['ok'] ? '' : $probe['error'],
        ]);
    }

    // ── POST /api/provision — auto-provision a guest bot session on the fly ───
    if ($path === '/api/provision' && in_array($method, ['GET', 'POST'], true)) {
        $provisioned = fd_auto_provision_guest();
        if (!$provisioned || !empty($provisioned['error'])) {
            $errMsg = !empty($provisioned['error'])
                ? $provisioned['error']
                : 'Could not obtain guest bot session from server.';
            fd_json([
                'ok' => 0,
                'message' => $errMsg,
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
    $versionCheck = fd_check_version();
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

    // ── GET / POST /api/debug-mode — Toggle debug logging ────────────────────
    // URL query toggle support: /api/debug-mode?enable=1 or ?enable=0
    if ($path === '/api/debug-mode') {
        if ($method === 'POST') {
            $input = json_decode((string) file_get_contents('php://input'), true);
            $enable = isset($input['enabled']) ? (bool) $input['enabled'] : false;
            fd_set_debug_enabled($enable);
            fd_json(['ok' => 1, 'enabled' => fd_is_debug_enabled()]);
        }
        if ($method === 'GET') {
            if (isset($_GET['enable'])) {
                $val = trim((string) $_GET['enable']);
                $enable = ($val === '1' || strtolower($val) === 'true' || strtolower($val) === 'on');
                fd_set_debug_enabled($enable);
            }
            fd_json(['ok' => 1, 'enabled' => fd_is_debug_enabled()]);
        }
    }

    // ── GET /api/logs — View debug.log and MadelineProto.log from browser ────
    // Supports query/path parameter: ?file=debug.log or ?file=MadelineProto.log
    if ($path === '/api/logs' && $method === 'GET') {
        if (!fd_is_debug_enabled()) {
            fd_json(['ok' => 0, 'message' => 'Debug logging is currently disabled. Enable it via /api/debug-mode?enable=1'], 403);
        }

        $reqFile = strtolower(trim((string) ($_GET['file'] ?? $_GET['path'] ?? 'debug.log')));
        $reqFile = basename($reqFile);

        $allowedFiles = [
            'debug.log' => FD_DEBUG_LOG_PATH,
            'madelineproto.log' => fd_storage_path('MadelineProto.log'),
        ];

        // Also check if MadelineProto.log is at repository root
        $rootMadelineLog = fd_get_app_root() . DIRECTORY_SEPARATOR . 'MadelineProto.log';
        if (is_file($rootMadelineLog)) {
            $allowedFiles['madelineproto.log'] = $rootMadelineLog;
        }

        // Candidate search across storage locations for debug.log / madelineproto.log
        if ($reqFile === 'debug.log' && !is_file(FD_DEBUG_LOG_PATH)) {
            $debugCandidates = [
                fd_storage_path('storage/debug.log'),
                fd_storage_path('debug.log'),
                fd_get_app_root() . DIRECTORY_SEPARATOR . 'storage' . DIRECTORY_SEPARATOR . 'debug.log',
                fd_get_app_root() . DIRECTORY_SEPARATOR . 'debug.log',
            ];
            foreach ($debugCandidates as $dc) {
                if (is_file($dc)) {
                    $allowedFiles['debug.log'] = $dc;
                    break;
                }
            }
        }

        if (!isset($allowedFiles[$reqFile])) {
            fd_json([
                'ok' => 0,
                'message' => 'Invalid log file requested. Allowed: ' . implode(', ', array_keys($allowedFiles)),
            ], 400);
        }

        $targetPath = $allowedFiles[$reqFile];
        if (!is_file($targetPath)) {
            header('Content-Type: text/plain; charset=utf-8');
            echo "(Log file {$reqFile} is empty or does not exist yet.)\n";
            exit;
        }

        // Apply size limiter before serving
        fd_limit_file_size($targetPath, FD_MAX_LOG_SIZE);

        $linesLimit = isset($_GET['lines']) ? max(10, min(5000, (int) $_GET['lines'])) : 1000;
        $download = !empty($_GET['download']);

        if ($download) {
            header('Content-Type: application/octet-stream');
            header('Content-Disposition: attachment; filename="' . $reqFile . '"');
            header('Content-Length: ' . filesize($targetPath));
            readfile($targetPath);
            exit;
        }

        header('Content-Type: text/plain; charset=utf-8');
        header('Cache-Control: no-cache, no-store, must-revalidate');

        // Read last N lines if file is large
        $fp = @fopen($targetPath, 'r');
        if ($fp) {
            $buffer = [];
            while (($line = fgets($fp)) !== false) {
                $buffer[] = $line;
                if (count($buffer) > $linesLimit) {
                    array_shift($buffer);
                }
            }
            fclose($fp);
            echo implode('', $buffer);
        } else {
            echo "(Could not open log file.)\n";
        }
        exit;
    }

    // ── GET /api/resolve-shortcode — proxy short_code resolution through
    //     the local backend so the API secret (X-API-Secret header) is
    //     automatically sent to WordPress. The frontend should call this
    //     instead of hitting WordPress directly. ────────────────────────────
    if ($path === '/api/resolve-shortcode' && ($method === 'GET' || $method === 'POST')) {
        $rawCodes = trim((string) ($_REQUEST['short_codes'] ?? ''));
        $botId = trim((string) ($_REQUEST['bot_id'] ?? ''));

        if ($botId === '') {
            $picked = fd_pick_pool_bot();
            if (!empty($picked['bot_id'])) {
                $botId = (string) $picked['bot_id'];
            } else {
                $botId = fd_get_bot_id();
            }
        }

        // Batch resolution mode
        if ($rawCodes !== '') {
            $codesList = preg_split('/[\s,]+/', $rawCodes);
            $batchResults = fd_resolve_shortcodes_batch($codesList, $botId);
            fd_json([
                'ok' => 1,
                'bot_id' => $botId,
                'total' => count($codesList),
                'resolved' => count($batchResults),
                'results' => $batchResults,
            ]);
        }

        $shortCode = trim((string) ($_REQUEST['short_code'] ?? ''));
        if ($shortCode === '') {
            fd_json(['ok' => 0, 'message' => 'short_code or short_codes is required.'], 400);
        }

        // Use concurrent multi-bot resolution across all pool bots for instantaneous resolution
        // Pass empty candidate bots so it queries all pool bots + active bot simultaneously
        $result = fd_resolve_shortcode_concurrent($shortCode);
        if (empty($result['ok'])) {
            fd_json($result, 200);
        }
        fd_json($result);
    }

    // ── POST /api/warmup-resolve — batch warm-up / repopulate the local
    //     resolve cache. Accepts either an explicit `short_codes` list or a
    //     `query` (which is searched via WordPress and every returned file is
    //     warmed). Unlike /api/resolve-shortcode this actually waits for and
    //     persists the resolved file_id_mt values. ──────────────────────────
    if ($path === '/api/warmup-resolve' && ($method === 'POST' || $method === 'GET')) {
        $input = [];
        if ($method === 'POST') {
            $raw = (string) file_get_contents('php://input');
            $decoded = json_decode($raw, true);
            $input = is_array($decoded) ? $decoded : $_POST;
        } else {
            $input = $_GET;
        }

        $botId = trim((string) ($input['bot_id'] ?? ''));
        if ($botId === '') {
            $picked = fd_pick_pool_bot();
            $botId = !empty($picked['bot_id']) ? (string) $picked['bot_id'] : fd_get_bot_id();
        }

        $codes = [];
        $rawCodes = $input['short_codes'] ?? '';
        if (is_array($rawCodes)) {
            $codes = $rawCodes;
        } elseif (is_string($rawCodes) && trim($rawCodes) !== '') {
            $codes = preg_split('/[\s,]+/', trim($rawCodes));
        }

        // Optional: warm up every file matching a search query. Uses the
        // WordPress `search_files` AJAX action (returns files[] with
        // short_code) so a title like "Sunday.Morning" can be reindexed with
        // its band/artist metadata.
        $query = trim((string) ($input['query'] ?? ''));
        if ($query !== '') {
            $limit = max(1, min(200, (int) ($input['limit'] ?? 50)));
            $searchRes = fd_fetch_stream_ajax('search_files', [
                'search' => $query,
                'limit' => $limit,
                'offset' => 0,
            ]);
            $rows = [];
            if (is_array($searchRes)) {
                if (isset($searchRes['files']) && is_array($searchRes['files'])) {
                    $rows = $searchRes['files'];
                } elseif (isset($searchRes['data']['files']) && is_array($searchRes['data']['files'])) {
                    $rows = $searchRes['data']['files'];
                } elseif (isset($searchRes['results']) && is_array($searchRes['results'])) {
                    $rows = $searchRes['results'];
                }
            }
            foreach ($rows as $row) {
                $sc = (string) ($row['short_code'] ?? '');
                if ($sc !== '') {
                    $codes[] = $sc;
                }
            }
        }

        $codes = array_values(array_unique(array_filter(array_map('trim', $codes))));
        if (empty($codes)) {
            fd_json(['ok' => 0, 'message' => 'Provide short_codes or query.'], 400);
        }

        $result = fd_warmup_resolve_batch($codes, $botId, (int) ($input['chunk_size'] ?? 40));
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
                // Do not call $mBot->logout() for pooled / leased guest bots because calling logout()
                // permanently terminates and revokes the Telegram bot auth session on Telegram's server,
                // causing lock contention and "session is busy / AUTH_KEY_UNREGISTERED" for subsequent connections.
            }
        }

        // Clean up all local sessions, locks, and pool files without invalidating keys on Telegram
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

        // Build the WordPress admin-ajax.php URL, strictly forwarding allowed parameters
        $wpUrl = defined('FD_WP_AJAX_URL') ? FD_WP_AJAX_URL : 'https://pencarimovie.com/wp-admin/admin-ajax.php';

        $allowedStreamParams = [
            'category',
            'slug',
            'search',
            'limit',
            'offset',
            'post_id',
            'short_code',
            'type',
            'genre',
            'year',
            'sort',
            'bot_id',
            'country'
        ];
        $queryParams = ['action' => $streamAction];
        foreach ($allowedStreamParams as $paramKey) {
            if (isset($_GET[$paramKey]) && is_scalar($_GET[$paramKey])) {
                $queryParams[$paramKey] = trim((string) $_GET[$paramKey]);
            }
        }

        if ($action === 'trending') {
            unset($queryParams['bot_id']);
        } elseif (empty($queryParams['bot_id'])) {
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
            fd_json(['ok' => 0, 'message' => 'Failed to fetch data from PencariMovie.'], 502);
        }

        // Try to decode as JSON to return proper Content-Type
        $decoded = json_decode($body, true);
        if (is_array($decoded)) {
            // When stream_post_files or stream_search_files returns files, pre-warm them in batch
            if (in_array($action, ['post_files', 'search_files'], true)) {
                $files = $decoded['data']['files'] ?? ($decoded['files'] ?? null);
                if (is_array($files) && !empty($files)) {
                    $activeBotId = !empty($queryParams['bot_id']) ? (string)$queryParams['bot_id'] : fd_get_bot_id();
                    $warmedFiles = fd_prewarm_streams_batch($files, $activeBotId);
                    if (isset($decoded['data']['files'])) {
                        $decoded['data']['files'] = $warmedFiles;
                    } elseif (isset($decoded['files'])) {
                        $decoded['files'] = $warmedFiles;
                    }
                }
            }
            fd_json($decoded);
        }

        // If not JSON, return raw with correct content type
        header('Content-Type: application/json; charset=utf-8');
        echo $body;
        return true;
    }

    // ── GET /api/download[/:payload/:filename] — stream Telegram file via MadelineProto ──
    // The payload is a path segment: /api/download/<base64url>/<filename>.<real-ext>.
    // Query ?d= remains supported. Remote clients must carry the auth token as a
    // /t/<token>/ prefix (stripped above) or a cookie/header.
    if (fd_is_public_download_path($path)) {
        if (!fd_is_authenticated()) {
            fd_json(['ok' => 0, 'message' => 'Password required', 'auth_required' => true], 401);
        }
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
                // Always ensure activeBotId is used if incoming botId is empty or not in pool
                if ($botId === '' || !fd_has_local_session($botId)) {
                    $botId = $activeBotId;
                }
            } else {
                header('Cache-Control: no-cache, no-store, must-revalidate');
                header('Connection: close');
                $isConnecting = fd_is_guest_provision_in_progress();
                fd_json([
                    'ok' => 0,
                    'message' => $isConnecting
                        ? 'Connecting guest bot in progress. Please refresh or try again in a few seconds.'
                        : 'Telegram Bot is not connected. Please connect your bot token in Settings to stream.',
                    'hint' => $isConnecting
                        ? 'Guest bot session is initializing. Please refresh playback shortly.'
                        : 'Open Settings and connect your bot token.',
                    'short_code' => $shortCode,
                    'bot_id' => $botId,
                ], $isConnecting ? 503 : 403);
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

        // Include active bot and local pool bots that have existing sessions
        if ($activeBotId !== '' && !in_array($activeBotId, $candidateBots, true) && fd_has_local_session($activeBotId)) {
            $candidateBots[] = $activeBotId;
        }

        if ($botId !== '' && !in_array($botId, $candidateBots, true) && fd_has_local_session($botId)) {
            $candidateBots[] = $botId;
        }

        $botPool = fd_get_bot_pool();
        foreach ($botPool as $pBot) {
            $pId = (string) ($pBot['bot_id'] ?? '');
            if ($pId !== '' && !in_array($pId, $candidateBots, true) && fd_has_local_session($pId)) {
                $candidateBots[] = $pId;
            }
        }

        if (empty($candidateBots) && $activeBotId !== '') {
            $candidateBots[] = $activeBotId;
        }

        if (empty($candidateBots)) {
            $candidateBots[] = $activeBotId;
        }

        // Preserve the requested $botId if it was specified and has a local session,
        // otherwise default to the first candidate bot
        if ($botId === '' || !fd_has_local_session($botId)) {
            $botId = (string) $candidateBots[0];
        }
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
                // If winning bot session boot failed or is missing, try with the caller's requested bot or active bot
                if (!$bootedMadeline && $botId !== '' && $botId !== $cBotId) {
                    [$bootedMadeline, $bootErr] = fd_boot_madeline(null, [], $botId);
                    if ($bootedMadeline) {
                        $cBotId = $botId;
                    }
                }
                if (!$bootedMadeline && $cBotId !== $activeBotId && $activeBotId !== '') {
                    [$bootedMadeline, $bootErr] = fd_boot_madeline(null, [], $activeBotId);
                    if ($bootedMadeline) {
                        $cBotId = $activeBotId;
                    }
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

            // If resolve failed for current candidates, retry up to 3 times with fresh auto-provisioned guest bots
            if ($fileId === '' && empty($madeline)) {
                for ($retryAttempt = 0; $retryAttempt < 3; $retryAttempt++) {
                    $retryProv = fd_auto_provision_guest();
                    if ($retryProv && !empty($retryProv['bot_id'])) {
                        $retryBotId = (string) $retryProv['bot_id'];
                        $retryRes = fd_resolve_shortcode($shortCode, $retryBotId);
                        $retryFileId = trim((string) ($retryRes['file_id_mt'] ?? $retryRes['file_id'] ?? ''));
                        if ($retryFileId !== '') {
                            $retryMadeline = $retryProv['madeline'] ?? null;
                            if (!$retryMadeline) {
                                [$retryBooted, $retryErr] = fd_boot_madeline(null, [], $retryBotId);
                                $retryMadeline = $retryBooted;
                            }
                            if ($retryMadeline) {
                                $madeline = $retryMadeline;
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
                                break;
                            }
                        } else {
                            $resolved = $retryRes;
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
                'hint' => 'Include file_size, file_name, and mime from your PencariMovie metadata response.',
            ], 400);
        }

        // If not already booted during short_code resolution loop, boot MadelineProto now
        if (!$madeline) {
            // First try with the requested botId if a local session exists
            if ($botId !== '' && fd_has_local_session($botId)) {
                [$madeline, $error] = fd_boot_madeline(null, [], $botId);
            }
            // If the specified bot has no session or failed, fallback to the primary active bot or any pool bot with a session
            if (!$madeline && fd_has_local_session()) {
                $picked = fd_pick_pool_bot();
                $fallbackBotId = !empty($picked['bot_id']) ? (string) $picked['bot_id'] : $activeBotId;
                if ($fallbackBotId !== '' && $fallbackBotId !== $botId) {
                    [$madeline, $error] = fd_boot_madeline(null, [], $fallbackBotId);
                    if ($madeline) {
                        $botId = $fallbackBotId;
                    }
                }
                if (!$madeline && $activeBotId !== '' && $activeBotId !== $botId) {
                    [$madeline, $error] = fd_boot_madeline(null, [], $activeBotId);
                    if ($madeline) {
                        $botId = $activeBotId;
                    }
                }
            }
            // If still no session available, auto-provision a guest bot
            if (!$madeline) {
                fd_log('no local session for download, auto-provisioning guest bot');
                $prov = fd_auto_provision_guest();
                if ($prov && !empty($prov['bot_id'])) {
                    $madeline = $prov['madeline'] ?? null;
                    $botId = (string) $prov['bot_id'];
                    if (!$madeline) {
                        [$madeline, $error] = fd_boot_madeline(null, [], $botId);
                    }
                } elseif ($prov && !empty($prov['error'])) {
                    // Surface the real reason (e.g. clock skew) instead of the
                    // generic "No valid MadelineProto session" fallback.
                    $error = (string) $prov['error'];
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

        $fileName = fd_stremio_stream_filename($fileName, $fileMime);
        $fileMime = fd_guess_video_mime($fileName, $fileMime);


        @set_time_limit(0);
        if (function_exists('ignore_user_abort')) {
            @ignore_user_abort(true);
        }

        $downloadAttempt = 0;
        $maxDownloadAttempts = ($shortCode !== '') ? 2 : 1;

        while ($downloadAttempt < $maxDownloadAttempts) {
            $downloadAttempt++;
            try {
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
                    'attempt' => $downloadAttempt,
                    'range' => $_SERVER['HTTP_RANGE'] ?? 'none',
                    'method' => $_SERVER['REQUEST_METHOD'] ?? 'GET',
                    'ip' => $_SERVER['REMOTE_ADDR'] ?? '',
                ]);
                $madeline->downloadToBrowser($fileId, null, $fileSize, $fileName, $fileMime);
                return true;
            } catch (Throwable $throwable) {
                $errStr = $throwable->getMessage();
                fd_log('downloadToBrowser failed', [
                    'error' => $errStr,
                    'attempt' => $downloadAttempt,
                    'short_code' => $shortCode,
                ]);

                // If FILE_REFERENCE_EXPIRED or could not refresh file reference and we have a short_code,
                // re-resolve the shortcode with cache bypassed to get a fresh file_id and retry
                $isRefExpired = (stripos($errStr, 'FILE_REFERENCE_EXPIRED') !== false || stripos($errStr, 'refresh file reference') !== false);
                if ($isRefExpired && $shortCode !== '') {
                    fd_log('file reference expired, re-resolving short_code with nocache', ['short_code' => $shortCode]);
                    // Re-resolve bypassing local cache
                    $reResolved = fd_resolve_shortcode($shortCode, $botId, true);
                    $newFileId = trim((string) ($reResolved['file_id_mt'] ?? $reResolved['file_id'] ?? ''));
                    if ($newFileId !== '' && $newFileId !== $fileId && $downloadAttempt < $maxDownloadAttempts) {
                        $fileId = $newFileId;
                        $fileSize = (int) ($reResolved['file_size'] ?? $fileSize);
                        continue;
                    }
                }

                fd_json([
                    'ok' => 0,
                    'message' => 'File download failed: ' . $errStr,
                ], 500);
            }
        }

        return true;
    }

    fd_json(['ok' => 0, 'message' => 'Unknown API endpoint'], 404);
}

// ─── Static file serving ────────────────────────────────────────────────────

// If running in CLI / warmup-ipc mode, do not attempt to serve static files
if (PHP_SAPI === 'cli' || PHP_SAPI === 'phpdbg') {
    return true;
}

// Auth gate: an unauthenticated visitor must never receive the dashboard HTML.
// This runs BEFORE the provisioning gate so a remote visitor sees the password
// page instead of the "Provisioning guest bot..." spinner (whose /api/session
// poll would 401 and loop forever).
if (($path === '/' || $path === '/index.html') && !fd_is_authenticated()) {
    fd_serve_auth_gate_html();
    return true;
}

// Block web UI (index.html, app.js) while guest bot provisioning is actively running in background
// to avoid incomplete loading, gate flicker, or concurrent race conditions from browser frontends.
if (fd_is_guest_provision_in_progress()) {
    header('Retry-After: 3');
    header('Cache-Control: no-store, no-cache, must-revalidate');
    if ($path === '/' || str_ends_with($path, '.html')) {
        header('Content-Type: text/html; charset=utf-8');
        echo '<!DOCTYPE html><html><head><meta charset="utf-8"><meta http-equiv="refresh" content="3"><title>PencariMovie Server</title>'
            . '<style>body{margin:0;padding:0;background:#141414;color:#fff;font-family:-apple-system,BlinkMacSystemFont,"Segoe UI",Roboto,sans-serif;display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:100vh;gap:20px;text-align:center;box-sizing:border-box;}'
            . '.loading-screen__spinner{width:44px;height:44px;border:4px solid rgba(255,255,255,0.1);border-top-color:#ff6b35;border-radius:50%;animation:loading-spin 0.8s linear infinite;}'
            . '.loading-screen__text{font-size:1rem;color:rgba(255,255,255,0.6);margin:0;animation:loading-pulse 1.5s ease-in-out infinite;}'
            . '@keyframes loading-spin{to{transform:rotate(360deg);}}'
            . '@keyframes loading-pulse{0%,100%{opacity:1;}50%{opacity:0.4;}}</style>'
            . '<script>'
            . 'async function checkSession() {'
            . '  try {'
            . '    const res = await fetch("/api/session?_t=" + Date.now());'
            . '    if (res.ok) {'
            . '      const d = await res.json();'
            . '      if (!d.is_provisioning && d.has_session) {'
            . '        window.location.reload();'
            . '        return;'
            . '      }'
            . '    }'
            . '  } catch (e) {}'
            . '  setTimeout(checkSession, 1500);'
            . '}'
            . 'setTimeout(checkSession, 1500);'
            . '</script></head>'
            . '<body><div class="loading-screen__spinner"></div><p class="loading-screen__text">Provisioning guest bot...</p></body></html>';
        return true;
    }
    if (str_ends_with($path, '.js')) {
        header('Content-Type: application/javascript; charset=utf-8');
        echo 'console.log("PencariMovie guest bot provisioning in progress...");'
            . 'setInterval(async function() {'
            . '  try {'
            . '    const r = await fetch("/api/session?_t=" + Date.now());'
            . '    if (r.ok) {'
            . '      const d = await r.json();'
            . '      if (!d.is_provisioning && d.has_session) window.location.reload();'
            . '    }'
            . '  } catch (e) {}'
            . '}, 2000);';
        return true;
    }
}

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
