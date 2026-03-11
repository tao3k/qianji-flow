use super::super::ConsensusManager;
use super::super::keys::VoteKeys;
use crate::consensus::models::{AgentVote, ConsensusPolicy, ConsensusResult};
use crate::consensus::thresholds::required_weight_threshold;
use anyhow::Result;

impl ConsensusManager {
    pub(in crate::consensus::manager) async fn submit_vote_payload(
        &self,
        session_id: &str,
        node_id: &str,
        vote: AgentVote,
        output_payload: Option<&str>,
        policy: &ConsensusPolicy,
    ) -> Result<ConsensusResult> {
        let keys = VoteKeys::new(session_id, node_id);
        if let Some(winner) = self.current_winner(keys.winner_marker.as_str()).await? {
            return Ok(ConsensusResult::Agreed(winner));
        }

        if let Some(payload) = output_payload {
            self.store_output_payload(&keys, vote.output_hash.as_str(), payload)
                .await?;
        }

        let snapshot = self.record_vote(keys.clone(), &vote).await?;
        let required_weight = required_weight_threshold(policy, snapshot.total_agents);
        if snapshot.total_agents >= policy.min_agents && snapshot.hash_weight >= required_weight {
            let winner = self
                .mark_or_read_winner(&keys, vote.output_hash.as_str())
                .await?;
            return Ok(ConsensusResult::Agreed(winner));
        }

        if self
            .timeout_exceeded(keys.first_seen_marker.as_str(), policy)
            .await?
        {
            return Ok(ConsensusResult::Failed("consensus_timeout".to_string()));
        }

        Ok(ConsensusResult::Pending)
    }
}
