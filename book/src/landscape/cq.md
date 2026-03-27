# cq

**Source:** [github.com/mozilla-ai/cq](https://github.com/mozilla-ai/cq)
**Language:** Python + TypeScript | **Status:** v0.4.0, Active (Mozilla AI)

Shared knowledge commons for AI agents — collective learning so agents
stop independently rediscovering the same failures. Built by Mozilla AI.

## What It Does

cq captures "knowledge units" (KUs) that emerge from agent sessions and
makes them queryable by other agents. Agents propose insights, confirm
what works, flag what's wrong, and reflect at session end. Knowledge
graduates from local (private) to team (org-shared, human-reviewed) to
global (public commons — not yet implemented).

The core thesis: agents worldwide burn tokens rediscovering the same
failures. A shared commons eliminates redundant learning.

## Key Features

- 6 MCP tools: `query`, `propose`, `confirm`, `flag`, `reflect`, `status`
- SQLite + FTS5 local store with domain tag Jaccard similarity scoring
- Confidence scoring: confirmations boost (+0.1), flags penalize (-0.15)
- Tiered graduation: local → team (human-in-the-loop review) → global
- Team API (FastAPI) with React review dashboard
- Post-error hook auto-queries commons before agent retries
- Session-end `reflect` mines conversations for shareable insights
- Claude Code plugin (SKILL.md behavioral protocol + hooks.json)
- 69KB proposal document covering trust layers, DID identity, ZK proofs

## Comparison to kb-mcp

| Aspect | cq | kb-mcp |
|--------|-----|--------|
| **Domain** | Agent collective knowledge | Curated document search |
| **Content source** | Agent-generated (propose/confirm/flag) | Human-authored markdown |
| **Search** | FTS5 + domain tag Jaccard | BM25 (Tantivy) |
| **Write model** | Propose → confirm/flag loop | kb_write to writable collections |
| **Storage** | SQLite (local) + team API (cloud) | Tantivy index + disk reads |
| **Unique feature** | Confidence scoring via peer confirmation | Token-efficient kb_context |
| **Tool count** | 6 | 10 |

**Relationship:** Complementary. kb-mcp serves curated reference knowledge
("what does our API spec say?"); cq serves collective agent wisdom
("what gotchas have agents hit with this API?"). They'd coexist naturally.

## Patterns Worth Adopting

- **Confirmation/flagging feedback** — lightweight signals for "this document
  helped" or "this seems stale" could inform search ranking or kb_health.
- **Session reflection mining** — a "reflect and write" workflow that mines
  a session for knowledge to capture via kb_write.
- **Post-error auto-lookup** — hook-based auto-search when agents encounter
  unfamiliar territory.
- **Domain tag scoring** — combining tag-based Jaccard similarity with
  text-based BM25 could improve kb_query relevance.

## Source Metrics

| Component | Language | Files | Code Lines | Comments | Total Lines |
|-----------|----------|------:|----------:|---------:|----------:|
| MCP server + team API | Python | 27 | 5,453 | 98 | 6,564 |
| Dashboard | TypeScript + TSX | 21 | 1,574 | 11 | 1,735 |
| Docs | Markdown | 12 | — | 1,529 | 2,280 |
| **Total** | | 81 | 10,129 | 1,667 | 14,378 |

Doc-to-code ratio: 0.2x (lean docs relative to codebase, though the 69KB
proposal document is the real design investment).
