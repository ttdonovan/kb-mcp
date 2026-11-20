//! kb — CLI for markdown knowledge bases.
//!
//! Mirrors every MCP tool as a subcommand so you can test tools, pipe output,
//! and script searches without needing an MCP client. All output is JSON to
//! stdout, errors to stderr.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use kb_core::AppContext;
use kb_core::config::ResolvedCollection;
use kb_core::format;
use kb_core::index::Index;
use kb_core::types::Document;

#[derive(Parser)]
#[command(name = "kb", about = "Knowledge base CLI", version)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Path to collections.ron config file
    #[arg(long, global = true)]
    config: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
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
        /// Maximum documents to include (default: 200)
        #[arg(long, default_value_t = 200)]
        max_documents: usize,
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

fn main() -> ExitCode {
    let cli = Cli::parse();
    let ctx = match kb_core::init(cli.config.as_deref()) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {e:?}");
            return ExitCode::FAILURE;
        }
    };
    match run(cli.command, &ctx) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

/// Run one command. An error is the message to print to stderr before
/// exiting with status 1.
fn run(command: Command, ctx: &AppContext) -> Result<(), String> {
    let index = &ctx.index;
    match command {
        Command::ListSections => println!("{}", format::format_sections(&index.sections)),
        Command::GetDocument { path } => {
            let doc = find(index, &path)?;
            println!("{}", format::format_document(doc, true));
        }
        Command::Search {
            query,
            collection,
            scope,
            max_results,
        } => {
            let results = ctx.search_engine.search(
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
        Command::Context { path } => println!("{}", format::format_context(find(index, &path)?)),
        Command::Write {
            collection,
            title,
            tags,
            body,
            status,
            source,
            directory,
            filename,
        } => write(
            &ctx.collections,
            &WriteArgs {
                coll_name: collection,
                title,
                tags: tags
                    .map(|t| t.split(',').map(|s| s.trim().to_string()).collect())
                    .unwrap_or_default(),
                body,
                status,
                source,
                directory,
                filename,
            },
        )?,
        Command::Reindex => reindex(ctx)?,
        Command::Digest { collection } => println!(
            "{}",
            format::format_digest(&index.documents, &index.sections, collection.as_deref())
        ),
        Command::Query {
            tag,
            status,
            created_after,
            collection,
            has_sources,
        } => query(
            index,
            &QueryArgs {
                tag,
                status,
                created_after,
                collection,
                has_sources,
            },
        )?,
        Command::Export {
            collection,
            max_documents,
        } => export(ctx, collection.as_deref(), max_documents)?,
        Command::Health {
            collection,
            stale_days,
            min_words,
        } => health(ctx, collection.as_deref(), stale_days, min_words)?,
    }
    Ok(())
}

/// The `query` command's filters.
struct QueryArgs {
    tag: Option<String>,
    status: Option<String>,
    created_after: Option<String>,
    collection: Option<String>,
    has_sources: bool,
}

fn query(index: &Index, args: &QueryArgs) -> Result<(), String> {
    if let Some(ref date_str) = args.created_after
        && chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_err()
    {
        return Err(format!(
            "Invalid created_after date '{date_str}', expected YYYY-MM-DD format"
        ));
    }
    let results: Vec<&Document> = index
        .documents
        .iter()
        .filter(|doc| {
            kb_core::query::matches_query(
                doc,
                args.collection.as_deref(),
                args.tag.as_deref(),
                args.status.as_deref(),
                args.created_after.as_deref(),
                args.has_sources,
            )
        })
        .collect();
    println!("{}", format::format_query(&results));
    Ok(())
}

fn health(
    ctx: &AppContext,
    collection: Option<&str>,
    stale_days: u32,
    min_words: u32,
) -> Result<(), String> {
    if let Some(coll) = collection
        && !ctx.collections.iter().any(|c| c.name == coll)
    {
        return Err(format!("Collection '{coll}' not found"));
    }
    println!(
        "{}",
        format::format_health(&ctx.index.documents, collection, stale_days, min_words)
    );
    Ok(())
}

/// A document by path, else by title.
fn find<'a>(index: &'a Index, path: &str) -> Result<&'a Document, String> {
    index
        .find_by_path(path)
        .or_else(|| index.find_by_title(path))
        .ok_or_else(|| format!("Document not found: {path}"))
}

/// The `write` command's arguments, with tags already split.
struct WriteArgs {
    coll_name: String,
    title: String,
    tags: Vec<String>,
    body: String,
    status: Option<String>,
    source: Option<String>,
    directory: Option<String>,
    filename: Option<String>,
}

