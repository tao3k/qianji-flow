//! Thought Aggregator for Sovereign Memory (Blueprint V6.1).
//!
//! Captures streaming events during workflow execution and aggregates
//! them into `CognitiveTrace` artifacts for persistent storage in Wendao.

use std::sync::Arc;
use xiuxian_wendao::link_graph::CognitiveTraceRecord;
use xiuxian_zhenfa::ZhenfaStreamingEvent;

/// Aggregates streaming events into a cognitive trace artifact.
///
/// This struct captures the reasoning flow during workflow execution,
/// enabling "historical sovereignty" - the ability to query the knowledge
/// graph for the reasoning chain that led to any decision or commit.
#[derive(Debug, Clone)]
pub struct ThoughtAggregator {
    /// Unique identifier for this trace.
    trace_id: String,
    /// Session identifier from Qianji execution.
    session_id: Option<String>,
    /// Node identifier from the compiled flow graph.
    node_id: String,
    /// The original user intent/prompt.
    intent: String,
    /// Aggregated reasoning content.
    reasoning_chunks: Vec<String>,
    /// Tool calls made during execution.
    tool_calls: Vec<ToolCallRecord>,
    /// Final outcome or conclusion.
    outcome: Option<String>,
    /// Cognitive coherence score during execution.
    coherence_score: Option<f32>,
    /// Whether early halt was triggered.
    early_halt_triggered: bool,
    /// Timestamp when aggregation started.
    start_timestamp_ms: u64,
}

/// Record of a tool call during execution.
#[derive(Debug, Clone)]
pub struct ToolCallRecord {
    /// Tool call identifier.
    pub id: String,
    /// Tool name.
    pub name: String,
    /// Input parameters.
    pub input: serde_json::Value,
    /// Output result (if available).
    pub output: Option<serde_json::Value>,
}

impl ThoughtAggregator {
    /// Create a new thought aggregator for a workflow node.
    #[must_use]
    pub fn new(session_id: Option<String>, node_id: String, intent: String) -> Self {
        let trace_id = format!("trace-{}-{}", node_id, unix_timestamp_ms());
        Self {
            trace_id,
            session_id,
            node_id,
            intent,
            reasoning_chunks: Vec::new(),
            tool_calls: Vec::new(),
            outcome: None,
            coherence_score: None,
            early_halt_triggered: false,
            start_timestamp_ms: unix_timestamp_ms(),
        }
    }

    /// Process a streaming event and aggregate into the trace.
    pub fn process_event(&mut self, event: &ZhenfaStreamingEvent) {
        match event {
            ZhenfaStreamingEvent::Thought(text) => {
                self.reasoning_chunks.push(format!("[THOUGHT] {}", text));
            }
            ZhenfaStreamingEvent::TextDelta(text) => {
                self.reasoning_chunks.push(text.to_string());
            }
            ZhenfaStreamingEvent::ToolCall { id, name, input } => {
                self.tool_calls.push(ToolCallRecord {
                    id: id.to_string(),
                    name: name.to_string(),
                    input: input.clone(),
                    output: None,
                });
            }
            ZhenfaStreamingEvent::ToolResult { id, output } => {
                if let Some(tool_call) = self
                    .tool_calls
                    .iter_mut()
                    .rev()
                    .find(|tc| tc.id.as_str() == id.as_ref())
                {
                    tool_call.output = Some(output.clone());
                }
            }
            ZhenfaStreamingEvent::Status(text) => {
                self.reasoning_chunks.push(format!("[STATUS] {}", text));
            }
            ZhenfaStreamingEvent::Progress { message, percent } => {
                self.reasoning_chunks
                    .push(format!("[PROGRESS {}%] {}", percent, message));
            }
            ZhenfaStreamingEvent::Finished(outcome) => {
                self.outcome = Some(outcome.final_text.as_ref().to_string());
            }
            ZhenfaStreamingEvent::Error { code, message } => {
                self.reasoning_chunks
                    .push(format!("[ERROR {}] {}", code, message));
            }
        }
    }

    /// Set the cognitive coherence score.
    pub fn set_coherence_score(&mut self, score: f32) {
        self.coherence_score = Some(score);
    }

    /// Mark that early halt was triggered.
    pub fn set_early_halt(&mut self) {
        self.early_halt_triggered = true;
    }

    /// Set the final outcome.
    pub fn set_outcome(&mut self, outcome: String) {
        self.outcome = Some(outcome);
    }

