---
tags: [memory, retrieval, search, bm25, vector-search]
created: 2026-03-18
updated: 2026-03-18
---

# Retrieval Strategies

How agents find relevant information in their knowledge base. The choice
of retrieval strategy determines whether an agent can answer "what is X?"
(keyword), "what's similar to X?" (vector), or both (hybrid).

## BM25 (Keyword Search)

Term-frequency ranking with stemming. The workhorse of text search.

**Strengths:**
- Fast, deterministic, no model required
- Excellent for specific terms ("PostgreSQL", "rate limit", "CORS error")
- Interpretable — you can explain why a result matched

**Weaknesses:**
- Misses conceptual similarity ("how do agents share state?" won't match
  "MCP server pattern" even though it's the answer)
- Sensitive to vocabulary mismatch between query and document

**When to use:** As the default. BM25 handles 80%+ of knowledge base
queries well. Start here and add vector search only when keyword search
consistently fails on conceptual queries.

## Vector Similarity (Embedding Search)

Encode documents and queries as dense vectors, find nearest neighbors.

**Strengths:**
- Captures semantic meaning — "authentication" matches "login flow"
- Handles vocabulary mismatch and paraphrasing
- Good for exploratory queries ("how does X relate to Y?")

**Weaknesses:**
- Requires an embedding model (local ONNX or cloud API)
- Slower and more resource-intensive than BM25
- Less interpretable — hard to explain why a result matched
- Can return confidently wrong results (high similarity, low relevance)

**When to use:** When agents frequently ask conceptual questions that
keyword search can't answer. Not worth the complexity for small,
well-organized collections where agents know the terminology.

## Hybrid (BM25 + Vector)

Run both searches, merge results with Reciprocal Rank Fusion (RRF).

**Strengths:**
- Best of both worlds — catches both exact terms and semantic matches
- RRF is simple: `score = Σ 1/(k + rank_i)` across search methods
- Degrades gracefully — if one method returns nothing, the other still works

**Weaknesses:**
- Double the computation and complexity
- Requires tuning the fusion constant `k` (60 is a common default)
- Two indexes to maintain and keep in sync

**When to use:** For large knowledge bases (500+ docs) where agents ask
both factual and conceptual questions. The complexity is justified when
the collection is big enough that keyword search alone misses important
results.

## LLM Re-ranking

Use an LLM to re-score the top-N results from BM25/vector search.

**Strengths:**
- Can evaluate relevance with full context understanding
- Catches subtle matches that statistical methods miss
- Works as a quality filter on top of any retrieval method

**Weaknesses:**
- Expensive — one LLM call per re-ranking pass
- Adds latency to every search
- Overkill for most knowledge base queries

**When to use:** Rarely. Consider it when search quality is critical and
the cost is justified — e.g., a customer-facing knowledge base or a
safety-critical decision-support system.

## Practical Guidance

For most markdown knowledge bases:

1. **Start with BM25.** Fast, simple, no dependencies beyond Tantivy.
2. **Add vector search when BM25 fails.** Track queries that return poor
   results — if they're conceptual, vector search will help.
3. **Use hybrid with RRF** as the combination strategy. Don't build
   custom merging logic.
4. **Skip LLM re-ranking** unless you have a specific, measured quality
   problem that cheaper methods can't solve.
