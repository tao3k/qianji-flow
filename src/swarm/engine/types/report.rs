/// One worker run result from swarm orchestration.
#[derive(Debug, Clone)]
pub struct SwarmAgentReport {
    /// Worker identity.
    pub agent_id: String,
    /// Optional worker role class.
    pub role_class: Option<String>,
    /// Whether this worker finished successfully.
    pub success: bool,
    /// Final workflow context for this worker on success.
    pub context: Option<serde_json::Value>,
    /// Error message on failure.
    pub error: Option<String>,
    /// Number of turns kept in the local session window.
    pub window_turns: u64,
    /// Number of tool calls tracked in the local session window.
    pub window_tool_calls: u64,
}

/// Final report for one swarm execution.
#[derive(Debug, Clone)]
pub struct SwarmExecutionReport {
    /// Shared session id used by all workers.
    pub session_id: String,
    /// Selected final context (first successful worker output).
    pub final_context: serde_json::Value,
    /// Per-agent run reports.
    pub workers: Vec<SwarmAgentReport>,
}
