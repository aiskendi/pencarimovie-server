//! PencariMusic addon builders: descriptor, Eclipse catalogs and detail pages,
//! Stremio metas and streams, and the sponsored row.
//!
//! WHY ONE MODULE. The same catalogue is read by three clients:
//!
//! * Eclipse (and the BitChord fork) speak the music addon: `catalogs` of type
//!   `track`/`album`/`artist`/`playlist`, `/catalog/{id}` answering `items`,
//!   plus `/artist/{id}` and `/album/{id}` pages;
//! * Stremio and Nuvio have no music content type at all. A working music
//!   addon there maps a track onto a `movie` item with a `/stream` answer, which
//!   is what Flix-Music and Stremio Open Music do;
//! * the manifest is one document for all of them, so the shapes have to be
//!   built side by side rather than in two addons.
//!
//! Everything here is pure: it takes formatted track rows (the shape
//! [`crate::music::format_track`] emits) and returns JSON. `routes.rs` owns the
//! origin calls, the cache and the HTTP envelope.
//!
//! HOUSE RULES THIS MODULE OBEYS:
//!
//! * a Stremio stream row NEVER carries `title`; stremio-core deserializes a
//!   stream into `{name, description, url|externalUrl|infoHash, behaviorHints}`
//!   and drops any row with an unknown key (the "only the sponsored item
//!   shows" bug). The readable text lives on `name` + `description`;
//! * no field is fabricated. `isrc`/`album`/`artworkURL` are emitted only when
//!   the origin sent them, and an unknown album or artist group is left out
//!   rather than guessed from a file name.

use base64::Engine as _;
use serde_json::{json, Value};
use std::collections::HashSet;

/// The description in the addon list. One constant, so the wording never
/// drifts between the manifest and the sponsor fallback.
pub const DESCRIPTION: &str =
    "Search and stream music and audio files from the PencariMovie Telegram index.";

/// What the sponsor card says when WordPress has no sponsor configured.
pub const SPONSOR_FALLBACK_DESCRIPTION: &str = DESCRIPTION;

/// Id of the sponsored Eclipse playlist row.
pub const SPONSOR_ID: &str = "sponsor";

/// Prefix every Stremio item id carries. The manifest declares it as
/// `idPrefixes`, so Stremio asks this addon only about its own items.
pub const STREMIO_PREFIX: &str = "pm_";

/// The sponsored item's Stremio id.
pub const STREMIO_SPONSOR_ID: &str = "pm_sponsor";

/// Where the catalogue rows live. `latest` is the empty-query browse, the
/// others are filters over the same sample plus the artist/album groupings.
pub fn canonical_row(id: &str) -> Option<&'static str> {
    match id {
        "search" | "pm_music_search" => Some("search"),
        "latest" | "pm_music_latest" => Some("latest"),
        "lossless" | "pm_music_lossless" => Some("lossless"),
        "hi_res" | "pm_music_hi_res" => Some("hi_res"),
        "artists" | "pm_music_artists" => Some("artists"),
        "albums" | "pm_music_albums" => Some("albums"),
        "sponsor" | "pm_music_sponsor" => Some("sponsor"),
        _ => None,
    }
}

