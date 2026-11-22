//! Bot API `file_id` decoder.
//!
//! Bot API `file_id` decoding — the TL layout Telegram uses for file references.
//! (`unpackFileId`) plus `vendor/danog/tg-file-decoder` (`base64urlDecode`,
//! `rleDecode`, TL string). grammers needs an
//! `InputDocumentFileLocation { id, access_hash, file_reference }` **and** the
//! `dc_id` that hosts the file, and a Bot API `file_id` is the only thing
//! WordPress `/resolve-file` returns, so this decoder is load-bearing.
//!
//! Layout (all little-endian), exactly like the reference implementations:
//! ```text
//!   4  type_id        (bit 25 = has file_reference, bit 24 = web location)
//!   4  dc_id
//!   ?  file_reference  TL string (1 byte len, or 254 + 3-byte len), 4-byte padded
//!   8  id
//!   8  access_hash
//!   .. photo-size source when type_id <= PHOTO (not needed for media streaming)
//!   1  version      (byte at end)
//!   1  sub_version  (byte before it, version 4 only)
//! ```
//! and the whole blob is null-byte RLE encoded *then* base64url encoded.

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};

pub const FILE_REFERENCE_FLAG: u32 = 1 << 25;
pub const WEB_LOCATION_FLAG: u32 = 1 << 24;

/// Document-backed types: the ones that use `inputDocumentFileLocation`.
const DOCUMENT_TYPES: [i32; 7] = [3, 4, 5, 8, 9, 10, 13];

pub const TYPE_NAMES: [(i32, &str); 18] = [
    (0, "thumbnail"),
    (1, "profile_photo"),
    (2, "photo"),
    (3, "voice"),
    (4, "video"),
    (5, "document"),
    (6, "encrypted"),
    (7, "temp"),
    (8, "sticker"),
    (9, "audio"),
    (10, "animation"),
    (11, "encrypted_thumbnail"),
    (12, "wallpaper"),
    (13, "video_note"),
    (14, "secure_raw"),
    (15, "secure"),
    (16, "background"),
    (17, "size"),
];

#[derive(Debug, Clone)]
pub struct DecodedFileId {
    pub id: i64,
    pub access_hash: i64,
    pub file_reference: Option<Vec<u8>>,
    pub dc_id: i32,
    pub file_type: i32,
    pub type_name: String,
    pub version: u8,
    pub sub_version: u8,
    pub web_url: Option<String>,
    /// Not carried by a Bot API file_id; the caller fills it from the payload.
    pub size: u64,
    pub raw: String,
}

/// MD5 hex digest — the PHP keys every cache file with `md5()`.
pub fn md5_hex(bytes: &[u8]) -> String {
    use md5::{Digest, Md5};
    let mut h = Md5::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

pub fn base64url_decode(data: &str) -> Vec<u8> {
    let mut s = data.replace('-', "+").replace('_', "/");
    let pad = s.len() % 4;
    if pad != 0 {
        s.push_str(&"=".repeat(4 - pad));
    }
    STANDARD.decode(s).unwrap_or_default()
}

pub fn base64url_encode(buf: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(buf)
}

/// Null-byte RLE decode, byte-for-byte the PHP loop (initial sentinel `''`).
pub fn rle_decode(input: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len() * 2);
    let mut last: i32 = -1;
    let mut last_was_null = false;
    for &cur in input {
        if last_was_null {
            for _ in 0..cur {
                out.push(0);
            }
            last_was_null = false;
            last = -1;
        } else {
            if last >= 0 {
                out.push(last as u8);
            }
            last = cur as i32;
            last_was_null = cur == 0;
        }
    }
    if last >= 0 {
        out.push(last as u8);
    }
    out
}

/// Null-byte RLE encode, the exact inverse of [`rle_decode`].
///
/// The TL blob is RLE-encoded *before* base64url, so writing the TL bytes
/// straight into the string produces an id Telegram rejects. A zero run becomes
/// `0x00` + run length; the 255 cap makes a longer run several pairs, which
/// `rle_decode` reads back identically because it consumes one pair per run.
pub fn rle_encode(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        if data[i] == 0 {
            let mut n = 0usize;
            while i < data.len() && data[i] == 0 && n < 255 {
                n += 1;
                i += 1;
            }
            out.push(0);
            out.push(n as u8);
        } else {
            out.push(data[i]);
            i += 1;
        }
    }
    out
}

