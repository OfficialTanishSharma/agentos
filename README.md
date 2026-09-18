# AgentOS Phase 0.5

A deliberately small, local-first memory MCP server for Claude Code.

## Scope

- Rust single binary
- SQLite + FTS5
- `memory.search`
- `memory.remember`
- MCP over stdio
- No embeddings, file indexing, handoffs, skills, daemon, or cloud

## Build

```bash
cargo build --release
cargo test
```

Binary:

- Windows: `target\\release\\agentos.exe`
- macOS/Linux: `target/release/agentos`

## Claude Code setup

From the project directory, use the absolute binary path:

```bash
claude mcp add agentos --scope project -- /absolute/path/to/agentos
```

Windows PowerShell example:

```powershell
claude mcp add agentos --scope project -- C:\\code\\agentos\\target\\release\\agentos.exe
```

Verify:

```bash
claude mcp list
```

Start Claude Code in the same project and ask:

```text
Use memory.remember to save that we chose SQLite FTS5 for Phase 0.5 because it keeps the binary local and the MVP small. Tag it architecture and mvp.
```

Then start a new session and ask:

```text
Use memory.search to find our Phase 0.5 storage decision.
```

## Storage

Default database locations:

- Windows: `%USERPROFILE%\\.agentos\\agentos.db`
- macOS/Linux: `$HOME/.agentos/agentos.db`

Override with:

```bash
agentos --db /custom/path/agentos.db
```

or `AGENTOS_DB`.

Memories are isolated by the MCP process working directory. Claude Code should launch the server from the project root.

## Debug MCP manually

Each request and response is one JSON object per line:

```text
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"manual","version":"1"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"memory.remember","arguments":{"title":"Use FTS5","body":"Phase 0.5 uses SQLite FTS5 only.","tags":["architecture"]}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"memory.search","arguments":{"query":"SQLite FTS5","limit":10}}}
```

On PowerShell, paste those lines into the running executable one at a time. Diagnostic logs go to stderr; protocol messages use stdout.
