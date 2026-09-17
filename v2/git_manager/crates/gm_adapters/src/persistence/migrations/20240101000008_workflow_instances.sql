-- Runtime execution state for each workflow run.
CREATE TABLE IF NOT EXISTS workflow_instances (
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
