//! Port of the outbound HTTP helpers, backend.php:1901-2073 and 12667-12791:
//! `fd_http_json`, `fd_cached_wp_json`, `fd_fallback_url`,
//! `fd_is_connection_reset`, `fd_http_get_contents`,
//! `fd_http_fire_and_forget`, `fd_http_download_file`.
//!
//! Amp and cURL do not exist here; `reqwest` (rustls) is the single transport.
//! The behaviour that matters is preserved: JSON API + `X-API-Secret`, the
//! `telegra.my` DPI fallback, disk caching with a stale fallback, and a
//! timeout that never throws to the caller (returns `{}` like the PHP).

use std::collections::HashMap;
use std::sync::OnceLock;
use std::time::Duration;

use serde_json::Value;

use crate::config::{user_agent, wp_api_base, wp_api_base_fallback, WP_FALLBACK_HOST};
use crate::credentials::get_api_secret;
use crate::log::log;

pub fn client() -> &'static reqwest::Client {
    static C: OnceLock<reqwest::Client> = OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .user_agent(user_agent())
            .connect_timeout(Duration::from_secs(10))
            .pool_idle_timeout(Duration::from_secs(90))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

pub fn fallback_url(url: &str) -> Option<String> {
    let base = wp_api_base();
    if let Some(rest) = url.strip_prefix(&base) {
        return Some(format!("{}{}", wp_api_base_fallback(), rest));
    }
    if url.contains("pencarimovie.com") {
        return Some(url.replace("https://pencarimovie.com", &format!("https://{}", WP_FALLBACK_HOST)));
    }
    None
}

pub fn is_connection_reset(message: &str) -> bool {
    let s = message.to_ascii_lowercase();
    s.contains("econnreset")
        || s.contains("reset by peer")
        || s.contains("tls negotiation failed")
        || s.contains("socket hang up")
        || s.contains("etimedout")
        || s.contains("econnrefused")
        || s.contains("connection reset")
        // Added after the reqwest cause chain started being logged: a pooled
        // connection the peer already closed, a TLS handshake the middlebox ate,
        // and a body cut mid-stream all arrive as these.
        || s.contains("connection closed")
        || s.contains("connection aborted")
        || s.contains("incomplete message")
        || s.contains("body error")
        || s.contains("tcp connect error")
        || s.contains("dns error")
        || s.contains("error decoding response body")
        || s.contains("unexpected eof")
        || s.contains("broken pipe")
        // reqwest words a client timeout as `operation timed out`, so the PHP-era
        // `etimedout` needle above never matched it. Measured: every `hary*` search
        // went straight to a `status:0` failure with this text while the identical
        // query answered on a second attempt.
        || s.contains("timed out")
        || s.contains("timeout")
}

