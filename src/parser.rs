use anyhow::{Context, Result};
use regex::{Captures, Regex};
use std::path::Path;
use syn::File;

use crate::extractor::{
    extract_function_body,
    find_enum,
    find_function_or_method, find_struct, find_struct_impl,
    find_trait, find_trait_impl, read_and_parse_file,
};

use crate::formatter::{format_enum, format_function, format_function_body, format_hidden, format_impl, format_source_file, format_struct, format_trait, format_visible};

/// Process the markdown content to find and replace include-doc code blocks
pub fn process_markdown(base_dir: &Path, content: &mut String) -> Result<()> {
    // This regex finds our directives anywhere in the content
    let re = Regex::new(r"(?ms)^#!\[((?:source_file|function|struct|enum|trait|impl|trait_impl|function_body)![\s\S]*?)\]$")?;

    let result = re.replace_all(content, |caps: &Captures| {
        let include_doc_directive = caps.get(1).map_or("", |m| m.as_str());

        // Process the directive with include_doc_macro
        match process_include_doc_directive(base_dir, include_doc_directive) {
            Ok(processed) => processed,
            Err(e) => {
                eprintln!("Error processing include-doc directive: {}", e);
                format!("Error processing include-doc directive: {}", e)
            }
        }
    });

    *content = result.to_string();
    Ok(())
}

/// Represents a processed directive with hidden and visible code
struct ProcessedDirective {
    hidden_imports: Vec<String>,
    hidden_dependencies: Vec<String>,
    visible_content: Vec<String>,
}

impl ProcessedDirective {
    fn new() -> Self {
        Self {
            hidden_imports: Vec::new(),
            hidden_dependencies: Vec::new(),
            visible_content: Vec::new(),
        }
    }

    fn add_hidden_import(&mut self, import: String) {
        self.hidden_imports.push(import);
    }

    fn add_hidden_dependency(&mut self, dependency: String) {
        self.hidden_dependencies.push(dependency);
    }

    fn add_visible_content(&mut self, content: String) {
        self.visible_content.push(content);
    }

    fn format(&self) -> String {
        let mut result = String::new();
        
        // Add hidden imports
        for import in &self.hidden_imports {
            result.push_str(&format_hidden(import));
        }
        
        // Add hidden dependencies
        for dependency in &self.hidden_dependencies {
            result.push_str(&format_hidden(dependency));
        }
        
        // Add visible content
        for content in &self.visible_content {
            result.push_str(&format_visible(content));
        }
        
        result
    }
}

/// Process an include-doc directive
pub fn process_include_doc_directive(base_dir: &Path, directive: &str) -> Result<String> {
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
        "function_body" => process_function_body_directive(base_dir, directive)?,
        "struct" => process_struct_directive(base_dir, directive)?,
        "enum" => process_enum_directive(base_dir, directive)?,
        "trait" => process_trait_directive(base_dir, directive)?,
        "impl" => process_impl_directive(base_dir, directive)?,
        "trait_impl" => process_trait_impl_directive(base_dir, directive)?,
        "function" => process_function_directive(base_dir, directive)?,
        _ => {
            // Not a recognized directive
            return Ok(directive.to_string());
        }
    };
    
    // Format the result as a Rust code block
    Ok(format!("{}", result.trim()))
}

/// Parse directive arguments (file path, item name, optional dependencies)
fn parse_directive_args(directive: &str) -> Result<(String, Option<String>, Option<Vec<String>>)> {
    // Basic regex to parse directive: directive_name!("path/to/file.rs", item_name, [deps...])
    let re = Regex::new(r#"([a-z_]+)!\s*\(\s*"([^"]+)"\s*(?:,\s*([^,\[\]]+))?(?:,\s*\[(.*)\])?\s*\)"#)?;
    
    let captures = re.captures(directive)
        .with_context(|| format!("Failed to parse directive: {}", directive))?;
    
    let file_path = captures.get(2)
        .map(|m| m.as_str().to_string())
        .with_context(|| "File path is required")?;
    
    let item_name = captures.get(3).map(|m| m.as_str().trim().to_string());
    
    let dependencies = if let Some(deps_match) = captures.get(4) {
        let deps_str = deps_match.as_str();
        if deps_str.trim().is_empty() {
            None
        } else {
            let deps: Vec<String> = deps_str.split(',')
                .map(|s| s.trim().to_string())
                .collect();
            Some(deps)
        }
    } else {
        None
    };
    
    Ok((file_path, item_name, dependencies))
}

/// Helper function to extract imports from a parsed file
fn extract_imports(parsed_file: &File) -> Vec<String> {
    let mut imports = Vec::new();
    
    for item in &parsed_file.items {
        if let syn::Item::Use(_) = item {
            let formatted = prettyplease::unparse(&syn::File {
                shebang: None,
                attrs: vec![],
                items: vec![item.clone()],
            });
            imports.push(formatted);
        }
    }
    
    imports
}

/// Process source_file! directive
fn process_source_file_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let (file_path, _, _) = parse_directive_args(directive)?;
    
    let absolute_path = base_dir.join(file_path);
    let parsed_file = read_and_parse_file(&absolute_path)?;
    
    let formatted_code = format_source_file(&parsed_file);
    
    Ok(formatted_code)
}

