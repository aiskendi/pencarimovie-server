//! api_id lanes: several Telegram "applications" in ONE process.
//!
//! The measurement that motivates this module (VPS, 2026-11-26, four processes):
//! one api_id tops out around 5.7-11 MB/s whatever the socket count, two bots on
//! the SAME api_id do not add up (5.74 MB/s combined, each halved), and four
//! DISTINCT api_ids in four processes reached 37.9 MB/s total. So the ceiling is
//! per `api_id`, and the only way one `/api/download` response can pass it is to
//! fetch its parts through several api_ids at once.
//!
//! A **lane** is `(api_id, bot)`: one `TelegramPool` holding connections that
//! were opened with that api_id, and one bot signed in on it. Lanes are
//! independent — their own connections, their own `file_reference` per file,
//! their own flood record — which is exactly why adding one adds throughput.
//!
//! An install with a single api_id (every install before this module) gets one
//! lane that IS the process pool, so nothing about its behaviour changes; the
//! dispatcher in `download.rs` only runs when there are two or more.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use serde_json::Value;

use crate::log::log;
use crate::storage::{now, storage_path};
use crate::telegram::TelegramPool;

pub const API_IDS_FILE: &str = "api_ids.json";

/* ── the api_id pool on disk ─────────────────────────────────────────────── */

/// One api_id in `storage/api_ids.json`.
///
/// The hash is a secret in the same sense as a session file: it is never logged,
/// never returned by a route (see [`public_rows()`]) and never written into a
/// commit. Only the pool file and the connection that uses it see it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApiIdEntry {
    pub api_id: i32,
    pub api_hash: String,
    /// `file`, `origin` or `legacy` — where the row came from, for the UI.
    pub source: String,
    pub enabled: bool,
    pub added_at: i64,
    /// The guest bot this lane is using, if it has one. Empty means the lane is
    /// configured but has not been provisioned yet.
    pub bot_id: String,
}

impl ApiIdEntry {
    pub fn new(api_id: i32, api_hash: &str, source: &str) -> Self {
        Self {
            api_id,
            api_hash: api_hash.trim().to_string(),
            source: source.to_string(),
            enabled: true,
            added_at: now(),
            bot_id: String::new(),
        }
    }

    fn valid(&self) -> bool {
        self.api_id > 0 && !self.api_hash.is_empty()
    }
}

pub fn api_ids_path() -> PathBuf {
    storage_path(API_IDS_FILE)
}

/// The hash as a route may show it: enough to tell two rows apart, not enough to
/// use. `abcd…1234`.
pub fn masked_hash(hash: &str) -> String {
    let h = hash.trim();
    if h.is_empty() {
        return String::new();
    }
    let n = h.chars().count();
    if n <= 8 {
        return "…".to_string();
    }
    let head: String = h.chars().take(4).collect();
    let tail: String = h.chars().skip(n - 4).collect();
    format!("{head}…{tail}")
}

fn entry_from_value(v: &Value) -> Option<ApiIdEntry> {
    let api_id = v.get("api_id").and_then(|x| x.as_i64()).unwrap_or(0) as i32;
    let api_hash = v.get("api_hash").and_then(|x| x.as_str()).unwrap_or("");
    let mut e = ApiIdEntry::new(api_id, api_hash, "file");
    if let Some(s) = v.get("source").and_then(|x| x.as_str()) {
        if !s.trim().is_empty() {
            e.source = s.trim().to_string();
        }
    }
    e.enabled = v.get("enabled").and_then(|x| x.as_bool()).unwrap_or(true);
    e.added_at = v.get("added_at").and_then(|x| x.as_i64()).unwrap_or(0);
    e.bot_id = v
        .get("bot_id")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if !e.valid() {
        return None;
    }
    Some(e)
}

