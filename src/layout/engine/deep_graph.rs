//! Deep graph export utilities.

use crate::engine::QianjiEngine;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};

use super::layout_core::QianjiLayoutEngine;
use super::types::{DeepEdge, DeepKnowledgeGraph, DeepNode, EntityKind};

impl QianjiLayoutEngine {
    /// Builds an Obsidian-style deep graph from the execution graph.
    #[must_use]
    pub fn compute_obsidian_graph(engine: &QianjiEngine) -> DeepKnowledgeGraph {
        let nodes = engine
            .graph
            .node_indices()
            .map(|node_idx| {
                let node = &engine.graph[node_idx];
                DeepNode {
                    id: node.id.clone(),
                    label: node.id.clone(),
                    kind: EntityKind::Mechanism,
                    parent_id: None,
                }
            })
            .collect();
        let edges = engine
            .graph
            .edge_references()
            .map(|edge| DeepEdge {
                from: engine.graph[edge.source()].id.clone(),
                to: engine.graph[edge.target()].id.clone(),
                relation: "FLOW".to_string(),
            })
            .collect();
        DeepKnowledgeGraph { nodes, edges }
    }
}