/// Helper function to process dependencies
fn process_dependencies(parsed_file: &File, dependencies: &[String]) -> (Vec<String>, Vec<String>) {
    let mut hidden = Vec::new();
    let mut visible = Vec::new();
    
    for dep in dependencies {
        if dep.starts_with("struct ") {
            let struct_name = dep.trim_start_matches("struct ").trim();
            if let Some(struct_def) = find_struct(parsed_file, struct_name) {
                visible.push(format_struct(&struct_def));
            }
        } else if dep.starts_with("enum ") {
            let enum_name = dep.trim_start_matches("enum ").trim();
            if let Some(enum_def) = find_enum(parsed_file, enum_name) {
                visible.push(format_enum(&enum_def));
            }
        } else if dep.starts_with("trait ") {
            let trait_name = dep.trim_start_matches("trait ").trim();
            if let Some(trait_def) = find_trait(parsed_file, trait_name) {
                visible.push(format_trait(&trait_def));
            }
        } else if dep.starts_with("impl ") {
            if dep.contains(" for ") {
                // Trait implementation for a struct
                let parts: Vec<&str> = dep.trim_start_matches("impl ").split(" for ").collect();
                if parts.len() == 2 {
                    let trait_name = parts[0].trim();
                    let struct_name = parts[1].trim();
                    if let Some(impl_def) = find_trait_impl(parsed_file, trait_name, struct_name) {
                        visible.push(format_impl(&impl_def));
                    }
                }
            } else {
                // Struct implementation
                let struct_name = dep.trim_start_matches("impl ").trim();
                if let Some(impl_def) = find_struct_impl(parsed_file, struct_name) {
                    visible.push(format_impl(&impl_def));
                }
            }
        } else {
            // Assume it's a struct or enum
            if let Some(struct_def) = find_struct(parsed_file, dep) {
                hidden.push(format_struct(&struct_def));
            } else if let Some(enum_def) = find_enum(parsed_file, dep) {
                hidden.push(format_enum(&enum_def));
            }
        }
    }
    
    (hidden, visible)
}

/// Process function_body! directive
fn process_function_body_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let (parsed_file, function_name, _, mut result) = setup_directive_processor(base_dir, directive)?;
    
    if function_name.is_empty() {
        return Err(anyhow::anyhow!("Function name is required"));
    }
    
    let function_def = extract_function_body(&parsed_file, &function_name)
        .with_context(|| format!("Function '{}' not found", function_name))?;
    
    // Add the function body
    let function_str = format_function_body(&function_def);
    result.add_visible_content(function_str);
    
    Ok(result.format())
}

/// Process struct! directive
fn process_struct_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let (parsed_file, struct_name, _, mut result) = setup_directive_processor(base_dir, directive)?;
    
    if struct_name.is_empty() {
        return Err(anyhow::anyhow!("Struct name is required"));
    }
    
    let struct_def = find_struct(&parsed_file, &struct_name)
        .with_context(|| format!("Struct '{}' not found", struct_name))?;
    
    // Add struct definition
    let struct_str = format_struct(&struct_def);
    result.add_visible_content(struct_str);
    
    Ok(result.format())
}

/// Process enum! directive
fn process_enum_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let (parsed_file, enum_name, _, mut result) = setup_directive_processor(base_dir, directive)?;
    
    if enum_name.is_empty() {
        return Err(anyhow::anyhow!("Enum name is required"));
    }
    
    let enum_def = find_enum(&parsed_file, &enum_name)
        .with_context(|| format!("Enum '{}' not found", enum_name))?;
    
    // Add enum definition
    let enum_str = format_enum(&enum_def);
    result.add_visible_content(enum_str);
    
    Ok(result.format())
}

