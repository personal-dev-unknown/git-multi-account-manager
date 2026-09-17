// crates/gm_domain/src/ssh/services/ssh_service.rs
//
// The SshService is the domain orchestrator for everything related to SSH keys:
// generating them, testing them against platforms, loading them into the agent,
// and writing the corresponding ~/.ssh/config entries.
//
// ── Why the SshOperations trait is defined here ───────────────────────────────
// The domain cannot depend on gm_ports (that would invert the dependency arrow).
// But the SshService needs a port to the actual SSH binary operations that live
// in the Zig layer. The solution: the service defines its OWN minimal abstraction
// right here. The ZigSshProvider in gm_adapters implements BOTH this domain-level
// trait AND the gm_ports::outbound::SshProvider trait (which is the official
// cross-cutting version for the kernel). The kernel wires the same concrete type
// for both. This keeps gm_domain completely self-contained.
//
// ── Key lifecycle ─────────────────────────────────────────────────────────────
// 1. generate_and_setup_key:
//      a. Deactivate any existing active key for the account
//      b. Run ssh-keygen via SshOperations::generate_key()
//      c. Persist the new SshKey record
//      d. Write ~/.ssh/config Host block via SshOperations::write_config_entry()
//      e. Optionally add to SSH agent via SshOperations::add_to_agent()
//      f. Publish SshKeyGenerated event
//
// 2. test_connection:
//      a. Find the active key for the account
//      b. Run SSH connection test via SshOperations::test_connection()
//      c. Update key test_status based on result
//      d. Publish SshKeyTested event

use std::path::{Path, PathBuf};
use std::sync::Arc;
use async_trait::async_trait;
use uuid::Uuid;

use crate::ssh::{
    entities::{SshKey, SshHostConfig},
    events::{SshKeyGenerated, SshKeyTested, SshKeyAddedToAgent},
    ports::{SshKeyRepository, SshHostConfigRepository},
    value_objects::KeyType,
};
use gm_shared::errors::SshError;

// ── Domain port for SSH OS operations ─────────────────────────────────────────
// This is the domain's own minimal abstraction over the Zig SSH layer.
// It mirrors gm_ports::outbound::SshProvider in shape; the kernel adapter
// implements both from the same concrete struct.

/// The result of a successful SSH key generation operation.
#[derive(Debug)]
pub struct GenerateKeyResult {
    /// The full public key string: "ssh-ed25519 AAAA... user@email"
    pub public_key:  String,
    /// SHA-256 fingerprint: "SHA256:xxxxxxxx..."
    pub fingerprint: String,
}

/// The result of an SSH connection test.
#[derive(Debug)]
pub struct ConnectionTestResult {
    /// True if the server authenticated the key successfully.
    pub success:  bool,
    /// The username the server confirmed (e.g. "Hi shakamoses!").
    pub username: Option<String>,
    /// Error description if the test failed.
    pub error:    Option<String>,
}

/// Domain-level abstraction over SSH OS operations.
/// Implemented by ZigSshProvider in gm_adapters.
#[async_trait]
pub trait SshOperations: Send + Sync + std::fmt::Debug {
    async fn generate_key(
        &self,
        key_type:     &str,
        email:        &str,
        output_path:  &Path,
        passphrase:   Option<&str>,
    ) -> Result<GenerateKeyResult, SshError>;

    async fn add_to_agent(
        &self,
        key_path:   &Path,
        passphrase: Option<&str>,
    ) -> Result<(), SshError>;

    async fn test_connection(
        &self,
        host:       &str,
        key_path:   &Path,
        timeout_ms: u32,
    ) -> Result<ConnectionTestResult, SshError>;

    async fn write_config_entry(
        &self,
        host_alias:    &str,
        hostname:      &str,
        identity_file: &Path,
        port:          u16,
    ) -> Result<(), SshError>;

    async fn remove_config_entry(&self, host_alias: &str) -> Result<(), SshError>;

    /// Check that the private key file has correct permissions (0600) and the
    /// public key file has 0644.
    async fn check_key_permissions(&self, key_path: &Path) -> Result<(), SshError>;

    /// Check whether the key is currently loaded in the running SSH agent.
    async fn check_agent_has_key(&self, key_path: &Path) -> Result<bool, SshError>;

    /// Check whether the SSH agent is running (SSH_AUTH_SOCK is set).
    async fn is_agent_running(&self) -> bool;
}

// ── Result structs carrying domain events ─────────────────────────────────────

