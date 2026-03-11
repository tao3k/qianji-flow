use super::super::constants::{
    DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION, DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE,
    DEFAULT_MEMORY_PROMOTION_PERSIST, DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT,
};
use super::super::env_vars::{
    env_var_or_override, normalize_non_empty, parse_bool_env_override, parse_usize_env_override,
};
use super::super::model::{QianjiRuntimeEnv, QianjiRuntimeWendaoIngesterConfig};
use super::super::toml_config::QianjiTomlWendaoIngester;
use std::io;
use xiuxian_macros::string_first_non_empty;

pub(super) fn resolve_qianji_runtime_wendao_ingester(
    file_wendao: &QianjiTomlWendaoIngester,
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeWendaoIngesterConfig> {
    let graph_scope = string_first_non_empty!(
        runtime_env.qianji_memory_promotion_graph_scope.as_deref(),
        env_var_or_override(runtime_env, "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE").as_deref(),
        file_wendao.graph_scope.as_deref(),
        Some(DEFAULT_MEMORY_PROMOTION_GRAPH_SCOPE),
    );
    let graph_scope_key = normalize_non_empty(Some(string_first_non_empty!(
        runtime_env
            .qianji_memory_promotion_graph_scope_key
            .as_deref(),
        env_var_or_override(runtime_env, "QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY").as_deref(),
        file_wendao.graph_scope_key.as_deref(),
    )));

    let graph_dimension = runtime_env
        .qianji_memory_promotion_graph_dimension
        .or_else(|| {
            parse_usize_env_override(runtime_env, "QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION")
        })
        .or(file_wendao.graph_dimension)
        .unwrap_or(DEFAULT_MEMORY_PROMOTION_GRAPH_DIMENSION);

    let persist = runtime_env
        .qianji_memory_promotion_persist
        .or_else(|| parse_bool_env_override(runtime_env, "QIANJI_MEMORY_PROMOTION_PERSIST"))
        .or(file_wendao.persist)
        .unwrap_or(DEFAULT_MEMORY_PROMOTION_PERSIST);

    let persist_best_effort = runtime_env
        .qianji_memory_promotion_persist_best_effort
        .or_else(|| {
            parse_bool_env_override(runtime_env, "QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT")
        })
        .or(file_wendao.persist_best_effort)
        .unwrap_or(DEFAULT_MEMORY_PROMOTION_PERSIST_BEST_EFFORT);

    Ok(QianjiRuntimeWendaoIngesterConfig {
        graph_scope,
        graph_scope_key,
        graph_dimension,
        persist,
        persist_best_effort,
    })
}
