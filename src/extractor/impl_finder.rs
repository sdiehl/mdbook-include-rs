use syn::{
    File, ItemImpl, Path, Type,
    visit::{self, Visit},
};

/// Find a struct implementation in a parsed Rust file
pub(crate) fn find_struct_impl(parsed_file: &File, struct_name: &str) -> Option<ItemImpl> {
    let mut finder = StructImplFinder::new(struct_name);
    finder.visit_file(parsed_file);
    finder.impl_item
}

/// Find a trait implementation for a struct in a parsed Rust file
pub(crate) fn find_trait_impl(
    parsed_file: &File,
    trait_name: &str,
    struct_name: &str,
) -> Option<ItemImpl> {
    let mut finder = TraitImplFinder::new(trait_name, struct_name);
    finder.visit_file(parsed_file);
    finder.impl_item
}

/// A visitor that finds a struct implementation by struct name
struct StructImplFinder {
    struct_name: String,
    impl_item: Option<ItemImpl>,
}

impl StructImplFinder {
    pub fn new(struct_name: &str) -> Self {
        Self {
            struct_name: struct_name.to_string(),
            impl_item: None,
        }
    }

    fn get_type_path<'a>(&self, ty: &'a Type) -> Option<&'a Path> {
        if let Type::Path(type_path) = ty {
            Some(&type_path.path)
        } else {
            None
        }
    }
}

impl<'ast> Visit<'ast> for StructImplFinder {
    fn visit_item_impl(&mut self, item_impl: &'ast ItemImpl) {
        // Check if this is a struct implementation (not a trait implementation)
        if item_impl.trait_.is_none() {
            if let Some(path) = self.get_type_path(&item_impl.self_ty) {
                if path
                    .segments
                    .last().is_some_and(|seg| seg.ident == self.struct_name)
                {
                    self.impl_item = Some(item_impl.clone());
                }
            }
        }

        // Continue visiting
        visit::visit_item_impl(self, item_impl);
    }
}

/// A visitor that finds a trait implementation for a struct
pub struct TraitImplFinder {
    trait_name: String,
    struct_name: String,
    impl_item: Option<ItemImpl>,
}

impl TraitImplFinder {
    pub fn new(trait_name: &str, struct_name: &str) -> Self {
        Self {
            trait_name: trait_name.to_string(),
            struct_name: struct_name.to_string(),
            impl_item: None,
        }
    }

    fn get_type_path<'a>(&self, ty: &'a Type) -> Option<&'a Path> {
        if let Type::Path(type_path) = ty {
            Some(&type_path.path)
        } else {
            None
        }
    }
}

impl<'ast> Visit<'ast> for TraitImplFinder {
    fn visit_item_impl(&mut self, item_impl: &'ast ItemImpl) {
        // Check if this is a trait implementation
        if let Some((_, trait_path, _)) = &item_impl.trait_ {
            if trait_path
                .segments
                .last().is_some_and(|seg| seg.ident == self.trait_name)
            {
                if let Some(path) = self.get_type_path(&item_impl.self_ty) {
                    if path
                        .segments
                        .last().is_some_and(|seg| seg.ident == self.struct_name)
                    {
                        self.impl_item = Some(item_impl.clone());
                    }
                }
            }
        }

        // Continue visiting
        visit::visit_item_impl(self, item_impl);
    }
}
