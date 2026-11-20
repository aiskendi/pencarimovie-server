//! B+ seekable remux for container parts.
//!
//! The parts are complete Matroska documents. A live remux can only stream them
//! linearly; players will not seek it because it has no `SeekHead` pointing at
//! `Cues` and no exact length. This module builds that index first, then serves
//! a deterministic byte stream whose every region is known:
//!
//! ```text
//! EBML | Segment(size) | SeekHead | Info | Tracks | Cluster… | Cues
//! ```
//!
//! Only cluster headers are read while indexing (the parts’ own `Cues` give the
//! cluster positions), and a viewer’s `Range` regenerates just the clusters it
//! asks for. The live path in `split_remux` stays as the fallback.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use bytes::Bytes;
use futures_util::StreamExt;
use tokio::sync::mpsc::Sender;

use crate::split_remux::Fetcher;

const TAIL: u64 = 128 * 1024;
const ID_EBML: [u8; 4] = [0x1A, 0x45, 0xDF, 0xA3];
const ID_SEGMENT: [u8; 4] = [0x18, 0x53, 0x80, 0x67];
const ID_INFO: [u8; 4] = [0x15, 0x49, 0xA9, 0x66];
const ID_TRACKS: [u8; 4] = [0x16, 0x54, 0xAE, 0x6B];
const ID_SEEKHEAD: [u8; 4] = [0x11, 0x4D, 0x9B, 0x74];
const ID_CUES: [u8; 4] = [0x1C, 0x53, 0xBB, 0x6B];
const ID_CLUSTER: [u8; 4] = [0x1F, 0x43, 0xB6, 0x75];
const ID_CRC: [u8; 2] = [0xBF, 0x84];
const ID_TIMESTAMP: u8 = 0xE7;

#[derive(Debug, Clone)]
pub struct Cluster {
    pub part: usize,
    pub input_pos: u64,
    pub input_size: u64,
    pub output_pos: u64,
    pub output_size: u64,
    pub time: u64,
    pub in_size_len: u8,
    pub in_body_len: u64,
    pub body_prefix: u8,
    pub old_ts_len: u8,
    pub out_size_len: u8,
}

#[derive(Debug)]
pub struct Index {
    /// EBML header + known-size Segment header + SeekHead/Info/Tracks.
    pub header: Vec<u8>,
    /// Length of SeekHead+Info+Tracks inside `header` (payload-relative base).
    pub payload_len: u64,
    pub clusters: Vec<Cluster>,
    pub cues: Vec<u8>,
    pub total: u64,
}

fn vint_len(n: u64) -> usize {
    for len in 1..=8usize {
        let max = if len == 8 {
            u64::MAX - 1
        } else {
            (1u64 << (7 * len)) - 2
        };
        if n <= max {
            return len;
        }
    }
    8
}

fn put_vint(out: &mut Vec<u8>, n: u64) {
    let len = vint_len(n);
    let mut buf = [0u8; 8];
    let mut v = n;
    for i in (0..len).rev() {
        buf[i] = (v & 0xFF) as u8;
        v >>= 8;
    }
    buf[0] |= 1 << (8 - len);
    out.extend_from_slice(&buf[..len]);
}

fn put_elem(out: &mut Vec<u8>, id: &[u8], body: &[u8]) {
    out.extend_from_slice(id);
    put_vint(out, body.len() as u64);
    out.extend_from_slice(body);
}

fn put_uint(out: &mut Vec<u8>, id: &[u8], value: u64) {
    let len = vint_len(value).min(8);
    let mut body = vec![0u8; len];
    for i in (0..len).rev() {
        body[i] = (value >> (8 * (len - 1 - i))) as u8;
    }
    put_elem(out, id, &body);
}

fn get_u(b: &[u8]) -> u64 {
    let mut v = 0u64;
    for &x in b {
        v = (v << 8) | x as u64;
    }
    v
}

fn id_at(b: &[u8], p: usize) -> Option<(u32, usize)> {
    let first = *b.get(p)?;
    let len = 1 + first.leading_zeros() as usize;
    if len > 4 || p + len > b.len() {
        return None;
    }
    let mut v = 0u32;
    for i in 0..len {
        v = (v << 8) | b[p + i] as u32;
    }
    Some((v, len))
}

