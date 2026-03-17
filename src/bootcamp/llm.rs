use super::BootcampLlmMode;
use crate::QianjiLlmClient;
use crate::error::QianjiError;
#[cfg(feature = "llm")]
use crate::runtime_config::resolve_qianji_runtime_llm_config;
use std::sync::Arc;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::{ChatRequest, LlmClient, LlmResult, OpenAICompatibleClient, OpenAIWireApi};

#[cfg(feature = "llm")]
use async_trait::async_trait;

#[cfg(feature = "llm")]
pub(super) fn resolve_bootcamp_llm_client(
    requires_llm: bool,
    llm_mode: BootcampLlmMode,
) -> Result<Option<Arc<QianjiLlmClient>>, QianjiError> {
    match llm_mode {
        BootcampLlmMode::Disabled => {
            if requires_llm {
                return Err(QianjiError::Topology(
                    "workflow requires LLM, but bootcamp llm_mode is disabled".to_string(),
                ));
            }
            Ok(None)
        }
        BootcampLlmMode::RuntimeDefault => {
            let runtime = resolve_qianji_runtime_llm_config().map_err(|error| {
                QianjiError::Topology(format!(
                    "failed to resolve qianji llm runtime config for bootcamp: {error}"
                ))
            })?;
            let client: Arc<QianjiLlmClient> = Arc::new(OpenAICompatibleClient {
                api_key: runtime.api_key,
                base_url: runtime.base_url,
                wire_api: OpenAIWireApi::parse(Some(runtime.wire_api.as_str())),
                http: reqwest::Client::new(),
            });
            Ok(Some(client))
        }
        BootcampLlmMode::Mock { response } => {
            let client: Arc<QianjiLlmClient> = Arc::new(MockBootcampLlmClient { response });
            Ok(Some(client))
        }
        BootcampLlmMode::External(client) => Ok(Some(client)),
    }
}

#[cfg(not(feature = "llm"))]
pub(super) fn resolve_bootcamp_llm_client(
    requires_llm: bool,
    _llm_mode: BootcampLlmMode,
) -> Result<Option<Arc<QianjiLlmClient>>, QianjiError> {
    if requires_llm {
        return Err(QianjiError::Topology(
            "workflow requires LLM; enable feature `llm` for xiuxian-qianji".to_string(),
        ));
    }
    Ok(None)
}

#[cfg(feature = "llm")]
#[derive(Debug, Clone)]
struct MockBootcampLlmClient {
    response: String,
}

#[cfg(feature = "llm")]
#[async_trait]
impl LlmClient for MockBootcampLlmClient {
    async fn chat(&self, _request: ChatRequest) -> LlmResult<String> {
        Ok(self.response.clone())
    }

    async fn chat_stream(&self, _request: ChatRequest) -> LlmResult<xiuxian_llm::llm::client::ChatStream> {
        use futures::stream;

        let chunks: Vec<LlmResult<String>> = vec![Ok(self.response.clone())];
        Ok(Box::pin(stream::iter(chunks)))
    }
}
