use insta::assert_snapshot;
use mdbook::Config;
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook_include_rs::IncludeRsPreprocessor;
use std::path::PathBuf;

#[test]
fn test_empty() {
    // Create a simple book with a single chapter
    let mut book = Book::new();
    let chapter = Chapter {
        name: "Chapter 1".to_string(),
        content: "Some content".to_string(),
        number: None,
        sub_items: vec![],
        path: Some(PathBuf::from("chapter_1.md")),
        source_path: Some(PathBuf::from("chapter_1.md")),
        parent_names: vec![],
    };
    book.push_item(BookItem::Chapter(chapter));

    // Create a preprocessor context
    let ctx = create_test_context();

    // Run the preprocessor
    let preprocessor = IncludeRsPreprocessor;
    let processed_book = preprocessor.run(&ctx, book).unwrap();

    // Since there are no include-doc snippets, the book should remain unchanged
    let mut chapter_found = false;
    for item in processed_book.iter() {
        if let BookItem::Chapter(chapter) = item {
            if chapter.name == "Chapter 1" {
                assert_eq!(chapter.content, "Some content");
                chapter_found = true;
            }
        }
    }
    assert!(chapter_found, "Chapter not found in processed book");
}

/// Create a test book with a single chapter containing the given content
fn create_test_book(chapter_name: &str, content: &str, chapter_path: &str) -> Book {
    let mut book = Book::new();
    let chapter = Chapter {
        name: chapter_name.to_string(),
        content: content.to_string(),
        number: None,
        sub_items: vec![],
        path: Some(PathBuf::from(chapter_path)),
        source_path: Some(PathBuf::from(chapter_path)),
        parent_names: vec![],
    };
    book.push_item(BookItem::Chapter(chapter));
    book
}

/// Run the preprocessor on a book and return the processed content of the specified chapter
fn run_and_extract_content(book: Book, chapter_name: &str) -> String {
    // Create a preprocessor context
    let ctx = create_test_context();

    // Run the preprocessor
    let preprocessor = IncludeRsPreprocessor;
    let processed_book = preprocessor.run(&ctx, book).unwrap();

    // Extract the processed content
    let mut processed_content = String::new();
    for item in processed_book.iter() {
        if let BookItem::Chapter(chapter) = item {
            if chapter.name == chapter_name {
                processed_content = chapter.content.clone();
                break;
            }
        }
    }
    processed_content
}

/// Helper function to run a directive test and perform snapshot testing
fn test_directive(
    directive_name: &str,
    directive_content: &str,
    chapter_name: &str,
    preamble: &str,
) {
    let content = format!(
        "{}\n```rust\n{}\n```\nafter {}",
        preamble, directive_content, preamble
    );
    let book = create_test_book(chapter_name, &content, "chapter_1.md");
    let processed_content = run_and_extract_content(book, chapter_name);
    assert_snapshot!(directive_name, processed_content);
}

#[test]
fn test_source_file() {
    test_directive(
        "source_file",
        "#![source_file!(\"../test_file.rs\")]",
        "Chapter 1",
        "Some preamble",
    );
}

#[test]
fn test_function_body() {
    test_directive(
        "function_body",
        "#![function_body!(\"../test_file.rs\", free_function)]",
        "Chapter 1",
        "some preamble",
    );
}

#[test]
fn test_complex_function_body() {
    test_directive(
        "complex_function_body",
        "#![function_body!(\"../test_file.rs\", free_function, [struct TestStruct, impl TestStruct, trait TestTrait, impl TestTrait for TestStruct, enum TestEnum])]",
        "Chapter 1",
        "some preamble",
    );
}

#[test]
fn test_struct() {
    test_directive(
        "struct",
        "#![struct!(\"../test_file.rs\", TestStruct)]",
        "Chapter 1",
        "struct preamble",
    );
}

#[test]
fn test_enum() {
    test_directive(
        "enum",
        "#![enum!(\"../test_file.rs\", TestEnum)]",
        "Chapter 1",
        "enum preamble",
    );
}

#[test]
fn test_trait() {
    test_directive(
        "trait",
        "#![trait!(\"../test_file.rs\", TestTrait)]",
        "Chapter 1",
        "trait preamble",
    );
}

#[test]
fn test_impl() {
    test_directive(
        "impl",
        "#![impl!(\"../test_file.rs\", TestStruct)]",
        "Chapter 1",
        "impl preamble",
    );
}

#[test]
fn test_trait_impl() {
    test_directive(
        "trait_impl",
        "#![trait_impl!(\"../test_file.rs\", TestTrait for TestStruct)]",
        "Chapter 1",
        "trait impl preamble",
    );
}

#[test]
fn test_function() {
    test_directive(
        "function",
        "#![function!(\"../test_file.rs\", free_function)]",
        "Chapter 1",
        "function preamble",
    );
}

#[test]
fn test_relative_path_with_source_path() {
    test_directive(
        "relative_path",
        "#![source_file!(\"../test_file.rs\")]",
        "Relative Path Test",
        "relative path preamble",
    );
}

// Create a mock PreprocessorContext for testing
fn create_test_context() -> PreprocessorContext {
    let mut config = Config::default();
    config.set("book.title", "Test Book").unwrap();

    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixtures_dir = PathBuf::from(manifest_dir).join("tests").join("fixtures");
    // Use a test-specific approach since PreprocessorContext has private fields
    let ctx_json = format!(
        r#"{{
            "root": "{root}",
            "config": {config},
            "renderer": "html",
            "mdbook_version": "0.4.47"
        }}"#,
        root = fixtures_dir.display(),
        config = serde_json::to_string(&config).unwrap()
    );

    serde_json::from_str(&ctx_json).unwrap()
}
