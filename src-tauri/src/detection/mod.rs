// Ginger Code — Project Detection and Recommendations
// Scans deterministic artifacts (package.json, Cargo.toml, etc.) to detect capabilities.
// Capabilities map to curated packages. Detection runs async, never blocks startup.

use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DetectionError {
    #[error("detection error: {0}")]
    Inner(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    LanguageTypeScript,
    LanguageJavaScript,
    LanguagePython,
    LanguageRust,
    LanguageGo,
    LanguageCSharp,
    FrameworkReact,
    FrameworkNextjs,
    FrameworkSvelte,
    FrameworkFastapi,
    FrameworkExpress,
    StyleTailwind,
    TestVitest,
    TestPlaywright,
    TestPytest,
    TestCargo,
    DatabasePrisma,
    DatabaseSqlalchemy,
    ContainerDocker,
    DotnetAspNet,
    PackageManagerPnpm,
    PackageManagerNpm,
    PackageManagerYarn,
    PackageManagerPip,
    PackageManagerCargo,
    PackageManagerGo,
}

impl Capability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LanguageTypeScript => "language.typescript",
            Self::LanguageJavaScript => "language.javascript",
            Self::LanguagePython => "language.python",
            Self::LanguageRust => "language.rust",
            Self::LanguageGo => "language.go",
            Self::LanguageCSharp => "language.csharp",
            Self::FrameworkReact => "framework.react",
            Self::FrameworkNextjs => "framework.nextjs",
            Self::FrameworkSvelte => "framework.svelte",
            Self::FrameworkFastapi => "framework.fastapi",
            Self::FrameworkExpress => "framework.express",
            Self::StyleTailwind => "style.tailwind",
            Self::TestVitest => "test.vitest",
            Self::TestPlaywright => "test.playwright",
            Self::TestPytest => "test.pytest",
            Self::TestCargo => "test.cargo",
            Self::DatabasePrisma => "database.prisma",
            Self::DatabaseSqlalchemy => "database.sqlalchemy",
            Self::ContainerDocker => "container.docker",
            Self::DotnetAspNet => "dotnet.aspnet",
            Self::PackageManagerPnpm => "package-manager.pnpm",
            Self::PackageManagerNpm => "package-manager.npm",
            Self::PackageManagerYarn => "package-manager.yarn",
            Self::PackageManagerPip => "package-manager.pip",
            Self::PackageManagerCargo => "package-manager.cargo",
            Self::PackageManagerGo => "package-manager.go",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub root: String,
    pub capabilities: Vec<String>,
    pub detected_files: Vec<String>,
}

pub struct ProjectScanner;

impl ProjectScanner {
    pub fn new() -> Self { Self }

