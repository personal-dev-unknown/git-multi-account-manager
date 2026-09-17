<script lang="ts">
  import { repositories, accounts, navigate, notify } from '../lib/store';
  import { gitPull, gitPush, gitStatus, syncRepository } from '../lib/tauri';
  import ProgressBar from '../components/ProgressBar.svelte';
  import Terminal from '../components/Terminal.svelte';
  import type { GitStatusEntry } from '../lib/types';

  // Selected repo and account
  let selectedRepoUuid  = $repositories.filter((r) => r.is_cloned)[0]?.uuid ?? '';
  let selectedAccountUuid = $accounts[0]?.uuid ?? '';

  $: selectedRepo    = $repositories.find((r) => r.uuid === selectedRepoUuid);
  $: selectedAccount = $accounts.find((a) => a.uuid === selectedAccountUuid);

  // Status
  let statusEntries: GitStatusEntry[] = [];
  let statusLoading = false;

  // Commit / push
  let commitMessage = '';
  let branch = '';

  // Operation state
  type OpState = 'idle' | 'pulling' | 'pushing' | 'syncing';
  let opState: OpState = 'idle';
  let opResult: { commits_transferred: number; current_sha: string | null } | null = null;
  let opError = '';

  // Terminal log
  let logLines: Array<{ time: string; cls: string; text: string }> = [];

  function log(cls: 'ok' | 'err' | 'info', text: string) {
    const time = new Date().toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    logLines = [...logLines, { time, cls: `terminal-${cls}`, text }];
  }

  async function loadStatus() {
    if (!selectedRepoUuid) return;
    statusLoading = true;
    try {
      statusEntries = await gitStatus(selectedRepoUuid);
    } catch (e) {
      notify('error', 'Status failed', String(e));
    } finally {
      statusLoading = false;
    }
  }

  $: { if (selectedRepoUuid) { void loadStatus(); } }

  async function doPull() {
    if (!selectedRepoUuid || !selectedAccountUuid) return;
    opState = 'pulling'; opError = ''; opResult = null;
    log('info', `Pulling ${selectedRepo?.name ?? 'repository'}…`);
    try {
      const r = await gitPull({
        repository_uuid: selectedRepoUuid,
        account_uuid:    selectedAccountUuid,
        branch:          branch.trim() || null,
        rebase:          false,
      });
      opResult = r;
      log('ok', `Pulled ${r.commits_transferred} commit(s). HEAD: ${r.current_sha?.slice(0, 8) ?? 'unknown'}`);
      notify('success', 'Pulled', `${r.commits_transferred} commits pulled.`);
      await loadStatus();
    } catch (e) {
      opError = String(e);
      log('err', String(e));
      notify('error', 'Pull failed', String(e));
    } finally {
      opState = 'idle';
    }
  }

  async function doPush() {
    if (!selectedRepoUuid || !selectedAccountUuid) return;
    if (!commitMessage.trim()) { notify('warning', 'Message required', 'Enter a commit message.'); return; }
    opState = 'pushing'; opError = ''; opResult = null;
    log('info', `Committing: "${commitMessage}"`);
    try {
      const r = await gitPush({
        repository_uuid: selectedRepoUuid,
        account_uuid:    selectedAccountUuid,
        commit_message:  commitMessage.trim(),
        branch:          branch.trim() || selectedRepo?.default_branch || 'main',
        force:           false,
      });
      opResult = r;
      log('ok', `Pushed ${r.commits_pushed} commit(s). HEAD: ${r.current_sha?.slice(0, 8) ?? 'unknown'}`);
      notify('success', 'Pushed', `${r.commits_pushed} commit(s) pushed.`);
      commitMessage = '';
      await loadStatus();
    } catch (e) {
      opError = String(e);
      log('err', String(e));
      notify('error', 'Push failed', String(e));
    } finally {
      opState = 'idle';
    }
  }

  async function doSync() {
    if (!selectedRepoUuid || !selectedAccountUuid) return;
    opState = 'syncing'; opError = ''; opResult = null;
    log('info', 'Starting full sync (pull --rebase then push)…');
    try {
      const r = await syncRepository({
        repository_uuid: selectedRepoUuid,
        account_uuid:    selectedAccountUuid,
        commit_message:  commitMessage.trim() || 'chore: sync',
        branch:          branch.trim() || selectedRepo?.default_branch || 'main',
      });
      log(r.had_conflicts ? 'err' : 'ok', r.message);
      if (!r.had_conflicts) {
        notify('success', 'Sync complete', r.message);
        commitMessage = '';
        await loadStatus();
      } else {
        notify('warning', 'Conflicts detected', r.message);
      }
    } catch (e) {
      opError = String(e);
      log('err', String(e));
      notify('error', 'Sync failed', String(e));
    } finally {
      opState = 'idle';
    }
  }

  function statusSymbol(s: GitStatusEntry): string {
    if (s.status[0] !== ' ' && s.status[0] !== '?') return '●';
    if (s.status[1] === 'M') return '○';
    if (s.status === '??')   return '+';
    return '·';
  }
</script>

<div class="page-header">
  <div>
    <h1 class="page-title">Sync</h1>
    <p class="page-subtitle">Pull, commit, and push changes</p>
  </div>
</div>

