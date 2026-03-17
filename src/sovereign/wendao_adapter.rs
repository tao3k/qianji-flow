//! Wendao Adapter for Sovereign Memory (Blueprint V6.1).
//!
//! Provides adapter implementations for `WendaoIngestionSink` trait,
//! connecting Qianji's ArtifactObserver with Wendao's LinkGraphIndex
//! for persistent storage.

use async_trait::async_trait;
use std::sync::Arc;
use xiuxian_wendao::link_graph::{CognitiveTraceRecord, LinkGraphSemanticDocument};

use super::artifact_observer::WendaoIngestionSink;
use super::wendao_sink::{FileWendaoSink, InMemoryWendaoSink};

/// Adapter that bridges Qianji's ArtifactObserver with Wendao's LinkGraphIndex
/// for persistent storage.
///
/// This adapter provides a composite sink that writes cognitive traces to
/// files (for Wendao to index) with an in-memory fallback for testing.
#[derive(Debug)]
pub struct WendaoIndexAdapter {
    /// The file-based sink for persistent storage.
    file_sink: FileWendaoSink,
    /// The in-memory sink for testing/fallback.
    memory_sink: InMemoryWendaoSink,
}

impl WendaoIndexAdapter {
    /// Create a new adapter with default settings.
    ///
    /// Uses `.cognitive/traces` as the base directory for file storage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            file_sink: FileWendaoSink::default(),
            memory_sink: InMemoryWendaoSink::new(),
        }
    }

    /// Create a new adapter with custom file sink.
    #[must_use]
    pub fn with_file_sink(file_sink: FileWendaoSink) -> Self {
        Self {
            file_sink,
            memory_sink: InMemoryWendaoSink::new(),
        }
    }

    /// Create a new adapter with both sinks configured.
    #[must_use]
    pub fn with_sinks(file_sink: FileWendaoSink, memory_sink: InMemoryWendaoSink) -> Self {
        Self {
            file_sink,
            memory_sink,
        }
    }

    /// Get a builder for constructing a WendaoIndexAdapter.
    #[must_use]
    pub fn builder() -> WendaoIndexAdapterBuilder {
        WendaoIndexAdapterBuilder::default()
    }
}

impl Default for WendaoIndexAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WendaoIngestionSink for WendaoIndexAdapter {
    async fn ingest_trace(
        &self,
        trace: &CognitiveTraceRecord,
        document: &LinkGraphSemanticDocument,
    ) -> Result<String, String> {
        // First try the file sink
        match self.file_sink.ingest_trace(trace, document).await {
            Ok(anchor_id) => Ok(anchor_id),
            Err(file_error) => {
                // Fallback to memory sink
                self.memory_sink
                    .ingest_trace(trace, document)
                    .await
                    .map_err(|mem_error| {
                        format!(
                            "File sink failed: {}; Memory sink failed: {}",
                            file_error, mem_error
                        )
                    })
            }
        }
    }
}

/// Builder for constructing a WendaoIndexAdapter.
#[derive(Debug, Default)]
pub struct WendaoIndexAdapterBuilder {
    file_sink: Option<FileWendaoSink>,
    memory_sink: Option<InMemoryWendaoSink>,
}

impl WendaoIndexAdapterBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the file-based sink.
    #[must_use]
    pub fn file_sink(mut self, sink: FileWendaoSink) -> Self {
        self.file_sink = Some(sink);
        self
    }

    /// Set the in-memory sink.
    #[must_use]
    pub fn memory_sink(mut self, sink: InMemoryWendaoSink) -> Self {
        self.memory_sink = Some(sink);
        self
    }

    /// Build the adapter.
    ///
    /// # Panics
    ///
    /// Panics if no file sink was configured.
    #[must_use]
    pub fn build(self) -> WendaoIndexAdapter {
        let file_sink = self.file_sink.expect("file_sink must be configured");
        let memory_sink = self.memory_sink.unwrap_or_default();
        WendaoIndexAdapter::with_sinks(file_sink, memory_sink)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn wendao_index_adapter_new_creates_adapter() {
        let adapter = WendaoIndexAdapter::new();
        assert!(adapter.file_sink.base_dir().ends_with(".cognitive/traces"));
    }

    #[test]
    fn wendao_index_adapter_default() {
        let adapter = WendaoIndexAdapter::default();
        assert!(adapter.file_sink.base_dir().ends_with(".cognitive/traces"));
    }

    #[test]
    fn wendao_index_adapter_with_file_sink() {
        let temp_dir = TempDir::new().unwrap();
        let file_sink = FileWendaoSink::new(temp_dir.path());
        let adapter = WendaoIndexAdapter::with_file_sink(file_sink.clone());
        assert_eq!(adapter.file_sink.base_dir(), temp_dir.path());
    }

    #[test]
    fn wendao_index_adapter_builder_file_sink() {
        let temp_dir = TempDir::new().unwrap();
        let file_sink = FileWendaoSink::new(temp_dir.path());
        let adapter = WendaoIndexAdapterBuilder::new()
            .file_sink(file_sink)
            .build();
        assert_eq!(adapter.file_sink.base_dir(), temp_dir.path());
    }

    #[test]
    fn wendao_index_adapter_builder_requires_file_sink() {
        let result = std::panic::catch_unwind(|| {
            let _adapter = WendaoIndexAdapterBuilder::new().build();
        });
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn wendao_index_adapter_ingest_trace() {
        let temp_dir = TempDir::new().unwrap();
        let file_sink = FileWendaoSink::new(temp_dir.path());
        let adapter = WendaoIndexAdapter::with_file_sink(file_sink);

        let trace = CognitiveTraceRecord::new(
            "trace-adapter-test".to_string(),
            None,
            "AdapterNode".to_string(),
            "Test trace".to_string(),
        );

        let doc = trace.to_semantic_document("doc-1", "test.md");
        let result = adapter.ingest_trace(&trace, &doc).await;

        assert!(result.is_ok());
        let anchor_id = result.unwrap();
        assert!(anchor_id.starts_with("file:"));
    }

    #[tokio::test]
    async fn wendao_index_adapter_fallback_to_memory() {
        // Create adapter with a file sink pointing to a non-existent directory
        let file_sink = FileWendaoSink::new_no_create("/nonexistent/path/that/cannot/be/created");
        let memory_sink = InMemoryWendaoSink::new();
        let adapter = WendaoIndexAdapter::with_sinks(file_sink, memory_sink);

        let trace = CognitiveTraceRecord::new(
            "trace-fallback-test".to_string(),
            None,
            "FallbackNode".to_string(),
            "Test fallback".to_string(),
        );

        let doc = trace.to_semantic_document("doc-2", "test.md");

        // This should fail on file sink but succeed on memory sink
        let result = adapter.ingest_trace(&trace, &doc).await;

        // Memory fallback should succeed
        assert!(result.is_ok());
        let anchor_id = result.unwrap();
        assert!(anchor_id.starts_with("memory:"));
    }

    #[test]
    fn wendao_index_adapter_debug_format() {
        let adapter = WendaoIndexAdapter::new();
        let debug_str = format!("{:?}", adapter);
        assert!(debug_str.contains("WendaoIndexAdapter"));
    }
}
