<?php

/**
 * Local-only catalog ID backfill.
 *
 * Walks the WordPress catalog and populates external IDs (`_media_ids` postmeta
 * + Manticore `media_ids_idx`) for posts that do not have them yet, so future
 * users can resolve `tmdb:`, `kitsu:`, `mal:`, `anilist:`, `tvdb:` IDs WITHOUT
 * configuring upstream Stremio addons.
 *
 * This script is meant to run ONLY on the operator's own machine. It is a CLI
 * tool and is never exposed over HTTP. It talks to the WordPress REST API
 * (which owns the Manticore connection), so no local Manticore is required.
 *
 * Usage:
 *   php backfill-ids.php                 # process the next batch (resumable)
 *   php backfill-ids.php --limit=50      # batch size (default 25)
 *   php backfill-ids.php --dry-run       # show what would be scraped
 *   php backfill-ids.php --reindex-only  # only re-index posts that already have IDs
 *   php backfill-ids.php --reset         # clear progress and start over
 *   php backfill-ids.php --status        # show progress
 *
 * Progress is stored in storage/backfill_ids_state.json so runs can resume.
 */

declare(strict_types=1);

if (PHP_SAPI !== 'cli') {
    http_response_code(403);
    exit('CLI only');
}

require_once __DIR__ . DIRECTORY_SEPARATOR . 'backend.php';

// ── Options ──────────────────────────────────────────────────────────────────
$opts = getopt('', ['limit::', 'dry-run', 'reindex-only', 'reset', 'status', 'sleep::']);
$limit = isset($opts['limit']) ? max(1, (int) $opts['limit']) : 25;
$dryRun = array_key_exists('dry-run', $opts);
$reindexOnly = array_key_exists('reindex-only', $opts);
$reset = array_key_exists('reset', $opts);
$showStatus = array_key_exists('status', $opts);
$sleepMs = isset($opts['sleep']) ? max(0, (int) $opts['sleep']) : 800;

$stateFile = fd_storage_path('storage/backfill_ids_state.json');

function backfill_load_state(string $file): array
{
    if (is_file($file)) {
        $data = json_decode((string) @file_get_contents($file), true);
        if (is_array($data)) {
            return $data;
        }
    }
    return [
        'last_post_id' => 0,
        'processed'    => 0,
        'updated'      => 0,
        'skipped'      => 0,
        'failed'       => 0,
        'started_at'   => time(),
        'updated_at'   => time(),
    ];
}

function backfill_save_state(string $file, array $state): void
{
    $state['updated_at'] = time();
    @file_put_contents($file, json_encode($state, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES), LOCK_EX);
}

$state = backfill_load_state($stateFile);

if ($reset) {
    @unlink($stateFile);
    echo "Progress reset.\n";
    exit(0);
}

if ($showStatus) {
    echo "Backfill status:\n";
    echo json_encode($state, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES) . "\n";
    exit(0);
}

/**
 * Fetch the next batch of catalog posts that still need IDs, via WordPress.
 *
 * @return array<int,array{id:int,title:string,year:int,imdb_id:string}>
 */
function backfill_next_batch(int $afterId, int $limit, bool $reindexOnly): array
{
    $resp = fd_http_json(FD_WP_API_BASE . '/catalog-ids', [
        'after_id'     => $afterId,
        'limit'        => $limit,
        'reindex_only' => $reindexOnly ? 1 : 0,
    ], 'GET', 20);

    if (empty($resp['ok']) || empty($resp['items']) || !is_array($resp['items'])) {
        return [];
    }

    $rows = [];
    foreach ($resp['items'] as $item) {
        $rows[] = [
            'id'      => (int) ($item['id'] ?? 0),
            'title'   => (string) ($item['title'] ?? ''),
            'year'    => (int) ($item['year'] ?? 0),
            'imdb_id' => (string) ($item['imdb_id'] ?? ''),
        ];
    }
    return $rows;
}

/**
 * Strip the "• Movie" / "• TvSeries" suffix, release tags, and trailing year
 * from a post title so it can be used as a clean scrape query.
 */