struct Reader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn bytes(&mut self, n: usize) -> &'a [u8] {
        let start = self.pos.min(self.buf.len());
        let end = (self.pos + n).min(self.buf.len());
        self.pos += n;
        &self.buf[start..end]
    }
    fn i32(&mut self) -> i32 {
        let b = self.bytes(4);
        if b.len() == 4 {
            i32::from_le_bytes([b[0], b[1], b[2], b[3]])
        } else {
            0
        }
    }
    fn i64(&mut self) -> i64 {
        let b = self.bytes(8);
        if b.len() == 8 {
            i64::from_le_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
        } else {
            0
        }
    }
    /// TL string: 1-byte len (254 => 3-byte LE len), then the body, then the
    /// pad that makes `length-prefix + body` a multiple of 4.
    ///
    /// THE PAD COUNTS THE LENGTH PREFIX. Using `len % 4` instead of
    /// `(prefix + len) % 4` leaves the reader ONE byte past the string, so every
    /// following field (the document `id`, then `access_hash`) is read shifted.
    /// The `file_reference` itself is unaffected, which is why Telegram then
    /// answers `FILE_REFERENCE_EXPIRED` (the reference matches but the id it
    /// belongs to does not) instead of reporting a malformed request.
    fn tl_string(&mut self) -> Vec<u8> {
        let start = self.pos;
        let mut len = self.bytes(1).first().copied().unwrap_or(0) as usize;
        if len == 254 {
            let b = self.bytes(3);
            len = b.first().copied().unwrap_or(0) as usize
                | ((b.get(1).copied().unwrap_or(0) as usize) << 8)
                | ((b.get(2).copied().unwrap_or(0) as usize) << 16);
        }
        let prefix = self.pos - start;
        let body = self.bytes(len).to_vec();
        let pad = (4 - ((prefix + len) % 4)) % 4;
        if pad > 0 {
            let _ = self.bytes(pad);
        }
        body
    }
}

pub fn decode_file_id(file_id_mt: &str) -> Result<DecodedFileId, String> {
    let raw = file_id_mt.trim().to_string();
    if raw.is_empty() {
        return Err("empty file_id".into());
    }
    let decoded = rle_decode(&base64url_decode(&raw));
    if decoded.len() < 17 {
        return Err("file_id too short".into());
    }
    let version = *decoded.last().unwrap();
    let sub_version = if version == 4 {
        decoded[decoded.len() - 2]
    } else {
        0
    };
    if version != 2 && version != 4 {
        return Err(format!("Invalid bot API file ID version {}", version));
    }

    let mut r = Reader::new(&decoded);
    let mut type_raw = r.i32() as u32;
    let dc_id = r.i32();
    let has_reference = type_raw & FILE_REFERENCE_FLAG != 0;
    let file_reference = if has_reference {
        Some(r.tl_string())
    } else {
        None
    };
    let has_web = type_raw & WEB_LOCATION_FLAG != 0;
    type_raw &= !FILE_REFERENCE_FLAG;
    type_raw &= !WEB_LOCATION_FLAG;
    let file_type = type_raw as i32;
    let type_name = TYPE_NAMES
        .iter()
        .find(|(id, _)| *id == file_type)
        .map(|(_, n)| n.to_string())
        .unwrap_or_else(|| format!("unknown:{}", file_type));

    if has_web {
        let web_url = String::from_utf8_lossy(&r.tl_string()).to_string();
        let access_hash = r.i64();
        return Ok(DecodedFileId {
            id: 0,
            access_hash,
            file_reference,
            dc_id,
            file_type,
            type_name,
            version,
            sub_version,
            web_url: Some(web_url),
            size: 0,
            raw,
        });
    }

    let id = r.i64();
    let access_hash = r.i64();
    Ok(DecodedFileId {
        id,
        access_hash,
        file_reference,
        dc_id,
        file_type,
        type_name,
        version,
        sub_version,
        web_url: None,
        size: 0,
        raw,
    })
}

