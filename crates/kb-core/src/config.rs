//! Configuration loading and resolution.
//!
//! Separates raw RON config (relative paths, optional fields) from resolved
//! config (absolute paths, defaults applied). This two-phase approach lets
//! collection paths in the RON file be relative to the config file's location,
//! enabling the same binary to serve different projects via different configs.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
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
}

impl Config {
    /// Resolve relative paths against the config file's parent directory.
    pub fn resolve(self, config_dir: &Path) -> ResolvedConfig {
        let cache_dir = self
            .cache_dir
            // Relative to the config file, like collection paths, so it does not depend on the working directory.
            .map(|s| config_dir.join(shellexpand(&s)))
            .unwrap_or_else(default_cache_dir);

        let index_names = self.index_names;
        let collections = self
            .collections
            .into_iter()
            .map(|c| {
                let path = config_dir.join(&c.path);
                let path = path.canonicalize().unwrap_or(path);
                ResolvedCollection {
                    name: c.name,
                    path,
                    description: c.description,
                    writable: c.writable,
                    sections: c.sections,
                    index_names,
                }
            })
            .collect();

        ResolvedConfig {
            cache_dir,
            collections,
        }
    }
}

/// Load config using resolution chain:
/// 1. --config CLI flag
/// 2. KB_MCP_CONFIG env var
/// 3. ./collections.ron
/// 4. ~/.config/kb-mcp/collections.ron
///
/// `KB_MCP_CACHE_DIR` overrides the file's `cache_dir`, so a launcher that cannot edit a shared
/// config, such as a plugin, can still choose where the index goes.
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
    Ok(config.resolve(&config_dir))
}

fn find_config(explicit_path: Option<&Path>) -> Result<(PathBuf, PathBuf)> {
    // 1. Explicit --config flag
    if let Some(path) = explicit_path {
        let path = path.to_path_buf();
        let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        return Ok((path, dir));
    }

    // 2. KB_MCP_CONFIG env var
    if let Ok(env_path) = std::env::var("KB_MCP_CONFIG") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
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
        let global = config_home.join("kb-mcp").join("collections.ron");
        if global.exists() {
            let dir = global.parent().unwrap().to_path_buf();
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
    if s.starts_with("~/")
        && let Some(home) = dirs::home_dir()
    {
        return format!("{}{}", home.display(), &s[1..]);
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
        let resolved = config(Some("index")).resolve(Path::new("/srv/kb"));
        assert_eq!(resolved.cache_dir, PathBuf::from("/srv/kb/index"));
    }

    #[test]
    fn absolute_cache_dir_is_kept() {
        let resolved = config(Some("/var/cache/kb")).resolve(Path::new("/srv/kb"));
        assert_eq!(resolved.cache_dir, PathBuf::from("/var/cache/kb"));
    }

    #[test]
    fn missing_cache_dir_uses_the_default() {
        let resolved = config(None).resolve(Path::new("/srv/kb"));
        assert_eq!(resolved.cache_dir, default_cache_dir());
    }
}
