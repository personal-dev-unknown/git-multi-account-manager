/**
 * Type definitions that mirror the Rust DTO structs serialised by the backend.
 *
 * The Rust side uses serde's default serialisation, which emits snake_case field
 * names.  All UUIDs arrive as hyphenated lowercase strings ("xxxxxxxx-xxxx-…").
 * Timestamps arrive as ISO 8601 UTC strings (DateTime<Utc> → serde_json::to_string).
 *
 * Keep this file in sync with the DTO structs in gm_shared.
 */

// ── Accounts ──────────────────────────────────────────────────────────────────

export interface AccountDto {
  uuid:          string;
  platform_id:   string;
  platform_name: string | null;
  alias:         string;
  username:      string;
  email:         string;
  display_name:  string | null;
  auth_method:   string;   // "ssh" | "https_pat" | "https_password" | "oauth"
  is_default:    boolean;
  is_active:     boolean;
  ssh_host_alias: string | null;
  created_at:    string;
}

// ── SSH Keys ──────────────────────────────────────────────────────────────────

export type KeyType = 'Ed25519' | 'Rsa' | 'Ecdsa' | 'Dsa';
export type TestStatus = 'NotTested' | 'Success' | 'Failed';

export interface SshKeyDto {
  uuid:              string;
  account_id:        string | null;
  name:              string;
  key_type:          KeyType;
  key_size_bits:     number | null;
  public_key:        string;
  fingerprint:       string;
  private_key_path:  string;
  comment_email:     string | null;
  is_added_to_agent: boolean;
  is_active:         boolean;
  last_tested_at:    string | null;
  last_test_status:  TestStatus;
  last_test_error:   string | null;
  created_at:        string;
}

// ── Repositories ──────────────────────────────────────────────────────────────

export type RepositoryVisibility = 'Public' | 'Private' | 'Internal';

export interface RepositoryDto {
  uuid:                string;
  account_id:          string;
  platform_id:         string;
  name:                string;
  full_name:           string;
  description:         string | null;
  local_path:          string | null;
  remote_url:          string;
  clone_url_ssh:       string | null;
  clone_url_https:     string | null;
  default_branch:      string;
  current_branch:      string | null;
  visibility:          RepositoryVisibility;
  is_cloned:           boolean;
  is_archived:         boolean;
  is_forked:           boolean;
  last_commit_sha:     string | null;
  last_commit_message: string | null;
  last_commit_author:  string | null;
  last_commit_at:      string | null;
  last_synced_at:      string | null;
  primary_language:    string | null;
  stargazers_count:    number;
  forks_count:         number;
  open_issues_count:   number;
  created_at:          string;
}

// ── Platform ──────────────────────────────────────────────────────────────────

/** Static platform info returned by the list_platforms command. */
export interface Platform {
  uuid:         string;
  name:         string;
  display_name: string;
  ssh_host:     string;
}

// ── Operation results ─────────────────────────────────────────────────────────

export interface GitOpResult {
  commits_transferred: number;
  current_sha:         string | null;
  had_conflicts:       boolean;
}

export interface SshTestResult {
  success:  boolean;
  username: string | null;
  error:    string | null;
}

export interface GitStatusEntry {
  path:   string;
  /** Two-char porcelain status code, e.g. "M ", " M", "A ", "??" */
  status: string;
}

export interface SyncResult {
  pull:           GitOpResult | null;
  push:           GitOpResult | null;
  had_conflicts:  boolean;
  message:        string;
}

// ── UI helpers ────────────────────────────────────────────────────────────────

/** Notification shown in the top-right corner. */
export interface Notification {
  id:      number;
  type:    'success' | 'error' | 'warning' | 'info';
  title:   string;
  message: string;
}

/** Application page identifiers for the hash-based router. */
export type Page =
  | 'dashboard'
  | 'accounts'
  | 'accounts_new'
  | 'account_detail'
  | 'repositories'
  | 'repositories_clone'
  | 'repository_detail'
  | 'ssh'
  | 'ssh_generate'
  | 'sync';