pub fn value_from_entry(e: &ApiIdEntry) -> Value {
    serde_json::json!({
        "api_id": e.api_id,
        "api_hash": e.api_hash,
        "source": e.source,
        "enabled": e.enabled,
        "added_at": e.added_at,
        "bot_id": e.bot_id,
    })
}

/// Parse a pool document. Accepts the documented object (`{"api_ids":[…]}`) and
/// a bare array, because an operator hand-editing the file will try both.
pub fn parse_entries(text: &str) -> Vec<ApiIdEntry> {
    let Ok(v) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    let list = match &v {
        Value::Array(a) => Some(a.clone()),
        Value::Object(o) => o.get("api_ids").and_then(|x| x.as_array()).cloned(),
        _ => None,
    }
    .unwrap_or_default();
    let mut out: Vec<ApiIdEntry> = Vec::new();
    for item in list.iter() {
        let Some(e) = entry_from_value(item) else {
            continue;
        };
        // One row per api_id: a duplicate is a hand-edit mistake, and two lanes
        // on one api_id would divide its ceiling instead of adding to it.
        if out.iter().any(|x| x.api_id == e.api_id) {
            continue;
        }
        out.push(e);
    }
    out
}

/// The rows on disk only. `[]` when the file is absent or unreadable.
pub fn read_entries() -> Vec<ApiIdEntry> {
    std::fs::read_to_string(api_ids_path())
        .ok()
        .map(|t| parse_entries(&t))
        .unwrap_or_default()
}

pub fn save_entries(entries: &[ApiIdEntry]) -> Result<(), String> {
    let body = serde_json::json!({
        "api_ids": entries.iter().map(value_from_entry).collect::<Vec<Value>>(),
    });
    std::fs::write(api_ids_path(), body.to_string())
        .map_err(|e| format!("could not write {}: {}", api_ids_path().display(), e))
}

/// The pool as the engine uses it: the file's rows, with the legacy single
/// credential (`storage/api_credentials.json` or `FD_API_ID`) prepended as the
/// default lane unless the file already names that api_id.
///
/// This order is the compatibility promise: an install that never created
/// `api_ids.json` behaves exactly like the single-api_id build, and an install
/// that did keeps the operator's own credential as lane 1.
pub fn load_entries() -> Vec<ApiIdEntry> {
    let mut rows = read_entries();
    if let Some((id, hash)) = crate::credentials::load_cached_api_credentials() {
        if !rows.iter().any(|e| e.api_id == id) {
            rows.insert(0, ApiIdEntry::new(id, &hash, "legacy"));
        }
    }
    rows
}

pub fn enabled_specs() -> Vec<(i32, String)> {
    load_entries()
        .into_iter()
        .filter(|e| e.enabled && e.valid())
        .map(|e| (e.api_id, e.api_hash))
        .collect()
}

pub fn add_entry(api_id: i32, api_hash: &str) -> Result<ApiIdEntry, String> {
    let hash = api_hash.trim();
    if api_id <= 0 {
        return Err("api_id must be a positive integer".to_string());
    }
    if hash.len() < 16 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("api_hash must be the 32-character hex string Telegram issued".to_string());
    }
    let mut rows = read_entries();
    if rows.iter().any(|e| e.api_id == api_id) {
        return Err(format!("api_id {} is already in the pool", api_id));
    }
    let entry = ApiIdEntry::new(api_id, hash, "file");
    rows.push(entry.clone());
    save_entries(&rows)?;
    Ok(entry)
}

pub fn remove_entry(api_id: i32) -> Result<bool, String> {
    let mut rows = read_entries();
    let before = rows.len();
    rows.retain(|e| e.api_id != api_id);
    if rows.len() == before {
        return Ok(false);
    }
    save_entries(&rows)?;
    Ok(true)
}

pub fn set_enabled(api_id: i32, enabled: bool) -> Result<bool, String> {
    let mut rows = read_entries();
    let mut hit = false;
    for e in rows.iter_mut().filter(|e| e.api_id == api_id) {
        e.enabled = enabled;
        hit = true;
    }
    if !hit {
        return Ok(false);
    }
    save_entries(&rows)?;
    Ok(true)
}

