use crate::contracts::NodeDefinition;

pub(super) fn target_node_id(node_def: &NodeDefinition) -> String {
    node_def
        .params
        .get("target_node_id")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .to_string()
}
