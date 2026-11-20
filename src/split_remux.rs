//! Streaming Matroska remux for a multi-part upload.
//!
//! A `mkvmerge --split` set is N complete EBML documents. Concatenating them
//! byte-for-byte produces an EBML *stream*, not one Matroska document: every
//! demuxer reads the first `Segment` and stops (measured with ffmpeg 8.1.2:
//! two 3.000 s files `cat`'d together probe as 3.000 s, not 6).
//!
//! This module emits one `Segment` instead:
//!
//! * copy part 1's `EBML` header, rebuild its `Info` without `Duration`, copy its
//!   `Tracks`;
//! * keep the `Segment` size unknown (`01 FF FF FF FF FF FF FF`);
//! * for every `Cluster`, re-emit it with a fixed 8-byte `Timestamp` rebased by
//!   the running offset, then byte-copy the rest of the cluster;
//! * drop `SeekHead`, `Cues`, `Chapters`, `Tags` and `Attachments` (their
//!   positions are relative to the old document).
//!
//! Output length is not known up front, so the HTTP side is chunked and `Range`
//! is not honoured. That is the B-live stage; an index/Cues pass is B+.

use std::sync::Arc;

use bytes::Bytes;
use futures_util::future::BoxFuture;
use tokio::sync::mpsc::Sender;

pub type FetchResult = Result<Vec<u8>, String>;

/// `(part_index, part-local position, bytes)`.
pub type Fetcher = Arc<dyn Fn(usize, u64, u64) -> BoxFuture<'static, FetchResult> + Send + Sync>;

const ID_EBML: u32 = 0x1A45_DFA3;
const ID_SEGMENT: u32 = 0x1853_8067;
const ID_INFO: u32 = 0x1549_A966;
const ID_TRACKS: u32 = 0x1654_AE6B;
const ID_CLUSTER: u32 = 0x1F43_B675;
const ID_TIMESTAMP: u32 = 0xE7;
const ID_DURATION: u32 = 0x4489;
const ID_TIMECODE_SCALE: u32 = 0x2AD7_B1;
const ID_CRC32: u32 = 0xBF;
const ID_POSITION: u32 = 0xA7;
const ID_PREVSIZE: u32 = 0xAB;
const UNKNOWN: u64 = u64::MAX;
const MB: u64 = 1024 * 1024;
const READ_WINDOW: u64 = MB;

