---
title: "refactor: Split kb-mcp into cargo workspace (kb-core, kb-cli, kb-mcp-server)"
type: refactor
status: completed
date: 2026-03-30
origin: docs/brainstorms/2026-03-30-cargo-workspace-split-brainstorm.md
---

# refactor: Split kb-mcp into cargo workspace

## Overview

Reorganize kb-mcp from a single binary crate into a cargo workspace with three
crates: `kb-core` (library), `kb-cli` (binary), `kb-mcp-server` (binary).
Eliminates CLI/MCP code duplication, guarantees behavioral parity, and enables
independent compilation.

## Problem Statement / Motivation

The single-crate architecture has outgrown its design. Ten tools now have
parallel implementations in `cli.rs` and `tools/*.rs` with no shared command
layer. The `write` command duplicates 90+ lines. `auto_reindex` only runs in
MCP mode. `find_available_path()` is copy-pasted between modules. A bug fix in
one transport may not propagate to the other.

The v2 plan (2026-03-18) correctly rejected splitting as YAGNI. Since then, 10
tools with CLI parity shipped, duplication is concrete, and the v2 roadmap
mentions HTTP daemon mode as a future transport — a natural third binary.

(See brainstorm: `docs/brainstorms/2026-03-30-cargo-workspace-split-brainstorm.md`)

## Proposed Solution

Incremental two-phase migration:

- **Phase 1**: Extract `kb-core` as a library crate. Keep the existing single
  binary working against it. Workspace root is both `[workspace]` and
  `[package]` member.
- **Phase 2**: Split the binary into `kb-cli` and `kb-mcp-server`. Remove the
  root-level binary. Workspace root becomes `[workspace]` only.

Each phase is independently compilable and shippable.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Directory layout | `crates/` nesting | Keeps root clean as crate count grows |
| Binary names | `kb-mcp` (server) + `kb` (CLI) | MCP clients keep working (zero breakage); CLI gets shorter name |
| Shared logic | Direct core calls, loose args | No intermediate abstraction, no Command trait, no ops module |
| Init pattern | `AppContext` struct with owned values | Each binary wraps differently (CLI: direct, MCP: Arc/RwLock) |
| Params structs | Core takes loose args, not structs | MCP keeps `JsonSchema` params, CLI keeps `clap` args, both call core with individual parameters |
| Feature flag | `hybrid` on kb-core, passthrough from binaries | `kb-mcp-server = { features = ["hybrid"] }` enables `kb-core/hybrid` |
| Version management | `workspace.package.version` inheritance | All crates share version, `CARGO_PKG_VERSION` stays consistent |
| Migration approach | Incremental (2 phases) | Each phase compilable, never breaks the build |

## Technical Approach

### Architecture

#### Final State (after Phase 2)

```
kb-mcp/
├── Cargo.toml                  # [workspace] members = ["crates/*"]
├── crates/
│   ├── kb-core/
│   │   ├── Cargo.toml          # [lib] name = "kb_core"
│   │   └── src/
│   │       ├── lib.rs          # AppContext, init(), re-exports
│   │       ├── types.rs        # Document, Section
│   │       ├── config.rs       # RON config loading, resolution chain
│   │       ├── index.rs        # Filesystem scanning, frontmatter parsing
│   │       ├── store.rs        # .mv2 lifecycle, sync, hashing
│   │       ├── search.rs       # SearchEngine, BM25/hybrid
│   │       ├── format.rs       # All output formatters (shared contract)
│   │       ├── query.rs        # matches_query() extracted from tools/query.rs
│   │       └── write.rs        # slugify_title(), find_available_path(), write_document()
│   ├── kb-cli/
│   │   ├── Cargo.toml          # [[bin]] name = "kb"
│   │   └── src/
│   │       └── main.rs         # Clap args → kb_core::* calls → JSON stdout
│   └── kb-mcp-server/
│       ├── Cargo.toml          # [[bin]] name = "kb-mcp"
│       └── src/
│           ├── main.rs         # MCP stdio server startup
│           ├── server.rs       # KbMcpServer, auto-reindex, ServerHandler
│           └── tools/          # One file per tool (rmcp wrappers)
│               ├── mod.rs
│               ├── sections.rs
│               ├── documents.rs
│               ├── search.rs
│               ├── context.rs
│               ├── write.rs
│               ├── reindex.rs
│               ├── digest.rs
│               ├── query.rs
│               ├── export.rs
│               └── health.rs
├── book/
├── docs/
└── tests/                      # Workspace-level integration tests
```