/// Record which bot a lane signed in as. Best effort: a lane that cannot write
/// this still works for the life of the process.
pub fn set_lane_bot(api_id: i32, bot_id: &str) {
    let mut rows = read_entries();
    // A legacy row is not in the file yet; materialise it so the bot survives a
    // restart instead of being re-leased every boot.
    if rows.is_empty() {
        rows = load_entries();
    }
    let mut hit = false;
    for e in rows.iter_mut().filter(|e| e.api_id == api_id) {
        e.bot_id = bot_id.trim().to_string();
        hit = true;
    }
    if !hit {
        return;
    }
    let _ = save_entries(&rows);
}

/// Route shape: the hash is masked, the bot id is not a secret.
pub fn public_rows() -> Vec<Value> {
    load_entries()
        .iter()
        .map(|e| {
            serde_json::json!({
                "api_id": e.api_id,
                "api_hash": masked_hash(&e.api_hash),
                "source": e.source,
                "enabled": e.enabled,
                "added_at": e.added_at,
                "bot_id": e.bot_id,
                "has_bot": !e.bot_id.is_empty(),
            })
        })
        .collect()
}

/* ── per-lane measurement ────────────────────────────────────────────────── */

/// How many part latencies the p50 keeps. 64 parts is ~2 s of a saturating
/// stream, which is long enough to survive one bad window and short enough to
/// follow a lane that recovers.
const P50_WINDOW: usize = 64;

/// Starts at a neutral 8 MB/s so the first assignment is not decided by a lane
/// that has never run. Fast enough not to be capped as congested, slow enough
/// that a measured lane (10-11 MB/s) immediately wins the second part.
const EWMA_START_MBPS: f64 = 8.0;

/// A lane is congested when its EWMA sits below this fraction of the median.
const CONGESTED_RATIO: f64 = 0.6;

/// EWMA weight for the newest sample. 0.2 = ~5 parts to move most of the way,
/// which is one window at the shipped prefetch depth.
const EWMA_ALPHA: f64 = 0.2;

#[derive(Default)]
pub struct LaneStats {
    ewma_mbps: AtomicU64,
    parts_ok: AtomicU64,
    parts_failed: AtomicU64,
    bytes: AtomicU64,
    last_part_us: AtomicU64,
    cooling_until: AtomicI64,
    congested: AtomicBool,
    p50_us: Mutex<Vec<u64>>,
    last_error: Mutex<String>,
}

impl LaneStats {
    pub fn new() -> Self {
        let s = Self::default();
        s.ewma_mbps
            .store((EWMA_START_MBPS * 100.0) as u64, Ordering::Relaxed);
        s
    }

    pub fn ewma_mbps(&self) -> f64 {
        self.ewma_mbps.load(Ordering::Relaxed) as f64 / 100.0
    }

    pub fn parts_ok(&self) -> u64 {
        self.parts_ok.load(Ordering::Relaxed)
    }

    pub fn parts_failed(&self) -> u64 {
        self.parts_failed.load(Ordering::Relaxed)
    }

    pub fn bytes(&self) -> u64 {
        self.bytes.load(Ordering::Relaxed)
    }

    pub fn record_ok(&self, bytes: u64, micros: u64) {
        self.parts_ok.fetch_add(1, Ordering::Relaxed);
        self.bytes.fetch_add(bytes, Ordering::Relaxed);
        self.last_part_us.store(micros, Ordering::Relaxed);
        if let Ok(mut ring) = self.p50_us.lock() {
            ring.push(micros);
            if ring.len() > P50_WINDOW {
                ring.remove(0);
            }
        }
        if micros == 0 {
            return;
        }
        let mbps = (bytes as f64) / (micros as f64 / 1_000_000.0) / 1_048_576.0;
        let prev = self.ewma_mbps();
        let next = prev * (1.0 - EWMA_ALPHA) + mbps * EWMA_ALPHA;
        // A single absurd sample (a 4 KB part answered from cache) must not
        // decide the lane's weight, so only a bandwidth-shaped one moves it.
        if mbps.is_finite() && mbps > 0.0 && mbps < 500.0 {
            self.ewma_mbps
                .store((next * 100.0).round() as u64, Ordering::Relaxed);
        }
    }