#[derive(Debug)]
pub struct GenerateAndSetupResult {
    pub ssh_key:    SshKey,
    pub host_config: SshHostConfig,
    pub event:      SshKeyGenerated,
}

#[derive(Debug)]
pub struct TestConnectionResult {
    pub ssh_key: SshKey,
    pub event:   SshKeyTested,
}

#[derive(Debug)]
pub struct AddToAgentResult {
    pub event: SshKeyAddedToAgent,
}

// ── Validation result types ──────────────────────────────────────────────────

/// The result of a single SSH setup validation check.
#[derive(Debug, Clone)]
pub struct SshValidationItem {
    /// Human-readable name of the check (e.g. "Key permissions").
    pub name:   &'static str,
    /// Whether the check passed.
    pub passed: bool,
    /// Detailed message describing what was checked and the outcome.
    pub detail: String,
}

/// Comprehensive result of an `ssh validate-setup` run.
#[derive(Debug, Clone)]
pub struct SshValidationResult {
    /// True if every check passed.
    pub all_passed: bool,
    /// Individual check results.
    pub checks:     Vec<SshValidationItem>,
}

// ── The service ───────────────────────────────────────────────────────────────

/// Domain service for SSH key lifecycle management.
#[derive(Debug)]
pub struct SshService {
    key_repo:        Arc<dyn SshKeyRepository>,
    host_config_repo: Arc<dyn SshHostConfigRepository>,
    ssh_ops:         Arc<dyn SshOperations>,
}

impl SshService {
    pub fn new(
        key_repo: Arc<dyn SshKeyRepository>,
        host_config_repo: Arc<dyn SshHostConfigRepository>,
        ssh_ops: Arc<dyn SshOperations>,
    ) -> Self {
        Self { key_repo, host_config_repo, ssh_ops }
    }

    /// Generates a new SSH key pair for an account and fully sets it up:
    /// deactivates the old key, writes the private/public key files, creates
    /// the database records, writes the ~/.ssh/config Host block, and
    /// optionally adds the key to the running SSH agent.
    pub async fn generate_and_setup_key(
        &self,
        account_id:      Uuid,
        account_alias:   &str,
        platform_slug:   &str,
        email:           &str,
        hostname:        &str,
        key_type:        KeyType,
        ssh_dir:         PathBuf,
        port:            u16,
        add_to_agent:    bool,
        passphrase:      Option<&str>,
        _connect_timeout_ms: u32,
    ) -> Result<GenerateAndSetupResult, SshError> {
        // Step 1: Deactivate existing keys so there is only one active at a time.
        // Deactivation is idempotent if no active key exists.
        self.key_repo.deactivate_all_for_account(account_id).await?;

        // Step 2: Derive a deterministic key filename from the account identity.
        // Format: "id_{key_type}_{platform}_{alias}" e.g. "id_ed25519_github_work"
        let key_filename  = format!("id_{}_{}_{}",
            key_type.as_keygen_type(), platform_slug, account_alias);
        let private_path  = ssh_dir.join(&key_filename);
        let host_alias    = format!("{}-{}", hostname, account_alias);

        // Step 3: Generate the key pair via the OS layer.
        let gen_result = self.ssh_ops.generate_key(
            key_type.as_keygen_type(),
            email,
            &private_path,
            passphrase,
        ).await?;

        // Step 4: Persist the SshKey entity.
        let ssh_key = SshKey::new(
            account_id,
            key_filename.clone(),
            key_type,
            gen_result.fingerprint.clone(),
            gen_result.public_key.clone(),
            private_path.to_string_lossy().to_string(),
        );
        self.key_repo.save(&ssh_key).await?;

        // Step 5: Write the ~/.ssh/config Host block atomically via the OS layer.
        self.ssh_ops.write_config_entry(
            &host_alias,
            hostname,
            &private_path,
            port,
        ).await?;

        // Step 6: Persist the SshHostConfig entity.
        let host_config = SshHostConfig::new(
            account_id,
            ssh_key.uuid(),
            host_alias.clone(),
            hostname.to_string(),
            private_path.to_string_lossy().to_string(),
            port,
        );
        self.host_config_repo.save(&host_config).await?;

        // Step 7: Optionally add the key to the SSH agent.
        if add_to_agent {
            // Non-fatal if the agent is not running — the user can add it manually.
            let _ = self.ssh_ops.add_to_agent(&private_path, passphrase).await;
        }

        let event = SshKeyGenerated {
            ssh_key_uuid: ssh_key.uuid(),
            account_id,
            fingerprint:  gen_result.fingerprint,
            public_key:   gen_result.public_key,
            key_type_str: key_type_display(&ssh_key),
            host_alias,
        };

        Ok(GenerateAndSetupResult { ssh_key, host_config, event })
    }

