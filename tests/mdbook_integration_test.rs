use mdbook::MDBook;
use mdbook_include_rs::IncludeDocPreprocessor;
use std::fs;
use std::path::Path;

#[test]
fn test_preprocessor_with_complete_mdbook() {
    // Install the package locally first
    let status = std::process::Command::new("cargo")
        .args(["install", "--path", ".", "--force"])
        .status()
        .expect("Failed to execute cargo install command");

    assert!(status.success(), "Failed to install the preprocessor");

    // Get the path to our test book
    let project_root = env!("CARGO_MANIFEST_DIR");
    let book_dir = Path::new(project_root).join("tests/fixtures");

    // Initialize the book
    let mut mdbook = MDBook::load(&book_dir).unwrap();

    // Register our preprocessor
    mdbook.with_preprocessor(IncludeDocPreprocessor);

    // Build the book
    mdbook.build().unwrap();

    // Verify the output - check if our HTML contains the processed content
    let html_output = fs::read_to_string(book_dir.join("book/chapter_1.html")).unwrap();

    // The HTML should contain the function body we extracted, not the original directive
    assert!(
        html_output.contains("println!(\"Hello, world!\");"),
        "HTML output doesn't contain the function body"
    );

    // The HTML should not contain the original directive
    assert!(
        !html_output.contains("{{ #function_body!"),
        "HTML output still contains the original directive"
    );

    // Check the second chapter with source file directive
    let html_output2 = fs::read_to_string(book_dir.join("book/chapter_2.html")).unwrap();

    // Should contain the struct definition
    assert!(
        html_output2.contains("struct TestStruct"),
        "HTML output doesn't contain the source file content"
    );

    // The HTML should not contain the original directive
    assert!(
        !html_output2.contains("{{ #source_file!"),
        "HTML output still contains the original directive"
    );
}
