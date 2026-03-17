//! Sovereign Memory Module (Blueprint V6.1).
//!
//! This module implements the "问道归元" (Wendao Guiyuan) architecture for
//! giving agents true sovereign memory - persistent reasoning traces that
//! connect Intent → Reasoning → Outcome.
//!
//! ## Architecture
//!
//! ```text
//! Qianji Execution Loop
//!        │
//!        ▼ ZhenfaStreamingEvent
//! ThoughtAggregator.process_event()
//!        │
//!        ▼ CognitiveTraceRecord
//! ArtifactObserver.ingest_artifact()
//!        │
//!        ▼ WendaoIngestionSink (FileWendaoSink)
//!        │
//!        ▼ Markdown file in .cognitive/traces/
//!        │
//!        ▼ Wendao LinkGraphIndex (on next rebuild)
//! ```
//!
//! ## Historical Sovereignty
//!
//! This enables querying the knowledge graph for the reasoning chain that
//! led to any commit or decision: "Query Wendao for the reasoning chain
//! that led to Commit-X".

pub mod artifact_observer;
pub mod thought_aggregator;
pub mod wendao_sink;

pub use artifact_observer::{
    ArtifactIngestionResult, ArtifactObserver, ArtifactObserverBuilder, ArtifactObserverConfig,
    NoopWendaoIngestionSink, WendaoIngestionSink,
};
pub use thought_aggregator::{ThoughtAggregator, ToolCallRecord};
pub use wendao_sink::{
    CompositeWendaoSink, CompositeWendaoSinkBuilder, FileWendaoSink, InMemoryWendaoSink,
};
