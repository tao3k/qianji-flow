use std::collections::HashSet;

pub(in crate::scheduler::core) struct ConsensusCheckpointView<'a> {
    pub(in crate::scheduler::core) session_id: Option<&'a str>,
    pub(in crate::scheduler::core) redis_url: Option<&'a str>,
    pub(in crate::scheduler::core) total_steps: u32,
    pub(in crate::scheduler::core) active_branches: &'a HashSet<String>,
    pub(in crate::scheduler::core) context: &'a serde_json::Value,
}

pub(in crate::scheduler::core) enum ConsensusOutcome {
    Proceed(serde_json::Value),
    Suspend(serde_json::Value),
}
