//! Artifact Observer for Sovereign Memory (Blueprint V6.1).
//!
//! Observes workflow completion events and triggers final ingestion of
//! `CognitiveTrace` records into Wendao for persistent historical sovereignty.
//!
//! ## Architecture
//!
//! ```text
//! NodeTransition (Exiting) ──► ArtifactObserver ──► Wendao Ingestion
//!         │                         │                    │
//!         │                         ▼                    ▼
//!         └───���────────────► ThoughtAggregator    CognitiveTraceRecord
//!                                   .build()        persisted to LinkGraph
//! ```

use crate::telemetry::{NodeTransitionPhase, SwarmEvent};
use async_trait::async_trait;
use std::sync::Arc;
use xiuxian_wendao::link_graph::{CognitiveTraceRecord, LinkGraphSemanticDocument};

/// Result of artifact observation and ingestion.
#[derive(Debug, Clone, PartialEq)]
pub enum ArtifactIngestionResult {
    /// Artifact was successfully ingested into Wendao.
    Ingested {
        /// The trace ID that was ingested.
        trace_id: String,
        /// The anchor ID in Wendao.
        anchor_id: String,
    },
    /// No artifact was available to ingest.
    NoArtifact,
    /// Ingestion was skipped due to configuration.
    Skipped {
        /// Reason for skipping.
        reason: Arc<str>,
    },
    /// Ingestion failed.
    Failed {
        /// Error message.
        error: Arc<str>,
    },
}

/// Configuration for the artifact observer.
#[derive(Debug, Clone)]
pub struct ArtifactObserverConfig {
    /// Whether ingestion is enabled.
    pub enabled: bool,
    /// Base path for cognitive trace documents in Wendao.
    pub trace_base_path: String,
    /// Whether to ingest on node exit (workflow completion).
    pub ingest_on_exit: bool,
    /// Whether to ingest on early halt.
    pub ingest_on_early_halt: bool,
}

impl Default for ArtifactObserverConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            trace_base_path: ".cognitive/traces".to_string(),
            ingest_on_exit: true,
            ingest_on_early_halt: true,
        }
    }
}

/// Sink trait for Wendao ingestion of cognitive traces.
#[async_trait]
pub trait WendaoIngestionSink: Send + Sync + std::fmt::Debug {
    /// Ingest a cognitive trace into Wendao.
    ///
    /// # Errors
    ///
    /// Returns an error string if ingestion fails.
    async fn ingest_trace(
        &self,
        trace: &CognitiveTraceRecord,
        document: &LinkGraphSemanticDocument,
    ) -> Result<String, String>;
}

/// No-op sink used when Wendao ingestion is disabled.
#[derive(Debug, Default)]
pub struct NoopWendaoIngestionSink;

#[async_trait]
impl WendaoIngestionSink for NoopWendaoIngestionSink {
    async fn ingest_trace(
        &self,
        trace: &CognitiveTraceRecord,
        _document: &LinkGraphSemanticDocument,
    ) -> Result<String, String> {
        Ok(format!("noop:{}", trace.trace_id))
    }
}

/// Observer for workflow artifacts that triggers Wendao ingestion.
///
/// This observer listens for workflow completion events and triggers
/// the ingestion of cognitive traces into Wendao for persistent storage.
#[derive(Debug)]
pub struct ArtifactObserver<S: WendaoIngestionSink = NoopWendaoIngestionSink> {
    /// Configuration for the observer.
    config: ArtifactObserverConfig,
    /// Sink for Wendao ingestion.
    sink: S,
}

impl Default for ArtifactObserver<NoopWendaoIngestionSink> {
    fn default() -> Self {
        Self::new(ArtifactObserverConfig::default(), NoopWendaoIngestionSink)
    }
}

impl<S: WendaoIngestionSink> ArtifactObserver<S> {
    /// Create a new artifact observer with the given configuration and sink.
    #[must_use]
    pub fn new(config: ArtifactObserverConfig, sink: S) -> Self {
        Self { config, sink }
    }

