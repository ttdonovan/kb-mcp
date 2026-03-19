---
tags: [patterns, agents, memory, curation]
created: 2026-03-18
updated: 2026-03-18
---

# Knowledge Keeper Pattern

A dedicated agent role responsible for curating shared knowledge. Instead
of every agent writing to the knowledge base ad-hoc, a Knowledge Keeper
sweeps sessions for notable information, filters noise, and maintains
quality.

## Why a Dedicated Role?

Agents generating knowledge as a side effect of their primary task
produce inconsistent, noisy output. A builder agent fixing a bug might
capture "fixed the bug" — useful as an event log but not as reusable
knowledge. A Knowledge Keeper transforms this into "PostgreSQL connection
pools exhaust under concurrent migrations — fix: set `max_connections`
per-worker, not globally."

The distinction: **events** vs **lessons**.

## What Gets Captured

| Category | Example | Memory tier |
|----------|---------|-------------|
| Failure patterns | "OOM when Redis cache exceeds 2GB" | Episodic → Procedural |
| Successful approaches | "Batch inserts 10x faster than individual" | Procedural |
| New conventions | "All API errors return structured JSON" | Semantic |
| Decision outcomes | "Chose PostgreSQL over SQLite — scaling proved it right" | Episodic |
| External discoveries | "Tantivy 0.25 changed the snippet API" | Semantic |

## Workflow

```
1. Sweep    — scan recent sessions/commits for notable information
2. Extract  — pull out the lesson, not just the event
3. Classify — assign memory tier, tags, and section
4. Write    — create properly formatted vault entry
5. Prune    — remove or update stale knowledge
```

## When to Use

- **Small teams / solo:** The developer acts as Knowledge Keeper during
  review. Capture happens via `kb_write` at the end of significant sessions.
- **Multi-agent swarms:** A scheduled agent runs periodically, scanning
  session logs and git history for knowledge worth preserving.
- **Always:** Someone (or something) should be curating. Uncurated
  knowledge bases decay into noise within weeks.

## Anti-Patterns

- **Capture everything** — the knowledge base becomes a log. No one
  searches logs for guidance.
- **Capture nothing** — the team re-discovers the same lessons repeatedly.
- **Capture without pruning** — stale entries erode trust. If the knowledge
  base says "use Redis 6" but the project upgraded to Redis 7, agents
  follow outdated advice.
