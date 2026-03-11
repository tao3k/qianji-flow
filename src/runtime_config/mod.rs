//! Runtime configuration loader for `qianji.toml`.
//!
//! Resolution order:
//! 1. System config: `<PRJ_ROOT>/packages/rust/crates/xiuxian-qianji/resources/config/qianji.toml`
//! 2. User config: `<PRJ_CONFIG_HOME>/xiuxian-artisan-workshop/qianji.toml`
//! 3. Explicit config path: `$QIANJI_CONFIG_PATH`
//! 4. Environment overrides:
//!    - `QIANJI_LLM_MODEL`
//!    - `OPENAI_API_BASE`
//!    - `OPENAI_API_KEY`
//!    - `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE`
//!    - `QIANJI_MEMORY_PROMOTION_GRAPH_SCOPE_KEY`
//!    - `QIANJI_MEMORY_PROMOTION_GRAPH_DIMENSION`
//!    - `QIANJI_MEMORY_PROMOTION_PERSIST`
//!    - `QIANJI_MEMORY_PROMOTION_PERSIST_BEST_EFFORT`

mod constants;
mod env_vars;
mod loader;
mod model;
mod pathing;
mod resolve;
mod toml_config;

pub use model::{QianjiRuntimeEnv, QianjiRuntimeLlmConfig, QianjiRuntimeWendaoIngesterConfig};
pub use resolve::{
    resolve_qianji_runtime_llm_config, resolve_qianji_runtime_llm_config_with_env,
    resolve_qianji_runtime_wendao_ingester_config,
    resolve_qianji_runtime_wendao_ingester_config_with_env,
};
