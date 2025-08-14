use syn::{
    File, ImplItem, ImplItemFn, ItemImpl,
    visit::{self, Visit},
};

/// Find a method in a parsed Rust file by searching through impl blocks
pub(crate) fn find_method(parsed_file: &File, method_spec: &str) -> Option<ImplItemFn> {
    // Parse method specification: "StructName::method_name" or "TraitName::method_name for StructName"
    if let Some((type_part, method_name)) = method_spec.rsplit_once("::") {
        if type_part.contains(" for ") {
            // Handle trait impl methods: "TraitName for StructName::method_name"
            if let Some((trait_name, struct_name)) = type_part.split_once(" for ") {
                let mut finder =
                    TraitMethodFinder::new(trait_name.trim(), struct_name.trim(), method_name);
                finder.visit_file(parsed_file);
                return finder.method_item;
            }
        } else {
            // Handle struct impl methods: "StructName::method_name"
            let mut finder = StructMethodFinder::new(type_part, method_name);
            finder.visit_file(parsed_file);
            return finder.method_item;
        }
    }
    None
}

/// A visitor that finds a method in a struct implementation by struct and method name
struct StructMethodFinder {
    struct_name: String,
    method_name: String,
    method_item: Option<ImplItemFn>,
}

impl StructMethodFinder {
    pub fn new(struct_name: &str, method_name: &str) -> Self {
        Self {
            struct_name: struct_name.to_string(),
            method_name: method_name.to_string(),
            method_item: None,
        }
    }

    fn matches_struct_impl(&self, item_impl: &ItemImpl) -> bool {
        // Check if this is a struct implementation (not a trait implementation)
        if item_impl.trait_.is_some() {
            return false;
        }

        // Check if the self type matches our struct name
        if let syn::Type::Path(type_path) = &*item_impl.self_ty {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == self.struct_name;
            }
        }

        false
    }
}

impl<'ast> Visit<'ast> for StructMethodFinder {
    fn visit_item_impl(&mut self, item_impl: &'ast ItemImpl) {
        if self.matches_struct_impl(item_impl) {
            // Look for the method in this impl block
            for impl_item in &item_impl.items {
                if let ImplItem::Fn(method) = impl_item {
                    if method.sig.ident == self.method_name {
                        self.method_item = Some(method.clone());
                        return;
                    }
                }
            }
        }

        // Continue visiting
        visit::visit_item_impl(self, item_impl);
    }
}

/// A visitor that finds a method in a trait implementation
struct TraitMethodFinder {
    trait_name: String,
    struct_name: String,
    method_name: String,
    method_item: Option<ImplItemFn>,
}

impl TraitMethodFinder {
    pub fn new(trait_name: &str, struct_name: &str, method_name: &str) -> Self {
        Self {
            trait_name: trait_name.to_string(),
            struct_name: struct_name.to_string(),
            method_name: method_name.to_string(),
            method_item: None,
        }
    }

    fn matches_trait_impl(&self, item_impl: &ItemImpl) -> bool {
        // Check if this is a trait implementation
        if let Some((_, trait_path, _)) = &item_impl.trait_ {
            // Check trait name
            if let Some(segment) = trait_path.segments.last() {
                if segment.ident != self.trait_name {
                    return false;
                }
            } else {
                return false;
            }

            // Check struct name
            if let syn::Type::Path(type_path) = &*item_impl.self_ty {
                if let Some(segment) = type_path.path.segments.last() {
                    return segment.ident == self.struct_name;
                }
            }
        }

        false
    }
}

impl<'ast> Visit<'ast> for TraitMethodFinder {
    fn visit_item_impl(&mut self, item_impl: &'ast ItemImpl) {
        if self.matches_trait_impl(item_impl) {
            // Look for the method in this impl block
            for impl_item in &item_impl.items {
                if let ImplItem::Fn(method) = impl_item {
                    if method.sig.ident == self.method_name {
                        self.method_item = Some(method.clone());
                        return;
                    }
                }
            }
        }

        // Continue visiting
        visit::visit_item_impl(self, item_impl);
    }
}