pub async fn stream(
    parts: &[u64],
    fetcher: Fetcher,
    tx: Sender<Result<Bytes, std::io::Error>>,
    duration_sec: Option<f64>,
) -> Result<(), String> {
    if parts.is_empty() {
        return Err("no parts".to_string());
    }
    let mut offset = 0u64;
    let mut last_ts = 0u64;

    for (index, &size) in parts.iter().enumerate() {
        let mut r = Reader::new(index, size, fetcher.clone());
        if size < 4 {
            return Err(format!("part {} is too short to be Matroska", index + 1));
        }

        let ebml = read_header(&mut r).await?;
        if ebml.id != ID_EBML {
            return Err(format!("part {} does not start with an EBML header", index + 1));
        }
        if ebml.size == UNKNOWN || ebml.size > MB {
            return Err("EBML header has an implausible size".to_string());
        }
        if index == 0 {
            emit(&tx, ebml.header.clone()).await?;
            emit(&tx, r.take(ebml.size as usize).await?).await?;
        } else {
            r.skip(ebml.size);
        }

        let segment = read_header(&mut r).await?;
        if segment.id != ID_SEGMENT {
            return Err(format!("part {} has no Segment", index + 1));
        }
        if index == 0 {
            emit(&tx, segment.header[..segment.id_len].to_vec()).await?;
            // Unknown size: legal (unknownsizeallowed) and removes the need to
            // know the output length before the first byte.
            emit(&tx, vec![0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]).await?;
        }

        while r.pos < size {
            let h = read_header(&mut r).await?;
            match h.id {
                ID_INFO if index == 0 => {
                    if h.size == UNKNOWN || h.size > MB {
                        return Err("Info has an implausible size".to_string());
                    }
                    let body = r.take(h.size as usize).await?;
                    let body = rewrite_info(&body, duration_sec)?;
                    emit(&tx, h.header[..h.id_len].to_vec()).await?;
                    emit(&tx, vint(body.len() as u64)).await?;
                    emit(&tx, body).await?;
                }
                ID_TRACKS if index == 0 => {
                    if h.size == UNKNOWN || h.size > 4 * MB {
                        return Err("Tracks has an implausible size".to_string());
                    }
                    let body = r.take(h.size as usize).await?;
                    emit(&tx, h.header[..h.id_len].to_vec()).await?;
                    emit(&tx, vint(body.len() as u64)).await?;
                    emit(&tx, body).await?;
                }
                ID_CLUSTER => {
                    handle_cluster(&mut r, h, &mut last_ts, offset, &tx).await?;
                }
                _ => {
                    if h.size == UNKNOWN {
                        return Err(format!(
                            "unknown-size element 0x{:X} in part {}",
                            h.id,
                            index + 1
                        ));
                    }
                    r.skip(h.size);
                }
            }
            if tx.is_closed() {
                return Ok(());
            }
        }

        // The next part restarts its timeline at 0. One tick after the last
        // cluster timestamp is enough to keep the output monotonic.
        offset = last_ts.saturating_add(1);
    }
    Ok(())
}

async fn emit(
    tx: &Sender<Result<Bytes, std::io::Error>>,
    data: Vec<u8>,
) -> Result<(), String> {
    if data.is_empty() {
        return Ok(());
    }
    tx.send(Ok(Bytes::from(data)))
        .await
        .map_err(|_| "client disconnected".to_string())
}

async fn handle_cluster(
    r: &mut Reader,
    h: Header,
    last_ts: &mut u64,
    offset: u64,
    tx: &Sender<Result<Bytes, std::io::Error>>,
) -> Result<(), String> {
    if h.size == UNKNOWN {
        return Err("cluster with unknown size is not supported".to_string());
    }
    let body_start = r.pos;
    let body_end = body_start
        .checked_add(h.size)
        .ok_or_else(|| "cluster size overflows".to_string())?;
    if body_end > r.size {
        return Err("cluster runs past the end of its part".to_string());
    }

    // `Timestamp` is only *recommended* to be first: mkvmerge writes a CRC-32
    // and/or Position/PrevSize before it. Keep the harmless prefix, drop the
    // stale position fields, then rewrite the Timestamp. The cluster size must
    // account for both the timestamp width change and the dropped children.
    let mut prefix: Vec<u8> = Vec::new();
    let mut dropped = 0u64;
    let mut old_ts_total = 0u64;
    let mut old_ts = 0u64;
    let mut found = false;
    while r.pos < body_end {
        let child = read_header(r).await?;
        if child.size == UNKNOWN {
            return Err("unknown-size child inside a Cluster".to_string());
        }
        let total = (child.header.len() as u64)
            .checked_add(child.size)
            .ok_or_else(|| "cluster child overflows".to_string())?;
        if r.pos + total > body_end {
            return Err("cluster child runs past its cluster".to_string());
        }
        if child.id == ID_TIMESTAMP {
            if child.size == 0 || child.size > 8 {
                return Err("cluster Timestamp has an invalid width".to_string());
            }
            let bytes = r.take(child.size as usize).await?;
            old_ts = be_uint(&bytes);
            old_ts_total = total;
            found = true;
            break;
        }
        if child.id == ID_CRC32 || child.id == ID_POSITION || child.id == ID_PREVSIZE {
            dropped += total;
            r.skip(child.size);
        } else {
            if prefix.len() as u64 + total > 64 * 1024 {
                return Err("cluster prefix before Timestamp is too large".to_string());
            }
            prefix.extend_from_slice(&child.header);
            prefix.extend_from_slice(&r.take(child.size as usize).await?);
        }
    }
    if !found {
        return Err("cluster has no Timestamp".to_string());
    }
    let new_ts = old_ts
        .checked_add(offset)
        .ok_or_else(|| "timestamp overflows u64".to_string())?;

    // ID 0xE7 + one-byte size 0x88 + 8-byte value.
    let new_total = 10u64;
    let new_size = h.size - old_ts_total - dropped + new_total;

    emit(tx, h.header[..h.id_len].to_vec()).await?;
    emit(tx, vint(new_size)).await?;
    emit(tx, prefix).await?;
    emit(tx, vec![0xE7, 0x88]).await?;
    emit(tx, new_ts.to_be_bytes().to_vec()).await?;

    let rest = body_end - r.pos;
    r.copy_to(rest, tx).await?;
    *last_ts = (*last_ts).max(new_ts);
    Ok(())
}

