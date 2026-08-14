/// Ginger Code — Terminal Scrollback & Rendering States (LLD 146-147)
/// Bounded visible scrollback by default. Optional transcript retention has
/// independent size/retention controls. Rendering states: starting,
/// connected, exited, disconnected, recovered-metadata-only.

use serde::{Deserialize, Serialize};

pub const DEFAULT_SCROLLBACK_LINES: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerminalRenderState {
    Starting,
    Connected,
    Exited,
    Disconnected,
    RecoveredMetadataOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrollbackConfig {
    pub visible_lines: usize,
    pub transcript_enabled: bool,
    pub transcript_max_bytes: usize,
    pub transcript_retention_days: u32,
}

impl Default for ScrollbackConfig {
    fn default() -> Self {
        Self {
            visible_lines: DEFAULT_SCROLLBACK_LINES,
            transcript_enabled: false,
            transcript_max_bytes: 5 * 1024 * 1024,
            transcript_retention_days: 7,
        }
    }
}

pub struct ScrollbackBuffer {
    lines: Vec<String>,
    max_lines: usize,
}

impl ScrollbackBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::new(),
            max_lines,
        }
    }

    /// Append a line, trimming from the front if over the bound.
    pub fn push(&mut self, line: String) {
        self.lines.push(line);
        if self.lines.len() > self.max_lines {
            let excess = self.lines.len() - self.max_lines;
            self.lines.drain(0..excess);
        }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

impl Default for ScrollbackBuffer {
    fn default() -> Self {
        Self::new(DEFAULT_SCROLLBACK_LINES)
    }
}