fn size_at(b: &[u8], p: usize) -> Option<(u64, usize)> {
    let first = *b.get(p)?;
    let len = 1 + first.leading_zeros() as usize;
    if len > 8 || p + len > b.len() {
        return None;
    }
    let data_bits = 8 - len;
    let mask = if data_bits == 0 { 0 } else { (1u64 << data_bits) - 1 };
    let mut v = first as u64 & mask;
    for i in 1..len {
        v = (v << 8) | b[p + i] as u64;
    }
    // Unknown size is the marker followed by all-one data bits.
    if v == mask && (1..len).all(|i| b[p + i] == 0xFF) {
        return None;
    }
    Some((v, len))
}

fn children(b: &[u8], mut p: usize, end: usize) -> Vec<(u32, usize, usize)> {
    let mut out = Vec::new();
    while p < end {
        let Some((id, il)) = id_at(b, p) else { break };
        let Some((size, sl)) = size_at(b, p + il) else { break };
        let body = p + il + sl;
        let Some(e) = body.checked_add(size as usize) else { break };
        if e > end {
            break;
        }
        out.push((id, body, e));
        p = e;
    }
    out
}

async fn read_at(
    fetcher: &Fetcher,
    part: usize,
    pos: u64,
    len: usize,
) -> Result<Vec<u8>, String> {
    let aligned = pos & !4095;
    let skip = (pos - aligned) as usize;
    let want = ((skip + len + 4095) / 4096 * 4096) as u64;
    let data = fetcher(part, aligned, want).await?;
    if data.len() < skip + len {
        return Err("short read".to_string());
    }
    Ok(data[skip..skip + len].to_vec())
}

/// Cluster-body reads must start on a 512 KiB boundary or grammers fetches a
/// swarm of 4 KiB parts and the body stalls after a couple of megabytes.
async fn read_aligned(
    fetcher: &Fetcher,
    part: usize,
    pos: u64,
    len: usize,
) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(len);
    let mut p = pos;
    let mut left = len;
    while left > 0 {
        let aligned = p & !4095;
        let skip = (p - aligned) as usize;
        // End this window at the next 1 MiB boundary, exactly like split_windows.
        let boundary = 1024 * 1024 - (aligned % (1024 * 1024));
        let want = ((skip + left + 4095) / 4096 * 4096).min(boundary as usize) as u64;
        if want <= skip as u64 {
            return Err("empty aligned window".to_string());
        }
        let data = fetcher(part, aligned, want).await?;
        if data.len() <= skip {
            return Err("short aligned read".to_string());
        }
        let take = (data.len() - skip).min(left);
        out.extend_from_slice(&data[skip..skip + take]);
        p += take as u64;
        left -= take;
    }
    Ok(out)
}

async fn read_tail(fetcher: &Fetcher, part: usize, size: u64) -> Result<(u64, Vec<u8>), String> {
    let len = size.min(TAIL);
    let start = size - len;
    // A 4 KiB-aligned tail read makes grammers fetch dozens of 4 KiB parts.
    // One 512 KiB-aligned window covers the whole Cues tail in a single part.
    let aligned = start & !(512 * 1024 - 1);
    let want = (size - aligned).min(512 * 1024) as usize;
    let data = crate::split_index::read_at(fetcher, part, aligned, want).await?;
    let skip = (start - aligned) as usize;
    if data.len() < skip + len as usize {
        return Err("short tail read".to_string());
    }
    Ok((start, data[skip..skip + len as usize].to_vec()))
}

