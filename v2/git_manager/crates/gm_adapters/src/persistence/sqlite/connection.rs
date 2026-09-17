use sqlx::SqlitePool;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::time::Duration;
use gm_shared::errors::GitManagerError;

pub async fn create_sqlite_pool(database_path: &str) -> Result<SqlitePool, GitManagerError> {
    let opts = if database_path == ":memory:" {
        SqliteConnectOptions::new()
            .filename(":memory:")
            .create_if_missing(true)
    } else {
        SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true)
    };

    SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .connect_with(opts)
        .await
        .map_err(|e| GitManagerError::Database(format!("SQLite pool creation failed: {e}")))
}

const DDL: &[&str] = &[
    // ── platforms ──────────────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS platforms (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        uuid            TEXT NOT NULL UNIQUE,
        name            TEXT NOT NULL UNIQUE,
        display_name    TEXT NOT NULL,
        api_base_url    TEXT,
        ssh_host        TEXT,
        icon_url        TEXT,
        supports_oauth  INTEGER NOT NULL DEFAULT 0,
        supports_pat    INTEGER NOT NULL DEFAULT 1,
        supports_ssh    INTEGER NOT NULL DEFAULT 1,
        supports_https  INTEGER NOT NULL DEFAULT 1,
        is_active       INTEGER NOT NULL DEFAULT 1,
        is_self_hosted  INTEGER NOT NULL DEFAULT 0,
        rate_limit_rpm  INTEGER,
        metadata        TEXT,
        created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    )",

    // ── accounts ───────────────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS accounts (
        id               INTEGER PRIMARY KEY AUTOINCREMENT,
        uuid             TEXT NOT NULL UNIQUE,
        platform_id      INTEGER NOT NULL REFERENCES platforms(id),
        alias            TEXT NOT NULL,
        username         TEXT NOT NULL,
        email            TEXT NOT NULL,
        display_name     TEXT,
        avatar_url       TEXT,
        profile_url      TEXT,
        auth_method      TEXT NOT NULL DEFAULT 'ssh',
        ssh_host_alias   TEXT,
        status           TEXT NOT NULL DEFAULT 'active',
        is_default       INTEGER NOT NULL DEFAULT 0,
        last_used_at     TEXT,
        last_verified_at TEXT,
        metadata         TEXT,
        created_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        updated_at       TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        deleted_at       TEXT,
        UNIQUE(alias, platform_id)
    )",

    // ── ssh_keys ───────────────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS ssh_keys (
        id                   INTEGER PRIMARY KEY AUTOINCREMENT,
        uuid                 TEXT NOT NULL UNIQUE,
        account_id           INTEGER REFERENCES accounts(id),
        name                 TEXT NOT NULL,
        key_type             TEXT NOT NULL DEFAULT 'ed25519',
        key_size_bits        INTEGER,
        public_key           TEXT NOT NULL,
        fingerprint          TEXT NOT NULL UNIQUE,
        private_key_path     TEXT NOT NULL,
        comment_email        TEXT,
        passphrase_encrypted TEXT,
        passphrase_iv_hex    TEXT,
        is_added_to_agent    INTEGER NOT NULL DEFAULT 0,
        is_active            INTEGER NOT NULL DEFAULT 1,
        last_tested_at       TEXT,
        last_test_status     TEXT NOT NULL DEFAULT 'untested',
        last_test_error      TEXT,
        created_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        updated_at           TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    )",

    // ── ssh_host_configs ───────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS ssh_host_configs (
        id                      INTEGER PRIMARY KEY AUTOINCREMENT,
        uuid                    TEXT NOT NULL UNIQUE,
        account_id              INTEGER NOT NULL REFERENCES accounts(id),
        ssh_key_id              INTEGER NOT NULL REFERENCES ssh_keys(id),
        host_alias              TEXT NOT NULL UNIQUE,
        hostname                TEXT NOT NULL,
        \"user\"                TEXT NOT NULL DEFAULT 'git',
        identity_file           TEXT NOT NULL,
        port                    INTEGER NOT NULL DEFAULT 22,
        add_keys_to_agent       INTEGER NOT NULL DEFAULT 1,
        identity_only           INTEGER NOT NULL DEFAULT 1,
        strict_host_key_checking TEXT NOT NULL DEFAULT 'accept-new',
        is_synced_to_disk       INTEGER NOT NULL DEFAULT 0,
        created_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        updated_at              TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    )",

    // ── repositories ───────────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS repositories (
        id                INTEGER PRIMARY KEY AUTOINCREMENT,
        uuid              TEXT NOT NULL UNIQUE,
        account_id        INTEGER NOT NULL REFERENCES accounts(id),
        platform_id       INTEGER NOT NULL REFERENCES platforms(id),
        platform_repo_id  TEXT,
        name              TEXT NOT NULL,
        full_name         TEXT NOT NULL,
        description       TEXT,
        local_path        TEXT,
        remote_url        TEXT NOT NULL,
        clone_url_https   TEXT,
        clone_url_ssh     TEXT,
        default_branch    TEXT NOT NULL DEFAULT 'main',
        current_branch    TEXT,
        visibility        TEXT NOT NULL DEFAULT 'private',
        is_cloned         INTEGER NOT NULL DEFAULT 0,
        is_archived       INTEGER NOT NULL DEFAULT 0,
        is_forked         INTEGER NOT NULL DEFAULT 0,
        is_empty          INTEGER NOT NULL DEFAULT 0,
        is_template       INTEGER NOT NULL DEFAULT 0,
        parent_full_name  TEXT,
        primary_language  TEXT,
        topics            TEXT,
        size_kb           INTEGER,
        open_issues_count INTEGER NOT NULL DEFAULT 0,
        stargazers_count  INTEGER NOT NULL DEFAULT 0,
        forks_count       INTEGER NOT NULL DEFAULT 0,
        last_synced_at    TEXT,
        last_commit_sha   TEXT,
        last_commit_message TEXT,
        last_commit_author  TEXT,
        last_commit_at    TEXT,
        clone_depth       INTEGER,
        metadata          TEXT,
        created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        updated_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        UNIQUE(full_name, account_id)
    )",

    // ── git_operations ─────────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS git_operations (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        uuid            TEXT NOT NULL UNIQUE,
        repository_id   INTEGER NOT NULL REFERENCES repositories(id),
        account_id      INTEGER NOT NULL REFERENCES accounts(id),
        operation_type  TEXT NOT NULL,
        status          TEXT NOT NULL DEFAULT 'pending',
        triggered_by    TEXT NOT NULL DEFAULT 'cli',
        branch_source   TEXT,
        branch_target   TEXT,
        remote_name     TEXT NOT NULL DEFAULT 'origin',
        commit_sha      TEXT,
        commit_message  TEXT,
        options         TEXT,
        raw_command     TEXT,
        stdout_log      TEXT,
        stderr_log      TEXT,
        exit_code       INTEGER,
        duration_ms     INTEGER,
        error_message   TEXT,
        parent_op_id    INTEGER,
        workflow_id     TEXT,
        started_at      TEXT,
        completed_at    TEXT,
        created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    )",

    // ── events ─────────────────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS events (
        id                  INTEGER PRIMARY KEY AUTOINCREMENT,
        uuid                TEXT NOT NULL UNIQUE,
        event_type          TEXT NOT NULL,
        event_version       TEXT NOT NULL DEFAULT '1.0',
        aggregate_type      TEXT,
        aggregate_id        TEXT,
        payload             TEXT NOT NULL,
        metadata            TEXT,
        is_processed        INTEGER NOT NULL DEFAULT 0,
        processing_attempts INTEGER NOT NULL DEFAULT 0,
        first_processed_at  TEXT,
        last_error          TEXT,
        published_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    )",

    // ── audit_logs ─────────────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS audit_logs (
        id              INTEGER PRIMARY KEY AUTOINCREMENT,
        uuid            TEXT NOT NULL UNIQUE,
        action          TEXT NOT NULL,
        actor           TEXT,
        resource_type   TEXT,
        resource_id     TEXT,
        resource_name   TEXT,
        snapshot_before TEXT,
        snapshot_after  TEXT,
        changes         TEXT,
        interface       TEXT,
        ip_address      TEXT,
        session_id      TEXT,
        severity        TEXT NOT NULL DEFAULT 'info',
        correlation_id  TEXT,
        created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    )",

    // ── app_configurations ─────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS app_configurations (
        id           INTEGER PRIMARY KEY AUTOINCREMENT,
        config_key   TEXT NOT NULL UNIQUE,
        config_value TEXT,
        data_type    TEXT NOT NULL DEFAULT 'string',
        description  TEXT,
        is_secret    INTEGER NOT NULL DEFAULT 0,
        is_readonly  INTEGER NOT NULL DEFAULT 0,
        namespace    TEXT NOT NULL DEFAULT 'app',
        created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
        updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    )",

    // ── sync_sessions ──────────────────────────────────────────────────────
    "CREATE TABLE IF NOT EXISTS sync_sessions (
        id                INTEGER PRIMARY KEY AUTOINCREMENT,
        uuid              TEXT NOT NULL UNIQUE,
        repository_id     INTEGER NOT NULL REFERENCES repositories(id),
        account_id        INTEGER NOT NULL REFERENCES accounts(id),
        session_type      TEXT NOT NULL,
        status            TEXT NOT NULL DEFAULT 'pending',
        branch_name       TEXT NOT NULL,
        remote_name       TEXT NOT NULL DEFAULT 'origin',
        commit_message    TEXT,
        commit_sha_before TEXT,
        commit_sha_after  TEXT,
        files_staged      INTEGER NOT NULL DEFAULT 0,
        files_added       INTEGER NOT NULL DEFAULT 0,
        files_modified    INTEGER NOT NULL DEFAULT 0,
        files_deleted     INTEGER NOT NULL DEFAULT 0,
        files_renamed     INTEGER NOT NULL DEFAULT 0,
        commits_pushed    INTEGER NOT NULL DEFAULT 0,
        commits_pulled    INTEGER NOT NULL DEFAULT 0,
        commits_ahead     INTEGER NOT NULL DEFAULT 0,
        commits_behind    INTEGER NOT NULL DEFAULT 0,
        conflicts_count   INTEGER NOT NULL DEFAULT 0,
        conflict_resolution TEXT NOT NULL DEFAULT 'none',
        conflict_files    TEXT,
        options           TEXT,
        error_message     TEXT,
        started_at        TEXT,
        completed_at      TEXT,
        created_at        TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
    )",
];

