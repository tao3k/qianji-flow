use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

const RECONNECT_BACKOFF_MS: u64 = 200;

pub(super) async fn run_publish_loop(
    redis_url: Arc<str>,
    channel: Arc<str>,
    mut queue_rx: mpsc::Receiver<Arc<str>>,
) {
    let mut connection: Option<redis::aio::MultiplexedConnection> = None;
    while let Some(payload) = queue_rx.recv().await {
        if let Err(error) = publish_payload(
            redis_url.as_ref(),
            channel.as_ref(),
            payload.as_ref(),
            &mut connection,
        )
        .await
        {
            log::warn!(
                "swarm pulse publish failed on channel '{}': {error}",
                channel.as_ref()
            );
        }
    }
}

async fn publish_payload(
    redis_url: &str,
    channel: &str,
    payload: &str,
    connection: &mut Option<redis::aio::MultiplexedConnection>,
) -> Result<(), String> {
    if connection.is_none() {
        *connection = Some(connect_valkey(redis_url).await?);
    }

    if try_publish_once(channel, payload, connection).await.is_ok() {
        return Ok(());
    }

    *connection = None;
    sleep(Duration::from_millis(RECONNECT_BACKOFF_MS)).await;
    *connection = Some(connect_valkey(redis_url).await?);
    try_publish_once(channel, payload, connection).await
}

async fn connect_valkey(redis_url: &str) -> Result<redis::aio::MultiplexedConnection, String> {
    let client = redis::Client::open(redis_url).map_err(|error| error.to_string())?;
    client
        .get_multiplexed_async_connection()
        .await
        .map_err(|error| error.to_string())
}

async fn try_publish_once(
    channel: &str,
    payload: &str,
    connection: &mut Option<redis::aio::MultiplexedConnection>,
) -> Result<(), String> {
    let Some(connection) = connection.as_mut() else {
        return Err("missing valkey connection".to_string());
    };
    let mut command = redis::cmd("PUBLISH");
    command.arg(channel).arg(payload);
    command
        .query_async::<i64>(connection)
        .await
        .map(|_| ())
        .map_err(|error| error.to_string())
}
