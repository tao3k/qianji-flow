use serde_json::Value;

const DEFAULT_GRAPH_SCOPE: &str = "qianji:memory_promotion";

pub(super) fn resolve_graph_scope(
    context: &Value,
    static_scope: Option<&String>,
    scope_key: Option<&String>,
) -> String {
    let dynamic_scope = scope_key
        .and_then(|key| context.get(key.as_str()))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|scope| !scope.is_empty())
        .map(ToString::to_string);
    let static_scope = static_scope
        .map(|scope| scope.trim())
        .filter(|scope| !scope.is_empty())
        .map(ToString::to_string);
    dynamic_scope
        .or(static_scope)
        .unwrap_or_else(|| DEFAULT_GRAPH_SCOPE.to_string())
}
