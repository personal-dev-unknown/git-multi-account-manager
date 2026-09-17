// crates/gm_adapters/src/ssh/managed_config.rs
//
// Managed SSH configuration — V1-compatible approach:
//
//   ~/.ssh/gitzyrix/config   ← app writes Host blocks here
//   ~/.ssh/config               ← app ensures Include directive is present
//
// Benefits over writing directly to ~/.ssh/config:
//   - Clean separation of user-managed and app-managed entries
//   - App-managed section is removable without touching user entries
//   - Idempotent Include directive
//   - Atomic backups via the directory structure

use std::io::{BufRead, Write};
use std::path::PathBuf;
use gm_shared::errors::SshError;

/// Manages the app-dedicated SSH configuration directory and file.
pub struct ManagedSshConfig;

impl ManagedSshConfig {
    /// Resolve the path to `~/.ssh/gitzyrix/config`.
    fn managed_config_path() -> Result<PathBuf, SshError> {
        let home = std::env::var("HOME")
            .map_err(|_| SshError::ConfigWriteFailed {
                reason: "HOME environment variable not set".to_string(),
            })?;
        Ok(PathBuf::from(home).join(".ssh").join("gitzyrix").join("config"))
    }

    /// Resolve the path to `~/.ssh/config`.
    fn user_config_path() -> Result<PathBuf, SshError> {
        let home = std::env::var("HOME")
            .map_err(|_| SshError::ConfigWriteFailed {
                reason: "HOME environment variable not set".to_string(),
            })?;
        Ok(PathBuf::from(home).join(".ssh").join("config"))
    }

