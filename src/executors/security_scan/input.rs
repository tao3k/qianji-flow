use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) fn collect_file_paths(context: &Value, files_key: &str) -> Result<Vec<String>, String> {
    let files_val = context
        .get(files_key)
        .ok_or_else(|| format!("Missing context key: {files_key}"))?;

    let mut file_paths = Vec::new();
    if let Some(arr) = files_val.as_array() {
        for v in arr {
            if let Some(s) = v.as_str() {
                file_paths.push(s.to_string());
            }
        }
        return Ok(file_paths);
    }

    if let Some(s) = files_val.as_str() {
        for line in s.split('\n') {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                file_paths.push(trimmed.to_string());
            }
        }
        return Ok(file_paths);
    }

    Err(format!("Context key {files_key} must be a string or array"))
}

pub(super) fn resolve_base_dir<'a>(
    context: &'a Value,
    cwd_key: Option<&String>,
) -> Option<&'a Path> {
    cwd_key
        .and_then(|key| context.get(key))
        .and_then(Value::as_str)
        .map(Path::new)
}

pub(super) fn resolve_scan_path(file_str: &str, base_dir: Option<&Path>) -> PathBuf {
    let mut path_buf = PathBuf::from(file_str);
    if let Some(base) = base_dir
        && path_buf.is_relative()
    {
        path_buf = base.join(path_buf);
    }
    path_buf
}
