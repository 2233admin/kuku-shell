//! MCP (Model Context Protocol) client — stdio transport, JSON-RPC 2.0.

use anyhow::{bail, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::profile;

// ── Config ──────────────────────────────────────────────────────

#[derive(Deserialize, Clone, Debug)]
pub struct McpConfig {
    #[serde(default)]
    pub servers: HashMap<String, McpServerConfig>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

pub fn load_mcp_config() -> anyhow::Result<McpConfig> {
    let path = profile::config_dir().join("mcp.toml");
    if !path.exists() {
        return Ok(McpConfig {
            servers: HashMap::new(),
        });
    }
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("read {}", path.display()))?;
    let config: McpConfig = toml::from_str(&raw)
        .with_context(|| format!("parse {}", path.display()))?;
    Ok(config)
}

// ── JSON-RPC ────────────────────────────────────────────────────

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    id: Option<u64>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

#[derive(Deserialize, Debug)]
struct JsonRpcError {
    #[allow(dead_code)]
    code: i64,
    message: String,
}

// ── Tool schema ─────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, rename = "inputSchema")]
    pub input_schema: Option<Value>,
}

// ── MCP Client ──────────────────────────────────────────────────

pub struct McpClient {
    child: Child,
    next_id: AtomicU64,
}

impl McpClient {
    pub fn connect(config: &McpServerConfig) -> anyhow::Result<Self> {
        let mut cmd = Command::new(&config.command);
        cmd.args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        for (k, v) in &config.env {
            cmd.env(k, v);
        }

        let child = cmd.spawn()
            .with_context(|| format!("spawn MCP server: {}", config.command))?;

        let mut client = Self {
            child,
            next_id: AtomicU64::new(1),
        };

        // Initialize handshake
        client.call(
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "kuku",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
        )?;

        // Send initialized notification (no id)
        client.notify("notifications/initialized", None)?;

        Ok(client)
    }

    pub fn list_tools(&mut self) -> anyhow::Result<Vec<McpTool>> {
        let result = self.call("tools/list", None)?;
        let tools_val = result
            .get("tools")
            .cloned()
            .unwrap_or(Value::Array(vec![]));
        let tools: Vec<McpTool> = serde_json::from_value(tools_val)
            .context("parse tools/list")?;
        Ok(tools)
    }

    pub fn call_tool(&mut self, name: &str, arguments: Value) -> anyhow::Result<String> {
        let result = self.call(
            "tools/call",
            Some(json!({
                "name": name,
                "arguments": arguments
            })),
        )?;

        // Extract text from content array
        if let Some(content) = result.get("content").and_then(|c| c.as_array()) {
            let texts: Vec<&str> = content
                .iter()
                .filter_map(|item| {
                    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                        item.get("text").and_then(|t| t.as_str())
                    } else {
                        None
                    }
                })
                .collect();
            Ok(texts.join("\n"))
        } else {
            Ok(result.to_string())
        }
    }

    fn call(&mut self, method: &str, params: Option<Value>) -> anyhow::Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id,
            method: method.to_string(),
            params,
        };

        let stdin = self.child.stdin.as_mut().context("no stdin")?;
        let msg = serde_json::to_string(&request)?;
        writeln!(stdin, "{msg}")?;
        stdin.flush()?;

        let stdout = self.child.stdout.as_mut().context("no stdout")?;
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line)?;

        let resp: JsonRpcResponse = serde_json::from_str(line.trim())
            .with_context(|| format!("parse response for {method}"))?;

        if let Some(err) = resp.error {
            bail!("MCP error: {}", err.message);
        }

        Ok(resp.result.unwrap_or(Value::Null))
    }

    fn notify(&mut self, method: &str, params: Option<Value>) -> anyhow::Result<()> {
        let msg = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params.unwrap_or(Value::Object(Default::default()))
        });
        let stdin = self.child.stdin.as_mut().context("no stdin")?;
        let s = serde_json::to_string(&msg)?;
        writeln!(stdin, "{s}")?;
        stdin.flush()?;
        Ok(())
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

// ── Multi-server manager ────────────────────────────────────────

pub struct McpManager {
    clients: Vec<(String, McpClient)>,
    tools: Vec<(String, McpTool)>, // (server_name, tool)
}

impl McpManager {
    pub fn connect_all(config: &McpConfig) -> Self {
        let mut clients = Vec::new();
        let mut tools = Vec::new();

        for (name, server_config) in &config.servers {
            match McpClient::connect(server_config) {
                Ok(mut client) => {
                    match client.list_tools() {
                        Ok(server_tools) => {
                            for tool in &server_tools {
                                tools.push((name.clone(), tool.clone()));
                            }
                            eprintln!(
                                "\x1b[90m[kuku] MCP {name}: {} tools\x1b[0m",
                                server_tools.len()
                            );
                            clients.push((name.clone(), client));
                        }
                        Err(e) => {
                            eprintln!("\x1b[90m[kuku] MCP {name} tools/list 失败: {e}\x1b[0m");
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\x1b[90m[kuku] MCP {name} 连不上: {e}\x1b[0m");
                }
            }
        }

        Self { clients, tools }
    }

    pub fn has_tools(&self) -> bool {
        !self.tools.is_empty()
    }

    /// Build OpenAI-compatible tools array for chat API
    pub fn tools_for_api(&self) -> Vec<Value> {
        self.tools
            .iter()
            .map(|(server, tool)| {
                json!({
                    "type": "function",
                    "function": {
                        "name": format!("{}_{}", server, tool.name),
                        "description": tool.description.as_deref().unwrap_or(""),
                        "parameters": tool.input_schema.clone().unwrap_or(json!({"type": "object", "properties": {}}))
                    }
                })
            })
            .collect()
    }

    /// Execute a tool call, returns result text
    pub fn execute_tool(&mut self, full_name: &str, arguments: Value) -> anyhow::Result<String> {
        // full_name is "servername_toolname"
        let (server_name, tool_name) = full_name
            .split_once('_')
            .context("invalid tool name format")?;

        let client = self
            .clients
            .iter_mut()
            .find(|(name, _)| name == server_name)
            .map(|(_, c)| c)
            .with_context(|| format!("MCP server not found: {server_name}"))?;

        client.call_tool(tool_name, arguments)
    }
}
