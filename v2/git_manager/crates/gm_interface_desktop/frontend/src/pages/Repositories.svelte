<script lang="ts">
  import { repositories, accounts, currentPage, selectedUuid, navigate, notify } from '../lib/store';
  import { listRepositories, getRepository, gitStatus } from '../lib/tauri';
  import RepoList from '../components/RepoList.svelte';
  import type { RepositoryDto, GitStatusEntry } from '../lib/types';

  let detailRepo: RepositoryDto | null = null;
  let statusEntries: GitStatusEntry[] = [];
  let detailLoading = false;
  let statusLoading = false;
  let filterAccountUuid = '';

  $: filteredRepos = filterAccountUuid
    ? $repositories.filter((r) => r.account_id === filterAccountUuid)
    : $repositories;

  $: clonedRepos  = filteredRepos.filter((r) => r.is_cloned);
  $: remoteRepos  = filteredRepos.filter((r) => !r.is_cloned);

  $: if ($currentPage === 'repository_detail' && $selectedUuid) {
    loadDetail($selectedUuid);
  }

  async function loadDetail(uuid: string) {
    detailLoading = true;
    statusEntries = [];
    try {
      detailRepo = await getRepository(uuid);
      if (detailRepo?.is_cloned) {
        statusLoading = true;
        try {
          statusEntries = await gitStatus(uuid);
        } catch { /* non-fatal */ } finally {
          statusLoading = false;
        }
      }
    } catch (e) {
      notify('error', 'Load failed', String(e));
    } finally {
      detailLoading = false;
    }
  }

  async function refreshRepos() {
    try {
      const r = await listRepositories(filterAccountUuid || undefined);
      repositories.set(r);
    } catch (e) {
      notify('error', 'Refresh failed', String(e));
    }
  }

  function statusBadgeClass(s: GitStatusEntry): string {
    if (s.status.includes('M')) return 'badge--warning';
    if (s.status.includes('A')) return 'badge--success';
    if (s.status.includes('D')) return 'badge--error';
    if (s.status === '??')     return 'badge--neutral';
    return 'badge--neutral';
  }

  function statusLabel(s: GitStatusEntry): string {
    if (s.status[0] !== ' ' && s.status[0] !== '?') return 'staged';
    if (s.status[1] === 'M') return 'modified';
    if (s.status[1] === 'D') return 'deleted';
    if (s.status === '??')   return 'untracked';
    return s.status;
  }
</script>

