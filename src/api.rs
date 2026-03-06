//! Shared API client for OpenAI-compatible chat completions.

use crate::profile;
use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Clone)]
pub struct AssistantConfig {
    pub enabled: Option<bool>,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub custom_headers: Option<Vec<String>>,
}

pub const DEFAULT_MODEL: &str = "Doubao-Seed-2.0-Code";
pub const DEFAULT_BASE_URL: &str = "https://ark.cn-beijing.volces.com/api/coding/v1";

pub const AVAILABLE_MODELS: &[&str] = &[
    "Doubao-Seed-2.0-Code",
    "Doubao-Seed-2.0-pro",
    "DeepSeek-V3.2",
    "MiniMax-M2.5",
    "GLM-4.7",
];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub fn load_config() -> anyhow::Result<AssistantConfig> {
    let path = profile::assistant_toml_path();
    if !path.exists() {
        return Ok(AssistantConfig {
            enabled: Some(true),
            api_key: None,
            model: Some(DEFAULT_MODEL.to_string()),
            base_url: Some(DEFAULT_BASE_URL.to_string()),
            custom_headers: None,
        });
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let config: AssistantConfig = toml::from_str(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(config)
}

pub fn save_config(config: &AssistantConfig) -> anyhow::Result<()> {
    let path = profile::assistant_toml_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut content = String::from("# Kuku Assistant configuration\n\n");
    content.push_str(&format!(
        "enabled = {}\n",
        config.enabled.unwrap_or(true)
    ));
    if let Some(key) = &config.api_key {
        content.push_str(&format!("api_key = \"{key}\"\n"));
    } else {
        content.push_str("# api_key = \"<your_api_key>\"\n");
    }
    content.push_str(&format!(
        "model = \"{}\"\n",
        config.model.as_deref().unwrap_or(DEFAULT_MODEL)
    ));
    content.push_str(&format!(
        "base_url = \"{}\"\n",
        config.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL)
    ));
    if let Some(headers) = &config.custom_headers {
        let items: Vec<String> = headers.iter().map(|h| format!("\"{h}\"")).collect();
        content.push_str(&format!("custom_headers = [{}]\n", items.join(", ")));
    }
    content.push_str("# Available models: Doubao-Seed-2.0-pro, Doubao-Seed-2.0-Code, MiniMax-M2.5, GLM-4.7, DeepSeek-V3.2\n");

    std::fs::write(&path, content)
        .with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub async fn chat(
    config: &AssistantConfig,
    system_prompt: &str,
    user_msg: &str,
) -> anyhow::Result<String> {
    chat_with_tokens(config, system_prompt, user_msg, 512).await
}

pub async fn chat_with_tokens(
    config: &AssistantConfig,
    system_prompt: &str,
    user_msg: &str,
    max_tokens: u32,
) -> anyhow::Result<String> {
    let messages = vec![
        Message {
            role: "system".into(),
            content: Some(system_prompt.into()),
            tool_calls: None,
            tool_call_id: None,
        },
        Message {
            role: "user".into(),
            content: Some(user_msg.into()),
            tool_calls: None,
            tool_call_id: None,
        },
    ];
    let resp = raw_chat(config, messages, max_tokens, None).await?;
    Ok(resp
        .content
        .unwrap_or_default())
}

/// Send chat with tools support. Returns the full response message.
pub async fn chat_with_tools(
    config: &AssistantConfig,
    messages: Vec<Message>,
    max_tokens: u32,
    tools: Option<Vec<Value>>,
) -> anyhow::Result<Message> {
    raw_chat(config, messages, max_tokens, tools).await
}

async fn raw_chat(
    config: &AssistantConfig,
    messages: Vec<Message>,
    max_tokens: u32,
    tools: Option<Vec<Value>>,
) -> anyhow::Result<Message> {
    let api_key = config
        .api_key
        .as_deref()
        .filter(|k| !k.is_empty())
        .ok_or_else(|| anyhow::anyhow!("No API key configured"))?;

    let model = config.model.as_deref().unwrap_or(DEFAULT_MODEL);
    let base_url = config.base_url.as_deref().unwrap_or(DEFAULT_BASE_URL);
    let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

    let mut builder = reqwest::Client::new()
        .post(&url)
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json");

    if let Some(headers) = &config.custom_headers {
        for h in headers {
            if let Some((name, value)) = h.split_once(':') {
                let name = name.trim();
                let lower = name.to_ascii_lowercase();
                if lower == "authorization" || lower == "content-type" {
                    continue;
                }
                builder = builder.header(name, value.trim());
            }
        }
    }

    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
        "temperature": 0.3,
    });

    if let Some(tools) = &tools {
        if !tools.is_empty() {
            body["tools"] = Value::Array(tools.clone());
        }
    }

    let resp = builder.json(&body).send().await.context("send API request")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("API {status}: {text}");
    }

    let chat: ChatResponse = resp.json().await.context("parse API response")?;
    chat.choices
        .into_iter()
        .next()
        .map(|c| c.message)
        .context("empty response")
}
