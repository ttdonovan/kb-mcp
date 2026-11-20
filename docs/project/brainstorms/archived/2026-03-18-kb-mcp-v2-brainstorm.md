# Brainstorm: kb-mcp v2 — memvid-core Integration

**Date:** 2026-03-18
**Status:** Design complete

## What We're Building

Rewrite kb-mcp's storage and search layer on top of `memvid-core`, the
Rust crate that provides single-file `.mv2` persistent storage with
Tantivy BM25, crash-safe WAL, and smart chunking. Add flexible
per-collection configuration, a token-efficient context/briefing tool,
and write-back capability for writable collections.

**Context:** kb-mcp v1 was a ~600-line Rust MCP server that indexed
two hardcoded markdown directories (~124 docs) into an in-memory Tantivy
index rebuilt on every startup. It exposed 4 read-only MCP tools via
rmcp (stdio transport).

This brainstorm combines a prior enhancement roadmap with new learnings
from two reference projects:

- **memvid** — 63K-line Rust library; single-file `.mv2` memory with
  Tantivy BM25, HNSW vector search, smart chunking, local ONNX
  embeddings, crash-safe WAL. Published as `memvid-core` crate with
  aggressive feature gating.
- **knowledge-base-server** — TypeScript MCP server with `kb_context`
  (token-efficient briefings), `kb_write` (vault write-back),
  classification pipeline, incremental indexing. Not a code dependency
  but a source of workflow patterns.

## Why This Approach

**memvid-core as a dependency** was chosen over:

- **Cherry-pick patterns only** — memvid's storage layer, chunking, and
  Tantivy integration are exactly what kb-mcp needs. Reimplementing them
  means maintaining the same code ourselves for no benefit.
- **Replace kb-mcp entirely** — memvid has no MCP server; kb-mcp's MCP
  layer (rmcp, tools, CLI mode) is still needed on top.
- **Stay with in-memory index** — rebuilding on every startup is
  architecturally wasteful and blocks future features (vector search,
  incremental reindex).

**Key insight:** memvid-core is heavily feature-gated. With
`default-features = false, features = ["lex"]`, it pulls in essentially
the same dependencies kb-mcp already has (Tantivy, serde, compression).
ONNX, embeddings, audio, image, and encryption are all behind optional
feature flags — zero cost if unused.

## Key Decisions

| Decision | Choice | Alternatives Considered |
|----------|--------|------------------------|
| Storage backend | memvid-core `.mv2` files | Tantivy MmapDirectory, SQLite FTS5 |
| Dependency strategy | memvid-core with `lex` feature only | Full memvid-core, cherry-pick patterns, self-contained |
| Collection config | Per-collection in config with `writable` flag | Hardcoded paths, compile-time feature gate |
| Write-back | Per-collection `writable` flag in config | Read-only forever, compile-time feature gate |
| Token efficiency | `kb_context` tool returning frontmatter + summary | Search excerpts only |
| Vector search | Future phase — enable memvid `vec` feature | Include now, never |
| MCP transport | Keep rmcp stdio (existing) | Add HTTP transport |

## Scope: Initial Build

### Replace storage layer with memvid-core

Swap the in-memory Tantivy index for memvid-core's persistent `.mv2`
storage. Each collection gets its own `.mv2` file stored at
`~/.cache/kb-mcp/<collection-name>.mv2`.

On startup: open existing `.mv2` files. Incremental reindex only
processes changed files (content hashing). Cold start goes from
"rebuild everything" to "open file, ready."

memvid-core's smart chunking improves search precision on long
files — a query about "rate limits" hits the relevant section of a
long overview page, not the entire document.

### Flexible collections via configuration

Replace hardcoded directory paths with a collection model:

