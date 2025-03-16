use syn::{Block, File, Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait};

/// Format an item as a string
pub fn format_item(item: &Item) -> String {
    // Create a file with just this item
    let file = File {
        shebang: None,
        attrs: vec![],
        items: vec![item.clone()],
    };
    
    // Format the file
    let formatted = prettyplease::unparse(&file);
    
    // Extract the item part (skip any file-level metadata)
    formatted
}

/// Format a struct item as a string
pub fn format_struct(item: &ItemStruct) -> String {
    format_item(&Item::Struct(item.clone()))
}

/// Format an enum item as a string
pub fn format_enum(item: &ItemEnum) -> String {
    format_item(&Item::Enum(item.clone()))
}

/// Format a trait item as a string
pub fn format_trait(item: &ItemTrait) -> String {
    format_item(&Item::Trait(item.clone()))
}

/// Format an impl item as a string
pub fn format_impl(item: &ItemImpl) -> String {
    format_item(&Item::Impl(item.clone()))
}

/// Format a function item as a string
pub fn format_function(item: &ItemFn) -> String {
    format_item(&Item::Fn(item.clone()))
}

/// Format a function body (the contents between { }) as a string
pub fn format_function_body(block_content: &Block) -> String {
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
    lines.remove(0);
    lines.remove(lines.len() - 1);
    // Remove the `    ` from the start of each line
    for i in 0..lines.len() {
        lines[i] = &lines[i][4..];
    }
    lines.join("\n")

}

/// Format content with a # prefix for hidden code
pub fn format_hidden(content: &str) -> String {
    let mut result = String::new();
    for line in content.lines() {
        if line.trim().is_empty() {
            result.push_str("#\n");
        } else {
            result.push_str(&format!("#{}\n", line));
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