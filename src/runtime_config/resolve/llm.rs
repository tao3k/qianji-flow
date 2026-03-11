use super::super::constants::{DEFAULT_API_KEY_ENV, DEFAULT_BASE_URL, DEFAULT_MODEL};
use super::super::env_vars::env_var_or_override;
use super::super::model::{QianjiRuntimeEnv, QianjiRuntimeLlmConfig};
use super::super::toml_config::QianjiTomlLlm;
use std::io;

#[cfg(feature = "llm")]
use std::collections::HashMap;
#[cfg(feature = "llm")]
use xiuxian_llm::llm::{
    LlmProviderProfileInput, LlmRuntimeDefaults, LlmRuntimeProfileEnv, LlmRuntimeProfileInput,
    OpenAIWireApi, resolve_openai_runtime_profile,
};
#[cfg(not(feature = "llm"))]
use xiuxian_macros::string_first_non_empty;

pub(super) fn resolve_qianji_runtime_llm(
    file_llm: &QianjiTomlLlm,
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    #[cfg(feature = "llm")]
    {
        resolve_qianji_runtime_llm_with_llm_feature(file_llm, runtime_env)
    }

    #[cfg(not(feature = "llm"))]
    {
        resolve_qianji_runtime_llm_without_llm_feature(file_llm, runtime_env)
    }
}

#[cfg(feature = "llm")]
fn resolve_qianji_runtime_llm_with_llm_feature(
    file_llm: &QianjiTomlLlm,
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    let providers = file_llm
        .providers
        .as_ref()
        .map_or_else(HashMap::new, |providers| {
            providers
                .iter()
                .map(|(name, config)| {
                    (
                        name.clone(),
                        LlmProviderProfileInput {
                            model: config.model.clone(),
                            base_url: config.base_url.clone(),
                            api_key: config.api_key.clone(),
                            api_key_env: config.api_key_env.clone(),
                            wire_api: config.wire_api.clone(),
                        },
                    )
                })
                .collect::<HashMap<_, _>>()
        });
    let profile_input = LlmRuntimeProfileInput {
        model: file_llm.model.clone(),
        default_model: file_llm.default_model.clone(),
        base_url: file_llm.base_url.clone(),
        api_key_env: file_llm.api_key_env.clone(),
        api_key: file_llm.api_key.clone(),
        wire_api: file_llm.wire_api.clone(),
        default_provider: file_llm.default_provider.clone(),
        providers,
    };
    let profile_env = LlmRuntimeProfileEnv {
        provider_override: runtime_env
            .qianji_llm_provider
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "QIANJI_LLM_PROVIDER")),
        model_override: runtime_env
            .qianji_llm_model
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "QIANJI_LLM_MODEL")),
        base_url_override: runtime_env
            .openai_api_base
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "OPENAI_API_BASE")),
        api_key_override: runtime_env.openai_api_key.clone(),
        wire_api_override: runtime_env
            .qianji_llm_wire_api
            .clone()
            .or_else(|| env_var_or_override(runtime_env, "QIANJI_LLM_WIRE_API")),
        env_vars: runtime_env.extra_env.clone(),
    };
    let defaults = LlmRuntimeDefaults {
        provider: "openai".to_string(),
        model: DEFAULT_MODEL.to_string(),
        base_url: DEFAULT_BASE_URL.to_string(),
        api_key_env: DEFAULT_API_KEY_ENV.to_string(),
        wire_api: OpenAIWireApi::ChatCompletions,
    };
    let resolved = resolve_openai_runtime_profile(&profile_input, &profile_env, &defaults)
        .map_err(|error| {
            let message =
                format!("failed to resolve qianji runtime llm profile from xiuxian.toml: {error}");
            let kind = if message.contains("API key") {
                io::ErrorKind::NotFound
            } else {
                io::ErrorKind::InvalidData
            };
            io::Error::new(kind, message)
        })?;
    Ok(QianjiRuntimeLlmConfig {
        model: resolved.model,
        base_url: resolved.base_url,
        api_key_env: resolved.api_key_env,
        wire_api: resolved.wire_api.as_str().to_string(),
        api_key: resolved.api_key,
    })
}

#[cfg(not(feature = "llm"))]
fn resolve_qianji_runtime_llm_without_llm_feature(
    file_llm: &QianjiTomlLlm,
    runtime_env: &QianjiRuntimeEnv,
) -> io::Result<QianjiRuntimeLlmConfig> {
    let model = string_first_non_empty!(
        runtime_env.qianji_llm_model.as_deref(),
        env_var_or_override(runtime_env, "QIANJI_LLM_MODEL").as_deref(),
        file_llm.model.as_deref(),
        Some(DEFAULT_MODEL),
    );
    let base_url = string_first_non_empty!(
        runtime_env.openai_api_base.as_deref(),
        env_var_or_override(runtime_env, "OPENAI_API_BASE").as_deref(),
        file_llm.base_url.as_deref(),
        Some(DEFAULT_BASE_URL),
    );
    let api_key_env =
        string_first_non_empty!(file_llm.api_key_env.as_deref(), Some(DEFAULT_API_KEY_ENV),);
    let maybe_api_key = string_first_non_empty!(
        runtime_env.openai_api_key.as_deref(),
        env_var_or_override(runtime_env, "OPENAI_API_KEY").as_deref(),
        env_var_or_override(runtime_env, api_key_env.as_str()).as_deref(),
    );
    if maybe_api_key.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "missing Qianji API key; set OPENAI_API_KEY or {api_key_env} (resolved from qianji.toml)"
            ),
        ));
    }
    Ok(QianjiRuntimeLlmConfig {
        model,
        base_url,
        api_key_env,
        wire_api: "chat_completions".to_string(),
        api_key: maybe_api_key,
    })
}
