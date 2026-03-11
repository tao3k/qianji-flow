use crate::contracts::NodeDefinition;
use crate::runtime_config::{
    QianjiRuntimeWendaoIngesterConfig, resolve_qianji_runtime_wendao_ingester_config,
};

pub(super) struct WendaoIngesterMechanismConfig {
    pub(super) output_key: String,
    pub(super) graph_scope: Option<String>,
    pub(super) graph_scope_key: Option<String>,
    pub(super) graph_dimension: usize,
    pub(super) persist: bool,
    pub(super) persist_best_effort: bool,
}

pub(super) fn mechanism_config(node_def: &NodeDefinition) -> WendaoIngesterMechanismConfig {
    let runtime_defaults = resolve_qianji_runtime_wendao_ingester_config().unwrap_or_else(|error| {
        log::warn!(
            "failed to resolve qianji memory promotion runtime config; using hard defaults: {error}"
        );
        QianjiRuntimeWendaoIngesterConfig::default()
    });

    WendaoIngesterMechanismConfig {
        output_key: string_param(node_def, "output_key")
            .unwrap_or_else(|| "promotion_entity".to_string()),
        graph_scope: optional_string_param(node_def, "graph_scope")
            .or_else(|| Some(runtime_defaults.graph_scope.clone())),
        graph_scope_key: optional_string_param(node_def, "graph_scope_key")
            .or_else(|| runtime_defaults.graph_scope_key.clone()),
        graph_dimension: node_def
            .params
            .get("graph_dimension")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| usize::try_from(value).ok())
            .unwrap_or(runtime_defaults.graph_dimension),
        persist: bool_param(node_def, "persist", runtime_defaults.persist),
        persist_best_effort: bool_param(
            node_def,
            "persist_best_effort",
            runtime_defaults.persist_best_effort,
        ),
    }
}

fn bool_param(node_def: &NodeDefinition, key: &str, default: bool) -> bool {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(default)
}

fn string_param(node_def: &NodeDefinition, key: &str) -> Option<String> {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
}

fn optional_string_param(node_def: &NodeDefinition, key: &str) -> Option<String> {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}
