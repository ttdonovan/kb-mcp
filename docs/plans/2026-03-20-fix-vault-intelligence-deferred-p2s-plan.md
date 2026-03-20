---
title: "fix: Vault Intelligence Bundle — deferred P2 review findings"
type: fix
status: active
date: 2026-03-20
origin: Code review of feat/vault-intelligence-bundle branch
---

# fix: Vault Intelligence Bundle — Deferred P2s

## Overview

Three performance/robustness findings from the `/ce:review` of the Vault
Intelligence Bundle that were deferred from the initial fix commit. Each
is independent and can be tackled in any order.

## Problem Statement

The auto-reindex mechanism and `kb_export` tool have scaling limitations
that are acceptable for small vaults (dozens of docs) but become
problematic at scale (hundreds to thousands of documents).

## Deferred Findings

### 1. Incremental auto-reindex (rebuild only stale collections)

**Current behavior:** `auto_reindex_stale_collections()` in `server.rs`
calls `Index::build(&self.collections)` which walks the entire filesystem
for every collection — even if only one collection is stale.

**Impact:** For 5 collections where 1 is stale, all 5 are re-scanned,
re-parsed, and re-hashed. At 500+ documents this adds ~100ms+ per
search when any collection is stale.

**Proposed fix:** Add an `Index::rebuild_collection()` method that
re-scans a single collection and merges the results into the existing
index. Only call it for stale collections.

**Key files:** `src/index.rs` (add incremental rebuild), `src/server.rs`
(call per-collection rebuild instead of full `Index::build`)

**Effort:** Medium

### 2. Auto-reindex debounce/cooldown

**Current behavior:** Every `search`, `kb_digest`, `kb_query`, and
`kb_export` call runs `auto_reindex_stale_collections()`. Rapid-fire
tool calls (common with LLM agents) each independently check staleness
and potentially trigger redundant full rebuilds.

**Impact:** 10 search queries in quick succession during active editing
could trigger 10 staleness checks and potentially multiple overlapping
rebuilds. The RwLock serializes index writes, but filesystem walks still
happen redundantly.

**Proposed fix:** Add a `last_reindex: Arc<AtomicU64>` field to
`KbMcpServer` storing the last auto-reindex epoch. Skip the check if
less than 5 seconds have elapsed. One atomic read per tool call instead
of N stat() calls.

**Key files:** `src/server.rs` (add cooldown field + check)

**Effort:** Small

### 3. Bounded kb_export output

**Current behavior:** `kb_export` concatenates every document in the
vault (or a collection) into a single `String` with no size limit.

**Impact:** A vault with 1000 documents averaging 5KB each produces a
5MB response. This can exhaust memory, overwhelm MCP client context
windows, and waste tokens. At 10K docs, ~50MB+.

**Proposed fix:** Add a `max_documents` parameter (default: 200) to
`ExportParams`. When the limit is hit, truncate and append a summary
line: `"... truncated: showing 200 of 1000 documents"`. Alternatively,
add a `section` filter parameter for targeted exports.

**Key files:** `src/tools/export.rs` (add param + limit check),
`src/cli.rs` (add `--max-documents` flag),
`book/src/reference/tools.md` (document new param)

**Effort:** Small

## Acceptance Criteria

### Incremental auto-reindex
- [ ] Only stale collections are re-scanned during auto-reindex
- [ ] Fresh collections are untouched (no filesystem walk)
- [ ] Index remains consistent after partial rebuild

### Debounce/cooldown
- [ ] Auto-reindex skipped if last reindex was < 5 seconds ago
- [ ] First search after cooldown still triggers reindex if stale
- [ ] Cooldown is per-server, not per-collection

### Bounded kb_export
- [ ] `max_documents` parameter with sensible default
- [ ] Truncation message when limit is hit
- [ ] CLI `--max-documents` flag with parity
- [ ] Book docs updated

## Sources & References

- **Review agents:** Performance Oracle, Security Sentinel, Architecture
  Strategist (all flagged these independently)
- **Parent feature:** `docs/plans/2026-03-20-feat-vault-intelligence-bundle-plan.md`
