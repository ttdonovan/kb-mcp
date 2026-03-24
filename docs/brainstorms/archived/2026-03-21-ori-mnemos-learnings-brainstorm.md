---
title: "Ori-Mnemos Learnings: What to Adopt for kb-mcp and Beyond"
type: exploration
status: completed
date: 2026-03-21
origin: book/src/landscape/ori-mnemos.md
---

# Ori-Mnemos Learnings: What to Adopt

## What We're Exploring

Ori-Mnemos is a TypeScript cognitive memory system (14,261 LOC) that treats
agent memory as a learning problem. It has features kb-mcp doesn't:
knowledge graph with learned edges, Q-value retrieval learning, stage
meta-learning, vault health diagnostics, agent identity resources, and
memory zones with decay.

We want to classify each pattern as direct/roadmap/not-adopting for kb-mcp,
and explore whether some patterns belong in a *new project* rather than
stretching kb-mcp beyond its identity as a knowledge base server.

## Ori-Mnemos Pattern Evaluation

### 1. Vault Health Diagnostics

**What Ori does:** `ori_health` checks index freshness, orphan notes,
dangling wiki-links, backlink counts, vault metrics.

**For kb-mcp:** Natural extension of `kb_digest`. Could add orphan
detection (docs not linked from any other doc), stale content flags
(no `updated` date in 90+ days), section coverage gaps. All derivable
from the existing `Index` with no new data structures.

**Classification: Roadmap (kb-mcp)**
**When:** When vaults grow large enough that quality degradation isn't
obvious from browsing. ~100+ documents.
**Effort:** Small (~100-150 lines). Extend `format_digest` or add a
`kb_health` tool.

### 2. Q-Value Retrieval Learning

**What Ori does:** Tracks per-note quality scores learned from downstream
signals (forward citations, re-recalls, updates, dead-ends). High-Q
notes rank higher in future searches. Exposure-corrected rewards.

**For kb-mcp:** Would require tracking which search results agents
actually use (get_document calls after search) and feeding that back
into ranking. Fundamentally changes kb-mcp from a stateless retrieval
tool to a stateful learning system.

**Classification: Not adopting (for kb-mcp)**
This crosses the "not agent memory" boundary. Q-value learning is
inherently about the agent's relationship with content, not the
content's intrinsic quality. Better suited to a dedicated agent
memory system.

**For a new project:** Core feature. The Rust implementation could
use SQLite for learning state (like Ori) or extend the `.mv2` sidecar.

### 3. Co-Occurrence Edge Learning

**What Ori does:** Notes retrieved together form stronger associations
via Hebbian learning (NPMI-weighted). Edges decay with time. Homeostasis
normalization prevents hub dominance. Used in Personalized PageRank.

**For kb-mcp:** Lightweight version possible — track co-retrieval
frequency in a sidecar file, use it as a boost signal in search. But
this is a learning system, which conflicts with kb-mcp's stateless
design.

**Classification: Not adopting (for kb-mcp) / Core (for new project)**
Same reasoning as Q-value. Co-occurrence is about learned associations,
not intrinsic document properties.

### 4. Identity / Context Resources

**What Ori does:** `ori://identity`, `ori://goals`, `ori://methodology`,
`ori://daily`, `ori://reminders` — persistent agent context loaded at
session start.

**For kb-mcp:** Could be implemented as a special collection or
convention (e.g., `_identity/` directory with well-known filenames).
But this is explicitly agent memory — kb-mcp's GOALS.md says it's
"not a replacement for agent memory."

**Classification: Not adopting (for kb-mcp) / Core (for new project)**
Identity resources are the defining feature of an agent memory system.
They don't belong in a knowledge base server.

### 5. Memory Zones with Decay

**What Ori does:** Three zones — identity (0.1x decay), knowledge (1x),
operations (3x fast decay). Notes gradually lose "vitality" and become
archival candidates.

**For kb-mcp:** kb-mcp treats all documents equally. Decay implies
temporal awareness and lifecycle management, which is agent memory
territory.

**Classification: Not adopting (for kb-mcp) / Roadmap (for new project)**
Interesting for the Knowledge Keeper agent on the roadmap, but belongs
in the agent layer, not the knowledge base layer.

### 6. Stage Meta-Learning (LinUCB)

**What Ori does:** Learns which retrieval stages (semantic, BM25,
PageRank, warmth) contribute value for different query types. Skips
expensive stages when they don't help. Time-budgeted.

**For kb-mcp:** Only relevant if kb-mcp adds multiple retrieval signals.
Currently BM25 + optional vector. With only 2 signals, there's nothing
to learn about — just use both.

**Classification: Not adopting (either project, for now)**
Premature until there are 4+ retrieval signals. Revisit if a new
project builds a multi-signal pipeline.

