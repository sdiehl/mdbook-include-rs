use syn::{File, ItemEnum, visit::{self, Visit}};

/// Find an enum in a parsed Rust file
pub fn find_enum(parsed_file: &File, enum_name: &str) -> Option<ItemEnum> {
    let mut finder = EnumFinder::new(enum_name);
    finder.visit_file(parsed_file);
    finder.enum_item
}

/// A visitor that finds an enum by name
pub struct EnumFinder {
    enum_name: String,
    enum_item: Option<ItemEnum>,
}

impl EnumFinder {
    pub fn new(enum_name: &str) -> Self {
        Self {
            enum_name: enum_name.to_string(),
            enum_item: None,
        }
    }
}

impl<'ast> Visit<'ast> for EnumFinder {
    fn visit_item_enum(&mut self, item_enum: &'ast ItemEnum) {
        if item_enum.ident == self.enum_name {
            self.enum_item = Some(item_enum.clone());
        }
        
        // Continue visiting
        visit::visit_item_enum(self, item_enum);
    }
}