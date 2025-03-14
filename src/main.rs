use anyhow::Result;
use clap::{Parser, Subcommand};
use mdbook::preprocess::{CmdPreprocessor, Preprocessor};
use std::io;
use std::path::PathBuf;
use std::process;

use mdbook_include_doc::IncludeDocPreprocessor;

/// An mdBook preprocessor that integrates with include-doc to render external source files
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Check whether a renderer is supported by this preprocessor
    Supports {
        /// Renderer to check
        renderer: String,
    },
    /// Run the preprocessor
    #[command(name = "pre-process")]
    PreProcess {
        /// Path to book source directory
        #[arg(long)]
        dir: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();
    let preprocessor = IncludeDocPreprocessor;

    match args.command {
        Some(Commands::Supports { renderer }) => {
            if preprocessor.supports_renderer(&renderer) {
                process::exit(0);
            } else {
                process::exit(1);
            }
        }
        Some(Commands::PreProcess { dir: _ }) | None => {
            // Default behavior is to preprocess
            // Read the book from stdin instead of directly from the filesystem
            let (ctx, book) = CmdPreprocessor::parse_input(io::stdin())?;

            // Preprocess the book
            let processed_book = preprocessor.run(&ctx, book)?;

            // Output the processed book to stdout
            serde_json::to_writer(io::stdout(), &processed_book)?;
        }
    }

    Ok(())
}
