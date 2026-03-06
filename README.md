<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&customColorList=12,14,16,18,20&height=200&section=header&text=Kuku%20Shell&fontSize=60&fontColor=fff&animation=fadeIn&fontAlignY=35&desc=%E4%BD%A0%E7%9A%84%20PowerShell%20AI%20%E5%B0%8F%E5%8A%A9%E6%89%8B~%20%E6%9B%BC%E6%B3%A2%E6%9B%BC%E6%B3%A2~&descSize=16&descAlignY=55" />
</p>

<p align="center">
  <a href="https://github.com/2233admin/kuku/actions"><img src="https://img.shields.io/github/actions/workflow/status/2233admin/kuku/ci.yml?style=flat-square&logo=github&label=build&color=8B5CF6" /></a>
  <a href="https://github.com/2233admin/kuku/releases"><img src="https://img.shields.io/github/v/release/2233admin/kuku?style=flat-square&color=EC4899&label=release" /></a>
  <img src="https://img.shields.io/badge/platform-Windows-0EA5E9?style=flat-square&logo=windows" />
  <img src="https://img.shields.io/badge/lang-Rust-F97316?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/github/license/2233admin/kuku?style=flat-square&color=10B981" />
  <img src="https://img.shields.io/badge/vibe-曼波曼波-FF6B9D?style=flat-square" />
</p>

<br>

<div align="center">

```
       /\_/\
      ( ˶°ω°˶ )    命令翻车了？
      (/    \)     Kuku 自动分析、给出修复
      /|    |\     Ctrl+Shift+E 一键应用
     (_|    |_)    曼波~
```

</div>

<br>

## ✨ Features

<table>
<tr>
<td width="50%">

### 🐱 AI 错误分析
命令失败？Kuku 自动闻闻出了什么问题，给你建议修复命令

### 🎯 一键应用
按 `Ctrl+Shift+E` 把建议填进命令行，不用自己打字

### 💬 自由提问
`kuku ask "任何问题"` 直接问 AI，不用等命令出错

</td>
<td width="50%">

### 🎨 TUI 配置面板
`kuku ai` 打开交互式设置界面，模型、密钥、连接测试一站搞定

### 🩺 环境体检
`kuku doctor` 帮你闻闻嗅嗅，检查 PowerShell、工具链、API 配置

### 🪶 零依赖
单个 ~2.5MB 静态二进制，不需要任何运行时

</td>
</tr>
</table>

<br>

## 🚀 Quick Start

```powershell
# 让 Kuku 钻进你的 PowerShell
kuku init

# 设置 AI（交互式 TUI）
kuku ai

# 自由提问
kuku ask "怎么用 PowerShell 批量重命名文件"

# 环境体检
kuku doctor
```

<br>

## 📋 Commands

| Command | What it does |
|:--------|:-------------|
| `kuku` | 交互式主菜单 · 曼波~ |
| `kuku ai` | AI 助手设置 TUI |
| `kuku ask <问题>` | 自由提问，啥都能问 |
| `kuku assist` | 分析失败命令（shell hook 自动调用） |
| `kuku doctor` | 闻闻嗅嗅环境体检 |
| `kuku init` | 注入 PowerShell 集成 |
| `kuku config` | 编辑 `~/.config/kuku/kuku.toml` |
| `kuku reset` | 移除集成（呜呜...） |

<br>

## 🔧 How It Works

```
  你打了一条命令        命令炸了 💥        Kuku 自动分析
 ┌──────────────┐    ┌──────────────┐    ┌──────────────┐
 │ git psh      │───>│ exit code 1  │───>│ kuku assist  │
 └──────────────┘    └──────────────┘    └──────┬───────┘
                                                │
                      Ctrl+Shift+E              v
                     ┌──────────────┐    ┌──────────────┐
                     │ git push     │<───│ 建议: git    │
                     │ (已填入)      │    │ push         │
                     └──────────────┘    └──────────────┘
```

1. `kuku init` 往你的 PowerShell Profile 里加一段代码
2. 命令失败时，shell hook 自动调用 `kuku assist`
3. Kuku 把失败命令发给 AI，拿回修复建议
4. 按 **Ctrl+Shift+E** 把建议填进命令行

<br>

## 🌐 Supported API Providers

任何 OpenAI 兼容的 `/v1/chat/completions` 接口都行：

| Provider | Models |
|:---------|:-------|
| Volcengine | `Doubao-Seed-2.0-Code` · `Doubao-Seed-2.0-pro` |
| DeepSeek | `DeepSeek-V3.2` |
| MiniMax | `MiniMax-M2.5` |
| Zhipu | `GLM-4.7` |
| OpenAI / Anthropic (proxy) | Any compatible model |

<br>

## 🏗️ Tech Stack

<p>
  <img src="https://img.shields.io/badge/Rust-000?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/ratatui-TUI-8B5CF6?style=for-the-badge" />
  <img src="https://img.shields.io/badge/tokio-async-F97316?style=for-the-badge" />
  <img src="https://img.shields.io/badge/clap-CLI-10B981?style=for-the-badge" />
  <img src="https://img.shields.io/badge/reqwest-HTTP-0EA5E9?style=for-the-badge" />
</p>

- **Rust** — 单个静态二进制 ~2.5MB
- **ratatui** + **crossterm** — Terminal UI
- **reqwest** + **tokio** — 异步 HTTP
- **clap** — CLI 解析
- **toml** / **serde** — 配置

<br>

## 📁 Configuration

配置文件在 `~/.config/kuku/`：

```
~/.config/kuku/
├── assistant.toml    # AI 模型、API key、base URL
└── kuku.toml         # 通用设置
```

<br>

## 📜 License

MIT

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&customColorList=12,14,16,18,20&height=100&section=footer" />
</p>
