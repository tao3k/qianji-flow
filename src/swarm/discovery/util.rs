use std::time::{SystemTime, UNIX_EPOCH};

pub(super) const REGISTRY_INDEX_KEY: &str = "xiuxian:swarm:registry:index";

pub(super) fn normalize_optional_text(value: Option<String>) -> Option<String> {
    value
        .map(|inner| inner.trim().to_string())
        .and_then(|inner| if inner.is_empty() { None } else { Some(inner) })
}

pub(super) fn current_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