fn parse_head(prefix: &[u8]) -> Result<(usize, usize, Vec<u8>, Vec<u8>), String> {
    let (id, il) = id_at(prefix, 0).ok_or("no EBML header")?;
    if id != get_u(&ID_EBML) as u32 {
        return Err("part does not start with EBML".to_string());
    }
    let (sz, sl) = size_at(prefix, il).ok_or("bad EBML size")?;
    let ebml_end = il + sl + sz as usize;
    let (sid, sil) = id_at(prefix, ebml_end).ok_or("no Segment")?;
    if sid != get_u(&ID_SEGMENT) as u32 {
        return Err("no Segment".to_string());
    }
    let (ssz, ssl) = size_at(prefix, ebml_end + sil).ok_or("bad Segment size")?;
    let data_start = ebml_end + sil + ssl;
    let seg_end = (data_start + ssz as usize).min(prefix.len());
    let mut info = Vec::new();
    let mut tracks = Vec::new();
    for (id, s, e) in children(prefix, data_start, seg_end) {
        if id == get_u(&ID_INFO) as u32 && info.is_empty() {
            put_elem(&mut info, &ID_INFO, &prefix[s..e]);
        } else if id == get_u(&ID_TRACKS) as u32 && tracks.is_empty() {
            put_elem(&mut tracks, &ID_TRACKS, &prefix[s..e]);
        }
    }
    if info.is_empty() || tracks.is_empty() {
        return Err("part 1 has no Info/Tracks in its prefix".to_string());
    }
    Ok((ebml_end, data_start, info, tracks))
}

fn parse_info(body: &[u8]) -> (u64, Option<f64>) {
    let mut scale = 1_000_000u64;
    let mut dur = None;
    for (id, s, e) in children(body, 0, body.len()) {
        if id == 0x2AD7B1 {
            scale = get_u(&body[s..e]).max(1);
        } else if id == 0x4489 {
            dur = match e - s {
                4 => Some(f32::from_be_bytes(body[s..e].try_into().unwrap()) as f64),
                8 => Some(f64::from_be_bytes(body[s..e].try_into().unwrap())),
                _ => None,
            };
        }
    }
    (scale, dur)
}

fn rewrite_info(raw: &[u8], total_ticks: f64) -> Result<(Vec<u8>, u64, f64), String> {
    let (id, il) = id_at(raw, 0).ok_or("Info id")?;
    if id != get_u(&ID_INFO) as u32 {
        return Err("not Info".to_string());
    }
    let (sz, sl) = size_at(raw, il).ok_or("Info size")?;
    let body = &raw[il + sl..il + sl + sz as usize];
    let (scale, _) = parse_info(body);
    let mut out = Vec::with_capacity(body.len() + 11);
    let mut p = 0usize;
    while p < body.len() {
        let (cid, cil) = id_at(body, p).ok_or("Info child id")?;
        let (csz, csl) = size_at(body, p + cil).ok_or("Info child size")?;
        let e = p + cil + csl + csz as usize;
        if cid != 0x4489 && cid != 0xBF {
            out.extend_from_slice(&body[p..e]);
        }
        p = e;
    }
    out.extend_from_slice(&[0x44, 0x89, 0x88]);
    out.extend_from_slice(&total_ticks.to_be_bytes());
    let mut elem = Vec::new();
    put_elem(&mut elem, &ID_INFO, &out);
    Ok((elem, scale, total_ticks))
}

fn duration_from_prefix(prefix: &[u8]) -> Option<f64> {
    let (_, _, info, _) = parse_head(prefix).ok()?;
    let (isz, isl) = size_at(&info, 4)?;
    let body = &info[4 + isl..4 + isl + isz as usize];
    let (scale, dur) = parse_info(body);
    dur.map(|d| d * scale as f64 / 1_000_000_000.0)
}

fn parse_cues(tail: &[u8], tail_start: u64, part_size: u64) -> Option<(Vec<(u64, u64)>, u64)> {
    let mut cue_at = None;
    for i in (0..tail.len().saturating_sub(4)).rev() {
        if tail[i..i + 4] == ID_CUES {
            cue_at = Some(i);
            break;
        }
    }
    let c = cue_at?;
    let (sz, sl) = size_at(tail, c + 4)?;
    let body = c + 4 + sl;
    let end = (body + sz as usize).min(tail.len());
    let mut rows = Vec::new();
    for (id, s, e) in children(tail, body, end) {
        if id != 0xBB {
            continue;
        }
        let mut time = None;
        let mut pos = None;
        for (cid, cs, ce) in children(tail, s, e) {
            if cid == 0xB3 {
                time = Some(get_u(&tail[cs..ce]));
            } else if cid == 0xB7 {
                let mut track = None;
                let mut cpos = None;
                for (tid, ts, te) in children(tail, cs, ce) {
                    if tid == 0xF7 {
                        track = Some(get_u(&tail[ts..te]));
                    } else if tid == 0xF1 {
                        cpos = Some(get_u(&tail[ts..te]));
                    }
                }
                if track == Some(1) {
                    pos = cpos;
                }
            }
        }
        if let (Some(t), Some(p)) = (time, pos) {
            rows.push((t, p));
        }
    }
    if rows.len() < 2 {
        return None;
    }
    let cues_global = tail_start + c as u64;
    let _ = part_size;
    Some((rows, cues_global))
}

