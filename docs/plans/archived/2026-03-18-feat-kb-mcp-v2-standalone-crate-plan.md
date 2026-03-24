---
title: "feat: kb-mcp v2 — Standalone Rust MCP Server for Markdown Knowledge Bases"
type: feat
status: completed
date: 2026-03-18
origin: docs/brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md
---

# feat: kb-mcp v2 — Standalone Rust MCP Server for Markdown Knowledge Bases

## Overview

Rewrite kb-mcp as a standalone, project-agnostic Rust binary crate. Replace
the in-memory Tantivy index with memvid-core `.mv2` persistent storage.
Replace all hardcoded project-specific values with RON-based collection
configuration. Add `kb_context` and `kb_write` tools. Push to GitHub as
its own repo.

The v1 kb-mcp (~600 lines) becomes the reference for porting. The new
project has zero knowledge of any specific vault structure — it indexes
whatever its `collections.ron` tells it to.

## Problem Statement / Motivation

The v1 kb-mcp had three problems that prevent reuse:

1. **Hardcoded section descriptions** — 8 entries baked into `index.rs`
2. **Hardcoded collection paths** — two specific directories with a thin
   config layer on top
3. **In-memory index** — rebuilt from scratch on every startup, no persistence

These make it a single-project tool. Other projects that want a markdown
knowledge base MCP server cannot use it without forking.

(See brainstorm: `docs/brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md`)

## Proposed Solution

A standalone binary crate configured entirely via RON files. No project-specific
code in the binary. memvid-core provides persistent `.mv2` storage with BM25
search, smart markdown chunking, and crash-safe WAL.

### RON Collection Schema

```ron
// ~/.config/kb-mcp/collections.ron (or project-local)
(
    cache_dir: "~/.cache/kb-mcp",
    collections: [
        (
            name: "docs",
            path: "docs",
            description: "Project documentation",
            writable: false,
            sections: [
                (prefix: "guides", description: "How-to guides"),
                (prefix: "reference", description: "API reference"),
            ],
        ),
        (
            name: "notes",
            path: "notes",
            description: "Working notes",
            writable: true,
            sections: [],
        ),
    ],
)
```

### Rust Types for RON Schema

```rust
// src/config.rs
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub cache_dir: Option<String>,  // default: ~/.cache/kb-mcp
    pub collections: Vec<Collection>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Collection {
    pub name: String,
    pub path: String,               // relative to config file location
    pub description: String,
    pub writable: bool,
    pub sections: Vec<SectionDef>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SectionDef {
    pub prefix: String,
    pub description: String,
}
```

### Config Resolution Order

1. `--config <path>` CLI flag (explicit)
2. `KB_MCP_CONFIG` env var
3. `./collections.ron` (current working directory)
4. `~/.config/kb-mcp/collections.ron` (user default)

Collection paths resolve relative to the config file's parent directory.

### Cache Namespacing

`.mv2` files are stored at:
```
<cache_dir>/<sha256-of-absolute-collection-path-first-8-chars>-<collection-name>.mv2
```

Example: `~/.cache/kb-mcp/a3f1b2c4-docs.mv2`

This prevents collisions when two projects both define a collection with
the same name but different paths.

## Technical Considerations

### Architecture

```
kb-mcp/
├── Cargo.toml
├── Cargo.lock
├── CLAUDE.md
├── README.md
├── LICENSE
├── collections.example.ron       # Example config for users
├── src/
│   ├── main.rs                   # Dual-mode entry (MCP stdio / CLI)
│   ├── config.rs                 # RON config loading + resolution
│   ├── index.rs                  # Document scanning, frontmatter parsing
│   ├── search.rs                 # BM25 search engine
│   ├── format.rs                 # JSON output formatting
│   ├── types.rs                  # Core data types
│   ├── cli.rs                    # Clap CLI subcommands
│   ├── server.rs                 # rmcp ServerHandler
│   └── tools/
│       ├── mod.rs                # Router composition
│       ├── sections.rs           # list_sections
│       ├── search.rs             # search
│       ├── documents.rs          # get_document
│       ├── context.rs            # kb_context (NEW)
│       ├── write.rs              # kb_write (NEW)
│       └── reindex.rs            # reindex
└── tests/
    └── integration.rs            # Test against sample markdown collections
```

### Dependencies

```toml
[package]
name = "kb-mcp"
version = "0.2.0"
edition = "2024"
description = "MCP server for markdown knowledge bases"
license = "MIT"
repository = "https://github.com/ttdonovan/kb-mcp"

[dependencies]
# MCP server
rmcp = { version = "1.2", features = ["server", "macros", "transport-io"] }
schemars = "1.0"

# Storage + search
memvid-core = { version = "2.0", default-features = false, features = ["lex"] }

# Configuration
ron = "0.12"

# Common
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
tokio = { version = "1", features = ["full"] }
anyhow = "1"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dirs = "6"
sha2 = "0.10"
```

