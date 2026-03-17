//! Global swarm discovery via Valkey heartbeat registry.

pub(crate) mod model;
mod parse;
mod registry;
pub(crate) mod util;

pub use model::{ClusterNodeIdentity, ClusterNodeRecord};
pub use registry::GlobalSwarmRegistry;