#### Phase 1 Intermediate State

```
kb-mcp/
├── Cargo.toml                  # [workspace] + [package] (root is a member)
├── src/                        # Existing binary, imports from kb_core
│   ├── main.rs                 # Uses kb_core::AppContext::init()
│   ├── cli.rs                  # Delegates to kb_core::format::*, etc.
│   ├── server.rs               # KbMcpServer wraps kb_core types
│   └── tools/                  # rmcp tool wrappers call kb_core::*
├── crates/
│   └── kb-core/
│       ├── Cargo.toml
│       └── src/
└── tests/
```

#### Dependency Graph

```
kb-cli ──────────┐
                  ├──→ kb-core
kb-mcp-server ───┘
```

- `kb-core` depends on: anyhow, blake3, chrono, dirs, memvid-core, regex, ron,
  serde, serde_json, serde_yaml, sha2, tokio
- `kb-cli` depends on: kb-core, clap, anyhow, tokio
- `kb-mcp-server` depends on: kb-core, rmcp, schemars, anyhow, tokio, tracing,
  tracing-subscriber

### AppContext Design

```rust
// kb-core/src/lib.rs

/// Bundled application state from initialization.
/// Returns owned values — each binary wraps as needed
/// (CLI uses directly, MCP server wraps in Arc/RwLock).
pub struct AppContext {
    pub index: Index,
    pub search_engine: SearchEngine,
    pub collections: Vec<ResolvedCollection>,
    pub cache_dir: PathBuf,
    #[cfg(feature = "hybrid")]
    pub embedder: Arc<LocalTextEmbedder>,
}

/// Initialize the full application: load config, build index, sync stores,
/// create search engine.
pub async fn init(config_path: Option<&Path>) -> Result<AppContext> {
    let (config, collections) = load_config(config_path)?;
    let cache_dir = ensure_cache_dir(&config)?;
    let index = Index::build(&collections)?;
    // ... sync stores, create engine ...
    Ok(AppContext { index, search_engine, collections, cache_dir })
}
```

### Core Function Signatures (Loose Args Pattern)

Core functions take individual parameters, not params structs. Both CLI (clap
args) and MCP (JsonSchema params) destructure into these calls:

```rust
// kb-core/src/format.rs (existing pattern, already works this way)
pub fn format_search(documents: &[Document], results: &[SearchResult]) -> String
pub fn format_document(doc: &Document, body: &str) -> String
pub fn format_digest(documents: &[Document], collections: &[ResolvedCollection]) -> String

// kb-core/src/search.rs
impl SearchEngine {
    pub fn search(&self, query: &str, collection: Option<&str>,
                  scope: Option<&str>, max_results: usize) -> Vec<SearchResult>
}

// kb-core/src/write.rs (extracted from tools/write.rs + cli.rs)
pub fn slugify_title(title: &str) -> String
pub fn find_available_path(dir: &Path, filename: &str) -> PathBuf
pub fn write_document(collection: &ResolvedCollection, title: &str,
                      section: Option<&str>, content: &str, tags: &[String],
                      filename: Option<&str>) -> Result<PathBuf>

// kb-core/src/query.rs (extracted from tools/query.rs)
pub fn matches_query(doc: &Document, tags: &[String], status: Option<&str>,
                     created_after: Option<&str>, created_before: Option<&str>) -> bool
```

### Feature Flag Flow

```
kb-mcp-server/Cargo.toml:
  [features]
  hybrid = ["kb-core/hybrid"]

kb-core/Cargo.toml:
  [features]
  hybrid = ["memvid-core/vec"]

kb-cli/Cargo.toml:
  [features]
  hybrid = ["kb-core/hybrid"]
```