fn build_cues(clusters: &[Cluster], track: u64) -> Vec<u8> {
    let mut body = Vec::new();
    for c in clusters {
        let mut cp = Vec::new();
        put_uint(&mut cp, &[0xB3], c.time);
        let mut tp = Vec::new();
        put_uint(&mut tp, &[0xF7], track);
        put_uint(&mut tp, &[0xF1], c.output_pos);
        put_elem(&mut cp, &[0xB7], &tp);
        put_elem(&mut body, &[0xBB], &cp);
    }
    let mut out = Vec::new();
    put_elem(&mut out, &ID_CUES, &body);
    out
}

fn build_seekhead(info_pos: u64, tracks_pos: u64, cues_pos: u64) -> Vec<u8> {
    let mut body = Vec::new();
    for (id, pos) in [
        (ID_INFO, info_pos),
        (ID_TRACKS, tracks_pos),
        (ID_CUES, cues_pos),
    ] {
        let mut seek = Vec::new();
        put_elem(&mut seek, &[0x53, 0xAB], &id);
        // fixed 8-byte position keeps SeekHead length stable across rebuilds
        let mut p = vec![0u8; 8];
        p[0] = (pos >> 56) as u8;
        p[1] = (pos >> 48) as u8;
        p[2] = (pos >> 40) as u8;
        p[3] = (pos >> 32) as u8;
        p[4] = (pos >> 24) as u8;
        p[5] = (pos >> 16) as u8;
        p[6] = (pos >> 8) as u8;
        p[7] = pos as u8;
        put_elem(&mut seek, &[0x53, 0xAC], &p);
        put_elem(&mut body, &[0x4D, 0xBB], &seek);
    }
    let mut out = Vec::new();
    put_elem(&mut out, &ID_SEEKHEAD, &body);
    out
}

fn encode_index(idx: &Index) -> serde_json::Value {
    let clusters: Vec<serde_json::Value> = idx
        .clusters
        .iter()
        .map(|c| {
            serde_json::json!([
                c.part,
                c.input_pos,
                c.input_size,
                c.output_pos,
                c.output_size,
                c.time,
                c.in_size_len,
                c.in_body_len,
                c.body_prefix,
                c.old_ts_len,
                c.out_size_len
            ])
        })
        .collect();
    serde_json::json!({
        "header": STANDARD.encode(&idx.header),
        "payload_len": idx.payload_len,
        "cues": STANDARD.encode(&idx.cues),
        "total": idx.total,
        "clusters": clusters,
    })
}

fn decode_index(v: &serde_json::Value) -> Option<Index> {
    let header = STANDARD.decode(v.get("header")?.as_str()?).ok()?;
    let cues = STANDARD.decode(v.get("cues")?.as_str()?).ok()?;
    let payload_len = v.get("payload_len")?.as_u64()?;
    let total = v.get("total")?.as_u64()?;
    let mut clusters = Vec::new();
    for a in v.get("clusters")?.as_array()? {
        let x = a.as_array()?;
        clusters.push(Cluster {
            part: x.first()?.as_u64()? as usize,
            input_pos: x.get(1)?.as_u64()?,
            input_size: x.get(2)?.as_u64()?,
            output_pos: x.get(3)?.as_u64()?,
            output_size: x.get(4)?.as_u64()?,
            time: x.get(5)?.as_u64()?,
            in_size_len: x.get(6)?.as_u64()? as u8,
            in_body_len: x.get(7)?.as_u64()?,
            body_prefix: x.get(8)?.as_u64()? as u8,
            old_ts_len: x.get(9)?.as_u64()? as u8,
            out_size_len: x.get(10)?.as_u64()? as u8,
        });
    }
    Some(Index {
        header,
        payload_len,
        clusters,
        cues,
        total,
    })
}

