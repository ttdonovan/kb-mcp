//! CLI interface — mirrors every MCP tool as a subcommand.
//!
//! Exists so you can test tools, pipe output, and script searches without
//! needing an MCP client. All output is JSON to stdout, errors to stderr.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::config::ResolvedCollection;
use crate::format;
use crate::index::Index;
use crate::search::SearchEngine;

#[derive(Parser)]
#[command(name = "kb-mcp", about = "Knowledge base MCP server and CLI")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Path to collections.ron config file
    #[arg(long, global = true)]
    pub config: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Command {
    /// List all sections with doc counts
    ListSections,
    /// Get a document by path or title
    GetDocument {
        /// Document path or title
        #[arg(long)]
        path: String,
    },
    /// Search across collections
    Search {
        /// Search query
        #[arg(long)]
        query: String,
        /// Filter by collection name
        #[arg(long)]
        collection: Option<String>,
        /// Filter by section scope
        #[arg(long)]
        scope: Option<String>,
        /// Maximum results (default: 10)
        #[arg(long, default_value_t = 10)]
        max_results: usize,
    },
    /// Token-efficient document briefing (frontmatter + summary)
    Context {
        /// Document path or title
        #[arg(long)]
        path: String,
    },
    /// Create a document in a writable collection
    Write {
        /// Target collection name
        #[arg(long)]
        collection: String,
        /// Document title
        #[arg(long)]
        title: String,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        /// Document body content
        #[arg(long)]
        body: String,
        /// Optional status field
        #[arg(long)]
        status: Option<String>,
        /// Optional source field
        #[arg(long)]
        source: Option<String>,
        /// Subdirectory within the collection (e.g. "concepts/memory")
        #[arg(long)]
        directory: Option<String>,
        /// Explicit filename (e.g. "cognitive-memory-model.md"). Skips date prefix.
        #[arg(long)]
        filename: Option<String>,
    },
    /// Rebuild the search index from disk
    Reindex,
    /// Vault summary — coverage, gaps, recent additions
    Digest {
        /// Filter to a specific collection
        #[arg(long)]
        collection: Option<String>,
    },
    /// Filter documents by frontmatter fields
    Query {
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
        /// Filter by frontmatter status
        #[arg(long)]
        status: Option<String>,
        /// Filter by created date (YYYY-MM-DD, docs on or after)
        #[arg(long)]
        created_after: Option<String>,
        /// Filter by collection name
        #[arg(long)]
        collection: Option<String>,
        /// Only docs with a sources field
        #[arg(long)]
        has_sources: bool,
    },
    /// Export vault as a single markdown document
    Export {
        /// Collection to export (default: all)
        #[arg(long)]
        collection: Option<String>,
    },
    /// Vault health diagnostics — document quality checks
    Health {
        /// Filter to a specific collection
        #[arg(long)]
        collection: Option<String>,
        /// Days threshold for staleness (default: 90)
        #[arg(long, default_value_t = 90)]
        stale_days: u32,
        /// Minimum word count for stub detection (default: 50)
        #[arg(long, default_value_t = 50)]
        min_words: u32,
    },
}

