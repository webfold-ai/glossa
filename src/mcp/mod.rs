//! Model Context Protocol (MCP) Server for AI Coding Assistants.
//!
//! Exposes localization tools to Claude Code, Cursor, Windsurf, Copilot, and custom agents.

use crate::config::LingoConfig;
use crate::emitters::{emit_trans_call, TargetFramework};
use crate::engine::providers::offline::OfflineProvider;
use crate::engine::TranslationEngine;
use crate::ir::term::ApeironTerm;
use crate::lockfile::LingoLock;
use crate::parsers::{universal_lift, SourceFormat};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
struct McpRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Serialize, Deserialize)]
struct McpResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

/// Run the MCP server listening on stdio.
pub async fn run_mcp_server() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let engine = TranslationEngine::new(Arc::new(OfflineProvider::new()));

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(req) = serde_json::from_str::<McpRequest>(&line) {
            let res = handle_mcp_request(&req, &engine).await;
            let output = serde_json::to_string(&res).unwrap_or_default();
            writeln!(stdout, "{output}")?;
            stdout.flush()?;
        }
    }

    Ok(())
}

async fn handle_mcp_request(req: &McpRequest, engine: &TranslationEngine) -> McpResponse {
    match req.method.as_str() {
        "initialize" => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "glossa-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        },
        "tools/list" => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: Some(json!({
                "tools": [
                    {
                        "name": "extract_strings",
                        "description": "Extract translatable strings and slots from code snippet or file across any framework (React, Vue, Svelte, HTML, Markdown, JSON)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "code": { "type": "string", "description": "Source code" },
                                "format": { "type": "string", "description": "Format: jsx, vue, svelte, html, md, json" },
                                "file_path": { "type": "string", "description": "Optional file path context" }
                            },
                            "required": ["code", "format"]
                        }
                    },
                    {
                        "name": "translate_string",
                        "description": "Translate a string to a target locale, strictly preserving placeholders and XML/HTML tags through the Apeiron Null-Gate",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string", "description": "Canonical text with {slots} or <0>tags</0>" },
                                "target_locale": { "type": "string", "description": "Target locale code (e.g. es, fr, de, ja)" }
                            },
                            "required": ["text", "target_locale"]
                        }
                    },
                    {
                        "name": "compile_component",
                        "description": "Compile a component template into localized framework calls (React Trans/t, Vue i18n-t/$t, Svelte $t)",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "key": { "type": "string", "description": "Translation key" },
                                "text": { "type": "string", "description": "Source text or canonical term" },
                                "target_framework": { "type": "string", "description": "react, vue, svelte, or solid" }
                            },
                            "required": ["key", "text", "target_framework"]
                        }
                    },
                    {
                        "name": "get_status",
                        "description": "Get current localization coverage and lockfile status for the workspace",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "dir": { "type": "string", "description": "Workspace root directory" }
                            }
                        }
                    }
                ]
            })),
            error: None,
        },
        "tools/call" => {
            let params = req.params.as_ref();
            let name = params.and_then(|p| p.get("name")).and_then(|v| v.as_str()).unwrap_or_default();
            let args = params.and_then(|p| p.get("arguments")).cloned().unwrap_or(json!({}));

            let result = handle_tool_call(name, &args, engine).await;
            match result {
                Ok(val) => McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: Some(json!({ "content": [{ "type": "text", "text": val }] })),
                    error: None,
                },
                Err(err) => McpResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id.clone(),
                    result: None,
                    error: Some(json!({ "code": -32603, "message": err })),
                },
            }
        }
        _ => McpResponse {
            jsonrpc: "2.0".to_string(),
            id: req.id.clone(),
            result: None,
            error: Some(json!({ "code": -32601, "message": "Method not found" })),
        },
    }
}

async fn handle_tool_call(name: &str, args: &Value, engine: &TranslationEngine) -> Result<String, String> {
    match name {
        "extract_strings" => {
            let code = args.get("code").and_then(|v| v.as_str()).ok_or("Missing 'code'")?;
            let fmt_str = args.get("format").and_then(|v| v.as_str()).unwrap_or("jsx");
            let file_path = args.get("file_path").and_then(|v| v.as_str()).unwrap_or("inline");

            let format = match fmt_str {
                "jsx" | "tsx" => SourceFormat::Jsx,
                "vue" => SourceFormat::Vue,
                "svelte" => SourceFormat::Svelte,
                "html" => SourceFormat::Html,
                "md" | "markdown" => SourceFormat::Markdown,
                "json" => SourceFormat::Json,
                _ => SourceFormat::Jsx,
            };

            let items = universal_lift(code, format, file_path);
            let summary: Vec<Value> = items
                .into_iter()
                .map(|it| {
                    json!({
                        "coord": it.coord.to_hex(),
                        "canonical_text": it.canonical_text,
                        "slots": ApeironTerm::collect_slots(&it.terms),
                        "line": it.locations.first().map(|l| l.line).unwrap_or(1)
                    })
                })
                .collect();

            Ok(serde_json::to_string_pretty(&summary).unwrap_or_default())
        }
        "translate_string" => {
            let text = args.get("text").and_then(|v| v.as_str()).ok_or("Missing 'text'")?;
            let target_locale = args.get("target_locale").and_then(|v| v.as_str()).ok_or("Missing 'target_locale'")?;

            let res = engine
                .translate_and_verify(&[text.to_string()], target_locale)
                .await
                .map_err(|e| e.to_string())?;

            let translated = res.first().cloned().unwrap_or_default();
            Ok(json!({ "translated": translated }).to_string())
        }
        "compile_component" => {
            let key = args.get("key").and_then(|v| v.as_str()).ok_or("Missing 'key'")?;
            let text = args.get("text").and_then(|v| v.as_str()).ok_or("Missing 'text'")?;
            let framework_str = args.get("target_framework").and_then(|v| v.as_str()).unwrap_or("react");

            let target = TargetFramework::parse(framework_str).unwrap_or(TargetFramework::React);
            let parsed_terms = crate::parsers::resource::parse_interpolated_string(text);
            let compiled = emit_trans_call(key, &parsed_terms, target);

            Ok(json!({ "compiled": compiled }).to_string())
        }
        "get_status" => {
            let dir_str = args.get("dir").and_then(|v| v.as_str()).unwrap_or(".");
            let dir = Path::new(dir_str);
            let config = LingoConfig::load_or_default(dir);
            let lock_path = LingoLock::resolve_path(dir);
            let lock = LingoLock::load_from_file(&lock_path).unwrap_or_default();
            let deltas = lock.compute_deltas(&config.target_locales);

            Ok(json!({
                "total_units": lock.entries.len(),
                "target_locales": config.target_locales,
                "pending_translations": deltas.len(),
                "provider": config.provider,
                "lockfile_version": lock.version
            }).to_string())
        }
        _ => Err(format!("Unknown tool: {name}")),
    }
}