/// Process trait! directive
fn process_trait_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let (parsed_file, trait_name, _, mut result) = setup_directive_processor(base_dir, directive)?;
    
    if trait_name.is_empty() {
        return Err(anyhow::anyhow!("Trait name is required"));
    }
    
    let trait_def = find_trait(&parsed_file, &trait_name)
        .with_context(|| format!("Trait '{}' not found", trait_name))?;
    
    // Add trait definition
    let trait_str = format_trait(&trait_def);
    result.add_visible_content(trait_str);
    
    Ok(result.format())
}

/// Helper function to load a file and set up a directive processor
fn setup_directive_processor(base_dir: &Path, directive: &str) -> Result<(File, String, Option<Vec<String>>, ProcessedDirective)> {
    let (file_path, item_name, dependencies) = parse_directive_args(directive)?;
    
    let absolute_path = base_dir.join(file_path);
    let parsed_file = read_and_parse_file(&absolute_path)?;
    
    let mut result = ProcessedDirective::new();
    
    // Add imports
    for import in extract_imports(&parsed_file) {
        result.add_hidden_import(import);
    }
    
    // Process dependencies if provided
    if let Some(deps) = &dependencies {
        let (hidden_deps, visible_deps) = process_dependencies(&parsed_file, deps);
        
        for dep in hidden_deps {
            result.add_hidden_dependency(dep);
        }
        
        for dep in visible_deps {
            result.add_visible_content(dep);
        }
    }
    
    Ok((parsed_file, item_name.unwrap_or_default(), dependencies, result))
}

/// Process impl! directive
fn process_impl_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let (parsed_file, struct_name, _, mut result) = setup_directive_processor(base_dir, directive)?;
    
    if struct_name.is_empty() {
        return Err(anyhow::anyhow!("Struct name is required"));
    }
    
    let impl_def = find_struct_impl(&parsed_file, &struct_name)
        .with_context(|| format!("Implementation for struct '{}' not found", struct_name))?;
    
    // Add hidden struct definition if it exists
    if let Some(struct_def) = find_struct(&parsed_file, &struct_name) {
        let struct_str = format_struct(&struct_def);
        result.add_hidden_dependency(struct_str);
    }
    
    // Add impl definition
    let impl_str = format_impl(&impl_def);
    result.add_visible_content(impl_str);
    
    Ok(result.format())
}

/// Process trait_impl! directive
fn process_trait_impl_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let (parsed_file, item_name, _, mut result) = setup_directive_processor(base_dir, directive)?;
    
    // For trait_impl, the item_name should have the format "TraitName for StructName"
    let parts: Vec<&str> = item_name.split(" for ").collect();
    if parts.len() != 2 {
        return Err(anyhow::anyhow!("Trait and struct names are required in the format 'TraitName for StructName'"));
    }

    let trait_name = parts[0].trim();
    let struct_name = parts[1].trim();

    let impl_def = find_trait_impl(&parsed_file, trait_name, struct_name)
        .with_context(|| format!("Implementation of trait '{}' for struct '{}' not found", trait_name, struct_name))?;
    
    // Add hidden struct definition
    if let Some(struct_def) = find_struct(&parsed_file, struct_name) {
        let struct_str = format_struct(&struct_def);
        result.add_hidden_dependency(struct_str);
    }
    
    // Add hidden trait definition
    if let Some(trait_def) = find_trait(&parsed_file, trait_name) {
        let trait_str = format_trait(&trait_def);
        result.add_hidden_dependency(trait_str);
    }
    
    // Add trait impl definition
    let impl_str = format_impl(&impl_def);
    result.add_visible_content(impl_str);
    
    Ok(result.format())
}

/// Process function! directive
fn process_function_directive(base_dir: &Path, directive: &str) -> Result<String> {
    let (parsed_file, function_name, _, mut result) = setup_directive_processor(base_dir, directive)?;
    
    if function_name.is_empty() {
        return Err(anyhow::anyhow!("Function name is required"));
    }
    
    let function_def = find_function_or_method(&parsed_file, &function_name)
        .with_context(|| format!("Function '{}' not found", function_name))?;
    
    // Add function definition
    let fn_str = format_function(&function_def);
    result.add_visible_content(fn_str);
    
    Ok(result.format())
}