/// The addon descriptor both protocols read.
///
/// `version` is the caller's `config::descriptor_version()`, not a literal: the
/// old `1.1.0` made every build advertise the same number forever.
///
/// `albums` is deliberately NOT advertised yet. MEASURED 2026-11-15: every
/// `/search-music` row the origin returned had no `album` tag, so the row was
/// empty on every probe. The builder is wired (`canonical_row`, `album_items`,
/// `album_detail`) and belongs in this list the day the origin sends album
/// metadata; until then an empty shelf is worse than no shelf.
///
/// `latest`, `lossless` and `hi_res` are also NOT advertised any more. The
/// origin's newest rows are mostly long-form speech (podcasts, radio, readings),
/// so those shelves were "audio", not music, and the row names promised a
/// selection the catalogue does not have. The handlers stay (`canonical_row`
/// still maps them) so an already-installed row keeps answering; search, the
/// artists row and the sponsor row are the browse surface now.
pub fn manifest(base: &str, version: &str) -> Value {
    let logo = format!("{}/logo.png", base.trim_end_matches('/'));
    json!({
        "id": "pencarimovie.com",
        "name": "PencariMusic (PencariMovie.Com)",
        "version": version,
        "description": DESCRIPTION,
        "icon": logo.clone(),
        "logo": logo.clone(),
        "background": logo,
        // Strings, not objects: BitChord parses `resources` as a list of
        // strings (`AddonModels.kt`), and Stremio accepts the legacy form. A
        // resource object here would make the addon uninstallable in BitChord.
        "resources": ["search", "stream", "catalog", "meta"],
        // `track`/`album`/`artist`/`playlist` are the Eclipse rows; `movie` is
        // how Stremio/Nuvio can see the same catalogue. Each client ignores the
        // families it does not render, and BitChord ignores `catalogs` entirely.
        "types": ["track", "album", "artist", "playlist", "movie", "series"],
        "idPrefixes": [STREMIO_PREFIX],
        "contentType": "music",
        "catalogs": [
            {"id": "artists", "type": "artist",   "name": "Artists"},
            {"id": "sponsor", "type": "playlist", "name": "PencariMusic"},
            // Stremio/Nuvio Discover and Search: the search row is the one a
            // query is routed through (`extra.search`), the artists row is the
            // browse row, and the sponsor row is the brand tile.
            {"id": "pm_music_search",  "type": "movie", "name": "PencariMusic · Search",
             "extra": [{"name": "search", "isRequired": true}]},
            {"id": "pm_music_artists", "type": "series", "name": "PencariMusic · Artists"},
            {"id": "pm_music_sponsor", "type": "movie", "name": "PencariMusic"}
        ]
    })
}

/* ── id codecs ───────────────────────────────────────────────────────────── */

