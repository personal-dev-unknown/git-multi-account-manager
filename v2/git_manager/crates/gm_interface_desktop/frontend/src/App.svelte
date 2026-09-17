<script lang="ts">
  import { onMount } from 'svelte';
  import { currentPage, navigate, accounts, platforms, repositories } from './lib/store';
  import { listAccounts, listPlatforms, listRepositories } from './lib/tauri';
  import { notify } from './lib/store';

  import Notification from './components/Notification.svelte';
  import Dashboard     from './pages/Dashboard.svelte';
  import Accounts      from './pages/Accounts.svelte';
  import Repositories  from './pages/Repositories.svelte';
  import SSH           from './pages/SSH.svelte';
  import CloneWizard   from './pages/CloneWizard.svelte';
  import SyncView      from './pages/SyncView.svelte';

  // Load initial data when the app mounts.
  onMount(async () => {
    try {
      const [accs, plats, repos] = await Promise.all([
        listAccounts(),
        listPlatforms(),
        listRepositories(),
      ]);
      accounts.set(accs);
      platforms.set(plats);
      repositories.set(repos);
    } catch (e) {
      notify('error', 'Startup failed', String(e), 0);
    }
  });

  // Reactive page label for the topbar breadcrumb.
  const pageLabels: Record<string, string> = {
    dashboard:          'Dashboard',
    accounts:           'Accounts',
    accounts_new:       'Add Account',
    account_detail:     'Account',
    repositories:       'Repositories',
    repositories_clone: 'Clone Repository',
    repository_detail:  'Repository',
    ssh:                'SSH Keys',
    ssh_generate:       'Generate SSH Key',
    sync:               'Sync',
  };
</script>

<div class="app-layout" data-tauri-drag-region>

  <!-- ── Sidebar ──────────────────────────────────────────────────────────── -->
  <nav class="sidebar" aria-label="Main navigation">
    <!-- Logo — also acts as the Tauri drag region for the window. -->
    <div class="sidebar-logo drag-region" aria-hidden="true">
      <svg width="22" height="22" viewBox="0 0 24 24" fill="none"
           stroke="currentColor" stroke-width="2">
        <circle cx="12" cy="12" r="4"/>
        <path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41
                 M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/>
      </svg>
      Git Manager
    </div>

    <div class="sidebar-nav no-drag">
      <!-- Dashboard -->
      <button class="nav-item {$currentPage === 'dashboard' ? 'active' : ''}"
              on:click={() => navigate('dashboard')} aria-current={$currentPage === 'dashboard' ? 'page' : undefined}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/>
          <rect x="14" y="14" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/>
        </svg>
        Dashboard
      </button>

      <!-- Accounts -->
      <button class="nav-item {$currentPage === 'accounts' || $currentPage === 'accounts_new' || $currentPage === 'account_detail' ? 'active' : ''}"
              on:click={() => navigate('accounts')}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2"/>
          <circle cx="12" cy="7" r="4"/>
        </svg>
        Accounts
        {#if $accounts.length > 0}
          <span class="badge badge--neutral" style="margin-left:auto;padding:0 6px">{$accounts.length}</span>
        {/if}
      </button>

      <!-- Repositories -->
      <button class="nav-item {$currentPage === 'repositories' || $currentPage === 'repositories_clone' || $currentPage === 'repository_detail' ? 'active' : ''}"
              on:click={() => navigate('repositories')}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 3h18M3 9h18M3 15h18M3 21h18"/>
        </svg>
        Repositories
      </button>

      <!-- SSH Keys -->
      <button class="nav-item {$currentPage === 'ssh' || $currentPage === 'ssh_generate' ? 'active' : ''}"
              on:click={() => navigate('ssh')}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0
                   L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4"/>
        </svg>
        SSH Keys
      </button>

      <!-- Sync -->
      <button class="nav-item {$currentPage === 'sync' ? 'active' : ''}"
              on:click={() => navigate('sync')}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <polyline points="23 4 23 10 17 10"/>
          <polyline points="1 20 1 14 7 14"/>
          <path d="M3.51 9a9 9 0 0114.85-3.36L23 10M1 14l4.64 4.36A9 9 0 0020.49 15"/>
        </svg>
        Sync
      </button>
    </div>

    <!-- Footer: theme toggle -->
    <div class="sidebar-footer no-drag">
      <button class="btn btn--ghost btn--sm w-full" style="justify-content:flex-start;gap:var(--space-3);"
              on:click={() => {
                const html = document.documentElement;
                const current = html.getAttribute('data-theme') || 'auto';
                const next = current === 'dark' ? 'light' : 'dark';
                html.setAttribute('data-theme', next);
                localStorage.setItem('gm-theme', next);
              }}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" width="14" height="14">
          <path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z"/>
        </svg>
        Toggle theme
      </button>
    </div>
  </nav>

  <!-- ── Main area ─────────────────────────────────────────────────────────── -->
  <div class="main-area">
    <!-- Topbar -->
    <header class="topbar drag-region">
      <span class="text-sm text-muted no-drag">{pageLabels[$currentPage] ?? $currentPage}</span>
      <div class="flex items-center gap-2 no-drag">
        {#if $currentPage !== 'accounts_new'}
          <button class="btn btn--primary btn--sm" on:click={() => navigate('accounts_new')}>
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 5v14M5 12h14"/>
            </svg>
            Add Account
          </button>
        {/if}
      </div>
    </header>

    <!-- Notifications overlay -->
    <Notification />

    <!-- Page router -->
    <div class="page-scroll">
      {#if $currentPage === 'dashboard'}
        <Dashboard />
      {:else if $currentPage === 'accounts' || $currentPage === 'accounts_new' || $currentPage === 'account_detail'}
        <Accounts />
      {:else if $currentPage === 'repositories' || $currentPage === 'repository_detail'}
        <Repositories />
      {:else if $currentPage === 'repositories_clone'}
        <CloneWizard />
      {:else if $currentPage === 'ssh' || $currentPage === 'ssh_generate'}
        <SSH />
      {:else if $currentPage === 'sync'}
        <SyncView />
      {/if}
    </div>
  </div>
</div>

<script lang="ts" context="module">
  // Restore theme preference before first render to prevent flash of wrong theme.
  if (typeof localStorage !== 'undefined') {
    const stored = localStorage.getItem('gm-theme');
    if (stored === 'dark' || stored === 'light') {
      document.documentElement.setAttribute('data-theme', stored);
    }
  }
</script>