    /// Scan a project root for capabilities. Never blocks, never fails — returns empty on error.
    pub fn scan(&self, root: &PathBuf) -> ProjectInfo {
        let mut caps = HashSet::new();
        let mut files = Vec::new();

        // Node.js / TypeScript
        if root.join("package.json").exists() {
            files.push("package.json".into());
            if let Ok(content) = std::fs::read_to_string(root.join("package.json")) {
                if content.contains("\"typescript\"") || root.join("tsconfig.json").exists() {
                    caps.insert(Capability::LanguageTypeScript);
                } else {
                    caps.insert(Capability::LanguageJavaScript);
                }
                if content.contains("\"react\"") { caps.insert(Capability::FrameworkReact); }
                if content.contains("\"next\"") { caps.insert(Capability::FrameworkNextjs); }
                if content.contains("\"svelte\"") { caps.insert(Capability::FrameworkSvelte); }
                if content.contains("\"tailwindcss\"") { caps.insert(Capability::StyleTailwind); }
                if content.contains("\"vitest\"") { caps.insert(Capability::TestVitest); }
                if content.contains("\"@playwright/test\"") { caps.insert(Capability::TestPlaywright); }
                if content.contains("\"prisma\"") { caps.insert(Capability::DatabasePrisma); }
                if content.contains("\"express\"") { caps.insert(Capability::FrameworkExpress); }
                if root.join("pnpm-lock.yaml").exists() { caps.insert(Capability::PackageManagerPnpm); }
                else if root.join("yarn.lock").exists() { caps.insert(Capability::PackageManagerYarn); }
                else { caps.insert(Capability::PackageManagerNpm); }
            }
        }

        if root.join("tsconfig.json").exists() {
            files.push("tsconfig.json".into());
            caps.insert(Capability::LanguageTypeScript);
        }

        // Python
        if root.join("pyproject.toml").exists() {
            files.push("pyproject.toml".into());
            caps.insert(Capability::LanguagePython);
            caps.insert(Capability::PackageManagerPip);
            if let Ok(content) = std::fs::read_to_string(root.join("pyproject.toml")) {
                if content.contains("fastapi") { caps.insert(Capability::FrameworkFastapi); }
                if content.contains("pytest") { caps.insert(Capability::TestPytest); }
                if content.contains("sqlalchemy") { caps.insert(Capability::DatabaseSqlalchemy); }
            }
        }
        if root.join("setup.py").exists() {
            files.push("setup.py".into());
            caps.insert(Capability::LanguagePython);
            caps.insert(Capability::PackageManagerPip);
        }

        // Rust
        if root.join("Cargo.toml").exists() {
            files.push("Cargo.toml".into());
            caps.insert(Capability::LanguageRust);
            caps.insert(Capability::PackageManagerCargo);
            caps.insert(Capability::TestCargo);
        }

        // Go
        if root.join("go.mod").exists() {
            files.push("go.mod".into());
            caps.insert(Capability::LanguageGo);
            caps.insert(Capability::PackageManagerGo);
        }

        // .NET
        if let Ok(entries) = std::fs::read_dir(root) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".csproj") {
                    files.push(name.clone());
                    caps.insert(Capability::LanguageCSharp);
                    if name.contains("aspnet") || name.contains("web") {
                        caps.insert(Capability::DotnetAspNet);
                    }
                }
            }
        }

        // Docker
        if root.join("Dockerfile").exists() || root.join("docker-compose.yml").exists() {
            files.push("Dockerfile".into());
            caps.insert(Capability::ContainerDocker);
        }

        let capabilities: Vec<String> = caps.iter().map(|c| c.as_str().to_string()).collect();
        ProjectInfo {
            root: root.display().to_string(),
            capabilities,
            detected_files: files,
        }
    }

    /// Map capabilities to recommended package IDs from the catalog.
    pub fn recommend(&self, caps: &[String]) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        for cap in caps {
            match cap.as_str() {
                "language.typescript" | "language.javascript" => {
                    recs.push(Recommendation { package_id: "eslint".into(), reason: format!("{cap} project") });
                }
                "framework.react" | "framework.nextjs" => {
                    recs.push(Recommendation { package_id: "prettier".into(), reason: format!("{cap} project") });
                }
                "language.rust" => {
                    recs.push(Recommendation { package_id: "rust-analyzer".into(), reason: "Rust project".into() });
                }
                "language.python" => {
                    recs.push(Recommendation { package_id: "pyright".into(), reason: "Python project".into() });
                }
                "language.go" => {
                    recs.push(Recommendation { package_id: "gopls".into(), reason: "Go project".into() });
                }
                "language.lua" => {
                    recs.push(Recommendation { package_id: "lua-language-server".into(), reason: "Lua files detected".into() });
                }
                _ => {}
            }
        }

        // Always recommend treesitter + telescope
        recs.insert(0, Recommendation { package_id: "nvim-treesitter".into(), reason: "always".into() });
        recs.insert(1, Recommendation { package_id: "telescope-nvim".into(), reason: "always".into() });

        recs
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub package_id: String,
    pub reason: String,
}

impl Default for ProjectScanner {
    fn default() -> Self { Self::new() }
}