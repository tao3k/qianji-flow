use crate::contracts::NodeDefinition;

pub(super) struct SecurityScanMechanismConfig {
    pub(super) files_key: String,
    pub(super) output_key: String,
    pub(super) abort_on_violation: bool,
    pub(super) cwd_key: Option<String>,
}

pub(super) fn mechanism_config(node_def: &NodeDefinition) -> SecurityScanMechanismConfig {
    SecurityScanMechanismConfig {
        files_key: string_param(node_def, "files_key")
            .unwrap_or_else(|| "staged_files".to_string()),
        output_key: string_param(node_def, "output_key")
            .unwrap_or_else(|| "security_issues".to_string()),
        abort_on_violation: bool_param(node_def, "abort_on_violation", true),
        cwd_key: string_param(node_def, "cwd_key"),
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
