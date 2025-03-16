mod function_extractor;
mod struct_finder;
mod enum_finder;
mod trait_finder;
mod impl_finder;

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use syn::{File, ItemFn};

pub use enum_finder::find_enum;
pub use function_extractor::{extract_function_body, find_function};
pub use impl_finder::{find_struct_impl, find_struct_method, find_trait_impl};
pub use struct_finder::find_struct;
pub use trait_finder::find_trait;

/// Read and parse a Rust source file
pub fn read_and_parse_file(file_path: &Path) -> Result<File> {
    let content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
    
    // Pretty print the code for consistent formatting
    let syntax_tree = syn::parse_file(&content)
        .with_context(|| format!("Failed to parse file: {}", file_path.display()))?;
    
    // Format the code using prettyplease
    let formatted_code = prettyplease::unparse(&syntax_tree);
    
    // Parse the formatted code
    syn::parse_file(&formatted_code)
        .with_context(|| format!("Failed to parse formatted file: {}", file_path.display()))
}


/// Find a function or method by name
pub fn find_function_or_method(parsed_file: &File, name: &str) -> Option<ItemFn> {
    // If it's a method (e.g., "StructName::method_name")
    if name.contains("::") {
        let parts: Vec<&str> = name.split("::").collect();
        if parts.len() == 2 {
            let struct_name = parts[0];
            let method_name = parts[1];
            
            if let Some(method) = find_struct_method(parsed_file, struct_name, method_name) {
                // Convert ImplItemFn to ItemFn (simplified)
                let fn_item = ItemFn {
                    attrs: method.attrs,
                    vis: method.vis,
                    sig: method.sig,
                    block: Box::new(method.block),
                };
                return Some(fn_item);
            }
        }
        None
    } else {
        // Look for a free function
        find_function(parsed_file, name)
    }
}