<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { AccountDto } from '../lib/types';

  export let account: AccountDto;

  const dispatch = createEventDispatcher<{ click: void; remove: void }>();

  function platformClass(name: string | null): string {
    switch (name?.toLowerCase()) {
      case 'github':       return 'badge--neutral';
      case 'gitlab':       return 'badge--warning';
      case 'bitbucket':    return 'badge--info';
      case 'azure devops': return 'badge--info';
      default:             return 'badge--neutral';
    }
  }
</script>

<div class="account-row" role="button" tabindex="0"
     on:click={() => dispatch('click')}
     on:keydown={(e) => e.key === 'Enter' && dispatch('click')}>
  <div class="avatar">{account.alias.slice(0, 2).toUpperCase()}</div>

  <div class="account-info">
    <div class="flex items-center gap-2">
      <span class="font-medium text-sm">{account.alias}</span>
      {#if account.is_default}
        <span class="badge badge--success" style="font-size:10px;padding:1px 5px">default</span>
      {/if}
    </div>
    <p class="text-xs text-muted truncate">{account.username} · {account.email}</p>
  </div>

  <span class="badge {platformClass(account.platform_name)}" style="flex-shrink:0">
    {account.platform_name ?? '—'}
  </span>

  <span class="badge {account.is_active ? 'badge--success' : 'badge--neutral'}" style="flex-shrink:0">
    {account.is_active ? 'Active' : 'Inactive'}
  </span>

  <button class="btn btn--danger btn--sm no-propagate" style="flex-shrink:0"
          on:click|stopPropagation={() => dispatch('remove')}>
    Remove
  </button>
</div>

<style>
  .account-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 16px;
    cursor: pointer;
    transition: background 100ms ease;
    border: none;
    background: transparent;
    width: 100%;
    text-align: left;
  }
  .account-row:hover { background: var(--color-bg-elevated); }
  .account-info { flex: 1; min-width: 0; }
</style>