<!-- ── List view ────────────────────────────────────────────────────────────── -->
{#if $currentPage === 'repositories'}
  <div class="page-header">
    <div>
      <h1 class="page-title">Repositories</h1>
      <p class="page-subtitle">{clonedRepos.length} cloned · {remoteRepos.length} remote</p>
    </div>
    <div class="flex gap-2">
      <button class="btn btn--ghost btn--sm" on:click={refreshRepos}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10"/><polyline points="1 20 1 14 7 14"/>
          <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
        </svg>
        Refresh
      </button>
      <button class="btn btn--primary btn--sm" on:click={() => navigate('repositories_clone')}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
          <polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
        </svg>
        Clone Repo
      </button>
    </div>
  </div>

  <div class="page-body">
    <!-- Filter by account -->
    {#if $accounts.length > 1}
      <div class="flex gap-2 items-center mb-4">
        <span class="text-xs text-muted">Filter:</span>
        <button class="btn btn--sm {filterAccountUuid === '' ? 'btn--primary' : 'btn--secondary'}"
                on:click={() => (filterAccountUuid = '')}>All</button>
        {#each $accounts as acc (acc.uuid)}
          <button class="btn btn--sm {filterAccountUuid === acc.uuid ? 'btn--primary' : 'btn--secondary'}"
                  on:click={() => (filterAccountUuid = acc.uuid)}>{acc.alias}</button>
        {/each}
      </div>
    {/if}

    {#if clonedRepos.length > 0}
      <h2 class="text-sm font-semibold text-muted mb-2" style="text-transform:uppercase;letter-spacing:.06em">
        Cloned Locally
      </h2>
      <div class="card mb-5">
        <RepoList repos={clonedRepos} on:select={(e) => navigate('repository_detail', e.detail)} />
      </div>
    {/if}

    {#if remoteRepos.length > 0}
      <h2 class="text-sm font-semibold text-muted mb-2" style="text-transform:uppercase;letter-spacing:.06em">
        Remote Only
      </h2>
      <div class="card">
        <RepoList repos={remoteRepos} on:select={(e) => navigate('repository_detail', e.detail)} />
      </div>
    {/if}

    {#if filteredRepos.length === 0}
      <div class="empty-state card">
        <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M3 3h18M3 9h18M3 15h18M3 21h18"/>
        </svg>
        <p class="empty-title">No repositories</p>
        <p class="empty-desc">Clone a repository to start working with it locally.</p>
        <button class="btn btn--primary btn--sm" on:click={() => navigate('repositories_clone')}>
          Clone Repository
        </button>
      </div>
    {/if}
  </div>

<!-- ── Detail view ──────────────────────────────────────────────────────────── -->
{:else if $currentPage === 'repository_detail'}
  {#if detailLoading}
    <div class="empty-state"><p class="text-muted">Loading…</p></div>
  {:else if detailRepo}
    {@const repo = detailRepo}
    <div class="page-header">
      <div>
        <h1 class="page-title">{repo.name}</h1>
        <p class="page-subtitle font-mono text-xs">{repo.full_name}</p>
      </div>
      <div class="flex gap-2">
        <button class="btn btn--ghost btn--sm" on:click={() => navigate('repositories')}>← Back</button>
        {#if repo.is_cloned}
          <button class="btn btn--primary btn--sm" on:click={() => navigate('sync')}>Sync</button>
        {/if}
      </div>
    </div>

    <div class="page-body">
      <div class="grid-2 gap-4">
        <div class="card">
          <div class="card-header"><span class="card-title">Info</span></div>
          <div class="card-body">
            <dl style="display:grid;grid-template-columns:1fr 2fr;gap:var(--space-2) var(--space-3);">
              <dt class="text-xs text-muted">Status</dt>
              <dd><span class="badge {repo.is_cloned ? 'badge--info' : 'badge--neutral'}">
                {repo.is_cloned ? 'Cloned' : 'Remote'}
              </span></dd>
              <dt class="text-xs text-muted">Branch</dt>
              <dd class="font-mono text-sm">{repo.default_branch}</dd>
              {#if repo.local_path}
                <dt class="text-xs text-muted">Path</dt>
                <dd class="font-mono text-xs truncate">{repo.local_path}</dd>
              {/if}
              {#if repo.last_commit_sha}
                <dt class="text-xs text-muted">Last SHA</dt>
                <dd class="font-mono text-xs">{repo.last_commit_sha.slice(0, 8)}</dd>
              {/if}
            </dl>
          </div>
        </div>

        {#if repo.is_cloned}
          <div class="card">
            <div class="card-header">
              <span class="card-title">Working Directory</span>
              {#if statusLoading}
                <span class="text-xs text-muted">Checking…</span>
              {:else}
                <span class="text-xs text-muted">{statusEntries.length} change{statusEntries.length !== 1 ? 's' : ''}</span>
              {/if}
            </div>
            {#if statusEntries.length > 0}
              <div class="divide-y" style="max-height:240px;overflow-y:auto;">
                {#each statusEntries as entry (entry.path)}
                  <div class="flex items-center gap-3 p-3">
                    <code class="text-xs font-mono text-muted" style="width:20px">{entry.status}</code>
                    <span class="text-sm truncate flex-1">{entry.path}</span>
                    <span class="badge {statusBadgeClass(entry)}">{statusLabel(entry)}</span>
                  </div>
                {/each}
              </div>
            {:else}
              <div class="p-5 text-sm text-muted">Working directory clean</div>
            {/if}
            {#if statusEntries.length > 0}
              <div class="card-footer">
                <button class="btn btn--primary btn--sm" on:click={() => navigate('sync')}>
                  Commit & Push
                </button>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  {/if}
{/if}