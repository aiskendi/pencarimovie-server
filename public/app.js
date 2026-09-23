 /**
 * PencariMovieApp — Merged streaming UI + session management + file download.
 *
 * Combines:
 *   - StreamApp UI (hero, trending, category rows, search, modal, file cards)
 *   - Session management (bot login, logout, session check via /api/session)
 *   - File Detail Page (resolve short_code → file_id_mt, Stream + Download)
 *   - Backend proxy for streaming data (/api/proxy-stream)
 *   - MadelineProto download integration (/api/download)
 */
class PencariMovieApp {
  constructor() {
    // ── Config ──
    this.localApiBase = window.location.origin;
    this.wpApiBase = 'https://pencarimovie.com/wp-json/pencarimovie-server/v1';
    this.wpAjaxUrl = 'https://pencarimovie.com/wp-admin/admin-ajax.php';
    this.siteName = 'PencariMovie';

    // ── State ──
    this.categories = [];
    this.trending = [];
    this.posts = {};
    this.searchTimeout = null;
    this.isSearchOpen = false;
    this.isModalOpen = false;
    this.isMobileNavOpen = false;
    this.heroIndex = 0;
    this.heroInterval = null;
    this._currentPostId = null;
    this._suppressHash = false;
    this._cameFromPost = false;
    this._previousPostData = null;
    this._savedSearchQuery = '';
    this._knownFiles = new Map(); // short_code -> file object (tracks multi-part metadata)

    // ── Category page state ──
    this._categorySlug = '';
    this._categoryName = '';
    this._categoryOffset = 0;
    this._categoryHasMore = true;
    this._categoryLoading = false;
    this._categoryObserver = null;
    this._isCategoryPageOpen = false;
    this._cameFromCategory = false;

    // ── Cache ──
    this._cache = new Map();
    this._cachePrefix = 'pencarimovie_cache:';

    // Session state
    this.version = '2.1.8';
    this.botId = '';
    this.botUsername = '';
    this.botName = '';
    this.apiSecret = '';
    this.hasSession = false;
    this.lanIp = localStorage.getItem('pm.lan_ip') || '';
    this.listenPort = 8088;
    this.deviceId = '';
    this.tunnelUrl = localStorage.getItem('pm.tunnel_url') || '';
    this.tunnelPublicUrl = localStorage.getItem('pm.tunnel_public_url') || '';
    this.tunnelCustomDomain = localStorage.getItem('pm.tunnel_custom_domain') || '';
    this.tunnelCustomDomains = [];
    try {
      const cd = localStorage.getItem('pm.tunnel_custom_domains');
      if (cd) this.tunnelCustomDomains = JSON.parse(cd);
    } catch (e) {}
    this.tunnelEnabled = localStorage.getItem('pm.tunnel_enabled') === '1';
    this.tunnelRunning = false;
    this._tunnelBusy = false;
    this.sponsor = null;
    this._updateAddonModalUrls = () => {};
    this._authToken = localStorage.getItem('pm.auth') || '';
    this._restoreCachedSession();

    // ── DOM ref shortcuts ──
    this.$ = (sel) => document.querySelector(sel);
    this.$$ = (sel) => document.querySelectorAll(sel);

    // ── Init ──
    this._ready = this.init();
  }

  // ══════════════════════════════════════════════════════════════
  //  INIT
  // ══════════════════════════════════════════════════════════════

  async init() {
    this.detectTelegram();
    this.bindGlobalEvents();

    // ── Auth gate — must pass before anything else loads ──
    if (!(await this.checkAuth())) {
      return; // Auth gate is showing; stop init until the user logs in
    }

    await this.continueInit();
  }

  async continueInit() {
    this.loadLanIp();

    // ── Instant Modal Opening if requested by hash (zero waiting) ──
    const initHash = window.location.hash;
    if (initHash === '#configure' || initHash === '#addon') {
      const addonM = document.getElementById('addonModal');
      if (addonM) {
        addonM.classList.remove('hidden');
        addonM.setAttribute('aria-hidden', 'false');
      }
      this.openAddonModal?.();
    } else if (initHash === '#settings') {
      if (this.hasSession) {
        const sGate = document.getElementById('settingsGate');
        if (sGate) {
          sGate.classList.remove('hidden');
          sGate.setAttribute('aria-hidden', 'false');
        }
        this.showSettingsGate({ forceToken: false });
      } else {
        // Keep loading indicator visible while session check or background auto-provision runs
        const sGate = document.getElementById('settingsGate');
        if (sGate) sGate.classList.add('hidden');
      }
    }

    // ── Version check — block everything if update is required ──
    try {
      const versionInfo = await this.checkVersion();
      if (versionInfo && versionInfo.update_needed) {
        this._hideLoadingScreen();
        this.showUpdateRequired(versionInfo);
        return; // Stop — overlay blocks all interaction
      }
    } catch (e) {
      console.warn('Version check failed, proceeding:', e);
    }

    // ── Probe existing session status first ──
    try {
      await this.loadSessionStatus();
    } catch (e) {
      console.warn('Session check failed:', e);
      if (!this.hasSession) {
        this._restoreCachedSession();
      }
    }

    // ── Check for ?token= URL parameter for adding bot or 1-click auto-login ──
    const urlParams = new URLSearchParams(window.location.search);
    const tokenFromUrl = urlParams.get('token');
    let tokenAddResult = null;

    if (tokenFromUrl && tokenFromUrl.trim() !== '') {
      const cleanToken = tokenFromUrl.trim();
      try {
        if (this.hasSession) {
          // Existing active session exists — add bot token directly into the pool
          const addResp = await this.requestJson(`${this.localApiBase}/api/bots/add`, {
            method: 'POST',
            body: JSON.stringify({ bot_token: cleanToken })
          });
          if (addResp?.ok) {
            await this.loadSessionStatus();
            await this.loadBotPool();
            const botInfo = addResp?.results?.[0];
            const botLabel = botInfo?.bot_name ? `${botInfo.bot_name} (@${botInfo.bot_username || botInfo.bot_id})` : `Bot ${botInfo?.bot_id || ''}`;
            tokenAddResult = {
              success: true,
              message: `✓ Successfully added ${botLabel} to Bot Pool!`
            };
          } else {
            const errDetail = addResp?.results?.[0]?.error || addResp?.message || 'Failed to connect bot.';
            tokenAddResult = {
              success: false,
              message: `✕ Failed to add bot: ${errDetail}`,
              token: cleanToken
            };
          }
        } else {
          // No session yet — perform primary login
          const input = this.$('#botTokenInput');
          if (input) input.value = cleanToken;
          await this.saveSettings(cleanToken);
          if (this.hasSession) {
            tokenAddResult = {
              success: true,
              message: `✓ Connected as ${this.botName || 'Primary Bot'} (@${this.botUsername})`
            };
          } else {
            tokenAddResult = {
              success: false,
              message: `✕ Login failed with provided token. Please verify token and try again.`,
              token: cleanToken
            };
          }
        }

        // Clean query parameter from browser history
        urlParams.delete('token');
        const newSearch = urlParams.toString() ? `?${urlParams.toString()}` : '';
        window.history.replaceState({}, document.title, `${window.location.pathname}${newSearch}${window.location.hash}`);
      } catch (err) {
        tokenAddResult = {
          success: false,
          message: `✕ Error adding bot: ${err.message}`,
          token: cleanToken
        };
      }
    }

    // Loading screen is hidden by showSettingsGate() or hideSettingsGate()
    if (this.hasSession) {
      this.hideSettingsGate();
      this.updateBotBadge();

      // Restore file detail origin context from sessionStorage (survives page refresh)
      this._restoreFileDetailContext();

      // Check for deep links or ?token= feedback popup
      const hash = window.location.hash;
      if (tokenAddResult) {
        await this.loadInitialData();
        this.showSettingsGate({
          forceToken: false,
          message: tokenAddResult.message,
          messageType: tokenAddResult.success ? 'success' : 'error'
        });
        if (!tokenAddResult.success && tokenAddResult.token) {
          const addBotsSection = this.$('#addBotsSection');
          const bulkInput = this.$('#bulkBotTokensInput');
          const addBotsStatus = this.$('#addBotsStatus');
          if (addBotsSection) addBotsSection.classList.remove('hidden');
          if (bulkInput) {
            bulkInput.value = tokenAddResult.token;
            bulkInput.focus();
          }
          if (addBotsStatus) addBotsStatus.textContent = tokenAddResult.message;
        }
      } else if (hash === '#settings') {
        // Instant show settings card without waiting for initial catalog/media data
        this.showSettingsGate({ forceToken: !this.botId });
        this.loadInitialData().catch((err) => console.warn('Background init data load failed:', err));
      } else if (hash === '#configure' || hash === '#addon') {
        // Instant show addon / configure modal immediately
        this.openAddonModal?.();
        this.loadInitialData().catch((err) => console.warn('Background init data load failed:', err));
      } else if (hash.startsWith('#file/')) {
        const shortCode = hash.replace('#file/', '');
        // Load main page data in background so it's rendered when user goes back
        this.loadInitialData().catch((err) => console.warn('Background init data load failed:', err));
        await this.openFileDetail(shortCode);
      } else {
        await this.loadInitialData();
        this._checkHash();
      }
    } else {
      const gateMessage = tokenAddResult?.message || this.provisionError || null;
      const isClockErr = gateMessage && (gateMessage.toLowerCase().includes('clock') || gateMessage.toLowerCase().includes('ntp') || gateMessage.toLowerCase().includes('time'));
      if (hash === '#configure' || hash === '#addon') {
        this.openAddonModal?.();
      } else {
        this.showSettingsGate({
          forceToken: true,
          message: gateMessage,
          messageType: tokenAddResult?.success ? 'success' : (isClockErr ? 'error' : (this.provisionError ? 'info' : 'error'))
        });
      }
      if (tokenAddResult && !tokenAddResult.success && tokenAddResult.token) {
        const input = this.$('#botTokenInput');
        if (input) input.value = tokenAddResult.token;
      }
    }

    // Start the periodic tunnel watchdog (revives cloudflared if it dies).
    this._startTunnelWatchdog();
  }

  detectTelegram() {
    if (typeof Telegram !== 'undefined' && Telegram.WebApp) {
      document.body.classList.add('tg');
      Telegram.WebApp.expand();
      Telegram.WebApp.enableClosingConfirmation();
      Telegram.WebApp.onEvent('backButtonClicked', () => {
        if (this.isModalOpen) {
          this.closeModal();
        } else if (this.isSearchOpen) {
          this.closeSearch();
        } else if (this.isFileDetailOpen()) {
          this.closeFileDetail();
        } else {
          window.history.back();
        }
      });
    }
  }

