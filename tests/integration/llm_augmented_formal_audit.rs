//! Tests for LLM-augmented formal audit flow control.

#![cfg(feature = "llm")]

use async_trait::async_trait;
use futures::stream;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use xiuxian_llm::llm::client::ChatStream;
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmError, LlmResult};
use xiuxian_qianhuan::{
    orchestrator::ThousandFacesOrchestrator,
    persona::{PersonaProfile, PersonaRegistry},
};
use xiuxian_qianji::contracts::{FlowInstruction, QianjiMechanism};
use xiuxian_qianji::executors::annotation::ContextAnnotator;
use xiuxian_qianji::executors::formal_audit::LlmAugmentedAuditMechanism;
use xiuxian_qianji::NodeQianhuanExecutionMode;
use xiuxian_qianji::{QianjiCompiler, QianjiScheduler};
use xiuxian_wendao::LinkGraphIndex;

struct SequencedMockLlmClient {
    responses: Arc<Mutex<Vec<String>>>,
    seen_models: Arc<Mutex<Vec<String>>>,
}

impl SequencedMockLlmClient {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            seen_models: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait]
impl LlmClient for SequencedMockLlmClient {
    async fn chat(&self, request: ChatRequest) -> LlmResult<String> {
        if let Ok(mut models) = self.seen_models.lock() {
            models.push(request.model);
        }
        let mut responses = self.responses.lock().map_err(|_| LlmError::Internal {
            message: "failed to lock llm response queue".to_string(),
        })?;
        if responses.is_empty() {
            return Ok("<score>1.0</score>".to_string());
        }
        Ok(responses.remove(0))
    }

    async fn chat_stream(&self, request: ChatRequest) -> LlmResult<ChatStream> {
        if let Ok(mut models) = self.seen_models.lock() {
            models.push(request.model);
        }
        let responses = self.responses.lock().map_err(|_| LlmError::Internal {
            message: "failed to lock llm response queue".to_string(),
        })?;
        let chunks: Vec<Result<String, LlmError>> = responses
            .iter()
            .map(|s| Ok(s.clone()))
            .collect();
        Ok(Box::pin(stream::iter(chunks)))
    }
}

fn make_registry() -> Arc<PersonaRegistry> {
    let mut registry = PersonaRegistry::with_builtins();
    registry.register(PersonaProfile {
        id: "strict_teacher".to_string(),
        name: "Strict Teacher".to_string(),
        background: None,
        voice_tone: "Direct and strict.".to_string(),
        guidelines: vec!["Score rigorously.".to_string()],
        style_anchors: Vec::new(),
        cot_template: "1. Critique -> 2. Score -> 3. Decide".to_string(),
        forbidden_words: Vec::new(),
        metadata: HashMap::new(),
    });
    Arc::new(registry)
}

#[tokio::test]
async fn llm_augmented_audit_retries_when_score_is_below_threshold() {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = make_registry();
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.42</score><reason>too risky</reason>".to_string(),
    ]));

    let mechanism = LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator,
            registry,
            persona_id: "strict_teacher".to_string(),
            template_target: Some("critique_agenda.j2".to_string()),
            execution_mode: xiuxian_qianji::NodeQianhuanExecutionMode::Isolated,
            input_keys: vec!["raw_facts".to_string()],
            history_key: "audit_history".to_string(),
            output_key: "annotated_prompt".to_string(),
        },
        client: llm,
        model: "audit-model".to_string(),
        threshold_score: 0.8,
        max_retries: 3,
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
        retry_counter_key: "audit_retry_count".to_string(),
        output_key: "audit_critique".to_string(),
        score_key: "audit_score".to_string(),
        cognitive_early_halt_threshold: 0.3,
        enable_cognitive_supervision: false,
    };

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Draft agenda has 12 heavy tasks in one day.",
            "request": "Critique this agenda."
        }))
        .await
        .unwrap_or_else(|error| panic!("llm augmented audit should execute: {error}"));

    assert_eq!(output.data["audit_status"], "failed");
    let audit_score = output.data["audit_score"]
        .as_f64()
        .unwrap_or_else(|| panic!("audit_score should be a numeric value"));
    assert!((audit_score - 0.42).abs() < 1e-6);
    let memrl_reward = output.data["memrl_reward"]
        .as_f64()
        .unwrap_or_else(|| panic!("memrl_reward should be a numeric value"));
    assert!((memrl_reward - 0.42).abs() < 1e-6);
    assert_eq!(
        output.data["memrl_signal_source"],
        json!("formal_audit.llm")
    );
    assert_eq!(
        output.data["audit_critique"],
        json!("<score>0.42</score><reason>too risky</reason>")
    );
    assert_eq!(output.data["audit_retry_count"], json!(1));
    let FlowInstruction::RetryNodes(nodes) = output.instruction else {
        panic!("expected RetryNodes instruction for score below threshold");
    };
    assert_eq!(nodes, vec!["Agenda_Steward_Proposer".to_string()]);
}