/// Rebuild `Info`, dropping its per-part `Duration` and, when the caller knows
/// the summed duration of all parts, writing that total instead. A stale part
/// duration would stop the timeline at part 1's length.
fn rewrite_info(body: &[u8], duration_sec: Option<f64>) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(body.len() + 11);
    let mut pos = 0usize;
    let mut scale = 1_000_000u64;
    while pos < body.len() {
        let (id, id_len) = id_at(body, pos).ok_or_else(|| "bad Info child id".to_string())?;
        let (size, size_len) = size_at(body, pos + id_len)
            .ok_or_else(|| "bad Info child size".to_string())?;
        if size == UNKNOWN {
            return Err("unknown-size child in Info".to_string());
        }
        let total = id_len
            .checked_add(size_len)
            .and_then(|n| n.checked_add(size as usize))
            .ok_or_else(|| "Info child overflows".to_string())?;
        if pos + total > body.len() {
            return Err("Info child runs past Info".to_string());
        }
        if id == ID_TIMECODE_SCALE {
            let start = pos + id_len + size_len;
            let mut v = 0u64;
            for &b in &body[start..start + size as usize] {
                v = (v << 8) | b as u64;
            }
            scale = v.max(1);
        }
        if id != ID_DURATION && id != ID_CRC32 {
            out.extend_from_slice(&body[pos..pos + total]);
        }
        pos += total;
    }
    if let Some(sec) = duration_sec {
        if sec.is_finite() && sec > 0.0 {
            let value = sec * 1_000_000_000.0 / scale.max(1) as f64;
            out.extend_from_slice(&[0x44, 0x89, 0x88]);
            out.extend_from_slice(&value.to_be_bytes());
        }
    }
    Ok(out)
}

/// Read one part's `Info.Duration` (seconds) without walking its clusters.
pub async fn probe_duration(fetcher: Fetcher, index: usize, size: u64) -> Option<f64> {
    let mut r = Reader::new(index, size, fetcher);
    let ebml = read_header(&mut r).await.ok()?;
    if ebml.id != ID_EBML || ebml.size == UNKNOWN || ebml.size > 1_048_576 {
        return None;
    }
    let _ = r.take(ebml.size as usize).await.ok()?;
    let segment = read_header(&mut r).await.ok()?;
    if segment.id != ID_SEGMENT {
        return None;
    }
    for _ in 0..8 {
        let h = read_header(&mut r).await.ok()?;
        match h.id {
            ID_INFO => {
                if h.size == UNKNOWN || h.size > 1_048_576 {
                    return None;
                }
                let body = r.take(h.size as usize).await.ok()?;
                return info_duration(&body);
            }
            ID_CLUSTER => return None,
            _ => {
                if h.size == UNKNOWN {
                    return None;
                }
                let _ = r.take(h.size as usize).await.ok()?;
            }
        }
    }
    None
}

