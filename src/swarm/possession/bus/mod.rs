use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

mod connection;
mod keys;
mod request;
mod response;

/// Valkey transport for remote possession request/response orchestration.
pub struct RemotePossessionBus {
    pub(super) redis_url: String,
    pub(super) connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    pub(super) reconnect_lock: Arc<Mutex<()>>,
}

impl RemotePossessionBus {
    /// Creates a new possession bus from Valkey URL.
    #[must_use]
    pub fn new(redis_url: String) -> Self {
        Self {
            redis_url,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
        }
    }
}
