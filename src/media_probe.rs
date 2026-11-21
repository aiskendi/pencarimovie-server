//! Container tracks from a bounded prefix.
//!
//! `/stream` derives its audio and subtitle metadata from the filename today.
//! The container states it outright, and the bytes the caller already fetches
//! (one range request for the first N bytes) hold Matroska's `Tracks` element or
//! the whole MP4 `moov` box.
//!
//! This is the synchronous counterpart of `split_remux`'s EBML reader: that one
//! walks an async `Reader`, this one walks bytes already in memory. An unknown
//! length, a box that runs past the buffer, or a prefix that stops before the
//! metadata is `None` — never a partial guess, never a panic.
//!
//! Labels match `matcher::MediaTags` and the eight names in
//! `routes::language_flags` (without the flag glyph), so container facts and
//! filename facts can be mixed without translating between them.

/// Tracks found in the container's prefix.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContainerTracks {
    /// Normalised audio codec labels, de-duplicated, in container order:
    /// "AAC", "AC3", "DDP", "DTS", "DTS-HD", "TrueHD", "FLAC", "Opus",
    /// "Vorbis", "MP3".
    pub audio: Vec<String>,
    /// Channel count per entry in `audio`, as the container writes it (6 for
    /// 5.1, 2 for stereo). `0` when the track states none, so both vectors
    /// always have the same length.
    pub channels: Vec<u8>,
    /// Normalised language labels on audio tracks, de-duplicated: "English",
    /// "Malay", "Indo", "Hindi", "Korean", "Japanese", "Chinese", "Thai".
    pub audio_languages: Vec<String>,
    /// The same labels for subtitle tracks.
    pub subtitle_languages: Vec<String>,
}

const ID_EBML: u32 = 0x1A45_DFA3;
const ID_SEGMENT: u32 = 0x1853_8067;
const ID_TRACKS: u32 = 0x1654_AE6B;
const ID_CLUSTER: u32 = 0x1F43_B675;
const ID_TRACK_ENTRY: u32 = 0xAE;
const ID_TRACK_TYPE: u32 = 0x83;
const ID_CODEC_ID: u32 = 0x86;
const ID_LANGUAGE: u32 = 0x22B5_9C;
const ID_LANGUAGE_IETF: u32 = 0x22B5_9D;
const ID_AUDIO: u32 = 0xE1;
const ID_CHANNELS: u32 = 0x9F;

const TRACK_AUDIO: u64 = 2;
const TRACK_SUBTITLE: u64 = 0x11;

/// Caps on every scan. A prefix is a few MB at most and no real file has more
/// tracks, nesting or top-level elements than this.
const MAX_TRACKS: usize = 64;
const MAX_DEPTH: usize = 4;
const MAX_ELEMENTS: usize = 256;

const ISO_TOP_LEVEL: [[u8; 4]; 8] = [
    *b"ftyp", *b"styp", *b"moov", *b"moof", *b"mdat", *b"free", *b"skip", *b"wide",
];

/// Parse a bounded prefix of the file.
///
/// `None` means "could not determine": not a container we understand, a prefix
/// truncated before the metadata, or malformed bytes. An empty track list is a
/// real answer, so an unreadable prefix is never reported as one.
pub fn probe(prefix: &[u8]) -> Option<ContainerTracks> {
    if prefix.starts_with(&ID_EBML.to_be_bytes()) {
        return probe_mkv(prefix);
    }
    if is_iso_bmff(prefix) {
        return probe_mp4(prefix);
    }
    None
}

fn is_iso_bmff(prefix: &[u8]) -> bool {
    let Some(kind) = prefix.get(4..8).and_then(|s| <[u8; 4]>::try_from(s).ok()) else {
        return false;
    };
    ISO_TOP_LEVEL.contains(&kind)
}

/* ── EBML primitives ────────────────────────────────────────────────────── */

