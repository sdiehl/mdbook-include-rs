use crate::formatter::{format_hidden, format_visible};

/// Represents a processed directive with hidden and visible code
pub(crate) struct Output {
    hidden_content: Vec<String>,
    visible_content: Vec<String>,
}

impl Output {
    pub(crate) fn new() -> Self {
        Self {
            hidden_content: Vec::new(),
            visible_content: Vec::new(),
        }
    }

    pub(crate) fn add_hidden_content(&mut self, content: String) {
        self.hidden_content.push(content);
    }

    pub(crate) fn add_visible_content(&mut self, content: String) {
        self.visible_content.push(content);
    }

    pub(crate) fn format(&self) -> String {
        let mut result = String::new();

        // Add hidden dependencies
        for content in &self.hidden_content {
            result.push_str(&format_hidden(content));
        }

        // Add visible content
        for content in &self.visible_content {
            result.push_str(&format_visible(content));
        }

        result
    }
}
