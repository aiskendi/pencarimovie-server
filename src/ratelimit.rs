//! Per-client abuse limits on `/api/download`.
//!
//! Three counters, all switchable off (`0` = disabled, one integer compare, no map
//! touched), all keyed on a client identity resolved by [`client_key()`]:
//!
//! | counter | measures | lever |
//! | --- | --- | --- |
//! | `titles` | concurrent **distinct short codes** per client | scraper fan-out, hotlink sharing |
//! | `bodies` | concurrent **bodies** per client | socket flood backstop |
//! | `cold`   | **cold starts** per client per window | origin `/resolve-file` cost |
//!
//! ## Why `titles` and not just `bodies`
//!
//! Multi-connection throughput in this engine is INTERNAL to one HTTP response:
//! `pick_slot()` stripes windows across `--workers` Telegram connections and
//! `.buffered(pipeline)` keeps several in flight (see `download()` in
//! `download.rs`). One Play is one body, whatever those knobs are set to.
//!
//! But three legitimate behaviours DO produce more than one body for the same file:
//! a seek (the player abandons the old range and opens a new one), a segmented
//! download manager (aria2 defaults to 16 connections, IDM to 8), and a player
//! preloading the next episode. All three are **the same short code**, so counting
//! `(client, short_code)` pairs instead of raw bodies makes every one of them cost
//! exactly 1 — while 40 different titles in parallel still cost 40.
//!
//! `bodies` stays as a high backstop, deliberately above every real download
//! manager, for the "100 sockets against one code" case.
//!
//! ## Why `cold` is charged at the resolve site
//!
//! A player emits dozens of `Range` requests for one film. Charging them would
//! throttle seeking, so the budget is spent only where a resolve is actually about
//! to be issued (`download.rs`, after the cached-id block). A warm short code, a
//! seek and a replay therefore cost nothing; a catalogue walk pays per title.
//!
//! ## Scope and lifetime
//!
//! In-memory and therefore **per process**, like `resolve`, `wp_ajax_*` and the
//! catalog caches (AGENTS.md §9 trap 3). On a multi-dyno deployment the effective
//! limit is `cap x dynos`.
//!
//! A held slot is released by `Drop`, which runs when the STREAMING task ends. That
//! end is prompt but not instant: the body task notices a dropped client at its
//! `tx.is_closed()` check, which sits before the next Telegram window fetch, so a
//! cancelled body can hold its slot for up to one window. Counters must therefore be
//! sized for overlap, never at the edge of what one viewer legitimately opens.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use crate::auth::ReqCtx;

/// Past this many keys the cold-start map is swept. Bodies and titles cannot grow
/// past the number of live bodies, so they need no sweep.
const MAX_KEYS: usize = 4096;

/* ── client identity ─────────────────────────────────────────────────────── */

/// The identity the limiter counts against, or `None` to exempt the request.
///
/// Deliberately NOT [`crate::auth::auth_client_ip()`]: that one takes the FIRST
/// `x-forwarded-for` entry, which is the one the CLIENT writes, so a scraper
/// rotates the header and never hits a limit. It also feeds the login lockout, and
/// changing its meaning would change lockout behaviour.
///
/// Order:
/// 1. `cf-connecting-ip` when a `cf-*` header is present — Cloudflare sets it and a
///    client cannot forge it through the edge.
/// 2. the **rightmost** `x-forwarded-for` hop, and ONLY when a proxy is known to be
///    in front. PaaS routers append, so the rightmost entry is theirs; the leftmost
///    is attacker-controlled. Trusting it without a proxy would be a free bypass,
///    hence [`trust_xff()`] and the opt-in `PM_TRUST_XFF=1` for a self-hosted nginx.
/// 3. `ctx.remote` from `ConnectInfo` — unforgeable, and correct for every instance
///    that is not behind a proxy.
///
/// IPv6 collapses to its `/64`: one host otherwise rotates `/128`s for free.
///
/// ## Local clients are exempt
///
/// [`crate::auth::is_local_request()`] decides, and it is the right predicate
/// because it already refuses to classify a proxied request as local:
///
/// - loopback — the local dashboard and the Eclipse web client both play
///   `127.0.0.1:8088`, so limiting them would break the product's own UI;
/// - the LAN — a home server is normally watched by a TV, a phone and a laptop on
///   the same Wi-Fi. Counting a household against a per-IP cap is all cost and no
///   protection: the abuser worth stopping is on the public side of a tunnel;
/// - link-local, and carrier-grade NAT (`100.64/10`). CGNAT is NOT a home network,
///   but every subscriber behind one carrier shares a single address, so limiting it
///   would let one abuser lock out thousands of mobile users.
///
/// A tunnelled request is never exempt: `is_cloudflare_tunnel_request()` is checked
/// first inside `is_local_request()`, so cloudflared dialling us from `127.0.0.1`
/// still counts (and is keyed by `cf-connecting-ip`).
pub fn client_key(ctx: &ReqCtx) -> Option<String> {
    if crate::auth::is_local_request(ctx) {
        return None;
    }
    if !ctx.hdr("cf-ray").is_empty() || !ctx.hdr("cf-connecting-ip").is_empty() {
        if let Some(k) = key_of(&ctx.hdr("cf-connecting-ip")) {
            return Some(k);
        }
    }
    if trust_xff(ctx) {
        let fwd = ctx.hdr("x-forwarded-for");
        if let Some(last) = fwd
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .next_back()
        {
            if let Some(k) = key_of(last) {
                return Some(k);
            }
        }
    }
    if ctx.remote.is_empty() {
        return None;
    }
    key_of(&ctx.remote)
}

