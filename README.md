# \# AgentOS — Local-first persistent memory for AI coding agents

# 

# \*\*Phase 0.5 MVP:\*\* a small Rust MCP server that gives Claude Code persistent, project-scoped memory using SQLite FTS5.

# 

# \[!\[Rust 1.80+](https://img.shields.io/badge/Rust-1.80%2B-000000?logo=rust)](https://www.rust-lang.org/)

# \[!\[License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

# \[!\[Protocol: MCP](https://img.shields.io/badge/Protocol-MCP-blue)](https://modelcontextprotocol.io/)

# 

# \## Why AgentOS?

# 

# AI coding agents lose important context when a session ends. Architecture decisions, failed approaches, project conventions, and bug-fix details often have to be explained again.

# 

# AgentOS provides a small local memory layer that coding agents can access through MCP. Memories are stored in SQLite and retrieved with FTS5 keyword search — without a cloud account, embedding API, or external database.

# 

# Phase 0.5 is intentionally narrow: save explicit project memories and retrieve them in future Claude Code sessions.

# 

# \## Features

# 

# \- Persistent memory across Claude Code sessions

# \- Local SQLite storage with bundled SQLite

# \- Fast keyword retrieval through SQLite FTS5

# \- Two focused MCP tools: `memory.remember` and `memory.search`

# \- Project isolation using the MCP server's working directory

# \- MCP JSON-RPC communication over standard input and output

# \- No telemetry, network service, cloud database, or API key

# 

# \## Architecture

# 

# ```text

# ┌─────────────────────┐

# │     Claude Code     │

# └──────────┬──────────┘

# &#x20;          │

# &#x20;          │ MCP JSON-RPC

# &#x20;          │ newline-delimited stdio

# &#x20;          ▼

# ┌─────────────────────┐

# │       AgentOS       │

# │                     │

# │  memory.remember    │

# │  memory.search      │

# └──────────┬──────────┘

# &#x20;          │

# &#x20;          │ rusqlite

# &#x20;          ▼

# ┌─────────────────────┐

# │   SQLite + FTS5     │

# │                     │

# │  Local persistence  │

# │  Keyword search     │

# └─────────────────────┘