/// TL string writer, the inverse of [`Reader::tl_string`].
fn write_tl_string(out: &mut Vec<u8>, body: &[u8]) {
    let len = body.len();
    let prefix = if len < 254 {
        out.push(len as u8);
        1
    } else {
        out.push(254);
        out.push(len as u8);
        out.push((len >> 8) as u8);
        out.push((len >> 16) as u8);
        4
    };
    out.extend_from_slice(body);
    // The pad counts the length PREFIX, the same rule the reader uses; `len % 4`
    // here re-creates the off-by-one that shifted `id`/`access_hash` and made
    // every download answer FILE_REFERENCE_EXPIRED.
    let pad = (4 - ((prefix + len) % 4)) % 4;
    out.resize(out.len() + pad, 0);
}

/// The exact inverse of [`decode_file_id`]: TL layout, null-byte RLE, base64url.
///
/// `size` is not on the wire — the caller fills it from the payload — so it is
/// the only decoded field that cannot round-trip. A web-location id has no `id`
/// field (its body stops at `access_hash`), which the decoder reports through
/// `web_url` and this function mirrors. The document layout round-trips exactly
/// (pinned against the live fixture); a PHOTO id also carries its size source
/// between `access_hash` and the version bytes, which the decoder skips, so
/// encoding one would not reproduce the input — `--mint-probe` rejects those.
///
/// `version`/`sub_version` come from the caller because a document received from
/// `messages.sendMedia` carries no encoded version of its own: Bot API ids from
/// this account are version 4 / sub 0, which is what `--mint-probe` uses.
pub fn encode_file_id(d: &DecodedFileId) -> String {
    let mut buf = Vec::with_capacity(64);
    let mut type_raw = d.file_type as u32;
    if d.file_reference.is_some() {
        type_raw |= FILE_REFERENCE_FLAG;
    }
    if d.web_url.is_some() {
        type_raw |= WEB_LOCATION_FLAG;
    }
    buf.extend_from_slice(&type_raw.to_le_bytes());
    buf.extend_from_slice(&d.dc_id.to_le_bytes());
    if let Some(reference) = d.file_reference.as_deref() {
        write_tl_string(&mut buf, reference);
    }
    match d.web_url.as_deref() {
        Some(url) => {
            write_tl_string(&mut buf, url.as_bytes());
            buf.extend_from_slice(&d.access_hash.to_le_bytes());
        }
        None => {
            buf.extend_from_slice(&d.id.to_le_bytes());
            buf.extend_from_slice(&d.access_hash.to_le_bytes());
        }
    }
    if d.version == 4 {
        buf.push(d.sub_version);
    }
    buf.push(d.version);
    base64url_encode(&rle_encode(&buf))
}

pub fn is_document_file_id(d: &DecodedFileId) -> bool {
    DOCUMENT_TYPES.contains(&d.file_type)
}

/// `document` | `photo` | `unsupported`
pub fn location_kind(d: &DecodedFileId) -> &'static str {
    if DOCUMENT_TYPES.contains(&d.file_type) {
        return "document";
    }
    if d.file_type == 0 || d.file_type == 2 {
        return "photo";
    }
    "unsupported"
}

impl DecodedFileId {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id.to_string(),
            "access_hash": self.access_hash.to_string(),
            "file_reference_hex": self.file_reference.as_ref().map(hex::encode),
            "file_reference_len": self.file_reference.as_ref().map(|r| r.len()).unwrap_or(0),
            "dc_id": self.dc_id,
            "type": self.file_type,
            "type_name": self.type_name,
            "version": self.version,
            "sub_version": self.sub_version,
            "web_url": self.web_url,
            "size": self.size,
            "location_kind": location_kind(self),
        })
    }
}

