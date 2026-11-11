/*
 * PencariMovie Server — dashboard controller.
 *
 * Owns three things and nothing else:
 *   1. routing between the six sections
 *   2. refreshing the settings surfaces that app.js used to refresh when it
 *      opened #settingsGate / #addonModal (both of which no longer exist)
 *   3. the setup checklist on Overview, from real endpoints only
 *
 * Everything that mutates state — bot tokens, catalog toggles, tunnel,
 * Stremio sync, password — is still bound by app.js. This file never
 * duplicates that logic; it only decides what is visible.
 *
 * There are no sub-tabs. Catalogs is its own section because it is server
 * configuration for every client at once, not a sub-view of Security. Connect
 * has no client tabs because Stremio, Nuvio, Streamr, Debrify, WuPlay and
 * Playtario all consume the SAME manifest; only music differs.
 */
(function () {
  'use strict';

  var SECTIONS = ['overview', 'addon', 'bots', 'tunnel', 'catalogs', 'system'];

  // The section is #addon, not #connect: the button that opens it is "AddOn" on
  // the Android app and this is the add-on setup screen, so the old name sent
  // people looking for a "Connect" tab that no longer exists. #connect still
  // resolves here — Telegram bot-setup links are in the wild (?token=...&#settings)
  // and the add-on modal predates the panel.
  // #configure is NOT here: it is Stremio's gear, and configuring the addon means
  // choosing which rows and filters it publishes, so it resolves to Catalogs.
  // #system/catalogs predates Catalogs becoming its own section.
  var LEGACY = {
    '#': 'overview',
    'addon': 'addon',
    'connect': 'addon',
    'configure': 'catalogs',
    'settings': 'bots',
    'bots': 'bots',
    'catalogs': 'catalogs',
    'system/catalogs': 'catalogs',
    'tunnel': 'tunnel',
    'overview': 'overview',
    'system': 'system'
  };

  var current = 'overview';

  function el(id) {
    return document.getElementById(id);
  }

  function readHash() {
    var raw = String(window.location.hash || '').replace(/^#/, '').trim();
    if (!raw) return 'overview';
    if (LEGACY[raw]) return LEGACY[raw];
    var head = raw.split('/')[0];
    if (LEGACY[head]) return LEGACY[head];
    return 'overview';
  }

  /* ── routing ───────────────────────────────────────────────────────── */

  function applyRoute(section, writeHash) {
    current = section;

    var panels = document.querySelectorAll('.pm-panel');
    for (var i = 0; i < panels.length; i++) {
      var p = panels[i];
      if (p.getAttribute('data-panel') === section) p.classList.remove('hidden');
      else p.classList.add('hidden');
    }

    var nav = el('pmNav');
    if (nav) {
      var items = nav.querySelectorAll('[data-section]');
      for (var n = 0; n < items.length; n++) {
        if (items[n].getAttribute('data-section') === section) items[n].setAttribute('aria-current', 'true');
        else items[n].removeAttribute('aria-current');
      }
    }

    if (writeHash !== false) {
      var next = '#' + section;
      if (window.location.hash !== next) {
        try {
          window.history.replaceState(null, '', next);
        } catch (e) {
          window.location.hash = next;
        }
      }
    }

    onEnter(section);
  }

  function onEnter(section) {
    var app = window.app;
    if (section !== 'overview') stopLive();
    try {
      if (section === 'addon') {
        // app.js refreshed these inside openAddonModal(), which is gone.
        if (app && typeof app.refreshConnectState === 'function') app.refreshConnectState();
      } else if (section === 'bots') {
        // No version badge here on purpose: the sticky header already shows it,
        // and showSettingsGate()'s renderSettingsVersion() wrote the same value
        // into two different elements, which read as a duplicate.
        applyBotsVisibility();
        ensureBot();
        if (app && typeof app.loadBotPool === 'function') app.loadBotPool();
        if (app && typeof app.loadSessionStatus === 'function') {
          // loadSessionStatus() is what settles hasSession; re-assert the face
          // once it lands so an async result cannot leave the token form up on
          // a connected server.
          var pending = app.loadSessionStatus();
          if (pending && typeof pending.then === 'function') pending.then(applyBotsVisibility, applyBotsVisibility);
        }
      } else if (section === 'tunnel') {
        if (app && typeof app.loadTunnelStatus === 'function') app.loadTunnelStatus();
      } else if (section === 'catalogs') {
        if (app && typeof app.loadCatalogSettings === 'function') app.loadCatalogSettings();
      } else if (section === 'overview') {
        refreshChecklist();
        startLive();
      }
    } catch (e) {
      if (window.console) console.warn('dashboard: refresh on enter failed', e);
    }
  }

  function switchTo(section) {
    if (SECTIONS.indexOf(section) === -1) section = 'overview';
    applyRoute(section, true);
  }

  /* ── Bots: one of two faces, never both ───────────────────────────── */

  /*
   * The old #settingsGate swapped #settingsTokenSection and
   * #settingsConnectedSection from inside showSettingsGate(). Both blocks are
   * now siblings in the Bots panel, so nothing did the swapping and the page
   * showed an empty bot pool above the add-a-token form. This restores it from
   * the session endpoint.
   */
  function applyBotsVisibility() {
    return getJson('/api/session').then(function (s) {
      var connected = !!(s && (s.has_session || s.hasSession || s.connected));
      var tokenSection = el('settingsTokenSection');
      var pool = el('botPoolList');
      var poolBox = pool ? pool.closest('.settings-gate__bots-container') : null;
      var disconnect = el('settingsDisconnectBtn');
      var disconnectRow = disconnect ? disconnect.parentNode : null;

      if (tokenSection) tokenSection.classList.toggle('hidden', connected);
      if (poolBox) poolBox.classList.toggle('hidden', !connected);
      if (disconnectRow) disconnectRow.classList.toggle('hidden', !connected);
    });
  }

  /* ── no bot? provision one ────────────────────────────────────────── */

  /*
   * app.js already POSTs /api/provision from loadSessionStatus() when there is
   * no session, but its only feedback was #loadingScreen (removed) and its
   * failure path opened #settingsGate (also gone). So on the dashboard it either
   * happened silently or not at all.
   *
   * This does not duplicate the call: it reads /api/session first, waits while
   * app.js is mid-flight, and retries at most twice if that attempt failed.
   */
  var provisionAttempts = 0;

  function setBotNotice(text) {
    var target = el('settingsStatus');
    if (!target) return;
    target.textContent = text || '';
  }

  function ensureBot() {
    return getJson('/api/session').then(function (s) {
      if (!s) return;
      if (s.has_session) {
        setBotNotice('');
        applyBotsVisibility();
        return;
      }
      if (s.is_provisioning) {
        setBotNotice('Provisioning a guest bot\u2026');
        if (provisionAttempts < 30) {
          provisionAttempts++;
          window.setTimeout(ensureBot, 2500);
        }
        return;
      }
      if (provisionAttempts >= 2) {
        setBotNotice('No bot connected. Paste a token below.');
        applyBotsVisibility();
        return;
      }
      provisionAttempts++;
      setBotNotice('Provisioning a guest bot\u2026');
      fetch('/api/provision', { method: 'POST', credentials: 'same-origin' })
        .then(function (r) {
          return r.json().then(
            function (j) {
              return j;
            },
            function () {
              return null;
            }
          );
        })
        .then(
          function (res) {
            if (res && res.ok && res.bot_id) {
              setBotNotice('Guest bot connected.');
              var app = window.app;
              if (app && typeof app.loadSessionStatus === 'function') app.loadSessionStatus();
              window.setTimeout(function () {
                applyBotsVisibility();
                refreshChecklist();
              }, 900);
              return;
            }
            setBotNotice((res && res.message) || 'Auto-connect unavailable. Paste a token below.');
            applyBotsVisibility();
          },
          function () {
            setBotNotice('Auto-connect unavailable. Paste a token below.');
            applyBotsVisibility();
          }
        );
    });
  }

  /* ── status strip ─────────────────────────────────────────────────── */

  function getJson(path) {
    return fetch(path, { cache: 'no-store', credentials: 'same-origin' })
      .then(function (r) {
        if (!r.ok) return null;
        return r.json().then(
          function (j) {
            return j;
          },
          function () {
            return null;
          }
        );
      })
      .catch(function () {
        return null;
      });
  }

  function setOnline(up, label) {
    var badge = el('dashOnlineBadge');
    var text = el('dashOnlineText');
    if (badge) {
      if (up) badge.classList.remove('is-down');
      else badge.classList.add('is-down');
    }
    if (text) text.textContent = label || (up ? 'Running' : 'Unreachable');
  }

  function refreshStatus() {
    var host = el('dashHostPill');
    // Empty means "not filled yet" (the pill starts empty, and `.pm-pill:empty`
    // hides it), so the placeholder dash the markup used to carry is gone.
    if (host && !host.textContent.trim()) {
      host.textContent = window.location.host || 'localhost';
    }
    return getJson('/api/version').then(function (v) {
      if (!v) {
        setOnline(false);
        return;
      }
      setOnline(true);
      var pill = el('dashVersion');
      if (pill) {
        var ver = v.version || v.app_version || v.current_version || v.server_version;
        pill.textContent = ver ? 'v' + String(ver).replace(/^v/, '') : 'Server';
      }
    });
  }

  /*
   * Update state comes from the RELEASE FEED, never from `/api/version`.
   *
   * `/api/version` is the old WordPress contract: it compares this build against
   * the plugin's `min_version`, which is still the PHP-era `2.7.9`, so
   * `update_needed` is false forever on a Rust build while GitHub already has a
   * newer tag. Measured: `/api/version` said `update_needed:false` while
   * `/api/update/check` said `latest: v3.0.3` against `current: 3.0.2`. See
   * AGENTS.md §8b before changing any of this.
   *
   * Versioning differs per build on purpose: a desktop build is compared with the
   * `pencarimovie-server-<target>.tar.gz` tag, an APP-MANAGED build (the Android
   * APK, which passes `--app-version`) with `apk-server-latest`. The engine already
   * resolves both and reports which one it used (`app_managed`, `apply_blocked_reason`).
   */
  var updateInfo = null;

  /**
   * Close the card.
   *
   * `returnFocus` (or having focus inside the card) sends the caret back to the
   * pill. Closing because the user clicked somewhere else must NOT steal focus
   * from what they clicked.
   */
  function hideUpdateCard(returnFocus) {
    var card = el('dashUpdateCard');
    var pill = el('dashUpdatePill');
    if (!card || card.classList.contains('hidden')) return;
    var inside = card.contains(document.activeElement);
    card.classList.add('hidden');
    if (pill) pill.setAttribute('aria-expanded', 'false');
    if ((returnFocus || inside) && pill) pill.focus();
  }

  function showUpdateCard() {
    var card = el('dashUpdateCard');
    var pill = el('dashUpdatePill');
    if (!card) return;
    card.classList.remove('hidden');
    if (pill) pill.setAttribute('aria-expanded', 'true');
    // Keyboard users land inside the card instead of at the top of the page.
    var order = ['dashUpdateApply', 'dashUpdateLink', 'dashUpdateClose'];
    for (var i = 0; i < order.length; i++) {
      var node = el(order[i]);
      if (node && !node.classList.contains('hidden')) {
        node.focus();
        return;
      }
    }
  }

  function toggleUpdateCard() {
    var card = el('dashUpdateCard');
    if (!card) return;
    if (card.classList.contains('hidden')) showUpdateCard();
    else hideUpdateCard(true);
  }

  /*
   * One card, four update paths. Which one appears is decided by the engine, not
   * guessed here: `can_apply` means this exact binary may replace itself, and
   * `apply_blocked_reason` names why it may not (`container`, `app_managed`,
   * `no_script`, `no_asset`).
   */
  function fillUpdateCard(u) {
    var title = el('dashUpdateTitle');
    var body = el('dashUpdateBody');
    var cmd = el('dashUpdateCmd');
    var apply = el('dashUpdateApply');
    var link = el('dashUpdateLink');
    var msg = el('dashUpdateMsg');
    if (!title || !body) return;
    var clean = function (s) {
      return String(s || '').replace(/^v/, '');
    };
    var latest = clean(u.latest_display || u.latest);
    var current = clean(u.current_display || u.current);
    var reason = String(u.apply_blocked_reason || '');

    title.textContent = latest ? 'Update to v' + latest : 'Update available';
    if (msg) {
      msg.classList.add('hidden');
      msg.textContent = '';
    }
    if (cmd) {
      cmd.classList.add('hidden');
      cmd.textContent = '';
    }
    if (apply) {
      apply.classList.add('hidden');
      apply.disabled = false;
      apply.textContent = 'Update now';
    }

    var text;
    if (u.can_apply) {
      text =
        'This build is v' + current + '. The server downloads the release, checks its SHA-256, then restarts.';
      if (apply) apply.classList.remove('hidden');
    } else if (reason === 'app_managed') {
      // Android: the APK is the unit of update, and its version number is separate
      // from the engine's, so installing the APK is the whole update.
      text =
        'This engine runs inside the PencariMovie Server app, and the app updates itself. ' +
        'Install the newest APK and it brings this engine with it. The app is at ' + current +
        ' and the engine has its own version number, so the APK is the only way to update.';
    } else if (reason === 'container') {
      text = 'This runs in a container. Update it by pulling the new image and recreating the service.';
      if (cmd) {
        cmd.textContent = 'docker compose pull && docker compose up -d';
        cmd.classList.remove('hidden');
      }
    } else if (reason === 'no_asset') {
      text =
        'This build is v' + current + '. The release has no download for this CPU, so it has to be updated by hand.';
    } else {
      text =
        'This build is v' + current + '. There is no apply-update script next to the binary, so it updates by hand: ' +
        'download the release and extract it over this folder. storage/ is not in the archive, so your sessions survive.';
    }
    body.textContent = text;

    if (link) {
      var target = u.notes_url || u.releases_url || '';
      if (target) {
        link.href = target;
        link.textContent = u.notes_url ? 'Release notes' : 'Latest release';
        link.classList.remove('hidden');
      } else {
        link.classList.add('hidden');
      }
    }
  }

  /**
   * POST /api/update/apply — download, verify, restart.
   *
   * Only offered when the engine said `can_apply`. The server exits about a second
   * after answering, so the page is reloaded a few seconds later instead of being
   * left on a dead connection.
   */
  function applyUpdate() {
    var apply = el('dashUpdateApply');
    var msg = el('dashUpdateMsg');
    if (!apply) return;
    apply.disabled = true;
    apply.textContent = 'Downloading\u2026';
    fetch('/api/update/apply', { method: 'POST', body: '{}' })
      .then(function (r) {
        return r.json();
      })
      .then(function (res) {
        if (res && res.ok && res.starting) {
          apply.textContent = 'Restarting\u2026';
          if (msg) {
            msg.textContent = res.message || 'Update applied. The server restarts now.';
            msg.classList.remove('hidden');
          }
          window.setTimeout(function () {
            window.location.reload();
          }, 6000);
          return;
        }
        apply.disabled = false;
        apply.textContent = 'Update now';
        if (msg) {
          msg.textContent =
            (res && (res.error || res.message)) || 'The server refused to apply this update.';
          msg.classList.remove('hidden');
        }
      })
      .catch(function () {
        apply.disabled = false;
        apply.textContent = 'Update now';
        if (msg) {
          msg.textContent = 'Could not reach the server.';
          msg.classList.remove('hidden');
        }
      });
  }

  function refreshUpdate() {
    var pill = el('dashUpdatePill');
    if (!pill) return null;
    return getJson('/api/update/check').then(
      function (u) {
        updateInfo = u && u.ok && u.update_available ? u : null;
        if (updateInfo) {
          var latest = String(updateInfo.latest_display || updateInfo.latest || '').replace(/^v/, '');
          pill.textContent = latest ? 'Update v' + latest : 'Update';
          pill.title = 'A newer release is available. Click to see how to install it.';
          pill.classList.remove('hidden');
          fillUpdateCard(updateInfo);
        } else {
          pill.classList.add('hidden');
          pill.setAttribute('aria-expanded', 'false');
          hideUpdateCard();
        }
      },
      function () {
        updateInfo = null;
        pill.classList.add('hidden');
        pill.setAttribute('aria-expanded', 'false');
        hideUpdateCard();
      }
    );
  }

  /** Pill, close button, Escape, and a click anywhere else. */
  function bindUpdateCard() {
    var pill = el('dashUpdatePill');
    var close = el('dashUpdateClose');
    var apply = el('dashUpdateApply');
    var card = el('dashUpdateCard');
    if (pill) {
      pill.addEventListener('click', function (e) {
        e.stopPropagation();
        if (!updateInfo) return;
        fillUpdateCard(updateInfo);
        toggleUpdateCard();
      });
    }
    if (close) close.addEventListener('click', function () {
      hideUpdateCard(true);
    });
    if (apply) apply.addEventListener('click', applyUpdate);
    if (card) card.addEventListener('click', function (e) {
      e.stopPropagation();
    });
    document.addEventListener('click', function () {
      hideUpdateCard();
    });
    document.addEventListener('keydown', function (e) {
      if (e.key === 'Escape') hideUpdateCard(true);
    });
  }

  /* ── setup checklist ──────────────────────────────────────────────── */

  var CHECK_ROWS = [
    {
      key: 'origin',
      // The card lists what is WRONG, so this row behaves like every other one: it
      // says its name and state, and when all is well the whole list collapses to
      // "All set". There is no button because nothing on this machine fixes an origin
      // outage — the hint names the host and how long the gate has been armed.
      label: 'The PencariMovie API answers',
      section: 'system',
      noAction: true,
      ok: function (d) {
        if (!d || !d.origin) return null;
        return !(Number(d.origin.down_s) > 0) && !(Number(d.origin.resolve_cooldown_s) > 0);
      },
      danger: function (d) {
        return !!(d && d.origin) &&
          (Number(d.origin.down_s) > 0 || Number(d.origin.resolve_cooldown_s) > 0);
      },
      hint: function (d) {
        var o = d && d.origin;
        if (!o) return '';
        var host = o.host || 'the API';
        var down = Number(o.down_s) || 0;
        var pause = Number(o.resolve_cooldown_s) || 0;
        if (down > 0) {
          return host + ' is not answering (down ' + formatUptime(down) +
            '). New titles cannot resolve until it does.';
        }
        if (pause > 0) {
          return host + ': resolves paused ' + formatUptime(pause) +
            '. New titles wait for the origin; playing streams carry on.';
        }
        return host + ' · reachable';
      }
    },
    {
      key: 'bot',
      label: 'A Telegram bot is connected',
      section: 'bots',
      ok: function (d) {
        if (!d) return null;
        if (d.has_session === undefined && d.hasSession === undefined && d.connected === undefined) return null;
        return !!(d.has_session || d.hasSession || d.connected);
      }
    },
    {
      key: 'tunnel',
      label: 'Access from outside your network',
      hint: 'Optional. Only turn this on if you need it — exposing the server publicly is not recommended otherwise.',
      section: 'tunnel',
      // Optional on purpose: it is a choice, not a requirement, and it never
      // counts as an unfinished step or blocks "all set".
      optional: true,
      action: 'Enable tunnel',
      ok: function (d) {
        if (!d) return null;
        return !!(d.enabled && (d.tunnel_url || d.public_url));
      }
    },
    {
      key: 'catalogs',
      label: 'PencariMovie publishes its own catalog rows',
      hint: 'Popular, Trending and the country rows. With this off, clients keep streams and bridged add-ons but show no rows from this server.',
      section: 'catalogs',
      // Optional: it is on by default, and turning it off is a choice, not a
      // fault. Being told to switch it back on would be wrong.
      optional: true,
      action: 'Turn on',
      ok: function (d) {
        if (!d) return null;
        // The endpoint nests the flag: { settings: { catalogs_enabled } }.
        var s = d.settings || d;
        if (typeof s.catalogs_enabled === 'boolean') return s.catalogs_enabled;
        if (typeof d.mode === 'string') return d.mode !== 'disabled';
        return null;
      }
    },
    {
      key: 'password',
      label: 'The dashboard password is still the built-in default',
      section: 'system',
      ok: function (d) {
        if (!d) return null;
        // No password in use at all: there is nothing here to rotate.
        if (!d.enabled) return true;
        // Older server that does not report the flag: do not nag about it.
        if (d.default_password === undefined) return null;
        return !d.default_password;
      },
      // Still on 123456 is not an unfinished step, it is an open door: this
      // row is rendered as danger, not as a neutral to-do.
      danger: function (d) {
        return !!(d && d.enabled && d.default_password === true);
      }
    }
  ];

  function renderChecklist(results) {
    var list = el('pmChecklist');
    var ready = el('pmChecklistReady');
    if (!list || !ready) return;

    var allDone = true;
    var html = '';

    for (var i = 0; i < CHECK_ROWS.length; i++) {
      var row = CHECK_ROWS[i];
      var verdict = row.ok(results[row.key]);
      // A failed fetch is not the user's fault: treat it as satisfied rather
      // than nagging about a step that may already be done.
      var ok = verdict === null ? true : verdict;
      var optional = !!row.optional;
      var danger = !ok && typeof row.danger === 'function' ? row.danger(results[row.key]) : false;
      // `hint` may be a live sentence built from the response, not only a constant.
      var hint = typeof row.hint === 'function' ? row.hint(results[row.key]) : row.hint;
      if (!ok && !optional) allDone = false;
      var cls = ok ? ' pm-check--ok' : danger ? ' pm-check--danger' : optional ? ' pm-check--optional' : '';
      html +=
        '<li class="pm-check' + cls + '">' +
        '<span class="pm-check__mark" aria-hidden="true">' + (ok ? '\u2713' : danger ? '!' : '') + '</span>' +
        '<span class="pm-check__text">' + row.label +
        (hint ? '<small class="pm-check__hint">' + hint + '</small>' : '') +
        '</span>' +
        // A row with nothing local to fix must not grow a dead button.
        (ok || row.noAction
          ? ''
          : '<button type="button" class="pm-btn pm-btn--sm' +
            (danger ? ' pm-btn--danger' : '') +
            '" data-go="' + row.section + '">' +
            (danger ? 'Change it' : row.action || 'Fix') + '</button>') +
        '</li>';
    }

    list.innerHTML = html;
    list.classList.toggle('hidden', allDone);
    ready.classList.toggle('hidden', !allDone);

    var buttons = list.querySelectorAll('[data-go]');
    for (var b = 0; b < buttons.length; b++) {
      buttons[b].addEventListener('click', function () {
        switchTo(this.getAttribute('data-go'));
      });
    }
  }

  var checklistBusy = false;

  function refreshChecklist() {
    if (checklistBusy) return;
    checklistBusy = true;
    Promise.all([
      getJson('/api/session'),
      getJson('/api/tunnel/status'),
      getJson('/api/catalog-settings'),
      getJson('/api/auth/status'),
      // The origin row is the only checklist entry that is live state rather than a
      // step, so it reads the same `/api/diagnostics` the Live tiles use.
      getJson('/api/diagnostics')
    ]).then(
      function (rows) {
        renderChecklist({
          bot: rows[0],
          tunnel: rows[1],
          catalogs: rows[2],
          password: rows[3],
          origin: rows[4]
        });
        checklistBusy = false;
      },
      function () {
        checklistBusy = false;
      }
    );
  }

  /* ── Live tiles ───────────────────────────────────────────────────── */

  var LIVE_POLL_MS = 5000;
  var liveTimer = null;
  var liveBusy = false;

  function setTile(id, value, unit, state) {
    var tile = el(id);
    if (!tile) return;
    tile.setAttribute('data-state', state || 'ok');
    var v = tile.querySelector('.pm-stat__value');
    var u = tile.querySelector('.pm-stat__unit');
    if (v) v.textContent = value;
    if (u) u.textContent = unit || '';
  }

  function formatUptime(secs) {
    var s = Math.max(0, Math.floor(Number(secs) || 0));
    var d = Math.floor(s / 86400);
    var h = Math.floor((s % 86400) / 3600);
    var m = Math.floor((s % 3600) / 60);
    if (d) return d + 'd ' + h + 'h';
    if (h) return h + 'h ' + m + 'm';
    if (m) return m + 'm';
    return s + 's';
  }

  function formatMegabytes(mb) {
    var n = Math.max(0, Number(mb) || 0);
    if (n >= 1024) return [(n / 1024).toFixed(n >= 10240 ? 0 : 1), 'GB'];
    return [String(Math.round(n)), 'MB'];
  }

  function renderLive(diag, session) {
    var activity = (diag && diag.activity) || {};
    var info = (diag && diag.info) || {};
    var dash = '\u2014';

    /*
     * "1 / 1" = one stream playing one distinct title. They belong together: a
     * stream count above the title count only means the same film is being pulled
     * through several bodies (a seek, or a segmented download manager), which on
     * its own looks like a fault and in this pair reads as normal.
     */
    var streams = activity.streams == null ? null : Number(activity.streams);
    var titles = activity.titles == null ? null : Number(activity.titles);
    if (streams == null || titles == null) setTile('pmLiveStreams', dash);
    else setTile('pmLiveStreams', streams + ' / ' + titles);

    setTile('pmLiveOnline', activity.online != null ? String(activity.online) : dash);

    if (activity.cpu_percent == null) setTile('pmLiveCpu', dash);
    else setTile('pmLiveCpu', String(Math.round(activity.cpu_percent)), '%');

    /*
     * "617 MB of 1 GB" — the ceiling is the CONTAINER's (cgroup), not the host's.
     * `total_memory_mb` is that ceiling; `host_total_memory_mb` is the machine's RAM
     * and must never be shown as available to us, because on a dyno it says 62 GB
     * while the process dies at 1 GB. The note says which of the two it is.
     */
    var ram = info.process_rss_mb == null ? null : formatMegabytes(info.process_rss_mb);
    var limit = info.total_memory_mb == null ? 0 : Number(info.total_memory_mb);
    var ramNote = el('pmLiveRamNote');
    if (!ram) setTile('pmLiveRam', dash);
    else if (limit > 0) {
      var tot = formatMegabytes(limit);
      setTile('pmLiveRam', ram[0] + ' ' + ram[1], '/ ' + tot[0] + ' ' + tot[1]);
      if (ramNote) ramNote.textContent = info.container ? 'of this container\u2019s limit' : 'of this machine';
    } else {
      setTile('pmLiveRam', ram[0], ram[1]);
      if (ramNote) ramNote.textContent = 'in use';
    }

    if (info.proc_uptime_s == null) setTile('pmLiveUptime', dash);
    else setTile('pmLiveUptime', formatUptime(info.proc_uptime_s));

    /*
     * Bots are a POOL. `status` on a `bot_pool` row is what was written when the
     * bot was added, so counting it always says "everyone online"; the server
     * computes `bot_pool_health.ready` with the same two tests `pick_pool_bot()`
     * uses — a session file on this instance, and no flood cooldown.
     */
    var health = (session && session.bot_pool_health) || null;
    var botsTotal = health ? Number(health.total) || 0 : 0;
    var botsReady = health ? Number(health.ready) || 0 : 0;
    if (!session) setTile('pmLiveBots', dash, 'up', 'ok');
    else if (botsTotal > 0) {
      setTile(
        'pmLiveBots',
        botsReady + ' / ' + botsTotal,
        'up',
        botsReady === botsTotal ? 'ok' : botsReady > 0 ? 'warn' : 'bad'
      );
    } else setTile('pmLiveBots', dash, 'up', 'bad');

    // The REST origin is not a tile: it is a row in the setup checklist, rendered by
    // `renderChecklist()` from the same `/api/diagnostics`. Live tiles stay counters.
  }

  function refreshLive() {
    if (liveBusy || document.hidden) return;
    liveBusy = true;
    // Two endpoints, not three: `/api/cache/stats` was only feeding the cache tile,
    // which no longer exists — its hit rate is a tuning number, not a user fact.
    Promise.all([getJson('/api/diagnostics'), getJson('/api/session')]).then(
      function (rows) {
        renderLive(rows[0], rows[1]);
        liveBusy = false;
      },
      function () {
        liveBusy = false;
      }
    );
  }

  function startLive() {
    stopLive();
    refreshLive();
    liveTimer = window.setInterval(function () {
      // A hidden tab still fires the timer; skip the fetch, it resumes on show.
      if (!document.hidden) refreshLive();
    }, LIVE_POLL_MS);
  }

  function stopLive() {
    if (liveTimer !== null) {
      window.clearInterval(liveTimer);
      liveTimer = null;
    }
  }

  /* ── wiring ───────────────────────────────────────────────────────── */

  function hideLoading() {
    // The element is gone from the dashboard markup; this stays as a guard in
    // case an older cached app.js re-creates nothing but something else shows it.
    var screen = el('loadingScreen');
    if (screen) {
      screen.classList.add('hidden');
      screen.setAttribute('aria-hidden', 'true');
    }
  }

  /*
   * Disconnecting reaches the guest-bot provision path, which calls
   * _showLoadingScreen() and then showSettingsGate() to swap to the token face.
   * The gate no longer opens, so nothing re-read the session and the Bots panel
   * stayed on the connected face. Re-check after the two async hops settle.
   */
  function watchSessionChanges() {
    var ids = ['settingsDisconnectBtn', 'logoutBtn'];
    for (var i = 0; i < ids.length; i++) {
      var btn = el(ids[i]);
      if (!btn) continue;
      btn.addEventListener('click', function () {
        window.setTimeout(applyBotsVisibility, 1200);
        window.setTimeout(applyBotsVisibility, 4000);
        window.setTimeout(refreshChecklist, 4000);
      });
    }
  }

  function boot() {
    hideLoading();

    var nav = el('pmNav');
    if (nav) {
      nav.addEventListener('click', function (e) {
        var target = e.target.closest ? e.target.closest('[data-section]') : null;
        if (target) switchTo(target.getAttribute('data-section'));
      });
    }

    bindUpdateCard();
    watchSessionChanges();

    // The ?token= flow finishes inside app.js after boot; it forwards the
    // result so the Bots section can show what happened instead of a modal.
    function applyTokenResult(detail) {
      detail = detail || {};
      switchTo('bots');
      var target = el('settingsConnectedStatus') || el('settingsStatus');
      if (!target) return;
      target.textContent = detail.message || '';
      target.style.color = detail.success ? 'var(--pm-status-online)' : 'var(--pm-status-danger)';
    }
    window.addEventListener('pm:token-result', function (ev) {
      applyTokenResult(ev && ev.detail);
    });

    // Returning to a backgrounded tab shows stale numbers until the next tick.
    document.addEventListener('visibilitychange', function () {
      if (!document.hidden && current === 'overview') refreshLive();
    });

    window.addEventListener('hashchange', function () {
      var route = readHash();
      if (route !== current) applyRoute(route, false);
    });

    applyRoute(readHash(), false);

    // app.js can finish the ?token= flow before this listener exists. It
    // stashes the result and rewrites the hash to #bots; replay the notice so
    // the user always sees what the link did.
    if (window.__pmTokenResult) {
      applyTokenResult(window.__pmTokenResult);
      window.__pmTokenResult = null;
    }

    // Keep the legacy global adapters pointing at sections. app.js re-assigns
    // showSettingsGate internally, but that method now returns immediately
    // (there is no #settingsGate element), so this is safe to re-assert.
    window.showSettingsGate = function () {
      switchTo('bots');
    };
    window.openAddonModal = function () {
      switchTo('addon');
    };

    // Disarm the startup guard. It exists to catch failures while the shell is
    // still loading, and both of its listeners act only while `active` is true —
    // which is cleared solely by ready(). The dashboard path returns from
    // continueInit() before the legacy branch that calls ready(), so the guard
    // used to stay armed all session and turned any later handler error into a
    // full "Nuvio TV could not start" screen.
    if (window.NuvioBootGuard && typeof window.NuvioBootGuard.ready === 'function') {
      window.NuvioBootGuard.ready();
    }

    refreshStatus();
    refreshUpdate();
    setInterval(function () {
      refreshStatus();
      refreshUpdate();
    }, 20000);

    // Give app.js's own provision attempt a head start, then make sure a bot
    // exists — on refresh, before the user has clicked anything.
    window.setTimeout(ensureBot, 2500);
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', boot);
  } else {
    boot();
  }
})();
