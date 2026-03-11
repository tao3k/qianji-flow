use super::constants::{
    DEFAULT_API_KEY_ENV, DEFAULT_BASE_URL, DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION,
    DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE, DEFAULT_MEMORY_PROMOTION_PERSIST,
    DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT, DEFAULT_MODEL,
};
use super::env_vars::{
    env_var_or_override, normalize_non_empty, parse_bool_env_override, parse_usize_env_override,
    resolve_api_key_from_env,
};
use super::loader::load_qianji_toml;
use super::model::{QianjiRuntimeEnv, QianjiRuntimeLlmConfig, QianjiRuntimeWendaoIngesterConfig};
use super::pathing::{resolve_prj_config_home, resolve_project_root};
use std::io;
use xiuxian_macros::string_first_non_empty;

/// Resolve `qianji.toml` and environment into an effective LLM runtime config.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_llm_config() -> io::Result<QianjiRuntimeLlmConfig> {
    resolve_qianji_runtime_llm_config_with_env(&QianjiRuntimeEnv::default())
}

/// Resolve config with explicit runtime environment overrides (for tests and tooling).
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_llm_config_with_env(
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let file_cfg = load_qianji_toml(runtime_env, &project_root, &config_home)?;

    let model = string_first_non_empty!(
        runtime_env.qianji_llm_model.as_deref(),
        env_var_or_override(runtime_env, "QIANJI_LLM_MODEL").as_deref(),
        file_cfg.llm.model.as_deref(),
        Some(DEFAULT_MODEL),
    );

    let base_url = string_first_non_empty!(
        runtime_env.openai_api_base.as_deref(),
        env_var_or_override(runtime_env, "OPENAI_API_BASE").as_deref(),
        file_cfg.llm.base_url.as_deref(),
        Some(DEFAULT_BASE_URL),
    );

    let api_key_env = string_first_non_empty!(
        file_cfg.llm.api_key_env.as_deref(),
        Some(DEFAULT_API_KEY_ENV),
    );

    let maybe_api_key = string_first_non_empty!(
        runtime_env.openai_api_key.as_deref(),
        resolve_api_key_from_env(runtime_env, api_key_env.as_str()).as_deref(),
    );
    let api_key = if maybe_api_key.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing Qianji API key; set OPENAI_API_KEY or {api_key_env} (resolved from qianji.toml)"
            ),
        ));
    } else {
        maybe_api_key
    };

    Ok(QianjiRuntimeLlmConfig {
        model,
        base_url,
        api_key_env,
        api_key,
    })
}

/// Resolve `qianji.toml` and environment into native `Wendao` ingestion defaults.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_wendao_ingester_config()
-> io::Result<QianjiRuntimeWendaoIngesterConfig> {
    resolve_qianji_runtime_wendao_ingester_config_with_env(&QianjiRuntimeEnv::default())
}

/// Resolve `Wendao` ingestion defaults with explicit runtime environment overrides.
///
/// # Errors
///
/// Returns [`io::Error`] when a discovered `qianji.toml` file cannot be read or parsed.
pub fn resolve_qianji_runtime_wendao_ingester_config_with_env(
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeWendaoIngesterConfig> {
    let project_root = resolve_project_root(runtime_env);
    let config_home = resolve_prj_config_home(runtime_env, &project_root);
    let file_cfg = load_qianji_toml(runtime_env, &project_root, &config_home)?;

    let graph_scope = string_first_non_empty!(
        runtime_env.qianji_memory_promotion_graph_scope.as_deref(),
        env_var_or_override(runtime_env, "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE").as_deref(),
        file_cfg.memory_promotion.wendao.graph_scope.as_deref(),
        Some(DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE),
    );
    let graph_scope_key = normalize_non_empty(Some(string_first_non_empty!(
        runtime_env
            .qianji_memory_promotion_graph_scope_key
            .as_deref(),
        env_var_or_override(runtime_env, "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY").as_deref(),
        file_cfg.memory_promotion.wendao.graph_scope_key.as_deref(),
    )));

    let graph_dimension = runtime_env
        .qianji_memory_promotion_graph_dimension
        .or_else(|| {
            parse_usize_env_override(runtime_env, "QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION")
        })
        .or(file_cfg.memory_promotion.wendao.graph_dimension)
        .unwrap_or(DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION);

    let persist = runtime_env
        .qianji_memory_promotion_persist
        .or_else(|| parse_bool_env_override(runtime_env, "QIANJI_MEMORY_PROMOTION_PERSIST"))
        .or(file_cfg.memory_promotion.wendao.persist)
        .unwrap_or(DEFAULT_MEMORY_PROMOTION_PERSIST);

    let persist_best_effort = runtime_env
        .qianji_memory_promotion_persist_best_effort
        .or_else(|| {
            parse_bool_env_override(runtime_env, "QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT")
        })
        .or(file_cfg.memory_promotion.wendao.persist_best_effort)
        .unwrap_or(DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT);

    Ok(QianjiRuntimeWendaoIngesterConfig {
        graph_scope,
        graph_scope_key,
        graph_dimension,
        persist,
        persist_best_effort,
    })
}
