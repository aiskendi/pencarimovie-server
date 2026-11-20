//! Read a movie straight out of a split ZIP volume, on the fly.
//!
//! A `.mkv.zip.001` set is a ZIP file split at a byte boundary (the measured
//! `.001` is exactly 4,000 MiB). If the movie entry is STORED (method 0), its
//! bytes are contiguous inside the archive, so one HTTP `Range` maps to
//! `entry.data_start + range.start` over the existing virtual concatenation.
//!
//! The probe reads only the tail (EOCD / ZIP64 locator), the central directory,
//! and the entry's local header. No extraction, no full download.
//!
//! Deflated entries are reported as such; the caller refuses them because they
//! have no random access (see `docs/research/zip-split-streaming.md`).

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use crate::split_remux::Fetcher;

pub const TAIL_BYTES: u64 = 128 * 1024;
const MAX_CD: u64 = 8 * 1024 * 1024;
const MB: u64 = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ZipEntry {
    pub name: String,
    pub method: u16,
    pub comp_size: u64,
    pub uncomp_size: u64,
    /// Absolute offset of the entry's first data byte in the virtual archive.
    pub data_start: u64,
    /// Central-directory local-header offset (kept for the local probe).
    pub local_header_offset: u64,
}

struct VirtualFile {
    sizes: Vec<u64>,
    total: u64,
    fetcher: Fetcher,
}

impl VirtualFile {
    fn new(sizes: Vec<u64>, fetcher: Fetcher) -> Self {
        let total = sizes.iter().sum();
        VirtualFile {
            sizes,
            total,
            fetcher,
        }
    }

    fn locate(&self, pos: u64) -> Option<(usize, u64)> {
        let mut base = 0u64;
        for (i, &size) in self.sizes.iter().enumerate() {
            if pos < base + size {
                return Some((i, pos - base));
            }
            base += size;
        }
        None
    }

    async fn read(&self, mut pos: u64, len: usize) -> Result<Vec<u8>, String> {
        let mut out = Vec::with_capacity(len);
        let mut left = len;
        while left > 0 {
            let (part, local) = self
                .locate(pos)
                .ok_or_else(|| "ZIP read past the end".to_string())?;
            let skip = local % 4096;
            let aligned = local - skip;
            let part_size = self.sizes[part];
            let want = (skip + left as u64)
                .min(MB)
                .min(part_size - aligned);
            if want == 0 {
                return Err("empty ZIP read".to_string());
            }
            let data = (self.fetcher)(part, aligned, want).await?;
            let start = skip as usize;
            if data.len() <= start {
                return Err("short ZIP read".to_string());
            }
            let take = (data.len() - start).min(left);
            out.extend_from_slice(&data[start..start + take]);
            pos += take as u64;
            left -= take;
        }
        Ok(out)
    }
}

fn u16le(b: &[u8], o: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(o..o + 2)?.try_into().ok()?))
}

fn u32le(b: &[u8], o: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(o..o + 4)?.try_into().ok()?))
}

fn u64le(b: &[u8], o: usize) -> Option<u64> {
    Some(u64::from_le_bytes(b.get(o..o + 8)?.try_into().ok()?))
}

fn is_media_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    [".mkv", ".mp4", ".m4v", ".avi", ".mov", ".webm", ".ts", ".m2ts"]
        .iter()
        .any(|ext| lower.ends_with(ext))
}

