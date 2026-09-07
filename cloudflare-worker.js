/**
 * PencariMovie Server - Cloudflare Worker Subdomain Relay
 *
 * Routes:
 *   - {bot_username}-tunnel.pencarimovie.com  -> proxy to that bot's active TryCloudflare tunnel
 *   - tunnel.pencarimovie.com/r/{bot_username} -> path-based proxy (fallback)
 *   - tunnel.pencarimovie.com/api/tunnel/register|lookup -> KV registration endpoints
 *
 * SAFETY: This worker is deployed on *.pencarimovie.com so it MUST pass through
 * every host that is NOT a tunnel host. Only hosts ending in "-tunnel.pencarimovie.com"
 * (or the /r/ path on tunnel.pencarimovie.com) are intercepted. Everything else —
 * pencarimovie.com, www.pencarimovie.com, telegram-webhook.pencarimovie.com, and all
 * WordPress {bot_username}.pencarimovie.com wildcard sites — is forwarded to origin
 * untouched via `return fetch(request)`.
 */

const registry = new Map();

// Hosts that must NEVER be intercepted (passed straight through to origin).
// The WordPress wildcard serves {bot_username}.pencarimovie.com, so we only
// intercept the explicit "-tunnel" suffix below.
const TUNNEL_SUFFIX = "-tunnel.pencarimovie.com";

export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const host = url.hostname.toLowerCase();

    const corsHeaders = {
      "Access-Control-Allow-Origin": "*",
      "Access-Control-Allow-Methods": "GET, POST, PUT, DELETE, OPTIONS",
      "Access-Control-Allow-Headers": "Content-Type, Authorization, X-App-Version, X-API-Secret, Range",
      "Access-Control-Max-Age": "86400",
    };

    if (request.method === "OPTIONS") {
      return new Response(null, {
        status: 204,
        headers: corsHeaders,
      });
    }

    // ── Registration endpoint: POST /api/tunnel/register ────────────────
    if (url.pathname === "/api/tunnel/register" && request.method === "POST") {
      try {
        const body = await request.json();
        const shortId = String(body.shortId || "").toLowerCase().trim();
        const tunnelUrl = String(body.tunnelUrl || "").trim();
        const secret = String(body.secret || "").trim();

        if (env.RELAY_SECRET && env.RELAY_SECRET !== secret) {
          return new Response(JSON.stringify({ ok: false, error: "Unauthorized" }), {
            status: 401,
            headers: { "Content-Type": "application/json", ...corsHeaders },
          });
        }

        if (!shortId || !tunnelUrl) {
          return new Response(JSON.stringify({ ok: false, error: "Missing shortId or tunnelUrl" }), {
            status: 400,
            headers: { "Content-Type": "application/json", ...corsHeaders },
          });
        }

        if (env && env.TUNNEL_KV && typeof env.TUNNEL_KV.put === "function") {
          await env.TUNNEL_KV.put(`tunnel:${shortId}`, tunnelUrl, { expirationTtl: 86400 * 14 });
        } else {
          registry.set(shortId, { tunnelUrl, updated: Date.now() });
        }

        return new Response(JSON.stringify({ ok: true, shortId, tunnelUrl }), {
          status: 200,
          headers: { "Content-Type": "application/json", ...corsHeaders },
        });
      } catch (e) {
        return new Response(JSON.stringify({ ok: false, error: e.message }), {
          status: 500,
          headers: { "Content-Type": "application/json", ...corsHeaders },
        });
      }
    }

    // ── Lookup endpoint: GET /api/tunnel/lookup?shortId=... ─────────────
    if (url.pathname === "/api/tunnel/lookup" && request.method === "GET") {
      const qShortId = String(url.searchParams.get("shortId") || "").toLowerCase().trim();
      let target = "";
      if (env && env.TUNNEL_KV && typeof env.TUNNEL_KV.get === "function") {
        target = (await env.TUNNEL_KV.get(`tunnel:${qShortId}`)) || "";
      } else {
        const item = registry.get(qShortId);
        target = item ? item.tunnelUrl : "";
      }
      return new Response(JSON.stringify({ ok: Boolean(target), shortId: qShortId, tunnelUrl: target }), {
        status: 200,
        headers: { "Content-Type": "application/json", ...corsHeaders },
      });
    }

    // ── Resolve which bot this request targets ─────────────────────────
    let shortId = "";
    let relayPath = url.pathname;

    // Format A: {bot_username}-tunnel.pencarimovie.com  (e.g. kakimoviesbot-tunnel.pencarimovie.com)
    const dashMatch = host.match(/^([a-z0-9_-]+)-tunnel\.pencarimovie\.com$/i);
    if (dashMatch) {
      shortId = dashMatch[1].toLowerCase();
    } else if (host === "tunnel.pencarimovie.com") {
      // Format B: tunnel.pencarimovie.com/r/{bot_username}/...
      const pathMatch = url.pathname.match(/^\/r\/([a-z0-9_-]{3,32})(\/.*)?$/i);
      if (pathMatch) {
        shortId = pathMatch[1].toLowerCase();
        relayPath = pathMatch[2] || "/";
      }
    }

    // ── SAFETY: If this host is NOT a tunnel host, pass through to origin ──
    // This protects pencarimovie.com, www, telegram-webhook, and every
    // WordPress {bot_username}.pencarimovie.com wildcard site.
    if (!shortId) {
      return fetch(request);
    }

    let targetTunnelUrl = "";
    if (env && env.TUNNEL_KV && typeof env.TUNNEL_KV.get === "function") {
      targetTunnelUrl = (await env.TUNNEL_KV.get(`tunnel:${shortId}`)) || "";
    } else {
      const item = registry.get(shortId);
      targetTunnelUrl = item ? item.tunnelUrl : "";
    }

    if (!targetTunnelUrl) {
      return new Response(
        `Tunnel for bot [${shortId}] is currently offline or not started.\n` +
        `Please click "Enable Tunnel" on your local PencariMovie server.`,
        {
          status: 502,
          headers: { "Content-Type": "text/plain; charset=utf-8" },
        }
      );
    }

    // ── Zero-buffer stream proxying to active TryCloudflare tunnel ─────
    const targetUrl = targetTunnelUrl.replace(/\/+$/, "") + relayPath + url.search;
    const newHeaders = new Headers(request.headers);
    newHeaders.delete("host");
    newHeaders.set("X-Forwarded-Host", host);
    newHeaders.set("X-Forwarded-Proto", "https");

    const reqInit = {
      method: request.method,
      headers: newHeaders,
      redirect: "follow",
    };

    if (request.method !== "GET" && request.method !== "HEAD") {
      reqInit.body = request.body;
      reqInit.duplex = "half";
    }

    try {
      const resp = await fetch(targetUrl, reqInit);
      const respHeaders = new Headers(resp.headers);
      respHeaders.set("Access-Control-Allow-Origin", "*");
      return new Response(resp.body, {
        status: resp.status,
        headers: respHeaders,
      });
    } catch (err) {
      return new Response(
        JSON.stringify({ error: `Cannot reach upstream quick tunnel: ${err.message}` }),
        {
          status: 502,
          headers: { "Content-Type": "application/json", ...corsHeaders },
        }
      );
    }
  },
};
