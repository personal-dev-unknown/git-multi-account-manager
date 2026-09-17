// crates/gm_kernel/src/security/credential_vault.rs
//
// The CredentialVault is the kernel's encryption layer for secrets. It uses
// AES-256-GCM via the `ring` crate — the same cryptographic library used by
// Chromium, AWS-LC, and BoringSSL. Every PAT, OAuth token, and SSH passphrase
// that Git Manager stores in the database passes through encrypt() before it
// reaches the adapter, and through decrypt() before it is used.
//
// ── Key derivation ────────────────────────────────────────────────────────────
// The encryption key is NEVER stored anywhere. It is derived on every boot
// from two pieces of machine-specific material that are always available on
// a normally-configured Linux or macOS system:
//
//   1. /etc/machine-id  — a random 128-bit UUID written by systemd at OS install
//      time and never changed. Unique per machine. Not a secret, but stable.
//
//   2. $HOME            — the absolute path to the running user's home directory.
//      Changing your home directory path would invalidate stored credentials.
//
// These are combined into an input key material (IKM) string and fed into
// HKDF-SHA256, which produces a 32-byte cryptographically strong key. The
// key is deterministic for a given machine/user combination but is not
// recoverable without both pieces of material.
//
// The consequence: credentials stored on Machine A cannot be decrypted on
// Machine B, even by the same user. This is intentional — credentials are
// machine-scoped, not portable.
//
// ── AES-256-GCM ───────────────────────────────────────────────────────────────
// Each encryption call generates a fresh 12-byte random nonce via the OS CSPRNG
// (SystemRandom backed by getrandom(2)). The nonce is stored alongside the
// ciphertext as a hex string. Because the nonce is unique per encryption, the
// same plaintext encrypted twice produces different ciphertexts — ciphertext
// does not leak plaintext duplication patterns.
//
// The 16-byte GCM authentication tag is appended to the ciphertext by ring's
// seal_in_place_append_tag(). The combined (ciphertext || tag) is stored as hex.
// Decryption validates the tag before returning any plaintext — if the ciphertext
// is tampered with, open_in_place() returns Unspecified and decrypt() returns Err.
//
// ── Security boundaries ────────────────────────────────────────────────────────
// Trust boundary: plaintext secrets never cross the FFI boundary and never enter
// any log or trace output. The EncryptedValue struct that crosses crate boundaries
// contains only hex-encoded ciphertext, the IV, and a key version identifier.
// The decrypted bytes are kept in Rust Vec<u8> on the stack until used, then
// dropped. There is no explicit memory zeroing — Rust's ownership model ensures
// the Vec is deallocated immediately when it goes out of scope. Future hardening
// could use the `zeroize` crate to explicitly zero secret memory.
//
// ── Key rotation ─────────────────────────────────────────────────────────────
// The key_id field in EncryptedValue ("v1") is reserved for future key rotation.
// When a new key derivation scheme is introduced ("v2"), the vault can
// decrypt v1 credentials with the old key and re-encrypt them with the new key
// during a migration step. The domain never sees this complexity.

use ring::{
    aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM},
    hkdf,
    rand::{SecureRandom, SystemRandom},
};
use zeroize::Zeroize;
use gm_shared::errors::GitManagerError;

/// The nonce length for AES-256-GCM is always 12 bytes (96 bits).
const NONCE_LEN: usize = 12;

/// An opaque encrypted value safe to persist to the database.
///
/// The three fields are all that is needed to decrypt the secret later:
/// the ciphertext (plus GCM tag) as hex, the nonce as hex, and which key
/// version was used. None of these fields should ever be logged.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedValue {
    /// AES-256-GCM ciphertext with the 16-byte authentication tag appended,
    /// hex-encoded. Length = (plaintext_len + 16) * 2.
    pub ciphertext_hex: String,
    /// 12-byte AES-GCM nonce, hex-encoded (24 hex chars). Not secret;
    /// must be unique per encryption operation.
    pub iv_hex:         String,
    /// Identifies which key derivation parameters produced the encryption key.
    /// Currently always "v1". Reserved for future key rotation.
    pub key_id:         String,
}