```ron
(
    collections: [
        (
            name: "docs",
            path: "docs",
            description: "Project documentation",
            writable: false,
            sections: [
                (prefix: "guides", description: "How-to guides"),
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

Cross-project use (via `KB_MCP_CONFIG` env var) allows any project to
configure its own collections:

```ron
(
    collections: [
        (
            name: "community",
            path: "vaults/community",
            description: "Community-contributed knowledge",
            writable: false,
            sections: [],
        ),
        (
            name: "knowledge",
            path: "vaults/knowledge",
            description: "Agent-curated knowledge",
            writable: true,
            sections: [],
        ),
    ],
)
```

### Add `kb_context` tool (token-efficient briefing)

New MCP tool that returns frontmatter metadata and first paragraph
(or summary if available) without the full document body. Pattern
from knowledge-base-server's `kb_context` — agents call this first
to survey relevance, then selectively `get_document` for full content.

Saves 90%+ tokens on retrieval-heavy workflows where most search
results are scanned but not read.

### Add `kb_write` tool (writable collections)

New MCP tool for creating notes with proper frontmatter. Only works on
collections marked `writable = true` in config. Returns an error for
read-only collections.

Accepts: title, tags, body content, optional source/status fields.
Generates: properly formatted file with frontmatter, date prefix
filename, written to the collection's directory.

The `.mv2` index updates incrementally after write (no full reindex).

### Updated MCP tool surface

| Tool | Status | Description |
|------|--------|-------------|
| `list_sections` | Keep | List collections with doc counts and descriptions |
| `search` | Rewrite | Full-text search via memvid-core (BM25 + chunking) |
| `get_document` | Keep | Retrieve full document content by path or title |
| `kb_context` | New | Token-efficient briefing (frontmatter + summary) |
| `kb_write` | New | Write note to a writable collection |
| `reindex` | Rewrite | Incremental reindex via content hashing |

## Future Enhancement Roadmap

Ordered by value. Each is a separate phase after the initial build.

### Phase 2: Hybrid search (enable memvid `vec` feature)

Add `vec` feature to memvid-core dependency. This pulls in ONNX runtime
and HNSW — the "heavy" deps, but only when opted in.

- Local ONNX embeddings (BGE-small-en-v1.5, 384 dims)
- HNSW vector index stored in the `.mv2` file alongside Tantivy
- Hybrid ranking: BM25 + vector similarity via RRF fusion
- New `search_smart` tool (or parameter on `search`)

**When to do it:** When keyword search consistently fails on conceptual
queries ("how do agents share state?" not matching "MCP Server Pattern").

### Phase 3: Knowledge capture tools

Specialized write tools inspired by knowledge-base-server:

- `kb_capture_session` — record debugging/coding sessions
- `kb_capture_fix` — record bug fixes with symptom/cause/resolution
- `kb_classify` — auto-tag unprocessed notes (type, tags, summary)

**When to do it:** When agents are actively writing to knowledge vaults
and would benefit from structured capture templates.

### Phase 4: HTTP daemon mode

Add HTTP transport alongside stdio. Long-lived server process eliminates
MCP cold starts. Becomes valuable when vector search makes startup
expensive (model loading).

### Phase 5: Cross-agent knowledge sharing

Multiple projects share a single kb-mcp instance (or federated `.mv2`
files). Agents in different repos contribute to and query from shared
knowledge. This is the long-term vision for "knowledge sharing between
agents and memory retention."

## Patterns Adopted from knowledge-base-server

These TypeScript patterns inform our Rust implementation:

| Pattern | Adoption |
|---------|----------|
| `kb_context` (briefing tool) | Direct — implement as MCP tool |
| Per-collection writable flag | Direct — in config |
| Content hashing for incremental reindex | Direct — memvid handles this |
| `kb_write` with frontmatter generation | Direct — implement as MCP tool |
| Classification pipeline | Roadmap (Phase 3) |
| Session/fix capture templates | Roadmap (Phase 3) |
| Safety check tool | Not adopting — different use case |
| Dashboard UI | Not adopting — CLI + MCP is sufficient |

## Patterns Adopted from memvid

| Pattern | Adoption |
|---------|----------|
| `.mv2` single-file persistent storage | Direct — replace in-memory index |
| Tantivy BM25 (feature-gated) | Direct — already using Tantivy |
| Smart markdown chunking | Direct — improves search precision |
| Crash-safe WAL | Direct — comes with `.mv2` storage |
| HNSW vector search (feature-gated) | Roadmap (Phase 2) |
| Local ONNX embeddings | Roadmap (Phase 2) |
| RAG/Ask capabilities | Not adopting — agents handle synthesis |
| Entity graph (logic_mesh) | Not adopting — overkill for vault |
| Encryption capsules | Not adopting — vault is local |

## Open Questions

*None — all resolved during brainstorm.*

## Resolved Questions

| Question | Resolution |
|----------|------------|
| memvid-core dependency weight? | Minimal with feature gating — `lex` only pulls Tantivy (same as current) |
| Write-back model? | Per-collection `writable` flag in config |
| Token-efficient retrieval? | Include `kb_context` tool in initial build |
| Initial scope? | Storage + search + context + write; vector search as Phase 2 |
| Index backend? | memvid-core `.mv2` (supersedes prior Tantivy MmapDirectory vs SQLite FTS5 debate) |
| Strategy? | memvid-core as dependency, not cherry-pick (prior brainstorm leaned cherry-pick) |
