use crate::directive::parse_directive_args;
use crate::extractor::enum_finder::find_enum;
use crate::extractor::function_extractor::find_function;
use crate::extractor::impl_finder::{find_struct_impl, find_trait_impl};
use crate::extractor::method_extractor::find_method;
use crate::extractor::read_and_parse_file;
use crate::extractor::struct_finder::find_struct;
use crate::extractor::trait_finder::find_trait;
use crate::formatter::{format_function_body, format_item, format_method_body};
use crate::output::Output;
use anyhow::{Context, Result};
use regex::{Captures, Regex};
use std::path::Path;
use std::{env, fs};
use syn::token::{Enum, Impl, Struct, Trait};
use syn::{File, ImplItemFn, Item, ItemFn};

/// Process the markdown content to find and replace include-rs directives
pub fn process_markdown(base_dir: &Path, source_path: &Path, content: &mut String) -> Result<()> {
    // This regex finds our directives anywhere in the content
    let re = Regex::new(
        r"(?ms)^#!\[((?:source_file|function|struct|enum|trait|impl|trait_impl|function_body)![\s\S]*?)\]$",
    )?;

    // Track the start position of each line to calculate line numbers
    let mut line_positions = Vec::new();
    let mut pos = 0;
    for line in content.lines() {
        line_positions.push(pos);
        pos += line.len() + 1; // +1 for the newline character
    }

    let result = re.replace_all(content, |caps: &Captures| {
        let include_doc_directive = caps.get(1).map_or("", |m| m.as_str());

        // Get match position information
        let match_start = caps.get(0).map_or(0, |m| m.start());

        // Find line number and column based on position
        let (line_num, col_num) = find_line_and_col(&line_positions, match_start);

        // Process the directive with include_doc_macro
        match process_include_rs_directive(base_dir, include_doc_directive) {
            Ok(processed) => processed,
            Err(e) => {
                let rel_path = get_relative_path(source_path);
                eprintln!("{}:{}:{}: {}", rel_path, line_num, col_num, e);
                format!("{}:{}:{}: {}", rel_path, line_num, col_num, e)
            }
        }
    });

    *content = result.to_string();
    Ok(())
}

/// Find line and column number from a position in the text
fn find_line_and_col(line_positions: &[usize], position: usize) -> (usize, usize) {
    let mut line_idx = 0;

    // Find the line containing the position
    for (idx, &start) in line_positions.iter().enumerate() {
        if position >= start {
            line_idx = idx;
        } else {
            break;
        }
    }

    // Line numbers are 1-indexed
    let line_num = line_idx + 1;
    // Calculate column number (1-indexed)
    let col_num = position - line_positions[line_idx] + 1;

    (line_num, col_num)
}

/// Get the path relative to the current working directory
pub(crate) fn get_relative_path(path: &Path) -> String {
    if let Ok(current_dir) = env::current_dir() {
        if let Ok(relative) = path.strip_prefix(&current_dir) {
            return format!(
                ".{}{}",
                std::path::MAIN_SEPARATOR,
                relative.to_string_lossy()
            );
        }
    }

    // Fall back to the original path if we can't get a relative path
    format!(".{}{}", std::path::MAIN_SEPARATOR, path.to_string_lossy())
}

/// Process an include-rs directive
fn process_include_rs_directive(base_dir: &Path, directive: &str) -> Result<String> {
    // Parse the directive name
    let directive_name = if let Some(pos) = directive.find('!') {
        &directive[0..pos]
    } else {
        // Not a recognized directive format
        return Ok(directive.to_string());
    };

    // Process the directive based on its type
    let result = match directive_name {
        "source_file" => process_source_file_directive(base_dir, directive)?,
        "function_body" => {
            // Try to find as a regular function first
            if let Ok(result) = process_directive::<ItemFn>(
                base_dir,
                directive,
                |f, n| Some(Item::Fn(find_function(f, n)?)),
                format_function_body,
            ) {
                result
            } else {
                // If not found, try to find as a method
                process_method_body_directive(base_dir, directive)?
            }
        }
        "struct" => process_directive::<Struct>(
            base_dir,
            directive,
            |f, n| Some(Item::Struct(find_struct(f, n)?)),
            format_item,
        )?,
        "enum" => process_directive::<Enum>(
            base_dir,
            directive,
            |f, n| Some(Item::Enum(find_enum(f, n)?)),
            format_item,
        )?,
        "trait" => process_directive::<Trait>(
            base_dir,
            directive,
            |f, n| Some(Item::Trait(find_trait(f, n)?)),
            format_item,
        )?,
        "impl" => process_directive::<Impl>(
            base_dir,
            directive,
            |f, n| Some(Item::Impl(find_struct_impl(f, n)?)),
            format_item,
        )?,
        "trait_impl" => process_directive::<Impl>(
            base_dir,
            directive,
            |f, n| {
                // For trait_impl, the item_name should have the format "TraitName for StructName"
                let parts: Vec<&str> = n.split(" for ").collect();
                if parts.len() != 2 {
                    return None;
                }

                let trait_name = parts[0].trim();
                let struct_name = parts[1].trim();

                Some(Item::Impl(find_trait_impl(f, trait_name, struct_name)?))
            },
            format_item,
        )?,
        "function" => {
            // Try to find as a regular function first
            if let Ok(result) = process_directive::<ItemFn>(
                base_dir,
                directive,
                |f, n| Some(Item::Fn(find_function(f, n)?)),
                format_item,
            ) {
                result
            } else {
                // If not found, try to find as a method
                process_method_directive(base_dir, directive)?
            }
        }
        _ => {
            // Not a recognized directive
            return Ok(directive.to_string());
        }
    };

    // Format the result as a Rust code block
    Ok(result.trim().to_string())
}

