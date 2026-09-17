// crates/gm_domain/src/ssh/entities/ssh_host_config.rs
//
// The SshHostConfig entity represents a single `Host` block that Git Manager
// writes into the user's ~/.ssh/config file. Each account that uses SSH
// authentication gets one host config entry. The entry ties a human-readable
// host alias (e.g. "github.com-work") to the real hostname, the identity file,
// and the port — allowing git commands to use the alias as the remote host
// and have OpenSSH automatically select the correct key.
//
// Example ~/.ssh/config entry produced from this entity:
//
//   Host github.com-work
//     HostName github.com
//     User git
//     IdentityFile ~/.ssh/id_ed25519_work
//     IdentitiesOnly yes
//     Port 22
//
// The `IdentitiesOnly yes` directive is critical — it prevents the SSH agent
// from offering other keys to the server, ensuring per-account identity isolation.

use chrono::{DateTime, Utc};
use uuid::Uuid;

/// An SSH config Host block managed by Git Manager.
#[derive(Debug, Clone)]
pub struct SshHostConfig {
    uuid:              Uuid,
    account_id:        Uuid,
    ssh_key_id:        Uuid,
    /// The alias used in git remote URLs: "github.com-work"
    host_alias:        String,
    /// The real SSH server hostname: "github.com"
    hostname:          String,
    /// The SSH username. Always "git" for Git hosting platforms.
    username:          String,
    /// Absolute path to the private key file: "/home/shaka/.ssh/id_ed25519_work"
    identity_file:     String,
    port:              u16,
    /// If true, OpenSSH will only offer the specified identity file and will
    /// not attempt other keys from the agent. Always true for Git Manager entries.
    identities_only:   bool,
    is_active:         bool,
    created_at:        DateTime<Utc>,
}

impl SshHostConfig {
    /// Constructs a new host config entry. `identities_only` is always set to
    /// true — Git Manager enforces per-account key isolation.
    pub fn new(
        account_id:    Uuid,
        ssh_key_id:    Uuid,
        host_alias:    String,
        hostname:      String,
        identity_file: String,
        port:          u16,
    ) -> Self {
        Self {
            uuid:           Uuid::new_v4(),
            account_id,
            ssh_key_id,
            host_alias,
            hostname,
            username:       "git".to_string(),
            identity_file,
            port,
            identities_only: true,
            is_active:      true,
            created_at:     Utc::now(),
        }
    }

    /// Rehydrates from the database without re-running construction logic.
    pub fn rehydrate(
        uuid:            Uuid,
        account_id:      Uuid,
        ssh_key_id:      Uuid,
        host_alias:      String,
        hostname:        String,
        username:        String,
        identity_file:   String,
        port:            u16,
        identities_only: bool,
        is_active:       bool,
        created_at:      DateTime<Utc>,
    ) -> Self {
        Self { uuid, account_id, ssh_key_id, host_alias, hostname, username,
               identity_file, port, identities_only, is_active, created_at }
    }

    /// Renders this entity as a properly-formatted ~/.ssh/config Host block string.
    /// This is the string the Zig layer appends to the config file atomically.
    pub fn render_config_block(&self) -> String {
        let mut block = format!("Host {}\n", self.host_alias);
        block.push_str(&format!("  HostName {}\n", self.hostname));
        block.push_str(&format!("  User {}\n", self.username));
        block.push_str(&format!("  IdentityFile {}\n", self.identity_file));
        if self.identities_only {
            block.push_str("  IdentitiesOnly yes\n");
        }
        if self.port != 22 {
            block.push_str(&format!("  Port {}\n", self.port));
        }
        block
    }

    pub fn deactivate(&mut self) { self.is_active = false; }

    pub fn uuid(&self)             -> Uuid            { self.uuid }
    pub fn account_id(&self)       -> Uuid            { self.account_id }
    pub fn ssh_key_id(&self)       -> Uuid            { self.ssh_key_id }
    pub fn host_alias(&self)       -> &str            { &self.host_alias }
    pub fn hostname(&self)         -> &str            { &self.hostname }
    pub fn username(&self)         -> &str            { &self.username }
    pub fn identity_file(&self)    -> &str            { &self.identity_file }
    pub fn port(&self)             -> u16             { self.port }
    pub fn identities_only(&self)  -> bool            { self.identities_only }
    pub fn is_active(&self)        -> bool            { self.is_active }
    pub fn created_at(&self)       -> DateTime<Utc>   { self.created_at }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_config_block_includes_identities_only() {
        let cfg = SshHostConfig::new(
            Uuid::new_v4(), Uuid::new_v4(),
            "github.com-work".to_string(),
            "github.com".to_string(),
            "~/.ssh/id_ed25519_work".to_string(),
            22,
        );
        let block = cfg.render_config_block();
        assert!(block.contains("Host github.com-work"));
        assert!(block.contains("HostName github.com"));
        assert!(block.contains("IdentitiesOnly yes"));
        // Port 22 is default and should be omitted
        assert!(!block.contains("Port 22"));
    }

    #[test]
    fn render_config_block_includes_non_default_port() {
        let cfg = SshHostConfig::new(
            Uuid::new_v4(), Uuid::new_v4(),
            "github.com-work".to_string(),
            "github.com".to_string(),
            "~/.ssh/id_ed25519_work".to_string(),
            443,
        );
        assert!(cfg.render_config_block().contains("Port 443"));
    }
}