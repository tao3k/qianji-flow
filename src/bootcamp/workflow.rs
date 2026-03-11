use super::llm::resolve_bootcamp_llm_client;
use super::manifest::{parse_manifest, parsed_manifest_requires_llm, resolve_flow_manifest_toml};
use super::runtime::{build_link_graph_index, unix_timestamp_millis};
use super::{BootcampRunOptions, BootcampVfsMount, WorkflowReport};
use crate::QianjiApp;
use crate::error::QianjiError;
use crate::scheduler::preflight::{RuntimeWendaoMount, install_runtime_wendao_mounts};
use serde_json::Value;
use std::sync::Arc;
use std::time::Instant;
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};

/// Runs one workflow manifest resolved from a canonical `wendao://` URI.
///
/// This is the high-level "laboratory" entrypoint:
/// 1. resolve manifest URI from embedded Wendao resources,
/// 2. hydrate compiler dependencies (index/orchestrator/registry),
/// 3. compile and execute through `QianjiScheduler`,
/// 4. return execution metadata plus final context.
///
/// # Errors
///
/// Returns [`QianjiError`] when URI resolution, manifest parsing, dependency
/// bootstrap, workflow compilation, or runtime execution fails.
pub async fn run_workflow(
    flow_uri: &str,
    initial_context: Value,
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    run_workflow_with_mounts(flow_uri, initial_context, &[], options).await
}

/// Runs one workflow manifest with optional extra embedded VFS mounts.
///
/// Mounts are used during initial flow TOML loading. When the same URI exists
/// in both extra mounts and Wendao built-in embedded registry, extra mounts
/// take precedence.
///
/// # Errors
///
/// Returns [`QianjiError`] when URI resolution, manifest parsing, dependency
/// bootstrap, workflow compilation, or runtime execution fails.
pub async fn run_workflow_with_mounts(
    flow_uri: &str,
    initial_context: Value,
    vfs_mounts: &[BootcampVfsMount],
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    let trimmed_flow_uri = flow_uri.trim();
    if trimmed_flow_uri.is_empty() {
        return Err(QianjiError::Topology(
            "bootcamp flow URI must be non-empty".to_string(),
        ));
    }

    let manifest_toml = resolve_flow_manifest_toml(trimmed_flow_uri, vfs_mounts)?;
    let manifest = parse_manifest(manifest_toml.as_str())?;
    let requires_llm = parsed_manifest_requires_llm(&manifest);

    let BootcampRunOptions {
        repo_path,
        session_id,
        redis_url,
        genesis_rules,
        index,
        orchestrator,
        persona_registry,
        llm_mode,
        consensus_manager,
    } = options;

    let index = match index {
        Some(index) => index,
        None => Arc::new(build_link_graph_index(repo_path.as_deref())?),
    };
    let orchestrator = orchestrator
        .unwrap_or_else(|| Arc::new(ThousandFacesOrchestrator::new(genesis_rules, None)));
    let registry = persona_registry.unwrap_or_else(|| Arc::new(PersonaRegistry::with_builtins()));
    let llm_client = resolve_bootcamp_llm_client(requires_llm, llm_mode)?;
    let scheduler = QianjiApp::create_pipeline_from_manifest_with_consensus(
        manifest_toml.as_str(),
        index,
        orchestrator,
        registry,
        llm_client,
        consensus_manager,
    )?;
    let runtime_mounts = vfs_mounts
        .iter()
        .copied()
        .map(RuntimeWendaoMount::from)
        .collect::<Vec<_>>();
    let _mount_guard = install_runtime_wendao_mounts(runtime_mounts);

    let started_at_unix_ms = unix_timestamp_millis()?;
    let started_at = Instant::now();
    let final_context = scheduler
        .run_with_checkpoint(initial_context, session_id, redis_url)
        .await?;
    let finished_at_unix_ms = unix_timestamp_millis()?;
    let duration_ms = started_at.elapsed().as_millis();

    Ok(WorkflowReport {
        flow_uri: trimmed_flow_uri.to_string(),
        manifest_name: manifest.name,
        node_count: manifest.nodes.len(),
        edge_count: manifest.edges.len(),
        requires_llm,
        started_at_unix_ms,
        finished_at_unix_ms,
        duration_ms,
        final_context,
    })
}

/// Compatibility alias of [`run_workflow`] for scenario-style callers.
///
/// This API accepts extra `include_dir` mounts so domain crates can provide
/// embedded resources directly without requiring hardcoded path wiring.
///
/// # Errors
///
/// Returns the same errors as [`run_workflow_with_mounts`].
pub async fn run_scenario(
    flow_uri: &str,
    initial_context: Value,
    vfs_mounts: &[BootcampVfsMount],
    options: BootcampRunOptions,
) -> Result<WorkflowReport, QianjiError> {
    run_workflow_with_mounts(flow_uri, initial_context, vfs_mounts, options).await
}
