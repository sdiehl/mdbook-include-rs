use syn::spanned::Spanned;
use syn::{ImplItemFn, Item};

/// Format an item as a string
pub fn format_item(item: &Item) -> String {
    // Create a file with just this item
    item.span()
        .source_text()
        .expect("Failed to get source text")
}

/// Format a function body as a string
/// It will always replace the function name with `main`
/// It will always prefix the first and last lines with `# `
/// If the body has the comments:
/// * `// DISPLAY START` - This line and any before are prefixed with `# `
/// * `// DISPLAY END` - This line and any after are prefixed with `# `
pub(crate) fn format_function_body(fn_item: &Item) -> String {
    if matches!(fn_item, Item::Fn { .. }) {
        let source_text = fn_item
            .span()
            .source_text()
            .expect("Failed to get source text");
        let mut lines = source_text.split("\n").collect::<Vec<_>>();
        if lines.len() == 1 {
            return String::new();
        }
        lines[0] = "fn main() {\n";

        // Process display markers
        let mut result = String::new();
        let mut display_started = false;
        let mut display_ended = false;

        // Check if display markers exist
        let has_display_start = lines.iter().any(|line| line.trim() == "// DISPLAY START");
        let has_display_end = lines.iter().any(|line| line.trim() == "// DISPLAY END");

        // Skip the function signature and closing brace
        for (i, line) in lines.iter().enumerate() {
            // Skip the first and last line (fn main() and closing brace)
            if i == 0 || i == lines.len() - 1 {
                result.push_str(&format!("# {}\n", line.trim()));
                continue;
            }

            let trimmed_line = if line.len() >= 4 { &line[4..] } else { line };

            if trimmed_line.trim() == "// DISPLAY START" {
                display_started = true;
                continue; // Skip the DISPLAY START line itself
            } else if trimmed_line.trim() == "// DISPLAY END" {
                display_ended = true;
                continue; // Skip the DISPLAY END line itself
            }

            let should_hide =
                (has_display_start && !display_started) || (has_display_end && display_ended);

            if should_hide {
                // Add as hidden line
                if trimmed_line.trim().is_empty() {
                    result.push_str("# \n");
                } else {
                    result.push_str(&format!("# {}\n", trimmed_line));
                }
            } else {
                // Add as visible line
                result.push_str(&format!("{}\n", trimmed_line));
            }
        }

        // Remove trailing newline if present
        if result.ends_with('\n') {
            result.pop();
        }

        result
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

/// Format a method as a string
pub fn format_method(method: &ImplItemFn) -> String {
    method
        .span()
        .source_text()
        .expect("Failed to get source text")
}

/// Format a method body as a string, similar to format_function_body
pub fn format_method_body(method: &ImplItemFn) -> String {
    let source_text = method
        .span()
        .source_text()
        .expect("Failed to get source text");
    let mut lines = source_text.split("\n").collect::<Vec<_>>();
    if lines.len() == 1 {
        return String::new();
    }
    lines[0] = "fn main() {\n";

    // Process display markers
    let mut result = String::new();
    let mut display_started = false;
    let mut display_ended = false;

    // Check if display markers exist
    let has_display_start = lines.iter().any(|line| line.trim() == "// DISPLAY START");
    let has_display_end = lines.iter().any(|line| line.trim() == "// DISPLAY END");

    // Skip the function signature and closing brace
    for (i, line) in lines.iter().enumerate() {
        // Skip the first and last line (fn main() and closing brace)
        if i == 0 || i == lines.len() - 1 {
            result.push_str(&format!("# {}\n", line.trim()));
            continue;
        }

        let trimmed_line = if line.len() >= 4 { &line[4..] } else { line };

        if trimmed_line.trim() == "// DISPLAY START" {
            display_started = true;
            continue; // Skip the DISPLAY START line itself
        } else if trimmed_line.trim() == "// DISPLAY END" {
            display_ended = true;
            continue; // Skip the DISPLAY END line itself
        }

        let should_hide =
            (has_display_start && !display_started) || (has_display_end && display_ended);

        if should_hide {
            // Add as hidden line
            if trimmed_line.trim().is_empty() {
                result.push_str("# \n");
            } else {
                result.push_str(&format!("# {}\n", trimmed_line));
            }
        } else {
            // Add as visible line
            result.push_str(&format!("{}\n", trimmed_line));
        }
    }

    // Remove trailing newline if present
    if result.ends_with('\n') {
        result.pop();
    }

    result
}
