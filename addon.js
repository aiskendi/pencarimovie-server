const isBun = typeof Bun !== 'undefined';
const http = !isBun ? await import('node:http') : null;
const dns = await import('node:dns');
const fs = await import('node:fs');
const crypto = await import('node:crypto');

// Force DNS over Cloudflare and Google (eliminates Termux/Android/Windows DNS blocks)
try {
  dns.setServers(['1.1.1.1', '8.8.8.8', '1.0.0.1', '8.8.4.4']);
  console.log('[Addon] DNS initialized with 1.1.1.1, 8.8.8.8');
} catch (e) {
  console.warn('[Addon] DNS setup warning:', e);
}

const PORT = parseInt(process.env.ADDON_PORT || '8089', 10);
// The Addon is a local-only helper/proxy daemon (backend.php and app.js reach it
// via 127.0.0.1:8089). It must NOT be exposed to the LAN/internet, so bind to
// loopback by default. Override with ADDON_HOST if a different bind is needed.
const HOST = process.env.ADDON_HOST || '127.0.0.1';
const PHP_PORT = parseInt(process.env.PORT || '8088', 10);
const WP_AJAX_URL = 'https://pencarimovie.com/wp-admin/admin-ajax.php';
const WP_API_BASE = 'https://pencarimovie.com/wp-json/pencarimovie-server/v1';

// ── Storage directory (shared with backend.php) ─────────────────────────────
// The addon is launched with cwd = app root (same dir that holds storage/).
const STORAGE_DIR = process.env.STORAGE_DIR || 'storage';

// ── In-memory TTL cache ─────────────────────────────────────────────────────
// Keys are namespaced (e.g. 'manifest', 'catalog:movie:pm_movies_latest', 'resolve:CODE:BOT').
const cache = new Map();
const CACHE_TTL = {
  manifest: 60 * 60 * 1000,      // 1 hour
  catalog: 5 * 60 * 1000,        // 5 minutes
  resolve: 60 * 60 * 1000,       // 1 hour (matches PHP 86400s disk cache intent)
  proxy: 60 * 1000,              // 1 minute for generic proxied GETs
};

function cacheGet(key) {
  const hit = cache.get(key);
  if (!hit) return null;
  if (Date.now() - hit.ts > hit.ttl) {
    cache.delete(key);
    return null;
  }
  return hit.value;
}

function cacheSet(key, value, ttl) {
  cache.set(key, { value, ts: Date.now(), ttl });
}

// ── Shared on-disk cache helpers (mirror backend.php formats) ──────────────
// backend.php writes resolve_cache_<md5(shortCode:botId)>.json and reads
// bot_id.txt as a legacy fallback. Writing these from JS lets PHP reuse the
// result on later /api/download calls even when resolution happened here.

function md5hex(str) {
  // Use node:crypto MD5 (available in both Node and Bun) so the hash exactly
  // matches backend.php's md5($shortCode . ':' . $botId) disk cache filenames.
  return crypto.createHash('md5').update(String(str), 'utf8').digest('hex');
}

function storagePath(name) {
  return `${STORAGE_DIR}/${name}`;
}

function writeFileAtomic(path, data) {
  try {
    fs.mkdirSync(STORAGE_DIR, { recursive: true });
    fs.writeFileSync(path, data);
    return true;
  } catch (e) {
    console.warn('[Addon] writeFile failed:', path, e.message);
    return false;
  }
}

function readFileIfFresh(path, maxAgeMs) {
  try {
    if (!fs.existsSync(path)) return null;
    const stat = fs.statSync(path);
    if (maxAgeMs > 0 && Date.now() - stat.mtimeMs > maxAgeMs) return null;
    return fs.readFileSync(path, 'utf8');
  } catch (_) {
    return null;
  }
}