    pub fn record_fail(&self, message: &str) {
        self.parts_failed.fetch_add(1, Ordering::Relaxed);
        if let Ok(mut slot) = self.last_error.lock() {
            *slot = message.chars().take(200).collect();
        }
    }

    pub fn cooling_remaining(&self) -> i64 {
        (self.cooling_until.load(Ordering::Relaxed) - now()).max(0)
    }

    pub fn cool_for(&self, secs: i64, message: &str) {
        let until = now() + secs.max(0);
        self.cooling_until.store(until, Ordering::Relaxed);
        if let Ok(mut slot) = self.last_error.lock() {
            *slot = message.chars().take(200).collect();
        }
        log(
            "lane cooling",
            &serde_json::json!({"seconds": secs, "error": message}),
        );
    }

    pub fn congested(&self) -> bool {
        self.congested.load(Ordering::Relaxed)
    }

    pub fn set_congested(&self, value: bool) {
        self.congested.store(value, Ordering::Relaxed);
    }

    /// Median of the last [`P50_WINDOW`] part latencies, in milliseconds.
    pub fn p50_part_ms(&self) -> u64 {
        let Ok(ring) = self.p50_us.lock() else {
            return 0;
        };
        if ring.is_empty() {
            return 0;
        }
        let mut v = ring.clone();
        v.sort_unstable();
        v[v.len() / 2] / 1000
    }

    /// The last part's latency in milliseconds (`0` before the first part).
    pub fn last_part_ms(&self) -> u64 {
        self.last_part_us.load(Ordering::Relaxed) / 1000
    }

    pub fn last_error(&self) -> String {
        self.last_error
            .lock()
            .ok()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub fn to_json(&self, api_id: i32, state: &str) -> Value {
        serde_json::json!({
            "api_id": api_id,
            "state": state,
            "ewma_mbps": (self.ewma_mbps() * 100.0).round() / 100.0,
            "p50_part_ms": self.p50_part_ms(),
            "parts_ok": self.parts_ok(),
            "parts_failed": self.parts_failed(),
            "bytes": self.bytes(),
            "cooling_until": self.cooling_until.load(Ordering::Relaxed),
            "cooling_s": self.cooling_remaining(),
            "last_error": self.last_error(),
            "congested": self.congested(),
        })
    }
}

/// A lane slower than [`CONGESTED_RATIO`] of the fleet median may run ONE part
/// at a time until it recovers, so a bad api_id cannot drag a stream to its own
/// rate. Pure, so the rule is testable without any lanes.
pub fn is_congested(ewma: f64, median: f64) -> bool {
    median > 0.0 && ewma > 0.0 && ewma < median * CONGESTED_RATIO
}

pub fn median(values: &[f64]) -> f64 {
    let mut v: Vec<f64> = values.iter().copied().filter(|x| x.is_finite()).collect();
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if v.len() % 2 == 1 {
        v[v.len() / 2]
    } else {
        (v[v.len() / 2 - 1] + v[v.len() / 2]) / 2.0
    }
}

/// Cost of handing one more part to a lane: expected seconds, so the smallest
/// wins. `in_flight + 1` is deliberate — a lane with 8 parts already queued is
/// worse than an idle one even when it is twice as fast.
pub fn lane_cost(in_flight: u32, ewma_mbps: f64) -> f64 {
    let speed = if ewma_mbps.is_finite() && ewma_mbps > 0.0 {
        ewma_mbps
    } else {
        EWMA_START_MBPS
    };
    (in_flight as f64 + 1.0) / speed
}

/// Pick the cheapest lane, skipping any with no candidate (`None`).
pub fn choose_lane(costs: &[Option<f64>]) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (i, c) in costs.iter().enumerate() {
        let Some(c) = c else { continue };
        if !c.is_finite() || *c < 0.0 {
            continue;
        }
        match best {
            Some((_, b)) if *c >= b => {}
            _ => best = Some((i, *c)),
        }
    }
    best.map(|(i, _)| i)
}