/// Build and cache the index for one part set. `None` means the parts did not
/// look like a compatible pair; the caller falls back to the live remux.
pub async fn build(
    key: String,
    parts: Vec<(String, u64)>,
    fetcher: Fetcher,
) -> Result<Arc<Index>, String> {
    if let Some(hit) = cache().lock().ok().and_then(|m| m.get(&key).cloned()) {
        return Ok(hit);
    }
    if let Some(v) = crate::cache::get("split_index", &key) {
        if let Some(index) = decode_index(&v) {
            let index = Arc::new(index);
            if let Ok(mut m) = cache().lock() {
                m.insert(key.clone(), index.clone());
            }
            return Ok(index);
        }
    }
    let sizes: Vec<u64> = parts.iter().map(|p| p.1).collect();
    let prefix = read_at(&fetcher, 0, 0, 4096).await?;
    let (_, data_start, info_raw, tracks_raw) = parse_head(&prefix)?;

    let mut durations = vec![duration_from_prefix(&prefix).unwrap_or(0.0)];
    for i in 1..sizes.len() {
        let p = read_at(&fetcher, i, 0, 4096).await?;
        durations.push(duration_from_prefix(&p).unwrap_or(0.0));
    }
    let (isz, isl) = size_at(&info_raw, 4).ok_or("Info size")?;
    let scale = parse_info(&info_raw[4 + isl..4 + isl + isz as usize])
        .0
        .max(1);
    let total_ticks: f64 = durations.iter().map(|s| s * 1_000_000_000.0 / scale as f64).sum();
    let (info_elem, _, _) = rewrite_info(&info_raw, total_ticks)?;

    let mut clusters: Vec<Cluster> = Vec::new();
    let mut part_offset_ticks = 0u64;
    for (pi, &size) in sizes.iter().enumerate() {
        let (tail_start, tail) = read_tail(&fetcher, pi, size).await?;
        let (rows, cues_global) = parse_cues(&tail, tail_start, size)
            .ok_or_else(|| format!("part {} has no usable Cues", pi + 1))?;
        for (i, &(t, p)) in rows.iter().enumerate() {
            let input_pos = data_start as u64 + p;
            let input_size = if i + 1 < rows.len() {
                let next = data_start as u64 + rows[i + 1].1;
                next.saturating_sub(input_pos)
            } else {
                cues_global.saturating_sub(input_pos)
            };
            if input_size < 12 {
                continue;
            }
            let time_local = t;
            let mut body = input_size.saturating_sub(5);
            let mut in_len = 1usize;
            for len in 1..=8usize {
                body = input_size - 4 - len as u64;
                if vint_len(body) == len {
                    in_len = len;
                    break;
                }
            }
            let old_ts = 2 + vint_len(time_local);
            let new_body = body - old_ts as u64 + 10;
            let out_len = vint_len(new_body);
            let output_size = 4 + out_len as u64 + new_body;
            clusters.push(Cluster {
                part: pi,
                input_pos,
                input_size,
                output_pos: 0,
                output_size,
                time: part_offset_ticks + time_local,
                in_size_len: in_len as u8,
                in_body_len: body,
                body_prefix: 6,
                old_ts_len: old_ts as u8,
                out_size_len: out_len as u8,
            });
        }
        part_offset_ticks += (durations[pi] * 1_000_000_000.0 / scale as f64) as u64;
    }
    if clusters.is_empty() {
        return Err("no clusters".to_string());
    }

    // Two-pass SeekHead: its length is stable because positions are fixed width.
    let probe = build_seekhead(0, 0, 0);
    let info_pos = probe.len() as u64;
    let tracks_pos = info_pos + info_elem.len() as u64;
    let mut cluster_total = 0u64;
    for c in &clusters {
        cluster_total += c.output_size;
    }
    let cues_pos = probe.len() as u64 + info_elem.len() as u64 + tracks_raw.len() as u64 + cluster_total;
    let seekhead = build_seekhead(info_pos, tracks_pos, cues_pos);
    let mut header_payload = Vec::new();
    header_payload.extend_from_slice(&seekhead);
    header_payload.extend_from_slice(&info_elem);
    header_payload.extend_from_slice(&tracks_raw);

    let mut at = header_payload.len() as u64;
    for c in &mut clusters {
        c.output_pos = at;
        at += c.output_size;
    }
    // Only now do the cluster positions exist; building Cues earlier wrote 0
    // into every CueClusterPosition, which makes ExoPlayer's seek map useless.
    let cues = build_cues(&clusters, 1);
    let segment_payload = header_payload.len() as u64 + cluster_total + cues.len() as u64;
    let mut segment_header = Vec::new();
    segment_header.extend_from_slice(&ID_SEGMENT);
    let mut size_body = [0u8; 8];
    size_body[0] = 0x01;
    size_body[1] = (segment_payload >> 48) as u8;
    size_body[2] = (segment_payload >> 40) as u8;
    size_body[3] = (segment_payload >> 32) as u8;
    size_body[4] = (segment_payload >> 24) as u8;
    size_body[5] = (segment_payload >> 16) as u8;
    size_body[6] = (segment_payload >> 8) as u8;
    size_body[7] = segment_payload as u8;
    segment_header.extend_from_slice(&size_body);

    let mut header = Vec::new();
    header.extend_from_slice(&prefix[..parse_head(&prefix)?.0]);
    header.extend_from_slice(&segment_header);
    header.extend_from_slice(&header_payload);
    let total = header.len() as u64 + cluster_total + cues.len() as u64;
    let index = Arc::new(Index {
        header,
        payload_len: header_payload.len() as u64,
        clusters,
        cues,
        total,
    });
    if let Ok(mut m) = cache().lock() {
        if m.len() >= 32 {
            m.clear();
        }
        m.insert(key.clone(), index.clone());
    }
    crate::cache::put(
        "split_index",
        &key,
        &encode_index(&index),
        7 * 24 * 3600,
        0,
    );
    Ok(index)
}

