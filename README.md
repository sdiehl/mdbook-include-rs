# mdbook-include-rs

A powerful preprocessor for [mdBook](https://rust-lang.github.io/mdBook/) that lets you include code from external source files with intelligent extraction capabilities. This makes it easy to maintain accurate code examples that are always in sync with your actual codebase.

## Features

- Include entire source files or just the parts you need (specific functions, methods, structs, etc.)
- Automatically include dependencies for functions when needed
- Show only function bodies when desired, with supporting types shown separately
- Extract methods from structs and trait implementations cleanly
- Your code examples stay as regular source files that you can compile and test

## Installation

First, install the preprocessor:

```bash
cargo install mdbook-include-doc
```

Then, add it to your mdBook dependencies. In your book's directory, make sure to modify your `book.toml` file:

```toml
[book]
title = "My Book"
authors = ["Your Name"]

# Add the preprocessor to your mdBook
[preprocessor.include-doc]

# Optional: Specify a base directory for all file paths
# If not provided, paths will be relative to each markdown file's directory
# [preprocessor.include-doc]
# base-dir = "examples"
```

## Usage Examples

### Include a Complete Source File

To include an entire source file:

````markdown
```rust
#![source_file!("source_file.rs")]
```
````

This will include the entire contents of the file, with `use` statements automatically filtered out for cleaner output.

### Include Just a Function Body

To include just the body of a specific function (without the function declaration):

````markdown
```rust
#![function_body!("source_file.rs", hello_world)]
```
````

The output will only contain the content inside the function body, not the function declaration itself.

### Include a Function with Dependencies

If your function depends on other types or functions, you can include them too:

````markdown
```rust
#![function_body!("src/models.rs", User::display_profile, [struct User, trait Displayable, impl Displayable for User])]
```
````

This will include:
1. The `User` struct definition
2. The `Displayable` trait definition
3. The implementation of `Displayable` for `User`
4. And finally, the body of the `display_profile` method

The dependencies will be included in the order you list them, with the main function's body appearing last.

### Dependency Types

You can include various types of dependencies:

- `struct StructName` - includes a struct definition
- `trait TraitName` - includes a trait definition
- `impl StructName` - includes all methods in an impl block for a struct
- `impl StructName::method_name` - includes a specific method from an impl block
- `impl TraitName for StructName` - includes a trait implementation
- `function_name` - includes another function

## Real-World Example

For a document explaining user authentication:

````markdown
# User Authentication

Our system handles authentication with a simple API:

```rust
{{#function_body!("src/auth.rs", authenticate_user, [struct Credentials, struct User, fn validate_password])}}
```

Behind the scenes, password hashing works like this:

```rust
{{#function_body!("src/auth.rs", hash_password)}}
```
````

## How It Works

The preprocessor:

1. Parses your markdown looking for code blocks with `include-doc` directives
2. Uses Rust's `syn` library to parse and extract the requested code elements
3. Formats the extracted code with proper syntax highlighting
4. Replaces the directive in your markdown with the extracted code

## Command Line Interface

As per the mdBook preprocessor standard:

```bash
# Check if the preprocessor supports a renderer
mdbook-include-doc supports <renderer>

# Process a book
mdbook-include-doc pre-process <path-to-book>
```

## License

This project is licensed under:

- Apache License, Version 2.0