/// One element header: `body` is the payload offset and `size` its length, or
/// `None` for an unknown-length element.
fn read_elem(buf: &[u8], pos: usize) -> Option<(u32, usize, Option<usize>)> {
    let (id, id_len) = ebml_id(buf, pos)?;
    let (size, size_len) = ebml_size(buf, pos.checked_add(id_len)?)?;
    let body = pos.checked_add(id_len)?.checked_add(size_len)?;
    if body > buf.len() {
        return None;
    }
    let size = match size {
        Some(n) => Some(usize::try_from(n).ok()?),
        None => None,
    };
    Some((id, body, size))
}

fn ebml_id(buf: &[u8], pos: usize) -> Option<(u32, usize)> {
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
    let bytes = buf.get(pos..pos.checked_add(len)?)?;
    let id = bytes.iter().fold(0u32, |a, b| (a << 8) | *b as u32);
    Some((id, len))
}

/// `(value, width)`, with an all-value-bits-set size reported as `None`. An
/// element whose length is unknown cannot be skipped without guessing, so every
/// caller treats that as fatal.
fn ebml_size(buf: &[u8], pos: usize) -> Option<(Option<u64>, usize)> {
    let b0 = *buf.get(pos)?;
    let len = b0.leading_zeros() as usize + 1;
    if len > 8 {
        return None;
    }
    let bytes = buf.get(pos..pos.checked_add(len)?)?;
    let mut value = (b0 as u64) & ((1u64 << (8 - len)) - 1);
    for b in bytes.iter().skip(1) {
        value = (value << 8) | *b as u64;
    }
    if value == (1u64 << (7 * len)) - 1 {
        return Some((None, len));
    }
    Some((Some(value), len))
}

fn be_uint(value: &[u8]) -> u64 {
    value.iter().fold(0u64, |a, b| (a << 8) | *b as u64)
}

