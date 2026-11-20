//! JSON output formatting.
//!
//! Separate output structs decouple serialization shape from internal types.
//! Both MCP tools and CLI commands use these same formatters, ensuring
//! identical JSON output regardless of transport.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write as _;

use serde::Serialize;

use crate::search::SearchResult;
use crate::types::{Document, Section};

// --- Digest output types ---

#[derive(Serialize)]
pub struct DigestOutput {
    pub total_documents: usize,
    pub total_sections: usize,
    pub collections: Vec<DigestCollectionOutput>,
}

#[derive(Serialize)]
pub struct DigestCollectionOutput {
    pub name: String,
    pub doc_count: usize,
    pub sections: Vec<DigestSectionOutput>,
    pub recent: Vec<DigestRecentOutput>,
}

#[derive(Serialize)]
pub struct DigestSectionOutput {
    pub name: String,
    pub doc_count: usize,
    pub topics: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Serialize)]
pub struct DigestRecentOutput {
    pub path: String,
    pub title: String,
    pub created: String,
}

#[derive(Serialize)]
pub struct SectionOutput {
    pub name: String,
    pub description: String,
    pub doc_count: usize,
    pub collection: String,
}

#[derive(Serialize)]
pub struct DocumentOutput {
    pub path: String,
    pub title: String,
    pub tags: Vec<String>,
    pub section: String,
    pub collection: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Serialize)]
pub struct ContextOutput {
    pub path: String,
    pub title: String,
    pub tags: Vec<String>,
    pub section: String,
    pub collection: String,
    pub frontmatter: HashMap<String, serde_yaml::Value>,
    pub summary: String,
}

#[derive(Serialize)]
pub struct SearchResultOutput {
    pub path: String,
    pub title: String,
    pub section: String,
    pub collection: String,
    pub score: f64,
    pub excerpt: String,
}

#[derive(Serialize)]
pub struct SearchOutput {
    pub query: String,
    pub total: usize,
    pub results: Vec<SearchResultOutput>,
}

#[derive(Serialize)]
pub struct WriteOutput {
    pub path: String,
    pub collection: String,
    pub title: String,
    pub tags: Vec<String>,
}

