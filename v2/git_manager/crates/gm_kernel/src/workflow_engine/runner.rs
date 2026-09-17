// crates/gm_kernel/src/workflow_engine/runner.rs
//
// The WorkflowRunner is the top-level orchestrator for named multi-step
// workflows. It fetches the definition, resolves each step's handler, and
// drives the StepExecutor through each step while accumulating the context.
//
// ── Rollback on failure ───────────────────────────────────────────────────────
// When the FailurePolicy is Rollback and a step fails, the runner calls
// compensate() on each previously completed step handler in reverse order.
// Compensation errors are logged but do not override the original failure —
// the caller receives the original error from the failing step.
//
// ── Tracing correlation ───────────────────────────────────────────────────────
// Every workflow execution receives a correlation_id UUID at start time. All
// events published by step handlers during this workflow carry this ID so that
// the complete audit trail for a single user action (e.g. "add account") can
// be reconstructed by filtering the event log for one correlation_id.

use std::sync::Arc;
use uuid::Uuid;
use gm_shared::errors::GitManagerError;
use crate::{
    event_bus::EventBus,
    service_registry::ServiceRegistry,
    workflow_engine::{
        registry::{FailurePolicy, WorkflowContext, WorkflowRegistry},
        step_executor::StepExecutor,
    },
};

#[derive(Debug)]
pub struct WorkflowRunner {
    workflow_registry: Arc<WorkflowRegistry>,
    #[allow(dead_code)]
    event_bus:         Arc<EventBus>,
}

impl WorkflowRunner {
    pub fn new(
        workflow_registry: Arc<WorkflowRegistry>,
        _service_registry: Arc<ServiceRegistry>, // reserved for future handler injection
        event_bus:         Arc<EventBus>,
    ) -> Self {
        Self { workflow_registry, event_bus }
    }

    /// Runs a named workflow with the provided initial context.
    ///
    /// Returns the final accumulated context on success. On failure, the error
    /// from the failing step is returned after any rollback compensation is applied.
    pub async fn run(
        &self,
        workflow_name:    &str,
        initial_context:  WorkflowContext,
        correlation_id:   Option<Uuid>,
    ) -> Result<WorkflowContext, GitManagerError> {
        let def = self.workflow_registry
            .get_definition(workflow_name)
            .ok_or_else(|| GitManagerError::Other(
                format!("workflow '{workflow_name}' is not registered")
            ))?;

        let corr_id = correlation_id.unwrap_or_else(Uuid::new_v4);

        tracing::info!(
            workflow       = %workflow_name,
            version        = %def.version,
            correlation_id = %corr_id,
            steps          = def.steps.len(),
            "starting workflow execution"
        );

        let mut context      = initial_context;
        // Track the index of the last completed step for rollback.
        let mut completed: Vec<usize> = Vec::with_capacity(def.steps.len());

        for (idx, step_def) in def.steps.iter().enumerate() {
            let handler = self.workflow_registry
                .get_handler(&step_def.handler)
                .ok_or_else(|| GitManagerError::Other(
                    format!("workflow step handler '{}' is not registered", step_def.handler)
                ))?;

            tracing::debug!(
                workflow    = %workflow_name,
                step        = %step_def.name,
                handler     = %step_def.handler,
                step_index  = idx,
                "executing workflow step"
            );

            match StepExecutor::execute(step_def, Arc::clone(&handler), context.clone()).await {
                Ok(new_ctx) => {
                    completed.push(idx);
                    context = new_ctx;
                }
                Err(e) => {
                    tracing::error!(
                        workflow   = %workflow_name,
                        step       = %step_def.name,
                        error      = %e,
                        "workflow step failed"
                    );

                    match def.on_failure {
                        FailurePolicy::Abort => {
                            return Err(e);
                        }
                        FailurePolicy::Rollback => {
                            self.rollback(&def, &completed, &context).await;
                            return Err(e);
                        }
                        FailurePolicy::Continue => {
                            tracing::warn!(
                                step = %step_def.name,
                                "step failed but workflow continues (FailurePolicy::Continue)"
                            );
                            completed.push(idx);
                        }
                    }
                }
            }
        }

        tracing::info!(
            workflow       = %workflow_name,
            correlation_id = %corr_id,
            "workflow completed successfully"
        );

        Ok(context)
    }

    /// Calls compensate() on all completed steps in reverse order.
    async fn rollback(
        &self,
        def:       &crate::workflow_engine::registry::WorkflowDefinition,
        completed: &[usize],
        context:   &WorkflowContext,
    ) {
        tracing::info!(
            workflow = %def.name,
            steps    = completed.len(),
            "rolling back completed workflow steps"
        );

        for &idx in completed.iter().rev() {
            let step_def = &def.steps[idx];
            if let Some(handler) = self.workflow_registry.get_handler(&step_def.handler) {
                if let Err(e) = handler.compensate(context).await {
                    tracing::error!(
                        step  = %step_def.name,
                        error = %e,
                        "workflow step compensation failed — manual cleanup may be required"
                    );
                }
            }
        }
    }
}