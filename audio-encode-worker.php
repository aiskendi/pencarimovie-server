<?php

declare(strict_types=1);

/**
 * Detached CLI worker: transcode an already-downloaded ALAC `.m4a` to lossless
 * FLAC with FFmpeg.
 *
 * This worker does NOT boot MadelineProto. The web request downloads the raw
 * file itself (it already holds a working MadelineProto instance) and then
 * spawns this worker to run FFmpeg only. Booting a second MadelineProto in the
 * worker makes two IPC clients compete for the same session, which stalls
 * downloadToCallable and leaves the raw file at 0 bytes.
 *
 * Usage:
 *   php audio-encode-worker.php <short_code> <raw_file> <flac_file>
 *
 * Progress is written to storage/cache/audio/convert_<short_code>.json so the
 * web request can wait for completion.
 */

require_once __DIR__ . '/backend.php';

if (PHP_SAPI !== 'cli' && PHP_SAPI !== 'phpdbg') {
    fwrite(STDERR, "This script must be run from the CLI.\n");
    exit(1);
}

$shortCode = trim((string) ($argv[1] ?? ''));
$rawFile   = trim((string) ($argv[2] ?? ''));
$flacFile  = trim((string) ($argv[3] ?? ''));

if ($shortCode === '' || $rawFile === '' || $flacFile === '') {
    fwrite(STDERR, "short_code, raw_file and flac_file are required.\n");
    exit(1);
}

$safeCode = preg_replace('/[^A-Za-z0-9_-]/', '', $shortCode);
$audioCacheDir = dirname($flacFile);
$flacTmp = $audioCacheDir . DIRECTORY_SEPARATOR . 'grow_' . $safeCode . '.flac';

function fd_encode_state(string $shortCode, array $patch): void
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

if (!is_file($rawFile) || filesize($rawFile) <= 1024) {
    fd_encode_state($shortCode, ['status' => 'failed', 'error' => 'raw file missing or empty']);
    exit(1);
}

[$ffmpegBin] = fd_ensure_audio_ffmpeg();
if ($ffmpegBin === '' || !is_file($ffmpegBin)) {
    fd_encode_state($shortCode, ['status' => 'failed', 'error' => 'ffmpeg unavailable']);
    exit(1);
}

@unlink($flacTmp);

// NOTE: no `2>&1` — with bypass_shell=true the shell is not involved, so `2>&1`
// would be passed to FFmpeg as a literal output filename. stderr is captured
// via the descriptor spec below.
$ffCmd = escapeshellarg($ffmpegBin)
    . ' -y -i ' . escapeshellarg($rawFile)
    . ' -vn -c:a flac -f flac ' . escapeshellarg($flacTmp);

$logPath = $audioCacheDir . DIRECTORY_SEPARATOR . 'ffmpeg_' . $safeCode . '.log';
$ffDesc = [
    0 => ['pipe', 'r'],
    1 => ['file', $logPath, 'a'],
    2 => ['file', $logPath, 'a'],
];

fd_encode_state($shortCode, [
    'short_code' => $shortCode,
    'status' => 'encoding',
    'pid' => getmypid(),
    'raw_size' => (int) filesize($rawFile),
]);

$ffProc = @proc_open($ffCmd, $ffDesc, $ffPipes, null, null, ['bypass_shell' => true]);
if (!is_resource($ffProc)) {
    fd_encode_state($shortCode, ['status' => 'failed', 'error' => 'ffmpeg spawn failed']);
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
    fd_encode_state($shortCode, [
        'status' => 'done',
        'flac_size' => (int) filesize($flacFile),
        'ffmpeg_exit' => $exitCode,
    ]);
    exit(0);
}

// A competing worker may have already published the FLAC while we were
// encoding. Never overwrite a good result with a failure.
if (is_file($flacFile) && filesize($flacFile) > 1024) {
    fd_encode_state($shortCode, [
        'status' => 'done',
        'flac_size' => (int) filesize($flacFile),
        'ffmpeg_exit' => $exitCode,
    ]);
    exit(0);
}

fd_encode_state($shortCode, [
    'status' => 'failed',
    'error' => 'ffmpeg produced no output',
    'ffmpeg_exit' => $exitCode,
]);
exit(1);