function backfill_clean_title(string $title, int $year): string
{
    $t = preg_replace('/\s*[•·]\s*(Movie|TvSeries|TVSeries|Series|TV|TvMovie|TvMiniSeries).*$/iu', '', $title);
    $t = html_entity_decode((string) $t, ENT_QUOTES | ENT_HTML5, 'UTF-8');

    // Drop release/quality/size noise commonly present in catalog titles.
    $noise = [
        '/\b(2160p|1080p|720p|480p|360p|4k|uhd|fhd|hd|sd)\b/iu',
        '/\b(bluray|blu-ray|brrip|bdrip|web-?dl|webrip|hdtv|dvdrip|remux|hdrip)\b/iu',
        '/\b(x264|x265|h264|h265|hevc|avc|aac|ac3|dts|ddp?5\.?1)\b/iu',
        '/\b(hdr10\+?|hdr|dolby\s*vision|dovi|dv)\b/iu',
        '/\b\d+(?:\.\d+)?\s*(?:gb|mb|kb)\b/iu',
        '/\b(dual\s*audio|multi\s*audio|subs?|subtitle|malay|indo|tamil|hindi|english)\b/iu',
        '/\b(part\s*\d+|complete|season\s*\d+|s\d{1,2}(?:e\d{1,3})?)\b/iu',
    ];
    foreach ($noise as $pattern) {
        $t = preg_replace($pattern, ' ', (string) $t);
    }

    // Remove bracketed/parenthesised tag groups like [720p] or (2024)
    $t = preg_replace('/[\[\(\{][^\]\)\}]*[\]\)\}]/u', ' ', (string) $t);
    $t = preg_replace('/[._]+/', ' ', (string) $t);
    $t = trim(preg_replace('/\s+/', ' ', (string) $t));

    if ($year > 0) {
        $t = trim(preg_replace('/\b' . $year . '\b/', '', $t));
    }

    // Trim stray separators left behind
    $t = trim((string) $t, " -–—:.,|/\\");
    return trim(preg_replace('/\s+/', ' ', $t));
}

// ── Main ─────────────────────────────────────────────────────────────────────
$batch = backfill_next_batch((int) $state['last_post_id'], $limit, $reindexOnly);
if (empty($batch)) {
    echo "No more posts to process.\n";
    echo json_encode($state, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES) . "\n";
    exit(0);
}

echo "Processing " . count($batch) . " post(s) after id {$state['last_post_id']}...\n";

foreach ($batch as $row) {
    $postId = $row['id'];
    $title = $row['title'];
    $year = $row['year'];
    $state['last_post_id'] = $postId;
    $state['processed']++;

    $query = backfill_clean_title($title, $year);
    if ($query === '' || mb_strlen($query) < 2) {
        $state['skipped']++;
        echo "  - #{$postId} skipped (empty title)\n";
        continue;
    }

    if ($dryRun) {
        echo "  ? #{$postId} would scrape: \"{$query}\"\n";
        continue;
    }

    // Re-index only: the post already has an ID, just refresh the index rows.
    if ($reindexOnly) {
        $resp = fd_http_json(FD_WP_API_BASE . '/scrape', [
            'id' => $row['imdb_id'] !== '' ? $row['imdb_id'] : $query,
        ], 'POST', 20);
        if (!empty($resp['ok'])) {
            $state['updated']++;
            echo "  ↻ #{$postId} re-indexed\n";
        } else {
            $state['failed']++;
            echo "  ✗ #{$postId} re-index failed\n";
        }
        if ($sleepMs > 0) {
            usleep($sleepMs * 1000);
        }
        continue;
    }

    // Scrape by title to discover and store IDs.
    // `force` bypasses the "Already indexed" short-circuit, and `post_id` makes
    // the scraper UPDATE this exact catalog post instead of creating a duplicate
    // from the cleaned title.
    $resp = fd_http_json(FD_WP_API_BASE . '/scrape', [
        'query'   => $query,
        'force'   => 1,
        'post_id' => $postId,
    ], 'POST', 30);

    if (!empty($resp['ok'])) {
        $state['updated']++;
        $msg = (string) ($resp['message'] ?? '');
        echo "  ✓ #{$postId} \"{$query}\" -> {$msg}\n";
    } else {
        $state['failed']++;
        $msg = (string) ($resp['message'] ?? ($resp['error'] ?? 'no result'));
        echo "  ✗ #{$postId} \"{$query}\" -> {$msg}\n";
    }

    if ($sleepMs > 0) {
        usleep($sleepMs * 1000);
    }
}

backfill_save_state($stateFile, $state);

echo "\nBatch done.\n";
echo json_encode($state, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES) . "\n";
