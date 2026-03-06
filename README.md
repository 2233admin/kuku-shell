<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&customColorList=12,14,16,18,20&height=180&section=header&text=&fontSize=1" />
</p>

<h1 align="center">
  <br>
  <code>( ˶°ω°˶ )</code>
  <br>
  Kuku Shell
  <br>
</h1>

<h3 align="center">
  命令翻车了？别慌。<br>
  Kuku 2 秒给你修好。曼波~
</h3>

<p align="center">
  <strong>PowerShell 命令写错 → AI 自动分析 → 一键修复</strong><br>
  <sub>2.5MB 单文件 · 零依赖 · 比你打开浏览器搜索快 10 倍</sub>
</p>

<br>

<p align="center">
  <a href="https://github.com/2233admin/kuku/actions"><img src="https://img.shields.io/github/actions/workflow/status/2233admin/kuku/ci.yml?style=flat-square&logo=github&label=build&color=8B5CF6" /></a>
  <a href="https://github.com/2233admin/kuku/releases"><img src="https://img.shields.io/github/v/release/2233admin/kuku?style=flat-square&color=EC4899&label=release" /></a>
  <img src="https://img.shields.io/badge/binary-2.5MB-10B981?style=flat-square" />
  <img src="https://img.shields.io/badge/platform-Windows-0EA5E9?style=flat-square&logo=windows" />
  <img src="https://img.shields.io/badge/lang-Rust-F97316?style=flat-square&logo=rust" />
  <img src="https://img.shields.io/badge/vibe-曼波曼波-FF6B9D?style=flat-square" />
</p>

<br>

<!-- TODO: 录一段终端 GIF 放这里，效果直接炸裂 -->
<!-- <p align="center"><img src="./assets/demo.gif" width="700" /></p> -->

<p align="center">
  <sub>⬆️ GIF demo coming soon — 命令翻车 → Kuku 修复 → Ctrl+Shift+E 应用</sub>
</p>

<br>

## 为什么用 Kuku？

> **你现在**：命令报错 → 复制错误信息 → 打开浏览器 → 搜索 → 翻 Stack Overflow → 复制答案 → 粘贴回来
>
> **用 Kuku**：命令报错 → Kuku 弹出修复建议 → 按 `Ctrl+Shift+E` → 搞定
>
> **省掉的时间**：30 秒 → 2 秒。每天省 50 次，一年省回一个假期。

<br>

## 30 秒上手

```powershell
# 1. 让 Kuku 钻进你的 PowerShell
kuku init

# 2. 填上 API key
kuku ai

# 3. 没了。故意打错一条命令试试
gti status
# [kuku] 啊啊啊报错了！等等我看看！
#   git status
#   Ctrl+Shift+E 我帮你填上去！曼波~
```

<br>

## ✨ 能干嘛

<table>
<tr>
<td width="50%" valign="top">

### 🔥 自动修命令
打错命令？Kuku 自动拦截、分析、给修复建议。你只管按 `Ctrl+Shift+E`。

### 💬 随便问
```
kuku ask "PowerShell 怎么批量改文件名"
```
不用等命令出错也能问。2048 token，回答够长。

### 🩺 环境体检
```
kuku doctor
```
闻闻嗅嗅你的环境，PowerShell 版本、工具链、API 配置一目了然。

</td>
<td width="50%" valign="top">

### 🎨 设置面板
```
kuku ai
```
全键盘操作的 TUI，切模型、填密钥、测连接，不用手改配置文件。

### 🪶 就一个文件
2.5MB。不装 Python，不装 Node，不装运行时。丢进 PATH 就能用。

### 🐱 有性格
不是冷冰冰的工具。Kuku 会说"啊啊啊报错了！"、"想到了！曼波！"、"呜呜...要赶走我吗"。

</td>
</tr>
</table>

<br>

## 📋 所有命令

