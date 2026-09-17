/**
 * Global Svelte stores for the Git Manager desktop application.
 *
 * The store layer is intentionally thin — it holds only the data that multiple
 * page components need to share. Data that belongs to a single page is kept
 * local to that page's component with Svelte's `let` declarations.
 *
 * Stores are plain Svelte `writable()` values. There is no Redux-style action
 * dispatch layer: components call the typed Tauri API wrappers from `tauri.ts`
 * directly and update the relevant store after a successful call. This keeps
 * the data flow predictable and avoids over-engineering for a single-user
 * desktop application.
 */

import { writable, derived } from 'svelte/store';
import type {
  AccountDto,
  RepositoryDto,
  Notification,
  Page,
  Platform,
} from './types';

// ── Navigation ────────────────────────────────────────────────────────────────

/** The currently displayed page. Components subscribe to this to show/hide. */
export const currentPage = writable<Page>('dashboard');

/**
 * When navigating to a detail page (account_detail, repository_detail),
 * this holds the UUID of the selected resource so the detail page can
 * load it. The page component is responsible for clearing it on unmount.
 */
export const selectedUuid = writable<string | null>(null);

// ── Accounts ──────────────────────────────────────────────────────────────────

/** All accounts loaded from the backend. */
export const accounts = writable<AccountDto[]>([]);

/** Platform metadata from the list_platforms command. */
export const platforms = writable<Platform[]>([]);

/** Look up a platform by UUID. Used by components that only know the platform_id. */
export const platformMap = derived(platforms, ($platforms) =>
  Object.fromEntries($platforms.map((p) => [p.uuid, p]))
);

// ── Repositories ──────────────────────────────────────────────────────────────

/** All repositories currently known to the system. */
export const repositories = writable<RepositoryDto[]>([]);

// ── Loading state ─────────────────────────────────────────────────────────────

/**
 * Global loading flag. Set to true before an async operation, false after.
 * The Notification component uses this to show a spinner in the topbar.
 * Individual page components track per-operation loading with local `let` state.
 */
export const globalLoading = writable<boolean>(false);

// ── Notifications ─────────────────────────────────────────────────────────────

/** Queue of notifications shown in the top-right corner. */
export const notifications = writable<Notification[]>([]);

let _notifId = 0;

/** Push a new notification. It auto-dismisses after `durationMs` milliseconds. */
export function notify(
  type: Notification['type'],
  title: string,
  message: string,
  durationMs = 5000
): void {
  const id = ++_notifId;
  notifications.update((n) => [...n, { id, type, title, message }]);
  if (durationMs > 0) {
    setTimeout(() => dismissNotification(id), durationMs);
  }
}

/** Remove a notification by its id. */
export function dismissNotification(id: number): void {
  notifications.update((n) => n.filter((x) => x.id !== id));
}

// ── Navigation helpers ────────────────────────────────────────────────────────

/** Navigate to a page, optionally setting the selected resource UUID. */
export function navigate(page: Page, uuid?: string): void {
  selectedUuid.set(uuid ?? null);
  currentPage.set(page);
}