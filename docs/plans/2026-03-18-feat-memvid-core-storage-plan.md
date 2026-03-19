---
title: "feat: Replace in-memory Tantivy with memvid-core persistent storage"
type: feat
status: active
date: 2026-03-18
origin: docs/brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md
---

# feat: Replace in-memory Tantivy with memvid-core persistent storage

## Overview

Replace kb-mcp's in-memory Tantivy BM25 index (rebuilt on every startup)
with memvid-core's persistent `.mv2` storage. Each collection gets its own
`.mv2` file. Startup opens existing files instead of scanning + indexing.
Incremental reindex processes only changed files.

This is the deferred storage swap from the initial v2 plan. The tool
surface, CLI, config, and format modules are unchanged.

## Problem Statement / Motivation

The current in-memory Tantivy index works but has two costs:

1. **Startup latency** — every startup scans all markdown files, parses
   frontmatter, and builds a Tantivy index from scratch. Fine for ~130 docs,
   but scales linearly.
2. **Full rebuild on every reindex/write** — `reindex` and `kb_write` both
   trigger `Index::build()` + `SearchEngine::rebuild()`, re-reading every
   file even when only one changed.

memvid-core eliminates both: persistent `.mv2` files open in milliseconds,
and smart markdown chunking improves search precision on long documents.

(See brainstorm: `docs/brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md`)

## Proposed Solution: Hybrid Two-Layer Architecture

Replace only the search layer (`SearchEngine` in `search.rs`) with
memvid-core. Keep the `Index` struct (`Vec<Document>`) for metadata
operations that memvid-core cannot serve natively (exact path lookup,
frontmatter retrieval, section counting).

```
Startup:
  1. Load collections.ron              (unchanged)
  2. Scan filesystem → Vec<Document>   (unchanged — fast, no Tantivy)
  3. Open/create .mv2 per collection   (NEW — replaces Tantivy build)
  4. Sync .mv2 against filesystem      (NEW — incremental hash diff)
  5. Enter mode                        (unchanged)
```

### Why Two Layers?

memvid-core is a search engine, not a document database. It does not
support:
- Exact-match lookup by URI (only full-text search)
- Metadata retrieval (frontmatter fields, tags) without searching
- Faceted counting (docs per section) for `list_sections`

These operations are fast against `Vec<Document>` (built from a
filesystem scan that takes <100ms for ~200 docs). The expensive part
was always the Tantivy index build, which memvid-core eliminates.

### What Changes

| Module | Change |
|--------|--------|
| `search.rs` | Replace `SearchEngine` with memvid-core wrapper |
| `index.rs` | Add content hashing for incremental sync |
| `server.rs` | Hold `Mutex<HashMap<String, Memvid>>` for per-collection .mv2 handles |
| `Cargo.toml` | Add `memvid-core`, remove direct `tantivy` dep |
| `tools/reindex.rs` | Use incremental sync instead of full rebuild |
| `tools/write.rs` | Add single document to .mv2 after file write |

### What Doesn't Change

| Module | Why |
|--------|-----|
| `config.rs` | `cache_dir` already exists and resolves correctly |
| `types.rs` | `Document` and `Section` structs unchanged |
| `format.rs` | Output structs unchanged |
| `cli.rs` | Commands unchanged (delegates to same functions) |
| `tools/sections.rs` | Reads from `Index`, not search engine |
| `tools/documents.rs` | Reads from disk via `read_fresh()` |
| `tools/context.rs` | Reads from `Index` metadata |

## Technical Approach

### .mv2 File Management

One `.mv2` file per collection, stored at:
```
<cache_dir>/<sha256-first-8>-<collection-name>.mv2
```

Example: `~/.cache/kb-mcp/a3f1b2c4-vault.mv2`

The hash prefix prevents collisions when two projects define a collection
with the same name but different paths. The `sha2` crate is already a
dependency.

### URI Scheme

Documents stored in memvid use the URI format:
```
<collection>://<relative-path>
```

Example: `vault://concepts/mcp-server-pattern.md`

This ensures uniqueness across collections and is parseable back to
collection name + filesystem path for `get_document` disk reads.

### Content Hashing for Incremental Sync

A sidecar file `<collection>.hashes` alongside each `.mv2` stores a
mapping of `relative_path → blake3_hash`. On startup/reindex:

1. Scan filesystem, compute hash per file
2. Load sidecar hashes
3. Diff: new files (in fs, not in sidecar), changed files (hash differs),
   deleted files (in sidecar, not in fs)
4. Add new/changed docs to `.mv2` via `put_bytes_with_options()`
5. Commit `.mv2`
6. Update sidecar

