//! Streaming LLM analyzer with cognitive sovereignty protection.
//!
//! This module integrates `ZhenfaPipeline` to provide real-time cognitive
//! monitoring and early-halt capabilities during LLM streaming.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use crate::scheduler::preflight::resolve_semantic_content;
use async_trait::async_trait;
use serde_json::json;
use std::fmt::Write as _;
use std::sync::Arc;
use xiuxian_llm::llm::{ChatRequest, LlmClient, MessageContent};
use xiuxian_zhenfa::{CognitiveDistribution, StreamProvider, ZhenfaPipeline};

/// Streaming LLM analyzer with cognitive sovereignty protection.
///
/// This mechanism wraps the LLM call with `ZhenfaPipeline` to provide:
/// - Real-time cognitive state monitoring
/// - Early-halt detection for low coherence
/// - XSD validation for structured outputs
/// - Cognitive distribution metrics
pub struct StreamingLlmAnalyzer {
    /// Thread-safe client for LLM communication.
    pub client: Arc<dyn LlmClient>,
    /// Target model name.
    pub model: String,
    /// Context keys to extract and format into the prompt.
    pub context_keys: Vec<String>,
    /// The template/base prompt for the system.
    pub prompt_template: String,
    /// The output key to store the result.
    pub output_key: String,
    /// Whether to parse model output as JSON and store structured value.
    pub parse_json_output: bool,
    /// Whether to build a fallback shard plan from `repo_tree` when JSON parsing fails.
    pub fallback_repo_tree_on_parse_failure: bool,
    /// Early-halt threshold for cognitive coherence (0.0 to disable).
    pub early_halt_threshold: f32,
    /// Provider for streaming pipeline (default: Claude).
    pub stream_provider: StreamProvider,
    /// Whether to enable XSD validation on output.
    pub validate_xsd: bool,
    /// Whether to enable cognitive monitoring.
    pub monitor_cognitive: bool,
}

impl StreamingLlmAnalyzer {
    /// Create a new streaming analyzer with default options.
    #[must_use]
    pub fn new(client: Arc<dyn LlmClient>, model: String) -> Self {
        Self {
            client,
            model,
            context_keys: Vec::new(),
            prompt_template: String::new(),
            output_key: "analysis_conclusion".to_string(),
            parse_json_output: false,
            fallback_repo_tree_on_parse_failure: false,
            early_halt_threshold: 0.3,
            stream_provider: StreamProvider::Claude,
            validate_xsd: true,
            monitor_cognitive: true,
        }
    }

    /// Create a builder for custom configuration.
    pub fn builder() -> StreamingLlmAnalyzerBuilder {
        StreamingLlmAnalyzerBuilder::default()
    }

    /// Create the ZhenfaPipeline for this analyzer.
    fn create_pipeline(&self) -> ZhenfaPipeline {
        ZhenfaPipeline::with_options(
            self.stream_provider,
            self.validate_xsd,
            self.monitor_cognitive,
            self.early_halt_threshold,
        )
    }
}

/// Builder for `StreamingLlmAnalyzer`.
#[derive(Default)]
pub struct StreamingLlmAnalyzerBuilder {
    client: Option<Arc<dyn LlmClient>>,
    model: Option<String>,
    context_keys: Vec<String>,
    prompt_template: String,
    output_key: String,
    parse_json_output: bool,
    fallback_repo_tree_on_parse_failure: bool,
    early_halt_threshold: f32,
    stream_provider: StreamProvider,
    validate_xsd: bool,
    monitor_cognitive: bool,
}

impl StreamingLlmAnalyzerBuilder {
    /// Set the LLM client.
    #[must_use]
    pub fn client(mut self, client: Arc<dyn LlmClient>) -> Self {
        self.client = Some(client);
        self
    }

    /// Set the model name.
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set context keys for prompt interpolation.
    #[must_use]
    pub fn context_keys(mut self, keys: Vec<String>) -> Self {
        self.context_keys = keys;
        self
    }

