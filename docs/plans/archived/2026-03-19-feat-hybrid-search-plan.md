---
title: "feat: Hybrid BM25 + vector search via memvid-core vec feature"
type: feat
status: completed
date: 2026-03-19
origin: docs/brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md
---

# feat: Hybrid BM25 + vector search via memvid-core vec feature

## Overview

Enable memvid-core's `vec` feature to add HNSW vector similarity search
alongside the existing BM25 keyword search. Hybrid ranking via Reciprocal
Rank Fusion (RRF) combines both signals so conceptual queries ("how do
agents share state?") match documents that don't contain the exact
keywords ("shared memory").

## Problem Statement / Motivation

The researcher agent and human users ask conceptual questions that BM25
can't answer well. BM25 excels at exact keyword matching but misses
semantic similarity. With ~130 documents growing via the researcher agent,
this gap will widen as the vault covers more interconnected topics.

(See brainstorm Phase 2: `docs/brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md`)

## Proposed Solution

Add vector embeddings at ingest time and hybrid search at query time,
using memvid-core's existing infrastructure:

**Ingest:** Use `LocalTextEmbedder` (ONNX, BGE-small-en-v1.5, 384 dims)
to embed document body text, then call `put_with_embedding()` instead of
`put_bytes_with_options()`.

**Search:** Use `Memvid::ask()` with `AskMode::Hybrid` instead of
`Memvid::search()`. This runs BM25 + vector search and fuses results
via RRF (k=60).

**Key API insight from research:** `Memvid::search()` is BM25-only
regardless of features enabled. Hybrid search exclusively goes through
`Memvid::ask()` with a `VecEmbedder` implementation for query-time
embedding.

## Technical Approach

### Dependency Changes

```toml
# Cargo.toml — add vec feature
memvid-core = { version = "2.0", default-features = false, features = ["lex", "vec"] }
```

This pulls in: `ort` (ONNX Runtime), `hnsw`, `ndarray`, `tokenizers`,
`space`, `rand`. Build time increases significantly due to ONNX Runtime
C++ compilation.

### ONNX Model Delivery

The BGE-small-en-v1.5 model files must be present at
`~/.cache/memvid/text-models/` (or `.memvid-cache/text-models/` fallback):

- `bge-small-en-v1.5.onnx` (~33MB)
- `bge-small-en-v1.5_tokenizer.json` (~700KB)

**Host (dev):** Download once via curl commands (documented in README).

**Container:** Either bake into the Docker image (adds ~34MB) or download
in the entrypoint. Baking in is simpler and avoids network dependency at
runtime.

### Embedding at Ingest Time (`src/store.rs`)

Replace `put_bytes_with_options()` with `put_with_embedding()`:

```rust
// Current (BM25 only)
mem.put_bytes_with_options(doc.body.as_bytes(), options)?;

// New (BM25 + vector)
let embedding = embedder.embed_text(&doc.body)?;
mem.put_with_embedding_and_options(doc.body.as_bytes(), embedding, options)?;
```

The `LocalTextEmbedder` is created once at startup and passed to the
ingest functions. It uses an LRU cache (1000 entries) and auto-unloads
after 5 minutes idle.

**Existing .mv2 files:** Documents ingested without embeddings won't
have vector index entries. Options:
1. Force full re-ingest on first hybrid-enabled startup (delete .mv2 + .hashes)
2. Use memvid's enrichment worker to retroactively embed existing frames
3. Accept that old docs are BM25-only until next reindex

Option 1 is simplest and the vault is small enough (<200 docs) that
full re-ingest takes seconds.

### Hybrid Search at Query Time (`src/search.rs`)

Replace `Memvid::search(SearchRequest)` with `Memvid::ask(AskRequest)`:

```rust
use memvid_core::{AskRequest, AskMode};

// NOTE: AskRequest may not implement Default — verify during Phase 3.
// If not, construct all fields explicitly (same pattern as SearchRequest).
let response = mem.ask(AskRequest {
    question: query_str.to_string(),
    mode: AskMode::Hybrid,  // or AskMode::Lex for BM25-only
    top_k: max_results * 3,
    snippet_chars: 300,
    // ... remaining fields TBD from AskRequest struct definition
}, Some(&embedder))?;
```

`AskMode` controls the strategy:
- `AskMode::Lex` — BM25 only (current behavior)
- `AskMode::Sem` — semantic re-ranking of BM25 results
- `AskMode::Hybrid` — full RRF fusion of BM25 + vector candidate lists

### Search Tool Behavior

When the `hybrid` feature is enabled, the search tool automatically uses
hybrid mode (`AskMode::Hybrid`). No new parameter needed — agents get
better results without changing their queries. The existing `search` tool
description stays the same.

A `mode` parameter (keyword/hybrid/semantic) can be added later if users
need explicit control. For now, hybrid-when-available is the right default.

### Snippet Improvement

~~The fallback excerpt used the first 3 lines (heading + blank + partial
sentence).~~ **Fixed** — fallback now uses `format::extract_summary()`
which returns the first paragraph after the H1 heading (same as
`kb_context`).

With hybrid search, also prefer memvid's `hit.text` snippet when
available since it contains the actual matched chunk:

```rust
// Use memvid snippet when available, fall back to extract_summary()
let excerpt = if !hit.text.is_empty() {
    hit.text.clone()
} else {
    crate::format::extract_summary(&doc.body)
};
```

### VecEmbedder Adapter

memvid's `ask()` takes `Option<&dyn VecEmbedder>` for query-time embedding.
`LocalTextEmbedder` implements `EmbeddingProvider`, not `VecEmbedder`
directly. We need a thin adapter:

```rust
struct KbVecEmbedder {
    embedder: LocalTextEmbedder,
}

impl VecEmbedder for KbVecEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.embedder.embed_text(text)
    }
    fn dims(&self) -> usize { 384 }
}
```

### Feature Gating

Make hybrid search opt-in via a cargo feature on kb-mcp itself:

```toml
[features]
default = []
hybrid = ["memvid-core/vec"]
```

This way the default build stays lightweight (BM25 only). Users who want
hybrid search compile with `cargo build --features hybrid`.

## Acceptance Criteria

- [x] `memvid-core` vec feature compiles when enabled
- [x] ONNX model downloads documented (curl commands in README + book)
- [x] Documents ingested with embeddings via `put_with_embedding()`
- [x] `search` tool automatically uses hybrid mode when vec feature enabled
- [ ] Conceptual query "how do agents share state?" matches "shared memory" doc *(needs ONNX model to test)*
- [x] Keyword query "BM25" still works correctly (no regression)
- [ ] First startup with vec enabled re-ingests all docs to build vector index *(needs ONNX model to test)*
- [x] Snippet fallback fixed (uses extract_summary instead of take(3))
- [x] Snippet uses memvid hit.text when available (both modes)
- [x] Container Dockerfile updated with ONNX model files (build-arg gated)
- [x] `cargo build` (no features) still works — BM25 only, no ONNX deps
- [x] `cargo clippy` clean (both default and hybrid)

## Implementation Phases

### Phase 1: Feature flag + dependency

1. Add `hybrid` feature to kb-mcp's Cargo.toml gating `memvid-core/vec`
2. Verify `cargo build --features hybrid` compiles
3. Document ONNX model download in README

### Phase 2: Embed at ingest time

1. Create `LocalTextEmbedder` wrapper in `store.rs`
2. Update `ingest_document_refs()` to call `put_with_embedding()`
3. Gate embedding code behind `#[cfg(feature = "hybrid")]`
4. On first hybrid startup, force full re-ingest (detect missing vector index)
5. Verify: .mv2 files contain vector data

### Phase 3: Hybrid search

1. Verify `AskRequest` struct fields — construct explicitly if no Default
2. Create `VecEmbedder` adapter for query-time embedding
3. Replace `Memvid::search()` with `Memvid::ask()` when hybrid enabled
4. Use memvid `hit.text` for snippets
5. Gate behind `#[cfg(feature = "hybrid")]` with BM25 fallback
6. Verify: conceptual queries return relevant results

### Phase 4: Container + docs

1. Add ONNX model download to Dockerfile (bake into image)
2. Update researcher agent config examples
3. Update docs/ARCHITECTURE.md and docs/ROADMAP.md
4. Add hybrid search smoke test
5. Verify: researcher agent benefits from hybrid search

## Dependencies & Prerequisites

- **memvid-core** v2.0+ with `vec` feature
- **BGE-small-en-v1.5** ONNX model files (~34MB total)
- ONNX Runtime (transitive via `ort` crate)

## Open Questions

| Question | Impact | Default if unresolved |
|----------|--------|-----------------------|
| Does `AskRequest` implement Default? | Code examples use `..Default::default()` — if not, need explicit construction | Verify in Phase 3; `SearchRequest` didn't, so likely not |
| Does `ask()` require `&mut self`? | Affects Mutex contention (same as `search()`) | Likely yes — verify in Phase 3 |
| Can `LocalTextEmbedder` be shared across collections? | One embedder vs per-collection | One shared embedder (model is stateless) |
| Should `ask()` response include BM25 score + vector score separately? | Debugging hybrid ranking | Use fused score only; log components at debug level |
| ONNX Runtime binary size impact on Docker image? | Container size | Accept it — model quality is worth ~100MB |

## Sources & References

### Origin

- **Brainstorm Phase 2:** [docs/brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md](../../brainstorms/archived/2026-03-18-kb-mcp-v2-brainstorm.md) — HNSW + ONNX embeddings, RRF fusion, `search_smart` tool

### Key Files

- `src/search.rs` — current BM25 search wrapper, main change target
- `src/store.rs` — ingest pipeline, add embedding calls
- `Cargo.toml` — feature flag addition
- `sandbox/memvid/src/memvid/ask.rs` — hybrid search via `ask()` API
- `sandbox/memvid/src/text_embed.rs` — `LocalTextEmbedder`, model registry
- `sandbox/memvid/src/vec.rs` — HNSW index, brute-force fallback

### API Reference

- `Memvid::ask(AskRequest, Option<&dyn VecEmbedder>)` — hybrid BM25+vec search
- `AskMode::Hybrid` / `AskMode::Lex` / `AskMode::Sem` — search strategy
- `LocalTextEmbedder::new(TextEmbedConfig)` — ONNX embedder (BGE-small-en-v1.5)
- `put_with_embedding(payload, embedding)` — ingest with vector
- `put_with_embedding_and_options(payload, embedding, options)` — ingest with vector + metadata