Using blake3 (fast) rather than sha256 for content hashing — different
purpose than the path-based cache key.

### Concurrency Model

```
KbMcpServer
  ├── index: Arc<RwLock<Index>>              # metadata (unchanged)
  ├── memvid_stores: Arc<Mutex<HashMap<String, Memvid>>>  # per-collection .mv2
  └── collections: Arc<Vec<ResolvedCollection>>            # config (unchanged)
```

**MCP server (long-lived):** Opens `.mv2` files with exclusive lock on
startup. Holds them for the process lifetime. Search and write operations
acquire the Mutex briefly.

**CLI (short-lived):** For read operations (`search`, `context`,
`get-document`, `list-sections`), opens `.mv2` with
`Memvid::open_read_only()` (shared flock — concurrent with MCP server).
For write operations (`write`, `reindex`), opens with `Memvid::open()`
(exclusive — blocks if MCP server is running).

**Contention analysis:** The Mutex serializes access to each collection's
Memvid handle. Search queries are fast (<10ms for ~200 docs). Write
operations (commit) take longer but are infrequent. For the expected
workload (one agent session), this is not a bottleneck.

### Search Result Mapping

memvid-core's `SearchHit` returns `uri`, `title`, `text`, `score`.
The search wrapper:

1. Queries each relevant `.mv2` file (all if no collection filter,
   one if filtered)
2. Deduplicates by URI (keeps highest-scoring chunk per document)
3. Parses URI to extract collection name and relative path
4. Maps back to `Vec<Document>` for section/tag metadata
5. Returns `SearchResult` in the existing format

Cross-collection score merging: BM25 scores from different indexes are
not perfectly comparable, but for collections of similar size the
approximation is acceptable. Sort by raw score across all collections.

### Smart Chunking

memvid-core's `put_bytes_with_options()` automatically chunks large
documents via its structural chunker. This means a query about "rate
limits" can hit the specific section of a long document rather than
scoring against the entire body.

Multiple chunks per document share the same URI. Search deduplication
(highest score per URI) ensures one result per document.

### Commit Strategy

- **`reindex`:** Batch mode — `begin_batch()`, process all changes,
  `end_batch()`, `commit()`. Single file copy.
- **`kb_write`:** Single `put_bytes_with_options()` + `commit()`.
  Acceptable cost for one-at-a-time writes. The file is small enough
  (<10MB for ~500 docs) that the atomic copy takes <1s.
- **Startup sync:** Same as reindex — batch mode for any detected changes.

### Error Recovery

- **Corrupted .mv2:** Log warning, delete file, rebuild from disk on next
  startup. The `.mv2` is a cache — disk markdown is the source of truth.
- **WAL recovery:** Runs automatically on `Memvid::open()`. No action needed.
- **Missing cache dir:** Create `~/.cache/kb-mcp/` on first use.
- **Lock contention (CLI):** If `.mv2` is exclusively locked (MCP running),
  CLI read operations use `open_read_only()`. CLI write operations print
  an error suggesting the user stop the MCP server or use MCP tools instead.

## Acceptance Criteria

### Functional Requirements

- [ ] `.mv2` file created per collection at `<cache_dir>/<hash>-<name>.mv2`
- [ ] Cold start opens existing `.mv2` files (no Tantivy rebuild)
- [ ] First run creates `.mv2` and bulk-ingests all documents
- [ ] Incremental reindex via content hashing (only changed/added/deleted files)
- [ ] `search` returns results from memvid-core's embedded Tantivy
- [ ] Cross-collection search merges results from multiple `.mv2` files
- [ ] Search deduplicates by URI (one result per document, highest-scoring chunk)
- [ ] `kb_write` adds document to `.mv2` after writing to disk
- [ ] `get_document` still reads from disk (fresh-read guarantee preserved)
- [ ] `kb_context` and `list_sections` still read from `Vec<Document>` metadata
- [ ] CLI read operations work concurrently with MCP server (shared flock)
- [ ] Corrupted `.mv2` auto-recovers by deleting and rebuilding

### Non-Functional Requirements

- [ ] Startup <500ms when `.mv2` files exist and no changes detected
- [ ] Direct `tantivy` dependency removed from Cargo.toml (comes via memvid-core)
- [ ] No changes to JSON output format (tools, CLI, format.rs unchanged)
- [ ] `cargo clippy` clean

### Quality Gates

- [ ] Verified search results match current behavior (same queries, same top results)
- [ ] Verified `.mv2` persists across process restarts
- [ ] Verified incremental reindex detects added, changed, and deleted files
- [ ] Verified CLI search works while MCP server is running

