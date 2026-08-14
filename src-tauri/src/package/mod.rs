// Ginger Code — Package Manager
// Two ownership lanes: CURATED, CUSTOM.
// Package kinds: neovim plugin, LSP server, formatter, linter, debugger, CLI tool.
// Shared downloads are deduplicated; workspace resolution/pinning remains independent.

use std::path::PathBuf;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PackageError {
    #[error("package error: {0}")]
    Inner(String),
    #[error("package not found: {0}")]
    NotFound(String),
    #[error("install failed: {0}")]
    InstallFailed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OwnershipLane {
    Curated,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum PackageKind {
    NeovimPlugin,
    LspServer,
    Formatter,
    Linter,
    Debugger,
    CliTool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    pub description: String,
    pub kind: PackageKind,
    pub source: String,
    pub version: String,
    pub ownership: OwnershipLane,
    pub install_mechanism: String,
    pub runtime_compatibility: Vec<String>,
    pub detection_rules: Vec<String>,
    pub recommendation_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    pub catalog_id: String,
    pub version: String,
    pub installed_at: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    pub workspace_id: i64,
    pub runtime_version: String,
    pub packages: Vec<InstalledPackage>,
    pub manifest_hash: String,
    pub lock_hash: String,
}

pub struct PackageManager {
    catalog: RwLock<HashMap<String, CatalogEntry>>,
    cache_dir: PathBuf,
}

impl PackageManager {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self {
            catalog: RwLock::new(HashMap::new()),
            cache_dir,
        }
    }

    pub fn register_curated(&self, entry: CatalogEntry) {
        self.catalog.write().insert(entry.id.clone(), entry);
    }

    pub fn list_catalog(&self) -> Vec<CatalogEntry> {
        self.catalog.read().values().cloned().collect()
    }

    pub fn get_entry(&self, id: &str) -> Option<CatalogEntry> {
        self.catalog.read().get(id).cloned()
    }

    pub fn search(&self, query: &str) -> Vec<CatalogEntry> {
        let q = query.to_lowercase();
        self.catalog.read().values()
            .filter(|e| {
                e.name.to_lowercase().contains(&q)
                    || e.description.to_lowercase().contains(&q)
                    || e.id.contains(&q)
            })
            .cloned()
            .collect()
    }

    /// Install a package into the shared cache (deduplicated).
    pub fn install(&self, catalog_id: &str) -> Result<InstalledPackage, PackageError> {
        let entry = self.get_entry(catalog_id)
            .ok_or_else(|| PackageError::NotFound(catalog_id.into()))?;

        let pkg_dir = self.cache_dir.join(&entry.id).join(&entry.version);

        if pkg_dir.exists() {
            tracing::info!("Package {} v{} already cached", entry.id, entry.version);
            return Ok(InstalledPackage {
                catalog_id: entry.id,
                version: entry.version,
                installed_at: chrono::Utc::now().to_rfc3339(),
                path: pkg_dir.display().to_string(),
            });
        }

        std::fs::create_dir_all(&pkg_dir)
            .map_err(|e| PackageError::InstallFailed(e.to_string()))?;

        // TODO: actual install based on install_mechanism (git clone, npm, pip, cargo, etc.)
        tracing::info!("Installed {} v{}", entry.id, entry.version);

        Ok(InstalledPackage {
            catalog_id: entry.id,
            version: entry.version,
            installed_at: chrono::Utc::now().to_rfc3339(),
            path: pkg_dir.display().to_string(),
        })
    }

    pub fn cache_dir(&self) -> &PathBuf { &self.cache_dir }
}

/// Initialize the curated catalog with default entries.
pub fn init_curated_catalog(pm: &PackageManager) {
    let entries = vec![
        CatalogEntry {
            id: "nvim-treesitter".into(),
            name: "nvim-treesitter".into(),
            description: "Syntax highlighting and parsing".into(),
            kind: PackageKind::NeovimPlugin,
            source: "https://github.com/nvim-treesitter/nvim-treesitter".into(),
            version: "latest".into(),
            ownership: OwnershipLane::Curated,
            install_mechanism: "git".into(),
            runtime_compatibility: vec!["neovim>=0.9".into()],
            detection_rules: vec![],
            recommendation_rules: vec!["always".into()],
        },
        CatalogEntry {
            id: "telescope-nvim".into(),
            name: "telescope.nvim".into(),
            description: "Fuzzy finder".into(),
            kind: PackageKind::NeovimPlugin,
            source: "https://github.com/nvim-telescope/telescope.nvim".into(),
            version: "latest".into(),
            ownership: OwnershipLane::Curated,
            install_mechanism: "git".into(),
            runtime_compatibility: vec!["neovim>=0.9".into()],
            detection_rules: vec![],
            recommendation_rules: vec!["always".into()],
        },
        CatalogEntry {
            id: "lua-language-server".into(),
            name: "lua-language-server".into(),
            description: "LSP for Lua".into(),
            kind: PackageKind::LspServer,
            source: "https://github.com/LuaLS/lua-language-server".into(),
            version: "latest".into(),
            ownership: OwnershipLane::Curated,
            install_mechanism: "github-release".into(),
            runtime_compatibility: vec!["neovim>=0.9".into()],
            detection_rules: vec!["file:*.lua".into()],
            recommendation_rules: vec!["file:*.lua".into()],
        },
        CatalogEntry {
            id: "prettier".into(),
            name: "prettier".into(),
            description: "Code formatter".into(),
            kind: PackageKind::Formatter,
            source: "npm:prettier".into(),
            version: "latest".into(),
            ownership: OwnershipLane::Curated,
            install_mechanism: "npm".into(),
            runtime_compatibility: vec![],
            detection_rules: vec!["file:package.json".into()],
            recommendation_rules: vec!["framework.react".into(), "framework.nextjs".into()],
        },
        CatalogEntry {
            id: "eslint".into(),
            name: "eslint".into(),
            description: "Linter for JS/TS".into(),
            kind: PackageKind::Linter,
            source: "npm:eslint".into(),
            version: "latest".into(),
            ownership: OwnershipLane::Curated,
            install_mechanism: "npm".into(),
            runtime_compatibility: vec![],
            detection_rules: vec!["file:package.json".into()],
            recommendation_rules: vec!["language.typescript".into(), "language.javascript".into()],
        },
        CatalogEntry {
            id: "rust-analyzer".into(),
            name: "rust-analyzer".into(),
            description: "LSP for Rust".into(),
            kind: PackageKind::LspServer,
            source: "https://github.com/rust-lang/rust-analyzer".into(),
            version: "latest".into(),
            ownership: OwnershipLane::Curated,
            install_mechanism: "github-release".into(),
            runtime_compatibility: vec![],
            detection_rules: vec!["file:Cargo.toml".into()],
            recommendation_rules: vec!["language.rust".into()],
        },
        CatalogEntry {
            id: "pyright".into(),
            name: "pyright".into(),
            description: "LSP for Python".into(),
            kind: PackageKind::LspServer,
            source: "npm:pyright".into(),
            version: "latest".into(),
            ownership: OwnershipLane::Curated,
            install_mechanism: "npm".into(),
            runtime_compatibility: vec![],
            detection_rules: vec!["file:pyproject.toml".into(), "file:setup.py".into()],
            recommendation_rules: vec!["language.python".into()],
        },
        CatalogEntry {
            id: "gopls".into(),
            name: "gopls".into(),
            description: "LSP for Go".into(),
            kind: PackageKind::LspServer,
            source: "golang.org/x/tools/gopls".into(),
            version: "latest".into(),
            ownership: OwnershipLane::Curated,
            install_mechanism: "go-install".into(),
            runtime_compatibility: vec![],
            detection_rules: vec!["file:go.mod".into()],
            recommendation_rules: vec!["language.go".into()],
        },
    ];

    for entry in entries {
        pm.register_curated(entry);
    }
    tracing::info!("Curated catalog initialized with {} entries", pm.list_catalog().len());
}