const SEED_PLATFORMS: &str = "INSERT OR IGNORE INTO platforms (uuid, name, display_name, api_base_url, ssh_host, supports_oauth, supports_pat, supports_ssh) VALUES
    ('00000000-0001-0000-0000-000000000001', 'github',        'GitHub',             'https://api.github.com',          'github.com',         1, 1, 1),
    ('00000000-0002-0000-0000-000000000001', 'gitlab',        'GitLab',             'https://gitlab.com/api/v4',       'gitlab.com',         1, 1, 1),
    ('00000000-0003-0000-0000-000000000001', 'bitbucket',     'Bitbucket',          'https://api.bitbucket.org/2.0',   'bitbucket.org',      1, 1, 1),
    ('00000000-0004-0000-0000-000000000001', 'azure_devops',  'Azure DevOps',       'https://dev.azure.com',           'ssh.dev.azure.com',  0, 1, 1),
    ('00000000-0005-0000-0000-000000000001', 'sourceforge',   'SourceForge',        'https://sourceforge.net/rest',    'git.code.sf.net',    0, 0, 1),
    ('00000000-0006-0000-0000-000000000001', 'self_hosted',   'Self-Hosted',        NULL,                              NULL,                 0, 1, 1),
    ('00000000-0007-0000-0000-000000000001', 'cloud_storage', 'Cloud Storage',      NULL,                              NULL,                 0, 0, 0),
    ('00000000-0008-0000-0000-000000000001', 'local_path',    'Local Path',         NULL,                              NULL,                 0, 0, 0),
    ('00000000-0009-0000-0000-000000000001', 'custom',        'Custom',             NULL,                              NULL,                 0, 1, 1)";

