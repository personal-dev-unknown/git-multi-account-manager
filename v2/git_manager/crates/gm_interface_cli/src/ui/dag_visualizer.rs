// crates/gm_interface_cli/src/ui/dag_visualizer.rs
//
// Caller:  commands/dag.rs → handle_dag_command() → Dag::render()
// Purpose: Renders Directed Acyclic Graphs in the terminal.
//          Supports three graph types:
//            1. Workflow DAG   — workflow steps and their ordering
//            2. Plugin DAG     — crate dependency graph
//            3. Event flow DAG — which events trigger which handlers/workflows
//
// Design:  The renderer uses a layered layout algorithm:
//            Step 1: Topological sort to determine node depth (layer index)
//            Step 2: Group nodes into layers
//            Step 3: Render each layer as a row of labeled boxes
//            Step 4: Draw vertical connector lines and arrows between layers

use owo_colors::OwoColorize;
use std::collections::{HashMap, VecDeque};

// ─── Node type ────────────────────────────────────────────────────────────────

/// Classifies a node so the visualizer can apply consistent colors.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Kernel,     // Dark green  — the stable microkernel
    Domain,     // Blue        — pure business logic
    Port,       // Cyan        — the hexagonal boundary
    Adapter,    // Yellow      — infrastructure implementations
    Plugin,     // Magenta     — extensible providers
    Interface,  // Green       — CLI / Web / Desktop
    Zig,        // Bright yellow — native OS layer (different language)
    Workflow,   // White       — workflow steps
    Event,      // Bright cyan — domain events
    Normal,     // Default     — generic nodes
}

/// A single node in the DAG.
#[derive(Debug, Clone)]
pub struct DagNode {
    /// Unique identifier used for edge references. Not displayed.
    pub id: String,
    /// The label shown inside the box.
    pub label: String,
    /// Optional second line inside the box (e.g. "impl GitExecutor").
    pub sublabel: Option<String>,
    /// Controls the box border color.
    pub kind: NodeKind,
}

impl DagNode {
    pub fn new(id: impl Into<String>, label: impl Into<String>, kind: NodeKind) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            sublabel: None,
            kind,
        }
    }

    pub fn with_sublabel(mut self, sub: impl Into<String>) -> Self {
        self.sublabel = Some(sub.into());
        self
    }
}

/// A directed edge from one node to another.
#[derive(Debug, Clone)]
pub struct DagEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

impl DagEdge {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self { from: from.into(), to: to.into(), label: None }
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// The complete graph model. Build this, then call `.render()`.
#[derive(Debug, Default)]
pub struct Dag {
    nodes: Vec<DagNode>,
    edges: Vec<DagEdge>,
}

impl Dag {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node. Nodes must be added before edges reference them.
    pub fn add_node(&mut self, node: DagNode) -> &mut Self {
        self.nodes.push(node);
        self
    }

    /// Adds a directed edge (from → to). Both IDs must already be nodes.
    pub fn add_edge(&mut self, edge: DagEdge) -> &mut Self {
        self.edges.push(edge);
        self
    }

    // ─── Layout algorithm ─────────────────────────────────────────────────────

    /// Computes the layer (depth) for each node using BFS from root nodes.
    /// Root nodes are those with no incoming edges (in-degree == 0).
    /// Returns a map: node_id → layer_index (0 = topmost).
    fn compute_layers(&self) -> HashMap<String, usize> {
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut adjacency: HashMap<&str, Vec<&str>> = HashMap::new();

        for node in &self.nodes {
            in_degree.entry(node.id.as_str()).or_insert(0);
            adjacency.entry(node.id.as_str()).or_default();
        }

        for edge in &self.edges {
            *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
            adjacency
                .entry(edge.from.as_str())
                .or_default()
                .push(edge.to.as_str());
        }

        let mut layers: HashMap<String, usize> = HashMap::new();
        let mut queue: VecDeque<(&str, usize)> = VecDeque::new();

        for (id, &deg) in &in_degree {
            if deg == 0 {
                // id: &&str — deref once to get &str for the queue
                queue.push_back((*id, 0));
                layers.insert(id.to_string(), 0);
            }
        }

        while let Some((node_id, depth)) = queue.pop_front() {
            if let Some(neighbors) = adjacency.get(node_id) {
                for &neighbor in neighbors {
                    let new_depth = depth + 1;
                    let current = layers.get(neighbor).copied().unwrap_or(0);
                    if new_depth > current {
                        layers.insert(neighbor.to_string(), new_depth);
                        queue.push_back((neighbor, new_depth));
                    }
                }
            }
        }

        // Any isolated node (no edges) gets layer 0
        for node in &self.nodes {
            layers.entry(node.id.clone()).or_insert(0);
        }

        layers
    }

