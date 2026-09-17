//! SSH operation error variants.

use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum SshError {
    /// ssh-keygen exited non-zero. The `reason` contains its stderr output.
    #[error("SSH key generation failed: {reason}")]
    KeyGenerationFailed { reason: String },

    /// SSH_AUTH_SOCK is not set or points to a non-existent socket.
    /// The user needs to start an SSH agent (eval "$(ssh-agent -s)").
    #[error("SSH agent is not running — set SSH_AUTH_SOCK or run: eval \"$(ssh-agent -s)\"")]
    AgentNotRunning,

    /// ssh-add rejected the key. Possible causes: wrong passphrase, corrupt key
    /// file, or the agent socket is stale.
    #[error("Failed to load key '{key_path}' into SSH agent: {reason}")]
    AgentLoadFailed { key_path: String, reason: String },

    /// The SSH connection test subprocess failed to connect at all (network level).
    #[error("SSH connection to '{host}' failed: {reason}")]
    ConnectionFailed { host: String, reason: String },

    /// The server responded but did not confirm authentication. The public key
    /// is probably not registered with the hosting platform account.
    #[error("SSH key not accepted by '{host}' — add the public key to your account")]
    AuthenticationFailed { host: String },

    /// The atomic write to ~/.ssh/config failed. The original file is intact
    /// (the atomic write pattern guarantees this) but the new entry was not added.
    #[error("Failed to write SSH config entry: {reason}")]
    ConfigWriteFailed { reason: String },

    /// The private key file at the expected path does not exist. This means
    /// the file was deleted outside of Git Manager's control.
    #[error("SSH private key file not found: {path}")]
    KeyFileNotFound { path: String },

    /// The `ssh-keygen` binary is not available on PATH.
    #[error("ssh-keygen not found on PATH — please install OpenSSH")]
    SshKeygenNotFound,

    /// The SSH key file has incorrect permissions (not 0600). OpenSSH refuses
    /// to use keys that are readable by other users.
    #[error("SSH key file '{path}' has incorrect permissions — expected 0600, got {mode:#o}")]
    InsecureKeyPermissions { path: String, mode: u32 },

    /// A persistence/storage operation (save, find, delete) failed. Wraps the
    /// underlying database or I/O error message. Used by repository adapters
    /// that implement SshKeyRepository and SshHostConfigRepository.
    #[error("SSH storage error: {0}")]
    StorageFailed(String),
}