# Brainstorm: Cargo Workspace Reorganization

**Date:** 2026-03-30
**Status:** Ready for planning
**Inspiration:** [crabtalk/crabtalk-mcp](https://github.com/crabtalk/crabtalk-mcp) pattern of MCP-as-thin-wrapper-around-core

## What We're Building

Reorganize kb-mcp from a single binary crate into a cargo workspace with three crates:

- **`kb-core`** (library) — types, config, index, store, search, format. Zero dependency on rmcp or clap.
- **`kb-cli`** (binary) — thin CLI shell. Clap parsing -> `kb_core::*` calls -> JSON to stdout.
- **`kb-mcp-server`** (binary) — thin MCP server. rmcp tool wrappers -> `kb_core::*` calls -> CallToolResult.

Directory layout uses `crates/` nesting:

```
kb-mcp/
├── Cargo.toml              # [workspace] members = ["crates/*"]
├── crates/
│   ├── kb-core/
│   │   ├── Cargo.toml      # [lib]
│   │   └── src/
│   │       ├── lib.rs       # AppContext, init()
│   │       ├── config.rs
│   │       ├── index.rs
│   │       ├── store.rs
│   │       ├── search.rs
│   │       ├── format.rs
│   │       └── types.rs
│   ├── kb-cli/
│   │   ├── Cargo.toml      # [[bin]], deps: kb-core, clap
│   │   └── src/main.rs
│   └── kb-mcp-server/
│       ├── Cargo.toml      # [[bin]], deps: kb-core, rmcp, schemars
│       └── src/
│           ├── main.rs
│           ├── server.rs    # KbMcpServer, auto-reindex, ServerHandler
│           └── tools/       # One file per tool (existing pattern)
├── book/
├── docs/
└── tests/
```

## Why This Approach

**Primary motivation:** Clean architecture. The codebase has grown past the point where a single crate is tidy. CLI and MCP tool implementations duplicate logic (write command is 90+ lines duplicated, query validation duplicated, search auto-reindex only in MCP). Fixing a bug in one path doesn't fix the other.

**Why now:** The v2 plan (2026-03-18) rejected this split as YAGNI — correctly, at that scope. Since then, 10 tools with CLI parity have been implemented, and the duplication is concrete and growing. The v2 roadmap also mentions HTTP daemon mode as a future transport, which would be a natural third binary.

**Why not crabtalk's full pattern:** Crabtalk uses proc macros (`#[command]`), a daemon framework, and a `McpService` trait — all overkill for 10 tools. We take the workspace structure and core-library-extraction pattern, skip the framework abstractions.

## Key Decisions

1. **`crates/` directory layout** — crates nested under `crates/` (not flat at repo root). Keeps root clean as crate count grows.

2. **Direct core calls, no ops abstraction** — MCP tools and CLI both call `kb_core::format::*`, `kb_core::search()`, etc. directly. No intermediate Command trait or ops module. The format functions *are* the shared contract (they already serve this role).

3. **AppContext struct for shared init** — `kb_core::init(config_path) -> Result<AppContext>` bundles the full initialization sequence (load config, build index, sync stores, create search engine). Both binaries call this one function. Eliminates the duplicated init logic in current `main.rs`.

4. **Incremental migration** — Phase 1: extract kb-core, keep single binary working against it. Phase 2: split into kb-cli and kb-mcp-server. Each phase is a separate commit/PR, independently compilable.

5. **Feature flag passthrough** — `hybrid` feature lives on `kb-core` and is re-exported by both binary crates via `kb-core/hybrid`.

## What Moves Where

### Into `kb-core`

| Current location | Destination | Notes |
|---|---|---|
| `src/types.rs` | `kb-core/src/types.rs` | As-is |
| `src/config.rs` | `kb-core/src/config.rs` | As-is |
| `src/index.rs` | `kb-core/src/index.rs` | As-is |
| `src/store.rs` | `kb-core/src/store.rs` | As-is (carries existing tests) |
| `src/search.rs` | `kb-core/src/search.rs` | As-is |
| `src/format.rs` | `kb-core/src/format.rs` | As-is (678 lines, the shared output contract) |
| `main.rs` `sync_stores()` | `kb-core/src/store.rs` or `lib.rs` | Shared init logic |
| `tools/write.rs` `slugify_title()` | `kb-core` (utility) | Currently called from cli.rs |
| `tools/write.rs` `find_available_path()` | `kb-core` (utility) | Duplicated in cli.rs |
| `tools/query.rs` `matches_query()` | `kb-core` (query logic) | Currently called from cli.rs |

### Into `kb-mcp-server`

| Current location | Destination | Notes |
|---|---|---|
| `src/server.rs` | `kb-mcp-server/src/server.rs` | KbMcpServer wraps AppContext in Arc/RwLock |
| `src/tools/*.rs` | `kb-mcp-server/src/tools/*.rs` | Keep rmcp router pattern, but bodies shrink to core calls |

### Into `kb-cli`

| Current location | Destination | Notes |
|---|---|---|
| `src/cli.rs` | `kb-cli/src/main.rs` | Shrinks dramatically — each command becomes ~5-10 lines |

## Coupling Seams That Need Surgery

1. **`format::extract_summary()` called from `search.rs`** — both move to core together, so this resolves naturally.
2. **`tools::query::matches_query()` called from `cli.rs`** — extract to core. MCP tool's `QueryParams` (with `JsonSchema` derive) stays in MCP crate, converts to core params.
3. **`tools::write::slugify_title()` and `find_available_path()`** — extract to core. Currently duplicated.
4. **`server.rs` `read_fresh()` uses `format::read_document_body()`** — both in core, clean.
5. **`server.rs` `auto_reindex_stale_collections()`** — MCP-specific (uses Arc/RwLock + debounce), stays in MCP crate. Calls `Index::rebuild_collection()` and `store::sync_collection()` from core.

## What Stays the Same

- `collections.ron` format and resolution chain
- One-file-per-tool pattern in MCP server
- `format::*` functions as the shared output contract
- All four Hard Rules from CLAUDE.md
- The justfile (updated for workspace: `cargo build -p kb-cli`, etc.)

## Open Questions

None — all key decisions resolved during brainstorm.
