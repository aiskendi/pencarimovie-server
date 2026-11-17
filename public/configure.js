/* PencariMovie public Configure page.
 *
 * The whole configuration lives in the first path segment:
 *   https://host/<base64url(JSON)>/manifest.json
 *
 * There is no POST and no server-side per-user state. The page only reads
 * /api/public/options (sanitized defaults) and /api/public/config/<payload>
 * (for the Stremio gear's prefilled edit link).
 */
(function () {
  "use strict";

  var $ = function (id) { return document.getElementById(id); };
  var state = {
    catalogsEnabled: true,
    types: { movie: true, series: true, other: true },
    catalogOverrides: {},
    stream: {}
  };
  var options = null;
  var currentCode = "";

  var GROUP_LABELS = {
    resolutions: {
      "4k": "4K / UHD (2160p)",
      "1080p": "1080p (FHD)",
      "720p": "720p (HD)",
      sd: "SD (480p / 360p)",
      unknown: "Unknown"
    },
    qualities: {
      remux: "BluRay REMUX",
      bluray: "BluRay",
      webdl: "WEB-DL",
      webrip: "WEBRip",
      hdtv: "HDTV",
      cam: "CAM / Telesync"
    },
    encodes: { hevc: "HEVC (H.265)", avc: "AVC (H.264)", av1: "AV1" },
    visual_tags: { hdr: "HDR / HDR10+", dv: "Dolby Vision" }
  };
  var GROUP_TITLES = {
    resolutions: "Resolutions:",
    qualities: "Source Qualities:",
    encodes: "Encodes & Formats:"
  };
  var FLAGS = [
    ["exclude_cam", "Block All CAM"],
    ["exclude_unplayable", "Exclude Unplayable"],
    ["exclude_part_title", "Exclude Part Titles"]
  ];
  var NUMBERS = [
    ["max_streams_total", "Max streams total (0 = no cap)"],
    ["max_streams_per_resolution", "Max streams per resolution (0 = no cap)"],
    ["min_size_mb", "Minimum size (MB, 0 = no minimum)"],
    ["max_size_gb", "Maximum size (GB, 0 = no maximum)"]
  ];

  function setStatus(text, isError) {
    var el = isError ? $("error") : $("status");
    if (!el) return;
    el.textContent = text || "";
    el.classList.toggle("hidden", !text);
  }

  function b64url(text) {
    var bytes;
    if (window.TextEncoder) {
      bytes = new TextEncoder().encode(text);
    } else {
      var escaped = unescape(encodeURIComponent(text));
      bytes = new Uint8Array(escaped.length);
      for (var i = 0; i < escaped.length; i++) bytes[i] = escaped.charCodeAt(i);
    }
    var binary = "";
    for (var j = 0; j < bytes.length; j++) binary += String.fromCharCode(bytes[j]);
    return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
  }

  function defaultCatalogOn(id) {
    var map = (options && options.defaults && options.defaults.enabled_catalogs) || {};
    return map[id] !== false;
  }

  function defaultTypeOn(type) {
    var map = (options && options.defaults && options.defaults.enabled_types) || {};
    return map[type] !== false;
  }

  function globalStreamValue(group, key) {
    var sc = (options && options.defaults && options.defaults.stream_config) || {};
    var g = sc[group];
    if (!g || typeof g !== "object") return true;
    return g[key] !== false;
  }

  function globalFlag(key) {
    var sc = (options && options.defaults && options.defaults.stream_config) || {};
    return sc[key] === true;
  }

  function globalNumber(key) {
    var sc = (options && options.defaults && options.defaults.stream_config) || {};
    var n = Number(sc[key]);
    return isFinite(n) && n > 0 ? n : 0;
  }

  function globalText(key) {
    var sc = (options && options.defaults && options.defaults.stream_config) || {};
    return typeof sc[key] === "string" ? sc[key] : "";
  }

  function defaultCatalogsEnabled() {
    return (options && options.defaults && options.defaults.catalogs_enabled) !== false;
  }

  function defaultTypeMask() {
    var map = (options && options.defaults && options.defaults.enabled_types) || {};
    return (map.movie !== false ? 1 : 0) |
      (map.series !== false ? 2 : 0) |
      (map.other !== false ? 4 : 0);
  }

  function makeCheck(labelText, checked, disabled, onChange) {
    var label = document.createElement("label");
    label.className = "addon-catalog-item" + (disabled ? " locked" : "");
    var input = document.createElement("input");
    input.type = "checkbox";
    input.checked = !!checked;
    input.disabled = !!disabled;
    input.addEventListener("change", function () { onChange(input.checked); });
    var span = document.createElement("span");
    span.textContent = labelText;
    label.appendChild(input);
    label.appendChild(span);
    return label;
  }

  function makePill(labelText, checked, disabled, onChange) {
    var label = document.createElement("label");
    label.className = "addon-catalog-pill" + (disabled ? " locked" : "");
    var input = document.createElement("input");
    input.type = "checkbox";
    input.checked = !!checked;
    input.disabled = !!disabled;
    input.addEventListener("change", function () { onChange(input.checked); });
    var span = document.createElement("span");
    span.textContent = labelText;
    label.appendChild(input);
    label.appendChild(span);
    return label;
  }

  function renderCatalogs() {
    var special = $("specialList");
    var browse = $("catalogList");
    special.textContent = "";
    browse.textContent = "";
    var cats = (options && options.catalog_options) || {};
    var on = $("catalogMasterOn");
    var off = $("catalogMasterOff");
    on.checked = state.catalogsEnabled;
    off.checked = !state.catalogsEnabled;
    var details = $("addonCatalogDetails");
    if (details) details.classList.toggle("hidden", !state.catalogsEnabled);
    var badge = $("catalogModeBadge");
    if (badge) {
      badge.textContent = state.catalogsEnabled ? "Enabled" : "Disabled";
      badge.classList.toggle("disabled", !state.catalogsEnabled);
    }
    ["movie", "series", "other"].forEach(function (type) {
      var input = $("type" + type.charAt(0).toUpperCase() + type.slice(1));
      if (!input) return;
      input.checked = state.types[type] !== false;
      input.disabled = false;
    });

    Object.keys(cats).sort(function (a, b) {
      var ga = (cats[a].group || "") + "|" + (cats[a].name || "");
      var gb = (cats[b].group || "") + "|" + (cats[b].name || "");
      return ga.localeCompare(gb);
    }).forEach(function (id) {
      var info = cats[id] || {};
      var globallyOn = defaultCatalogOn(id);
      var override = state.catalogOverrides[id];
      var checked = globallyOn ? override !== false : override === true;
      var disabled = false;
      var node = makeCheck(info.name || id, checked, disabled, function (on) {
        if (globallyOn) {
          if (on) delete state.catalogOverrides[id];
          else state.catalogOverrides[id] = false;
        } else if (on) {
          state.catalogOverrides[id] = true;
        } else {
          delete state.catalogOverrides[id];
        }
        renderInstall();
      });
      (info.group === "special" ? special : browse).appendChild(node);
    });
  }

  function streamGroupValue(group, key) {
    var g = state.stream[group];
    if (!g || typeof g !== "object") return undefined;
    return g[key];
  }

  function setStreamGroup(group, key, value) {
    if (value === undefined) {
      if (state.stream[group]) {
        delete state.stream[group][key];
        if (!Object.keys(state.stream[group]).length) delete state.stream[group];
      }
    } else {
      if (!state.stream[group]) state.stream[group] = {};
      state.stream[group][key] = value;
    }
  }

  function renderFilters() {
    var host = $("streamFilters");
    host.textContent = "";

    ["resolutions", "qualities", "encodes"].forEach(function (group) {
      var section = document.createElement("div");
      section.className = "addon-catalog-section";
      var title = document.createElement("div");
      title.className = "addon-catalog-section__label";
      title.textContent = GROUP_TITLES[group];
      var row = document.createElement("div");
      row.className = "addon-catalog-pill-group";
      var entries = Object.keys(GROUP_LABELS[group]).map(function (key) {
        return [group, key];
      });
      if (group === "encodes") {
        entries = entries.concat(Object.keys(GROUP_LABELS.visual_tags).map(function (key) {
          return ["visual_tags", key];
        }));
      }
      entries.forEach(function (entry) {
        var g = entry[0];
        var key = entry[1];
        var globallyOn = globalStreamValue(g, key);
        var override = streamGroupValue(g, key);
        var checked = typeof override === "boolean" ? override : globallyOn;
        row.appendChild(makePill(GROUP_LABELS[g][key], checked, false, function (on) {
          if (on === globallyOn) setStreamGroup(g, key, undefined);
          else setStreamGroup(g, key, on);
          renderInstall();
        }));
      });
      section.appendChild(title);
      section.appendChild(row);
      host.appendChild(section);
    });

    var flagSection = document.createElement("div");
    flagSection.className = "addon-catalog-section";
    var flagTitle = document.createElement("div");
    flagTitle.className = "addon-catalog-section__label";
    flagTitle.textContent = "Exclusions:";
    var flagRow = document.createElement("div");
    flagRow.className = "addon-catalog-pill-group";
    FLAGS.forEach(function (pair) {
      var key = pair[0];
      var globallyOn = globalFlag(key);
      var override = state.stream[key];
      var checked = typeof override === "boolean" ? override : globallyOn;
      flagRow.appendChild(makePill(pair[1], checked, false, function (on) {
        if (on === globallyOn) delete state.stream[key];
        else state.stream[key] = on;
        renderInstall();
      }));
    });
    flagSection.appendChild(flagTitle);
    flagSection.appendChild(flagRow);
    host.appendChild(flagSection);

    var numberSection = document.createElement("div");
    numberSection.className = "addon-catalog-section";
    var numberTitle = document.createElement("div");
    numberTitle.className = "addon-catalog-section__label";
    numberTitle.textContent = "Size and Count Limits:";
    var numberGrid = document.createElement("div");
    numberGrid.className = "public-filter-grid";
    NUMBERS.forEach(function (pair) {
      var key = pair[0];
      var field = document.createElement("label");
      field.className = "public-field";
      var span = document.createElement("span");
      span.textContent = pair[1];
      var input = document.createElement("input");
      input.className = "pm-input";
      input.type = "number";
      input.min = "0";
      input.step = "1";
      input.value = Number(state.stream[key] || 0) || "";
      input.placeholder = String(globalNumber(key) || 0);
      input.addEventListener("input", function () {
        var n = Math.max(0, parseInt(input.value, 10) || 0);
        if (!n || n === globalNumber(key)) delete state.stream[key];
        else state.stream[key] = n;
        renderInstall();
      });
      field.appendChild(span);
      field.appendChild(input);
      numberGrid.appendChild(field);
    });
    numberSection.appendChild(numberTitle);
    numberSection.appendChild(numberGrid);
    host.appendChild(numberSection);

    var kwSection = document.createElement("div");
    kwSection.className = "addon-catalog-section";
    var kwTitle = document.createElement("div");
    kwTitle.className = "addon-catalog-section__label";
    kwTitle.textContent = "Keywords:";
    var kwGrid = document.createElement("div");
    kwGrid.className = "public-filter-grid";
    [["excluded_keywords", "Exclude titles containing (comma separated)"], ["required_keywords", "Only titles containing (comma separated)"]].forEach(function (pair) {
      var key = pair[0];
      var field = document.createElement("label");
      field.className = "public-field";
      var span = document.createElement("span");
      span.textContent = pair[1];
      var input = document.createElement("input");
      input.className = "pm-input";
      input.type = "text";
      input.value = typeof state.stream[key] === "string" ? state.stream[key] : globalText(key);
      input.placeholder = globalText(key);
      input.addEventListener("input", function () {
        var value = input.value.trim();
        if (value === globalText(key)) delete state.stream[key];
        else state.stream[key] = value;
        renderInstall();
      });
      field.appendChild(span);
      field.appendChild(input);
      kwGrid.appendChild(field);
    });
    kwSection.appendChild(kwTitle);
    kwSection.appendChild(kwGrid);
    host.appendChild(kwSection);
  }

  function buildConfig() {
    var cfg = { v: 1 };
    // Always encode the master choice. Without this, enabling a
    // default-enabled catalog removes `c` and the install URL looks unchanged
    // even though the user just toggled the control.
    cfg.c = state.catalogsEnabled ? 1 : 0;
    var mask = 0;
    if (state.types.movie !== false) mask |= 1;
    if (state.types.series !== false) mask |= 2;
    if (state.types.other !== false) mask |= 4;
    if (mask !== defaultTypeMask() && mask !== 0) cfg.t = mask;
    var cats = (options && options.catalog_options) || {};
    var hide = [];
    var show = [];
    Object.keys(cats).forEach(function (id) {
      var globallyOn = defaultCatalogOn(id);
      var override = state.catalogOverrides[id];
      if (globallyOn && override === false) hide.push(id);
      if (!globallyOn && override === true) show.push(id);
    });
    if (hide.length) cfg.h = hide;
    if (show.length) cfg.e = show;
    if (Object.keys(state.stream).length) cfg.s = state.stream;
    return cfg;
  }

  function renderInstall() {
    var cfg = buildConfig();
    var json = JSON.stringify(cfg);
    var code = b64url(json);
    currentCode = code;
    var path = "/" + code + "/manifest.json";
    var url = window.location.origin + path;
    $("installUrl").value = url;
    $("installBtn").href = "stremio://" + window.location.host + path;
    $("configMeta").textContent = code.length + " chars, " + Object.keys(state.stream).length + " filter override(s)";
    try { localStorage.setItem("pm.public.last", code); } catch (e) {}
  }

  function applyConfig(cfg) {
    if (!cfg || typeof cfg !== "object") return;
    state.catalogsEnabled = typeof cfg.c === "number" ? cfg.c !== 0 : defaultCatalogsEnabled();
    state.catalogOverrides = {};
    (cfg.h || []).forEach(function (id) { state.catalogOverrides[id] = false; });
    (cfg.e || []).forEach(function (id) { state.catalogOverrides[id] = true; });
    state.stream = (cfg.s && typeof cfg.s === "object") ? cfg.s : {};
    var mask = typeof cfg.t === "number" ? cfg.t : defaultTypeMask();
    state.types = {
      movie: !!(mask & 1),
      series: !!(mask & 2),
      other: !!(mask & 4)
    };
    renderAll();
  }

  function resetConfig() {
    state.catalogsEnabled = true;
    state.types = { movie: true, series: true, other: true };
    state.catalogOverrides = {};
    state.stream = {};
    renderAll();
  }

  function renderAll() {
    renderCatalogs();
    renderFilters();
    renderInstall();
  }

  function bind() {
    $("catalogMasterOn").addEventListener("change", function () {
      if (!this.checked) return;
      state.catalogsEnabled = true;
      renderCatalogs();
      renderInstall();
    });
    $("catalogMasterOff").addEventListener("change", function () {
      if (!this.checked) return;
      state.catalogsEnabled = false;
      renderCatalogs();
      renderInstall();
    });
    ["movie", "series", "other"].forEach(function (type) {
      var input = $("type" + type.charAt(0).toUpperCase() + type.slice(1));
      input.addEventListener("change", function () {
        state.types[type] = input.checked;
        if (!state.types.movie && !state.types.series && !state.types.other) {
          state.types[type] = true;
          input.checked = true;
          return;
        }
        renderInstall();
      });
    });
    var setCatalogGroup = function (group, on) {
      var cats = (options && options.catalog_options) || {};
      Object.keys(cats).forEach(function (id) {
        var isSpecial = (cats[id] && cats[id].group) === "special";
        if (group === "special" ? !isSpecial : isSpecial) return;
        if (on) {
          if (defaultCatalogOn(id)) delete state.catalogOverrides[id];
          else state.catalogOverrides[id] = true;
        } else {
          if (defaultCatalogOn(id)) state.catalogOverrides[id] = false;
          else delete state.catalogOverrides[id];
        }
      });
      renderCatalogs();
      renderInstall();
    };
    $("catalogAllSpecial").addEventListener("click", function () { setCatalogGroup("special", true); });
    $("catalogNoneSpecial").addEventListener("click", function () { setCatalogGroup("special", false); });
    $("catalogAllBrowse").addEventListener("click", function () { setCatalogGroup("browse", true); });
    $("catalogNoneBrowse").addEventListener("click", function () { setCatalogGroup("browse", false); });
    $("copyBtn").addEventListener("click", function () {
      var text = $("installUrl").value;
      var btn = $("copyBtn");
      var done = function () {
        var old = btn.textContent;
        btn.textContent = "Copied";
        setTimeout(function () { btn.textContent = old; }, 1400);
      };
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(text).then(done, function () {
          $("installUrl").select();
        });
      } else {
        $("installUrl").select();
        try { document.execCommand("copy"); done(); } catch (e) {}
      }
    });
    $("resetBtn").addEventListener("click", function () {
      resetConfig();
      setStatus("Reset to the server defaults.", false);
    });
  }

  function loadConfigFromApi(code) {
    return fetch("/api/public/config/" + encodeURIComponent(code), { cache: "no-store" })
      .then(function (res) {
        if (!res.ok) throw new Error("This configuration link is invalid.");
        return res.json();
      })
      .then(function (data) {
        applyConfig(data.config);
        setStatus("Editing an existing configuration. Re-install after changes.", false);
      });
  }

  function init() {
    bind();
    fetch("/api/public/options", { cache: "no-store" })
      .then(function (res) {
        if (res.status === 404) throw new Error("Public mode is off on this server.");
        if (!res.ok) throw new Error("The server did not answer the public options request.");
        return res.json();
      })
      .then(function (data) {
        options = data;
        setStatus("", false);
        var params = new URLSearchParams(window.location.search);
        var code = params.get("c");
        if (!code) {
          try { code = localStorage.getItem("pm.public.last"); } catch (e) {}
        }
        if (code) {
          loadConfigFromApi(code).catch(function (err) {
            setStatus(err.message || String(err), true);
            renderAll();
          });
        } else {
          state.catalogsEnabled = (data.defaults && data.defaults.catalogs_enabled) !== false;
          state.types = {
            movie: defaultTypeOn("movie"),
            series: defaultTypeOn("series"),
            other: defaultTypeOn("other")
          };
          renderAll();
        }
      })
      .catch(function (err) {
        setStatus(err.message || String(err), true);
        $("installBtn").setAttribute("aria-disabled", "true");
      });
  }

  if (document.readyState === "loading") {
    document.addEventListener("DOMContentLoaded", init);
  } else {
    init();
  }
})();
