//! Compiler for declarative Qianji manifests.

use crate::QianjiLlmClient;
use crate::engine::QianjiEngine;
use crate::error::QianjiError;
use std::sync::Arc;
use xiuxian_qianhuan::orchestrator::ThousandFacesOrchestrator;
use xiuxian_qianhuan::persona::PersonaRegistry;
use xiuxian_wendao::LinkGraphIndex;

mod annotation;
mod calibration;
mod formal_audit;
mod graph_assembly;
mod io_mechanisms;
#[cfg(feature = "llm")]
mod llm_client;
#[cfg(feature = "llm")]
mod llm_node;
mod manifest_parser;
mod mechanism_dispatch;
mod router;
mod security_scan;
mod stateful_mechanisms;
mod task_mechanisms;
mod task_type;
mod topology_validation;
mod wendao_ingester;
mod wendao_refresh;

/// Orchestrates the conversion of TOML manifests into executable engines.
pub struct QianjiCompiler {
    index: Arc<LinkGraphIndex>,
    orchestrator: Arc<ThousandFacesOrchestrator>,
    registry: Arc<PersonaRegistry>,
    #[cfg(feature = "llm")]
    llm_client: Option<Arc<QianjiLlmClient>>,
}

impl QianjiCompiler {
    /// Creates a new compiler with provided trinity dependencies.
    #[cfg(feature = "llm")]
    #[must_use]
    pub fn new(
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Self {
        Self {
            index,
            orchestrator,
            registry,
            llm_client,
        }
    }

    /// Creates a new compiler with provided trinity dependencies.
    #[cfg(not(feature = "llm"))]
    #[must_use]
    pub fn new(
        index: Arc<LinkGraphIndex>,
        orchestrator: Arc<ThousandFacesOrchestrator>,
        registry: Arc<PersonaRegistry>,
        _llm_client: Option<Arc<QianjiLlmClient>>,
    ) -> Self {
        Self {
            index,
            orchestrator,
            registry,
        }
    }

    /// Compiles a TOML manifest into a ready-to-run `QianjiEngine`.
    ///
    /// # Errors
    ///
    /// Returns [`QianjiError`] when TOML parsing fails, a task type is unsupported,
    /// required dependencies are missing, manifest edges reference unknown nodes,
    /// or the graph contains static cycles.
    pub fn compile(&self, manifest_toml: &str) -> Result<QianjiEngine, QianjiError> {
        let manifest = manifest_parser::parse(manifest_toml)?;
        let mut engine = QianjiEngine::new();
        let id_to_index = graph_assembly::add_manifest_nodes(
            &mut engine,
            manifest.nodes,
            |node_def| mechanism_dispatch::build(self, node_def),
            annotation::node_execution_affinity,
        )?;
        graph_assembly::add_manifest_edges(&mut engine, &id_to_index, manifest.edges)?;
        topology_validation::ensure_static_acyclic(&engine)?;

        Ok(engine)
    }
}