fn write(collections: &[ResolvedCollection], args: &WriteArgs) -> Result<(), String> {
    let coll_name = &args.coll_name;
    let collection = collections
        .iter()
        .find(|c| c.name == *coll_name)
        .ok_or_else(|| format!("Collection '{coll_name}' not found"))?;

    if !collection.writable {
        return Err(format!("Collection '{coll_name}' is read-only"));
    }

    let target_dir = if let Some(ref dir) = args.directory {
        let resolved = collection.path.join(dir);
        let canonical_base = collection
            .path
            .canonicalize()
            .unwrap_or_else(|_| collection.path.clone());
        std::fs::create_dir_all(&resolved)
            .map_err(|e| format!("Failed to create directory '{dir}': {e}"))?;
        let canonical_resolved = resolved.canonicalize().unwrap_or(resolved);
        if !canonical_resolved.starts_with(&canonical_base) {
            return Err(format!("Directory '{dir}' escapes the collection root"));
        }
        canonical_resolved
    } else {
        collection.path.clone()
    };

    let today = chrono::Local::now().format("%Y-%m-%d").to_string();
    let base_name = args.filename.as_ref().map_or_else(
        || {
            let slug = kb_core::write::slugify_title(&args.title);
            format!("{today}-{slug}.md")
        },
        |name| {
            if Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
            {
                name.clone()
            } else {
                format!("{name}.md")
            }
        },
    );
    let file_path = kb_core::write::find_available_path(&target_dir, &base_name);

    // Writing to a String cannot fail.
    let mut frontmatter = String::from("---\n");
    if !args.tags.is_empty() {
        let _ = writeln!(frontmatter, "tags: [{}]", args.tags.join(", "));
    }
    let _ = writeln!(frontmatter, "created: {today}");
    let _ = writeln!(frontmatter, "updated: {today}");
    if let Some(s) = &args.status {
        let _ = writeln!(frontmatter, "status: {s}");
    }
    if let Some(s) = &args.source {
        let _ = writeln!(frontmatter, "source: {s}");
    }
    frontmatter.push_str("---\n\n");

    let content = format!("{frontmatter}# {}\n\n{}\n", args.title, args.body);
    std::fs::write(&file_path, &content).map_err(|e| format!("Failed to write: {e}"))?;

    let rel = file_path
        .strip_prefix(&collection.path)
        .unwrap_or(&file_path);
    println!(
        "{}",
        format::format_write(&rel.to_string_lossy(), coll_name, &args.title, &args.tags)
    );
    Ok(())
}

fn reindex(ctx: &AppContext) -> Result<(), String> {
    let new_index = Index::build(&ctx.collections);

    // Explicit reindex forces a full store rebuild — clearing each
    // store and sidecar repairs state that incremental sync (which
    // trusts the sidecar hashes) can never fix. Matches the MCP
    // server's reindex tool.
    let mut total_changes: usize = 0;
    for collection in &ctx.collections {
        kb_core::store::clear_collection_store(&ctx.cache_dir, collection)
            .map_err(|e| format!("Failed to clear store for '{}': {}", collection.name, e))?;

        let current_hashes = new_index
            .content_hashes
            .get(&collection.name)
            .cloned()
            .unwrap_or_default();

        let (_, changes) = kb_core::store::sync_collection(
            &ctx.cache_dir,
            collection,
            &current_hashes,
            &new_index.documents,
            #[cfg(feature = "hybrid")]
            &ctx.embedder,
        )
        .map_err(|e| format!("Failed to sync collection '{}': {}", collection.name, e))?;
        total_changes = total_changes.saturating_add(changes);
    }

    println!(
        "Reindexed {} documents across {} sections ({} changes synced)",
        new_index.documents.len(),
        new_index.sections.len(),
        total_changes
    );
    Ok(())
}

fn export(ctx: &AppContext, collection: Option<&str>, max_documents: usize) -> Result<(), String> {
    let in_collection = |doc: &&Document| collection.is_none_or(|f| doc.collection == f);
    let matching_count = ctx.index.documents.iter().filter(in_collection).count();

    let docs_with_bodies: Vec<(&Document, String)> = ctx
        .index
        .documents
        .iter()
        .filter(in_collection)
        .take(max_documents)
        .filter_map(|doc| {
            let coll = ctx.collections.iter().find(|c| c.name == doc.collection)?;
            let body = format::read_document_body(&coll.path.join(&doc.path))?;
            Some((doc, body))
        })
        .collect();

    if docs_with_bodies.is_empty() {
        return Err(format!(
            "No documents found{}",
            collection
                .map(|c| format!(" in collection '{c}'"))
                .unwrap_or_default()
        ));
    }

    let mut output = format::format_export(&docs_with_bodies, collection);
    if matching_count > max_documents {
        // Writing to a String cannot fail.
        let _ = writeln!(
            output,
            "... truncated: showing {} of {matching_count} documents",
            docs_with_bodies.len()
        );
    }
    print!("{output}");
    Ok(())
}