/// Process source_file! directive
fn process_source_file_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let directive = parse_directive_args(directive)?;
    let absolute_path = base_dir.join(directive.file_path);
    let content = fs::read_to_string(&absolute_path)
        .with_context(|| format!("Failed to read file: {}", get_relative_path(&absolute_path)))?;
    Ok(content)
}

/// Process method_body directive for methods in impl blocks
fn process_method_body_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let directive = parse_directive_args(directive)?;
    if directive.item.is_none() {
        return Err(anyhow::anyhow!("Method specification is required"));
    }
    let absolute_path = base_dir.join(directive.file_path);
    let parsed_file = read_and_parse_file(&absolute_path)?;
    let method_spec = directive.item.as_ref().expect("method spec is required");
    let method = find_method(&parsed_file, method_spec)
        .with_context(|| format!("Method '{}' not found", method_spec))?;

    // Process extra dependencies if provided
    let (hidden_deps, visible_deps) =
        process_extra_for_method(&parsed_file, &method, &directive.extra_items);
    let mut result = Output::new();
    for dep in hidden_deps {
        result.add_hidden_content(format_item(&dep));
    }
    for dep in visible_deps {
        result.add_visible_content(format_item(&dep));
    }

    result.add_visible_content(format_method_body(&method));
    Ok(result.format())
}

/// Process method directive for methods in impl blocks (complete method including signature)
fn process_method_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let directive = parse_directive_args(directive)?;
    if directive.item.is_none() {
        return Err(anyhow::anyhow!("Method specification is required"));
    }
    let absolute_path = base_dir.join(directive.file_path);
    let parsed_file = read_and_parse_file(&absolute_path)?;
    let method_spec = directive.item.as_ref().expect("method spec is required");
    let method = find_method(&parsed_file, method_spec)
        .with_context(|| format!("Method '{}' not found", method_spec))?;

    // Process extra dependencies if provided
    let (hidden_deps, visible_deps) =
        process_extra_for_method(&parsed_file, &method, &directive.extra_items);
    let mut result = Output::new();
    for dep in hidden_deps {
        result.add_hidden_content(format_item(&dep));
    }
    for dep in visible_deps {
        result.add_visible_content(format_item(&dep));
    }

    // Use the method formatter to show the complete method signature and body
    use crate::formatter::format_method;
    result.add_visible_content(format_method(&method));
    Ok(result.format())
}