/// Self-check used by `--decode` / `--selftest` and by the side-by-side run
/// against the TypeScript decoder.
pub fn verify(file_id_mt: &str, expected_id: Option<&str>) -> (DecodedFileId, Vec<(String, bool, String)>) {
    let d = decode_file_id(file_id_mt).expect("decode");
    let mut checks: Vec<(String, bool, String)> = Vec::new();
    if let Some(exp) = expected_id {
        let ok = exp == d.id.to_string() || exp == (d.id as u64).to_string();
        checks.push(("id matches expected".into(), ok, d.id.to_string()));
    }
    checks.push((
        "dc_id in 1..5".into(),
        (1..=5).contains(&d.dc_id),
        d.dc_id.to_string(),
    ));
    checks.push((
        "has file_reference".into(),
        d.file_reference.as_ref().map(|r| !r.is_empty()).unwrap_or(false),
        d.file_reference.as_ref().map(|r| r.len()).unwrap_or(0).to_string(),
    ));
    checks.push((
        "document-backed type".into(),
        is_document_file_id(&d),
        d.type_name.clone(),
    ));
    (d, checks)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Live `file_id_mt` for short_code `1v9OPj` (Avengers Infinity War 2160p),
    /// resolved through the PHP runtime's WordPress `/resolve-file`.
    const LIVE_ID: &str = "BQACAgUAAxkBAAII3mrWOzVpKE4Q6skql_D_lL_9MWTpAAJzBQACk8FIVwzlNXvo-CvsPQQ";

    #[test]
    fn decodes_live_document_id() {
        let d = decode_file_id(LIVE_ID).unwrap();
        assert_eq!(d.version, 4);
        assert_eq!(d.dc_id, 5, "Telegram DC 5 hosts this file");
        assert_eq!(d.file_type, 5, "document");
        assert_eq!(d.file_reference.as_ref().unwrap().len(), 25);
        // Telegram document ids are positive; a negative value is the signature
        // of the off-by-one padding bug. (An access_hash is a random i64 and may
        // legitimately be negative, so it is not asserted.)
        assert!(d.id > 0, "document id must be positive, got {}", d.id);
        assert!(is_document_file_id(&d));
        assert_eq!(location_kind(&d), "document");
    }

    /// Regression guard for the TL padding rule: with the wrong pad the id came
    /// out negative and every download failed with FILE_REFERENCE_EXPIRED.
    #[test]
    fn tl_padding_counts_the_length_prefix() {
        // 25-byte payload: prefix 1 + 25 = 26 -> pad 2 (not 3).
        assert_eq!((4 - ((1 + 25) % 4)) % 4, 2);

        let mut buf = Vec::new();
        // type_raw = document (5) with FILE_REFERENCE_FLAG (1 << 25) set.
        buf.extend_from_slice(&0x0200_0005u32.to_le_bytes());
        buf.extend_from_slice(&5i32.to_le_bytes()); // dc_id
        buf.push(25); // TL length
        buf.extend_from_slice(&[7u8; 25]); // payload
        buf.extend_from_slice(&[0, 0]); // pad = 2
        buf.extend_from_slice(&42i64.to_le_bytes()); // id
        buf.extend_from_slice(&43i64.to_le_bytes()); // access_hash
        // version 4 ids end with `<sub_version><version>`; the version byte is last.
        buf.push(0); // sub_version
        buf.push(4); // version

        let d = decode_file_id(&base64url_encode(&rle_encode(&buf))).unwrap();
        assert_eq!(d.id, 42);
        assert_eq!(d.access_hash, 43);
        assert_eq!(d.file_reference.unwrap().len(), 25);
    }

    /// `encode_file_id` must reproduce the exact Bot API string the decoder read.
    /// Comparing fields alone would still accept a wrong RLE or a wrong base64
    /// alphabet, and Telegram refuses both with BAD_REQUEST.
    #[test]
    fn encode_is_the_inverse_of_decode() {
        let d = decode_file_id(LIVE_ID).unwrap();
        assert_eq!(encode_file_id(&d), LIVE_ID);

        // ...and the parts survive a second pass, which is what `--mint-probe`
        // does with a document object that never had an encoded form.
        let again = decode_file_id(&encode_file_id(&d)).unwrap();
        assert_eq!(again.id, d.id);
        assert_eq!(again.access_hash, d.access_hash);
        assert_eq!(again.file_reference, d.file_reference);
        assert_eq!(again.dc_id, d.dc_id);
        assert_eq!(again.file_type, d.file_type);
        assert_eq!(again.version, d.version);
        assert_eq!(again.sub_version, d.sub_version);
    }

    #[test]
    fn rle_roundtrip_of_zeros() {
        // 0x00 0x03 encodes three zero bytes in the PHP loop.
        assert_eq!(rle_decode(&[0, 3]), vec![0, 0, 0]);
        // A plain non-zero run is returned verbatim.
        assert_eq!(rle_decode(&[1, 2, 3]), vec![1, 2, 3]);
    }

    #[test]
    fn rejects_garbage() {
        assert!(decode_file_id("").is_err());
        assert!(decode_file_id("AAAA").is_err());
    }
}