fn info_duration(body: &[u8]) -> Option<f64> {
    let mut pos = 0usize;
    let mut scale = 1_000_000u64;
    let mut duration: Option<f64> = None;
    while pos < body.len() {
        let (id, id_len) = id_at(body, pos)?;
        let (size, size_len) = size_at(body, pos + id_len)?;
        if size == UNKNOWN {
            return None;
        }
        let start = pos + id_len + size_len;
        let end = start.checked_add(size as usize)?;
        if end > body.len() {
            return None;
        }
        match id {
            ID_TIMECODE_SCALE => {
                let mut v = 0u64;
                for &b in &body[start..end] {
                    v = (v << 8) | b as u64;
                }
                scale = v.max(1);
            }
            ID_DURATION => {
                duration = match size {
                    4 => Some(f32::from_be_bytes(body[start..end].try_into().ok()?) as f64),
                    8 => Some(f64::from_be_bytes(body[start..end].try_into().ok()?)),
                    _ => None,
                };
            }
            _ => {}
        }
        pos = end;
    }
    duration.map(|d| d * scale as f64 / 1_000_000_000.0)
}

/* ── EBML primitives ────────────────────────────────────────────────────── */

struct Header {
    id: u32,
    id_len: usize,
    size: u64,
    header: Vec<u8>,
}

struct Reader {
    index: usize,
    pos: u64,
    size: u64,
    buf: Vec<u8>,
    buf_start: u64,
    fetcher: Fetcher,
}

impl Reader {
    fn new(index: usize, size: u64, fetcher: Fetcher) -> Self {
        Reader {
            index,
            pos: 0,
            size,
            buf: Vec::new(),
            buf_start: 0,
            fetcher,
        }
    }

    fn skip(&mut self, n: u64) {
        self.pos = self.pos.saturating_add(n);
    }

    async fn ensure(&mut self, n: usize) -> Result<(), String> {
        if self.pos + n as u64 <= self.buf_start + self.buf.len() as u64 {
            return Ok(());
        }
        if self.pos >= self.size {
            return Err("unexpected end of part".to_string());
        }
        let align = self.pos - self.pos % 4096;
        let want = READ_WINDOW.min(self.size - align);
        if want == 0 {
            return Err("read past the end of part".to_string());
        }
        let data = (self.fetcher)(self.index, align, want).await?;
        if data.is_empty() {
            return Err("Telegram returned an empty read".to_string());
        }
        self.buf = data;
        self.buf_start = align;
        if self.pos + n as u64 > self.buf_start + self.buf.len() as u64 {
            return Err("short read".to_string());
        }
        Ok(())
    }

    async fn peek_u8(&mut self) -> Result<u8, String> {
        self.ensure(1).await?;
        Ok(self.buf[(self.pos - self.buf_start) as usize])
    }

    async fn take(&mut self, n: usize) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(n.min(READ_WINDOW as usize));
        let mut left = n;
        while left > 0 {
            self.ensure(1).await?;
            let start = (self.pos - self.buf_start) as usize;
            let avail = self.buf.len() - start;
            let take = avail.min(left);
            out.extend_from_slice(&self.buf[start..start + take]);
            self.pos += take as u64;
            left -= take;
        }
        Ok(out)
    }

    async fn copy_to(
        &mut self,
        mut n: u64,
        tx: &Sender<Result<Bytes, std::io::Error>>,
    ) -> Result<(), String> {
        while n > 0 {
            self.ensure(1).await?;
            let start = (self.pos - self.buf_start) as usize;
            let avail = self.buf.len() - start;
            let take = (avail as u64).min(n).min(256 * 1024);
            if take == 0 {
                return Err("zero-length copy window".to_string());
            }
            let chunk = Bytes::copy_from_slice(&self.buf[start..start + take as usize]);
            tx.send(Ok(chunk))
                .await
                .map_err(|_| "client disconnected".to_string())?;
            self.pos += take;
            n -= take;
        }
        Ok(())
    }
}

