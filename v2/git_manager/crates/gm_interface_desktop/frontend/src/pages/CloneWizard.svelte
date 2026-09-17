<script lang="ts">
  import { accounts, repositories, navigate, notify } from '../lib/store';
  import { cloneRepository, listRemoteRepositories } from '../lib/tauri';

  // Step management
  let step: 1 | 2 | 3 | 4 = 1;

  // Step 1 — Method choice: PAT list or URL paste
  let method: 'pat' | 'url' | '' = '';

  // Step 1a — PAT: account select + remote repo list
  let patAccountUuid = $accounts[0]?.uuid ?? '';
  let remoteRepos: string[] = [];
  let loadingRepos = false;
  let remotePage = 1;
  let repoListError = '';

  // Step 1b — URL paste
  let repoUrl = '';
  let urlError = '';

  // Step 2 — Options
  let selectedAccountUuid = $accounts[0]?.uuid ?? '';
  let destination = '';
  let branch = '';
  let shallow = false;

  // Step 3 — Progress + result
  let cloning = false;
  let cloneError = '';
  let clonedName = '';

  function inferName(url: string): string {
    return url.trim().replace(/\.git$/, '').split(/[/:@]/).pop() ?? 'repo';
  }

  $: { if (repoUrl) { clonedName = inferName(repoUrl); } }

  function validateUrl(): boolean {
    if (!repoUrl.trim()) { urlError = 'URL is required.'; return false; }
    urlError = '';
    return true;
  }

  async function loadRemoteRepos(page = 1) {
    if (!patAccountUuid) { repoListError = 'Select an account first.'; return; }
    loadingRepos = true;
    repoListError = '';
    try {
      remoteRepos = await listRemoteRepositories(patAccountUuid, page);
      remotePage = page;
      if (remoteRepos.length === 0) {
        repoListError = 'No repositories found.';
      }
    } catch (e) {
      repoListError = String(e);
      remoteRepos = [];
    } finally {
      loadingRepos = false;
    }
  }

  function pickPatRepo(name: string) {
    repoUrl = name;
    selectedAccountUuid = patAccountUuid;
    clonedName = inferName(name);
    step = 2;
  }

  function startUrlMethod() {
    selectedAccountUuid = $accounts[0]?.uuid ?? '';
    step = 1;
  }

  function goToOptions() {
    if (!validateUrl()) return;
    step = 2;
  }

  async function doClone() {
    if (!selectedAccountUuid) { notify('error', 'No account', 'Select an account.'); return; }
    cloning = true;
    cloneError = '';
    try {
      const repo = await cloneRepository({
        url:          repoUrl.trim(),
        account_uuid: selectedAccountUuid,
        destination:  destination.trim() || null,
        branch:       branch.trim() || null,
        shallow,
      });
      repositories.update((r) => [...r, repo]);
      step = 4;
      notify('success', 'Cloned!', `${repo.name} cloned successfully.`);
    } catch (e) {
      cloneError = String(e);
    } finally {
      cloning = false;
    }
  }
</script>

<div class="page-header">
  <div>
    <h1 class="page-title">Clone Repository</h1>
    <p class="page-subtitle">Step {step === 4 ? 3 : step} of 3</p>
  </div>
  <button class="btn btn--ghost btn--sm" on:click={() => navigate('repositories')}>← Cancel</button>
</div>

