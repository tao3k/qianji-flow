//! Pre-execution context preflight for semantic placeholder resolution.

use serde_json::{Map, Value};
use xiuxian_wendao::embedded_resource_text_from_wendao_uri;

/// Resolves `$wendao://...` placeholders recursively before node execution.
///
/// # Errors
///
/// Returns an error when a placeholder token is empty or when one semantic URI
/// cannot be resolved from embedded Wendao resources.
pub(crate) fn resolve_wendao_placeholders_in_context(context: &Value) -> Result<Value, String> {
    resolve_value(context, context)
}

fn resolve_value(value: &Value, context: &Value) -> Result<Value, String> {
    match value {
        Value::String(raw) => resolve_string(raw, context).map(Value::String),
        Value::Array(items) => items
            .iter()
            .map(|item| resolve_value(item, context))
            .collect::<Result<Vec<_>, _>>()
            .map(Value::Array),
        Value::Object(object) => {
            let mut resolved = Map::with_capacity(object.len());
            for (key, item) in object {
                resolved.insert(key.clone(), resolve_value(item, context)?);
            }
            Ok(Value::Object(resolved))
        }
        _ => Ok(value.clone()),
    }
}

fn resolve_string(raw: &str, context: &Value) -> Result<String, String> {
    let trimmed = raw.trim();
    let Some(token) = trimmed.strip_prefix('$') else {
        return Ok(raw.to_string());
    };
    let token = token.trim();
    if token.is_empty() {
        return Err("semantic placeholder must not be empty".to_string());
    }
    if !token.starts_with("wendao://") {
        if let Some(value) = lookup_context_path(context, token)
            && let Some(text) = context_value_to_text(value)
        {
            return Ok(text);
        }
        return Ok(raw.to_string());
    }
    embedded_resource_text_from_wendao_uri(token)
        .map(str::to_string)
        .ok_or_else(|| format!("semantic resource URI `{token}` could not be resolved"))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ContextPathSegment {
    Key(String),
    Index(usize),
}

fn parse_context_path(path: &str) -> Option<Vec<ContextPathSegment>> {
    if path.is_empty() {
        return None;
    }

    let bytes = path.as_bytes();
    let mut cursor = 0usize;
    let mut segments = Vec::new();

    while cursor < bytes.len() {
        if bytes[cursor] == b'.' {
            cursor += 1;
            continue;
        }

        if bytes[cursor] == b'[' {
            cursor += 1;
            let index_start = cursor;
            while cursor < bytes.len() && bytes[cursor].is_ascii_digit() {
                cursor += 1;
            }
            if index_start == cursor || cursor >= bytes.len() || bytes[cursor] != b']' {
                return None;
            }
            let index_text = &path[index_start..cursor];
            let index = index_text.parse::<usize>().ok()?;
            segments.push(ContextPathSegment::Index(index));
            cursor += 1;
            continue;
        }

        let key_start = cursor;
        while cursor < bytes.len() && bytes[cursor] != b'.' && bytes[cursor] != b'[' {
            cursor += 1;
        }
        let key = path[key_start..cursor].trim();
        if key.is_empty() {
            return None;
        }
        segments.push(ContextPathSegment::Key(key.to_string()));
    }

    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

/// Looks up one context value using a dot/bracket semantic path.
///
/// Examples: `agenda_steward_propose.output`, `hits[0].content`.
#[must_use]
pub(crate) fn lookup_context_path<'a>(context: &'a Value, path: &str) -> Option<&'a Value> {
    let segments = parse_context_path(path)?;
    let mut current = context;

    for segment in segments {
        match segment {
            ContextPathSegment::Key(key) => match current {
                Value::Object(map) => {
                    current = map.get(&key)?;
                }
                _ => return None,
            },
            ContextPathSegment::Index(index) => match current {
                Value::Array(items) => {
                    current = items.get(index)?;
                }
                _ => return None,
            },
        }
    }
    Some(current)
}

/// Converts one context value to non-empty text for semantic placeholder use.
#[must_use]
pub(crate) fn context_value_to_text(value: &Value) -> Option<String> {
    let text = match value {
        Value::String(raw) => raw.trim().to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    };
    if text.is_empty() { None } else { Some(text) }
}
