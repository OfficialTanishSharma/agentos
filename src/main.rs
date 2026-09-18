mod db;

use anyhow::{bail, Context, Result};
use db::Database;
use serde::Deserialize;
use serde_json::{json, Value};
use std::{
    env,
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Deserialize)]
struct SearchArgs {
    query: String,
    #[serde(default = "default_limit")]
    limit: u32,
}

#[derive(Debug, Deserialize)]
struct RememberArgs {
    title: String,
    body: String,
    #[serde(default)]
    tags: Vec<String>,
}

fn default_limit() -> u32 {
    10
}

fn main() -> Result<()> {
    let db_path = parse_db_path()?;
    let project = canonical_project()?;
    let db = Database::open(&db_path)?;

    eprintln!(
        "AgentOS MCP server started: db={}, project={}",
        db_path.display(),
        project
    );

    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();

    for line in stdin.lock().lines() {
        let line = line.context("failed to read MCP request from stdin")?;
        if line.trim().is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
            Ok(value) => value,
            Err(error) => {
                write_message(
                    &mut stdout,
                    &json!({
                        "jsonrpc": "2.0",
                        "id": null,
                        "error": {"code": -32700, "message": error.to_string()}
                    }),
                )?;
                continue;
            }
        };

        // Notifications do not have an id and must not receive a response.
        if request.get("id").is_none() {
            continue;
        }

        let response = handle_request(&db, &project, &request);
        write_message(&mut stdout, &response)?;
    }

    Ok(())
}

fn handle_request(db: &Database, default_project: &str, request: &Value) -> Value {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {"tools": {"listChanged": false}},
            "serverInfo": {"name": "agentos", "version": env!("CARGO_PKG_VERSION")}
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(tool_list()),
        "tools/call" => call_tool(db, default_project, request.get("params")),
        _ => return error_response(id, -32601, format!("method not found: {method}")),
    };

    match result {
        Ok(value) => json!({"jsonrpc": "2.0", "id": id, "result": value}),
        Err(error) => error_response(id, -32603, error.to_string()),
    }
}

fn call_tool(db: &Database, default_project: &str, params: Option<&Value>) -> Result<Value> {
    let params = params.context("missing tools/call params")?;
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .context("missing tool name")?;
    let arguments = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

    let tool_result = match name {
        "memory.search" => {
            let args: SearchArgs = serde_json::from_value(arguments)
                .context("invalid memory.search arguments")?;
            if args.query.trim().is_empty() {
                bail!("query cannot be empty");
            }
            let limit = args.limit.clamp(1, 50);
            let project = default_project;
            let memories = db.search(project, &args.query, limit)?;
            let text = if memories.is_empty() {
                "No matching memories found.".to_owned()
            } else {
                memories
                    .iter()
                    .enumerate()
                    .map(|(index, memory)| {
                        format!(
                            "{}. {}\nID: {}\nTags: {}\nCreated: {}\n{}",
                            index + 1,
                            memory.title,
                            memory.id,
                            if memory.tags.is_empty() {
                                "-".to_owned()
                            } else {
                                memory.tags.join(", ")
                            },
                            memory.created_at,
                            memory.body
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n\n")
            };

            json!({
                "content": [{"type": "text", "text": text}],
                "structuredContent": {"memories": memories}
            })
        }
        "memory.remember" => {
            let args: RememberArgs = serde_json::from_value(arguments)
                .context("invalid memory.remember arguments")?;
            if args.title.trim().is_empty() || args.body.trim().is_empty() {
                bail!("title and body cannot be empty");
            }
            let project = default_project;
            let memory = db.remember(project, &args.title, &args.body, &args.tags)?;
            json!({
                "content": [{
                    "type": "text",
                    "text": format!("Remembered '{}' with ID {}.", memory.title, memory.id)
                }],
                "structuredContent": {"memory": memory}
            })
        }
        _ => bail!("unknown tool: {name}"),
    };

    Ok(tool_result)
}

fn tool_list() -> Value {
    json!({
        "tools": [
            {
                "name": "memory.search",
                "description": "Search durable project memories using local SQLite FTS5 keyword search. Call this before work that may depend on previous decisions, fixes, conventions, or failed approaches.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["query"],
                    "properties": {
                        "query": {"type": "string", "minLength": 1, "maxLength": 2000},
                        "limit": {"type": "integer", "minimum": 1, "maximum": 50, "default": 10}
                    }
                }
            },
            {
                "name": "memory.remember",
                "description": "Store a durable project memory locally. Use for decisions, bug fixes, conventions, API contracts, and failed approaches that should survive future sessions.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["title", "body"],
                    "properties": {
                        "title": {"type": "string", "minLength": 1, "maxLength": 300},
                        "body": {"type": "string", "minLength": 1, "maxLength": 20000},
                        "tags": {"type": "array", "items": {"type": "string"}, "maxItems": 20, "default": []}
                    }
                }
            }
        ]
    })
}

fn parse_db_path() -> Result<PathBuf> {
    let args: Vec<String> = env::args().collect();
    if let Some(index) = args.iter().position(|arg| arg == "--db") {
        let value = args.get(index + 1).context("--db requires a path")?;
        return Ok(PathBuf::from(value));
    }

    if let Ok(value) = env::var("AGENTOS_DB") {
        return Ok(PathBuf::from(value));
    }

    let home = env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .context("could not determine home directory; set AGENTOS_DB")?;

    Ok(Path::new(&home).join(".agentos").join("agentos.db"))
}

fn canonical_project() -> Result<String> {
    let current = env::current_dir().context("failed to read current directory")?;
    let mut project = current
        .canonicalize()
        .context("failed to canonicalize project path")?
        .to_string_lossy()
        .replace('\\', "/");
    if cfg!(windows) {
        project = project.to_lowercase();
    }
    Ok(project)
}

fn error_response(id: Value, code: i32, message: String) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message}
    })
}

fn write_message(writer: &mut impl Write, message: &Value) -> Result<()> {
    serde_json::to_writer(&mut *writer, message)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}
