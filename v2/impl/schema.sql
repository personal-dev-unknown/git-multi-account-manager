-- ============================================================
-- Git Multi-Account Manager — Production MySQL Schema
-- Architecture: Microkernel + Hexagonal Core
-- Engine: InnoDB | Charset: utf8mb4 | Collation: utf8mb4_unicode_ci
-- ============================================================

SET FOREIGN_KEY_CHECKS = 0;
SET SQL_MODE = 'STRICT_TRANS_TABLES,NO_ZERO_IN_DATE,ERROR_FOR_DIVISION_BY_ZERO';

-- ============================================================
-- DOMAIN: PLATFORM REGISTRY
-- The supported Git hosting platforms (GitHub, GitLab, etc.)
-- These are seeded records; plugins register themselves here.
-- ============================================================

CREATE TABLE platforms (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    uuid            CHAR(36)            NOT NULL,
    name            VARCHAR(50)         NOT NULL COMMENT 'Internal slug: github, gitlab, bitbucket, azure_devops, custom',
    display_name    VARCHAR(100)        NOT NULL,
    api_base_url    VARCHAR(500)        NULL     COMMENT 'REST API root URL for the platform',
    ssh_host        VARCHAR(255)        NULL     COMMENT 'Default SSH hostname e.g. github.com',
    icon_url        VARCHAR(500)        NULL,
    supports_oauth  TINYINT(1)          NOT NULL DEFAULT 0,
    supports_pat    TINYINT(1)          NOT NULL DEFAULT 1,
    supports_ssh    TINYINT(1)          NOT NULL DEFAULT 1,
    supports_https  TINYINT(1)          NOT NULL DEFAULT 1,
    is_active       TINYINT(1)          NOT NULL DEFAULT 1,
    is_self_hosted  TINYINT(1)          NOT NULL DEFAULT 0 COMMENT 'For custom/enterprise instances',
    rate_limit_rpm  INT UNSIGNED        NULL     COMMENT 'Requests per minute limit for the platform',
    metadata        JSON                NULL     COMMENT 'Platform-specific settings as JSON',
    created_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_platforms_uuid     (uuid),
    UNIQUE KEY uq_platforms_name     (name)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Registered Git hosting platforms. Seeded by provider plugins.';

-- ============================================================
-- DOMAIN: ACCOUNTS
-- A single Git hosting account per row, linked to a platform.
-- One physical person may have many accounts across platforms.
-- ============================================================

CREATE TABLE accounts (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    uuid            CHAR(36)            NOT NULL,
    platform_id     BIGINT UNSIGNED     NOT NULL,
    alias           VARCHAR(100)        NOT NULL COMMENT 'Human-readable name: work, personal, freelance',
    username        VARCHAR(150)        NOT NULL COMMENT 'Git platform username',
    email           VARCHAR(255)        NOT NULL,
    display_name    VARCHAR(255)        NULL,
    avatar_url      VARCHAR(500)        NULL,
    profile_url     VARCHAR(500)        NULL,
    auth_method     ENUM(
                        'ssh',
                        'https_pat',
                        'https_password',
                        'oauth',
                        'anonymous'
                    )                   NOT NULL DEFAULT 'ssh',
    ssh_host_alias  VARCHAR(255)        NULL     COMMENT 'Custom SSH host alias e.g. github.com-work in ~/.ssh/config',
    status          ENUM(
                        'active',
                        'inactive',
                        'suspended',
                        'token_expired',
                        'unverified'
                    )                   NOT NULL DEFAULT 'active',
    is_default      TINYINT(1)          NOT NULL DEFAULT 0,
    last_used_at    TIMESTAMP           NULL,
    last_verified_at TIMESTAMP          NULL,
    metadata        JSON                NULL,
    created_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_accounts_uuid         (uuid),
    UNIQUE KEY uq_accounts_alias_platform (alias, platform_id),
    KEY         idx_accounts_platform   (platform_id),
    KEY         idx_accounts_status     (status),
    KEY         idx_accounts_email      (email),

    CONSTRAINT fk_accounts_platform
        FOREIGN KEY (platform_id) REFERENCES platforms (id)
        ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Git hosting accounts. One row per account-platform pairing.';

-- ============================================================
-- DOMAIN: CREDENTIALS
-- Encrypted secrets associated with accounts (PATs, OAuth
-- tokens, HTTPS passwords). Never store plaintext here.
-- ============================================================

CREATE TABLE credentials (
    id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    uuid                CHAR(36)        NOT NULL,
    account_id          BIGINT UNSIGNED NOT NULL,
    credential_type     ENUM(
                            'pat',
                            'oauth_access_token',
                            'oauth_refresh_token',
                            'https_password',
                            'ssh_passphrase'
                        )               NOT NULL,
    encrypted_value     TEXT            NOT NULL COMMENT 'AES-256-GCM encrypted secret',
    encryption_key_id   VARCHAR(100)    NOT NULL COMMENT 'Key ID used for encryption (for rotation)',
    iv_hex              VARCHAR(64)     NOT NULL COMMENT 'Initialization vector for AES-GCM',
    scopes              JSON            NULL     COMMENT 'OAuth scopes granted e.g. ["repo", "read:org"]',
    expires_at          TIMESTAMP       NULL     COMMENT 'NULL means non-expiring',
    is_active           TINYINT(1)      NOT NULL DEFAULT 1,
    last_rotated_at     TIMESTAMP       NULL,
    rotation_count      INT UNSIGNED    NOT NULL DEFAULT 0,
    created_at          TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_credentials_uuid        (uuid),
    KEY         idx_credentials_account   (account_id),
    KEY         idx_credentials_expires   (expires_at),

    CONSTRAINT fk_credentials_account
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Encrypted credentials. One account may have multiple (e.g. access + refresh token).';

-- ============================================================
-- DOMAIN: SSH KEYS
-- ED25519/RSA SSH keys generated and managed by the system.
-- A key may be associated with an account or standalone.
-- ============================================================

CREATE TABLE ssh_keys (
    id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    uuid                CHAR(36)        NOT NULL,
    account_id          BIGINT UNSIGNED NULL     COMMENT 'NULL for standalone / unassigned keys',
    name                VARCHAR(150)    NOT NULL COMMENT 'Human-readable key name e.g. work-github-key',
    key_type            ENUM(
                            'ed25519',
                            'rsa',
                            'ecdsa',
                            'dsa'
                        )               NOT NULL DEFAULT 'ed25519',
    key_size_bits       SMALLINT UNSIGNED NULL   COMMENT 'Bit size for RSA/ECDSA keys',
    public_key          TEXT            NOT NULL,
    fingerprint         VARCHAR(150)    NOT NULL,
    private_key_path    VARCHAR(1000)   NOT NULL COMMENT 'Absolute path on disk',
    comment_email       VARCHAR(255)    NULL     COMMENT 'Email embedded in key comment',
    passphrase_encrypted TEXT           NULL     COMMENT 'AES-256-GCM encrypted passphrase if set',
    passphrase_iv_hex   VARCHAR(64)     NULL,
    is_added_to_agent   TINYINT(1)      NOT NULL DEFAULT 0,
    is_active           TINYINT(1)      NOT NULL DEFAULT 1,
    last_tested_at      TIMESTAMP       NULL,
    last_test_status    ENUM(
                            'success',
                            'failed',
                            'untested'
                        )               NOT NULL DEFAULT 'untested',
    last_test_error     TEXT            NULL,
    created_at          TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_ssh_keys_uuid         (uuid),
    UNIQUE KEY uq_ssh_keys_fingerprint  (fingerprint),
    KEY         idx_ssh_keys_account    (account_id),

    CONSTRAINT fk_ssh_keys_account
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='SSH key pairs managed by the system.';

-- ============================================================
-- DOMAIN: SSH HOST CONFIGURATIONS
-- Maps SSH host aliases to actual hostnames + key files.
-- Represents entries that would appear in ~/.ssh/config.
-- ============================================================

CREATE TABLE ssh_host_configs (
    id                          BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    uuid                        CHAR(36)        NOT NULL,
    account_id                  BIGINT UNSIGNED NOT NULL,
    ssh_key_id                  BIGINT UNSIGNED NOT NULL,
    host_alias                  VARCHAR(255)    NOT NULL COMMENT 'Host alias: github.com-work',
    hostname                    VARCHAR(255)    NOT NULL COMMENT 'Actual hostname: github.com',
    identity_file               VARCHAR(1000)   NOT NULL COMMENT 'Path to private key',
    port                        SMALLINT UNSIGNED NOT NULL DEFAULT 22,
    user                        VARCHAR(100)    NOT NULL DEFAULT 'git',
    add_keys_to_agent           TINYINT(1)      NOT NULL DEFAULT 1,
    identity_only               TINYINT(1)      NOT NULL DEFAULT 1,
    strict_host_key_checking    ENUM(
                                    'yes',
                                    'no',
                                    'accept-new'
                                )               NOT NULL DEFAULT 'accept-new',
    is_synced_to_disk           TINYINT(1)      NOT NULL DEFAULT 0 COMMENT 'Has this been written to ~/.ssh/config',
    created_at                  TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at                  TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_ssh_host_configs_uuid         (uuid),
    UNIQUE KEY uq_ssh_host_configs_alias        (host_alias),
    KEY         idx_ssh_host_configs_account    (account_id),

    CONSTRAINT fk_ssh_host_configs_account
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_ssh_host_configs_key
        FOREIGN KEY (ssh_key_id) REFERENCES ssh_keys (id)
        ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='SSH config file entries. One row = one Host block in ~/.ssh/config.';

-- ============================================================
-- DOMAIN: REPOSITORIES
-- Every Git repository the system knows about, whether cloned
-- locally or just discovered via the platform API.
-- ============================================================

CREATE TABLE repositories (
    id                      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    uuid                    CHAR(36)        NOT NULL,
    account_id              BIGINT UNSIGNED NOT NULL,
    platform_id             BIGINT UNSIGNED NOT NULL,
    platform_repo_id        VARCHAR(100)    NULL     COMMENT 'Platform-assigned numeric or string ID',
    name                    VARCHAR(255)    NOT NULL COMMENT 'Short repo name without owner prefix',
    full_name               VARCHAR(600)    NOT NULL COMMENT 'owner/repo-name',
    description             TEXT            NULL,
    local_path              VARCHAR(2000)   NULL     COMMENT 'Absolute local filesystem path if cloned',
    remote_url              VARCHAR(2000)   NOT NULL COMMENT 'Canonical remote URL used at clone time',
    clone_url_https         VARCHAR(2000)   NULL,
    clone_url_ssh           VARCHAR(2000)   NULL,
    default_branch          VARCHAR(255)    NOT NULL DEFAULT 'main',
    current_branch          VARCHAR(255)    NULL,
    visibility              ENUM(
                                'public',
                                'private',
                                'internal'
                            )               NOT NULL DEFAULT 'private',
    is_cloned               TINYINT(1)      NOT NULL DEFAULT 0,
    is_archived             TINYINT(1)      NOT NULL DEFAULT 0,
    is_forked               TINYINT(1)      NOT NULL DEFAULT 0,
    is_empty                TINYINT(1)      NOT NULL DEFAULT 0,
    is_template             TINYINT(1)      NOT NULL DEFAULT 0,
    parent_full_name        VARCHAR(600)    NULL     COMMENT 'Parent repo if this is a fork',
    primary_language        VARCHAR(100)    NULL,
    topics                  JSON            NULL     COMMENT 'Array of topic strings',
    size_kb                 INT UNSIGNED    NULL,
    open_issues_count       INT UNSIGNED    NULL DEFAULT 0,
    stargazers_count        INT UNSIGNED    NULL DEFAULT 0,
    forks_count             INT UNSIGNED    NULL DEFAULT 0,
    last_synced_at          TIMESTAMP       NULL,
    last_commit_sha         VARCHAR(100)    NULL,
    last_commit_message     TEXT            NULL,
    last_commit_author      VARCHAR(255)    NULL,
    last_commit_at          TIMESTAMP       NULL,
    clone_depth             INT UNSIGNED    NULL     COMMENT 'Shallow clone depth; NULL means full',
    metadata                JSON            NULL,
    created_at              TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at              TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_repositories_uuid             (uuid),
    UNIQUE KEY uq_repositories_full_name_acct   (full_name, account_id),
    KEY         idx_repositories_account        (account_id),
    KEY         idx_repositories_platform       (platform_id),
    KEY         idx_repositories_is_cloned      (is_cloned),
    FULLTEXT KEY ft_repositories_search         (name, description),

    CONSTRAINT fk_repositories_account
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT fk_repositories_platform
        FOREIGN KEY (platform_id) REFERENCES platforms (id)
        ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Git repositories known to the system (cloned or discovered).';

-- ============================================================
-- DOMAIN: GIT OPERATIONS LOG
-- Every discrete git action (clone, pull, push, commit, etc.)
-- is written here. This is the operation audit trail.
-- ============================================================

CREATE TABLE git_operations (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    uuid            CHAR(36)            NOT NULL,
    repository_id   BIGINT UNSIGNED     NOT NULL,
    account_id      BIGINT UNSIGNED     NOT NULL,
    operation_type  ENUM(
                        'clone',
                        'pull',
                        'push',
                        'fetch',
                        'merge',
                        'rebase',
                        'commit',
                        'checkout',
                        'branch_create',
                        'branch_delete',
                        'branch_rename',
                        'tag_create',
                        'tag_delete',
                        'stash',
                        'stash_pop',
                        'reset',
                        'cherry_pick',
                        'revert'
                    )                   NOT NULL,
    status          ENUM(
                        'pending',
                        'running',
                        'success',
                        'failed',
                        'cancelled',
                        'skipped'
                    )                   NOT NULL DEFAULT 'pending',
    triggered_by    ENUM(
                        'cli',
                        'web',
                        'desktop',
                        'api',
                        'scheduler',
                        'workflow',
                        'plugin'
                    )                   NOT NULL DEFAULT 'cli',
    branch_source   VARCHAR(255)        NULL,
    branch_target   VARCHAR(255)        NULL,
    remote_name     VARCHAR(100)        NOT NULL DEFAULT 'origin',
    commit_sha      VARCHAR(100)        NULL,
    commit_message  TEXT                NULL,
    options         JSON                NULL     COMMENT 'Flags and options passed to git command',
    raw_command     TEXT                NULL     COMMENT 'Exact git command executed',
    stdout_log      LONGTEXT            NULL,
    stderr_log      LONGTEXT            NULL,
    exit_code       TINYINT             NULL,
    duration_ms     INT UNSIGNED        NULL,
    error_message   TEXT                NULL,
    parent_op_id    BIGINT UNSIGNED     NULL     COMMENT 'Parent operation (for chained ops)',
    workflow_id     CHAR(36)            NULL     COMMENT 'FK to workflow_instances.uuid',
    started_at      TIMESTAMP           NULL,
    completed_at    TIMESTAMP           NULL,
    created_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_git_operations_uuid           (uuid),
    KEY         idx_git_ops_repository          (repository_id),
    KEY         idx_git_ops_account             (account_id),
    KEY         idx_git_ops_type_status         (operation_type, status),
    KEY         idx_git_ops_created             (created_at),

    CONSTRAINT fk_git_ops_repository
        FOREIGN KEY (repository_id) REFERENCES repositories (id)
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_git_ops_account
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT fk_git_ops_parent
        FOREIGN KEY (parent_op_id) REFERENCES git_operations (id)
        ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Immutable log of every git operation executed by the system.';

-- ============================================================
-- DOMAIN: SYNC SESSIONS
-- Higher-level than git_operations. A sync session represents
-- a complete push/pull cycle with conflict tracking.
-- ============================================================

CREATE TABLE sync_sessions (
    id                      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    uuid                    CHAR(36)        NOT NULL,
    repository_id           BIGINT UNSIGNED NOT NULL,
    account_id              BIGINT UNSIGNED NOT NULL,
    session_type            ENUM(
                                'push',
                                'pull',
                                'full_sync',
                                'fetch_only'
                            )               NOT NULL,
    status                  ENUM(
                                'pending',
                                'staging',
                                'committing',
                                'pushing',
                                'pulling',
                                'merging',
                                'success',
                                'partial_success',
                                'conflict',
                                'failed',
                                'cancelled'
                            )               NOT NULL DEFAULT 'pending',
    branch_name             VARCHAR(255)    NOT NULL,
    remote_name             VARCHAR(100)    NOT NULL DEFAULT 'origin',
    commit_message          TEXT            NULL,
    commit_sha_before       VARCHAR(100)    NULL,
    commit_sha_after        VARCHAR(100)    NULL,
    files_staged            INT UNSIGNED    NOT NULL DEFAULT 0,
    files_added             INT UNSIGNED    NOT NULL DEFAULT 0,
    files_modified          INT UNSIGNED    NOT NULL DEFAULT 0,
    files_deleted           INT UNSIGNED    NOT NULL DEFAULT 0,
    files_renamed           INT UNSIGNED    NOT NULL DEFAULT 0,
    commits_pushed          INT UNSIGNED    NOT NULL DEFAULT 0,
    commits_pulled          INT UNSIGNED    NOT NULL DEFAULT 0,
    commits_ahead           INT UNSIGNED    NOT NULL DEFAULT 0,
    commits_behind          INT UNSIGNED    NOT NULL DEFAULT 0,
    conflicts_count         INT UNSIGNED    NOT NULL DEFAULT 0,
    conflict_resolution     ENUM(
                                'ours',
                                'theirs',
                                'manual',
                                'abort',
                                'none'
                            )               NOT NULL DEFAULT 'none',
    conflict_files          JSON            NULL     COMMENT 'Array of conflicting file paths',
    options                 JSON            NULL     COMMENT 'Force push, rebase pull, etc.',
    error_message           TEXT            NULL,
    started_at              TIMESTAMP       NULL,
    completed_at            TIMESTAMP       NULL,
    created_at              TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_sync_sessions_uuid            (uuid),
    KEY         idx_sync_sessions_repository    (repository_id),
    KEY         idx_sync_sessions_account       (account_id),
    KEY         idx_sync_sessions_status        (status),
    KEY         idx_sync_sessions_created       (created_at),

    CONSTRAINT fk_sync_sessions_repository
        FOREIGN KEY (repository_id) REFERENCES repositories (id)
        ON DELETE CASCADE ON UPDATE CASCADE,
    CONSTRAINT fk_sync_sessions_account
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE RESTRICT ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Complete push/pull sync cycles with file change statistics.';

-- ============================================================
-- KERNEL: PLUGIN REGISTRY
-- All plugins (built-in and external) self-register here
-- when the kernel boots. The kernel loads them in priority order.
-- ============================================================

CREATE TABLE plugins (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    uuid            CHAR(36)            NOT NULL,
    name            VARCHAR(100)        NOT NULL COMMENT 'Unique internal slug e.g. github_provider',
    display_name    VARCHAR(200)        NOT NULL,
    version         VARCHAR(50)         NOT NULL,
    plugin_type     ENUM(
                        'provider',
                        'interface',
                        'workflow',
                        'storage',
                        'auth',
                        'notification',
                        'analytics',
                        'ai_assistant'
                    )                   NOT NULL,
    entry_point     VARCHAR(500)        NOT NULL COMMENT 'Python import path e.g. git_manager.plugins.github.plugin:GitHubPlugin',
    is_enabled      TINYINT(1)          NOT NULL DEFAULT 1,
    is_core         TINYINT(1)          NOT NULL DEFAULT 0 COMMENT 'Bundled vs externally installed',
    load_priority   SMALLINT            NOT NULL DEFAULT 100 COMMENT 'Lower number loads first',
    config_schema   JSON                NULL     COMMENT 'JSON Schema for validating plugin config',
    dependencies    JSON                NULL     COMMENT 'Array of other plugin names this requires',
    author          VARCHAR(255)        NULL,
    description     TEXT                NULL,
    homepage        VARCHAR(500)        NULL,
    min_kernel_ver  VARCHAR(50)         NULL     COMMENT 'Minimum kernel version required',
    installed_at    TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_plugins_uuid          (uuid),
    UNIQUE KEY uq_plugins_name          (name),
    KEY         idx_plugins_type        (plugin_type),
    KEY         idx_plugins_enabled     (is_enabled)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Plugin registry. Kernel reads this at boot to know what to load.';

-- ============================================================
-- KERNEL: PLUGIN CONFIGURATIONS
-- Key-value config store for each plugin. Supports encrypted
-- values for sensitive plugin settings (API keys, secrets).
-- ============================================================

CREATE TABLE plugin_configs (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    plugin_id       BIGINT UNSIGNED     NOT NULL,
    config_key      VARCHAR(255)        NOT NULL,
    config_value    JSON                NULL,
    data_type       ENUM(
                        'string',
                        'integer',
                        'boolean',
                        'json',
                        'array'
                    )                   NOT NULL DEFAULT 'string',
    is_encrypted    TINYINT(1)          NOT NULL DEFAULT 0,
    description     VARCHAR(500)        NULL,
    created_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_plugin_configs_key    (plugin_id, config_key),
    KEY         idx_plugin_configs_pid  (plugin_id),

    CONSTRAINT fk_plugin_configs_plugin
        FOREIGN KEY (plugin_id) REFERENCES plugins (id)
        ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Per-plugin key-value configuration store.';

-- ============================================================
-- KERNEL: EVENT STORE
-- Every domain event published through the event bus lands here.
-- Supports replaying, debugging, and eventual-consistency patterns.
-- ============================================================

CREATE TABLE events (
    id                  BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    uuid                CHAR(36)        NOT NULL,
    event_type          VARCHAR(150)    NOT NULL COMMENT 'e.g. AccountAdded, RepositoryCloned, SSHKeyGenerated',
    event_version       VARCHAR(20)     NOT NULL DEFAULT '1.0' COMMENT 'Event schema version for evolution',
    aggregate_type      VARCHAR(100)    NULL     COMMENT 'Domain aggregate: Account, Repository, SSHKey',
    aggregate_id        CHAR(36)        NULL     COMMENT 'UUID of the aggregate that emitted this event',
    payload             JSON            NOT NULL COMMENT 'Full event payload',
    metadata            JSON            NULL     COMMENT 'Contextual info: triggered_by, interface, session_id',
    is_processed        TINYINT(1)      NOT NULL DEFAULT 0,
    processing_attempts TINYINT UNSIGNED NOT NULL DEFAULT 0,
    first_processed_at  TIMESTAMP       NULL,
    last_error          TEXT            NULL,
    published_at        TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_events_uuid                   (uuid),
    KEY         idx_events_type                 (event_type),
    KEY         idx_events_aggregate            (aggregate_type, aggregate_id),
    KEY         idx_events_published            (published_at),
    KEY         idx_events_unprocessed          (is_processed, published_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Immutable event store for the internal event bus.';

-- ============================================================
-- KERNEL: EVENT SUBSCRIPTIONS
-- Maps event types to plugin handlers. The event bus reads
-- this table to know which plugins to notify for each event.
-- ============================================================

CREATE TABLE event_subscriptions (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    plugin_id       BIGINT UNSIGNED     NOT NULL,
    event_type      VARCHAR(150)        NOT NULL COMMENT 'Glob supported: Account.* or specific AccountAdded',
    handler_method  VARCHAR(255)        NOT NULL COMMENT 'Plugin class method to invoke',
    is_active       TINYINT(1)          NOT NULL DEFAULT 1,
    priority        SMALLINT            NOT NULL DEFAULT 100 COMMENT 'Lower number fires first',
    is_async        TINYINT(1)          NOT NULL DEFAULT 0 COMMENT 'Whether handler runs in background',
    created_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    KEY         idx_event_subs_event_type   (event_type),
    KEY         idx_event_subs_plugin       (plugin_id),

    CONSTRAINT fk_event_subs_plugin
        FOREIGN KEY (plugin_id) REFERENCES plugins (id)
        ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Event type to plugin handler routing table for the event bus.';

-- ============================================================
-- KERNEL: WORKFLOW DEFINITIONS
-- Named, versioned workflow templates. Each workflow defines
-- a sequence of steps executed by the workflow engine.
-- ============================================================

CREATE TABLE workflow_definitions (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    uuid            CHAR(36)            NOT NULL,
    name            VARCHAR(150)        NOT NULL COMMENT 'Unique slug e.g. clone_and_configure',
    display_name    VARCHAR(255)        NOT NULL,
    description     TEXT                NULL,
    version         VARCHAR(50)         NOT NULL DEFAULT '1.0',
    steps           JSON                NOT NULL COMMENT 'Ordered array of step definitions with plugins and handlers',
    trigger_events  JSON                NULL     COMMENT 'Events that auto-trigger this workflow',
    is_active       TINYINT(1)          NOT NULL DEFAULT 1,
    timeout_seconds INT UNSIGNED        NOT NULL DEFAULT 300,
    retry_attempts  TINYINT UNSIGNED    NOT NULL DEFAULT 3,
    on_failure      ENUM(
                        'abort',
                        'continue',
                        'rollback'
                    )                   NOT NULL DEFAULT 'abort',
    created_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_workflow_defs_uuid        (uuid),
    UNIQUE KEY uq_workflow_defs_name_ver    (name, version)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Workflow templates used by the workflow engine.';

-- ============================================================
-- KERNEL: WORKFLOW INSTANCES
-- A runtime execution of a workflow definition. Tracks current
-- step, context state, and completion status.
-- ============================================================

CREATE TABLE workflow_instances (
    id                      BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    uuid                    CHAR(36)        NOT NULL,
    workflow_definition_id  BIGINT UNSIGNED NOT NULL,
    status                  ENUM(
                                'pending',
                                'running',
                                'paused',
                                'waiting_retry',
                                'completed',
                                'failed',
                                'cancelled',
                                'rolled_back'
                            )               NOT NULL DEFAULT 'pending',
    context                 JSON            NULL     COMMENT 'Mutable runtime state passed between steps',
    current_step_name       VARCHAR(100)    NULL,
    current_step_index      TINYINT UNSIGNED NULL,
    completed_steps         JSON            NOT NULL DEFAULT '[]',
    failed_step_name        VARCHAR(100)    NULL,
    retry_count             TINYINT UNSIGNED NOT NULL DEFAULT 0,
    error_message           TEXT            NULL,
    triggered_by_event_id   BIGINT UNSIGNED NULL,
    triggered_by_interface  ENUM(
                                'cli',
                                'web',
                                'desktop',
                                'api',
                                'scheduler',
                                'event'
                            )               NOT NULL DEFAULT 'event',
    started_at              TIMESTAMP       NULL,
    completed_at            TIMESTAMP       NULL,
    created_at              TIMESTAMP       NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_workflow_instances_uuid       (uuid),
    KEY         idx_workflow_inst_definition    (workflow_definition_id),
    KEY         idx_workflow_inst_status        (status),
    KEY         idx_workflow_inst_created       (created_at),

    CONSTRAINT fk_workflow_inst_definition
        FOREIGN KEY (workflow_definition_id) REFERENCES workflow_definitions (id)
        ON DELETE RESTRICT ON UPDATE CASCADE,
    CONSTRAINT fk_workflow_inst_event
        FOREIGN KEY (triggered_by_event_id) REFERENCES events (id)
        ON DELETE SET NULL ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Runtime execution state for each workflow run.';

-- ============================================================
-- DOMAIN: CLONE CACHE
-- Tracks metadata about repositories discovered during API
-- browsing so the UI can show them without re-fetching.
-- ============================================================

CREATE TABLE clone_cache (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    account_id      BIGINT UNSIGNED     NOT NULL,
    platform_id     BIGINT UNSIGNED     NOT NULL,
    full_name       VARCHAR(600)        NOT NULL,
    clone_url_https VARCHAR(2000)       NULL,
    clone_url_ssh   VARCHAR(2000)       NULL,
    description     TEXT                NULL,
    is_private      TINYINT(1)          NOT NULL DEFAULT 0,
    default_branch  VARCHAR(255)        NOT NULL DEFAULT 'main',
    topics          JSON                NULL,
    metadata        JSON                NULL,
    fetched_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at      TIMESTAMP           NOT NULL,

    PRIMARY KEY (id),
    UNIQUE KEY uq_clone_cache_account_repo      (account_id, full_name),
    KEY         idx_clone_cache_expires         (expires_at),

    CONSTRAINT fk_clone_cache_account
        FOREIGN KEY (account_id) REFERENCES accounts (id)
        ON DELETE CASCADE ON UPDATE CASCADE
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Short-lived cache of API-browsed repositories for the clone wizard UI.';

-- ============================================================
-- INFRASTRUCTURE: APPLICATION CONFIGURATIONS
-- Global key-value configuration for the application. Some
-- values are encrypted (is_secret=1). Plugins can extend this.
-- ============================================================

CREATE TABLE app_configurations (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    config_key      VARCHAR(255)        NOT NULL COMMENT 'Namespaced key e.g. ssh.key_type, ui.theme',
    config_value    JSON                NULL,
    data_type       ENUM(
                        'string',
                        'integer',
                        'boolean',
                        'json',
                        'array'
                    )                   NOT NULL DEFAULT 'string',
    description     TEXT                NULL,
    is_secret       TINYINT(1)          NOT NULL DEFAULT 0,
    is_readonly     TINYINT(1)          NOT NULL DEFAULT 0 COMMENT 'Set by kernel; plugins cannot override',
    namespace       VARCHAR(100)        NOT NULL DEFAULT 'app' COMMENT 'Owner namespace: app, plugin, user',
    created_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_app_config_key        (config_key)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Global application configuration store.';

-- ============================================================
-- INFRASTRUCTURE: AUDIT LOGS
-- Append-only security and activity audit trail. Every
-- sensitive action in the system writes a record here.
-- ============================================================

CREATE TABLE audit_logs (
    id              BIGINT UNSIGNED     NOT NULL AUTO_INCREMENT,
    uuid            CHAR(36)            NOT NULL,
    action          VARCHAR(150)        NOT NULL COMMENT 'Namespaced action e.g. account.created, ssh_key.generated',
    actor           VARCHAR(255)        NULL     COMMENT 'Who/what initiated the action',
    resource_type   VARCHAR(100)        NULL     COMMENT 'Domain type: Account, Repository, SSHKey',
    resource_id     CHAR(36)            NULL     COMMENT 'UUID of affected resource',
    resource_name   VARCHAR(500)        NULL     COMMENT 'Human-readable identifier for display',
    snapshot_before JSON                NULL     COMMENT 'Resource state before the action',
    snapshot_after  JSON                NULL     COMMENT 'Resource state after the action',
    changes         JSON                NULL     COMMENT 'Diff of changed fields',
    interface       ENUM(
                        'cli',
                        'web',
                        'desktop',
                        'api',
                        'scheduler',
                        'kernel'
                    )                   NULL,
    ip_address      VARCHAR(45)         NULL     COMMENT 'IPv4 or IPv6',
    session_id      VARCHAR(255)        NULL,
    severity        ENUM(
                        'info',
                        'warning',
                        'critical'
                    )                   NOT NULL DEFAULT 'info',
    correlation_id  CHAR(36)            NULL     COMMENT 'Links related audit entries in one logical operation',
    created_at      TIMESTAMP           NOT NULL DEFAULT CURRENT_TIMESTAMP,

    PRIMARY KEY (id),
    UNIQUE KEY uq_audit_logs_uuid           (uuid),
    KEY         idx_audit_logs_action       (action),
    KEY         idx_audit_logs_resource     (resource_type, resource_id),
    KEY         idx_audit_logs_severity     (severity),
    KEY         idx_audit_logs_created      (created_at),
    KEY         idx_audit_logs_correlation  (correlation_id)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci
  COMMENT='Append-only security and activity audit trail.';

-- ============================================================
-- SEED DATA: BUILT-IN PLATFORMS
-- These are inserted at first-run by the bootstrap process.
-- ============================================================

INSERT INTO platforms (uuid, name, display_name, api_base_url, ssh_host, supports_oauth, supports_pat, supports_ssh) VALUES
    (UUID(), 'github',        'GitHub',          'https://api.github.com',                    'github.com',           1, 1, 1),
    (UUID(), 'gitlab',        'GitLab',          'https://gitlab.com/api/v4',                 'gitlab.com',           1, 1, 1),
    (UUID(), 'bitbucket',     'Bitbucket',       'https://api.bitbucket.org/2.0',             'bitbucket.org',        1, 1, 1),
    (UUID(), 'azure_devops',  'Azure DevOps',    'https://dev.azure.com',                     'ssh.dev.azure.com',    0, 1, 1),
    (UUID(), 'sourceforge',   'SourceForge',     'https://sourceforge.net/rest',              'git.code.sf.net',      0, 0, 1),
    (UUID(), 'self_hosted',   'Self-Hosted',     NULL,                                        NULL,                   0, 1, 1),
    (UUID(), 'cloud_storage', 'Cloud Storage',   NULL,                                        NULL,                   0, 0, 0),
    (UUID(), 'local_path',    'Local Path',      NULL,                                        NULL,                   0, 0, 0),
    (UUID(), 'custom',        'Custom',          NULL,                                        NULL,                   0, 1, 1);

-- ============================================================
-- SEED DATA: BUILT-IN PLUGINS
-- Registered at bootstrap. is_core=1 means bundled with the app.
-- ============================================================

INSERT INTO plugins (uuid, name, display_name, version, plugin_type, entry_point, is_core, load_priority) VALUES
    (UUID(), 'github_provider',     'GitHub Provider',      '1.0.0', 'provider',   'git_manager.plugins.providers.github.plugin:GitHubPlugin',      1, 10),
    (UUID(), 'gitlab_provider',     'GitLab Provider',      '1.0.0', 'provider',   'git_manager.plugins.providers.gitlab.plugin:GitLabPlugin',      1, 10),
    (UUID(), 'bitbucket_provider',  'Bitbucket Provider',   '1.0.0', 'provider',   'git_manager.plugins.providers.bitbucket.plugin:BitbucketPlugin', 1, 10),
    (UUID(), 'azure_devops_provider','Azure DevOps Provider','1.0.0','provider',   'git_manager.plugins.providers.azure_devops.plugin:AzureDevOpsPlugin', 1, 10),
    (UUID(), 'cli_interface',       'CLI Interface',        '1.0.0', 'interface',  'git_manager.plugins.interfaces.cli.plugin:CLIPlugin',           1, 50),
    (UUID(), 'web_interface',       'Web Interface',        '1.0.0', 'interface',  'git_manager.plugins.interfaces.web.plugin:WebPlugin',           1, 50),
    (UUID(), 'desktop_interface',   'Desktop Interface',    '1.0.0', 'interface',  'git_manager.plugins.interfaces.desktop.plugin:DesktopPlugin',   1, 50);

SET FOREIGN_KEY_CHECKS = 1;