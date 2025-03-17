use anyhow::Result;
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use toml::Value;

use crate::parser::process_markdown;

/// Preprocessor that handles include-rs code blocks
pub struct IncludeRsPreprocessor;

impl Preprocessor for IncludeRsPreprocessor {
    fn name(&self) -> &str {
        "include-rs"
    }

    fn run(&self, ctx: &PreprocessorContext, mut book: Book) -> Result<Book> {
        let config_section = ctx.config.get_preprocessor(self.name());
        // Get global base_dir from config if provided, otherwise set to None
        let global_base_dir = if let Some(config) = config_section {
            if let Some(Value::String(dir)) = config.get("base-dir") {
                Some(ctx.root.join(dir))
            } else {
                None
            }
        } else {
            None
        };

        let src_dir = ctx.root.join("src");

        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                // Get the directory of the chapter markdown file to use as the base if no global base_dir
                let base_dir = if let Some(ref global_dir) = global_base_dir {
                    global_dir.clone()
                } else if let Some(ref source_path) = chapter.source_path {
                    // The SUMMARY.md file is always in src
                    // Use the directory containing the markdown file as base
                    if let Some(parent) = source_path.parent() {
                        src_dir.join(parent)
                    } else {
                        src_dir.clone()
                    }
                } else {
                    // Fallback to root if no source path
                    src_dir.clone()
                };

                if let Err(e) = process_markdown(&base_dir, &mut chapter.content) {
                    eprintln!("Error processing chapter '{}': {}", chapter.name, e);
                }
            }
        });

        Ok(book)
    }

    fn supports_renderer(&self, _renderer: &str) -> bool {
        // This preprocessor supports all renderers
        true
    }
}
