//! Filesystem scanning and document parsing.
//!
//! Walks each collection's directory tree, parses markdown files (extracting
//! YAML frontmatter, title, section), and builds the in-memory document index.
//! Section descriptions come from the RON config, not from the documents
//! themselves — this is what makes kb-mcp project-agnostic.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::config::{ResolvedCollection, SectionDef};
use crate::store;
use crate::types::{Document, Section};

/// The in-memory document index. Rebuilt on startup and on `reindex` calls.
/// Also carries per-collection content hashes for incremental `.mv2` sync.
pub struct Index {
    pub documents: Vec<Document>,
    pub sections: Vec<Section>,
    /// Per-collection content hashes: collection_name → (relative_path → blake3 hash).
    /// Used by `store::sync_collection` to determine which docs changed.
    pub content_hashes: HashMap<String, store::HashIndex>,
}

impl Index {
    pub fn build(collections: &[ResolvedCollection]) -> Self {
        let mut documents = Vec::new();
        let mut content_hashes: HashMap<String, store::HashIndex> = HashMap::new();

        for collection in collections {
            if collection.path.is_dir() {
                let mut coll_hashes = store::HashIndex::new();
                scan_dir(
                    &collection.path,
                    &collection.path,
                    &collection.name,
                    &collection.sections,
                    &mut documents,
                    &mut coll_hashes,
                );
                content_hashes.insert(collection.name.clone(), coll_hashes);
            }
        }

        let sections = build_sections(&documents, collections);

        Index {
            documents,
            sections,
            content_hashes,
        }
    }

    /// Rebuild a single collection's documents and hashes in place.
    ///
    /// Removes all documents belonging to `collection_name`, re-scans the
    /// collection's directory, and rebuilds sections from the updated doc list.
    /// Much cheaper than `Index::build` when only one collection changed.
    pub fn rebuild_collection(
        &mut self,
        collection: &ResolvedCollection,
        all_collections: &[ResolvedCollection],
    ) {
        // Remove old docs for this collection
        self.documents.retain(|d| d.collection != collection.name);

        // Re-scan the collection
        if collection.path.is_dir() {
            let mut coll_hashes = store::HashIndex::new();
            scan_dir(
                &collection.path,
                &collection.path,
                &collection.name,
                &collection.sections,
                &mut self.documents,
                &mut coll_hashes,
            );
            self.content_hashes
                .insert(collection.name.clone(), coll_hashes);
        } else {
            self.content_hashes.remove(&collection.name);
        }

        // Rebuild sections from the full (updated) document list
        self.sections = build_sections(&self.documents, all_collections);
    }

    pub fn find_by_path(&self, path: &str) -> Option<&Document> {
        self.documents.iter().find(|d| d.path == path)
    }

    pub fn find_by_title(&self, title: &str) -> Option<&Document> {
        let lower = title.to_lowercase();
        self.documents
            .iter()
            .find(|d| d.title.to_lowercase() == lower)
    }
}

fn scan_dir(
    base: &Path,
    dir: &Path,
    collection_name: &str,
    section_defs: &[SectionDef],
    docs: &mut Vec<Document>,
    hashes: &mut store::HashIndex,
) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_dir(base, &path, collection_name, section_defs, docs, hashes);
        } else if path.extension().is_some_and(|e| e == "md")
            && let Some((doc, hash)) = parse_document_with_hash(base, &path, collection_name, section_defs)
        {
            hashes.insert(doc.path.clone(), hash);
            docs.push(doc);
        }
    }
}

/// Parse a markdown file into a Document and compute its blake3 content hash.
/// Returns both so the caller can build the hash index alongside the document list.
fn parse_document_with_hash(
    base: &Path,
    file_path: &PathBuf,
    collection_name: &str,
    section_defs: &[SectionDef],
) -> Option<(Document, String)> {
    let content = std::fs::read_to_string(file_path).ok()?;
    let content_hash = store::hash_content(content.as_bytes());
    let rel_path = file_path.strip_prefix(base).ok()?;
    let rel_str = rel_path.to_string_lossy().to_string();

    // Section = first directory component
    let section = if rel_path.components().count() > 1 {
        rel_path
            .components()
            .next()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .unwrap_or_default()
    } else {
        String::new()
    };

    let (frontmatter, tags, body) = parse_frontmatter(&content);

    let title = body
        .lines()
        .find(|l| l.starts_with("# "))
        .map(|l| l.trim_start_matches("# ").to_string())
        .unwrap_or_else(|| {
            rel_path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default()
        });

    let _ = section_defs; // used by build_sections, not here

    Some((
        Document {
            path: rel_str,
            title,
            tags,
            body,
            section,
            collection: collection_name.to_string(),
            frontmatter,
        },
        content_hash,
    ))
}

/// Parse YAML frontmatter delimited by `---`. Returns all fields as a HashMap
/// (for `kb_context`), extracted tags (for search indexing), and the body with
/// frontmatter stripped. Gracefully handles missing or malformed frontmatter.
fn parse_frontmatter(content: &str) -> (HashMap<String, serde_yaml::Value>, Vec<String>, String) {
    if !content.starts_with("---") {
        return (HashMap::new(), vec![], content.to_string());
    }

    let after_first = &content[3..];
    if let Some(end) = after_first.find("\n---") {
        let yaml_str = &after_first[..end];
        let body = &after_first[end + 4..];

        let fm: HashMap<String, serde_yaml::Value> =
            serde_yaml::from_str(yaml_str).unwrap_or_default();

        let tags = fm
            .get("tags")
            .and_then(|v| serde_yaml::from_value::<Vec<String>>(v.clone()).ok())
            .unwrap_or_default();

        (fm, tags, body.trim_start_matches('\n').to_string())
    } else {
        (HashMap::new(), vec![], content.to_string())
    }
}

/// Derive sections from document counts, enriched with descriptions from RON config.
/// Sections without a config definition still appear — they just have an empty description.
fn build_sections(documents: &[Document], collections: &[ResolvedCollection]) -> Vec<Section> {
    // Descriptions come from RON config, not from documents.
    let mut desc_map: HashMap<(&str, &str), &str> = HashMap::new();
    for coll in collections {
        for sec in &coll.sections {
            desc_map.insert((&coll.name, &sec.prefix), &sec.description);
        }
    }

    // Count docs per (collection, section)
    let mut counts: HashMap<(String, String), usize> = HashMap::new();
    for doc in documents {
        if !doc.section.is_empty() {
            *counts
                .entry((doc.collection.clone(), doc.section.clone()))
                .or_default() += 1;
        }
    }

    let mut sections: Vec<Section> = counts
        .into_iter()
        .map(|((collection, name), doc_count)| {
            let description = desc_map
                .get(&(collection.as_str(), name.as_str()))
                .map(|d| d.to_string())
                .unwrap_or_default();

            Section {
                name,
                description,
                doc_count,
                collection,
            }
        })
        .collect();

    sections.sort_by(|a, b| (&a.collection, &a.name).cmp(&(&b.collection, &b.name)));
    sections
}