    // ─── Rendering ────────────────────────────────────────────────────────────

    /// Renders a single node as a box.
    /// Returns (top_border, label_line, sublabel_line, bottom_border, box_width).
    fn render_node_box(node: &DagNode) -> (String, String, Option<String>, String, usize) {
        let label = &node.label;
        let sublabel = node.sublabel.as_deref();

        let content_width = sublabel
            .map(|s| s.len().max(label.len()))
            .unwrap_or(label.len());

        let inner_width = content_width + 2; // 1 space padding each side
        let box_width   = inner_width + 2;   // + 2 for the │ borders

        let top    = format!("┌{}┐", "─".repeat(inner_width));
        let bottom = format!("└{}┘", "─".repeat(inner_width));
        let mid    = format!("│ {:^width$} │", label,    width = content_width);
        let sub    = sublabel.map(|s| format!("│ {:^width$} │", s, width = content_width));

        let (top_c, mid_c, sub_c, bot_c) = match node.kind {
            NodeKind::Kernel => (
                top.bright_green().to_string(),
                mid.bright_green().to_string(),
                sub.map(|s| s.bright_green().to_string()),
                bottom.bright_green().to_string(),
            ),
            NodeKind::Domain => (
                top.bright_blue().to_string(),
                mid.bright_blue().to_string(),
                sub.map(|s| s.bright_blue().to_string()),
                bottom.bright_blue().to_string(),
            ),
            NodeKind::Port => (
                top.cyan().to_string(),
                mid.cyan().to_string(),
                sub.map(|s| s.cyan().to_string()),
                bottom.cyan().to_string(),
            ),
            NodeKind::Adapter => (
                top.yellow().to_string(),
                mid.yellow().to_string(),
                sub.map(|s| s.yellow().to_string()),
                bottom.yellow().to_string(),
            ),
            NodeKind::Plugin => (
                top.magenta().to_string(),
                mid.magenta().to_string(),
                sub.map(|s| s.magenta().to_string()),
                bottom.magenta().to_string(),
            ),
            NodeKind::Interface => (
                top.green().to_string(),
                mid.green().to_string(),
                sub.map(|s| s.green().to_string()),
                bottom.green().to_string(),
            ),
            NodeKind::Zig => (
                top.bright_yellow().to_string(),
                mid.bright_yellow().to_string(),
                sub.map(|s| s.bright_yellow().to_string()),
                bottom.bright_yellow().to_string(),
            ),
            NodeKind::Workflow => (
                top.white().to_string(),
                mid.white().to_string(),
                sub.map(|s| s.white().to_string()),
                bottom.white().to_string(),
            ),
            NodeKind::Event => (
                top.bright_cyan().to_string(),
                mid.bright_cyan().to_string(),
                sub.map(|s| s.bright_cyan().to_string()),
                bottom.bright_cyan().to_string(),
            ),
            NodeKind::Normal => (top, mid, sub, bottom),
        };

        (top_c, mid_c, sub_c, bot_c, box_width)
    }

