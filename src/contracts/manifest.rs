use serde::{Deserialize, Serialize};

use crate::consensus::ConsensusPolicy;

use super::{NodeLlmBinding, NodeQianhuanBinding};

/// Definition of a node in the declarative manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeDefinition {
    /// Unique identifier for the node.
    pub id: String,
    /// Type of task (e.g., knowledge, annotation).
    pub task_type: String,
    /// Priority weight for scheduling.
    pub weight: f32,
    /// Task-specific parameters.
    pub params: serde_json::Value,
    /// Optional node-level Qianhuan binding metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub qianhuan: Option<NodeQianhuanBinding>,
    /// Optional node-level LLM tenant binding metadata.
    ///
    /// Backward compatibility:
    /// - preferred table: `[nodes.llm]`
    /// - legacy alias: `[nodes.llm_config]`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[serde(alias = "llm_config")]
    pub llm: Option<NodeLlmBinding>,
    /// Optional consensus policy for distributed voting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consensus: Option<ConsensusPolicy>,
}

/// Definition of an edge between nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDefinition {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Optional label for branch selection.
    pub label: Option<String>,
    /// Transition weight.
    pub weight: f32,
}

/// Declarative manifest for a Qianji workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QianjiManifest {
    /// Name of the pipeline.
    pub name: String,
    /// All node definitions.
    pub nodes: Vec<NodeDefinition>,
    /// All edge definitions.
    #[serde(default)]
    pub edges: Vec<EdgeDefinition>,
}
