-- Track clone operations for observability and audit.
CREATE TABLE IF NOT EXISTS clone_operations (
    uuid           CHAR(36)     NOT NULL PRIMARY KEY,
    account_id     CHAR(36)     NOT NULL,
    repository_id  CHAR(36)     NULL,
    url            VARCHAR(2048) NOT NULL,
    destination    VARCHAR(1024) NOT NULL,
    strategy       VARCHAR(32)   NOT NULL COMMENT 'ssh|https_pat|https_password|anonymous',
    protocol       VARCHAR(8)    NOT NULL COMMENT 'ssh|https',
    status         VARCHAR(16)   NOT NULL DEFAULT 'started' COMMENT 'started|completed|failed',
    attempts       INT UNSIGNED  NOT NULL DEFAULT 1,
    duration_ms    BIGINT UNSIGNED NULL,
    error_message  TEXT          NULL,
    started_at     TIMESTAMP(6)  NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
    completed_at   TIMESTAMP(6)  NULL,
    INDEX idx_clone_ops_account (account_id),
    INDEX idx_clone_ops_status  (status)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