#[must_use]
pub fn format_sections(sections: &[Section]) -> String {
    let output: Vec<SectionOutput> = sections
        .iter()
        .map(|s| SectionOutput {
            name: s.name.clone(),
            description: s.description.clone(),
            doc_count: s.doc_count,
            collection: s.collection.clone(),
        })
        .collect();

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

#[must_use]
pub fn format_document(doc: &Document, include_content: bool) -> String {
    let output = DocumentOutput {
        path: doc.path.clone(),
        title: doc.title.clone(),
        tags: doc.tags.clone(),
        section: doc.section.clone(),
        collection: doc.collection.clone(),
        content: if include_content {
            Some(doc.body.clone())
        } else {
            None
        },
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

#[must_use]
pub fn format_context(doc: &Document) -> String {
    let summary = extract_summary(&doc.body);
    let output = ContextOutput {
        path: doc.path.clone(),
        title: doc.title.clone(),
        tags: doc.tags.clone(),
        section: doc.section.clone(),
        collection: doc.collection.clone(),
        frontmatter: doc.frontmatter.clone(),
        summary,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

#[must_use]
pub fn format_search(query: &str, results: &[SearchResult], documents: &[Document]) -> String {
    let result_outputs: Vec<SearchResultOutput> = results
        .iter()
        // Results index into the same `documents`; a stale index is skipped, not a panic.
        .filter_map(|r| {
            let doc = documents.get(r.doc_index)?;
            Some(SearchResultOutput {
                path: doc.path.clone(),
                title: doc.title.clone(),
                section: doc.section.clone(),
                collection: doc.collection.clone(),
                score: (r.score * 1000.0).round() / 1000.0,
                excerpt: r.excerpt.clone(),
            })
        })
        .collect();

    let output = SearchOutput {
        query: query.to_string(),
        total: result_outputs.len(),
        results: result_outputs,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

#[must_use]
pub fn format_write(path: &str, collection: &str, title: &str, tags: &[String]) -> String {
    let output = WriteOutput {
        path: path.to_string(),
        collection: collection.to_string(),
        title: title.to_string(),
        tags: tags.to_vec(),
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

/// Extract the first paragraph after the H1 heading as a summary.
///
/// Falls back to the first non-empty paragraph if no H1 is found.
/// This powers `kb_context` — agents get a meaningful preview without
/// reading the full document, saving 90%+ tokens on scan-heavy workflows.
#[must_use]
pub fn extract_summary(body: &str) -> String {
    let mut lines = body.lines().peekable();
    let mut found_heading = false;

    // Skip to after the first H1
    while let Some(line) = lines.peek() {
        if line.starts_with("# ") {
            found_heading = true;
            lines.next();
            // Skip blank lines after heading
            while lines.peek().is_some_and(|l| l.trim().is_empty()) {
                lines.next();
            }
            break;
        }
        lines.next();
    }

    if !found_heading {
        // No heading — take first non-empty paragraph
        let mut lines = body.lines().peekable();
        while lines.peek().is_some_and(|l| l.trim().is_empty()) {
            lines.next();
        }
        return collect_paragraph(&mut lines);
    }

    collect_paragraph(&mut lines)
}

fn collect_paragraph(lines: &mut std::iter::Peekable<std::str::Lines>) -> String {
    let mut paragraph = Vec::new();
    for line in lines {
        if line.trim().is_empty() {
            break;
        }
        paragraph.push(line);
    }
    paragraph.join("\n")
}

/// Convert a `serde_yaml::Value` to a display string. `serde_yaml::Value` doesn't
/// implement Display, so we serialize via `serde_json` for non-string variants.
#[must_use]
pub fn yaml_value_to_string(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Number(n) => n.as_i64().map_or_else(
            || n.as_f64().map_or_else(String::new, |f| f.to_string()),
            |i| i.to_string(),
        ),
        serde_yaml::Value::Null => "null".to_string(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

/// Build a vault digest — structured overview of collections, sections, topics,
/// and recent additions. The 7-day recency window uses frontmatter `created` dates.
#[must_use]
pub fn format_digest(
    documents: &[Document],
    sections: &[Section],
    collection_filter: Option<&str>,
) -> String {
    let today = chrono::Local::now().date_naive();
    let week_ago = today
        .checked_sub_days(chrono::Days::new(7))
        .unwrap_or(chrono::NaiveDate::MIN);

    // Group documents by collection
    let mut coll_docs: HashMap<&str, Vec<&Document>> = HashMap::new();
    for doc in documents {
        if collection_filter.is_some_and(|f| doc.collection != f) {
            continue;
        }
        coll_docs.entry(&doc.collection).or_default().push(doc);
    }

    // Group sections by collection
    let mut coll_sections: HashMap<&str, Vec<&Section>> = HashMap::new();
    for sec in sections {
        if collection_filter.is_some_and(|f| sec.collection != f) {
            continue;
        }
        coll_sections.entry(&sec.collection).or_default().push(sec);
    }

    let mut collection_docs: Vec<(&str, Vec<&Document>)> = coll_docs.into_iter().collect();
    collection_docs.sort_unstable_by_key(|(name, _)| *name);

    let mut total_documents: usize = 0;
    let mut total_sections: usize = 0;
    let mut collections = Vec::new();

    for (coll_name, docs) in &collection_docs {
        let coll_name = *coll_name;
        total_documents = total_documents.saturating_add(docs.len());

        // Build section digests with topics (from doc titles)
        let empty_secs = vec![];
        let secs = coll_sections.get(coll_name).unwrap_or(&empty_secs);
        total_sections = total_sections.saturating_add(secs.len());

        let mut section_outputs = Vec::new();
        for sec in secs {
            let sec_docs: Vec<&&Document> = docs.iter().filter(|d| d.section == sec.name).collect();
            let topics: Vec<String> = sec_docs.iter().map(|d| d.title.clone()).collect();
            let filtered_count = sec_docs.len();
            let hint = if filtered_count < 2 {
                Some("thin — fewer than 2 documents".to_string())
            } else {
                None
            };
            section_outputs.push(DigestSectionOutput {
                name: sec.name.clone(),
                doc_count: filtered_count,
                topics,
                hint,
            });
        }

        // Recent: docs with `created` date in last 7 days
        let mut recent = Vec::new();
        for doc in docs {
            if let Some(created_val) = doc.frontmatter.get("created") {
                let created_str = yaml_value_to_string(created_val);
                if let Ok(created_date) =
                    chrono::NaiveDate::parse_from_str(&created_str, "%Y-%m-%d")
                    && created_date >= week_ago
                {
                    recent.push(DigestRecentOutput {
                        path: doc.path.clone(),
                        title: doc.title.clone(),
                        created: created_str,
                    });
                }
            }
        }
        recent.sort_by(|a, b| b.created.cmp(&a.created));

        collections.push(DigestCollectionOutput {
            name: coll_name.to_string(),
            doc_count: docs.len(),
            sections: section_outputs,
            recent,
        });
    }

    let output = DigestOutput {
        total_documents,
        total_sections,
        collections,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

// --- Query output types ---

#[derive(Serialize)]
pub struct QueryOutput {
    pub total: usize,
    pub documents: Vec<QueryDocumentOutput>,
}

#[derive(Serialize)]
pub struct QueryDocumentOutput {
    pub path: String,
    pub title: String,
    pub tags: Vec<String>,
    pub section: String,
    pub collection: String,
}

/// Format query results — document metadata without body content.
#[must_use]
pub fn format_query(docs: &[&Document]) -> String {
    let documents: Vec<QueryDocumentOutput> = docs
        .iter()
        .map(|d| QueryDocumentOutput {
            path: d.path.clone(),
            title: d.title.clone(),
            tags: d.tags.clone(),
            section: d.section.clone(),
            collection: d.collection.clone(),
        })
        .collect();

    let output = QueryOutput {
        total: documents.len(),
        documents,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}

/// Read a document's body from disk, stripping YAML frontmatter.
///
/// Shared implementation for both MCP and CLI fresh-from-disk reads.
/// Callers resolve the full filesystem path; this handles frontmatter stripping.
#[must_use]
pub fn read_document_body(file_path: &std::path::Path) -> Option<String> {
    let content = std::fs::read_to_string(file_path).ok()?;
    if let Some(after) = content.strip_prefix("---")
        && let Some((_, body)) = after.split_once("\n---")
    {
        return Some(body.trim_start_matches('\n').to_string());
    }
    Some(content)
}

/// Build an export document — concatenated markdown with frontmatter headers.
///
/// Takes a list of (document, body) pairs where bodies are pre-read from disk.
/// Both the MCP tool and CLI command delegate to this function to avoid
/// duplicating the markdown assembly logic.
#[must_use]
pub fn format_export(
    docs_with_bodies: &[(&Document, String)],
    collection_filter: Option<&str>,
) -> String {
    let today = chrono::Local::now().format("%Y-%m-%d");

    let mut output = String::new();

    let header = collection_filter.map_or_else(
        || format!("# Vault Export ({today})\n\n"),
        |c| format!("# Vault Export: {c} ({today})\n\n"),
    );
    output.push_str(&header);

    for (doc, body) in docs_with_bodies {
        output.push_str("---\n");
        // Writing to a String cannot fail.
        let _ = writeln!(output, "## {}", doc.path);
        for (key, value) in &doc.frontmatter {
            let _ = writeln!(output, "{key}: {}", yaml_value_to_string(value));
        }
        output.push_str("---\n\n");
        output.push_str(body);
        output.push_str("\n\n");
    }

    output
}

// --- Health output types ---

#[derive(Serialize)]
pub struct HealthOutput {
    pub total_documents_checked: usize,
    pub total_issues: usize,
    pub collections: Vec<HealthCollectionOutput>,
}

#[derive(Serialize)]
pub struct HealthCollectionOutput {
    pub name: String,
    pub doc_count: usize,
    pub issues: usize,
    pub malformed_frontmatter: Vec<HealthMalformedRef>,
    pub missing_created: Vec<HealthDocRef>,
    pub missing_updated: Vec<HealthDocRef>,
    pub no_tags: Vec<HealthDocRef>,
    pub stale: Vec<HealthStaleRef>,
    pub stubs: Vec<HealthStubRef>,
    pub orphans: Vec<HealthDocRef>,
    pub broken_links: Vec<HealthBrokenLinkRef>,
}

#[derive(Serialize)]
pub struct HealthDocRef {
    pub path: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct HealthMalformedRef {
    pub path: String,
    pub title: String,
    /// The YAML parser's message, so the user can find the bad line.
    pub error: String,
}

#[derive(Serialize)]
pub struct HealthStaleRef {
    pub path: String,
    pub title: String,
    pub last_date: String,
}

#[derive(Serialize)]
pub struct HealthStubRef {
    pub path: String,
    pub title: String,
    pub word_count: usize,
}

#[derive(Serialize)]
pub struct HealthBrokenLinkRef {
    pub source: String,
    pub target: String,
    pub raw: String,
}

/// Matches `[[target]]`, `[[target|alias]]` and `[[target#heading]]`.
#[expect(
    clippy::unwrap_used,
    reason = "a constant pattern; the health tests compile it"
)]
static WIKI_LINK_RE: std::sync::LazyLock<regex::Regex> =
    std::sync::LazyLock::new(|| regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap());

/// Wiki-links across the checked documents, indexed by position in that list.
struct LinkGraph {
    /// Inbound links per document, self-links excluded.
    inbound: Vec<usize>,
    /// All wiki-links seen; orphans are only reported when there are any.
    total: usize,
    /// (target, raw link) of each unresolved link, keyed by source document.
    broken_by_source: HashMap<usize, Vec<(String, String)>>,
}

/// Resolve every wiki-link against document titles and paths (with and
/// without `.md`), case-insensitively.
fn link_graph(docs: &[&Document]) -> LinkGraph {
    let mut doc_id_to_idx: HashMap<String, usize> = HashMap::new();
    for (i, doc) in docs.iter().enumerate() {
        doc_id_to_idx.insert(doc.title.to_lowercase(), i);
        doc_id_to_idx.insert(doc.path.to_lowercase(), i);
        let path_no_ext = doc.path.trim_end_matches(".md").to_lowercase();
        doc_id_to_idx.insert(path_no_ext, i);
    }

    let mut graph = LinkGraph {
        inbound: vec![0; docs.len()],
        total: 0,
        broken_by_source: HashMap::new(),
    };

    for (src_idx, doc) in docs.iter().enumerate() {
        for cap in WIKI_LINK_RE.captures_iter(&doc.body) {
            graph.total = graph.total.saturating_add(1);
            let raw = cap[0].to_string();
            let inner = &cap[1];
            // Strip alias after | and heading anchor after #
            let target = inner
                .split('|')
                .next()
                .unwrap_or(inner)
                .split('#')
                .next()
                .unwrap_or(inner)
                .trim();

            if let Some(&tgt_idx) = doc_id_to_idx.get(&target.to_lowercase()) {
                if tgt_idx != src_idx
                    && let Some(count) = graph.inbound.get_mut(tgt_idx)
                {
                    *count = count.saturating_add(1);
                }
            } else {
                graph
                    .broken_by_source
                    .entry(src_idx)
                    .or_default()
                    .push((target.to_string(), raw));
            }
        }
    }

    graph
}

/// Run every check on one collection's documents, given as (index in the
/// link graph, document).
fn check_collection(
    name: &str,
    entries: &[(usize, &Document)],
    graph: &LinkGraph,
    stale_cutoff: chrono::NaiveDate,
    min_words: u32,
) -> HealthCollectionOutput {
    let min_words = usize::try_from(min_words).unwrap_or(usize::MAX);

    let mut malformed_frontmatter = Vec::new();
    let mut missing_created = Vec::new();
    let mut missing_updated = Vec::new();
    let mut no_tags = Vec::new();
    let mut stale = Vec::new();
    let mut stubs = Vec::new();
    let mut orphans = Vec::new();
    let mut broken_links = Vec::new();

    for &(idx, doc) in entries {
        let doc_ref = || HealthDocRef {
            path: doc.path.clone(),
            title: doc.title.clone(),
        };

        // A block that failed to parse has no fields: report it once,
        // rather than as missing created, updated and tags.
        if let Some(error) = &doc.frontmatter_error {
            malformed_frontmatter.push(HealthMalformedRef {
                path: doc.path.clone(),
                title: doc.title.clone(),
                error: error.clone(),
            });
        } else {
            if !doc.frontmatter.contains_key("created") {
                missing_created.push(doc_ref());
            }
            if !doc.frontmatter.contains_key("updated") {
                missing_updated.push(doc_ref());
            }
            if doc.tags.is_empty() {
                no_tags.push(doc_ref());
            }
        }

        // Staleness: use updated, fall back to created
        let last_date_str = doc
            .frontmatter
            .get("updated")
            .or_else(|| doc.frontmatter.get("created"))
            .map(yaml_value_to_string);

        if let Some(ref date_str) = last_date_str
            && let Ok(date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            && date < stale_cutoff
        {
            stale.push(HealthStaleRef {
                path: doc.path.clone(),
                title: doc.title.clone(),
                last_date: date_str.clone(),
            });
        }

        let word_count = doc.body.split_whitespace().count();
        if word_count < min_words {
            stubs.push(HealthStubRef {
                path: doc.path.clone(),
                title: doc.title.clone(),
                word_count,
            });
        }

        // Orphans (only if vault has wiki-links at all)
        if graph.total > 0 && graph.inbound.get(idx) == Some(&0) {
            orphans.push(doc_ref());
        }

        if let Some(links) = graph.broken_by_source.get(&idx) {
            for (target, raw) in links {
                broken_links.push(HealthBrokenLinkRef {
                    source: doc.path.clone(),
                    target: target.clone(),
                    raw: raw.clone(),
                });
            }
        }
    }

    let issues = [
        malformed_frontmatter.len(),
        missing_created.len(),
        missing_updated.len(),
        no_tags.len(),
        stale.len(),
        stubs.len(),
        orphans.len(),
        broken_links.len(),
    ]
    .into_iter()
    .fold(0, usize::saturating_add);

    HealthCollectionOutput {
        name: name.to_string(),
        doc_count: entries.len(),
        issues,
        malformed_frontmatter,
        missing_created,
        missing_updated,
        no_tags,
        stale,
        stubs,
        orphans,
        broken_links,
    }
}

/// Build a vault health report — document quality and hygiene checks.
///
/// Checks frontmatter that fails to parse, frontmatter completeness
/// (created/updated/tags), content quality
/// (staleness, stub detection), and link integrity (orphans, broken wiki-links).
#[must_use]
pub fn format_health(
    documents: &[Document],
    collection_filter: Option<&str>,
    stale_days: u32,
    min_words: u32,
) -> String {
    let today = chrono::Local::now().date_naive();
    let stale_cutoff = today
        .checked_sub_days(chrono::Days::new(u64::from(stale_days)))
        .unwrap_or(chrono::NaiveDate::MIN);

    let docs: Vec<&Document> = documents
        .iter()
        .filter(|d| collection_filter.is_none_or(|f| d.collection == f))
        .collect();
    let graph = link_graph(&docs);

    // Group by collection, keeping each document's index in the link graph.
    let mut coll_docs: BTreeMap<&str, Vec<(usize, &Document)>> = BTreeMap::new();
    for (i, doc) in docs.iter().enumerate() {
        coll_docs.entry(&doc.collection).or_default().push((i, doc));
    }

    let collections: Vec<HealthCollectionOutput> = coll_docs
        .iter()
        .map(|(name, entries)| check_collection(name, entries, &graph, stale_cutoff, min_words))
        .collect();

    let output = HealthOutput {
        total_documents_checked: docs.len(),
        total_issues: collections
            .iter()
            .map(|c| c.issues)
            .fold(0, usize::saturating_add),
        collections,
    };

    serde_json::to_string_pretty(&output).unwrap_or_default()
}