async fn read_header(r: &mut Reader) -> Result<Header, String> {
    let (id, id_len) = read_id(r).await?;
    let (size, size_len, size_bytes) = read_size(r).await?;
    let mut header = Vec::with_capacity(id_len + size_len);
    header.extend_from_slice(&id.to_be_bytes()[4 - id_len..]);
    header.extend_from_slice(&size_bytes);
    Ok(Header {
        id,
        id_len,
        size,
        header,
    })
}

async fn read_id(r: &mut Reader) -> Result<(u32, usize), String> {
    let b0 = r.peek_u8().await?;
    let len = if b0 & 0x80 != 0 {
        1
    } else if b0 & 0x40 != 0 {
        2
    } else if b0 & 0x20 != 0 {
        3
    } else if b0 & 0x10 != 0 {
        4
    } else {
        return Err(format!("invalid EBML id byte 0x{b0:02X}"));
    };
    let bytes = r.take(len).await?;
    let id = bytes.iter().fold(0u32, |a, b| (a << 8) | *b as u32);
    Ok((id, len))
}

async fn read_size(r: &mut Reader) -> Result<(u64, usize, Vec<u8>), String> {
    let b0 = r.peek_u8().await?;
    let len = b0.leading_zeros() as usize + 1;
    if len > 8 {
        return Err(format!("invalid EBML size byte 0x{b0:02X}"));
    }
    let bytes = r.take(len).await?;
    let mut val = (b0 as u64) & ((1u64 << (8 - len)) - 1);
    for b in &bytes[1..] {
        val = (val << 8) | *b as u64;
    }
    let unknown = val == (1u64 << (7 * len)) - 1;
    Ok((if unknown { UNKNOWN } else { val }, len, bytes))
}

fn vint(value: u64) -> Vec<u8> {
    for len in 1..=8usize {
        let max = (1u64 << (7 * len)) - 1;
        if value < max {
            let mut out = vec![0u8; len];
            let mut v = value;
            for i in (0..len).rev() {
                out[i] = (v & 0xFF) as u8;
                v >>= 8;
            }
            out[0] |= 1 << (8 - len);
            return out;
        }
    }
    Vec::new()
}

fn be_uint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0u64, |a, b| (a << 8) | *b as u64)
}

fn id_at(buf: &[u8], pos: usize) -> Option<(u32, usize)> {
    let b0 = *buf.get(pos)?;
    let len = if b0 & 0x80 != 0 {
        1
    } else if b0 & 0x40 != 0 {
        2
    } else if b0 & 0x20 != 0 {
        3
    } else if b0 & 0x10 != 0 {
        4
    } else {
        return None;
    };
    if pos + len > buf.len() {
        return None;
    }
    let id = buf[pos..pos + len]
        .iter()
        .fold(0u32, |a, b| (a << 8) | *b as u32);
    Some((id, len))
}

