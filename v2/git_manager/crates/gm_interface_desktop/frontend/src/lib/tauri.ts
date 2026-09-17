/**
 * Typed Tauri API wrappers.
 *
 * Every function in this file is a thin, strongly-typed wrapper around
 * `invoke()` from `@tauri-apps/api/tauri`. The naming mirrors the Rust
 * command names exactly — this is the contract between frontend and backend.
 *
 * Parameter objects use snake_case keys because the Rust command parameters
 * use snake_case and Tauri 1.x does not auto-convert to camelCase.
 *
 * All functions return a `Promise<T>`. On error, the promise rejects with a
 * `string` message (the `Err(String)` returned by the Rust command handler).
 * Callers should `try/catch` and call `notify('error', ...)` from `store.ts`.
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  AccountDto,
  GitOpResult,
  GitStatusEntry,
  Platform,
  RepositoryDto,
  SshKeyDto,
  SshTestResult,
} from './types';

// ── Account commands ──────────────────────────────────────────────────────────

export async function listAccounts(platform_id?: string): Promise<AccountDto[]> {
  return invoke<AccountDto[]>('list_accounts', {
    platformId: platform_id ?? null,
  });
}

export async function getAccount(uuid: string): Promise<AccountDto | null> {
  return invoke<AccountDto | null>('get_account', { uuid });
}

export async function addAccount(params: {
  alias:       string;
  platform_id: string;
  username:    string;
  email:       string;
  auth_method: string;
}): Promise<AccountDto> {
  return invoke<AccountDto>('add_account', { cmd: params });
}

export async function removeAccount(uuid: string): Promise<void> {
  return invoke<void>('remove_account', { uuid });
}

export async function setDefaultAccount(
  account_uuid: string,
  platform_id:  string
): Promise<void> {
  return invoke<void>('set_default_account', { accountUuid: account_uuid, platformId: platform_id });
}

export async function storeAccountToken(
  account_uuid: string,
  token:        string
): Promise<void> {
  return invoke<void>('store_account_token', { accountUuid: account_uuid, token });
}

export async function listPlatforms(): Promise<Platform[]> {
  return invoke<Platform[]>('list_platforms');
}

// ── SSH key commands ──────────────────────────────────────────────────────────

export async function listSshKeys(account_uuid: string): Promise<SshKeyDto[]> {
  return invoke<SshKeyDto[]>('list_ssh_keys', { accountUuid: account_uuid });
}

export async function generateSshKey(params: {
  account_uuid: string;
  key_type:     string;
  comment:      string | null;
  passphrase:   string | null;
  add_to_agent: boolean;
}): Promise<SshKeyDto> {
  return invoke<SshKeyDto>('generate_ssh_key', { cmd: params });
}

export async function testSshConnection(
  account_uuid: string,
  timeout_ms?:  number
): Promise<SshTestResult> {
  return invoke<SshTestResult>('test_ssh_connection', {
    cmd: { account_uuid, timeout_ms: timeout_ms ?? 10000 },
  });
}

export async function addKeyToAgent(account_uuid: string): Promise<void> {
  return invoke<void>('add_key_to_agent', { accountUuid: account_uuid });
}

export async function getPublicKey(account_uuid: string): Promise<string | null> {
  return invoke<string | null>('get_public_key', { accountUuid: account_uuid });
}

// ── Repository commands ───────────────────────────────────────────────────────

export async function listRepositories(account_uuid?: string): Promise<RepositoryDto[]> {
  return invoke<RepositoryDto[]>('list_repositories', {
    accountUuid: account_uuid ?? null,
  });
}

export async function listRemoteRepositories(
  account_uuid: string,
  page?: number,
  per_page?: number,
): Promise<string[]> {
  return invoke<string[]>('list_remote_repositories', {
    accountUuid: account_uuid,
    page: page ?? 1,
    perPage: per_page ?? 15,
  });
}

export async function cloneRepository(params: {
  url:          string;
  account_uuid: string;
  destination:  string | null;
  branch:       string | null;
  shallow:      boolean;
}): Promise<RepositoryDto> {
  return invoke<RepositoryDto>('clone_repository', { cmd: params });
}

export async function getRepository(uuid: string): Promise<RepositoryDto | null> {
  return invoke<RepositoryDto | null>('get_repository', { uuid });
}

export async function getRepositoryLocalPath(uuid: string): Promise<string | null> {
  return invoke<string | null>('get_repository_local_path', { uuid });
}

// ── Git operation commands ────────────────────────────────────────────────────

export async function gitPull(params: {
  repository_uuid: string;
  account_uuid:    string;
  branch:          string | null;
  rebase:          boolean;
}): Promise<GitOpResult> {
  return invoke<GitOpResult>('git_pull', { cmd: params });
}

export async function gitPush(params: {
  repository_uuid: string;
  account_uuid:    string;
  commit_message:  string;
  branch:          string;
  force:           boolean;
}): Promise<GitOpResult> {
  return invoke<GitOpResult>('git_push', { cmd: params });
}

export async function gitStatus(repository_uuid: string): Promise<GitStatusEntry[]> {
  return invoke<GitStatusEntry[]>('git_status', { repositoryUuid: repository_uuid });
}

export async function syncRepository(params: {
  repository_uuid: string;
  account_uuid:    string;
  commit_message:  string;
  branch:          string;
}): Promise<{ pull: GitOpResult | null; push: GitOpResult | null; had_conflicts: boolean; message: string }> {
  return invoke('sync_repository', {
    repositoryUuid: params.repository_uuid,
    accountUuid:    params.account_uuid,
    commitMessage:  params.commit_message,
    branch:         params.branch,
  });
}