## Implementation Phases

### Phase 1: Add memvid-core dependency + .mv2 lifecycle

1. Add `memvid-core` to `Cargo.toml` with `default-features = false, features = ["lex"]`
2. Remove direct `tantivy` dep (verify memvid-core re-exports what's needed)
3. Add `blake3` crate for content hashing
4. Implement `.mv2` file path derivation from `cache_dir` + collection path hash
5. Implement open-or-create logic for `.mv2` files
6. Verify: `cargo build` compiles with memvid-core

### Phase 2: Content hashing + incremental sync

1. Add content hash computation to `index.rs` (blake3 per file)
2. Implement sidecar `.hashes` file (serialize/deserialize)
3. Implement diff logic: new, changed, deleted files
4. Implement batch ingest: `begin_batch()` → `put_bytes_with_options()` per doc → `end_batch()` → `commit()`
5. Implement delete handling (if memvid-core supports frame removal; otherwise full rebuild on deletes)
6. Verify: `.mv2` file created, sidecar written, second startup detects no changes

### Phase 3: Replace SearchEngine with memvid search

1. Create new `search.rs` wrapping `Memvid::search(SearchRequest)`
2. Implement URI-based result mapping back to `Vec<Document>`
3. Implement cross-collection fan-out search
4. Implement chunk deduplication (highest score per URI)
5. Wire new search into `tools/search.rs` and `cli.rs`
6. Verify: search returns same quality results as before

### Phase 4: Update reindex + kb_write paths

1. Update `tools/reindex.rs` to use incremental sync instead of full rebuild
2. Update `tools/write.rs` to add document to `.mv2` after disk write
3. Implement CLI concurrency: `open_read_only()` for reads, error message for writes when locked
4. Verify: `reindex` only processes changed files, `kb_write` adds to index immediately

### Phase 5: Polish + docs

1. Update `docs/ARCHITECTURE.md` — remove "Future" section, update diagrams
2. Update `docs/GOALS.md` — mark Phase 1 (Persistent Storage) as complete
3. Run `cargo clippy`, fix warnings
4. Test against real vault, verify search quality
5. Commit and push

## Dependencies & Prerequisites

- **memvid-core** crate v2.0.139+ with `lex` feature
- **blake3** crate for content hashing
- **fs2** (transitive via memvid-core) for file locking

## Open Questions

| Question | Impact | Default if unresolved |
|----------|--------|-----------------------|
| Does memvid-core support deleting individual frames by URI? | Determines whether deleted files require full .mv2 rebuild | Investigate in Phase 2; if no delete API, rebuild .mv2 from scratch when files are deleted |
| Can memvid-core re-export Tantivy types we need? | Determines whether direct tantivy dep can be fully removed | Keep tantivy as direct dep if memvid-core doesn't re-export SearchHit fields we need |
| Mutex contention at scale (>500 docs, concurrent requests)? | May need RwLock or reader/writer split | Profile in Phase 3; Mutex is fine for expected workload |

## Sources & References

### Origin

- **Brainstorm:** [docs/brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md](../brainstorms/2026-03-18-kb-mcp-v2-brainstorm.md) — memvid-core as dependency (not cherry-pick), persistent `.mv2` storage, smart chunking, incremental reindex

### Key Files

- `src/search.rs` — primary replacement target (current Tantivy wrapper)
- `src/index.rs` — add content hashing here
- `src/server.rs` — `Arc<SearchEngine>` becomes `Arc<Mutex<HashMap<String, Memvid>>>`
- `src/config.rs:33` — `ResolvedConfig.cache_dir` already exists, unused until now

### External References

- [memvid-core on crates.io](https://crates.io/crates/memvid-core) (v2.0.139)
- [memvid-core docs.rs](https://docs.rs/memvid-core)
- [memvid GitHub](https://github.com/memvid/memvid)
- [blake3 crate](https://crates.io/crates/blake3)

### API Reference (from source analysis)

- `Memvid::create(path)` / `Memvid::open(path)` — exclusive flock
- `Memvid::open_read_only(path)` — shared flock (concurrent reads)
- `put_bytes_with_options(content, PutOptions)` — add document with URI, title, tags
- `begin_batch(PutManyOpts)` / `end_batch()` — bulk ingestion without per-entry fsync
- `commit()` — atomic file copy, WAL replay, index rebuild
- `search(SearchRequest)` → `SearchResponse { hits: Vec<SearchHit>, total_hits, ... }`
- `SearchHit { frame_id, uri, title, text, score, ... }`
