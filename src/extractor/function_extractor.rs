use syn::{visit::{self, Visit}, Block, File, ItemFn};

/// Extract function body from a parsed Rust file
pub fn extract_function_body(parsed_file: &File, function_name: &str) -> Option<Block> {
    let mut extractor = FunctionExtractor::new(function_name);
    extractor.visit_file(parsed_file);
    extractor.function_body
}

/// Find a function in a parsed Rust file
pub fn find_function(parsed_file: &File, function_name: &str) -> Option<ItemFn> {
    let mut finder = FunctionFinder::new(function_name);
    finder.visit_file(parsed_file);
    finder.function_item
}

/// A visitor that extracts a function's body by name
pub struct FunctionExtractor {
    function_name: String,
    function_body: Option<Block>,
}

impl FunctionExtractor {
    pub fn new(function_name: &str) -> Self {
        Self {
            function_name: function_name.to_string(),
            function_body: None,
        }
    }
}

impl<'ast> Visit<'ast> for FunctionExtractor {
    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        if item_fn.sig.ident == self.function_name {
            // Extract the function body content
            self.function_body = Some((*item_fn.block).clone());
        }
        
        // Continue visiting
        visit::visit_item_fn(self, item_fn);
    }

    fn visit_impl_item_fn(&mut self, item_fn: &'ast syn::ImplItemFn) {
        // For method extraction (when a full path like StructName::method_name is given)
        let parts: Vec<&str> = self.function_name.split("::").collect();
        if parts.len() == 2 && item_fn.sig.ident == parts[1] {
            // Extract the method body content
            self.function_body = Some(item_fn.block.clone());
        }
        
        // Continue visiting
        visit::visit_impl_item_fn(self, item_fn);
    }
}

/// A visitor that finds a function by name
pub struct FunctionFinder {
    function_name: String,
    function_item: Option<ItemFn>,
}

impl FunctionFinder {
    pub fn new(function_name: &str) -> Self {
        Self {
            function_name: function_name.to_string(),
            function_item: None,
        }
    }
}

impl<'ast> Visit<'ast> for FunctionFinder {
    fn visit_item_fn(&mut self, item_fn: &'ast ItemFn) {
        if item_fn.sig.ident == self.function_name {
            self.function_item = Some(item_fn.clone());
        }
        
        // Continue visiting
        visit::visit_item_fn(self, item_fn);
    }
}