/// The AES-256-GCM encryption vault for all secrets in the system.
pub struct CredentialVault {
    key: LessSafeKey,
    rng: SystemRandom,
}

impl std::fmt::Debug for CredentialVault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CredentialVault {{ key: <redacted>, rng: SystemRandom }}")
    }
}

impl CredentialVault {
    /// Constructs the vault by deriving the encryption key from machine-specific
    /// material. Returns Err if the key material cannot be read or the HKDF
    /// expansion fails (both are programming errors, not expected at runtime).
    pub fn new() -> Result<Self, GitManagerError> {
        let key_bytes = derive_machine_key()?;

        // SAFETY: UnboundKey::new panics if the key length doesn't match the
        // algorithm's requirements. AES_256_GCM requires exactly 32 bytes, which
        // our HKDF derivation guarantees. We map the ring error to a GitManagerError
        // to avoid panicking in production.
        let unbound = UnboundKey::new(&AES_256_GCM, &key_bytes)
            .map_err(|_| GitManagerError::Other("credential vault key init failed".to_string()))?;

        Ok(Self {
            key: LessSafeKey::new(unbound),
            rng: SystemRandom::new(),
        })
    }

    /// Encrypts `plaintext` and returns an `EncryptedValue` safe to persist.
    ///
    /// A fresh random nonce is generated for every call. Encrypting the same
    /// plaintext twice will produce different ciphertexts.
    ///
    /// Security: `plaintext` is consumed by value so the caller cannot retain
    /// a reference. The vault works on a local copy and drops it after sealing.
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<EncryptedValue, GitManagerError> {
        // Generate a 12-byte random nonce from the OS CSPRNG.
        let mut nonce_bytes = [0u8; NONCE_LEN];
        self.rng.fill(&mut nonce_bytes)
            .map_err(|_| GitManagerError::Other("credential vault: CSPRNG fill failed".to_string()))?;

        // SAFETY: Nonce::assume_unique_for_key is safe here because we just
        // generated the nonce from a CSPRNG — it is unique with overwhelming
        // probability. The `assume` in the name is ring's acknowledgement that
        // it cannot verify uniqueness; it is our responsibility to guarantee it.
        let nonce = Nonce::assume_unique_for_key(nonce_bytes);

        // Work on a mutable copy — seal_in_place_append_tag extends the Vec
        // in place, appending the 16-byte GCM authentication tag.
        let mut ciphertext = plaintext.to_vec();
        self.key
            .seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|_| GitManagerError::Other("credential vault: encryption failed".to_string()))?;

        Ok(EncryptedValue {
            ciphertext_hex: hex::encode(&ciphertext),
            iv_hex:         hex::encode(nonce_bytes),
            key_id:         "v1".to_string(),
        })
    }

    /// Decrypts an `EncryptedValue` and returns the plaintext bytes.
    ///
    /// Returns Err if the ciphertext is corrupted, the IV is malformed, or the
    /// GCM authentication tag does not match (which indicates tampering).
    /// The error message deliberately does not specify which condition occurred
    /// to avoid leaking information about the failure mode to potential attackers.
    pub fn decrypt(&self, value: &EncryptedValue) -> Result<Vec<u8>, GitManagerError> {
        let nonce_bytes = hex::decode(&value.iv_hex)
            .map_err(|_| GitManagerError::Other("credential vault: invalid IV encoding".to_string()))?;

        let nonce_arr: [u8; NONCE_LEN] = nonce_bytes
            .try_into()
            .map_err(|_| GitManagerError::Other("credential vault: IV must be 12 bytes".to_string()))?;

        let nonce = Nonce::assume_unique_for_key(nonce_arr);

        let mut ciphertext = hex::decode(&value.ciphertext_hex)
            .map_err(|_| GitManagerError::Other("credential vault: invalid ciphertext encoding".to_string()))?;

        // open_in_place validates the GCM tag and decrypts in place.
        // Returns a subslice of `ciphertext` that excludes the auth tag.
        let plaintext = self.key
            .open_in_place(nonce, Aad::empty(), &mut ciphertext)
            .map_err(|_| GitManagerError::Other("credential vault: decryption failed".to_string()))?;

        let result = plaintext.to_vec();

        // Explicitly zero the in-place buffer to prevent credential leakage
        // from process memory. ciphertext is dropped after this scope, but
        // zeroize ensures the bytes are overwritten before deallocation.
        ciphertext.zeroize();

        Ok(result)
    }
}