/// Is a reverse proxy known to be in front of us, so `x-forwarded-for` can be read?
///
/// `PM_TRUST_XFF=1` opts in explicitly (a self-hosted nginx on a custom domain).
/// Otherwise only the known PaaS hosts count, because those routers set the header
/// themselves. Anything else means the client could have written it.
fn trust_xff(ctx: &ReqCtx) -> bool {
    if storage_flag("PM_TRUST_XFF") {
        return true;
    }
    for raw in [ctx.hdr("host"), ctx.hdr("x-forwarded-host")] {
        for part in raw.split(',') {
            let h = part.trim().to_ascii_lowercase();
            let h = h.split(':').next().unwrap_or("");
            if h.ends_with(".herokuapp.com")
                || h == "herokuapp.com"
                || h.ends_with(".up.railway.app")
                || h.ends_with(".onrender.com")
                || h.ends_with(".fly.dev")
            {
                return true;
            }
        }
    }
    false
}

fn storage_flag(name: &str) -> bool {
    match crate::storage::env(name) {
        Some(v) => {
            let v = v.trim().to_ascii_lowercase();
            v == "1" || v == "true" || v == "on" || v == "yes"
        }
        None => false,
    }
}

/// Normalise one address string into a counting key.
fn key_of(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    match raw.parse::<IpAddr>() {
        Ok(IpAddr::V4(v4)) => Some(format!("v4:{v4}")),
        Ok(IpAddr::V6(v6)) => {
            // `::ffff:203.0.113.5` is an IPv4 client on a dual-stack socket: it must
            // land in the same bucket as the plain IPv4 form.
            if let Some(v4) = v6.to_ipv4_mapped() {
                return Some(format!("v4:{v4}"));
            }
            let s = v6.segments();
            Some(format!("v6:{:x}:{:x}:{:x}:{:x}::/64", s[0], s[1], s[2], s[3]))
        }
        Err(_) => None,
    }
}

/* ── refusal ─────────────────────────────────────────────────────────────── */

/// Why a request was refused. Rendered as `429` + `Retry-After` by the caller —
/// never `403`, never `503`, never `404`: a client reads those as fatal and never
/// retries (AGENTS.md: "a rate limit must answer 429").
#[derive(Debug)]
pub struct Refusal {
    pub error: &'static str,
    pub message: &'static str,
    pub retry_after: u64,
}

/* ── the limiter ─────────────────────────────────────────────────────────── */

/// `pairs` counts bodies per `(client, short_code)`; `per_key` counts DISTINCT short
/// codes per client.
///
/// Both live under ONE lock on purpose: counting the distinct titles of a client by
/// scanning `pairs` would be O(every client streaming anywhere) **inside a global
/// mutex**, which is the only cost here that could show up under load. Two counters
/// make the check O(1).
#[derive(Default, Debug)]
struct Titles {
    pairs: HashMap<(String, String), u32>,
    per_key: HashMap<String, u32>,
}

