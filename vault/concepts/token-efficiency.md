---
tags: [memory, retrieval, token-efficiency, agent-patterns]
created: 2026-03-18
updated: 2026-03-18
---

# Token Efficiency in Knowledge Retrieval

Agents pay for every token they read. A naive "search and dump"
approach wastes context window on documents that turn out to be
irrelevant. Token-efficient retrieval is a staged pipeline:
scan cheaply, read selectively.

## The Problem

An agent searching for "rate limits" gets 10 results. Reading all 10
full documents costs ~15K tokens. But only 2-3 are actually relevant.
The other 7 are wasted context that displaces useful information.

With a 200K context window this feels affordable. With a 4K tool
response limit or a cost-conscious workflow, it's not.

## Staged Retrieval Pattern

```
1. Search       → ranked list with excerpts (~50 tokens each)
2. Briefing     → frontmatter + summary per candidate (~100 tokens each)
3. Full read    → complete document for confirmed-relevant results
```

**Stage 1: Search.** BM25 returns paths, scores, and short excerpts.
The agent scans excerpts to shortlist candidates. Cost: ~500 tokens
for 10 results.

**Stage 2: Briefing.** For shortlisted candidates, retrieve frontmatter
(tags, dates, status) and the first paragraph. The agent decides which
documents warrant full reading. Cost: ~100 tokens per document.

**Stage 3: Full read.** Only for documents confirmed relevant. The agent
reads the complete body. Cost: varies, but applied selectively.

## Token Savings

For a typical 10-result search where 3 documents are relevant:

| Approach | Tokens consumed |
|----------|----------------|
| Read all 10 full documents | ~15,000 |
| Staged (search → briefing → 3 full reads) | ~5,000 |
| **Savings** | **~67%** |

The savings compound in retrieval-heavy workflows where agents make
multiple searches per task.

## Implementation

kb-mcp implements this pattern with three tools:

- `search` — Stage 1. Returns ranked results with BM25 excerpts.
- `kb_context` — Stage 2. Returns frontmatter + first paragraph summary.
- `get_document` — Stage 3. Returns full document content fresh from disk.

The key design choice: `kb_context` returns *all* frontmatter fields, not
just tags. Agents use metadata (status, source, created date) to judge
relevance without reading the body.

## When Not to Optimize

- **Small collections (<50 docs):** Just read everything. The overhead of
  staged retrieval exceeds the token savings.
- **Known paths:** If the agent already knows which document it needs,
  skip search and briefing — go straight to `get_document`.
- **Write-then-read:** After `kb_write` creates a document, the agent
  already has the content. No need to retrieve what it just wrote.
