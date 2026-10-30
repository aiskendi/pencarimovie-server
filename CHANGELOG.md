# Changelog

All notable changes to the PencariMovie Server / Downloader project will be documented in this file.

## [Unreleased]

### Added

- **The dashboard header shows an update when the release feed has one.** A pill sits next
  to the version, and only when `/api/update/check` reports `update_available`: it opens a
  small card naming the path that works on that host — "Update now" (`POST
/api/update/apply`) for a binary that can replace itself, `docker compose pull &&
docker compose up -d` in a code block for a container, "install the newest APK" for an
  app-managed build, and the release page for a manual install. `/api/update/check` is the
  only source; `/api/version`'s `update_needed` comes from the WordPress plugin's PHP-era
  `min_version` and reported "no update" while GitHub already had `v3.0.3`. Versioning
  stays per build: desktop compares `pencarimovie-server-<target>.tar.gz` tags, the APK
  compares `apk-server-latest`. AGENTS.md §8b records the rule.
  Pinned by `routes::tests::update_notice_is_a_non_blocking_item`.

### Changed

- **telegra.my is redesigned, and its Worker is now tracked.** The install page was five
  glass cards with the platform name for a heading; it is now a mono synopsis: the local
  dashboard address `http://127.0.0.1:8088` set as the hero figure, the five platform names
  as an index, and one row per OS carrying its own command, its own fact and its Copy.
  One signal colour, square surfaces, hairlines; the phone layout wraps the commands at 12px
  instead of clipping them and drops the Copy button, because a phone copies by long-press.
  [`cloudflare-worker-telegra-my.js`](cloudflare-worker-telegra-my.js:1) and
  [`wrangler-telegra-my.toml`](wrangler-telegra-my.toml:1) are tracked from now on (the
  landing page and its installers are the site); the relay worker stays ignored.
- **The dashboard's Addon section is `#addon`** (was `#connect`), and the header nav item
  says "Addon". `#connect` and the PHP-era `#settings` still resolve through the legacy
  map, so links already out there keep working.
- **Stremio's configure gear works, and it opens Catalogs.** It asks for
  `<transport>/configure`, which for this server is `/<token>/manifest.json/configure`;
  that answered 404, so the gear was dead in every client. All four spellings now redirect
  to `/#catalogs`, because configuring this addon means choosing the rows and filters it
  publishes, while Addon only shows the address. The bare `#configure` link means the same
  thing now, which also fixes the "Streams Filtered Out" card in the stream list: it links
  to `#configure` and used to land on Addon, nowhere near the filters it told you to change.
- **The streams-list update notice is a notification, not a manual.** One line:
  `PencariMovie v3.0.3 is available (this install is v3.0.2).`, linking to this server's
  dashboard, which carries the how-to. Deleted along with it: "the streams below still
  work", the per-host "To update, ..." hint, and the `streams_below` / `had_streams`
  plumbing that existed only to word those two sentences. The row still never carries a
  `title` field (stremio-core drops such rows).

### Fixed

- **"Install into Stremio" no longer disappears when the dashboard is opened over https.**
  `updateAddonModalUrls()` hid `#addonStremioSync` whenever `location.protocol === "https:"`
  or the page was on a tunnel, so a dashboard opened from a tunnel or a served domain showed
  only the manifest text and left the reader to copy it. The API install works on https (the
  manifest is fetched same-origin and `api.strem.io` answers the cross-origin POST), so the
  card is shown there too and its preview is refreshed. Verified on a live
  `*.trycloudflare.com` origin: element present, `hidden: false`, preview naming the tunnel
  manifest.

- **A seek starts about 230 ms sooner.** The preflight prime is fetched from the enclosing
  1 MB boundary and, past byte 0, rounds up to a full 1 MB head. `fetch_range_core()` holds
  ONE connection permit for the whole call, so that head was two 512 KB `upload.getFile`
  parts fetched back to back, and the response body did not start until both had arrived.
  The two parts now go out together. Measured on a 406 MB 4K source, 4 MB range windows,
  one bot, median of 8 rounds: first byte **954 ms before, 723 ms after** (24% sooner).
  Sustained throughput is unchanged, because only the 1 MB head is split and the pipeline
  behind it is untouched. A byte-0 prefix keeps its single 512 KB part and sees no change.
  The part budget (`workers x inflight`) is shared, so it is worth stating: the split holds
  2 permits at once instead of 1, for half as long, which keeps part-seconds level.
  Pinned by `download::tests::prime_split_covers_the_head_in_legal_parts` and
  `download::tests::only_one_part_heads_skip_the_split`.