fn cache() -> &'static Mutex<HashMap<String, Arc<Index>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<Index>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn seg_slice(seg_start: u64, seg_len: u64, from: u64, to: u64) -> Option<(usize, usize)> {
    let s = from.max(seg_start);
    let e = to.min(seg_start + seg_len);
    if s < e {
        Some(((s - seg_start) as usize, (e - seg_start) as usize))
    } else {
        None
    }
}

/// Stream only the requested part of one output cluster. The body copy is done
/// in 1 MiB pieces, so a 13 MB cluster does not delay first byte.
pub async fn stream_cluster_range(
    fetcher: &Fetcher,
    c: &Cluster,
    from: u64,
    to: u64,
    tx: &Sender<Result<Bytes, std::io::Error>>,
) -> Result<(), String> {
    if from >= to {
        return Ok(());
    }
    let new_body = c.output_size - 4 - c.out_size_len as u64;
    let mut header = Vec::with_capacity(12);
    header.extend_from_slice(&ID_CLUSTER);
    put_vint(&mut header, new_body);
    let header_len = header.len() as u64;
    let prefix = c.body_prefix as u64;
    let ts_len = 10u64;
    let rest_len = c.in_body_len - prefix - c.old_ts_len as u64;

    if let Some((s, e)) = seg_slice(0, header_len, from, to) {
        if tx.send(Ok(Bytes::copy_from_slice(&header[s..e]))).await.is_err() {
            return Ok(());
        }
    }
    if prefix > 0 {
        if let Some((s, e)) = seg_slice(header_len, prefix, from, to) {
            let data = read_at(
                fetcher,
                c.part,
                c.input_pos + 4 + c.in_size_len as u64,
                prefix as usize,
            )
            .await?;
            if tx.send(Ok(Bytes::copy_from_slice(&data[s..e]))).await.is_err() {
                return Ok(());
            }
        }
    }
    let ts_start = header_len + prefix;
    if let Some((s, e)) = seg_slice(ts_start, ts_len, from, to) {
        let mut ts = Vec::with_capacity(10);
        ts.extend_from_slice(&[ID_TIMESTAMP, 0x88]);
        ts.extend_from_slice(&c.time.to_be_bytes());
        if tx.send(Ok(Bytes::copy_from_slice(&ts[s..e]))).await.is_err() {
            return Ok(());
        }
    }
    let rest_out = ts_start + ts_len;
    let rest_in = c.input_pos + 4 + c.in_size_len as u64 + prefix + c.old_ts_len as u64;
    // One window plan ending on 1 MiB boundaries. Independent jobs that each
    // start at an arbitrary 4 KiB offset make grammers fetch a swarm of 4 KiB
    // parts — the measured 0.5 MB/s path.
    let mut jobs: Vec<(u64, u64, usize, u64, usize, usize)> = Vec::new();
    let mut rel = 0u64;
    while rel < rest_len {
        let in_pos = rest_in + rel;
        let aligned = in_pos & !(512 * 1024 - 1);
        let skip = (in_pos - aligned) as usize;
        let want = 512 * 1024u64;
        let avail = (rest_len - rel) as usize + skip;
        let want = want.min(avail as u64);
        if want <= skip as u64 {
            break;
        }
        let produced = want - skip as u64;
        let out_rel = rest_out + rel;
        if let Some((s, e)) = seg_slice(out_rel, produced, from, to) {
            jobs.push((aligned, want, skip, out_rel, s, e));
        }
        rel += produced;
    }
    if !jobs.is_empty() {
        let stream = futures_util::stream::iter(jobs.into_iter().map(
            |(aligned, want, skip, out_rel, s, e)| {
                let fetcher = fetcher.clone();
                let part = c.part;
                async move {
                    let data = fetcher(part, aligned, want).await?;
                    Ok::<_, String>((data, skip, out_rel, s, e))
                }
            },
        ))
        .buffered(8);
        futures_util::pin_mut!(stream);
        while let Some(item) = futures_util::StreamExt::next(&mut stream).await {
            let (data, skip, _out_rel, s, e) = item?;
            if skip + e > data.len() {
                return Err("short window".to_string());
            }
            if tx
                .send(Ok(Bytes::copy_from_slice(&data[skip + s..skip + e])))
                .await
                .is_err()
            {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// Regenerate one whole output cluster (used by tests; serving streams pieces).
pub async fn render_cluster(fetcher: &Fetcher, c: &Cluster) -> Result<Vec<u8>, String> {
    let raw = read_at(fetcher, c.part, c.input_pos, c.input_size as usize).await?;
    let (id, il) = id_at(&raw, 0).ok_or("cluster id")?;
    if id != get_u(&ID_CLUSTER) as u32 {
        return Err("not a cluster".to_string());
    }
    let (sz, sl) = size_at(&raw, il).ok_or("cluster size")?;
    let body = &raw[il + sl..il + sl + sz as usize];
    let mut p = 0usize;
    if body.len() >= 2 && body[0..2] == ID_CRC {
        p = 6;
    }
    if p >= body.len() || body[p] != ID_TIMESTAMP {
        return Err("no Timestamp first in cluster".to_string());
    }
    let (tsz, tsl2) = size_at(body, p + 1).ok_or("timestamp size")?;
    let old_len = 1 + tsl2 + tsz as usize;
    if p + old_len > body.len() {
        return Err("timestamp overruns cluster".to_string());
    }
    let mut new_body = Vec::with_capacity(body.len() - old_len + 9);
    new_body.extend_from_slice(&body[..p]);
    new_body.extend_from_slice(&[ID_TIMESTAMP, 0x88]);
    new_body.extend_from_slice(&c.time.to_be_bytes());
    new_body.extend_from_slice(&body[p + old_len..]);
    let mut out = Vec::new();
    out.extend_from_slice(&ID_CLUSTER);
    put_vint(&mut out, new_body.len() as u64);
    out.extend_from_slice(&new_body);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vint_round_trip() {
        for n in [0u64, 1, 127, 128, 16_383, 16_384, 1 << 40] {
            let mut b = Vec::new();
            put_vint(&mut b, n);
            assert_eq!(size_at(&b, 0).unwrap().0, n);
        }
    }

    #[test]
    fn real_prefix_has_info_and_tracks() {
        let Ok(b) = std::fs::read("storage/_part_NszfTW.mkv") else {
            return;
        };
        let p = &b[..4096.min(b.len())];
        assert!(parse_head(p).is_ok());
    }
}