#[derive(Default, Debug)]
pub struct Limiter {
    /// client key -> bodies currently open
    bodies: Mutex<HashMap<String, u32>>,
    /// Distinct titles in flight per client, plus the per-pair body count that
    /// decides whether an arriving request is a NEW title or a repeat of one.
    titles: Mutex<Titles>,
    /// client key -> GCRA theoretical arrival time (unix seconds)
    cold: Mutex<HashMap<String, f64>>,
}

impl Limiter {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Take a slot for `(client, short_code)`. `max_titles` / `max_bodies` of `0`
    /// disable that counter.
    ///
    /// The returned guard decrements on `Drop`, so it must travel with the response
    /// body (see `download.rs`): released earlier, it would cap requests instead of
    /// live streams.
    pub fn acquire(
        self: &Arc<Self>,
        key: &str,
        short_code: &str,
        max_titles: usize,
        max_bodies: usize,
    ) -> Result<Guard, Refusal> {
        if max_titles == 0 && max_bodies == 0 {
            return Ok(Guard::disabled());
        }

        // Bodies first: it is the cheaper, coarser check and reads as "this address
        // has too many sockets", which is the honest message for a flood.
        if max_bodies > 0 {
            let mut map = match self.bodies.lock() {
                Ok(m) => m,
                Err(p) => p.into_inner(),
            };
            let n = map.get(key).copied().unwrap_or(0);
            if n as usize >= max_bodies {
                return Err(Refusal {
                    error: "IP_BODY_LIMIT",
                    message: "Too many simultaneous connections from this address.",
                    retry_after: 2,
                });
            }
            *map.entry(key.to_string()).or_insert(0) += 1;
        }

        if max_titles > 0 {
            let mut t = match self.titles.lock() {
                Ok(m) => m,
                Err(p) => p.into_inner(),
            };
            let pair = (key.to_string(), short_code.to_string());
            match t.pairs.get_mut(&pair) {
                // The PAIR already has a body open: this is a seek, a segmented
                // download or a preload of the SAME file. It costs no new title.
                Some(n) => *n += 1,
                None => {
                    let open = t.per_key.get(key).copied().unwrap_or(0);
                    if open as usize >= max_titles {
                        drop(t);
                        // Undo the body slot this request already took, or a client
                        // that keeps retrying would exhaust its own body budget.
                        if max_bodies > 0 {
                            self.release(key);
                        }
                        return Err(Refusal {
                            error: "IP_STREAM_LIMIT",
                            message: "Too many different titles streaming from this address.",
                            retry_after: 2,
                        });
                    }
                    t.pairs.insert(pair, 1);
                    t.per_key.insert(key.to_string(), open + 1);
                }
            }
        }

        Ok(Guard {
            lim: Some(self.clone()),
            key: key.to_string(),
            code: short_code.to_string(),
            counted_bodies: max_bodies > 0,
            counted_title: max_titles > 0,
        })
    }

    fn release(&self, key: &str) {
        let mut map = match self.bodies.lock() {
            Ok(m) => m,
            Err(p) => p.into_inner(),
        };
        if let Some(n) = map.get_mut(key) {
            *n = n.saturating_sub(1);
            if *n == 0 {
                map.remove(key);
            }
        }
    }

    /// Spend one cold-start from `key`'s budget.
    ///
    /// GCRA: one `f64` per key, one compare-and-swap, no allocation on the hit path.
    /// `burst` starts are allowed back to back, then one every `window / burst`
    /// seconds. Returns the number of seconds to wait when the budget is spent.
    pub fn check_cold(&self, key: &str, burst: usize, window_secs: u64) -> Result<(), u64> {
        if burst == 0 || window_secs == 0 {
            return Ok(());
        }
        let now = crate::storage::now() as f64;
        let interval = window_secs as f64 / burst as f64;
        let tolerance = interval * (burst as f64 - 1.0);

        let mut map = match self.cold.lock() {
            Ok(m) => m,
            Err(p) => p.into_inner(),
        };
        if map.len() >= MAX_KEYS {
            // Amortised sweep: a key whose arrival time is in the past is idle and
            // rebuilds in one hit. This is the eviction `tower-governor`'s README
            // makes you spawn a thread for; inline is enough at this size.
            map.retain(|_, tat| *tat - tolerance > now);
        }
        let tat = map.get(key).copied().unwrap_or(now);
        let allowed_at = tat.max(now);
        let earliest = allowed_at - tolerance;
        if earliest > now {
            let wait = (earliest - now).ceil() as u64;
            return Err(wait.max(1));
        }
        map.insert(key.to_string(), allowed_at + interval);
        Ok(())
    }
}

