use syn::{
    File, ItemFn,
    visit::{self, Visit},
};

/// Find a function in a parsed Rust file
pub(crate) fn find_function(parsed_file: &File, function_name: &str) -> Option<ItemFn> {
    let mut finder = FunctionFinder::new(function_name);
    finder.visit_file(parsed_file);
    finder.function_item
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
