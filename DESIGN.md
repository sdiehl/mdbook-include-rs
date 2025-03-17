# Design for mdbook-include-rs

## Overview

`mdbook-include-rs` is a preprocessor for [mdBook](https://rust-lang.github.io/mdBook/) that enables including Rust code snippets from source files directly into documentation. This allows keeping documentation examples in sync with actual codebase.

## Features

1. **Function Inclusion**: Extract complete functions or just their bodies
2. **Type Definition Inclusion**: Include specific struct, enum, and trait definitions
3. **Implementation Inclusion**: Include impl blocks and trait implementations
4. **Selective Import**: Include specific dependencies alongside code snippets
5. **Source File Inclusion**: Include entire source files with optional filtering
6. **Hidden Dependencies**: Automatically include dependencies prefixed with `#` for proper compilation in documentation
7. **Path Resolution**: Support for relative paths based on chapter location or a global base directory

## Directives

The preprocessor supports the following directives:

1. `#![function!("path/to/file.rs", function_name)]` - Include a complete function
2. `#![function_body!("path/to/file.rs", function_name, [optional_dependencies])]` - Include just a function body
3. `#![struct!("path/to/file.rs", struct_name)]` - Include a struct definition
4. `#![enum!("path/to/file.rs", enum_name)]` - Include an enum definition
5. `#![trait!("path/to/file.rs", trait_name)]` - Include a trait definition
6. `#![impl!("path/to/file.rs", struct_name)]` - Include an implementation block
7. `#![trait_impl!("path/to/file.rs", trait_name for struct_name)]` - Include a trait implementation
8. `#![source_file!("path/to/file.rs")]` - Include an entire source file

## Components

1. **Parser**:
   - Parses Markdown files to find code blocks with directives
   - Extracts directive parameters (file path, item name, optional dependencies)

2. **Directive Processor**:
   - Handles different directive types
   - Delegates to appropriate extractors

3. **Extractors**:
   - `function_extractor.rs` - Extracts functions and function bodies
   - `struct_finder.rs` - Extracts struct definitions
   - `enum_finder.rs` - Extracts enum definitions
   - `trait_finder.rs` - Extracts trait definitions
   - `impl_finder.rs` - Extracts implementation blocks and trait implementations

4. **Formatter**:
   - Formats extracted code with proper syntax highlighting
   - Manages comment prefixing for hidden dependencies

5. **Output Generator**:
   - Generates the final Markdown with the processed code blocks

## Algorithm

1. **Parsing**:
   - Parse the directive to determine the file path, target item, and optional dependencies
   - Locate and read the source file
   - Parse the file using `syn`

2. **Processing**:
   - For functions: Extract the complete function
   - For function bodies: Extract body and hide declaration
   - For structs/enums/traits: Extract the complete definition
   - For impl blocks: Extract all methods in the implementation
   - For trait implementations: Extract the specific trait implementation
   - For source files: Include entire file
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

Using directive: `#![struct!("../test_file.rs", Test2)]`

Output:
```rust
#use std::fmt;
#
struct Test2 {}
```

### Enum Example

Using directive: `#![enum!("../test_file.rs", Status)]`

Output:
```rust
#use std::fmt;
#
enum Status {
    Active,
    Inactive,
    Pending,
}
```

### Trait and Implementation Examples

Using directive: `#![trait!("../test_file.rs", Display)]`

Output:
```rust
trait Display {
    fn display(&self) -> String;
}
```

Using directive: `#![trait_impl!("../test_file.rs", Display for User)]`

Output:
```rust
#struct User {
#    name: String,
#}
#
#trait Display {
#    fn display(&self) -> String;
#}
#
impl Display for User {
    fn display(&self) -> String {
        format!("User: {}", self.name)
    }
}
```

## Current Implementation Status

- ✅ Full function extraction
- ✅ Function body extraction
- ✅ Struct extraction
- ✅ Enum extraction
- ✅ Trait extraction
- ✅ Implementation block extraction
- ✅ Trait implementation extraction
- ✅ Source file inclusion
- ✅ Dependency handling
- ✅ Path resolution
- ✅ Integration with mdBook

## Testing

The codebase uses:
- Unit tests for core functionality
- Integration tests with mdBook
- Snapshot tests via `insta` crate for complex outputs