/// A Matroska string element, with NUL padding trimmed. Empty is `None`, since
/// an absent language and an empty one mean the same thing.
fn text(value: &[u8]) -> Option<&str> {
    let text = std::str::from_utf8(value).ok()?;
    let text = text.trim_matches(|c: char| c == '\0' || c.is_whitespace());
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Walk a region's children as `(id, body)`, stopping at the first malformed or
/// unknown-length element: from there on nothing is addressable.
fn elems<'a>(region: &'a [u8]) -> impl Iterator<Item = (u32, &'a [u8])> + 'a {
    let mut pos = 0usize;
    std::iter::from_fn(move || {
        let (id, body, size) = read_elem(region, pos)?;
        let end = body.checked_add(size?)?;
        pos = end;
        region.get(body..end).map(|value| (id, value))
    })
}

/* ── Matroska / WebM ────────────────────────────────────────────────────── */

fn probe_mkv(prefix: &[u8]) -> Option<ContainerTracks> {
    let (id, body, size) = read_elem(prefix, 0)?;
    if id != ID_EBML {
        return None;
    }
    let end = body.checked_add(size?)?;
    if end > prefix.len() {
        return None;
    }

    let (id, body, size) = read_elem(prefix, end)?;
    if id != ID_SEGMENT {
        return None;
    }
    // An unknown-size Segment is the normal shape for a streamed file, and a
    // known one may still be only partly in the prefix. `Tracks` is near the
    // front either way, so walk up to whichever bound the bytes give us.
    let seg_end = match size {
        Some(n) => body.checked_add(n)?.min(prefix.len()),
        None => prefix.len(),
    };

    let mut pos = body;
    for _ in 0..MAX_ELEMENTS {
        if pos >= seg_end {
            return None;
        }
        let (id, body, size) = read_elem(prefix, pos)?;
        if id == ID_CLUSTER {
            // `Tracks` is written before the first cluster; a cluster here means
            // there is no track list left to find.
            return None;
        }
        let size = size?;
        let end = body.checked_add(size)?;
        if end > seg_end {
            return None;
        }
        if id == ID_TRACKS {
            return prefix.get(body..end).map(parse_tracks);
        }
        pos = end;
    }
    None
}

fn parse_tracks(tracks: &[u8]) -> ContainerTracks {
    let mut out = ContainerTracks::default();
    let mut entries = 0usize;
    for (id, child) in elems(tracks).take(MAX_ELEMENTS) {
        if id != ID_TRACK_ENTRY {
            continue;
        }
        if entries >= MAX_TRACKS {
            break;
        }
        entries += 1;
        read_track_entry(child, 1, &mut out);
    }
    out
}

fn read_track_entry(entry: &[u8], depth: usize, out: &mut ContainerTracks) {
    if depth > MAX_DEPTH {
        return;
    }
    let mut kind = 0u64;
    let mut codec: Option<&str> = None;
    let mut ietf: Option<&str> = None;
    let mut legacy: Option<&str> = None;
    let mut channels = 0u8;

    for (id, value) in elems(entry).take(MAX_ELEMENTS) {
        match id {
            ID_TRACK_TYPE => kind = be_uint(value),
            ID_CODEC_ID => codec = text(value),
            ID_LANGUAGE => legacy = text(value),
            ID_LANGUAGE_IETF => ietf = text(value),
            ID_AUDIO => channels = audio_channels(value, depth + 1),
            _ => {}
        }
    }

    if kind != TRACK_AUDIO && kind != TRACK_SUBTITLE {
        return;
    }
    // `LanguageIETF` overrides `Language`; a tag we cannot name (including
    // `und`) falls through to the legacy field rather than losing the track.
    let language = ietf
        .and_then(language_label)
        .or_else(|| legacy.and_then(language_label));
    if kind == TRACK_AUDIO {
        if let Some(label) = codec.and_then(audio_label) {
            push_audio(out, label, channels);
        }
        if let Some(label) = language {
            push_unique(&mut out.audio_languages, label);
        }
    } else if let Some(label) = language {
        push_unique(&mut out.subtitle_languages, label);
    }
}

fn audio_channels(audio: &[u8], depth: usize) -> u8 {
    if depth > MAX_DEPTH {
        return 0;
    }
    for (id, value) in elems(audio).take(MAX_ELEMENTS) {
        if id == ID_CHANNELS {
            return u8::try_from(be_uint(value)).unwrap_or(0);
        }
    }
    0
}

/* ── ISO-BMFF ───────────────────────────────────────────────────────────── */

/// A box inside a region: `body..end` is its payload and `end` its declared end.
struct IsoBox {
    kind: [u8; 4],
    body: usize,
    end: usize,
}

fn probe_mp4(prefix: &[u8]) -> Option<ContainerTracks> {
    let mut pos = 0usize;
    for _ in 0..MAX_ELEMENTS {
        if pos >= prefix.len() {
            return None;
        }
        let b = read_box(prefix, pos)?;
        if b.end > prefix.len() {
            // `mdat` is not fully in the prefix, so `moov` behind it is out of
            // reach: moov-at-end needs a second range request.
            return None;
        }
        if b.kind == *b"moov" {
            return prefix.get(b.body..b.end).and_then(parse_moov);
        }
        pos = b.end;
    }
    None
}

fn parse_moov(moov: &[u8]) -> Option<ContainerTracks> {
    let mut out = ContainerTracks::default();
    for b in children(moov)? {
        if b.kind != *b"trak" {
            continue;
        }
        read_trak(moov.get(b.body..b.end)?, &mut out);
    }
    Some(out)
}

fn read_trak(trak: &[u8], out: &mut ContainerTracks) {
    let Some(mdia) = child_body(trak, b"mdia") else {
        return;
    };
    let Some(hdlr) = child_body(mdia, b"hdlr") else {
        return;
    };
    // `hdlr`: version+flags, `pre_defined`, then the handler type fourcc.
    let Some(handler) = hdlr.get(8..12).and_then(|s| <[u8; 4]>::try_from(s).ok()) else {
        return;
    };
    let language = mdhd_language(mdia);
    if handler == *b"soun" {
        if let Some((fourcc, channels)) = sample_entry(mdia) {
            if let Some(label) = std::str::from_utf8(&fourcc).ok().and_then(audio_label) {
                push_audio(out, label, channels);
            }
        }
        if let Some(label) = language {
            push_unique(&mut out.audio_languages, label);
        }
    } else if handler == *b"subt" {
        if let Some(label) = language {
            push_unique(&mut out.subtitle_languages, label);
        }
    }
}

/// First sample entry of the track: its fourcc and, for audio, the channel
/// count (0 when the entry is too short to state one).
fn sample_entry(mdia: &[u8]) -> Option<([u8; 4], u8)> {
    let minf = child_body(mdia, b"minf")?;
    let stbl = child_body(minf, b"stbl")?;
    let stsd = child_body(stbl, b"stsd")?;
    // `stsd`: version+flags then entry_count, so the first entry starts at 8.
    let entry = read_box(stsd, 8)?;
    // `AudioSampleEntry`: 6 reserved, data_reference_index, version, revision and
    // vendor before `channelcount` at +16 of the entry body.
    let channels = be_u16(stsd, entry.body.checked_add(16)?)
        .and_then(|n| u8::try_from(n).ok())
        .unwrap_or(0);
    Some((entry.kind, channels))
}

fn mdhd_language(mdia: &[u8]) -> Option<&'static str> {
    let mdhd = child_body(mdia, b"mdhd")?;
    let version = *mdhd.first()?;
    // Creation/modification time widen to 64 bits in version 1, which moves the
    // language field from offset 20 to 32.
    let at = if version == 1 { 32 } else { 20 };
    let packed = be_u16(mdhd, at)? & 0x7FFF;
    if packed == 0 {
        return None;
    }
    // Three 5-bit letters biased by 0x60, the first letter in the high bits.
    let mut tag = [0u8; 3];
    for (i, slot) in tag.iter_mut().enumerate() {
        let shift = 10 - 5 * i as u32;
        *slot = (((packed >> shift) & 0x1F) as u8).wrapping_add(0x60);
    }
    language_label(std::str::from_utf8(&tag).ok()?)
}

