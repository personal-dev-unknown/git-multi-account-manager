<script lang="ts">
  import { accounts, currentPage, selectedUuid, navigate, notify } from '../lib/store';
  import {
    listSshKeys, generateSshKey, testSshConnection,
    addKeyToAgent, getPublicKey,
  } from '../lib/tauri';
  import SshKeyRow from '../components/SshKeyRow.svelte';
  import type { SshKeyDto } from '../lib/types';

  // The SSH page shows keys for a specific account (from selectedUuid)
  // or, if no account selected, prompts the user to pick one.
  let selectedAccountUuid: string = $selectedUuid ?? ($accounts[0]?.uuid ?? '');
  $: selectedAccount = $accounts.find((a) => a.uuid === selectedAccountUuid);

  let keys: SshKeyDto[] = [];
  let keysLoading = true;
  let publicKeyText = '';
  let showPublicKey = false;

  // Generate form
  let genKeyType: 'ed25519' | 'rsa' = 'ed25519';
  let genPassphrase = '';
  let genAddToAgent = true;
  let genLoading = false;

  // Test state
  let testLoading = false;
  let testResult: { success: boolean; username: string | null; error: string | null } | null = null;

  async function loadKeys() {
    if (!selectedAccountUuid) return;
    keysLoading = true;
    try {
      keys = await listSshKeys(selectedAccountUuid);
    } catch (e) {
      notify('error', 'Failed to load keys', String(e));
    } finally {
      keysLoading = false;
    }
  }

  $: { if (selectedAccountUuid) { void loadKeys(); } }

  async function handleGenerate() {
    if (!selectedAccountUuid) return;
    genLoading = true;
    try {
      const key = await generateSshKey({
        account_uuid: selectedAccountUuid,
        key_type:     genKeyType,
        comment:      null,
        passphrase:   genPassphrase || null,
        add_to_agent: genAddToAgent,
      });
      keys = [...keys.map((k) => ({ ...k, is_active: false })), { ...key, is_active: true }];
      publicKeyText = key.public_key;
      showPublicKey = true;
      genPassphrase = '';
      notify('success', 'SSH Key Generated', `Key "${key.name}" created. Add the public key to your platform.`);
    } catch (e) {
      notify('error', 'Key generation failed', String(e));
    } finally {
      genLoading = false;
    }
  }

  async function handleTest() {
    if (!selectedAccountUuid) return;
    testLoading = true;
    testResult = null;
    try {
      testResult = await testSshConnection(selectedAccountUuid);
      if (testResult.success) {
        notify('success', 'Connection OK', `Authenticated as ${testResult.username ?? 'unknown'}.`);
      } else {
        notify('warning', 'Connection failed', testResult.error ?? 'Unknown failure');
      }
    } catch (e) {
      notify('error', 'Test failed', String(e));
    } finally {
      testLoading = false;
    }
  }

  async function handleAddToAgent() {
    if (!selectedAccountUuid) return;
    try {
      await addKeyToAgent(selectedAccountUuid);
      notify('success', 'Key added', 'SSH key added to the agent.');
    } catch (e) {
      notify('error', 'Agent add failed', String(e));
    }
  }

  async function showPubKey() {
    if (!selectedAccountUuid) return;
    try {
      publicKeyText = (await getPublicKey(selectedAccountUuid)) ?? '';
      showPublicKey = !!publicKeyText;
    } catch (e) {
      notify('error', 'Failed', String(e));
    }
  }

  async function copyPubKey() {
    if (!publicKeyText) return;
    try {
      await navigator.clipboard.writeText(publicKeyText);
      notify('success', 'Copied', 'Public key copied to clipboard.');
    } catch { /* fallback */ }
  }
</script>

<div class="page-header">
  <div>
    <h1 class="page-title">SSH Keys</h1>
    <p class="page-subtitle">Generate and manage SSH keys for each account</p>
  </div>
</div>