    /// Check if this observer should handle the given swarm event.
    #[must_use]
    pub fn should_handle_event(&self, event: &SwarmEvent) -> bool {
        if !self.config.enabled {
            return false;
        }

        match event {
            SwarmEvent::NodeTransition { phase, .. } => match phase {
                NodeTransitionPhase::Exiting => self.config.ingest_on_exit,
                NodeTransitionPhase::Failed => self.config.ingest_on_exit,
                NodeTransitionPhase::Entering => false,
            },
            _ => false,
        }
    }

    /// Ingest a cognitive trace into Wendao.
    ///
    /// This method converts the trace to a semantic document and ingests it.
    pub async fn ingest_artifact(
        &self,
        trace: &CognitiveTraceRecord,
    ) -> ArtifactIngestionResult {
        if !self.config.enabled {
            return ArtifactIngestionResult::Skipped {
                reason: Arc::from("ingestion disabled"),
            };
        }

        // Check early halt policy
        if trace.early_halt_triggered && !self.config.ingest_on_early_halt {
            return ArtifactIngestionResult::Skipped {
                reason: Arc::from("early halt ingestion disabled"),
            };
        }

        // Build the semantic document for Wendao
        let doc_id = format!("trace:{}", trace.trace_id);
        let path = format!(
            "{}/{}.md",
            self.config.trace_base_path,
            trace.trace_id.replace(':', "-")
        );
        let document = trace.to_semantic_document(&doc_id, &path);

        // Ingest into Wendao
        match self.sink.ingest_trace(trace, &document).await {
            Ok(anchor_id) => ArtifactIngestionResult::Ingested {
                trace_id: trace.trace_id.clone(),
                anchor_id,
            },
            Err(error) => ArtifactIngestionResult::Failed {
                error: Arc::from(error),
            },
        }
    }

    /// Get the observer configuration.
    #[must_use]
    pub const fn config(&self) -> &ArtifactObserverConfig {
        &self.config
    }
}

/// Builder for creating configured artifact observers.
#[derive(Debug, Default)]
pub struct ArtifactObserverBuilder<S: WendaoIngestionSink = NoopWendaoIngestionSink> {
    config: ArtifactObserverConfig,
    sink: Option<S>,
}

impl<S: WendaoIngestionSink> ArtifactObserverBuilder<S> {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ArtifactObserverConfig::default(),
            sink: None,
        }
    }

    /// Set whether ingestion is enabled.
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set the base path for cognitive trace documents.
    #[must_use]
    pub fn trace_base_path(mut self, path: impl Into<String>) -> Self {
        self.config.trace_base_path = path.into();
        self
    }

    /// Set whether to ingest on node exit.
    #[must_use]
    pub fn ingest_on_exit(mut self, ingest: bool) -> Self {
        self.config.ingest_on_exit = ingest;
        self
    }

    /// Set whether to ingest on early halt.
    #[must_use]
    pub fn ingest_on_early_halt(mut self, ingest: bool) -> Self {
        self.config.ingest_on_early_halt = ingest;
        self
    }

    /// Set the Wendao ingestion sink.
    #[must_use]
    pub fn sink(mut self, sink: S) -> Self {
        self.sink = Some(sink);
        self
    }

    /// Build the artifact observer.
    ///
    /// # Panics
    ///
    /// Panics if no sink was provided.
    #[must_use]
    pub fn build(self) -> ArtifactObserver<S> {
        let sink = self.sink.expect("sink must be provided");
        ArtifactObserver::new(self.config, sink)
    }
}