fn size_at(buf: &[u8], pos: usize) -> Option<(u64, usize)> {
    let b0 = *buf.get(pos)?;
    let len = b0.leading_zeros() as usize + 1;
    if len > 8 || pos + len > buf.len() {
        return None;
    }
    let mut val = (b0 as u64) & ((1u64 << (8 - len)) - 1);
    for b in &buf[pos + 1..pos + len] {
        val = (val << 8) | *b as u64;
    }
    if val == (1u64 << (7 * len)) - 1 {
        return Some((UNKNOWN, len));
    }
    Some((val, len))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;

    fn el(id: &[u8], body: &[u8]) -> Vec<u8> {
        let mut out = id.to_vec();
        out.extend(vint(body.len() as u64));
        out.extend_from_slice(body);
        out
    }

    fn part(block: u8) -> Vec<u8> {
        let info = el(
            &[0x15, 0x49, 0xA9, 0x66],
            &[
                el(&[0x2A, 0xD7, 0xB1], &[0x0F, 0x42, 0x40]), // TimestampScale = 1e6
                el(&[0x44, 0x89], &[0x40, 0x00, 0x00, 0x00]), // Duration = 2.0
            ]
            .concat(),
        );
        let tracks = el(&[0x16, 0x54, 0xAE, 0x6B], &[0xAE, 0x84, 0x81, 0x01, 0x00, 0x00]);
        let cluster = el(
            &[0x1F, 0x43, 0xB6, 0x75],
            &[
                el(&[0xE7], &[0x00, 0x00]), // Timestamp 0
                el(&[0xA3], &[0x81, 0x00, 0x00, 0x80, block]), // fake block
            ]
            .concat(),
        );
        let mut out = el(&[0x1A, 0x45, 0xDF, 0xA3], &[0x42, 0x86, 0x81, 0x01]);
        out.extend(el(&[0x18, 0x53, 0x80, 0x67], &[info, tracks, cluster].concat()));
        out
    }

    fn local_fetcher(data: Arc<Vec<Vec<u8>>>) -> Fetcher {
        Arc::new(move |i, pos, want| {
            let data = data.clone();
            async move {
                let part = &data[i];
                let start = pos as usize;
                if start >= part.len() {
                    return Err("eof".to_string());
                }
                let end = (start + want as usize).min(part.len());
                Ok(part[start..end].to_vec())
            }
            .boxed()
        })
    }

    #[tokio::test]
    async fn remux_rebases_clusters_and_drops_duration() {
        let p1 = part(0x11);
        let p2 = part(0x22);
        let sizes = vec![p1.len() as u64, p2.len() as u64];
        let data = Arc::new(vec![p1, p2]);
        let (tx, mut rx) = tokio::sync::mpsc::channel(128);
        let fetcher = local_fetcher(data);
        let task = tokio::spawn(async move { stream(&sizes, fetcher, tx, None).await });

        let mut out = Vec::new();
        while let Some(item) = rx.recv().await {
            out.extend(item.unwrap());
        }
        task.await.unwrap().unwrap();

        // One EBML header and one Segment, not two documents.
        assert_eq!(
            out.windows(4).filter(|w| *w == ID_EBML.to_be_bytes()).count(),
            1
        );
        assert_eq!(
            out.windows(4).filter(|w| *w == ID_SEGMENT.to_be_bytes()).count(),
            1
        );
        // Duration is gone.
        assert!(!out.windows(2).any(|w| w == [0x44, 0x89]));

        // Cluster 1 keeps timestamp 0; cluster 2 starts one tick after it.
        let ts: Vec<u64> = out
            .windows(10)
            .filter(|w| w[0] == 0xE7 && w[1] == 0x88)
            .map(|w| u64::from_be_bytes(w[2..10].try_into().unwrap()))
            .collect();
        assert_eq!(ts, vec![0, 1]);
    }

    #[tokio::test]
    async fn remux_writes_the_summed_duration() {
        let p1 = part(0x11);
        let p2 = part(0x22);
        let sizes = vec![p1.len() as u64, p2.len() as u64];
        let data = Arc::new(vec![p1, p2]);
        let (tx, mut rx) = tokio::sync::mpsc::channel(128);
        let fetcher = local_fetcher(data);
        let task = tokio::spawn(async move {
            stream(&sizes, fetcher, tx, Some(4.0)).await
        });

        let mut out = Vec::new();
        while let Some(item) = rx.recv().await {
            out.extend(item.unwrap());
        }
        task.await.unwrap().unwrap();

        let at = out
            .windows(3)
            .position(|w| w == [0x44, 0x89, 0x88])
            .expect("a Duration element");
        let value = f64::from_be_bytes(out[at + 3..at + 11].try_into().unwrap());
        assert!((value - 4000.0).abs() < 0.001, "Duration = {value}");
    }
}

