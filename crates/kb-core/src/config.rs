//! Configuration loading and resolution.
//!
//! Separates raw RON config (relative paths, optional fields) from resolved
//! config (absolute paths, defaults applied). This two-phase approach lets
//! collection paths in the RON file be relative to the config file's location,
//! enabling the same binary to serve different projects via different configs.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

/// Raw configuration as deserialized from `collections.ron`.
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub cache_dir: Option<String>,
    #[serde(default)]
    pub index_names: IndexNames,
    pub collections: Vec<CollectionDef>,
}

/// How each collection's index files in `cache_dir` are named.
///
/// `Path` (default) is safe in a cache dir shared by many configs but leaves
/// an orphaned index whenever the collections move. `Name` keeps one index per
/// collection across moves (e.g. plugin updates), so it needs a cache dir
/// dedicated to this config: another config with the same collection name
/// would share, and keep rebuilding, the same file.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum IndexNames {
    /// `<hash of absolute collection path>-<name>.mv2`
    #[default]
    Path,
    /// `<name>.mv2`
    Name,
}

/// A collection definition from the RON config, with paths still relative.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollectionDef {
    pub name: String,
    pub path: String,
    pub description: String,
    #[serde(default)]
    pub writable: bool,
    #[serde(default)]
    pub sections: Vec<SectionDef>,
    /// Glob patterns, relative to the collection root, for files and
    /// directories not to index (e.g. an mdBook's `SUMMARY.md`).
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Maps a directory prefix to a human-readable description for `list_sections` output.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SectionDef {
    pub prefix: String,
    pub description: String,
}

/// Resolved config with absolute paths.
#[derive(Debug)]
pub struct ResolvedConfig {
    pub cache_dir: PathBuf,
    pub collections: Vec<ResolvedCollection>,
}

#[derive(Debug, Clone)]
pub struct ResolvedCollection {
    pub name: String,
    pub path: PathBuf,
    pub description: String,
    pub writable: bool,
    pub sections: Vec<SectionDef>,
    pub index_names: IndexNames,
    pub exclude: Exclude,
}

/// A collection's compiled `exclude` patterns. Compiled once at config load,
/// so a bad pattern fails startup instead of every scan.
#[derive(Debug, Clone)]
pub struct Exclude(GlobSet);

impl Default for Exclude {
    fn default() -> Self {
        Self(GlobSet::empty())
    }
}

impl Exclude {
    /// `*` and `?` stop at `/`; `**` crosses directories.
    ///
    /// # Errors
    ///
    /// A pattern is not a valid glob.
    pub fn new(patterns: &[String]) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            // Matched paths never end in `/`, so `drafts/` would match nothing.
            let trimmed = pattern.trim_end_matches('/');
            if trimmed.is_empty() {
                anyhow::bail!("invalid exclude pattern '{pattern}': empty");
            }
            let glob = GlobBuilder::new(trimmed)
                .literal_separator(true)
                .build()
                .with_context(|| format!("invalid exclude pattern '{pattern}'"))?;
            builder.add(glob);
        }
        Ok(Self(builder.build()?))
    }

    /// Whether `rel_path` (relative to the collection root) is excluded.
    #[must_use]
    pub fn is_match(&self, rel_path: &Path) -> bool {
        self.0.is_match(rel_path)
    }

    /// Whether a scan skips `rel_path`: it or a directory above it matches.
    #[must_use]
    pub fn excludes(&self, rel_path: &Path) -> bool {
        rel_path
            .ancestors()
            .any(|p| !p.as_os_str().is_empty() && self.is_match(p))
    }
}

impl Config {
    /// Resolve relative paths against the config file's parent directory.
    ///
    /// # Errors
    ///
    /// A collection's `exclude` pattern is not a valid glob.
    pub fn resolve(self, config_dir: &Path) -> Result<ResolvedConfig> {
        let cache_dir = self
            .cache_dir
            .map_or_else(default_cache_dir, |s| config_dir.join(shellexpand(&s)));

        let index_names = self.index_names;
        let collections = self
            .collections
            .into_iter()
            .map(|c| {
                let path = config_dir.join(&c.path);
                let path = path.canonicalize().unwrap_or(path);
                let exclude =
                    Exclude::new(&c.exclude).with_context(|| format!("collection '{}'", c.name))?;
                Ok(ResolvedCollection {
                    name: c.name,
                    path,
                    description: c.description,
                    writable: c.writable,
                    sections: c.sections,
                    index_names,
                    exclude,
                })
            })
            .collect::<Result<_>>()?;

        Ok(ResolvedConfig {
            cache_dir,
            collections,
        })
    }
}

