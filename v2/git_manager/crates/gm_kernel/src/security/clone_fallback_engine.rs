use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

use gm_domain::git::ports::git_executor::{CloneOptions, CloneResult, GitExecutor as GitExecutorTrait};
use gm_domain::git::ports::{CloneProgress, MemoryGuard, ProgressCallback, noop_progress, DiskSpaceChecker};
use gm_shared::errors::{GitError, GitManagerError};

use crate::security::{AuthStrategy, Cancelled, CloneUrlResolver, ResolvedStrategy};

const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_BASE_DELAY_MS: u64 = 1000;

/// Multi-strategy clone execution engine with progress reporting, disk space
/// validation, memory protection, exponential backoff, and Ctrl+C cancellation.
pub struct CloneFallbackEngine {
    executor:       Arc<dyn GitExecutorTrait>,
    url_resolver:   Arc<CloneUrlResolver>,
    max_retries:    u32,
    base_delay_ms:  u64,
    on_progress:    ProgressCallback,
    disk_checker:   Option<Arc<dyn DiskSpaceChecker>>,
    memory_guard:   MemoryGuard,
    cancelled:      Cancelled,
}

impl std::fmt::Debug for CloneFallbackEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CloneFallbackEngine")
            .field("executor", &"<GitExecutor>")
            .field("url_resolver", &self.url_resolver)
            .field("max_retries", &self.max_retries)
            .field("base_delay_ms", &self.base_delay_ms)
            .field("memory_guard", &self.memory_guard)
            .field("cancelled", &self.cancelled.is_cancelled())
            .finish()
    }
}

impl CloneFallbackEngine {
    pub fn new(
        executor: Arc<dyn GitExecutorTrait>,
        url_resolver: Arc<CloneUrlResolver>,
    ) -> Self {
        Self {
            executor,
            url_resolver,
            max_retries: DEFAULT_MAX_RETRIES,
            base_delay_ms: DEFAULT_BASE_DELAY_MS,
            on_progress: noop_progress(),
            disk_checker: None,
            memory_guard: MemoryGuard::default(),
            cancelled: Cancelled::new(),
        }
    }

    pub fn with_retry(
        executor: Arc<dyn GitExecutorTrait>,
        url_resolver: Arc<CloneUrlResolver>,
        max_retries: u32,
        base_delay_ms: u64,
    ) -> Self {
        Self {
            executor,
            url_resolver,
            max_retries,
            base_delay_ms,
            on_progress: noop_progress(),
            disk_checker: None,
            memory_guard: MemoryGuard::default(),
            cancelled: Cancelled::new(),
        }
    }

    pub fn with_progress(mut self, cb: ProgressCallback) -> Self {
        self.on_progress = cb;
        self
    }

    pub fn with_disk_checker(mut self, checker: Arc<dyn DiskSpaceChecker>) -> Self {
        self.disk_checker = Some(checker);
        self
    }

    pub fn with_memory_guard(mut self, guard: MemoryGuard) -> Self {
        self.memory_guard = guard;
        self
    }

    /// Attach a shared cancellation flag (typically set by Ctrl+C).
    pub fn with_cancellation(mut self, cancelled: Cancelled) -> Self {
        self.cancelled = cancelled;
        self
    }

    /// Returns a reference to the cancellation flag so callers can reset it.
    pub fn cancelled(&self) -> &Cancelled {
        &self.cancelled
    }

