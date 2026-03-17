use crate::consensus::ConsensusManager;
use crate::consensus::manager::time::current_unix_millis;
use crate::consensus::models::ConsensusPolicy;
use anyhow::Result;

impl ConsensusManager {
    pub(super) async fn timeout_exceeded(
        &self,
        first_seen_key: &str,
        policy: &ConsensusPolicy,
    ) -> Result<bool> {
        if policy.timeout_ms == 0 {
            return Ok(false);
        }
        let first_seen_ms: Option<u64> = self
            .run_command("consensus_get_first_seen_ms", || {
                let mut command = redis::cmd("GET");
                command.arg(first_seen_key);
                command
            })
            .await?;
        let Some(first_seen_ms) = first_seen_ms else {
            return Ok(false);
        };
        let now_ms = current_unix_millis();
        Ok(now_ms.saturating_sub(first_seen_ms) >= policy.timeout_ms)
    }
}