<div class="page-body">

  <!-- Repository and account selector -->
  <div class="flex gap-4 mb-5">
    <div class="form-group flex-1" style="margin:0">
      <label class="form-label" for="sync-repo">Repository</label>
      <select id="sync-repo" class="select" bind:value={selectedRepoUuid}
              on:change={() => loadStatus()}>
        {#if $repositories.filter((r) => r.is_cloned).length === 0}
          <option value="">— Clone a repository first —</option>
        {/if}
        {#each $repositories.filter((r) => r.is_cloned) as repo (repo.uuid)}
          <option value={repo.uuid}>{repo.name} ({repo.default_branch})</option>
        {/each}
      </select>
    </div>
    <div class="form-group flex-1" style="margin:0">
      <label class="form-label" for="sync-account">Account</label>
      <select id="sync-account" class="select" bind:value={selectedAccountUuid}>
        {#each $accounts as acc (acc.uuid)}
          <option value={acc.uuid}>{acc.alias}</option>
        {/each}
      </select>
    </div>
    <div class="form-group" style="margin:0;width:160px">
      <label class="form-label" for="sync-branch">Branch</label>
      <input id="sync-branch" class="input" bind:value={branch}
             placeholder={selectedRepo?.default_branch ?? 'main'} />
    </div>
  </div>

  {#if opState !== 'idle'}
    <div class="mb-4">
      <ProgressBar indeterminate={true} label={opState === 'pulling' ? 'Pulling…' : opState === 'pushing' ? 'Pushing…' : 'Syncing…'} />
    </div>
  {/if}

  <div class="grid-2 gap-4">

    <!-- Working directory status -->
    <div class="card">
      <div class="card-header">
        <span class="card-title">Working Directory</span>
        <button class="btn btn--ghost btn--sm" on:click={loadStatus} disabled={statusLoading}>
          {statusLoading ? 'Checking…' : 'Refresh'}
        </button>
      </div>
      {#if statusEntries.length > 0}
        <div style="overflow-y:auto;max-height:260px;">
          {#each statusEntries as entry (entry.path)}
            <div class="flex items-center gap-3 p-3 border-b"
                 style="border-color:var(--color-border);">
              <span class="font-mono text-xs" style="width:16px;color:var(--color-brand)">
                {statusSymbol(entry)}
              </span>
              <span class="text-sm truncate flex-1 font-mono">{entry.path}</span>
              <code class="text-xs text-muted">{entry.status}</code>
            </div>
          {/each}
        </div>
        <div class="card-footer">
          <span class="text-xs text-muted">{statusEntries.length} file{statusEntries.length !== 1 ? 's' : ''} changed</span>
        </div>
      {:else}
        <div class="p-5 text-sm text-muted">
          {statusLoading ? 'Loading status…' : 'Working directory clean'}
        </div>
      {/if}
    </div>

    <!-- Operations panel -->
    <div class="card">
      <div class="card-header"><span class="card-title">Git Operations</span></div>
      <div class="card-body">

        <!-- Pull -->
        <div class="mb-4 p-4 rounded" style="background:var(--color-bg-elevated);border-radius:var(--radius-md)">
          <div class="flex items-center justify-between">
            <div>
              <p class="font-semibold text-sm">Pull</p>
              <p class="text-xs text-muted">Fetch and integrate remote changes</p>
            </div>
            <button class="btn btn--secondary btn--sm {opState === 'pulling' ? 'btn--loading' : ''}"
                    disabled={opState !== 'idle' || !selectedRepoUuid}
                    on:click={doPull}>
              {#if opState !== 'pulling'}Pull{/if}
            </button>
          </div>
        </div>

        <!-- Commit message -->
        <div class="form-group">
          <label class="form-label" for="commit-msg">Commit Message</label>
          <input id="commit-msg" class="input" bind:value={commitMessage}
                 placeholder="feat: describe what changed" />
        </div>

        <!-- Push and Sync buttons -->
        <div class="flex gap-2">
          <button class="btn btn--secondary flex-1 {opState === 'pushing' ? 'btn--loading' : ''}"
                  disabled={opState !== 'idle' || !selectedRepoUuid || !commitMessage.trim()}
                  on:click={doPush}>
            {#if opState !== 'pushing'}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="23 4 23 10 17 10"/>
                <path d="M20.49 15a9 9 0 11-2.12-9.36L23 10"/>
              </svg>
              Commit & Push
            {/if}
          </button>
          <button class="btn btn--primary flex-1 {opState === 'syncing' ? 'btn--loading' : ''}"
                  disabled={opState !== 'idle' || !selectedRepoUuid}
                  on:click={doSync}>
            {#if opState !== 'syncing'}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
                <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
              </svg>
              Full Sync
            {/if}
          </button>
        </div>

        {#if opError}
          <div class="mt-3" style="background:var(--color-error-bg);color:var(--color-error-text);
                                   border-radius:var(--radius-md);padding:var(--space-3);font-size:var(--text-xs);
                                   font-family:var(--font-mono);">
            {opError}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Terminal log -->
  {#if logLines.length > 0}
    <div class="card mt-4">
      <div class="card-header">
        <span class="card-title">Operation Log</span>
        <button class="btn btn--ghost btn--sm" on:click={() => (logLines = [])}>Clear</button>
      </div>
      <div class="card-body" style="padding:0">
        <Terminal lines={logLines} />
      </div>
    </div>
  {/if}

</div>