    /// Tests the SSH connection for an account's active key against its platform.
    /// Updates the key's test_status and returns the event for the kernel to publish.
    pub async fn test_connection(
        &self,
        account_id:  Uuid,
        timeout_ms:  u32,
    ) -> Result<TestConnectionResult, SshError> {
        let mut ssh_key = self.key_repo
            .find_active_for_account(account_id)
            .await?
            .ok_or(SshError::KeyFileNotFound {
                path: "<no active key for this account>".to_string(),
            })?;

        let host_config = self.host_config_repo
            .find_by_account(account_id)
            .await?
            .ok_or(SshError::ConfigWriteFailed {
                reason: "no SSH host config entry for this account".to_string(),
            })?;

        let key_path = PathBuf::from(ssh_key.private_key_path());

        let test_result = self.ssh_ops.test_connection(
            host_config.host_alias(),
            &key_path,
            timeout_ms,
        ).await?;

        if test_result.success {
            let username = test_result.username.clone().unwrap_or_default();
            ssh_key.record_test_success(username.clone());
        } else {
            let reason = test_result.error.clone().unwrap_or_else(|| "unknown failure".to_string());
            ssh_key.record_test_failure(reason);
        }
        self.key_repo.save(&ssh_key).await?;

        let event = SshKeyTested {
            ssh_key_uuid: ssh_key.uuid(),
            account_id,
            success:      test_result.success,
            username:     test_result.username,
            error:        test_result.error,
        };

        Ok(TestConnectionResult { ssh_key, event })
    }

