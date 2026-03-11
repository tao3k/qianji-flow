//! Core graph engine based on petgraph.

use crate::contracts::{NodeStatus, QianjiMechanism};
use petgraph::Directed;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use std::sync::Arc;

/// Compiler for declarative manifests.
pub mod compiler;

/// Represents a single thought mechanism node in the execution graph.
#[derive(Clone)]
pub struct QianjiNode {
    /// Unique ID of the node.
    pub id: String,
    /// Current execution status.
    pub status: NodeStatus,
    /// The logic to be executed.
    pub mechanism: Arc<dyn QianjiMechanism>,
}

/// Represents an edge between nodes with optional label and weight.
#[derive(Debug, Clone)]
pub struct QianjiEdge {
    /// Label for branch selection.
    pub label: Option<String>,
    /// Probability/Priority weight.
    pub weight: f32,
}

/// The stateful execution engine holding the graph structure.
pub struct QianjiEngine {
    /// The underlying petgraph structure.
    pub graph: StableGraph<QianjiNode, QianjiEdge, Directed>,
}

impl QianjiEngine {
    /// Creates an empty engine.
    pub fn new() -> Self {
        Self {
            graph: StableGraph::new(),
        }
    }

    /// Adds a mechanism to the graph.
    pub fn add_mechanism(&mut self, id: &str, mechanism: Arc<dyn QianjiMechanism>) -> NodeIndex {
        self.graph.add_node(QianjiNode {
            id: id.to_string(),
            status: NodeStatus::Idle,
            mechanism,
        })
    }

    /// Adds a directional link between mechanisms.
    pub fn add_link(&mut self, from: NodeIndex, to: NodeIndex, label: Option<&str>, weight: f32) {
        self.graph.add_edge(
            from,
            to,
            QianjiEdge {
                label: label.map(|s| s.to_string()),
                weight,
            },
        );
    }
}

impl Default for QianjiEngine {
    fn default() -> Self {
        Self::new()
    }
}
