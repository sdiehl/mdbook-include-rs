use insta::assert_snapshot;
use mdbook::Config;
use mdbook::book::{Book, BookItem, Chapter};
use mdbook::preprocess::{Preprocessor, PreprocessorContext};
use mdbook_include_rs::IncludeDocPreprocessor;
use std::path::PathBuf;

#[test]
fn test_empty_preprocessor() {
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
    let preprocessor = IncludeDocPreprocessor;
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

#[test]
fn test_source_file_preprocessor() {
    // Create a simple book with a chapter containing an include-doc directive
    let mut book = Book::new();
    let chapter = Chapter {
        name: "Chapter 1".to_string(),
        content: "Some preamble\n```rust\n#![source_file!(\"../test_file.rs\")]\n```\n after"
            .to_string(),
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
    let preprocessor = IncludeDocPreprocessor;
    let processed_book = preprocessor.run(&ctx, book).unwrap();

    // Extract the processed content for snapshot testing
    let mut processed_content = String::new();
    for item in processed_book.iter() {
        if let BookItem::Chapter(chapter) = item {
            if chapter.name == "Chapter 1" {
                processed_content = chapter.content.clone();
                break;
            }
        }
    }

    // Use insta to snapshot test the result
    assert_snapshot!(processed_content);
}

#[test]
fn test_function_body_preprocessor() {
    // Create a simple book with a chapter containing a function_body directive
    let mut book = Book::new();
    let chapter = Chapter {
        name: "Chapter 1".to_string(),
        content:
            "some preamble\n```rust\n#![function_body!(\"../test_file.rs\", hello_world)]\n```\n after"
                .to_string(),
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
    let preprocessor = IncludeDocPreprocessor;
    let processed_book = preprocessor.run(&ctx, book).unwrap();

    // Extract the processed content for snapshot testing
    let mut processed_content = String::new();
    for item in processed_book.iter() {
        if let BookItem::Chapter(chapter) = item {
            if chapter.name == "Chapter 1" {
                processed_content = chapter.content.clone();
                break;
            }
        }
    }

    // Use insta to snapshot test the result
    assert_snapshot!(processed_content);
}

#[test]
fn test_complex_function_body_preprocessor() {
    // Create a book with a chapter containing function_body directive with multiple dependencies
    let mut book = Book::new();
    let chapter = Chapter {
        name: "Chapter 1".to_string(),
        content:
            "some preamble\n```rust\n#![function_body!(\"../test_file.rs\", TestStruct::print, [struct TestStruct, impl TestStruct::new, trait TestTrait, impl TestTrait for TestStruct])]\n```\n after"
                .to_string(),
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
    let preprocessor = IncludeDocPreprocessor;
    let processed_book = preprocessor.run(&ctx, book).unwrap();

    // Extract the processed content for snapshot testing
    let mut processed_content = String::new();
    for item in processed_book.iter() {
        if let BookItem::Chapter(chapter) = item {
            if chapter.name == "Chapter 1" {
                processed_content = chapter.content.clone();
                break;
            }
        }
    }

    // Use insta to snapshot test the result
    assert_snapshot!(processed_content);
}

#[test]
fn test_relative_path_with_source_path() {
    // Create a book with a chapter containing a relative path
    let mut book = Book::new();
    let chapter = Chapter {
        name: "Relative Path Test".to_string(),
        content: "```rust\n#![source_file!(\"../test_file.rs\")]\n```".to_string(),
        number: None,
        sub_items: vec![],
        path: Some(PathBuf::from("relative_test.md")),
        source_path: Some(PathBuf::from("relative_test.md")),
        parent_names: vec![],
    };
    book.push_item(BookItem::Chapter(chapter));

    // Create a preprocessor context
    let ctx = create_test_context();

    // Run the preprocessor
    let preprocessor = IncludeDocPreprocessor;
    let processed_book = preprocessor.run(&ctx, book).unwrap();

    // Extract the processed content
    let mut processed_content = String::new();
    for item in processed_book.iter() {
        if let BookItem::Chapter(chapter) = item {
            if chapter.name == "Relative Path Test" {
                processed_content = chapter.content.clone();
                break;
            }
        }
    }

    // Should contain the struct definition from test_file.rs
    assert!(
        processed_content.contains("struct TestStruct"),
        "Output doesn't contain expected content from test_file.rs"
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
