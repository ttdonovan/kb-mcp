---
title: "Vault Intelligence Bundle: Adding MCP tools with auto-reindex"
date: 2026-03-20
category: feature-patterns
tags:
  - rust
  - mcp
  - knowledge-base
  - auto-reindex
  - code-review
  - rmcp
  - format-layer
modules:
  - src/tools/digest.rs
  - src/tools/query.rs
  - src/tools/export.rs
  - src/tools/mod.rs
  - src/server.rs
  - src/format.rs
  - src/cli.rs
  - src/tools/search.rs
problem_type: feature_implementation
severity: n/a
summary: >
  Added three MCP tools (kb_digest, kb_query, kb_export) plus auto-reindex.
  Code review caught a P1 filtered-count bug, export duplication, auto-reindex
  scope gap, and silent validation failure. Key learnings about filtered views,
  cross-cutting concerns, and implementation parity between MCP and CLI.
---

# Vault Intelligence Bundle: Learnings

## What Was Built

Three new MCP tools and a behavior change for kb-mcp:

- **kb_digest** — vault summary with section topics, recent additions, gap hints
- **kb_query** — AND-logic frontmatter filtering (tag, status, date, sources)
- **kb_export** — full-vault markdown snapshot read fresh from disk
- **auto-reindex** — mtime-based staleness check before index reads

863 lines across 11 files. Each tool is 48-123 lines following the
established one-file-per-tool pattern.

## What Worked Well

**One file per tool pattern.** Adding three tools required three new files
plus three `+ module::router()` lines in `tools/mod.rs`. The pattern is
mechanical and hard to get wrong structurally.

**Format layer separation.** All output assembly in `format.rs` — not in
tools, not in CLI. Functions take domain types and return `String`. This
guarantees MCP/CLI output parity and keeps tool code focused on I/O.

**Plan-driven development.** The plan file had checkboxes for every
acceptance criterion. The review could trace each finding back to a
specific gap.

**yaml_value_to_string helper.** Handles `serde_yaml::Value` (no `Display`
impl) to string conversion. Special-cases `String` to avoid quotes, falls
through to `serde_json::to_string` for complex values.

## What Went Wrong

### P1: Filtered count bug in kb_digest

`format_digest` used `sec.doc_count` (global `Section` struct count)
instead of `sec_docs.len()` (filtered count) when `collection_filter` was
active. The "thin section" hint and count output were wrong for filtered
views.

**Root cause:** The variable name `sec.doc_count` reads naturally, masking
the semantic mismatch. Two similar values with different meanings depending
on context.

**Fix:** `let filtered_count = sec_docs.len()` — use the length of the
already-filtered vec for both count and hint.

### P2: Auto-reindex scope gap

Auto-reindex was only wired into `search`. The `kb_digest`, `kb_query`,
and `kb_export` tools also read from `self.index` but could serve stale
data after `kb_write`.

**Root cause:** The feature was conceived as "auto-reindex on search" and
implemented literally. The broader concern — all index-reading tools need
freshness — wasn't considered.

**Fix:** One line per tool: `self.auto_reindex_stale_collections().await`
before `self.index.read().await`.

### P2: Export logic duplication

The MCP tool and CLI handler each had their own copy of the markdown
assembly loop and frontmatter-stripping disk reads.

**Root cause:** Initial implementation treated MCP and CLI as separate
concerns. No `format_export` function existed yet.

**Fix:** Extracted `format::format_export()` and
`format::read_document_body()` into `format.rs`. Both callers now use
the shared functions.

### P2: Silent date parse failure

`kb_query` with an invalid `created_after` value silently returned zero
results. The `NaiveDate::parse_from_str` failure was swallowed by
`.ok()` in the filter predicate.

**Root cause:** Permissive parsing pattern — treating bad input as "no
input" via `.ok()`.

**Fix:** Upfront validation before filtering, returning a clear error
with the bad value and expected format.

## Prevention Rules

These rules would have caught all four issues:

1. **Filtered views must use filtered counts.** When a function accepts a
   filter parameter, every count and summary must derive from the filtered
   result set — never from a pre-computed aggregate on an unfiltered struct.

2. **Cross-cutting concerns go everywhere or get extracted.** If a guard
   applies to one tool reading from a data source, it applies to ALL tools
   reading from that source.

3. **One implementation, two interfaces.** CLI and MCP must share
   implementation code — not just feature parity. No parallel formatting
   or disk-read code in `cli.rs`.

4. **Invalid input is an error, never a silent default.** String-to-type
   parse failures must return a user-facing error. Never `.ok()` a
   user-supplied parse.

## Reusable Patterns

### Tool file structure (`src/tools/*.rs`)

```
params struct (Deserialize + JsonSchema)
pub(crate) fn router() -> ToolRouter<KbMcpServer>
#[rmcp::tool_router] impl block with #[rmcp::tool] method
```

Registration: `+ module::router()` in `combined_router()`.

### Format layer (`src/format.rs`)

Output structs decouple serialization from internal types. Functions take
`&[Document]` / `&[Section]` and return `String`. Both MCP tools and CLI
call the same functions.

### Auto-reindex guard

`self.auto_reindex_stale_collections().await` placed immediately before
`self.index.read().await` in every index-reading tool. The method is
idempotent — one `stat()` per collection, rebuilds only stale ones.

### read_document_body shared reader

Strips YAML frontmatter and returns the body. Lives in `format.rs`.
Used by `get_document`, `kb_export`, and CLI export.

## Related Documents

- [Brainstorm](../../brainstorms/2026-03-20-vault-intelligence-bundle-brainstorm.md)
- [Implementation plan](../../plans/2026-03-20-feat-vault-intelligence-bundle-plan.md) (completed)
- [Deferred P2s plan](../../plans/2026-03-20-fix-vault-intelligence-deferred-p2s-plan.md) (active)
- [Book tools reference](../../book/src/reference/tools.md)

## Doc Gaps Identified

- `docs/ROADMAP.md` — needs Vault Intelligence Bundle in "Completed"
- `docs/ARCHITECTURE.md` — module map lists 6 tools, should be 9
- `CLAUDE.md` — architecture tree says "6 MCP tools", should be 9