/// Release-on-drop slot. Holds nothing when the counters are disabled, so a `0`
/// install pays for one struct and no lock.
#[derive(Debug)]
pub struct Guard {
    lim: Option<Arc<Limiter>>,
    key: String,
    code: String,
    counted_bodies: bool,
    counted_title: bool,
}

impl Guard {
    fn disabled() -> Self {
        Guard {
            lim: None,
            key: String::new(),
            code: String::new(),
            counted_bodies: false,
            counted_title: false,
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        let Some(lim) = self.lim.take() else {
            return;
        };
        if self.counted_bodies {
            lim.release(&self.key);
        }
        if self.counted_title {
            let mut t = match lim.titles.lock() {
                Ok(m) => m,
                Err(p) => p.into_inner(),
            };
            let pair = (self.key.clone(), self.code.clone());
            let last = match t.pairs.get_mut(&pair) {
                Some(n) => {
                    *n = n.saturating_sub(1);
                    *n == 0
                }
                None => false,
            };
            if last {
                t.pairs.remove(&pair);
                let drop_key = match t.per_key.get_mut(&self.key) {
                    Some(c) => {
                        *c = c.saturating_sub(1);
                        *c == 0
                    }
                    None => false,
                };
                if drop_key {
                    t.per_key.remove(&self.key);
                }
            }
        }
    }
}

/* ── tests ───────────────────────────────────────────────────────────────── */

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    fn ctx_with(remote: &str, headers: &[(&str, &str)]) -> ReqCtx {
        let mut map = HeaderMap::new();
        for (k, v) in headers {
            map.insert(
                axum::http::HeaderName::from_bytes(k.as_bytes()).unwrap(),
                axum::http::HeaderValue::from_str(v).unwrap(),
            );
        }
        ReqCtx {
            method: "GET".to_string(),
            path: "/api/download".to_string(),
            raw_path: "/api/download".to_string(),
            query: Vec::new(),
            headers: map,
            cookies: HashMap::new(),
            remote: remote.to_string(),
            local: String::new(),
            base_url: String::new(),
        }
    }

    fn ctx(remote: &str) -> ReqCtx {
        ctx_with(remote, &[])
    }

    #[test]
    fn ipv4_key_is_stable() {
        assert_eq!(key_of("203.0.113.5").as_deref(), Some("v4:203.0.113.5"));
        assert_eq!(key_of(" 203.0.113.5 ").as_deref(), Some("v4:203.0.113.5"));
        assert_eq!(key_of(""), None);
        assert_eq!(key_of("not-an-ip"), None);
    }

    #[test]
    fn ipv6_clients_are_grouped_by_slash_64() {
        let a = key_of("2001:db8:1:2:aaaa:bbbb:cccc:dddd").unwrap();
        let b = key_of("2001:db8:1:2:1111:2222:3333:4444").unwrap();
        assert_eq!(a, b, "two /128s inside one /64 must share a bucket");
        let c = key_of("2001:db8:1:3::1").unwrap();
        assert_ne!(a, c, "a different /64 is a different client");
    }

    #[test]
    fn ipv4_mapped_ipv6_shares_the_ipv4_bucket() {
        assert_eq!(
            key_of("::ffff:203.0.113.5").as_deref(),
            Some("v4:203.0.113.5")
        );
    }

    #[test]
    fn loopback_is_exempt() {
        assert_eq!(client_key(&ctx("127.0.0.1")), None);
        assert_eq!(client_key(&ctx("::1")), None);
    }