    pub async fn clone_with_fallback(
        &self,
        base_opts: &CloneOptions,
        resolved: &ResolvedStrategy,
        account_uuid: Option<Uuid>,
    ) -> Result<CloneAttempt, GitManagerError> {
        // ── Pre-flight: disk space validation ──
        if let Some(ref checker) = self.disk_checker {
            let required = base_opts.estimated_size_bytes.max(50 * 1024 * 1024);
            // Resolve destination to an absolute path before extracting parent,
            // so that relative paths like "repo.git" don't produce an empty parent.
            let dest_abs = if base_opts.destination.is_relative() {
                std::env::current_dir()
                    .unwrap_or_default()
                    .join(&base_opts.destination)
            } else {
                base_opts.destination.clone()
            };
            let dest_parent = dest_abs.parent()
                .unwrap_or(&dest_abs);
            if let Err(msg) = checker.ensure_space(dest_parent, required) {
                return Err(GitManagerError::Other(format!("Disk space check failed: {msg}")));
            }
        }

        // ── Pre-flight: memory protection ──
        if base_opts.estimated_size_bytes > 0 {
            if let Err(msg) = self.memory_guard.check(base_opts.estimated_size_bytes) {
                return Err(GitManagerError::Other(format!("Memory protection check failed: {msg}")));
            }
        }

        // ── Check cancellation before starting ──
        if self.cancelled.is_cancelled() {
            return Err(GitManagerError::Other("Clone cancelled by user".into()));
        }

        self.on_progress(CloneProgress::new(0.0));

        let strategies = resolved.all();
        let mut last_error: Option<GitError> = None;

        for (index, strategy) in strategies.iter().enumerate() {
            if self.cancelled.is_cancelled() {
                return Err(GitManagerError::Other("Clone cancelled by user".into()));
            }

            let rewritten_url = self.url_resolver.resolve(
                &base_opts.url,
                strategy,
                account_uuid,
            ).await?;

            let opts = CloneOptions {
                url: rewritten_url,
                destination: base_opts.destination.clone(),
                ssh_key_path: if strategy.requires_ssh_key() {
                    base_opts.ssh_key_path.clone()
                } else {
                    None
                },
                ssh_host_alias: base_opts.ssh_host_alias.clone(),
                branch: base_opts.branch.clone(),
                depth: base_opts.depth,
                filter: base_opts.filter.clone(),
                bare: base_opts.bare,
                mirror: base_opts.mirror,
                sparse_checkout: base_opts.sparse_checkout.clone(),
                single_branch: base_opts.single_branch,
                no_checkout: base_opts.no_checkout,
                recurse_submodules: base_opts.recurse_submodules,
                tags_mode: base_opts.tags_mode.clone(),
                upload_pack: base_opts.upload_pack.clone(),
                estimated_size_bytes: base_opts.estimated_size_bytes,
            };

            match self.attempt_with_retry(strategy.clone(), opts, index).await {
                Ok(result) => {
                    self.on_progress(CloneProgress::done());
                    return Ok(CloneAttempt {
                        success: true,
                        strategy: strategy.clone(),
                        result: Some(result),
                        attempts: index + 1,
                        error: None,
                    });
                }
                Err(err) => {
                    if self.cancelled.is_cancelled() {
                        return Err(GitManagerError::Other("Clone cancelled by user".into()));
                    }
                    tracing::warn!(
                        strategy = %strategy,
                        error = %err,
                        "strategy exhausted, moving to next"
                    );
                    last_error = Some(err);
                }
            }
        }

        self.on_progress(CloneProgress::done());
        Ok(CloneAttempt {
            success: false,
            strategy: strategies.last().cloned().unwrap_or(AuthStrategy::Anonymous),
            result: None,
            attempts: strategies.len(),
            error: last_error,
        })
    }

    fn on_progress(&self, progress: CloneProgress) {
        (self.on_progress)(progress);
    }

    async fn attempt_with_retry(
        &self,
        strategy: AuthStrategy,
        opts: CloneOptions,
        strategy_index: usize,
    ) -> Result<CloneResult, GitError> {
        let mut last_error: Option<GitError> = None;

        for retry in 0..=self.max_retries {
            if self.cancelled.is_cancelled() {
                return Err(GitError::CloneFailed { reason: "cancelled by user".into() });
            }

            if retry > 0 {
                let delay_ms = self.base_delay_ms * (1u64 << (retry - 1));
                tracing::info!(
                    strategy = %strategy,
                    retry,
                    delay_ms,
                    "retrying clone after transient error"
                );
                // Use select! to allow Ctrl+C to interrupt the sleep
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_millis(delay_ms)) => {}
                    _ = async { while !self.cancelled.is_cancelled() { tokio::time::sleep(Duration::from_millis(100)).await; } } => {
                        return Err(GitError::CloneFailed { reason: "cancelled by user".into() });
                    }
                }
            }

            tracing::info!(
                attempt = strategy_index + 1,
                retry,
                strategy = %strategy,
                "clone attempt"
            );

            let executor = &*self.executor;
            match GitExecutorTrait::clone(executor, opts.clone()).await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if self.cancelled.is_cancelled() {
                        return Err(GitError::CloneFailed { reason: "cancelled by user".into() });
                    }
                    if is_transient_error(&err) {
                        tracing::warn!(
                            strategy = %strategy,
                            retry,
                            error = %err,
                            "transient error, will retry"
                        );
                        last_error = Some(err);
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        Err(last_error.unwrap_or(GitError::CloneFailed {
            reason: "all retries exhausted".to_string(),
        }))
    }
}

fn is_transient_error(err: &GitError) -> bool {
    match err {
        GitError::NetworkTimeout { .. } => true,
        GitError::CloneFailed { reason } => {
            let lower = reason.to_lowercase();
            lower.contains("timeout")
                || lower.contains("connection refused")
                || lower.contains("connection reset")
                || lower.contains("temporary failure")
                || lower.contains("could not resolve")
                || lower.contains("network is unreachable")
                || lower.contains("no route to host")
                || lower.contains("software caused connection abort")
                || lower.contains("broken pipe")
        }
        _ => false,
    }
}

#[derive(Debug)]
pub struct CloneAttempt {
    pub success:  bool,
    pub strategy: AuthStrategy,
    pub result:   Option<CloneResult>,
    pub attempts: usize,
    pub error:    Option<GitError>,
}
