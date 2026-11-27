//! Secrets AT REST: the api_hash pool and the cached api credentials.
//!
//! Why: `storage/api_ids.json` and `storage/api_credentials.json` held the
//! api_hash as readable text, and those files land in every backup, zip and
//! support bundle an operator makes — the exact copies that leave the machine.
//!
//! Construction follows `docs/research/rust-secret-at-rest.md`:
//!
//! * **XChaCha20-Poly1305** (`chacha20poly1305` 0.10, audited), not
//!   `crypto_box::ChaChaBox`. `ChaChaBox` is an ECIES primitive (X25519 +
//!   NaCl `crypto_secretbox`), it cannot bind AAD, and its ChaCha branch is the
//!   explicitly unaudited legacy crate.
//! * A **24-byte random nonce** per seal, so nonce reuse is not something this
//!   code has to reason about at all (draft-irtf-cfrg-xchacha §2.1).
//! * **Per-purpose AAD** (`api_ids`, `api_credentials`), so a ciphertext cannot
//!   be moved from one file to the other and still authenticate.
//! * Key material is `FD_API_SECRET` or a created-once `storage/at_rest.key`
//!   (atomic create, 0600), hashed to 32 bytes. Nothing here is a password KDF:
//!   the material is random, not user-chosen.
//! * **Fail closed**: callers refuse to write plaintext when sealing fails, and a
//!   value that will not decrypt is an error, never a silent pass-through.
//!
//! What it buys: a stolen COPY of `storage/` is unreadable. It does not defend
//! against code running as the same user — the key has to be there to boot.

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chacha20poly1305::aead::{Aead, AeadCore, KeyInit, OsRng, Payload};
use chacha20poly1305::{Key, XChaCha20Poly1305, XNonce};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::storage::storage_path;

/// Marker + format version. `enc1:` values are base64(nonce ‖ ciphertext).
const PREFIX: &str = "enc1:";
/// AAD is `LABEL ‖ purpose`, so ciphertexts are bound to the file they belong to.
const LABEL: &[u8] = b"pencarimovie/at-rest/v1\0";
const NONCE_LEN: usize = 24;
/// The key file must be exactly this many hex chars; anything else is a broken
/// key, not a shorter one. Accepting a truncated key silently derives a different
/// key and makes every sealed value undecryptable.
const KEY_HEX_LEN: usize = 64;

fn aad(purpose: &str) -> Vec<u8> {
    let mut v = LABEL.to_vec();
    v.extend_from_slice(purpose.as_bytes());
    v
}

/// The 32-byte AEAD key, cached for the life of the process.
///
/// Caching is load-bearing: generating fresh material per call would seal with one
/// key and unseal with another in the same run (a unit test caught exactly that).
fn key() -> Zeroizing32 {
    static KEY: std::sync::OnceLock<Zeroizing32> = std::sync::OnceLock::new();
    KEY.get_or_init(derive_key).clone()
}

#[derive(Clone)]
struct Zeroizing32([u8; 32]);

impl Drop for Zeroizing32 {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl Zeroizing32 {
    fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

fn read_material() -> Option<Vec<u8>> {
    if let Some(v) = crate::storage::env("FD_API_SECRET") {
        let t = v.trim().to_string();
        if !t.is_empty() {
            let mut bytes = t.into_bytes();
            let out = bytes.clone();
            bytes.zeroize();
            return Some(out);
        }
    }
    let file = storage_path("at_rest.key");
    if let Ok(text) = std::fs::read_to_string(&file) {
        let t = text.trim().to_string();
        // A short/truncated key file is a FAULT: report it instead of deriving a
        // different key and losing the pool.
        if t.len() == KEY_HEX_LEN && t.chars().all(|c| c.is_ascii_hexdigit()) {
            return Some(t.into_bytes());
        }
        log_bad_key(&file);
        return None;
    }
    None
}

fn log_bad_key(file: &std::path::Path) {
    crate::log::stderr_log(
        "at_rest.key is not a valid key - sealed secrets cannot be read",
        Some(&serde_json::json!({
            "path": file.display().to_string(),
            "expected_hex_chars": KEY_HEX_LEN,
            "fix": "restore the file from a backup, or delete it and re-add the api_ids (the pool is re-fetched from the server)",
        })),
    );
}

/// Create the key file once, atomically (create_new) and with 0600.
fn create_material() -> Vec<u8> {
    use rand::RngCore;
    let mut raw = [0u8; 32];
    rand::rng().fill_bytes(&mut raw);
    let text = hex::encode(raw);
    raw.zeroize();
    let file = storage_path("at_rest.key");
    if let Some(parent) = file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let mut opts = std::fs::OpenOptions::new();
    opts.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.mode(0o600);
    }
    match opts.open(&file) {
        Ok(mut f) => {
            use std::io::Write;
            let _ = f.write_all(text.as_bytes());
            let _ = f.sync_all();
            text.into_bytes()
        }
        Err(_) => {
            // Another process created it a moment ago: use THAT key, or the two
            // would seal with different material.
            read_material().unwrap_or_else(|| {
                // Last resort: an in-memory key keeps this run working; the log is
                // the only way the operator learns the pool will not survive a
                // restart.
                crate::log::stderr_log(
                    "at_rest.key could not be created - secrets are readable only this run",
                    Some(&serde_json::json!({"path": file.display().to_string()})),
                );
                let mut raw2 = [0u8; 32];
                rand::rng().fill_bytes(&mut raw2);
                hex::encode(raw2).into_bytes()
            })
        }
    }
}

fn derive_key() -> Zeroizing32 {
    let material = read_material().unwrap_or_else(create_material);
    let mut hasher = Sha256::new();
    hasher.update(LABEL);
    hasher.update(&material);
    let digest = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    let mut material = material;
    material.zeroize();
    Zeroizing32(out)
}

/// Encrypt a secret for disk. `purpose` binds it to one file.
pub fn seal(plain: &str, purpose: &str) -> Option<String> {
    let k = key();
    let cipher = XChaCha20Poly1305::new(Key::from_slice(k.as_bytes()));
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);
    let aad = aad(purpose);
    let ct = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: plain.as_bytes(),
                aad: &aad,
            },
        )
        .ok()?;
    let mut out = Vec::with_capacity(NONCE_LEN + ct.len());
    out.extend_from_slice(nonce.as_slice());
    out.extend_from_slice(&ct);
    Some(format!("{PREFIX}{}", STANDARD.encode(out)))
}

