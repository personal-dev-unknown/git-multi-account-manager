-- Per-plugin key-value configuration store.
CREATE TABLE IF NOT EXISTS plugin_configs (
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
    is_encrypted    TINYINT(1)          NOT NULL DEFAULT 0 COMMENT 'Whether config_value is stored encrypted',
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
