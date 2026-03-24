---
name: "Vibeship Spark: Notable Patterns"
description: Two patterns from Vibeship Spark Intelligence worth referencing for future Knowledge Keeper and observability work
type: reference
---

# Vibeship Spark: Notable Patterns

**Source:** [github.com/vibeforge1111/vibeship-spark-intelligence](https://github.com/vibeforge1111/vibeship-spark-intelligence)

Spark Intelligence is a Python-based agent behavior learning system
(127K LOC) that observes tool usage via Claude Code hooks and promotes
validated insights. It's not in kb-mcp's domain (knowledge retrieval),
but two patterns are worth referencing for future work.

## 1. Auto-Promotion of Validated Insights

Spark tracks "cognitive insights" with reliability scores. When an
insight reaches reliability >= 70% and has been validated 3+ times,
it auto-promotes to the appropriate project file:

| Target | Content type |
|--------|-------------|
| CLAUDE.md | Wisdom, reasoning, context rules |
| AGENTS.md | Meta-learning, self-awareness |
| TOOLS.md | Tool-specific patterns |
| SOUL.md | User preferences, communication style |

**Why this matters for kb-mcp:** The `/ce:compound` workflow captures
solutions manually. The Roadmap's Knowledge Keeper agent could automate
this — tracking which `docs/solutions/` entries are referenced by
agents and auto-promoting proven patterns to CLAUDE.md. The threshold
concept (N validations + reliability score) prevents noise.

**When to revisit:** When building the Knowledge Keeper agent or the
Knowledge Capture Tools (Roadmap Phase 3).

## 2. Obsidian Observatory Generation

Spark generates ~465 Obsidian markdown pages from its `~/.spark/`
internal state files in under 1 second. The vault includes:

- Mermaid pipeline diagrams with live metrics
- Dataview queries over JSON frontmatter
- Per-stage detail pages (12 pipeline stages)
- Explorer pages for browsing individual data items
- Canvas view for spatial navigation
- Auto-sync every 120 seconds

**Why this matters for kb-mcp:** kb-mcp already has mdBook docs, but
the idea of *generating vault content from system state* is interesting
for observability. A `kb_observatory` tool could generate markdown
pages showing search analytics, retrieval patterns, health trends, and
collection growth over time — browsable in Obsidian alongside the
vault content itself.

**When to revisit:** When kb-mcp gains usage analytics or when the
vault is large enough that observability becomes a need.