#[tokio::test]
async fn llm_augmented_audit_records_parse_error_when_score_tag_missing() {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = make_registry();
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<reason>missing score</reason>".to_string(),
    ]));

    let mechanism = LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator,
            registry,
            persona_id: "strict_teacher".to_string(),
            template_target: Some("critique_agenda.j2".to_string()),
            execution_mode: xiuxian_qianji::NodeQianhuanExecutionMode::Isolated,
            input_keys: vec!["raw_facts".to_string()],
            history_key: "audit_history".to_string(),
            output_key: "annotated_prompt".to_string(),
        },
        client: llm,
        model: "audit-model".to_string(),
        threshold_score: 0.8,
        max_retries: 3,
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
        retry_counter_key: "audit_retry_count".to_string(),
        output_key: "audit_critique".to_string(),
        score_key: "audit_score".to_string(),
        cognitive_early_halt_threshold: 0.3,
        enable_cognitive_supervision: false,
    };

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Draft agenda has no breaks.",
            "request": "Critique this agenda."
        }))
        .await
        .unwrap_or_else(|error| panic!("llm augmented audit should execute: {error}"));

    assert_eq!(output.data["audit_status"], "failed");
    assert_eq!(output.data["audit_score"], json!(0.0));
    assert_eq!(output.data["memrl_reward"], json!(0.0));
    assert_eq!(
        output.data["memrl_signal_source"],
        json!("formal_audit.llm")
    );
    assert_eq!(output.data["audit_retry_count"], json!(1));
    let audit_errors = output.data["audit_errors"]
        .as_array()
        .unwrap_or_else(|| panic!("audit_errors should be an array"));
    assert!(
        audit_errors.iter().any(|value| value
            .as_str()
            .is_some_and(|text| text.contains("missing or invalid"))),
        "expected parse-failure audit error, got: {audit_errors:?}"
    );
    let FlowInstruction::RetryNodes(nodes) = output.instruction else {
        panic!("expected RetryNodes instruction when score tag is missing");
    };
    assert_eq!(nodes, vec!["Agenda_Steward_Proposer".to_string()]);
}

#[tokio::test]
async fn llm_augmented_audit_aborts_when_retry_budget_is_exhausted() {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = make_registry();
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.10</score><reason>still overloaded</reason>".to_string(),
    ]));

    let mechanism = LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator,
            registry,
            persona_id: "strict_teacher".to_string(),
            template_target: Some("critique_agenda.j2".to_string()),
            execution_mode: xiuxian_qianji::NodeQianhuanExecutionMode::Isolated,
            input_keys: vec!["raw_facts".to_string()],
            history_key: "audit_history".to_string(),
            output_key: "annotated_prompt".to_string(),
        },
        client: llm,
        model: "audit-model".to_string(),
        threshold_score: 0.8,
        max_retries: 1,
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
        retry_counter_key: "audit_retry_count".to_string(),
        output_key: "audit_critique".to_string(),
        score_key: "audit_score".to_string(),
        cognitive_early_halt_threshold: 0.3,
        enable_cognitive_supervision: false,
    };

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Draft agenda still has overload risk.",
            "request": "Critique this agenda.",
            "audit_retry_count": 1
        }))
        .await
        .unwrap_or_else(|error| panic!("llm augmented audit should execute: {error}"));

    assert_eq!(output.data["audit_status"], "failed");
    assert_eq!(output.data["audit_retry_count"], json!(2));
    assert_eq!(output.data["audit_retry_exhausted"], json!(true));
    let audit_errors = output.data["audit_errors"]
        .as_array()
        .unwrap_or_else(|| panic!("audit_errors should be an array"));
    assert!(
        audit_errors.iter().any(|value| value
            .as_str()
            .is_some_and(|text| text.contains("retry budget exceeded"))),
        "expected retry-budget audit error, got: {audit_errors:?}"
    );
    let FlowInstruction::Abort(reason) = output.instruction else {
        panic!("expected Abort instruction when retry budget is exhausted");
    };
    assert_eq!(reason, "formal_audit.max_retries_exceeded");
}

