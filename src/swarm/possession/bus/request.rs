use super::RemotePossessionBus;
use super::keys::{queue_key, request_key};
use crate::swarm::possession::model::{RemoteNodeRequest, RemoteNodeResponse};
use anyhow::{Result, anyhow};
use tokio::time::Duration;

impl RemotePossessionBus {
    /// Submits a remote request and enqueues it for target role workers.
    ///
    /// # Errors
    ///
    /// Returns an error when serialization fails or Valkey commands fail.
    pub async fn submit_request(
        &self,
        request: &RemoteNodeRequest,
        ttl_seconds: u64,
    ) -> Result<()> {
        if ttl_seconds == 0 {
            return Err(anyhow!("ttl_seconds must be > 0"));
        }
        if request.role_class.trim().is_empty() {
            return Err(anyhow!("request.role_class must not be empty"));
        }
        let request_json = serde_json::to_string(request)?;
        let req_key = request_key(&request.request_id);
        let role_queue_key = queue_key(&request.role_class);

        let _: i64 = self
            .run_command("remote_possession_hset_request", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&req_key)
                    .arg("request")
                    .arg(&request_json)
                    .arg("status")
                    .arg("pending")
                    .arg("created_ms")
                    .arg(request.created_ms.to_string());
                command
            })
            .await?;
        let _: bool = self
            .run_command("remote_possession_expire_request", || {
                let mut command = redis::cmd("EXPIRE");
                command.arg(&req_key).arg(ttl_seconds);
                command
            })
            .await?;
        let _: i64 = self
            .run_command("remote_possession_queue_push", || {
                let mut command = redis::cmd("RPUSH");
                command.arg(&role_queue_key).arg(&request.request_id);
                command
            })
            .await?;
        Ok(())
    }

    /// Claims one pending request from a role queue.
    ///
    /// Returns `Ok(None)` when no request arrives in `block_timeout`.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey commands fail or request payload is malformed.
    pub async fn claim_next_for_role(
        &self,
        role_class: &str,
        claimer_id: &str,
        block_timeout: Duration,
    ) -> Result<Option<RemoteNodeRequest>> {
        let role_queue_key = queue_key(role_class);
        let timeout_seconds = block_timeout.as_secs().max(1);
        let popped: Option<Vec<String>> = self
            .run_command("remote_possession_blpop", || {
                let mut command = redis::cmd("BLPOP");
                command.arg(&role_queue_key).arg(timeout_seconds);
                command
            })
            .await?;

        let Some(raw) = popped else {
            return Ok(None);
        };
        if raw.len() != 2 {
            return Ok(None);
        }
        let request_id = raw[1].clone();
        let req_key = request_key(&request_id);
        let request_json: Option<String> = self
            .run_command("remote_possession_hget_request_payload", || {
                let mut command = redis::cmd("HGET");
                command.arg(&req_key).arg("request");
                command
            })
            .await?;
        let Some(request_json) = request_json else {
            return Ok(None);
        };
        let request: RemoteNodeRequest = serde_json::from_str(&request_json)?;
        let _: i64 = self
            .run_command("remote_possession_hset_claimed", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&req_key)
                    .arg("status")
                    .arg("claimed")
                    .arg("claimer_id")
                    .arg(claimer_id);
                command
            })
            .await?;
        Ok(Some(request))
    }

    /// Convenience helper: submit request and wait for one response.
    ///
    /// # Errors
    ///
    /// Returns an error when submit or wait operations fail.
    pub async fn request_and_wait(
        &self,
        request: &RemoteNodeRequest,
        ttl_seconds: u64,
        max_wait: Duration,
    ) -> Result<Option<RemoteNodeResponse>> {
        self.submit_request(request, ttl_seconds).await?;
        self.wait_response(&request.request_id, max_wait).await
    }
}