### Key Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Config format | RON | Typed, Rust-native, maps directly to derive structs, comments supported (see brainstorm: TOML was considered but RON is more natural for nested structs) |
| Storage backend | memvid-core `.mv2` (lex feature) | Persistent BM25, smart chunking, crash-safe WAL, minimal deps with feature gating |
| get_document source | Read from disk | `.mv2` for search/lookup only; disk read ensures freshness without reindex |
| Cache namespacing | Hash prefix + collection name | Prevents cross-project collisions for same-named collections |
| Cross-collection search | Optional `collection` parameter; default searches all | Simple, correct; scores merged by normalized BM25 |
| Deleted file detection | Full dir listing vs indexed paths on reindex | Paths in index but missing from disk are removed |
| Concurrent access | Rely on memvid-core WAL + file locking | Document limitation if memvid-core doesn't support multiple writers |

### Performance Implications

- **Cold start (existing .mv2):** Open file, ready. Sub-100ms for typical collections.
- **First index (~200 docs):** Estimated <2s. Logs progress to stderr.
- **Incremental reindex:** Content hash comparison, only changed/added/deleted files processed.
- **Search latency:** Same as current (Tantivy BM25), but against persistent index.

### Security Considerations

- `kb_write` only writes to collections with `writable: true`
- Collection paths validated to prevent directory traversal
- No network access — purely local filesystem operations
- No secrets in config (paths only)

## System-Wide Impact

- **Interaction graph:** MCP client → kb-mcp (stdio) → memvid-core → filesystem. No external services.
- **Error propagation:** memvid-core errors → anyhow → MCP `CallToolResult` with `is_error: true` and actionable message.
- **State lifecycle:** `.mv2` files are the only persistent state. Safe to delete (triggers full rebuild on next startup).
- **API surface parity:** All 6 MCP tools have CLI equivalents.

## Acceptance Criteria

### Functional Requirements

- [x] `collections.ron` parsed with all fields (name, path, description, writable, sections)
- [x] Config resolution chain works: `--config` → env var → `./collections.ron` → `~/.config/kb-mcp/collections.ron`
- [x] Collection paths resolve relative to config file location
- [ ] `.mv2` files created per-collection at `<cache_dir>/<hash>-<name>.mv2` *(deferred — using in-memory Tantivy for now, memvid-core swap is Phase 2 follow-up)*
- [x] `list_sections` returns collections with section doc counts and descriptions from RON
- [x] `search` returns BM25-ranked results with snippets from smart chunks
- [x] `search` accepts optional `collection` scope parameter
- [x] `get_document` reads file from disk (fresh content), uses index for path resolution
- [x] `kb_context` returns frontmatter fields + first paragraph/summary
- [x] `kb_context` handles documents without frontmatter (title + first paragraph)
- [x] `kb_write` creates file with generated frontmatter in writable collection
- [x] `kb_write` generates date-prefix filename (`YYYY-MM-DD-kebab-title.md`)
- [x] `kb_write` rejects writes to non-writable collections with actionable error
- [x] `kb_write` handles filename collisions (numeric suffix)
- [x] `reindex` rebuilds index from all collections on disk
- [x] All 6 tools work as both MCP tools and CLI subcommands
- [x] Dual-mode binary: no args → MCP stdio server, with args → CLI
- [x] Logs to stderr only (stdout reserved for MCP JSON-RPC)

### Non-Functional Requirements

- [x] Zero hardcoded project-specific values in the binary
- [x] Builds with `cargo build --release` on stable Rust
- [x] No required runtime dependencies beyond the binary itself
- [x] `collections.example.ron` included for user reference

### Quality Gates

- [ ] Integration test against sample markdown collection
- [x] Verified working against real vault via `collections.ron`
- [x] `cargo clippy` clean (2 expected dead-code warnings for future `.mv2` fields)
- [x] README with setup instructions

## Implementation Phases

### Phase 1: Scaffold + Config (src/main.rs, src/config.rs, src/types.rs)

1. `cargo init`
2. Set up `Cargo.toml` with all dependencies
3. Create `CLAUDE.md` with project conventions
4. Implement RON config loading with resolution chain
5. Define core types: `Config`, `Collection`, `SectionDef`, `Document`
6. Create `collections.example.ron`
7. Verify: `cargo build` compiles, config loads from RON file

### Phase 2: Storage Layer (src/index.rs, src/search.rs)

1. Implement `.mv2` file lifecycle: create, open, cache path derivation
2. Port document scanning — frontmatter parsing, section detection
3. Integrate memvid-core: add documents to `.mv2` with smart chunking
4. Implement BM25 search wrapper with collection scoping
5. Implement incremental reindex with content hashing + deleted file detection
6. Verify: documents indexed into `.mv2`, search returns ranked results

### Phase 3: MCP Tools (src/tools/*.rs, src/server.rs)

1. Implement `list_sections` — read from RON config + index doc counts
2. Implement `search` — add collection scope parameter, BM25 ranking
3. Implement `get_document` — path resolution via index, content from disk
4. Implement `kb_context` — frontmatter extraction + first paragraph summary
5. Implement `kb_write` — validate writable, generate frontmatter, create file, update index
6. Implement `reindex` — expose incremental reindex as tool
7. Wire all tools into rmcp `ServerHandler` via router composition
8. Verify: all 6 tools respond correctly via MCP stdio