/// Parse the central directory and return the largest media entry.
pub async fn probe(sizes: Vec<u64>, fetcher: Fetcher) -> Result<ZipEntry, String> {
    let v = VirtualFile::new(sizes, fetcher);
    if v.total < 22 {
        return Err("archive is too short".to_string());
    }
    let tail_len = TAIL_BYTES.min(v.total);
    let tail_start = v.total - tail_len;
    let tail = v.read(tail_start, tail_len as usize).await?;

    // EOCD must END the archive: `E + 22 + comment_len == tail.len()`.
    let mut eocd = None;
    for e in (0..=tail.len().saturating_sub(22)).rev() {
        if tail[e..e + 4] == [0x50, 0x4B, 0x05, 0x06] {
            let comment = u16le(&tail, e + 20).unwrap_or(0) as usize;
            if e + 22 + comment == tail.len() {
                eocd = Some(e);
                break;
            }
        }
    }
    let e = eocd.ok_or_else(|| "ZIP EOCD not found in the last 128 KiB".to_string())?;
    let eocd_global = tail_start + e as u64;

    let disk = u16le(&tail, e + 4).unwrap_or(0);
    let cd_disk = u16le(&tail, e + 6).unwrap_or(0);
    let entries = u16le(&tail, e + 10).unwrap_or(0);
    let mut cd_size = u32le(&tail, e + 12).unwrap_or(0) as u64;
    let mut cd_off = u32le(&tail, e + 16).unwrap_or(0) as u64;
    let mut zip64_eocd_off: Option<u64> = None;
    let entries32 = u32le(&tail, e + 8).unwrap_or(0);
    let cd_size32 = u32le(&tail, e + 12).unwrap_or(0);
    let cd_off32 = u32le(&tail, e + 16).unwrap_or(0);

    let zip64 = disk == 0xFFFF
        || cd_disk == 0xFFFF
        || entries == 0xFFFF
        || entries32 == 0xFFFF_FFFF
        || cd_size32 == 0xFFFF_FFFF
        || cd_off32 == 0xFFFF_FFFF;

    if zip64 {
        if e < 20 || tail[e - 20..e - 16] != [0x50, 0x4B, 0x06, 0x07] {
            return Err("ZIP64 EOCD locator missing".to_string());
        }
        let z_off = u64le(&tail, e - 20 + 8).ok_or_else(|| "bad ZIP64 locator".to_string())?;
        zip64_eocd_off = Some(z_off);
        let total_disks = u32le(&tail, e - 20 + 16).unwrap_or(0);
        if total_disks != 1 {
            return Err(format!("multi-disk ZIP ({} disks) is not supported", total_disks));
        }
        let z = v.read(z_off, 56).await?;
        if z[..4] != [0x50, 0x4B, 0x06, 0x06] {
            return Err("ZIP64 EOCD record missing".to_string());
        }
        if u32le(&z, 16).unwrap_or(1) != 0 || u32le(&z, 20).unwrap_or(1) != 0 {
            return Err("ZIP64 EOCD is not disk 0".to_string());
        }
        cd_size = u64le(&z, 40).ok_or_else(|| "bad ZIP64 cd size".to_string())?;
        cd_off = u64le(&z, 48).ok_or_else(|| "bad ZIP64 cd offset".to_string())?;
    }

    let cd_end = cd_off.checked_add(cd_size);
    let expected_cd_end = zip64_eocd_off.unwrap_or(eocd_global);
    if cd_end != Some(expected_cd_end) {
        return Err(format!(
            "ZIP central directory is not contiguous (cd {}+{} != {})",
            cd_off, cd_size, expected_cd_end
        ));
    }
    if cd_size == 0 || cd_size > MAX_CD {
        return Err(format!("central directory size {} is out of range", cd_size));
    }

    let cd = v.read(cd_off, cd_size as usize).await?;
    let mut pos = 0usize;
    let mut best: Option<ZipEntry> = None;
    while pos + 46 <= cd.len() {
        if cd[pos..pos + 4] != [0x50, 0x4B, 0x01, 0x02] {
            break;
        }
        let flags = u16le(&cd, pos + 8).unwrap_or(0);
        let method = u16le(&cd, pos + 10).unwrap_or(0);
        let mut comp = u32le(&cd, pos + 20).unwrap_or(0) as u64;
        let mut uncomp = u32le(&cd, pos + 24).unwrap_or(0) as u64;
        let name_len = u16le(&cd, pos + 28).unwrap_or(0) as usize;
        let extra_len = u16le(&cd, pos + 30).unwrap_or(0) as usize;
        let comment_len = u16le(&cd, pos + 32).unwrap_or(0) as usize;
        let mut lho = u32le(&cd, pos + 42).unwrap_or(0) as u64;
        let name_start = pos + 46;
        let extra_start = name_start + name_len;
        let total = 46 + name_len + extra_len + comment_len;
        if extra_start + extra_len > cd.len() || pos + total > cd.len() {
            return Err("central directory entry runs past the directory".to_string());
        }
        let name = String::from_utf8_lossy(&cd[name_start..extra_start]).to_string();

        // ZIP64 extra: variable length, sentinel-driven, fixed order
        // (uncomp, comp, local header offset, disk).
        let mut x = extra_start;
        while x + 4 <= extra_start + extra_len {
            let tag = u16le(&cd, x).unwrap_or(0);
            let sz = u16le(&cd, x + 2).unwrap_or(0) as usize;
            let body = x + 4;
            if body + sz > extra_start + extra_len {
                break;
            }
            if tag == 0x0001 {
                let mut p = body;
                if uncomp == 0xFFFF_FFFF && p + 8 <= body + sz {
                    uncomp = u64le(&cd, p).unwrap_or(uncomp);
                    p += 8;
                }
                if comp == 0xFFFF_FFFF && p + 8 <= body + sz {
                    comp = u64le(&cd, p).unwrap_or(comp);
                    p += 8;
                }
                if lho == 0xFFFF_FFFF && p + 8 <= body + sz {
                    lho = u64le(&cd, p).unwrap_or(lho);
                }
            }
            x = body + sz;
        }

        if flags & 1 == 0 && !name.ends_with('/') && is_media_name(&name) {
            let candidate = ZipEntry {
                name: name.clone(),
                method,
                comp_size: comp,
                uncomp_size: uncomp,
                data_start: 0,
                local_header_offset: lho,
            };
            if best
                .as_ref()
                .map(|b| candidate.uncomp_size > b.uncomp_size)
                .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
        pos += total;
    }

    let mut entry = best.ok_or_else(|| "ZIP contains no unencrypted media entry".to_string())?;
    if entry.uncomp_size == 0 {
        return Err("ZIP media entry is empty".to_string());
    }

    // Local header: its OWN name/extra lengths decide where data starts.
    let lho = entry.local_header_offset;
    let local = v.read(lho, 30).await?;
    if local[..4] != [0x50, 0x4B, 0x03, 0x04] {
        return Err("ZIP local header missing".to_string());
    }
    let local_name = u16le(&local, 26).unwrap_or(0) as u64;
    let local_extra = u16le(&local, 28).unwrap_or(0) as u64;
    entry.data_start = lho + 30 + local_name + local_extra;
    if entry
        .data_start
        .checked_add(entry.comp_size)
        .map(|end| end > cd_off)
        .unwrap_or(true)
    {
        return Err("ZIP entry data overlaps the central directory".to_string());
    }
    if entry.method == 0 && entry.comp_size != entry.uncomp_size {
        return Err("stored ZIP entry has mismatched sizes".to_string());
    }
    Ok(entry)
}

/// The entry layout is immutable for a given part set, but a player sends a new
/// Range for every seek. Probing the tail + CD each time cost 3-10 s of TTFB in
/// the live log; cache the result by `(short_code,size)` list.
pub async fn probe_cached(
    key: String,
    sizes: Vec<u64>,
    fetcher: Fetcher,
) -> Result<ZipEntry, String> {
    if let Some(entry) = zip_cache().lock().ok().and_then(|m| m.get(&key).cloned()) {
        return Ok(entry);
    }
    let entry = probe(sizes, fetcher).await?;
    if let Ok(mut m) = zip_cache().lock() {
        if m.len() >= 64 {
            m.clear();
        }
        m.insert(key, entry.clone());
    }
    Ok(entry)
}

/// A previously probed layout, if any. The route uses this to skip the part-head
/// checks on every seek: the archive cannot change under a fixed part set.
pub fn cached(key: &str) -> Option<ZipEntry> {
    zip_cache().lock().ok().and_then(|m| m.get(key).cloned())
}

fn zip_cache() -> &'static Mutex<HashMap<String, ZipEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, ZipEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;
    use std::sync::Arc;

    fn local(name: &str, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&[0x50, 0x4B, 0x03, 0x04]);
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
        out.extend_from_slice(&[0, 0, 0, 0]);
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(data);
        out
    }

    fn central(name: &str, data: &[u8], lho: u32) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&[0x50, 0x4B, 0x01, 0x02]);
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&20u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes()); // stored
        out.extend_from_slice(&[0, 0, 0, 0]);
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(data.len() as u32).to_le_bytes());
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&lho.to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out
    }

    fn zip_with(name: &str, data: &[u8]) -> Vec<u8> {
        let l = local(name, data);
        let c = central(name, data, 0);
        let mut out = l;
        let cd_off = out.len() as u32;
        out.extend_from_slice(&c);
        out.extend_from_slice(&[0x50, 0x4B, 0x05, 0x06]);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&(c.len() as u32).to_le_bytes());
        out.extend_from_slice(&cd_off.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out
    }

    fn local_fetcher(parts: Vec<Vec<u8>>) -> Fetcher {
        let parts = Arc::new(parts);
        Arc::new(move |i, pos, want| {
            let parts = parts.clone();
            async move {
                let part = &parts[i];
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
    async fn stored_entry_is_located_across_a_split() {
        let data = vec![0x1A, 0x45, 0xDF, 0xA3, 1, 2, 3, 4];
        let zip = zip_with("Movie.2026.mkv", &data);
        // Split at an awkward byte, not a block boundary.
        let cut = zip.len() / 2 + 3;
        let parts = vec![zip[..cut].to_vec(), zip[cut..].to_vec()];
        let sizes = parts.iter().map(|p| p.len() as u64).collect();
        let entry = probe(sizes, local_fetcher(parts)).await.unwrap();
        assert_eq!(entry.name, "Movie.2026.mkv");
        assert_eq!(entry.method, 0);
        assert_eq!(entry.uncomp_size, data.len() as u64);
        assert_eq!(entry.comp_size, data.len() as u64);
    }
}
