//! Full-text search via memvid-core.
//!
//! Each collection has its own `.mv2` file with an embedded Tantivy index.
//! Search queries fan out across collections (unless filtered), results are
//! merged by score and deduplicated by URI (one result per document, highest-
//! scoring chunk wins).
//!
//! When the `hybrid` feature is enabled, search uses `Memvid::ask()` with
//! RRF fusion of BM25 + vector results. Otherwise, BM25-only via `search()`.

use std::collections::HashMap;
use std::fs::Metadata;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};

use memvid_core::Memvid;

use crate::store;
use crate::types::Document;

/// Holds per-collection Memvid handles behind a Mutex.
/// Both `search()` and `ask()` require `&mut self`, so Mutex is required.
pub struct SearchEngine {
    stores: Mutex<HashMap<String, Memvid>>,
    #[cfg(feature = "hybrid")]
    embedder: std::sync::Arc<memvid_core::LocalTextEmbedder>,
}

pub struct SearchResult {
    pub doc_index: usize,
    pub score: f64,
    pub excerpt: String,
}

/// Adapter: memvid's `ask()` needs `VecEmbedder` trait, but `LocalTextEmbedder`
/// implements `EmbeddingProvider`. This bridges the two.
#[cfg(feature = "hybrid")]
struct EmbedderAdapter<'a> {
    inner: &'a memvid_core::LocalTextEmbedder,
}

#[cfg(feature = "hybrid")]
impl memvid_core::VecEmbedder for EmbedderAdapter<'_> {
    fn embed_query(&self, text: &str) -> memvid_core::Result<Vec<f32>> {
        use memvid_core::EmbeddingProvider;
        self.inner.embed_text(text)
    }

    fn embedding_dimension(&self) -> usize {
        // u32 always fits in usize on the 32- and 64-bit targets kb-mcp builds for.
        usize::try_from(self.inner.model_info().dims).unwrap_or(usize::MAX)
    }
}

impl SearchEngine {
    /// Create from a map of `collection_name` → Memvid handle.
    #[must_use]
    pub const fn new(
        stores: HashMap<String, Memvid>,
        #[cfg(feature = "hybrid")] embedder: std::sync::Arc<memvid_core::LocalTextEmbedder>,
    ) -> Self {
        Self {
            stores: Mutex::new(stores),
            #[cfg(feature = "hybrid")]
            embedder,
        }
    }

    /// The stores, even if a thread panicked while holding the lock: the map
    /// only holds handles, each replaced whole, so it is never left half-updated.
    fn stores(&self) -> MutexGuard<'_, HashMap<String, Memvid>> {
        self.stores.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Replace the Memvid handle for a single collection (after `kb_write` or reindex).
    pub fn replace_store(&self, collection_name: &str, mem: Memvid) {
        self.stores().insert(collection_name.to_string(), mem);
    }

    /// Replace all stores (after full reindex).
    pub fn replace_all_stores(&self, new_stores: HashMap<String, Memvid>) {
        *self.stores() = new_stores;
    }

    /// Whether the collection's `.mv2` at `mv2` is no longer the file our handle
    /// has open, i.e. another process synced and renamed a new store over it.
    ///
    /// Compares the held file with the path instead of the shared `.hashes`
    /// sidecar, whose mtime the other process has already bumped. Returns
    /// false when no handle is held or either side can't be stat'ed.
    pub fn store_replaced(&self, collection_name: &str, mv2: &Path) -> bool {
        // The guard is a temporary, released at the end of this statement.
        let Some(held) = self.stores().get(collection_name).map(|mem| {
            mem.lock_handle()
                .clone_handle()
                .and_then(|f| Ok(f.metadata()?))
                .ok()
        }) else {
            return false;
        };
        match (held, std::fs::metadata(mv2).ok()) {
            (Some(held), Some(on_disk)) => !same_file(&held, &on_disk),
            _ => false,
        }
    }

    /// Search across collections. When hybrid feature is enabled, uses RRF
    /// fusion of BM25 + vector results via `ask()`. Otherwise BM25-only.
    pub fn search(
        &self,
        documents: &[Document],
        query_str: &str,
        collection: Option<&str>,
        scope: Option<&str>,
        max_results: usize,
    ) -> Vec<SearchResult> {
        if query_str.trim().is_empty() {
            return vec![];
        }

        // Over-fetch for dedup and the scope filter.
        let top_k = max_results.saturating_mul(3);
        let all_hits = self.collect_hits(query_str, collection, top_k);
        rank_hits(all_hits, documents, scope, max_results)
    }

