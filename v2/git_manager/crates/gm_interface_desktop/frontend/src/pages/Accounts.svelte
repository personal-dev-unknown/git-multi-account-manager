<script lang="ts">
  import { onMount } from 'svelte';
  import {
    accounts, platforms, platformMap,
    currentPage, selectedUuid, navigate, notify,
  } from '../lib/store';
  import {
    listAccounts, getAccount, addAccount,
    removeAccount, setDefaultAccount, storeAccountToken,
  } from '../lib/tauri';
  import AccountCard from '../components/AccountCard.svelte';
  import type { AccountDto } from '../lib/types';

  // ── Add-account form state ─────────────────────────────────────────────────
  let form = { alias: '', platform_id: '', username: '', email: '', auth_method: 'ssh', pat: '' };
  let formError = '';
  let formLoading = false;

  // ── Detail view state ──────────────────────────────────────────────────────
  let detailAccount: AccountDto | null = null;
  let detailLoading = false;

  // ── Token input (shown in detail view) ────────────────────────────────────
  let tokenInput = '';
  let tokenLoading = false;

  $: if ($currentPage === 'account_detail' && $selectedUuid) {
    loadDetail($selectedUuid);
  }

  async function loadDetail(uuid: string) {
    detailLoading = true;
    try { detailAccount = await getAccount(uuid); }
    catch (e) { notify('error', 'Load failed', String(e)); }
    finally { detailLoading = false; }
  }

  async function handleAddAccount() {
    formError = '';
    if (!form.alias || !form.platform_id || !form.username || !form.email) {
      formError = 'All fields are required.';
      return;
    }
    formLoading = true;
    try {
      const acc = await addAccount({
        alias:       form.alias.trim(),
        platform_id: form.platform_id,
        username:    form.username.trim(),
        email:       form.email.trim(),
        auth_method: form.auth_method,
      });
      accounts.update((a) => [...a, acc]);
      // Store PAT if provided
      if (form.auth_method === 'https_pat' && form.pat.trim()) {
        await storeAccountToken(acc.uuid, form.pat.trim());
      }
      notify('success', 'Account added', `${acc.alias} was added successfully.`);
      form = { alias: '', platform_id: '', username: '', email: '', auth_method: 'ssh', pat: '' };
      navigate('accounts');
    } catch (e) {
      formError = String(e);
    } finally {
      formLoading = false;
    }
  }

  async function handleRemove(uuid: string, alias: string) {
    if (!confirm(`Remove account "${alias}"? This also deletes its SSH keys and credentials.`)) return;
    try {
      await removeAccount(uuid);
      accounts.update((a) => a.filter((x) => x.uuid !== uuid));
      notify('success', 'Account removed', `${alias} was removed.`);
      if ($currentPage === 'account_detail') navigate('accounts');
    } catch (e) {
      notify('error', 'Remove failed', String(e));
    }
  }

  async function handleSetDefault(uuid: string, platform_id: string) {
    try {
      await setDefaultAccount(uuid, platform_id);
      accounts.update((a) =>
        a.map((x) =>
          x.platform_id === platform_id
            ? { ...x, is_default: x.uuid === uuid }
            : x
        )
      );
      notify('success', 'Default set', 'Default account updated.');
    } catch (e) {
      notify('error', 'Failed', String(e));
    }
  }

  async function handleStoreToken(uuid: string) {
    if (!tokenInput.trim()) return;
    tokenLoading = true;
    try {
      await storeAccountToken(uuid, tokenInput.trim());
      notify('success', 'Token saved', 'Credential stored securely in the OS keychain.');
      tokenInput = '';
    } catch (e) {
      notify('error', 'Token save failed', String(e));
    } finally {
      tokenLoading = false;
    }
  }
</script>

