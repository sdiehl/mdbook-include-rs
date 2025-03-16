use syn::{File, ItemStruct, visit::{self, Visit}};

/// Find a struct in a parsed Rust file
pub fn find_struct(parsed_file: &File, struct_name: &str) -> Option<ItemStruct> {
    let mut finder = StructFinder::new(struct_name);
    finder.visit_file(parsed_file);
    finder.struct_item
}

/// A visitor that finds a struct by name
pub struct StructFinder {
    struct_name: String,
    struct_item: Option<ItemStruct>,
}

impl StructFinder {
    pub fn new(struct_name: &str) -> Self {
        Self {
            struct_name: struct_name.to_string(),
            struct_item: None,
        }
    }
}

impl<'ast> Visit<'ast> for StructFinder {
    fn visit_item_struct(&mut self, item_struct: &'ast ItemStruct) {
        if item_struct.ident == self.struct_name {
            self.struct_item = Some(item_struct.clone());
        }
        
        // Continue visiting
        visit::visit_item_struct(self, item_struct);
    }
}