/* ── lanes and the registry ──────────────────────────────────────────────── */

pub struct Lane {
    pub api_id: i32,
    pub pool: Arc<TelegramPool>,
    pub stats: Arc<LaneStats>,
    bot_id: Mutex<String>,
    in_flight: AtomicU32,
    healthy_session: AtomicBool,
}

impl Lane {
    pub fn new(api_id: i32, pool: Arc<TelegramPool>, bot_id: &str) -> Arc<Self> {
        Arc::new(Self {
            api_id,
            pool,
            stats: Arc::new(LaneStats::new()),
            bot_id: Mutex::new(bot_id.trim().to_string()),
            in_flight: AtomicU32::new(0),
            healthy_session: AtomicBool::new(!bot_id.trim().is_empty()),
        })
    }

    pub fn bot_id(&self) -> String {
        self.bot_id.lock().map(|s| s.clone()).unwrap_or_default()
    }

    pub fn set_bot_id(&self, bot_id: &str) {
        let id = bot_id.trim().to_string();
        if let Ok(mut slot) = self.bot_id.lock() {
            *slot = id.clone();
        }
        self.healthy_session.store(!id.is_empty(), Ordering::Relaxed);
    }

    pub fn has_bot(&self) -> bool {
        self.healthy_session.load(Ordering::Relaxed) && !self.bot_id().is_empty()
    }

    pub fn in_flight(&self) -> u32 {
        self.in_flight.load(Ordering::Relaxed)
    }

    /// Ready to receive a part right now.
    pub fn usable(&self) -> bool {
        self.has_bot() && self.stats.cooling_remaining() == 0
    }

    pub fn begin_part(self: &Arc<Self>) -> LanePartGuard {
        self.in_flight.fetch_add(1, Ordering::Relaxed);
        LanePartGuard(self.clone())
    }

    pub fn state(&self) -> &'static str {
        if !self.has_bot() {
            "no_bot"
        } else if self.stats.cooling_remaining() > 0 {
            "cooling"
        } else if self.stats.congested() {
            "congested"
        } else {
            "ready"
        }
    }

    pub fn to_json(&self) -> Value {
        let mut v = self.stats.to_json(self.api_id, self.state());
        if let Some(o) = v.as_object_mut() {
            o.insert("bot_id".into(), serde_json::json!(self.bot_id()));
            o.insert("in_flight".into(), serde_json::json!(self.in_flight()));
            o.insert(
                "workers".into(),
                serde_json::json!(self.pool.workers()),
            );
        }
        v
    }
}

pub struct LanePartGuard(Arc<Lane>);

impl Drop for LanePartGuard {
    fn drop(&mut self) {
        self.0.in_flight.fetch_sub(1, Ordering::Relaxed);
    }
}

pub struct LaneRegistry {
    lanes: Vec<Arc<Lane>>,
    default: Arc<Lane>,
    by_bot: Mutex<HashMap<String, Arc<Lane>>>,
}

impl LaneRegistry {
    pub fn new(default: Arc<Lane>, extra: Vec<Arc<Lane>>) -> Arc<Self> {
        let mut lanes = vec![default.clone()];
        for lane in extra {
            // One lane per api_id: a second lane on the same application would
            // divide its ceiling instead of adding to it.
            if lanes.iter().any(|l| l.api_id == lane.api_id) {
                continue;
            }
            lanes.push(lane);
        }
        let mut by_bot = HashMap::new();
        for lane in &lanes {
            let id = lane.bot_id();
            if !id.is_empty() {
                by_bot.insert(id, lane.clone());
            }
        }
        Arc::new(Self {
            lanes,
            default,
            by_bot: Mutex::new(by_bot),
        })
    }

