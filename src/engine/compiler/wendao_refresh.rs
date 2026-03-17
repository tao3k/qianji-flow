use crate::contracts::NodeDefinition;

pub(super) struct WendaoRefreshMechanismConfig {
    pub(super) output_key: String,
    pub(super) changed_paths_key: String,
    pub(super) root_dir_key: Option<String>,
    pub(super) root_dir: Option<String>,
    pub(super) force_full: bool,
    pub(super) prefer_incremental: bool,
    pub(super) allow_full_fallback: bool,
    pub(super) full_rebuild_threshold: Option<usize>,
    pub(super) include_dirs: Vec<String>,
    pub(super) excluded_dirs: Vec<String>,
}

pub(super) fn mechanism_config(node_def: &NodeDefinition) -> WendaoRefreshMechanismConfig {
    WendaoRefreshMechanismConfig {
        output_key: string_param(node_def, "output_key")
            .unwrap_or_else(|| "wendao_refresh".to_string()),
        changed_paths_key: string_param(node_def, "changed_paths_key")
            .unwrap_or_else(|| "changed_paths".to_string()),
        root_dir_key: string_param(node_def, "root_dir_key"),
        root_dir: string_param(node_def, "root_dir"),
        force_full: bool_param(node_def, "force_full", false),
        prefer_incremental: bool_param(node_def, "prefer_incremental", true),
        allow_full_fallback: bool_param(node_def, "allow_full_fallback", true),
        full_rebuild_threshold: usize_param(node_def, "full_rebuild_threshold"),
        include_dirs: string_list_param(node_def, "include_dirs"),
        excluded_dirs: string_list_param(node_def, "excluded_dirs"),
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

fn usize_param(node_def: &NodeDefinition, key: &str) -> Option<usize> {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .map(|value| value as usize)
}

fn string_list_param(node_def: &NodeDefinition, key: &str) -> Vec<String> {
    node_def
        .params
        .get(key)
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}
