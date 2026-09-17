// crates/gm_adapters/src/ssh/zig_ssh_provider.rs
//
// ZigSshProvider is the Rust adapter that wraps the unsafe Zig FFI calls in a
// safe, ergonomic interface. It implements two traits:
//
//   gm_ports::outbound::SshProvider       — the official cross-cutting port
//   gm_domain::ssh::services::SshOperations — the minimal domain-level port
//
// Both traits serve the same conceptual purpose (SSH operations) but from
// different vantage points. The domain trait is narrower; the port trait adds
// remove_from_agent, remove_config_entry, and is_agent_running which the domain
// service doesn't need but command handlers may use directly.
//
// ── Safety boundary ───────────────────────────────────────────────────────────
// All `unsafe` blocks are in this file and in ffi.rs. No other file in
// gm_adapters uses unsafe. Each unsafe block has an explicit SAFETY comment
// documenting why the invariants are upheld.
//
// ── Concurrency ───────────────────────────────────────────────────────────────
// ZigSshProvider is stateless. Every method constructs CStrings, calls the Zig
// function, reads the result, and returns. Multiple async tasks can call it
// concurrently because each call operates on independent stack allocations.
// The Zig layer itself uses only subprocess execution (no shared mutable state),
// so concurrent calls are safe.

use std::path::Path;

use async_trait::async_trait;
use gm_shared::errors::SshError;
use gm_ports::outbound::ssh_provider::{
    SshConnectionResult, SshKeygenOptions, SshKeygenResult, SshProvider,
};

use super::ffi;

// ─────────────────────────────────────────────────────────────────────────────
// The adapter struct
// ─────────────────────────────────────────────────────────────────────────────

/// Stateless SSH operations adapter backed by the Zig native layer.
/// Wrap in Arc<ZigSshProvider> and register in the kernel's service registry.
#[derive(Debug, Default)]
pub struct ZigSshProvider;