<div class="page-body">

  <!-- Account selector -->
  {#if $accounts.length > 0}
    <div class="form-group" style="max-width:320px;margin-bottom:var(--space-5);">
      <label class="form-label" for="account-select">Account</label>
      <select id="account-select" class="select" bind:value={selectedAccountUuid}>
        {#each $accounts as acc (acc.uuid)}
          <option value={acc.uuid}>{acc.alias} ({acc.platform_name ?? 'unknown'})</option>
        {/each}
      </select>
    </div>
  {/if}

  {#if !selectedAccountUuid}
    <div class="empty-state card">
      <p class="empty-title">No account selected</p>
      <button class="btn btn--primary btn--sm" on:click={() => navigate('accounts_new')}>Add Account</button>
    </div>
  {:else}
    <div class="grid-2 gap-4">

      <!-- Key list -->
      <div class="card">
        <div class="card-header">
          <span class="card-title">Keys for {selectedAccount?.alias ?? '…'}</span>
          <div class="flex gap-2">
            <button class="btn btn--secondary btn--sm {testLoading ? 'btn--loading' : ''}"
                    disabled={testLoading || keys.filter((k) => k.is_active).length === 0}
                    on:click={handleTest}>
              {#if !testLoading}Test Connection{/if}
            </button>
            <button class="btn btn--ghost btn--sm" on:click={showPubKey}>Show Key</button>
          </div>
        </div>

        {#if keysLoading}
          <div class="p-5 text-muted text-sm">Loading…</div>
        {:else if keys.length > 0}
          <div class="divide-y">
            {#each keys as key (key.uuid)}
              <SshKeyRow {key} />
            {/each}
          </div>
        {:else}
          <div class="empty-state" style="padding:var(--space-8);">
            <p class="empty-desc">No SSH keys yet. Generate one below.</p>
          </div>
        {/if}

        {#if testResult}
          <div class="card-footer">
            {#if testResult.success}
              <span class="badge badge--success">Connected as {testResult.username ?? 'unknown'}</span>
            {:else}
              <span class="badge badge--error">{testResult.error ?? 'Connection failed'}</span>
            {/if}
          </div>
        {/if}
      </div>

      <!-- Generate new key -->
      <div class="card">
        <div class="card-header"><span class="card-title">Generate SSH Key</span></div>
        <div class="card-body">
          <div class="form-group">
            <label class="form-label" for="key-type">Key Type</label>
            <select id="key-type" class="select" bind:value={genKeyType}>
              <option value="ed25519">Ed25519 (recommended)</option>
              <option value="rsa">RSA 4096</option>
            </select>
          </div>
          <div class="form-group">
            <label class="form-label" for="passphrase">Passphrase (optional)</label>
            <input id="passphrase" class="input" type="password" bind:value={genPassphrase}
                   placeholder="Leave blank for passwordless" autocomplete="new-password" />
          </div>
          <label class="flex items-center gap-2 text-sm" style="margin-bottom:var(--space-4);cursor:pointer;">
            <input type="checkbox" bind:checked={genAddToAgent} />
            Add to SSH agent automatically
          </label>

          <button class="btn btn--primary w-full {genLoading ? 'btn--loading' : ''}"
                  disabled={genLoading} on:click={handleGenerate}>
            {#if !genLoading}
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778z"/>
              </svg>
              Generate Key Pair
            {/if}
          </button>

          <button class="btn btn--ghost btn--sm w-full mt-2" on:click={handleAddToAgent}>
            Add Active Key to Agent
          </button>
        </div>

        <!-- Public key display after generation -->
        {#if showPublicKey && publicKeyText}
          <div class="card-footer">
            <div class="form-group" style="margin:0">
              <div class="flex justify-between items-center mb-2">
                <span class="form-label" style="margin:0">Public Key — Add to your platform</span>
                <button class="btn btn--secondary btn--sm" on:click={copyPubKey}>Copy</button>
              </div>
              <div class="pub-key selectable">{publicKeyText}</div>
            </div>
          </div>
        {/if}
      </div>

    </div>
  {/if}

</div>