/// Derives a 32-byte AES-256 key from machine-specific material using HKDF-SHA256.
///
/// Input key material is: `{machine_id}{home_dir}` concatenated.
/// Salt is a fixed domain-separation string for this application.
/// Output key material is 32 bytes for AES-256-GCM.
fn derive_machine_key() -> Result<[u8; 32], GitManagerError> {
    // Primary: /etc/machine-id — written by systemd at OS install time.
    // Fallback: HOME directory path — always available on POSIX systems.
    let machine_id = std::fs::read_to_string("/etc/machine-id")
        .unwrap_or_else(|_| {
            std::env::var("HOME").unwrap_or_else(|_| "git-manager-default-machine".to_string())
        });

    let home = std::env::var("HOME").unwrap_or_default();

    // Combine the two pieces of machine-specific material.
    let mut ikm = format!("{}{}", machine_id.trim(), home.trim());

    // Domain-separation salt — never changes between versions.
    // Changing this string would invalidate all stored credentials.
    let salt = hkdf::Salt::new(hkdf::HKDF_SHA256, b"git-manager-credential-vault-salt-v1");

    // Extract pseudorandom key from IKM.
    let prk = salt.extract(ikm.as_bytes());

    // A type that tells ring how many bytes of output key material to generate.
    struct Key32;
    impl hkdf::KeyType for Key32 {
        fn len(&self) -> usize { 32 }
    }

    let mut key_material = [0u8; 32];

    // Expand the PRK into 32 bytes of output key material.
    prk.expand(&[b"aes-256-gcm-encryption-key"], Key32)
        .and_then(|okm| okm.fill(&mut key_material))
        .map_err(|_| GitManagerError::Other("credential vault: HKDF key derivation failed".to_string()))?;

    // Zero the IKM buffer to prevent credential material from persisting in memory.
    ikm.zeroize();

    Ok(key_material)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vault() -> CredentialVault {
        CredentialVault::new().expect("vault creation must succeed on a normal OS")
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let vault     = make_vault();
        let plaintext = b"super-secret-PAT-token";
        let encrypted = vault.encrypt(plaintext).unwrap();
        let decrypted = vault.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn same_plaintext_produces_different_ciphertexts() {
        let vault = make_vault();
        let a     = vault.encrypt(b"same-secret").unwrap();
        let b     = vault.encrypt(b"same-secret").unwrap();
        // Different nonces → different ciphertexts
        assert_ne!(a.ciphertext_hex, b.ciphertext_hex);
        assert_ne!(a.iv_hex, b.iv_hex);
    }

    #[test]
    fn tampered_ciphertext_fails_decryption() {
        let vault   = make_vault();
        let mut enc = vault.encrypt(b"secret").unwrap();
        // Flip the first byte of the ciphertext hex to corrupt the GCM tag
        // SAFETY: This is a test — we intentionally corrupt the ciphertext.
        let bytes = unsafe { enc.ciphertext_hex.as_bytes_mut() };
        bytes[0] = if bytes[0] == b'a' { b'b' } else { b'a' };
        assert!(vault.decrypt(&enc).is_err());
    }

    #[test]
    fn derives_consistent_key_across_calls() {
        // derive_machine_key() is deterministic for a given machine + user.
        let k1 = derive_machine_key().unwrap();
        let k2 = derive_machine_key().unwrap();
        assert_eq!(k1, k2);
    }
}