impl ArtifactObserverBuilder<NoopWendaoIngestionSink> {
    /// Build with the no-op sink.
    #[must_use]
    pub fn build_noop(self) -> ArtifactObserver<NoopWendaoIngestionSink> {
        ArtifactObserver::new(self.config, NoopWendaoIngestionSink)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // === ArtifactObserverConfig Tests ===

    #[test]
    fn config_default_values() {
        let config = ArtifactObserverConfig::default();
        assert!(config.enabled);
        assert_eq!(config.trace_base_path, ".cognitive/traces");
        assert!(config.ingest_on_exit);
        assert!(config.ingest_on_early_halt);
    }

    #[test]
    fn config_clone_preserves_values() {
        let config = ArtifactObserverConfig {
            enabled: false,
            trace_base_path: "custom/path".to_string(),
            ingest_on_exit: false,
            ingest_on_early_halt: true,
        };
        let cloned = config.clone();
        assert!(!cloned.enabled);
        assert_eq!(cloned.trace_base_path, "custom/path");
        assert!(!cloned.ingest_on_exit);
        assert!(cloned.ingest_on_early_halt);
    }

    // === ArtifactIngestionResult Tests ===

    #[test]
    fn ingestion_result_ingested() {
        let result = ArtifactIngestionResult::Ingested {
            trace_id: "trace-123".to_string(),
            anchor_id: "anchor-456".to_string(),
        };
        match result {
            ArtifactIngestionResult::Ingested { trace_id, anchor_id } => {
                assert_eq!(trace_id, "trace-123");
                assert_eq!(anchor_id, "anchor-456");
            }
            _ => panic!("expected Ingested variant"),
        }
    }

    #[test]
    fn ingestion_result_no_artifact() {
        let result = ArtifactIngestionResult::NoArtifact;
        assert!(matches!(result, ArtifactIngestionResult::NoArtifact));
    }

    #[test]
    fn ingestion_result_skipped() {
        let result = ArtifactIngestionResult::Skipped {
            reason: Arc::from("test skip"),
        };
        match result {
            ArtifactIngestionResult::Skipped { reason } => {
                assert_eq!(reason.as_ref(), "test skip");
            }
            _ => panic!("expected Skipped variant"),
        }
    }

    #[test]
    fn ingestion_result_failed() {
        let result = ArtifactIngestionResult::Failed {
            error: Arc::from("test error"),
        };
        match result {
            ArtifactIngestionResult::Failed { error } => {
                assert_eq!(error.as_ref(), "test error");
            }
            _ => panic!("expected Failed variant"),
        }
    }

    #[test]
    fn ingestion_result_clone() {
        let result = ArtifactIngestionResult::Ingested {
            trace_id: "trace-789".to_string(),
            anchor_id: "anchor-012".to_string(),
        };
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }

    #[test]
    fn ingestion_result_partial_eq() {
        let result1 = ArtifactIngestionResult::Ingested {
            trace_id: "trace-1".to_string(),
            anchor_id: "anchor-1".to_string(),
        };
        let result2 = ArtifactIngestionResult::Ingested {
            trace_id: "trace-1".to_string(),
            anchor_id: "anchor-1".to_string(),
        };
        let result3 = ArtifactIngestionResult::Ingested {
            trace_id: "trace-2".to_string(),
            anchor_id: "anchor-1".to_string(),
        };
        assert_eq!(result1, result2);
        assert_ne!(result1, result3);
    }

    // === NoopWendaoIngestionSink Tests ===

    #[tokio::test]
    async fn noop_sink_returns_ok() {
        let sink = NoopWendaoIngestionSink::default();
        let trace = CognitiveTraceRecord::new(
            "trace-test".to_string(),
            None,
            "TestNode".to_string(),
            "Test intent".to_string(),
        );
        let doc = trace.to_semantic_document("doc-1", "path.md");
        let result = sink.ingest_trace(&trace, &doc).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "noop:trace-test");
    }

    // === ArtifactObserver Tests ===

    #[test]
    fn observer_default_creation() {
        let observer = ArtifactObserver::default();
        assert!(observer.config().enabled);
    }

    #[test]
    fn observer_should_handle_exit_event() {
        let observer = ArtifactObserver::default();
        let event = SwarmEvent::NodeTransition {
            session_id: Some("session-1".to_string()),
            agent_id: None,
            role_class: None,
            node_id: "TestNode".to_string(),
            phase: NodeTransitionPhase::Exiting,
            timestamp_ms: 1700000000000,
        };
        assert!(observer.should_handle_event(&event));
    }

    #[test]
    fn observer_should_handle_failed_event() {
        let observer = ArtifactObserver::default();
        let event = SwarmEvent::NodeTransition {
            session_id: Some("session-1".to_string()),
            agent_id: None,
            role_class: None,
            node_id: "TestNode".to_string(),
            phase: NodeTransitionPhase::Failed,
            timestamp_ms: 1700000000000,
        };
        assert!(observer.should_handle_event(&event));
    }