fn child_body<'a>(region: &'a [u8], kind: &[u8; 4]) -> Option<&'a [u8]> {
    children(region)?
        .into_iter()
        .find(|b| &b.kind == kind)
        .and_then(|b| region.get(b.body..b.end))
}

fn children(region: &[u8]) -> Option<Vec<IsoBox>> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < region.len() {
        if out.len() >= MAX_ELEMENTS {
            return None;
        }
        let b = read_box(region, pos)?;
        if b.end > region.len() {
            return None;
        }
        pos = b.end;
        out.push(b);
    }
    Some(out)
}

fn read_box(buf: &[u8], pos: usize) -> Option<IsoBox> {
    let declared = be_u32(buf, pos)?;
    let kind = buf.get(pos.checked_add(4)?..pos.checked_add(8)?)?;
    let (header, size) = match declared {
        1 => (
            16usize,
            usize::try_from(be_u64(buf, pos.checked_add(8)?)?).ok()?,
        ),
        // "To end of file" is indistinguishable from a truncated box inside a
        // bounded prefix, and taking it literally would leave `pos` unchanged.
        0 => return None,
        n => (8usize, n as usize),
    };
    if size < header {
        return None;
    }
    Some(IsoBox {
        kind: <[u8; 4]>::try_from(kind).ok()?,
        body: pos.checked_add(header)?,
        end: pos.checked_add(size)?,
    })
}

fn be_u16(buf: &[u8], pos: usize) -> Option<u16> {
    let bytes = buf.get(pos..pos.checked_add(2)?)?;
    Some(u16::from_be_bytes(bytes.try_into().ok()?))
}

fn be_u32(buf: &[u8], pos: usize) -> Option<u32> {
    let bytes = buf.get(pos..pos.checked_add(4)?)?;
    Some(u32::from_be_bytes(bytes.try_into().ok()?))
}

fn be_u64(buf: &[u8], pos: usize) -> Option<u64> {
    let bytes = buf.get(pos..pos.checked_add(8)?)?;
    Some(u64::from_be_bytes(bytes.try_into().ok()?))
}

/* ── Normalisation ──────────────────────────────────────────────────────── */

