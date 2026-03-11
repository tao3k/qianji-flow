use std::sync::Arc;

use tokio::sync::{Mutex, RwLock};

mod connection;
mod discover;
mod heartbeat;
mod keys;
mod payload;

/// Valkey-backed global discovery registry.
pub struct GlobalSwarmRegistry {
    pub(super) redis_url: String,
    pub(super) connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    pub(super) reconnect_lock: Arc<Mutex<()>>,
}

impl GlobalSwarmRegistry {
    /// Creates a discovery registry using the provided Valkey URL.
    #[must_use]
    pub fn new(redis_url: String) -> Self {
        Self {
            redis_url,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }
}