/// Helper function to process extra items
fn process_extra(
    parsed_file: &File,
    primary_item: &Item,
    extra_items: &[String],
) -> (Vec<Item>, Vec<Item>) {
    let mut hidden = Vec::new();
    let mut visible = Vec::new();

    for item in extra_items {
        if item.starts_with("struct ") {
            let struct_name = item.trim_start_matches("struct ").trim();
            if let Some(struct_def) = find_struct(parsed_file, struct_name) {
                visible.push(Item::Struct(struct_def));
            }
        } else if item.starts_with("enum ") {
            let enum_name = item.trim_start_matches("enum ").trim();
            if let Some(enum_def) = find_enum(parsed_file, enum_name) {
                visible.push(Item::Enum(enum_def));
            }
        } else if item.starts_with("trait ") {
            let trait_name = item.trim_start_matches("trait ").trim();
            if let Some(trait_def) = find_trait(parsed_file, trait_name) {
                visible.push(Item::Trait(trait_def));
            }
        } else if item.starts_with("impl ") {
            if item.contains(" for ") {
                // Trait implementation for a struct
                let parts: Vec<&str> = item.trim_start_matches("impl ").split(" for ").collect();
                if parts.len() == 2 {
                    let trait_name = parts[0].trim();
                    let struct_name = parts[1].trim();
                    if let Some(impl_def) = find_trait_impl(parsed_file, trait_name, struct_name) {
                        visible.push(Item::Impl(impl_def));
                    }
                }
            } else {
                // Struct implementation
                let struct_name = item.trim_start_matches("impl ").trim();
                if let Some(impl_def) = find_struct_impl(parsed_file, struct_name) {
                    visible.push(Item::Impl(impl_def));
                }
            }
        } else {
            // Assume it's a struct or enum
            if let Some(struct_def) = find_struct(parsed_file, item) {
                visible.push(Item::Struct(struct_def));
            } else if let Some(enum_def) = find_enum(parsed_file, item) {
                visible.push(Item::Enum(enum_def));
            }
        }
    }

    // Now go through every item in the file, and if it's not in visible it must be hidden
    for item in &parsed_file.items {
        if item == primary_item {
            continue;
        }
        if !visible.contains(item) {
            hidden.push(item.clone());
        }
    }

    (hidden, visible)
}

/// Helper function to process extra items for methods - simplified version
fn process_extra_for_method(
    parsed_file: &File,
    _method: &ImplItemFn,
    extra_items: &[String],
) -> (Vec<Item>, Vec<Item>) {
    let hidden = Vec::new();
    let mut visible = Vec::new();

    for item in extra_items {
        if item.starts_with("struct ") {
            let struct_name = item.trim_start_matches("struct ").trim();
            if let Some(struct_def) = find_struct(parsed_file, struct_name) {
                visible.push(Item::Struct(struct_def));
            }
        } else if item.starts_with("enum ") {
            let enum_name = item.trim_start_matches("enum ").trim();
            if let Some(enum_def) = find_enum(parsed_file, enum_name) {
                visible.push(Item::Enum(enum_def));
            }
        } else if item.starts_with("trait ") {
            let trait_name = item.trim_start_matches("trait ").trim();
            if let Some(trait_def) = find_trait(parsed_file, trait_name) {
                visible.push(Item::Trait(trait_def));
            }
        } else if item.starts_with("impl ") {
            if item.contains(" for ") {
                // Trait implementation for a struct
                let parts: Vec<&str> = item.trim_start_matches("impl ").split(" for ").collect();
                if parts.len() == 2 {
                    let trait_name = parts[0].trim();
                    let struct_name = parts[1].trim();
                    if let Some(impl_def) = find_trait_impl(parsed_file, trait_name, struct_name) {
                        visible.push(Item::Impl(impl_def));
                    }
                }
            } else {
                // Struct implementation
                let struct_name = item.trim_start_matches("impl ").trim();
                if let Some(impl_def) = find_struct_impl(parsed_file, struct_name) {
                    visible.push(Item::Impl(impl_def));
                }
            }
        } else {
            // Assume it's a struct or enum
            if let Some(struct_def) = find_struct(parsed_file, item) {
                visible.push(Item::Struct(struct_def));
            } else if let Some(enum_def) = find_enum(parsed_file, item) {
                visible.push(Item::Enum(enum_def));
            }
        }
    }

    // For methods, we don't add all other items as hidden by default
    // since the method is part of an impl block

    (hidden, visible)
}

/// Process enum! directive
fn process_directive<T>(
    base_dir: &Path,
    directive: &str,
    finder: impl Fn(&File, &str) -> Option<Item>,
    formatter: impl Fn(&Item) -> String,
) -> Result<String> {
    let directive = parse_directive_args(directive)?;
    if directive.item.is_none() {
        return Err(anyhow::anyhow!(
            "{} name is required",
            std::any::type_name::<T>()
        ));
    }
    let absolute_path = base_dir.join(directive.file_path);
    let parsed_file = read_and_parse_file(&absolute_path)?;
    let item_name = directive.item.as_ref().expect("item name is required");
    let item = finder(&parsed_file, item_name)
        .with_context(|| format!("{} '{}' not found", std::any::type_name::<T>(), item_name))?;
    let (hidden_deps, visible_deps) = process_extra(&parsed_file, &item, &directive.extra_items);
    let mut result = Output::new();
    for dep in hidden_deps {
        result.add_hidden_content(format_item(&dep));
    }
    for dep in visible_deps {
        result.add_visible_content(format_item(&dep));
    }

    result.add_visible_content(formatter(&item));
    Ok(result.format())
}