/// Audio label for a Matroska `CodecID` or an ISO-BMFF sample entry fourcc.
/// Anything else is audio we cannot name, so it gets no label at all.
fn audio_label(codec: &str) -> Option<&'static str> {
    Some(match codec {
        "A_AAC" | "mp4a" => "AAC",
        "A_AC3" | "ac-3" => "AC3",
        "A_EAC3" | "ec-3" => "DDP",
        // `dtsh` is DTS-HD in the container, but the filename vocabulary this
        // feeds has one DTS label for the family.
        "A_DTS" | "dtsc" | "dtsh" | "dtsl" => "DTS",
        "A_TRUEHD" | "mlpa" => "TrueHD",
        "A_FLAC" | "fLaC" => "FLAC",
        "A_OPUS" | "Opus" => "Opus",
        "A_VORBIS" => "Vorbis",
        "A_MPEG/L3" | ".mp3" => "MP3",
        _ => return None,
    })
}

/// Language label for a container tag: an ISO-639 code (`en`, `eng`, `may`) or
/// the primary subtag of an IETF tag such as `en-MY` or `zh-Hans`.
///
/// `und` ("undetermined"), an empty tag and every language outside our eight get
/// no label: naming one would put a flag on a track nobody labelled.
fn language_label(tag: &str) -> Option<&'static str> {
    let primary = tag.split(|c: char| c == '-' || c == '_').next()?;
    let primary = primary.to_ascii_lowercase();
    Some(match primary.as_str() {
        "en" | "eng" => "English",
        "ms" | "may" | "msa" => "Malay",
        "id" | "ind" => "Indo",
        "hi" | "hin" => "Hindi",
        "ko" | "kor" => "Korean",
        "ja" | "jpn" => "Japanese",
        "zh" | "chi" | "zho" | "cmn" => "Chinese",
        "th" | "tha" => "Thai",
        _ => return None,
    })
}

fn push_unique(list: &mut Vec<String>, label: &str) {
    if !list.iter().any(|seen| seen == label) {
        list.push(label.to_string());
    }
}

fn push_audio(out: &mut ContainerTracks, label: &str, channels: u8) {
    if out.audio.len() >= MAX_TRACKS || out.audio.iter().any(|seen| seen == label) {
        return;
    }
    out.audio.push(label.to_string());
    out.channels.push(channels);
}

#[cfg(test)]
mod tests {
    use super::*;

    const LANG: &[u8] = &[0x22, 0xB5, 0x9C];
    const LANG_IETF: &[u8] = &[0x22, 0xB5, 0x9D];

    /* ── EBML fixtures ─────────────────────────────────────────────────── */

    fn vint(value: u64) -> Vec<u8> {
        let len = (1..=8usize)
            .find(|&n| value < (1u64 << (7 * n)) - 1)
            .expect("a size that fits an EBML vint");
        let mut out = value.to_be_bytes()[8 - len..].to_vec();
        out[0] |= 1 << (8 - len);
        out
    }

    fn el(id: &[u8], body: &[u8]) -> Vec<u8> {
        let mut out = id.to_vec();
        out.extend(vint(body.len() as u64));
        out.extend_from_slice(body);
        out
    }

    fn uint(id: &[u8], value: u8) -> Vec<u8> {
        el(id, &[value])
    }

    fn text_el(id: &[u8], value: &str) -> Vec<u8> {
        el(id, value.as_bytes())
    }

    /// A Matroska document with an unknown-size Segment, like a streamed file.
    fn mkv(doctype: &str, segment: &[u8]) -> Vec<u8> {
        let header = el(
            &[0x1A, 0x45, 0xDF, 0xA3],
            &[uint(&[0x42, 0x86], 1), text_el(&[0x42, 0x82], doctype)].concat(),
        );
        let mut out = header;
        out.extend_from_slice(&[
            0x18, 0x53, 0x80, 0x67, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        ]);
        out.extend_from_slice(segment);
        out
    }

    fn tracks(entries: &[Vec<u8>]) -> Vec<u8> {
        el(&[0x16, 0x54, 0xAE, 0x6B], &entries.concat())
    }

    /// One `TrackEntry`, with `language` as `(element id, tag)` pairs.
    fn track(kind: u8, codec: &str, language: &[(&[u8], &str)], channels: Option<u8>) -> Vec<u8> {
        let mut body = uint(&[0x83], kind);
        body.extend(text_el(&[0x86], codec));
        for (id, tag) in language {
            body.extend(text_el(id, tag));
        }
        if let Some(n) = channels {
            body.extend(el(&[0xE1], &uint(&[0x9F], n)));
        }
        el(&[0xAE], &body)
    }