const SEED_CONFIG: &str = "INSERT OR IGNORE INTO app_configurations (config_key, config_value, data_type, description, namespace) VALUES
    ('ssh.default_key_type',     '\"ed25519\"',   'string',  'Default SSH key type for new key generation',    'app'),
    ('ssh.connect_timeout_ms',   '10000',       'integer', 'SSH connection test timeout in milliseconds',    'app'),
    ('ssh.auto_add_to_agent',    'true',        'boolean', 'Automatically add new SSH keys to the agent',   'app'),
    ('git.max_concurrent_ops',   '4',           'integer', 'Maximum concurrent git operations',              'app'),
    ('ui.show_banner',           'true',        'boolean', 'Show the startup banner',                       'app'),
    ('ui.log_level',             '\"info\"',     'string',  'Application log level (trace/debug/info/warn/error)', 'app')";

/// Creates all tables, indexes, and seeds initial data for the SQLite database.
/// Safe to call repeatedly — uses IF NOT EXISTS / INSERT OR IGNORE.
pub async fn setup_sqlite_schema(pool: &SqlitePool) -> Result<(), GitManagerError> {
    for ddl in DDL {
        sqlx::query(ddl)
            .execute(pool)
            .await
            .map_err(|e| GitManagerError::Database(format!("SQLite DDL failed: {e}")))?;
    }

    sqlx::query(SEED_PLATFORMS)
        .execute(pool)
        .await
        .map_err(|e| GitManagerError::Database(format!("SQLite seed platforms failed: {e}")))?;

    sqlx::query(SEED_CONFIG)
        .execute(pool)
        .await
        .map_err(|e| GitManagerError::Database(format!("SQLite seed config failed: {e}")))?;

    tracing::info!("SQLite schema created and seeded");
    Ok(())
}
