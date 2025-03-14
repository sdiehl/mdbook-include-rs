use anyhow::{Context, Result};
use mdbook::book::{Book, BookItem};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use regex::{Captures, Regex};
use std::fs;
use std::path::Path;
use toml::Value;

/// Preprocessor that handles include-doc code blocks
pub struct IncludeDocPreprocessor;

impl Preprocessor for IncludeDocPreprocessor {
    fn name(&self) -> &str {
        "include-doc"
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


        book.for_each_mut(|item| {
            if let BookItem::Chapter(chapter) = item {
                // Get the directory of the chapter markdown file to use as the base if no global base_dir
                let base_dir = if let Some(ref global_dir) = global_base_dir {
                    global_dir.clone()
                } else if let Some(ref source_path) = chapter.source_path {
                    // The SUMMARY.md file is always in src

                    // Use the directory containing the markdown file as base
                    if let Some(parent) = source_path.parent() {
                        ctx.root.join(parent)
                    } else {
                        ctx.root.clone()
                    }.join("src")
                } else {
                    // Fallback to root if no source path
                    ctx.root.clone()
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

/// Process the markdown content to find and replace include-doc code blocks
pub fn process_markdown(base_dir: &Path, content: &mut String) -> Result<()> {
    // This regex finds our directives anywhere in the content
    let re = Regex::new(r"\{\{[\s]*(#(?:source_file|function_body)![\s\S]*?)[\s]*\}\}")?;

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

/// Process an include-doc directive
fn process_include_doc_directive(base_dir: &Path, directive: &str) -> Result<String> {
    // Find if it's a source_file or function_body directive
    if directive.starts_with("#source_file!") {
        process_source_file_directive(base_dir, directive)
    } else if directive.starts_with("#function_body!") {
        process_function_body_directive(base_dir, directive)
    } else {
        // Pass through unchanged if not recognized
        Ok(directive.to_string())
    }
}

/// Process a source_file directive
fn process_source_file_directive(base_dir: &Path, directive: &str) -> Result<String> {
    // Extract the file path from the directive
    // The pattern is: #source_file!("path/to/file.rs")
    // Using (?s) to enable DOTALL mode for handling multi-line input
    let re = Regex::new(r#"(?s)#source_file!\s*\(\s*"([^"]+)"\s*\)"#)?;

    if let Some(caps) = re.captures(directive) {
        let file_path = caps.get(1).map_or("", |m| m.as_str());
        let absolute_path = base_dir.join(file_path);

        // Check if file exists
        if !absolute_path.exists() {
            return Err(anyhow::anyhow!(
                "File not found: {}",
                absolute_path.display()
            ));
        }

        // Read the file content
        let file_content = fs::read_to_string(&absolute_path)
            .with_context(|| format!("Failed to read file: {}", absolute_path.display()))?;

        // Filter out 'use' statements
        let filtered_content = file_content
            .lines()
            .filter(|line| !line.trim_start().starts_with("use "))
            .collect::<Vec<_>>()
            .join("\n");

        Ok(filtered_content)
    } else {
        Err(anyhow::anyhow!(
            "Invalid source_file directive: {}",
            directive
        ))
    }
}

/// Process a function_body directive
fn process_function_body_directive(base_dir: &Path, directive: &str) -> Result<String> {
    // Extract the file path and function name from the directive
    // The pattern is: #function_body!("path/to/file.rs", function_name, [optional, dependencies])
    // Using (?s) to enable DOTALL mode for handling multi-line input
    let re = Regex::new(
        r#"(?s)#function_body!\s*\(\s*"([^"]+)"\s*,\s*([^,\]]+)(?:\s*,\s*\[(.*?)\])?\s*\)"#,
    )?;

    if let Some(caps) = re.captures(directive) {
        let file_path = caps.get(1).map_or("", |m| m.as_str());
        let function_name = caps.get(2).map_or("", |m| m.as_str()).trim();
        let dependencies = caps.get(3).map(|m| m.as_str().trim());

        let absolute_path = base_dir.join(file_path);

        // Check if file exists
        if !absolute_path.exists() {
            return Err(anyhow::anyhow!(
                "File not found: {}",
                absolute_path.display()
            ));
        }

        // Read the file content
        let file_content = fs::read_to_string(&absolute_path)
            .with_context(|| format!("Failed to read file: {}", absolute_path.display()))?;

        // Parse the file and extract the function body and any dependencies
        if let Some(deps) = dependencies {
            extract_function_with_dependencies(&file_content, function_name, deps)
        } else {
            extract_function_body(&file_content, function_name)
        }
    } else {
        Err(anyhow::anyhow!(
            "Invalid function_body directive: {}",
            directive
        ))
    }
}

/// Extract a function body from Rust code
fn extract_function_body(content: &str, function_name: &str) -> Result<String> {
    // Parse the Rust file
    let syntax = syn::parse_file(content)
        .map_err(|e| anyhow::anyhow!("Failed to parse Rust code: {}", e))?;

    // Helper function to extract function body using regex
    fn extract_body_with_regex(code: &str) -> Option<String> {
        if let Ok(re) = Regex::new(r"fn\s+[^{]+\{([\s\S]*)\}\s*$") {
            if let Some(caps) = re.captures(code) {
                if let Some(body) = caps.get(1) {
                    // Get the body and trim it
                    return Some(body.as_str().trim().to_string());
                }
            }
        }
        None
    }

    // Look for the function in top-level items
    for item in &syntax.items {
        if let syn::Item::Fn(item_fn) = item {
            if item_fn.sig.ident == function_name {
                // Generate full function code
                let fn_code = prettyplease::unparse(&syn::File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![syn::Item::Fn(item_fn.clone())],
                });

                // Try to extract just the body
                if let Some(body) = extract_body_with_regex(&fn_code) {
                    return Ok(body);
                }

                // Fallback: return the full function code
                let result = fn_code
                    .lines()
                    .filter(|line| !line.trim_start().starts_with("use "))
                    .collect::<Vec<_>>()
                    .join("\n");

                return Ok(result);
            }
        }
    }

    // Also look for methods inside impl blocks
    for item in &syntax.items {
        if let syn::Item::Impl(item_impl) = item {
            for impl_item in &item_impl.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if method.sig.ident == function_name {
                        // Create a standalone function from the method
                        let standalone_fn = syn::ItemFn {
                            attrs: method.attrs.clone(),
                            vis: syn::Visibility::Inherited,
                            sig: method.sig.clone(),
                            block: Box::new(method.block.clone()),
                        };

                        // Generate code for just this function
                        let fn_code = prettyplease::unparse(&syn::File {
                            shebang: None,
                            attrs: vec![],
                            items: vec![syn::Item::Fn(standalone_fn)],
                        });

                        // Try to extract just the body
                        if let Some(body) = extract_body_with_regex(&fn_code) {
                            return Ok(body);
                        }

                        // Fallback: return the function code
                        let result = fn_code
                            .lines()
                            .filter(|line| !line.trim_start().starts_with("use "))
                            .collect::<Vec<_>>()
                            .join("\n");

                        return Ok(result);
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!("Function '{}' not found", function_name))
}

/// Extract a function with its dependencies from Rust code
fn extract_function_with_dependencies(
    content: &str,
    function_name: &str,
    dependencies: &str,
) -> Result<String> {
    // Parse the Rust file
    let syntax = syn::parse_file(content)
        .map_err(|e| anyhow::anyhow!("Failed to parse Rust code: {}", e))?;

    // Split dependencies by comma and trim whitespace
    let deps: Vec<&str> = dependencies.split(',').map(|s| s.trim()).collect();

    // First collect all the code blocks we need (main function and dependencies)
    let mut main_function_code = String::new();
    let mut dependency_blocks: Vec<(String, String)> = Vec::new(); // (name, code)

    // First, try to find the main function/method
    let function_name = function_name.trim();

    // Handle method reference in format StructName::method_name
    if function_name.contains("::") {
        let parts: Vec<&str> = function_name.split("::").collect();
        if parts.len() == 2 {
            let struct_name = parts[0].trim();
            let method_name = parts[1].trim();

            let mut struct_method_code = String::new();
            if find_struct_method(&syntax, struct_name, method_name, &mut struct_method_code) {
                main_function_code = struct_method_code;
            } else {
                return Err(anyhow::anyhow!(
                    "Method '{}' for struct '{}' not found",
                    method_name,
                    struct_name
                ));
            }
        } else {
            return Err(anyhow::anyhow!(
                "Invalid function name format: {}",
                function_name
            ));
        }
    } else {
        // Try regular function or method
        let found_main_function =
            find_function_or_method(&syntax, function_name, &mut main_function_code)?;
        if !found_main_function {
            return Err(anyhow::anyhow!(
                "Main function '{}' not found",
                function_name
            ));
        }
    }

    // Now process each dependency
    for dep in deps {
        // Check if it's a struct dependency
        if dep.starts_with("struct ") {
            let struct_name = dep.trim_start_matches("struct ").trim();
            let mut struct_code = String::new();

            if find_struct(&syntax, struct_name, &mut struct_code) {
                dependency_blocks.push((format!("struct {}", struct_name), struct_code));
            } else {
                return Err(anyhow::anyhow!("Struct '{}' not found", struct_name));
            }
        }
        // Check if it's a trait dependency
        else if dep.starts_with("trait ") {
            let trait_name = dep.trim_start_matches("trait ").trim();
            let mut trait_code = String::new();

            if find_trait(&syntax, trait_name, &mut trait_code) {
                dependency_blocks.push((format!("trait {}", trait_name), trait_code));
            } else {
                return Err(anyhow::anyhow!("Trait '{}' not found", trait_name));
            }
        }
        // Check if it's an impl dependency
        else if dep.starts_with("impl ") {
            let impl_info = dep.trim_start_matches("impl ").trim();
            let mut impl_code = String::new();

            // Handle both regular impl and trait impl
            if impl_info.contains(" for ") {
                // Format is "impl TraitName for StructName"
                let parts: Vec<&str> = impl_info.split(" for ").collect();
                if parts.len() == 2 {
                    let trait_name = parts[0].trim();
                    let struct_name = parts[1].trim();

                    if find_trait_impl(&syntax, trait_name, struct_name, &mut impl_code) {
                        dependency_blocks.push((
                            format!("impl {} for {}", trait_name, struct_name),
                            impl_code,
                        ));
                    } else {
                        return Err(anyhow::anyhow!(
                            "Impl of trait '{}' for '{}' not found",
                            trait_name,
                            struct_name
                        ));
                    }
                }
            } else if impl_info.contains("::") {
                // Format is "StructName::method_name" - looking for a specific method
                let parts: Vec<&str> = impl_info.split("::").collect();
                if parts.len() == 2 {
                    let struct_name = parts[0].trim();
                    let method_name = parts[1].trim();

                    if find_struct_method(&syntax, struct_name, method_name, &mut impl_code) {
                        dependency_blocks
                            .push((format!("impl {}::{}", struct_name, method_name), impl_code));
                    } else {
                        return Err(anyhow::anyhow!(
                            "Method '{}' for struct '{}' not found",
                            method_name,
                            struct_name
                        ));
                    }
                }
            } else {
                // Format is "StructName" - all methods
                let struct_name = impl_info;

                if find_struct_impl(&syntax, struct_name, &mut impl_code) {
                    dependency_blocks.push((format!("impl {}", struct_name), impl_code));
                } else {
                    return Err(anyhow::anyhow!("Impl for '{}' not found", struct_name));
                }
            }
        }
        // Otherwise assume it's a function
        else {
            let mut fn_code = String::new();
            if find_function_or_method(&syntax, dep, &mut fn_code)? {
                dependency_blocks.push((dep.to_string(), fn_code));
            } else {
                return Err(anyhow::anyhow!("Function/method '{}' not found", dep));
            }
        }
    }

    // Combine the results in the requested order
    let mut result = String::new();

    // First all dependencies
    let total_deps = dependency_blocks.len();
    for (i, (_, code)) in dependency_blocks.iter().enumerate() {
        result.push_str(code);
        if i < total_deps - 1 || total_deps > 0 {
            // Add exactly one blank line between dependencies
            result.push_str("\n\n");
        }
    }

    // For the main function, just get the function body (handled by extract_function_body for regular case)
    // For methods with impl blocks, we need to extract the function body from the impl block
    if main_function_code.contains("impl") {
        // This is an impl block with a method
        // Extract just the function body from an impl method
        let re = Regex::new(r"fn\s+[^{]+\{([\s\S]*)\}\s*\}")?;
        if let Some(caps) = re.captures(&main_function_code) {
            if let Some(body) = caps.get(1) {
                // Get the body and remove any initial indentation
                let body_text = body.as_str().trim();
                result.push_str(body_text);
            } else {
                // Fallback to the original code
                result.push_str(&main_function_code);
            }
        } else {
            // If regex failed, fallback to the original code
            result.push_str(&main_function_code);
        }
    } else {
        // For regular functions, the body is already extracted
        result.push_str(&main_function_code);
    }

    Ok(result.trim().to_string())
}

// Helper functions for extracting different Rust code elements

fn find_function_or_method(syntax: &syn::File, name: &str, output: &mut String) -> Result<bool> {
    // Check top-level functions
    for item in &syntax.items {
        if let syn::Item::Fn(item_fn) = item {
            if item_fn.sig.ident == name {
                // Generate code for just this function without use statements
                let fn_code = prettyplease::unparse(&syn::File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![syn::Item::Fn(item_fn.clone())],
                });

                *output = fn_code
                    .lines()
                    .filter(|line| !line.trim_start().starts_with("use "))
                    .collect::<Vec<_>>()
                    .join("\n");

                return Ok(true);
            }
        }
    }

    // Check methods in impl blocks
    for item in &syntax.items {
        if let syn::Item::Impl(item_impl) = item {
            for impl_item in &item_impl.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if method.sig.ident == name {
                        // For methods, we try to keep some context by showing type path
                        let type_name = if let Some((_, path, _)) = &item_impl.trait_ {
                            format!("<impl {} for ", path.segments.last().unwrap().ident)
                        } else {
                            "impl ".to_string()
                        };

                        let type_path = if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                            format!(
                                "{}{}",
                                type_name,
                                type_path.path.segments.last().unwrap().ident
                            )
                        } else {
                            "Unknown".to_string()
                        };

                        // Create a standalone function from the method
                        let standalone_fn = syn::ItemFn {
                            attrs: method.attrs.clone(),
                            vis: syn::Visibility::Inherited,
                            sig: method.sig.clone(),
                            block: Box::new(method.block.clone()),
                        };

                        let fn_code = prettyplease::unparse(&syn::File {
                            shebang: None,
                            attrs: vec![],
                            items: vec![syn::Item::Fn(standalone_fn)],
                        });

                        // Add a comment showing where this method comes from
                        *output = format!("// From {}\n{}", type_path, fn_code);

                        return Ok(true);
                    }
                }
            }
        }
    }

    Ok(false)
}

fn find_struct(syntax: &syn::File, name: &str, output: &mut String) -> bool {
    for item in &syntax.items {
        if let syn::Item::Struct(item_struct) = item {
            if item_struct.ident == name {
                let struct_code = prettyplease::unparse(&syn::File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![syn::Item::Struct(item_struct.clone())],
                });

                // Remove trailing newlines to ensure consistent formatting
                *output = struct_code.trim_end().to_string();
                return true;
            }
        }
    }
    false
}

fn find_trait(syntax: &syn::File, name: &str, output: &mut String) -> bool {
    for item in &syntax.items {
        if let syn::Item::Trait(item_trait) = item {
            if item_trait.ident == name {
                let trait_code = prettyplease::unparse(&syn::File {
                    shebang: None,
                    attrs: vec![],
                    items: vec![syn::Item::Trait(item_trait.clone())],
                });

                // Remove trailing newlines to ensure consistent formatting
                *output = trait_code.trim_end().to_string();
                return true;
            }
        }
    }
    false
}

fn find_struct_impl(syntax: &syn::File, name: &str, output: &mut String) -> bool {
    for item in &syntax.items {
        if let syn::Item::Impl(item_impl) = item {
            // Check if this is an impl for the specified struct (and not a trait impl)
            if item_impl.trait_.is_none() {
                if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                    if type_path.path.segments.last().unwrap().ident == name {
                        let impl_code = prettyplease::unparse(&syn::File {
                            shebang: None,
                            attrs: vec![],
                            items: vec![syn::Item::Impl(item_impl.clone())],
                        });

                        // Remove trailing newlines to ensure consistent formatting
                        *output = impl_code.trim_end().to_string();
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn find_struct_method(
    syntax: &syn::File,
    struct_name: &str,
    method_name: &str,
    output: &mut String,
) -> bool {
    for item in &syntax.items {
        if let syn::Item::Impl(item_impl) = item {
            // Check if this is an impl for the specified struct
            if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                if type_path.path.segments.last().unwrap().ident == struct_name {
                    // Find the specific method
                    for impl_item in &item_impl.items {
                        if let syn::ImplItem::Fn(method) = impl_item {
                            if method.sig.ident == method_name {
                                // Create a special impl block with just this method
                                let mut method_impl = item_impl.clone();
                                method_impl.items = vec![syn::ImplItem::Fn(method.clone())];

                                let impl_code = prettyplease::unparse(&syn::File {
                                    shebang: None,
                                    attrs: vec![],
                                    items: vec![syn::Item::Impl(method_impl)],
                                });

                                // Remove trailing newlines to ensure consistent formatting
                                *output = impl_code.trim_end().to_string();
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }
    false
}

fn find_trait_impl(
    syntax: &syn::File,
    trait_name: &str,
    struct_name: &str,
    output: &mut String,
) -> bool {
    for item in &syntax.items {
        if let syn::Item::Impl(item_impl) = item {
            // Check if this is an impl of the specified trait for the specified struct
            if let Some((_, path, _)) = &item_impl.trait_ {
                if path.segments.last().unwrap().ident == trait_name {
                    if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                        if type_path.path.segments.last().unwrap().ident == struct_name {
                            let impl_code = prettyplease::unparse(&syn::File {
                                shebang: None,
                                attrs: vec![],
                                items: vec![syn::Item::Impl(item_impl.clone())],
                            });

                            // Remove trailing newlines to ensure consistent formatting
                            *output = impl_code.trim_end().to_string();
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}
