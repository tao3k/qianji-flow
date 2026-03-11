//! Cross-cluster remote possession protocol over Valkey.

mod bus;
mod error_map;
mod model;
mod util;

pub use bus::RemotePossessionBus;
pub use error_map::map_execution_error_to_response;
pub use model::{RemoteNodeRequest, RemoteNodeResponse};