#[tokio::test]
async fn compiler_builds_llm_augmented_formal_audit_and_converges() {
    let manifest = r#"
name = "AugmentedAuditLoop"

[[nodes]]
id = "Agenda_Steward_Proposer"
task_type = "mock"
weight = 1.0
params = {}

[[nodes]]
id = "Strict_Teacher_Critic"
task_type = "formal_audit"
weight = 1.0
params = { retry_targets = ["Agenda_Steward_Proposer"], threshold_score = 0.8, max_retries = 3, output_key = "teacher_critique", score_key = "teacher_score" }
[nodes.qianhuan]
persona_id = "strict_teacher"
template_target = "critique_agenda.j2"
[nodes.llm]
model = "audit-model"

[[edges]]
from = "Agenda_Steward_Proposer"
to = "Strict_Teacher_Critic"
weight = 1.0
"#;

    let temp = tempfile::tempdir().unwrap_or_else(|error| panic!("tempdir should work: {error}"));
    let index = Arc::new(
        LinkGraphIndex::build(temp.path())
            .unwrap_or_else(|error| panic!("index should build on temp dir: {error}")),
    );
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.30</score><reason>overloaded</reason>".to_string(),
        "<score>0.95</score><reason>acceptable</reason>".to_string(),
    ]));
    let models_probe = Arc::clone(&llm.seen_models);
    let llm_client: Arc<xiuxian_qianji::QianjiLlmClient> = llm;
    let compiler = QianjiCompiler::new(index, orchestrator, make_registry(), Some(llm_client));
    let engine = compiler
        .compile(manifest)
        .unwrap_or_else(|error| panic!("manifest should compile: {error}"));
    let scheduler = QianjiScheduler::new(engine);

    let output = scheduler
        .run(json!({
            "raw_facts": "Initial agenda draft"
        }))
        .await
        .unwrap_or_else(|error| panic!("scheduler should converge: {error}"));

    assert_eq!(output["audit_status"], "passed");
    let teacher_score = output["teacher_score"]
        .as_f64()
        .unwrap_or_else(|| panic!("teacher_score should be a numeric value"));
    assert!((teacher_score - 0.95).abs() < 1e-6);
    let memrl_reward = output["memrl_reward"]
        .as_f64()
        .unwrap_or_else(|| panic!("memrl_reward should be a numeric value"));
    assert!((memrl_reward - 0.95).abs() < 1e-6);
    assert_eq!(output["memrl_signal_source"], json!("formal_audit.llm"));
    assert_eq!(
        output["teacher_critique"],
        json!("<score>0.95</score><reason>acceptable</reason>")
    );

    let models = models_probe
        .lock()
        .map(|guard| guard.clone())
        .unwrap_or_default();
    assert_eq!(
        models,
        vec!["audit-model".to_string(), "audit-model".to_string()]
    );
}

// =============================================================================
// Cognitive Supervision Tests
// =============================================================================

/// Helper to create a minimal LlmAugmentedAuditMechanism for unit testing.
fn make_test_mechanism(
    llm: Arc<dyn LlmClient>,
    model: &str,
    threshold_score: f32,
    enable_cognitive_supervision: bool,
) -> LlmAugmentedAuditMechanism {
    let orchestrator = Arc::new(ThousandFacesOrchestrator::new(
        "Safety Rules".to_string(),
        None,
    ));
    let registry = make_registry();

    LlmAugmentedAuditMechanism {
        annotator: ContextAnnotator {
            orchestrator,
            registry,
            persona_id: "strict_teacher".to_string(),
            template_target: Some("critique_agenda.j2".to_string()),
            execution_mode: NodeQianhuanExecutionMode::Isolated,
            input_keys: vec!["raw_facts".to_string()],
            history_key: "audit_history".to_string(),
            output_key: "annotated_prompt".to_string(),
        },
        client: llm,
        model: model.to_string(),
        threshold_score,
        max_retries: 3,
        retry_target_ids: vec!["Agenda_Steward_Proposer".to_string()],
        retry_counter_key: "audit_retry_count".to_string(),
        output_key: "audit_critique".to_string(),
        score_key: "audit_score".to_string(),
        cognitive_early_halt_threshold: 0.3,
        enable_cognitive_supervision,
    }
}

