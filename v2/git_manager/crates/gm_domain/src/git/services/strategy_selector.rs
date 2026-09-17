// crates/gm_domain/src/git/services/strategy_selector.rs
//
// Default implementation of StrategySelector that optimizes clone
// settings based on repository analysis and connection quality.

use crate::git::entities::repository_analysis::{RepoSizeClass, RepositoryAnalysis};
use crate::git::ports::git_executor::{CloneOptions, TagsMode};
use crate::git::ports::repository_analyzer::{BandwidthClass, CloneStrategy, ConnectionQuality, StrategySelector};

/// The default strategy selector implementing domain logic for
/// choosing optimal clone settings.
#[derive(Debug, Default)]
pub struct DefaultStrategySelector;

impl StrategySelector for DefaultStrategySelector {
    fn select_strategy(
        &self,
        analysis: &RepositoryAnalysis,
        quality: &ConnectionQuality,
    ) -> CloneStrategy {
        if !quality.is_reachable {
            return CloneStrategy::Full;
        }

        if matches!(analysis.size_class, RepoSizeClass::Huge)
            && quality.bandwidth_class.is_slow()
        {
            return CloneStrategy::Partial("blob:none");
        }

        if analysis.shallow_recommended() && quality.bandwidth_class.is_slow() {
            return CloneStrategy::Shallow(1);
        }

        if analysis.partial_clone_recommended() {
            return CloneStrategy::Partial("blob:none");
        }

        CloneStrategy::Full
    }

    fn optimize_options(
        &self,
        opts: &CloneOptions,
        analysis: &RepositoryAnalysis,
        quality: &ConnectionQuality,
    ) -> CloneOptions {
        let mut optimized = opts.clone();

        match self.select_strategy(analysis, quality) {
            CloneStrategy::Full => {
                optimized.depth = None;
                optimized.filter = None;
            }
            CloneStrategy::Shallow(d) => {
                if opts.depth.is_none() || opts.depth.unwrap_or(0) > d {
                    optimized.depth = Some(d);
                }
            }
            CloneStrategy::Partial(filter) => {
                optimized.filter = Some(filter.to_string());
                optimized.single_branch = true;
            }
            CloneStrategy::Sparse(paths) => {
                optimized.sparse_checkout = Some(paths);
                optimized.filter = Some("blob:none".to_string());
                optimized.no_checkout = true;
            }
            CloneStrategy::DepthOverride(d) => {
                optimized.depth = Some(d);
            }
        }

        if quality.bandwidth_class.is_slow() && !opts.bare && !opts.mirror {
            optimized.single_branch = true;
        }

        if matches!(quality.bandwidth_class, BandwidthClass::DialUp) {
            optimized.tags_mode = TagsMode::None;
        }

        optimized
    }
}
