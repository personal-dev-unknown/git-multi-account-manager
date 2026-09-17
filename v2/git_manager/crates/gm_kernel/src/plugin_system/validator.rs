// crates/gm_kernel/src/plugin_system/validator.rs
//
// The PluginValidator checks two things before any plugin is allowed to load:
//
//   1. Version compatibility — the plugin's `min_kernel_version` is compared
//      against the running kernel version. If the plugin requires a newer kernel,
//      the boot sequence fails with a clear diagnostic rather than crashing
//      inside on_load() with a confusing error about a missing symbol.
//
//   2. Dependency cycle detection — the plugin's declared dependencies are used
//      to build a directed graph and Kahn's algorithm is used to detect cycles.
//      A cycle (Plugin A depends on B, B depends on A) makes the topological
//      load order undefined. Rather than choosing an arbitrary order and getting
//      hard-to-debug "service not found" errors, the kernel fails immediately
//      with a diagnostic listing which plugins form the cycle.
//
// ── Kahn's algorithm for cycle detection ─────────────────────────────────────
// Kahn's algorithm works by repeatedly removing nodes with in-degree 0 (no
// remaining dependencies) from the graph. If at the end, some nodes remain
// with non-zero in-degree, those nodes form cycles. The remaining nodes are
// collected into the `cycle` field of the PluginError::CircularDependency error.
//
// Time complexity: O(V + E) where V is the number of plugins and E is the total
// number of declared dependencies. For typical plugin counts (< 20) this is
// effectively O(1). Even with hundreds of plugins, it runs in microseconds.

use std::collections::{HashMap, VecDeque};
use gm_shared::errors::PluginError;
use crate::contracts::plugin::PluginMetadata;

/// The kernel version string used for compatibility checks.
/// Follows semantic versioning: MAJOR.MINOR.PATCH.
pub const KERNEL_VERSION: &str = "1.0.0";

#[derive(Debug)]
pub struct PluginValidator;

impl PluginValidator {
    /// Returns Err if the plugin requires a newer kernel than the one running.
    ///
    /// The comparison is intentionally simple: only the major version is checked.
    /// A plugin that requires "1.x.x" works with any "1.y.z" kernel regardless
    /// of minor version. This follows the principle that minor and patch kernel
    /// versions are backward-compatible additions.
    pub fn check_version(metadata: &PluginMetadata) -> Result<(), PluginError> {
        let required = parse_major(&metadata.min_kernel_version);
        let running  = parse_major(KERNEL_VERSION);

        if required > running {
            return Err(PluginError::VersionIncompatible {
                plugin:   metadata.name.clone(),
                required: metadata.min_kernel_version.clone(),
                actual:   KERNEL_VERSION.to_string(),
            });
        }
        Ok(())
    }

    /// Performs topological sort validation on a list of plugin metadata.
    ///
    /// Returns the load order (sorted metadata list) on success, or
    /// `PluginError::CircularDependency` if any cycle is detected.
    /// Also returns `PluginError::DependencyMissing` if a declared dependency
    /// is not present in the provided metadata list.
    pub fn topological_sort(
        plugins: &[PluginMetadata],
    ) -> Result<Vec<PluginMetadata>, PluginError> {
        let names: std::collections::HashSet<&str> = plugins.iter()
            .map(|p| p.name.as_str())
            .collect();

        // Validate all dependencies exist before attempting the sort.
        for plugin in plugins {
            for dep in &plugin.dependencies {
                if !names.contains(dep.as_str()) {
                    return Err(PluginError::DependencyMissing {
                        plugin:   plugin.name.clone(),
                        required: dep.clone(),
                    });
                }
            }
        }

        // Build adjacency list and in-degree map for Kahn's algorithm.
        let mut in_degree:  HashMap<String, usize>      = HashMap::new();
        let mut dependents: HashMap<String, Vec<String>> = HashMap::new();

        for plugin in plugins {
            in_degree.entry(plugin.name.clone()).or_insert(0);
            for dep in &plugin.dependencies {
                // dep → plugin (dep must load before plugin)
                dependents.entry(dep.clone()).or_default().push(plugin.name.clone());
                *in_degree.entry(plugin.name.clone()).or_insert(0) += 1;
            }
        }

        // Kahn's algorithm: start with nodes of in-degree 0.
        let mut queue: VecDeque<String> = in_degree.iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(name, _)| name.clone())
            .collect();

