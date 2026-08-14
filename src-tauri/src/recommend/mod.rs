/// Ginger Code — Package Recommendation Engine (LLD 79-80)
/// Recommendations derive from deterministic project evidence. Capabilities
/// include language/framework/style/test/database/container markers with
/// confidence and evidence lists. Recommendation items map only to known
/// package IDs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationState {
    New,
    Shown,
    Installed,
    Ignored,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityEvidence {
    pub capability: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub package_id: String,
    pub capability: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
    pub state: RecommendationState,
    pub reason: String,
}

pub struct RecommendationEngine;

impl RecommendationEngine {
    /// Rank recommendations by priority: capability gap, project convention,
    /// curated preset relevance, confidence, and user history.
    /// `known_package_ids` = the set of package IDs the catalog knows.
    pub fn rank(
        &self,
        capabilities: &[CapabilityEvidence],
        known_package_ids: &[String],
        installed: &[String],
    ) -> Vec<Recommendation> {
        let mut recs = Vec::new();

        for cap in capabilities {
            // Map capability -> package id (deterministic, known only)
            let package_id = match cap.capability.as_str() {
                "typescript" => "typescript-language-server",
                "react" => "typescript-language-server",
                "rust" => "rust-analyzer",
                "python" => "pyright",
                "go" => "gopls",
                "lua" => "lua-language-server",
                "prettier" => "prettier",
                "eslint" => "eslint",
                _ => continue,
            };

            // Only recommend known package IDs
            if !known_package_ids.contains(&package_id.to_string()) {
                continue;
            }

            let state = if installed.contains(&package_id.to_string()) {
                RecommendationState::Installed
            } else {
                RecommendationState::New
            };

            recs.push(Recommendation {
                package_id: package_id.to_string(),
                capability: cap.capability.clone(),
                confidence: cap.confidence,
                evidence: cap.evidence.clone(),
                state,
                reason: format!("Detected {} with confidence {:.0}%", cap.capability, cap.confidence * 100.0),
            });
        }

        // Sort by confidence descending
        recs.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        recs
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_known_packages_recommended() {
        let caps = vec![CapabilityEvidence {
            capability: "typescript".to_string(),
            confidence: 0.9,
            evidence: vec!["tsconfig.json".to_string()],
        }];
        let known = vec!["typescript-language-server".to_string()];
        let recs = RecommendationEngine.rank(&caps, &known, &[]);
        assert_eq!(recs.len(), 1);
        assert_eq!(recs[0].package_id, "typescript-language-server");
    }

    #[test]
    fn unknown_capability_skipped() {
        let caps = vec![CapabilityEvidence {
            capability: "cobol".to_string(),
            confidence: 0.9,
            evidence: vec![],
        }];
        let known = vec!["typescript-language-server".to_string()];
        let recs = RecommendationEngine.rank(&caps, &known, &[]);
        assert!(recs.is_empty());
    }
}