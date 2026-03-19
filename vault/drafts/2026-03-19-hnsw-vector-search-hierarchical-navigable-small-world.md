---
tags: [vector-search, ann, hnsw, graphs, retrieval]
created: 2026-03-19
updated: 2026-03-19
status: draft
source: arxiv + hnswlib + faiss wiki
---

# HNSW Vector Search (Hierarchical Navigable Small World)

# HNSW Vector Search (Hierarchical Navigable Small World)

HNSW is a popular **approximate nearest neighbor (ANN)** index for vector similarity search. It represents the dataset as a **multi-layer proximity graph** and answers queries by **navigating the graph** from coarse to fine layers, trading accuracy for speed via a small set of tunable parameters.

## Core idea

- **Graph-based ANN:** each vector is a node; edges connect it to a limited number of “nearby” nodes.
- **Hierarchy (layers):** nodes appear in multiple layers with an **exponentially decaying probability** of being present in higher layers.
- **Search procedure:** start from an entry point in the **top layer**, do greedy navigation to get close, then descend layer-by-layer, using a bounded best-first expansion at the bottom layer to collect the final k neighbors.

The original paper emphasizes that the hierarchy acts like a **skip-list-like** structure, yielding efficient navigation and strong empirical performance across datasets.

## Key parameters (practical tuning)

Most HNSW implementations expose the following knobs:

### `M` — graph degree / connectivity
- Controls how many (bi-directional) links are created per inserted element.
- Larger `M` increases **recall** but also increases **memory** and **construction time**.
- Rule of thumb from hnswlib docs:
  - Reasonable range: **2–100**
  - Commonly useful range: **12–48**
  - High-dimensional embeddings at high recall may benefit from **48–64**
- **Memory:** roughly **`M * 8–10 bytes` per stored element** for link storage (implementation-dependent).

### `efConstruction` — build-time search width
- Analogous to `efSearch` but used during insertion.
- Bigger values improve graph quality / recall, but slow builds.
- One suggested check: set `efSearch = efConstruction` and measure recall; if it’s **< 0.9**, you may benefit from increasing `efConstruction`.

### `efSearch` (often called `ef`) — query-time search width
- Size of the dynamic candidate list used during query graph exploration.
- Higher `efSearch` → higher recall but slower queries.
- Constraint: `efSearch >= k`.

## Complexity & behavior (high level)

- Empirically, HNSW often shows **near-logarithmic scaling** for search due to the hierarchy and “small-world” navigability.
- Performance depends heavily on:
  - intrinsic dimension / clustering of the dataset
  - choice of metric (L2 vs inner-product / cosine)
  - parameter settings (`M`, `efConstruction`, `efSearch`)

## Metrics: L2 vs inner product vs cosine

Many libraries support:
- **Squared L2**
- **Inner product** (maximum dot product)
- **Cosine similarity**

Important practical detail (hnswlib): **inner product is not a true metric**, which can affect graph properties; the hnswlib README notes that an element can be “closer to another element than to itself” under this distance definition.

For cosine similarity, a common approach is to **L2-normalize vectors** and then use inner product / dot product search.

## Deletions, updates, and mutability

Implementation support varies:

- **hnswlib** supports:
  - insertions and **updates** (re-adding with same id updates vector)
  - **mark_deleted / unmark_deleted** semantics
  - optional **replace deleted** elements to control index size
- **Faiss** (per its wiki): HNSW variants **do not support removing vectors** from the index because it would “destroy the graph structure.” (Some systems provide logical deletions via ID filtering/wrapping, but that is different from structural deletion.)

## Where HNSW fits among ANN methods

HNSW is often chosen when you want:
- strong recall/latency trade-offs on CPUs
- relatively simple ops model (build + add + search)
- good performance without needing training (unlike IVF/PQ families)

However, for very large-scale deployments, you might consider hybrids:
- **IVF + (HNSW as a coarse quantizer)** is used in some systems to speed assignment to inverted lists.
- **HNSW + quantization** (e.g., Faiss’s `IndexHNSWSQ`, `IndexHNSWPQ`) can reduce memory by compressing stored vectors while keeping HNSW routing.

## Implementation notes (Faiss)

Faiss exposes HNSW as a family of indexes:
- `IndexHNSWFlat` (no compression)
- `IndexHNSWSQ` / `IndexHNSWPQ` (scalar / product quantization-backed storage)

The Faiss wiki highlights the three key parameters:
- `M`
- `efConstruction`
- `efSearch`

## Quick tuning recipes

1. **Start point (common defaults):**
   - `M = 16`
   - `efConstruction = 100–200`
   - `efSearch = 50–200` (must be ≥ k)
2. **If recall is too low:** increase `efSearch` first; then increase `efConstruction`; then consider increasing `M`.
3. **If memory is too high:** reduce `M` or use a compressed storage backend (SQ/PQ variants) if supported.
4. **If build time is too slow:** reduce `efConstruction` and/or `M`.

## Sources

- Malkov, Y. A., & Yashunin, D. A. **“Efficient and robust approximate nearest neighbor search using Hierarchical Navigable Small World graphs.”** arXiv:1603.09320 (v4, 2018). https://arxiv.org/abs/1603.09320
- hnswlib README (API, supported distances, mutability features): https://github.com/nmslib/hnswlib (raw: https://github.com/nmslib/hnswlib/raw/refs/heads/master/README.md)
- hnswlib parameter guide (`M`, `ef`, `efConstruction`, memory notes): https://github.com/nmslib/hnswlib/blob/master/ALGO_PARAMS.md (raw: https://github.com/nmslib/hnswlib/raw/refs/heads/master/ALGO_PARAMS.md)
- Faiss wiki “Faiss indexes” (HNSW parameters, variants, removal limitation): https://github.com/facebookresearch/faiss/wiki/Faiss-indexes