    /// Run the full SSH setup validation suite for an account.
    ///
    /// Checks performed (in order):
    ///   1. Key exists in database
    ///   2. Private key file exists on disk
    ///   3. Private key permissions (0600)
    ///   4. Public key permissions (0644)
    ///   5. SSH host config entry exists in database
    ///   6. Include directive present in ~/.ssh/config
    ///   7. Managed config file exists
    ///   8. SSH agent is running
    ///   9. Key is loaded in the agent
    ///  10. Connection test against the configured host alias
    ///
    /// Every check is independent — the method collects all results and returns
    /// them together rather than short-circuiting on the first failure.
    pub async fn validate_setup(&self, account_id: Uuid, timeout_ms: u32) -> SshValidationResult {
        let mut checks = Vec::new();
        let ssh_key_path;

        // ── 1. Key exists in database ────────────────────────────────────────
        match self.key_repo.find_active_for_account(account_id).await {
            Ok(Some(key)) => {
                checks.push(SshValidationItem {
                    name: "Key record",
                    passed: true,
                    detail: format!("Active SSH key '{}' (fingerprint {}) found in database",
                        key.name(), key.fingerprint()),
                });
                ssh_key_path = PathBuf::from(key.private_key_path());
            }
            Ok(None) => {
                checks.push(SshValidationItem {
                    name: "Key record",
                    passed: false,
                    detail: "No active SSH key for this account. Run 'ssh generate' first.".to_string(),
                });
                ssh_key_path = PathBuf::new();
            }
            Err(e) => {
                checks.push(SshValidationItem {
                    name: "Key record",
                    passed: false,
                    detail: format!("Database error: {e}"),
                });
                ssh_key_path = PathBuf::new();
            }
        }

        let has_key = !ssh_key_path.as_os_str().is_empty();

        // ── 2. Private key file exists ───────────────────────────────────────
        if has_key {
            let exists = ssh_key_path.exists();
            checks.push(SshValidationItem {
                name: "Key file exists",
                passed: exists,
                detail: if exists {
                    format!("Private key found at {}", ssh_key_path.display())
                } else {
                    format!("Private key file missing at {}", ssh_key_path.display())
                },
            });
        } else {
            checks.push(SshValidationItem {
                name: "Key file exists",
                passed: false,
                detail: "Skipped — no key record.".to_string(),
            });
        }

        // ── 3. Private key permissions (0600) ────────────────────────────────
        if has_key && ssh_key_path.exists() {
            match check_permissions(&ssh_key_path, 0o600) {
                Ok(ok) => checks.push(SshValidationItem {
                    name: "Private key permissions",
                    passed: ok,
                    detail: if ok {
                        format!("{} has correct permissions (0600)", ssh_key_path.display())
                    } else {
                        format!("{} permissions are not 0600", ssh_key_path.display())
                    },
                }),
                Err(e) => checks.push(SshValidationItem {
                    name: "Private key permissions",
                    passed: false,
                    detail: format!("Cannot read permissions: {e}"),
                }),
            }

            // Public key path (same with .pub suffix)
            let pub_key_path = ssh_key_path.with_extension("pub");
            match check_permissions(&pub_key_path, 0o644) {
                Ok(ok) => checks.push(SshValidationItem {
                    name: "Public key permissions",
                    passed: ok,
                    detail: if ok {
                        format!("{} has correct permissions (0644)", pub_key_path.display())
                    } else {
                        format!("{} permissions are not 0644", pub_key_path.display())
                    },
                }),
                Err(e) => checks.push(SshValidationItem {
                    name: "Public key permissions",
                    passed: false,
                    detail: format!("Cannot read permissions: {e}"),
                }),
            }
        } else {
            checks.push(SshValidationItem {
                name: "Private key permissions",
                passed: false,
                detail: "Skipped — key file not available.".to_string(),
            });
            checks.push(SshValidationItem {
                name: "Public key permissions",
                passed: false,
                detail: "Skipped — key file not available.".to_string(),
            });
        }

        // ── 5. Host config entry exists ──────────────────────────────────────
        match self.host_config_repo.find_by_account(account_id).await {
            Ok(Some(cfg)) => {
                checks.push(SshValidationItem {
                    name: "Host config",
                    passed: true,
                    detail: format!("Host config entry '{}' for host '{}'", cfg.host_alias(), cfg.hostname()),
                });
            }
            Ok(None) => {
                checks.push(SshValidationItem {
                    name: "Host config",
                    passed: false,
                    detail: "No SSH host config entry for this account. Run 'ssh generate' first.".to_string(),
                });
            }
            Err(e) => {
                checks.push(SshValidationItem {
                    name: "Host config",
                    passed: false,
                    detail: format!("Database error: {e}"),
                });
            }
        }

        // ── 6. Managed config directory exists ───────────────────────────────
        let home = std::env::var("HOME").ok()
            .map(std::path::PathBuf::from);
        let managed_dir = home.as_ref().map(|h| h.join(".ssh").join("gitzyrix"));
        let managed_cfg = managed_dir.as_ref().map(|d| d.join("config"));

        match &managed_cfg {
            Some(path) if path.exists() => {
                checks.push(SshValidationItem {
                    name: "Managed config",
                    passed: true,
                    detail: format!("{} exists", path.display()),
                });
            }
            Some(path) => {
                checks.push(SshValidationItem {
                    name: "Managed config",
                    passed: false,
                    detail: format!("{} does not exist — run 'ssh generate' to create it", path.display()),
                });
            }
            None => {
                checks.push(SshValidationItem {
                    name: "Managed config",
                    passed: false,
                    detail: "Cannot determine HOME directory.".to_string(),
                });
            }
        }

        // ── 7. Include directive in ~/.ssh/config ────────────────────────────
        let user_cfg = home.as_ref().map(|h| h.join(".ssh").join("config"));
        match &user_cfg {
            Some(path) if path.exists() => {
                let content = std::fs::read_to_string(path).unwrap_or_default();
                let has_include = content.lines().any(|l| {
                    let t = l.trim();
                    t == "Include ~/.ssh/gitzyrix/config"
                });
                checks.push(SshValidationItem {
                    name: "Include directive",
                    passed: has_include,
                    detail: if has_include {
                        "~/.ssh/config includes ~/.ssh/gitzyrix/config".to_string()
                    } else {
                        "~/.ssh/config is missing 'Include ~/.ssh/gitzyrix/config'".to_string()
                    },
                });
            }
            Some(path) => {
                checks.push(SshValidationItem {
                    name: "Include directive",
                    passed: false,
                    detail: format!("{} does not exist", path.display()),
                });
            }
            None => {
                checks.push(SshValidationItem {
                    name: "Include directive",
                    passed: false,
                    detail: "Cannot determine HOME directory.".to_string(),
                });
            }
        }

        // ── 8. SSH agent running ─────────────────────────────────────────────
        let agent_running = self.ssh_ops.is_agent_running().await;
        checks.push(SshValidationItem {
            name: "SSH agent",
            passed: agent_running,
            detail: if agent_running {
                "SSH agent is running (SSH_AUTH_SOCK set)".to_string()
            } else {
                "SSH agent is not running — agent forwarding may fail".to_string()
            },
        });

        // ── 9. Key loaded in agent ───────────────────────────────────────────
        if has_key {
            match self.ssh_ops.check_agent_has_key(&ssh_key_path).await {
                Ok(loaded) => checks.push(SshValidationItem {
                    name: "Agent loaded",
                    passed: loaded,
                    detail: if loaded {
                        format!("Key {} is loaded in the SSH agent", ssh_key_path.display())
                    } else {
                        format!("Key {} is not loaded in the SSH agent — run 'ssh add-to-agent'", ssh_key_path.display())
                    },
                }),
                Err(e) => checks.push(SshValidationItem {
                    name: "Agent loaded",
                    passed: false,
                    detail: format!("Agent check failed: {e}"),
                }),
            }
        } else {
            checks.push(SshValidationItem {
                name: "Agent loaded",
                passed: false,
                detail: "Skipped — no key record.".to_string(),
            });
        }

        // ── 10. Connection test ──────────────────────────────────────────────
        if has_key {
            match self.host_config_repo.find_by_account(account_id).await {
                Ok(Some(cfg)) => {
                    match self.ssh_ops.test_connection(cfg.host_alias(), &ssh_key_path, timeout_ms).await {
                        Ok(r) => checks.push(SshValidationItem {
                            name: "Connection test",
                            passed: r.success,
                            detail: if r.success {
                                format!("SSH connection to '{}' succeeded{}",
                                    cfg.host_alias(),
                                    r.username.as_ref().map(|u| format!(" (greeted as {u})")).unwrap_or_default())
                            } else {
                                format!("SSH connection to '{}' failed: {}",
                                    cfg.host_alias(),
                                    r.error.as_deref().unwrap_or("unknown error"))
                            },
                        }),
                        Err(e) => checks.push(SshValidationItem {
                            name: "Connection test",
                            passed: false,
                            detail: format!("SSH connection test error: {e}"),
                        }),
                    }
                }
                Ok(None) => {
                    checks.push(SshValidationItem {
                        name: "Connection test",
                        passed: false,
                        detail: "Skipped — no host config entry.".to_string(),
                    });
                }
                Err(e) => {
                    checks.push(SshValidationItem {
                        name: "Connection test",
                        passed: false,
                        detail: format!("Database error: {e}"),
                    });
                }
            }
        } else {
            checks.push(SshValidationItem {
                name: "Connection test",
                passed: false,
                detail: "Skipped — no key record.".to_string(),
            });
        }

        let all_passed = checks.iter().all(|c| c.passed);
        SshValidationResult { all_passed, checks }
    }

