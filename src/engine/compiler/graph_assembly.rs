use crate::contracts::{EdgeDefinition, NodeDefinition, QianjiMechanism};
use crate::engine::{NodeExecutionAffinity, QianjiEngine};
use crate::error::QianjiError;
use petgraph::stable_graph::NodeIndex;
use std::collections::HashMap;
use std::sync::Arc;

pub(super) fn add_manifest_nodes<F, A>(
    engine: &mut QianjiEngine,
    node_defs: Vec<NodeDefinition>,
    mut build_mechanism: F,
    mut resolve_affinity: A,
) -> Result<HashMap<String, NodeIndex>, QianjiError>
where
    F: FnMut(&NodeDefinition) -> Result<Arc<dyn QianjiMechanism>, QianjiError>,
    A: FnMut(&NodeDefinition) -> NodeExecutionAffinity,
{
    let mut id_to_index = HashMap::new();
    for node_def in node_defs {
        let consensus = node_def.consensus.clone();
        let mechanism = build_mechanism(&node_def)?;
        let execution_affinity = resolve_affinity(&node_def);
        let idx = engine.add_mechanism_with_affinity(
            &node_def.id,
            mechanism,
            consensus,
            execution_affinity,
        );
        id_to_index.insert(node_def.id, idx);
    }
    Ok(id_to_index)
}

pub(super) fn add_manifest_edges(
    engine: &mut QianjiEngine,
    id_to_index: &HashMap<String, NodeIndex>,
    edge_defs: Vec<EdgeDefinition>,
) -> Result<(), QianjiError> {
    for edge_def in edge_defs {
        let from_idx = node_index_by_id(id_to_index, &edge_def.from, "Source")?;
        let to_idx = node_index_by_id(id_to_index, &edge_def.to, "Target")?;
        engine.add_link(from_idx, to_idx, edge_def.label.as_deref(), edge_def.weight);
    }
    Ok(())
}

fn node_index_by_id(
    id_to_index: &HashMap<String, NodeIndex>,
    node_id: &str,
    role: &str,
) -> Result<NodeIndex, QianjiError> {
    id_to_index
        .get(node_id)
        .copied()
        .ok_or(QianjiError::Topology(format!(
            "{role} node not found: {node_id}"
        )))
}