    /* ── ISO-BMFF fixtures ─────────────────────────────────────────────── */

    fn iso(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut out = ((body.len() + 8) as u32).to_be_bytes().to_vec();
        out.extend_from_slice(kind);
        out.extend_from_slice(body);
        out
    }

    fn stsd(entry: &[u8]) -> Vec<u8> {
        let mut body = vec![0u8; 4]; // version + flags
        body.extend_from_slice(&1u32.to_be_bytes()); // entry_count
        body.extend_from_slice(entry);
        iso(b"stsd", &body)
    }

    fn audio_entry(fourcc: &[u8; 4], channels: u16) -> Vec<u8> {
        // 6 reserved, data_reference_index, version, revision, vendor, then
        // `channelcount` at +16 of the entry body.
        let mut body = vec![0u8; 16];
        body.extend_from_slice(&channels.to_be_bytes());
        body.extend_from_slice(&16u16.to_be_bytes()); // samplesize
        body.extend_from_slice(&[0u8; 4]); // pre_defined + reserved
        body.extend_from_slice(&(48_000u32 << 16).to_be_bytes()); // samplerate
        iso(fourcc, &body)
    }

    fn mdhd(language: u16) -> Vec<u8> {
        // Version 0: version+flags, creation, modification, timescale, duration,
        // then the 15-bit packed ISO-639-2/T code at offset 20. Real muxers set
        // the top bit of that field, so the fixture does too.
        let mut body = vec![0u8; 20];
        body.extend_from_slice(&(language | 0x8000).to_be_bytes());
        body.extend_from_slice(&[0u8; 2]);
        iso(b"mdhd", &body)
    }

    fn hdlr(handler: &[u8; 4]) -> Vec<u8> {
        let mut body = vec![0u8; 8]; // version+flags, pre_defined
        body.extend_from_slice(handler); // handler_type
        body.extend_from_slice(&[0u8; 13]); // 3 reserved words + empty name
        iso(b"hdlr", &body)
    }

    fn packed(tag: &[u8; 3]) -> u16 {
        tag.iter()
            .fold(0u16, |acc, &b| (acc << 5) | (b as u16 - 0x60))
    }

    fn trak(handler: &[u8; 4], language: u16, entry: &[u8]) -> Vec<u8> {
        let minf = iso(b"minf", &iso(b"stbl", &stsd(entry)));
        let mdia = iso(b"mdia", &[mdhd(language), hdlr(handler), minf].concat());
        iso(b"trak", &mdia)
    }

    fn ftyp() -> Vec<u8> {
        iso(b"ftyp", b"isom\x00\x00\x02\x00isommp41")
    }

    /* ── tests ─────────────────────────────────────────────────────────── */

    #[test]
    fn mkv_audio_and_subtitle_tracks() {
        let doc = mkv(
            "matroska",
            &tracks(&[
                track(2, "A_AAC", &[(LANG, "eng")], Some(2)),
                track(2, "A_EAC3", &[(LANG_IETF, "ms-MY")], Some(6)),
                track(0x11, "S_TEXT/UTF8", &[(LANG, "ind")], None),
            ]),
        );

        let got = probe(&doc).expect("a track list");
        assert_eq!(got.audio, ["AAC", "DDP"]);
        assert_eq!(got.channels, [2, 6]);
        assert_eq!(got.audio_languages, ["English", "Malay"]);
        assert_eq!(got.subtitle_languages, ["Indo"]);
    }

    #[test]
    fn mkv_skips_headers_before_tracks() {
        // A real file opens with SeekHead and Info before `Tracks`.
        let segment = [
            el(&[0x11, 0x4D, 0x9B, 0x74], &[0u8; 12]),
            el(&[0x15, 0x49, 0xA9, 0x66], &uint(&[0x2A, 0xD7, 0xB1], 1)),
            tracks(&[track(2, "A_AAC", &[(LANG, "eng")], Some(2))]),
        ]
        .concat();
        let got = probe(&mkv("matroska", &segment)).expect("a track list");
        assert_eq!(got.audio, ["AAC"]);
        assert_eq!(got.audio_languages, ["English"]);
    }

