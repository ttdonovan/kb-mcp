//! A collection's `exclude` globs keep files out of the index, such as the
//! `SUMMARY.md` of an mdBook `src/` folder served as a collection (issue #34).

#![expect(
    clippy::unwrap_used,
    reason = "a test: a failed setup step should fail it"
)]

use std::path::{Path, PathBuf};

use kb_core::config::Config;
use kb_core::index::Index;

/// A collection `book` with a root `SUMMARY.md`, a nested one, drafts and two pages.
fn fixture(test_name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("kb-mcp-exclude-{test_name}"));
    let _ = std::fs::remove_dir_all(&root);
    let book = root.join("book");
    std::fs::create_dir_all(book.join("guide/drafts")).unwrap();
    for file in [
        "SUMMARY.md",
        "intro.md",
        "guide/SUMMARY.md",
        "guide/setup.md",
        "guide/drafts/wip.md",
    ] {
        std::fs::write(book.join(file), format!("# {file}\n\nBody.\n")).unwrap();
    }
    root
}

/// Index the fixture's `book` collection with the given `exclude` list.
fn indexed_paths(root: &Path, exclude: &str) -> Vec<String> {
    let ron = format!(
        r#"(collections: [(name: "book", path: "book", description: "Book", exclude: {exclude})])"#
    );
    let config: Config = ron::from_str(&ron).unwrap();
    let resolved = config.resolve(root).unwrap();
    let index = Index::build(&resolved.collections);
    let mut paths: Vec<String> = index.documents.into_iter().map(|d| d.path).collect();
    paths.sort_unstable();
    paths
}

#[test]
fn no_exclude_indexes_everything() {
    let root = fixture("none");
    let ron = r#"(collections: [(name: "book", path: "book", description: "Book")])"#;
    let config: Config = ron::from_str(ron).unwrap();
    let index = Index::build(&config.resolve(&root).unwrap().collections);
    assert_eq!(index.documents.len(), 5);
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn star_does_not_cross_directories() {
    let root = fixture("root-only");
    assert_eq!(
        indexed_paths(&root, r#"["SUMMARY.md"]"#),
        [
            "guide/SUMMARY.md",
            "guide/drafts/wip.md",
            "guide/setup.md",
            "intro.md"
        ]
    );
    assert_eq!(
        indexed_paths(&root, r#"["*.md"]"#),
        ["guide/SUMMARY.md", "guide/drafts/wip.md", "guide/setup.md"]
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn double_star_matches_at_any_depth() {
    let root = fixture("any-depth");
    assert_eq!(
        indexed_paths(&root, r#"["**/SUMMARY.md"]"#),
        ["guide/drafts/wip.md", "guide/setup.md", "intro.md"]
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn excluded_directory_is_skipped() {
    let root = fixture("directory");
    assert_eq!(
        indexed_paths(&root, r#"["guide/drafts"]"#),
        [
            "SUMMARY.md",
            "guide/SUMMARY.md",
            "guide/setup.md",
            "intro.md"
        ]
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn invalid_pattern_fails_config_resolution() {
    let ron =
        r#"(collections: [(name: "book", path: "book", description: "Book", exclude: ["[oops"])])"#;
    let config: Config = ron::from_str(ron).unwrap();
    let err = format!("{:#}", config.resolve(&std::env::temp_dir()).unwrap_err());
    assert!(
        err.contains("book") && err.contains("[oops"),
        "error names the collection and pattern: {err}"
    );
}

#[test]
fn trailing_slash_is_ignored() {
    let root = fixture("trailing-slash");
    assert_eq!(
        indexed_paths(&root, r#"["guide/drafts/"]"#),
        [
            "SUMMARY.md",
            "guide/SUMMARY.md",
            "guide/setup.md",
            "intro.md"
        ]
    );
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn write_into_an_excluded_path_is_refused() {
    let root = fixture("write");
    let ron = r#"(collections: [(name: "book", path: "book", description: "Book", exclude: ["guide/drafts", "**/SUMMARY.md"])])"#;
    let config: Config = ron::from_str(ron).unwrap();
    let collection = config.resolve(&root).unwrap().collections.remove(0);
    let check =
        |rel: &str| kb_core::write::check_not_excluded(&collection, &collection.path.join(rel));

    let err = check("guide/drafts/new.md").unwrap_err();
    assert!(
        err.contains("guide/drafts/new.md") && err.contains("'book'"),
        "{err}"
    );
    assert!(check("guide/SUMMARY.md").is_err());
    assert!(check("guide/new.md").is_ok());
    assert!(kb_core::write::check_not_excluded(&collection, &root.join("elsewhere.md")).is_err());
    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn empty_pattern_fails_config_resolution() {
    let ron =
        r#"(collections: [(name: "book", path: "book", description: "Book", exclude: ["/"])])"#;
    let config: Config = ron::from_str(ron).unwrap();
    let err = format!("{:#}", config.resolve(&std::env::temp_dir()).unwrap_err());
    assert!(err.contains("invalid exclude pattern '/'"), "{err}");
}
