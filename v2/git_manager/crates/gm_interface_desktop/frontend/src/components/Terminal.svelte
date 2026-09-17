<script lang="ts">
  import { afterUpdate } from 'svelte';

  /** Each log line from SyncView — { time, cls, text } */
  export let lines: Array<{ time: string; cls: string; text: string }> = [];

  let container: HTMLDivElement;

  // Auto-scroll to bottom when new lines appear.
  afterUpdate(() => {
    if (container) {
      container.scrollTop = container.scrollHeight;
    }
  });
</script>

<div class="terminal" bind:this={container} aria-live="polite" aria-label="Operation log">
  {#if lines.length === 0}
    <span class="terminal-info" style="opacity:.5">Ready.</span>
  {:else}
    {#each lines as line (line.time + line.text)}
      <div class="terminal-line">
        <span class="terminal-time">{line.time}</span>
        <span class={line.cls}>{line.text}</span>
      </div>
    {/each}
  {/if}
</div>