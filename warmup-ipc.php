<?php

/**
 * Warm up persistent MadelineProto IPC workers for every bot in the pool.
 *
 * Called from start.bat / start.sh after the server boots. Under FrankenPHP,
 * workers spawned from within a web request die when the request ends, so we
 * pre-spawn detached workers here (outside any request) so fd_boot_madeline()
 * can connect to them as IPC clients (~40-60ms) instead of doing slow full
 * direct-mode boots.
 */
require_once __DIR__ . '/backend.php';

$pool = fd_get_bot_pool();
if (empty($pool)) {
    echo "No bots in pool, nothing to warm up.\n";
    exit(0);
}

$root = fd_get_app_root();
// Always spawn workers through the bundled bin/php wrapper (or bin/php.exe on
// Windows). On FrankenPHP the wrapper execs `frankenphp php-cli`, which is the
// only way to run a persistent CLI worker under FrankenPHP. Using PHP_BINARY
// here is wrong when this script itself runs under `frankenphp php-cli`,
// because PHP_BINARY would be the frankenphp binary and the spawned command
// would be missing the required `php-cli` subcommand.
$phpBin = $root . DIRECTORY_SEPARATOR . 'bin' . DIRECTORY_SEPARATOR . (fd_is_windows() ? 'php.exe' : 'php');
if (!is_file($phpBin)) {
    $prefix = $_SERVER['PREFIX'] ?? ($_ENV['PREFIX'] ?? '');
    if ($prefix !== '' && is_file($prefix . '/bin/php')) {
        $phpBin = $prefix . '/bin/php';
    } else {
        $phpBin = PHP_BINARY;
    }
}
$entry = $root . DIRECTORY_SEPARATOR . 'vendor' . DIRECTORY_SEPARATOR . 'danog' . DIRECTORY_SEPARATOR . 'madelineproto' . DIRECTORY_SEPARATOR . 'src' . DIRECTORY_SEPARATOR . 'Ipc' . DIRECTORY_SEPARATOR . 'Runner' . DIRECTORY_SEPARATOR . 'entry.php';
$ps1 = $root . DIRECTORY_SEPARATOR . 'spawn-ipc-worker.ps1';

function fd_warmup_single_bot_ipc(string $botId, string $phpBin, string $entry): bool
{
    $botId = trim($botId);
    if ($botId === '') {
        return false;
    }
    $sessionDir = fd_get_bot_session_path($botId);
    if (!is_dir($sessionDir) && !is_file($sessionDir)) {
        return false;
    }
    if (fd_ipc_worker_running($sessionDir)) {
        return true;
    }
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
        @pclose(@popen('start "" /b ' . $cmd . ' > NUL 2>&1', 'r'));
    } else {
        @shell_exec('nohup ' . $cmd . ' > /dev/null 2>&1 &');
    }
    return true;
}

foreach ($pool as $bot) {
    $botId = trim((string) ($bot['bot_id'] ?? ''));
    if ($botId === '') {
        continue;
    }
    $sessionDir = fd_get_bot_session_path($botId);
    if (!is_dir($sessionDir) && !is_file($sessionDir)) {
        echo "No session for bot $botId, skipping.\n";
        continue;
    }
    if (fd_ipc_worker_running($sessionDir)) {
        echo "Worker already running for bot $botId.\n";
        continue;
    }
    if (fd_warmup_single_bot_ipc($botId, $phpBin, $entry)) {
        echo "Spawned IPC worker for bot $botId.\n";
    }
}

// Give workers a moment to come up.
sleep(3);
echo "Warmup complete.\n";