/// Load config using resolution chain:
/// 1. --config CLI flag
/// 2. `KB_MCP_CONFIG` env var
/// 3. ./collections.ron
/// 4. ~/.config/kb-mcp/collections.ron
///
/// `KB_MCP_CACHE_DIR` overrides the file's `cache_dir`, so a launcher that cannot edit a shared
/// config, such as a plugin, can still choose where the index goes.
///
/// # Errors
///
/// No config file is found, the one found cannot be read or parsed as RON,
/// or it has an invalid `exclude` pattern.
pub fn load_config(explicit_path: Option<&Path>) -> Result<ResolvedConfig> {
    let (config_path, config_dir) = find_config(explicit_path)?;
    let content = std::fs::read_to_string(&config_path)
        .with_context(|| format!("failed to read config: {}", config_path.display()))?;
    let mut config: Config = ron::from_str(&content)
        .with_context(|| format!("failed to parse: {}", config_path.display()))?;
    if let Ok(dir) = std::env::var("KB_MCP_CACHE_DIR")
        && !dir.is_empty()
    {
        config.cache_dir = Some(dir);
    }
    config
        .resolve(&config_dir)
        .with_context(|| format!("invalid config: {}", config_path.display()))
}

fn find_config(explicit_path: Option<&Path>) -> Result<(PathBuf, PathBuf)> {
    // 1. Explicit --config flag
    if let Some(path) = explicit_path {
        let path = path.to_path_buf();
        let dir = path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        return Ok((path, dir));
    }

    // 2. KB_MCP_CONFIG env var
    if let Ok(env_path) = std::env::var("KB_MCP_CONFIG") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            let dir = path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            return Ok((path, dir));
        }
    }

    // 3. ./collections.ron (current working directory)
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let local = cwd.join("collections.ron");
    if local.exists() {
        return Ok((local, cwd));
    }

    // 4. ~/.config/kb-mcp/collections.ron
    if let Some(config_home) = dirs::config_dir() {
        let dir = config_home.join("kb-mcp");
        let global = dir.join("collections.ron");
        if global.exists() {
            return Ok((global, dir));
        }
    }

    anyhow::bail!(
        "No collections.ron found. Searched:\n  \
         - KB_MCP_CONFIG env var\n  \
         - ./collections.ron\n  \
         - ~/.config/kb-mcp/collections.ron\n\n\
         Create a collections.ron or pass --config <path>"
    )
}

fn default_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("kb-mcp")
}

/// Minimal `~` expansion — only handles `~/...` prefix. We avoid pulling in
/// a full shellexpand crate for this single use case.
fn shellexpand(s: &str) -> String {
    if let Some(rest) = s.strip_prefix("~/")
        && let Some(home) = dirs::home_dir()
    {
        return format!("{}/{rest}", home.display());
    }
    s.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(cache_dir: Option<&str>) -> Config {
        Config {
            cache_dir: cache_dir.map(str::to_string),
            index_names: IndexNames::default(),
            collections: vec![],
        }
    }

    #[test]
    fn relative_cache_dir_resolves_against_the_config_file() {
        let resolved = config(Some("index")).resolve(Path::new("/srv/kb")).unwrap();
        assert_eq!(resolved.cache_dir, PathBuf::from("/srv/kb/index"));
    }

    #[test]
    fn absolute_cache_dir_is_kept() {
        let resolved = config(Some("/var/cache/kb"))
            .resolve(Path::new("/srv/kb"))
            .unwrap();
        assert_eq!(resolved.cache_dir, PathBuf::from("/var/cache/kb"));
    }

    #[test]
    fn missing_cache_dir_uses_the_default() {
        let resolved = config(None).resolve(Path::new("/srv/kb")).unwrap();
        assert_eq!(resolved.cache_dir, default_cache_dir());
    }
}