    #[test]
    fn observer_should_not_handle_entering_event() {
        let observer = ArtifactObserver::default();
        let event = SwarmEvent::NodeTransition {
            session_id: Some("session-1".to_string()),
            agent_id: None,
            role_class: None,
            node_id: "TestNode".to_string(),
            phase: NodeTransitionPhase::Entering,
            timestamp_ms: 1700000000000,
        };
        assert!(!observer.should_handle_event(&event));
    }

    #[test]
    fn observer_disabled_ignores_events() {
        let config = ArtifactObserverConfig {
            enabled: false,
            ..Default::default()
        };
        let observer = ArtifactObserver::new(config, NoopWendaoIngestionSink::default());
        let event = SwarmEvent::NodeTransition {
            session_id: None,
            agent_id: None,
            role_class: None,
            node_id: "TestNode".to_string(),
            phase: NodeTransitionPhase::Exiting,
            timestamp_ms: 0,
        };
        assert!(!observer.should_handle_event(&event));
    }

    #[test]
    fn observer_ignores_non_transition_events() {
        let observer = ArtifactObserver::default();
        let event = SwarmEvent::SwarmHeartbeat {
            session_id: None,
            cluster_id: None,
            agent_id: None,
            role_class: None,
            cpu_percent: None,
            memory_bytes: None,
            timestamp_ms: 0,
        };
        assert!(!observer.should_handle_event(&event));
    }

