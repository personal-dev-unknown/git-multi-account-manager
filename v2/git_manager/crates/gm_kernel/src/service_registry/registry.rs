// crates/gm_kernel/src/service_registry/registry.rs
//
// The ServiceRegistry is the kernel's dependency injection mechanism. It is a
// concurrent map from Rust's TypeId (the compile-time identity of a type) to an
// Arc wrapping that type as `dyn Any + Send + Sync`. Plugins register their
// concrete implementations during on_load(); other parts of the system retrieve
// them by specifying the trait or concrete type they need.
//
// ── Why TypeId and not strings ────────────────────────────────────────────────
// String-keyed registries require callers to match strings exactly, have no
// compile-time checking, and leak implementation details into the API. TypeId
// is 100% compile-time: `registry.get::<ZigGitExecutor>()` is either correct
// at compile time (the type exists and is Send+Sync+Any) or it isn't. Typos
// produce compile errors, not runtime None values.
//
// ── DashMap and concurrency ───────────────────────────────────────────────────
// DashMap shards its internal HashMap into N buckets and locks only the relevant
// shard per operation. For a registry that is written once during boot and read
// thousands of times afterwards, this means reads are effectively lock-free in
// practice. The alternative (RwLock<HashMap>) would block all readers while a
// writer holds the write lock — unacceptable if an event handler tries to look
// up a service during plugin load.
//
// ── Overwrite semantics ───────────────────────────────────────────────────────
// Registering a type that is already present OVERWRITES the existing entry.
// This is intentional: test code can inject mock implementations by registering
// after production code does. The last registration wins. In production, the
// boot order guarantees that adapters register before anything tries to use them,
// so there is never an accidental overwrite.

use dashmap::DashMap;
use std::any::{Any, TypeId};
use std::sync::Arc;

/// The kernel's type-indexed dependency injection container.
///
/// Plugins call `registry.register::<MyService>(Arc::new(impl))` during `on_load`.
/// Domain services and handlers call `registry.get::<MyService>()` to resolve them.
#[derive(Debug, Default)]
pub struct ServiceRegistry {
    map: DashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

impl ServiceRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a service under its concrete type `T`.
    ///
    /// If a service of type `T` is already registered, it is silently replaced.
    /// Callers should only register each type once in production — the overwrite
    /// behaviour exists to support test injection, not production re-registration.
    ///
    /// `T: Any + Send + Sync` ensures the type can be stored as `dyn Any` and
    /// shared safely across async task boundaries.
    pub fn register<T: Any + Send + Sync>(&self, service: Arc<T>) {
        self.map.insert(
            TypeId::of::<T>(),
            service as Arc<dyn Any + Send + Sync>,
        );
    }

    /// Retrieves a service by its concrete type `T`.
    ///
    /// Returns `Some(Arc<T>)` if a service of that type is registered, or `None`
    /// if no registration has been made for `T`. The downcast is guaranteed safe
    /// because we stored the value under exactly `TypeId::of::<T>()`.
    ///
    /// Complexity: O(1) amortised (DashMap shard lookup + single Arc clone).
    pub fn get<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.map
            .get(&TypeId::of::<T>())
            .and_then(|entry| {
                // SAFETY: We inserted this value under TypeId::of::<T>(), so
                // the concrete type behind the Arc<dyn Any> IS T. The downcast
                // cannot fail, but we still use the safe path to avoid unsafe.
                Arc::clone(entry.value())
                    .downcast::<T>()
                    .ok()
            })
    }

    /// Returns true if a service of type `T` is registered.
    pub fn contains<T: Any + Send + Sync>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<T>())
    }

    /// Returns the number of registered services.
    /// Primarily used in diagnostic and bootstrap log output.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if no services have been registered.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeService {
        value: u32,
    }

    #[test]
    fn register_and_get_returns_same_service() {
        let registry = ServiceRegistry::new();
        let svc = Arc::new(FakeService { value: 42 });
        registry.register::<FakeService>(svc);
        let retrieved = registry.get::<FakeService>().expect("service should be registered");
        assert_eq!(retrieved.value, 42);
    }

    #[test]
    fn get_unregistered_type_returns_none() {
        let registry = ServiceRegistry::new();
        assert!(registry.get::<FakeService>().is_none());
    }

    #[test]
    fn register_overwrites_previous_entry() {
        let registry = ServiceRegistry::new();
        registry.register::<FakeService>(Arc::new(FakeService { value: 1 }));
        registry.register::<FakeService>(Arc::new(FakeService { value: 2 }));
        let retrieved = registry.get::<FakeService>().unwrap();
        assert_eq!(retrieved.value, 2);
    }

    #[test]
    fn len_reflects_unique_types() {
        let registry = ServiceRegistry::new();
        assert_eq!(registry.len(), 0);
        registry.register::<FakeService>(Arc::new(FakeService { value: 0 }));
        assert_eq!(registry.len(), 1);
        // Registering the same type again does not increase the count
        registry.register::<FakeService>(Arc::new(FakeService { value: 1 }));
        assert_eq!(registry.len(), 1);
    }
}