// crates/gm_domain/src/ssh/entities/ssh_key.rs
//
// The SshKey entity represents a single SSH key pair managed by Git Manager.
// It stores metadata about the key (type, fingerprint, file path, test status)
// but never stores the private key bytes themselves — those live on the
// filesystem at private_key_path with mode 0600, enforced by the Zig layer.
//
// One SshKey belongs to one Account. An account may have at most one active
// key at a time (enforced by the SshService), though old keys are soft-deleted
// and kept for audit purposes rather than permanently removed.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::ssh::value_objects::{KeyType, TestStatus};
use gm_shared::models::ssh_key::SshKeyDto;

/// An SSH key pair tracked by Git Manager.
/// The private key lives at `private_key_path` on the filesystem (never in the DB).
/// All other fields are metadata.
#[derive(Debug, Clone)]
pub struct SshKey {
    uuid:             Uuid,
    account_id:       Uuid,
    /// A human-readable name: "work-github-ed25519" or auto-generated.
    name:             String,
    key_type:         KeyType,
    /// SHA-256 fingerprint from `ssh-keygen -l -E sha256 -f key.pub`.
    /// Format: "SHA256:xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".
    /// Used as a short identifier in the UI instead of the full public key.
    fingerprint:      String,
    /// The full public key string: "ssh-ed25519 AAAA... comment"
    public_key:       String,
    /// Absolute filesystem path to the private key file. The file itself is
    /// not stored in the database — only the path is.
    private_key_path: String,
    test_status:      TestStatus,
    is_active:        bool,
    created_at:       DateTime<Utc>,
}

impl SshKey {
    /// Creates a new SshKey from the material returned by the SSH keygen operation.
    /// Called by SshService after the Zig layer generates the key pair.
    pub fn new(
        account_id:       Uuid,
        name:             String,
        key_type:         KeyType,
        fingerprint:      String,
        public_key:       String,
        private_key_path: String,
    ) -> Self {
        Self {
            uuid:             Uuid::new_v4(),
            account_id,
            name,
            key_type,
            fingerprint,
            public_key,
            private_key_path,
            test_status:      TestStatus::Untested,
            is_active:        true,
            created_at:       Utc::now(),
        }
    }

    /// Rehydrates from a database row without re-running construction logic.
    pub fn rehydrate(
        uuid:             Uuid,
        account_id:       Uuid,
        name:             String,
        key_type:         KeyType,
        fingerprint:      String,
        public_key:       String,
        private_key_path: String,
        test_status:      TestStatus,
        is_active:        bool,
        created_at:       DateTime<Utc>,
    ) -> Self {
        Self { uuid, account_id, name, key_type, fingerprint, public_key,
               private_key_path, test_status, is_active, created_at }
    }

    // ── Domain operations ─────────────────────────────────────────────────────

    /// Records a successful connection test result, including the platform-confirmed username.
    pub fn record_test_success(&mut self, username: String) {
        self.test_status = TestStatus::Success { username, tested_at: Utc::now() };
    }

    /// Records a failed connection test with the reason for failure.
    pub fn record_test_failure(&mut self, reason: String) {
        self.test_status = TestStatus::Failed { reason, tested_at: Utc::now() };
    }

    /// Deactivates the key. Active keys are the ones used for git operations.
    /// Deactivated keys are retained for audit history but not used.
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    // ── Getters ───────────────────────────────────────────────────────────────
    pub fn uuid(&self)              -> Uuid             { self.uuid }
    pub fn account_id(&self)        -> Uuid             { self.account_id }
    pub fn name(&self)              -> &str             { &self.name }
    pub fn key_type(&self)          -> &KeyType         { &self.key_type }
    pub fn fingerprint(&self)       -> &str             { &self.fingerprint }
    pub fn public_key(&self)        -> &str             { &self.public_key }
    pub fn private_key_path(&self)  -> &str             { &self.private_key_path }
    pub fn test_status(&self)       -> &TestStatus      { &self.test_status }
    pub fn is_active(&self)         -> bool             { self.is_active }
    pub fn created_at(&self)        -> DateTime<Utc>    { self.created_at }

    pub fn to_dto(&self) -> SshKeyDto {
        SshKeyDto {
            uuid:              self.uuid,
            account_id:        Some(self.account_id),
            name:              self.name.clone(),
            key_type:          self.key_type.to_shared(),
            key_size_bits:     self.key_type.rsa_bits(),
            public_key:        self.public_key.clone(),
            fingerprint:       self.fingerprint.clone(),
            private_key_path:  self.private_key_path.clone(),
            comment_email:     None,
            is_added_to_agent: false,
            is_active:         self.is_active,
            last_tested_at:    self.test_status.last_tested_at(),
            last_test_status:  self.test_status.to_shared(),
            last_test_error:   match &self.test_status {
                crate::ssh::value_objects::TestStatus::Failed { reason, .. } => Some(reason.clone()),
                _ => None,
            },
            created_at:        self.created_at,
        }
    }
}