# Kaku Shell

A Rust CLI that turns PowerShell into an AI coding terminal.

Failed command? Kaku auto-analyzes it with LLM, suggests a fix, and lets you apply it with Ctrl+Shift+E.

## Features

- **Auto Error Analysis** - Commands that fail are automatically sent to LLM for diagnosis
- **Fix Suggestions** - Corrected command appears inline, press Ctrl+Shift+E to apply
- **AI Config TUI** - Full terminal UI to manage model, API key, and connection
- **Environment Doctor** - Diagnose PowerShell version, tools, and config health
- **Non-invasive** - Injects a tiny managed block into `$PROFILE`, preserves your existing setup
- **Zero dependencies** - Single static binary, no runtime needed

## Quick Start

```powershell
# Build from source
cargo build --release --target x86_64-pc-windows-gnu

# Initialize PowerShell integration
kaku init

# Configure AI (interactive TUI)
kaku ai

# Check environment health
kaku doctor
```

## Commands

| Command | Description |
|---------|-------------|
| `kaku` | Interactive main menu |
| `kaku ai` | AI assistant config TUI (model, key, test connection) |
| `kaku assist` | Analyze failed command (called automatically by shell hook) |
| `kaku doctor` | Diagnose environment and tool health |
| `kaku init` | Inject integration into PowerShell `$PROFILE` |
| `kaku config` | Edit `~/.config/kaku/kaku.toml` |
| `kaku reset` | Remove integration from `$PROFILE` |

## How It Works

1. `kaku init` adds a managed block to your PowerShell profile
2. When a command fails (non-zero exit code), the shell hook calls `kaku assist`
3. `kaku assist` sends the failed command + exit code to a configured LLM API
4. The suggested fix is displayed and saved to a temp file
5. Press **Ctrl+Shift+E** to insert the suggestion into your command line

## Configuration

Config files live in `~/.config/kaku/`:

- `assistant.toml` - AI model, API key, base URL
- `kaku.toml` - General settings

### Supported API Providers

Any OpenAI-compatible `/v1/chat/completions` endpoint works:

- Volcengine (Doubao-Seed-2.0-Code, Doubao-Seed-2.0-pro)
- DeepSeek (DeepSeek-V3.2)
- MiniMax (MiniMax-M2.5)
- Zhipu (GLM-4.7)
- OpenAI, Anthropic (via proxy), or any compatible gateway

## Tech Stack

- **Rust** - Single static binary (~3MB release)
- **ratatui** + **crossterm** - Terminal UI
- **reqwest** + **tokio** - Async HTTP client
- **clap** - CLI argument parsing
- **toml** / **serde** - Configuration

## License

MIT
