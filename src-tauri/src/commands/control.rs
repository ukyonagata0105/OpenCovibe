use crate::agent::codex_control::{self, CodexInfoCache};
use crate::agent::control::{self, CliInfoCache};
use crate::models::{CliInfo, CliModelInfo, CodexModelList};
use serde::Deserialize;
use tauri::State;

#[tauri::command]
pub async fn get_cli_info(
    cache: State<'_, CliInfoCache>,
    force_refresh: Option<bool>,
) -> Result<CliInfo, String> {
    log::debug!(
        "[control] get_cli_info IPC, force={}",
        force_refresh.unwrap_or(false)
    );
    match control::get_cli_info(&cache, force_refresh.unwrap_or(false)).await {
        Ok(info) => Ok(info),
        Err(e) => {
            log::warn!(
                "[control] CLI info failed ({}): {}, using fallback",
                e.code,
                e.message
            );
            Ok(control::fallback_cli_info())
        }
    }
}

#[tauri::command]
pub async fn get_codex_models(
    cache: State<'_, CodexInfoCache>,
    force_refresh: Option<bool>,
) -> Result<CodexModelList, String> {
    log::debug!(
        "[control] get_codex_models IPC, force={}",
        force_refresh.unwrap_or(false)
    );
    match codex_control::get_codex_models(&cache, force_refresh.unwrap_or(false)).await {
        Ok(list) => Ok(list),
        Err(e) => {
            log::warn!(
                "[control] codex models failed ({}): {}, using fallback",
                e.code,
                e.message
            );
            Ok(codex_control::fallback_models())
        }
    }
}

#[derive(Debug, Deserialize)]
struct OpenAiModelsResponse {
    data: Vec<OpenAiModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAiModel {
    id: String,
}

#[tauri::command]
pub async fn list_codex_provider_models() -> Result<CodexModelList, String> {
    let settings = crate::storage::settings::get_user_settings();
    let provider = settings
        .codex_provider
        .ok_or_else(|| "LM Studio provider is not configured".to_string())?;
    let base_url = provider.base_url.trim_end_matches('/');
    let model_urls = if base_url.ends_with("/v1") {
        vec![format!("{}/models", base_url)]
    } else {
        vec![
            format!("{}/v1/models", base_url),
            format!("{}/models", base_url),
        ]
    };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .no_proxy()
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
    let mut last_error = None;
    let mut body = None;
    for models_url in model_urls {
        let mut request = client.get(&models_url);
        if let Some(api_key) = provider.api_key.as_ref().filter(|v| !v.trim().is_empty()) {
            request = request.bearer_auth(api_key);
        }
        match request.send().await {
            Ok(response) if response.status().is_success() => {
                match response.json::<OpenAiModelsResponse>().await {
                    Ok(parsed) => {
                        body = Some(parsed);
                        break;
                    }
                    Err(e) => {
                        last_error = Some(format!(
                            "Failed to parse LM Studio models from {}: {}",
                            models_url, e
                        ))
                    }
                }
            }
            Ok(response) => {
                last_error = Some(format!(
                    "LM Studio models request to {} failed with HTTP {}",
                    models_url,
                    response.status()
                ));
            }
            Err(e) => {
                last_error = Some(format!(
                    "Failed to fetch LM Studio models from {}: {}",
                    models_url, e
                ))
            }
        }
    }
    let body = body.ok_or_else(|| {
        last_error.unwrap_or_else(|| "Failed to fetch LM Studio models".to_string())
    })?;
    let mut models: Vec<CliModelInfo> = body
        .data
        .into_iter()
        .map(|m| CliModelInfo {
            value: m.id.clone(),
            display_name: m.id,
            description: provider.name.clone(),
            supports_effort: Some(false),
            supported_effort_levels: None,
            supports_adaptive_thinking: None,
        })
        .collect();
    models.sort_by(|a, b| a.value.cmp(&b.value));
    let default_model = models
        .first()
        .map(|m| m.value.clone())
        .or_else(|| (!provider.model.trim().is_empty()).then(|| provider.model.clone()));
    if models.is_empty() && !provider.model.trim().is_empty() {
        models.insert(
            0,
            CliModelInfo {
                value: provider.model.clone(),
                display_name: provider.model.clone(),
                description: provider.name,
                supports_effort: Some(false),
                supported_effort_levels: None,
                supports_adaptive_thinking: None,
            },
        );
    }
    Ok(CodexModelList {
        models,
        default_model,
    })
}
