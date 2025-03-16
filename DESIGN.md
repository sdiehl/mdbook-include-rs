# Design for mdbook-include-rs

## Overview

`mdbook-include-rs` is a preprocessor for [mdBook](https://rust-lang.github.io/mdBook/) that enables including Rust code snippets from source files directly into documentation. This allows keeping documentation examples in sync with actual codebase.

## Features

1. **Function Body Inclusion**: Extract function body without declaration
2. **Struct/Enum Definition Inclusion**: Include specific struct or enum definitions
3. **Selective Import**: Include specific dependencies alongside code snippets
4. **Source File Inclusion**: Include entire source files with optional filtering
5. **Hidden Dependencies**: Automatically include dependencies prefixed with `#` for proper compilation in documentation

## Directives

The preprocessor supports the following directives:

1. `#![function_body!("path/to/file.rs", function_name, [optional_dependencies])]`
2. `#![struct!("path/to/file.rs", struct_name, [optional_dependencies])]`
3. `#![enum!("path/to/file.rs", enum_name, [optional_dependencies])]`
4. `#![source_file!("path/to/file.rs", [optional_items_to_keep])]`

## Algorithm

1. **Parsing**:
   - Parse the directive to determine the file path, target item, and optional dependencies
   - Locate and read the source file
   - Pretty print the Rust file using `prettyplease` or a similar formatter
   - Parse the formatted file using `syn`

2. **Processing**:
   - For function bodies: Extract body and hide declaration
   - For structs/enums: Extract the complete definition
   - For source files: Include entire file or filter by specified items
   - Add specified dependencies as visible content
   - Mark all other code with `#` prefix to indicate hidden dependencies

3. **Output Generation**:
   - Generate markdown code block with the processed content
   - Ensure proper indentation and formatting is maintained

## Examples

### Function Body Example

Given this source code:
```rust
use std::fmt;

struct Test1 {}
struct Test2 {}

fn free_function() {
   println!("Hello, world!");
}
```

Using directive: `#![function_body!("../test_file.rs", free_function, [struct Test2])]`

Output:
```rust
#use std::fmt;
#
#struct Test1 {}
struct Test2 {}
#
println!("Hello, world!");
```

### Struct Example

Using directive: `#![struct!("../test_file.rs", Test2, [])]`

Output:
```rust
#use std::fmt;
#
struct Test2 {}
```

## Implementation Plan

1. **Parser Module**:
   - Parse directives using regex or a custom parser
   - Extract file paths, item names, and dependencies

2. **Preprocessor Module**:
   - Handle file reading and processing
   - Implement item extraction logic for functions, structs, and enums
   - Generate properly formatted output

3. **Integration with mdBook**:
   - Register preprocessor with mdBook API
   - Hook into the build process

4. **Testing**:
   - Unit tests for parser and preprocessor
   - Integration tests with mdBook
   - Snapshot tests for complex outputs