    /// Renders the entire DAG to stdout with a title and legend.
    pub fn render(&self, title: &str) {
        let layers = self.compute_layers();

        let max_layer = layers.values().copied().max().unwrap_or(0);
        let mut layer_groups: Vec<Vec<&DagNode>> = vec![Vec::new(); max_layer + 1];

        for node in &self.nodes {
            let layer = layers.get(&node.id).copied().unwrap_or(0);
            layer_groups[layer].push(node);
        }

        // Build edge lookup: (from_id, to_id) → Option<label>
        let edge_label: HashMap<(&str, &str), Option<&str>> = self
            .edges
            .iter()
            .map(|e| ((e.from.as_str(), e.to.as_str()), e.label.as_deref()))
            .collect();

        // ─── Header ───────────────────────────────────────────────────────────
        println!();
        println!("  {}", title.bold().underline());
        let separator = "═".repeat(title.len() + 4);
        println!("  {}", separator.dimmed());
        println!();

        // ─── Layers ───────────────────────────────────────────────────────────
        for (layer_idx, layer_nodes) in layer_groups.iter().enumerate() {
            if layer_nodes.is_empty() {
                continue;
            }

            // Render each node's box
            let rendered: Vec<(String, String, Option<String>, String, usize)> =
                layer_nodes.iter().map(|n| Self::render_node_box(n)).collect();

            // Top borders
            print!("  ");
            for (top, _, _, _, _) in &rendered {
                print!("{}  ", top);
            }
            println!();

            // Label lines
            print!("  ");
            for (_, mid, _, _, _) in &rendered {
                print!("{}  ", mid);
            }
            println!();

            // Sublabel lines — only when at least one node in this layer has one.
            // Nodes without a sublabel get an empty box row so heights are uniform.
            let has_sublabel = rendered.iter().any(|(_, _, sub, _, _)| sub.is_some());
            if has_sublabel {
                print!("  ");
                for (_, _, sub, _, width) in &rendered {
                    // Build the fallback empty row with the correct width.
                    // We bind it first so the &str borrow lives for the print call.
                    let inner  = width.saturating_sub(2);
                    let fallback = format!("│{}│", " ".repeat(inner));
                    let placeholder: &str = sub.as_deref().unwrap_or(&fallback);
                    print!("{}  ", placeholder);
                }
                println!();
            }

            // Bottom borders
            print!("  ");
            for (_, _, _, bot, _) in &rendered {
                print!("{}  ", bot);
            }
            println!();

            // ─── Connectors to the next layer ─────────────────────────────────
            if layer_idx < max_layer {
                let next_layer = &layer_groups[layer_idx + 1];

                // Which nodes in this layer have at least one edge into the next?
                let has_outgoing: Vec<bool> = layer_nodes
                    .iter()
                    .map(|n| {
                        next_layer.iter().any(|next_n| {
                            edge_label
                                .contains_key(&(n.id.as_str(), next_n.id.as_str()))
                        })
                    })
                    .collect();

                // Edge label for each current-layer node's outgoing connector
                let connector_labels: Vec<Option<&str>> = layer_nodes
                    .iter()
                    .map(|n| {
                        next_layer.iter().find_map(|next_n| {
                            edge_label
                                .get(&(n.id.as_str(), next_n.id.as_str()))
                                .copied()
                                .flatten()
                        })
                    })
                    .collect();

                // Vertical connector lines (│)
                print!("  ");
                for ((_, _, _, _, width), &outgoing) in
                    rendered.iter().zip(has_outgoing.iter())
                {
                    if outgoing {
                        let center = *width / 2;
                        print!("{}", " ".repeat(center));
                        print!("{}", "│".dimmed());
                        print!("{}", " ".repeat(*width - center + 1));
                    } else {
                        print!("{}", " ".repeat(*width + 2));
                    }
                }
                println!();

                // Edge labels (if any connector in this row has a label)
                let any_label = connector_labels.iter().any(|l| l.is_some());
                if any_label {
                    print!("  ");
                    for ((_, _, _, _, width), label) in
                        rendered.iter().zip(connector_labels.iter())
                    {
                        let center = *width / 2;
                        if let Some(lbl) = label {
                            let total = *width + 2;
                            let label_start = center.saturating_sub(lbl.len() / 2);
                            print!("{}", " ".repeat(label_start));
                            print!("{}", lbl.dimmed().italic());
                            let remaining =
                                total.saturating_sub(label_start + lbl.len());
                            print!("{}", " ".repeat(remaining));
                        } else {
                            print!("{}", " ".repeat(*width + 2));
                        }
                    }
                    println!();
                }

                // Downward arrow (▼)
                print!("  ");
                for ((_, _, _, _, width), &outgoing) in
                    rendered.iter().zip(has_outgoing.iter())
                {
                    if outgoing {
                        let center = *width / 2;
                        print!("{}", " ".repeat(center));
                        print!("{}", "▼".cyan());
                        print!("{}", " ".repeat(*width - center + 1));
                    } else {
                        print!("{}", " ".repeat(*width + 2));
                    }
                }
                println!();
            }
        }

        println!();

        // ─── Legend ───────────────────────────────────────────────────────────
        println!("  {}", "Legend:".dimmed());
        println!(
            "  {}  {}  {}  {}  {}  {}  {}",
            "■ Kernel".bright_green(),
            "■ Domain".bright_blue(),
            "■ Port".cyan(),
            "■ Adapter".yellow(),
            "■ Plugin".magenta(),
            "■ Interface".green(),
            "■ Zig Native".bright_yellow()
        );
        println!();
    }
}

// ─── Pre-built graph factories ────────────────────────────────────────────────

/// Builds the crate dependency DAG for the entire workspace.
pub fn crate_dependency_dag() -> Dag {
    let mut dag = Dag::new();

    dag.add_node(
        DagNode::new("shared", "gm_shared", NodeKind::Domain)
            .with_sublabel("serde · uuid · chrono"),
    );
    dag.add_node(
        DagNode::new("zig", "zig_native", NodeKind::Zig)
            .with_sublabel("libgm_native.a"),
    );
    dag.add_node(
        DagNode::new("domain", "gm_domain", NodeKind::Domain)
            .with_sublabel("pure business logic"),
    );
    dag.add_node(
        DagNode::new("ports", "gm_ports", NodeKind::Port)
            .with_sublabel("hexagonal boundary"),
    );
    dag.add_node(
        DagNode::new("kernel", "gm_kernel", NodeKind::Kernel)
            .with_sublabel("microkernel runtime"),
    );
    dag.add_node(
        DagNode::new("adapters", "gm_adapters", NodeKind::Adapter)
            .with_sublabel("Rust → Zig FFI"),
    );
    dag.add_node(DagNode::new("github",    "gm_plugin_github",    NodeKind::Plugin));
    dag.add_node(DagNode::new("gitlab",    "gm_plugin_gitlab",    NodeKind::Plugin));
    dag.add_node(DagNode::new("bitbucket", "gm_plugin_bitbucket", NodeKind::Plugin));
    dag.add_node(
        DagNode::new("cli", "gm_interface_cli", NodeKind::Interface)
            .with_sublabel("Clap + owo-colors"),
    );
    dag.add_node(
        DagNode::new("web", "gm_interface_web", NodeKind::Interface)
            .with_sublabel("Axum + Askama"),
    );
    dag.add_node(
        DagNode::new("desktop", "gm_interface_desktop", NodeKind::Interface)
            .with_sublabel("Tauri + Svelte"),
    );
    dag.add_node(DagNode::new("app_cli",  "apps/cli",     NodeKind::Normal));
    dag.add_node(DagNode::new("app_web",  "apps/web",     NodeKind::Normal));
    dag.add_node(DagNode::new("app_desk", "apps/desktop", NodeKind::Normal));

    dag.add_edge(DagEdge::new("shared",   "domain").with_label("imported by"));
    dag.add_edge(DagEdge::new("shared",   "ports"));
    dag.add_edge(DagEdge::new("domain",   "ports"));
    dag.add_edge(DagEdge::new("shared",   "kernel"));
    dag.add_edge(DagEdge::new("domain",   "kernel"));
    dag.add_edge(DagEdge::new("ports",    "kernel"));
    dag.add_edge(DagEdge::new("domain",   "adapters"));
    dag.add_edge(DagEdge::new("ports",    "adapters"));
    dag.add_edge(DagEdge::new("zig",      "adapters").with_label("links via FFI"));
    dag.add_edge(DagEdge::new("ports",    "github"));
    dag.add_edge(DagEdge::new("ports",    "gitlab"));
    dag.add_edge(DagEdge::new("ports",    "bitbucket"));
    dag.add_edge(DagEdge::new("kernel",   "cli"));
    dag.add_edge(DagEdge::new("ports",    "cli"));
    dag.add_edge(DagEdge::new("kernel",   "web"));
    dag.add_edge(DagEdge::new("ports",    "web"));
    dag.add_edge(DagEdge::new("kernel",   "desktop"));
    dag.add_edge(DagEdge::new("ports",    "desktop"));
    dag.add_edge(DagEdge::new("kernel",   "app_cli"));
    dag.add_edge(DagEdge::new("cli",      "app_cli"));
    dag.add_edge(DagEdge::new("kernel",   "app_web"));
    dag.add_edge(DagEdge::new("web",      "app_web"));
    dag.add_edge(DagEdge::new("kernel",   "app_desk"));
    dag.add_edge(DagEdge::new("desktop",  "app_desk"));

    dag
}

/// Builds the DAG for the built-in `clone_and_configure` workflow.
pub fn clone_workflow_dag() -> Dag {
    let mut dag = Dag::new();

    dag.add_node(DagNode::new("parse",     "parse_url",          NodeKind::Workflow)
        .with_sublabel("url_parser.rs"));
    dag.add_node(DagNode::new("detect",    "detect_platform",    NodeKind::Workflow)
        .with_sublabel("PlatformType enum"));
    dag.add_node(DagNode::new("auth",      "authenticate",       NodeKind::Workflow)
        .with_sublabel("AuthProvider trait"));
    dag.add_node(DagNode::new("fetch",     "fetch_repo_info",    NodeKind::Workflow)
        .with_sublabel("RepositoryProvider"));
    dag.add_node(DagNode::new("clone",     "clone_repository",   NodeKind::Workflow)
        .with_sublabel("ZigGitExecutor"));
    dag.add_node(DagNode::new("configure", "configure_identity", NodeKind::Workflow)
        .with_sublabel("git config local"));
    dag.add_node(DagNode::new("publish",   "publish_events",     NodeKind::Workflow)
        .with_sublabel("RepositoryCloned"));

    dag.add_edge(DagEdge::new("parse",     "detect").with_label("GitUrl"));
    dag.add_edge(DagEdge::new("detect",    "auth").with_label("PlatformType"));
    dag.add_edge(DagEdge::new("auth",      "fetch").with_label("AuthResult"));
    dag.add_edge(DagEdge::new("fetch",     "clone").with_label("RepositoryInfo"));
    dag.add_edge(DagEdge::new("clone",     "configure").with_label("CloneResult"));
    dag.add_edge(DagEdge::new("configure", "publish").with_label("done"));

    dag
}

/// Builds the event flow DAG showing which events trigger which handlers.
pub fn event_flow_dag() -> Dag {
    let mut dag = Dag::new();

    dag.add_node(DagNode::new("ev_acc",  "AccountAdded",     NodeKind::Event));
    dag.add_node(DagNode::new("ev_ssh",  "SshKeyGenerated",  NodeKind::Event));
    dag.add_node(DagNode::new("ev_rep",  "RepositoryCloned", NodeKind::Event));
    dag.add_node(DagNode::new("ev_sync", "SyncCompleted",    NodeKind::Event));

    dag.add_node(DagNode::new("audit",  "AuditHandler",    NodeKind::Kernel)
        .with_sublabel("→ audit_logs table"));
    dag.add_node(DagNode::new("cli_h",  "CLI Handler",     NodeKind::Interface)
        .with_sublabel("owo-colors output"));
    dag.add_node(DagNode::new("web_h",  "Web SSE Handler", NodeKind::Interface)
        .with_sublabel("EventSource push"));
    dag.add_node(DagNode::new("desk_h", "Tauri Handler",   NodeKind::Interface)
        .with_sublabel("window.emit()"));
    dag.add_node(DagNode::new("wf_h",   "Workflow Trigger", NodeKind::Kernel)
        .with_sublabel("configure_identity"));

    dag.add_edge(DagEdge::new("ev_acc",  "audit"));
    dag.add_edge(DagEdge::new("ev_ssh",  "audit"));
    dag.add_edge(DagEdge::new("ev_rep",  "audit"));
    dag.add_edge(DagEdge::new("ev_sync", "audit"));

    dag.add_edge(DagEdge::new("ev_acc",  "cli_h"));
    dag.add_edge(DagEdge::new("ev_rep",  "cli_h"));
    dag.add_edge(DagEdge::new("ev_sync", "cli_h"));
    dag.add_edge(DagEdge::new("ev_rep",  "web_h"));
    dag.add_edge(DagEdge::new("ev_sync", "web_h"));
    dag.add_edge(DagEdge::new("ev_acc",  "desk_h"));
    dag.add_edge(DagEdge::new("ev_rep",  "desk_h"));
    dag.add_edge(DagEdge::new("ev_rep",  "wf_h").with_label("triggers"));

    dag
}
