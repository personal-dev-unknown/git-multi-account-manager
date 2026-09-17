#!/usr/bin/env bash
# populate_domain_mods.sh
#
# Run from the workspace root (the directory that contains Cargo.toml and crates/).
# This script writes the correct content into every empty mod.rs in gm_domain.
# It is idempotent — running it twice produces the same result.
#
# Usage:
#   cd ~/Documents/zanabuni/git-multi-account-manager/v2/git_manager
#   bash populate_domain_mods.sh

set -euo pipefail

DOMAIN="crates/gm_domain/src"

echo "Populating gm_domain mod.rs files from: $(pwd)"

# ── configuration ─────────────────────────────────────────────────────────────

cat > "${DOMAIN}/configuration/mod.rs" << 'RUST'
pub mod entities;
pub mod ports;
pub mod services;
RUST

cat > "${DOMAIN}/configuration/entities/mod.rs" << 'RUST'
pub mod configuration;
pub use configuration::Configuration;
RUST

cat > "${DOMAIN}/configuration/ports/mod.rs" << 'RUST'
pub mod config_repository;
pub use config_repository::ConfigRepository;
RUST

cat > "${DOMAIN}/configuration/services/mod.rs" << 'RUST'
pub mod config_service;
pub use config_service::ConfigService;
RUST

echo "  ✓ configuration"

# ── git ───────────────────────────────────────────────────────────────────────

cat > "${DOMAIN}/git/mod.rs" << 'RUST'
pub mod entities;
pub mod events;
pub mod ports;
pub mod services;
RUST

cat > "${DOMAIN}/git/entities/mod.rs" << 'RUST'
pub mod branch;
pub mod commit;
pub use branch::Branch;
pub use commit::Commit;
RUST

cat > "${DOMAIN}/git/events/mod.rs" << 'RUST'
pub mod branch_created;
pub mod commit_created;
pub mod merge_conflict_detected;
pub use branch_created::BranchCreated;
pub use commit_created::CommitCreated;
pub use merge_conflict_detected::MergeConflictDetected;
RUST

cat > "${DOMAIN}/git/ports/mod.rs" << 'RUST'
pub mod git_executor;
pub use git_executor::{
    GitExecutor,
    CloneOptions,
    CloneResult,
    CommitOptions,
    CommitResult,
    GitStatus,
    PullOptions,
    PullResult,
    PushOptions,
    PushResult,
    StatusEntry,
};
RUST

cat > "${DOMAIN}/git/services/mod.rs" << 'RUST'
pub mod conflict_resolver;
pub mod git_service;
pub use conflict_resolver::ConflictResolver;
pub use git_service::GitService;
RUST

echo "  ✓ git"

# ── ssh ───────────────────────────────────────────────────────────────────────

cat > "${DOMAIN}/ssh/mod.rs" << 'RUST'
pub mod entities;
pub mod events;
pub mod ports;
pub mod services;
pub mod value_objects;
RUST

cat > "${DOMAIN}/ssh/entities/mod.rs" << 'RUST'
pub mod ssh_host_config;
pub mod ssh_key;
pub use ssh_host_config::SshHostConfig;
pub use ssh_key::SshKey;
RUST

cat > "${DOMAIN}/ssh/events/mod.rs" << 'RUST'
pub mod ssh_key_added_to_agent;
pub mod ssh_key_generated;
pub mod ssh_key_tested;
pub use ssh_key_added_to_agent::SshKeyAddedToAgent;
pub use ssh_key_generated::SshKeyGenerated;
pub use ssh_key_tested::SshKeyTested;
RUST

cat > "${DOMAIN}/ssh/ports/mod.rs" << 'RUST'
pub mod ssh_host_config_repository;
pub mod ssh_key_repository;
pub use ssh_host_config_repository::SshHostConfigRepository;
pub use ssh_key_repository::SshKeyRepository;
RUST

