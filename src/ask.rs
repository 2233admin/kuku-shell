//! Free-form AI Q&A — ask anything without a failed command.

use crate::api;
use anyhow::Context;
use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct AskCommand {
    /// The question to ask AI
    #[arg(trailing_var_arg = true, required = true)]
    question: Vec<String>,
}

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
            OS: Windows\n\
            Shell: PowerShell";

        eprintln!("\x1b[90m[kuku] 哈基米哈基米...让我想想...\x1b[0m");

        let rt = tokio::runtime::Runtime::new()?;
        let result = rt
            .block_on(api::chat_with_tokens(&config, system_prompt, &question, 2048))
            .context("AI API call")?;

        println!("\n\x1b[1;35m[kuku]\x1b[0m 想到了！曼波！\n");
        println!("{}", result.trim());
        println!();

        Ok(())
    }
}