The `#[cfg(feature = "hybrid")]` annotations stay in place within each crate.
Server and CLI must enable the feature in lockstep with core. Workspace-level
`--features` does NOT work in Cargo — use `-p` flag:
`cargo build -p kb-mcp-server --features hybrid`.

### Coupling Seams Requiring Surgery

| Seam | Current State | Resolution |
|------|--------------|------------|
| `format::extract_summary()` ← `search.rs` | Both in same crate | Both move to kb-core together; resolves naturally |
| `tools::query::matches_query()` ← `cli.rs` | Function in tools/ called from CLI | Extract to `kb-core/src/query.rs` |
| `tools::write::slugify_title()` ← `cli.rs` | Function in tools/ called from CLI | Extract to `kb-core/src/write.rs` |
| `tools::write::find_available_path()` ≅ `cli.rs` | Duplicated in both modules | Unify in `kb-core/src/write.rs` |
| `server::read_fresh()` → `format::read_document_body()` | Server calls format fn | `read_document_body` moves to core; `read_fresh` stays in MCP server |
| `main.rs::sync_stores()` | Inline helper in main | Move to `kb-core/src/store.rs` as `pub fn sync_all_stores()` |

## Implementation Phases

### Phase 0: Safety Net (Pre-migration)

Write parity tests before touching the structure. Currently there are zero
tests for format output, config resolution, CLI commands, or MCP/CLI
equivalence.

**Tasks:**

- [ ] Create test fixture: small `collections.ron` + 3-4 markdown files with
  frontmatter in `tests/fixtures/`
- [ ] Add integration test: `list-sections` produces expected JSON
- [ ] Add integration test: `search --query` returns expected results
- [ ] Add integration test: `get-document` returns fresh content
- [ ] Add integration test: `write` creates file with correct frontmatter/path
- [ ] Add integration test: `query --tag` filters correctly
- [ ] Add integration test: config resolution chain (explicit path, env var,
  CWD fallback)

**Verify:** `cargo test` — all new tests pass against current single-crate structure.

### Phase 1: Extract kb-core

Create the library crate and move shared modules. The existing binary stays at
the workspace root, importing from `kb-core` instead of local `mod` declarations.

**Tasks:**

- [ ] Create `crates/kb-core/Cargo.toml` with shared dependencies
  → verify: `cargo check -p kb-core`
- [ ] Create `crates/kb-core/src/lib.rs` with module declarations and re-exports
  → verify: `cargo check -p kb-core`
- [ ] Move `src/types.rs` → `crates/kb-core/src/types.rs`
  → verify: `cargo check`
- [ ] Move `src/config.rs` → `crates/kb-core/src/config.rs`
  → verify: `cargo check`
- [ ] Move `src/store.rs` → `crates/kb-core/src/store.rs` (with unit tests)
  → verify: `cargo test -p kb-core`
- [ ] Move `src/index.rs` → `crates/kb-core/src/index.rs`
  → verify: `cargo check`
- [ ] Move `src/search.rs` → `crates/kb-core/src/search.rs`
  → verify: `cargo check`
- [ ] Move `src/format.rs` → `crates/kb-core/src/format.rs`
  → verify: `cargo check`
- [ ] Extract `matches_query()` from `src/tools/query.rs` → `crates/kb-core/src/query.rs`
  → verify: `cargo check`
- [ ] Extract `slugify_title()`, `find_available_path()`, `write_document()` from
  `src/tools/write.rs` + `src/cli.rs` → `crates/kb-core/src/write.rs`
  → verify: `cargo check`
- [ ] Extract `sync_stores()` from `src/main.rs` → `crates/kb-core/src/store.rs`
  → verify: `cargo check`
- [ ] Create `AppContext` struct and `init()` function in `crates/kb-core/src/lib.rs`
  → verify: `cargo check`
- [ ] Update root `Cargo.toml` to `[workspace]` + `[package]` with
  `kb-core = { path = "crates/kb-core" }` dependency
  → verify: `cargo build`