    /// Build the final cognitive trace record.
    #[must_use]
    pub fn build(self) -> CognitiveTraceRecord {
        let reasoning = self.reasoning_chunks.join("\n");
        let reasoning_arc: Arc<str> = Arc::<str>::from(reasoning);
        let outcome_arc = self.outcome.map(|o| Arc::<str>::from(o));

        CognitiveTraceRecord {
            trace_id: self.trace_id,
            session_id: self.session_id,
            node_id: self.node_id,
            intent: self.intent,
            reasoning: reasoning_arc,
            outcome: outcome_arc,
            commit_sha: None,
            timestamp_ms: self.start_timestamp_ms,
            coherence_score: self.coherence_score,
            early_halt_triggered: self.early_halt_triggered,
        }
    }

    /// Get the current reasoning length (for budget tracking).
    #[must_use]
    pub fn reasoning_length(&self) -> usize {
        self.reasoning_chunks.iter().map(|c| c.len()).sum()
    }

    /// Check if the aggregator has captured any content.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reasoning_chunks.is_empty() && self.tool_calls.is_empty()
    }
}

/// Get current timestamp in milliseconds.
fn unix_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn thought_aggregator_creates_trace_with_intent() {
        let aggregator = ThoughtAggregator::new(
            Some("session-123".to_string()),
            "AuditNode".to_string(),
            "Critique the agenda".to_string(),
        );

        let trace = aggregator.build();
        assert!(trace.trace_id.starts_with("trace-AuditNode-"));
        assert_eq!(trace.session_id, Some("session-123".to_string()));
        assert_eq!(trace.node_id, "AuditNode");
        assert_eq!(trace.intent, "Critique the agenda");
    }

    #[test]
    fn thought_aggregator_processes_thought_events() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        aggregator.process_event(&ZhenfaStreamingEvent::Thought(Arc::<str>::from(
            "Thinking...",
        )));
        aggregator.process_event(&ZhenfaStreamingEvent::TextDelta(Arc::<str>::from(
            "Output text",
        )));

        let trace = aggregator.build();
        assert!(trace.reasoning.contains("[THOUGHT] Thinking..."));
        assert!(trace.reasoning.contains("Output text"));
    }

    #[test]
    fn thought_aggregator_processes_tool_calls() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        aggregator.process_event(&ZhenfaStreamingEvent::ToolCall {
            id: Arc::<str>::from("call-1"),
            name: Arc::<str>::from("search"),
            input: json!({"query": "test"}),
        });
        aggregator.process_event(&ZhenfaStreamingEvent::ToolResult {
            id: Arc::<str>::from("call-1"),
            output: json!({"results": []}),
        });

        assert_eq!(aggregator.tool_calls.len(), 1);
        assert_eq!(aggregator.tool_calls[0].name, "search");
        assert!(aggregator.tool_calls[0].output.is_some());
    }

    #[test]
    fn thought_aggregator_records_outcome() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        aggregator.set_outcome("Task completed successfully".to_string());

        let trace = aggregator.build();
        assert_eq!(
            trace.outcome,
            Some(Arc::<str>::from("Task completed successfully"))
        );
    }

    #[test]
    fn thought_aggregator_tracks_coherence() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        aggregator.set_coherence_score(0.85);
        aggregator.set_early_halt();

        let trace = aggregator.build();
        assert_eq!(trace.coherence_score, Some(0.85));
        assert!(trace.early_halt_triggered);
    }

    #[test]
    fn thought_aggregator_reasoning_length() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        assert!(aggregator.is_empty());
        assert_eq!(aggregator.reasoning_length(), 0);

        aggregator.process_event(&ZhenfaStreamingEvent::TextDelta(Arc::<str>::from("Hello")));

        assert!(!aggregator.is_empty());
        assert_eq!(aggregator.reasoning_length(), 5);
    }

    #[test]
    fn thought_aggregator_processes_status_events() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        aggregator.process_event(&ZhenfaStreamingEvent::Status(Arc::<str>::from(
            "Scanning files...",
        )));

        let trace = aggregator.build();
        assert!(trace.reasoning.contains("[STATUS] Scanning files..."));
    }

    #[test]
    fn thought_aggregator_processes_progress_events() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        aggregator.process_event(&ZhenfaStreamingEvent::Progress {
            message: Arc::<str>::from("Processing"),
            percent: 50,
        });

        let trace = aggregator.build();
        assert!(trace.reasoning.contains("[PROGRESS 50%] Processing"));
    }

    #[test]
    fn thought_aggregator_processes_error_events() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        aggregator.process_event(&ZhenfaStreamingEvent::Error {
            code: Arc::<str>::from("E001"),
            message: Arc::<str>::from("Something went wrong"),
        });

        let trace = aggregator.build();
        assert!(trace.reasoning.contains("[ERROR E001] Something went wrong"));
    }

    #[test]
    fn thought_aggregator_processes_finished_event() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        let outcome = xiuxian_zhenfa::StreamingOutcome {
            success: true,
            tokens_used: Some(xiuxian_zhenfa::TokenUsage {
                input: 50,
                output: 50,
                total: 100,
            }),
            final_text: Arc::<str>::from("Final result text"),
            tool_calls: Vec::new(),
            exit_code: None,
        };
        aggregator.process_event(&ZhenfaStreamingEvent::Finished(outcome));

        let trace = aggregator.build();
        assert_eq!(trace.outcome, Some(Arc::<str>::from("Final result text")));
    }

    #[test]
    fn thought_aggregator_multiple_tool_calls_match_results() {
        let mut aggregator =
            ThoughtAggregator::new(None, "TestNode".to_string(), "Test intent".to_string());

        // First tool call
        aggregator.process_event(&ZhenfaStreamingEvent::ToolCall {
            id: Arc::<str>::from("call-1"),
            name: Arc::<str>::from("search"),
            input: json!({"query": "test1"}),
        });
        // Second tool call
        aggregator.process_event(&ZhenfaStreamingEvent::ToolCall {
            id: Arc::<str>::from("call-2"),
            name: Arc::<str>::from("read"),
            input: json!({"path": "test.md"}),
        });
        // Results in reverse order
        aggregator.process_event(&ZhenfaStreamingEvent::ToolResult {
            id: Arc::<str>::from("call-2"),
            output: json!({"content": "file content"}),
        });
        aggregator.process_event(&ZhenfaStreamingEvent::ToolResult {
            id: Arc::<str>::from("call-1"),
            output: json!({"results": ["a", "b"]}),
        });

        assert_eq!(aggregator.tool_calls.len(), 2);
        assert_eq!(aggregator.tool_calls[0].name, "search");
        assert_eq!(aggregator.tool_calls[1].name, "read");
        assert!(aggregator.tool_calls[0].output.is_some());
        assert!(aggregator.tool_calls[1].output.is_some());
    }

    #[test]
    fn thought_aggregator_builds_complete_trace() {
        let mut aggregator = ThoughtAggregator::new(
            Some("session-complete".to_string()),
            "CompleteNode".to_string(),
            "Complete workflow".to_string(),
        );

        aggregator.process_event(&ZhenfaStreamingEvent::Thought(Arc::<str>::from(
            "Planning...",
        )));
        aggregator.process_event(&ZhenfaStreamingEvent::TextDelta(Arc::<str>::from(
            "Step 1",
        )));
        aggregator.process_event(&ZhenfaStreamingEvent::ToolCall {
            id: Arc::<str>::from("call-1"),
            name: Arc::<str>::from("execute"),
            input: json!({"cmd": "test"}),
        });
        aggregator.process_event(&ZhenfaStreamingEvent::ToolResult {
            id: Arc::<str>::from("call-1"),
            output: json!({"success": true}),
        });
        aggregator.set_coherence_score(0.92);
        aggregator.set_outcome("Workflow completed".to_string());

        let trace = aggregator.build();

        assert!(trace.trace_id.starts_with("trace-CompleteNode-"));
        assert_eq!(trace.session_id, Some("session-complete".to_string()));
        assert_eq!(trace.node_id, "CompleteNode");
        assert_eq!(trace.intent, "Complete workflow");
        assert!(trace.reasoning.contains("[THOUGHT] Planning..."));
        assert!(trace.reasoning.contains("Step 1"));
        assert_eq!(trace.outcome, Some(Arc::<str>::from("Workflow completed")));
        assert_eq!(trace.coherence_score, Some(0.92));
        assert!(!trace.early_halt_triggered);
    }

    #[test]
    fn thought_aggregator_empty_trace_still_builds() {
        let aggregator =
            ThoughtAggregator::new(None, "EmptyNode".to_string(), "Empty intent".to_string());

        let trace = aggregator.build();

        assert!(trace.trace_id.starts_with("trace-EmptyNode-"));
        assert!(trace.reasoning.is_empty());
        assert!(trace.outcome.is_none());
    }

    #[test]
    fn thought_aggregator_clone_preserves_state() {
        let mut aggregator =
            ThoughtAggregator::new(None, "CloneNode".to_string(), "Clone intent".to_string());

        aggregator.process_event(&ZhenfaStreamingEvent::TextDelta(Arc::<str>::from(
            "Content",
        )));
        aggregator.set_coherence_score(0.75);

        let cloned = aggregator.clone();

        assert_eq!(cloned.node_id, "CloneNode");
        assert_eq!(cloned.intent, "Clone intent");
        assert_eq!(cloned.reasoning_length(), 7);
        assert_eq!(cloned.coherence_score, Some(0.75));
    }
}
