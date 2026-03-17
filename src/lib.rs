//! xiuxian-qianji: The Thousand Mechanisms Engine.
//!
//! A high-performance, probabilistic DAG executor based on petgraph.
//! Follows Rust 2024 Edition standards.

/// Application-layer scheduler factories and built-in pipeline presets.
pub mod app;
/// High-level laboratory API for end-to-end workflow execution.
pub mod bootcamp;
/// Distributed consensus management for multi-agent synchronization.
pub mod consensus;
/// Contract definitions for nodes, instructions, and manifests.
pub mod contracts;
/// Core graph engine based on petgraph.
pub mod engine;
/// Unified error handling.
pub mod error;
/// Built-in node execution mechanisms.
pub mod executors;
/// Graphical layout and aesthetic engine (QGS).
pub mod layout;
/// Manifest inspection helpers.
pub mod manifest;
/// Runtime configuration resolver (`resources/config/qianji.toml` + user overrides).
pub mod runtime_config;
/// Formal logic and safety auditing.
pub mod safety;
/// Asynchronous synaptic-flow scheduler.
pub mod scheduler;
/// Sovereign Memory Module (Blueprint V6.1) - Agent reasoning trace persistence.
pub mod sovereign;
/// Multi-agent swarm orchestration runtime.
pub mod swarm;
/// Real-time swarm telemetry contracts and Valkey emitter.
pub mod telemetry;

#[cfg(feature = "pyo3")]
/// Python bindings via `PyO3`.
pub mod python_module;

pub use app::{MEMORY_PROMOTION_PIPELINE_TOML, QianjiApp, RESEARCH_TRINITY_TOML};
pub use bootcamp::{
    BootcampLlmMode, BootcampRunOptions, BootcampVfsMount, WorkflowReport, run_scenario,
    run_workflow, run_workflow_with_mounts,
};
pub use contracts::{
    FlowInstruction, NodeQianhuanExecutionMode, NodeStatus, QianjiManifest, QianjiMechanism,
    QianjiOutput,
};
pub use engine::QianjiEngine;
pub use engine::compiler::QianjiCompiler;
pub use manifest::{manifest_declares_qianhuan_bindings, manifest_requires_llm};
pub use safety::QianjiSafetyGuard;
pub use scheduler::QianjiScheduler;
pub use scheduler::SchedulerAgentIdentity;
pub use scheduler::{RoleAvailabilityRegistry, SchedulerExecutionPolicy};
pub use swarm::{
    ClusterNodeIdentity, ClusterNodeRecord, GlobalSwarmRegistry, RemoteNodeRequest,
    RemoteNodeResponse, RemotePossessionBus, SwarmAgentConfig, SwarmAgentReport, SwarmEngine,
    SwarmExecutionOptions, SwarmExecutionReport, map_execution_error_to_response,
};
pub use telemetry::{
    ConsensusStatus, DEFAULT_PULSE_CHANNEL, NodeTransitionPhase, NoopPulseEmitter, PulseEmitter,
    SwarmEvent, ValkeyPulseEmitter, unix_millis_now,
};

#[cfg(feature = "llm")]
/// Shared LLM client trait object type when `llm` feature is enabled.
pub type QianjiLlmClient = dyn xiuxian_llm::llm::LlmClient;

#[cfg(not(feature = "llm"))]
/// Placeholder trait object type when `llm` feature is disabled.
pub type QianjiLlmClient = dyn std::any::Any + Send + Sync;
