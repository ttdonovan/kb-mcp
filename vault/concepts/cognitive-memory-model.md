---
tags: [memory, architecture, cognitive-science]
created: 2026-03-18
updated: 2026-03-18
sources:
  - https://github.com/mengram/mengram
  - https://arxiv.org/abs/2304.03442
---

# Cognitive Memory Model

AI agents benefit from memory systems modeled on human cognition. Rather
than treating all stored information the same, a three-tier model
distinguishes between *knowing*, *experiencing*, and *doing*.

## Three Tiers

### Semantic Memory — What the Agent Knows

Facts, entities, preferences, and relationships. The stable knowledge
layer that persists across sessions.

- User preferences and project conventions
- Entity relationships (who owns what, what depends on what)
- Domain knowledge (API schemas, architecture decisions)

**Key property:** Semantic memory is queryable by concept, not by time.
An agent asking "what database does this project use?" queries semantic
memory.

### Episodic Memory — What the Agent Has Experienced

Timestamped events, decisions, and outcomes. The experiential layer that
enables learning from past sessions.

- Debugging sessions with symptoms, attempts, and resolutions
- Decisions made and their outcomes
- Conversations and their context

**Key property:** Episodic memory is temporal — ordered by when things
happened. An agent asking "what did we try last time this broke?" queries
episodic memory.

### Procedural Memory — What the Agent Knows How to Do

Workflows and procedures that evolve from failure analysis and success
clustering. The operational layer.

- Build and deploy procedures
- Debugging playbooks that worked
- Patterns that emerged from repeated tasks

**Key property:** Procedural memory improves over time. Failed approaches
are pruned, successful ones are reinforced. This is the tier that enables
compounding — each cycle makes the next one better.

## Why Three Tiers?

A flat key-value store treats "the database is PostgreSQL" and "we tried
Redis caching on Tuesday and it caused OOM errors" as the same kind of
information. The three-tier model lets agents:

1. **Answer factual questions** without sifting through event logs (semantic)
2. **Learn from experience** without polluting the fact store (episodic)
3. **Improve procedures** by analyzing what worked and what didn't (procedural)

## Retrieval Strategy

Each tier benefits from different search approaches:

| Tier | Best retrieval method | Why |
|------|----------------------|-----|
| Semantic | Keyword search (BM25) | Facts have specific terms ("PostgreSQL", "rate limit") |
| Episodic | Temporal + similarity | "Similar situations" requires fuzzy matching + time context |
| Procedural | Exact lookup + ranking | Procedures are named and versioned; rank by success rate |

Hybrid search (BM25 + vector similarity) bridges the gap for queries that
span tiers — "how do we handle rate limits?" might need facts (semantic),
past incidents (episodic), and the current playbook (procedural).

## Reference Implementations

- **Mengram** — three-tier cognitive memory with procedural evolution from
  failure analysis. Cloud or local MCP mode.
- **Claude Code memory** — file-based persistent memory with user, feedback,
  project, and reference types. Simpler than three-tier but effective for
  single-agent workflows.
- **Basic Memory** — knowledge graph with typed observations and relations.
  Stronger on semantic tier, weaker on episodic/procedural.