### Phase 4: CLI + Polish (src/cli.rs, tests/, README)

1. Add CLI subcommands — `context` and `write` alongside existing commands
2. Wire dual-mode detection in `main.rs`
3. Write integration tests against a sample markdown collection in `tests/fixtures/`
4. Write README with installation, configuration, and usage
5. Run `cargo clippy`, fix warnings
6. Verify: CLI and MCP modes both work, tests pass

### Phase 5: Verify + Push

1. Build and install: `cargo install --path .`
2. Test against a real markdown vault via `collections.ron`
3. Verify: all MCP queries work, plus new `kb_context` and `kb_write` tools
4. `git init`, push to GitHub

## Alternative Approaches Considered

| Approach | Why rejected |
|----------|-------------|
| Library crate + thin binary | YAGNI — no one needs to embed kb-mcp as a library yet. Can extract later. |
| Keep TOML config | RON maps more naturally to nested Rust structs; TOML's `[[collections.sections]]` is repetitive and harder to read for arrays-of-structs. |
| Cherry-pick memvid patterns | memvid-core's storage layer, chunking, and Tantivy integration are exactly what we need — reimplementing means maintaining the same code for no benefit. (See brainstorm) |
| Replace kb-mcp entirely with memvid | memvid has no MCP server; kb-mcp's MCP layer (rmcp, tools, CLI) is still needed. (See brainstorm) |
| Stay with in-memory index | Rebuilding on startup is wasteful and blocks persistent caching, vector search, incremental reindex. (See brainstorm) |

## Dependencies & Prerequisites

- **memvid-core** crate published on crates.io with `lex` feature (verified: v2.0.139)
- **rmcp** crate v1.2+ with `transport-io` feature (verified: v1.2.0)
- **ron** crate v0.12 (verified: v0.12.0)
- Stable Rust toolchain (edition 2024)

## Resolved Questions (from SpecFlow analysis)

| Question | Resolution |
|----------|-----------|
| Config format: RON or TOML? | RON — chosen for Rust-native nested struct mapping |
| Cache namespacing across projects? | Hash prefix of absolute collection path + collection name |
| Cross-collection search ranking? | Optional `collection` parameter; default searches all with normalized BM25 merge |
| Deleted file detection? | Full dir listing compared against indexed paths; missing paths removed |
| Concurrent .mv2 access? | Rely on memvid-core WAL/locking; document limitation if insufficient |
| Config resolution? | `--config` → `KB_MCP_CONFIG` env → `./collections.ron` → `~/.config/kb-mcp/collections.ron` |
| kb_context without frontmatter? | Title (from H1 or filename) + first paragraph as summary |
| kb_write filename collisions? | Numeric suffix: `2026-03-18-my-note-2.md` |
| get_document source? | Read from disk; `.mv2` for search/lookup only |
| CLI parity? | All 6 MCP tools have CLI equivalents |

## Open Questions

| Question | Impact | Default if unresolved |
|----------|--------|-----------------------|
| memvid-core concurrent writer support? | Data safety for CLI + MCP simultaneous use | Investigate during Phase 2; document limitation if unsupported |
| .mv2 format version migration? | Upgrade path between kb-mcp versions | Delete cache + rebuild (fast enough for <1000 docs) |
| Should `kb_context` support batch queries? | Token savings for multi-doc briefing | Single-doc only in initial build; batch as future enhancement |

## Future Roadmap

(From brainstorm — each is a separate phase after initial build)

1. **Hybrid search** — enable memvid `vec` feature for ONNX embeddings + HNSW
2. **Knowledge capture tools** — `kb_capture_session`, `kb_capture_fix`, `kb_classify`
3. **HTTP daemon mode** — long-lived server, eliminates MCP cold starts
4. **Cross-agent knowledge sharing** — federated `.mv2` files across projects

## Sources & References

### Origin

- **Brainstorm document:** [docs/brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md](../../brainstorms/archived/2026-03-18-kb-mcp-v2-brainstorm.md) — key decisions carried forward: memvid-core as dependency (not cherry-pick), per-collection config model, `kb_context` + `kb_write` in initial scope

### External References

- [memvid-core on crates.io](https://crates.io/crates/memvid-core) (v2.0.139)
- [rmcp on crates.io](https://crates.io/crates/rmcp) (v1.2.0, official MCP Rust SDK)
- [ron on crates.io](https://crates.io/crates/ron) (v0.12.0)
- [memvid-core docs.rs](https://docs.rs/memvid-core)
- [rmcp docs.rs](https://docs.rs/rmcp/1.2.0)

### Inspiration Sources

- **knowledge-base-server** — TypeScript MCP server with `kb_context` briefing pattern, writable collections, content hashing
- **memvid** — Rust library providing `.mv2` storage, smart chunking, feature gating patterns
