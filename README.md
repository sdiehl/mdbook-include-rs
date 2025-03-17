# mdbook-include-rs

A preprocessor for [mdBook](https://rust-lang.github.io/mdBook/) that lets you include code from external Rust source files with extraction capabilities. This makes it easy to maintain code examples that are always in sync with your actual codebase.

## Features

- Include entire source files or just the parts you need (functions, structs, enums, traits, impls)
- Extract specific elements like function bodies, struct definitions, trait implementations
- Automatically include dependencies for functions when needed
- Show only function bodies when desired, with supporting types shown separately
- Extract methods from structs and trait implementations cleanly
- Your code examples stay as regular source files that you can compile and test
- Support for relative paths based on chapter location

## Installation

First, install the preprocessor:

```bash
cargo install mdbook-include-rs
```

Then, modify your `book.toml` file:

```toml
[book]
title = "My Book"
authors = ["Your Name"]

# Add the preprocessor to your mdBook
[preprocessor.include-rs]

# Optional: Specify a base directory for all file paths
# If not provided, paths will be relative to each markdown file's directory
[preprocessor.include-rs]
base-dir = "examples"  # Optional
```

## Supported Directives

`mdbook-include-rs` supports the following directives:

- `#![source_file!("path/to/file.rs")]` - Include entire source file
- `#![function!("path/to/file.rs", function_name)]` - Include complete function
- `#![function_body!("path/to/file.rs", function_name, [optional_dependencies])]` - Include just the function body
- `#![struct!("path/to/file.rs", struct_name)]` - Include struct definition
- `#![enum!("path/to/file.rs", enum_name)]` - Include enum definition
- `#![trait!("path/to/file.rs", trait_name)]` - Include trait definition
- `#![impl!("path/to/file.rs", struct_name)]` - Include implementation block
- `#![trait_impl!("path/to/file.rs", trait_name for struct_name)]` - Include trait implementation

## Usage Examples

### Include a Complete Source File

To include an entire source file:

````markdown
```rust
#![source_file!("source_file.rs")]
```
````

This will include the entire contents of the file, with `use` statements automatically filtered out for cleaner output.

### Include a Complete Function

To include a full function definition:

````markdown
```rust
#![function!("source_file.rs", hello_world)]
```
````

### Include a Function Body

To focus on the body of a specific function while keeping the code runnable:

````markdown
```rust
#![function_body!("source_file.rs", hello_world)]
```
````

The output will show the function body with its declaration commented out, which:
1. Maintains the focus on the function body
2. Preserves the code structure to make it runnable in mdBook
3. Shows the context of the function signature for reference

### Include Specific Type Definitions

Extract specific type definitions from source files:

````markdown
```rust
#![struct!("models.rs", User)]
#![enum!("models.rs", AccountType)]
#![trait!("behaviors.rs", Displayable)]
```
````

### Include Implementation Blocks

Extract implementation blocks:

````markdown
```rust
#![impl!("models.rs", User)]
#![trait_impl!("models.rs", Displayable for User)]
```
````

### Include a Function with Dependencies

If your function depends on other types or functions, you can include them too:

````markdown
```rust
#![function_body!("src/models.rs", User::display_profile, [struct User, trait Displayable, impl Displayable for User, enum Account])]
```
````

This will include:
1. The `User` struct definition
2. The `Displayable` trait definition
3. The implementation of `Displayable` for `User`
4. The `Account` enum definition
5. And finally, the body of the `display_profile` method
6. All other code in the file will be hidden but prefixed with `#` to allow it to remain runnable in mdBook.

The dependencies will be included in the order you list them, with the main function's body appearing last.

### Dependency Types

You can include various types of dependencies:

- `struct StructName` - includes a struct definition
- `enum EnumName` - includes an enum definition
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
#![function_body!("src/auth.rs", authenticate_user, [struct Credentials, struct User, fn validate_password])]
```

Behind the scenes, password hashing works like this:

```rust
#![function_body!("src/auth.rs", hash_password)]
```
````

## How It Works

The preprocessor:

1. Parses your markdown looking for code blocks with directives
2. Uses Rust's `syn` library to parse and extract the requested code elements
3. Formats the extracted code with proper syntax highlighting
4. Replaces the directive in your markdown with the extracted code

## Command Line Interface

As per the mdBook preprocessor standard:

```bash
# Check if the preprocessor supports a renderer
mdbook-include-rs supports <renderer>

# Process a book
mdbook-include-rs pre-process <path-to-book>
```

## License

This project is licensed under:

- Apache License, Version 2.0