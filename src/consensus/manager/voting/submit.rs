use crate::consensus::ConsensusManager;
use crate::consensus::manager::CONSENSUS_VOTE_TTL_SECONDS;
use crate::consensus::manager::keys::{VoteKeys, VoteSnapshot};
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

    async fn store_output_payload(
        &self,
        keys: &VoteKeys,
        output_hash: &str,
        payload: &str,
    ) -> Result<()> {
        let _: i64 = self
            .run_command("consensus_store_output_payload", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&keys.output_payloads)
                    .arg(output_hash)
                    .arg(payload);
                command
            })
            .await?;
        self.touch_vote_ttl(keys).await?;
        Ok(())
    }

    async fn record_vote(&self, keys: VoteKeys, vote: &AgentVote) -> Result<VoteSnapshot> {
        let existing_vote: Option<String> = self
            .run_command("consensus_get_existing_vote", || {
                let mut command = redis::cmd("HGET");
                command.arg(&keys.votes_hash).arg(&vote.agent_id);
                command
            })
            .await?;
        let previous_vote = existing_vote
            .as_deref()
            .and_then(|raw| serde_json::from_str::<AgentVote>(raw).ok());
        let same_vote = previous_vote.as_ref().is_some_and(|previous| {
            previous.output_hash == vote.output_hash
                && (previous.weight - vote.weight).abs() <= f32::EPSILON
        });

        if let Some(previous) = previous_vote.as_ref().filter(|_| !same_vote) {
            let _: f64 = self
                .run_command("consensus_decrement_previous_weight", || {
                    let mut command = redis::cmd("HINCRBYFLOAT");
                    command
                        .arg(&keys.weight_counter)
                        .arg(&previous.output_hash)
                        .arg(-f64::from(previous.weight));
                    command
                })
                .await?;
        }

        if !same_vote {
            let serialized_vote = serde_json::to_string(vote)?;
            let _: i64 = self
                .run_command("consensus_store_vote", || {
                    let mut command = redis::cmd("HSET");
                    command
                        .arg(&keys.votes_hash)
                        .arg(&vote.agent_id)
                        .arg(&serialized_vote);
                    command
                })
                .await?;
        }

        let hash_weight = if same_vote {
            self.current_hash_weight(&keys, vote.output_hash.as_str())
                .await?
        } else {
            self.bump_hash_weight(&keys, vote.output_hash.as_str(), f64::from(vote.weight))
                .await?
        };

        let _: i64 = self
            .run_command("consensus_mark_first_seen", || {
                let mut command = redis::cmd("SETNX");
                command
                    .arg(&keys.first_seen_marker)
                    .arg(vote.timestamp_ms.to_string());
                command
            })
            .await?;

        let total_agents: usize = self
            .run_command("consensus_count_votes", || {
                let mut command = redis::cmd("HLEN");
                command.arg(&keys.votes_hash);
                command
            })
            .await?;

        self.touch_vote_ttl(&keys).await?;

        Ok(VoteSnapshot {
            total_agents,
            hash_weight,
        })
    }

    async fn current_hash_weight(&self, keys: &VoteKeys, output_hash: &str) -> Result<f64> {
        let weight: Option<f64> = self
            .run_command("consensus_get_hash_weight", || {
                let mut command = redis::cmd("HGET");
                command.arg(&keys.weight_counter).arg(output_hash);
                command
            })
            .await?;
        Ok(weight.unwrap_or_default())
    }

    async fn bump_hash_weight(
        &self,
        keys: &VoteKeys,
        output_hash: &str,
        delta: f64,
    ) -> Result<f64> {
        self.run_command("consensus_increment_hash_weight", || {
            let mut command = redis::cmd("HINCRBYFLOAT");
            command
                .arg(&keys.weight_counter)
                .arg(output_hash)
                .arg(delta);
            command
        })
        .await
    }

    async fn touch_vote_ttl(&self, keys: &VoteKeys) -> Result<()> {
        for key in [
            &keys.votes_hash,
            &keys.weight_counter,
            &keys.winner_marker,
            &keys.first_seen_marker,
            &keys.output_payloads,
        ] {
            let _: bool = self
                .run_command("consensus_touch_vote_ttl", || {
                    let mut command = redis::cmd("EXPIRE");
                    command.arg(key).arg(CONSENSUS_VOTE_TTL_SECONDS);
                    command
                })
                .await?;
        }
        Ok(())
    }
}