// Persist a resolved file to PHP's shared disk cache (resolve_cache_<md5>.json)
// and update bot_id.txt so PHP's fd_get_bot_id() fallback stays in sync.
function persistResolveCache(shortCode, botId, data) {
  if (!shortCode || !botId) return;
  const cacheKey = `${shortCode}:${botId}`;
  const file = storagePath(`resolve_cache_${md5hex(cacheKey)}.json`);
  const payload = JSON.stringify(data);
  if (writeFileAtomic(file, payload)) {
    console.log(`[Addon] Cached resolve ${shortCode} -> ${file}`);
  }
  // Keep bot_id.txt in sync (legacy fallback used by backend.php)
  const botIdFile = storagePath('bot_id.txt');
  const existing = readFileIfFresh(botIdFile, 0); // read regardless of age
  if (existing === null || existing.trim() !== String(botId)) {
    writeFileAtomic(botIdFile, String(botId) + '\n');
  }
}

const CORS_HEADERS = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, X-Requested-With, X-App-Version, X-API-Secret, Range',
  'Access-Control-Expose-Headers': 'X-Min-Version, X-Update-Url, X-Update-Required',
};

function jsonResponse(data, status = 200, cacheControl = 'max-age=600, public') {
  return new Response(JSON.stringify(data), {
    status,
    headers: {
      ...CORS_HEADERS,
      'Content-Type': 'application/json; charset=utf-8',
      'Cache-Control': cacheControl,
    },
  });
}

function cleanMediaTitle(title = '') {
  return String(title)
    .replace(/^\[[^\]]*\]\s*/g, '')
    .replace(/\s*-\s*by\s+[a-z0-9_.]+/gi, '')
    .replace(/\s*@\w+/g, '')
    .trim();
}

function cleanPostTitle(title = '') {
  let t = cleanMediaTitle(title);
  t = t.replace(/\s*[•·]\s*(?:TvSeries|Movie|Series|Drama)\s*$/i, '');
  return t.trim();
}

function extractYear(title = '', date = '') {
  const m = title.match(/\b(19\d\d|20\d\d)\b/);
  if (m) return m[1];
  if (date) {
    const dm = date.match(/\b(19\d\d|20\d\d)\b/);
    if (dm) return dm[1];
  }
  return '';
}

function guessMime(filename = '', rawMime = '') {
  if (rawMime && rawMime.includes('/')) return rawMime;
  const lower = filename.toLowerCase();
  if (lower.endsWith('.mkv')) return 'video/x-matroska';
  if (lower.endsWith('.mp4')) return 'video/mp4';
  if (lower.endsWith('.avi')) return 'video/x-msvideo';
  if (lower.endsWith('.webm')) return 'video/webm';
  return 'video/mp4';
}

function extractSplitPartInfo(filename = '', caption = '') {
  const f = String(filename).trim();
  const cap = String(caption).trim();
  let partNum = 0;
  let totalParts = 0;
  let matched = false;
  let cleanBase = f;

  let m = f.match(/[._\s-]part[._\s-]*0*(\d{1,4})(?:[._\s\/-]+(?:of[._\s-]+)?0*(\d{1,4}))?/i);
  if (m) {
    partNum = parseInt(m[1], 10);
    if (m[2]) totalParts = parseInt(m[2], 10);
    matched = true;
    cleanBase = f.replace(/[._\s-]part[._\s-]*0*\d{1,4}(?:[._\s\/-]+(?:of[._\s-]+)?0*\d{1,4})?/i, '');
  } else if ((m = f.match(/[._\s-]0*(\d{1,3})\.(mp4|mkv|avi|webm)$/i)) && !f.match(/\b(2160p|1080p|720p|480p|360p)\b/i)) {
    partNum = parseInt(m[1], 10);
    matched = true;
    cleanBase = f.replace(new RegExp(`[._\\s-]0*${m[1]}\\.${m[2]}$`, 'i'), `.${m[2]}`);
  } else if ((m = f.match(/\.(?:mp4|mkv|avi|webm)\.0*(\d{1,4})$/i))) {
    partNum = parseInt(m[1], 10);
    matched = true;
    cleanBase = f.replace(new RegExp(`\\.0*${m[1]}$`, 'i'), '');
  }

  if (matched && totalParts === 0 && cap) {
    const cm = cap.match(new RegExp(`part[._\\s-]*0*${partNum}\\s*[\\/|of]\\s*0*(\\d{1,4})`, 'i'));
    if (cm) totalParts = parseInt(cm[1], 10);
  }

  if (!matched || partNum <= 0) {
    return { isPart: false, baseKey: '', cleanBase: f, partNum: 0, totalParts: 0 };
  }

  const baseKey = cleanBase.toLowerCase().replace(/[^\p{L}\p{N}]+/gu, '.').replace(/^\.+|\.+$/g, '');
  return { isPart: true, baseKey, cleanBase, partNum, totalParts };
}