    #[tokio::test]
    async fn observer_ingest_artifact_success() {
        let observer = ArtifactObserver::default();
        let trace = CognitiveTraceRecord::new(
            "trace-ingest-1".to_string(),
            Some("session-1".to_string()),
            "AuditNode".to_string(),
            "Critique the agenda".to_string(),
        );
        let result = observer.ingest_artifact(&trace).await;
        match result {
            ArtifactIngestionResult::Ingested { trace_id, anchor_id } => {
                assert_eq!(trace_id, "trace-ingest-1");
                assert_eq!(anchor_id, "noop:trace-ingest-1");
            }
            _ => panic!("expected Ingested variant, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn observer_ingest_disabled_returns_skipped() {
        let config = ArtifactObserverConfig {
            enabled: false,
            ..Default::default()
        };
        let observer = ArtifactObserver::new(config, NoopWendaoIngestionSink::default());
        let trace = CognitiveTraceRecord::new(
            "trace-disabled".to_string(),
            None,
            "TestNode".to_string(),
            "Test".to_string(),
        );
        let result = observer.ingest_artifact(&trace).await;
        match result {
            ArtifactIngestionResult::Skipped { reason } => {
                assert_eq!(reason.as_ref(), "ingestion disabled");
            }
            _ => panic!("expected Skipped variant"),
        }
    }

    #[tokio::test]
    async fn observer_ingest_early_halt_skipped_when_disabled() {
        let config = ArtifactObserverConfig {
            ingest_on_early_halt: false,
            ..Default::default()
        };
        let observer = ArtifactObserver::new(config, NoopWendaoIngestionSink::default());
        let mut trace = CognitiveTraceRecord::new(
            "trace-halt".to_string(),
            None,
            "MonitorNode".to_string(),
            "Monitor".to_string(),
        );
        trace.early_halt_triggered = true;
        let result = observer.ingest_artifact(&trace).await;
        match result {
            ArtifactIngestionResult::Skipped { reason } => {
                assert_eq!(reason.as_ref(), "early halt ingestion disabled");
            }
            _ => panic!("expected Skipped variant"),
        }
    }

    #[tokio::test]
    async fn observer_ingest_early_halt_allowed_when_enabled() {
        let observer = ArtifactObserver::default();
        let mut trace = CognitiveTraceRecord::new(
            "trace-halt-enabled".to_string(),
            None,
            "MonitorNode".to_string(),
            "Monitor".to_string(),
        );
        trace.early_halt_triggered = true;
        let result = observer.ingest_artifact(&trace).await;
        match result {
            ArtifactIngestionResult::Ingested { trace_id, .. } => {
                assert_eq!(trace_id, "trace-halt-enabled");
            }
            _ => panic!("expected Ingested variant"),
        }
    }

    // === ArtifactObserverBuilder Tests ===

    #[test]
    fn builder_creates_default_observer() {
        let observer = ArtifactObserverBuilder::new()
            .build_noop();
        assert!(observer.config().enabled);
    }

    #[test]
    fn builder_disabled() {
        let observer = ArtifactObserverBuilder::new()
            .enabled(false)
            .build_noop();
        assert!(!observer.config().enabled);
    }

    #[test]
    fn builder_custom_trace_path() {
        let observer = ArtifactObserverBuilder::new()
            .trace_base_path("custom/traces")
            .build_noop();
        assert_eq!(observer.config().trace_base_path, "custom/traces");
    }

    #[test]
    fn builder_ingest_on_exit_false() {
        let observer = ArtifactObserverBuilder::new()
            .ingest_on_exit(false)
            .build_noop();
        assert!(!observer.config().ingest_on_exit);
    }

    #[test]
    fn builder_ingest_on_early_halt_false() {
        let observer = ArtifactObserverBuilder::new()
            .ingest_on_early_halt(false)
            .build_noop();
        assert!(!observer.config().ingest_on_early_halt);
    }

    #[test]
    fn builder_chained_config() {
        let observer = ArtifactObserverBuilder::new()
            .enabled(false)
            .trace_base_path("my/path")
            .ingest_on_exit(false)
            .ingest_on_early_halt(false)
            .build_noop();
        let config = observer.config();
        assert!(!config.enabled);
        assert_eq!(config.trace_base_path, "my/path");
        assert!(!config.ingest_on_exit);
        assert!(!config.ingest_on_early_halt);
    }

    // === Mock Sink for Testing ===

    /// Mock sink that tracks ingestion calls.
    #[derive(Debug, Default)]
    struct MockIngestionSink {
        call_count: AtomicUsize,
        last_trace_id: std::sync::Mutex<Option<String>>,
    }

    #[async_trait]
    impl WendaoIngestionSink for MockIngestionSink {
        async fn ingest_trace(
            &self,
            trace: &CognitiveTraceRecord,
            _document: &LinkGraphSemanticDocument,
        ) -> Result<String, String> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            let mut last = self.last_trace_id.lock().unwrap();
            *last = Some(trace.trace_id.clone());
            Ok(format!("mock:{}", trace.trace_id))
        }
    }

    impl MockIngestionSink {
        fn call_count(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }

        fn last_trace_id(&self) -> Option<String> {
            self.last_trace_id.lock().unwrap().clone()
        }
    }

    #[tokio::test]
    async fn observer_with_mock_sink() {
        let sink = MockIngestionSink::default();
        let observer = ArtifactObserverBuilder::new()
            .sink(sink)
            .build();

        let trace = CognitiveTraceRecord::new(
            "trace-mock".to_string(),
            None,
            "TestNode".to_string(),
            "Test".to_string(),
        );

        let result = observer.ingest_artifact(&trace).await;
        match result {
            ArtifactIngestionResult::Ingested { trace_id, anchor_id } => {
                assert_eq!(trace_id, "trace-mock");
                assert_eq!(anchor_id, "mock:trace-mock");
            }
            _ => panic!("expected Ingested variant"),
        }
    }

    #[tokio::test]
    async fn observer_debug_format() {
        let observer = ArtifactObserver::default();
        let debug_str = format!("{:?}", observer);
        assert!(debug_str.contains("ArtifactObserver"));
    }

    #[test]
    fn observer_config_access() {
        let config = ArtifactObserverConfig {
            enabled: false,
            trace_base_path: "test/path".to_string(),
            ingest_on_exit: true,
            ingest_on_early_halt: false,
        };
        let observer = ArtifactObserver::new(config.clone(), NoopWendaoIngestionSink::default());
        let observed_config = observer.config();
        assert!(!observed_config.enabled);
        assert_eq!(observed_config.trace_base_path, "test/path");
        assert!(observed_config.ingest_on_exit);
        assert!(!observed_config.ingest_on_early_halt);
    }
}
