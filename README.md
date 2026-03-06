# Kuku

```
       /\_/\
      ( ˶°ω°˶ )   Kuku
      (/    \)    你的 PowerShell AI 小助手~ 曼波曼波~
      /|    |\
     (_|    |_)
```

命令打错了？Kuku 自动帮你分析报错、给出修复建议，Ctrl+Shift+E 一键应用。

## Quick Start

```powershell
# 初始化 PowerShell 集成
kuku init

# 设置 AI（交互式 TUI）
kuku ai

# 自由提问
kuku ask "怎么用 PowerShell 批量重命名文件"

# 环境体检
kuku doctor
```

## Commands

| Command | Description |
|---------|-------------|
| `kuku` | 交互式主菜单 |
| `kuku ai` | AI 助手设置 TUI |
| `kuku ask` | 自由提问 |
| `kuku assist` | 分析失败命令（shell hook 自动调用） |
| `kuku doctor` | 环境健康诊断 |
| `kuku init` | 注入 PowerShell 集成 |
| `kuku config` | 编辑 `~/.config/kuku/kuku.toml` |
| `kuku reset` | 移除 PowerShell 集成 |

## How It Works

1. `kuku init` 往你的 PowerShell Profile 里加一段代码
2. 命令失败时，shell hook 自动调用 `kuku assist`
3. Kuku 把失败命令发给 AI，拿回修复建议
4. 按 **Ctrl+Shift+E** 把建议填进命令行

## Configuration

配置文件在 `~/.config/kuku/`：

- `assistant.toml` - AI 模型、API key、base URL
- `kuku.toml` - 通用设置

### Supported API Providers

任何 OpenAI 兼容的 `/v1/chat/completions` 接口都行：

- Volcengine (Doubao-Seed-2.0-Code, Doubao-Seed-2.0-pro)
- DeepSeek (DeepSeek-V3.2)
- MiniMax (MiniMax-M2.5)
- Zhipu (GLM-4.7)
- OpenAI, Anthropic (via proxy), or any compatible gateway

## Tech Stack

- **Rust** - Single static binary (~2.5MB)
- **ratatui** + **crossterm** - Terminal UI
- **reqwest** + **tokio** - Async HTTP
- **clap** - CLI parsing
- **toml** / **serde** - Config

## License

MIT