    #[test]
    fn mkv_unknown_size_element_before_tracks_is_none() {
        // `Void` with a one-byte unknown length: nothing after it is addressable,
        // so the walk must give up rather than guess where it ends.
        let segment = [
            vec![0xEC, 0xFF],
            tracks(&[track(2, "A_AAC", &[], Some(2))]),
        ]
        .concat();
        assert_eq!(probe(&mkv("matroska", &segment)), None);
    }

    #[test]
    fn mkv_truncated_inside_tracks_is_none() {
        let doc = mkv(
            "matroska",
            &tracks(&[track(2, "A_AAC", &[(LANG, "eng")], Some(2))]),
        );
        let prefix = doc.get(..doc.len() - 4).expect("a cut inside the fixture");
        assert_eq!(probe(prefix), None);
    }

    #[test]
    fn webm_with_only_a_video_track_reports_no_audio() {
        let doc = mkv("webm", &tracks(&[track(1, "V_VP9", &[], None)]));
        let got = probe(&doc).expect("a track list");
        assert_eq!(got, ContainerTracks::default());
    }

    #[test]
    fn mp4_reads_audio_and_subtitle_tracks() {
        let mut file = ftyp();
        file.extend_from_slice(&iso(
            b"moov",
            &[
                trak(b"soun", packed(b"eng"), &audio_entry(b"mp4a", 2)),
                trak(b"subt", packed(b"may"), &iso(b"tx3g", &[0u8; 16])),
            ]
            .concat(),
        ));
        file.extend_from_slice(&iso(b"mdat", &[0u8; 16]));

        let got = probe(&file).expect("moov in the prefix");
        assert_eq!(got.audio, ["AAC"]);
        assert_eq!(got.channels, [2]);
        assert_eq!(got.audio_languages, ["English"]);
        assert_eq!(got.subtitle_languages, ["Malay"]);
    }

    #[test]
    fn mp4_moov_behind_an_unreachable_mdat_is_none() {
        let ftyp = ftyp();
        let moov = iso(
            b"moov",
            &trak(b"soun", packed(b"eng"), &audio_entry(b"mp4a", 2)),
        );
        let full = [ftyp.clone(), iso(b"mdat", &[0u8; 4096]), moov].concat();
        // One range request: ftyp, the mdat header, 32 bytes of the mdat.
        let prefix = full
            .get(..ftyp.len() + 8 + 32)
            .expect("a prefix inside the fixture");
        assert_eq!(probe(prefix), None);
    }

    #[test]
    fn garbage_is_none() {
        assert_eq!(probe(&[]), None);
        assert_eq!(probe(b"MZ\x90\x00\x03\x00\x00\x00"), None);
        assert_eq!(probe(&[0x1A, 0x45, 0xDF, 0xA3]), None);
        let noise: Vec<u8> = (0..96u32).map(|i| (i * 37 % 251) as u8).collect();
        assert_eq!(probe(&noise), None);
    }

    #[test]
    fn mp4_zero_or_short_box_size_in_the_middle_is_none() {
        let good = trak(b"soun", packed(b"eng"), &audio_entry(b"mp4a", 2));

        let mut zero = Vec::new();
        zero.extend_from_slice(&0u32.to_be_bytes()); // "to end of file"
        zero.extend_from_slice(b"trak");
        zero.extend_from_slice(&[0u8; 8]);

        let mut short = Vec::new();
        short.extend_from_slice(&4u32.to_be_bytes()); // smaller than its own header
        short.extend_from_slice(b"trak");
        short.extend_from_slice(&[0u8; 4]);

        for junk in [zero, short] {
            let mut moov = junk;
            moov.extend_from_slice(&good);
            let mut file = ftyp();
            file.extend_from_slice(&iso(b"moov", &moov));
            assert_eq!(probe(&file), None);
        }
    }
}