function groupSplitParts(files = []) {
  // Sort split parts in ascending order so they display sequentially
  const list = [...files];
  list.sort((a, b) => {
    const aInfo = extractSplitPartInfo(a.title || '', a.caption || '');
    const bInfo = extractSplitPartInfo(b.title || '', b.caption || '');
    if (aInfo.isPart && bInfo.isPart && aInfo.baseKey === bInfo.baseKey) {
      return aInfo.partNum - bInfo.partNum;
    }
    return 0;
  });

  return list.map(f => {
    const info = extractSplitPartInfo(f.title || '', f.caption || '');
    if (info.isPart) {
      return {
        ...f,
        is_split_part: true,
        part_num: info.partNum,
        total_parts: info.totalParts,
        clean_base: info.cleanBase
      };
    }
    return f;
  });
}

const ALL_GENRES = [
  'Action', 'Adventure', 'Animation', 'Anime', 'Biography', 'Comedy', 'Crime',
  'Documentary', 'Drama', 'Family', 'Fantasy', 'History', 'Horror', 'Music',
  'Musical', 'Mystery', 'Romance', 'Sci-Fi', 'Sport', 'Thriller', 'War', 'Western'
];

const MANIFEST_CATALOGS = [
  {
    type: 'movie',
    id: 'pm_movies_latest',
    name: 'Latest Releases',
    genres: ALL_GENRES,
    extra: [{ name: 'genre', options: ALL_GENRES, isRequired: false }, { name: 'skip', isRequired: false }]
  },
  {
    type: 'movie',
    id: 'pm_search_movie',
    name: 'Search Movies',
    genres: ALL_GENRES,
    extra: [{ name: 'search', isRequired: true }, { name: 'genre', options: ALL_GENRES, isRequired: false }, { name: 'skip', isRequired: false }]
  },
  {
    type: 'movie',
    id: 'pm_movies_malay',
    name: 'Malaysia',
    genres: ALL_GENRES,
    extra: [{ name: 'genre', options: ALL_GENRES, isRequired: false }, { name: 'skip', isRequired: false }]
  },
  {
    type: 'movie',
    id: 'pm_movies_korean',
    name: 'Korea',
    genres: ALL_GENRES,
    extra: [{ name: 'genre', options: ALL_GENRES, isRequired: false }, { name: 'skip', isRequired: false }]
  },
  {
    type: 'series',
    id: 'pm_series_latest',
    name: 'Latest Releases',
    genres: ALL_GENRES,
    extra: [{ name: 'genre', options: ALL_GENRES, isRequired: false }, { name: 'skip', isRequired: false }]
  },
  {
    type: 'series',
    id: 'pm_search_series',
    name: 'Search Series',
    genres: ALL_GENRES,
    extra: [{ name: 'search', isRequired: true }, { name: 'genre', options: ALL_GENRES, isRequired: false }, { name: 'skip', isRequired: false }]
  },
  {
    type: 'series',
    id: 'pm_series_kdrama',
    name: 'K-Drama',
    genres: ALL_GENRES,
    extra: [{ name: 'genre', options: ALL_GENRES, isRequired: false }, { name: 'skip', isRequired: false }]
  },
  {
    type: 'series',
    id: 'pm_series_malay',
    name: 'Malaysia',
    genres: ALL_GENRES,
    extra: [{ name: 'genre', options: ALL_GENRES, isRequired: false }, { name: 'skip', isRequired: false }]
  }
];

