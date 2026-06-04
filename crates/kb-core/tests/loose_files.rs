//! Regression tests for loose root-level files (e.g. `languages/rust.md`).
//!
//! Documents sitting directly at a collection root used to get an empty
//! section, which made them invisible to `list_sections`, absent from the
//! digest's sections/topics, and unselectable by any search scope — even
//! though they were indexed and retrievable by exact path. They now carry
//! the `ROOT_SECTION` sentinel so every section-shaped view includes them.

use std::path::PathBuf;

use kb_core::config::{ResolvedCollection, SectionDef};
use kb_core::index::Index;
use kb_core::types::ROOT_SECTION;

/// Build a fixture vault with one subdirectory section (`golang/`) and one
/// loose file (`rust.md`) at the collection root.
fn fixture_collection(test_name: &str) -> (PathBuf, ResolvedCollection) {
    let root = std::env::temp_dir().join(format!("kb-mcp-loose-files-{}", test_name));
    let _ = std::fs::remove_dir_all(&root);
    let languages = root.join("languages");
    std::fs::create_dir_all(languages.join("golang")).unwrap();

    std::fs::write(
        languages.join("golang/web-frameworks.md"),
        "# Go Web Frameworks\n\nGin and Echo are common Go web frameworks.\n",
    )
    .unwrap();
    std::fs::write(
        languages.join("rust.md"),
        "# Rust Best Practices\n\nOwnership and the borrow checker prevent data races.\n",
    )
    .unwrap();

    let collection = ResolvedCollection {
        name: "languages".to_string(),
        path: languages,
        description: "Language docs".to_string(),
        writable: false,
        sections: vec![SectionDef {
            prefix: "golang".to_string(),
            description: "Go docs".to_string(),
        }],
    };

    (root, collection)
}

#[test]
fn loose_root_file_gets_root_section() {
    let (root, collection) = fixture_collection("section");
    let index = Index::build(&[collection]);

    let rust = index.find_by_path("rust.md").expect("rust.md indexed");
    assert_eq!(rust.section, ROOT_SECTION);

    let go = index
        .find_by_path("golang/web-frameworks.md")
        .expect("golang doc indexed");
    assert_eq!(go.section, "golang");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn loose_root_file_appears_in_section_listing() {
    let (root, collection) = fixture_collection("listing");
    let index = Index::build(&[collection]);

    let root_section = index
        .sections
        .iter()
        .find(|s| s.name == ROOT_SECTION && s.collection == "languages")
        .expect("root section listed");
    assert_eq!(root_section.doc_count, 1);

    // Every document is accounted for by some section.
    let counted: usize = index.sections.iter().map(|s| s.doc_count).sum();
    assert_eq!(counted, index.documents.len());

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn loose_root_file_appears_in_digest() {
    let (root, collection) = fixture_collection("digest");
    let index = Index::build(&[collection]);

    let digest = kb_core::format::format_digest(&index.documents, &index.sections, None);
    assert!(
        digest.contains(ROOT_SECTION),
        "digest lists the root section: {digest}"
    );
    assert!(
        digest.contains("Rust Best Practices"),
        "digest surfaces the loose doc's topic: {digest}"
    );

    std::fs::remove_dir_all(&root).ok();
}

/// Search-engine behavior (ingestion, scope filtering, forced rebuild) is
/// exercised BM25-only; the hybrid path differs only in embedding, not in
/// which documents are ingested or filtered.
#[cfg(not(feature = "hybrid"))]
mod search {
    use std::collections::HashMap;

    use kb_core::search::SearchEngine;
    use kb_core::store;
    use kb_core::types::ROOT_SECTION;

    use super::fixture_collection;

    #[test]
    fn loose_root_file_is_searchable_and_scopable() {
        let (root, collection) = fixture_collection("search");
        let cache_dir = root.join("cache");
        let index = kb_core::index::Index::build(std::slice::from_ref(&collection));

        let current_hashes = index
            .content_hashes
            .get(&collection.name)
            .cloned()
            .unwrap_or_default();
        let (mem, changes) =
            store::sync_collection(&cache_dir, &collection, &current_hashes, &index.documents)
                .expect("sync succeeds");
        assert_eq!(changes, 2, "both docs ingested");

        let engine = SearchEngine::new(HashMap::from([("languages".to_string(), mem)]));

        // Unscoped search finds the loose doc.
        let results = engine.search(&index.documents, "ownership borrow checker", None, None, 10);
        assert!(
            results
                .iter()
                .any(|r| index.documents[r.doc_index].path == "rust.md"),
            "unscoped search finds rust.md"
        );

        // The root section is a selectable scope.
        let results = engine.search(
            &index.documents,
            "ownership borrow checker",
            None,
            Some(ROOT_SECTION),
            10,
        );
        assert!(
            results
                .iter()
                .any(|r| index.documents[r.doc_index].path == "rust.md"),
            "scope \"{ROOT_SECTION}\" selects rust.md"
        );

        // A subdirectory scope still excludes it.
        let results = engine.search(
            &index.documents,
            "ownership borrow checker",
            None,
            Some("golang"),
            10,
        );
        assert!(
            results
                .iter()
                .all(|r| index.documents[r.doc_index].path != "rust.md"),
            "scope \"golang\" excludes rust.md"
        );

        std::fs::remove_dir_all(&root).ok();
    }

    #[test]
    fn clear_collection_store_forces_full_reingest() {
        let (root, collection) = fixture_collection("rebuild");
        let cache_dir = root.join("cache");
        let index = kb_core::index::Index::build(std::slice::from_ref(&collection));

        let current_hashes = index
            .content_hashes
            .get(&collection.name)
            .cloned()
            .unwrap_or_default();

        let (_, changes) =
            store::sync_collection(&cache_dir, &collection, &current_hashes, &index.documents)
                .expect("initial sync succeeds");
        assert_eq!(changes, 2);

        // Incremental sync trusts the sidecar: nothing to do.
        let (_, changes) =
            store::sync_collection(&cache_dir, &collection, &current_hashes, &index.documents)
                .expect("re-sync succeeds");
        assert_eq!(changes, 0);

        // Clearing the store discards the sidecar, so the next sync
        // re-ingests everything — the repair path for stale sidecars.
        store::clear_collection_store(&cache_dir, &collection).expect("clear succeeds");
        let (_, changes) =
            store::sync_collection(&cache_dir, &collection, &current_hashes, &index.documents)
                .expect("post-clear sync succeeds");
        assert_eq!(changes, 2, "full re-ingest after clear");

        std::fs::remove_dir_all(&root).ok();
    }
}