    /// Run the query against each selected store, holding the lock only for this.
    fn collect_hits(&self, query_str: &str, collection: Option<&str>, top_k: usize) -> Vec<Hit> {
        let mut stores = self.stores();

        // (collection_name, uri, score, snippet)
        let mut all_hits: Vec<Hit> = Vec::new();

        let collection_names: Vec<String> = collection.map_or_else(
            || stores.keys().cloned().collect(),
            |coll| vec![coll.to_string()],
        );

        for coll_name in &collection_names {
            let Some(mem) = stores.get_mut(coll_name) else {
                continue;
            };

            #[cfg(feature = "hybrid")]
            {
                let adapter = EmbedderAdapter {
                    inner: &self.embedder,
                };
                let request = memvid_core::AskRequest {
                    question: query_str.to_string(),
                    top_k,
                    snippet_chars: 300,
                    mode: memvid_core::AskMode::Hybrid,
                    context_only: true, // we only want retrieval, not LLM synthesis
                    uri: None,
                    scope: None,
                    cursor: None,
                    start: None,
                    end: None,
                    as_of_frame: None,
                    as_of_ts: None,
                    adaptive: None,
                    acl_context: None,
                    acl_enforcement_mode: memvid_core::AclEnforcementMode::Audit,
                };

                if let Ok(response) = mem.ask(request, Some(&adapter)) {
                    for hit in response.retrieval.hits {
                        let score = hit.score.unwrap_or(0.0);
                        let snippet = hit.text.clone();
                        all_hits.push((coll_name.clone(), hit.uri.clone(), score, snippet));
                    }
                }
            }

            #[cfg(not(feature = "hybrid"))]
            {
                let request = memvid_core::SearchRequest {
                    query: query_str.to_string(),
                    top_k,
                    snippet_chars: 300,
                    uri: None,
                    scope: None,
                    cursor: None,
                    as_of_frame: None,
                    as_of_ts: None,
                    no_sketch: false,
                    acl_context: None,
                    acl_enforcement_mode: memvid_core::AclEnforcementMode::Audit,
                };

                if let Ok(response) = mem.search(request) {
                    for hit in response.hits {
                        let score = hit.score.unwrap_or(0.0);
                        let snippet = hit.text.clone();
                        all_hits.push((coll_name.clone(), hit.uri.clone(), score, snippet));
                    }
                }
            }
        }

        drop(stores);

        all_hits
    }
}

/// (collection name, uri, score, snippet) of one chunk a store returned.
type Hit = (String, String, f32, String);

/// Keep the best chunk per document, rank by score, map to indexed documents
/// and apply the scope filter.
fn rank_hits(
    all_hits: Vec<Hit>,
    documents: &[Document],
    scope: Option<&str>,
    max_results: usize,
) -> Vec<SearchResult> {
    // Deduplicate by URI — keep highest-scoring chunk per document
    let mut best_by_uri: HashMap<String, (String, f32, String)> = HashMap::new();
    for (coll, uri, score, snippet) in all_hits {
        let entry = best_by_uri
            .entry(uri.clone())
            .or_insert_with(|| (coll.clone(), score, snippet.clone()));
        if score > entry.1 {
            *entry = (coll, score, snippet);
        }
    }

    // Sort by score descending
    let mut ranked: Vec<Hit> = best_by_uri
        .into_iter()
        .map(|(uri, (coll, score, snippet))| (coll, uri, score, snippet))
        .collect();
    ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Map back to document index + apply scope filter
    let mut results = Vec::new();
    for (_, uri, score, snippet) in ranked {
        let Some((coll_name, rel_path)) = store::parse_uri(&uri) else {
            continue;
        };
        let Some((doc_index, doc)) = documents
            .iter()
            .enumerate()
            .find(|(_, d)| d.collection == coll_name && d.path == rel_path)
        else {
            continue;
        };

        if let Some(scope) = scope
            && !doc.section.starts_with(scope)
        {
            continue;
        }

        // Prefer memvid's snippet (relevant chunk), fall back to first paragraph
        let excerpt = if snippet.is_empty() {
            crate::format::extract_summary(&doc.body)
        } else {
            snippet
        };

        results.push(SearchResult {
            doc_index,
            score: f64::from(score),
            excerpt,
        });

        if results.len() >= max_results {
            break;
        }
    }

    results
}

/// Stores are replaced by rename, never modified in place, so a new inode
/// means a new store. Non-unix falls back to size + mtime.
#[cfg(unix)]
fn same_file(a: &Metadata, b: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    a.dev() == b.dev() && a.ino() == b.ino()
}

#[cfg(not(unix))]
fn same_file(a: &Metadata, b: &Metadata) -> bool {
    a.len() == b.len() && a.modified().ok() == b.modified().ok()
}