## Decision Summary

| Pattern | kb-mcp | New Project |
|---------|--------|-------------|
| Vault health diagnostics | Roadmap | Include |
| Q-value retrieval learning | Not adopting | Core |
| Co-occurrence edges | Not adopting | Core |
| Identity resources | Not adopting | Core |
| Memory zones with decay | Not adopting | Roadmap |
| Stage meta-learning | Not adopting | Not yet |

**Key insight:** Only vault health diagnostics fits kb-mcp's identity.
The other 5 patterns all cross into agent memory territory. This
strongly suggests a separate project if we want these capabilities.

## Architecture Paths

### Path A: Evolve kb-mcp

Add vault health diagnostics to kb-mcp. Stop there. The other Ori
patterns don't fit.

```
kb-mcp (today)
  + kb_health tool (orphans, stale content, link analysis)
  = knowledge-aware retrieval server
```

**Pros:**
- Incremental, low risk
- Each addition is useful standalone
- Keeps kb-mcp simple and focused

**Cons:**
- Doesn't address agent memory needs
- OpenFang agents still lack identity/learning

**Best when:** Knowledge base retrieval is the primary use case.

### Path B: New Standalone Project

A new Rust crate (working name: `ori-rs` or `agora`) inspired by
Ori-Mnemos architecture but built with kb-mcp's Rust patterns and
simplicity philosophy.

```
agora (new project)
  ├── Knowledge graph (wiki-links + learned edges)
  ├── Multi-signal retrieval (BM25 + semantic + PageRank)
  ├── Q-value learning layer
  ├── Co-occurrence edge learning
  ├── Identity resources
  ├── Memory zones with decay
  ├── MCP tools (16+) + CLI parity
  └── Storage: markdown + SQLite
```

**Pros:**
- Clean architecture, purpose-built for agent memory
- kb-mcp stays simple — two tools for two jobs
- Can adopt Ori's proven architecture without compromise

**Cons:**
- Two projects to maintain
- Duplicated indexing/search code
- Cold start on adoption

**Best when:** Agent memory is a real, distinct need.

### Path C: kb-mcp Core Library + Agent Memory Extension

Refactor kb-mcp into a library crate (`kb-core`) and a binary crate
(`kb-mcp`). A new `kb-agent-memory` crate depends on `kb-core` for
indexing and search, adding learning and identity layers.

```
kb-core (library crate)
  ├── config.rs, index.rs, search.rs, format.rs, types.rs, store.rs
  └── Shared markdown indexing + BM25 search

kb-mcp (binary crate, depends on kb-core)
  ├── server.rs, cli.rs, tools/*.rs
  └── 9 MCP tools, same as today

kb-agent-memory (binary crate, depends on kb-core)
  ├── graph.rs, learning.rs, identity.rs, decay.rs
  ├── tools/ (ori-inspired MCP tools)
  └── Agent memory with learning layers
```

**Pros:**
- Code reuse — shared indexing, search, format patterns
- kb-mcp stays unchanged for existing users
- Agent memory gets purpose-built design
- Both share the same vault files

**Cons:**
- Requires refactoring kb-mcp into lib + bin (breaking change)
- API stability pressure on kb-core
- More complex workspace management

**Best when:** Both use cases are real and you want maximum code reuse.

## Resolved Questions

1. **Is agent memory a real need today?** Mild friction — agents would
   benefit from continuity but it's not blocking. Not urgent enough to
   drive a new project immediately.

2. **Timeline?** Someday / backlog. Document the patterns now, revisit
   when agent memory friction becomes a real blocker. The vault
   intelligence bundle should be proven and stable first.

## Open Questions

1. **Would a new project use memvid-core or build its own search?**
   memvid-core provides .mv2 persistent storage with BM25 + vector.
   A new project could use it or go with SQLite FTS5 + a vector lib.

2. **Naming and positioning?** If a new project is created, how does it
   relate to kb-mcp in the ecosystem? Companion tool? Successor? Fork?

## Recommendation

**Near-term (next feature cycle):** Add `kb_health` to kb-mcp (Path A).
This is the one Ori pattern that cleanly fits kb-mcp's identity. Small
scope (~100-150 lines), extends `kb_digest`, no architectural changes.

**Long-term (when agent memory friction grows):** Revisit Path C (library
extraction + agent memory crate). This gives the best code reuse while
keeping kb-mcp simple. Path B (fully standalone) is the fallback if the
refactoring cost is too high.

**Not adopting:** Q-value learning, co-occurrence edges, identity
resources, memory zones, and stage meta-learning all belong in an agent
memory system, not a knowledge base server. They're documented here as
reference for when that project starts.
