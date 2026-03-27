# prism-mcp

**Source:** [github.com/dcostenco/prism-mcp](https://github.com/dcostenco/prism-mcp)
**Language:** TypeScript | **Status:** v5.1.0, Very Early (solo author)

"Mind Palace" for AI agents — persistent session memory with behavioral
learning, time travel, multi-agent sync, and a visual dashboard.

## What It Does

prism-mcp gives agents persistent memory across conversations through
three layers: an append-only session ledger (what happened), mutable
handoff state with optimistic concurrency control (what's current), and
behavioral memory that learns from corrections (what to avoid). High-
importance lessons can auto-graduate into `.cursorrules` / `.clauderules`.

## Key Features

- 30+ MCP tools across session, memory, search, and dashboard domains
- Three-layer memory: session ledger, handoff state (OCC versioned), behavioral
- Three-tier search: FTS5, sqlite-vec vectors, TurboQuant JS fallback
- TurboQuant: pure-TS embedding compression (ICLR 2026) — 768-dim from
  3,072 bytes to ~400 bytes (7x), >90% top-1 retrieval accuracy
- Time travel via versioned handoff snapshots (`memory_checkout`)
- Multi-agent hivemind with role isolation (dev/qa/pm)
- Behavioral learning: corrections accumulate importance, auto-surface
- Progressive context loading: quick/standard/deep tiers
- Web dashboard at localhost:3000 (knowledge graph, timeline, health)
- Morning briefings after 4+ hours of inactivity
- SQLite (local) or Supabase (cloud) backends

## Comparison to kb-mcp

| Aspect | prism-mcp | kb-mcp |
|--------|-----------|--------|
| **Domain** | Agent session memory | Curated document search |
| **Content source** | Agent-generated session logs | Human-authored markdown |
| **Search** | FTS5 + sqlite-vec + TurboQuant | BM25 (Tantivy) |
| **Write model** | Append ledger + upsert handoff | kb_write to writable collections |
| **Storage** | SQLite or Supabase | Tantivy index + disk reads |
| **Unique feature** | Behavioral learning + time travel | Token-efficient kb_context |
| **Tool count** | 30+ | 10 |

**Relationship:** Different domains entirely. kb-mcp retrieves curated
knowledge; prism-mcp persists agent session state. Complementary — an
agent would use both simultaneously.

## Patterns Worth Adopting

- **Progressive context loading** — formalized quick/standard/deep tiers
  for kb_context could help agents pick the right depth.
- **Optimistic concurrency control** — relevant if kb-mcp ever supports
  concurrent writers to the same collection.
- **Health check with auto-repair** — extending kb_health to suggest or
  apply fixes, not just diagnose.

## Source Metrics

| Component | Language | Files | Code Lines | Comments | Total Lines |
|-----------|----------|------:|----------:|---------:|----------:|
| Core server | TypeScript | 69 | 17,012 | 6,414 | 26,141 |
| Migrations | SQL | 14 | 2,227 | 670 | 3,207 |
| Tests | Python | 15 | 2,150 | 268 | 2,754 |
| Docs | Markdown | 9 | — | 1,009 | 1,466 |
| **Total** | | 121 | 28,367 | 8,587 | 40,977 |

Doc-to-code ratio: 0.05x. The codebase is large relative to documentation.
Notable: single-author v1→v5 in 3 days with 96KB handler files suggests
rapid feature accretion. The 30+ tool count is unusually high for an MCP
server and may cause prompt bloat.