    pub fn default_lane(&self) -> Arc<Lane> {
        self.default.clone()
    }

    pub fn lanes(&self) -> Vec<Arc<Lane>> {
        self.lanes.clone()
    }

    pub fn lane_for_api_id(&self, api_id: i32) -> Option<Arc<Lane>> {
        self.lanes.iter().find(|l| l.api_id == api_id).cloned()
    }

    /// Which lane's connections a bot belongs to. `mint.rs` needs this: it is
    /// reached with only a bot id, and booting that bot on the DEFAULT api_id
    /// after it signed in on another one is a second login for the same session
    /// (and the wrong app).
    pub fn pool_for_bot(&self, bot_id: &str) -> Option<Arc<TelegramPool>> {
        let id = bot_id.trim();
        if id.is_empty() {
            return None;
        }
        if let Some(lane) = self.by_bot.lock().ok().and_then(|m| m.get(id).cloned()) {
            return Some(lane.pool.clone());
        }
        // The lane may know the bot by its real identity while the caller holds
        // the token-derived id (or the reverse): match on either.
        for lane in &self.lanes {
            let known = lane.bot_id();
            if known == id {
                return Some(lane.pool.clone());
            }
        }
        None
    }

    pub fn register_bot(&self, bot_id: &str, lane: &Arc<Lane>) {
        let id = bot_id.trim();
        if id.is_empty() {
            return;
        }
        if let Ok(mut map) = self.by_bot.lock() {
            map.insert(id.to_string(), lane.clone());
        }
    }

    /// Lanes that can take a part now, with their costs, in pool order.
    pub fn candidates(&self) -> Vec<(Arc<Lane>, f64)> {
        let usable: Vec<Arc<Lane>> = self.lanes.iter().filter(|l| l.usable()).cloned().collect();
        let ewm = usable.iter().map(|l| l.stats.ewma_mbps()).collect::<Vec<_>>();
        let med = median(&ewm);
        usable
            .into_iter()
            .map(|l| {
                let congested = is_congested(l.stats.ewma_mbps(), med);
                l.stats.set_congested(congested);
                // A congested lane may run one part at a time: report it as if one
                // part were always outstanding, so it is only chosen when every
                // healthy lane is busier.
                let in_flight = if congested {
                    l.in_flight().max(1)
                } else {
                    l.in_flight()
                };
                let cost = lane_cost(in_flight, l.stats.ewma_mbps());
                (l, cost)
            })
            .collect()
    }

    pub fn total_mbps(&self) -> f64 {
        self.lanes
            .iter()
            .filter(|l| l.usable())
            .map(|l| l.stats.ewma_mbps())
            .sum()
    }
}

static REGISTRY: OnceLock<Arc<LaneRegistry>> = OnceLock::new();

pub fn publish(registry: Arc<LaneRegistry>) {
    let _ = REGISTRY.set(registry);
}

pub fn registry() -> Option<Arc<LaneRegistry>> {
    REGISTRY.get().cloned()
}