    #[test]
    fn lan_clients_are_exempt() {
        // A household on one Wi-Fi must never contend with itself: phone, TV and
        // laptop are all the same address as far as the public internet is
        // concerned, and counting them is all cost and no protection.
        assert_eq!(client_key(&ctx("192.168.1.20")), None);
        assert_eq!(client_key(&ctx("10.0.0.5")), None);
        assert_eq!(client_key(&ctx("172.16.4.9")), None);
        // Link-local and carrier-grade NAT, which `is_local_request` also treats as
        // non-public. CGNAT is not a home network, but its subscribers share ONE
        // address, so limiting it would let one abuser lock out thousands.
        assert_eq!(client_key(&ctx("169.254.10.1")), None);
        assert_eq!(client_key(&ctx("100.64.7.7")), None);
        assert_eq!(client_key(&ctx("fe80::1")), None);
    }

    #[test]
    fn a_tunnelled_loopback_peer_is_still_limited() {
        // cloudflared dials us from 127.0.0.1. That must NOT read as "local", or a
        // public visitor through the tunnel would be exempt from every counter.
        let c = ctx_with(
            "127.0.0.1",
            &[("cf-ray", "abc123"), ("cf-connecting-ip", "203.0.113.9")],
        );
        assert_eq!(client_key(&c).as_deref(), Some("v4:203.0.113.9"));
    }

    #[test]
    fn a_public_client_is_still_limited() {
        assert!(client_key(&ctx("198.51.100.7")).is_some());
        assert!(client_key(&ctx("2001:db8::1")).is_some());
    }

    #[test]
    fn a_direct_client_is_keyed_by_its_unforgeable_peer_address() {
        // No proxy signal, so a spoofed XFF must be IGNORED.
        let c = ctx_with(
            "198.51.100.7",
            &[("x-forwarded-for", "1.2.3.4, 5.6.7.8")],
        );
        assert_eq!(client_key(&c).as_deref(), Some("v4:198.51.100.7"));
    }

    #[test]
    fn xff_is_read_from_the_right_hand_end() {
        // A PaaS host means the router appended the real client last.
        let c = ctx_with(
            "10.0.0.1",
            &[
                ("host", "app.herokuapp.com"),
                ("x-forwarded-for", "1.2.3.4, 9.9.9.9"),
            ],
        );
        assert_eq!(
            client_key(&c).as_deref(),
            Some("v4:9.9.9.9"),
            "the leftmost hop is client-supplied and must not become the key"
        );
    }

    #[test]
    fn cloudflare_connecting_ip_wins() {
        let c = ctx_with(
            "127.0.0.1",
            &[
                ("cf-ray", "abc123"),
                ("cf-connecting-ip", "203.0.113.9"),
                ("x-forwarded-for", "1.2.3.4"),
            ],
        );
        assert_eq!(client_key(&c).as_deref(), Some("v4:203.0.113.9"));
    }

    #[test]
    fn sixteen_bodies_of_one_short_code_count_as_one_title() {
        // The segmented-download / seek pin: aria2 -x16 on one URL must never be
        // mistaken for sixteen titles.
        let lim = Limiter::new();
        let mut guards = Vec::new();
        for _ in 0..16 {
            guards.push(lim.acquire("v4:203.0.113.5", "AbCdEf", 8, 32).unwrap());
        }
        let t = lim.titles.lock().unwrap();
        assert_eq!(t.per_key.get("v4:203.0.113.5"), Some(&1));
        assert_eq!(
            t.pairs
                .get(&("v4:203.0.113.5".to_string(), "AbCdEf".to_string())),
            Some(&16)
        );
    }

    #[test]
    fn releasing_a_title_frees_a_slot() {
        // The distinct-title counter must go back down on Drop, or a player that
        // changes film would run out of slots after 8 titles in one session.
        let lim = Limiter::new();
        for round in 0..20 {
            let code = format!("code{round}");
            let g = lim.acquire("v4:203.0.113.5", &code, 1, 0).unwrap();
            drop(g);
        }
        assert!(lim.titles.lock().unwrap().per_key.is_empty());
        assert!(lim.acquire("v4:203.0.113.5", "final", 1, 0).is_ok());
    }

    #[test]
    fn distinct_titles_from_one_client_are_refused_at_the_cap() {
        let lim = Limiter::new();
        let mut guards = Vec::new();
        for i in 0..8 {
            guards.push(lim.acquire("v4:203.0.113.5", &format!("code{i}"), 8, 0).unwrap());
        }
        let err = lim.acquire("v4:203.0.113.5", "code8", 8, 0).unwrap_err();
        assert_eq!(err.error, "IP_STREAM_LIMIT");
        assert!(err.retry_after >= 1);
    }