| 命令 | 干嘛的 |
|:-----|:-------|
| `kuku` | 打开主菜单，曼波~ |
| `kuku ask <问题>` | 自由提问 |
| `kuku ai` | AI 设置面板 (TUI) |
| `kuku doctor` | 环境体检 |
| `kuku init` | 装进 PowerShell |
| `kuku reset` | 拆掉（呜呜...想我了就 `kuku init` 叫我回来） |
| `kuku config` | 打开配置文件 |
| `kuku assist` | 分析失败命令（自动调用，你不用管） |

<br>

## 🔧 原理

```
  你：gti status          PowerShell：报错！        Kuku：让我看看！
 ┌──────────────┐       ┌──────────────┐       ┌──────────────┐
 │   gti status │──────>│  exit code 1 │──────>│  kuku assist │
 └──────────────┘       └──────────────┘       └──────┬───────┘
                                                      │ 问 AI
                                                      v
                    你按 Ctrl+Shift+E           ┌──────────────┐
                   ┌──────────────┐             │  建议:       │
                   │  git status  │<────────────│  git status  │
                   │  (自动填入)   │             └──────────────┘
                   └──────────────┘
```

**就四步：**
1. `kuku init` — Kuku 往你的 `$PROFILE` 里加一小段 hook
2. 你正常用 PowerShell，命令失败时 hook 自动触发
3. Kuku 把报错发给 AI，拿回修复建议
4. `Ctrl+Shift+E` 一键填进命令行

**不侵入**：不改你的 Prompt、不影响现有配置、`kuku reset` 一键还原。

<br>

## 🌐 支持的 AI

任何兼容 OpenAI `/v1/chat/completions` 的接口：

| 厂商 | 推荐模型 | 说明 |
|:-----|:---------|:-----|
| **Volcengine** | `Doubao-Seed-2.0-Code` | 默认，字节跳动，写代码专用 |
| **DeepSeek** | `DeepSeek-V3.2` | 便宜好用 |
| **MiniMax** | `MiniMax-M2.5` | 白嫖额度多 |
| **Zhipu** | `GLM-4.7` | 智谱 |
| **OpenAI** | `gpt-4o` | 通过官方或代理 |
| **任意兼容网关** | — | 只要是 `/v1/chat/completions` 就行 |

<br>

## 🏗️ 技术栈

<p>
  <img src="https://img.shields.io/badge/Rust-000?style=for-the-badge&logo=rust&logoColor=white" />
  <img src="https://img.shields.io/badge/ratatui-TUI-8B5CF6?style=for-the-badge" />
  <img src="https://img.shields.io/badge/tokio-async-F97316?style=for-the-badge" />
  <img src="https://img.shields.io/badge/clap-CLI-10B981?style=for-the-badge" />
  <img src="https://img.shields.io/badge/reqwest-HTTP-0EA5E9?style=for-the-badge" />
</p>

| 组件 | 干嘛的 |
|:-----|:-------|
| Rust | 编译成 2.5MB 单文件，启动 <10ms |
| ratatui + crossterm | TUI 界面（`kuku ai` 设置面板） |
| reqwest + tokio | 异步调 AI API |
| clap | 命令行解析 |
| toml + serde | 读写配置 |

<br>

## 📁 配置

```
~/.config/kuku/
├── assistant.toml    # API key、模型、base URL
└── kuku.toml         # 通用设置
```

<details>
<summary><b>assistant.toml 长这样</b></summary>

```toml
enabled = true
api_key = "your-api-key"
model = "Doubao-Seed-2.0-Code"
base_url = "https://ark.cn-beijing.volces.com/api/coding/v1"
```

</details>

<br>

## 🤝 Contributing

欢迎 PR！不管是修 bug、加功能、还是让 Kuku 说更多可爱的话。

<br>

## 📜 License

MIT — 随便用，Kuku 不介意~

<p align="center">
  <br>
  <code>( ˶°ω°˶ ) 曼波~ 用得开心的话给个 ⭐ 吧！</code>
  <br><br>
</p>

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=waving&color=gradient&customColorList=12,14,16,18,20&height=100&section=footer" />
</p>
