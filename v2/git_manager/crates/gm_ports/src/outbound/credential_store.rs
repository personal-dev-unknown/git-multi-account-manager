// crates/gm_ports/src/outbound/credential_store.rs
//
// The CredentialStore outbound port — the platform OS credential backend.
// This is implemented by ZigCredentialStore in gm_adapters, which delegates
// to the platform keychain (GNOME libsecret on Linux, Keychain on macOS,
// Windows Credential Manager on Windows, encrypted file as fallback).
//
// The kernel's CredentialVault uses AES-256-GCM to encrypt credentials before
// they reach this layer, so the OS keychain holds already-encrypted ciphertext.
// The redundant encryption is intentional — defence in depth.
 

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;

/// Outbound port for persisting and retrieving encrypted credential blobs.
///
/// Each entry is addressed by a `(label, username)` pair. The caller (kernel)
/// is responsible for encrypting secrets before calling `store` and for
/// decrypting them after `retrieve` — this trait only moves opaque bytes.


/// Platform OS credential backend.
/// Implemented by ZigCredentialStore, which calls into the Zig native layer.
#[async_trait]
pub trait CredentialStore: Send + Sync {
    /// Persist an encrypted secret blob under `(label, username)`.
    async fn store(
        &self,
        label:    &str,
        username: &str,
        secret:   &[u8],
    ) -> Result<(), GitManagerError>;

    /// Retrieve the encrypted secret blob for `(label, username)`.
    ///
    /// Returns `None` if no entry exists for this key.
    async fn retrieve(
        &self,
        label:    &str,
        username: &str,
    ) -> Result<Option<Vec<u8>>, GitManagerError>;

    /// Remove the entry for `(label, username)`. Idempotent — deleting a
    /// non-existent entry is not an error.
    async fn delete(
        &self,
        label:    &str,
        username: &str,
    ) -> Result<(), GitManagerError>;
}