    #[test]
    fn a_second_client_is_unaffected_by_the_first() {
        let lim = Limiter::new();
        let _a = lim.acquire("v4:203.0.113.5", "code0", 1, 0).unwrap();
        assert!(lim.acquire("v4:203.0.113.6", "code0", 1, 0).is_ok());
    }

    #[test]
    fn guard_releases_on_drop_and_removes_the_key_at_zero() {
        let lim = Limiter::new();
        {
            let _g = lim.acquire("v4:203.0.113.5", "code0", 2, 2).unwrap();
            assert_eq!(lim.bodies.lock().unwrap().get("v4:203.0.113.5"), Some(&1));
        }
        assert!(
            lim.bodies.lock().unwrap().is_empty(),
            "a released key must not linger"
        );
        let t = lim.titles.lock().unwrap();
        assert!(t.pairs.is_empty() && t.per_key.is_empty());
    }

    #[test]
    fn a_refused_title_does_not_leak_a_body_slot() {
        // The body counter is incremented first, so the title refusal MUST undo it or
        // a client that keeps trying would exhaust its own body budget.
        let lim = Limiter::new();
        let _held = lim.acquire("v4:203.0.113.5", "code0", 1, 4).unwrap();
        let err = lim.acquire("v4:203.0.113.5", "code1", 1, 4).unwrap_err();
        assert_eq!(err.error, "IP_STREAM_LIMIT");
        let bodies = lim.bodies.lock().unwrap().get("v4:203.0.113.5").copied();
        assert_eq!(bodies, Some(1), "the refused request must not hold a body");
    }

    #[test]
    fn the_body_backstop_allows_a_real_download_manager() {
        let lim = Limiter::new();
        let mut guards = Vec::new();
        // 16 parallel segments of ONE code: under both caps, both counters pass.
        for _ in 0..16 {
            guards.push(lim.acquire("v4:203.0.113.5", "code0", 8, 32).unwrap());
        }
        assert_eq!(lim.bodies.lock().unwrap().get("v4:203.0.113.5"), Some(&16));
    }

    #[test]
    fn the_body_backstop_refuses_above_its_cap() {
        let lim = Limiter::new();
        let mut guards = Vec::new();
        for _ in 0..32 {
            guards.push(lim.acquire("v4:203.0.113.5", "code0", 0, 32).unwrap());
        }
        let err = lim.acquire("v4:203.0.113.5", "code0", 0, 32).unwrap_err();
        assert_eq!(err.error, "IP_BODY_LIMIT");
    }

    #[test]
    fn cap_zero_touches_no_map() {
        let lim = Limiter::new();
        let g = lim.acquire("v4:203.0.113.5", "code0", 0, 0).unwrap();
        assert!(lim.bodies.lock().unwrap().is_empty());
        assert!(lim.titles.lock().unwrap().pairs.is_empty());
        assert!(lim.titles.lock().unwrap().per_key.is_empty());
        assert!(lim.check_cold("v4:203.0.113.5", 0, 300).is_ok());
        drop(g);
        assert!(lim.cold.lock().unwrap().is_empty());
    }

    #[test]
    fn cold_start_budget_allows_the_burst_then_refuses() {
        let lim = Limiter::new();
        for i in 0..20 {
            assert!(
                lim.check_cold("v4:203.0.113.5", 20, 300).is_ok(),
                "hit {i} must be inside the burst"
            );
        }
        let wait = lim.check_cold("v4:203.0.113.5", 20, 300).unwrap_err();
        assert!(
            (1..=300).contains(&wait),
            "retry_after must be positive and inside the window, got {wait}"
        );
    }

    #[test]
    fn cold_start_budget_is_per_client() {
        let lim = Limiter::new();
        for _ in 0..5 {
            let _ = lim.check_cold("v4:203.0.113.5", 5, 300);
        }
        assert!(lim.check_cold("v4:203.0.113.5", 5, 300).is_err());
        assert!(lim.check_cold("v4:203.0.113.6", 5, 300).is_ok());
    }

}
