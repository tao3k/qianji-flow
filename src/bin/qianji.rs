//! Qianji (千机) - The automated execution engine binary.
//!
//! This binary provides the entrypoint for compiling manifests and executing
//! long-running agentic workflows within the Xiuxian ecosystem.

use std::env;
use std::fs;
use std::io;
use std::sync::Arc;
use xiuxian_llm::llm::{OpenAICompatibleClient, OpenAIWireApi};
use xiuxian_logging::{init, split_logging_args};
use xiuxian_qianhuan::{orchestrator::ThousandFacesOrchestrator, persona::PersonaRegistry};
use xiuxian_qianji::layout::{QgsTheme, QianjiLayoutEngine, generate_bpmn_xml};
use xiuxian_qianji::manifest_requires_llm;
use xiuxian_qianji::runtime_config::resolve_qianji_runtime_llm_config;
use xiuxian_qianji::{QianjiCompiler, QianjiLlmClient, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

/// Main entry point for the Qianji execution engine.
///
/// # Errors
/// Returns an error if environment resolution, compilation, or execution fails.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_args: Vec<String> = env::args().collect();
    let (log_settings, args) = split_logging_args(&raw_args);
    init("xiuxian_qianji", &log_settings)?;

    // Support "graph" subcommand: qianji graph <manifest_path> <output_path>
    if args.len() >= 4 && args[1] == "graph" {
        return handle_graph_export(&args[2], &args[3]);
    }

    if args.len() < 4 {
        eprintln!("Usage:");
        eprintln!(
            "  Execution: qianji [-v|--log-verbose] <repo_path> <manifest_path> <context_json> [session_id]"
        );
        eprintln!("  Graph:     qianji [-v|--log-verbose] graph <manifest_path> <output_path>");
        std::process::exit(1);
    }

    let repo_path = &args[1];
    let manifest_path = &args[2];
    let context_json = &args[3];
    let session_id = args.get(4).cloned();

    let manifest_toml = fs::read_to_string(manifest_path).map_err(|e| {
        io::Error::other(format!(
            "Failed to read manifest file at {manifest_path}: {e}"
        ))
    })?;

    let mut context: serde_json::Value = serde_json::from_str(context_json).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Failed to parse context_json as valid JSON: {e}"),
        )
    })?;

    let requires_llm = manifest_requires_llm(&manifest_toml).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to inspect manifest for llm requirements: {e}"),
        )
    })?;
    let llm_runtime = if requires_llm {
        let resolved = resolve_qianji_runtime_llm_config().map_err(|e| {
            io::Error::other(format!(
                "Failed to resolve Qianji runtime config from qianji.toml: {e}"
            ))
        })?;
        inject_llm_model_fallback_if_missing(&mut context, &resolved.model);
        Some(resolved)
    } else {
        None
    };

    let redis_url = env::var("VALKEY_URL")
        .ok()
        .unwrap_or_else(|| "redis://localhost:6379/0".to_string());

    println!("Initializing Qianji Engine on: {repo_path}");
    if let Some(runtime) = llm_runtime.as_ref() {
        println!(
            "Resolved Qianji LLM runtime config: model='{}', base_url='{}', api_key_env='{}', wire_api='{}'",
            runtime.model, runtime.base_url, runtime.api_key_env, runtime.wire_api
        );
    } else {
        println!("Manifest has no llm nodes; skipping Qianji LLM runtime initialization.");
    }

    let index = Arc::new(
        match LinkGraphIndex::build(std::path::Path::new(repo_path)) {
            Ok(index) => index,
            Err(primary_error) => {
                LinkGraphIndex::build(std::env::temp_dir().as_path()).map_err(|fallback_error| {
                    io::Error::other(format!(
                        "Failed to build LinkGraph index at repo path ({primary_error}); \
fallback temp index also failed ({fallback_error})"
                    ))
                })?
            }
        },
    );

    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));

    let registry = PersonaRegistry::with_builtins();
    let llm_client: Option<Arc<QianjiLlmClient>> = llm_runtime.as_ref().map(|runtime| {
        Arc::new(OpenAICompatibleClient {
            api_key: runtime.api_key.clone(),
            base_url: runtime.base_url.clone(),
            wire_api: OpenAIWireApi::parse(Some(runtime.wire_api.as_str())),
            http: reqwest::Client::new(),
        }) as Arc<QianjiLlmClient>
    });

    let compiler = QianjiCompiler::new(index, orchestrator, Arc::new(registry), llm_client);
    let engine = compiler.compile(&manifest_toml)?;
    let scheduler = QianjiScheduler::new(engine);

    println!("Executing Context: {context_json}");

    let result = scheduler
        .run_with_checkpoint(context, session_id, Some(redis_url))
        .await?;

    println!("\n=== Final Qianji Execution Result ===");
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}

fn handle_graph_export(
    manifest_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Qianji Graph from: {manifest_path}");

    let manifest_toml = fs::read_to_string(manifest_path)?;

    // Using simple defaults for the compiler as we only need the topology
    let index = Arc::new(LinkGraphIndex::build(std::env::temp_dir().as_path())?);
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new("Visualizer".into(), None));
    let registry = Arc::new(PersonaRegistry::with_builtins());

    // Provide a dummy client to satisfy compilation check for LLM nodes
    let llm_client: Option<Arc<QianjiLlmClient>> = Some(Arc::new(NoopLlmClient));

    let compiler = QianjiCompiler::new(index, orchestrator, registry, llm_client);
    let engine = compiler.compile(&manifest_toml)?;

    let layout_engine = QianjiLayoutEngine::new(QgsTheme::default());
    let layout_result = layout_engine.compute_from_engine(&engine);
    let bpmn_xml = generate_bpmn_xml(&layout_result);

    // Export rich knowledge graph for 3D view
    let obsidian_graph = QianjiLayoutEngine::compute_obsidian_graph(&engine);

    let obsidian_path = format!(
        "{}_obsidian.json",
        output_path.strip_suffix(".bpmn").unwrap_or(output_path)
    );
    fs::write(
        &obsidian_path,
        serde_json::to_string_pretty(&obsidian_graph)?,
    )?;

    fs::write(output_path, bpmn_xml)?;
    println!("Successfully exported BPMN XML to: {output_path}");
    println!("Successfully exported Obsidian Graph to: {obsidian_path}");

    Ok(())
}

struct NoopLlmClient;

#[async_trait::async_trait]
impl xiuxian_llm::llm::LlmClient for NoopLlmClient {
    async fn chat(
        &self,
        _request: xiuxian_llm::llm::ChatRequest,
    ) -> xiuxian_llm::llm::LlmResult<String> {
        Ok("noop".into())
    }

    async fn chat_stream(
        &self,
        _request: xiuxian_llm::llm::ChatRequest,
    ) -> xiuxian_llm::llm::LlmResult<xiuxian_llm::llm::client::ChatStream> {
        use futures::stream;
        Ok(Box::pin(stream::iter(vec![Ok("noop".to_string())])))
    }
}

fn inject_llm_model_fallback_if_missing(context: &mut serde_json::Value, default_model: &str) {
    let Some(map) = context.as_object_mut() else {
        return;
    };

    let has_explicit_model = map
        .get("llm_model")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    let has_fallback_model = map
        .get("llm_model_fallback")
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .is_some_and(|value| !value.is_empty());
    if has_explicit_model || has_fallback_model {
        return;
    }

    map.insert(
        "llm_model_fallback".to_string(),
        serde_json::Value::String(default_model.to_string()),
    );
}
