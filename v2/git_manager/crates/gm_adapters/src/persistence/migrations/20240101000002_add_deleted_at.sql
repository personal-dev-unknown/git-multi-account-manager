-- Add soft-delete support to accounts table (idempotent).
ALTER TABLE accounts ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMP NULL DEFAULT NULL AFTER updated_at;
ALTER TABLE accounts ADD KEY IF NOT EXISTS idx_accounts_deleted (deleted_at);
