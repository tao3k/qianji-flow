use xiuxian_wendao::{Entity, KnowledgeGraph, Relation};

pub(super) fn persist_promotion_graph(
    graph_scope: &str,
    entity: &Entity,
    topic_entity: &Entity,
    relation: &Relation,
    graph_dimension: usize,
) -> Result<(), String> {
    let mut graph = KnowledgeGraph::new();
    graph
        .load_from_valkey(graph_scope)
        .map_err(|error| format!("failed to load existing wendao graph: {error}"))?;
    graph
        .add_entity(entity.clone())
        .map_err(|error| format!("failed to add promotion entity: {error}"))?;
    graph
        .add_entity(topic_entity.clone())
        .map_err(|error| format!("failed to add topic entity: {error}"))?;
    graph
        .add_relation(relation.clone())
        .map_err(|error| format!("failed to add promotion relation: {error}"))?;
    graph
        .save_to_valkey(graph_scope, graph_dimension)
        .map_err(|error| format!("failed to save wendao graph: {error}"))?;
    Ok(())
}
