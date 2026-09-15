<?php

declare(strict_types=1);

/**
 * Detached CLI worker: pre-warm an ALAC `.m4a` -> FLAC conversion.
 *
 * Spawned by fd_maybe_prewarm_flac() from the /eclipse/stream route so the FLAC
 * is usually ready by the time the client fetches the stream URL. This avoids
 * the client's HTTP timeout firing during the ~30s raw download, which used to
 * cause a re-request that fell through to the raw ALAC .m4a (undecodable on
 * Android) and got skipped.
 *
 * Unlike audio-encode-worker.php (which only runs FFmpeg on an already-
 * downloaded file), this worker downloads the raw .m4a itself because it runs
 * detached from any web request that holds a MadelineProto instance.
 *
 * Usage:
 *   php audio-prewarm-worker.php <short_code> <bot_id> <file_id> <file_size> <file_name> <mime> <flac_file>
 *
 * Progress is written to storage/cache/audio/convert_<short_code>.json.
 */

require_once __DIR__ . '/backend.php';

if (PHP_SAPI !== 'cli' && PHP_SAPI !== 'phpdbg') {
    fwrite(STDERR, "This script must be run from the CLI.\n");
    exit(1);
}

$shortCode = trim((string) ($argv[1] ?? ''));
$botId     = trim((string) ($argv[2] ?? ''));
$fileId    = trim((string) ($argv[3] ?? ''));
$fileSize  = (int) ($argv[4] ?? 0);
$fileName  = trim((string) ($argv[5] ?? 'track.m4a'));
$mime      = trim((string) ($argv[6] ?? 'audio/mp4'));
$flacFile  = trim((string) ($argv[7] ?? ''));

if ($shortCode === '' || $fileId === '' || $flacFile === '') {
    fwrite(STDERR, "short_code, file_id and flac_file are required.\n");
    exit(1);
}

$safeCode = preg_replace('/[^A-Za-z0-9_-]/', '', $shortCode);
$audioCacheDir = dirname($flacFile);
if (!is_dir($audioCacheDir)) {
    @mkdir($audioCacheDir, 0777, true);
}
$rawFile = $audioCacheDir . DIRECTORY_SEPARATOR . 'raw_' . $safeCode . '.m4a';
$flacTmp = $audioCacheDir . DIRECTORY_SEPARATOR . 'grow_' . $safeCode . '.flac';

function fd_prewarm_state(string $shortCode, array $patch): void
{
    $path = fd_audio_convert_state_path($shortCode);
    $existing = [];
    if (is_file($path)) {
        $json = json_decode((string) @file_get_contents($path), true);
        if (is_array($json)) {
            $existing = $json;
        }
    }
    $merged = array_merge($existing, $patch);
    $merged['updated_at'] = time();
    @file_put_contents($path, json_encode($merged, JSON_UNESCAPED_SLASHES), LOCK_EX);
}

// A completed FLAC means there is nothing to do.
if (is_file($flacFile) && filesize($flacFile) > 1024) {
    fd_prewarm_state($shortCode, ['status' => 'done']);
    exit(0);
}

// Only one worker may download+convert a given short_code.
$lockHandle = fd_audio_convert_lock_acquire($shortCode);
if ($lockHandle === null) {
    // Another worker is already on it.
    exit(0);
}

