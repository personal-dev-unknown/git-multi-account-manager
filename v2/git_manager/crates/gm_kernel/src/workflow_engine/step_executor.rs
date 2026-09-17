// crates/gm_kernel/src/workflow_engine/step_executor.rs
//
// The StepExecutor handles the execution of a single workflow step, including
// retry logic with exponential back-off and optional timeout enforcement.
//
// ── Retry strategy ────────────────────────────────────────────────────────────
// When a step fails and retry_count > 0, the executor waits for an exponentially
// increasing delay before retrying: 200ms, 400ms, 800ms, up to a maximum of
// 10 seconds per attempt. Exponential back-off reduces thundering herd effects
// when the failure is caused by a temporarily unavailable resource (e.g. git
// server returning 503).
//
// ── Timeout enforcement ───────────────────────────────────────────────────────
// If timeout_ms > 0, the step is wrapped in tokio::time::timeout(). If the step
// exceeds its budget, the future is cancelled and the step is treated as a
// failure. This prevents runaway steps (e.g. a git clone that stalls because
// the network is down) from blocking the workflow forever.
//
// ── Context immutability during retry ────────────────────────────────────────
// The context passed to the step on each retry attempt is the ORIGINAL context
// from before the first attempt, not any partially-modified context from the
// failed attempt. This ensures that retries are semantically idempotent from
// the step's perspective — the step sees the same input each time.

use std::sync::Arc;
use std::time::Duration;
use gm_shared::errors::GitManagerError;
use crate::workflow_engine::registry::{StepHandler, WorkflowContext, WorkflowStepDef};

#[derive(Debug)]
pub struct StepExecutor;

impl StepExecutor {
    /// Executes one workflow step with the configured retry and timeout policy.
    ///
    /// Returns the updated context on success, or the final error after all
    /// retry attempts are exhausted.
    pub async fn execute(
        step:    &WorkflowStepDef,
        handler: Arc<dyn StepHandler>,
        context: WorkflowContext,
    ) -> Result<WorkflowContext, GitManagerError> {
        let max_attempts = step.retry_count + 1; // at least one attempt
        let mut last_error = GitManagerError::Other(
            format!("step '{}' exhausted all {} attempts", step.name, max_attempts)
        );

        for attempt in 0..max_attempts {
            if attempt > 0 {
                // Exponential back-off: 200ms * 2^(attempt-1), capped at 10s.
                let delay_ms = (200u64 * (1u64 << (attempt - 1).min(5))).min(10_000);
                tracing::debug!(
                    step    = %step.name,
                    attempt = attempt,
                    delay_ms = delay_ms,
                    "retrying workflow step after delay"
                );
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }

            let result = if step.timeout_ms > 0 {
                tokio::time::timeout(
                    Duration::from_millis(step.timeout_ms),
                    handler.execute(context.clone()),
                )
                .await
                .unwrap_or_else(|_| Err(GitManagerError::Other(
                    format!("step '{}' timed out after {}ms", step.name, step.timeout_ms)
                )))
            } else {
                handler.execute(context.clone()).await
            };

            match result {
                Ok(new_context) => {
                    tracing::debug!(step = %step.name, attempt = attempt, "workflow step succeeded");
                    return Ok(new_context);
                }
                Err(e) => {
                    tracing::warn!(
                        step    = %step.name,
                        attempt = attempt,
                        error   = %e,
                        "workflow step failed"
                    );
                    last_error = e;
                }
            }
        }

        Err(last_error)
    }
}