cat > "${DOMAIN}/ssh/services/mod.rs" << 'RUST'
pub mod ssh_service;
pub use ssh_service::SshService;
RUST

cat > "${DOMAIN}/ssh/value_objects/mod.rs" << 'RUST'
pub mod key_type;
pub mod test_status;
pub use key_type::KeyType;
pub use test_status::TestStatus;
RUST

echo "  ✓ ssh"

# ── sync ──────────────────────────────────────────────────────────────────────

cat > "${DOMAIN}/sync/mod.rs" << 'RUST'
pub mod entities;
pub mod events;
pub mod ports;
pub mod services;
RUST

cat > "${DOMAIN}/sync/entities/mod.rs" << 'RUST'
pub mod sync_session;
pub use sync_session::SyncSession;
RUST

cat > "${DOMAIN}/sync/events/mod.rs" << 'RUST'
pub mod conflict_detected;
pub mod sync_completed;
pub mod sync_started;
pub use conflict_detected::ConflictDetected;
pub use sync_completed::SyncCompleted;
pub use sync_started::SyncStarted;
RUST

cat > "${DOMAIN}/sync/ports/mod.rs" << 'RUST'
pub mod sync_session_repository;
pub use sync_session_repository::SyncSessionRepository;
RUST

cat > "${DOMAIN}/sync/services/mod.rs" << 'RUST'
pub mod pull_service;
pub mod push_service;
pub mod sync_service;
pub use pull_service::PullService;
pub use push_service::PushService;
pub use sync_service::SyncService;
RUST

echo "  ✓ sync"

# ── accounts (should already be correct, but enforce for safety) ──────────────

cat > "${DOMAIN}/accounts/entities/mod.rs" << 'RUST'
pub mod account;
pub mod credential;
pub use account::Account;
pub use credential::Credential;
RUST

cat > "${DOMAIN}/accounts/events/mod.rs" << 'RUST'
pub mod account_added;
pub mod account_removed;
pub mod account_status_changed;
pub use account_added::AccountAdded;
pub use account_removed::AccountRemoved;
pub use account_status_changed::AccountStatusChanged;
RUST

cat > "${DOMAIN}/accounts/ports/mod.rs" << 'RUST'
pub mod account_repository;
pub use account_repository::AccountRepository;
RUST

cat > "${DOMAIN}/accounts/services/mod.rs" << 'RUST'
pub mod account_service;
pub use account_service::AccountService;
RUST

cat > "${DOMAIN}/accounts/value_objects/mod.rs" << 'RUST'
pub mod account_status;
pub mod auth_method;
pub mod platform_type;
pub use account_status::AccountStatus;
pub use auth_method::AuthMethod;
pub use platform_type::PlatformType;
RUST

echo "  ✓ accounts"

# ── repositories ──────────────────────────────────────────────────────────────

cat > "${DOMAIN}/repositories/entities/mod.rs" << 'RUST'
pub mod repository;
pub use repository::Repository;
RUST

cat > "${DOMAIN}/repositories/events/mod.rs" << 'RUST'
pub mod repository_cloned;
pub mod repository_discovered;
pub mod repository_synced;
pub use repository_cloned::RepositoryCloned;
pub use repository_discovered::RepositoryDiscovered;
pub use repository_synced::RepositorySynced;
RUST

cat > "${DOMAIN}/repositories/ports/mod.rs" << 'RUST'
pub mod repository_repository;
pub use repository_repository::RepositoryRepository;
RUST

cat > "${DOMAIN}/repositories/services/mod.rs" << 'RUST'
pub mod repository_service;
pub use repository_service::RepositoryService;
RUST

cat > "${DOMAIN}/repositories/value_objects/mod.rs" << 'RUST'
pub mod clone_status;
pub mod repository_url;
pub use clone_status::CloneStatus;
pub use repository_url::RepositoryUrl;
RUST

echo "  ✓ repositories"

echo ""
echo "All mod.rs files populated."
echo "Verify with: cargo check -p gm_domain"