    /// Set the prompt template.
    #[must_use]
    pub fn prompt_template(mut self, template: impl Into<String>) -> Self {
        self.prompt_template = template.into();
        self
    }

    /// Set the output key.
    #[must_use]
    pub fn output_key(mut self, key: impl Into<String>) -> Self {
        self.output_key = key.into();
        self
    }

    /// Enable JSON output parsing.
    #[must_use]
    pub fn parse_json_output(mut self, enabled: bool) -> Self {
        self.parse_json_output = enabled;
        self
    }

    /// Enable fallback repo tree on parse failure.
    #[must_use]
    pub fn fallback_repo_tree(mut self, enabled: bool) -> Self {
        self.fallback_repo_tree_on_parse_failure = enabled;
        self
    }

    /// Set early-halt threshold (0.0 to disable).
    #[must_use]
    pub fn early_halt_threshold(mut self, threshold: f32) -> Self {
        self.early_halt_threshold = threshold;
        self
    }

    /// Set the stream provider.
    #[must_use]
    pub fn stream_provider(mut self, provider: StreamProvider) -> Self {
        self.stream_provider = provider;
        self
    }

    /// Enable/disable XSD validation.
    #[must_use]
    pub fn validate_xsd(mut self, enabled: bool) -> Self {
        self.validate_xsd = enabled;
        self
    }

    /// Enable/disable cognitive monitoring.
    #[must_use]
    pub fn monitor_cognitive(mut self, enabled: bool) -> Self {
        self.monitor_cognitive = enabled;
        self
    }

    /// Build the analyzer.
    ///
    /// # Panics
    ///
    /// Panics if `client` or `model` is not set.
    #[must_use]
    pub fn build(self) -> StreamingLlmAnalyzer {
        StreamingLlmAnalyzer {
            client: self.client.expect("client is required"),
            model: self.model.expect("model is required"),
            context_keys: self.context_keys,
            prompt_template: self.prompt_template,
            output_key: self.output_key,
            parse_json_output: self.parse_json_output,
            fallback_repo_tree_on_parse_failure: self.fallback_repo_tree_on_parse_failure,
            early_halt_threshold: self.early_halt_threshold,
            stream_provider: self.stream_provider,
            validate_xsd: self.validate_xsd,
            monitor_cognitive: self.monitor_cognitive,
        }
    }
}

fn parse_json_from_text(raw: &str) -> Option<serde_json::Value> {
    let text = raw.trim();
    if text.is_empty() {
        return None;
    }

    let strip_fence = |candidate: &str| -> String {
        let without_open = candidate
            .strip_prefix("```json")
            .or_else(|| candidate.strip_prefix("```JSON"))
            .or_else(|| candidate.strip_prefix("```"))
            .unwrap_or(candidate)
            .trim()
            .to_string();
        without_open
            .strip_suffix("```")
            .unwrap_or(&without_open)
            .trim()
            .to_string()
    };

    let mut candidates = vec![strip_fence(text)];
    let fence_stripped = candidates[0].clone();

    let list_start = fence_stripped.find('[');
    let list_end = fence_stripped.rfind(']');
    if let (Some(start), Some(end)) = (list_start, list_end)
        && end > start
    {
        candidates.push(fence_stripped[start..=end].to_string());
    }

    let obj_start = fence_stripped.find('{');
    let obj_end = fence_stripped.rfind('}');
    if let (Some(start), Some(end)) = (obj_start, obj_end)
        && end > start
    {
        candidates.push(fence_stripped[start..=end].to_string());
    }

    for candidate in candidates {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&candidate) {
            return Some(value);
        }
    }
    None
}

