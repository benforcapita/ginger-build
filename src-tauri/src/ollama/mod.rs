/// Ginger Code — Ollama Native Helper (LLD 70-72)
/// Ollama may power optional local features: diff summaries, selected-code
/// explanations, commit-message drafts, lightweight reviews, agent-output
/// summaries. If unavailable, no editor functionality is blocked.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OllamaFeature {
    DiffSummary,
    CodeExplanation,
    CommitMessageDraft,
    LightweightReview,
    AgentOutputSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaRequest {
    pub feature: OllamaFeature,
    pub model: String,
    pub context: String, // bounded, explicit context
    pub max_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaResponse {
    pub text: String,
    pub ok: bool,
}

pub struct OllamaClient {
    pub base_url: String,
    pub available: bool,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            available: false,
        }
    }

    /// Check if Ollama is reachable. Never blocks editor functionality.
    pub fn check_available(&mut self) -> bool {
        // In a real impl this would ping the Ollama API.
        // Here we just report the configured state.
        self.available
    }

    /// Run a bounded helper request. Returns None if unavailable.
    pub fn run(&self, _req: &OllamaRequest) -> Option<OllamaResponse> {
        if !self.available {
            return None;
        }
        // Placeholder: real impl would POST to /api/generate.
        None
    }
}

impl Default for OllamaClient {
    fn default() -> Self {
        Self::new("http://127.0.0.1:11434")
    }
}