---
tags: [patterns, memory, episodic, sessions]
created: 2026-03-18
updated: 2026-03-18
---

# Session Digests

A structured summary created at the end of a work session, capturing
context, decisions, and outcomes that would otherwise be lost when the
session ends.

## Why Session Digests?

AI agent sessions are ephemeral. When the context window closes, everything
the agent learned during that session — what it tried, what failed, what
the user clarified — disappears. The next session starts from zero.

Session digests bridge this gap by extracting the non-obvious learnings
and persisting them as episodic memory.

## Structure

```yaml
session: "2026-03-18 — kb-mcp v2 scaffolding"
duration: ~2 hours
outcome: success

context:
  - Porting kb-mcp from single-project to standalone crate
  - RON chosen over TOML for collection config

decisions:
  - memvid-core integration deferred — in-memory Tantivy sufficient for now
  - All section descriptions moved from code to RON config
  - get_document reads from disk, not index (freshness over consistency)

discoveries:
  - rmcp 1.2 requires explicit transport-io feature flag
  - RON supports comments (TOML does too, but RON's nested struct syntax is cleaner)
  - Tantivy STRING fields don't support prefix queries — post-filter instead

open_threads:
  - memvid-core concurrent writer support unknown
  - No integration tests yet
```

## What to Capture

**Capture:**
- Decisions and their rationale (especially non-obvious ones)
- Things that surprised you or contradicted assumptions
- Workarounds and why they were necessary
- User preferences that emerged during the session

**Skip:**
- Code changes (those are in git)
- Routine operations that went as expected
- Information already documented elsewhere

## Storage

Session digests belong in a writable collection. Using kb-mcp:

```sh
kb-mcp write \
  --collection notes \
  --title "Session: kb-mcp v2 scaffolding" \
  --tags "session, kb-mcp, architecture" \
  --body "..."
```

The digest becomes searchable immediately. Future sessions can query
"what happened last time we worked on X?" and get structured context
instead of nothing.