- [ ] Update root `src/main.rs` to use `kb_core::AppContext::init()`
  → verify: `cargo run -- list-sections`
- [ ] Update root `src/cli.rs` to call `kb_core::*` functions instead of local modules
  → verify: `cargo run -- search --query "test"`
- [ ] Update root `src/server.rs` and `src/tools/*.rs` to use `kb_core::*`
  → verify: MCP server starts and responds to tools
- [ ] Set up `hybrid` feature flag passthrough
  → verify: `cargo check --features hybrid`
- [ ] Run full test suite
  → verify: `cargo test` — all Phase 0 tests still pass

**Root Cargo.toml (Phase 1):**

```toml
[workspace]
members = ["crates/*"]
resolver = "3"

[workspace.package]
version = "0.3.0"
edition = "2024"
license = "MIT"

[package]
name = "kb-mcp"
version.workspace = true
edition.workspace = true

[dependencies]
kb-core = { path = "crates/kb-core" }
clap = { version = "4", features = ["derive"] }
rmcp = { version = "1.1", features = ["server", "transport-io", "schemars"] }
schemars = "0.8"
# ... remaining MCP + CLI deps
```

### Phase 2: Split Binaries

Remove the root-level binary. Create `kb-cli` and `kb-mcp-server` as separate
crates under `crates/`.

**Tasks:**

- [ ] Create `crates/kb-cli/Cargo.toml` with `[[bin]] name = "kb"`
  → verify: `cargo check -p kb-cli`
- [ ] Move CLI logic from root `src/cli.rs` → `crates/kb-cli/src/main.rs`
  → verify: `cargo run -p kb-cli -- list-sections`
- [ ] Create `crates/kb-mcp-server/Cargo.toml` with `[[bin]] name = "kb-mcp"`
  → verify: `cargo check -p kb-mcp-server`
- [ ] Move MCP logic from root `src/{main.rs,server.rs,tools/}` →
  `crates/kb-mcp-server/src/`
  → verify: `cargo run -p kb-mcp-server` (MCP stdio starts)
- [ ] Remove root `[package]` section and `src/` directory
  → verify: `cargo build` builds both binaries
- [ ] Update justfile commands for workspace
  → verify: `just build`, `just test`, `just clippy`, `just install`
- [ ] Update Dockerfile for workspace binary paths
  → verify: `docker build` succeeds
- [ ] Update `docker-compose.yml` if binary name references changed
  → verify: Docker agent starts
- [ ] Update `book/` documentation for new binary names and install commands
  → verify: `just book-build`
- [ ] Update CLAUDE.md architecture section and common commands
  → verify: CLAUDE.md reflects new structure
- [ ] Move `tests/memvid_smoke.rs` to workspace-level or kb-core tests
  → verify: `cargo test` in workspace
- [ ] Run full test suite
  → verify: `cargo test` — all parity tests pass

**Justfile Updates:**

```just
build:
    cargo build --workspace

release:
    cargo build --release --workspace

check:
    cargo check --workspace

clippy:
    cargo clippy --workspace

test:
    cargo test --workspace

install:
    cargo install --path crates/kb-mcp-server
    cargo install --path crates/kb-cli

install-server:
    cargo install --path crates/kb-mcp-server

install-cli:
    cargo install --path crates/kb-cli

run-cli *args:
    cargo run -p kb-cli -- {{args}}

run-server:
    cargo run -p kb-mcp-server
```

### Phase 3: Cleanup and Polish

- [ ] Remove any leftover `pub(crate)` that should now be `pub` in kb-core
  → verify: `cargo clippy --workspace`
- [ ] Ensure all kb-core public types and functions have `///` doc comments
  → verify: `cargo doc -p kb-core --no-deps`
- [ ] Add module-level `//!` docs to each kb-core source file
- [ ] Version bump to 0.3.0 across workspace
  → verify: `cargo run -p kb-mcp-server` reports correct version

## System-Wide Impact

### Interaction Graph

