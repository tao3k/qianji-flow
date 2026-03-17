//! Asynchronous synaptic-flow scheduler.
/// Valkey checkpointing integration for persisting workflow state.
pub mod checkpoint;
/// Core execution loop and scheduler logic.
pub mod core;
/// Scheduler execution identity for role-aware routing.
pub mod identity;
/// Scheduler execution policy and role-availability probing.
pub mod policy;
/// Pre-execution context preflight utilities.
pub mod preflight;
/// Graph topological tracking and scheduling state.
pub mod state;
pub use self::core::{QianjiScheduler, SchedulerRuntimeServices};
pub use self::identity::SchedulerAgentIdentity;
pub use self::policy::{RoleAvailabilityRegistry, SchedulerExecutionPolicy};