    /// Ensure `~/.ssh/gitzyrix/` directory exists with correct permissions.
    pub fn ensure_directory() -> Result<PathBuf, SshError> {
        let path = Self::managed_config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SshError::ConfigWriteFailed {
                    reason: format!("cannot create {}: {e}", parent.display()),
                })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(meta) = std::fs::metadata(parent) {
                    let perms = meta.permissions();
                    if perms.mode() & 0o777 != 0o700 {
                        std::fs::set_permissions(parent, std::fs::Permissions::from_mode(0o700))
                            .ok();
                    }
                }
            }
        }
        Ok(path)
    }

    /// Ensure `Include ~/.ssh/gitzyrix/config` is present in `~/.ssh/config`.
    ///
    /// - If `~/.ssh/config` does not exist, it is created (0600 permissions)
    ///   with the Include line.
    /// - If the Include line is already present, no change is made.
    /// - The directive is placed at the top of the file (before any Host blocks),
    ///   matching V1 behaviour.
    pub fn ensure_include_directive() -> Result<(), SshError> {
        let path = Self::user_config_path()?;
        let include_line = "Include ~/.ssh/gitzyrix/config";

        // If the file doesn't exist, create it with the Include line.
        if !path.exists() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| SshError::ConfigWriteFailed {
                        reason: format!("cannot create {}: {e}", parent.display()),
                    })?;
            }
            let mut file = std::fs::File::create(&path)
                .map_err(|e| SshError::ConfigWriteFailed {
                    reason: format!("cannot create {}: {e}", path.display()),
                })?;
            writeln!(file, "{include_line}")
                .map_err(|e| SshError::ConfigWriteFailed {
                    reason: format!("cannot write {}: {e}", path.display()),
                })?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).ok();
            }
            return Ok(());
        }

        // Read existing lines, check if Include is present.
        let file = std::fs::File::open(&path)
            .map_err(|e| SshError::ConfigWriteFailed {
                reason: format!("cannot read {}: {e}", path.display()),
            })?;
        let reader = std::io::BufReader::new(&file);
        for line in reader.lines() {
            if let Ok(l) = line {
                if l.trim() == include_line {
                    return Ok(()); // Already present — idempotent.
                }
            }
        }

        // Prepend the Include directive to the file.
        let content = std::fs::read_to_string(&path)
            .map_err(|e| SshError::ConfigWriteFailed {
                reason: format!("cannot read {}: {e}", path.display()),
            })?;
        let mut file = std::fs::File::create(&path)
            .map_err(|e| SshError::ConfigWriteFailed {
                reason: format!("cannot write {}: {e}", path.display()),
            })?;
        writeln!(file, "{include_line}")
            .map_err(|e| SshError::ConfigWriteFailed {
                reason: format!("cannot write {}: {e}", path.display()),
            })?;
        write!(file, "{content}")
            .map_err(|e| SshError::ConfigWriteFailed {
                reason: format!("cannot write {}: {e}", path.display()),
            })?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).ok();
        }

        Ok(())
    }

    /// Write or update a Host block in the managed config file.
    ///
    /// If a Host block for the same alias already exists, it is replaced.
    /// The new block is appended at the end of the file.
    pub fn write_host_entry(
        host_alias: &str,
        hostname: &str,
        identity_file: &str,
        port: u16,
        comment: Option<&str>,
    ) -> Result<(), SshError> {
        let path = Self::ensure_directory()?;

        // Build the new Host block.
        let mut block = String::new();
        if let Some(c) = comment {
            block.push_str(&format!("# {c}\n"));
        }
        block.push_str(&format!("Host {host_alias}\n"));
        block.push_str(&format!("  HostName {hostname}\n"));
        block.push_str("  User git\n");
        block.push_str(&format!("  IdentityFile {identity_file}\n"));
        block.push_str("  IdentitiesOnly yes\n");
        if port != 22 {
            block.push_str(&format!("  Port {port}\n"));
        }
        block.push('\n');

        // Read existing config.
        let existing = if path.exists() {
            std::fs::read_to_string(&path)
                .map_err(|e| SshError::ConfigWriteFailed {
                    reason: format!("cannot read {}: {e}", path.display()),
                })?
        } else {
            String::new()
        };

        // Remove any existing block for the same alias.
        let updated = Self::remove_host_block(&existing, host_alias);

        // Append the new block.
        let final_content = format!("{updated}{block}");

        // Write atomically via temp file + rename.
        let tmp_path = path.with_extension("tmp");
        {
            let mut tmp = std::fs::File::create(&tmp_path)
                .map_err(|e| SshError::ConfigWriteFailed {
                    reason: format!("cannot create temp file: {e}"),
                })?;
            write!(tmp, "{final_content}")
                .map_err(|e| SshError::ConfigWriteFailed {
                    reason: format!("cannot write temp file: {e}"),
                })?;
        }
        std::fs::rename(&tmp_path, &path)
            .map_err(|e| SshError::ConfigWriteFailed {
                reason: format!("cannot rename temp file -> {}: {e}", path.display()),
            })?;

        // Set permissions to 600 on the managed config. This matches V1 behaviour.
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&path) {
                if meta.permissions().mode() & 0o777 != 0o600 {
                    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).ok();
                }
            }
        }

        Ok(())
    }

    /// Remove a Host block from the managed config file.
    pub fn remove_host_entry(host_alias: &str) -> Result<(), SshError> {
        let path = Self::managed_config_path()?;
        if !path.exists() {
            return Ok(()); // Nothing to remove.
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|e| SshError::ConfigWriteFailed {
                reason: format!("cannot read {}: {e}", path.display()),
            })?;

        let updated = Self::remove_host_block(&content, host_alias);

        if updated == content {
            return Ok(()); // No change — idempotent.
        }

        std::fs::write(&path, &updated)
            .map_err(|e| SshError::ConfigWriteFailed {
                reason: format!("cannot write {}: {e}", path.display()),
            })?;

        Ok(())
    }

    // ── Internal helpers ────────────────────────────────────────────────────

    /// Remove a Host block from the given config text. Returns the cleaned text.
    fn remove_host_block(config: &str, host_alias: &str) -> String {
        let mut result = String::new();
        let mut skip = false;
        let target = format!("Host {host_alias}");

        for line in config.lines() {
            let trimmed = line.trim();

            if !skip && trimmed == target {
                skip = true;
                continue;
            }

            if skip {
                // A Host block continues until the next Host line or EOF.
                // Comment/blank lines before the next Host block are part of this block.
                if trimmed.starts_with("Host ") {
                    skip = false; // fall through to output this line
                } else {
                    continue;
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        result
    }
}
