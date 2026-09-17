<script lang="ts">
  /** 0–100 for determinate mode. Ignored when `indeterminate` is true. */
  export let value:         number       = 0;
  export let indeterminate: boolean      = false;
  export let label:         string       = '';
  export let variant:       'default' | 'success' | 'error' = 'default';

  $: barClass = variant === 'success'
    ? 'progress-bar--success'
    : variant === 'error'
    ? 'progress-bar--error'
    : '';

  $: pct = Math.min(100, Math.max(0, value));
</script>

{#if label}
  <div class="flex justify-between items-center mb-1">
    <span class="text-xs text-muted">{label}</span>
    {#if !indeterminate}
      <span class="text-xs text-muted">{pct}%</span>
    {/if}
  </div>
{/if}

<div class="progress" role="progressbar"
     aria-label={label || 'progress'}
     aria-valuenow={indeterminate ? undefined : pct}
     aria-valuemin={0}
     aria-valuemax={100}>
  <div class="progress-bar {barClass} {indeterminate ? 'progress-bar--indeterminate' : ''}"
       style={indeterminate ? '' : `width: ${pct}%`}>
  </div>
</div>