use syn::{File, ItemTrait, visit::{self, Visit}};

/// Find a trait in a parsed Rust file
pub fn find_trait(parsed_file: &File, trait_name: &str) -> Option<ItemTrait> {
    let mut finder = TraitFinder::new(trait_name);
    finder.visit_file(parsed_file);
    finder.trait_item
}

/// A visitor that finds a trait by name
pub struct TraitFinder {
    trait_name: String,
    trait_item: Option<ItemTrait>,
}

impl TraitFinder {
    pub fn new(trait_name: &str) -> Self {
        Self {
            trait_name: trait_name.to_string(),
            trait_item: None,
        }
    }
}

impl<'ast> Visit<'ast> for TraitFinder {
    fn visit_item_trait(&mut self, item_trait: &'ast ItemTrait) {
        if item_trait.ident == self.trait_name {
            self.trait_item = Some(item_trait.clone());
        }
        
        // Continue visiting
        visit::visit_item_trait(self, item_trait);
    }
}