fn b64(raw: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

fn unb64(raw: &str) -> Option<String> {
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(raw.trim())
        .ok()?;
    String::from_utf8(bytes).ok()
}

pub fn encode_artist(name: &str) -> String {
    b64(&format!("a:{name}"))
}

pub fn decode_artist(id: &str) -> Option<String> {
    let raw = unb64(id)?;
    raw.strip_prefix("a:")
        .map(str::to_string)
        .filter(|s| !s.trim().is_empty())
}

/// An album is identified by its artist AND its title: two bands can both have
/// a "Greatest Hits", and a compilation lists many artists under one album.
pub fn encode_album(artist: &str, album: &str) -> String {
    b64(&format!("al:{artist}\u{1f}{album}"))
}

pub fn decode_album(id: &str) -> Option<(String, String)> {
    let raw = unb64(id)?;
    let rest = raw.strip_prefix("al:")?;
    let (artist, album) = rest.split_once('\u{1f}')?;
    if album.trim().is_empty() {
        return None;
    }
    Some((artist.to_string(), album.to_string()))
}

/// `pm_<short_code>` for a track.
pub fn stremio_id(short_code: &str) -> String {
    format!("{STREMIO_PREFIX}{short_code}")
}

/// The short code back out of a Stremio track id.
pub fn stremio_code(id: &str) -> Option<&str> {
    id.strip_prefix(STREMIO_PREFIX)
        .filter(|c| !c.is_empty() && !c.starts_with("a_"))
}

/// `pm_a_<base64 name>` for an artist tile.
pub fn stremio_artist_id(name: &str) -> String {
    format!("{STREMIO_PREFIX}a_{}", b64(name))
}

pub fn stremio_artist_name(id: &str) -> Option<String> {
    let raw = id.strip_prefix(STREMIO_PREFIX)?.strip_prefix("a_")?;
    unb64(raw).filter(|s| !s.trim().is_empty())
}

/* ── small helpers ───────────────────────────────────────────────────────── */

fn s(v: &Value, key: &str) -> String {
    v.get(key)
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn u(v: &Value, key: &str) -> u64 {
    v.get(key).and_then(|x| x.as_u64()).unwrap_or(0)
}

fn i(v: &Value, key: &str) -> i64 {
    match v.get(key) {
        Some(Value::Number(n)) => n.as_i64().unwrap_or(0),
        Some(Value::String(raw)) => raw.trim().parse().unwrap_or(0),
        _ => 0,
    }
}

fn flagged(v: &Value, key: &str) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(false)
}

/// Case- and accent-insensitive key for grouping and matching.
fn fold(raw: &str) -> String {
    crate::matcher::fold_latin_accents(&raw.to_lowercase())
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Artwork a music tile may show. The origin's shared placeholder is a blank
/// document icon, not a cover: an artist or album tile is better off with no
/// poster than with a fake one.
fn real_art(t: &Value) -> Option<String> {
    let art = s(t, "artworkURL");
    if art.is_empty() || art.contains("tg-placeholder") {
        None
    } else {
        Some(art)
    }
}

fn human_size(bytes: u64) -> String {
    if bytes == 0 {
        return String::new();
    }
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    if gb >= 1.0 {
        return format!("{gb:.1} GB");
    }
    let mb = bytes as f64 / (1024.0 * 1024.0);
    if mb >= 1.0 {
        return format!("{mb:.0} MB");
    }
    format!("{:.0} KB", bytes as f64 / 1024.0)
}

fn clock(secs: i64) -> String {
    if secs <= 0 {
        return String::new();
    }
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// One line of facts for a tile, in the same order on every surface.
pub fn track_line(t: &Value) -> String {
    let format = s(t, "format").to_uppercase();
    let tier = crate::music::tier(&s(t, "format"), u(t, "size"), i(t, "duration")).to_string();
    let parts = [format, tier, human_size(u(t, "size")), clock(i(t, "duration"))];
    parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" · ")
}

fn lossless(t: &Value) -> bool {
    crate::music::is_lossless(&s(t, "format"), u(t, "size"), i(t, "duration"))
}

fn hi_res(t: &Value) -> bool {
    crate::music::is_hi_res(&s(t, "format"), u(t, "size"), i(t, "duration"))
}

/* ── catalogue rows ──────────────────────────────────────────────────────── */

/// Which of the sampled tracks belong to a row. `artists`/`albums` read the
/// sample too, but are grouped by their own builders below.
pub fn row_tracks(row: &str, tracks: &[Value], limit: usize) -> Vec<Value> {
    let filtered: Vec<Value> = match row {
        "lossless" => tracks.iter().filter(|t| lossless(t)).cloned().collect(),
        "hi_res" => tracks.iter().filter(|t| hi_res(t)).cloned().collect(),
        _ => tracks.to_vec(),
    };
    filtered.into_iter().take(limit).collect()
}

/// One Eclipse catalogue item for a track.
pub fn track_item(t: &Value) -> Option<Value> {
    let id = s(t, "id");
    if id.is_empty() {
        return None;
    }
    let duration = i(t, "duration");
    let mut item = json!({
        "id": id,
        "type": "track",
        "title": s(t, "title"),
        "artist": s(t, "artist"),
        "hiRes": crate::music::is_hi_res(&s(t, "format"), u(t, "size"), duration),
    });
    // Milliseconds, as the catalog contract requires; omitted when the length
    // is unknown rather than filled with a guess.
    let ms = if duration > 0 { duration * 1000 } else { i(t, "durationMs") };
    if ms > 0 {
        item["durationMs"] = json!(ms);
    }
    for key in ["album", "isrc", "artworkURL"] {
        let value = s(t, key);
        if !value.is_empty() {
            item[key] = json!(value);
        }
    }
    Some(item)
}

/// Distinct artists, in the order the catalogue returns them.
fn artist_groups(tracks: &[Value], limit: usize) -> Vec<(String, Option<String>)> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut out: Vec<(String, Option<String>)> = Vec::new();
    for t in tracks {
        if !flagged(t, "artistTagged") {
            continue;
        }
        let name = s(t, "artist");
        if name.is_empty() {
            continue;
        }
        if !seen.insert(fold(&name)) {
            continue;
        }
        out.push((name, real_art(t)));
        if out.len() >= limit {
            break;
        }
    }
    out
}

/// Campaign tiles for the Eclipse `artists` row.
pub fn artist_items(tracks: &[Value], limit: usize) -> Vec<Value> {
    artist_groups(tracks, limit)
        .into_iter()
        .map(|(name, art)| {
            let mut item = json!({
                "id": encode_artist(&name),
                "type": "artist",
                "title": name,
                "artist": name,
            });
            if let Some(art) = art {
                item["artworkURL"] = json!(art);
            }
            item
        })
        .collect()
}

/// The `artists` array of an Eclipse search answer.
pub fn search_artists(tracks: &[Value], limit: usize) -> Vec<Value> {
    artist_groups(tracks, limit)
        .into_iter()
        .map(|(name, art)| {
            let mut item = json!({"id": encode_artist(&name), "name": name});
            if let Some(art) = art {
                item["artworkURL"] = json!(art);
            }
            item
        })
        .collect()
}

struct AlbumGroup {
    key: (String, String),
    title: String,
    artist: String,
    art: Option<String>,
    count: usize,
}

fn album_groups(tracks: &[Value], limit: usize) -> Vec<AlbumGroup> {
    let mut out: Vec<AlbumGroup> = Vec::new();
    for t in tracks {
        let album = s(t, "album");
        if album.is_empty() {
            continue;
        }
        let artist = if flagged(t, "artistTagged") {
            s(t, "artist")
        } else {
            String::new()
        };
        let key = (fold(&album), fold(&artist));
        if let Some(group) = out.iter_mut().find(|g| g.key == key) {
            group.count += 1;
            if group.art.is_none() {
                group.art = real_art(t);
            }
            continue;
        }
        if out.len() >= limit {
            continue;
        }
        out.push(AlbumGroup {
            key,
            title: album,
            artist,
            art: real_art(t),
            count: 1,
        });
    }
    out
}

/// Campaign tiles for the Eclipse `albums` row.
pub fn album_items(tracks: &[Value], limit: usize) -> Vec<Value> {
    album_groups(tracks, limit)
        .into_iter()
        .map(|g| {
            let mut item = json!({
                "id": encode_album(&g.artist, &g.title),
                "type": "album",
                "title": g.title,
                "artist": g.artist,
            });
            if let Some(art) = g.art {
                item["artworkURL"] = json!(art);
            }
            item
        })
        .collect()
}

/// The `albums` array of an Eclipse search answer.
pub fn search_albums(tracks: &[Value], limit: usize) -> Vec<Value> {
    album_groups(tracks, limit)
        .into_iter()
        .map(|g| {
            let mut item = json!({
                "id": encode_album(&g.artist, &g.title),
                "title": g.title,
                "artist": g.artist,
                "trackCount": g.count,
            });
            if let Some(art) = g.art {
                item["artworkURL"] = json!(art);
            }
            item
        })
        .collect()
}

/* ── detail pages ────────────────────────────────────────────────────────── */

/// `/artist/{id}`: top tracks and albums, matched on the folded artist name.
pub fn artist_detail(id: &str, tracks: &[Value]) -> Option<Value> {
    let name = decode_artist(id)?;
    let want = fold(&name);
    let matched: Vec<&Value> = tracks
        .iter()
        .filter(|t| fold(&s(t, "artist")) == want)
        .collect();
    if matched.is_empty() {
        return None;
    }
    let owned: Vec<Value> = matched.iter().map(|t| (*t).clone()).collect();
    let top: Vec<Value> = owned.iter().take(50).cloned().collect();
    let albums = search_albums(&owned, 20);
    let mut detail = json!({"id": id, "name": name, "topTracks": top, "albums": albums});
    if let Some(art) = matched.iter().find_map(|t| real_art(t)) {
        detail["artworkURL"] = json!(art);
    }
    Some(detail)
}

/// `/album/{id}`: the tracks that carry that album tag.
pub fn album_detail(id: &str, tracks: &[Value]) -> Option<Value> {
    let (artist, album) = decode_album(id)?;
    let want = fold(&album);
    let want_artist = fold(&artist);
    let matched: Vec<&Value> = tracks
        .iter()
        .filter(|t| {
            if fold(&s(t, "album")) != want {
                return false;
            }
            // A tagged row must be by this artist; an untagged one has no
            // artist to compare, so the album match is all it can offer.
            want_artist.is_empty()
                || !flagged(t, "artistTagged")
                || fold(&s(t, "artist")) == want_artist
        })
        .collect();
    if matched.is_empty() {
        return None;
    }
    let rows: Vec<Value> = matched.iter().map(|t| (*t).clone()).collect();
    let mut detail = json!({
        "id": id,
        "title": album,
        "artist": artist,
        "trackCount": rows.len(),
        "tracks": rows,
    });
    if let Some(art) = matched.iter().find_map(|t| real_art(t)) {
        detail["artworkURL"] = json!(art);
    }
    Some(detail)
}

/* ── Stremio / Nuvio ─────────────────────────────────────────────────────── */

/// One `metas` entry (a tile) for a track.
pub fn stremio_preview(t: &Value) -> Option<Value> {
    let id = s(t, "id");
    if id.is_empty() {
        return None;
    }
    let title = s(t, "title");
    let artist = s(t, "artist");
    let name = if !artist.is_empty() && artist != title {
        format!("{artist} - {title}")
    } else {
        title
    };
    let mut item = json!({
        "id": stremio_id(&id),
        "type": "movie",
        "name": name,
        "posterShape": "square",
        "description": track_line(t),
    });
    if let Some(art) = real_art(t) {
        item["poster"] = json!(art);
    }
    Some(item)
}

/// One `meta` page for a track: the preview plus a background.
pub fn stremio_meta(t: &Value) -> Option<Value> {
    let mut meta = stremio_preview(t)?;
    if let Some(art) = real_art(t) {
        meta["background"] = json!(art);
    }
    Some(meta)
}

/// The stream row, built from the resolved Eclipse payload.
///
/// No `title` key, ever: stremio-core drops a row that carries one, which is
/// how a full list once collapsed to the single sponsored item.
pub fn stremio_stream_row(body: &Value, file_name: &str, size: u64, duration: i64) -> Value {
    let format = body.get("format").and_then(|v| v.as_str()).unwrap_or("");
    let quality = match body.get("quality").and_then(|v| v.as_str()).unwrap_or("") {
        "hi_res_lossless" => "HI-RES LOSSLESS",
        "lossless" => "LOSSLESS",
        _ => "HIGH",
    };
    let parts = [
        format.to_uppercase(),
        quality.to_string(),
        human_size(size),
        clock(duration),
    ];
    let line = parts
        .into_iter()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");
    json!({
        "name": "PencariMusic",
        "description": line,
        "url": body.get("url").cloned().unwrap_or(Value::Null),
        "behaviorHints": {"filename": file_name, "videoSize": size},
    })
}

/// One Stremio tile per distinct artist.
///
/// Artists are `series`, not `movie`: a movie meta has ONE stream list, which
/// made an artist page show a single file. A series meta carries `videos`, and
/// each track is an episode, so tapping an artist lists the whole catalogue and
/// Stremio resolves only the episode that is played.
pub fn stremio_artist_metas(tracks: &[Value], limit: usize) -> Vec<Value> {
    artist_groups(tracks, limit)
        .into_iter()
        .map(|(name, art)| {
            let mut item = json!({
                "id": stremio_artist_id(&name),
                "type": "series",
                "name": name,
                "posterShape": "square",
                "description": "Artist",
            });
            if let Some(art) = art {
                item["poster"] = json!(art);
            }
            item
        })
        .collect()
}

/// The artist meta page: one episode per matching track.
pub fn stremio_artist_meta(name: &str, tracks: &[Value]) -> Value {
    let want = fold(name);
    let matched: Vec<&Value> = tracks
        .iter()
        .filter(|t| fold(&s(t, "artist")) == want)
        .collect();
    let description = if matched.is_empty() {
        "Artist on PencariMusic".to_string()
    } else {
        format!("{} track(s) on PencariMusic", matched.len())
    };
    let videos: Vec<Value> = matched
        .iter()
        .enumerate()
        .filter_map(|(index, t)| {
            let code = s(t, "id");
            if code.is_empty() {
                return None;
            }
            let title = s(t, "title");
            let title = if title.is_empty() { code.clone() } else { title };
            Some(json!({
                "id": stremio_id(&code),
                "title": title,
                "season": 1,
                "episode": index + 1,
            }))
        })
        .collect();
    let mut meta = json!({
        "id": stremio_artist_id(name),
        "type": "series",
        "name": name,
        "posterShape": "square",
        "description": description,
        "videos": videos,
    });
    if let Some(art) = matched.iter().find_map(|t| real_art(t)) {
        meta["poster"] = json!(art);
        meta["background"] = json!(art);
    }
    meta
}

/* ── sponsored row ───────────────────────────────────────────────────────── */

/// The brand card, in both protocols at once.
pub struct SponsorCard {
    pub item: Value,
    pub playlist: Value,
    pub meta: Value,
    pub stream: Option<Value>,
}

/// Build the sponsored row from the WordPress `/version` sponsor.
///
/// The row exists even with no sponsor configured: PencariMusic is the brand
/// this addon belongs to, and a row that disappears when WordPress is down is
/// not a watermark. `url` only decides whether the tile LINKS anywhere; the
/// title, description and logo still render.
pub fn sponsor_card(
    name: Option<&str>,
    description: Option<&str>,
    url: Option<&str>,
    image: Option<&str>,
    base: &str,
) -> SponsorCard {
    let logo = format!("{}/logo.png", base.trim_end_matches('/'));
    let title = name
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("PencariMusic")
        .to_string();
    let text = description
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(SPONSOR_FALLBACK_DESCRIPTION)
        .to_string();
    let link = url.map(str::trim).unwrap_or("").to_string();
    let art = image
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or(logo);
    let body = if link.is_empty() {
        text
    } else {
        format!("{text}\n{link}")
    };

    let item = json!({
        "id": SPONSOR_ID,
        "type": "playlist",
        "title": title,
        "artist": "PencariMusic",
        "artworkURL": art,
    });
    let playlist = json!({
        "id": SPONSOR_ID,
        "title": title,
        "description": body,
        "creator": "PencariMusic",
        "artworkURL": art,
        "trackCount": 0,
        "tracks": [],
    });
    let meta = json!({
        "id": STREMIO_SPONSOR_ID,
        "type": "movie",
        "name": title,
        "poster": art,
        "posterShape": "square",
        "description": body,
    });
    let stream = if link.is_empty() {
        None
    } else {
        Some(json!({
            "name": "PencariMusic",
            "description": title,
            "externalUrl": link,
        }))
    };
    SponsorCard {
        item,
        playlist,
        meta,
        stream,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn track(id: &str, title: &str, artist: &str, tagged: bool) -> Value {
        json!({
            "id": id,
            "title": title,
            "artist": artist,
            "artistTagged": tagged,
            "format": "flac",
            "size": 60_000_000u64,
            "duration": 240,
            "durationMs": 240_000,
            "audioQuality": "LOSSLESS",
        })
    }

    #[test]
    fn the_manifest_names_the_brand_and_both_type_families() {
        let m = manifest("http://127.0.0.1:18099/", "3.1.2");
        assert_eq!(m["name"], "PencariMusic (PencariMovie.Com)");
        assert_eq!(m["version"], "3.1.2");
        assert_eq!(m["id"], "pencarimovie.com");
        assert_eq!(m["description"], DESCRIPTION);
        assert!(m["resources"].as_array().unwrap().iter().any(|r| r == "meta"));
        assert!(m["types"].as_array().unwrap().iter().any(|t| t == "movie"));
        assert!(m["types"].as_array().unwrap().iter().any(|t| t == "series"));
        assert_eq!(m["idPrefixes"][0], STREMIO_PREFIX);
        let artists_row = m["catalogs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == "pm_music_artists")
            .unwrap();
        assert_eq!(artists_row["type"], "series");
        let ids: Vec<&str> = m["catalogs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["id"].as_str().unwrap())
            .collect();
        for id in [
            "artists",
            "sponsor",
            "pm_music_search",
            "pm_music_artists",
            "pm_music_sponsor",
        ] {
            assert!(ids.contains(&id), "catalog {id} is advertised");
        }
        for id in [
            "latest",
            "lossless",
            "hi_res",
            "pm_music_latest",
            "pm_music_lossless",
        ] {
            assert!(!ids.contains(&id), "catalog {id} was removed on request");
        }
        let search = m["catalogs"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["id"] == "pm_music_search")
            .unwrap();
        assert_eq!(search["type"], "movie");
        assert_eq!(search["extra"][0]["name"], "search");
        assert_eq!(search["extra"][0]["isRequired"], true);
        assert_eq!(m["icon"], "http://127.0.0.1:18099/logo.png");
    }

    #[test]
    fn an_eclipse_track_item_carries_the_catalog_fields_and_never_nests_tracks() {
        let item = track_item(&track("abc123", "Be Her", "Ella Langley", true)).unwrap();
        assert_eq!(item["type"], "track");
        assert_eq!(item["id"], "abc123");
        assert_eq!(item["durationMs"], 240_000);
        assert_eq!(item["hiRes"], true, "60 MB / 240 s is above CD bitrate");
        assert!(item.get("tracks").is_none());
        assert!(item.get("album").is_none(), "no album tag, no album field");
        assert!(item.get("isrc").is_none(), "no empty ISRC");
    }

    #[test]
    fn a_stremio_stream_row_has_no_title_field() {
        let body = json!({"url": "http://host/api/download?d=x", "format": "flac", "quality": "lossless"});
        let row = stremio_stream_row(&body, "Be Her.flac", 40_000_000, 240);
        assert_eq!(row["name"], "PencariMusic");
        assert_eq!(row["url"], "http://host/api/download?d=x");
        assert_eq!(row["behaviorHints"]["filename"], "Be Her.flac");
        assert_eq!(row["behaviorHints"]["videoSize"], 40_000_000u64);
        assert!(row.get("title").is_none(), "stremio-core drops rows carrying title");
    }

    #[test]
    fn a_stremio_preview_names_the_artist_and_track() {
        let preview = stremio_preview(&track("abc123", "Be Her", "Ella Langley", true)).unwrap();
        assert_eq!(preview["id"], "pm_abc123");
        assert_eq!(preview["type"], "movie");
        assert_eq!(preview["name"], "Ella Langley - Be Her");
        assert_eq!(preview["posterShape"], "square");
    }

    #[test]
    fn artist_and_album_ids_round_trip_through_url_safe_base64() {
        for name in ["Charles Mingus", "Týr", "AC/DC", "坂本龍一"] {
            let id = encode_artist(name);
            assert!(!id.contains('/'), "{id} is url safe");
            assert_eq!(decode_artist(&id).as_deref(), Some(name));
        }
        let id = encode_album("Ella Langley", "Hungover");
        assert_eq!(decode_album(&id), Some(("Ella Langley".into(), "Hungover".into())));
        assert_eq!(decode_album("not-base64"), None);
        assert_eq!(decode_album(&b64("al:\u{1f}")), None, "no album title");
    }

    #[test]
    fn the_artists_row_uses_tags_never_file_names() {
        let rows = vec![
            track("a1", "Be Her", "Ella Langley", true),
            track("a2", "Another", "Ella Langley", true),
            track("a3", "Some Show", "Reborn Rookie S01E09 1080p", false),
        ];
        let items = artist_items(&rows, 10);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0]["title"], "Ella Langley");
        assert_eq!(items[0]["type"], "artist");
        assert_eq!(
            items[0]["id"].as_str().unwrap(),
            encode_artist("Ella Langley")
        );
    }

    #[test]
    fn the_albums_row_counts_each_album_once() {
        let mut one = track("a1", "One", "Ella Langley", true);
        one["album"] = json!("Hungover");
        let mut two = track("a2", "Two", "Ella Langley", true);
        two["album"] = json!("Hungover");
        let albums = search_albums(&[one, two], 10);
        assert_eq!(albums.len(), 1);
        assert_eq!(albums[0]["trackCount"], 2);
        assert_eq!(albums[0]["title"], "Hungover");
    }

    #[test]
    fn a_lossless_row_drops_the_lossy_track() {
        let mut lossy = track("a1", "One", "X", true);
        lossy["format"] = json!("mp3");
        let rows = row_tracks("lossless", &[lossy, track("a2", "Two", "Y", true)], 10);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["id"], "a2");
    }

    #[test]
    fn the_sponsor_row_uses_wordpress_and_falls_back_to_the_brand() {
        let card = sponsor_card(
            Some("📢 Help us index"),
            Some("Send files to the bot"),
            Some("https://t.me/Y_Flixbot"),
            None,
            "http://host:8088",
        );
        assert_eq!(card.item["title"], "📢 Help us index");
        assert_eq!(card.item["artworkURL"], "http://host:8088/logo.png");
        assert_eq!(card.meta["id"], STREMIO_SPONSOR_ID);
        assert_eq!(card.stream.as_ref().unwrap()["externalUrl"], "https://t.me/Y_Flixbot");
        assert!(card.playlist["description"].as_str().unwrap().contains("t.me/Y_Flixbot"));

        let fallback = sponsor_card(None, None, None, None, "http://host:8088");
        assert_eq!(fallback.item["title"], "PencariMusic");
        assert_eq!(fallback.meta["description"], SPONSOR_FALLBACK_DESCRIPTION);
        assert!(fallback.stream.is_none(), "no link, no external stream");
    }

    #[test]
    fn an_artist_page_lists_every_track_as_an_episode() {
        let rows = vec![
            track("t1", "One", "Ella Langley", true),
            track("t2", "Two", "Ella Langley", true),
            track("t3", "Other", "Someone Else", true),
        ];
        let metas = stremio_artist_metas(&rows, 10);
        assert_eq!(metas.len(), 2, "two distinct artists");
        assert_eq!(metas[0]["type"], "series");

        let meta = stremio_artist_meta("Ella Langley", &rows);
        assert_eq!(meta["type"], "series");
        let videos = meta["videos"].as_array().unwrap();
        assert_eq!(videos.len(), 2, "both tracks are episodes");
        assert_eq!(videos[0]["id"], "pm_t1");
        assert_eq!(videos[0]["episode"], 1);
        assert_eq!(videos[1]["id"], "pm_t2");
        assert_eq!(videos[1]["episode"], 2);
    }

    #[test]
    fn an_artist_detail_returns_only_that_artist() {
        let rows = vec![
            track("a1", "Be Her", "Ella Langley", true),
            track("a2", "Other", "Someone Else", true),
        ];
        let id = encode_artist("Ella Langley");
        let detail = artist_detail(&id, &rows).unwrap();
        assert_eq!(detail["name"], "Ella Langley");
        assert_eq!(detail["topTracks"].as_array().unwrap().len(), 1);
        assert!(artist_detail(&encode_artist("Nobody"), &rows).is_none());
    }
}