try {
    [$ffmpegBin] = fd_ensure_audio_ffmpeg();
    if ($ffmpegBin === '' || !is_file($ffmpegBin)) {
        fd_prewarm_state($shortCode, ['status' => 'failed', 'error' => 'ffmpeg unavailable']);
        exit(1);
    }

    fd_prewarm_state($shortCode, [
        'short_code' => $shortCode,
        'status' => 'downloading',
        'pid' => getmypid(),
        'file_size' => $fileSize,
    ]);

    // Boot MadelineProto for this bot. This worker is detached, so it does not
    // compete with a web request for the session.
    [$madeline, $bootError] = fd_boot_madeline(null, [], $botId);
    if (!$madeline) {
        fd_prewarm_state($shortCode, ['status' => 'failed', 'error' => 'madeline boot failed: ' . $bootError]);
        exit(1);
    }

    $rawHandle = null;
    $writeChunk = static function (string $payload, int $offset) use (&$rawHandle): void {
        if ($payload === '' || $rawHandle === null) {
            return;
        }
        if (fseek($rawHandle, $offset) === 0) {
            fwrite($rawHandle, $payload);
            fflush($rawHandle);
        }
    };

    $openRaw = static function () use ($rawFile, &$rawHandle): void {
        @file_put_contents($rawFile, '');
        $rawHandle = @fopen($rawFile, 'r+b');
    };
    $closeRaw = static function () use (&$rawHandle): void {
        if (is_resource($rawHandle)) {
            fclose($rawHandle);
            $rawHandle = null;
        }
    };

    $currentFileId = $fileId;
    $currentSize = $fileSize > 0 ? $fileSize : 0;

    try {
        $openRaw();
        $madeline->downloadToCallable($currentFileId, $writeChunk, null, true, 0, $currentSize);
    } catch (Throwable $e) {
        $errStr = $e->getMessage();
        $isRefExpired = (stripos($errStr, 'FILE_REFERENCE_EXPIRED') !== false
            || stripos($errStr, 'refresh file reference') !== false);
        if ($isRefExpired) {
            $reResolved = fd_resolve_shortcode($shortCode, $botId, true);
            $newFileId = trim((string) ($reResolved['file_id_mt'] ?? $reResolved['file_id'] ?? ''));
            if ($newFileId !== '' && $newFileId !== $currentFileId) {
                try {
                    $closeRaw();
                    $openRaw();
                    $currentSize = (int) ($reResolved['file_size'] ?? $currentSize);
                    $madeline->downloadToCallable($newFileId, $writeChunk, null, true, 0, $currentSize);
                    $currentFileId = $newFileId;
                } catch (Throwable $e2) {
                    fd_prewarm_state($shortCode, ['status' => 'failed', 'error' => 'retry failed: ' . $e2->getMessage()]);
                }
            }
        } else {
            fd_prewarm_state($shortCode, ['status' => 'failed', 'error' => $errStr]);
        }
    }
    $closeRaw();

    if (!is_file($rawFile) || filesize($rawFile) <= 1024) {
        fd_prewarm_state($shortCode, ['status' => 'failed', 'error' => 'raw file missing or empty']);
        exit(1);
    }

    fd_prewarm_state($shortCode, [
        'status' => 'encoding',
        'raw_size' => (int) filesize($rawFile),
    ]);

    @unlink($flacTmp);

    // NOTE: no `2>&1` — with bypass_shell=true the shell is not involved, so
    // `2>&1` would be passed to FFmpeg as a literal output filename. stderr is
    // captured via the descriptor spec below.
    $ffCmd = escapeshellarg($ffmpegBin)
        . ' -y -i ' . escapeshellarg($rawFile)
        . ' -vn -c:a flac -f flac ' . escapeshellarg($flacTmp);

    $logPath = $audioCacheDir . DIRECTORY_SEPARATOR . 'ffmpeg_' . $safeCode . '.log';
    $ffDesc = [
        0 => ['pipe', 'r'],
        1 => ['file', $logPath, 'a'],
        2 => ['file', $logPath, 'a'],
    ];

    $ffProc = @proc_open($ffCmd, $ffDesc, $ffPipes, null, null, ['bypass_shell' => true]);
    if (!is_resource($ffProc)) {
        fd_prewarm_state($shortCode, ['status' => 'failed', 'error' => 'ffmpeg spawn failed']);
        exit(1);
    }
    if (isset($ffPipes[0]) && is_resource($ffPipes[0])) {
        fclose($ffPipes[0]);
    }

    $deadline = microtime(true) + 900; // 15 min hard cap
    while (microtime(true) < $deadline) {
        $st = proc_get_status($ffProc);
        if (!$st['running']) {
            break;
        }
        usleep(300000);
    }
    $exitCode = proc_close($ffProc);

    if (is_file($flacTmp) && filesize($flacTmp) > 1024) {
        @rename($flacTmp, $flacFile);
        @unlink($rawFile);
        fd_prewarm_state($shortCode, [
            'status' => 'done',
            'flac_size' => (int) filesize($flacFile),
            'ffmpeg_exit' => $exitCode,
        ]);
        exit(0);
    }

    fd_prewarm_state($shortCode, [
        'status' => 'failed',
        'error' => 'ffmpeg produced no output',
        'ffmpeg_exit' => $exitCode,
    ]);
    exit(1);
} finally {
    if ($lockHandle !== null) {
        @flock($lockHandle, LOCK_UN);
        @fclose($lockHandle);
    }
}
