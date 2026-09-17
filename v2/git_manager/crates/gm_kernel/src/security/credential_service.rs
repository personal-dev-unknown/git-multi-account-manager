
// crates/gm_kernel/src/security/credential_service.rs
//
// CredentialService is the kernel-level bridge that provider plugins use to
// retrieve the plaintext credentials (PAT, OAuth token) for a specific account.
// It is registered in the service registry during bootstrap, giving plugins a
// clean way to fetch credentials without importing gm_adapters.
//
// ── Why in gm_kernel and not gm_ports ─────────────────────────────────────────
// CredentialService combines TWO kernel concerns:
//   1. The CredentialStore (gm_ports outbound port) — retrieves encrypted bytes
//   2. The CredentialVault (gm_kernel security) — decrypts those bytes
//
// Because it depends on CredentialVault (defined in this same crate), it belongs
// in gm_kernel. Plugins already import gm_kernel, so they can look this up from
// the service registry in on_load() without needing gm_adapters.
//
// ── Credential storage convention ────────────────────────────────────────────
// The store key scheme is:
//   label    = "account-pat"
//   username = "<account_uuid_as_string>"
//
// The value is the JSON-serialised EncryptedValue from CredentialVault::encrypt().
// This allows the vault to re-derive the machine key and decrypt at retrieval time.

use std::sync::Arc;
use serde_json;
use uuid::Uuid;

use gm_ports::outbound::CredentialStore;
use gm_shared::errors::GitManagerError;
use crate::security::{CredentialVault, EncryptedValue};

/// The key namespace used when storing account credentials.
pub const ACCOUNT_PAT_LABEL: &str = "account-pat";

/// Stores and retrieves plaintext credentials for accounts.
/// Register in the service registry during bootstrap so that provider plugins
/// can look it up in `on_load()` without importing the adapter crate.
pub struct CredentialService {
    store: Arc<dyn CredentialStore>,
    vault: Arc<CredentialVault>,
}

impl std::fmt::Debug for CredentialService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialService")
            .field("store", &"<CredentialStore>")
            .field("vault", &self.vault)
            .finish()
    }
}

impl CredentialService {
    pub fn new(store: Arc<dyn CredentialStore>, vault: Arc<CredentialVault>) -> Self {
        Self { store, vault }
    }

    /// Encrypts `plaintext_token` with the CredentialVault and stores the
    /// result in the CredentialStore under the account's UUID as the key.
    pub async fn store_token(
        &self,
        account_id: Uuid,
        plaintext_token: &str,
    ) -> Result<(), GitManagerError> {
        let encrypted = self.vault.encrypt(plaintext_token.as_bytes())?;
        // Serialise the EncryptedValue struct to JSON so the store holds a
        // self-contained blob (ciphertext + IV + key_id together).
        let blob = serde_json::to_vec(&encrypted)
            .map_err(|e| GitManagerError::Other(format!("credential serialise: {e}")))?;
        self.store
            .store(ACCOUNT_PAT_LABEL, &account_id.to_string(), &blob)
            .await
    }

    /// Retrieves and decrypts the plaintext token for an account.
    /// Returns None if no credential has been stored for this account.
    pub async fn get_token(&self, account_id: Uuid) -> Result<Option<String>, GitManagerError> {
        let blob = self.store
            .retrieve(ACCOUNT_PAT_LABEL, &account_id.to_string())
            .await?;

        let Some(bytes) = blob else { return Ok(None) };

        let ev: EncryptedValue = serde_json::from_slice(&bytes)
            .map_err(|e| GitManagerError::Other(format!("credential deserialise: {e}")))?;

        let plaintext = self.vault.decrypt(&ev)?;
        let token = String::from_utf8(plaintext)
            .map_err(|e| GitManagerError::Other(format!("credential not valid UTF-8: {e}")))?;

        Ok(Some(token))
    }

    /// Removes the stored credential for an account (called during account deletion).
    pub async fn delete_token(&self, account_id: Uuid) -> Result<(), GitManagerError> {
        self.store
            .delete(ACCOUNT_PAT_LABEL, &account_id.to_string())
            .await
    }
}

// EncryptedValue needs serde for JSON round-trip through the credential store.
// Add derives to gm_kernel/src/security/credential_vault.rs::EncryptedValue.