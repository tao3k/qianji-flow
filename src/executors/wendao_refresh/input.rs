use serde_json::Value;
use std::path::PathBuf;

pub(super) fn collect_changed_paths(context: &Value, key: &str) -> Vec<PathBuf> {
    let Some(value) = context
        .get(key)
        .or_else(|| lookup_nested_value(context, key))
    else {
        return Vec::new();
    };
    match value {
        Value::String(single) => {
            let trimmed = single.trim();
            if trimmed.is_empty() {
                Vec::new()
            } else {
                vec![PathBuf::from(trimmed)]
            }
        }
        Value::Array(items) => items
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .collect(),
        _ => Vec::new(),
    }
}

fn lookup_nested_value<'a>(context: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = context;
    for segment in path.split('.') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        current = current.get(segment)?;
    }
    Some(current)
}

pub(super) fn resolve_root_dir(
    context: &Value,
    explicit: Option<&str>,
    root_dir_key: Option<&str>,
) -> Result<PathBuf, String> {
    if let Some(path) = explicit.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    if let Some(key) = root_dir_key
        && let Some(path) = context.get(key).and_then(Value::as_str)
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path.trim()));
    }

    for fallback_key in ["project_root", "repo_root", "notebook_root"] {
        if let Some(path) = context.get(fallback_key).and_then(Value::as_str)
            && !path.trim().is_empty()
        {
            return Ok(PathBuf::from(path.trim()));
        }
    }

    if let Ok(path) = std::env::var("PRJ_ROOT")
        && !path.trim().is_empty()
    {
        return Ok(PathBuf::from(path.trim()));
    }

    std::env::current_dir().map_err(|error| format!("failed to resolve current_dir: {error}"))
}
