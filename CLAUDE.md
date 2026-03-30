# CLAUDE.md — kb-mcp

## What This Is

Cargo workspace — MCP server + CLI for markdown knowledge bases.
Indexes markdown collections (configured via RON) into a BM25 search index
and exposes 10 MCP tools via rmcp (stdio transport).

Three crates: `kb-core` (shared library), `kb-cli` (CLI binary `kb`),
`kb-mcp-server` (MCP binary `kb-mcp`).

## Architecture

```
kb-mcp/
├── Cargo.toml                      # Workspace root
├── collections.ron                  # Local dev config (gitignored)
├── collections.example.ron          # Template for users
├── crates/
│   ├── kb-core/                     # Shared library (no rmcp/clap deps)
│   │   └── src/
│   │       ├── lib.rs               # AppContext, init(), re-exports
│   │       ├── types.rs             # Document, Section
│   │       ├── config.rs            # RON config loading + resolution chain
│   │       ├── index.rs             # Document scanning, frontmatter parsing
│   │       ├── store.rs             # .mv2 lifecycle, content hashing, sync
│   │       ├── search.rs            # Tantivy BM25 search engine
│   │       ├── format.rs            # JSON output formatting (shared contract)
│   │       ├── query.rs             # matches_query() for frontmatter filtering
│   │       └── write.rs             # slugify_title(), find_available_path()
│   ├── kb-cli/                      # CLI binary (installs as `kb`)
│   │   └── src/
│   │       └── main.rs              # Clap subcommands → kb_core::* calls
│   └── kb-mcp-server/               # MCP server binary (installs as `kb-mcp`)
│       └── src/
│           ├── main.rs              # MCP stdio server startup
│           ├── server.rs            # KbMcpServer, auto-reindex, ServerHandler
│           └── tools/               # One file per tool (rmcp wrappers)
│               ├── mod.rs           # Router composition
│               ├── sections.rs      # list_sections
│               ├── search.rs        # search (with auto-reindex)
│               ├── documents.rs     # get_document
│               ├── context.rs       # kb_context
│               ├── write.rs         # kb_write
│               ├── reindex.rs       # reindex
│               ├── digest.rs        # kb_digest
│               ├── query.rs         # kb_query
│               ├── export.rs        # kb_export
│               └── health.rs        # kb_health
├── book/                            # mdBook documentation
└── docs/
```

## Key Patterns

- **Workspace with three crates:** `kb-core` (library), `kb-cli` (binary `kb`),
  `kb-mcp-server` (binary `kb-mcp`). Both binaries depend on `kb-core`.
- **kb-core has zero dependency on rmcp, schemars, or clap.** Transport-specific
  deps belong in the binary crates only.
- **AppContext:** `kb_core::init(config_path)` returns an `AppContext` with owned
  values. CLI uses them directly; MCP server wraps in Arc/RwLock.
- **RON config:** `collections.ron` defines all collections, sections, and descriptions.
  Zero hardcoded project-specific values in the binary.
- **Config resolution:** `--config` → `KB_MCP_CONFIG` env → `./collections.ron` → `~/.config/kb-mcp/collections.ron`
- **Collection paths** resolve relative to the config file's parent directory.
- **Tool pattern:** One file per tool in `kb-mcp-server/src/tools/`. Each has params struct
  (`Deserialize + JsonSchema`), router function, and `#[rmcp::tool]` annotation.
  Routers composed with `+` in `tools/mod.rs`.
- **Logs to stderr** — stdout is the MCP JSON-RPC transport.
- **Shared output contract:** Both CLI and MCP tools call the same `kb_core::format::*`
  functions, guaranteeing identical JSON output regardless of transport.

## Hard Rules

1. **No hardcoded paths or project-specific values.** Everything comes from `collections.ron`.
2. **All MCP tools must have CLI equivalents.** Parity between modes.
3. **get_document reads from disk** — the index is for search/lookup only. Fresh content always.
4. **kb_write only writes to collections with `writable: true`** — error with actionable message otherwise.
5. **kb-core must not depend on rmcp, schemars, or clap.** Transport concerns stay in binary crates.

## Code Quality Rules

### Think Before Coding
- State assumptions explicitly. If uncertain, ask.
- If multiple approaches exist, present them — don't pick silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

### Simplicity First
- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.

### Surgical Changes
- Touch only what you must.
- Don't "improve" adjacent code, comments, or formatting.
- Match existing style. Every changed line should trace to the request.

### Rust Documentation
- Every doc comment explains **why**, not **what**. The code shows what — docs
  explain the design choice, tradeoff, or constraint that isn't obvious from reading it.
- Module-level `//!` docs on every file explaining the module's role and key design decisions.
- Doc comments (`///`) on public types and non-trivial functions.
- Don't doc the obvious — `/// Returns the path` on `fn path()` is noise.

### Goal-Driven Execution
Transform tasks into verifiable goals:
```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
```

## Common Commands

```sh
just              # List commands
just build        # cargo build --workspace
just release      # cargo build --release --workspace
just check        # cargo check --workspace
just clippy       # cargo clippy --workspace
just test         # cargo test --workspace
just install      # Install both kb and kb-mcp binaries
just install-cli  # Install kb CLI only
just install-server # Install kb-mcp server only
just run <args>   # cargo run -p kb-cli -- <args>
just run-server   # cargo run -p kb-mcp-server
just book-build   # Build mdBook docs
just book-serve   # Serve docs with live reload
```

## Verification

After making changes, verify with:

- `cargo build --workspace` — compiles without errors
- `cargo clippy --workspace` — no new warnings
- `cargo test --workspace` — tests pass
- `just book-build` — mdBook builds cleanly (if docs changed)
- `cargo run -p kb-cli -- list-sections` — CLI works against `collections.ron`
- `cargo run -p kb-cli -- search --query "test"` — search returns results

## Development Tooling

See [docs/DEV.md](docs/DEV.md) for the full development methodology,
including the Compound Engineering workflow (`/ce:brainstorm` → `/ce:plan`
→ `/ce:work` → `/ce:review` → `/ce:compound`) and session patterns.

Brainstorms and plans are preserved in `docs/brainstorms/` and `docs/plans/`.

## Adding a New Tool

1. Create `crates/kb-mcp-server/src/tools/my_tool.rs` with params struct and `#[rmcp::tool]` impl
2. Add `pub(crate) mod my_tool;` to `crates/kb-mcp-server/src/tools/mod.rs`
3. Add `+ my_tool::router()` to `combined_router()`
4. Add corresponding CLI subcommand in `crates/kb-cli/src/main.rs`
5. If new shared logic is needed, add to `crates/kb-core/src/`
6. Update server instructions in `crates/kb-mcp-server/src/server.rs`
