/**
 * Runtime shims for the downloader UI (#settings / #addon) on old WebViews.
 *
 * Android 5.1 ships the AOSP WebView (Chromium 39-44): it has no `fetch`
 * (Chrome 42), no `Object.entries` (54), no `Array.from` (45), no
 * `String.prototype.padStart` (57), no `Element.prototype.closest` (41) and no
 * `AbortController` (66). core-js covers the language built-ins but NOT fetch,
 * so every app.js request failed and the Catalog Configuration lists rendered
 * empty — the static markup was all that showed.
 *
 * Loaded before core-js.bundle.js / app.js. Every shim is a no-op when the
 * native implementation exists, so modern engines are unaffected.
 */
(function installLegacyPolyfills(window, document) {
  "use strict";

  if (!window || !document) return;

  function define(target, name, value) {
    if (!target || target[name]) return;
    try {
      Object.defineProperty(target, name, { value: value, writable: true, configurable: true });
    } catch (error) {
      target[name] = value;
    }
  }

  // ── language built-ins ──────────────────────────────────────────────────
  define(Object, "entries", function entries(object) {
    var out = [];
    if (object === null || object === undefined) return out;
    var source = Object(object);
    for (var key in source) {
      if (Object.prototype.hasOwnProperty.call(source, key)) out.push([key, source[key]]);
    }
    return out;
  });

  define(Array, "from", function from(list) {
    var out = [];
    if (list === null || list === undefined) return out;
    if (typeof list.length === "number") {
      for (var i = 0; i < list.length; i++) out.push(list[i]);
      return out;
    }
    for (var key in list) {
      if (Object.prototype.hasOwnProperty.call(list, key)) out.push(list[key]);
    }
    return out;
  });

  define(Array.prototype, "includes", function includes(value) {
    for (var i = 0; i < this.length; i++) {
      if (this[i] === value) return true;
      if (value !== value && this[i] !== this[i]) return true; // NaN
    }
    return false;
  });

  define(String.prototype, "includes", function includes(needle) {
    return String(this).indexOf(needle) !== -1;
  });

  define(String.prototype, "padStart", function padStart(targetLength, padString) {
    var value = String(this);
    var fill = padString === undefined ? " " : String(padString);
    if (value.length >= targetLength || !fill) return value;
    var padding = "";
    while (padding.length < targetLength - value.length) padding += fill;
    return padding.slice(0, targetLength - value.length) + value;
  });

  define(String.prototype, "trimStart", function trimStart() {
    return String(this).replace(/^\s+/, "");
  });

  // ── DOM ────────────────────────────────────────────────────────────────
  if (window.Element && window.Element.prototype && !window.Element.prototype.closest) {
    window.Element.prototype.closest = function closest(selector) {
      var node = this;
      while (node && node.nodeType === 1) {
        if (node.matches) {
          if (node.matches(selector)) return node;
        } else if (node.msMatchesSelector && node.msMatchesSelector(selector)) {
          return node;
        } else {
          return null;
        }
        node = node.parentElement;
      }
      return null;
    };
  }

  // ── fetch via XMLHttpRequest ───────────────────────────────────────────
  if (window.Promise && !window.fetch) {
    window.fetch = function fetch(url, options) {
      var opts = options || {};
      return new window.Promise(function (resolve, reject) {
        var xhr = new XMLHttpRequest();
        xhr.open(String(opts.method || "GET").toUpperCase(), url, true);
        var headers = opts.headers || {};
        for (var name in headers) {
          if (Object.prototype.hasOwnProperty.call(headers, name)) {
            try {
              xhr.setRequestHeader(name, headers[name]);
            } catch (error) {
              /* forbidden header — ignore */
            }
          }
        }
        xhr.onload = function onLoad() {
          var rawHeaders = String(xhr.getAllResponseHeaders() || "");
          var map = {};
          var lines = rawHeaders.replace(/\r/g, "").split("\n");
          for (var i = 0; i < lines.length; i++) {
            var separator = lines[i].indexOf(":");
            if (separator > 0) {
              map[lines[i].slice(0, separator).toLowerCase().trim()] = lines[i].slice(separator + 1).trim();
            }
          }
          resolve({
            ok: xhr.status >= 200 && xhr.status < 300,
            status: xhr.status,
            statusText: xhr.statusText,
            url: url,
            headers: {
              get: function get(header) {
                return map[String(header).toLowerCase()] || null;
              }
            },
            text: function text() {
              return window.Promise.resolve(xhr.responseText);
            },
            json: function json() {
              try {
                return window.Promise.resolve(JSON.parse(xhr.responseText || "null"));
              } catch (error) {
                return window.Promise.reject(error);
              }
            }
          });
        };
        xhr.onerror = function onError() {
          reject(new TypeError("Network request failed"));
        };
        xhr.ontimeout = function onTimeout() {
          reject(new TypeError("Network request timed out"));
        };
        var body = opts.body;
        if (body !== undefined && body !== null && typeof body !== "string") {
          try {
            body = JSON.stringify(body);
          } catch (error) {
            /* send as-is */
          }
        }
        xhr.send(body === undefined ? null : body);
      });
    };
  }

  // ── AbortController (only .signal / .abort are used) ───────────────────
  if (!window.AbortController) {
    window.AbortController = function AbortController() {
      this.signal = { aborted: false };
    };
    window.AbortController.prototype.abort = function abort() {
      if (this.signal) this.signal.aborted = true;
    };
  }
})(window, document);
