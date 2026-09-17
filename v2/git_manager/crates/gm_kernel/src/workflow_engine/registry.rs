// crates/gm_kernel/src/workflow_engine/registry.rs
//
// The WorkflowRegistry holds two kinds of registrations:
//
//   1. WorkflowDefinitions — named, versioned sequences of steps that describe
//      a multi-stage operation (e.g. "clone_and_configure" involves 3 steps).
//      Definitions are loaded from the database at boot and cached here for
//      fast lookup during execution.
//
//   2. StepHandlers — the Rust functions that execute individual workflow steps.
//      Handlers are registered by name (string) by the kernel and by plugins
//      during on_load(). The workflow runner resolves step names to handlers
//      at execution time via this registry.
//
// ── Versioning of workflow definitions ───────────────────────────────────────
// Each WorkflowDefinition carries a `version` string. If a step's handler
// signature changes between versions, the old handler can be kept under its
// old name while the new version is registered under a new name, and the
// workflow definition is bumped to reference the new handler. This preserves
// backward compatibility for in-flight workflow instances that were persisted
// to the database before the upgrade.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use gm_shared::errors::GitManagerError;

/// Describes what happens when a workflow step fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FailurePolicy {
    /// Stop execution and return the error to the caller. Completed steps are
    /// not reversed (the operation is partially complete).
    Abort,
    /// Attempt to reverse all completed steps in reverse order before returning
    /// the error. Each step handler must implement a compensate() method.
    Rollback,
    /// Continue to the next step even if this step fails. Used for non-critical
    /// optional enrichment steps.
    Continue,
}

/// A single step within a workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepDef {
    /// Human-readable name used in tracing and diagnostic output.
    pub name:        String,
    /// The key under which the handler is registered in the step handler registry.
    pub handler:     String,
    /// Number of times to retry on failure before applying the failure policy.
    pub retry_count: u32,
    /// Maximum duration for this step in milliseconds. 0 means no timeout.
    pub timeout_ms:  u64,
}

/// A complete, named, versioned workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name:           String,
    pub version:        String,
    pub steps:          Vec<WorkflowStepDef>,
    /// Event types that trigger this workflow automatically.
    pub trigger_events: Vec<String>,
    pub on_failure:     FailurePolicy,
}

/// The execution context passed between workflow steps.
/// Each step receives the context from the previous step and returns a
/// (potentially modified) context for the next step.
/// The context is serialised as JSON for persistence and logging.
pub type WorkflowContext = serde_json::Value;

/// A workflow step handler — the code that executes one step of a workflow.
///
/// Handlers are registered by name and resolved at runtime. The `execute`
/// method receives the current context (accumulated from all previous steps)
/// and returns the updated context.
#[async_trait]
pub trait StepHandler: Send + Sync {
    /// Executes one step of the workflow.
    async fn execute(&self, context: WorkflowContext) -> Result<WorkflowContext, GitManagerError>;

    /// Attempts to reverse this step's effects. Called by the Rollback failure
    /// policy after a later step fails. If compensation is not possible (e.g.
    /// the step already committed to an external system), return Ok(()) silently
    /// — the operation log already captured what was done.
    async fn compensate(&self, context: &WorkflowContext) -> Result<(), GitManagerError>;
}

/// The central registry for both workflow definitions and their step handlers.
#[derive(Default)]
pub struct WorkflowRegistry {
    definitions:   RwLock<HashMap<String, WorkflowDefinition>>,
    step_handlers: RwLock<HashMap<String, Arc<dyn StepHandler>>>,
}

impl std::fmt::Debug for WorkflowRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let def_count = self.definitions.read().map(|d| d.len()).unwrap_or(0);
        let handler_count = self.step_handlers.read().map(|h| h.len()).unwrap_or(0);
        f.debug_struct("WorkflowRegistry")
            .field("definitions", &format_args!("[{} definition(s)]", def_count))
            .field("step_handlers", &format_args!("[{} step handler(s)]", handler_count))
            .finish()
    }
}

impl WorkflowRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers or replaces a workflow definition by name.
    pub fn register_definition(&self, def: WorkflowDefinition) {
        self.definitions.write()
            .expect("workflow registry: definitions RwLock poisoned")
            .insert(def.name.clone(), def);
    }

    /// Registers a step handler under a string key.
    pub fn register_handler(&self, name: String, handler: Arc<dyn StepHandler>) {
        self.step_handlers.write()
            .expect("workflow registry: step_handlers RwLock poisoned")
            .insert(name, handler);
    }

    /// Retrieves a workflow definition by name.
    pub fn get_definition(&self, name: &str) -> Option<WorkflowDefinition> {
        self.definitions.read()
            .expect("workflow registry: definitions RwLock poisoned")
            .get(name).cloned()
    }

    /// Retrieves a step handler by name.
    pub fn get_handler(&self, name: &str) -> Option<Arc<dyn StepHandler>> {
        self.step_handlers.read()
            .expect("workflow registry: step_handlers RwLock poisoned")
            .get(name).cloned()
    }
}