// crates/gm_adapters/src/cache/memory_cache.rs
//
// A generic, thread-safe, TTL-aware in-memory cache backed by a DashMap.
// Used primarily to cache platform API responses (repository lists, user info)
// so that the CLI feels instant for repeated operations without hammering the
// GitHub/GitLab rate limits.
//
// ── TTL and eviction ─────────────────────────────────────────────────────────
// Each entry stores an expiry instant. Eviction is lazy: expired entries are
// only removed when they are accessed (get() returns None for expired entries)
// or when purge_expired() is called explicitly. This trades peak memory
// usage (entries linger until touched) for simplicity (no background tasks,
// no Tokio dependency in this module). For a single-user tool with at most
// a few hundred cached entries this is a good trade.
//
// ── Thread safety ────────────────────────────────────────────────────────────
// DashMap provides concurrent reads without a global lock. Multiple async tasks
// can call get() simultaneously. Concurrent inserts and reads are handled by
// DashMap's internal shard locking, which has very low contention for typical
// cache access patterns (occasional inserts, frequent reads).

use std::time::{Duration, Instant};
use dashmap::DashMap;

#[derive(Debug)]
struct CacheEntry<V> {
    value:   V,
    expires: Instant,
}

/// A thread-safe, key-value, TTL-aware in-memory cache.
pub struct MemoryCache<K, V>
where
    K: Eq + std::hash::Hash,
{
    store: DashMap<K, CacheEntry<V>>,
    ttl:   Duration,
}

impl<K, V> MemoryCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    /// Creates a cache where every entry lives for `ttl` after insertion.
    pub fn new(ttl: Duration) -> Self {
        Self {
            store: DashMap::new(),
            ttl,
        }
    }

    /// Returns the cached value for `key` if it exists and has not expired.
    /// Lazily removes the entry if it is found but has expired.
    pub fn get(&self, key: &K) -> Option<V> {
        if let Some(entry) = self.store.get(key) {
            if entry.expires > Instant::now() {
                return Some(entry.value.clone());
            }
            // Expired — drop the read guard before removing to avoid deadlock
            drop(entry);
            self.store.remove(key);
        }
        None
    }

    /// Inserts a value with the configured TTL, overwriting any existing entry.
    pub fn insert(&self, key: K, value: V) {
        self.store.insert(
            key,
            CacheEntry {
                value,
                expires: Instant::now() + self.ttl,
            },
        );
    }

    /// Removes an entry by key. Returns the value if it was present and not expired.
    pub fn remove(&self, key: &K) -> Option<V> {
        self.store.remove(key).and_then(|(_, entry)| {
            if entry.expires > Instant::now() {
                Some(entry.value)
            } else {
                None
            }
        })
    }

    /// Removes all entries that have passed their TTL.
    /// Call this periodically (e.g. from a CLI command's cleanup path) to
    /// prevent unbounded memory growth in long-running server processes.
    pub fn purge_expired(&self) {
        let now = Instant::now();
        self.store.retain(|_, entry| entry.expires > now);
    }

    /// Returns the number of entries (including possibly expired ones that
    /// have not yet been evicted).
    pub fn len(&self) -> usize {
        self.store.len()
    }

    /// Returns true if the cache has no entries.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}