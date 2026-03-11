//! Native `Wendao` ingestion mechanism for memory-promotion workflows.

mod entity;
mod mechanism;
mod persistence;
mod scope;

pub use mechanism::WendaoIngesterMechanism;
