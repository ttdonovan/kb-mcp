# Hybrid Search

By default, kb-mcp uses BM25 keyword search via Tantivy. Enable the
`hybrid` feature to add vector similarity search alongside BM25.

## Why Hybrid?

BM25 is excellent for exact keyword queries ("PostgreSQL connection pool",
"rate limit error"). But it misses conceptual matches:

| Query | BM25 alone | Hybrid (BM25 + vector) |
|-------|-----------|----------------------|
| "how do agents share state?" | Misses "Shared Memory" doc | Matches via semantic similarity |
| "memory architecture" | Finds docs with those exact words | Also finds "Cognitive Memory Model" |
| "BM25 ranking" | Works perfectly | Works perfectly (BM25 still contributes) |

Hybrid search combines both signals using Reciprocal Rank Fusion (RRF),
so keyword precision isn't lost — it's augmented.

## How It Works

1. **At ingest time:** Each document is embedded into a 384-dimensional
   vector using BGE-small-en-v1.5 (local ONNX, no cloud API)
2. **At query time:** The query is embedded, then both BM25 and HNSW
   vector search run in parallel
3. **Fusion:** Results are merged via RRF (`score = Σ 1/(k + rank)`)
   with k=60, combining keyword and semantic signals

The vector index is stored inside the `.mv2` file alongside the Tantivy
BM25 index — no separate database or service.

## Setup

### 1. Install with hybrid feature

```sh
cargo install --path . --features hybrid
```

This pulls in ONNX Runtime and the HNSW library. Build time is longer
than the default BM25-only build.

### 2. Download the embedding model

The BGE-small-en-v1.5 model files (~34MB total) must be present locally:

```sh
mkdir -p ~/.cache/memvid/text-models

curl -L -o ~/.cache/memvid/text-models/bge-small-en-v1.5.onnx \
  https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/onnx/model.onnx

curl -L -o ~/.cache/memvid/text-models/bge-small-en-v1.5_tokenizer.json \
  https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main/tokenizer.json
```

### 3. Re-index to build vector index

Existing `.mv2` files only have BM25 data. Delete the cache to force
a full re-ingest with embeddings:

```sh
rm -rf ~/.cache/kb-mcp/   # or ~/Library/Caches/kb-mcp/ on macOS
kb-mcp list-sections      # triggers re-ingest with vectors
```

## Usage

No changes to your queries. The `search` tool automatically uses hybrid
mode when compiled with the feature. Agents get better results without
knowing about search modes.

```sh
# These work the same — but with hybrid, conceptual matches are found
kb-mcp search --query "how do agents share state?"
kb-mcp search --query "memory architecture" --collection vault
```

## Default Build (No Hybrid)

If you don't need vector search, the default build stays lightweight:

```sh
cargo install --path .   # BM25 only, no ONNX dependency
```

The same search queries work — they just use BM25 keyword matching
without the vector similarity signal.

## Technical Details

- **Model:** BGE-small-en-v1.5 (BAAI), 384 dimensions
- **Index:** HNSW graph (brute-force below 1000 vectors)
- **Distance:** L2 (Euclidean)
- **Fusion:** Reciprocal Rank Fusion, k=60
- **Storage:** Vectors stored inside `.mv2` files alongside Tantivy
- **Embedding cache:** LRU, 1000 entries, auto-unloads after 5min idle
