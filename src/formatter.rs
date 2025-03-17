use syn::{File, Item};

/// Format an item as a string
pub fn format_item(item: &Item) -> String {
    // Create a file with just this item
    let file = File {
        shebang: None,
        attrs: vec![],
        items: vec![item.clone()],
    };

    // Extract the item part (skip any file-level metadata)
    prettyplease::unparse(&file)
}

/// Format a function body (the contents between { }) as a string
pub fn format_function_body(fn_item: &Item) -> String {
    if let Item::Fn(fn_def) = fn_item {
        let block_content = &fn_def.block;
        let file = syn::File {
            shebang: None,
            attrs: vec![],
            items: vec![syn::parse_quote! {
                fn tmp() #block_content

            }],
        };

        // Format the file
        let formatted = prettyplease::unparse(&file);
        let mut lines = formatted.lines().collect::<Vec<_>>();
        if lines.len() == 1 {
            return String::new();
        }
        lines.remove(0);
        lines.remove(lines.len() - 1);
        // Remove the `    ` from the start of each line
        for line in &mut lines {
            *line = &line[4..];
        }
        lines.join("\n")
    } else {
        panic!("Expected Item::Fn, got {:?}", fn_item);
    }
}

/// Format content with a # prefix for hidden code
pub fn format_hidden(content: &str) -> String {
    let mut result = String::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            result.push_str("# \n");
        } else {
            result.push_str(&format!("# {}\n", line));
        }
    }
    result
}

/// Format content without a prefix for visible code
pub fn format_visible(content: &str) -> String {
    let mut result = String::new();
    for line in content.lines() {
        result.push_str(&format!("{}\n", line));
    }
    result
}

/// Format a source file with specified visible items
pub fn format_source_file(file: &File) -> String {
    // Get the full file content
    prettyplease::unparse(file)
}