/// Decrypt a sealed value. A legacy plaintext value is accepted (so an upgrade
/// keeps working) and the caller re-saves it sealed; a sealed value that will not
/// open is `None`, NEVER the ciphertext.
pub fn open(value: &str, purpose: &str) -> Option<String> {
    let v = value.trim();
    if v.is_empty() {
        return None;
    }
    let Some(b64) = v.strip_prefix(PREFIX) else {
        return Some(v.to_string());
    };
    let raw = STANDARD.decode(b64).ok()?;
    if raw.len() < NONCE_LEN + 16 {
        return None;
    }
    let (nonce_bytes, ct) = raw.split_at(NONCE_LEN);
    let k = key();
    let cipher = XChaCha20Poly1305::new(Key::from_slice(k.as_bytes()));
    let aad = aad(purpose);
    let plain = cipher
        .decrypt(
            XNonce::from_slice(nonce_bytes),
            Payload {
                msg: ct,
                aad: &aad,
            },
        )
        .ok()?;
    let mut plain = plain;
    let text = String::from_utf8(plain.clone()).ok();
    plain.zeroize();
    text
}

pub fn is_sealed(value: &str) -> bool {
    value.trim().starts_with(PREFIX)
}

/// Best effort: keep a secret-bearing file readable only by its owner.
pub fn tighten(path: &std::path::Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sealed_value_round_trips_and_is_not_readable_text() {
        let sealed = seal("24e740e6382b709864f0adf8d7d86c3f", "api_ids").expect("seal");
        assert!(is_sealed(&sealed));
        assert!(
            !sealed.contains("24e740e6"),
            "the hash must not survive in the file"
        );
        assert_eq!(
            open(&sealed, "api_ids").as_deref(),
            Some("24e740e6382b709864f0adf8d7d86c3f")
        );
    }

    #[test]
    fn two_seals_of_one_value_differ() {
        let a = seal("same", "api_ids").unwrap();
        let b = seal("same", "api_ids").unwrap();
        assert_ne!(a, b, "a fresh nonce every time");
        assert_eq!(open(&a, "api_ids"), open(&b, "api_ids"));
    }

    #[test]
    fn a_ciphertext_cannot_be_moved_to_another_file() {
        let sealed = seal("42", "api_ids").unwrap();
        assert_eq!(open(&sealed, "api_ids").as_deref(), Some("42"));
        // Same key, different purpose: the AAD no longer matches.
        assert_eq!(open(&sealed, "api_credentials"), None);
    }

    #[test]
    fn a_legacy_plaintext_value_passes_through() {
        assert_eq!(open("plainvalue", "api_ids").as_deref(), Some("plainvalue"));
        assert!(!is_sealed("plainvalue"));
        assert_eq!(open("   ", "api_ids"), None);
    }

    #[test]
    fn a_tampered_value_is_refused_not_returned() {
        let sealed = seal("secret", "api_ids").unwrap();
        let mut raw = STANDARD.decode(sealed.trim_start_matches(PREFIX)).unwrap();
        let last = raw.len() - 1;
        raw[last] ^= 0x01;
        let broken = format!("{PREFIX}{}", STANDARD.encode(raw));
        assert_eq!(open(&broken, "api_ids"), None, "the tag must reject it");
    }
}