  bindGlobalEvents() {
    // ── Auth gate ──
    const authBtn = this.$('#authConnectBtn');
    if (authBtn) {
      authBtn.addEventListener('click', () => this.submitAuthPassword());
    }
    const authInput = this.$('#authPasswordInput');
    if (authInput) {
      authInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') this.submitAuthPassword();
      });
    }

    // ── Settings gate ──
    this.$('#connectBtn').addEventListener('click', () => {
      this.saveSettings().catch((err) => {
        const el = this.$('#settingsStatus');
        if (el) el.textContent = 'Error: ' + err.message;
      });
    });

    const afterLogout = () => {
      this.clearSession().then(() => {
        this._clearCachedSession();
        this.$('#botTokenInput').value = '';
        this.showSettingsGate();
      });
    };

    this.$('#logoutBtn').addEventListener('click', afterLogout);
    this.$('#settingsDisconnectBtn').addEventListener('click', afterLogout);

    this.$('#settingsClose').addEventListener('click', () => {
      this.closeSettingsGate();
    });

    this.$('#settingsBtn').addEventListener('click', () => {
      this.showSettingsGate();
    });

    // ── Multi-Bot Manager Listeners ──
    const toggleAddBotsBtn = this.$('#toggleAddBotsBtn');
    const addBotsSection = this.$('#addBotsSection');
    const submitAddBotsBtn = this.$('#submitAddBotsBtn');
    const cancelAddBotsBtn = this.$('#cancelAddBotsBtn');
    const bulkInput = this.$('#bulkBotTokensInput');
    const addBotsStatus = this.$('#addBotsStatus');

    if (toggleAddBotsBtn && addBotsSection) {
      toggleAddBotsBtn.addEventListener('click', () => {
        addBotsSection.classList.toggle('hidden');
        if (!addBotsSection.classList.contains('hidden') && bulkInput) {
          bulkInput.focus();
        }
      });
    }

    if (cancelAddBotsBtn && addBotsSection) {
      cancelAddBotsBtn.addEventListener('click', () => {
        addBotsSection.classList.add('hidden');
        if (bulkInput) bulkInput.value = '';
        if (addBotsStatus) addBotsStatus.textContent = '';
      });
    }

    if (submitAddBotsBtn && bulkInput) {
      submitAddBotsBtn.addEventListener('click', async () => {
        const text = bulkInput.value.trim();
        if (!text) {
          if (addBotsStatus) addBotsStatus.textContent = 'Please paste at least one bot token.';
          return;
        }
        submitAddBotsBtn.disabled = true;
        submitAddBotsBtn.textContent = 'Connecting...';
        if (addBotsStatus) addBotsStatus.textContent = 'Validating and connecting bots...';

        try {
          const resp = await this.requestJson(`${this.localApiBase}/api/bots/add`, {
            method: 'POST',
            body: JSON.stringify({ tokens_text: text })
          });

          if (resp.ok) {
            bulkInput.value = '';
            if (addBotsSection) addBotsSection.classList.add('hidden');
            if (addBotsStatus) addBotsStatus.textContent = '';
            await this.loadBotPool();
          } else {
            if (addBotsStatus) addBotsStatus.textContent = resp.message || 'Failed to add bots.';
          }
        } catch (err) {
          if (addBotsStatus) addBotsStatus.textContent = err.message || 'Error connecting bots.';
        } finally {
          submitAddBotsBtn.disabled = false;
          submitAddBotsBtn.textContent = 'Connect Bots';
        }
      });
    }

    // ── Nuvio / Stremio Addon Modal Card ──
    const addonModal = this.$('#addonModal');
    const addonBtn = this.$('#addonBtn');
    const addonClose = this.$('#addonModalClose');
    const addonModalTitle = this.$('#addonModalTitle');
    const addonModalDesc = this.$('#addonModalDesc');
    const copyAddonBtn = this.$('#copyAddonManifestBtn');
    const copyAddonLanBtn = this.$('#copyAddonManifestLanBtn');
    const copyAddonTunnelBtn = this.$('#copyAddonManifestTunnelBtn');
    const copyAddonEclipseBtn = this.$('#copyAddonManifestEclipseBtn');
    const manifestInput = this.$('#addonManifestInput');
    const manifestLanInput = this.$('#addonManifestLanInput');
    const manifestTunnelInput = this.$('#addonManifestTunnelInput');
    const manifestEclipseInput = this.$('#addonManifestEclipseInput');
    const addonLanField = this.$('#addonLanField');
    const addonLocalField = this.$('#addonLocalField');
    const addonTunnelField = this.$('#addonTunnelField');
    const addonStremioDirectBtn = this.$('#addonStremioDirectBtn');
    const addonStremioLocalDirectBtn = this.$('#addonStremioLocalDirectBtn');
    const addonStremioSync = this.$('#addonStremioSync');
    const addonNuvioInstructions = this.$('#addonNuvioInstructions');
    const copiedStatus = this.$('#addonCopiedStatus');
    const stremioSyncUrlPreview = this.$('#stremioSyncUrlPreview');
    const stremioSyncModeToggle = this.$('#stremioSyncModeToggle');
    const stremioSyncModeLan = this.$('#stremioSyncModeLan');
    const stremioSyncModeLocal = this.$('#stremioSyncModeLocal');
    const stremioSyncInstallBtn = this.$('#stremioSyncInstallBtn');
    const stremioSyncStatus = this.$('#stremioSyncStatus');

    // ── Access token card ──
    const addonTokenInput = this.$('#addonTokenInput');
    const addonTokenCopyBtn = this.$('#addonTokenCopyBtn');
    const addonTokenRotateBtn = this.$('#addonTokenRotateBtn');
    const addonTokenStatus = this.$('#addonTokenStatus');

    const showTokenStatus = (msg, ok = true) => {
      if (!addonTokenStatus) return;
      addonTokenStatus.textContent = msg;
      addonTokenStatus.classList.remove('hidden');
      addonTokenStatus.style.color = ok ? '#00d26a' : '#ff5c5c';
      setTimeout(() => addonTokenStatus.classList.add('hidden'), 3000);
    };

    const refreshTokenField = () => {
      if (addonTokenInput) addonTokenInput.value = this._authToken || '';
    };

    if (addonTokenCopyBtn && addonTokenInput) {
      addonTokenCopyBtn.addEventListener('click', async () => {
        await this.copyToClipboard(addonTokenInput.value, addonTokenInput);
        if (addonTokenCopyBtn) {
          const orig = addonTokenCopyBtn.textContent;
          addonTokenCopyBtn.textContent = '✓';
          setTimeout(() => { addonTokenCopyBtn.textContent = orig; }, 1500);
        }
        showTokenStatus('✓ Token copied');
      });
    }
    if (addonTokenRotateBtn) {
      addonTokenRotateBtn.addEventListener('click', async () => {
        if (!confirm('Regenerate the token? Every remote device using this addon will need the new URL.')) return;
        try {
          const res = await this.requestJson(`${this.localApiBase}/api/auth/token/rotate`, { method: 'POST' });
          if (res?.ok && res.token) {
            this._authToken = res.token;
            localStorage.setItem('pm.auth', res.token);
            refreshTokenField();
            updateAddonModalUrls();
            showTokenStatus('✓ Token regenerated');
          }
        } catch (err) {
          showTokenStatus('Failed: ' + err.message, false);
        }
      });
    }

    // ── Catalog settings elements ──
    const catalogModeEnabled = this.$('#catalogModeEnabled');
    const catalogModeDisabled = this.$('#catalogModeDisabled');
    const addonCatalogDetails = this.$('#addonCatalogDetails');
    const addonCatalogStatusBadge = this.$('#addonCatalogStatusBadge');
    const catTypeMovies = this.$('#catTypeMovies');
    const catTypeSeries = this.$('#catTypeSeries');
    const catTypeOther = this.$('#catTypeOther');
    const addonCatalogSpecialList = this.$('#addonCatalogSpecialList');
    const addonCatalogList = this.$('#addonCatalogList');
    const catSelectAllSpecialBtn = this.$('#catSelectAllSpecialBtn');
    const catDeselectAllSpecialBtn = this.$('#catDeselectAllSpecialBtn');
    const catSelectAllBtn = this.$('#catSelectAllBtn');
    const catDeselectAllBtn = this.$('#catDeselectAllBtn');
    const catSaveSettingsBtn = this.$('#catSaveSettingsBtn');
    const catSaveStatus = this.$('#catSaveStatus');

    let catalogSettingsState = {
      catalogs_enabled: true,
      enabled_types: { movie: true, series: true },
      enabled_catalogs: {}
    };
    let catalogOptionsState = {};

    const renderCatalogOptions = () => {
      if (addonCatalogSpecialList) addonCatalogSpecialList.innerHTML = '';
      if (addonCatalogList) addonCatalogList.innerHTML = '';

      const moviesActive = !!(catTypeMovies && catTypeMovies.checked);
      const seriesActive = !!(catTypeSeries && catTypeSeries.checked);
      const otherActive = !!(catTypeOther && catTypeOther.checked);

      Object.entries(catalogOptionsState).forEach(([id, info]) => {
        // Filter display based on media type checkbox
        if (info.type === 'movie' && !moviesActive) return;
        if (info.type === 'series' && !seriesActive) return;
        if (info.type === 'other' && !otherActive) return;

        const isChecked = catalogSettingsState.enabled_catalogs[id] !== false;

        const label = document.createElement('label');
        label.className = 'addon-catalog-item';
        label.title = info.name;

        const chk = document.createElement('input');
        chk.type = 'checkbox';
        chk.value = id;
        chk.checked = isChecked;

        chk.addEventListener('change', () => {
          catalogSettingsState.enabled_catalogs[id] = chk.checked;
        });

        const span = document.createElement('span');
        span.textContent = info.name;

        label.appendChild(chk);
        label.appendChild(span);

        if (info.group === 'special') {
          if (addonCatalogSpecialList) addonCatalogSpecialList.appendChild(label);
        } else {
          if (addonCatalogList) addonCatalogList.appendChild(label);
        }
      });
    };

    const upstreamAddonsList = this.$('#upstreamAddonsList');
    const upstreamAddonInput = this.$('#upstreamAddonInput');
    const upstreamAddonAddBtn = this.$('#upstreamAddonAddBtn');
    const upstreamAddonStatus = this.$('#upstreamAddonStatus');
    const upstreamEnabledToggle = this.$('#upstreamEnabledToggle');

    const renderUpstreamAddons = () => {
      if (!upstreamAddonsList) return;
      upstreamAddonsList.innerHTML = '';
      const list = catalogSettingsState.upstream_manifests || [];
      if (list.length === 0) {
        const emptyNotice = document.createElement('div');
        emptyNotice.style.fontSize = '0.74rem';
        emptyNotice.style.color = '#777';
        emptyNotice.style.fontStyle = 'italic';
        emptyNotice.textContent = 'No upstream addons configured yet.';
        upstreamAddonsList.appendChild(emptyNotice);
        return;
      }

      list.forEach((item, idx) => {
        const row = document.createElement('div');
        row.style.cssText = 'display: flex; align-items: center; justify-content: space-between; background: rgba(255,255,255,0.05); padding: 5px 10px; border-radius: 6px; border: 1px solid rgba(255,255,255,0.1); gap: 8px;';

        const info = document.createElement('div');
        info.style.cssText = 'overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1;';
        info.innerHTML = `<strong style="font-size: 0.8rem; color: #fff;">${item.name || 'Addon'}</strong> <span style="font-size: 0.72rem; color: #888;">(${item.version || '1.0.0'})</span><br/><span style="font-size: 0.7rem; color: #aaa; font-family: monospace;">${item.url}</span>`;

        const delBtn = document.createElement('button');
        delBtn.type = 'button';
        delBtn.style.cssText = 'background: none; border: none; color: #ff5e57; cursor: pointer; font-size: 0.82rem; padding: 4px;';
        delBtn.innerHTML = '<i class="fas fa-trash"></i>';
        delBtn.title = 'Remove addon';
        delBtn.addEventListener('click', () => {
          catalogSettingsState.upstream_manifests.splice(idx, 1);
          renderUpstreamAddons();
          updateCatalogModeUI();
        });

        row.appendChild(info);
        row.appendChild(delBtn);
        upstreamAddonsList.appendChild(row);
      });
    };

    if (upstreamAddonAddBtn && upstreamAddonInput) {
      upstreamAddonAddBtn.addEventListener('click', async () => {
        const url = upstreamAddonInput.value.trim();
        if (!url) return;
        upstreamAddonAddBtn.disabled = true;
        const origText = upstreamAddonAddBtn.textContent;
        upstreamAddonAddBtn.textContent = 'Validating...';
        if (upstreamAddonStatus) {
          upstreamAddonStatus.classList.add('hidden');
          upstreamAddonStatus.className = 'addon-catalog-status-msg';
        }

        try {
          const res = await fetch(`${this.localApiBase}/api/validate-manifest`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ url })
          });
          const resData = await res.json().catch(() => null);
          if (res.ok && resData?.ok && resData?.manifest) {
            const m = resData.manifest;
            if (!catalogSettingsState.upstream_manifests) {
              catalogSettingsState.upstream_manifests = [];
            }
            // Check if already present
            const exists = catalogSettingsState.upstream_manifests.some((existing) => existing.url === m.url);
            if (!exists) {
              catalogSettingsState.upstream_manifests.push({
                id: m.id,
                name: m.name,
                version: m.version,
                url: m.url
              });
              renderUpstreamAddons();
              updateCatalogModeUI();
              upstreamAddonInput.value = '';
              if (upstreamAddonStatus) {
                upstreamAddonStatus.textContent = `✓ Added "${m.name}" (${m.version || 'v1.0.0'}). Click "Save Catalog Settings" below to apply.`;
                upstreamAddonStatus.classList.remove('hidden');
                upstreamAddonStatus.style.color = '#00d26a';
              }
            } else {
              throw new Error('This manifest URL is already added.');
            }
          } else {
            throw new Error(resData?.error || 'Invalid manifest response.');
          }
        } catch (err) {
          if (upstreamAddonStatus) {
            upstreamAddonStatus.textContent = `✕ ${err.message || 'Validation failed.'}`;
            upstreamAddonStatus.classList.remove('hidden');
            upstreamAddonStatus.style.color = '#ff5e57';
          }
        } finally {
          upstreamAddonAddBtn.disabled = false;
          upstreamAddonAddBtn.textContent = origText;
        }
      });
    }

    // Master upstream toggle: one switch disables every bridged upstream
    // catalog/stream/subtitle. The configured list is kept so re-enabling
    // restores it without re-adding URLs.
    const updateUpstreamEnabledUI = () => {
      const on = catalogSettingsState.upstream_enabled !== false;
      if (upstreamEnabledToggle) upstreamEnabledToggle.checked = on;
      if (upstreamAddonsList) {
        upstreamAddonsList.style.opacity = on ? '1' : '0.45';
        upstreamAddonsList.style.pointerEvents = on ? 'auto' : 'none';
      }
      if (upstreamAddonInput) upstreamAddonInput.disabled = !on;
      if (upstreamAddonAddBtn) upstreamAddonAddBtn.disabled = !on;
    };

    if (upstreamEnabledToggle) {
      upstreamEnabledToggle.addEventListener('change', () => {
        catalogSettingsState.upstream_enabled = upstreamEnabledToggle.checked;
        updateUpstreamEnabledUI();
        updateCatalogModeUI();
      });
    }

    const updateCatalogModeUI = () => {
      const isEnabled = catalogSettingsState.catalogs_enabled;
      if (catalogModeEnabled) catalogModeEnabled.checked = isEnabled;
      if (catalogModeDisabled) catalogModeDisabled.checked = !isEnabled;

      if (addonCatalogDetails) {
        addonCatalogDetails.classList.toggle('hidden', !isEnabled);
      }
      const hasUpstreams = Array.isArray(catalogSettingsState.upstream_manifests) && catalogSettingsState.upstream_manifests.length > 0;
      if (addonCatalogStatusBadge) {
        if (isEnabled) {
          addonCatalogStatusBadge.textContent = 'Enabled';
          addonCatalogStatusBadge.classList.remove('disabled');
        } else if (hasUpstreams) {
          addonCatalogStatusBadge.textContent = 'Upstream Only';
          addonCatalogStatusBadge.classList.remove('disabled');
        } else {
          addonCatalogStatusBadge.textContent = 'Disabled';
          addonCatalogStatusBadge.classList.add('disabled');
        }
      }
    };

    const addonCountrySelect = this.$('#addonCountrySelect');

    const loadServerCountry = async () => {
      try {
        const res = await this.requestJson(`${this.localApiBase}/api/country`);
        if (res?.ok && res?.country) {
          const currentCountry = res.configured_country || '';
          this.country = res.country?.country_code || currentCountry;
          if (addonCountrySelect && Array.isArray(res.available_countries)) {
            addonCountrySelect.innerHTML = '';
            res.available_countries.forEach((ac) => {
              const opt = document.createElement('option');
              opt.value = ac.code;
              if (ac.code === '') {
                opt.textContent = `Auto (${res.country.country_name || res.country.country_code})`;
              } else {
                opt.textContent = `${ac.name} (${ac.code})`;
              }
              addonCountrySelect.appendChild(opt);
            });
            addonCountrySelect.value = currentCountry;
          }
        }
      } catch (err) {
        console.warn('Failed to detect server country:', err);
      }
    };

    if (addonCountrySelect) {
      addonCountrySelect.addEventListener('change', async () => {
        const newCountry = addonCountrySelect.value;
        try {
          const res = await fetch(`${this.localApiBase}/api/country`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ country: newCountry })
          });
          const resData = await res.json().catch(() => null);
          if (res.ok && resData?.ok) {
            catalogSettingsState.country = newCountry;
            this.country = resData.country?.country_code || newCountry;
            // Clear stream cache so fresh region queries are fetched immediately
            this._cacheClear();
            this.loadInitialData().catch((e) => console.warn('Failed to refresh data after country change:', e));
          }
        } catch (err) {
          console.warn('Failed to update country:', err);
        }
      });
    }

    const loadCatalogSettings = async () => {
      loadServerCountry();
      try {
        const res = await this.requestJson(`${this.localApiBase}/api/catalog-settings`);
        if (res?.ok) {
          catalogSettingsState = res.settings || catalogSettingsState;
          catalogOptionsState = res.catalog_options || catalogOptionsState;

          updateUpstreamEnabledUI();

          if (catTypeMovies) {
            catTypeMovies.checked = catalogSettingsState.enabled_types?.movie !== false;
          }
          if (catTypeSeries) {
            catTypeSeries.checked = catalogSettingsState.enabled_types?.series !== false;
          }
          if (catTypeOther) {
            catTypeOther.checked = catalogSettingsState.enabled_types?.other !== false;
          }

          // Stream Configuration: resolutions, qualities, encodes, visual tags, sizes, keywords
          const streamConfig = catalogSettingsState.stream_config || {};
          const resConfig = streamConfig.resolutions || {};
          const qualConfig = streamConfig.qualities || {};
          const encConfig = streamConfig.encodes || {};
          const visConfig = streamConfig.visual_tags || {};

          const res4kEl = this.$('#streamRes4k');
          const res1080pEl = this.$('#streamRes1080p');
          const res720pEl = this.$('#streamRes720p');
          const resSdEl = this.$('#streamResSd');
          const resUnknownEl = this.$('#streamResUnknown');

          if (res4kEl) res4kEl.checked = resConfig['4k'] !== false;
          if (res1080pEl) res1080pEl.checked = resConfig['1080p'] !== false;
          if (res720pEl) res720pEl.checked = resConfig['720p'] !== false;
          if (resSdEl) resSdEl.checked = resConfig['sd'] !== false;
          if (resUnknownEl) resUnknownEl.checked = resConfig['unknown'] !== false;

          const qualRemuxEl = this.$('#streamQualRemux');
          const qualBlurayEl = this.$('#streamQualBluray');
          const qualWebdlEl = this.$('#streamQualWebdl');
          const qualWebripEl = this.$('#streamQualWebrip');
          const qualHdtvEl = this.$('#streamQualHdtv');
          const qualCamEl = this.$('#streamQualCam');
          const excludeCamEl = this.$('#streamExcludeCam');
          const excludeUnplayableEl = this.$('#streamExcludeUnplayable');

          if (qualRemuxEl) qualRemuxEl.checked = qualConfig['remux'] !== false;
          if (qualBlurayEl) qualBlurayEl.checked = qualConfig['bluray'] !== false;
          if (qualWebdlEl) qualWebdlEl.checked = qualConfig['webdl'] !== false;
          if (qualWebripEl) qualWebripEl.checked = qualConfig['webrip'] !== false;
          if (qualHdtvEl) qualHdtvEl.checked = qualConfig['hdtv'] !== false;
          if (qualCamEl) qualCamEl.checked = qualConfig['cam'] !== false;
          if (excludeCamEl) excludeCamEl.checked = !!streamConfig.exclude_cam;
          if (excludeUnplayableEl) excludeUnplayableEl.checked = streamConfig.exclude_unplayable !== undefined ? !!streamConfig.exclude_unplayable : true;

          const encHevcEl = this.$('#streamEncHevc');
          const encAvcEl = this.$('#streamEncAvc');
          const encAv1El = this.$('#streamEncAv1');
          if (encHevcEl) encHevcEl.checked = encConfig['hevc'] !== false;
          if (encAvcEl) encAvcEl.checked = encConfig['avc'] !== false;
          if (encAv1El) encAv1El.checked = encConfig['av1'] !== false;

          const visHdrEl = this.$('#streamVisHdr');
          const visDvEl = this.$('#streamVisDv');
          if (visHdrEl) visHdrEl.checked = visConfig['hdr'] !== false;
          if (visDvEl) visDvEl.checked = visConfig['dv'] !== false;

          const preferredResEl = this.$('#streamPreferredRes');
          const maxPerResEl = this.$('#streamMaxPerRes');
          const maxTotalEl = this.$('#streamMaxTotal');
          const minSizeMbEl = this.$('#streamMinSizeMb');
          const maxSizeGbEl = this.$('#streamMaxSizeGb');
          const excludedKwEl = this.$('#streamExcludedKeywords');
          const requiredKwEl = this.$('#streamRequiredKeywords');

          if (preferredResEl) preferredResEl.value = streamConfig.preferred_resolution || 'auto';
          if (maxPerResEl) maxPerResEl.value = String(streamConfig.max_streams_per_resolution || 0);
          if (maxTotalEl) maxTotalEl.value = String(streamConfig.max_streams_total || 0);
          if (minSizeMbEl) minSizeMbEl.value = streamConfig.min_size_mb ? String(streamConfig.min_size_mb) : '';
          if (maxSizeGbEl) maxSizeGbEl.value = streamConfig.max_size_gb ? String(streamConfig.max_size_gb) : '';
          if (excludedKwEl) excludedKwEl.value = streamConfig.excluded_keywords || '';
          if (requiredKwEl) requiredKwEl.value = streamConfig.required_keywords || '';

          updateCatalogModeUI();
          renderCatalogOptions();
          renderUpstreamAddons();

        }
      } catch (err) {
        console.warn('Failed to load catalog settings:', err);
      }
    };

    const isCatalogFrozen = () => false;

    if (catalogModeEnabled) {
      catalogModeEnabled.addEventListener('change', () => {
        if (isCatalogFrozen() || catalogModeEnabled.disabled) return;
        catalogSettingsState.catalogs_enabled = true;
        updateCatalogModeUI();
      });
    }

    if (catalogModeDisabled) {
      catalogModeDisabled.addEventListener('change', () => {
        if (isCatalogFrozen() || catalogModeDisabled.disabled) return;
        catalogSettingsState.catalogs_enabled = false;
        updateCatalogModeUI();
      });
    }

    // Also support clicking anywhere on the mode option card
    const modeOpts = document.querySelectorAll('.addon-catalog-mode-opt');
    modeOpts.forEach((opt) => {
      opt.addEventListener('click', (e) => {
        if (isCatalogFrozen()) return;
        const radio = opt.querySelector('input[type="radio"]');
        if (radio && e.target !== radio && !radio.disabled) {
          radio.checked = true;
          catalogSettingsState.catalogs_enabled = (radio.value === 'enabled');
          updateCatalogModeUI();
        }
      });
    });

    if (catTypeMovies) {
      catTypeMovies.addEventListener('change', () => {
        if (!catalogSettingsState.enabled_types) catalogSettingsState.enabled_types = {};
        catalogSettingsState.enabled_types.movie = catTypeMovies.checked;
        renderCatalogOptions();
      });
    }

    if (catTypeSeries) {
      catTypeSeries.addEventListener('change', () => {
        if (!catalogSettingsState.enabled_types) catalogSettingsState.enabled_types = {};
        catalogSettingsState.enabled_types.series = catTypeSeries.checked;
        renderCatalogOptions();
      });
    }

    if (catTypeOther) {
      catTypeOther.addEventListener('change', () => {
        if (!catalogSettingsState.enabled_types) catalogSettingsState.enabled_types = {};
        catalogSettingsState.enabled_types.other = catTypeOther.checked;
        renderCatalogOptions();
      });
    }

    const bindSelectDeselect = (selectAllBtn, deselectAllBtn, container) => {
      if (selectAllBtn) {
        selectAllBtn.addEventListener('click', () => {
          const checkboxes = container ? container.querySelectorAll('input[type="checkbox"]') : [];
          checkboxes.forEach((c) => {
            c.checked = true;
            catalogSettingsState.enabled_catalogs[c.value] = true;
          });
        });
      }
      if (deselectAllBtn) {
        deselectAllBtn.addEventListener('click', () => {
          const checkboxes = container ? container.querySelectorAll('input[type="checkbox"]') : [];
          checkboxes.forEach((c) => {
            c.checked = false;
            catalogSettingsState.enabled_catalogs[c.value] = false;
          });
        });
      }
    };

    bindSelectDeselect(catSelectAllSpecialBtn, catDeselectAllSpecialBtn, addonCatalogSpecialList);
    bindSelectDeselect(catSelectAllBtn, catDeselectAllBtn, addonCatalogList);

    if (catSaveSettingsBtn) {
      catSaveSettingsBtn.addEventListener('click', async () => {
        catSaveSettingsBtn.disabled = true;
        const origText = catSaveSettingsBtn.textContent;
        catSaveSettingsBtn.textContent = 'Saving...';
        try {
          // Read state directly from radio and inputs
          const isEnabled = catalogModeDisabled && catalogModeDisabled.checked ? false : !!(catalogModeEnabled && catalogModeEnabled.checked);
          catalogSettingsState.catalogs_enabled = isEnabled;

          const res4kEl = this.$('#streamRes4k');
          const res1080pEl = this.$('#streamRes1080p');
          const res720pEl = this.$('#streamRes720p');
          const resSdEl = this.$('#streamResSd');
          const resUnknownEl = this.$('#streamResUnknown');
          const preferredResEl = this.$('#streamPreferredRes');
          const maxPerResEl = this.$('#streamMaxPerRes');

          const payload = {
            catalogs_enabled: isEnabled,
            country: addonCountrySelect ? addonCountrySelect.value : (catalogSettingsState.country || ''),
            enabled_types: {
              movie: !!(catTypeMovies && catTypeMovies.checked),
              series: !!(catTypeSeries && catTypeSeries.checked),
              other: !!(catTypeOther && catTypeOther.checked)
            },
            enabled_catalogs: catalogSettingsState.enabled_catalogs,
            upstream_enabled: catalogSettingsState.upstream_enabled !== false,
            // When upstreams are disabled, omit the list entirely so the
            // backend keeps the stored manifests (re-enabling restores them
            // without re-adding every URL).
            upstream_manifests:
              catalogSettingsState.upstream_enabled === false
                ? undefined
                : catalogSettingsState.upstream_manifests || [],
            stream_config: {
              resolutions: {
                '4k': res4kEl ? res4kEl.checked : true,
                '1080p': res1080pEl ? res1080pEl.checked : true,
                '720p': res720pEl ? res720pEl.checked : true,
                'sd': resSdEl ? resSdEl.checked : true,
                'unknown': resUnknownEl ? resUnknownEl.checked : true
              },
              qualities: {
                'remux': this.$('#streamQualRemux') ? this.$('#streamQualRemux').checked : true,
                'bluray': this.$('#streamQualBluray') ? this.$('#streamQualBluray').checked : true,
                'webdl': this.$('#streamQualWebdl') ? this.$('#streamQualWebdl').checked : true,
                'webrip': this.$('#streamQualWebrip') ? this.$('#streamQualWebrip').checked : true,
                'hdtv': this.$('#streamQualHdtv') ? this.$('#streamQualHdtv').checked : true,
                'cam': this.$('#streamQualCam') ? this.$('#streamQualCam').checked : true,
                'unknown': true
              },
              encodes: {
                'hevc': this.$('#streamEncHevc') ? this.$('#streamEncHevc').checked : true,
                'avc': this.$('#streamEncAvc') ? this.$('#streamEncAvc').checked : true,
                'av1': this.$('#streamEncAv1') ? this.$('#streamEncAv1').checked : true
              },
              visual_tags: {
                'hdr': this.$('#streamVisHdr') ? this.$('#streamVisHdr').checked : true,
                'dv': this.$('#streamVisDv') ? this.$('#streamVisDv').checked : true
              },
              exclude_cam: this.$('#streamExcludeCam') ? this.$('#streamExcludeCam').checked : false,
              exclude_unplayable: this.$('#streamExcludeUnplayable') ? this.$('#streamExcludeUnplayable').checked : false,
              preferred_resolution: preferredResEl ? preferredResEl.value : 'auto',
              max_streams_per_resolution: maxPerResEl ? parseInt(maxPerResEl.value, 10) || 0 : 0,
              max_streams_total: this.$('#streamMaxTotal') ? parseInt(this.$('#streamMaxTotal').value, 10) || 0 : 0,
              min_size_mb: this.$('#streamMinSizeMb') ? parseInt(this.$('#streamMinSizeMb').value, 10) || 0 : 0,
              max_size_gb: this.$('#streamMaxSizeGb') ? parseInt(this.$('#streamMaxSizeGb').value, 10) || 0 : 0,
              excluded_keywords: this.$('#streamExcludedKeywords') ? this.$('#streamExcludedKeywords').value.trim() : '',
              required_keywords: this.$('#streamRequiredKeywords') ? this.$('#streamRequiredKeywords').value.trim() : ''
            }
          };
          const res = await fetch(`${this.localApiBase}/api/catalog-settings`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
          });
          const resData = await res.json().catch(() => null);
          if (res.ok && resData?.ok) {
            catalogSettingsState = resData.settings || payload;
            updateCatalogModeUI();
            // Trigger real-time reload of streaming app UI using the updated manifest.json
            this.loadInitialData().catch((e) => console.warn('Failed to refresh data after catalog save:', e));

            if (catSaveStatus) {
              const hasUpstream = Array.isArray(catalogSettingsState.upstream_manifests) && catalogSettingsState.upstream_manifests.length > 0;
              if (isEnabled) {
                catSaveStatus.textContent = '✓ Saved! Local catalogs enabled.';
              } else if (hasUpstream) {
                catSaveStatus.textContent = '✓ Saved! Upstream catalogs only.';
              } else {
                catSaveStatus.textContent = '✓ Saved! Streams only.';
              }
              catSaveStatus.classList.remove('hidden', 'error');
              setTimeout(() => catSaveStatus.classList.add('hidden'), 3000);
            }
          } else {
            throw new Error(resData?.error || resData?.message || 'Failed to save');
          }
        } catch (err) {
          console.error('Save catalog settings error:', err);
          if (catSaveStatus) {
            catSaveStatus.textContent = `✕ ${err.message || 'Error saving settings'}`;
            catSaveStatus.classList.remove('hidden');
            catSaveStatus.classList.add('error');
            setTimeout(() => catSaveStatus.classList.add('hidden'), 3000);
          }
        } finally {
          catSaveSettingsBtn.disabled = false;
          catSaveSettingsBtn.textContent = origText;
        }
      });
    }

    const isUsableLanHost = (host) => {
      const value = String(host || '').trim();
      if (!value || value === 'localhost' || value === '::1') {
        return false;
      }
      const ipv4 = value.match(/^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$/);
      if (!ipv4) {
        return value !== '127.0.0.1';
      }
      const a = Number(ipv4[1]);
      const b = Number(ipv4[2]);
      if (a === 127 || a === 0 || (a === 169 && b === 254)) {
        return false;
      }
      // RFC1918 only — never show ISP/cellular public IPs on the Nuvio card.
      return a === 10 || (a === 172 && b >= 16 && b <= 31) || (a === 192 && b === 168);
    };

    const getAddonManifestUrls = () => {
      // Always HTTP for LAN/localhost so Stremio API sync can store a
      // non-HTTPS transportUrl. Tunnel pages have no :8088 in location.port.
      const listenPort = Number(this.listenPort) > 0 ? Number(this.listenPort) : 8088;
      const onTunnel = this._isCloudflareTunnelPage();
      const pagePort = Number(window.location.port);
      const httpPort = (!onTunnel && pagePort > 0) ? pagePort : listenPort;
      const portSuffix = httpPort && httpPort !== 80 ? `:${httpPort}` : '';
      // Clean path format: /<token>/manifest.json for remote players.
      // Localhost/LAN can also use it or the unauthenticated /manifest.json.
      const tokenPrefix = this._authToken ? `/${encodeURIComponent(this._authToken)}` : '';
      const pageHost = window.location.hostname;
      const isLocal = pageHost === 'localhost' || pageHost === '127.0.0.1' || pageHost === '::1';

      // 1. Localhost URL
      const localUrl = `http://127.0.0.1${portSuffix}${tokenPrefix}/manifest.json`;

      // 2. Server / Remote URL: for direct VPS / remote IP access
      let serverUrl = (!isLocal && !onTunnel)
        ? `${window.location.origin.replace(/\/+$/, '')}${tokenPrefix}/manifest.json`
        : '';

      const lanHost = isUsableLanHost(this.lanIp)
        ? this.lanIp
        : (isUsableLanHost(pageHost) ? pageHost : '');
      const lanUrl = lanHost ? `http://${lanHost}${portSuffix}${tokenPrefix}/manifest.json` : '';

      // 3. Tunnel URL: ONLY if Cloudflare tunnel is active/enabled or accessed via trycloudflare
      let tunnelUrl = '';
      if (this.tunnelEnabled || onTunnel) {
        if (this.tunnelPublicUrl) {
          tunnelUrl = this.tunnelPublicUrl;
        } else if (this.tunnelUrl) {
          tunnelUrl = String(this.tunnelUrl).replace(/\/+$/, '');
        } else if (onTunnel) {
          tunnelUrl = window.location.origin.replace(/\/+$/, '');
        }
      }
      const tunnelManifest = (tunnelUrl && (this.tunnelEnabled || onTunnel))
        ? `${tunnelUrl}${tokenPrefix}/manifest.json`
        : '';

      // If serverUrl matches tunnelManifest or current host matches tunnel host, suppress serverUrl
      let tunnelHost = '';
      if (tunnelUrl) {
        try { tunnelHost = new URL(tunnelUrl).hostname.toLowerCase(); } catch (e) {}
      }
      if (serverUrl && (onTunnel || (tunnelManifest && (serverUrl === tunnelManifest || pageHost.toLowerCase() === tunnelHost)))) {
        serverUrl = '';
      }

      // 4. Eclipse Music Addon URL:
      // When on Cloudflare Tunnel: use tunnel URL
      // When on remote server/VPS (public IP/domain): use current origin
      // When on localhost: use localhost (or tunnel if running)
      let eclipseBase = '';
      if (onTunnel && tunnelUrl) {
        eclipseBase = `${tunnelUrl}${tokenPrefix}`;
      } else if (!isLocal) {
        eclipseBase = `${window.location.origin.replace(/\/+$/, '')}${tokenPrefix}`;
      } else if (tunnelUrl) {
        eclipseBase = `${tunnelUrl}${tokenPrefix}`;
      } else if (lanUrl) {
        eclipseBase = lanUrl.replace(/\/manifest\.json.*$/i, '');
      } else {
        eclipseBase = localUrl.replace(/\/manifest\.json.*$/i, '');
      }
      const eclipseUrl = `${eclipseBase}/eclipse/manifest.json`;

      return { localUrl, serverUrl, lanUrl, tunnelManifest, eclipseUrl, lanHost, isLocal, onTunnel };
    };

    const getSelectedStremioSyncUrl = () => {
      const urls = getAddonManifestUrls();
      if (!urls.isLocal) {
        if (urls.onTunnel && urls.tunnelManifest) {
          return { url: urls.tunnelManifest, mode: 'tunnel', label: 'Cloudflare Tunnel' };
        }
        if (urls.serverUrl) {
          return { url: urls.serverUrl, mode: 'server', label: 'Server' };
        }
      }
      const wantLan = !!(stremioSyncModeLan && stremioSyncModeLan.checked);
      if (wantLan && urls.lanUrl) {
        return { url: urls.lanUrl, mode: 'lan', label: 'Wi-Fi / LAN' };
      }
      return { url: urls.localUrl, mode: 'localhost', label: 'Localhost' };
    };

    const updateStremioSyncPreview = () => {
      const urls = getAddonManifestUrls();
      const selected = getSelectedStremioSyncUrl();

      if (stremioSyncModeToggle) {
        // On VPS, remote server, or Cloudflare Tunnel: hide Wi-Fi/LAN/Localhost toggle
        stremioSyncModeToggle.style.display = (!urls.isLocal || urls.onTunnel) ? 'none' : 'flex';
      }

      if (stremioSyncUrlPreview) {
        stremioSyncUrlPreview.textContent = selected.url
          ? `${selected.label}: ${selected.url}`
          : '';
      }

      if (stremioSyncModeLan && urls.isLocal && !urls.onTunnel) {
        const lanMissing = !urls.lanUrl;
        stremioSyncModeLan.disabled = lanMissing;
        if (lanMissing && stremioSyncModeLocal) {
          stremioSyncModeLocal.checked = true;
        }
      }
    };

    const updateAddonModalUrls = () => {
      const { localUrl, serverUrl, lanUrl, tunnelManifest, eclipseUrl, isLocal, onTunnel } = getAddonManifestUrls();

      const addonLocalFieldLabel = this.$('#addonLocalFieldLabel');
      const addonLocalFieldBadge = this.$('#addonLocalFieldBadge');

      if (manifestInput) {
        if (!isLocal && serverUrl) {
          manifestInput.value = serverUrl;
          manifestInput.style.color = '#ff8a5b';
          if (addonLocalFieldLabel) addonLocalFieldLabel.textContent = '🌐 Server Manifest';
          if (addonLocalFieldBadge) {
            addonLocalFieldBadge.textContent = 'Public';
            addonLocalFieldBadge.style.color = '#ff8a5b';
            addonLocalFieldBadge.style.background = 'rgba(255, 107, 53, 0.18)';
          }
        } else {
          manifestInput.value = localUrl;
          manifestInput.style.color = '#bbb';
          if (addonLocalFieldLabel) addonLocalFieldLabel.textContent = '💻 Localhost Manifest (This Device Only)';
          if (addonLocalFieldBadge) {
            addonLocalFieldBadge.textContent = 'Local';
            addonLocalFieldBadge.style.color = '#aaa';
            addonLocalFieldBadge.style.background = 'rgba(255, 255, 255, 0.08)';
          }
        }
      }

      if (manifestLanInput) {
        manifestLanInput.value = lanUrl || (this.lanIp ? `http://${this.lanIp}${portSuffix}${tokenPrefix}/manifest.json` : '');
      }

      if (manifestTunnelInput) {
        manifestTunnelInput.value = tunnelManifest;
      }

      if (manifestEclipseInput) {
        manifestEclipseInput.value = eclipseUrl;
      }

      if (addonStremioDirectBtn) {
        const directUrl = tunnelManifest || serverUrl || (isLocal ? localUrl : '');
        if (directUrl) {
          const stremioDeepLink = directUrl.replace(/^https?:\/\//i, 'stremio://');
          addonStremioDirectBtn.href = stremioDeepLink;
          addonStremioDirectBtn.classList.remove('hidden');
        } else {
          addonStremioDirectBtn.href = '#';
          addonStremioDirectBtn.classList.add('hidden');
        }
      }

      const isHttps = window.location.protocol === 'https:' || onTunnel;

      if (addonStremioLocalDirectBtn) {
        const directUrl = serverUrl || localUrl;
        if (directUrl && isHttps) {
          const stremioDeepLink = directUrl.replace(/^https?:\/\//i, 'stremio://');
          addonStremioLocalDirectBtn.href = stremioDeepLink;
          addonStremioLocalDirectBtn.classList.remove('hidden');
        } else {
          addonStremioLocalDirectBtn.href = '#';
          addonStremioLocalDirectBtn.classList.add('hidden');
        }
      }

      if (addonModalTitle) addonModalTitle.textContent = '🧩 Nuvio / Stremio Addon';
      if (addonModalDesc) {
        addonModalDesc.textContent = isHttps
          ? 'Copy a manifest URL for Nuvio, or click Add to Stremio.'
          : 'Copy a manifest URL for Nuvio, or install an address into Stremio via API sync.';
      }
      if (addonLanField) {
        // Wi-Fi / LAN Manifest is ONLY for non-VPS local PC devices (localhost / 127.0.0.1)
        const showLan = isLocal && !onTunnel && Boolean(lanUrl || this.lanIp);
        addonLanField.classList.toggle('hidden', !showLan);
      }
      if (addonLocalField) {
        // Show server/local field unless specifically browsing on an active tunnel domain or when duplicate
        const hideLocal = onTunnel || (!isLocal && (!serverUrl || serverUrl === tunnelManifest));
        addonLocalField.classList.toggle('hidden', hideLocal);
      }
      if (addonTunnelField) {
        // Only show Cloudflare Tunnel field if a real tunnel is active with a valid manifest
        addonTunnelField.classList.toggle('hidden', !tunnelManifest);
      }
      if (addonStremioSync) {
        // If already HTTPS, no need to show HTTP API Sync
        addonStremioSync.classList.toggle('hidden', isHttps);
      }
      if (addonNuvioInstructions) {
        addonNuvioInstructions.classList.remove('hidden');
      }

      if (!isHttps) {
        updateStremioSyncPreview();
      }
    };

    this._updateAddonModalUrls = updateAddonModalUrls;
    updateAddonModalUrls();
    refreshTokenField();

    const openAddonModal = () => {
      if (addonModal) {
        // 1. Instant reveal modal and cached/default inputs immediately
        updateAddonModalUrls();
        refreshTokenField();
        addonModal.classList.remove('hidden');
        addonModal.setAttribute('aria-hidden', 'false');
        if (copiedStatus) copiedStatus.classList.add('hidden');

        // 2. Fetch fresh network, tunnel, and catalog states asynchronously in background
        loadCatalogSettings();
        this.loadLanIp().finally(() => {
          updateAddonModalUrls();
        });
        this.loadTunnelStatus().finally(() => {
          updateAddonModalUrls();
        });
      }
    };
    this.openAddonModal = openAddonModal;

    if (addonBtn) {
      addonBtn.addEventListener('click', openAddonModal);
    }

    const settingsOpenAddonBtn = this.$('#settingsOpenAddonBtn');
    if (settingsOpenAddonBtn) {
      settingsOpenAddonBtn.addEventListener('click', () => {
        this.closeSettingsGate();
        openAddonModal();
      });
    }

    const closeAddonModal = () => {
      if (addonModal) {
        addonModal.classList.add('hidden');
        addonModal.setAttribute('aria-hidden', 'true');
        if (window.location.hash === '#configure' || window.location.hash === '#addon') {
          history.replaceState(null, '', window.location.pathname);
        }
      }
    };
    this.closeAddonModal = closeAddonModal;

    if (addonClose) {
      addonClose.addEventListener('click', closeAddonModal);
    }

    const showCopiedFeedback = (msg = '✓ Copied to clipboard!') => {
      if (copiedStatus) {
        copiedStatus.textContent = msg;
        copiedStatus.classList.remove('hidden');
        setTimeout(() => copiedStatus.classList.add('hidden'), 3000);
      }
    };

    const flashCopyBtn = (btn, defaultText = '📋 Copy') => {
      if (!btn) return;
      btn.textContent = '✓ Copied!';
      btn.style.background = '#00d26a';
      btn.style.color = '#fff';
      setTimeout(() => {
        btn.textContent = defaultText;
        btn.style.background = '';
        btn.style.color = '';
      }, 1500);
    };

    if (copyAddonLanBtn && manifestLanInput) {
      copyAddonLanBtn.addEventListener('click', async () => {
        manifestLanInput.select();
        await this.copyToClipboard(manifestLanInput.value, manifestLanInput);
        flashCopyBtn(copyAddonLanBtn);
        showCopiedFeedback('✓ Copied Wi-Fi / LAN URL to clipboard!');
      });
    }

    if (copyAddonBtn && manifestInput) {
      copyAddonBtn.addEventListener('click', async () => {
        manifestInput.select();
        await this.copyToClipboard(manifestInput.value, manifestInput);
        flashCopyBtn(copyAddonBtn);
        showCopiedFeedback('✓ Copied Manifest URL to clipboard!');
      });
    }

    if (copyAddonTunnelBtn && manifestTunnelInput) {
      copyAddonTunnelBtn.addEventListener('click', async () => {
        manifestTunnelInput.select();
        await this.copyToClipboard(manifestTunnelInput.value, manifestTunnelInput);
        flashCopyBtn(copyAddonTunnelBtn);
        showCopiedFeedback('✓ Copied Cloudflare tunnel URL to clipboard!');
      });
    }

    if (copyAddonEclipseBtn && manifestEclipseInput) {
      copyAddonEclipseBtn.addEventListener('click', async () => {
        manifestEclipseInput.select();
        await this.copyToClipboard(manifestEclipseInput.value, manifestEclipseInput);
        flashCopyBtn(copyAddonEclipseBtn);
        showCopiedFeedback('✓ Copied Eclipse Music Addon URL to clipboard!');
      });
    }

    const setStremioSyncStatus = (message, kind = '') => {
      if (!stremioSyncStatus) return;
      stremioSyncStatus.textContent = message || '';
      stremioSyncStatus.classList.toggle('hidden', !message);
      stremioSyncStatus.classList.toggle('error', kind === 'error');
      stremioSyncStatus.classList.toggle('ok', kind === 'ok');
    };

    const stremioApiPost = async (path, body) => {
      const response = await fetch(`https://api.strem.io/api/${path}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(body),
      });
      let data = {};
      try {
        data = await response.json();
      } catch (e) {
        throw new Error(`Stremio API ${path} returned invalid JSON.`);
      }
      if (data.error) {
        const err = data.error;
        throw new Error(typeof err === 'string' ? err : (err.message || `Stremio API ${path} failed.`));
      }
      if (!response.ok) {
        throw new Error(`Stremio API ${path} failed (${response.status}).`);
      }
      return data;
    };

    const installStremioAddonViaApi = async () => {
      const selected = getSelectedStremioSyncUrl();
      const transportUrl = String(selected.url || '').trim();
      if (!transportUrl) {
        throw new Error('No HTTP manifest URL selected.');
      }

      const authKeyInput = this.$('#stremioSyncAuthKey');
      const emailInput = this.$('#stremioSyncEmail');
      const passwordInput = this.$('#stremioSyncPassword');
      let authKey = String(authKeyInput?.value || '').trim();

      if (!authKey) {
        const email = String(emailInput?.value || '').trim();
        const password = String(passwordInput?.value || '');
        if (!email || !password) {
          throw new Error('Enter Stremio email and password, or paste an auth key.');
        }
        const loginRes = await stremioApiPost('login', {
          email,
          password,
          type: 'Login',
        });
        authKey = String(loginRes.result?.authKey || '').trim();
        if (!authKey) {
          throw new Error('Login failed. Check your Stremio email and password.');
        }
      }

      const collectionRes = await stremioApiPost('addonCollectionGet', {
        type: 'AddonCollectionGet',
        authKey,
        update: true,
      });
      const existingAddons = Array.isArray(collectionRes.result?.addons)
        ? collectionRes.result.addons
        : [];

      // HTTPS dashboard cannot fetch http:// LAN URLs (mixed content).
      // Pull the labeled JSON from this origin, then store the HTTP transportUrl.
      const manifestFetchUrl = window.location.protocol === 'https:'
        ? `${window.location.origin}/manifest.json?mode=${encodeURIComponent(selected.mode)}`
        : transportUrl;
      const manifestRes = await fetch(manifestFetchUrl);
      if (!manifestRes.ok) {
        throw new Error(`Failed to fetch ${selected.label} manifest.`);
      }
      const manifest = await manifestRes.json();
      if (!manifest || typeof manifest !== 'object' || !manifest.id) {
        throw new Error('Invalid addon manifest.');
      }

      const newAddon = {
        transportUrl,
        transportName: '',
        flags: { official: false, protected: false },
        manifest,
      };

      const addonId = String(manifest.id || '');
      const updatedAddons = [];
      let replaced = false;
      existingAddons.forEach((addon) => {
        const sameUrl = addon && addon.transportUrl === transportUrl;
        const sameId = addon && addon.manifest && String(addon.manifest.id || '') === addonId;
        if (sameUrl || sameId) {
          if (!replaced) {
            updatedAddons.push(newAddon);
            replaced = true;
          }
          return;
        }
        updatedAddons.push(addon);
      });
      if (!replaced) {
        updatedAddons.push(newAddon);
      }

      await stremioApiPost('addonCollectionSet', {
        type: 'AddonCollectionSet',
        authKey,
        addons: updatedAddons,
      });

      return `${manifest.name || selected.label} installed. Restart Stremio if it is already open.`;
    };

    [stremioSyncModeLan, stremioSyncModeLocal].forEach((radio) => {
      if (!radio) return;
      radio.addEventListener('change', () => {
        updateStremioSyncPreview();
        setStremioSyncStatus('');
      });
    });

    if (stremioSyncInstallBtn) {
      stremioSyncInstallBtn.addEventListener('click', async () => {
        stremioSyncInstallBtn.disabled = true;
        const original = stremioSyncInstallBtn.textContent;
        stremioSyncInstallBtn.textContent = 'Installing...';
        setStremioSyncStatus('Logging in and syncing addon collection...');
        try {
          const message = await installStremioAddonViaApi();
          setStremioSyncStatus(message, 'ok');
          const passwordInput = this.$('#stremioSyncPassword');
          const authKeyInput = this.$('#stremioSyncAuthKey');
          if (passwordInput) passwordInput.value = '';
          if (authKeyInput) authKeyInput.value = '';
        } catch (err) {
          setStremioSyncStatus(err.message || 'Failed to install addon.', 'error');
        } finally {
          stremioSyncInstallBtn.disabled = false;
          stremioSyncInstallBtn.textContent = original;
        }
      });
    }

    this.bindTunnelControls();

    // Allow Enter key on token input
    this.$('#botTokenInput').addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        this.$('#connectBtn').click();
      }
    });

    // ── Nav ──
    this.$('#streamSearchBtn').addEventListener('click', () => this.openSearch());
    this.$('#streamHamburger').addEventListener('click', () => this.openMobileNav());

    // ── Search overlay ──
    this.$('#searchOverlayBack').addEventListener('click', () => this.closeSearch());
    this.$('#searchInput').addEventListener('input', (e) => {
      clearTimeout(this.searchTimeout);
      const query = e.target.value.trim();
      if (query.length < 2) {
        this.$('#searchResults').classList.add('hidden');
        this.$('#searchEmpty').classList.add('hidden');
        this.$('#searchSuggestions').classList.remove('hidden');
        return;
      }
      this.searchTimeout = setTimeout(() => this.doSearch(query), 400);
    });

    // ── Modal ──
    this.$('#modalBackdrop').addEventListener('click', () => this.closeModal());
    this.$('#modalClose').addEventListener('click', () => this.closeModal());

    // ── Hero CTA (scroll fallback for non-JS navigation) ──
    const heroCta = this.$('#heroCta');
    if (heroCta) {
      heroCta.addEventListener('click', () => {
        document.querySelector('.stream-content')?.scrollIntoView({ behavior: 'smooth' });
      });
    }

    // ── File Detail ──
    this.$('#fileDetailBack').addEventListener('click', () => this.closeFileDetail());
    this.$('#fileDetailBackdrop').addEventListener('click', () => this.closeFileDetail());
    this.$('#fileDetailStreamBtn').addEventListener('click', () => {
      const url = this.$('#fileDetailStreamBtn').getAttribute('data-url');
      if (url) window.open(url, '_blank');
    });
    this.$('#fileDetailDownloadBtn').addEventListener('click', () => {
      const url = this.$('#fileDetailDownloadBtn').getAttribute('data-url');
      if (url) window.location.href = url;
    });

    const handleMediaPlaybackError = (mediaEl) => {
      if (!mediaEl) return;
      if (mediaEl.dataset.pmIgnoreError === '1') {
        delete mediaEl.dataset.pmIgnoreError;
        return;
      }

      // src='' + load() resolves to the current #file/ page URL. That is
      // not a stream failure — it happens while "Resolving file info..."
      // is still showing.
      const attrSrc = String(mediaEl.getAttribute('src') || '').trim();
      const currentSrc = String(mediaEl.currentSrc || mediaEl.src || '').trim();
      const isDownloadSrc = attrSrc.includes('/api/download') || currentSrc.includes('/api/download');
      if (!attrSrc || !isDownloadSrc) return;

      const resolvingEl = this.$('#fileDetailResolving');
      if (resolvingEl && !resolvingEl.classList.contains('hidden')) return;

      console.warn('[Player] Media playback failed for source:', currentSrc || attrSrc);
      const titleEl = this.$('#fileDetailTitle');
      const tagsEl = this.$('#fileDetailTags');
      if (titleEl) titleEl.textContent = 'Stream playback failed';
      if (tagsEl) {
        tagsEl.innerHTML = `
          <div style="background:rgba(255,107,53,0.15);border:1px solid var(--accent);border-radius:8px;padding:10px 14px;margin-top:8px;">
            <p style="color:var(--accent);font-weight:600;margin:0 0 4px 0;"><i class="fas fa-exclamation-triangle"></i> Cannot play media stream</p>
            <p style="color:var(--text-secondary);font-size:0.85rem;margin:0 0 8px 0;">Make sure your Telegram bot is connected and the MadelineProto session is active.</p>
            <button id="fileDetailReconnectBtn" class="stream-btn stream-btn--primary stream-btn--sm" style="background:var(--accent);color:#fff;border:none;padding:5px 12px;border-radius:4px;cursor:pointer;font-size:0.8rem;">
              <i class="fas fa-key"></i> Connect Bot
            </button>
          </div>
        `;
        const reconnectBtn = this.$('#fileDetailReconnectBtn');
        if (reconnectBtn) {
          reconnectBtn.addEventListener('click', () => {
            this.showSettingsGate({ forceToken: !this.botId });
          });
        }
      }
    };

    const vEl = this.$('#fileDetailVideo');
    if (vEl) {
      vEl.addEventListener('error', () => handleMediaPlaybackError(vEl));
    }
    const aEl = this.$('#fileDetailAudio');
    if (aEl) {
      aEl.addEventListener('error', () => handleMediaPlaybackError(aEl));
    }

    // ── Stremio-Style Web Player Controls Binding ──
    this._bindPlayerToolbar();

    // ── Category Page ──
    this.$('#categoryPageBack').addEventListener('click', () => this.closeCategoryPage());

    // ── Mobile nav ──
    this.$('#mobileNavClose').addEventListener('click', () => this.closeMobileNav());
    this.$('#mobileNavOverlay').addEventListener('click', () => this.closeMobileNav());

    // ── Hash routing ──
    window.addEventListener('hashchange', () => this._handleHashChange());

    // ── Nav scroll effect ──
    window.addEventListener('scroll', () => {
      const nav = this.$('#streamNav');
      if (nav) {
        nav.classList.toggle('scrolled', window.scrollY > 60);
      }
    });

    // ── Keyboard Shortcuts (Stremio-style) ──
    document.addEventListener('keydown', (e) => {
      // Allow Esc to close overlays
      if (e.key === 'Escape') {
        // First close open player dropdowns if any
        if (this._closePlayerDropdowns && this._closePlayerDropdowns()) return;
        if (this.isModalOpen) this.closeModal();
        else if (this.isSearchOpen) this.closeSearch();
        else if (this.isFileDetailOpen()) this.closeFileDetail();
        else if (this._isCategoryPageOpen) this.closeCategoryPage();
        else if (this.isMobileNavOpen) this.closeMobileNav();
        else if (
          !this.$('#settingsGate').classList.contains('hidden') &&
          this.hasSession
        ) {
          this.closeSettingsGate();
        }
        return;
      }

      // If typing in input/textarea, do not intercept player hotkeys
      const activeTag = document.activeElement ? document.activeElement.tagName.toLowerCase() : '';
      if (activeTag === 'input' || activeTag === 'textarea' || activeTag === 'select') return;

      // Only handle player shortcuts when file detail page with video is open
      const videoEl = this.$('#fileDetailVideo');
      if (this.isFileDetailOpen() && videoEl && !videoEl.classList.contains('hidden') && videoEl.src) {
        if (e.key === ' ' || e.key.toLowerCase() === 'k') {
          e.preventDefault();
          if (videoEl.paused) videoEl.play();
          else videoEl.pause();
        } else if (e.key === 'ArrowRight' || e.key.toLowerCase() === 'l') {
          e.preventDefault();
          videoEl.currentTime = Math.min(videoEl.duration || Infinity, videoEl.currentTime + 10);
        } else if (e.key === 'ArrowLeft' || e.key.toLowerCase() === 'j') {
          e.preventDefault();
          videoEl.currentTime = Math.max(0, videoEl.currentTime - 10);
        } else if (e.key === 'ArrowUp') {
          e.preventDefault();
          videoEl.volume = Math.min(1, videoEl.volume + 0.05);
        } else if (e.key === 'ArrowDown') {
          e.preventDefault();
          videoEl.volume = Math.max(0, videoEl.volume - 0.05);
        } else if (e.key.toLowerCase() === 'f') {
          e.preventDefault();
          if (!document.fullscreenElement) {
            if (videoEl.requestFullscreen) videoEl.requestFullscreen();
            else if (videoEl.webkitRequestFullscreen) videoEl.webkitRequestFullscreen();
          } else {
            if (document.exitFullscreen) document.exitFullscreen();
          }
        } else if (e.key.toLowerCase() === 'm') {
          e.preventDefault();
          videoEl.muted = !videoEl.muted;
        } else if (e.key.toLowerCase() === 'g') {
          e.preventDefault();
          this._adjustSubtitleDelay(-0.25);
        } else if (e.key.toLowerCase() === 'h') {
          e.preventDefault();
          this._adjustSubtitleDelay(0.25);
        }
      }
    });
  }

  // ══════════════════════════════════════════════════════════════
  //  SESSION MANAGEMENT
  // ══════════════════════════════════════════════════════════════

  async loadLanIp() {
    try {
      const data = await this.requestJson(`${this.localApiBase}/api/lan-ip`);
      const lanIp = String(data?.lan_ip || '').trim();
      const port = Number(data?.port);
      if (port > 0 && port < 65536) {
        this.listenPort = port;
      }
      if (lanIp && lanIp !== '127.0.0.1') {
        this.lanIp = lanIp;
      }
      this._updateAddonModalUrls();
    } catch (e) {
      // Non-fatal — never tied to bot session.
    }
  }

  _isIpHost(host) {
    const h = String(host || '').trim();
    return /^(\d{1,3}\.){3}\d{1,3}$/.test(h) || h.includes(':') || h === 'localhost';
  }

  _isCloudflareTunnelPage() {
    const host = String(window.location.hostname || '').toLowerCase();
    if (!host || host === 'localhost' || host === '127.0.0.1' || host === '::1' || this._isLanHost(host) || this._isIpHost(host)) {
      return false;
    }
    if (host.endsWith('.trycloudflare.com')) {
      return true;
    }
    // Check against tunnel URL / public URL hostname
    for (const u of [this.tunnelPublicUrl, this.tunnelUrl]) {
      if (u) {
        try {
          if (new URL(u).hostname.toLowerCase() === host) return true;
        } catch (e) {}
      }
    }
    // Check custom domain or custom domains
    if (this.tunnelCustomDomain && host === String(this.tunnelCustomDomain).toLowerCase()) {
      return true;
    }
    if (Array.isArray(this.tunnelCustomDomains) && this.tunnelCustomDomains.some(d => String(d).toLowerCase() === host)) {
      return true;
    }
    return false;
  }

  _isLanHost(host) {
    const h = String(host || '').trim();
    if (/^10\./.test(h) || /^192\.168\./.test(h)) return true;
    const m = h.match(/^172\.(\d+)\./);
    if (m) {
      const o = parseInt(m[1], 10);
      if (o >= 16 && o <= 31) return true;
    }
    return false;
  }

  bindTunnelControls() {
    const enableBtn = this.$('#enableTunnelBtn');
    const disableBtn = this.$('#disableTunnelBtn');
    const copyBtn = this.$('#copyTunnelUrlBtn');
    const input = this.$('#tunnelUrlInput');

    if (enableBtn) {
      enableBtn.addEventListener('click', () => {
        this.enableTunnel().catch((err) => {
          this.renderTunnelStatus({
            enabled: false,
            message: err.message || 'Failed to enable tunnel.',
            error: true,
          });
        });
      });
    }

    if (disableBtn) {
      disableBtn.addEventListener('click', () => {
        this.disableTunnel().catch((err) => {
          this.renderTunnelStatus({
            enabled: this.tunnelEnabled,
            tunnel_url: this.tunnelUrl,
            message: err.message || 'Failed to disable tunnel.',
            error: true,
          });
        });
      });
    }

    if (copyBtn && input) {
      copyBtn.addEventListener('click', async () => {
        const value = String(input.value || '').trim();
        if (!value) return;
        await this.copyToClipboard(value, input);
        flashCopyBtn(copyBtn);
      });
    }

    // Change dashboard password handler
    const changePwBtn = this.$('#changePwBtn');
    const changePwCurrent = this.$('#changePwCurrent');
    const changePwNew = this.$('#changePwNew');
    const changePwStatus = this.$('#changePwStatus');

    if (changePwBtn) {
      changePwBtn.addEventListener('click', async () => {
        const current = changePwCurrent ? changePwCurrent.value : '';
        const next = changePwNew ? changePwNew.value : '';
        if (!current) {
          if (changePwStatus) { changePwStatus.className = 'settings-gate__status error'; changePwStatus.textContent = 'Please enter current password.'; }
          return;
        }
        if (!next || next.length < 4) {
          if (changePwStatus) { changePwStatus.className = 'settings-gate__status error'; changePwStatus.textContent = 'New password must be at least 4 characters.'; }
          return;
        }
        changePwBtn.disabled = true;
        if (changePwStatus) { changePwStatus.className = 'settings-gate__status'; changePwStatus.textContent = 'Updating...'; }

        try {
          const res = await this.requestJson(`${this.localApiBase}/api/auth/password`, {
            method: 'POST',
            body: JSON.stringify({ current, next })
          });
          if (res?.ok) {
            if (res.token) {
              this._authToken = res.token;
              localStorage.setItem('pm.auth', res.token);
            }
            if (changePwStatus) { changePwStatus.className = 'settings-gate__status ok'; changePwStatus.textContent = '✓ Password updated successfully!'; }
            if (changePwCurrent) changePwCurrent.value = '';
            if (changePwNew) changePwNew.value = '';
          } else {
            if (changePwStatus) { changePwStatus.className = 'settings-gate__status error'; changePwStatus.textContent = res?.message || 'Failed to update password.'; }
          }
        } catch (err) {
          if (changePwStatus) { changePwStatus.className = 'settings-gate__status error'; changePwStatus.textContent = err.message || 'Request failed.'; }
        }
        changePwBtn.disabled = false;
      });
    }

    this.renderTunnelStatus({
      enabled: this.tunnelEnabled,
      tunnel_url: this.tunnelUrl,
      message: this.tunnelEnabled ? 'Cloudflare tunnel is running.' : 'Tunnel is off.',
    });
  }

  renderTunnelStatus(data = {}) {
    const statusEl = this.$('#tunnelStatusText');
    const urlRow = this.$('#tunnelUrlRow');
    const input = this.$('#tunnelUrlInput');
    const enableBtn = this.$('#enableTunnelBtn');
    const disableBtn = this.$('#disableTunnelBtn');
    const localActions = this.$('#tunnelLocalActions');
    const remoteNote = this.$('#tunnelRemoteNote');
    const addBotsBtn = this.$('#toggleAddBotsBtn');
    const addBotsSection = this.$('#addBotsSection');
    const disconnectBtn = this.$('#settingsDisconnectBtn');
    const tokenInput = this.$('#tunnelTokenInput');
    const enabled = Boolean(data.enabled);
    const publicUrl = String(data.public_url || '').replace(/\/+$/, '');
    const quickUrl = String(data.tunnel_url || '').replace(/\/+$/, '');
    const displayUrl = publicUrl || quickUrl;
    const busy = Boolean(this._tunnelBusy);
    const error = Boolean(data.error);

    this.tunnelEnabled = enabled;
    this.tunnelUrl = displayUrl;
    this.tunnelPublicUrl = publicUrl;
    this.tunnelRunning = Boolean(data.running);
    this.tunnelCustomDomain = data.custom_domain || '';
    this.tunnelCustomDomains = Array.isArray(data.custom_domains) ? data.custom_domains : [];
    try {
      localStorage.setItem('pm.tunnel_enabled', enabled ? '1' : '0');
      if (displayUrl) localStorage.setItem('pm.tunnel_url', displayUrl);
      if (publicUrl) localStorage.setItem('pm.tunnel_public_url', publicUrl);
      if (data.custom_domain) localStorage.setItem('pm.tunnel_custom_domain', data.custom_domain);
      if (Array.isArray(data.custom_domains) && data.custom_domains.length > 0) {
        localStorage.setItem('pm.tunnel_custom_domains', JSON.stringify(data.custom_domains));
      }
    } catch (e) {}
    if (tokenInput && data.tunnel_token && !tokenInput.value) {
      tokenInput.value = data.tunnel_token;
    }
    this._updateAddonModalUrls();

    if (statusEl) {
      statusEl.classList.remove('is-on', 'is-busy', 'is-error');
      if (error) {
        statusEl.classList.add('is-error');
        statusEl.textContent = data.message || 'Tunnel error.';
      } else if (busy) {
        statusEl.classList.add('is-busy');
        statusEl.textContent = data.message || 'Working...';
      } else if (enabled && displayUrl) {
        statusEl.classList.add('is-on');
        statusEl.textContent = data.message || `Live: ${displayUrl}`;
      } else {
        statusEl.textContent = data.message || 'Tunnel is off.';
      }
    }

    const descPlaceholder = this.$('#tunnelSubdomainPlaceholder');
    if (descPlaceholder) {
      const devId = data.device_id || this.deviceId || '';
      descPlaceholder.textContent = publicUrl || (devId ? `https://${devId}-tunnel.pencarimovie.com` : 'https://{deviceId}-tunnel.pencarimovie.com');
    }

    if (input) input.value = displayUrl;
    if (urlRow) urlRow.classList.toggle('hidden', !displayUrl);
    if (remoteNote) remoteNote.classList.add('hidden');
    if (localActions) localActions.classList.remove('hidden');
    if (addBotsBtn) addBotsBtn.classList.remove('hidden');
    if (disconnectBtn) disconnectBtn.classList.remove('hidden');

    if (enableBtn) {
      enableBtn.disabled = busy || enabled;
      enableBtn.textContent = busy && !enabled ? 'Starting...' : 'Enable Tunnel';
      enableBtn.classList.toggle('hidden', enabled);
    }
    if (disableBtn) {
      disableBtn.disabled = busy || !enabled;
      disableBtn.textContent = busy && enabled ? 'Stopping...' : 'Disable';
      disableBtn.classList.toggle('hidden', !enabled);
    }
  }

  async loadTunnelStatus() {
    try {
      const data = await this.requestJson(`${this.localApiBase}/api/tunnel/status`);
      this.renderTunnelStatus(data || {});
      return data;
    } catch (e) {
      if (!this._tunnelBusy) {
        this.renderTunnelStatus({
          enabled: this.tunnelEnabled,
          tunnel_url: this.tunnelUrl,
          message: e.message || 'Could not read tunnel status.',
          error: true,
        });
      }
      return null;
    }
  }

  // Lightweight periodic watchdog: polls /api/tunnel/status so the backend's
  // auto-restart revives cloudflared if it died (e.g. Android/Termux killed it).
  // Only runs while a session exists and the page is open.
  _startTunnelWatchdog() {
    if (this._tunnelWatchdogStarted) return;
    this._tunnelWatchdogStarted = true;
    this._tunnelWatchdogTimer = setInterval(async () => {
      if (!this.hasSession) return;
      try {
        await this.requestJson(`${this.localApiBase}/api/tunnel/status`);
      } catch (_) {
        // Ignore transient errors; the next poll will retry.
      }
    }, 30000);
  }

  async enableTunnel() {
    if (this._tunnelBusy) return;
    this._tunnelBusy = true;
    this.renderTunnelStatus({
      enabled: false,
      message: 'Starting Cloudflare tunnel...',
    });

    let pollTimer = null;
    let pollFinished = false;

    const startPolling = () => {
      pollTimer = setInterval(async () => {
        if (pollFinished) return;
        try {
          const status = await this.requestJson(`${this.localApiBase}/api/tunnel/status`);
          if (pollFinished) return;
          if (status && status.enabled && status.tunnel_url) {
            pollFinished = true;
            if (pollTimer) clearInterval(pollTimer);
            this._tunnelBusy = false;
            this.renderTunnelStatus(status);
          }
        } catch (_) {}
      }, 1500);
    };

    startPolling();

    const tokenInput = this.$('#tunnelTokenInput');
    const tunnelToken = tokenInput ? tokenInput.value.trim() : '';

    try {
      const data = await this.requestJson(`${this.localApiBase}/api/tunnel/enable`, {
        method: 'POST',
        body: JSON.stringify({ tunnel_token: tunnelToken }),
      });
      pollFinished = true;
      if (pollTimer) clearInterval(pollTimer);
      this._tunnelBusy = false;
      this.renderTunnelStatus(data || {});
      return data;
    } catch (e) {
      // Check if status is actually running despite any request timeout/error
      try {
        const check = await this.requestJson(`${this.localApiBase}/api/tunnel/status`);
        if (check && check.enabled && check.tunnel_url) {
          pollFinished = true;
          if (pollTimer) clearInterval(pollTimer);
          this._tunnelBusy = false;
          this.renderTunnelStatus(check);
          return check;
        }
      } catch (_) {}

      pollFinished = true;
      if (pollTimer) clearInterval(pollTimer);
      this._tunnelBusy = false;
      this.renderTunnelStatus({
        enabled: false,
        message: e.message || 'Failed to enable tunnel.',
        error: true,
      });
      throw e;
    }
  }

  async disableTunnel() {
    if (this._tunnelBusy) return;
    this._tunnelBusy = true;
    this.renderTunnelStatus({
      enabled: this.tunnelEnabled,
      tunnel_url: this.tunnelUrl,
      message: 'Stopping Cloudflare tunnel...',
    });
    try {
      const data = await this.requestJson(`${this.localApiBase}/api/tunnel/disable`, {
        method: 'POST',
        body: '{}',
      });
      this._tunnelBusy = false;
      this.renderTunnelStatus(data || { enabled: false });
      return data;
    } catch (e) {
      this._tunnelBusy = false;
      this.renderTunnelStatus({
        enabled: this.tunnelEnabled,
        tunnel_url: this.tunnelUrl,
        message: e.message || 'Failed to disable tunnel.',
        error: true,
      });
      throw e;
    }
  }

  _sessionCacheKey() {
    return 'tgfd.session';
  }

  _restoreCachedSession() {
    try {
      const raw = localStorage.getItem(this._sessionCacheKey());
      if (!raw) {
        const legacyBotId = String(localStorage.getItem('tgfd.botId') || '').trim();
        if (legacyBotId) {
          this.botId = legacyBotId;
          this.hasSession = true;
        }
        return;
      }
      const cached = JSON.parse(raw);
      if (!cached || cached.hasSession !== true) {
        return;
      }
      this.botId = String(cached.botId || '').trim();
      this.botUsername = String(cached.botUsername || '');
      this.botName = String(cached.botName || '');
      this.apiSecret = String(cached.apiSecret || '');
      // A cached "logged in" flag without bot_id cannot resolve files on WordPress.
      this.hasSession = this.botId !== '';
    } catch (e) {
      // Ignore corrupt cache.
    }
  }

  _persistCachedSession() {
    try {
      if (!this.hasSession) {
        this._clearCachedSession();
        return;
      }
      localStorage.setItem(this._sessionCacheKey(), JSON.stringify({
        hasSession: true,
        botId: this.botId || '',
        botUsername: this.botUsername || '',
        botName: this.botName || '',
        apiSecret: this.apiSecret || '',
      }));
      if (this.botId) {
        localStorage.setItem('tgfd.botId', this.botId);
      }
    } catch (e) {
      // Private mode / quota — session files on disk still win.
    }
  }

  _clearCachedSession() {
    this.botId = '';
    this.botUsername = '';
    this.botName = '';
    this.apiSecret = '';
    this.hasSession = false;
    try {
      localStorage.removeItem(this._sessionCacheKey());
      localStorage.removeItem('tgfd.botId');
    } catch (e) {
      // Ignore storage errors.
    }
  }

  async loadBotPool() {
    try {
      const resp = await this.requestJson(`${this.localApiBase}/api/bots`);
      if (!resp || !resp.ok) return;

      const countEl = this.$('#botPoolCount');
      if (countEl) countEl.textContent = String(resp.total_bots || 1);

      const listEl = this.$('#botPoolList');
      if (!listEl) return;

      if (!resp.bots || resp.bots.length === 0) {
        listEl.innerHTML = '<div style="font-size:0.75rem;color:#888;padding:4px 0;">No extra bots connected yet.</div>';
        return;
      }

      listEl.innerHTML = resp.bots.map(b => {
        const isAct = b.is_active;
        const bId = this.escapeHtml(String(b.bot_id || ''));
        const bUser = this.escapeHtml(String(b.bot_username || ''));
        const bName = this.escapeHtml(String(b.bot_name || bId));
        let actions = isAct ? '<span class="bot-pool-item__badge">Primary</span>' : '';
        if (!isAct) {
          actions += `<button type="button" class="set-active-bot-btn bot-pool-item__set-btn" data-bot-id="${bId}">Set Primary</button>`;
        }
        actions += `<button type="button" class="remove-bot-btn bot-pool-item__remove-btn" data-bot-id="${bId}" title="Remove bot"><i class="fas fa-trash-alt"></i></button>`;

        return `
          <div class="bot-pool-item">
            <div class="bot-pool-item__info">
              <span class="bot-pool-item__dot ${isAct ? 'bot-pool-item__dot--active' : 'bot-pool-item__dot--inactive'}">●</span>
              <div class="bot-pool-item__name-wrap">
                <span class="bot-pool-item__name">${bName}</span>
                ${bUser ? `<span class="bot-pool-item__user">@${bUser}</span>` : ''}
              </div>
            </div>
            <div class="bot-pool-item__actions">
              ${actions}
            </div>
          </div>
        `;
      }).join('');

      // Bind action buttons
      listEl.querySelectorAll('.set-active-bot-btn').forEach(btn => {
        btn.addEventListener('click', async () => {
          const targetId = btn.getAttribute('data-bot-id');
          if (!targetId) return;
          try {
            await this.requestJson(`${this.localApiBase}/api/bots/set-active`, {
              method: 'POST',
              body: JSON.stringify({ bot_id: targetId })
            });
            await this.loadSessionStatus();
            await this.loadBotPool();
          } catch (e) {
            console.error('Failed to set active bot:', e);
          }
        });
      });

      listEl.querySelectorAll('.remove-bot-btn').forEach(btn => {
        btn.addEventListener('click', async () => {
          const targetId = btn.getAttribute('data-bot-id');
          if (!targetId || !confirm('Remove this bot from pool?')) return;
          try {
            await this.requestJson(`${this.localApiBase}/api/bots/remove`, {
              method: 'POST',
              body: JSON.stringify({ bot_id: targetId })
            });
            await this.loadSessionStatus();
            await this.loadBotPool();
          } catch (e) {
            console.error('Failed to remove bot:', e);
          }
        });
      });
    } catch (e) {
      console.warn('loadBotPool failed:', e);
    }
  }

  async loadSessionStatus() {
    try {
      const data = await this.requestJson(`${this.localApiBase}/api/session`);
      if (data?.version) {
        this.version = String(data.version);
        this.renderSettingsVersion();
      }
      if (data?.device_id) {
        this.deviceId = String(data.device_id).trim();
      }
      const hasSession = Boolean(data?.has_session);
      const isProvisioning = Boolean(data?.is_provisioning);

      // ── Runtime preflight ────────────────────────────────────────────────
      // A missing dependency (vendor/, fileinfo, openssl, mbstring, curl) or a
      // 32-bit PHP build can never be fixed by entering a bot token. Show a
      // dedicated error instead of the bot-token gate so the user knows what
      // is actually wrong.
      if (data?.environment_fatal) {
        this.showFatalError({
          problems: data.environment_problems || [],
          hints: data.environment_hints || []
        });
        return;
      }

      if (hasSession) {
        this.botId = String(data.bot_id || this.botId || '').trim();
        this.botUsername = String(data.bot_username || this.botUsername || '');
        this.botName = String(data.bot_name || this.botName || '');
        this.apiSecret = String(data.api_secret || this.apiSecret || '');
        if (!this.botId) {
          this._clearCachedSession();
        } else {
          this.hasSession = true;
          this._persistCachedSession();
          this.loadBotPool();
        }
      } else if (isProvisioning) {
        // Server is actively provisioning a guest bot session in the background.
        // Poll a bounded number of times so a stuck provision cannot loop the
        // UI forever (observed: an endless "Provisioning guest bot..." spinner).
        this._provisionPollCount = (this._provisionPollCount || 0) + 1;
        this.provisionError = 'Provisioning guest bot...';
        const loadText = document.querySelector('.loading-screen__text');
        if (loadText) loadText.textContent = 'Provisioning guest bot...';

        if (this._provisionPollCount > 20) {
          // ~50s of polling with no session — stop and surface the gate.
          this._provisionPollCount = 0;
          this.provisionError = 'Provisioning is taking longer than expected. Please restart the PencariMovie Server, or enter your Telegram bot token below.';
          this._clearCachedSession();
          this.showSettingsGate({ forceToken: true, message: this.provisionError, messageType: 'error' });
          return;
        }

        // Schedule an automatic poll in 2.5s to check if the session is ready
        setTimeout(async () => {
          await this.loadSessionStatus();
          if (this.hasSession) {
            this._provisionPollCount = 0;
            window.location.reload();
          }
        }, 2500);
      } else if (data && data.ok === 1) {
        // No session found and not currently provisioning — attempt 1-click automatic guest bot provisioning on the fly
        const loadText = document.querySelector('.loading-screen__text');
        if (loadText) loadText.textContent = 'Provisioning guest bot...';

        // Pre-flight clock check: a skewed clock makes every MTProto handshake
        // fail with a confusing "message ID too new/old" error. Detect it now
        // and show the exact offset so the user can fix it before we waste a
        // provisioning attempt.
        try {
          const clock = await this.requestJson(`${this.localApiBase}/api/clock-check`);
          // Use `skewed` (>30s), not `critical` (>60s): a measured 48s offset
          // was already enough to break the MTProto auth key exchange.
          if (clock && clock.ok && clock.skewed) {
            const off = Math.abs(Number(clock.offset_seconds) || 0);
            const mins = Math.floor(off / 60);
            const secs = off % 60;
            const human = mins > 0 ? `${mins}m ${secs}s` : `${secs}s`;
            const dir = Number(clock.offset_seconds) > 0 ? 'ahead of' : 'behind';
            this.provisionError =
              `Device clock out of sync!\nYour clock is ${human} ${dir} Telegram's server time.\n\n` +
              `Enable 'Set time automatically' (Automatic date and time / NTP) in your device Settings, then restart the PencariMovie Server.`;
            this._clearCachedSession();
            return;
          }
        } catch (clockErr) {
          // Non-fatal: if the probe fails, continue with provisioning.
          console.warn('Clock pre-check failed, continuing:', clockErr);
        }

        try {
          const provResp = await this.requestJson(`${this.localApiBase}/api/provision`, {
            method: 'POST'
          });

          if (provResp?.ok && provResp?.bot_id) {
            this.botId = String(provResp.bot_id).trim();
            this.botUsername = String(provResp.bot_username || '');
            this.botName = String(provResp.bot_name || '');
            this.apiSecret = String(provResp.api_secret || '');
            this.hasSession = true;
            this._persistCachedSession();
            this.loadBotPool();
            return;
          } else {
            this.provisionError = provResp?.message || 'Auto-connect unavailable. Please enter your Telegram bot token.';
          }
        } catch (provErr) {
          console.warn('Auto-provision failed, falling back to manual gate:', provErr);
          const errMsg = provErr?.message || '';
          this.provisionError = errMsg || 'Auto-connect unavailable. Please enter your Telegram bot token.';
        }
        this._clearCachedSession();
      }
    } catch (error) {
      console.warn('Session check failed:', error);
      // Keep the cached login. A failed probe after refresh must not
      // look like a logout.
      if (!this.hasSession) {
        this._restoreCachedSession();
      }
    }
  }

  _isReloginRequired(message) {
    const text = String(message || '').toLowerCase();
    // Do NOT trigger logout on channel permission errors ("not allowed", "forbidden", etc.)
    // Only trigger re-login when explicitly told the token/secret is invalid or bot_id not found
    return text.includes('bot_id not found')
      || text.includes('reset and enter')
      || text.includes('enter new bot token')
      || text.includes('enter new one');
  }

  promptBotRelogin(message) {
    const msg = String(message || '').trim()
      || 'Bot ID not found. Please enter your bot token again.';
    this.clearSession().then(() => {
      this._clearCachedSession();
      const input = this.$('#botTokenInput');
      if (input) input.value = '';
      this.showSettingsGate({ forceToken: true, message: msg });
    }).catch((err) => {
      console.warn('Failed to prompt bot re-login:', err);
      this._clearCachedSession();
      this.showSettingsGate({ forceToken: true, message: msg });
    });
  }

  /**
   * Show the fatal runtime error overlay (missing dependency / unsupported OS).
   * Used when the backend reports `environment_fatal` — a condition that
   * entering a bot token can never fix.
   */
  showFatalError(options = {}) {
    this._hideLoadingScreen();

    const overlay = this.$('#fatalErrorOverlay');
    if (!overlay) return;

    const problems = Array.isArray(options.problems) ? options.problems : [];
    const hints = Array.isArray(options.hints) ? options.hints : [];

    const listEl = this.$('#fatalErrorProblems');
    if (listEl) {
      listEl.innerHTML = '';
      problems.forEach((p) => {
        const li = document.createElement('li');
        li.textContent = String(p);
        listEl.appendChild(li);
      });
    }

    const hintsEl = this.$('#fatalErrorHints');
    if (hintsEl) {
      hintsEl.innerHTML = '';
      hints.forEach((h) => {
        const p = document.createElement('p');
        p.textContent = String(h);
        hintsEl.appendChild(p);
      });
    }

    // Hide every other gate so only the fatal error is visible.
    ['#settingsGate', '#authGate', '#streamApp'].forEach((sel) => {
      const el = this.$(sel);
      if (el) {
        el.classList.add('hidden');
        el.setAttribute('aria-hidden', 'true');
      }
    });

    overlay.classList.remove('hidden');
    overlay.setAttribute('aria-hidden', 'false');

    const retryBtn = this.$('#fatalErrorRetry');
    if (retryBtn && !retryBtn.dataset.bound) {
      retryBtn.dataset.bound = '1';
      retryBtn.addEventListener('click', () => window.location.reload());
    }
  }

  showSettingsGate(options = {}) {
    this._hideLoadingScreen();

    const gate = this.$('#settingsGate');
    const app = this.$('#streamApp');
    const tokenSection = this.$('#settingsTokenSection');
    const connectedSection = this.$('#settingsConnectedSection');
    const closeBtn = this.$('#settingsClose');
    const statusEl = this.$('#settingsStatus');
    const forceToken = Boolean(options.forceToken) || !this.hasSession;

    if (!gate) return;

    this.renderSettingsVersion();

    gate.classList.remove('hidden');
    gate.setAttribute('aria-hidden', 'false');
    gate.removeAttribute('inert');

    if (!forceToken && this.hasSession) {
      // ── Overlay mode: bot connected ──
      // Keep streamApp visible underneath
      if (app) {
        app.classList.remove('hidden');
        app.setAttribute('aria-hidden', 'false');
        app.removeAttribute('inert');
      }

      // Show connected info, hide token input
      if (tokenSection) tokenSection.classList.add('hidden');
      if (connectedSection) connectedSection.classList.remove('hidden');
      if (closeBtn) closeBtn.style.display = 'flex';

      // Always reload and render current bot pool in connected mode
      this.loadBotPool();
      this.loadTunnelStatus();

      // Fill bot info
      const nameEl = this.$('#settingsBotName');
      const usernameEl = this.$('#settingsBotUsername');
      if (nameEl) nameEl.textContent = this.botName || 'Connected';
      if (usernameEl) usernameEl.textContent = this.botUsername ? '@' + this.botUsername : '';

      const descPlaceholder = this.$('#tunnelSubdomainPlaceholder');
      if (descPlaceholder) {
        descPlaceholder.textContent = this.tunnelPublicUrl || (this.deviceId ? `https://${this.deviceId}-tunnel.pencarimovie.com` : 'https://{deviceId}-tunnel.pencarimovie.com');
      }
    } else {
      // ── Setup / re-login mode ──
      // Hide streamApp underneath
      if (app) {
        app.classList.add('hidden');
        app.setAttribute('aria-hidden', 'true');
        app.toggleAttribute('inert', true);
      }

      // Show token input, hide connected info
      if (tokenSection) tokenSection.classList.remove('hidden');
      if (connectedSection) connectedSection.classList.add('hidden');
      if (closeBtn) closeBtn.style.display = 'none';

      // Focus token input
      const input = this.$('#botTokenInput');
      if (input) setTimeout(() => input.focus(), 100);
    }

    const connectedStatusEl = this.$('#settingsConnectedStatus');
    const updateStatus = (el) => {
      if (!el) return;
      if (options.message) {
        el.textContent = options.message;
        if (options.messageType === 'success') {
          el.style.color = '#51cf66';
        } else if (options.messageType === 'error') {
          el.style.color = '#ff6b6b';
        } else {
          el.style.color = '';
        }
      } else {
        el.textContent = '';
        el.style.color = '';
      }
    };
    updateStatus(statusEl);
    updateStatus(connectedStatusEl);

    // Hide file detail page if open
    const filePage = this.$('#fileDetailPage');
    if (filePage) {
      filePage.classList.add('hidden');
      filePage.setAttribute('aria-hidden', 'true');
    }

    // Stop hero rotation while settings are open
    this._stopHeroRotation();
  }

  hideSettingsGate() {
    // Must hide the loading screen first — it has z-index 10000 (above everything)
    this._hideLoadingScreen();

    const gate = this.$('#settingsGate');
    const app = this.$('#streamApp');

    if (gate) {
      gate.classList.add('hidden');
      gate.setAttribute('aria-hidden', 'true');
      gate.toggleAttribute('inert', true);
    }
    if (app) {
      app.classList.remove('hidden');
      app.setAttribute('aria-hidden', 'false');
      app.removeAttribute('inert');
    }

    // Reset UI state to setup-mode defaults for next time
    const tokenSection = this.$('#settingsTokenSection');
    const connectedSection = this.$('#settingsConnectedSection');
    const closeBtn = this.$('#settingsClose');
    const statusEl = this.$('#settingsStatus');
    if (tokenSection) tokenSection.classList.remove('hidden');
    if (connectedSection) connectedSection.classList.add('hidden');
    if (closeBtn) closeBtn.style.display = '';
    if (statusEl) statusEl.textContent = '';
  }

  closeSettingsGate() {
    // Closes the settings overlay when bot is connected (no logout)
    this.hideSettingsGate();
    this._startHeroRotation();
  }

  updateBotBadge() {
    const badge = this.$('#botStatusBadge');
    if (!badge) return;
    if (this.hasSession && this.botUsername) {
      badge.innerHTML = `<span style="color:#4caf49;">●</span> @${this.escapeHtml(this.botUsername)}`;
      badge.style.color = '#4caf49';
    } else if (this.hasSession) {
      badge.innerHTML = `<span style="color:#4caf49;">●</span> Connected`;
      badge.style.color = '#4caf49';
    } else {
      badge.innerHTML = `○ Disconnected`;
      badge.style.color = 'var(--text-muted)';
    }
  }

  async saveSettings(providedToken = null) {
    const input = this.$('#botTokenInput');
    const statusEl = this.$('#settingsStatus');
    const rawTokens = providedToken ? providedToken.trim() : (input ? input.value.trim() : '');

    if (!rawTokens) {
      if (statusEl) statusEl.textContent = 'Bot Token is required.';
      return;
    }

    const tokens = rawTokens.split(/[\r\n\s,]+/).map(t => t.trim()).filter(Boolean);
    if (tokens.length === 0) {
      if (statusEl) statusEl.textContent = 'Please enter at least one valid bot token.';
      return;
    }

    const primaryToken = tokens[0];
    const extraTokens = tokens.slice(1);

    if (statusEl) statusEl.textContent = tokens.length > 1
      ? `Validating primary bot (1/${tokens.length})...`
      : 'Validating bot token...';

    try {
      // 1. Direct browser handshake to WordPress to avoid DNS/cURL issues on local device runtime
      let wpHandshake = null;
      try {
        const wpResp = await fetch(`${this.wpApiBase}/save-bot-token`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-App-Version': this.version || '2.0.0'
          },
          body: JSON.stringify({ bot_token: primaryToken })
        });
        if (wpResp.ok) {
          wpHandshake = await wpResp.json();
        }
      } catch (wpErr) {
        console.warn('Direct browser WordPress save-bot-token call skipped/failed, falling back to local backend proxy:', wpErr);
      }

      // 2. Post to local backend (forwarding WordPress encrypted credentials if browser obtained them)
      const loginPayload = { bot_token: primaryToken };
      if (wpHandshake && wpHandshake.ok && wpHandshake.encrypted_credentials) {
        loginPayload.encrypted_credentials = wpHandshake.encrypted_credentials;
        loginPayload.encryption_iv = wpHandshake.encryption_iv;
        if (wpHandshake.api_secret) {
          loginPayload.api_secret = wpHandshake.api_secret;
        }
      }

      const loginResp = await this.requestJson(`${this.localApiBase}/api/botlogin`, {
        method: 'POST',
        body: JSON.stringify(loginPayload)
      });

      if (loginResp?.ok !== 1) {
        if (statusEl) {
          statusEl.textContent = loginResp?.message || 'Login failed: unknown error';
          statusEl.style.whiteSpace = 'pre-line';
        }
        return;
      }

      const botId = String(loginResp.bot_id || '').trim();
      const botUsername = String(loginResp.bot_username || '');
      const botName = String(loginResp.bot_name || '');

      this.apiSecret = String(loginResp.api_secret || '');
      this.botId = botId;
      this.botUsername = botUsername;
      this.botName = botName;
      this.hasSession = true;
      this._persistCachedSession();

      // If multiple tokens were supplied, connect the rest into the bot pool in background/sequentially
      if (extraTokens.length > 0) {
        if (statusEl) statusEl.textContent = `Connecting ${extraTokens.length} extra bot(s)...`;
        try {
          await this.requestJson(`${this.localApiBase}/api/bots/add`, {
            method: 'POST',
            body: JSON.stringify({ tokens: extraTokens })
          });
        } catch (addErr) {
          console.warn('Failed to add extra bots during initial setup:', addErr);
        }
      }

      this.hideSettingsGate();
      this.updateBotBadge();
      if (statusEl) statusEl.textContent = '';

      // Load streaming data
      await this.loadInitialData();

      // After login, re-check version in case server-side policy changed
      try {
        const versionInfo = await this.checkVersion();
        if (versionInfo && versionInfo.update_needed) {
          this.showUpdateRequired(versionInfo);
          return;
        }
      } catch (e) {
        console.warn('Version check after login failed:', e);
      }
    } catch (error) {
      if (statusEl) {
        const rawMsg = error.message || '';
        if (
          rawMsg.includes('Failed to fetch') ||
          rawMsg.includes('NetworkError') ||
          rawMsg.includes('Load failed')
        ) {
          statusEl.textContent = 'Connection error: Could not connect to local server (Failed to fetch).\nPlease ensure the server is running on Termux / Android and not blocked by battery optimizer.';
        } else {
          statusEl.textContent = 'Login failed: ' + rawMsg;
        }
        statusEl.style.whiteSpace = 'pre-line';
      }
    }
  }

  async clearSession() {
    try {
      await this.requestJson(`${this.localApiBase}/api/botlogout`, {
        method: 'POST',
        body: '{}'
      });
    } catch (error) {
      console.warn('Failed to clear session:', error);
    }
    // Clear all cached data — session change means stale data
    this._cacheClear();
  }

  // ══════════════════════════════════════════════════════════════
  //  HELPERS
  // ══════════════════════════════════════════════════════════════

  decodeHtmlEntities(value) {
    if (!value) return '';
    let str = String(value);
    // 1. First standard unescape for named and numeric entities
    try {
      const doc = new DOMParser().parseFromString(str, 'text/html');
      str = doc.documentElement.textContent || str;
    } catch (e) {
      str = str.replace(/&/g, '&').replace(/"/g, '"').replace(/&#039;/g, "'").replace(/</g, '<').replace(/>/g, '>');
    }
    // 2. Fix unclosed or malformed entity remnants (e.g. &amp, &quot, &;, &amp;)
    str = str.replace(/&amp(?:;|\b(?=[^\w;]|$))/gi, '&');
    str = str.replace(/&quot(?:;|\b(?=[^\w;]|$))/gi, '"');
    str = str.replace(/&apos(?:;|\b(?=[^\w;]|$))/gi, "'");
    str = str.replace(/&lt(?:;|\b(?=[^\w;]|$))/gi, '<');
    str = str.replace(/&gt(?:;|\b(?=[^\w;]|$))/gi, '>');
    str = str.replace(/&\s*;\s*/g, '& ');
    try {
      const doc2 = new DOMParser().parseFromString(str, 'text/html');
      return (doc2.documentElement.textContent || str).trim();
    } catch (e) {
      return str.trim();
    }
  }

  escapeHtml(value) {
    const decoded = this.decodeHtmlEntities(value);
    return String(decoded ?? '').replace(/[&<>'"]/g, (char) => ({
      '&': '&',
      '<': '<',
      '>': '>',
      "'": '&#039;',
      '"': '"'
    })[char]);
  }

  cleanMediaTitle(title) {
    if (!title) return '';
    let t = String(title);
    // Strip emojis
    t = t.replace(/[\u{1F300}-\u{1F9FF}\u{2600}-\u{26FF}\u{2700}-\u{27BF}]/gu, ' ');
    // Strip website / streaming host prefixes
    t = t.replace(/^(?:on9[._\s]stream[._\s]+|stream[._\s]+|www\.[a-z0-9.-]+\.[a-z]{2,}[._\s]+)/i, '');
    // Strip forwarded and join channel spam
    t = t.replace(/forwarded[._\s]from.*$/i, '');
    t = t.replace(/(?:Join[._\s]Channel|Join[._\s]Group|Join[._\s]us|Join[._\s]@).*$/i, '');
    t = t.replace(/kumpulan[._\s]drama.*$/i, '');
    t = t.replace(/Please[.\s]Don['""]?t[.\s]Forward.*$/i, '');
    t = t.replace(/(?:Req\.By|Request\.By|File\.Request\.By|Requested\.By).*$/i, '');
    t = t.replace(/(?:Channel\.Terbaik\.Anda|Filemku\.bot|LayarAsiaBot|filembot).*$/i, '');
    t = t.replace(/(?:https?:\/\/|httpst\.me|https?\.?t\.me|\bt\.me\/)[\w./?=&_-]*/gi, '');
    t = t.replace(/[._\s]+Watch[._\s]Hd[._\s]Video[._\s]Online.*$/i, '');
    t = t.replace(/(?:^|[.\s_#-]+)Open[.\s_-]*Mini[.\s_-]*App.*$/iu, '');
    t = t.replace(/(?:[.\s_-]*\d+(?:[.,]\d+)?[.\s_-]*(?:MB|GB|KB|TB))+(?:[.\s_-]*https)?(?:[.\s_-]*Open[.\s_-]*Mini[.\s_-]*App)?$/iu, '');
    // Trim punctuation / whitespace
    return t.replace(/^[\s._\-=\t\n\r]+|[\s._\-=\t\n\r]+$/g, '');
  }

  extractMediaTags(title = '', caption = '') {
    const text = `${title} ${caption}`;

    // Resolution
    let res = '';
    if (/\b(2160p|4k|uhd)\b/i.test(text)) res = '4K';
    else if (/\b(1080p|fhd)\b/i.test(text)) res = '1080p';
    else if (/\b(720p|hd)\b/i.test(text)) res = '720p';
    else if (/\b(540p)\b/i.test(text)) res = '540p';
    else if (/\b(480p|sd)\b/i.test(text)) res = '480p';
    else if (/\b(360p)\b/i.test(text)) res = '360p';

    // Source
    let source = '';
    if (/\b(remux)\b/i.test(text)) source = 'REMUX';
    else if (/\b(bluray|blu-ray|bdrip|brrip)\b/i.test(text)) source = 'BluRay';
    else if (/\b(web-?dl|webrip|web)\b/i.test(text)) source = 'WEB-DL';
    else if (/\b(hdrip)\b/i.test(text)) source = 'HDRip';
    else if (/\b(hdtv|tvrip|pdtv)\b/i.test(text)) source = 'HDTV';
    else if (/\b(dvdrip|dvd)\b/i.test(text)) source = 'DVDRip';
    else if (/\b(hdcam|camrip|cam|telesync|ts|tc)\b/i.test(text)) source = 'CAM';

    // Platform
    let platform = '';
    if (/\b(nf|netflix)\b/i.test(text)) platform = 'NF';
    else if (/\b(amzn|primevideo|prime)\b/i.test(text)) platform = 'AMZN';
    else if (/\b(dsnp|disney\+?|disney)\b/i.test(text)) platform = 'DSNP';
    else if (/\b(atvp|apple\s*tv\+?)\b/i.test(text)) platform = 'ATVP';
    else if (/\b(hmax|hbo\s*max)\b/i.test(text)) platform = 'HMAX';
    else if (/\b(zee5)\b/i.test(text)) platform = 'ZEE5';
    else if (/\b(hotstar)\b/i.test(text)) platform = 'Hotstar';
    else if (/\b(viki)\b/i.test(text)) platform = 'Viki';
    else if (/\b(wetv)\b/i.test(text)) platform = 'WeTV';
    else if (/\b(iqiyi)\b/i.test(text)) platform = 'iQIYI';
    else if (/\b(starzplay)\b/i.test(text)) platform = 'StarzPlay';

    // Codec
    let codec = '';
    if (/\b(hevc|x265|h\.?265)\b/i.test(text)) codec = 'HEVC';
    else if (/\b(avc|x264|h\.?264)\b/i.test(text)) codec = 'H.264';
    else if (/\b(av1)\b/i.test(text)) codec = 'AV1';
    else if (/\b(xvid|divx)\b/i.test(text)) codec = 'XviD';

    // Audio
    let audio = '';
    if (/\b(atmos)\b/i.test(text)) audio = 'Atmos';
    else if (/\b(ddp\s*5\.1|dd\+\s*5\.1|eac3\s*5\.1)\b/i.test(text)) audio = 'DDP5.1';
    else if (/\b(ddp\s*2\.0|dd\+\s*2\.0|eac3\s*2\.0)\b/i.test(text)) audio = 'DDP2.0';
    else if (/\b(ddp|dd\+|eac3)\b/i.test(text)) audio = 'DDP';
    else if (/\b(dd\s*5\.1|ac3\s*5\.1)\b/i.test(text)) audio = 'DD5.1';
    else if (/\b(ac3|dd)\b/i.test(text)) audio = 'AC3';
    else if (/\b(dts-hd\s*ma)\b/i.test(text)) audio = 'DTS-HD MA';
    else if (/\b(dts-hd)\b/i.test(text)) audio = 'DTS-HD';
    else if (/\b(dts)\b/i.test(text)) audio = 'DTS';
    else if (/\b(truehd)\b/i.test(text)) audio = 'TrueHD';
    else if (/\b(aac\s*5\.1|5\.1\s*aac)\b/i.test(text)) audio = 'AAC5.1';
    else if (/\b(aac\s*2\.0|2\.0\s*aac|aac2)\b/i.test(text)) audio = 'AAC2.0';
    else if (/\b(aac)\b/i.test(text)) audio = 'AAC';
    else if (/\b(flac)\b/i.test(text)) audio = 'FLAC';
    else if (/\b(opus)\b/i.test(text)) audio = 'Opus';

    // Visual
    const visual = [];
    if (/\b(hdr10\+|hdr10|hdr)\b/i.test(text)) visual.push('HDR');
    if (/\b(dolby\s*vision|dovi|dv)\b/i.test(text)) visual.push('DV');
    if (/\b(10bit|10-bit|hi10p?)\b/i.test(text)) visual.push('10bit');
    if (/\b(imax)\b/i.test(text)) visual.push('IMAX');

    // Edition
    let edition = '';
    if (/\b(remastered)\b/i.test(text)) edition = 'Remastered';
    else if (/\b(extended)\b/i.test(text)) edition = 'Extended';
    else if (/\b(uncut)\b/i.test(text)) edition = 'Uncut';
    else if (/\b(repack|proper)\b/i.test(text)) edition = 'Proper';

    return { resolution: res, source, platform, codec, audio, visual, edition };
  }

  async copyToClipboard(text, inputEl = null) {
    const val = String(text || '').trim();
    if (!val) return false;
    let copied = false;
    if (navigator.clipboard && window.isSecureContext) {
      try {
        await navigator.clipboard.writeText(val);
        copied = true;
      } catch (e) {
        copied = false;
      }
    }
    if (!copied) {
      try {
        if (inputEl && typeof inputEl.select === 'function') {
          inputEl.focus();
          inputEl.select();
          if (typeof inputEl.setSelectionRange === 'function') {
            inputEl.setSelectionRange(0, 99999);
          }
          copied = document.execCommand('copy');
        } else {
          const ta = document.createElement('textarea');
          ta.value = val;
          ta.style.position = 'fixed';
          ta.style.left = '-9999px';
          ta.style.top = '0';
          ta.style.opacity = '0';
          document.body.appendChild(ta);
          ta.focus();
          ta.select();
          if (typeof ta.setSelectionRange === 'function') {
            ta.setSelectionRange(0, 99999);
          }
          copied = document.execCommand('copy');
          document.body.removeChild(ta);
        }
      } catch (e) {
        copied = false;
      }
    }
    return copied;
  }

  formatSize(bytes) {
    const size = Number(bytes || 0);
    if (!size) return 'Unknown size';
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let index = 0;
    let output = size;
    while (output >= 1024 && index < units.length - 1) {
      output /= 1024;
      index += 1;
    }
    return `${output.toFixed(output >= 10 || index === 0 ? 0 : 1)} ${units[index]}`;
  }

  encodeDownloadPayload(payload) {
    const json = JSON.stringify(payload);
    const base64 = btoa(unescape(encodeURIComponent(json)));
    return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/g, '');
  }

  formatStreamFilename(fileName, mime = '') {
    let name = String(fileName || '').trim();
    if (!name) name = 'file';

    // Preserve existing extension if it's already a raw split chunk (.001, .002) or archive
    if (/\.(?:0\d{2,3}|part\d+|\d{3}|zip|rar|7z|tar|gz|bz2|xz|iso|bin|exe|apk|pdf|epub)$/i.test(name)) {
      name = name.replace(/[^\w.\-]+/g, '_') || 'file';
      return name.replace(/^[._\-]+|[._\-]+$/g, '') || 'file';
    }

    const extFromMime = (m) => {
      const lower = String(m || '').toLowerCase().trim();
      if (!lower) return 'mp4';
      if (lower.includes('matroska')) return 'mkv';
      if (lower.includes('webm')) return 'webm';
      if (lower.includes('quicktime')) return 'mov';
      if (lower.includes('x-msvideo') || lower.includes('avi')) return 'avi';
      if (lower.includes('mp2t') || lower.includes('m2ts')) return 'ts';
      if (lower.includes('flv')) return 'flv';
      if (lower.includes('wmv')) return 'wmv';
      if (lower.includes('3gp')) return '3gp';
      if (lower.includes('mp4') || lower.includes('m4v')) return 'mp4';
      if (lower.includes('mpeg') || lower.includes('mpg')) return 'mpg';
      if (lower.includes('audio/mpeg') || lower.includes('mp3')) return 'mp3';
      if (lower.includes('audio/mp4') || lower.includes('m4a')) return 'm4a';
      if (lower.includes('audio/flac') || lower.includes('flac')) return 'flac';
      if (lower.includes('audio/wav') || lower.includes('wave')) return 'wav';
      if (lower.includes('audio/ogg') || lower.includes('opus')) return 'ogg';
      if (lower.includes('zip')) return 'zip';
      if (lower.includes('x-rar') || lower.includes('rar')) return 'rar';
      if (lower.includes('7z')) return '7z';
      if (lower.includes('tar')) return 'tar';
      if (lower.includes('gzip')) return 'gz';
      return 'mp4';
    };

    const targetExt = extFromMime(mime);
    if (!name) name = `video.${targetExt}`;

    // Strip any existing media extensions so we don't end up with duplicate extensions like .mp4.mkv
    const mediaExtsPattern = /\.(mp4|m4v|mkv|webm|avi|mov|ts|m2ts|flv|wmv|3gp|mpg|mpeg|mp3|m4a|flac|wav|ogg|opus)$/i;
    while (mediaExtsPattern.test(name)) {
      name = name.replace(mediaExtsPattern, '');
    }

    name = name.replace(/[^\w.\-]+/g, '_') || 'video';
    name = name.replace(/^[._\-]+|[._\-]+$/g, '') || 'video';

    name = `${name}.${targetExt}`;
    return name;
  }

  buildDownloadUrl(fileId, fileSize, fileName, fileMime, botId = null, shortCode = null) {
    const safeName = this.formatStreamFilename(fileName, fileMime);
    const payload = {
      file_id: fileId,
      file_size: fileSize,
      file_name: safeName,
      mime: fileMime
    };
    if (botId) {
      payload.bot_id = botId;
    }
    if (shortCode) {
      payload.short_code = shortCode;
    }
    const d = this.encodeDownloadPayload(payload);
    const base = String(this.localApiBase || '').replace(/\/+$/, '');
    return `${base}/api/download/${encodeURIComponent(d)}/${encodeURIComponent(safeName)}`;
  }


  /**
   * Guess whether a file is video, audio, or unknown based on
   * the title's file extension or the file_type field.
   * @param {string} title
   * @param {string} fileType
   * @returns {'video'|'audio'|null}
   */
  extractSplitPartInfo(filename = '', caption = '') {
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

  groupSplitParts(files = []) {
    const list = [...files];
    list.sort((a, b) => {
      const aInfo = this.extractSplitPartInfo(a.title || '', a.caption || '');
      const bInfo = this.extractSplitPartInfo(b.title || '', b.caption || '');
      if (aInfo.isPart && bInfo.isPart && aInfo.baseKey === bInfo.baseKey) {
        return aInfo.partNum - bInfo.partNum;
      }
      return 0;
    });

    return list.map(f => {
      const info = this.extractSplitPartInfo(f.title || '', f.caption || '');
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

  _guessMediaType(title, fileType) {
    const videoExts = ['mp4', 'mkv', 'avi', 'mov', 'wmv', 'flv', 'webm', 'm4v', '3gp', 'mpg', 'mpeg'];
    const audioExts = ['mp3', 'flac', 'm4a', 'wav', 'ogg', 'aac', 'wma', 'opus'];

    const t = String(title || '').toLowerCase().trim();

    // Check for unplayable split parts (.001, .002, .003, etc.) or archive formats
    if (/\.(0\d{2,3}|part\d+|\d{3})$/i.test(t)) {
      return null;
    }
    const archiveExts = ['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'iso', 'bin', 'exe', 'apk', 'pdf', 'epub', 'txt'];
    const dotIdx = t.lastIndexOf('.');
    if (dotIdx > 0) {
      const ext = t.substring(dotIdx + 1);
      if (archiveExts.includes(ext)) return null;
      if (videoExts.includes(ext)) return 'video';
      if (audioExts.includes(ext)) return 'audio';
    }

    // Check file_type
    const ft = String(fileType || '').toLowerCase().trim();
    if (ft.startsWith('video')) return 'video';
    if (ft.startsWith('audio')) return 'audio';

    return null; // unknown / unplayable inline
  }

  // ══════════════════════════════════════════════════════════════
  //  CACHE LAYER
  // ══════════════════════════════════════════════════════════════

  /**
   * Build a deterministic cache key from a type and params object.
   * @param {'stream'|'resolve'} type
   * @param {Object} params
   * @returns {string}
   */
  _cacheKey(type, params) {
    const sorted = Object.keys(params).sort().reduce((acc, k) => {
      acc[k] = params[k];
      return acc;
    }, {});
    return `${type}:${JSON.stringify(sorted)}`;
  }

  /**
   * Retrieve a cached value. Checks in-memory Map first, then sessionStorage.
   * Returns null if not found or expired (when requireFresh=true).
   * Returns {data, expired} — caller decides whether to use stale data.
   * @param {string} key
   * @returns {{data: any, expired: boolean}|null}
   */
  _cacheGet(key) {
    const now = Date.now();

    // 1. Check in-memory Map
    const memEntry = this._cache.get(key);
    if (memEntry) {
      return {
        data: memEntry.data,
        expired: now > memEntry.expiresAt
      };
    }

    // 2. Fallback to sessionStorage
    try {
      const raw = sessionStorage.getItem(this._cachePrefix + key);
      if (raw) {
        const entry = JSON.parse(raw);
        // Promote to in-memory
        this._cache.set(key, { data: entry.data, expiresAt: entry.expiresAt });
        return {
          data: entry.data,
          expired: now > entry.expiresAt
        };
      }
    } catch (_) {
      // Ignore corrupt sessionStorage entries
    }

    return null;
  }

  /**
   * Store a value in both in-memory Map and sessionStorage.
   * @param {string} key
   * @param {any} data
   * @param {number} ttlMs — time to live in milliseconds
   */
  _cacheSet(key, data, ttlMs) {
    const expiresAt = Date.now() + ttlMs;
    const entry = { data, expiresAt };

    // In-memory
    this._cache.set(key, entry);

    // sessionStorage (fire-and-forget, catch quota errors)
    try {
      sessionStorage.setItem(this._cachePrefix + key, JSON.stringify(entry));
    } catch (_) {
      // sessionStorage may be full or unavailable (private browsing)
    }
  }

  /**
   * Clear all cached data — both in-memory and sessionStorage.
   */
  _cacheClear() {
    this._cache.clear();

    // Remove all sessionStorage entries with our prefix
    try {
      const keysToRemove = [];
      for (let i = 0; i < sessionStorage.length; i++) {
        const k = sessionStorage.key(i);
        if (k && k.startsWith(this._cachePrefix)) {
          keysToRemove.push(k);
        }
      }
      keysToRemove.forEach((k) => sessionStorage.removeItem(k));
    } catch (_) {
      // Ignore
    }
  }

  /**
   * Return the TTL (in ms) for a given proxy-stream action.
   * @param {string} action
   * @returns {number}
   */
  _getStreamTTL(action) {
    switch (action) {
      case 'trending':
      case 'categories':
        return 5 * 60 * 1000; // 5 min
      case 'posts':
        return 2 * 60 * 1000; // 2 min
      case 'search_files':
      case 'search':
        return 30 * 1000;     // 30 sec — freshness matters
      case 'post_files':
      case 'get_post':
        return 5 * 60 * 1000; // 5 min — static metadata
      default:
        return 60 * 1000;     // 1 min fallback
    }
  }

  /**
   * Save file detail origin context to sessionStorage so it survives page refresh.
   * Called from openFileDetail() after setting _cameFromPost / _cameFromSearch.
   */
  _saveFileDetailContext() {
    try {
      const ctx = {};
      if (this._cameFromPost && this._previousPostData) {
        const postId = this._previousPostData.id || this._previousPostData.ID || '';
        if (postId) {
          ctx.from = 'post';
          ctx.postId = postId;
          ctx.postTitle = this._previousPostData.title || this._previousPostData.post_title || '';
          ctx.postExcerpt = this._previousPostData.excerpt || this._previousPostData.post_excerpt || '';
          ctx.postThumbnail = this._previousPostData.thumbnail_url || this._previousPostData._external_featured_image || '';
        }
      } else if (this._cameFromSearch) {
        ctx.from = 'search';
        if (this._savedSearchQuery) {
          ctx.searchQuery = this._savedSearchQuery;
        }
      }
      if (ctx.from) {
        sessionStorage.setItem('pencarimovie_fd_context', JSON.stringify(ctx));
      } else {
        sessionStorage.removeItem('pencarimovie_fd_context');
      }
    } catch (_) {
      // sessionStorage unavailable or quota exceeded
    }
  }

  /**
   * Restore file detail origin context from sessionStorage.
   * Called from init() after session check, before deep link handling.
   * This allows the back button to restore the correct context after a page refresh.
   */
  _restoreFileDetailContext() {
    try {
      const saved = sessionStorage.getItem('pencarimovie_fd_context');
      if (!saved) return;
      const ctx = JSON.parse(saved);
      if (ctx.from === 'post' && ctx.postId) {
        this._cameFromPost = true;
        this._previousPostData = {
          id: ctx.postId,
          ID: ctx.postId,
          title: ctx.postTitle || '',
          post_title: ctx.postTitle || '',
          excerpt: ctx.postExcerpt || '',
          post_excerpt: ctx.postExcerpt || '',
          thumbnail_url: ctx.postThumbnail || '',
          _external_featured_image: ctx.postThumbnail || ''
        };
      } else if (ctx.from === 'search') {
        this._cameFromSearch = true;
        if (ctx.searchQuery) {
          this._savedSearchQuery = ctx.searchQuery;
        }
      }
    } catch (_) {
      // Corrupted data or unavailable storage
    }
  }

  /**
   * Clear persisted file detail context from sessionStorage.
   * Called from closeFileDetail() when the context has been consumed.
   */
  _clearFileDetailContext() {
    try {
      sessionStorage.removeItem('pencarimovie_fd_context');
    } catch (_) {
      // Ignore
    }
  }

  /**
   * Fetch data from network and cache it.
   * @param {string} cacheKey
   * @param {number} ttl
   * @param {Function} fetcher — async function that returns the data
   * @returns {Promise<any>}
   */
  async _fetchAndCache(cacheKey, ttl, fetcher) {
    const data = await fetcher();
    this._cacheSet(cacheKey, data, ttl);
    return data;
  }

  /**
   * Fire a background refresh (no await) and update cache when done.
   * @param {string} cacheKey
   * @param {number} ttl
   * @param {Function} fetcher
   */
  _backgroundRefresh(cacheKey, ttl, fetcher) {
    fetcher()
      .then((data) => this._cacheSet(cacheKey, data, ttl))
      .catch(() => {
        // Silently ignore background refresh failures —
        // stale data is better than nothing.
      });
  }

  isFileDetailOpen() {
    const page = this.$('#fileDetailPage');
    return page && !page.classList.contains('hidden');
  }

  showLoading(show) {
    const el = this.$('#streamLoading');
    if (el) el.classList.toggle('hidden', !show);
  }

  /** Hide the full-page loading screen shown during session check */
  _hideLoadingScreen() {
    const el = this.$('#loadingScreen');
    if (el) {
      el.classList.add('hidden');
      el.setAttribute('aria-hidden', 'true');
    }
  }

  _resetMediaElement(el) {
    if (!el) return;
    el.pause();
    el.dataset.pmIgnoreError = '1';
    el.removeAttribute('poster');
    el.removeAttribute('src');
    try {
      el.src = '';
      el.load();
    } catch (e) {
      // Ignore unload errors from empty src.
    }
    el.classList.add('hidden');
  }

  _resetFileDetailPlayer() {
    const videoEl = this.$('#fileDetailVideo');
    if (videoEl) {
      const tracks = videoEl.querySelectorAll('track');
      tracks.forEach((t) => t.remove());
      videoEl.className = 'file-detail__video sub-size-normal';
      videoEl.style.objectFit = 'contain';
      videoEl.playbackRate = 1.0;
    }
    this._resetMediaElement(videoEl);
    this._resetMediaElement(this.$('#fileDetailAudio'));
    const playerEl = this.$('#fileDetailPlayer');
    if (playerEl) playerEl.classList.add('hidden');
    this._closePlayerDropdowns();
    this.currentSubDelay = 0;
    const delayValEl = this.$('#playerSubDelayVal');
    if (delayValEl) delayValEl.textContent = '0.0s';
  }

  /** Close all open dropdown menus on player toolbar */
  _closePlayerDropdowns() {
    let closedAny = false;
    ['#playerSpeedMenu', '#playerScaleMenu', '#playerSubsMenu', '#playerExtMenu'].forEach((sel) => {
      const el = this.$(sel);
      if (el && !el.classList.contains('hidden')) {
        el.classList.add('hidden');
        closedAny = true;
      }
    });
    return closedAny;
  }

  /** Stremio-style Player Toolbar Event Binding */
  _bindPlayerToolbar() {
    const videoEl = this.$('#fileDetailVideo');
    this.currentSubDelay = 0;

    // Toggle dropdown helper
    const toggleDropdown = (dropdownId) => {
      const target = this.$(dropdownId);
      if (!target) return;
      const wasHidden = target.classList.contains('hidden');
      this._closePlayerDropdowns();
      if (wasHidden) target.classList.remove('hidden');
    };

    // Close on clicking outside
    document.addEventListener('click', (e) => {
      if (!e.target.closest('#playerToolbar')) {
        this._closePlayerDropdowns();
      }
    });

    // Speed button & options
    const speedBtn = this.$('#playerSpeedBtn');
    if (speedBtn) {
      speedBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        toggleDropdown('#playerSpeedMenu');
      });
    }
    this.$$('.player-toolbar__opt[data-speed]').forEach((btn) => {
      btn.addEventListener('click', () => {
        const speed = parseFloat(btn.getAttribute('data-speed')) || 1;
        if (videoEl) videoEl.playbackRate = speed;
        const lbl = this.$('#playerSpeedLabel');
        if (lbl) lbl.textContent = `${speed}x`;
        this.$$('.player-toolbar__opt[data-speed]').forEach((b) => b.classList.remove('active'));
        btn.classList.add('active');
        this._closePlayerDropdowns();
      });
    });

    // Aspect Ratio / Scale button & options
    const scaleBtn = this.$('#playerScaleBtn');
    if (scaleBtn) {
      scaleBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        toggleDropdown('#playerScaleMenu');
      });
    }
    this.$$('.player-toolbar__opt[data-scale]').forEach((btn) => {
      btn.addEventListener('click', () => {
        const scale = btn.getAttribute('data-scale') || 'contain';
        if (videoEl) videoEl.style.objectFit = scale;
        const lbl = this.$('#playerScaleLabel');
        if (lbl) {
          lbl.textContent = scale === 'contain' ? 'Fit' : scale === 'cover' ? 'Fill' : 'Stretch';
        }
        this.$$('.player-toolbar__opt[data-scale]').forEach((b) => b.classList.remove('active'));
        btn.classList.add('active');
        this._closePlayerDropdowns();
      });
    });

    // Picture-in-Picture (PiP)
    const pipBtn = this.$('#playerPipBtn');
    if (pipBtn) {
      pipBtn.addEventListener('click', async () => {
        if (!videoEl) return;
        try {
          if (document.pictureInPictureElement) {
            await document.exitPictureInPicture();
          } else if (videoEl.requestPictureInPicture) {
            await videoEl.requestPictureInPicture();
          }
        } catch (err) {
          console.warn('[Player] PiP failed:', err);
        }
      });
    }

    // Subtitles dropdown toggle & track management
    const subsBtn = this.$('#playerSubsBtn');
    if (subsBtn) {
      subsBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        this._populateSubtitlesList();
        toggleDropdown('#playerSubsMenu');
      });
    }

    // Subtitle delay adjustments
    this.$('#playerSubDelayMinus')?.addEventListener('click', () => this._adjustSubtitleDelay(-0.25));
    this.$('#playerSubDelayPlus')?.addEventListener('click', () => this._adjustSubtitleDelay(0.25));
    this.$('#playerSubDelayReset')?.addEventListener('click', () => this._setSubtitleDelay(0));

    // Subtitle sizing
    this.$$('.player-toolbar__sync-btn[data-sub-size]').forEach((btn) => {
      btn.addEventListener('click', () => {
        const sz = btn.getAttribute('data-sub-size');
        if (videoEl) {
          videoEl.classList.remove('sub-size-small', 'sub-size-normal', 'sub-size-large');
          videoEl.classList.add(`sub-size-${sz}`);
        }
        this.$$('.player-toolbar__sync-btn[data-sub-size]').forEach((b) => b.classList.remove('active'));
        btn.classList.add('active');
      });
    });

    // Local Subtitle File Upload
    const subFileInput = this.$('#playerLocalSubFile');
    if (subFileInput) {
      subFileInput.addEventListener('change', async (e) => {
        const file = e.target.files?.[0];
        if (!file || !videoEl) return;
        const text = await file.text();
        let vtt = text;
        if (!text.trim().startsWith('WEBVTT')) {
          vtt = 'WEBVTT\n\n' + text.replace(/\r\n|\r/g, '\n').replace(/(\d{2}:\d{2}:\d{2}),(\d{3})/g, '$1.$2');
        }
        const blob = new Blob([vtt], { type: 'text/vtt' });
        const objectUrl = URL.createObjectURL(blob);
        const track = document.createElement('track');
        track.kind = 'subtitles';
        track.label = `Local: ${file.name}`;
        track.srclang = 'custom';
        track.src = objectUrl;
        track.default = true;
        videoEl.appendChild(track);
        setTimeout(() => {
          if (videoEl.textTracks && videoEl.textTracks.length > 0) {
            for (let i = 0; i < videoEl.textTracks.length; i++) {
              videoEl.textTracks[i].mode = (i === videoEl.textTracks.length - 1) ? 'showing' : 'disabled';
            }
          }
          this._populateSubtitlesList();
        }, 100);
        this._closePlayerDropdowns();
      });
    }

    // External Player Menu toggle
    const extMenuBtn = this.$('#playerExtMenuBtn');
    if (extMenuBtn) {
      extMenuBtn.addEventListener('click', (e) => {
        e.stopPropagation();
        this._updateExternalPlayerUrls();
        toggleDropdown('#playerExtMenu');
      });
    }

    // Copy stream URL button
    const copyStreamBtn = this.$('#playerCopyStreamBtn');
    if (copyStreamBtn) {
      copyStreamBtn.addEventListener('click', async () => {
        const currentUrl = videoEl?.src || this.currentStreamUrl || '';
        if (currentUrl) {
          try {
            await navigator.clipboard.writeText(currentUrl);
            const orig = copyStreamBtn.innerHTML;
            copyStreamBtn.innerHTML = '<i class="fas fa-check" style="color:#20bf6b;margin-right:8px;"></i> Copied!';
            setTimeout(() => { copyStreamBtn.innerHTML = orig; }, 2000);
          } catch (e) {
            window.prompt('Copy Stream URL:', currentUrl);
          }
        }
        this._closePlayerDropdowns();
      });
    }
  }

  /** Populate the subtitles list inside the dropdown from available textTracks */
  _populateSubtitlesList() {
    const listEl = this.$('#playerSubsList');
    const videoEl = this.$('#fileDetailVideo');
    if (!listEl || !videoEl) return;

    listEl.innerHTML = '';
    const tracks = videoEl.textTracks || [];
    let hasActive = false;

    // Track buttons
    for (let i = 0; i < tracks.length; i++) {
      const t = tracks[i];
      const isShowing = t.mode === 'showing';
      if (isShowing) hasActive = true;

      const btn = document.createElement('button');
      btn.type = 'button';
      btn.className = `player-toolbar__opt ${isShowing ? 'active' : ''}`;
      btn.textContent = t.label || t.language || `Track ${i + 1}`;
      btn.addEventListener('click', () => {
        for (let j = 0; j < tracks.length; j++) {
          tracks[j].mode = (j === i) ? 'showing' : 'disabled';
        }
        this._populateSubtitlesList();
        this._closePlayerDropdowns();
      });
      listEl.appendChild(btn);
    }

    // Off button
    const offBtn = document.createElement('button');
    offBtn.type = 'button';
    offBtn.className = `player-toolbar__opt ${!hasActive ? 'active' : ''}`;
    offBtn.textContent = 'Off';
    offBtn.addEventListener('click', () => {
      for (let j = 0; j < tracks.length; j++) {
        tracks[j].mode = 'disabled';
      }
      this._populateSubtitlesList();
      this._closePlayerDropdowns();
    });
    listEl.insertBefore(offBtn, listEl.firstChild);
  }

  /** Adjust subtitle delay by offset in seconds */
  _adjustSubtitleDelay(delta) {
    this._setSubtitleDelay((this.currentSubDelay || 0) + delta);
  }

  /** Set absolute subtitle delay in seconds */
  _setSubtitleDelay(delay) {
    this.currentSubDelay = Math.round(delay * 100) / 100;
    const delayValEl = this.$('#playerSubDelayVal');
    if (delayValEl) {
      delayValEl.textContent = `${this.currentSubDelay > 0 ? '+' : ''}${this.currentSubDelay.toFixed(2)}s`;
    }

    const videoEl = this.$('#fileDetailVideo');
    if (!videoEl || !videoEl.textTracks) return;

    // Apply offset shift to cues of all tracks
    for (let i = 0; i < videoEl.textTracks.length; i++) {
      const track = videoEl.textTracks[i];
      if (track.cues) {
        for (let j = 0; j < track.cues.length; j++) {
          const cue = track.cues[j];
          if (cue._origStart === undefined) cue._origStart = cue.startTime;
          if (cue._origEnd === undefined) cue._origEnd = cue.endTime;
          cue.startTime = Math.max(0, cue._origStart + this.currentSubDelay);
          cue.endTime = Math.max(0, cue._origEnd + this.currentSubDelay);
        }
      }
    }
  }

  /** Update External Player deep links (VLC, MPV, Web Stremio) */
  _updateExternalPlayerUrls() {
    const videoEl = this.$('#fileDetailVideo');
    const streamUrl = videoEl?.src || this.currentStreamUrl || '';
    if (!streamUrl) return;

    // VLC protocol: vlc://http...
    const vlcLink = this.$('#playerExtVlc');
    if (vlcLink) {
      vlcLink.href = `vlc://${streamUrl}`;
    }

    // MPV protocol: mpv://http...
    const mpvLink = this.$('#playerExtMpv');
    if (mpvLink) {
      mpvLink.href = `mpv://${streamUrl}`;
    }

    // Web Stremio player link: https://web.stremio.com/#/player/...
    const stremioWebLink = this.$('#playerExtStremioWeb');
    if (stremioWebLink) {
      const encodedStream = encodeURIComponent(JSON.stringify({ url: streamUrl }));
      stremioWebLink.href = `https://web.stremio.com/#/player/${encodedStream}`;
    }
  }

  /** Hide resolving spinner and optionally show action buttons */
  _endResolving(showActions = true) {
    const resolvingEl = this.$('#fileDetailResolving');
    const actionsEl = this.$('#fileDetailActions');
    if (resolvingEl) {
      resolvingEl.classList.add('hidden');
      resolvingEl.setAttribute('aria-hidden', 'true');
    }
    if (actionsEl) {
      if (showActions) {
        actionsEl.classList.remove('hidden');
      } else {
        actionsEl.classList.add('hidden');
      }
    }
  }

  // ══════════════════════════════════════════════════════════════
  //  CATEGORY PAGE (infinite scroll)
  // ══════════════════════════════════════════════════════════════

  /**
   * Open the full-page category browser. Hides the main streaming view,
   * resets pagination, and loads the first page of posts.
   */
  openCategoryPage(slug, name) {
    // Pause hero rotation while category page is displayed
    this._stopHeroRotation();

    // Close any open overlays
    if (this.isSearchOpen) this.closeSearch();
    if (this.isMobileNavOpen) this.closeMobileNav();
    if (this.isFileDetailOpen()) this.closeFileDetail();

    this._categorySlug = slug;
    this._categoryName = name;
    this._categoryOffset = 0;
    this._categoryHasMore = true;
    this._isCategoryPageOpen = true;

    // Clear grid
    const grid = this.$('#categoryPageGrid');
    if (grid) grid.innerHTML = '';

    // Set title
    this.$('#categoryPageTitle').textContent = name;

    // Hide main app, show category page
    const streamApp = this.$('#streamApp');
    const categoryPage = this.$('#categoryPage');
    if (streamApp) {
      streamApp.classList.add('hidden');
      streamApp.setAttribute('aria-hidden', 'true');
    }
    if (categoryPage) {
      categoryPage.classList.remove('hidden');
      categoryPage.setAttribute('aria-hidden', 'false');
      categoryPage.scrollTop = 0;
    }

    // Update hash
    this._suppressHash = true;
    window.location.hash = '#category/' + slug;
    setTimeout(() => { this._suppressHash = false; }, 0);

    // Load first page
    this._loadMoreCategoryPosts();
  }

  /**
   * Close the category page and restore the main streaming view.
   */
  closeCategoryPage() {
    this._isCategoryPageOpen = false;
    this._closeCategoryObserver();

    const categoryPage = this.$('#categoryPage');
    const streamApp = this.$('#streamApp');

    if (categoryPage) {
      categoryPage.classList.add('hidden');
      categoryPage.setAttribute('aria-hidden', 'true');
    }
    if (streamApp) {
      streamApp.classList.remove('hidden');
      streamApp.setAttribute('aria-hidden', 'false');
    }

    // Clear hash (only if a category hash is present)
    const currentHash = window.location.hash;
    if (currentHash && currentHash.startsWith('#category/')) {
      this._suppressHash = true;
      window.location.hash = '#';
      setTimeout(() => { this._suppressHash = false; }, 0);
    }

    // Resume hero rotation when returning to main view
    this._startHeroRotation();
  }

  /**
   * Fetch the next page of posts for the current category and append
   * to the grid. Stops when fewer than PAGE_SIZE results are returned.
   */
  async _loadMoreCategoryPosts() {
    if (this._categoryLoading || !this._categoryHasMore) return;
    this._categoryLoading = true;

    const PAGE_SIZE = 20;
    const loadingEl = this.$('#categoryPageLoading');
    const grid = this.$('#categoryPageGrid');

    if (loadingEl) loadingEl.classList.remove('hidden');

    try {
      const catObj = this.categories.find(c => c.slug === this._categorySlug);
      let posts = [];

      if (catObj && catObj.is_topkw) {
        // Direct file catalog (e.g. top keyword file row)
        const q = catObj.search_query || '';
        const sfRes = await this.fetchStream('search_files', {
          search: q,
          limit: PAGE_SIZE,
          offset: this._categoryOffset
        }).catch(() => null);

        if (sfRes?.files && Array.isArray(sfRes.files)) {
          posts = sfRes.files.map(f => ({
            id: f.short_code,
            short_code: f.short_code,
            title: f.title || 'Telegram File',
            thumbnail_url: f.thumbnail_url || '',
            is_file: true,
            file_size: f.file_size
          }));
        }
      } else {
        const params = {
          limit: PAGE_SIZE,
          offset: this._categoryOffset
        };
        if (this._categorySlug !== 'latest') {
          params.category = this._categorySlug;
        }
        if (catObj?.media_type) {
          params.media_type = catObj.media_type;
        }
        posts = await this.fetchStream('posts', params);
      }

      if (!Array.isArray(posts) || posts.length === 0) {
        this._categoryHasMore = false;
        this._closeCategoryObserver();
        return;
      }

      // If fewer than PAGE_SIZE, there are no more pages
      if (posts.length < PAGE_SIZE) {
        this._categoryHasMore = false;
        this._closeCategoryObserver();
      }

      // Append posts to grid
      if (grid) {
        for (const post of posts) {
          grid.insertAdjacentHTML('beforeend', this._renderCategoryPostCard(post));
        }

        // Re-bind click handlers for new cards
        const newCards = grid.querySelectorAll('.stream-card:not([data-bound])');
        newCards.forEach((card) => {
          card.setAttribute('data-bound', 'true');
          card.addEventListener('click', () => {
            const shortCode = card.getAttribute('data-short-code');
            if (shortCode) {
              this.openFileDetail(shortCode);
              return;
            }
            const postId = card.getAttribute('data-post-id');
            if (postId) {
              const postData = posts.find((p) => String(p.id) === postId || String(p.ID) === postId);
              if (postData) {
                if (postData.is_file || postData.short_code) {
                  this.openFileDetail(postData.short_code || postData.id);
                } else {
                  this.openModal(postData);
                  if (!postData.content && !postData.excerpt) {
                    this.fetchStream('get_post', { post_id: postId }).then((fullPost) => {
                      if (fullPost && this.isModalOpen) {
                        this.openModal(fullPost);
                      }
                    }).catch(() => {});
                  }
                }
              } else {
                this.openModal({ id: postId, post_title: card.getAttribute('data-post-title') || 'Details' });
                this.fetchStream('get_post', { post_id: postId }).then((fullPost) => {
                  if (fullPost && this.isModalOpen) {
                    this.openModal(fullPost);
                  }
                }).catch(() => {});
              }
            }
          });
        });

        // Also bind file cards if any
        const newFileCards = grid.querySelectorAll('.stream-file-card:not([data-bound])');
        newFileCards.forEach((card) => {
          card.setAttribute('data-bound', 'true');
          card.addEventListener('click', () => {
            const shortCode = card.getAttribute('data-short-code');
            if (shortCode) {
              this.openFileDetail(shortCode);
            }
          });
        });
      }

      this._categoryOffset += posts.length;

      // Re-observe sentinel if we have more pages
      if (this._categoryHasMore) {
        this._openCategoryObserver();
      }
    } catch (err) {
      console.warn('Failed to load category posts:', err);
    } finally {
      this._categoryLoading = false;
      if (loadingEl) loadingEl.classList.add('hidden');
    }
  }

  /**
   * Render a single post card for the category page grid.
   */
  _renderCategoryPostCard(item) {
    if (item.is_file || item.short_code) {
      return this._renderFileCard(item);
    }
    const title = item.title || item.post_title || 'Untitled';
    const thumbnail = item.thumbnail_url || item._external_featured_image || '';
    const category = item.category || '';
    const year = item.year || '';
    const id = item.id || '';

    return `
      <div class="stream-card" data-post-id="${id}" data-post-title="${this.escapeHtml(title)}">
        <img class="stream-card__thumb" src="${thumbnail}" alt="${this.escapeHtml(title)}" loading="lazy"
             onerror="this.src='data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%22280%22 height=%22158%22><rect fill=%22%232a2a2a%22 width=%22280%22 height=%22158%22/><text fill=%22%23808080%22 x=%22140%22 y=%2279%22 text-anchor=%22middle%22 font-size=%2214%22>${this.escapeHtml(title)}</text></svg>'">
        <div class="stream-card__overlay">
          <div class="stream-card__title">${this.escapeHtml(title)}</div>
          <div class="stream-card__meta">${year ? year : ''}${category ? ' · ' + this.escapeHtml(category) : ''}</div>
        </div>
      </div>
    `;
  }

  /** Start observing the sentinel element for infinite scroll */
  _openCategoryObserver() {
    this._closeCategoryObserver();
    this._categoryObserver = new IntersectionObserver((entries) => {
      const entry = entries[0];
      if (entry && entry.isIntersecting && this._categoryHasMore && !this._categoryLoading) {
        this._loadMoreCategoryPosts();
      }
    }, { rootMargin: '300px' });

    const sentinel = this.$('#categoryPageSentinel');
    if (sentinel) this._categoryObserver.observe(sentinel);
  }

  /** Disconnect the IntersectionObserver */
  _closeCategoryObserver() {
    if (this._categoryObserver) {
      this._categoryObserver.disconnect();
      this._categoryObserver = null;
    }
  }

  // ══════════════════════════════════════════════════════════════
  //  AUTH GATE (dashboard password)
  // ══════════════════════════════════════════════════════════════

  /**
   * Returns true when the dashboard may load. Shows the auth gate and returns
   * false when a password is required. Localhost/LAN always pass server-side.
   */
  async checkAuth() {
    try {
      const res = await fetch(`${this.localApiBase}/api/auth/status`, {
        cache: 'no-store',
        headers: this._authToken ? { 'X-Auth-Token': this._authToken } : {}
      });
      const data = await res.json();
      if (data?.authenticated) {
        if (data.token) {
          this._authToken = data.token;
          localStorage.setItem('pm.auth', data.token);
        }
        this.hideAuthGate();
        return true;
      }
      this.showAuthGate();
      return false;
    } catch (_) {
      // Network error — let the normal flow surface it rather than blocking.
      return true;
    }
  }

  showAuthGate() {
    const gate = this.$('#authGate');
    if (!gate) return;
    const addonM = this.$('#addonModal');
    if (addonM) addonM.classList.add('hidden');
    const settingsGate = this.$('#settingsGate');
    if (settingsGate) settingsGate.classList.add('hidden');
    gate.classList.remove('hidden');
    gate.setAttribute('aria-hidden', 'false');
    this._hideLoadingScreen?.();
    const input = this.$('#authPasswordInput');
    if (input) input.focus();
  }

  hideAuthGate() {
    const gate = this.$('#authGate');
    if (!gate) return;
    gate.classList.add('hidden');
    gate.setAttribute('aria-hidden', 'true');
  }

  async submitAuthPassword() {
    const input = this.$('#authPasswordInput');
    const btn = this.$('#authConnectBtn');
    const status = this.$('#authStatus');
    const pw = input ? input.value : '';
    if (!pw) return;
    if (btn) btn.disabled = true;
    if (status) status.textContent = '';
    try {
      const res = await fetch(`${this.localApiBase}/api/auth/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ password: pw })
      });
      const data = await res.json();
      if (data?.ok && data.token) {
        this._authToken = data.token;
        localStorage.setItem('pm.auth', data.token);
        this.hideAuthGate();
        this._updateAddonModalUrls?.();
        const addonTokenInput = this.$('#addonTokenInput');
        if (addonTokenInput) addonTokenInput.value = data.token;
        await this.continueInit();
        return;
      }
      if (status) status.textContent = data?.message || 'Wrong password';
    } catch (_) {
      if (status) status.textContent = 'Connection failed';
    }
    if (btn) btn.disabled = false;
  }

  async requestJson(url, options = {}) {
    const authToken = this._authToken || '';
    const response = await fetch(url, {
      cache: 'no-store',
      headers: {
        'Content-Type': 'application/json',
        ...(authToken ? { 'X-Auth-Token': authToken } : {}),
        ...(options.headers || {})
      },
      ...options
    });

    const text = await response.text();
    let data = null;
    try {
      data = text ? JSON.parse(text) : null;
    } catch (error) {
      // ═══ [DIAGNOSTIC] Log raw response when JSON parsing fails ═══
      const preview = text.length > 2000 ? text.substring(0, 2000) + '... [TRUNCATED]' : text;
      console.error('requestJson: non-JSON response', {
        url,
        status: response.status,
        contentType: response.headers?.get?.('content-type') || 'unknown',
        bodyLength: text.length,
        bodyPreview: preview,
      });
      throw new Error(`API returned non-JSON response (${response.status}).`);
    }

    if (!response.ok) {
      if (response.status === 426 && data?.update_needed) {
        this.showUpdateRequired(data);
      }
      if (response.status === 401 && data?.auth_required) {
        this.showAuthGate();
      }
      throw new Error(data?.message || data?.code || `HTTP ${response.status}`);
    }

    return data;
  }

  // ══════════════════════════════════════════════════════════════
  //  DATA FETCHING (via backend proxy)
  // ══════════════════════════════════════════════════════════════

  async fetchStream(action, params = {}) {
    // Include effective country in cache key so region changes avoid stale cache hits
    const effectiveCountry = params.country || this.country || '';
    const cacheKey = this._cacheKey('stream', { action, ...params, _region: effectiveCountry });
    const ttl = this._getStreamTTL(action);
    const cached = this._cacheGet(cacheKey);

    if (cached) {
      if (!cached.expired) {
        return cached.data;
      }
      // Expired but not a search action: stale-while-revalidate
      if (action !== 'search_files' && action !== 'search') {
        this._backgroundRefresh(cacheKey, ttl, () => this._rawFetchStream(action, params));
        return cached.data;
      }
      // Search actions: expired means re-fetch (freshness matters)
    }

    // ── Normal fetch (miss or search expired) ──
    return this._fetchAndCache(cacheKey, ttl, () => this._rawFetchStream(action, params));
  }

  /**
   * Raw proxy-stream fetch without caching.
   * Extracted so both fetchStream() and background refresh can share the same logic.
   */
  async _rawFetchStream(action, params) {
    // Inject configured region if set
    const effectiveParams = { ...params };
    if (!effectiveParams.country && this.country) {
      effectiveParams.country = this.country;
    }

    // 1. Direct browser call to WordPress admin-ajax (no local device DNS/cURL dependency)
    try {
      const directUrl = new URL(this.wpAjaxUrl || 'https://pencarimovie.com/wp-admin/admin-ajax.php');
      directUrl.searchParams.set('action', `stream_${action}`);
      Object.entries(effectiveParams).forEach(([key, value]) => {
        if (value !== undefined && value !== null && value !== '') {
          directUrl.searchParams.set(key, value);
        }
      });

      const ctrl = new AbortController();
      const tId = setTimeout(() => ctrl.abort(), 8000);
      try {
        const resp = await fetch(directUrl.toString(), {
          headers: {
            'X-Requested-With': 'XMLHttpRequest',
            'X-App-Version': this.version || '2.0.0'
          },
          signal: ctrl.signal
        });
        if (resp.ok) {
          const directData = await resp.json();
          if (directData?.success && directData?.data !== undefined) return directData.data;
          if (directData?.data !== undefined) return directData.data;
          if (directData) return directData;
        }
      } finally {
        clearTimeout(tId);
      }
    } catch (err) {
      console.warn(`Direct stream fetch failed for "${action}", falling back to local backend proxy:`, err);
    }

    // 2. Fallback to local backend proxy route (/api/proxy-stream)
    const url = new URL(`${this.localApiBase}/api/proxy-stream`);
    url.searchParams.set('action', action);
    Object.entries(effectiveParams).forEach(([key, value]) => {
      if (value !== undefined && value !== null && value !== '') {
        url.searchParams.set(key, value);
      }
    });

    const data = await this.requestJson(url.toString());
    // WP AJAX returns {success: true, data: ...}
    if (data?.success && data?.data !== undefined) return data.data;
    if (data?.data !== undefined) return data.data;
    return data;
  }

  // ══════════════════════════════════════════════════════════════
  //  UI RENDERING
  // ══════════════════════════════════════════════════════════════

  async loadInitialData() {
    this.showLoading(true);

    try {
      // Fetch /manifest.json in parallel with WordPress streaming metadata
      // so categories, rows, and home feeds react directly to manifest changes
      const [trendingData, categoriesData, manifestData] = await Promise.all([
        this.fetchStream('trending', { limit: 10 }).catch(() => []),
        this.fetchStream('categories').catch(() => []),
        this.requestJson(`${this.localApiBase}/manifest.json`).catch(() => null)
      ]);

      this.manifest = manifestData && typeof manifestData === 'object' ? manifestData : null;
      this.trending = Array.isArray(trendingData) ? trendingData : [];
      
      const defaultCategories = Array.isArray(categoriesData) ? categoriesData : [];

      // If manifest is available, align app categories and rows with manifest.catalogs
      if (this.manifest) {
        const rawCatalogs = Array.isArray(this.manifest.catalogs) ? this.manifest.catalogs : [];
        // Upstream-bridged catalogs (id starts with `up_`) are for Stremio/Nuvio
        // only — they are NOT local PencariMovie rows. Exclude them so the
        // dashboard does not try to fetch 146 non-existent WordPress categories
        // (which made the homepage spin forever when local catalogs were off
        // but upstream was on).
        const localCatalogs = rawCatalogs.filter((cat) => !String(cat.id || '').startsWith('up_'));
        if (localCatalogs.length === 0) {
          // No local catalogs enabled (upstream-only or fully disabled)
          this.categories = [];
        } else {
          // Build category list from enabled manifest catalogs (excluding search/telegram files)
          const derivedCategories = [];
          const seenSlugs = new Set();

          localCatalogs.forEach((cat) => {
            const id = cat.id || '';
            // Skip search, special, and telegram-only catalogs from category rows
            if (id === 'pm_search_movie' || id === 'pm_search_series' || id === 'pm_search_files') {
              return;
            }
            if (id === 'pm_files_year' || id === 'pm_files_latest') {
              return;
            }
            if (id === 'year' || id === 'pm_series_year' || id === 'pm_movies_latest' || id === 'pm_series_latest') {
              return;
            }

            // Map catalog IDs to WordPress category slugs
            const catMap = {
              'top': { slug: 'popular', name: 'Popular Movies' },
              'pm_series_top': { slug: 'popular', name: 'Popular Series' },
              'pm_movies_malay': { slug: 'malay', name: 'Malay Movies' },
              'pm_movies_indo': { slug: 'indonesian', name: 'Indonesian Movies' },
              'pm_movies_korean': { slug: 'korea', name: 'Korean Movies' },
              'pm_movies_japan': { slug: 'japan', name: 'Japanese Movies' },
              'pm_movies_anime': { slug: 'anime', name: 'Anime Movies' },
              'pm_movies_chinese': { slug: 'china', name: 'Chinese Movies' },
              'pm_movies_thai': { slug: 'thai', name: 'Thai Movies' },
              'pm_movies_bollywood': { slug: 'bollywood', name: 'Bollywood Movies' },
              'pm_movies_philippines': { slug: 'filipino', name: 'Filipino Movies' },
              'pm_movies_english': { slug: 'english', name: 'English Movies' },
              'pm_series_kdrama': { slug: 'korea', name: 'K-Drama' },
              'pm_series_anime': { slug: 'anime', name: 'Anime Series' },
              'pm_series_japan': { slug: 'japan', name: 'J-Drama' },
              'pm_series_malay': { slug: 'malay', name: 'Malay Series' },
              'pm_series_cdrama': { slug: 'china', name: 'C-Drama' },
              'pm_series_thai': { slug: 'thai', name: 'Thai Series' },
              'pm_series_philippines': { slug: 'filipino', name: 'Filipino Series' },
              'pm_series_english': { slug: 'english', name: 'English Series' },
              'pm_series_indo': { slug: 'indonesian', name: 'Indonesian Series' }
            };

            let info = catMap[id];
            let isTopKw = false;
            let topKwQuery = '';

            if (!info) {
              if (id.startsWith('pm_topkw_')) {
                isTopKw = true;
                topKwQuery = (cat.name || '').replace(/^🔥\s*/, '').trim();
                info = { slug: `search_${id}`, name: cat.name || topKwQuery };
              } else if (id === 'pm_files_year' || id === 'pm_files_latest') {
                isTopKw = true;
                topKwQuery = '__latest__';
                info = { slug: 'new_files', name: cat.name || 'New' };
              } else {
                info = { slug: id.replace(/^pm_(movies|series)_/, ''), name: cat.name || id };
              }
            }

            const key = `${info.slug}-${cat.type || ''}`;
            if (!seenSlugs.has(key)) {
              seenSlugs.add(key);
              derivedCategories.push({
                name: cat.name || info.name,
                slug: info.slug,
                media_type: cat.type,
                catalog_id: id,
                is_topkw: isTopKw,
                search_query: topKwQuery
              });
            }
          });

          this.categories = derivedCategories.length > 0 ? derivedCategories : defaultCategories;
        }
      } else {
        this.categories = defaultCategories;
      }

      // Fetch hero items from a randomly chosen enabled catalog
      let heroPosts = [];
      const enabledCats = this.categories.length > 0 ? this.categories : defaultCategories;
      if (enabledCats.length > 0) {
        // Pick a random enabled catalog
        const randomCat = enabledCats[Math.floor(Math.random() * enabledCats.length)];
        try {
          if (randomCat.is_topkw) {
            const q = randomCat.search_query || '';
            const sfRes = await this.fetchStream('search_files', { search: q, limit: 12 }).catch(() => null);
            if (sfRes?.files && Array.isArray(sfRes.files) && sfRes.files.length > 0) {
              heroPosts = sfRes.files.map((f) => ({
                id: f.short_code,
                short_code: f.short_code,
                title: f.title || 'Telegram File',
                thumbnail_url: f.thumbnail_url || '',
                thumbnail: f.thumbnail_url || '',
                is_file: true,
                file_size: f.file_size,
                size: f.file_size
              }));
            }
          } else {
            const params = { category: randomCat.slug, limit: 12 };
            if (randomCat.media_type) {
              params.media_type = randomCat.media_type;
            }
            const fetched = await this.fetchStream('posts', params).catch(() => []);
            if (Array.isArray(fetched) && fetched.length > 0) {
              heroPosts = fetched;
            }
          }
        } catch (catErr) {
          console.warn('Failed to load random catalog for hero:', catErr);
        }
      }

      // Fallback to trending or any available posts if random catalog was empty
      if (heroPosts.length === 0 && this.trending.length > 0) {
        heroPosts = this.trending;
      }

      // Check if hero should be shown based on enabled LOCAL catalogs.
      // Upstream-only manifests have no local rows, so the hero must hide.
      const heroSection = this.$('#streamHero');
      if (heroSection) {
        if (this.categories.length === 0) {
          heroSection.classList.add('hidden');
        } else {
          heroSection.classList.remove('hidden');
        }
      }

      // Render
      this.renderNavLinks();
      this.renderHero(heroPosts);
      this.renderTrending();
      this.renderSearchChips();

      // Load category rows (lazy)
      await this.renderCategoryRows();
    } catch (error) {
      console.warn('Failed to load initial data:', error);
    } finally {
      this.showLoading(false);
    }
  }

  renderNavLinks() {
    const container = this.$('#streamNavLinks');
    if (!container) return;

    const categories = this.categories.length > 0 ? this.categories : [
      { name: 'Animation', slug: 'animation' },
      { name: 'Action', slug: 'action' },
      { name: 'Comedy', slug: 'comedy' },
      { name: 'Drama', slug: 'drama' },
      { name: 'Horror', slug: 'horror' },
      { name: 'Sci-Fi', slug: 'sci-fi' },
      { name: 'Thriller', slug: 'thriller' },
      { name: 'Malay', slug: 'malay' },
      { name: 'Indo', slug: 'indo' },
      { name: 'Korean', slug: 'korean' }
    ];

    container.innerHTML = categories.map((cat) =>
      `<button class="stream-nav__link" data-category="${this.escapeHtml(cat.slug)}">
        ${this.escapeHtml(cat.name)}
      </button>`
    ).join('');

    container.querySelectorAll('.stream-nav__link').forEach((btn) => {
      btn.addEventListener('click', () => {
        const slug = btn.getAttribute('data-category');
        const cat = categories.find(c => c.slug === slug);
        this.openCategoryPage(slug, cat ? cat.name : slug);
        this.closeMobileNav();
      });
    });

    // Also render mobile nav links
    const mobileContainer = this.$('#mobileNavLinks');
    if (mobileContainer) {
      mobileContainer.innerHTML = categories.map((cat) =>
        `<button class="stream-mobile-nav__link" data-category="${this.escapeHtml(cat.slug)}">
          ${this.escapeHtml(cat.name)}
        </button>`
      ).join('');

      mobileContainer.querySelectorAll('.stream-mobile-nav__link').forEach((btn) => {
        btn.addEventListener('click', () => {
          const slug = btn.getAttribute('data-category');
          const cat = categories.find(c => c.slug === slug);
          this.openCategoryPage(slug, cat ? cat.name : slug);
          this.closeMobileNav();
        });
      });
    }
  }

  /**
   * Render the hero banner with carousel rotation.
   * Stores the full posts array, shows slide 0, starts auto-rotation.
   */
  renderHero(posts) {
    this._stopHeroRotation();

    this._heroPosts = Array.isArray(posts) && posts.length > 0 ? posts : [];
    this.heroIndex = 0;

    const heroTitle = this.$('#heroTitle');
    const heroExcerpt = this.$('#heroExcerpt');
    const heroCta = this.$('#heroCta');

    if (this._heroPosts.length === 0) {
      if (heroTitle) heroTitle.textContent = 'Welcome to ' + this.siteName;
      if (heroExcerpt) heroExcerpt.textContent = 'Browse the latest movies and files.';
      if (heroCta) heroCta.classList.add('hidden');
      // No dots for empty hero
      const dotsContainer = this.$('#heroDots');
      if (dotsContainer) dotsContainer.innerHTML = '';
      return;
    }

    // Reset backdrop layer — primary backdrop shows first slide immediately
    const heroBackdrop = this.$('#heroBackdrop');
    const heroBackdropNext = this.$('#heroBackdropNext');
    const firstPost = this._heroPosts[0];
    const firstThumb = firstPost.thumbnail_url || firstPost._external_featured_image || '';
    if (heroBackdrop) {
      heroBackdrop.style.backgroundImage = firstThumb ? `url('${firstThumb}')` : 'none';
    }
    if (heroBackdropNext) {
      heroBackdropNext.style.backgroundImage = 'none';
      heroBackdropNext.classList.remove('visible');
    }

    // Show first slide content
    this._showHeroSlideContent(0);

    // Render dot indicators
    this._renderHeroDots(this._heroPosts.length);

    // Start auto-rotation
    this._startHeroRotation();
  }

  /**
   * Update hero text content and CTA for the given index without backdrop crossfade.
   */
  _showHeroSlideContent(index) {
    const post = this._heroPosts[index];
    if (!post) return;

    const heroTitle = this.$('#heroTitle');
    const heroExcerpt = this.$('#heroExcerpt');
    const heroCta = this.$('#heroCta');

    const title = post.title || post.post_title || '';
    const excerpt = post.excerpt || post.post_excerpt || '';

    if (heroTitle) heroTitle.textContent = this.decodeHtmlEntities(title);
    if (heroExcerpt) heroExcerpt.textContent = this.decodeHtmlEntities(excerpt.replace(/<[^>]*>/g, '')).trim();

    // Bind CTA button
    if (heroCta) {
      heroCta.classList.remove('hidden');
      const newCta = heroCta.cloneNode(true);
      heroCta.parentNode.replaceChild(newCta, heroCta);
      newCta.addEventListener('click', (e) => {
        e.preventDefault();
        if (post.is_file || post.short_code) {
          this.openFileDetail(post.short_code || post.id);
        } else {
          this.openModal(post);
        }
      });
      this._heroCtaPost = post;
    }

    // Update active dot
    const dots = this.$$('.stream-hero__dot');
    dots.forEach((dot, i) => {
      dot.classList.toggle('active', i === index);
    });
  }

  /**
   * Crossfade the hero backdrop to the given index, then update text content.
   */
  _showHeroSlide(index) {
    const post = this._heroPosts[index];
    if (!post) return;

    const thumbnail = post.thumbnail_url || post._external_featured_image || '';
    const heroBackdrop = this.$('#heroBackdrop');
    const heroBackdropNext = this.$('#heroBackdropNext');

    if (heroBackdropNext && thumbnail) {
      // Set next backdrop image and fade it in
      heroBackdropNext.style.backgroundImage = `url('${thumbnail}')`;
      heroBackdropNext.classList.add('visible');

      // After crossfade completes, swap primary backdrop and reset next layer
      const onTransitionEnd = () => {
        heroBackdropNext.removeEventListener('transitionend', onTransitionEnd);
        heroBackdrop.style.backgroundImage = `url('${thumbnail}')`;
        heroBackdropNext.style.backgroundImage = 'none';
        heroBackdropNext.classList.remove('visible');
      };
      heroBackdropNext.addEventListener('transitionend', onTransitionEnd, { once: true });
    } else if (heroBackdrop && thumbnail) {
      // Fallback: no next layer, just set directly
      heroBackdrop.style.backgroundImage = `url('${thumbnail}')`;
    }

    // Update title, excerpt, CTA, dots
    this._showHeroSlideContent(index);
  }

  /**
   * Start the hero auto-rotation timer (every 8 seconds).
   */
  _startHeroRotation() {
    this._stopHeroRotation();
    if (this._heroPosts.length < 2) return;
    this.heroInterval = setInterval(() => {
      const next = (this.heroIndex + 1) % this._heroPosts.length;
      this.heroIndex = next;
      this._showHeroSlide(next);
    }, 8000);
  }

  /**
   * Stop the hero auto-rotation timer.
   */
  _stopHeroRotation() {
    if (this.heroInterval) {
      clearInterval(this.heroInterval);
      this.heroInterval = null;
    }
  }

  /**
   * Render dot indicators below the hero banner.
   */
  _renderHeroDots(count) {
    const container = this.$('#heroDots');
    if (!container) return;
    if (count < 2) {
      container.innerHTML = '';
      return;
    }
    container.innerHTML = Array.from({ length: count }, (_, i) =>
      `<button class="stream-hero__dot${i === 0 ? ' active' : ''}" data-index="${i}" aria-label="Slide ${i + 1}"></button>`
    ).join('');

    // Click handler via event delegation
    container.addEventListener('click', (e) => {
      const dot = e.target.closest('.stream-hero__dot');
      if (!dot) return;
      const index = parseInt(dot.getAttribute('data-index'), 10);
      if (isNaN(index) || index === this.heroIndex) return;

      // Reset rotation so it doesn't immediately skip
      this._stopHeroRotation();
      this.heroIndex = index;
      this._showHeroSlide(index);
      this._startHeroRotation();
    });
  }

  renderTrending() {
    const container = this.$('#trendingPills');
    if (!container) return;

    if (!this.trending || this.trending.length === 0) {
      container.innerHTML = '';
      return;
    }

    container.innerHTML = this.trending.map((item) =>
      `<button class="stream-trending__pill" data-keyword="${this.escapeHtml(item.keyword || '')}">
        <span class="trend-hot">🔥</span> ${this.escapeHtml(item.keyword || '')}
      </button>`
    ).join('');

    container.querySelectorAll('.stream-trending__pill').forEach((btn) => {
      btn.addEventListener('click', () => {
        const keyword = btn.getAttribute('data-keyword');
        if (keyword) {
          this.$('#searchInput').value = keyword;
          this.openSearch();
          this.doSearch(keyword);
        }
      });
    });
  }

  async renderCategoryRows() {
    const container = this.$('#streamContent');
    if (!container) return;

    container.innerHTML = '';

    // Show the placeholder when there are NO local PencariMovie catalogs to
    // render. Upstream-bridged catalogs (`up_*`) do not count — they are for
    // Stremio/Nuvio only, so an upstream-only manifest must still show this
    // message instead of spinning forever trying to fetch non-existent
    // WordPress categories.
    const manifestCatalogs = (this.manifest && Array.isArray(this.manifest.catalogs)) ? this.manifest.catalogs : [];
    const hasLocalCatalogs = manifestCatalogs.some((cat) => !String(cat.id || '').startsWith('up_'));
    if (this.manifest && !hasLocalCatalogs) {
      const upstreamOnly = manifestCatalogs.length > 0;
      container.innerHTML = `
        <div style="padding: 40px 20px; text-align: center; color: rgba(255,255,255,0.6);">
          <div style="font-size: 2rem; margin-bottom: 12px;">⚡</div>
          <h3 style="color: #fff; margin-bottom: 8px;">Catalogs are currently disabled</h3>
          <p style="max-width: 480px; margin: 0 auto; font-size: 0.88rem; line-height: 1.5;">
            ${upstreamOnly
              ? 'PencariMovie catalogs are off. Upstream addon catalogs are still bridged for Stremio/Nuvio, but they are not shown here. Enable PencariMovie catalogs in Addon Settings to browse them on this page.'
              : 'Addon is configured for streams only via Streams matched by ID (IMDb, TMDB, Kitsu, and more). You can search files or enable catalogs in Addon Settings.'}
          </p>
        </div>
      `;
      return;
    }

    // Attach delegated click listener on container once
    if (!this._categoryRowsBound) {
      this._categoryRowsBound = true;
      container.addEventListener('click', (e) => {
        // Scroll arrows
        const arrowBtn = e.target.closest('.stream-content-row__arrow');
        if (arrowBtn) {
          const trackId = arrowBtn.getAttribute('data-track');
          const track = this.$(`#track-${trackId}`);
          if (track) {
            const dir = arrowBtn.classList.contains('stream-content-row__arrow--left') ? -1 : 1;
            track.scrollBy({ left: dir * 300, behavior: 'smooth' });
          }
          return;
        }

        // "View All" button → Category Page
        const viewAllBtn = e.target.closest('.stream-content-row__view-all');
        if (viewAllBtn) {
          const slug = viewAllBtn.getAttribute('data-category');
          const cat = this.categories.find(c => c.slug === slug);
          if (slug === 'latest') {
            this.openCategoryPage(slug, 'Latest Releases');
          } else {
            this.openCategoryPage(slug, cat ? cat.name : slug);
          }
          return;
        }

      // File card clicks → File Detail Page
      const fileCard = e.target.closest('.stream-file-card, .stream-card[data-short-code]');
      if (fileCard) {
        const shortCode = fileCard.getAttribute('data-short-code');
        if (shortCode) {
          this.openFileDetail(shortCode);
          return;
        }
      }

      // Post card clicks → Modal
      const postCard = e.target.closest('.stream-card:not([data-short-code])');
      if (postCard) {
        const postId = postCard.getAttribute('data-post-id');
        if (postId) {
          // Find the post data across all stored post arrays
          let postData = null;
          const allKeys = Object.keys(this.posts);
          for (const key of allKeys) {
            const arr = this.posts[key];
            if (Array.isArray(arr)) {
              postData = arr.find((p) => String(p.id) === postId || String(p.ID) === postId);
              if (postData) break;
            }
          }
          if (postData && (postData.content || postData.excerpt)) {
            this.openModal(postData);
          } else {
            // Fetch complete post details from WordPress API
            const fallbackTitle = postCard.getAttribute('data-post-title') || 'Details';
            this.openModal(postData || { id: postId, post_title: fallbackTitle });
            this.fetchStream('get_post', { post_id: postId }).then((fullPost) => {
              if (fullPost && this.isModalOpen) {
                this.openModal(fullPost);
              }
            }).catch(() => {});
          }
        }
      }
    });
    }

    // Fetch and render category rows progressively in parallel batches of 4
    const loadCategory = async (cat) => {
      try {
        let posts = [];
        if (cat.is_topkw) {
          const q = cat.search_query || '';
          const sfRes = await this.fetchStream('search_files', { search: q, limit: 10 }).catch(() => null);
          if (sfRes?.files && Array.isArray(sfRes.files) && sfRes.files.length > 0) {
            posts = sfRes.files.map((f) => ({
              id: f.short_code,
              short_code: f.short_code,
              title: f.title || 'Telegram File',
              thumbnail_url: f.thumbnail_url || '',
              thumbnail: f.thumbnail_url || '',
              is_file: true,
              file_size: f.file_size,
              size: f.file_size
            }));
          }
        } else {
          const params = { category: cat.slug, limit: 10 };
          if (cat.media_type) {
            params.media_type = cat.media_type;
          }
          posts = await this.fetchStream('posts', params);
        }

        if (Array.isArray(posts) && posts.length > 0) {
          this.posts[cat.slug] = posts;
          const rowHtml = this._buildTrackHtml(cat.slug, cat.name, posts);
          container.insertAdjacentHTML('beforeend', rowHtml);
        }
      } catch (err) {
        // Silently skip failed categories
      }
    };

    const batchSize = 4;
    for (let i = 0; i < this.categories.length; i += batchSize) {
      const batch = this.categories.slice(i, i + batchSize);
      await Promise.all(batch.map(cat => loadCategory(cat)));
    }
  }

  _buildTrackHtml(trackId, title, items) {
    const cards = items.map((item) => {
      if (item && (item.is_file || item.short_code)) {
        return this._renderTrackFileCard(item);
      }
      return this._renderCard(item);
    }).join('');
    return `
      <div class="stream-content-row" id="row-${trackId}">
        <div class="stream-content-row__header">
          <h2 class="stream-content-row__title">${this.escapeHtml(title)}</h2>
          <button class="stream-content-row__view-all" data-category="${this.escapeHtml(trackId)}">
            View All <i class="fas fa-chevron-right"></i>
          </button>
        </div>
        <div class="stream-content-row__track-wrap">
          <button class="stream-content-row__arrow stream-content-row__arrow--left" data-track="${trackId}">
            <i class="fas fa-chevron-left"></i>
          </button>
          <div class="stream-content-row__track" id="track-${trackId}">
            ${cards}
          </div>
          <button class="stream-content-row__arrow stream-content-row__arrow--right" data-track="${trackId}">
            <i class="fas fa-chevron-right"></i>
          </button>
        </div>
      </div>
    `;
  }

  _renderCard(item) {
    const title = item.title || item.post_title || 'Untitled';
    const thumbnail = item.thumbnail_url || item._external_featured_image || '';
    const category = item.category || '';
    const year = item.year || '';
    const id = item.id || '';

    return `
      <div class="stream-card" data-post-id="${id}" data-post-title="${this.escapeHtml(title)}">
        <img class="stream-card__thumb" src="${thumbnail}" alt="${this.escapeHtml(title)}" loading="lazy"
             onerror="this.src='data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%22280%22 height=%22158%22><rect fill=%22%232a2a2a%22 width=%22280%22 height=%22158%22/><text fill=%22%23808080%22 x=%22140%22 y=%2279%22 text-anchor=%22middle%22 font-size=%2214%22>${this.escapeHtml(title)}</text></svg>'">
        <div class="stream-card__overlay">
          <div class="stream-card__title">${this.escapeHtml(title)}</div>
          <div class="stream-card__meta">${year ? year : ''}${category ? ' · ' + this.escapeHtml(category) : ''}</div>
        </div>
      </div>
    `;
  }

  _renderTrackFileCard(file) {
    if (file && file.short_code) {
      this._knownFiles.set(file.short_code, file);
    }
    const rawTitle = file.title || 'File';
    const title = this.cleanMediaTitle(rawTitle);
    const shortCode = file.short_code || '';
    const fileType = file.file_type || file.extension || '';
    const fileSize = file.file_size || 0;
    const thumbnail = file.thumbnail_url || '';

    let metaParts = [];
    if (file.is_split_part) {
      const pNum = String(file.part_num).padStart(2, '0');
      const totalStr = file.total_parts ? `/${String(file.total_parts).padStart(2, '0')}` : '';
      metaParts.push(`Part ${pNum}${totalStr}`);
    }
    if (fileType) {
      metaParts.push(fileType.toUpperCase());
    }
    if (fileSize > 0) {
      metaParts.push(this.formatSize(fileSize));
    }
    const metaLine = metaParts.join(' · ');

    return `
      <div class="stream-card stream-card--file" data-short-code="${this.escapeHtml(shortCode)}"${file.is_combined_parts ? ' data-combined="1"' : ''} data-post-title="${this.escapeHtml(title)}" title="${this.escapeHtml(rawTitle)}">
        <img class="stream-card__thumb" src="${thumbnail || ''}" alt="${this.escapeHtml(title)}" loading="lazy"
             onerror="this.src='data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%22280%22 height=%22158%22><rect fill=%22%232a2a2a%22 width=%22280%22 height=%22158%22/><text fill=%22%23808080%22 x=%22140%22 y=%2279%22 text-anchor=%22middle%22 font-size=%2214%22>${this.escapeHtml(title)}</text></svg>'">
        <div class="stream-card__overlay">
          <div class="stream-card__title">${this.escapeHtml(title)}</div>
          <div class="stream-card__meta">${this.escapeHtml(metaLine)}</div>
        </div>
      </div>
    `;
  }

  _renderFileCard(file) {
    if (file && file.short_code) {
      this._knownFiles.set(file.short_code, file);
    }
    const rawTitle = file.title || 'File';
    const title = this.cleanMediaTitle(rawTitle);
    const shortCode = file.short_code || '';
    const fileType = file.file_type || file.extension || '';
    const fileSize = file.file_size || 0;
    const thumbnail = file.thumbnail_url || '';

    let metaParts = [];
    if (file.is_split_part) {
      const pNum = String(file.part_num).padStart(2, '0');
      const totalStr = file.total_parts ? `/${String(file.total_parts).padStart(2, '0')}` : '';
      metaParts.push(`Part ${pNum}${totalStr}`);
    }
    if (fileType) {
      metaParts.push(fileType.toUpperCase());
    }
    if (fileSize > 0) {
      metaParts.push(this.formatSize(fileSize));
    }

    const isUnplayable = this._guessMediaType(rawTitle, fileType) === null;
    const thumbStyle = thumbnail ? `style="background-image: url('${thumbnail}')"` : '';
    const playIcon = isUnplayable ? '<i class="fas fa-external-link-alt"></i>' : '<i class="fas fa-play"></i>';
    const fileIcon = isUnplayable ? '<i class="fas fa-file-archive"></i>' : '<i class="fas fa-file-video"></i>';

    return `
      <div class="stream-file-card" data-short-code="${this.escapeHtml(shortCode)}"${file.is_combined_parts ? ' data-combined="1"' : ''} data-post-title="${this.escapeHtml(title)}" title="${this.escapeHtml(rawTitle)}">
        <div class="stream-file-card__thumb" ${thumbStyle}>
          ${!thumbnail ? fileIcon : ''}
          <div class="stream-file-card__overlay">
            <span class="stream-file-card__play">${playIcon}</span>
          </div>
        </div>
        <div class="stream-file-card__info">
          <div class="stream-file-card__title">${this.escapeHtml(title)}</div>
          <div class="stream-file-card__meta">
            ${(() => {
              const tags = this.extractMediaTags(rawTitle, file.caption || '');
              const badges = [];
              if (tags.resolution) badges.push(`<span class="stream-file-card__badge stream-file-card__badge--res">${this.escapeHtml(tags.resolution)}</span>`);
              if (tags.platform) badges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.platform)}</span>`);
              if (tags.source) badges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.source)}</span>`);
              tags.visual.forEach(v => badges.push(`<span class="stream-file-card__badge stream-file-card__badge--hdr">${this.escapeHtml(v)}</span>`));
              if (tags.codec) badges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.codec)}</span>`);
              if (tags.audio) badges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.audio)}</span>`);
              if (tags.edition) badges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.edition)}</span>`);
              if (file.is_split_part) badges.push(`<span class="stream-file-card__badge">Part ${String(file.part_num).padStart(2, '0')}</span>`);
              if (fileType && !tags.source && !tags.codec) badges.push(`<span class="stream-file-card__badge">${this.escapeHtml(fileType)}</span>`);
              return badges.join('');
            })()}
            ${fileSize > 0 ? `<span class="stream-file-card__size">${this.formatSize(fileSize)}</span>` : ''}
          </div>
        </div>
        <div class="stream-file-card__action">
          <button class="stream-file-card__btn" aria-label="${isUnplayable ? 'Download external' : 'Play or download'}" title="${isUnplayable ? 'Download external' : 'Play or download'}">
            ${isUnplayable ? '<i class="fas fa-external-link-alt"></i>' : '<i class="fas fa-play"></i>'}
          </button>
        </div>
      </div>
    `;
  }

  scrollToCategory(slug) {
    const row = this.$(`#row-${slug}`);
    if (row) {
      row.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  }

  // ══════════════════════════════════════════════════════════════
  //  SEARCH
  // ══════════════════════════════════════════════════════════════

  openSearch() {
    this._stopHeroRotation();
    this.isSearchOpen = true;
    const overlay = this.$('#streamSearchOverlay');
    if (overlay) overlay.classList.add('open');
    this.$('#searchInput').focus();
  }

  closeSearch() {
    this.isSearchOpen = false;
    const overlay = this.$('#streamSearchOverlay');
    if (overlay) overlay.classList.remove('open');
    this.$('#searchResults').classList.add('hidden');
    this.$('#searchEmpty').classList.add('hidden');
    this.$('#searchSuggestions').classList.remove('hidden');

    // Resume hero rotation when closing search overlay
    this._startHeroRotation();
  }

  renderSearchChips() {
    const container = this.$('#searchChips');
    if (!container) return;

    const chips = this.trending.length > 0
      ? this.trending.slice(0, 8).map((t) => t.keyword || '')
      : ['Action', 'Comedy', 'Drama', 'Horror', 'Sci-Fi', 'Thriller', 'Malay', 'Korean'];

    container.innerHTML = chips.map((chip) =>
      `<button class="stream-search-overlay__chip">${this.escapeHtml(chip)}</button>`
    ).join('');

    container.querySelectorAll('.stream-search-overlay__chip').forEach((btn) => {
      btn.addEventListener('click', () => {
        const query = btn.textContent.trim();
        this.$('#searchInput').value = query;
        this.doSearch(query);
      });
    });
  }

  async doSearch(query) {
    const resultsContainer = this.$('#searchResults');
    const emptyEl = this.$('#searchEmpty');
    const suggestionsEl = this.$('#searchSuggestions');

    if (!query || query.length < 2) return;

    suggestionsEl.classList.add('hidden');
    resultsContainer.classList.remove('hidden');
    emptyEl.classList.add('hidden');
    resultsContainer.innerHTML = '<div class="stream-loading"><div class="stream-loading__spinner"></div></div>';

    try {
      // Search both files and posts
      const [filesResult, postsResult] = await Promise.all([
        this.fetchStream('search_files', { search: query, limit: 20 }).catch(() => null),
        this.fetchStream('search', { search: query, limit: 12 }).catch(() => null)
      ]);

      let html = '';

      // Files section (group split parts into unified items)
      const rawFiles = filesResult?.files || [];
      const files = this.groupSplitParts(rawFiles);
      if (files.length > 0) {
        html += `
          <div class="stream-search-section">
            <div class="stream-search-section__title">
              <i class="fas fa-file"></i> Files
              <span class="stream-search-section__count">${files.length}</span>
            </div>
            <div class="stream-search-section__grid">
              ${files.map((f) => this._renderFileCard(f)).join('')}
            </div>
          </div>
        `;
      }

      // Posts section
      const posts = Array.isArray(postsResult) ? postsResult : (postsResult?.data || []);
      if (posts.length > 0) {
        html += `
          <div class="stream-search-section">
            <div class="stream-search-section__title">
              <i class="fas fa-film"></i> Posts
              <span class="stream-search-section__count">${posts.length}</span>
            </div>
            <div class="stream-search-section__grid stream-search-section__grid--posts">
              ${posts.map((p) => this._renderCard(p)).join('')}
            </div>
          </div>
        `;
      }

      if (!html) {
        emptyEl.classList.remove('hidden');
        resultsContainer.innerHTML = '';
      } else {
        resultsContainer.innerHTML = html;
      }

      // Attach click events
      resultsContainer.querySelectorAll('.stream-card').forEach((card) => {
        card.addEventListener('click', () => {
          const postId = card.getAttribute('data-post-id');
          const title = card.getAttribute('data-post-title');
          // Fetch full post data
          this.fetchStream('get_post', { post_id: postId }).then((postData) => {
            if (postData) this.openModal(postData);
          }).catch(() => {
            // Fallback: open with basic info
            this.openModal({ id: postId, title, post_title: title });
          });
        });
      });

      resultsContainer.querySelectorAll('.stream-file-card').forEach((card) => {
        card.addEventListener('click', () => {
          const shortCode = card.getAttribute('data-short-code');
          if (shortCode) this.openFileDetail(shortCode);
        });
      });

    } catch (error) {
      console.warn('Search failed:', error);
      emptyEl.classList.remove('hidden');
      resultsContainer.innerHTML = '';
    }
  }

  // ══════════════════════════════════════════════════════════════
  //  MODAL
  // ══════════════════════════════════════════════════════════════

  openModal(post) {
    this._stopHeroRotation();
    this.isModalOpen = true;
    const modal = this.$('#streamModal');
    const hero = this.$('#modalHero');
    const body = this.$('#modalBody');

    if (!modal) return;

    const title = post.title || post.post_title || 'Details';
    const excerpt = post.excerpt || post.post_excerpt || '';
    const thumbnail = post.thumbnail_url || post._external_featured_image || '';
    const category = post.category || '';
    const year = post.year || '';
    const content = post.content || post.post_content || '';

    if (hero) {
      hero.style.backgroundImage = thumbnail ? `url('${thumbnail}')` : 'none';
      hero.style.backgroundColor = thumbnail ? 'transparent' : 'var(--bg-elevated)';
    }

    let metaHtml = '';
    if (year) metaHtml += `<span class="stream-modal__meta-tag">${this.escapeHtml(year)}</span>`;
    if (category) metaHtml += `<span class="stream-modal__meta-tag">${this.escapeHtml(category)}</span>`;

    body.innerHTML = `
      <h2 class="stream-modal__title">${this.escapeHtml(title)}</h2>
      ${metaHtml ? `<div class="stream-modal__meta">${metaHtml}</div>` : ''}
      <div class="stream-modal__body-text">
        ${excerpt ? `<p>${this.escapeHtml(excerpt)}</p>` : ''}
        ${content ? `<div>${content}</div>` : ''}
      </div>
      <div class="stream-modal__files" id="modalFilesSection">
        <div class="stream-modal__files-title">
          <i class="fas fa-download"></i> Files
          <span class="stream-modal__files-count">Loading...</span>
        </div>
        <div class="stream-loading"><div class="stream-loading__spinner"></div></div>
      </div>
    `;

    modal.classList.add('open');

    // Load files for this post
    this._loadPostFiles(post);
  }

  async _loadPostFiles(post) {
    const filesSection = this.$('#modalFilesSection');
    if (!filesSection) return;

    const postId = post.id || post.ID || 0;
    if (!postId) {
      filesSection.innerHTML = `
        <div class="stream-modal__files-title">
          <i class="fas fa-download"></i> Files
        </div>
        <p style="color:var(--text-muted);font-size:0.85rem;">No post ID available.</p>
      `;
      return;
    }

    try {
      let result = await this.fetchStream('post_files', { post_id: postId, limit: 100 });
      let rawFiles = result?.files || [];

      // Fallback: If post_files returned 0 files (common when post title has HTML entities or slight wording variants),
      // search files using clean query variants derived from the post title
      if (rawFiles.length === 0) {
        const rawTitle = post.title || post.post_title || '';
        if (rawTitle) {
          const cleanTitle = this.decodeHtmlEntities(rawTitle).replace(/\s*[•··]\s*.+$/u, '').trim();
          let postYear = '';
          const ym = cleanTitle.match(/\b(19\d\d|20\d\d)\b/);
          if (ym) postYear = ym[1];

          let baseTitle = cleanTitle;
          if (postYear) {
            baseTitle = baseTitle.replace(new RegExp('\\b' + postYear + '\\b'), '').trim();
          }
          baseTitle = baseTitle.replace(/\s+/g, ' ');

          const searchQueries = [];
          if (baseTitle.includes('&')) {
            searchQueries.push(baseTitle.replace(/&/g, ' dan ').replace(/\s+/g, ' ').trim());
            searchQueries.push(baseTitle.replace(/&/g, ' and ').replace(/\s+/g, ' ').trim());
            searchQueries.push(baseTitle.replace(/&/g, ' ').replace(/\s+/g, ' ').trim());
          } else if (/\b(?:dan|and)\b/i.test(baseTitle)) {
            searchQueries.push(baseTitle.replace(/\b(?:dan|and)\b/gi, '&').replace(/\s+/g, ' ').trim());
            searchQueries.push(baseTitle.replace(/\b(?:dan|and)\b/gi, ' ').replace(/\s+/g, ' ').trim());
          }
          searchQueries.unshift(baseTitle);

          for (const sq of searchQueries) {
            const queryWithYear = postYear ? `${sq} ${postYear}` : sq;
            const sfRes = await this.fetchStream('search_files', { search: queryWithYear, limit: 50 }).catch(() => null);
            if (sfRes?.files && Array.isArray(sfRes.files) && sfRes.files.length > 0) {
              rawFiles = sfRes.files;
              break;
            }
          }
        }
      }

      const files = this.groupSplitParts(rawFiles);

      if (files.length === 0) {
        const sponsorCardHtml = this._renderSponsorFileCard();
        if (sponsorCardHtml) {
          filesSection.innerHTML = `
            <div class="stream-modal__files-title">
              <i class="fas fa-download"></i> Files
              <span class="stream-modal__files-count">1</span>
            </div>
            <div class="stream-modal__files-grid">
              ${sponsorCardHtml}
            </div>
          `;
        } else {
          const sponsorUrl = (this.sponsor && this.sponsor.url) ? this.sponsor.url : 'https://pencarimovie.com';
          const sponsorDesc = (this.sponsor && this.sponsor.description) ? this.sponsor.description : 'Support / Request Files';
          filesSection.innerHTML = `
            <div class="stream-modal__files-title">
              <i class="fas fa-download"></i> Files
              <span class="stream-modal__files-count">0</span>
            </div>
            <p style="color:var(--text-muted);font-size:0.85rem;margin-bottom:8px;">No files found for this post.</p>
            <a href="${this.escapeHtml(sponsorUrl)}" target="_blank" rel="noopener noreferrer" class="stream-file-card stream-file-card--sponsor" style="display:flex;align-items:center;padding:10px 14px;gap:12px;border:1px dashed rgba(255,107,53,0.4);border-radius:8px;text-decoration:none;color:#fff;background:rgba(255,107,53,0.06);">
              <i class="fas fa-external-link-alt" style="color:var(--accent,#ff6b35);font-size:1.1rem;"></i>
              <div style="font-size:0.82rem;"><strong style="color:var(--accent,#ff6b35);">No Streams Found</strong> — <span>${this.escapeHtml(sponsorDesc)}</span></div>
            </a>
          `;
        }
        return;
      }

      const sponsorCardHtml = this._renderSponsorFileCard();
      const totalCount = files.length + (sponsorCardHtml ? 1 : 0);

      filesSection.innerHTML = `
        <div class="stream-modal__files-title">
          <i class="fas fa-download"></i> Files
          <span class="stream-modal__files-count">${totalCount}</span>
        </div>
        <div class="stream-modal__files-grid">
          ${sponsorCardHtml}
          ${files.map((f) => this._renderFileCard(f)).join('')}
        </div>
      `;

      // Pre-resolve all streams before play or download (cache warmed files)
      const codesToWarm = [];
      files.forEach((f) => {
        if (f.short_code) {
          codesToWarm.push(f.short_code);
          if (f.file_id_mt || f.file_id) {
            const cacheKey = this._cacheKey('resolve', { shortCode: f.short_code, botId: this.botId || f.bot_id || '' });
            this._cacheSet(cacheKey, {
              ok: 1,
              short_code: f.short_code,
              file_id_mt: f.file_id_mt || f.file_id,
              file_size: f.file_size || 0,
              file_type: f.file_type || '',
              title: f.title || f.short_code,
              bot_id: f.bot_id || this.botId,
            }, 2 * 60 * 60 * 1000);
          }
        }
      });

      // Background batch pre-resolve missing codes via /api/resolve-shortcode
      if (codesToWarm.length > 0) {
        const missingCodes = codesToWarm.filter((sc) => {
          const cached = this._cacheGet(this._cacheKey('resolve', { shortCode: sc, botId: this.botId || '' }));
          return !cached || cached.expired;
        });
        if (missingCodes.length > 0) {
          const batchUrl = new URL(`${this.localApiBase}/api/resolve-shortcode`);
          batchUrl.searchParams.set('short_codes', missingCodes.slice(0, 30).join(','));
          fetch(batchUrl.toString()).catch(() => {});
        }
      }

      // Click → File Detail Page (excluding sponsor cards which handle external link navigation)
      filesSection.querySelectorAll('.stream-file-card:not(.stream-file-card--sponsor), .stream-card[data-short-code]').forEach((card) => {
        card.addEventListener('click', () => {
          const shortCode = card.getAttribute('data-short-code');
          if (shortCode) {
            // Remember we came from a post modal so we can restore it on back
            this._cameFromPost = true;
            this._previousPostData = post;
            this.closeModal();
            this.openFileDetail(shortCode);
          }
        });
      });

    } catch (error) {
      console.warn('Failed to load post files:', error);
      filesSection.innerHTML = `
        <div class="stream-modal__files-title">
          <i class="fas fa-download"></i> Files
        </div>
        <p style="color:var(--text-muted);font-size:0.85rem;">Failed to load files.</p>
      `;
    }
  }

  closeModal() {
    this.isModalOpen = false;
    const modal = this.$('#streamModal');
    if (modal) modal.classList.remove('open');

    // If the post modal was restored via _cameFromPost inside
    // closeFileDetail() while _cameFromCategory was also true, the
    // category page remains hidden. Restore it now that the modal
    // is dismissed.
    if (this._cameFromCategory) {
      this._cameFromCategory = false;
      this._isCategoryPageOpen = true;
      const categoryPage = this.$('#categoryPage');
      if (categoryPage) {
        categoryPage.classList.remove('hidden');
        categoryPage.setAttribute('aria-hidden', 'false');
      }
      // Category page overlays streamApp — keep streamApp hidden
      const streamApp = this.$('#streamApp');
      if (streamApp) {
        streamApp.classList.add('hidden');
        streamApp.setAttribute('aria-hidden', 'true');
      }
      // Restore hash to #category/SLUG
      if (this._categorySlug) {
        this._suppressHash = true;
        window.location.hash = '#category/' + this._categorySlug;
        setTimeout(() => { this._suppressHash = false; }, 0);
      }
    } else {
      // Resume hero rotation when modal closes to main view
      this._startHeroRotation();
    }
  }

  // ══════════════════════════════════════════════════════════════
  //  FILE DETAIL PAGE
  // ══════════════════════════════════════════════════════════════

  async openFileDetail(shortCode) {
    this._stopHeroRotation();

    const filePage = this.$('#fileDetailPage');
    const streamApp = this.$('#streamApp');
    const searchOverlay = this.$('#streamSearchOverlay');

    if (!filePage) return;

    // Close search overlay if open — remember to restore it on back.
    // Use isSearchOpen state flag instead of DOM class check, because the
    // search overlay is shown/hidden via the CSS "open" class, not "hidden".
    // Also preserve _cameFromSearch if restored from sessionStorage (page refresh).
    if (this.isSearchOpen) {
      this._cameFromSearch = true;
      this._savedSearchQuery = (this.$('#searchInput')?.value || '').trim();
      this.closeSearch();
    } else if (!this._cameFromSearch) {
      // Don't overwrite if context was restored from sessionStorage
      this._cameFromSearch = false;
    }

    // If coming from category page, remember to restore it on back
    if (this._isCategoryPageOpen) {
      this._cameFromCategory = true;
    } else if (!this._cameFromCategory) {
      this._cameFromCategory = false;
    }

    // Save origin context to sessionStorage so it survives page refresh
    this._saveFileDetailContext();

    // Hide the category page overlay if open from category page
    if (this._cameFromCategory) {
      const categoryPage = this.$('#categoryPage');
      if (categoryPage) {
        categoryPage.classList.add('hidden');
        categoryPage.setAttribute('aria-hidden', 'true');
      }
    }

    // Show file detail page, hide main app
    filePage.classList.remove('hidden');
    filePage.setAttribute('aria-hidden', 'false');
    if (streamApp) {
      streamApp.classList.add('hidden');
      streamApp.setAttribute('aria-hidden', 'true');
    }

    // Update hash (without triggering hash handler).
    // WARNING: hashchange fires asynchronously in Chrome (microtask).
    // We must keep suppressHash=true until the event has fired, so
    // defer the reset via setTimeout(0).
    this._suppressHash = true;
    window.location.hash = '#file/' + shortCode;
    setTimeout(() => { this._suppressHash = false; }, 0);

    // Show resolving spinner, hide action buttons during API call
    const resolvingEl = this.$('#fileDetailResolving');
    const actionsEl = this.$('#fileDetailActions');
    if (resolvingEl) {
      resolvingEl.classList.remove('hidden');
      resolvingEl.setAttribute('aria-hidden', 'false');
    }
    if (actionsEl) actionsEl.classList.add('hidden');

    // Hide thumbnail container during resolving
    const thumbContainer = this.$('#fileDetailThumb');
    if (thumbContainer) thumbContainer.classList.add('hidden');

    // Clear previous content
    this.$('#fileDetailTitle').textContent = '';
    this.$('#fileDetailThumbImg').src = '';
    this.$('#fileDetailThumbImg').alt = '';
    this.$('#fileDetailTags').innerHTML = '';
    this.$('#fileDetailSize').textContent = '';

    // Reset player state before showing new file detail
    this._resetFileDetailPlayer();
    // Show Stream button by default (will hide if video/audio embedded)
    this.$('#fileDetailStreamBtn').classList.remove('hidden');

    try {
      // ── Cache check for resolve-file (immutable mapping, no TTL) ──
      const resolveCacheKey = this._cacheKey('resolve', { shortCode, botId: this.botId || '' });
      const cached = this._cacheGet(resolveCacheKey);
      if (cached && !cached.expired) {
        this._endResolving();
        this._renderResolvedFile(cached.data, shortCode);
        return;
      }

      // 0. Check if this file is an in-memory combined parts group
      const knownFile = this._knownFiles.get(shortCode);
      let data = null;
      if (knownFile && knownFile.is_combined_parts) {
        data = {
          ok: 1,
          title: knownFile.title,
          file_size: knownFile.file_size,
          file_type: knownFile.file_type || knownFile.extension || 'video/mp4',
          mime: knownFile.mime || 'video/mp4',
          thumbnail_url: knownFile.thumbnail_url || '',
          play_url: knownFile.play_url || '',
          is_combined_parts: true,
          parts: knownFile.parts,
          bot_id: knownFile.bot_id || this.botId || null
        };
      }

      // 1. Always use local backend proxy (/api/resolve-shortcode)
      // The local backend runs concurrent multi-bot resolution across the active bot pool
      const resolveUrl = new URL(`${this.localApiBase}/api/resolve-shortcode`);
      resolveUrl.searchParams.set('short_code', shortCode);

      const FETCH_TIMEOUT_MS = 15000;

      for (let attempt = 1; attempt <= 3; attempt++) {
        try {
          const controller = new AbortController();
          const timeoutId = setTimeout(() => controller.abort(), FETCH_TIMEOUT_MS);
          try {
            data = await this.requestJson(resolveUrl.toString(), {
              signal: controller.signal,
            });
          } finally {
            clearTimeout(timeoutId);
          }
          if (data && data.ok) break;
        } catch (e) {
          if (attempt >= 3) throw e;
        }
        await new Promise(r => setTimeout(r, attempt * 1000));
      }

      // Resolving complete — hide spinner
      this._endResolving(data && data.ok);

      if (!data || !data.ok) {
        const failMsg = data?.message || 'Failed to resolve file';
        this.$('#fileDetailTitle').textContent = 'Failed to resolve file';
        this.$('#fileDetailTags').innerHTML = `<span style="color:var(--accent)">${this.escapeHtml(failMsg)}</span>`;
        if (this._isReloginRequired(failMsg) || !this.botId) {
          this.promptBotRelogin(failMsg);
        }
        return;
      }

      // Cache the result (TTL 2 hours because Telegram file_reference tokens expire)
      this._cacheSet(resolveCacheKey, data, 2 * 60 * 60 * 1000);

      // Render
      this._renderResolvedFile(data, shortCode);

    } catch (error) {
      const isTimeout = error.name === 'AbortError';
      const msg = isTimeout ? 'Request timed out. The file resolver is not responding.' : error.message;
      console.warn('[resolve-shortcode] Failed after 3 retries:', isTimeout ? 'timeout' : error.message);
      this._endResolving(false);
      this.$('#fileDetailTitle').textContent = 'Error resolving file';
      this.$('#fileDetailTags').innerHTML = `<span style="color:var(--accent)">${this.escapeHtml(msg)}</span>`;

      // Auto-logout when WordPress says the bot is missing / API secret is invalid.
      // Catalog browsing can still work from a leftover Madeline session, so the
      // settings gate must be forced into token-entry mode instead of "Connected".
      if (!isTimeout && (this._isReloginRequired(msg) || !this.botId)) {
        this.promptBotRelogin(msg);
      }
    }
  }

  /**
   * Render the resolved file data into the file detail page.
   * Extracted so both cache-hit and fresh-fetch paths can reuse the same rendering.
   * @param {Object} data — resolve-file API response
   * @param {string} shortCode — fallback title
   */
  _renderResolvedFile(data, shortCode) {
    const fileId = data.file_id_mt || data.file_id || '';
    const title = data.title || shortCode;
    const fileSize = data.file_size || 0;
    const fileType = data.file_type || data.mime || 'file';
    const thumbnail = data.thumbnail || data.thumbnail_url || '';

    const videoEl = this.$('#fileDetailVideo');
    const audioEl = this.$('#fileDetailAudio');
    const playerEl = this.$('#fileDetailPlayer');

    // Render file info
    this.$('#fileDetailTitle').textContent = title;
    this.$('#fileDetailSize').textContent = 'Size: ' + this.formatSize(fileSize);

    if (thumbnail) {
      this.$('#fileDetailThumbImg').src = thumbnail;
      this.$('#fileDetailThumbImg').alt = title;
      // Show thumbnail container (hidden during resolving)
      const thumbContainer = this.$('#fileDetailThumb');
      if (thumbContainer) thumbContainer.classList.remove('hidden');
    }

    // Tags
    const tags = this.extractMediaTags(title, data.caption || '');
    const isUnplayable = this._guessMediaType(title, fileType) === null;
    const detailBadges = [];
    if (tags.resolution) detailBadges.push(`<span class="stream-file-card__badge stream-file-card__badge--res">${this.escapeHtml(tags.resolution)}</span>`);
    if (tags.platform) detailBadges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.platform)}</span>`);
    if (tags.source) detailBadges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.source)}</span>`);
    tags.visual.forEach(v => detailBadges.push(`<span class="stream-file-card__badge stream-file-card__badge--hdr">${this.escapeHtml(v)}</span>`));
    if (tags.codec) detailBadges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.codec)}</span>`);
    if (tags.audio) detailBadges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.audio)}</span>`);
    if (tags.edition) detailBadges.push(`<span class="stream-file-card__badge">${this.escapeHtml(tags.edition)}</span>`);
    if (fileType && !tags.source && !tags.codec) {
      detailBadges.push(`<span class="stream-file-card__badge">${this.escapeHtml(fileType)}</span>`);
    }
    this.$('#fileDetailTags').innerHTML = detailBadges.join('');

    // Detect media type and embed player if video/audio
    const mediaType = this._guessMediaType(title, fileType);
    const isEmbeddable = mediaType === 'video' || mediaType === 'audio';

    if (isEmbeddable && fileId && fileSize > 0) {
      // Build local download URL as video/audio source
      const streamUrl = this.buildDownloadUrl(fileId, fileSize, title, data.mime || fileType, data.bot_id, shortCode);
      this.currentStreamUrl = streamUrl;

      if (mediaType === 'video') {
        if (videoEl) {
          videoEl.src = streamUrl;
          if (thumbnail) videoEl.poster = thumbnail;
          videoEl.classList.remove('hidden');

          // Fetch and attach subtitles (OpenSubtitles) using IMDb ID or media title fallback
          const imdbId = data.imdb_id || (this._previousPostData && this._previousPostData.imdb_id) || '';
          const subQueryId = imdbId || (data.title ? encodeURIComponent(data.title) : '');
          if (subQueryId) {
            const mediaTypeForSub = (data.type === 'series' || (this._previousPostData && this._previousPostData.type === 'series')) ? 'series' : 'movie';
            fetch(`${this.localApiBase}/subtitles/${mediaTypeForSub}/${subQueryId}.json`)
              .then((r) => (r.ok ? r.json() : null))
              .then((subData) => {
                if (subData && Array.isArray(subData.subtitles) && subData.subtitles.length > 0) {
                  subData.subtitles.slice(0, 40).forEach((sub, idx) => {
                    if (sub && sub.url) {
                      const track = document.createElement('track');
                      track.kind = 'subtitles';
                      const langLabel = (sub.lang || sub.lang_code || 'und').toUpperCase();
                      const subName = sub.subtitleFileName || sub.title || sub.movieReleaseName || `Track ${idx + 1}`;
                      track.label = `[${langLabel}] ${subName}`;
                      track.srclang = sub.lang || sub.lang_code || 'en';
                      // Route via local subtitle proxy to guarantee valid WebVTT and bypass CORS
                      track.src = `${this.localApiBase}/api/sub-proxy?url=${encodeURIComponent(sub.url)}`;
                      videoEl.appendChild(track);
                    }
                  });
                  // Update toolbar subtitle dropdown immediately
                  this._populateSubtitlesList();
                }
              })
              .catch((err) => {
                console.warn('[Subtitles] Failed to load OpenSubtitles:', err);
              });
          }
        }
        if (audioEl) audioEl.classList.add('hidden');
        this.$('#playerToolbar')?.classList.remove('hidden');
      } else {
        if (audioEl) {
          audioEl.src = streamUrl;
          audioEl.classList.remove('hidden');
        }
        if (videoEl) videoEl.classList.add('hidden');
        this.$('#playerToolbar')?.classList.add('hidden');
      }

      if (playerEl) playerEl.classList.remove('hidden');

      // Hide the Stream button (embedded player replaces it)
      this.$('#fileDetailStreamBtn').classList.add('hidden');
    } else {
      // Not embeddable inline (archive, .001 split chunk, etc.):
      // Hide inline player and Stream button; user can download the file via Download button
      this.$('#fileDetailStreamBtn').classList.add('hidden');
      if (playerEl) playerEl.classList.add('hidden');
    }

    // Download button: build local download URL
    if (fileId && fileSize > 0) {
      const downloadUrl = this.buildDownloadUrl(fileId, fileSize, title, data.mime || fileType, data.bot_id, shortCode);
      this.$('#fileDetailDownloadBtn').setAttribute('data-url', downloadUrl);
      this.$('#fileDetailDownloadBtn').disabled = false;
      this.$('#fileDetailDownloadBtn').innerHTML = '<i class="fas fa-download"></i> Download';
    } else {
      this.$('#fileDetailDownloadBtn').disabled = true;
      this.$('#fileDetailDownloadBtn').innerHTML = '<i class="fas fa-download"></i> No file ID';
    }

    // Render sponsored button in File Detail page if configured
    this._renderFileDetailSponsor();
  }

  /**
   * Render sponsored card for post modal files list.
   * Prepends a sponsored item to the files grid linking directly to sponsor url.
   */
  _renderSponsorFileCard() {
    if (!this.sponsor || !this.sponsor.url) return '';
    const sponsorName = this.sponsor.name || 'Sponsored';
    const sponsorDesc = this.sponsor.description || 'Support / Sponsor';
    const sponsorUrl = this.sponsor.url;

    return `
      <div class="stream-file-card stream-file-card--sponsor" data-sponsor-url="${this.escapeHtml(sponsorUrl)}" title="${this.escapeHtml(sponsorDesc)}" style="border: 1px solid rgba(255, 107, 53, 0.4); background: rgba(255, 107, 53, 0.05); cursor: pointer;" onclick="window.open('${this.escapeHtml(sponsorUrl)}', '_blank')">
        <div class="stream-file-card__thumb" style="display: flex; align-items: center; justify-content: center; background: rgba(255, 107, 53, 0.15); color: var(--accent, #ff6b35); font-size: 1.5rem;">
          <i class="fas fa-star"></i>
          <div class="stream-file-card__overlay">
            <span class="stream-file-card__play"><i class="fas fa-external-link-alt"></i></span>
          </div>
        </div>
        <div class="stream-file-card__info">
          <div class="stream-file-card__title" style="color: var(--accent, #ff6b35); font-weight: 600;">${this.escapeHtml(sponsorName)}</div>
          <div class="stream-file-card__meta">
            <span class="stream-file-card__size" style="color: rgba(255,255,255,0.7);">${this.escapeHtml(sponsorDesc)}</span>
          </div>
        </div>
        <div class="stream-file-card__action">
          <button class="stream-file-card__btn" style="background: var(--accent, #ff6b35); color: #fff;" aria-label="Open link" title="Open link">
            <i class="fas fa-external-link-alt"></i>
          </button>
        </div>
      </div>
    `;
  }

  /**
   * Render or update sponsored button on the File Detail page.
   */
  _renderFileDetailSponsor() {
    const actionsEl = this.$('#fileDetailActions');
    if (!actionsEl) return;

    let sponsorBtn = this.$('#fileDetailSponsorBtn');
    if (!this.sponsor || !this.sponsor.url) {
      if (sponsorBtn) sponsorBtn.remove();
      return;
    }

    const sponsorName = this.sponsor.name || 'Sponsored';
    const sponsorDesc = this.sponsor.description || '';
    const sponsorUrl = this.sponsor.url;

    if (!sponsorBtn) {
      sponsorBtn = document.createElement('a');
      sponsorBtn.id = 'fileDetailSponsorBtn';
      sponsorBtn.className = 'file-detail__sponsor-btn';
      sponsorBtn.target = '_blank';
      sponsorBtn.rel = 'noopener noreferrer';
      actionsEl.appendChild(sponsorBtn);
    }

    sponsorBtn.href = sponsorUrl;
    sponsorBtn.title = sponsorDesc;
    sponsorBtn.innerHTML = `${this.escapeHtml(sponsorName)}`;
  }

  closeFileDetail() {
    const filePage = this.$('#fileDetailPage');
    const streamApp = this.$('#streamApp');

    if (filePage) {
      filePage.classList.add('hidden');
      filePage.setAttribute('aria-hidden', 'true');
    }

    // Pause and reset embedded player before restoring post/search context.
    this._resetFileDetailPlayer();

    // Reset resolving state (in case file detail was closed mid-resolve)
    this._endResolving();

    // If user came from a post modal, restore it
    if (this._cameFromPost) {
      if (this._previousPostData) {
        this.openModal(this._previousPostData);
      }
      this._clearFileDetailContext();
      this._cameFromPost = false;
      this._previousPostData = null;
      // Keep streamApp visible underneath the modal
      if (streamApp) {
        streamApp.classList.remove('hidden');
        streamApp.setAttribute('aria-hidden', 'false');
      }
      // Clear the #file/ hash so the URL reflects the restored state
      const currentHash = window.location.hash;
      if (currentHash && currentHash !== '#') {
        this._suppressHash = true;
        window.location.hash = '#';
        setTimeout(() => { this._suppressHash = false; }, 0);
      }
      return;
    }

    // If user came from category page, restore it without resetting pagination
    if (this._cameFromCategory) {
      this._cameFromCategory = false;
      const categoryPage = this.$('#categoryPage');
      if (categoryPage) {
        categoryPage.classList.remove('hidden');
        categoryPage.setAttribute('aria-hidden', 'false');
      }
      // Keep streamApp hidden underneath the category overlay
      if (streamApp) {
        streamApp.classList.add('hidden');
        streamApp.setAttribute('aria-hidden', 'true');
      }
      // Clear the #file/ hash so the URL reflects the restored state
      const currentHash = window.location.hash;
      if (currentHash && currentHash !== '#') {
        this._suppressHash = true;
        window.location.hash = '#' + (!this._categorySlug ? '' : 'category/' + this._categorySlug);
        setTimeout(() => { this._suppressHash = false; }, 0);
      }
      return;
    }

    // If user came from search, restore search overlay using openSearch()
    // which properly sets isSearchOpen=true and adds the CSS "open" class.
    // The search overlay is position:fixed;z-index:1500 so it sits on top
    // of streamApp naturally — no need to hide streamApp underneath.
    if (this._cameFromSearch) {
      this.openSearch();
      const savedQuery = this._savedSearchQuery;
      this._savedSearchQuery = '';
      if (savedQuery) {
        const searchInput = this.$('#searchInput');
        if (searchInput) {
          searchInput.value = savedQuery;
          this.doSearch(savedQuery);
        }
      }
      // Keep streamApp visible underneath the fixed search overlay
      if (streamApp) {
        streamApp.classList.remove('hidden');
        streamApp.setAttribute('aria-hidden', 'false');
      }
      this._clearFileDetailContext();
      this._cameFromSearch = false;
    } else {
      // Normal flow — show main app
      this._clearFileDetailContext();
      if (streamApp) {
        streamApp.classList.remove('hidden');
        streamApp.setAttribute('aria-hidden', 'false');
      }
      // Resume hero rotation when returning to main view
      this._startHeroRotation();
    }

    // Clear hash — only if not already cleared (prevents recursive hashchange
    // when browser back button already restored the previous hash)
    const currentHash = window.location.hash;
    if (currentHash && currentHash !== '#') {
      this._suppressHash = true;
      window.location.hash = '#';
      // Defer reset so the async hashchange event sees suppressHash=true
      setTimeout(() => { this._suppressHash = false; }, 0);
    }
  }

  // ══════════════════════════════════════════════════════════════
  //  MOBILE NAV
  // ══════════════════════════════════════════════════════════════

  openMobileNav() {
    this._stopHeroRotation();
    this.isMobileNavOpen = true;
    const nav = this.$('#mobileNav');
    const overlay = this.$('#mobileNavOverlay');
    if (nav) nav.classList.add('open');
    if (overlay) overlay.classList.add('open');
  }

  closeMobileNav() {
    this.isMobileNavOpen = false;
    const nav = this.$('#mobileNav');
    const overlay = this.$('#mobileNavOverlay');
    if (nav) nav.classList.remove('open');
    if (overlay) overlay.classList.remove('open');

    // Resume hero rotation when closing mobile nav
    this._startHeroRotation();
  }

  // ══════════════════════════════════════════════════════════════
  //  HASH ROUTING
  // ══════════════════════════════════════════════════════════════

  _checkHash() {
    const hash = window.location.hash;
    if (hash === '#configure' || hash === '#addon') {
      this.openAddonModal?.();
    } else if (hash === '#settings') {
      this.showSettingsGate({ forceToken: !this.botId });
    } else if (hash.startsWith('#post/')) {
      const postId = hash.replace('#post/', '');
      this._openPostFromHash(postId);
    } else if (hash.startsWith('#file/')) {
      const shortCode = hash.replace('#file/', '');
      this.openFileDetail(shortCode);
    } else if (hash.startsWith('#category/')) {
      const slug = hash.replace('#category/', '');
      const cat = this.categories.find(c => c.slug === slug);
      this.openCategoryPage(slug, cat ? cat.name : slug);
    }
  }

  _handleHashChange() {
    if (this._suppressHash) return;
    const hash = window.location.hash;

    if (!hash || hash === '#') {
      if (this._isCategoryPageOpen) this.closeCategoryPage();
      if (this.isModalOpen) this.closeModal();
      if (this.isFileDetailOpen()) this.closeFileDetail();
      this.closeAddonModal?.();
    } else if (hash === '#configure' || hash === '#addon') {
      this.openAddonModal?.();
    } else if (hash === '#settings') {
      this.showSettingsGate({ forceToken: !this.botId });
    } else if (hash.startsWith('#category/')) {
      const slug = hash.replace('#category/', '');

      // If file detail is open, closeFileDetail() via _cameFromCategory
      // restores the category page without resetting pagination — avoid
      // a duplicate openCategoryPage() call that would clear the grid.
      if (this.isFileDetailOpen()) {
        this.closeFileDetail();
        return;
      }

      const cat = this.categories.find(c => c.slug === slug);
      this.openCategoryPage(slug, cat ? cat.name : slug);
    } else if (hash.startsWith('#post/')) {
      // Close file detail page first if it's open (e.g. browser back from #file/ to #post/)
      if (this.isFileDetailOpen()) this.closeFileDetail();
      const postId = hash.replace('#post/', '');
      this._openPostFromHash(postId);
    } else if (hash.startsWith('#file/')) {
      // Guard: don't re-open file detail if already open (prevents
      // _cameFromSearch reset when hashchange fires asynchronously
      // after programmatic hash assignment)
      if (this.isFileDetailOpen()) return;
      const shortCode = hash.replace('#file/', '');
      this.openFileDetail(shortCode);
    }
  }

  async _openPostFromHash(postId) {
    try {
      const postData = await this.fetchStream('get_post', { post_id: postId });
      if (postData) this.openModal(postData);
    } catch (error) {
      console.warn('Failed to load post from hash:', error);
    }
  }

  // ══════════════════════════════════════════════════════════════
  //  VERSION CHECK
  // ══════════════════════════════════════════════════════════════

  /**
   * Fetch version info from the local backend.
   * Returns null on failure or if no update needed.
   * Returns { update_needed, current_version, minimum_version, update_url } on match.
   */
  async checkVersion() {
    try {
      const resp = await fetch(`${this.localApiBase}/api/version`);
      if (!resp.ok) {
        console.warn('Version endpoint returned', resp.status);
        return null;
      }
      const data = await resp.json();
      if (data?.current_version) {
        this.version = String(data.current_version);
        this.renderSettingsVersion();
      }
      if (data?.sponsor && typeof data.sponsor === 'object') {
        const url = String(data.sponsor.url || '').trim();
        if (url) {
          this.sponsor = {
            name: String(data.sponsor.name || 'Sponsored').trim(),
            description: String(data.sponsor.description || '').trim(),
            url: url
          };
        } else {
          this.sponsor = null;
        }
      }
      if (data && data.update_needed) {
        return data;
      }
      return null;
    } catch (e) {
      console.warn('Version check request failed:', e);
      return null;
    }
  }

  renderSettingsVersion() {
    const versionText = this.version ? `v${this.version}` : '';
    const setupEl = this.$('#settingsVersion');
    const connectedEl = this.$('#settingsConnectedVersion');
    if (setupEl) setupEl.textContent = versionText;
    if (connectedEl) connectedEl.textContent = versionText;
  }

  /**
   * Show the update required overlay with version details.
   * Called when the server reports the app is outdated.
   */
  showUpdateRequired(info) {
    const overlay = this.$('#updateRequiredOverlay');
    if (!overlay) return;

    const currentEl = overlay.querySelector('.update-required__version-current');
    const minimumEl = overlay.querySelector('.update-required__version-minimum');
    const linkEl = overlay.querySelector('.update-required__link');

    if (currentEl) currentEl.textContent = info.current_version || '?';
    if (minimumEl) minimumEl.textContent = info.minimum_version || '?';
    if (linkEl) {
      if (info.update_url) {
        linkEl.href = info.update_url;
        linkEl.style.display = 'inline-block';
      } else {
        linkEl.style.display = 'none';
      }
    }

    overlay.classList.remove('hidden');
    overlay.setAttribute('aria-hidden', 'false');
  }
}

// ─── Boot ──────────────────────────────────────────────────────
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', () => {
    window.app = new PencariMovieApp();
  });
} else {
  window.app = new PencariMovieApp();
}