<!-- ── List view ────────────────────────────────────────────────────────────── -->
{#if $currentPage === 'accounts'}
  <div class="page-header">
    <div>
      <h1 class="page-title">Accounts</h1>
      <p class="page-subtitle">{$accounts.length} account{$accounts.length === 1 ? '' : 's'} registered</p>
    </div>
    <button class="btn btn--primary btn--sm" on:click={() => navigate('accounts_new')}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
      Add Account
    </button>
  </div>

  <div class="page-body">
    {#if $accounts.length > 0}
      <div class="card">
        <div class="divide-y">
          {#each $accounts as account (account.uuid)}
            <AccountCard
              {account}
              on:click={() => navigate('account_detail', account.uuid)}
              on:remove={() => handleRemove(account.uuid, account.alias)}
            />
          {/each}
        </div>
      </div>
    {:else}
      <div class="empty-state card">
        <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/>
        </svg>
        <p class="empty-title">No accounts yet</p>
        <p class="empty-desc">Add a GitHub, GitLab, Bitbucket, or Azure DevOps account to get started.</p>
        <button class="btn btn--primary btn--sm" on:click={() => navigate('accounts_new')}>Add Account</button>
      </div>
    {/if}
  </div>

<!-- ── Add account view ─────────────────────────────────────────────────────── -->
{:else if $currentPage === 'accounts_new'}
  <div class="page-header">
    <div>
      <h1 class="page-title">Add Account</h1>
      <p class="page-subtitle">Connect a Git hosting account with SSH key isolation</p>
    </div>
    <button class="btn btn--ghost btn--sm" on:click={() => navigate('accounts')}>← Back</button>
  </div>

  <div class="page-body">
    <div class="card card--elevated" style="max-width:540px">
      <div class="card-body">
        {#if formError}
          <div style="background:var(--color-error-bg);color:var(--color-error-text);
                      border-radius:var(--radius-md);padding:var(--space-3);font-size:var(--text-sm);
                      margin-bottom:var(--space-4);">
            {formError}
          </div>
        {/if}

        <div class="form-group">
          <label class="form-label" for="platform">Platform</label>
          <select id="platform" class="select" bind:value={form.platform_id} required>
            <option value="">— Choose platform —</option>
            {#each $platforms as p (p.uuid)}
              <option value={p.uuid}>{p.display_name}</option>
            {/each}
          </select>
        </div>

        <div class="form-group">
          <label class="form-label" for="alias">Alias</label>
          <input id="alias" class="input" bind:value={form.alias}
                 placeholder="work, personal, client-acme" maxlength="39" required />
          <p class="form-hint">Used in SSH key names and git config. Lowercase, hyphens ok.</p>
        </div>

        <div class="form-group">
          <label class="form-label" for="username">Username</label>
          <input id="username" class="input" bind:value={form.username}
                 placeholder="shakamoses" required maxlength="100" />
        </div>

        <div class="form-group">
          <label class="form-label" for="email">Email</label>
          <input id="email" class="input" type="email" bind:value={form.email}
                 placeholder="you@example.com" required maxlength="254" />
        </div>

        <div class="form-group">
          <label class="form-label" for="authmethod">Auth Method</label>
          <select id="authmethod" class="select" bind:value={form.auth_method}>
            <option value="ssh">SSH Key (recommended)</option>
            <option value="https_pat">Personal Access Token</option>
          </select>
        </div>

        {#if form.auth_method === 'https_pat'}
          <div class="form-group">
            <label class="form-label" for="pat">Personal Access Token</label>
            <input id="pat" class="input input--mono" type="password" bind:value={form.pat}
                   placeholder="ghp_... / glpat-... / org:pat" autocomplete="new-password" />
            <p class="form-hint">Stored encrypted in the OS keychain. Used for repository listing via API.</p>
          </div>
        {/if}

        <div class="flex justify-end gap-2 mt-4">
          <button class="btn btn--secondary" on:click={() => navigate('accounts')}>Cancel</button>
          <button class="btn btn--primary {formLoading ? 'btn--loading' : ''}"
                  disabled={formLoading} on:click={handleAddAccount}>
            {#if !formLoading}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
              Add Account
            {/if}
          </button>
        </div>
      </div>
    </div>

    <div style="max-width:540px;margin-top:var(--space-4);background:var(--color-info-bg);
                color:var(--color-info-text);border-radius:var(--radius-md);
                padding:var(--space-4);font-size:var(--text-sm);line-height:var(--leading-relaxed);">
      <strong>Next step:</strong> after adding the account, go to SSH Keys to generate a key pair,
      then add the public key to your hosting platform.
    </div>
  </div>

<!-- ── Detail view ──────────────────────────────────────────────────────────── -->
{:else if $currentPage === 'account_detail'}
  {#if detailLoading}
    <div class="empty-state"><p class="text-muted">Loading…</p></div>
  {:else if detailAccount}
    {@const acc = detailAccount}
    <div class="page-header">
      <div class="flex items-center gap-4">
        <div class="avatar" style="width:48px;height:48px;font-size:var(--text-xl)">
          {acc.alias.slice(0, 2).toUpperCase()}
        </div>
        <div>
          <h1 class="page-title">{acc.alias}</h1>
          <p class="page-subtitle">{acc.username} · {acc.email}</p>
        </div>
      </div>
      <div class="flex gap-2">
        <button class="btn btn--ghost btn--sm" on:click={() => navigate('accounts')}>← Back</button>
        <button class="btn btn--danger btn--sm" on:click={() => handleRemove(acc.uuid, acc.alias)}>Remove</button>
      </div>
    </div>

    <div class="page-body">
      <div class="grid-2 gap-4">
        <div class="card">
          <div class="card-header"><span class="card-title">Details</span></div>
          <div class="card-body">
            <dl style="display:grid;grid-template-columns:1fr 2fr;gap:var(--space-2) var(--space-3);">
              <dt class="text-xs text-muted font-medium">Platform</dt>
              <dd class="text-sm">{acc.platform_name ?? '—'}</dd>
              <dt class="text-xs text-muted font-medium">Username</dt>
              <dd class="text-sm font-mono">{acc.username}</dd>
              <dt class="text-xs text-muted font-medium">Email</dt>
              <dd class="text-sm">{acc.email}</dd>
              <dt class="text-xs text-muted font-medium">Auth</dt>
              <dd><span class="badge badge--neutral">{acc.auth_method}</span></dd>
              <dt class="text-xs text-muted font-medium">Status</dt>
              <dd>
                <span class="badge {acc.is_active ? 'badge--success' : 'badge--neutral'}">
                  {acc.is_active ? 'Active' : 'Inactive'}
                </span>
              </dd>
            </dl>
          </div>
          <div class="card-footer flex gap-2">
            <button class="btn btn--secondary btn--sm"
                    on:click={() => navigate('ssh_generate')}>Generate SSH Key</button>
            <button class="btn btn--ghost btn--sm"
                    on:click={() => navigate('ssh')}>View SSH Keys</button>
          </div>
        </div>

        {#if acc.auth_method === 'https_pat'}
          <div class="card">
            <div class="card-header"><span class="card-title">API Token</span></div>
            <div class="card-body">
              <p class="text-sm text-muted mb-4">Store or update the Personal Access Token used for API calls.</p>
              <div class="form-group">
                <label class="form-label" for="token-input">New Token</label>
                <input id="token-input" class="input input--mono" type="password"
                       bind:value={tokenInput} placeholder="Paste token here" autocomplete="new-password" />
              </div>
              <button class="btn btn--primary btn--sm {tokenLoading ? 'btn--loading' : ''}"
                      disabled={tokenLoading || !tokenInput.trim()}
                      on:click={() => handleStoreToken(acc.uuid)}>
                {#if !tokenLoading}Save Token{/if}
              </button>
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
{/if}