        // Sort the initial queue to make the load order fully deterministic.
        // Without this sort, the order within a tie would depend on HashMap
        // iteration order, which is non-deterministic across Rust versions.
        let mut sorted_names: Vec<String> = queue.drain(..).collect();
        sorted_names.sort_by(|a, b| {
            // Lower priority values load first within the same dependency tier.
            let pa = plugins.iter().find(|p| &p.name == a).map(|p| p.load_priority).unwrap_or(0);
            let pb = plugins.iter().find(|p| &p.name == b).map(|p| p.load_priority).unwrap_or(0);
            pa.cmp(&pb).then(a.cmp(b)) // Priority asc, then name asc as tiebreaker
        });
        queue.extend(sorted_names);

        let mut result: Vec<String> = Vec::with_capacity(plugins.len());

        while let Some(name) = queue.pop_front() {
            result.push(name.clone());
            if let Some(deps) = dependents.get(&name) {
                let mut next_batch: Vec<String> = Vec::new();
                for dep_name in deps {
                    let deg = in_degree.get_mut(dep_name)
                        .expect("plugin validator: dependency not in in_degree map — logic error");
                    *deg -= 1;
                    if *deg == 0 {
                        next_batch.push(dep_name.clone());
                    }
                }
                // Sort each newly-unblocked batch for determinism.
                next_batch.sort_by(|a, b| {
                    let pa = plugins.iter().find(|p| &p.name == a).map(|p| p.load_priority).unwrap_or(0);
                    let pb = plugins.iter().find(|p| &p.name == b).map(|p| p.load_priority).unwrap_or(0);
                    pa.cmp(&pb).then(a.cmp(b))
                });
                queue.extend(next_batch);
            }
        }

        // If we didn't process every plugin, there must be a cycle.
        if result.len() != plugins.len() {
            let cycle: Vec<String> = in_degree.iter()
                .filter(|(_, &deg)| deg > 0)
                .map(|(name, _)| name.clone())
                .collect();
            return Err(PluginError::CircularDependency { cycle });
        }

        // Reconstruct the sorted PluginMetadata list from the sorted name list.
        let index: HashMap<&str, &PluginMetadata> = plugins.iter()
            .map(|p| (p.name.as_str(), p))
            .collect();

        Ok(result.iter()
            .filter_map(|name| index.get(name.as_str()).copied().cloned())
            .collect())
    }
}

fn parse_major(version: &str) -> u32 {
    version.split('.')
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contracts::plugin::PluginMetadata;

    fn make_meta(name: &str, deps: &[&str]) -> PluginMetadata {
        PluginMetadata {
            name:              name.to_string(),
            version:           "1.0.0".to_string(),
            description:       String::new(),
            dependencies:      deps.iter().map(|s| s.to_string()).collect(),
            min_kernel_version: "1.0.0".to_string(),
            load_priority:     0,
        }
    }

    #[test]
    fn no_deps_sorts_alphabetically() {
        let plugins = vec![
            make_meta("zebra",  &[]),
            make_meta("alpha",  &[]),
            make_meta("middle", &[]),
        ];
        let sorted = PluginValidator::topological_sort(&plugins).unwrap();
        // No deps + same priority → alphabetical order
        assert_eq!(sorted[0].name, "alpha");
    }

    #[test]
    fn dependency_appears_before_dependent() {
        let plugins = vec![
            make_meta("b", &["a"]),
            make_meta("a", &[]),
        ];
        let sorted = PluginValidator::topological_sort(&plugins).unwrap();
        let pos_a = sorted.iter().position(|p| p.name == "a").unwrap();
        let pos_b = sorted.iter().position(|p| p.name == "b").unwrap();
        assert!(pos_a < pos_b);
    }

    #[test]
    fn cycle_detected() {
        let plugins = vec![
            make_meta("a", &["b"]),
            make_meta("b", &["a"]),
        ];
        assert!(matches!(
            PluginValidator::topological_sort(&plugins),
            Err(PluginError::CircularDependency { .. })
        ));
    }

    #[test]
    fn missing_dependency_detected() {
        let plugins = vec![
            make_meta("a", &["nonexistent"]),
        ];
        assert!(matches!(
            PluginValidator::topological_sort(&plugins),
            Err(PluginError::DependencyMissing { .. })
        ));
    }
}