/// The pool that owns `bot_id`, falling back to the process pool.
///
/// This replaces a bare `telegram::shared_pool()` in `mint.rs`: with lanes, the
/// bot may belong to a lane whose api_id is not the default one, and the mint's
/// login must use that lane's credentials.
pub fn pool_for_bot(bot_id: &str) -> Option<Arc<TelegramPool>> {
    if let Some(pool) = registry().and_then(|r| r.pool_for_bot(bot_id)) {
        return Some(pool);
    }
    crate::telegram::shared_pool()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hash_is_never_returned_in_full() {
        let h = "0123456789abcdef0123456789abcdef";
        let m = masked_hash(h);
        assert_eq!(m, "0123…cdef");
        assert!(!m.contains(h));
        assert_eq!(masked_hash(""), "");
        assert_eq!(masked_hash("short"), "…");
    }

    #[test]
    fn the_pool_file_accepts_an_object_and_a_bare_array() {
        let body = r#"{"api_ids":[{"api_id":11,"api_hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","enabled":true,"bot_id":"42"}]}"#;
        let rows = parse_entries(body);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].api_id, 11);
        assert_eq!(rows[0].bot_id, "42");
        assert!(rows[0].enabled);

        let bare = r#"[{"api_id":12,"api_hash":"bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"}]"#;
        assert_eq!(parse_entries(bare).len(), 1);
    }

    #[test]
    fn invalid_and_duplicate_rows_are_dropped_not_fatal() {
        let body = r#"{"api_ids":[
            {"api_id":0,"api_hash":"x"},
            {"api_id":13,"api_hash":""},
            {"api_id":14,"api_hash":"cccccccccccccccccccccccccccccccc"},
            {"api_id":14,"api_hash":"dddddddddddddddddddddddddddddddd"},
            {"api_id":"nope","api_hash":"eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"}
        ]}"#;
        let rows = parse_entries(body);
        assert_eq!(rows.len(), 1, "one valid api_id, first occurrence wins");
        assert_eq!(rows[0].api_id, 14);
        assert_eq!(rows[0].api_hash, "cccccccccccccccccccccccccccccccc");
    }

    #[test]
    fn add_rejects_a_bad_hash_and_a_duplicate() {
        // No storage is written on the error paths, so this needs no fixture.
        assert!(add_entry(0, "ffffffffffffffffffffffffffffffff").is_err());
        assert!(add_entry(15, "nothex").is_err());
        assert!(add_entry(15, "short").is_err());
    }

    #[test]
    fn a_lane_that_never_ran_starts_at_the_neutral_speed() {
        let s = LaneStats::new();
        assert!((s.ewma_mbps() - EWMA_START_MBPS).abs() < 0.01);
        assert_eq!(s.p50_part_ms(), 0);
        assert!(!s.congested());
    }

    #[test]
    fn a_fast_part_lifts_the_ewma_and_a_slow_one_lowers_it() {
        let s = LaneStats::new();
        // 1 MB in 100 ms = 10 MB/s.
        s.record_ok(1_048_576, 100_000);
        assert!(s.ewma_mbps() > 8.0 && s.ewma_mbps() < 11.0, "{}", s.ewma_mbps());
        // 1 MB in 5 s = 0.2 MB/s, repeated until the EWMA follows.
        for _ in 0..12 {
            s.record_ok(1_048_576, 5_000_000);
        }
        assert!(s.ewma_mbps() < 1.0, "{}", s.ewma_mbps());
        assert_eq!(s.parts_ok(), 13);
        assert_eq!(s.bytes(), 13 * 1_048_576);
    }

    #[test]
    fn a_stalled_part_does_not_look_like_bandwidth() {
        let s = LaneStats::new();
        s.record_ok(4096, 2);
        // Absurd MB/s is ignored rather than pinned as the best lane.
        assert!(s.ewma_mbps() < 20.0, "{}", s.ewma_mbps());
    }

    #[test]
    fn cooling_counts_down_and_makes_a_lane_unusable() {
        let s = LaneStats::new();
        s.cool_for(30, "FLOOD_WAIT_30");
        assert!(s.cooling_remaining() > 0);
        assert_eq!(s.last_error(), "FLOOD_WAIT_30");
        assert_eq!(s.cooling_remaining(), 30);
        s.cool_for(0, "cleared");
        assert_eq!(s.cooling_remaining(), 0);
    }

    #[test]
    fn congestion_is_relative_to_the_fleet_median() {
        assert!(is_congested(5.0, 10.0));
        assert!(is_congested(5.9, 10.0));
        assert!(!is_congested(6.0, 10.0), "60% is the boundary, not below it");
        assert!(!is_congested(10.0, 10.0));
        assert!(!is_congested(0.0, 10.0), "zero is 'unmeasured', not 'congested'");
        assert!(!is_congested(5.0, 0.0));
    }

    #[test]
    fn median_handles_even_and_odd_counts() {
        assert_eq!(median(&[]), 0.0);
        assert_eq!(median(&[3.0]), 3.0);
        assert_eq!(median(&[5.0, 1.0, 3.0]), 3.0);
        assert_eq!(median(&[1.0, 2.0, 3.0, 4.0]), 2.5);
    }

    #[test]
    fn the_dispatcher_prefers_the_lane_with_the_lowest_expected_wait() {
        // Two idle lanes: the faster one wins.
        assert_eq!(choose_lane(&[Some(lane_cost(0, 6.0)), Some(lane_cost(0, 11.0))]), Some(1));
        // The slow lane is free, the fast one is loaded: cost, not raw speed.
        let slow = lane_cost(0, 6.0); // 1/6
        let busy = lane_cost(2, 11.0); // 3/11
        assert!(slow < busy, "{slow} < {busy}");
        assert_eq!(choose_lane(&[Some(slow), Some(busy)]), Some(0));
        // A cooling lane is simply not offered.
        assert_eq!(choose_lane(&[None, Some(lane_cost(0, 5.0))]), Some(1));
        assert_eq!(choose_lane(&[None, None]), None);
    }

    #[test]
    fn a_congested_lane_is_offered_as_if_one_part_were_always_in_flight() {
        // 3 MB/s against an 11 MB/s median is congested, and even idle its cost
        // must lose to a loaded-but-healthy lane: that is what stops one bad
        // api_id from setting the rate of the whole response.
        let congested = lane_cost(1, 3.0);
        let healthy = lane_cost(2, 11.0);
        assert!(congested > healthy, "{congested} > {healthy}");
    }

    #[test]
    fn a_registry_keeps_one_lane_per_api_id_and_maps_bots_to_their_lane() {
        let d = Lane::new(1, Arc::new(TelegramPool::new(1, 1)), "100");
        let extra = Lane::new(2, Arc::new(TelegramPool::new(1, 1)), "200");
        let dup = Lane::new(2, Arc::new(TelegramPool::new(1, 1)), "201");
        let reg = LaneRegistry::new(d, vec![extra, dup]);
        assert_eq!(reg.lanes().len(), 2, "a second lane for api_id 2 is ignored");
        assert!(reg.lane_for_api_id(2).is_some());
        assert_eq!(reg.pool_for_bot("200").map(|p| p.workers()), Some(1));
        assert!(reg.pool_for_bot("").is_none());
        assert!(reg.pool_for_bot("999").is_none());
        assert_eq!(reg.default_lane().api_id, 1);
    }

    #[test]
    fn a_lane_without_a_bot_is_not_a_candidate() {
        let d = Lane::new(1, Arc::new(TelegramPool::new(1, 1)), "");
        let reg = LaneRegistry::new(d, vec![]);
        assert!(reg.candidates().is_empty());
        assert_eq!(reg.lanes()[0].state(), "no_bot");
        reg.lanes()[0].set_bot_id("77");
        assert_eq!(reg.candidates().len(), 1);
        assert_eq!(reg.lanes()[0].state(), "ready");
    }

    #[test]
    fn a_part_guard_tracks_in_flight_and_releases_on_drop() {
        let lane = Lane::new(1, Arc::new(TelegramPool::new(1, 1)), "1");
        assert_eq!(lane.in_flight(), 0);
        {
            let _a = lane.begin_part();
            let _b = lane.begin_part();
            assert_eq!(lane.in_flight(), 2);
        }
        assert_eq!(lane.in_flight(), 0);
    }
}
