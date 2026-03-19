//! BM25 full-text search via memvid-core.
//!
//! Each collection has its own `.mv2` file with an embedded Tantivy index.
//! Search queries fan out across collections (unless filtered), results are
//! merged by score and deduplicated by URI (one result per document, highest-
//! scoring chunk wins).

use std::collections::HashMap;
use std::sync::Mutex;

use memvid_core::{AclEnforcementMode, Memvid, SearchRequest};

use crate::store;
use crate::types::Document;

/// Holds per-collection Memvid handles behind a Mutex.
/// `search()` requires `&mut self` on Memvid even for reads, so Mutex is required.
pub struct SearchEngine {
    stores: Mutex<HashMap<String, Memvid>>,
}

pub struct SearchResult {
    pub doc_index: usize,
    pub score: f64,
    pub excerpt: String,
}

impl SearchEngine {
    /// Create from a map of collection_name → Memvid handle.
    pub fn new(stores: HashMap<String, Memvid>) -> Self {
        Self {
            stores: Mutex::new(stores),
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

    /// Search across collections via memvid-core's embedded Tantivy.
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
        let mut all_hits: Vec<(String, String, f32)> = Vec::new(); // (collection, uri, score)

        // Fan out: query each relevant collection's .mv2
        let collection_names: Vec<String> = if let Some(coll) = collection {
            vec![coll.to_string()]
        } else {
            stores.keys().cloned().collect()
        };

        for coll_name in &collection_names {
            let Some(mem) = stores.get_mut(coll_name) else {
                continue;
            };

            let request = SearchRequest {
                query: query_str.to_string(),
                top_k: max_results * 3, // over-fetch for dedup + post-filter
                snippet_chars: 300,
                uri: None,
                scope: None,
                cursor: None,
                as_of_frame: None,
                as_of_ts: None,
                no_sketch: false,
                acl_context: None,
                acl_enforcement_mode: AclEnforcementMode::Audit,
            };

            if let Ok(response) = mem.search(request) {
                for hit in response.hits {
                    let score = hit.score.unwrap_or(0.0);
                    all_hits.push((coll_name.clone(), hit.uri.clone(), score));
                }
            }
        }

        drop(stores); // release lock before document mapping

        // Deduplicate by URI — keep highest-scoring chunk per document
        let mut best_by_uri: HashMap<String, (String, f32)> = HashMap::new();
        for (coll, uri, score) in all_hits {
            let entry = best_by_uri
                .entry(uri.clone())
                .or_insert_with(|| (coll.clone(), score));
            if score > entry.1 {
                *entry = (coll, score);
            }
        }

        // Sort by score descending
        let mut ranked: Vec<(String, String, f32)> = best_by_uri
            .into_iter()
            .map(|(uri, (coll, score))| (coll, uri, score))
            .collect();
        ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

        // Map back to document index + apply scope filter
        let mut results = Vec::new();
        for (_, uri, score) in ranked {
            let Some((_coll_name, rel_path)) = store::parse_uri(&uri) else {
                continue;
            };

            let doc_index = match documents.iter().position(|d| d.path == rel_path) {
                Some(idx) => idx,
                None => continue,
            };

            let doc = &documents[doc_index];

            // Post-filter by section scope
            if let Some(scope) = scope
                && !doc.section.starts_with(scope)
            {
                continue;
            }

            let excerpt = doc.body.lines().take(3).collect::<Vec<_>>().join("\n");

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
