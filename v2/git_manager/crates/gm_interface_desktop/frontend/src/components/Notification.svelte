<script lang="ts">
  import { notifications, dismissNotification } from '../lib/store';

  function iconPath(type: string): string {
    switch (type) {
      case 'success': return 'M22 11.08V12a10 10 0 11-5.93-9.14 M22 4L12 14.01l-3-3';
      case 'error':   return 'M12 22a10 10 0 110-20 10 10 0 010 20z M12 8v4 M12 16h.01';
      case 'warning': return 'M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z M12 9v4 M12 17h.01';
      default:        return 'M12 22a10 10 0 110-20 10 10 0 010 20z M12 8v4 M12 16h.01';
    }
  }

  function colorVar(type: string): string {
    switch (type) {
      case 'success': return 'var(--color-success)';
      case 'error':   return 'var(--color-error)';
      case 'warning': return 'var(--color-warning)';
      default:        return 'var(--color-brand)';
    }
  }

  function bgVar(type: string): string {
    switch (type) {
      case 'success': return 'var(--color-success-bg)';
      case 'error':   return 'var(--color-error-bg)';
      case 'warning': return 'var(--color-warning-bg)';
      default:        return 'var(--color-info-bg)';
    }
  }

  function textVar(type: string): string {
    switch (type) {
      case 'success': return 'var(--color-success-text)';
      case 'error':   return 'var(--color-error-text)';
      case 'warning': return 'var(--color-warning-text)';
      default:        return 'var(--color-info-text)';
    }
  }
</script>

<div class="notif-container" aria-live="polite" aria-atomic="false">
  {#each $notifications as n (n.id)}
    <div class="notif animate-in"
         style="background:{bgVar(n.type)};color:{textVar(n.type)};"
         role="alert">
      <!-- Icon -->
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none"
           stroke={colorVar(n.type)} stroke-width="2"
           style="flex-shrink:0;margin-top:2px" aria-hidden="true">
        <path d={iconPath(n.type)} stroke-linecap="round" stroke-linejoin="round"/>
      </svg>

      <div style="flex:1;min-width:0;">
        <p style="font-weight:600;font-size:var(--text-sm)">{n.title}</p>
        {#if n.message}
          <p style="font-size:var(--text-xs);margin-top:2px;opacity:.85;word-break:break-word">{n.message}</p>
        {/if}
      </div>

      <button class="btn btn--ghost btn--icon btn--sm"
              style="flex-shrink:0;color:currentColor;opacity:.7;"
              on:click={() => dismissNotification(n.id)}
              aria-label="Dismiss notification">
        ×
      </button>
    </div>
  {/each}
</div>

<style>
  .notif-container {
    position: fixed;
    bottom: 20px;
    right: 20px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
    gap: 8px;
    max-width: 360px;
    pointer-events: none;
  }
  .notif {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px 14px;
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    pointer-events: all;
    font-family: var(--font-sans);
  }
</style>