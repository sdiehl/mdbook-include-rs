pub(crate) mod enum_finder;
pub(crate) mod function_extractor;
pub(crate) mod impl_finder;
pub(crate) mod struct_finder;
pub(crate) mod trait_finder;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use syn::File;
use crate::parser::get_relative_path;

/// Read and parse a Rust source file
pub(crate) fn read_and_parse_file(file_path: &Path) -> Result<File> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", get_relative_path(file_path)))?;

    // Pretty print the code for consistent formatting
    let syntax_tree = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", get_relative_path(file_path)))?;

    // Format the code using prettyplease
    let formatted_code = prettyplease::unparse(&syntax_tree);

    // Parse the formatted code
    syn::parse_file(&formatted_code)
        .with_context(|| format!("Failed to parse formatted file: {}", file_path.display()))
}
