<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { RepositoryDto } from '../lib/types';

  export let repos: RepositoryDto[] = [];

  const dispatch = createEventDispatcher<{ select: string }>();

  function visibilityBadge(v: string) {
    return v === 'Public' ? 'badge--neutral' : 'badge--info';
  }
</script>

{#if repos.length === 0}
  <div class="p-5 text-sm text-muted">No repositories to display.</div>
{:else}
  <div class="divide-y">
    {#each repos as repo (repo.uuid)}
      <button class="repo-row" on:click={() => dispatch('select', repo.uuid)}>
        <!-- Repo icon -->
        <div class="repo-icon">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none"
               stroke="var(--color-text-muted)" stroke-width="2">
            <path d="M3 3h18M3 9h18M3 15h18M3 21h18"/>
          </svg>
        </div>

        <!-- Name and path -->
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <span class="font-medium text-sm truncate">{repo.name}</span>
            {#if repo.is_archived}
              <span class="badge badge--neutral" style="font-size:10px">archived</span>
            {/if}
          </div>
          {#if repo.local_path}
            <p class="text-xs text-muted font-mono truncate">{repo.local_path}</p>
          {:else}
            <p class="text-xs text-muted truncate">{repo.full_name}</p>
          {/if}
        </div>

        <!-- Branch -->
        <code class="text-xs font-mono text-muted" style="flex-shrink:0">{repo.default_branch}</code>

        <!-- Visibility -->
        <span class="badge {visibilityBadge(repo.visibility)}" style="flex-shrink:0">
          {repo.visibility}
        </span>

        <!-- Clone status -->
        <span class="badge {repo.is_cloned ? 'badge--info' : 'badge--neutral'}" style="flex-shrink:0">
          {repo.is_cloned ? 'local' : 'remote'}
        </span>
      </button>
    {/each}
  </div>
{/if}

<style>
  .repo-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 16px;
    width: 100%;
    text-align: left;
    border: none;
    background: transparent;
    cursor: pointer;
    transition: background 100ms ease;
    color: inherit;
  }
  .repo-row:hover { background: var(--color-bg-elevated); }
  .repo-icon {
    width: 28px;
    height: 28px;
    border-radius: var(--radius-md);
    background: var(--color-bg-elevated);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }
</style>