<div class="page-body">
  <!-- Progress indicator: steps 1-2-3 (method choice/URL → options → success) -->
  <div class="flex gap-2 items-center mb-6">
    {#each [1, 2, 3] as s}
      <div style="width:28px;height:28px;border-radius:50%;display:flex;align-items:center;justify-content:center;
                  font-size:var(--text-xs);font-weight:var(--weight-bold);
                  background:{step === 4 || (s === 1 && step <= 2) || (s === 2 && step === 2) || (s === 3 && step === 4) ? 'var(--color-brand)' : step >= s ? 'var(--color-muted)' : 'var(--color-bg-elevated)'};
                  color:{step === 4 || step >= s ? 'white' : 'var(--color-text-muted)'};">
        {s === 1 && step <= 2 ? '1' : s === 2 || step === 1 ? '2' : '3'}
      </div>
      {#if s < 3}
        <div style="flex:1;height:2px;background:{step > s ? 'var(--color-brand)' : 'var(--color-border)'}"></div>
      {/if}
    {/each}
  </div>

  <div class="card card--elevated" style="max-width:540px">
    <div class="card-body">

      <!-- Step 0: Method choice (PAT list vs URL paste) -->
      {#if step === 1 && !method}
        <h2 class="font-semibold mb-4">Clone Method</h2>

        <button class="btn btn--primary btn--block mb-3" on:click={async () => { method = 'pat'; await loadRemoteRepos(); }}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
               width="16" height="16" style="margin-right:8px;vertical-align:middle;">
            <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
            <polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
          </svg>
          Clone my repositories (requires PAT)
        </button>

        <button class="btn btn--secondary btn--block" on:click={() => { method = 'url'; step = 1; }}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"
               width="16" height="16" style="margin-right:8px;vertical-align:middle;">
            <path d="M10 13a5 5 0 007.54.54l3-3a5 5 0 00-7.07-7.07l-1.72 1.71"/>
            <path d="M14 11a5 5 0 00-7.54-.54l-3 3a5 5 0 007.07 7.07l1.71-1.71"/>
          </svg>
          Paste collaborator repository URL
        </button>

      <!-- Step 1a: PAT — select account + pick repo -->
      {:else if step === 1 && method === 'pat'}
        <h2 class="font-semibold mb-4">Select Repository</h2>

        <div class="form-group">
          <label class="form-label" for="pat-account">Account</label>
          <select id="pat-account" class="select" bind:value={patAccountUuid}
                  on:change={async () => { await loadRemoteRepos(1); }}>
            {#each $accounts as acc (acc.uuid)}
              <option value={acc.uuid}>{acc.alias} ({acc.platform_name ?? 'unknown'})</option>
            {/each}
          </select>
        </div>

        {#if loadingRepos}
          <p class="text-sm text-muted">Loading repositories…</p>
        {:else if repoListError}
          <div style="background:var(--color-error-bg);color:var(--color-error-text);border-radius:var(--radius-md);padding:var(--space-3);font-size:var(--text-sm);margin-bottom:var(--space-3);">
            {repoListError}
          </div>
        {:else if remoteRepos.length > 0}
          <div style="max-height:320px;overflow-y:auto;border:1px solid var(--color-border);border-radius:var(--radius-md);">
            {#each remoteRepos as name}
              <button class="btn btn--ghost btn--block" style="text-align:left;justify-content:flex-start;padding:var(--space-2) var(--space-3);border-bottom:1px solid var(--color-border);border-radius:0;font-family:var(--font-mono);font-size:var(--text-sm);"
                      on:click={() => pickPatRepo(name)}>
                {name}
              </button>
            {/each}
          </div>
          <div class="flex gap-2 mt-3">
            <button class="btn btn--sm btn--secondary" disabled={remotePage <= 1}
                    on:click={() => loadRemoteRepos(remotePage - 1)}>← Previous</button>
            <button class="btn btn--sm btn--secondary"
                    on:click={() => loadRemoteRepos(remotePage + 1)}>Next →</button>
          </div>
        {/if}

        <div class="flex justify-start mt-4">
          <button class="btn btn--ghost" on:click={() => { method = ''; }}>← Back</button>
        </div>

      <!-- Step 1b: URL paste -->
      {:else if step === 1 && method === 'url'}
        <h2 class="font-semibold mb-4">Collaborator Repository URL</h2>

        <div class="form-group">
          <label class="form-label" for="repo-url">HTTPS or SSH URL</label>
          <input id="repo-url" class="input input--mono" bind:value={repoUrl}
                 placeholder="https://github.com/owner/repo.git"
                 class:border-error={!!urlError}
                 on:input={() => { urlError = ''; }} />
          {#if urlError}<p class="form-error">{urlError}</p>{/if}
          <p class="form-hint">
            Paste the URL of the repository to clone. HTTPS URLs are automatically
            converted to SSH for authentication with your account.
          </p>
        </div>

        <div class="flex justify-between mt-4">
          <button class="btn btn--ghost" on:click={() => { method = ''; }}>← Back</button>
          <button class="btn btn--primary" on:click={goToOptions}>Next →</button>
        </div>

      <!-- Step 2: Clone options -->
      {:else if step === 2}
        <h2 class="font-semibold mb-4">Clone Options</h2>

        <div class="form-group">
          <label class="form-label">Repository</label>
          <p class="font-mono text-sm" style="padding:var(--space-2);background:var(--color-bg-elevated);border-radius:var(--radius-md);">{repoUrl}</p>
        </div>

        <div class="form-group">
          <label class="form-label" for="clone-account">Account</label>
          <select id="clone-account" class="select" bind:value={selectedAccountUuid} required>
            {#if $accounts.length === 0}
              <option value="">— No accounts — Add one first</option>
            {/if}
            {#each $accounts as acc (acc.uuid)}
              <option value={acc.uuid}>{acc.alias} ({acc.platform_name ?? 'unknown'})</option>
            {/each}
          </select>
          <p class="form-hint">This account's SSH key is used for authentication.</p>
        </div>

        <div class="form-group">
          <label class="form-label" for="destination">Local Path (optional)</label>
          <input id="destination" class="input input--mono" bind:value={destination}
                 placeholder={`~/git-repos/${clonedName}`} />
          <p class="form-hint">Leave blank to clone into the default directory.</p>
        </div>

        <div class="form-group">
          <label class="form-label" for="branch">Branch (optional)</label>
          <input id="branch" class="input" bind:value={branch} placeholder="default branch" />
        </div>

        <label class="flex items-center gap-2 text-sm" style="margin-bottom:var(--space-4);cursor:pointer;">
          <input type="checkbox" bind:checked={shallow} />
          Shallow clone (depth=1) — faster for large repos
        </label>

        <div class="flex justify-between">
          <button class="btn btn--secondary" on:click={() => {
            if (method === 'pat') { step = 1; } else { method = ''; step = 1; }
          }}>← Back</button>
          <button class="btn btn--primary {cloning ? 'btn--loading' : ''}"
                  disabled={cloning || !selectedAccountUuid}
                  on:click={doClone}>
            {#if !cloning}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4"/>
                <polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
              </svg>
              Clone Repository
            {/if}
          </button>
        </div>

        {#if cloneError}
          <div style="margin-top:var(--space-3);background:var(--color-error-bg);color:var(--color-error-text);
                      border-radius:var(--radius-md);padding:var(--space-3);font-size:var(--text-sm);">
            {cloneError}
          </div>
        {/if}

      <!-- Step 3/4: Success -->
      {:else}
        <div style="text-align:center;padding:var(--space-8) 0;">
          <div style="width:56px;height:56px;border-radius:50%;background:var(--color-success-bg);
                      display:flex;align-items:center;justify-content:center;margin:0 auto var(--space-4);">
            <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="var(--color-success)" stroke-width="2">
              <path d="M22 11.08V12a10 10 0 11-5.93-9.14"/><path d="M22 4L12 14.01l-3-3"/>
            </svg>
          </div>
          <h2 class="font-semibold text-xl mb-2">Cloned Successfully</h2>
          <p class="text-sm text-muted mb-6">{clonedName} is ready to use.</p>
          <div class="flex gap-3 justify-center">
            <button class="btn btn--secondary" on:click={() => navigate('repositories')}>
              View Repositories
            </button>
            <button class="btn btn--primary" on:click={() => navigate('sync')}>
              Sync Now
            </button>
          </div>
        </div>
      {/if}

    </div>
  </div>
</div>