`cargo install --path .` → breaks (workspace root has no binary after Phase 2).
Must use `cargo install --path crates/kb-mcp-server` and `crates/kb-cli`.

`.mcp.json` → `"command": "kb-mcp"` → unchanged (server binary name preserved).

CLI scripts → `kb-mcp list-sections` → breaks, becomes `kb list-sections`.

Justfile → all recipes updated in Phase 2.

Docker → Dockerfile COPY path updated in Phase 2.

### Error Propagation

Config resolution errors propagate from `kb-core::init()` through to both
binaries identically. No new error paths introduced.

### State Lifecycle Risks

The `Arc<RwLock<Index>>` wrapping in the MCP server happens *after* `AppContext`
returns owned values. No risk of the CLI accidentally sharing mutable state.

### API Surface Parity

After the split, parity is guaranteed by construction: both binaries call the
same `kb_core::format::*` functions. The learnings document
(`docs/solutions/feature-patterns/vault-intelligence-bundle-learnings.md`)
confirms this pattern: "One implementation, two interfaces."

## Acceptance Criteria

### Functional

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes all parity tests from Phase 0
- [ ] `kb list-sections` produces identical JSON to current `kb-mcp list-sections`
- [ ] `kb search --query "test"` produces identical results
- [ ] `kb-mcp` (MCP server) starts via stdio, responds to all 10 tools
- [ ] `.mcp.json` with `"command": "kb-mcp"` works without changes
- [ ] `cargo build --workspace --features hybrid` compiles
- [ ] `just install` installs both `kb` and `kb-mcp` binaries
- [ ] Docker build produces working server image

### Non-Functional

- [ ] `kb-core` has zero dependency on `rmcp`, `schemars`, or `clap`
- [ ] `kb-cli` has zero dependency on `rmcp` or `schemars`
- [ ] `kb-mcp-server` has zero dependency on `clap`
- [ ] No code duplication between CLI and MCP tool implementations
- [ ] All kb-core public items have doc comments

## Dependencies & Risks

| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| Binary rename breaks CLI scripts | High | Document migration; `kb-mcp` preserved for MCP clients |
| `cargo install` workflow breaks | High | Update README, justfile, book before merge |
| Feature flag mismatch between crates | Medium | Compile-time safe; document lockstep requirement |
| Intermediate Phase 1 state is awkward | Low | Phase 1 is temporary; Phase 2 follows immediately |
| Parity regression during migration | Medium | Phase 0 parity tests catch this |

## Sources & References

### Origin

- **Brainstorm:** [docs/brainstorms/2026-03-30-cargo-workspace-split-brainstorm.md](docs/brainstorms/2026-03-30-cargo-workspace-split-brainstorm.md)
  — Key decisions: crates/ layout, direct core calls, AppContext, incremental migration
- **Inspiration:** [crabtalk/crabtalk-mcp](https://github.com/crabtalk/crabtalk-mcp)
  pattern of MCP-as-thin-wrapper-around-core-library

### Internal References

- Prior rejection of split: `docs/plans/archived/2026-03-18-feat-kb-mcp-v2-standalone-crate-plan.md`
  ("YAGNI — no one needs to embed kb-mcp as a library yet")
- Parity lessons: `docs/solutions/feature-patterns/vault-intelligence-bundle-learnings.md`
  ("One implementation, two interfaces" rule)
- Architecture: CLAUDE.md (Hard Rules, tool pattern, dual-mode design)

### Key Files

- `Cargo.toml` — current single-crate manifest
- `src/main.rs:1-163` — dual-mode entry, shared init (duplication target)
- `src/cli.rs:1-434` — CLI with duplicated tool logic (largest file)
- `src/tools/write.rs:1-230` — most duplicated tool
- `src/tools/query.rs:1-123` — `matches_query` called from CLI
- `src/format.rs:1-678` — shared output contract (the model for extraction)
- `src/server.rs:1-190` — MCP server with auto-reindex
- `justfile` — build recipes needing workspace updates
- `Dockerfile` — binary copy path needing update
