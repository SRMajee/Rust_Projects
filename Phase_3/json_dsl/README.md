# Declarative JSON DSL

An idiomatic, zero-runtime-parsing domain-specific language (DSL) macro (`json!`) built with Rust's `macro_rules!` system. It parses nested key-value and array syntax directly into a custom recursive `JsonValue` AST at compile time.

## Key Concepts

- **Token Trees (`tt`)**: Using token trees to capture delimited nested structures (`{...}`, `[...]`) and arbitrary literal/expression tokens before expansion.
- **Recursive Expansion**: Declarative macro rules recursively invoke `$crate::json!(...)` on inner elements and key-value entries.
- **Repetition Specifiers**: Uses `$( ... ),* $(,)?` to support flexible syntax with optional trailing commas in arrays and objects.
- **Key Agnostic**: The `json_key!` helper handles both quoted string keys (`"title": "value"`) and unquoted identifier keys (`title: "value"`).
- **Zero-Cost Interop**: Leaf nodes convert into `JsonValue` through generic `From` implementations for booleans, numbers, and strings, with support for parenthesized Rust expressions.

## Syntax Examples

```rust
use json_dsl::json;

let data = json!({
    "name": "Alice",
    active: true,
    "score": 98.5,
    "tags": ["rust", "macros",],
    "nested": {
        "verified": null,
        "calculation": (10 * 5),
    }
});
```

## Project Structure

```text
json_dsl/
├── src/
│   ├── lib.rs       # JsonValue AST, From impls, Display & recursive json! macro
│   └── main.rs      # Interactive demo showcasing nested DSL syntax & traversal
├── tests/
│   └── dsl_tests.rs # Test suite: primitives, arrays, objects, recursion & edge cases
└── Cargo.toml
```

## Running Tests & Verification

### 1. Run Unit Tests & Doc-tests

```bash
cargo test
```

Verifies:

- Primitive literal parsing (`null`, booleans, integers, floats, strings)
- Flat, empty, and nested arrays with trailing commas
- Flat, empty, and nested objects with trailing commas and ident keys
- Deeply nested structures and typed AST accessor methods
- Dynamic Rust variable and expression interpolation
- Deterministic `Display` JSON string formatting

### 2. Run Demo Application

```bash
cargo run
```