async function handleRequest(req) {
    const url = new URL(req.url);
    const path = url.pathname;

    if (req.method === 'OPTIONS') {
      return new Response(null, { status: 204, headers: CORS_HEADERS });
    }

    // ── Root / Health check (http://127.0.0.1:8089/) ──
    if (path === '/' || path === '') {
      return jsonResponse({
        ok: true,
        service: 'PencariMovie Addon & DNS Proxy Engine',
        version: '1.7.0',
        port: PORT,
        endpoints: [
          '/manifest.json',
          '/resolve-file',
          '/proxy'
        ]
      });
    }

    // ── 0. HTTP Proxy for WordPress calls (/proxy?url=...) ──
    if (path === '/proxy') {
      const targetUrl = url.searchParams.get('url');
      if (!targetUrl || (!targetUrl.startsWith('https://pencarimovie.com') && !targetUrl.startsWith('https://v3-cinemeta.strem.io'))) {
        return jsonResponse({ ok: 0, message: 'Invalid target URL' }, 400);
      }

      try {
        const fetchHeaders = {
          'User-Agent': 'pencarimovie-server/1.7.0',
          'X-App-Version': '1.7.0',
        };
        const fwdHeaders = ['content-type', 'x-api-secret', 'x-requested-with'];
        for (const h of fwdHeaders) {
          if (req.headers.has(h)) fetchHeaders[h] = req.headers.get(h);
        }

        const body = req.method !== 'GET' && req.method !== 'HEAD' ? await req.text() : undefined;
        const resp = await fetch(targetUrl, {
          method: req.method,
          headers: fetchHeaders,
          body,
        });

        const respText = await resp.text();
        const respHeaders = { ...CORS_HEADERS };
        if (resp.headers.has('content-type')) {
          respHeaders['content-type'] = resp.headers.get('content-type');
        }

        return new Response(respText, { status: resp.status, headers: respHeaders });
      } catch (err) {
        return jsonResponse({ ok: 0, message: 'Proxy request failed: ' + err.message }, 502);
      }
    }

    // ── 0.1 Dedicated JS short_code resolution endpoint ──
    if (path === '/resolve-file' || path === '/api/resolve-file') {
      const shortCode = url.searchParams.get('short_code') || '';
      let botId = url.searchParams.get('bot_id') || '';
      if (!shortCode) {
        return jsonResponse({ ok: 0, message: 'short_code is required' }, 400);
      }

      // If no bot_id supplied, fall back to the cached bot_id.txt (shared with PHP)
      if (!botId) {
        const botIdRaw = readFileIfFresh(storagePath('bot_id.txt'), 0);
        if (botIdRaw) botId = botIdRaw.trim();
      }

      // 1. In-memory TTL cache hit
      const memKey = `resolve:${shortCode}:${botId}`;
      const memHit = cacheGet(memKey);
      if (memHit) {
        return jsonResponse(memHit, 200);
      }

      // 2. Shared on-disk cache hit (written by PHP or a previous JS resolve)
      if (botId) {
        const diskFile = storagePath(`resolve_cache_${md5hex(`${shortCode}:${botId}`)}.json`);
        const diskRaw = readFileIfFresh(diskFile, 86400 * 1000); // 24h, matches PHP
        if (diskRaw) {
          try {
            const diskData = JSON.parse(diskRaw);
            if (diskData && (diskData.file_id_mt || diskData.file_id)) {
              cacheSet(memKey, diskData, CACHE_TTL.resolve);
              // Keep bot_id.txt in sync (idempotent — only writes if it differs)
              persistResolveCache(shortCode, botId, diskData);
              return jsonResponse(diskData, 200);
            }
          } catch (_) {}
        }
      }

      try {
        const wpUrl = new URL(`${WP_API_BASE}/resolve-file`);
        wpUrl.searchParams.set('short_code', shortCode);
        if (botId) wpUrl.searchParams.set('bot_id', botId);

        const fetchHeaders = {
          'User-Agent': 'pencarimovie-server/1.7.0',
          'X-App-Version': '1.7.0',
        };
        if (req.headers.has('x-api-secret')) {
          fetchHeaders['X-API-Secret'] = req.headers.get('x-api-secret');
        }

        const wpRes = await fetch(wpUrl.toString(), { headers: fetchHeaders });
        const wpData = await wpRes.json();

        // Cache successful resolutions in memory AND on disk (shared with PHP)
        if (wpData && (wpData.file_id_mt || wpData.file_id)) {
          cacheSet(memKey, wpData, CACHE_TTL.resolve);
          const resolvedBotId = String(wpData.bot_id || botId || '');
          if (resolvedBotId) {
            persistResolveCache(shortCode, resolvedBotId, wpData);
          }
        }
        return jsonResponse(wpData, wpRes.status);
      } catch (err) {
        return jsonResponse({ ok: 0, message: 'File resolve failed: ' + err.message }, 502);
      }
    }

    // ── 1. /manifest.json (Stremio / Nuvio Manifest) ──
    if (path === '/manifest.json' || path === '/stremio/manifest.json' || path === '/nuvio/manifest.json') {
      const manifest = {
        id: 'org.pencarimovie.addon',
        version: '1.7.0',
        name: 'PencariMovie (JS Engine)',
        description: 'Fast local Telegram movie & series downloader powered by PencariMovie and MadelineProto',
        resources: [
          { name: 'catalog', types: ['movie', 'series'] },
          { name: 'meta', types: ['movie', 'series'], idPrefixes: ['pm_', 'pm:'] },
          { name: 'stream', types: ['movie', 'series'], idPrefixes: ['pm_', 'pm:', 'tt'] }
        ],
        types: ['movie', 'series'],
        idPrefixes: ['pm_', 'pm:', 'tt'],
        catalogs: MANIFEST_CATALOGS,
        behaviorHints: { configurable: false, adult: false, p2p: false }
      };
      cacheSet('manifest', manifest, CACHE_TTL.manifest);
      return jsonResponse(manifest, 200, 'max-age=3600, public');
    }

    // ── 2. /catalog/:type/:id[/:extra].json (Catalog & Search) ──
    const catMatch = path.match(/^\/(?:stremio\/|nuvio\/)?catalog\/([^/]+)\/([^/]+?)(?:\/(.*))?\.json$/);
    if (catMatch) {
      const type = catMatch[1];
      const catalogId = catMatch[2];
      const extraStr = catMatch[3] || '';

      const searchParam = url.searchParams.get('search') || '';
      const genreParam = url.searchParams.get('genre') || '';
      const skipParam = parseInt(url.searchParams.get('skip') || '0', 10);

      // Cache key includes the full catalog URL so search/skip/genre variants stay distinct
      const catKey = `catalog:${url.pathname}?${url.search}`;
      const catHit = cacheGet(catKey);
      if (catHit) {
        return jsonResponse(catHit, 200);
      }

      try {
        let wpAction = 'stream_posts';
        let queryParams = { limit: '24', offset: String(skipParam), media_type: type };

        if (searchParam || catalogId.includes('search')) {
          wpAction = 'stream_search';
          queryParams.search = searchParam || '';
          queryParams.limit = '30';
        } else if (catalogId.includes('malay')) {
          queryParams.category = 'malay';
        } else if (catalogId.includes('korean') || catalogId.includes('kdrama')) {
          queryParams.category = 'korean';
        }

        if (genreParam) queryParams.genre = genreParam;

        const wpUrl = new URL(WP_AJAX_URL);
        wpUrl.searchParams.set('action', wpAction);
        for (const [k, v] of Object.entries(queryParams)) {
          wpUrl.searchParams.set(k, v);
        }

        const wpRes = await fetch(wpUrl.toString(), {
          headers: { 'X-Requested-With': 'XMLHttpRequest' }
        });
        const wpData = await wpRes.json();
        const posts = Array.isArray(wpData?.data) ? wpData.data : (Array.isArray(wpData) ? wpData : []);

        const metas = posts.map(p => {
          const pTitle = p.title || '';
          return {
            id: `pm:post:${p.id}`,
            type: type === 'series' ? 'series' : 'movie',
            name: cleanPostTitle(pTitle),
            poster: p.thumbnail || p.thumbnail_url || '',
            posterShape: 'poster',
            description: p.excerpt || pTitle,
            releaseInfo: extractYear(pTitle, p.date || ''),
          };
        });

        const result = { metas };
        cacheSet(catKey, result, CACHE_TTL.catalog);
        return jsonResponse(result, 200);
      } catch (err) {
        console.error('[Bun Addon] Catalog fetch error:', err);
        return jsonResponse({ metas: [] });
      }
    }

    // ── 3. /meta/:type/:id.json (Post & File Metadata) ──
    const metaMatch = path.match(/^\/(?:stremio\/|nuvio\/)?meta\/([^/]+)\/([^/]+?)(?:\.json)?$/);
    if (metaMatch) {
      const type = decodeURIComponent(metaMatch[1]);
      const rawId = decodeURIComponent(metaMatch[2]);

      try {
        if (rawId.startsWith('pm:post:')) {
          const postId = rawId.replace('pm:post:', '');
          const wpUrl = new URL(WP_AJAX_URL);
          wpUrl.searchParams.set('action', 'stream_get_post');
          wpUrl.searchParams.set('post_id', postId);

          const wpRes = await fetch(wpUrl.toString(), {
            headers: { 'X-Requested-With': 'XMLHttpRequest' }
          });
          const wpData = await wpRes.json();
          const post = wpData?.data || wpData;

          if (post && post.id) {
            // Also fetch attached files for episode list if series
            let videos = undefined;
            if (type === 'series') {
              const filesUrl = new URL(WP_AJAX_URL);
              filesUrl.searchParams.set('action', 'stream_post_files');
              filesUrl.searchParams.set('post_id', postId);
              filesUrl.searchParams.set('limit', '100');
              const fRes = await fetch(filesUrl.toString(), { headers: { 'X-Requested-With': 'XMLHttpRequest' } });
              const fData = await fRes.json();
              const files = fData?.data?.files || fData?.files || [];

              videos = files.map((f, idx) => {
                const epNum = f.episode_num || idx + 1;
                const seasonNum = f.season_num || 1;
                return {
                  id: `pm:post:${postId}:${seasonNum}:${epNum}`,
                  title: f.title || `Episode ${epNum}`,
                  episode: epNum,
                  season: seasonNum,
                  released: new Date().toISOString()
                };
              });
            }

            const pTitle = post.title || '';
            const meta = {
              id: `pm:post:${post.id}`,
              type,
              name: cleanPostTitle(pTitle),
              poster: post.thumbnail_url || post.thumbnail || '',
              posterShape: 'poster',
              description: post.excerpt || post.content || pTitle,
              releaseInfo: extractYear(pTitle, post.date || ''),
              videos
            };
            return jsonResponse({ meta });
          }
        }
      } catch (err) {
        console.error('[Bun Addon] Meta fetch error:', err);
      }
      return jsonResponse({ meta: null }, 404);
    }

    // ── 4. /stream/:type/:id.json (Stream Link Generation) ──
    const streamMatch = path.match(/^\/(?:stremio\/|nuvio\/)?stream\/([^/]+)\/([^/]+?)(?:\.json)?$/);
    if (streamMatch) {
      const type = decodeURIComponent(streamMatch[1]);
      const rawId = decodeURIComponent(streamMatch[2]);

      // Cache stream link lists per post/episode (static metadata, 5 min TTL)
      const streamKey = `stream:${rawId}`;
      const streamHit = cacheGet(streamKey);
      if (streamHit) {
        return jsonResponse(streamHit, 200);
      }

      try {
        let postId = '';
        let targetSeason = null;
        let targetEpisode = null;

        if (rawId.startsWith('pm:post:')) {
          const parts = rawId.replace('pm:post:', '').split(':');
          postId = parts[0];
          if (parts.length >= 3) {
            targetSeason = parseInt(parts[1], 10);
            targetEpisode = parseInt(parts[2], 10);
          }
        }

        if (postId) {
          const wpUrl = new URL(WP_AJAX_URL);
          wpUrl.searchParams.set('action', 'stream_post_files');
          wpUrl.searchParams.set('post_id', postId);
          wpUrl.searchParams.set('limit', '100');

          const wpRes = await fetch(wpUrl.toString(), {
            headers: { 'X-Requested-With': 'XMLHttpRequest' }
          });
          const wpData = await wpRes.json();
          let files = wpData?.data?.files || wpData?.files || [];

          if (targetEpisode !== null) {
            const epFiltered = files.filter(f => f.episode_num === targetEpisode);
            if (epFiltered.length > 0) files = epFiltered;
          }

          // Combine split parts (part001, part002, ...)
          const groupedFiles = groupSplitParts(files);

          const streams = groupedFiles.map(f => {
            const fileName = f.title || `${f.short_code}.mp4`;
            const mime = guessMime(fileName, f.mime || f.file_type);
            const payload = {
              short_code: f.short_code,
              file_size: f.file_size || 0,
              file_name: fileName,
              mime,
              bot_id: f.bot_id || null
            };
            const d = Buffer.from(JSON.stringify(payload)).toString('base64url');
            const streamUrl = `http://127.0.0.1:${PHP_PORT}/api/download/${d}/${encodeURIComponent(fileName)}`;

            const sizeMb = f.file_size ? `${(f.file_size / (1024 * 1024)).toFixed(1)} MB` : 'Stream';
            let partBadge = '';
            if (f.is_split_part) {
              const pNum = String(f.part_num).padStart(2, '0');
              const totalStr = f.total_parts ? `/${String(f.total_parts).padStart(2, '0')}` : '';
              partBadge = ` [Part ${pNum}${totalStr}]`;
            }
            return {
              name: 'PencariMovie',
              title: `⚡ ${cleanMediaTitle(fileName)}${partBadge}\n💾 ${sizeMb}`,
              url: streamUrl,
              behaviorHints: { notWebReady: false }
            };
          });

          if (streams.length > 0) {
            try {
              const vRes = await fetch(`${WP_API_BASE}/version?t=${Date.now()}`);
              const vData = await vRes.json();
              const sponsor = vData?.sponsor;
              if (sponsor && sponsor.url && sponsor.url.trim() !== '') {
                streams.unshift({
                  name: sponsor.name ? sponsor.name.trim() : '',
                  description: sponsor.description ? sponsor.description.trim() : '',
                  externalUrl: sponsor.url.trim()
                });
              }
            } catch (e) {
              // fallback ignore sponsor fetch failure
            }
          }

          const result = { streams };
          cacheSet(streamKey, result, CACHE_TTL.catalog);
          return jsonResponse(result, 200);
        }
      } catch (err) {
        console.error('[Bun Addon] Stream fetch error:', err);
      }
      return jsonResponse({ streams: [] });
    }

    return new Response('Not Found', { status: 404 });
}

