<script lang="ts">
  import { accounts, repositories, navigate } from '../lib/store';
  import type { AccountDto, RepositoryDto } from '../lib/types';

  $: totalAccounts = $accounts.length;
  $: activeAccounts = $accounts.filter((a) => a.is_active).length;
  $: clonedRepos = $repositories.filter((r) => r.is_cloned).length;
  $: totalRepos = $repositories.length;
  $: recentAccounts = $accounts.slice(0, 5);
  $: recentRepos = $repositories.filter((r) => r.is_cloned).slice(0, 5);

  function platformBadgeClass(name: string | null): string {
    switch (name?.toLowerCase()) {
      case 'github':       return 'badge--neutral';
      case 'gitlab':       return 'badge--warning';
      case 'bitbucket':    return 'badge--info';
      case 'azure devops': return 'badge--info';
      default:             return 'badge--neutral';
    }
  }

  function accountInitials(alias: string): string {
    return alias.slice(0, 2).toUpperCase();
  }
</script>

<div class="page-header">
  <div>
    <h1 class="page-title">Dashboard</h1>
    <p class="page-subtitle">Overview of your Git workspace</p>
  </div>
  <div class="flex gap-2">
    <button class="btn btn--secondary btn--sm" on:click={() => navigate('repositories_clone')}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
        <polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
      </svg>
      Clone Repo
    </button>
    <button class="btn btn--primary btn--sm" on:click={() => navigate('accounts_new')}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
      Add Account
    </button>
  </div>
</div>

<div class="page-body">

  <!-- Stats row -->
  <div class="grid-3 mb-4">
    <div class="card">
      <div class="card-body flex gap-4 items-center">
        <div style="width:44px;height:44px;border-radius:12px;background:var(--color-brand-muted);
                    display:flex;align-items:center;justify-content:center;flex-shrink:0;">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none"
               stroke="var(--color-brand)" stroke-width="2">
            <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/>
          </svg>
        </div>
        <div>
          <p class="text-xs text-muted">Accounts</p>
          <p style="font-size:var(--text-2xl);font-weight:var(--weight-bold)">{totalAccounts}</p>
          {#if activeAccounts < totalAccounts}
            <p class="text-xs text-muted">{activeAccounts} active</p>
          {/if}
        </div>
      </div>
    </div>

    <div class="card">
      <div class="card-body flex gap-4 items-center">
        <div style="width:44px;height:44px;border-radius:12px;background:var(--color-success-bg);
                    display:flex;align-items:center;justify-content:center;flex-shrink:0;">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none"
               stroke="var(--color-success)" stroke-width="2">
            <path d="M3 3h18M3 9h18M3 15h18M3 21h18"/>
          </svg>
        </div>
        <div>
          <p class="text-xs text-muted">Cloned Repos</p>
          <p style="font-size:var(--text-2xl);font-weight:var(--weight-bold)">{clonedRepos}</p>
          {#if totalRepos > clonedRepos}
            <p class="text-xs text-muted">{totalRepos} total known</p>
          {/if}
        </div>
      </div>
    </div>

    <div class="card">
      <div class="card-body flex gap-4 items-center">
        <div style="width:44px;height:44px;border-radius:12px;background:var(--color-warning-bg);
                    display:flex;align-items:center;justify-content:center;flex-shrink:0;">
          <svg width="22" height="22" viewBox="0 0 24 24" fill="none"
               stroke="var(--color-warning)" stroke-width="2">
            <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777z"/>
          </svg>
        </div>
        <div>
          <p class="text-xs text-muted">Active SSH Keys</p>
          <p style="font-size:var(--text-2xl);font-weight:var(--weight-bold)">{$accounts.length}</p>
          <p class="text-xs text-muted">one per account</p>
        </div>
      </div>
    </div>
  </div>

  <div class="grid-2 gap-4">

    <!-- Recent accounts -->
    <div class="card">
      <div class="card-header">
        <span class="card-title">Accounts</span>
        <button class="btn btn--ghost btn--sm" on:click={() => navigate('accounts')}>View all</button>
      </div>
      {#if recentAccounts.length > 0}
        <div class="divide-y">
          {#each recentAccounts as account (account.uuid)}
            <button class="w-full flex items-center gap-3 p-4"
                    style="background:transparent;border:none;cursor:pointer;text-align:left;"
                    on:click={() => { navigate('account_detail', account.uuid); }}>
              <div class="avatar">{accountInitials(account.alias)}</div>
              <div class="min-w-0 flex-1">
                <p class="font-medium truncate">{account.alias}</p>
                <p class="text-xs text-muted truncate">{account.username}</p>
              </div>
              <span class="badge {platformBadgeClass(account.platform_name)}" style="flex-shrink:0">
                {account.platform_name ?? '—'}
              </span>
            </button>
          {/each}
        </div>
      {:else}
        <div class="empty-state">
          <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/><circle cx="12" cy="7" r="4"/>
          </svg>
          <p class="empty-title" style="font-size:var(--text-base)">No accounts</p>
          <button class="btn btn--primary btn--sm" on:click={() => navigate('accounts_new')}>Add Account</button>
        </div>
      {/if}
    </div>

    <!-- Recent repositories -->
    <div class="card">
      <div class="card-header">
        <span class="card-title">Local Repositories</span>
        <button class="btn btn--ghost btn--sm" on:click={() => navigate('repositories')}>View all</button>
      </div>
      {#if recentRepos.length > 0}
        <div class="divide-y">
          {#each recentRepos as repo (repo.uuid)}
            <button class="w-full flex items-center gap-3 p-4"
                    style="background:transparent;border:none;cursor:pointer;text-align:left;"
                    on:click={() => navigate('repository_detail', repo.uuid)}>
              <div style="width:32px;height:32px;border-radius:var(--radius-md);background:var(--color-bg-elevated);
                          display:flex;align-items:center;justify-content:center;flex-shrink:0;">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--color-text-muted)" stroke-width="2">
                  <path d="M3 3h18M3 9h18M3 15h18M3 21h18"/>
                </svg>
              </div>
              <div class="min-w-0 flex-1">
                <p class="font-medium truncate">{repo.name}</p>
                <p class="text-xs text-muted font-mono truncate">{repo.default_branch}</p>
              </div>
              <span class="badge badge--info" style="flex-shrink:0">Cloned</span>
            </button>
          {/each}
        </div>
      {:else}
        <div class="empty-state">
          <svg class="empty-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M3 3h18M3 9h18M3 15h18M3 21h18"/>
          </svg>
          <p class="empty-title" style="font-size:var(--text-base)">No local repos</p>
          <button class="btn btn--primary btn--sm" on:click={() => navigate('repositories_clone')}>Clone Repository</button>
        </div>
      {/if}
    </div>
  </div>

</div>