/// reqwest's `Display` for a send failure is ONLY
/// `error sending request for url (…)`; the reason — "connection reset by peer",
/// "tcp connect error", "dns error" — lives in the `source()` chain.
///
/// Logging the bare message made [`is_connection_reset()`] unable to classify ANY
/// transport failure, so the `telegra.my` DPI fallback never fired and a burst of
/// dashboard traffic just filled the log with `status:0` (measured: 34 lines in
/// two minutes, against pencarimovie.com AND aiometadata.elfhosted.com, while the
/// same URLs answered 200 to another client on the same machine).
fn err_text(e: &reqwest::Error) -> String {
    let mut out = e.to_string();
    let mut src: Option<&(dyn std::error::Error + 'static)> = std::error::Error::source(e);
    let mut depth = 0;
    while let Some(s) = src {
        let t = s.to_string();
        if !t.is_empty() && !out.contains(&t) {
            out.push_str(": ");
            out.push_str(&t);
        }
        src = s.source();
        depth += 1;
        if depth > 6 {
            break;
        }
    }
    out
}

async fn raw_get(url: &str, timeout_secs: u64, extra: &HashMap<String, String>) -> (bool, u16, String, Option<String>) {
    let mut req = client().get(url).timeout(Duration::from_secs(timeout_secs.max(1)));
    req = req.header("Accept", "*/*");
    for (k, v) in extra {
        req = req.header(k.as_str(), v.as_str());
    }
    match req.send().await {
        Ok(r) => {
            let status = r.status().as_u16();
            let ok = r.status().is_success();
            match r.text().await {
                Ok(t) => (ok, status, t, None),
                Err(e) => (false, status, String::new(), Some(err_text(&e))),
            }
        }
        Err(e) => (false, 0, String::new(), Some(err_text(&e))),
    }
}

/// `fd_http_get_contents()`: one request, one retry through `telegra.my` when
/// the failure looks like an in-path firewall reset, never panics.
pub async fn get_contents(url: &str, timeout_secs: u64, extra: HashMap<String, String>) -> Option<String> {
    let (ok, status, text, err) = raw_get(url, timeout_secs, &extra).await;
    if ok {
        return Some(text);
    }
    let mut err = err;
    // ONE same-host retry before the DPI fallback. A pooled connection the peer
    // has already closed fails exactly once and then works, which is the pattern
    // that emptied whole catalogs (`catalog built {"count":0}`) on a healthy host.
    if err.as_deref().map(is_connection_reset).unwrap_or(false) {
        log(
            "outbound request reset, retrying once",
            &serde_json::json!({"url": url}),
        );
        let (ok2, _s2, text2, err2) = raw_get(url, timeout_secs, &extra).await;
        if ok2 {
            return Some(text2);
        }
        err = err2;
    }
    if let Some(e) = &err {
        if is_connection_reset(e) {
            if let Some(alt) = fallback_url(url) {
                log("primary host failed, falling back to DPI proxy", &serde_json::json!({"url": url, "error": e}));
                let mut h = extra.clone();
                h.insert("Host".into(), "pencarimovie.com".into());
                let (ok2, _, text2, err2) = raw_get(&alt, timeout_secs, &h).await;
                if ok2 {
                    return Some(text2);
                }
                log("fallback host also failed", &serde_json::json!({"url": alt, "error": err2}));
            }
        }
    }
    log("http_get_contents failed", &serde_json::json!({"url": url, "status": status, "error": err}));
    None
}

fn encode_query(params: &[(String, String)]) -> String {
    let mut ser = url::form_urlencoded::Serializer::new(String::new());
    for (k, v) in params {
        ser.append_pair(k, v);
    }
    ser.finish()
}

/// `X-PM-Auth` plus a per-request proof, when this install holds a token.
///
/// This is the Option-A credential: the token names the install and the key it
/// is bound to, and the proof shows we still hold that key. A copied token is
/// inert — it cannot produce a valid proof from another machine.
pub(crate) fn pm_auth_headers(method: &str, url: &str) -> Vec<(String, String)> {
    let token = crate::credentials::auth_token();
    if token.is_empty() {
        return Vec::new();
    }
    // Origin-corrected: the proof's ±60 s window is checked against the ORIGIN's
    // clock, and a device can be hours off (measured: 8 h on a CPH2581).
    let ts = crate::credentials::now_corrected();
    let nonce = crate::credentials::new_nonce();
    // The signed endpoint must match the origin's `pm_auth_endpoint()`, which
    // takes the last segment of the matched REST route.
    let endpoint = crate::credentials::endpoint_of(url);
    let Some((proof, pubkey)) = crate::credentials::sign_proof(method, &endpoint, ts, &nonce) else {
        return Vec::new();
    };
    vec![
        ("X-PM-Auth".to_string(), token),
        ("X-PM-Sign-Pub".to_string(), pubkey),
        ("X-PM-Ts".to_string(), ts.to_string()),
        ("X-PM-Nonce".to_string(), nonce),
        ("X-PM-Proof".to_string(), proof),
    ]
}

/// True only for the ORIGIN's REST API — never for a third-party host.
///
/// MEASURED 2026-11-07 from `heroku logs -n 1500` on the live app, and it was
/// self-inflicted: the backfill SOURCE hosts answer 401 by design (the removed
/// fallback did it on every path with `bitchord_auth`), and the 401 branch below
/// did not look at the host. So each source attempt ran `clear_auth_token()`:
///
/// ```text
/// wp 401 with a token, re-enrolling {"url":"…/a/stream/28210785"}
/// pm-auth enrol skipped {"reason":"another attempt is in flight"}
/// ```
///
/// — the engine logged ITSELF out of the origin every time the source answered
/// 401, `ensure_enrolled_throttled()` then refused to recover faster than once a
/// minute, and dynos left holding no token answered `/resolve-file` and
/// `/search-music` with 401 (`web.3` did exactly that for minutes:
/// `unreadable /data/storage/auth_token.json`). The 401-by-design host is gone,
/// but the remaining source is third-party too, so the host check stays.
///
/// The same predicate stops the WORSE half, which the logs cannot show: the
/// install's `X-PM-Auth` token, signing pubkey and proof were being chained onto
/// every outgoing request, including the third-party source hosts. An install
/// credential has no business leaving for a host that is not the origin.
fn is_origin_api(url: &str) -> bool {
    let host_of = |u: &str| {
        url::Url::parse(u)
            .ok()
            .and_then(|p| p.host_str().map(|h| h.to_ascii_lowercase()))
    };
    match (host_of(url), host_of(&crate::config::wp_api_base())) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// `fd_http_json()`: GET (query string) or POST (JSON body) with X-API-Secret.
pub async fn http_json(url: &str, params: &[(String, String)], method: &str, timeout_secs: u64) -> Value {
    http_json_h(url, params, method, timeout_secs, &[]).await
}

/// [`http_json()`] plus caller headers.
///
/// Every call also carries the bot-scoped provision ticket when this install
/// holds one, so the protected routes authorise on a ticket that names one bot
/// and expires, rather than on the site-wide secret. Callers that need extra
/// headers pass them here — the lease advertises its X25519 public key this way.
pub async fn http_json_h(
    url: &str,
    params: &[(String, String)],
    method: &str,
    timeout_secs: u64,
    headers: &[(String, String)],
) -> Value {
    http_json_h_status(url, params, method, timeout_secs, headers).await.0
}

/// [`http_json_h()`] that ALSO reports the HTTP status.
///
/// `http_json()` collapses "the origin answered 503" and "the origin answered
/// `{"ok":0}`" into the same empty object — which is exactly why `/eclipse/stream`
/// reported a BUSY origin ("Server busy resolving file" while a Telegram relay
/// runs) as a track that does not exist. The music route needs the two apart, so
/// it uses this variant. `0` means the request never completed.
pub async fn http_json_h_status(
    url: &str,
    params: &[(String, String)],
    method: &str,
    timeout_secs: u64,
    headers: &[(String, String)],
) -> (Value, u16) {
    // PM-Auth self-heal, BEFORE the call that needs it.
    //
    // Boot enrolment alone is not enough on a phone: the engine is only started
    // when the user taps Start/Restart, so an APK update or a stale process could
    // be sitting there with no token — and a token minted on the previous boot
    // expires after 60 minutes anyway. Measured on a CPH2581: /api/resolve-shortcode
    // answered `{"ok":0}` forever because the resolve call carried no credential at
    // all. This makes every origin call enrol first when it has nothing to send.
    let mut target = url.to_string();
    // Origin call? Only then is a credential even meaningful. Sending it to a
    // third-party host leaks an install token for nothing, and asking to enrol
    // because a SOURCE answered 401 is what took the dynos offline on Heroku.
    let origin_call = is_origin_api(&target);
    if origin_call && crate::credentials::auth_token().is_empty() {
        crate::credentials::ensure_enrolled_throttled().await;
    }

    let secret = get_api_secret();
    let mut extra: Vec<(String, String)> = headers.iter().cloned().collect();
    if origin_call {
        extra.extend(crate::credentials::ticket_headers());
        extra.extend(pm_auth_headers(method, &target));
    }

    let mut body: Option<String> = None;
    if method.eq_ignore_ascii_case("GET") {
        let q = encode_query(params);
        if !q.is_empty() {
            target.push(if target.contains('?') { '&' } else { '?' });
            target.push_str(&q);
        }
    } else {
        let map: serde_json::Map<String, Value> = params
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        body = Some(Value::Object(map).to_string());
    }

    // Headers are passed IN, so the retry below can rebuild them after a re-enrol
    // (a closure that captured `extra` could not be used twice with new values).
    let call = |u: &str, extra: Vec<(String, String)>, secret: String, body: Option<String>| {
        let target = u.to_string();
        async move {
            let mut req = client()
                .request(
                    if method.eq_ignore_ascii_case("GET") {
                        reqwest::Method::GET
                    } else {
                        reqwest::Method::POST
                    },
                    &target,
                )
                .timeout(Duration::from_secs(timeout_secs.max(1)))
                .header("Accept", "application/json")
                .header("X-Requested-With", "XMLHttpRequest");
            if !secret.is_empty() {
                req = req.header("X-API-Secret", secret);
            }
            for (k, v) in &extra {
                req = req.header(k.as_str(), v.as_str());
            }
            if let Some(b) = body {
                req = req.header("Content-Type", "application/json").body(b);
            }
            match req.send().await {
                Ok(r) => {
                    let status = r.status().as_u16();
                    let ok = r.status().is_success();
                    match r.text().await {
                        Ok(t) => (ok, status, t, None::<String>),
                        Err(e) => (false, status, String::new(), Some(err_text(&e))),
                    }
                }
                Err(e) => (false, 0, String::new(), Some(err_text(&e))),
            }
        }
    };

    let had_auth = extra.iter().any(|(k, _)| k.eq_ignore_ascii_case("X-PM-Auth"));
    let (mut ok, mut status, mut text, mut err) =
        call(&target, extra.clone(), secret.clone(), body.clone()).await;

    // A 401 means the origin refused this request, and there are TWO cases — both
    // need the same recovery, and the second one used to get none:
    //
    //  1. we HELD a token and it is no longer accepted: the install is unknown or
    //     revoked, the origin's signing key was regenerated, or the proof fell
    //     outside the 60 s clock window. Without this, a stale token wedges the
    //     install forever (measured on a CPH2581: every /resolve-file answered 401
    //     while the UI showed a healthy session).
    //  2. we held NO token at all, because enrolment had not happened yet or its
    //     last attempt failed. `had_auth` was false, so this branch skipped — and
    //     that is the intermittent "streaming / api/download sometimes not working":
    //     the request went out unauthenticated, `/resolve-file` answered 401, the
    //     resolve found no file id, and `/api/download` gave up with
    //     `400 MISSING_FILE_ID`. Nothing re-enrolled, because nothing thought it
    //     should.
    //
    // One retry only: the retry's own status is not re-examined, so this can never
    // loop, and `ensure_enrolled_throttled()` still caps attempts at one a minute.
    // One clock probe per 10 minutes, process-wide.
    //
    // `measure_clock_offset()` is an extra round-trip to the origin, and a REVOKED
    // install 401s on every single request while the player retries its ranges —
    // probing per 401 meant one wasted request (radio wake, DNS, TLS) for each one,
    // which is exactly the kind of thing that shows up as battery drain. The clock
    // cannot drift meaningfully in 10 minutes, so this is free.
    static LAST_CLOCK_PROBE: std::sync::atomic::AtomicI64 =
        std::sync::atomic::AtomicI64::new(0);
    if status == 401 && origin_call {
        // Re-learn the clock BEFORE re-signing, in BOTH cases.
        //
        // A skewed clock is the one fault that makes an install look tokenless: a
        // token minted with the ORIGIN's expiry is discarded by `auth_token()` the
        // moment local time is past it, so `had_auth` is false on every request
        // forever. Probing only when a token was sent meant the correction could
        // never fire in exactly that state — the install re-enrolled once a minute,
        // threw each token away, and logged nothing but 401s (issue #14).
        let now_s = crate::storage::now();
        let probe_due = now_s - LAST_CLOCK_PROBE.load(std::sync::atomic::Ordering::SeqCst) > 600;
        if probe_due {
            LAST_CLOCK_PROBE.store(now_s, std::sync::atomic::Ordering::SeqCst);
            let probe = crate::session::measure_clock_offset().await;
            if probe.ok {
                crate::credentials::set_clock_offset(probe.offset);
                log(
                    "pm-auth clock corrected",
                    &serde_json::json!({"offset_secs": probe.offset, "had_token": had_auth}),
                );
            }
        }
        if had_auth {
            log("wp 401 with a token, re-enrolling", &serde_json::json!({"url": target}));
            crate::credentials::clear_auth_token();
        } else {
            // The reason is the whole point: "without a token" alone cannot tell a
            // read-only volume from a clock that ate the token from a missing file.
            log(
                "wp 401 without a token, enrolling",
                &serde_json::json!({
                    "url": target,
                    "reason": crate::credentials::auth_token_reason(),
                }),
            );
        }
        if crate::credentials::ensure_enrolled_throttled().await {
            extra = headers
                .iter()
                .cloned()
                .chain(crate::credentials::ticket_headers())
                .chain(pm_auth_headers(method, &target))
                .collect();
            let r = call(&target, extra.clone(), secret.clone(), body.clone()).await;
            ok = r.0;
            status = r.1;
            text = r.2;
            err = r.3;
        }
    }

    if !ok {
        if let Some(e) = &err {
            if is_connection_reset(e) {
                if let Some(alt) = fallback_url(&target) {
                    log("wp request reset, retrying via fallback host", &serde_json::json!({"url": target}));
                    let r = call(&alt, extra.clone(), secret.clone(), body.clone()).await;
                    ok = r.0;
                    status = r.1;
                    text = r.2;
                    err = r.3;
                }
            }
        }
    }
    // THE arming point for the origin-down gate.
    //
    // Every origin call outside the catalog path lands here — `resolve-file`,
    // `resolve-files`, the auth routes — and none of them went through
    // `cached_wp_json()`, which was the only place that called `wp_note_status()`.
    // So `status == 0` from a resolve recorded nothing, and the next thing to ask
    // "is the origin reachable?" got "yes" and burned another full timeout. Measured
    // on a phone (2026-11-02): `http_json failed … operation timed out`, then
    // `resolve returned no file id … status:0`, then `/api/download` answered
    // `400 MISSING_FILE_ID` — a client error for the origin's outage.
    wp_note_status(status);
    if !ok {
        log(
            "http_json failed",
            &serde_json::json!({"url": target, "status": status, "error": err}),
        );
        return (Value::Object(serde_json::Map::new()), status);
    }
    (
        serde_json::from_str(&text).unwrap_or_else(|_| Value::Object(serde_json::Map::new())),
        status,
    )
}

/// Is this a usable WordPress body (as opposed to an error envelope)?
fn truthy_ok(v: &Value) -> bool {
    v.get("ok")
        .map(|x| x.as_bool().unwrap_or(false) || x.as_i64().unwrap_or(0) != 0)
        .unwrap_or(false)
}

/// How long a stale WordPress body may still be served when a refresh fails.
const WP_STALE_GRACE: i64 = 21_600;

/// How long we are willing to WAIT for a fresh body when a stale copy already
/// exists.
///
/// A stale copy is always better than a hang: the viewer is staring at an empty
/// episode list. Measured on a real browse, `stream-files` for one episode timed
/// out twice in a row — 15 s + 400 ms + 15 s — and only THEN fell back to the
/// stale copy that was sitting in the cache the whole time. Three seconds covers
/// a healthy origin by a wide margin (a warm call measures ~0.4-0.8 s) and turns
/// the worst case for a known-but-slow title into 3 s instead of 30 s.
const WP_STALE_BUDGET_S: u64 = 3;

/// How long the origin is treated as DOWN after a call that never completed.
///
/// The episode path asks the origin TWICE before it can build a list — once for
/// `…/stream-files?title=X&id=…` and again for
/// `…/stream-files?season=1&episode=8&limit=250&title=X&id=…` — and each has its
/// own 15 s budget. Measured with the origin unreachable: 15 s + 15 s + the probe
/// ladder = **33-45 s per episode**, paid again for every episode the viewer opens.
///
/// A transport timeout is not a property of ONE url: it is the origin being gone.
/// So the first `status == 0` arms this gate and every later call fails fast
/// instead of re-discovering the same outage.
///
/// 45 s, and the reason is exact: a resolve ladder is bounded by
/// `resolve::RESOLVE_BUDGET` = **20 s**, and it is that ladder's LAST failed attempt
/// which arms this. The old 20 s therefore expired at the same instant the ladder
/// gave up, so the caller asking "was that an outage?" one line later always read
/// `0` — measured on 2026-11-02 against an unresolvable origin: `/api/download`
/// answered `400 MISSING_FILE_ID` for a request whose every hop had failed to
/// connect. The window must outlive the ladder that arms it; it is still short
/// enough that a blip recovers on its own.
const WP_DOWN_SECS: i64 = 45;

/// The refusal code for "the origin never answered".
///
/// One name, because the same outage reaches the user through three different
/// routes (`/api/download`, the guest provision, `/stream`), and three spellings of
/// one fault is what made it unfindable in the incident log.
pub const ORIGIN_UNREACHABLE: &str = "ORIGIN_UNREACHABLE";

/// One sentence for it too — host-free on purpose, because `wp_api_base()` is
/// env-overridable and a hard-coded domain would be a lie on a self-hosted origin.
pub const ORIGIN_UNREACHABLE_MSG: &str =
    "The catalog origin is unreachable. Retry in a few seconds.";

/// Until this instant the origin counts as down (`0` = healthy).
static WP_DOWN_UNTIL: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(0);

/// How long the origin is considered unreachable, in seconds (0 = healthy).
pub fn wp_origin_down_secs() -> i64 {
    let until = WP_DOWN_UNTIL.load(std::sync::atomic::Ordering::Relaxed);
    (until - crate::storage::now()).max(0)
}

/// Arm or clear the gate from a call's final status (`0` = never completed).
///
/// Public so a hand-rolled origin call elsewhere — the music search is one — keeps
/// the same bookkeeping instead of growing its own private copy.
pub fn wp_note_status(status: u16) {
    if status == 0 {
        note_wp_origin_down();
    } else {
        note_wp_origin_answered(status);
    }
}

fn note_wp_origin_down() {
    let until = crate::storage::now() + WP_DOWN_SECS;
    WP_DOWN_UNTIL.store(until, std::sync::atomic::Ordering::Relaxed);
    log(
        "wp origin marked down - later calls fail fast",
        &serde_json::json!({"for_secs": WP_DOWN_SECS}),
    );
}

/// Any completed answer clears the gate — including a real `{"ok":0}` miss, which
/// proves the origin is alive even though this particular query found nothing.
fn note_wp_origin_answered(status: u16) {
    if status != 0 && wp_origin_down_secs() > 0 {
        WP_DOWN_UNTIL.store(0, std::sync::atomic::Ordering::Relaxed);
        log("wp origin answering again - gate cleared", &serde_json::json!({}));
    }
}

/// Did the call fail to answer at all?
///
/// `http_json()` collapses a transport error, a timeout and every 5xx into an
/// EMPTY object. A real answer is never empty: a miss is `{"ok":0,"error":…}`
/// and a hit always carries `ok`. So only the empty shape is worth a retry —
/// retrying an honest 404/400 would double the work and change nothing.
fn needs_wp_retry(body: &Value) -> bool {
    body.as_object().map(|m| m.is_empty()).unwrap_or(false)
}

/// `fd_cached_wp_json()`: cache keyed on the FULL url, with single-flight and a
/// stale fallback so a slow origin never blocks playback. Only a truthy `ok` body
/// is stored, so an error response cannot poison the entry.
///
/// The old version had **no single-flight**: a burst of cold requests for the same
/// URL all opened their own WordPress connection (the PHP guarded this with a
/// `flock`). `fetch_cached` coalesces them into one call.
///
/// Two waits used to stack on top of each other here, and both are bounded now:
///
///   • **A timeout was retried at full length.** `needs_wp_retry()` cannot tell a
///     spent timeout from a fast 5xx, so a 15 s timeout bought another 15 s —
///     measured on `stream-files` for one episode, twice in a row. The status is
///     now checked ([`http_json_h_status`]): a retry only happens when the origin
///     ANSWERED (5xx); `status == 0` means the whole timeout was already spent,
///     and paying it again only delays the stale answer.
///   • **The stale copy was consulted last.** It sat in the cache while the
///     request burned 30 s. When a stale copy exists the fresh attempt is now
///     bounded by [`WP_STALE_BUDGET_S`], so the block is 3 s at worst.
pub async fn cached_wp_json(url: &str, ttl: i64, timeout_secs: u64) -> Value {
    let url_owned = url.to_string();
    // Decided OUTSIDE the single-flight closure: whether a usable old copy exists
    // is what sets the budget, and the closure runs once for every waiter.
    let has_stale = crate::cache::get_stale("wp_json", url).is_some();
    let budget = if has_stale {
        timeout_secs.min(WP_STALE_BUDGET_S)
    } else {
        timeout_secs
    };
    let body = crate::cache::fetch_cached("wp_json", url, ttl, WP_STALE_GRACE, move || async move {
        // The previous call already spent a full timeout on this origin: paying
        // another one here is what made a cold episode cost 30 s twice over. The
        // gate is armed by ANY transport failure, not by this particular url, so
        // the second `stream-files` of the same episode fails in microseconds.
        let down = wp_origin_down_secs();
        if down > 0 {
            log(
                "wp origin down - failing fast",
                &serde_json::json!({"url": url_owned, "for_secs": down}),
            );
            return (Value::Object(serde_json::Map::new()), false);
        }
        let (mut fresh, mut status) = http_json_h_status(&url_owned, &[], "GET", budget, &[]).await;
        // Retry ONLY a failure the origin actually answered. `status == 0` is a
        // transport error or a timeout: the budget is gone, and the old code spent
        // a second full timeout on it (`wp request empty, retrying once`) before
        // the stale fallback ever ran.
        if needs_wp_retry(&fresh) && status != 0 {
            log(
                "wp request empty, retrying once",
                &serde_json::json!({"url": url_owned, "status": status}),
            );
            tokio::time::sleep(Duration::from_millis(400)).await;
            let again = http_json_h_status(&url_owned, &[], "GET", budget, &[]).await;
            fresh = again.0;
            status = again.1;
        } else if needs_wp_retry(&fresh) {
            // Worth a line of its own: this is the case that used to double the
            // wait, and its frequency is the signal that the origin is unhealthy.
            log(
                "wp request timed out - not retrying, serving stale if held",
                &serde_json::json!({"url": url_owned, "budget_secs": budget}),
            );
        }
        // Arm or clear the gate from the FINAL outcome, so the next request knows
        // whether the origin is worth waiting on.
        wp_note_status(status);
        let cacheable = truthy_ok(&fresh);
        (fresh, cacheable)
    })
    .await;
    if !truthy_ok(&body) {
        if let Some(stale) = crate::cache::get_stale("wp_json", url) {
            log(
                "cached wp json served stale",
                &serde_json::json!({"url": url}),
            );
            return (*stale).clone();
        }
    }
    body
}

/// `fd_http_fire_and_forget()`: opens its own connection and returns at once;
/// the server-side work runs on another worker thread.
pub fn fire_and_forget(url: &str, params: &[(String, String)]) {
    let mut target = url.to_string();
    let q = encode_query(params);
    if !q.is_empty() {
        target.push(if target.contains('?') { '&' } else { '?' });
        target.push_str(&q);
    }
    let secret = get_api_secret();
    tokio::spawn(async move {
        let mut req = client().get(&target).timeout(Duration::from_secs(60));
        if !secret.is_empty() {
            req = req.header("X-API-Secret", secret);
        }
        let _ = req.send().await;
    });
}

/// `fd_http_download_file()`: resumable-ish file download to a path.
pub async fn download_file(url: &str, dest: &std::path::Path, timeout_secs: u64) -> bool {
    match client()
        .get(url)
        .timeout(Duration::from_secs(timeout_secs.max(1)))
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => match r.bytes().await {
            Ok(b) => std::fs::write(dest, &b).is_ok(),
            Err(_) => false,
        },
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only a DEAD call is retried. An honest miss (`ok:0` + error) and an honest
    /// empty list (`ok:1`, no items) are answers, and retrying them would double
    /// the origin work for nothing.
    #[test]
    fn only_a_dead_wp_call_is_retried() {
        assert!(needs_wp_retry(&serde_json::json!({})));
        assert!(!needs_wp_retry(&serde_json::json!({"ok": 0, "error": "Not found"})));
        assert!(!needs_wp_retry(&serde_json::json!({"ok": 1, "items": []})));
        assert!(!needs_wp_retry(&serde_json::json!({"ok": 1, "total": 0})));
        assert!(!needs_wp_retry(&serde_json::Value::Null));
    }

    /// The bug this closes: reqwest's `Display` alone is
    /// `error sending request for url (…)`, which matched NOTHING here, so the
    /// retry and the DPI fallback were dead code for every transport failure.
    /// The cause only arrives once `err_text()` walks the `source()` chain.
    #[test]
    fn a_reqwest_send_error_is_classified_only_with_its_cause_chain() {
        let bare = "error sending request for url (https://pencarimovie.com/wp-json/x)";
        assert!(!is_connection_reset(bare), "the bare Display carries no cause");

        for msg in [
            "error sending request for url (…): connection reset by peer (os error 10054)",
            "error sending request for url (…): tcp connect error: Connection refused (os error 111)",
            "error sending request for url (…): dns error: failed to lookup address information",
            "error decoding response body: connection closed before message completed",
            "error sending request for url (…): TLS handshake failed: unexpected eof",
            // reqwest's own wording; the PHP-era `etimedout` needle never matched it.
            "error sending request for url (…): operation timed out",
        ] {
            assert!(is_connection_reset(msg), "should be retried: {msg}");
        }
        // An HTTP answer is never a transport failure, whatever the status.
        assert!(!is_connection_reset("503 Service Unavailable"));
    }

    /// Only a request that never completed is an outage. A 404/401/500 is the
    /// origin talking, and telling the user "unreachable" for those would hide a
    /// real answer — `note_wp_origin_answered()` clears the gate on any of them.
    #[test]
    fn only_a_status_of_zero_arms_the_origin_gate() {
        wp_note_status(200);
        assert_eq!(wp_origin_down_secs(), 0, "healthy origin: gate closed");
        wp_note_status(404);
        assert_eq!(wp_origin_down_secs(), 0, "a 404 proves the origin is alive");
        wp_note_status(0);
        assert!(wp_origin_down_secs() > 0, "a transport failure arms the gate");
        wp_note_status(200);
        assert_eq!(wp_origin_down_secs(), 0, "first answer after an outage reopens it");
    }

    /// The regression that made the whole classification useless: the window was
    /// 20 s and `resolve::RESOLVE_BUDGET` is 20 s, so the gate armed by the ladder's
    /// last failed attempt expired at the instant the ladder gave up — every later
    /// `wp_origin_down_secs()` read 0 and `/api/download` answered 400.
    #[test]
    fn the_origin_down_window_outlives_a_resolve_ladder() {
        assert!(
            WP_DOWN_SECS > 20,
            "must outlive resolve::RESOLVE_BUDGET (20 s), or the caller reads 0"
        );
    }

    /// The Heroku regression (2026-11-07): a 401 from the hi-res fallback host
    /// cleared the origin token and started an enrolment the throttle then refused,
    /// so dynos ran with no credential and 401'd every origin call.
    #[test]
    fn only_the_origin_host_may_touch_the_install_credential() {
        let base = crate::config::wp_api_base();
        assert!(
            is_origin_api(&format!("{}/resolve-file?short_code=x", base)),
            "the origin's own API must be recognised"
        );
        assert!(
            is_origin_api(&format!("{}/search-music?search=x", base.trim_end_matches('/'))),
            "a query string must not change the verdict"
        );
        assert!(
            !is_origin_api("https://heisenbergbackaaahh.de.deplexo.com/a/search?q=x"),
            "a third-party source host is never the origin"
        );
        assert!(
            !is_origin_api("https://streaming-qobuz-std.akamaized.net/file?eid=1"),
            "nor is the CDN the signed master comes from"
        );
        assert!(!is_origin_api("not a url"), "garbage is not the origin");
    }
}
