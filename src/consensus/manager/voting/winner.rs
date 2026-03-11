use super::super::ConsensusManager;
use super::super::keys::VoteKeys;
use anyhow::Result;

impl ConsensusManager {
    pub(in crate::consensus::manager) async fn current_winner(
        &self,
        winner_key: &str,
    ) -> Result<Option<String>> {
        self.run_command("consensus_get_winner", || {
            let mut command = redis::cmd("GET");
            command.arg(winner_key);
            command
        })
        .await
    }

    pub(super) async fn mark_or_read_winner(&self, keys: &VoteKeys, hash: &str) -> Result<String> {
        let was_inserted: i64 = self
            .run_command("consensus_set_winner_nx", || {
                let mut command = redis::cmd("SETNX");
                command.arg(&keys.winner_marker).arg(hash);
                command
            })
            .await?;
        if was_inserted == 1 {
            self.publish_winner(keys, hash).await?;
        }
        if let Some(winner) = self.current_winner(keys.winner_marker.as_str()).await? {
            return Ok(winner);
        }
        Ok(hash.to_string())
    }

    async fn publish_winner(&self, keys: &VoteKeys, hash: &str) -> Result<()> {
        let channel = keys.quorum_channel();
        let _: i64 = self
            .run_command("consensus_publish_winner", || {
                let mut command = redis::cmd("PUBLISH");
                command.arg(&channel).arg(hash);
                command
            })
            .await?;
        Ok(())
    }
}
