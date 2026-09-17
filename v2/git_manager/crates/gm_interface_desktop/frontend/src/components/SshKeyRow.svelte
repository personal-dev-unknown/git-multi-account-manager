<script lang="ts">
  import type { SshKeyDto } from '../lib/types';

  export let key: SshKeyDto;

  $: dotClass = key.last_test_status === 'Success'
    ? 'key-dot--active'
    : key.last_test_status === 'Failed'
    ? 'key-dot--failed'
    : 'key-dot--inactive';

  $: statusLabel = key.last_test_status === 'Success'
    ? 'Verified'
    : key.last_test_status === 'Failed'
    ? 'Failed'
    : 'Not tested';

  function truncateSha(fp: string): string {
    return fp.length > 30 ? fp.slice(0, 30) + '…' : fp;
  }
</script>

<div class="key-row">
  <!-- Status dot -->
  <span class="key-dot {dotClass}" title={statusLabel}></span>

  <!-- Key info -->
  <div class="min-w-0 flex-1">
    <div class="flex items-center gap-2">
      <span class="font-medium text-sm">{key.name}</span>
      {#if key.is_active}
        <span class="badge badge--success" style="font-size:10px;padding:1px 5px">active</span>
      {/if}
      {#if key.is_added_to_agent}
        <span class="badge badge--info" style="font-size:10px;padding:1px 5px">in agent</span>
      {/if}
    </div>
    <p class="text-xs text-muted font-mono" style="margin-top:2px">{truncateSha(key.fingerprint)}</p>
  </div>

  <!-- Key type -->
  <span class="text-xs text-muted font-mono" style="flex-shrink:0">{key.key_type}</span>

  <!-- Test status -->
  <span class="badge {key.last_test_status === 'Success' ? 'badge--success' : key.last_test_status === 'Failed' ? 'badge--error' : 'badge--neutral'}"
        style="flex-shrink:0">
    {statusLabel}
  </span>
</div>

<style>
  .key-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    transition: background 100ms ease;
  }
  .key-row:hover { background: var(--color-bg-elevated); }
</style>