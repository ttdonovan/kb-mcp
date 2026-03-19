//! BM25 full-text search via Tantivy.
//!
//! Builds an in-RAM index from all documents across all collections. Search
//! queries run against the full index; collection and section filtering happen
//! post-query because Tantivy's STRING fields don't support efficient pre-filtering
//! in a combined query. The 5× over-fetch compensates for post-filter reduction.

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::snippet::SnippetGenerator;
use tantivy::{Index, IndexWriter, TantivyDocument};

use crate::types::Document;

/// Wraps a Tantivy in-RAM index. Rebuilt from scratch on reindex — this is
/// fast enough for hundreds of documents and avoids incremental update complexity.
pub struct SearchEngine {
    index: Index,
    path_field: Field,
    title_field: Field,
    body_field: Field,
    tags_field: Field,
    section_field: Field,
    collection_field: Field,
}

pub struct SearchResult {
    pub doc_index: usize,
    pub score: f64,
    pub excerpt: String,
}

impl SearchEngine {
    pub fn build(documents: &[Document]) -> Self {
        let mut schema_builder = Schema::builder();

        let path_field = schema_builder.add_text_field("path", STRING | STORED);
        let title_field = schema_builder.add_text_field("title", TEXT | STORED);
        let body_field = schema_builder.add_text_field("body", TEXT | STORED);
        let tags_field = schema_builder.add_text_field("tags", TEXT | STORED);
        let section_field = schema_builder.add_text_field("section", STRING | STORED);
        let collection_field = schema_builder.add_text_field("collection", STRING | STORED);

        let schema = schema_builder.build();
        let index = Index::create_in_ram(schema);

        let engine = SearchEngine {
            index,
            path_field,
            title_field,
            body_field,
            tags_field,
            section_field,
            collection_field,
        };

        let mut writer = engine.writer();
        engine.add_documents(&mut writer, documents);
        writer.commit().expect("failed to commit index");
        drop(writer);

        engine
    }

    fn writer(&self) -> IndexWriter {
        self.index
            .writer(15_000_000)
            .expect("failed to create index writer")
    }

    fn add_documents(&self, writer: &mut IndexWriter, documents: &[Document]) {
        for doc in documents {
            let mut tantivy_doc = TantivyDocument::new();
            tantivy_doc.add_text(self.path_field, &doc.path);
            tantivy_doc.add_text(self.title_field, &doc.title);
            tantivy_doc.add_text(self.body_field, &doc.body);
            tantivy_doc.add_text(self.tags_field, doc.tags.join(" "));
            tantivy_doc.add_text(self.section_field, &doc.section);
            tantivy_doc.add_text(self.collection_field, &doc.collection);

            writer
                .add_document(tantivy_doc)
                .expect("failed to add document");
        }
    }

    /// Replace the entire index contents. Used by `reindex` and `kb_write` to
    /// pick up filesystem changes without restarting the process.
    pub fn rebuild(&self, documents: &[Document]) {
        let mut writer = self.writer();
        writer
            .delete_all_documents()
            .expect("failed to clear index");
        self.add_documents(&mut writer, documents);
        writer.commit().expect("failed to commit reindex");
    }

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

        let reader = self.index.reader().expect("failed to create reader");
        let searcher = reader.searcher();

        let query_parser = QueryParser::for_index(
            &self.index,
            vec![self.title_field, self.body_field, self.tags_field],
        );

        let query = match query_parser.parse_query(query_str) {
            Ok(q) => q,
            Err(_) => return vec![],
        };

        let needs_filter = collection.is_some() || scope.is_some();
        let limit = if needs_filter {
            max_results * 5
        } else {
            max_results
        };

        let top_docs = match searcher.search(&query, &TopDocs::with_limit(limit)) {
            Ok(results) => results,
            Err(_) => return vec![],
        };

        let snippet_generator = SnippetGenerator::create(&searcher, &*query, self.body_field)
            .unwrap_or_else(|_| {
                SnippetGenerator::create(&searcher, &*query, self.title_field)
                    .expect("failed to create snippet generator")
            });

        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let retrieved: TantivyDocument = searcher
                .doc(doc_address)
                .expect("failed to retrieve document");

            let path = retrieved
                .get_first(self.path_field)
                .and_then(|v| v.as_str())
                .unwrap_or_default();

            let doc_index = match documents.iter().position(|d| d.path == path) {
                Some(idx) => idx,
                None => continue,
            };

            let doc = &documents[doc_index];

            // Filter by collection
            if let Some(coll) = collection
                && doc.collection != coll {
                    continue;
                }

            // Filter by section scope
            if let Some(scope) = scope
                && !doc.section.starts_with(scope) {
                    continue;
                }

            let snippet = snippet_generator.snippet_from_doc(&retrieved);
            let excerpt = if snippet.is_empty() {
                doc.body.lines().take(3).collect::<Vec<_>>().join("\n")
            } else {
                snippet.fragment().to_string()
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