    /// Adds the account's active SSH key to the running SSH agent.
    pub async fn add_to_agent(
        &self,
        account_id: Uuid,
        passphrase: Option<&str>,
    ) -> Result<AddToAgentResult, SshError> {
        let ssh_key = self.key_repo
            .find_active_for_account(account_id)
            .await?
            .ok_or(SshError::KeyFileNotFound {
                path: "<no active key>".to_string(),
            })?;

        let key_path = PathBuf::from(ssh_key.private_key_path());
        self.ssh_ops.add_to_agent(&key_path, passphrase).await?;

        let event = SshKeyAddedToAgent {
            ssh_key_uuid: ssh_key.uuid(),
            account_id,
        };
        Ok(AddToAgentResult { event })
    }

    /// Lists all SSH keys for an account, ordered newest first.
    pub async fn list_keys_for_account(&self, account_id: Uuid) -> Result<Vec<SshKey>, SshError> {
        self.key_repo.list_by_account(account_id).await
    }

    /// Retrieves the active SSH key for an account. Returns None if no key has been set up.
    pub async fn get_active_key(&self, account_id: Uuid) -> Result<Option<SshKey>, SshError> {
        self.key_repo.find_active_for_account(account_id).await
    }
}

fn key_type_display(key: &SshKey) -> String {
    key.key_type().to_string()
}

/// Check that a file has the expected Unix permissions.
#[cfg(unix)]
fn check_permissions(path: &std::path::Path, expected: u32) -> Result<bool, String> {
    use std::os::unix::fs::PermissionsExt;
    let meta = std::fs::metadata(path).map_err(|e| e.to_string())?;
    let actual = meta.permissions().mode() & 0o777;
    Ok(actual == expected)
}

#[cfg(not(unix))]
fn check_permissions(_path: &std::path::Path, _expected: u32) -> Result<bool, String> {
    // Permission checks are a no-op on non-Unix platforms.
    Ok(true)
}