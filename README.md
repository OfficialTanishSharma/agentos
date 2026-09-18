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

# AgentOS provides a small local memory layer that coding agents can access through MCP. Memories are stored in SQLite and retrieved with FTS5 keyword search—without a cloud account, embedding API, or external database.

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

# ```

# 

# SQLite is the source of truth. The FTS5 index is kept synchronized with the `memories` table through SQLite triggers.

# 

# \## Quick Start

# 

# \### Requirements

# 

# AgentOS currently uses a Windows-first development workflow.

# 

# Install:

# 

# \- Windows 10 or Windows 11

# \- \[Rust 1.80 or newer](https://rustup.rs/)

# \- Visual Studio 2022 Build Tools

# \- The \*\*Desktop development with C++\*\* workload

# \- MSVC v143 build tools

# \- Windows 10 or Windows 11 SDK

# \- \[Claude Code](https://code.claude.com/)

# 

# A separate SQLite installation is not required. AgentOS compiles and links bundled SQLite through `rusqlite`.

# 

# \### 1. Clone the repository

# 

# Open PowerShell:

# 

# ```powershell

# git clone https://github.com/OfficialTanishSharma/agentos.git

# Set-Location .\\agentos

# ```

# 

# \### 2. Verify the Rust toolchain

# 

# ```powershell

# rustc --version

# cargo --version

# rustup show

# ```

# 

# The active host should normally be:

# 

# ```text

# x86\_64-pc-windows-msvc

# ```

# 

# If required, select it explicitly:

# 

# ```powershell

# rustup default stable-x86\_64-pc-windows-msvc

# ```

# 

# \### 3. Test and build AgentOS

# 

# ```powershell

# cargo test

# cargo build --release

# ```

# 

# The release binary will be created at:

# 

# ```text

# target\\release\\agentos.exe

# ```

# 

# Verify it:

# 

# ```powershell

# Get-Item .\\target\\release\\agentos.exe

# ```

# 

# \### 4. Connect AgentOS to Claude Code

# 

# Resolve the release binary to an absolute path:

# 

# ```powershell

# $agentos = (Resolve-Path .\\target\\release\\agentos.exe).Path

# ```

# 

# Register it as a project-scoped stdio MCP server:

# 

# ```powershell

# claude mcp add --transport stdio --scope project agentos -- $agentos

# ```

# 

# Inspect the configuration and connection status:

# 

# ```powershell

# claude mcp get agentos

# claude mcp list

# ```

# 

# The expected status is:

# 

# ```text

# ✔ Connected

# ```

# 

# If the server is waiting for project approval, start Claude Code and approve the MCP configuration:

# 

# ```powershell

# claude

# ```

# 

# \### 5. Save a memory

# 

# Inside Claude Code, ask:

# 

# ```text

# Call memory.remember with these values:

# 

# title: AgentOS storage decision

# body: AgentOS Phase 0.5 uses bundled SQLite with FTS5 for local persistent keyword search.

# tags: architecture, sqlite, phase-0.5

# ```

# 

# \### 6. Retrieve the memory

# 

# Ask:

# 

# ```text

# Call memory.search with query "SQLite FTS5 storage" and limit 10.

# ```

# 

# Exit Claude Code, start a new session from the same project directory, and repeat the search. The saved memory should remain available.

# 

# \### Database location

# 

# On Windows, the default database is:

# 

# ```text

# %USERPROFILE%\\.agentos\\agentos.db

# ```

# 

# Inspect it with PowerShell:

# 

# ```powershell

# $db = "$env:USERPROFILE\\.agentos\\agentos.db"

# 

# Get-Item $db

# Get-Item "$db-wal" -ErrorAction SilentlyContinue

# Get-Item "$db-shm" -ErrorAction SilentlyContinue

# ```

# 

# Override the location for the current PowerShell session:

# 

# ```powershell

# $env:AGENTOS\_DB = "$PWD\\agentos-test.db"

# ```

# 

# Remove the override:

# 

# ```powershell

# Remove-Item Env:AGENTOS\_DB

# ```

# 

# \## MCP Tools

# 

# | Tool | Purpose | Required arguments |

# |---|---|---|

# | `memory.remember` | Store a durable memory for the current project | `title`, `body` |

# | `memory.search` | Search current-project memories with SQLite FTS5 | `query` |

# 

# \### `memory.remember`

# 

# Use `memory.remember` for information that should survive future coding sessions:

# 

# \- Architecture decisions

# \- Bug fixes and root causes

# \- API contracts

# \- Project conventions

# \- Commands that solved a problem

# \- Failed approaches that should not be repeated

# 

# Example arguments:

# 

# ```json

# {

# &#x20; "title": "Use WAL mode for SQLite",

# &#x20; "body": "AgentOS uses SQLite WAL mode so readers are not blocked by normal write activity. A five-second busy timeout handles short lock contention.",

# &#x20; "tags": \[

# &#x20;   "architecture",

# &#x20;   "sqlite",

# &#x20;   "concurrency"

# &#x20; ]

# }

# ```

# 

# Example result:

# 

# ```text

# Remembered 'Use WAL mode for SQLite' with ID 46a71d9cb5b748efbd66738758cb089a.

# ```

# 

# Arguments:

# 

# | Name | Type | Required | Description |

# |---|---|---:|---|

# | `title` | string | Yes | Short, searchable memory title |

# | `body` | string | Yes | Full memory content |

# | `tags` | string array | No | Searchable labels |

# | `project` | string | No | Explicit project key; defaults to the server working directory |

# 

# \### `memory.search`

# 

# `memory.search` performs local FTS5 keyword search. Titles receive a higher BM25 ranking weight than memory bodies and tags.

# 

# Example arguments:

# 

# ```json

# {

# &#x20; "query": "SQLite WAL concurrency",

