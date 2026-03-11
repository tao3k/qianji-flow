#[derive(Debug, Clone)]
pub(super) struct VoteKeys {
    pub(super) base: String,
    pub(super) votes_hash: String,
    pub(super) weight_counter: String,
    pub(super) winner_marker: String,
    pub(super) first_seen_marker: String,
    pub(super) output_payloads: String,
}

impl VoteKeys {
    pub(super) fn new(session_id: &str, node_id: &str) -> Self {
        let base = format!("xiuxian:consensus:{session_id}:{node_id}");
        Self {
            base: base.clone(),
            votes_hash: format!("{base}:votes"),
            weight_counter: format!("{base}:counts"),
            winner_marker: format!("{base}:winner"),
            first_seen_marker: format!("{base}:first_seen_ms"),
            output_payloads: format!("{base}:outputs"),
        }
    }

    pub(super) fn quorum_channel(&self) -> String {
        format!("{}:channel", self.base)
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct VoteSnapshot {
    pub(super) total_agents: usize,
    pub(super) hash_weight: f64,
}
