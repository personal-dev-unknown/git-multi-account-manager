// crates/gm_interface_cli/src/commands/dag.rs
//
// Caller:  app.rs → Commands::Dag dispatch
// Purpose: CLI command that exposes the DAG visualizer to users.
//
// Usage:
//   git-zyrix dag crates    → crate dependency graph
//   git-zyrix dag workflow  → clone_and_configure step DAG
//   git-zyrix dag events    → domain event flow diagram

use clap::{Args, Subcommand};

use crate::ui::dag_visualizer::{
    crate_dependency_dag, clone_workflow_dag, event_flow_dag,
};

/// Wrapper struct so `DagCmd` fits the `Commands::Dag(DagArgs)` pattern
/// used by every other command in `app.rs`.
#[derive(Debug, Args)]
pub struct DagArgs {
    #[command(subcommand)]
    pub cmd: DagCmd,
}

/// The `dag` subcommand variants.
#[derive(Debug, Subcommand)]
pub enum DagCmd {
    /// Shows the Cargo workspace crate dependency graph.
    ///
    /// Useful for verifying that hexagonal architecture boundaries are
    /// respected and that adapters never leak into the domain layer.
    Crates,

    /// Shows the built-in clone_and_configure workflow as a step DAG.
    ///
    /// Each box is a workflow step; arrows carry the data flowing between them.
    Workflow,

    /// Shows the domain event flow: which events trigger which handlers.
    ///
    /// Includes CLI, Web SSE, Desktop (Tauri), and Workflow Engine handlers.
    Events,
}

/// Handles the `git-zyrix dag <subcommand>` dispatch.
/// Synchronous — the DAG renderer only writes to stdout.
pub fn handle_dag_command(args: DagArgs) {
    match args.cmd {
        DagCmd::Crates => {
            crate_dependency_dag().render("Workspace Crate Dependency Graph");
        }
        DagCmd::Workflow => {
            clone_workflow_dag()
                .render("Workflow: clone_and_configure (step DAG)");
        }
        DagCmd::Events => {
            event_flow_dag().render("Domain Event Flow Diagram");
        }
    }
}
