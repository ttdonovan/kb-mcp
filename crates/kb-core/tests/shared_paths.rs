//! Regression tests for two collections holding a document at the same path
//! and title (issue #34). Search used to report the second collection's hit
//! as the first's, and lookups always returned the first collection's document.

#![expect(
    clippy::unwrap_used,
    reason = "a test: a failed setup step should fail it"
)]

use std::path::PathBuf;

use kb_core::config::{IndexNames, ResolvedCollection};
use kb_core::index::Index;

/// `docs/fleets/mining.md` and `guide/fleets/mining.md`, both `# Mining`.
fn fixture(test_name: &str) -> (PathBuf, Vec<ResolvedCollection>) {
    let root = std::env::temp_dir().join(format!("kb-mcp-shared-paths-{test_name}"));
    let _ = std::fs::remove_dir_all(&root);

    let collection = |name: &str, body: &str| {
        let path = root.join(name);
        std::fs::create_dir_all(path.join("fleets")).unwrap();
        std::fs::write(
            path.join("fleets/mining.md"),
            format!("# Mining\n\n{body}\n"),
        )
        .unwrap();
        ResolvedCollection {
            name: name.to_string(),
            path,
            description: name.to_string(),
            writable: false,
            sections: vec![],
            index_names: IndexNames::Path,
            exclude: kb_core::config::Exclude::default(),
        }
    };

    let collections = vec![
        collection("docs", "Reference: mining lasers yield ore."),
        collection("guide", "Guide: start mining near the station."),
    ];
    (root, collections)
}

#[test]
fn lookup_without_collection_is_ambiguous() {
    let (root, collections) = fixture("ambiguous");
    let index = Index::build(&collections);

    for query in ["fleets/mining.md", "Mining"] {
        let err = index.lookup(query, None).unwrap_err();
        assert!(
            err.contains("docs (fleets/mining.md)") && err.contains("guide (fleets/mining.md)"),
            "error lists both matches: {err}"
        );
        let hint = if query.contains('/') {
            "Pass a collection to choose one."
        } else {
            "Pass a collection or the path to choose one."
        };
        assert!(err.ends_with(hint), "{err}");
    }

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn lookup_with_collection_picks_that_one() {
    let (root, collections) = fixture("scoped");
    let index = Index::build(&collections);

    for coll in ["docs", "guide"] {
        for query in ["fleets/mining.md", "mining"] {
            let doc = index.lookup(query, Some(coll)).unwrap();
            assert_eq!(doc.collection, coll);
        }
    }
    let err = index.lookup("fleets/mining.md", Some("nope")).unwrap_err();
    assert!(err.contains("Collection 'nope' not found"), "{err}");

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn repeated_title_in_one_collection_asks_for_the_path() {
    let (root, collections) = fixture("one-collection");
    let guide = &collections.iter().find(|c| c.name == "guide").unwrap().path;
    std::fs::write(guide.join("mining-faq.md"), "# Mining\n\nFAQ.\n").unwrap();
    let index = Index::build(&collections);

    let err = index.lookup("Mining", Some("guide")).unwrap_err();
    assert!(
        err.contains("guide (fleets/mining.md)")
            && err.contains("guide (mining-faq.md)")
            && err.contains("Pass the path"),
        "{err}"
    );
    let err = index.lookup("Mining", None).unwrap_err();
    assert!(err.contains("Pass a collection or the path"), "{err}");
    assert_eq!(
        index.lookup("mining-faq.md", None).unwrap().collection,
        "guide"
    );

    std::fs::remove_dir_all(&root).ok();
}

#[cfg(not(feature = "hybrid"))]
#[test]
fn search_reports_each_hit_under_its_own_collection() {
    use std::collections::HashMap;

    use kb_core::search::SearchEngine;
    use kb_core::store;

    let (root, collections) = fixture("search");
    let cache_dir = root.join("cache");
    let index = Index::build(&collections);

    let mut stores = HashMap::new();
    for collection in &collections {
        let hashes = index
            .content_hashes
            .get(&collection.name)
            .cloned()
            .unwrap_or_default();
        let (mem, _) =
            store::sync_collection(&cache_dir, collection, &hashes, &index.documents).unwrap();
        stores.insert(collection.name.clone(), mem);
    }
    let engine = SearchEngine::new(stores);

    let results = engine.search(&index.documents, "mining", None, None, 10);
    let mut found: Vec<(&str, &str)> = results
        .iter()
        .filter_map(|r| index.documents.get(r.doc_index))
        .map(|d| (d.collection.as_str(), d.path.as_str()))
        .collect();
    found.sort_unstable();
    assert_eq!(
        found,
        [("docs", "fleets/mining.md"), ("guide", "fleets/mining.md")]
    );

    // Each excerpt comes from its own collection's file.
    for r in &results {
        let doc = index.documents.get(r.doc_index).unwrap();
        let other = if doc.collection == "docs" {
            "Guide:"
        } else {
            "Reference:"
        };
        assert!(
            !r.excerpt.contains(other),
            "{}: {}",
            doc.collection,
            r.excerpt
        );
    }

    std::fs::remove_dir_all(&root).ok();
}
