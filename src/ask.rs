//! Free-form AI Q&A with optional MCP tool support.

use crate::api::{self, Message};
use crate::mcp;
use anyhow::Context;
use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct AskCommand {
    /// The question to ask AI
    #[arg(trailing_var_arg = true, required = true)]
    question: Vec<String>,
}

const MAX_TOOL_ROUNDS: usize = 5;

impl AskCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let config = api::load_config()?;

        if config.enabled == Some(false) {
            eprintln!("\x1b[33m[kuku] AI assistant is disabled.\x1b[0m");
            return Ok(());
        }

        if config.api_key.as_deref().filter(|k| !k.is_empty()).is_none() {
            eprintln!("\x1b[33m[kuku] No API key configured. Run `kuku ai` to set up.\x1b[0m");
            return Ok(());
        }

        let question = self.question.join(" ");

        let system_prompt = "\
            You are a helpful PowerShell and Windows command-line assistant.\n\
            Answer concisely. If the answer involves a command, show it first, then explain briefly.\n\
            You may have tools available. Use them when they help answer the question better.\n\
            OS: Windows\n\
            Shell: PowerShell";

        // Connect MCP servers
        let mcp_config = mcp::load_mcp_config()?;
        let mut manager = mcp::McpManager::connect_all(&mcp_config);

        let tools = if manager.has_tools() {
            Some(manager.tools_for_api())
        } else {
            None
        };

        eprintln!("\x1b[90m[kuku] 哈基米哈基米...让我想想...\x1b[0m");

        let rt = tokio::runtime::Runtime::new()?;

        let mut messages = vec![
            Message {
                role: "system".into(),
                content: Some(system_prompt.into()),
                tool_calls: None,
                tool_call_id: None,
            },
            Message {
                role: "user".into(),
                content: Some(question),
                tool_calls: None,
                tool_call_id: None,
            },
        ];

        // Tool call loop
        for _ in 0..MAX_TOOL_ROUNDS {
            let resp = rt
                .block_on(api::chat_with_tools(&config, messages.clone(), 2048, tools.clone()))
                .context("AI API call")?;

            if let Some(tool_calls) = &resp.tool_calls {
                if !tool_calls.is_empty() {
                    // Append assistant message with tool_calls
                    messages.push(resp.clone());

                    for tc in tool_calls {
                        eprintln!(
                            "\x1b[90m[kuku] 调用工具: {} ...\x1b[0m",
                            tc.function.name
                        );

                        let args: serde_json::Value =
                            serde_json::from_str(&tc.function.arguments).unwrap_or_default();

                        let result = manager
                            .execute_tool(&tc.function.name, args)
                            .unwrap_or_else(|e| format!("Error: {e}"));

                        messages.push(Message {
                            role: "tool".into(),
                            content: Some(result),
                            tool_calls: None,
                            tool_call_id: Some(tc.id.clone()),
                        });
                    }
                    continue; // Next round
                }
            }

            // No tool calls — final text response
            let text = resp.content.unwrap_or_default();
            println!("\n\x1b[1;35m[kuku]\x1b[0m 想到了！曼波！\n");
            println!("{}", text.trim());
            println!();
            return Ok(());
        }

        eprintln!("\x1b[33m[kuku] 工具调用轮次用完了...直接回答！\x1b[0m");
        // Final call without tools to force text response
        let resp = rt
            .block_on(api::chat_with_tools(&config, messages, 2048, None))
            .context("AI API call")?;
        let text = resp.content.unwrap_or_default();
        println!("\n\x1b[1;35m[kuku]\x1b[0m 想到了！曼波！\n");
        println!("{}", text.trim());
        println!();

        Ok(())
    }
}
