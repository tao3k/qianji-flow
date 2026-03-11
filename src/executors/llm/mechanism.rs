//! LLM analysis mechanism for high-precision reasoning.

use crate::contracts::{FlowInstruction, QianjiMechanism, QianjiOutput};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use xiuxian_llm::llm::{ChatMessage, ChatRequest, LlmClient};

/// Mechanism responsible for performing LLM inference based on annotated context.
pub struct LlmAnalyzer {
    /// Thread-safe client for LLM communication.
    pub client: Arc<dyn LlmClient>,
    /// Target model name.
    pub model: String,
}

#[async_trait]
impl QianjiMechanism for LlmAnalyzer {
    async fn execute(&self, context: &serde_json::Value) -> Result<QianjiOutput, String> {
        let prompt = context
            .get("annotated_prompt")
            .and_then(|v| v.as_str())
            .ok_or("Missing 'annotated_prompt'")?;
        let user_query = context
            .get("query")
            .and_then(|v| v.as_str())
            .unwrap_or("Summarize facts.");

        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: user_query.to_string(),
                },
            ],
            temperature: 0.1,
        };

        let conclusion = self
            .client
            .chat(request)
            .await
            .map_err(|e| format!("LLM execution failed: {}", e))?;

        Ok(QianjiOutput {
            data: json!({ "analysis_conclusion": conclusion }),
            instruction: FlowInstruction::Continue,
        })
    }

    fn weight(&self) -> f32 {
        3.0
    }
}