impl ZigSshProvider {
    pub fn new() -> Self {
        Self
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// impl SshProvider (gm_ports)
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl SshProvider for ZigSshProvider {
    async fn generate_key(&self, opts: SshKeygenOptions) -> Result<SshKeygenResult, SshError> {
        let key_type   = ffi::to_cstring(&opts.key_type);
        let email      = ffi::to_cstring(&opts.email);
        let out_path   = ffi::to_cstring(opts.output_path.to_str().ok_or_else(|| {
            SshError::ConfigWriteFailed { reason: "output path contains non-UTF-8".to_string() }
        })?);
        let passphrase = ffi::to_cstring(opts.passphrase.as_deref().unwrap_or(""));

        let mut result = ffi::FfiKeygenResult::zeroed();

        // SAFETY: All CStrings are valid null-terminated UTF-8 strings created
        // by ffi::to_cstring. result is a valid stack allocation. The Zig
        // function writes into the result buffer within the declared bounds.
        let success = unsafe {
            ffi::gm_ssh_generate_key(
                key_type.as_ptr(),
                email.as_ptr(),
                out_path.as_ptr(),
                passphrase.as_ptr(),
                &mut result as *mut ffi::FfiKeygenResult,
            )
        };

        if success {
            Ok(SshKeygenResult {
                public_key:  result.public_key_str().to_string(),
                fingerprint: result.fingerprint_str().to_string(),
            })
        } else {
            Err(SshError::KeyGenerationFailed {
                reason: result.error_str().to_string(),
            })
        }
    }

    async fn add_to_agent(&self, key_path: &Path, passphrase: Option<&str>) -> Result<(), SshError> {
        let path       = ffi::to_cstring(key_path.to_str().unwrap_or(""));
        let passphrase = ffi::to_cstring(passphrase.unwrap_or(""));

        // SAFETY: Both CStrings are valid null-terminated strings.
        let success = unsafe {
            ffi::gm_ssh_add_to_agent(path.as_ptr(), passphrase.as_ptr())
        };

        if success {
            Ok(())
        } else {
            Err(SshError::AgentLoadFailed {
                key_path: key_path.to_string_lossy().into_owned(),
                reason:   "ssh-add returned non-zero".to_string(),
            })
        }
    }

    async fn remove_from_agent(&self, key_path: &Path) -> Result<(), SshError> {
        let path = ffi::to_cstring(key_path.to_str().unwrap_or(""));

        // SAFETY: CString is valid null-terminated.
        let success = unsafe { ffi::gm_ssh_remove_from_agent(path.as_ptr()) };

        if success {
            Ok(())
        } else {
            Err(SshError::AgentLoadFailed {
                key_path: key_path.to_string_lossy().into_owned(),
                reason:   "ssh-add -d returned non-zero".to_string(),
            })
        }
    }

    async fn test_connection(
        &self,
        host:       &str,
        key_path:   &Path,
        timeout_ms: u32,
    ) -> Result<SshConnectionResult, SshError> {
        let c_host = ffi::to_cstring(host);
        let c_path = ffi::to_cstring(key_path.to_str().unwrap_or(""));
        let mut result = ffi::FfiConnectionResult::zeroed();

        // SAFETY: CStrings are valid null-terminated. result is a valid stack
        // allocation. timeout_ms is a plain integer.
        let success = unsafe {
            ffi::gm_ssh_test_connection(
                c_host.as_ptr(),
                c_path.as_ptr(),
                timeout_ms,
                &mut result as *mut ffi::FfiConnectionResult,
            )
        };

        if success {
            let username = result.username_str();
            Ok(SshConnectionResult {
                success:  true,
                username: if username.is_empty() { None } else { Some(username.to_string()) },
                error:    None,
            })
        } else {
            Ok(SshConnectionResult {
                success:  false,
                username: None,
                error:    Some(result.error_str().to_string()),
            })
        }
    }

    async fn write_config_entry(
        &self,
        host_alias:    &str,
        hostname:      &str,
        identity_file: &Path,
        port:          u16,
    ) -> Result<(), SshError> {
        // Use the managed config approach (V1-compatible):
        //   1. Write Host block to ~/.ssh/gitzyrix/config
        //   2. Ensure Include ~/.ssh/gitzyrix/config in ~/.ssh/config
        super::managed_config::ManagedSshConfig::ensure_directory()?;
        super::managed_config::ManagedSshConfig::ensure_include_directive()?;
        super::managed_config::ManagedSshConfig::write_host_entry(
            host_alias,
            hostname,
            &identity_file.to_string_lossy(),
            port,
            None,
        )
    }

    async fn remove_config_entry(&self, host_alias: &str) -> Result<(), SshError> {
        super::managed_config::ManagedSshConfig::remove_host_entry(host_alias)
    }

    async fn is_agent_running(&self) -> bool {
        // SAFETY: gm_ssh_agent_running takes no arguments and returns bool.
        unsafe { ffi::gm_ssh_agent_running() }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// impl SshOperations (gm_domain minimal interface)
// ─────────────────────────────────────────────────────────────────────────────
//
// The domain's SshService uses a minimal SshOperations trait rather than the
// full SshProvider because the domain should not depend on gm_ports. Implementing
// both traits on the same struct avoids code duplication. The kernel registers
// ZigSshProvider under both trait types in the service registry.

use gm_domain::ssh::services::ssh_service::{SshOperations, GenerateKeyResult, ConnectionTestResult};

#[async_trait]
impl SshOperations for ZigSshProvider {
    async fn generate_key(
        &self,
        key_type_str: &str,
        email:        &str,
        output_path:  &Path,
        passphrase:   Option<&str>,
    ) -> Result<GenerateKeyResult, SshError> {
        let result = SshProvider::generate_key(
            self,
            SshKeygenOptions {
                key_type:    key_type_str.to_string(),
                email:       email.to_string(),
                output_path: output_path.to_path_buf(),
                passphrase:  passphrase.map(str::to_string),
            },
        )
        .await?;
        Ok(GenerateKeyResult { public_key: result.public_key, fingerprint: result.fingerprint })
    }

    async fn add_to_agent(&self, key_path: &Path, passphrase: Option<&str>) -> Result<(), SshError> {
        SshProvider::add_to_agent(self, key_path, passphrase).await
    }

    async fn test_connection(
        &self,
        host:       &str,
        key_path:   &Path,
        timeout_ms: u32,
    ) -> Result<ConnectionTestResult, SshError> {
        let r = SshProvider::test_connection(self, host, key_path, timeout_ms).await?;
        Ok(ConnectionTestResult { success: r.success, username: r.username, error: None })
    }

    async fn write_config_entry(
        &self,
        host_alias:    &str,
        hostname:      &str,
        identity_file: &Path,
        port:          u16,
    ) -> Result<(), SshError> {
        SshProvider::write_config_entry(self, host_alias, hostname, identity_file, port).await
    }

    async fn remove_config_entry(&self, host_alias: &str) -> Result<(), SshError> {
        SshProvider::remove_config_entry(self, host_alias).await
    }

    async fn check_key_permissions(&self, key_path: &Path) -> Result<(), SshError> {
        #[cfg(unix)]
        {
            let meta = std::fs::metadata(key_path).map_err(|e| SshError::KeyFileNotFound {
                path: format!("{}: {e}", key_path.display()),
            })?;
            use std::os::unix::fs::PermissionsExt;
            let mode = meta.permissions().mode() & 0o777;
            if mode != 0o600 {
                return Err(SshError::ConfigWriteFailed {
                    reason: format!("{} has bad permissions {:#o}, expected 0600", key_path.display(), mode),
                });
            }
            Ok(())
        }
        #[cfg(not(unix))]
        {
            let _ = key_path;
            Ok(())
        }
    }

    async fn check_agent_has_key(&self, key_path: &Path) -> Result<bool, SshError> {
        let c_path = ffi::to_cstring(key_path.to_str().unwrap_or(""));
        // SAFETY: CString is valid null-terminated.
        let present = unsafe { ffi::gm_ssh_agent_has_key(c_path.as_ptr()) };
        Ok(present)
    }

    async fn is_agent_running(&self) -> bool {
        // SAFETY: gm_ssh_agent_running takes no arguments.
        unsafe { ffi::gm_ssh_agent_running() }
    }
}