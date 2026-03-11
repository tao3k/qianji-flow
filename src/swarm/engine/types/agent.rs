/// Runtime identity and scheduler hints for one swarm worker.
#[derive(Debug, Clone)]
pub struct SwarmAgentConfig {
    /// Stable logical agent id (used for consensus vote identity).
    pub agent_id: String,
    /// Optional role class for node routing (for example `student`/`steward`).
    pub role_class: Option<String>,
    /// Vote weight used by distributed consensus policy.
    pub weight: f32,
    /// Session window size for this worker.
    pub window_size: usize,
}

impl SwarmAgentConfig {
    /// Creates a new agent profile with defaults.
    #[must_use]
    pub fn new(agent_id: impl Into<String>) -> Self {
        Self {
            agent_id: agent_id.into(),
            role_class: None,
            weight: 1.0,
            window_size: 1000,
        }
    }
}