# &#x20; "limit": 10

# }

# ```

# 

# Example result:

# 

# ```text

# 1\. Use WAL mode for SQLite

# ID: 46a71d9cb5b748efbd66738758cb089a

# Tags: architecture, sqlite, concurrency

# Created: 2026-09-18T14:32:10.125Z

# AgentOS uses SQLite WAL mode so readers are not blocked by normal write activity. A five-second busy timeout handles short lock contention.

# ```

# 

# Arguments:

# 

# | Name | Type | Required | Description |

# |---|---|---:|---|

# | `query` | string | Yes | Keywords to search for |

# | `limit` | integer | No | Number of results, from 1 to 50; defaults to 10 |

# | `project` | string | No | Explicit project key; defaults to the server working directory |

# 

# Search terms are quoted and joined with `OR`. This keeps the FTS query safe and favors useful partial matches, but it is not semantic search.

# 

# \## How It Works

# 

# Claude Code starts AgentOS as a child process and communicates with it using newline-delimited JSON-RPC over stdin and stdout.

# 

# AgentOS implements the MCP methods required for this MVP:

# 

# ```text

# initialize

# ping

# tools/list

# tools/call

# ```

# 

# Protocol responses are written only to stdout. Startup information and diagnostic messages are written to stderr so they do not corrupt MCP framing.

# 

# When the process starts, AgentOS:

# 

# 1\. Resolves the database path.

# 2\. Creates the database directory if it does not exist.

# 3\. Opens SQLite with a five-second busy timeout.

# 4\. Enables WAL journal mode.

# 5\. Creates the memory table, FTS5 index, and synchronization triggers.

# 6\. Resolves the current working directory as the default project key.

# 7\. Waits for MCP requests on stdin.

# 

# The default project key is the canonical working-directory path with Windows path separators converted to forward slashes. Every search filters by this key.

# 

# Because project isolation depends on the working directory, Claude Code should be started from the same project root when memories are saved and retrieved.

# 

# \## What's NOT Included

# 

# Phase 0.5 does not include:

# 

# \- Semantic search

# \- Embeddings or local language models

# \- LanceDB, Qdrant, or another vector database

# \- Automatic source-code indexing

# \- Git-history indexing

# \- Conversation import

# \- Cross-agent session handoffs

# \- Skill discovery or skill routing

# \- Background daemon or file watcher

# \- Memory editing or deletion MCP tools

# \- Memory deduplication

# \- Team synchronization

# \- Cloud backup

# \- HTTP transport

# \- TUI or desktop interface

# \- Telemetry

# 

# These limitations are intentional. Phase 0.5 tests the smallest useful version of persistent coding-agent memory before introducing additional storage and retrieval systems.

# 

# \## Development

# 

# \### Format

# 

# ```powershell

# cargo fmt

# cargo fmt --check

# ```

# 

# \### Static checks

# 

# ```powershell

# cargo check

# cargo clippy --all-targets --all-features -- -D warnings

# ```

# 

# \### Tests

# 

# ```powershell

# cargo test

# cargo test -- --nocapture

# ```

# 

# The current tests cover:

# 

# \- Saving and retrieving a memory through FTS5

# \- Isolating search results by project key

# 

# \### Release build

# 

# ```powershell

# cargo build --release

# ```

# 

# Generate a SHA-256 checksum:

# 

# ```powershell

# Get-FileHash .\\target\\release\\agentos.exe -Algorithm SHA256 |

# &#x20;   Format-List

# ```

# 

# \### Manual MCP smoke test

# 

# Create a JSONL request file:

# 

# ```powershell

# @'

# {"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"manual-test","version":"1.0"}}}

# {"jsonrpc":"2.0","method":"notifications/initialized","params":{}}

# {"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}

# {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"memory.remember","arguments":{"title":"Manual test","body":"AgentOS stored this memory through MCP JSON-RPC.","tags":\["test","mcp"]}}}

# {"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"memory.search","arguments":{"query":"manual MCP test","limit":10}}}

# '@ | Set-Content .\\requests.jsonl -Encoding utf8

# ```

# 

# Run it through AgentOS:

# 

# ```powershell

# Get-Content .\\requests.jsonl -Encoding utf8 |

# &#x20;   .\\target\\release\\agentos.exe --db "$PWD\\manual-test.db"

# ```

# 

# The server should respond to request IDs `1`, `2`, `3`, and `4`. It should not respond to `notifications/initialized`.

# 

# \## Roadmap

# 

# \### Phase 1.0 — Semantic memory

# 

# Planned direction:

# 

# \- Local embeddings

# \- Hybrid FTS5 and vector retrieval

# \- Automatic project-file indexing

# \- Git-aware memory provenance

# \- Memory lifecycle and stale-memory detection

# \- Search result deduplication and ranking

# 

# The local-first requirement remains: memory search should not require a hosted embedding API.

# 

# \### Phase 2.0 — Skill router

# 

# Planned direction:

# 

# \- Local skill registry

# \- Git-based skill sources

# \- Task-to-skill matching

# \- Agent-specific installation adapters

# \- Skill provenance and version pinning

# \- Local feedback on whether a skill helped

# 

# Skill installation should remain explicit and reviewable.

# 

# \### Phase 3.0 — Skill sandbox

# 

# Planned direction:

# 

# \- Declared skill capabilities

# \- File-system and command boundaries

# \- Permission review before execution

# \- Isolated skill processes

# \- Auditable command and file activity

# \- Cross-agent session handoffs built on structured memory

# 

# The roadmap may change based on Phase 0.5 usage and reported failure cases.

# 

# \## License

# 

# AgentOS is available under the \[MIT License](LICENSE).

# 

# ```text

# Copyright (c) 2026 Tanish Sharma

# ```