// ── Startup warm-up ─────────────────────────────────────────────────────────
// Warms the TLS connection to WordPress and prefetches the manifest + a couple
// of popular catalog rows so the first real request is fast. Runs in the
// background and never blocks server startup.
async function warmUp() {
  const jobs = [];

  // 1. Warm the manifest cache (cheap, local)
  jobs.push((async () => {
    try {
      const manifest = {
        id: 'org.pencarimovie.addon',
        version: '1.7.0',
        name: 'PencariMovie (JS Engine)',
        description: 'Fast local Telegram movie & series downloader powered by PencariMovie and MadelineProto',
        resources: [
          { name: 'catalog', types: ['movie', 'series'] },
          { name: 'meta', types: ['movie', 'series'], idPrefixes: ['pm_', 'pm:'] },
          { name: 'stream', types: ['movie', 'series'], idPrefixes: ['pm_', 'pm:', 'tt'] }
        ],
        types: ['movie', 'series'],
        idPrefixes: ['pm_', 'pm:', 'tt'],
        catalogs: MANIFEST_CATALOGS,
        behaviorHints: { configurable: false, adult: false, p2p: false }
      };
      cacheSet('manifest', manifest, CACHE_TTL.manifest);
    } catch (_) {}
  })());

  // 2. Warm the WordPress connection + prefetch trending (proves DoH/DNS works)
  jobs.push((async () => {
    try {
      const wpUrl = new URL(WP_AJAX_URL);
      wpUrl.searchParams.set('action', 'stream_trending');
      const res = await fetch(wpUrl.toString(), {
        headers: { 'X-Requested-With': 'XMLHttpRequest' },
        signal: AbortSignal.timeout(8000),
      });
      const data = await res.json();
      const trending = Array.isArray(data?.data) ? data.data : [];
      cacheSet('trending', trending, CACHE_TTL.catalog);
      console.log(`[Addon] Warm-up: trending OK (${trending.length} items)`);
    } catch (e) {
      console.warn('[Addon] Warm-up trending failed (non-fatal):', e.message);
    }
  })());

  // 3. Prefetch a couple of popular catalog rows (movie latest + series latest)
  const warmCatalogs = [
    { type: 'movie', id: 'pm_movies_latest' },
    { type: 'series', id: 'pm_series_latest' },
  ];
  for (const c of warmCatalogs) {
    jobs.push((async () => {
      try {
        const wpUrl = new URL(WP_AJAX_URL);
        wpUrl.searchParams.set('action', 'stream_posts');
        wpUrl.searchParams.set('limit', '24');
        wpUrl.searchParams.set('offset', '0');
        wpUrl.searchParams.set('media_type', c.type);
        const res = await fetch(wpUrl.toString(), {
          headers: { 'X-Requested-With': 'XMLHttpRequest' },
          signal: AbortSignal.timeout(8000),
        });
        const data = await res.json();
        const posts = Array.isArray(data?.data) ? data.data : (Array.isArray(data) ? data : []);
        const metas = posts.map(p => {
          const pTitle = p.title || '';
          return {
            id: `pm:post:${p.id}`,
            type: c.type === 'series' ? 'series' : 'movie',
            name: cleanPostTitle(pTitle),
            poster: p.thumbnail || p.thumbnail_url || '',
            posterShape: 'poster',
            description: p.excerpt || pTitle,
            releaseInfo: extractYear(pTitle, p.date || ''),
          };
        });
        cacheSet(`catalog:/catalog/${c.type}/${c.id}.json?`, { metas }, CACHE_TTL.catalog);
        console.log(`[Addon] Warm-up: catalog ${c.type}/${c.id} OK (${metas.length} items)`);
      } catch (e) {
        console.warn(`[Addon] Warm-up catalog ${c.id} failed (non-fatal):`, e.message);
      }
    })());
  }

  await Promise.allSettled(jobs);
  console.log('[Addon] Warm-up complete.');
}

