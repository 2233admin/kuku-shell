//! Environment diagnostics for PowerShell + AI tool health.

use crate::mcp;
use crate::profile;
use std::io::Write;
use std::process::Command;

struct Check {
    name: &'static str,
    status: Status,
    detail: String,
    fix: Option<String>,
}

#[derive(Clone, Copy)]
enum Status {
    Ok,
    Warn,
    Fail,
    Info,
}

impl Status {
    fn icon(self) -> &'static str {
        match self {
            Self::Ok => "\x1b[32m✓\x1b[0m",
            Self::Warn => "\x1b[33m!\x1b[0m",
            Self::Fail => "\x1b[31mx\x1b[0m",
            Self::Info => "\x1b[36mi\x1b[0m",
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    println!("\x1b[1;35mKuku Doctor\x1b[0m  \x1b[90m闻闻~ 嗅嗅~ 让我检查一下！\x1b[0m\n");

    let checks = vec![
        check_powershell(),
        check_profile_integration(),
        check_assistant_config(),
        check_mcp_config(),
        check_tool("starship", "starship --version", "winget install Starship.Starship"),
        check_tool("delta", "delta --version", "winget install dandavison.delta"),
        check_tool("lazygit", "lazygit --version", "winget install jesseduffield.lazygit"),
        check_tool("yazi", "yazi --version", "winget install sxyazi.yazi"),
        check_tool("zoxide", "zoxide --version", "winget install ajeetdsouza.zoxide"),
    ];

    let mut ok = 0;
    let mut warn = 0;
    let mut fail = 0;

    for c in &checks {
        let icon = c.status.icon();
        println!("  {icon} {}: {}", c.name, c.detail);
        if let Some(fix) = &c.fix {
            println!("    \x1b[90mFix: {fix}\x1b[0m");
        }
        match c.status {
            Status::Ok => ok += 1,
            Status::Warn => warn += 1,
            Status::Fail => fail += 1,
            Status::Info => {}
        }
    }

    println!();
    if fail == 0 && warn == 0 {
        println!("  \x1b[32m曼波！全部没问题！你好棒！\x1b[0m");
    } else if fail == 0 {
        println!("  \x1b[33m基本OK~ 就 {warn} 个小事情啦\x1b[0m");
    } else {
        println!("  \x1b[31m呜呜呜 {fail} 个地方坏掉了...别担心我帮你！\x1b[0m");
    }
    println!("  {ok} ok  {warn} warn  {fail} fail");
    std::io::stdout().flush()?;
    Ok(())
}

fn check_powershell() -> Check {
    let version = Command::new("pwsh")
        .args(["--version"])
        .output()
        .or_else(|_| Command::new("powershell").args(["--version"]).output());

    match version {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Check {
                name: "PowerShell",
                status: if v.contains("7.") { Status::Ok } else { Status::Warn },
                detail: v.clone(),
                fix: if v.contains("7.") {
                    None
                } else {
                    Some("winget install Microsoft.PowerShell".into())
                },
            }
        }
        _ => Check {
            name: "PowerShell",
            status: Status::Fail,
            detail: "PowerShell not found".into(),
            fix: Some("winget install Microsoft.PowerShell".into()),
        },
    }
}

fn check_profile_integration() -> Check {
    let path = match profile::powershell_profile_path() {
        Some(p) => p,
        None => {
            return Check {
                name: "Profile Integration",
                status: Status::Warn,
                detail: "Cannot determine PowerShell profile path".into(),
                fix: Some("Run `kuku init`".into()),
            }
        }
    };

    if !path.exists() {
        return Check {
            name: "Profile Integration",
            status: Status::Warn,
            detail: format!("Profile not found: {}", path.display()),
            fix: Some("Run `kuku init`".into()),
        };
    }

    let content = std::fs::read_to_string(&path).unwrap_or_default();
    if content.contains(profile::PROFILE_MARKER) {
        Check {
            name: "Profile Integration",
            status: Status::Ok,
            detail: format!("Kuku block found in {}", path.display()),
            fix: None,
        }
    } else {
        Check {
            name: "Profile Integration",
            status: Status::Warn,
            detail: format!("No kuku block in {}", path.display()),
            fix: Some("Run `kuku init`".into()),
        }
    }
}

fn check_assistant_config() -> Check {
    let path = profile::assistant_toml_path();
    if !path.exists() {
        return Check {
            name: "AI Assistant Config",
            status: Status::Warn,
            detail: format!("Missing {}", path.display()),
            fix: Some("Run `kuku ai` to configure".into()),
        };
    }

    let raw = std::fs::read_to_string(&path).unwrap_or_default();
    let has_key = raw.lines().any(|l| {
        let t = l.trim();
        !t.starts_with('#') && t.starts_with("api_key") && t.contains('=')
    });

    if has_key {
        Check {
            name: "AI Assistant Config",
            status: Status::Ok,
            detail: "API key configured".into(),
            fix: None,
        }
    } else {
        Check {
            name: "AI Assistant Config",
            status: Status::Warn,
            detail: "API key not set".into(),
            fix: Some("Run `kuku ai` to set API key".into()),
        }
    }
}

fn check_mcp_config() -> Check {
    match mcp::load_mcp_config() {
        Ok(config) => {
            let count = config.servers.len();
            if count == 0 {
                Check {
                    name: "MCP Servers",
                    status: Status::Info,
                    detail: "没配置 MCP server（可选）".into(),
                    fix: Some("复制 mcp.example.toml 到 ~/.config/kuku/mcp.toml".into()),
                }
            } else {
                let names: Vec<&String> = config.servers.keys().collect();
                Check {
                    name: "MCP Servers",
                    status: Status::Ok,
                    detail: format!("{count} 个 server: {}", names.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")),
                    fix: None,
                }
            }
        }
        Err(e) => Check {
            name: "MCP Servers",
            status: Status::Warn,
            detail: format!("配置解析失败: {e}"),
            fix: Some("检查 ~/.config/kuku/mcp.toml 格式".into()),
        },
    }
}

fn check_tool(name: &'static str, version_cmd: &str, install_cmd: &str) -> Check {
    let parts: Vec<&str> = version_cmd.split_whitespace().collect();
    let result = Command::new(parts[0]).args(&parts[1..]).output();

    match result {
        Ok(out) if out.status.success() => {
            let v = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            Check {
                name,
                status: Status::Ok,
                detail: if v.is_empty() { "installed".into() } else { v },
                fix: None,
            }
        }
        _ => Check {
            name,
            status: Status::Info,
            detail: "not installed (optional)".into(),
            fix: Some(install_cmd.into()),
        },
    }
}