pub fn run(
    cli: Cli,
    index: &Index,
    search_engine: &SearchEngine,
    collections: &[ResolvedCollection],
) {
    match cli.command {
        Command::ListSections => {
            println!("{}", format::format_sections(&index.sections));
        }
        Command::GetDocument { path } => {
            let doc = index
                .find_by_path(&path)
                .or_else(|| index.find_by_title(&path));

            match doc {
                Some(doc) => println!("{}", format::format_document(doc, true)),
                None => {
                    eprintln!("Document not found: {}", path);
                    std::process::exit(1);
                }
            }
        }
        Command::Search {
            query,
            collection,
            scope,
            max_results,
        } => {
            let results = search_engine.search(
                &index.documents,
                &query,
                collection.as_deref(),
                scope.as_deref(),
                max_results,
            );
            println!(
                "{}",
                format::format_search(&query, &results, &index.documents)
            );
        }
        Command::Context { path } => {
            let doc = index
                .find_by_path(&path)
                .or_else(|| index.find_by_title(&path));

            match doc {
                Some(doc) => println!("{}", format::format_context(doc)),
                None => {
                    eprintln!("Document not found: {}", path);
                    std::process::exit(1);
                }
            }
        }
        Command::Write {
            collection: coll_name,
            title,
            tags,
            body,
            status,
            source,
            directory,
            filename,
        } => {
            let collection = collections.iter().find(|c| c.name == coll_name);
            let collection = match collection {
                Some(c) => c,
                None => {
                    eprintln!("Collection '{}' not found", coll_name);
                    std::process::exit(1);
                }
            };

            if !collection.writable {
                eprintln!("Collection '{}' is read-only", coll_name);
                std::process::exit(1);
            }

            let tags: Vec<String> = tags
                .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                .unwrap_or_default();

            // Resolve target directory
            let target_dir = if let Some(ref dir) = directory {
                let resolved = collection.path.join(dir);
                let canonical_base = collection
                    .path
                    .canonicalize()
                    .unwrap_or(collection.path.clone());
                if let Err(e) = std::fs::create_dir_all(&resolved) {
                    eprintln!("Failed to create directory '{}': {}", dir, e);
                    std::process::exit(1);
                }
                let canonical_resolved = resolved.canonicalize().unwrap_or(resolved);
                if !canonical_resolved.starts_with(&canonical_base) {
                    eprintln!(
                        "Directory '{}' escapes the collection root",
                        dir
                    );
                    std::process::exit(1);
                }
                canonical_resolved
            } else {
                collection.path.clone()
            };

            // Generate filename: explicit or date-prefixed slug
            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let base_name = if let Some(ref name) = filename {
                if name.ends_with(".md") {
                    name.clone()
                } else {
                    format!("{}.md", name)
                }
            } else {
                let slug = crate::tools::write::slugify_title(&title);
                format!("{}-{}.md", today, slug)
            };
            let file_path = find_available_path(&target_dir, &base_name);

            let mut frontmatter = String::new();
            frontmatter.push_str("---\n");
            if !tags.is_empty() {
                frontmatter.push_str(&format!("tags: [{}]\n", tags.join(", ")));
            }
            frontmatter.push_str(&format!("created: {}\n", today));
            frontmatter.push_str(&format!("updated: {}\n", today));
            if let Some(s) = &status {
                frontmatter.push_str(&format!("status: {}\n", s));
            }
            if let Some(s) = &source {
                frontmatter.push_str(&format!("source: {}\n", s));
            }
            frontmatter.push_str("---\n\n");

            let content = format!("{}# {}\n\n{}\n", frontmatter, title, body);

            match std::fs::write(&file_path, &content) {
                Ok(_) => {
                    let rel = file_path
                        .strip_prefix(&collection.path)
                        .unwrap_or(&file_path);
                    println!(
                        "{}",
                        format::format_write(
                            &rel.to_string_lossy(),
                            &coll_name,
                            &title,
                            &tags
                        )
                    );
                }
                Err(e) => {
                    eprintln!("Failed to write: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Command::Reindex => {
            // CLI reindex: rebuild index + re-sync all .mv2 files.
            // Note: if MCP server holds exclusive locks on .mv2 files,
            // this will fail. Use the MCP reindex tool instead.
            let new_index = Index::build(collections);
            println!(
                "Reindexed {} documents across {} sections",
                new_index.documents.len(),
                new_index.sections.len()
            );
        }
        Command::Digest { collection } => {
            println!(
                "{}",
                format::format_digest(
                    &index.documents,
                    &index.sections,
                    collection.as_deref()
                )
            );
        }
        Command::Query {
            tag,
            status,
            created_after,
            collection,
            has_sources,
        } => {
            if let Some(ref date_str) = created_after
                && chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err()
            {
                eprintln!(
                    "Invalid created_after date '{}', expected YYYY-MM-DD format",
                    date_str
                );
                std::process::exit(1);
            }
            let params = crate::tools::query::QueryParams {
                tag,
                status,
                created_after,
                collection,
                has_sources,
            };
            let results: Vec<&crate::types::Document> = index
                .documents
                .iter()
                .filter(|doc| crate::tools::query::matches_query(doc, &params))
                .collect();
            println!("{}", format::format_query(&results));
        }
        Command::Export { collection } => {
            let docs_with_bodies: Vec<(&crate::types::Document, String)> = index
                .documents
                .iter()
                .filter(|doc| {
                    collection
                        .as_ref()
                        .is_none_or(|f| doc.collection == *f)
                })
                .filter_map(|doc| {
                    let coll = collections.iter().find(|c| c.name == doc.collection)?;
                    let file_path = coll.path.join(&doc.path);
                    let body = format::read_document_body(&file_path)?;
                    Some((doc, body))
                })
                .collect();

            if docs_with_bodies.is_empty() {
                eprintln!(
                    "No documents found{}",
                    collection
                        .as_ref()
                        .map(|c| format!(" in collection '{}'", c))
                        .unwrap_or_default()
                );
                std::process::exit(1);
            }

            print!(
                "{}",
                format::format_export(&docs_with_bodies, collection.as_deref())
            );
        }
        Command::Health {
            collection,
            stale_days,
            min_words,
        } => {
            if let Some(ref coll) = collection
                && !collections.iter().any(|c| c.name == *coll)
            {
                eprintln!("Collection '{}' not found", coll);
                std::process::exit(1);
            }
            println!(
                "{}",
                format::format_health(
                    &index.documents,
                    collection.as_deref(),
                    stale_days,
                    min_words,
                )
            );
        }
    }
}

fn find_available_path(dir: &std::path::Path, base_name: &str) -> std::path::PathBuf {
    let candidate = dir.join(base_name);
    if !candidate.exists() {
        return candidate;
    }
    let stem = base_name.trim_end_matches(".md");
    for i in 2..100 {
        let c = dir.join(format!("{}-{}.md", stem, i));
        if !c.exists() {
            return c;
        }
    }
    dir.join(base_name)
}
