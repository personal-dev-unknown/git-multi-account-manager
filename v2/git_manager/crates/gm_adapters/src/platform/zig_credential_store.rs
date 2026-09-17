// crates/gm_adapters/src/platform/zig_credential_store.rs
//
// ZigCredentialStore implements the CredentialStore outbound port defined in
// gm_ports. It delegates every operation to the Zig platform layer, which
// dispatches to the correct OS backend at compile time:
//   Linux   → GNOME libsecret (via D-Bus)
//   macOS   → Keychain Access (via Security.framework)
//   Windows → Windows Credential Manager (via CredWriteW/CredReadW)
//   Other   → encrypted file under ~/.git-zyrix/.keyring
//
// ── Defence in depth ─────────────────────────────────────────────────────────
// The kernel's CredentialVault encrypts secrets with AES-256-GCM BEFORE they
// reach this layer, so the OS keychain is holding ciphertext, not plaintext.
// This means even a keychain breach reveals only encrypted blobs — an attacker
// would additionally need the machine-derived key (not stored anywhere).
//
// ── Thread safety ────────────────────────────────────────────────────────────
// The FFI calls are blocking (they go through D-Bus or framework calls). Each
// call is dispatched via spawn_blocking to avoid stalling the Tokio scheduler.
// ZigCredentialStore is stateless so multiple concurrent calls are safe.

use async_trait::async_trait;
use gm_shared::errors::GitManagerError;
use gm_ports::outbound::CredentialStore;

use super::ffi;

#[derive(Debug, Default)]
pub struct ZigCredentialStore;

impl ZigCredentialStore {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl CredentialStore for ZigCredentialStore {
    async fn store(
        &self,
        label:    &str,
        username: &str,
        secret:   &[u8],
    ) -> Result<(), GitManagerError> {
        let label_owned    = label.to_owned();
        let username_owned  = username.to_owned();
        let c_label         = ffi::to_cstring(label);
        let c_username      = ffi::to_cstring(username);
        let secret_vec      = secret.to_vec();

        tokio::task::spawn_blocking(move || {
            // SAFETY: CStrings are valid null-terminated. secret_vec is a valid
            // contiguous byte buffer; its pointer is only dereferenced during
            // this synchronous call and is not retained by the Zig function.
            let ok = unsafe {
                ffi::gm_platform_store_secret(
                    c_label.as_ptr(),
                    c_username.as_ptr(),
                    secret_vec.as_ptr(),
                    secret_vec.len(),
                )
            };
            if ok {
                Ok(())
            } else {
                Err(GitManagerError::Other(
                    format!("credential store: failed to store secret for '{label_owned}/{username_owned}'")
                ))
            }
        })
        .await
        .map_err(|e| GitManagerError::Other(format!("credential store spawn_blocking panic: {e}")))?
    }

    async fn retrieve(
        &self,
        label:    &str,
        username: &str,
    ) -> Result<Option<Vec<u8>>, GitManagerError> {
        let c_label    = ffi::to_cstring(label);
        let c_username = ffi::to_cstring(username);

        tokio::task::spawn_blocking(move || {
            let mut result = ffi::FfiSecretResult::zeroed();

            // SAFETY: CStrings valid; result is valid stack allocation.
            let ok = unsafe {
                ffi::gm_platform_retrieve_secret(
                    c_label.as_ptr(),
                    c_username.as_ptr(),
                    &mut result as *mut ffi::FfiSecretResult,
                )
            };

            if !ok {
                let err_msg = result.error_str();
                // Zig returns false + "credential not found" when the secret
                // doesn't exist — this is a valid state, not an error.
                if err_msg.contains("credential not found") {
                    return Ok(None);
                }
                return Err(GitManagerError::Other(
                    format!("credential store: retrieve failed: {err_msg}")
                ));
            }

            if result.secret_len > 0 {
                Ok(Some(result.secret_bytes().to_vec()))
            } else {
                Ok(None)
            }
        })
        .await
        .map_err(|e| GitManagerError::Other(format!("credential store spawn_blocking panic: {e}")))?
    }

    async fn delete(&self, label: &str, username: &str) -> Result<(), GitManagerError> {
        let c_label    = ffi::to_cstring(label);
        let c_username = ffi::to_cstring(username);

        tokio::task::spawn_blocking(move || {
            // SAFETY: CStrings valid null-terminated strings.
            let _ = unsafe {
                ffi::gm_platform_delete_secret(c_label.as_ptr(), c_username.as_ptr())
            };
            // delete is idempotent: even false means "already gone", which is fine.
            Ok::<(), GitManagerError>(())
        })
        .await
        .map_err(|e| GitManagerError::Other(format!("credential store spawn_blocking panic: {e}")))?
    }
}