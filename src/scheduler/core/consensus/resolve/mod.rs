use super::super::QianjiScheduler;
use super::super::types::{ConsensusCheckpointView, ConsensusOutcome};
use crate::consensus::ConsensusResult;
use crate::error::QianjiError;
use crate::telemetry::ConsensusStatus;

mod call_ctx;
mod handlers;
mod policy;

use call_ctx::ConsensusCallCtx;
use policy::consensus_target_progress;

impl QianjiScheduler {
    pub(in crate::scheduler::core) async fn resolve_consensus_output(
        &self,
        node_idx: petgraph::stable_graph::NodeIndex,
        output_data: &serde_json::Value,
        checkpoint: &ConsensusCheckpointView<'_>,
    ) -> Result<ConsensusOutcome, QianjiError> {
        let (node_id, consensus_policy) = {
            let engine = self.engine.read().await;
            (
                engine.graph[node_idx].id.clone(),
                engine.graph[node_idx].consensus.clone(),
            )
        };

        let (Some(policy), Some(manager), Some(sid)) = (
            consensus_policy,
            &self.consensus_manager,
            checkpoint.session_id,
        ) else {
            return Ok(ConsensusOutcome::Proceed(output_data.clone()));
        };

        let output_json = serde_json::to_string(output_data).unwrap_or_default();
        let output_hash = format!("{:x}", md5::compute(&output_json));
        let telemetry_target = Some(consensus_target_progress(&policy));
        let vote_result = manager
            .submit_vote_with_payload(
                sid,
                &node_id,
                output_hash.clone(),
                Some(&output_json),
                &policy,
            )
            .await
            .map_err(|error| QianjiError::Execution(error.to_string()))?;
        let call = ConsensusCallCtx {
            manager,
            session_id: sid,
            node_id: &node_id,
            output_hash: &output_hash,
            output_data,
            telemetry_target,
        };
        match vote_result {
            ConsensusResult::Agreed(agreed_hash) => {
                self.handle_consensus_agreed(&call, &agreed_hash).await
            }
            ConsensusResult::Pending => {
                self.handle_consensus_pending(node_idx, checkpoint, &policy, &call)
                    .await
            }
            ConsensusResult::Failed(reason) => {
                self.emit_consensus_spike(
                    call.session_id,
                    call.node_id,
                    ConsensusStatus::Failed,
                    None,
                    call.telemetry_target,
                );
                Err(QianjiError::Execution(format!(
                    "Consensus failed for {}: {reason}",
                    call.node_id
                )))
            }
        }
    }
}
