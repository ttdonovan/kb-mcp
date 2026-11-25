//! `index_names: Name` keeps one index per collection when the config and its
//! collections move, e.g. each plugin update installing to a new folder (issue #3).

#![cfg(not(feature = "hybrid"))]
#![expect(
    clippy::unwrap_used,
    reason = "a test: a failed setup step should fail it"
)]

use std::collections::HashMap;
use std::path::Path;

use kb_core::config::Config;
use kb_core::index::Index;
use kb_core::search::SearchEngine;
use kb_core::store;

const RON: &str = r#"(
    index_names: Name,
    collections: [(name: "wiki", path: "wiki", description: "Wiki")],
)"#;

fn install(dir: &Path, body: &str) {
    std::fs::create_dir_all(dir.join("wiki")).unwrap();
    std::fs::write(dir.join("collections.ron"), RON).unwrap();
    std::fs::write(dir.join("wiki/page.md"), body).unwrap();
}

/// Resolve the config at `dir`, sync its one collection, and search it.
fn start(dir: &Path, cache_dir: &Path, query: &str) -> (usize, bool) {
    let config: Config = ron::from_str(RON).unwrap();
    let mut resolved = config.resolve(dir).unwrap();
    let collection = resolved.collections.remove(0);
    let index = Index::build(std::slice::from_ref(&collection));
    let hashes = index
        .content_hashes
        .get("wiki")
        .cloned()
        .unwrap_or_default();
    let (mem, changes) =
        store::sync_collection(cache_dir, &collection, &hashes, &index.documents).unwrap();
    let engine = SearchEngine::new(HashMap::from([("wiki".to_string(), mem)]));
    let found = !engine
        .search(&index.documents, query, None, None, 5)
        .is_empty();
    (changes, found)
}

fn cache_files(cache_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(cache_dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn moved_config_reuses_its_index() {
    let root = std::env::temp_dir().join("kb-mcp-stable-index-names");
    let _ = std::fs::remove_dir_all(&root);
    let cache_dir = root.join("data/cache");

    install(&root.join("1.0"), "# Page\n\nAardvarks dig burrows.\n");
    let (changes, found) = start(&root.join("1.0"), &cache_dir, "aardvarks burrows");
    assert_eq!(changes, 1);
    assert!(found);
    let files = cache_files(&cache_dir);
    assert_eq!(files, ["wiki.hashes", "wiki.mv2"]);

    // Same content at a new path: reused as-is, nothing re-ingested.
    std::fs::rename(root.join("1.0"), root.join("1.1")).unwrap();
    let (changes, found) = start(&root.join("1.1"), &cache_dir, "aardvarks burrows");
    assert_eq!(changes, 0);
    assert!(found);
    assert_eq!(cache_files(&cache_dir), files);

    // A new version with changed content updates the same file.
    install(&root.join("1.2"), "# Page\n\nPangolins roll into balls.\n");
    let (changes, found) = start(&root.join("1.2"), &cache_dir, "pangolins");
    assert_eq!(changes, 1);
    assert!(found);
    assert_eq!(cache_files(&cache_dir), files);

    std::fs::remove_dir_all(&root).ok();
}
