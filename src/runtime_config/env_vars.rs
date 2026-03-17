use super::model::QianjiRuntimeEnv;

pub(super) fn env_var_or_override(runtime_env: &QianjiRuntimeEnv, key: &str) -> Option<String> {
    match env_override_state(runtime_env, key) {
        EnvOverrideState::Value(value) => return Some(value),
        EnvOverrideState::Empty => return Some(String::new()),
        EnvOverrideState::Missing => {}
    }
    read_env_non_empty(key)
}

pub(super) fn resolve_api_key_from_env(
    runtime_env: &QianjiRuntimeEnv,
    api_key_env: &str,
) -> Option<String> {
    let openai_override = env_override_state(runtime_env, "OPENAI_API_KEY");
    if let EnvOverrideState::Value(value) = openai_override {
        return Some(value);
    }

    match env_override_state(runtime_env, api_key_env) {
        EnvOverrideState::Value(value) => return Some(value),
        EnvOverrideState::Empty => return None,
        EnvOverrideState::Missing => {}
    }

    if matches!(openai_override, EnvOverrideState::Empty) {
        return None;
    }

    read_first_non_empty_env(["OPENAI_API_KEY", api_key_env])
}

pub(super) fn parse_usize_env_override(runtime_env: &QianjiRuntimeEnv, key: &str) -> Option<usize> {
    env_var_or_override(runtime_env, key).and_then(|value| value.trim().parse::<usize>().ok())
}

pub(super) fn parse_bool_env_override(runtime_env: &QianjiRuntimeEnv, key: &str) -> Option<bool> {
    env_var_or_override(runtime_env, key).and_then(|value| parse_bool_flag(value.as_str()))
}

pub(super) fn normalize_non_empty(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EnvOverrideState {
    Missing,
    Empty,
    Value(String),
}

fn env_override_state(runtime_env: &QianjiRuntimeEnv, key: &str) -> EnvOverrideState {
    let Some((_, value)) = runtime_env
        .extra_env
        .iter()
        .find(|(candidate_key, _)| candidate_key == key)
    else {
        return EnvOverrideState::Missing;
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        EnvOverrideState::Empty
    } else {
        EnvOverrideState::Value(trimmed.to_string())
    }
}

fn parse_bool_flag(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn read_env_non_empty(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn read_first_non_empty_env<const N: usize>(keys: [&str; N]) -> Option<String> {
    keys.into_iter().find_map(read_env_non_empty)
}
