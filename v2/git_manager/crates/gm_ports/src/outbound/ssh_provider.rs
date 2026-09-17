// crates/gm_ports/src/outbound/ssh_provider.rs
//
// The official SshProvider outbound port — the full public interface that the
// kernel uses to wire the Zig SSH adapter. This is distinct from the minimal
// SshOperations trait defined inside gm_domain's ssh_service.rs:
//
//   gm_domain::ssh::services::SshOperations  — what SshService needs (minimal)
//   gm_ports::outbound::SshProvider           — what the kernel wires (full)
//
// ZigSshProvider in gm_adapters implements BOTH. The kernel registers it under
// SshProvider; the SshService constructor receives it as SshOperations via the
// service registry.

use std::path::{Path, PathBuf};
use async_trait::async_trait;
use gm_shared::errors::SshError;

/// Options passed to the SSH key generation operation.
pub struct SshKeygenOptions {
    /// "ed25519", "rsa", or "ecdsa"
    pub key_type:    String,
    pub email:       String,
    /// Absolute path for the private key file.
    pub output_path: PathBuf,
    pub passphrase:  Option<String>,
}

/// Result of a successful SSH key generation.
pub struct SshKeygenResult {
    pub public_key:  String,
    pub fingerprint: String,
}

/// Result of an SSH connection test.
pub struct SshConnectionResult {
    pub success:  bool,
    /// The platform-confirmed username if successful (e.g. "shakamoses" from "Hi shakamoses!").
    pub username: Option<String>,
    pub error:    Option<String>,
}

/// The official SSH provider outbound port.
/// Implemented by ZigSshProvider in gm_adapters.
/// The kernel registers the concrete adapter in the service registry so that
/// both the SshService (via SshOperations) and any kernel-level SSH
/// orchestration can reach it.
#[async_trait]
pub trait SshProvider: Send + Sync {
    async fn generate_key(&self, opts: SshKeygenOptions) -> Result<SshKeygenResult, SshError>;
    async fn add_to_agent(&self, key_path: &Path, passphrase: Option<&str>) -> Result<(), SshError>;
    async fn remove_from_agent(&self, key_path: &Path) -> Result<(), SshError>;
    async fn test_connection(&self, host: &str, key_path: &Path, timeout_ms: u32) -> Result<SshConnectionResult, SshError>;
    async fn write_config_entry(&self, host_alias: &str, hostname: &str, identity_file: &Path, port: u16) -> Result<(), SshError>;
    async fn remove_config_entry(&self, host_alias: &str) -> Result<(), SshError>;
    async fn is_agent_running(&self) -> bool;
}