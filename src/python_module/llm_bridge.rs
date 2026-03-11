use std::path::Path;
use std::sync::Arc;

use pyo3::prelude::*;

use crate::scheduler::QianjiScheduler;
use xiuxian_llm::llm::OpenAIClient;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_wendao::LinkGraphIndex;

use super::runtime::{create_tokio_runtime, serialize_json_result};

/// Runs the built-in master-research workflow and returns the final context as JSON.
///
/// # Errors
///
/// Returns `PyRuntimeError` for runtime, indexing, compilation, or scheduler
/// failures and `PyValueError` when result serialization fails.
#[pyfunction]
pub fn run_master_research_array(
    py: Python<'_>,
    repo_path: &str,
    query: &str,
    api_key: &str,
    base_url: &str,
) -> PyResult<String> {
    let repo_path = repo_path.to_string();
    let query = query.to_string();
    let api_key = api_key.to_string();
    let base_url = base_url.to_string();
    py.detach(move || {
        let runtime = create_tokio_runtime()?;
        runtime.block_on(async move {
            let index = Arc::new(
                LinkGraphIndex::build(Path::new(&repo_path))
                    .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.clone()))?,
            );
            let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Rules".to_string(), None));
            let registry = Arc::new(PersonaRegistry::with_builtins());
            let llm_client = Arc::new(OpenAIClient {
                api_key,
                base_url,
                http: reqwest::Client::new(),
            });

            let compiler = crate::engine::compiler::QianjiCompiler::new(
                index,
                orchestrator,
                registry,
                Some(llm_client),
            );

            let master_toml = include_str!("../../resources/research_master.toml");
            let engine = compiler
                .compile(master_toml)
                .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;
            let scheduler = QianjiScheduler::new(engine);

            let result = scheduler
                .run(serde_json::json!({
                    "query": query,
                }))
                .await
                .map_err(|error| pyo3::exceptions::PyRuntimeError::new_err(error.to_string()))?;

            serialize_json_result(&result)
        })
    })
}
