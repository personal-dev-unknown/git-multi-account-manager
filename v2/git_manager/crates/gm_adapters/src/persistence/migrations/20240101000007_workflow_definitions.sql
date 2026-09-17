-- Workflow templates used by the workflow engine.
CREATE TABLE IF NOT EXISTS workflow_definitions (
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