- **Saving catalog settings can no longer blank the configuration.**
  `write_catalog_settings()` truncated `storage/catalog_settings.json` with `fs::write` before
  filling it, while `read_catalog_settings()` sits on the config path of every `/catalog`,
  `/meta`, `/stream` and `/api/settings` request: a reader inside that window parsed an empty
  file, and the catalog rows, upstream manifests and stream filters all silently fell back to
  the `PM_CATALOG_SETTINGS` env layer alone. A failed encode was mapped to `{}` as well, so a
  serialization error wiped the file and still answered `ok: 1`. It now writes
  `catalog_settings.json.tmp` and renames over the target (the same pattern, and the same
  reason, as `save_auth_token()`), and writes nothing at all when the encode fails.
- **`can_apply` is no longer frozen inside the update cache.** `update::check()` memorises
  the feed answer for an hour, and `can_apply` / `apply_blocked_reason` / `apply_hint` were
  part of that body, so a freshly extracted release kept answering `no_script` (no "Update
  now" button) for up to an hour, and a deleted apply script kept offering a button that
  could not work. They are recomputed on the way out of the cache, exactly like the
  process-specific fields in `routes::check_version`. Measured both ways on one process:
  script present → `can_apply:true`, script removed → `can_apply:false`.

- **Header pills and the update card meet WCAG AA.** `--pm-text-muted` (#64748b) measures
  4.16:1 on the page background and 3.49:1 on the card surface, so the version/host pills
  and the card's close button now use `--pm-text-secondary` (7.4:1 and 6.48:1). The update
  pill is a real `<button>` carrying `aria-haspopup="dialog"`, `aria-expanded` and
  `aria-controls`; opening the card moves focus into it, closing returns focus to the pill,
  and Escape closes it.

- **A series list can no longer adopt another show's EPISODE TITLE as its own
  name.** `/stream/series/pm:post:8798831284.json` ("The Witch 2025") served
  three episodes of **I'm Not Afraid**:
  `Im.Not.Afraid.S01E01.The.Witch.1080p.NF.WEB.DL…` — I'm Not Afraid's S01E01
  just happens to be _titled_ "The Witch". The matcher built its title
  candidates from both sides of the episode marker and accepted the
  post-marker text as a title, so the file's own show name (the text BEFORE
  `S01E01`) was ignored. `after_ep` now only stands in for the title when the
  pre-marker part is a channel/uploader tag or absent — the
  `CDL.E01.The.Rebel.Princess` / `DramaOST.E01.The.Witch` / `E01.The.Witch` shape
  the port exists for — and the multi-word phrase search runs over the title
  region instead of the whole file name (`Other.Show.S01E01.My.Demon` is not
  "My Demon"). `legacy/php/backend.php:9386` has the same hole
  (`array_filter([$baseF, $afterEp])`), so this is a deliberate divergence from
  the archived engine.
  Pinned by `matcher::tests::series_does_not_take_a_foreign_episode_name_as_its_title`
  (3 rejected rows + 17 kept rows, taken verbatim from the live answer) and
  re-checked against every existing series rule (`series_keeps_embedded_year`,
  `series_glory_cases_from_php_regression_suite`, …).

- **A series reached by its `tt…` id no longer answers "No Streams Found" for an
  episode whose files are outside the one page the origin returns for that id.**
  Measured on `tt35324004` ("Tigers Sniff the Rose"): the origin answered 5 rows
  for the id, none of them E1, and `handle_external_stream()` returned those 5
  straight to the matcher — so E1 answered "No Streams Found" while the show's
  own post carried 3 playable files (865 / 530 / 146 MB). The by-id page is now
  only a fast path when [`by_id_page_covers_episode()`](src/routes.rs:2874) says
  it carries the requested episode; otherwise it joins the episode ladder as a
  seed, exactly where the title search's rows go.
  - The post route was never affected (`pm:post:9000035085:1:1` answered 3 rows
    throughout), which is what made this look like a metadata bug from the app.
  - Pinned by `routes::tests::by_id_page_must_cover_the_requested_episode`
    (E1 outside a E5–E9 page → ladder; E7 inside → fast path; season must match;
    a movie page and a series root stay covered).

- **A series' episode list is no longer short by the episodes that live only in
  the origin index.** `pm:post:9000018021` ("To My Beloved Thief 2026") owns
  E03–E14 and E16 — the 13 rows its `post-files` returns — so `/meta` offered 13
  episodes while `/stream` answered E01, E02 and E15 out of the index
  (`nunadrama.To.My.Beloved.Thief.E01.1080p.mp4`,
  `OLD.E01.To.My.Beloved.Thief.2025.mp4`, …). The episode PICKER is
  `meta.videos`, so three playable episodes were unreachable.
  `fd_fetch_series_episode_files()` ([`backend.php:10398`](legacy/php/backend.php:10398))
  computes exactly this gap, but its early return tested only `$hasExplicitEp`,
  never `$hasGaps`, so the PHP never backfilled either. This is that divergence
  closed: [`episode_gaps()`](src/routes.rs:5368) finds the numbers between what
  the post owns, and [`gap_episode_videos()`](src/routes.rs:5409) resolves them
  against the index through the SAME strict matcher `/stream` uses, so the
  picker can never advertise an episode the player cannot answer.
  - One index search, and only when the numbering has a hole; the outcome —
    including "nothing to fill" — is cached 10 min, so a series like
    `The Witch 2025` (E1–E10 + E16, with nothing in the index between them) is
    not re-searched on every `/meta`.
  - Verified live: `To My Beloved Thief 2026` now lists **16 videos (E1–E16)**;
    `The Witch 2025` is unchanged at 11 (E1–E10, E16) — no false episodes were
    invented. Pinned by
    `routes::tests::episode_gaps_are_the_numbers_between_what_the_post_owns`.

- **A series' episodes no longer report "released just now", and the episode list
  is ordered.** `/meta/series/pm:post:<id>.json` stamped every `released` with
  `Utc::now()` at serialisation, so the value tracked the request clock — three
  requests 7 s apart returned `16:14:12Z`, `16:14:19Z`, `16:14:26Z` for the same
  episode — and every episode of a 2018 series claimed the current date. The
  `videos` array was also left in raw scan order (11 of 12 K-drama series
  scrambled or descending), because the PHP's explicit
  `usort(season, episode)` (backend.php:16339-16348) was never ported.
  - [`WpFile.added_date`](src/wp_data.rs:196) now carries the origin's own Unix
    timestamp (`"1792386106"` — a STRING on the wire), the same field the PHP
    used as `released` before the port dropped it.
  - [`series_videos()`](src/routes.rs:5306) emits that stored date, falls back to
    the series anchor (the post's publish date, else `{year}-01-01`), and emits
    `null` — never a fabricated "now" — when the post has no date at all:
    clients sort on the field, so a moving value is worse than an absent one.
  - The same builder sorts by `(season, episode)` ascending, so next-episode and
    autoplay can no longer advance to the wrong episode.
  - Pinned by `routes::tests::episode_released_is_stored_not_the_wall_clock`,
    `…::series_videos_are_sorted_by_season_then_episode`,
    `…::series_anchor_prefers_the_post_date_over_the_year` and
    `…::iso_from_wp_date_reads_the_origin_format`; reproduced and re-verified
    live with `node tools/probe_issue10.mjs` (36/36 series across kdrama, malay
    and anime: 0 moved, 0 near the clock, 36/36 ascending).

- **An AAC `.m4a` is no longer pulled in full before the server admits it cannot
  transcode it.** The device log (CPH2581, 1.0.86) said
  `alac probe rejected the file … no decoder for codec CodecType(4100)` — and
  `4100` is `0x1004`, i.e. **AAC** (`symphonia_core::codecs::CODEC_TYPE_AAC`;
  ALAC is `0x2003`). Symphonia 0.5 has no AAC decoder at all, so an AAC `.m4a`
  could never take the ALAC→WAV path, yet
  [`serve_alac()`](src/download.rs:1) pulled the WHOLE track (measured 8 MB, up to
  25 MB) from Telegram before the probe said so — on the first play of every such
  file, which is what made `.m4a` tracks feel broken on Eclipse Android.
  - [`audio::mp4_audio_fourcc()`](src/audio.rs:38) reads the first `stsd` sample
    entry (`alac`, `mp4a` = AAC, `ac-3`, `opus`, …) from a prefix of the file.
  - `serve_alac` checks it after the first 1 MB chunk and returns immediately for
    a codec this build cannot decode, so the raw bytes the client can play start
    flowing at once; the verdict is cached (`alac_skip`, 1 h) as before.
  - An undecidable prefix (`moov` at the end of the file) keeps the old
    behaviour, so a real ALAC track is never skipped. Pinned by
    `audio::tests::mp4_codec_is_read_from_the_first_sample_entry` and
    `…::mp4_codec_is_unknown_without_a_sample_entry`.

- **A busy origin no longer reads as "this track/file is gone".** `/resolve-file`
  answers `503 Server busy resolving file` (with `retry_after: 5`) while a
  Telegram relay runs for that bot+code, and `http_json()` collapses every 5xx
  into an empty object — so `/music/stream/<code>` answered
  `404 {"error":"Track not found"}` and `/stream/movie/pm_file_<code>` rendered
  the "File could not be resolved" card, both after only ~4 s of retries (all
  inside the busy window). Measured live: four 503s in 4.3 s, then `Track not
found`; the same code resolved in one call a minute later.
  - [`http_json_h_status()`](src/httpc.rs:1) reports the status alongside the
    body; `http_json()`/`http_json_h()` are unchanged wrappers.
  - [`resolve_shortcode_status()`](src/resolve.rs:1) waits out the origin's
    advertised `retry_after` (5 s, once) before the existing fast ladder, and
    `resolve_shortcode()` keeps its old signature.
  - Both resolve-consuming routes now answer **503 + `Retry-After: 5`** instead
    of a not-found answer when the last origin status was 5xx or the request
    never completed: [`eclipse_stream()`](src/routes.rs:1) and the direct
    `pm_file_` branch of `/stream/*` (which used to answer "File could not be
    resolved"). A genuinely missing item still answers 404 / the card.
  - Pinned by `tools\probe_music_busy_origin.mjs` (stub origin via
    `FD_WP_API_BASE`: busy → 503 + Retry-After, 5 s busy → the track plays,
    movie item → 503 not a dead card) and `tools\probe_music_resolve.mjs`
    (live: search → stream → media bytes).

- **Removing the primary bot no longer ends in an endless "Provisioning guest
  bot..." spinner.** The Bot Pool trash button always re-ran the session check,
  so removing the last bot dropped the dashboard into guest provisioning — and a
  provision that FAILED only recorded `provisionError` and returned, leaving the
  full-page loading screen up with no token field anywhere. Three changes:
  - [`remove_pool_bot()`](src/session.rs:1) now promotes a bot that actually
    holds a session file (`pick_replacement()`), not merely `pool.first()`; a row
    that was added but never logged in produced `has_session: false` while the
    pool still listed a bot, which skipped the gate. Removing the LAST bot also
    clears `bot_id.txt`, so the removed identity cannot come back as "active".
  - `/api/bots/remove` answers `has_session` and `bots_remaining`
    ([`src/routes.rs`](src/routes.rs:1)), so the dashboard can tell "the primary
    switched to another bot" from "show the token gate" without guessing. The
    response is the ONLY thing that changes: automatic guest provisioning stays
    on everywhere else (a refresh, `/manifest.json` clients and the addon keep
    leasing a guest bot exactly as before).
  - [`public/app.js`](public/app.js:1): the remove click that empties the pool (or
    leaves only an unusable row) opens the bot-token gate exactly like
    **Disconnect All**, and a failed auto-provision falls through to that gate
    with the engine's reason instead of leaving the spinner on screen. Pinned by
    `tools/probe_remove_primary.mjs` (real binary over a throwaway
    `PM_APP_ROOT`, plus `public/app.js` driven in a vm) and by
    `session::tests::removal_promotes_a_bot_that_can_actually_log_in`.

- **The Heroku deploy can push the image again** (`error from registry:
unsupported`). [`deploy-heroku.yml`](.github/workflows/deploy-heroku.yml:1)
  promoted `:sha-<7>` with `docker pull` → `docker push`, but that tag is a
  **multi-arch manifest list** (assembled by
  [`release.yml`](.github/workflows/release.yml:575) from `native-amd64` +
  `native-arm64`). A tag that resolves to an index is also _stored_ as an index,
  so the push re-uploaded the whole list — including the arm64 manifest whose
  layers were never downloaded (`4f4fb700ef54: Unavailable`) — and
  `registry.heroku.com` only stores a single-platform `linux/amd64` image in
  Docker v2 schema 2, so it answered `unsupported`. The promotion now resolves
  the `linux/amd64` sub-manifest by digest and copies exactly that image with
  `skopeo copy --all=false --override-arch amd64 --format v2s2`, then refuses to
  release if the registry somehow holds an index. Both architectures still stay
  published on GHCR (`render.yaml` pins `:native`, which must remain a list).
  The workflow is also **manual only** now (`workflow_dispatch`) and never
  dispatches `release.yml`: it promotes the `image` input when given, else the
  `:sha-<7>` of the commit the run was started on, and fails naming that tag
  when it is not published — instead of rebuilding, or silently falling back to
  `:native`. The `git push heroku` fallback is gone with it: Heroku's classic
  Docker builder rejects this repo's `Dockerfile` outright (`RUN
  --mount=type=cache` → "the --mount option requires BuildKit"), so that path
  only ever added a second, slower failure after the real one. The promotion
  itself also has to ask for `repo@digest`, not `repo:tag@digest` — skopeo
  refuses a reference carrying both.

- **The sponsored item is now `streams[0]` on EVERY `/stream/*` answer.** Only
  [`finish_streams()`](src/routes.rs:1) used to prepend it, so every answer that
  built its own array lost it:
  - the locked card (`fd_stremio_locked_stream`) — reported as "when the URL
    changed / password message shows, the sponsored message is gone";
  - the no-session cards (clock skew, guest-bot provisioning), "File could not be
    resolved", a malformed `pm:post:` id and the unsupported-id shape (which used
    to answer a bare `{"streams":[]}` — the one answer with no sponsor at all).

  All of them now go through one placement function,
  [`with_top_items()`](src/routes.rs:1): sponsor, then the update notice, then the
  rows. `sponsor` was also moved OUT of the 300 s `/stream` response cache — baking
  it in meant a single failed WordPress call hid the sponsor from every client for
  five minutes (the "not always on top" report), and froze an "update available"
  card long after the update. The cache key is now `v2|…` so pre-change entries
  cannot be served. `scripts/probe_sponsor_top.mjs` pins all of it over HTTP.

- **The system tray is now part of the binary, and the icon actually shows.** The
  tray was a separate `powershell.exe` process running `tray.ps1`. Windows stores
  the per-app tray settings in `HKCU\Control Panel\NotifyIconSettings` keyed by the
  **hosting executable**, so that icon shared `powershell.exe`'s single entry with
  every other PowerShell-hosted tray (it read `9Router` on the machine that
  reported this) and could only ever sit in the "hidden icons" flyout; running
  `pencarimovie-server.exe` itself produced no icon at all. Three separate causes,
  all removed:
  - `pencarimovie-server --tray` now creates the icon in-process over
    `Shell_NotifyIconW` (`src/tray.rs`, `user32`/`shell32` only, no new
    dependency). Verified: the icon appears with the tooltip
    `PencariMovie Server - port N` **and** the shell writes a
    `NotifyIconSettings` entry whose `ExecutablePath` is `pencarimovie-server.exe`.
  - The logo is compiled in (`include_bytes!("../tray.ico")`), so the icon is not
    the stock/blank one — and not missing at all — in a folder that holds nothing
    but the executable.
  - **No console window survives a tray start.** `--tray` (and a double-clicked
    exe, which also gets an icon) releases its console with `FreeConsole()` rather
    than hiding it: hiding leaves the process still owning a console, so conhost
    kept the window and the launching `cmd` alive forever. `start-tray.bat` uses a
    plain `start`, so the batch returns at once; verified `MainWindowHandle = 0`
    and no `conhost` child. Because there is no console to redirect,
    [`log::stderr_log`](src/log.rs:73) also appends to `storage/server.log` while
    the tray runs, which is the file "Open log" opens.
  - `tray.ps1` no longer runs the tray; it is a shim that forwards an old
    `HKCU Run` entry to `pencarimovie-server.exe --tray`. Nothing is registered
    for login any more, because the exe was never the login entry — the tray menu
    and `autostart-on.bat` now write the binary's own command line.
- **`--tray` works on Linux too**: the binary writes its embedded tray script
  (`desktop/linux/tray.sh`) to `storage/tray/pm-tray.sh` and runs it, so
  `start-tray.sh`/autostart need only `./pencarimovie-server --tray` (`yad` still
  draws the icon; without it the server runs and prints the URL). macOS keeps the
  LaunchAgent: an `NSStatusBar` item needs AppKit, i.e. a `.app` bundle.

### Changed

- `--tray` / `--no-tray` (and `PM_NO_TRAY`) are documented flags; a
  double-clicked `pencarimovie-server.exe` gets a tray icon as well as the
  browser. `--tray` hides its own console, so the launchers no longer need
  `powershell -WindowStyle Hidden`.

## [2.9.7] - 2026-10-22

### Fixed

- **Series returned "No Streams Found" when opened through an external id** (`tt…`,
  `tmdb:…`). Two separate causes, both on the origin query rather than the matcher:
  - `/stream-files` was sent `year=…` for series. The origin applies it as a real
    filter and series episode rows are un-tagged, so `title=Hey Kay Navin&year=2026`
    answered 0 rows where `title=Hey Kay Navin` answered 10. `year` is now omitted for
    series (matching the PHP at `backend.php:10180`), with a no-year retry for movies.
  - The external-id route never ran the episode ladder. The PHP calls
    `fd_fetch_episode_stream_files(0, …)` (`backend.php:17093`); this port only did the
    title search, so one episode listed 20 files through `tt…` and 86 through
    `pm:post:…`. The route now runs the same ladder with the external id as the origin
    `id`.
- **`--workers` default raised from 1 to 4 on every platform.** `workers × inflight`
  is the `upload.getFile` part budget shared by all viewers, and 1 connection meant 8
  parts for the whole process — a second stream starved the first. A real session at
  `--workers 1` logged repeated `download refused`; at 4 it logged 55/55 served with
  zero refusals. `PM_WORKERS` overrides it.
- **Manifest descriptor trimming no longer floods the log.** One line per dropped
  bridged catalog (3,021 of the last 4,000 log lines with a 300-catalog upstream) is
  now a single summary with `from_bytes`/`to_bytes`/`dropped`.

### Added

- **PHP `MUST NOT REGRESS` matcher cases ported** from
  `legacy/php/test_kudrat_series.php`: 11 cases covering "The Glory" (2022) vs
  "Glory" (2025) and the embedded-name-year titles ("Kudrat 1968: High Council 2026").
  They pin the strict leading-"The" check, the single-word year guard and the
  single-word foreign-indicator exemption.

## [2.3.3] - 2026-09-29

### Added

- **Nuvio TV Smart Player & Web UI Enhancements**:
  - Subtitle rail selection with mouse and touch interaction in the player subtitle modal.
  - Fullscreen auto-rotation, smooth mouse seeking, and auto-progression to next episode.
  - Auto-retry and recovery mechanism for playback stalls and network startup glitches.
  - Unified Sources and Episodes panels with responsive glass-card styling and no title truncation on large screens.
  - Fixed "Continue Watching" series history stack preserving clean back navigation without popstate corruption.
- **Sub-Second Stream Resolution & Fast-Path Metadata Indexing**:
  - Direct metadata lookups for instant stream card generation.
  - Direct PM stream resolution bypasses slow upstream requests and forwards post metadata directly.
  - Strict title matching distinguishing "Title" from "The Title", with full AKA alias search support.
- **Master Upstream Catalog Toggle (`upstream_enabled`)**:
  - One-switch control in Catalog Configuration to toggle upstream addons (e.g. AIOMetadata) without wiping custom URLs.
  - Prevents dashboard hangs by filtering out non-local `up_*` catalogs from web category rows.
- **Same-Origin Stream Cookie Auth**:
  - Cookie credentials automatically forwarded on same-origin requests for seamless remote and Cloudflare Tunnel playback.

## [2.1.8] - 2026-09-18

### Added

- **Server Password Authentication & Token Model**:
  - Remote and public requests (VPS public IP, Cloudflare Tunnel) are guarded by password authentication (default `123456`) and persistent 32-character tokens.
  - Manifest and stream paths support clean token URL routing (`/<token>/manifest.json`, `/<token>/stream/...`).
  - Localhost and private LAN (RFC-1918) requests remain password-free with instant access.
  - CLI management commands: `pms password <new>`, `pms reset-password`, `pms token`, and `pms token rotate` (implemented in [`pencarimovie-linux.sh`](pencarimovie-linux.sh:1), [`pencarimovie-termux.sh`](pencarimovie-termux.sh:1), [`pencarimovie-windows.bat`](pencarimovie-windows.bat:1), and [`auth-write.ps1`](auth-write.ps1:1)).
- **Cloudflare Zero Trust Named Tunnel & Token Persistence**:
  - Support for custom Named Tunnel tokens via Settings UI and `POST /api/tunnel/enable`.
  - Added CLI command `pms tunnel <TOKEN>` across Linux, Windows, and Termux launchers.
  - Dedicated token persistence in `storage/tunnel/token.txt` preserved across server restarts and `pms stop`.
  - Automatic tunnel warm-up in [`warmup-ipc.php`](warmup-ipc.php:1) on `pms start` or `pms restart` when previously enabled.
  - Dynamic discovery of mapped public hostnames matching local listening port from connector logs.
- **VPS & Remote Server Manifest Mode**:
  - Direct IP detection in frontend (`_isIpHost`) preventing public VPS addresses from showing TryCloudflare tunnel badges when tunnels are disabled.
  - Added `mode=server` to [`backend.php`](backend.php:6102) yielding `PencariMovie (Server)` / `org.pencarimovie.addon.server`.
  - Addon modal displays "🌐 Server Manifest" on VPS IPs and automatically selects the Server manifest in Stremio Sync.
  - Dynamically hides the Wi-Fi/LAN vs Localhost toggle when accessing from a public VPS IP or Cloudflare Tunnel.
- **Eclipse Music Addon Integration**:
  - Native discovery and streaming for 500,000+ audio tracks under `/eclipse` routes (`/eclipse/manifest.json`, `/eclipse/search`, `/eclipse/stream/{id}`, `/eclipse/catalog/{id}`, `/eclipse/resolve`).
  - Advanced caption and title parsing in [`backend.php`](backend.php:8165) separating artist, title, and featured collaborations with multi-word connector handling.
- **Stremio HTTP API Sync**:
  - Browser-side Stremio HTTP API sync (`api.strem.io` login → `addonCollectionGet` → `addonCollectionSet`) without transmitting user credentials to the PHP backend.

### Security

- **Caddy Sensitive File Shield**:
  - Added `@blocked` route matcher in [`Caddyfile`](Caddyfile:1) returning HTTP 404 for sensitive files and directories (`storage/*`, `vendor/*`, `bin/*`, `patches/*`, `Caddyfile*`, `*.sh`, `*.bat`, `*.key`, `*.log`, `*.lock`, `*.md`, `*.yml`, `*.yaml`, `composer.json`, `package.json`).
- **Progressive Brute-Force Rate Limiting**:
  - Exponential lockout ladder (30s, 2m, 10m, 30m) for failed password submissions tracked in `storage/cache/auth_lockout.json` returning HTTP 429 with `Retry-After`.
- **Locked Stream Responses**:
  - Unauthenticated remote stream requests receive an informative Stremio stream banner directing users to authenticate via `#addon`, rather than failing silently with 401.
- **Native PHP Static Boundary**:
  - Direct built-in PHP server execution (`php -S ... router.php`) strictly isolates file serving to `public/` via `realpath()` and `str_starts_with()` checks.

### Changed

- **Stream Object Specification**:
  - Playable `/stream` objects comply with AIOStreams specification (`name`, `description`, `url`, `behaviorHints`).
  - Stream URLs end with clean file extensions (`.mp4`, `.flac`, `.mkv`) to guarantee playback compatibility across Stremio Web, Android, and desktop players.
- **Release Packaging**:
  - Updated [`scripts/build-release.bat`](scripts/build-release.bat:1) and [`scripts/package-unix.sh`](scripts/package-unix.sh:1) to include [`pencarimovie-linux.sh`](pencarimovie-linux.sh:1), [`pencarimovie-termux.sh`](pencarimovie-termux.sh:1), [`README.md`](README.md:1), and [`patches/`](patches/) across all release archives.
  - Added automated line ending normalization (CRLF → LF) for all shell scripts and `bin/php` when building Unix packages on Windows.
  - Exported [`update.ps1`](update.ps1:1) to `dist/` alongside [`pencarimovie-windows.bat`](pencarimovie-windows.bat:1).
- **Process Management**:
  - Preserved tunnel token state during `stop.sh` and `stop.bat` process teardown.
  - Improved working directory resolution in Unix start scripts ensuring Caddy directives resolve from the repository root.

## [1.0.1] - 2026-08-29

### Added

- **Stremio & Nuvio Addon Integration**:
  - Implemented Stremio addon manifest and endpoints (`/manifest.json`, `/catalog/...`, `/meta/...`, `/stream/...`) in [`backend.php`](backend.php:2585).
  - Enables direct playback and library browsing from Stremio and Nuvio players with automatic Telegram bot stream resolution.
- **Windows System Tray Helper**:
  - Added background tray management scripts ([`tray.ps1`](tray.ps1:1), [`start-hidden.ps1`](start-hidden.ps1:1)) and tray icons ([`tray.ico`](tray.ico), [`tray.png`](tray.png)).
  - Runs FrankenPHP silently with hidden window and provides tray menu actions for opening the web dashboard and stopping the server.
- **One-File Installers with Start-Time OTA Updates**:
  - Added [`pencarimovie-windows.bat`](pencarimovie-windows.bat:1), [`pencarimovie-linux.sh`](pencarimovie-linux.sh:1), and [`pencarimovie-termux.sh`](pencarimovie-termux.sh:1).
  - Automatically queries GitHub `releases/latest` on start, updates the application files in-place while preserving `storage/` bot session data.
- **Version Check & Minimum Version Enforcement**:
  - Implemented version verification against WordPress API endpoint (`/fastdownloader/v1/version`) with local hourly caching.
  - Added full-screen update notification overlay in [`public/index.html`](public/index.html:27) and [`public/styles.css`](public/styles.css:681).
- **LAN IP Detection & Network Access**:
  - Cross-platform network IP discovery in [`start.bat`](start.bat:1), [`start.sh`](start.sh:1), and [`start-termux.sh`](start-termux.sh:1) displaying local and LAN network URLs upon startup.

### Changed

- **Frontend Streaming UI Overhaul**:
  - Integrated Netflix-style FlixBrowse UI directly into [`public/index.html`](public/index.html:1), [`public/app.js`](public/app.js:1), and [`public/styles.css`](public/styles.css:1).
  - Added rich search overlays, file detail views, inline Telegram stream player, and one-click MadelineProto downloader.
- **Backend Architecture & Security**:
  - Session-based authentication with automatic Telegram bot login via MadelineProto.
  - Eliminated plaintext configuration files; API credentials are securely fetched from the WordPress backend and encrypted with the bot token.
  - Enhanced FrankenPHP storage directory resolution to protect sessions in temp directory environments.
- **Launcher Scripts**:
  - Refactored [`start.bat`](start.bat:1), [`start.sh`](start.sh:1), [`start-termux.sh`](start-termux.sh:1), [`stop.bat`](stop.bat:1), [`stop.sh`](stop.sh:1), [`restart.bat`](restart.bat:1), [`restart.sh`](restart.sh:1), and [`restart-termux.sh`](restart-termux.sh:1) for unified port handling (default `8088`), clean output banners, and process lifecycle management.
- **Project Documentation**:
  - Updated [`README.md`](README.md:1) and [`RELEASE.md`](RELEASE.md:1) with Stremio setup guides, one-line installer commands, and packaging specifications.