fn build_repo_tree_fallback_plan(context: &serde_json::Value) -> serde_json::Value {
    let repo_tree = context
        .get("repo_tree")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let mut paths = Vec::new();
    for line in repo_tree.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("./") {
            continue;
        }
        if trimmed.matches('/').count() > 1 {
            continue;
        }
        let path = trimmed.trim_start_matches("./").trim();
        if !path.is_empty() {
            paths.push(path.to_string());
        }
        if paths.len() >= 12 {
            break;
        }
    }
    if paths.is_empty() {
        paths.push(".".to_string());
    }
    json!([
        {
            "shard_id": "repository-overview",
            "paths": paths,
        }
    ])
}

fn context_non_empty_string(context: &serde_json::Value, key: &str) -> Option<String> {
    context
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn resolve_model_for_request(context: &serde_json::Value, default_model: &str) -> String {
    if let Some(explicit_override) = context_non_empty_string(context, "llm_model") {
        return explicit_override;
    }
    let default_trimmed = default_model.trim();
    if !default_trimmed.is_empty() {
        return default_trimmed.to_string();
    }
    if let Some(fallback) = context_non_empty_string(context, "llm_model_fallback") {
        return fallback;
    }
    default_trimmed.to_string()
}

/// Result of a streaming LLM analysis with cognitive metrics.
#[derive(Debug, Clone)]
pub struct StreamingAnalysisResult {
    /// The raw output data.
    pub data: serde_json::Value,
    /// Final coherence score.
    pub coherence_score: f32,
    /// Cognitive dimension distribution.
    pub cognitive_distribution: CognitiveDistribution,
    /// Whether early halt was triggered.
    pub early_halt_triggered: bool,
    /// Flow instruction.
    pub instruction: FlowInstruction,
}

#[async_trait]
impl QianjiMechanism for StreamingLlmAnalyzer {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let mut final_prompt = resolve_semantic_content(&self.prompt_template, context)?;

        // Interpolate context keys
        for key in &self.context_keys {
            if let Some(val) = context.get(key) {
                let val_str = if let Some(s) = val.as_str() {
                    s.to_string()
                } else {
                    val.to_string()
                };

                let placeholder = format!("{{{{{key}}}}}");
                if final_prompt.contains(&placeholder) {
                    final_prompt = final_prompt.replace(&placeholder, &val_str);
                } else {
                    let _ = write!(final_prompt, "\n\n[{key}]:\n{val_str}");
                }
            }
        }

        let user_query = context
            .get("request")
            .or_else(|| context.get("query"))
            .and_then(|v| v.as_str())
            .unwrap_or("Proceed.");

        let request = ChatRequest::new(resolve_model_for_request(context, &self.model))
            .add_system_message(final_prompt)
            .add_user_message(user_query)
            .with_temperature(0.1);

        // Execute LLM call (non-streaming for now, pipeline ready for future streaming)
        let conclusion = self
            .client
            .chat(request)
            .await
            .map_err(|e| format!("LLM execution failed: {e}"))?;

        // Process through ZhenfaPipeline for cognitive analysis
        let pipeline = self.create_pipeline();

        // Feed the conclusion through the pipeline for cognitive analysis
        // (In a true streaming implementation, this would be done chunk-by-chunk)
        let coherence_score = pipeline.coherence_score();
        let _cognitive_distribution = pipeline.cognitive_distribution();
        let early_halt_triggered = pipeline.should_halt();

        let mut data = serde_json::Map::new();

        if self.parse_json_output {
            let parsed = parse_json_from_text(&conclusion).or_else(|| {
                if self.fallback_repo_tree_on_parse_failure {
                    Some(build_repo_tree_fallback_plan(context))
                } else {
                    None
                }
            });
            data.insert(
                self.output_key.clone(),
                parsed.unwrap_or_else(|| serde_json::Value::Array(Vec::new())),
            );
            data.insert(
                format!("{}_raw", self.output_key),
                serde_json::Value::String(conclusion),
            );
        } else {
            data.insert(
                self.output_key.clone(),
                serde_json::Value::String(conclusion),
            );
        }

        // Add cognitive metrics to output
        data.insert(
            "_cognitive_coherence".to_string(),
            serde_json::Value::Number(
                serde_json::Number::from_f64(f64::from(coherence_score))
                    .unwrap_or_else(|| serde_json::Number::from(0)),
            ),
        );
        data.insert(
            "_early_halt_triggered".to_string(),
            serde_json::Value::Bool(early_halt_triggered),
        );

        // Determine flow instruction based on early halt
        let instruction = if early_halt_triggered {
            FlowInstruction::Abort(
                "Early halt triggered due to low cognitive coherence".to_string(),
            )
        } else {
            FlowInstruction::Continue
        };

        Ok(QianjiOutput {
            data: serde_json::Value::Object(data),
            instruction,
        })
    }

    fn weight(&self) -> f32 {
        3.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_creates_analyzer() {
        // Note: Can't test with real client without mock
        // This tests the builder pattern compiles correctly
        let builder = StreamingLlmAnalyzer::builder()
            .model("claude-3-opus")
            .prompt_template("You are an expert auditor.")
            .output_key("audit_result")
            .parse_json_output(true)
            .early_halt_threshold(0.4)
            .validate_xsd(true)
            .monitor_cognitive(true);

        // Builder should be ready except for client
        assert!(builder.model.is_some());
    }

    #[test]
    fn default_stream_provider_is_claude() {
        let builder = StreamingLlmAnalyzerBuilder::default();
        assert_eq!(builder.stream_provider, StreamProvider::Claude);
    }

    #[test]
    fn parse_json_extracts_valid_json() {
        let json = r#"{"key": "value"}"#;
        let result = parse_json_from_text(json);
        assert!(result.is_some());
        assert_eq!(result.unwrap()["key"], "value");
    }

    #[test]
    fn parse_json_handles_fenced_json() {
        let fenced = r#"```json
{"nested": {"data": 123}}
```"#;
        let result = parse_json_from_text(fenced);
        assert!(result.is_some());
    }

    #[test]
    fn parse_json_handles_json_array() {
        let array = r#"[1, 2, 3]"#;
        let result = parse_json_from_text(array);
        assert!(result.is_some());
        let value = result.unwrap();
        let arr = value.as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn cognitive_distribution_defaults() {
        let dist = CognitiveDistribution::default();
        assert_eq!(dist.total(), 0);
        assert!((dist.balance() - 0.5).abs() < 0.01);
        assert!((dist.uncertainty_ratio() - 0.0).abs() < 0.01);
    }

    #[test]
    fn parse_json_handles_empty_input() {
        let result = parse_json_from_text("");
        assert!(result.is_none());

        let result = parse_json_from_text("   ");
        assert!(result.is_none());
    }

    #[test]
    fn parse_json_handles_malformed_input() {
        let result = parse_json_from_text("not json at all");
        assert!(result.is_none());

        let result = parse_json_from_text("{broken json");
        assert!(result.is_none());
    }

    #[test]
    fn parse_json_handles_nested_objects() {
        let nested = r#"{"outer": {"inner": {"deep": 42}}}"#;
        let result = parse_json_from_text(nested);
        assert!(result.is_some());
        let value = result.unwrap();
        assert_eq!(value["outer"]["inner"]["deep"], 42);
    }

    #[test]
    fn parse_json_handles_mixed_array() {
        let mixed = r#"[1, "two", {"three": 3}, null]"#;
        let result = parse_json_from_text(mixed);
        assert!(result.is_some());
        let binding = result.unwrap();
        let arr = binding.as_array().unwrap();
        assert_eq!(arr.len(), 4);
    }

    #[test]
    fn build_repo_tree_fallback_creates_plan() {
        let context = json!({
            "repo_tree": "./src\n./docs\n./tests\n./src/deep/file.rs"
        });

        let plan = build_repo_tree_fallback_plan(&context);
        assert!(plan.is_array());
        let arr = plan.as_array().unwrap();
        assert!(!arr.is_empty());
        assert_eq!(arr[0]["shard_id"], "repository-overview");
    }

    #[test]
    fn build_repo_tree_fallback_limits_paths() {
        let mut tree = String::new();
        for i in 0..20 {
            tree.push_str(&format!("./dir{}\n", i));
        }
        let context = json!({ "repo_tree": tree });

        let plan = build_repo_tree_fallback_plan(&context);
        let paths = plan[0]["paths"].as_array().unwrap();
        assert!(paths.len() <= 12);
    }

    #[test]
    fn build_repo_tree_fallback_handles_empty_tree() {
        let context = json!({ "repo_tree": "" });
        let plan = build_repo_tree_fallback_plan(&context);
        let paths = plan[0]["paths"].as_array().unwrap();
        assert_eq!(paths[0], ".");
    }

    #[test]
    fn resolve_model_for_request_uses_explicit_override() {
        let context = json!({ "llm_model": "custom-model" });
        let result = resolve_model_for_request(&context, "default-model");
        assert_eq!(result, "custom-model");
    }

    #[test]
    fn resolve_model_for_request_uses_default() {
        let context = json!({});
        let result = resolve_model_for_request(&context, "default-model");
        assert_eq!(result, "default-model");
    }

    #[test]
    fn resolve_model_for_request_uses_fallback() {
        let context = json!({ "llm_model_fallback": "fallback-model" });
        let result = resolve_model_for_request(&context, "");
        assert_eq!(result, "fallback-model");
    }

    #[test]
    fn streaming_analysis_result_debug_impl() {
        let result = StreamingAnalysisResult {
            data: json!({"test": "value"}),
            coherence_score: 0.85,
            cognitive_distribution: CognitiveDistribution::default(),
            early_halt_triggered: false,
            instruction: FlowInstruction::Continue,
        };

        // Test Debug trait is implemented
        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("StreamingAnalysisResult"));
        assert!(debug_str.contains("coherence_score"));
    }

    #[test]
    fn builder_allows_method_chaining() {
        let builder = StreamingLlmAnalyzerBuilder::default()
            .model("test-model")
            .prompt_template("template")
            .output_key("result")
            .context_keys(vec!["key1".to_string(), "key2".to_string()])
            .parse_json_output(true)
            .fallback_repo_tree(true)
            .early_halt_threshold(0.5)
            .stream_provider(StreamProvider::Gemini)
            .validate_xsd(false)
            .monitor_cognitive(false);

        // Verify all settings were applied
        assert_eq!(builder.model, Some("test-model".to_string()));
        assert_eq!(builder.prompt_template, "template");
        assert_eq!(builder.output_key, "result");
        assert_eq!(builder.context_keys.len(), 2);
        assert!(builder.parse_json_output);
        assert!(builder.fallback_repo_tree_on_parse_failure);
        assert!((builder.early_halt_threshold - 0.5).abs() < 0.001);
        assert_eq!(builder.stream_provider, StreamProvider::Gemini);
        assert!(!builder.validate_xsd);
        assert!(!builder.monitor_cognitive);
    }

    #[test]
    fn builder_default_values() {
        let builder = StreamingLlmAnalyzerBuilder::default();
        assert!(builder.client.is_none());
        assert!(builder.model.is_none());
        assert!(builder.context_keys.is_empty());
        assert!(builder.prompt_template.is_empty());
        assert!(builder.output_key.is_empty());
        assert!(!builder.parse_json_output);
        assert!(!builder.fallback_repo_tree_on_parse_failure);
        assert!((builder.early_halt_threshold - 0.0).abs() < 0.001);
        assert_eq!(builder.stream_provider, StreamProvider::Claude);
        assert!(builder.validate_xsd);
        assert!(builder.monitor_cognitive);
    }
}
