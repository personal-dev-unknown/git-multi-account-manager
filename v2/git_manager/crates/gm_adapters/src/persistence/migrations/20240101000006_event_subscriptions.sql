-- Event type to plugin handler routing table for the event bus.
CREATE TABLE IF NOT EXISTS event_subscriptions (
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
