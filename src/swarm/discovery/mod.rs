//! Global swarm discovery via Valkey heartbeat registry.

mod model;
mod parse;
mod registry;
mod util;

pub use model::{ClusterNodeIdentity, ClusterNodeRecord};
pub use registry::GlobalSwarmRegistry;
