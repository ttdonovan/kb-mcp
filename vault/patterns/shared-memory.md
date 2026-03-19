---
tags: [patterns, memory, multi-agent, swarms]
created: 2026-03-18
updated: 2026-03-18
---

# Shared Memory for Multi-Agent Systems

When multiple agents work on the same project, they need a way to share
knowledge without direct communication. Shared memory serves as the
common ground — a persistent knowledge layer that all agents can read
from and (selectively) write to.

## The Problem

Agents in isolation:
- **Duplicate work** — Agent A discovers a fix that Agent B rediscovers next week
- **Contradict each other** — Agent A decides on approach X while Agent B uses approach Y
- **Lose context** — knowledge dies when a session ends

## Approaches

### Flat File Store

Markdown files in a shared directory. Simple, version-controlled, human-readable.

- **Pros:** Zero infrastructure, works with git, humans can read and edit
- **Cons:** No search beyond grep, no structured queries, manual curation
- **Best for:** Small teams, early-stage projects

### Knowledge Graph

Typed nodes (facts, entities) with typed edges (relations). Queryable
by relationship.

- **Pros:** Relationship queries ("what depends on X?"), contradiction detection
- **Cons:** Complex to maintain, requires schema design, graph queries are unintuitive
- **Best for:** Large projects with complex entity relationships

### Indexed Collections (kb-mcp approach)

Markdown files indexed into a search engine. Queryable by keyword,
filterable by collection and section. Write-back via MCP tools.

- **Pros:** Human-readable source, BM25 search, MCP tool access, collection-level permissions
- **Cons:** No relationship queries, no contradiction detection
- **Best for:** Project knowledge bases where search is the primary access pattern

## Memory Types in Shared Context

| Type | Scope | Example |
|------|-------|---------|
| Working | Single session | Current task context, in-progress state |
| Episodic | Cross-session | Session digests, decision logs |
| Semantic | Permanent | Architecture docs, API schemas, conventions |
| Procedural | Evolving | Playbooks, debugging workflows |
| Identity | Per-agent | Agent role, expertise, working style |

Working memory doesn't belong in shared storage — it's too volatile.
Identity memory is per-agent, not shared. The shared layer holds
episodic, semantic, and procedural memory.

## Open Questions

- **Conflict resolution:** When two agents write contradictory facts,
  which wins? Timestamp-based? Confidence-scored? Human-arbitrated?
- **Retention policy:** How long do episodic memories persist before
  archival or deletion?
- **Privacy boundaries:** Should agents see each other's session digests,
  or only curated summaries?
- **Staleness detection:** How does the system identify and flag outdated
  knowledge?
