-- Short-lived cache of API-browsed repositories for the clone wizard UI.
CREATE TABLE IF NOT EXISTS clone_cache (
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
