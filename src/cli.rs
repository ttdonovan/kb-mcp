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
    },
    /// Rebuild the search index from disk
    Reindex,
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

            let today = chrono::Local::now().format("%Y-%m-%d").to_string();
            let slug = crate::tools::write::slugify_title(&title);
            let base_name = format!("{}-{}.md", today, slug);
            let file_path = find_available_path(&collection.path, &base_name);

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
            let new_index = Index::build(collections);
            search_engine.rebuild(&new_index.documents);
            println!(
                "Reindexed {} documents across {} sections",
                new_index.documents.len(),
                new_index.sections.len()
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
