-- Plugin registry table. Kernel reads this at boot to determine load order.
CREATE TABLE IF NOT EXISTS plugins (
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
