use crate::contracts::NodeDefinition;

pub(super) struct CommandMechanismConfig {
    pub(super) cmd: String,
    pub(super) output_key: String,
    pub(super) allow_fail: bool,
    pub(super) stop_on_empty_stdout: bool,
    pub(super) empty_reason: Option<String>,
}

pub(super) struct WriteFileMechanismConfig {
    pub(super) path: String,
    pub(super) content: String,
    pub(super) output_key: String,
}

pub(super) struct SuspendMechanismConfig {
    pub(super) reason: String,
    pub(super) prompt: String,
    pub(super) resume_key: Option<String>,
}

pub(super) fn command_mechanism_config(node_def: &NodeDefinition) -> CommandMechanismConfig {
    CommandMechanismConfig {
        cmd: string_param(node_def, "cmd").unwrap_or_default(),
        output_key: string_param(node_def, "output_key").unwrap_or_else(|| "stdout".to_string()),
        allow_fail: bool_param(node_def, "allow_fail", false),
        stop_on_empty_stdout: bool_param(node_def, "stop_on_empty_stdout", false),
        empty_reason: string_param(node_def, "empty_reason"),
    }
}

pub(super) fn write_file_mechanism_config(node_def: &NodeDefinition) -> WriteFileMechanismConfig {
    WriteFileMechanismConfig {
        path: string_param(node_def, "path")
            .or_else(|| string_param(node_def, "target_path"))
            .unwrap_or_default(),
        content: string_param(node_def, "content").unwrap_or_default(),
        output_key: string_param(node_def, "output_key")
            .unwrap_or_else(|| "write_file_result".to_string()),
    }
}

pub(super) fn suspend_mechanism_config(node_def: &NodeDefinition) -> SuspendMechanismConfig {
    SuspendMechanismConfig {
        reason: string_param(node_def, "reason").unwrap_or_else(|| "suspended".to_string()),
        prompt: string_param(node_def, "prompt")
            .unwrap_or_else(|| "Waiting for input...".to_string()),
        resume_key: optional_string_param(node_def, "resume_key"),
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