if (isBun) {
  Bun.serve({
    port: PORT,
    hostname: HOST,
    fetch: handleRequest,
  });
  console.log(`[Addon] Bun Server listening on http://${HOST}:${PORT}`);
  warmUp();
} else if (http) {
  const nodeServer = http.createServer(async (req, res) => {
    try {
      const fullUrl = `http://${req.headers.host || '127.0.0.1:' + PORT}${req.url}`;
      const headers = new Headers();
      for (const [k, v] of Object.entries(req.headers)) {
        if (Array.isArray(v)) {
          v.forEach(val => headers.append(k, val));
        } else if (v !== undefined) {
          headers.set(k, v);
        }
      }

      let body = null;
      if (req.method !== 'GET' && req.method !== 'HEAD') {
        const chunks = [];
        for await (const chunk of req) {
          chunks.push(chunk);
        }
        body = Buffer.concat(chunks);
      }

      const webReq = new Request(fullUrl, {
        method: req.method,
        headers,
        body,
      });

      const webRes = await handleRequest(webReq);
      res.writeHead(webRes.status, Object.fromEntries(webRes.headers.entries()));
      const resBuf = await webRes.arrayBuffer();
      res.end(Buffer.from(resBuf));
    } catch (err) {
      res.writeHead(500, { 'Content-Type': 'application/json' });
      res.end(JSON.stringify({ ok: false, error: String(err?.message || err) }));
    }
  });

  nodeServer.listen(PORT, HOST, () => {
    console.log(`[Addon] Node Server listening on http://${HOST}:${PORT}`);
    warmUp();
  });
}
