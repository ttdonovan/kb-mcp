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
use std::sync::Mutex;

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
        self.inner.model_info().dims as usize
    }
}

impl SearchEngine {
    /// Create from a map of collection_name → Memvid handle.
    pub fn new(
        stores: HashMap<String, Memvid>,
        #[cfg(feature = "hybrid")] embedder: std::sync::Arc<memvid_core::LocalTextEmbedder>,
    ) -> Self {
        Self {
            stores: Mutex::new(stores),
            #[cfg(feature = "hybrid")]
            embedder,
        }
    }

    /// Replace the Memvid handle for a single collection (after kb_write or reindex).
    pub fn replace_store(&self, collection_name: &str, mem: Memvid) {
        let mut stores = self.stores.lock().unwrap();
        stores.insert(collection_name.to_string(), mem);
    }

    /// Replace all stores (after full reindex).
    pub fn replace_all_stores(&self, new_stores: HashMap<String, Memvid>) {
        let mut stores = self.stores.lock().unwrap();
        *stores = new_stores;
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

        let mut stores = self.stores.lock().unwrap();

        // (collection_name, uri, score, snippet)
        let mut all_hits: Vec<(String, String, f32, String)> = Vec::new();

        let collection_names: Vec<String> = if let Some(coll) = collection {
            vec![coll.to_string()]
        } else {
            stores.keys().cloned().collect()
        };

        for coll_name in &collection_names {
            let Some(mem) = stores.get_mut(coll_name) else {
                continue;
            };

            let top_k = max_results * 3; // over-fetch for dedup + post-filter

            #[cfg(feature = "hybrid")]
            {
                let adapter = EmbedderAdapter { inner: &self.embedder };
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

        drop(stores); // release lock before document mapping

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
        let mut ranked: Vec<(String, String, f32, String)> = best_by_uri
            .into_iter()
            .map(|(uri, (coll, score, snippet))| (coll, uri, score, snippet))
            .collect();
        ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Map back to document index + apply scope filter
        let mut results = Vec::new();
        for (_, uri, score, snippet) in ranked {
            let Some((_coll_name, rel_path)) = store::parse_uri(&uri) else {
                continue;
            };

            let doc_index = match documents.iter().position(|d| d.path == rel_path) {
                Some(idx) => idx,
                None => continue,
            };

            let doc = &documents[doc_index];

            if let Some(scope) = scope
                && !doc.section.starts_with(scope)
            {
                continue;
            }

            // Prefer memvid's snippet (relevant chunk), fall back to first paragraph
            let excerpt = if !snippet.is_empty() {
                snippet
            } else {
                crate::format::extract_summary(&doc.body)
            };

            results.push(SearchResult {
                doc_index,
                score: score as f64,
                excerpt,
            });

            if results.len() >= max_results {
                break;
            }
        }

        results
    }
}
