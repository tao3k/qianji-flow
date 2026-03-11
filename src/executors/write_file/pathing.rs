use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn resolve_root_dir(context: &Value) -> Option<PathBuf> {
    for key in ["project_root", "repo_root", "notebook_root"] {
        if let Some(text) = context.get(key).and_then(Value::as_str) {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                return Some(PathBuf::from(trimmed));
            }
        }
    }
    None
}

pub(super) fn resolve_destination_path(
    destination: &Path,
    root_dir: Option<PathBuf>,
) -> Result<PathBuf, String> {
    let resolved = if let Some(root) = root_dir.as_ref() {
        if destination.is_absolute() {
            destination.to_path_buf()
        } else {
            root.join(destination)
        }
    } else {
        destination.to_path_buf()
    };

    let Some(parent) = resolved.parent() else {
        return Ok(resolved);
    };

    fs::create_dir_all(parent).map_err(|error| {
        format!(
            "write_file failed to create parent directory `{}`: {error}",
            parent.display()
        )
    })?;

    let Some(root) = root_dir else {
        return Ok(resolved);
    };

    let canonical_root = fs::canonicalize(&root).map_err(|error| {
        format!(
            "write_file failed to canonicalize root directory `{}`: {error}",
            root.display()
        )
    })?;
    let canonical_parent = fs::canonicalize(parent).map_err(|error| {
        format!(
            "write_file failed to canonicalize parent directory `{}`: {error}",
            parent.display()
        )
    })?;

    if !canonical_parent.starts_with(&canonical_root) {
        return Err(format!(
            "write_file path escapes root directory: destination=`{}`, root=`{}`",
            resolved.display(),
            canonical_root.display()
        ));
    }

    Ok(resolved)
}
