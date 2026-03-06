//! AI-powered command error analysis and fix suggestions.

use crate::api;
use anyhow::Context;
use clap::Parser;

#[derive(Debug, Parser, Clone)]
pub struct AssistCommand {
    /// The failed command string
    #[arg(long)]
    command: String,

    /// The exit code of the failed command
    #[arg(long)]
    exit_code: i32,

    /// Additional stderr output (optional)
    #[arg(long)]
    stderr: Option<String>,
}

impl AssistCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        let config = api::load_config()?;

        if config.enabled == Some(false) {
            return Ok(());
        }

        if config.api_key.as_deref().filter(|k| !k.is_empty()).is_none() {
            eprintln!("\x1b[33m[kaku] No API key configured. Run `kaku ai` to set up.\x1b[0m");
            return Ok(());
        }

        let system_prompt = "\
            You are a PowerShell command assistant. A command failed and you need to analyze why and suggest a fix.\n\
            Respond with ONLY the corrected command on the first line, then a blank line, then a brief explanation.\n\
            If you cannot determine a fix, say so briefly.\n\
            OS: Windows\n\
            Shell: PowerShell";

        let user_msg = format!(
            "Failed command: {}\nExit code: {}\n{}",
            self.command,
            self.exit_code,
            self.stderr
                .as_deref()
                .map(|s| format!("stderr:\n{s}"))
                .unwrap_or_default()
        );

        let rt = tokio::runtime::Runtime::new()?;
        let result = rt
            .block_on(api::chat(&config, system_prompt, &user_msg))
            .context("AI API call")?;

        let mut lines = result.lines();
        let suggestion = lines.next().unwrap_or("").trim();

        if !suggestion.is_empty() {
            let temp = std::env::temp_dir().join("kaku_last_suggestion.txt");
            let _ = std::fs::write(&temp, suggestion);

            println!("\x1b[1;35m[kaku]\x1b[0m Suggested fix:");
            println!("  \x1b[1;32m{suggestion}\x1b[0m");

            let explanation: String = lines.collect::<Vec<_>>().join("\n");
            let explanation = explanation.trim();
            if !explanation.is_empty() {
                println!("  \x1b[90m{explanation}\x1b[0m");
            }
            println!("  \x1b[90mPress Ctrl+Shift+E to apply\x1b[0m");
        }

        Ok(())
    }
}