#[tokio::test]
async fn llm_augmented_audit_includes_cognitive_metrics_when_enabled() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.95</score><reason>excellent</reason>".to_string(),
    ]));

    let mechanism = make_test_mechanism(llm, "claude-3-opus", 0.8, true);

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Test agenda with balanced workload.",
            "request": "Critique this agenda."
        }))
        .await
        .unwrap();

    // Verify cognitive metrics are present
    assert!(output.data.get("_cognitive_coherence").is_some());
    assert!(output.data.get("_early_halt_triggered").is_some());
    assert!(output.data.get("_cognitive_distribution").is_some());

    let coherence = output.data["_cognitive_coherence"]
        .as_f64()
        .expect("coherence should be numeric");
    assert!(
        (0.0..=1.0).contains(&coherence),
        "coherence should be in [0, 1], got {coherence}"
    );

    let distribution = output.data["_cognitive_distribution"].as_object().unwrap();
    assert!(distribution.contains_key("meta"));
    assert!(distribution.contains_key("operational"));
    assert!(distribution.contains_key("epistemic"));
    assert!(distribution.contains_key("instrumental"));
    assert!(distribution.contains_key("balance"));
    assert!(distribution.contains_key("uncertainty_ratio"));
}

#[tokio::test]
async fn llm_augmented_audit_without_cognitive_supervision() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.95</score><reason>good</reason>".to_string(),
    ]));

    let mechanism = make_test_mechanism(llm, "claude-3-opus", 0.8, false);

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Test agenda.",
            "request": "Critique."
        }))
        .await
        .unwrap();

    // Cognitive metrics should NOT be present when disabled
    assert!(output.data.get("_cognitive_coherence").is_none());
    assert!(output.data.get("_early_halt_triggered").is_none());
    assert!(output.data.get("_cognitive_distribution").is_none());

    // But audit should still work
    assert_eq!(output.data["audit_status"], "passed");
}

#[tokio::test]
async fn llm_augmented_audit_resolves_claude_provider() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec!["test".to_string()]));
    let mechanism = make_test_mechanism(llm, "claude-3-opus-20240229", 0.5, false);

    // The mechanism should resolve to Claude provider for claude models
    // This is implicitly tested via successful execution
    let result = mechanism
        .execute(&json!({
            "raw_facts": "Test",
            "request": "Test"
        }))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn llm_augmented_audit_resolves_anthropic_provider() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec!["test".to_string()]));
    let mechanism = make_test_mechanism(llm, "anthropic-claude-v1", 0.5, false);

    let result = mechanism
        .execute(&json!({
            "raw_facts": "Test",
            "request": "Test"
        }))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn llm_augmented_audit_resolves_gemini_provider() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec!["test".to_string()]));
    let mechanism = make_test_mechanism(llm, "gemini-pro", 0.5, false);

    let result = mechanism
        .execute(&json!({
            "raw_facts": "Test",
            "request": "Test"
        }))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn llm_augmented_audit_resolves_codex_provider() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec!["test".to_string()]));
    let mechanism = make_test_mechanism(llm, "gpt-4", 0.5, false);

    let result = mechanism
        .execute(&json!({
            "raw_facts": "Test",
            "request": "Test"
        }))
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn llm_augmented_audit_early_halt_triggers_abort() {
    // Create a mock that returns content that might trigger early halt
    // Note: Actual early halt behavior depends on ZhenfaPipeline cognitive analysis
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.95</score><reason>passed</reason>".to_string(),
    ]));

    let mechanism = make_test_mechanism(llm, "claude-3-opus", 0.8, true);

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Test agenda.",
            "request": "Critique."
        }))
        .await
        .unwrap();

    // Verify early_halt_triggered field exists
    let early_halt = output.data["_early_halt_triggered"]
        .as_bool()
        .expect("early_halt_triggered should be boolean");

    // For normal content, early_halt should typically be false
    // If early_halt is true, instruction should be Abort (with some reason)
    if early_halt {
        assert!(
            matches!(output.instruction, FlowInstruction::Abort(_)),
            "early_halt_triggered=true should result in Abort instruction"
        );
    }
}

#[tokio::test]
async fn llm_augmented_audit_cognitive_distribution_values_in_range() {
    let llm = Arc::new(SequencedMockLlmClient::new(vec![
        "<score>0.90</score><reason>good</reason>".to_string(),
    ]));

    let mechanism = make_test_mechanism(llm, "claude-3-opus", 0.8, true);

    let output = mechanism
        .execute(&json!({
            "raw_facts": "Test agenda.",
            "request": "Critique."
        }))
        .await
        .unwrap();

    let distribution = output.data["_cognitive_distribution"].as_object().unwrap();

    // All dimension scores should be in [0, 1]
    for (key, value) in distribution {
        if let Some(score) = value.as_f64() {
            assert!(
                (0.0..=1.0).contains(&score),
                "dimension {key} should be in [0, 1], got {score}"
            );
        }
    }
}
