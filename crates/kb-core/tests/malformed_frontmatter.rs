//! A frontmatter block that is not valid YAML used to vanish without a trace:
//! `kb_health` then reported the document as missing `created`, `updated` and
//! tags, pointing at the wrong fix (issue #27). It is now reported on its own.

use kb_core::config::{IndexNames, ResolvedCollection};
use kb_core::format::format_health;
use kb_core::index::Index;

#[test]
fn health_reports_malformed_frontmatter_instead_of_missing_fields() {
    let root = std::env::temp_dir().join("kb-mcp-malformed-frontmatter");
    let _ = std::fs::remove_dir_all(&root);
    let notes = root.join("notes");
    std::fs::create_dir_all(&notes).unwrap();
    std::fs::write(
        notes.join("broken.md"),
        "---\ntitle: Setup: the first steps\ncreated: 2026-01-01\n---\n\n# Broken\n\nBody.\n",
    )
    .unwrap();
    std::fs::write(
        notes.join("fine.md"),
        "---\ntags: [ok]\ncreated: 2026-01-01\nupdated: 2026-01-01\n---\n\n# Fine\n\nBody.\n",
    )
    .unwrap();

    let collection = ResolvedCollection {
        name: "notes".to_string(),
        path: notes,
        description: "Notes".to_string(),
        writable: false,
        sections: vec![],
        index_names: IndexNames::Path,
        exclude: kb_core::config::Exclude::default(),
    };
    let index = Index::build(&[collection]);

    let broken = index.find_by_path("broken.md").expect("broken.md indexed");
    assert!(broken.frontmatter_error.is_some());
    assert!(broken.frontmatter.is_empty());
    assert!(
        index
            .find_by_path("fine.md")
            .unwrap()
            .frontmatter_error
            .is_none()
    );

    let report: serde_json::Value =
        serde_json::from_str(&format_health(&index.documents, None, 36500, 0)).unwrap();
    let notes = &report["collections"][0];

    let malformed = notes["malformed_frontmatter"].as_array().unwrap();
    assert_eq!(malformed.len(), 1);
    assert_eq!(malformed[0]["path"], "broken.md");
    assert!(!malformed[0]["error"].as_str().unwrap().is_empty());

    for check in ["missing_created", "missing_updated", "no_tags"] {
        assert!(
            notes[check].as_array().unwrap().is_empty(),
            "{check} should not list the malformed document"
        );
    }
    assert_eq!(notes["issues"], 1);

    let _ = std::fs::remove_dir_all(&root);
}
