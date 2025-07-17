# Stria Rust Implementation

A Rust implementation of the Stria configuration management language.

## Overview

**Stria** is a declarative, schema-validated configuration management language designed for creating robust and maintainable configuration systems. This project provides a complete Rust implementation of the Stria language as specified in [`spec/docs/index.md`](spec/docs/index.md).

### Etymology

The name "Stria" derives from three Latin and English roots:

- **Structura** (Latin): Structure
- **Via** (Latin): Path/Way
- **Strict** (English): Rigorous

Together, these represent the language's philosophy of providing a **strict structure** and **clear path** for configuration management.

## Design Goals

- **Type Safety**: All configurations are validated against strict schemas
- **Immutability**: Configurations are immutable by default
- **Expressiveness**: Rich syntax for complex configuration scenarios
- **Maintainability**: Clear, readable configuration files
- **Validation**: Comprehensive error checking and reporting

## Language Features

### Core Capabilities

- **Struct-based Architecture**: Configuration organized using `struct` definitions
- **Multiple Initialization Patterns**: Flexible object instantiation
- **Schema Validation**: Compile-time validation against schemas
- **Expression-oriented**: Everything is an expression that returns a value
- **Type System**: Rich type system with primitives, collections, and unions
- **Pattern Matching**: Powerful `match` expressions with type guards
- **Configuration-first**: Designed specifically for configuration management

### Supported Types

#### Primitive Types

- **Integers**: `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`
- **Floating-point**: `f32`, `f64`
- **Text**: `string`
- **Boolean**: `bool`
- **Collections**: `List<T>`, ranges (`I32Range`, `F64Range`)
- **Optional**: `T?` for nullable types
- **Union**: `T | U` for multiple possible types

#### Example Configuration

```stria
struct ServerConfig {
    host: string
    port: u16 = 8080
    ssl: bool = false
    maxConnections?: u32
}

val config = ServerConfig {
    host = "api.example.com"
    port = 443
    ssl = true
    maxConnections = 1000
}
```

## Project Structure

```
stria-rust/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lexer/           # Lexical analysis
│   ├── parser/          # Syntax parsing
│   ├── semantic/        # Semantic analysis
│   ├── interpreter/     # Runtime execution
│   └── lib.rs           # Library interface
├── spec/
│   └── docs/
│       └── index.md     # Language specification
├── tests/               # Test suite
├── examples/            # Example configurations
└── README.md
```

## Implementation Components

### 1. Lexer

- Tokenizes Stria source code
- Handles comments, keywords, literals, and operators
- Supports numeric type suffixes (`42i32`, `3.14f64`)
- Built with `nom` parser combinators

### 2. Parser

- Parses tokens into Abstract Syntax Tree (AST)
- Handles expressions, statements, and declarations
- Supports pattern matching and type annotations
- Implements operator precedence and associativity

### 3. Semantic Analyzer

- Type checking and inference
- Scope resolution and variable binding
- Schema validation
- Error reporting with detailed diagnostics

### 4. Interpreter

- Executes validated AST
- Handles configuration construction and validation
- Outputs to JSON, YAML, or TOML formats
- Provides runtime error handling

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo package manager

### Build from Source

```bash
git clone https://github.com/sharo-jef/stria-rust.git
cd stria-rust
cargo build --release
```

### Install via Cargo

```bash
cargo install stria-rust
```

## Usage

### Command Line Interface

```bash
# Compile and validate a Stria configuration
stria-rust compile config.stria

# Output to JSON format
stria-rust compile config.stria --format json

# Output to YAML format
stria-rust compile config.stria --format yaml

# Validate configuration without output
stria-rust validate config.stria

# Show help
stria-rust --help
```

### Library Usage

```rust
use stria_rust::{compile, CompileOptions, OutputFormat};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        struct Config {
            name: string
            port: u16 = 8080
        }

        val config = Config {
            name = "my-service"
            port = 3000
        }
    "#;

    let options = CompileOptions {
        format: OutputFormat::Json,
        validate_only: false,
    };

    let result = compile(source, options)?;
    println!("{}", result);

    Ok(())
}
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test lexer

# Run integration tests
cargo test --test integration
```

### Code Style

This project follows Rust standard formatting:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run linter
cargo clippy

# Run all checks
cargo clippy -- -D warnings
```

### Updating Spec Submodule

The language specification is maintained as a git submodule in the `spec/` directory. To update the specification:

#### Using Update Scripts (Recommended)

Automated scripts are available for updating the spec submodule:

```bash
# Using bash script (Linux/macOS/WSL)
./scripts/update-spec.sh

# Using PowerShell script (Windows)
.\scripts\update-spec.ps1

# PowerShell with auto-commit (no prompts)
.\scripts\update-spec.ps1 -AutoCommit

# Show help for PowerShell script
.\scripts\update-spec.ps1 -Help
```

#### Manual Update

You can also update the submodule manually:

```bash
# Initialize and update all submodules
git submodule update --init --recursive

# Update spec submodule to latest commit
git submodule update --remote spec

# Or navigate to spec directory and pull manually
cd spec
git pull origin main
cd ..

# Commit the submodule update
git add spec
git commit -m "docs: update spec submodule to latest version"
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Implement your changes following the coding standards
4. Add tests for new functionality
5. Run the test suite (`cargo test`)
6. Format your code (`cargo fmt`)
7. Run linter (`cargo clippy`)
8. Commit your changes using [Conventional Commits](https://www.conventionalcommits.org/)
9. Push to the branch (`git push origin feature/amazing-feature`)
10. Open a Pull Request

### Commit Message Format

This project uses [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Examples:

- `feat(parser): add support for union types`
- `fix(lexer): handle numeric literals with type suffixes`
- `docs: update README with installation instructions`
- `test: add integration tests for semantic analyzer`

## Error Handling

Stria provides comprehensive error reporting similar to Rust's compiler:

```
error[E0001]: type mismatch
  --> config.stria:5:18
   |
5  |     port: string = 8080
   |                    ^^^^ expected `string`, found `i32`
   |
help: try converting the integer to a string
   |
5  |     port: string = "8080"
   |                    ~~~~~~
```

## VS Code Extension

This implementation is designed to support a VS Code extension with:

- Syntax highlighting
- Error diagnostics
- Auto-completion
- Go-to-definition
- Refactoring support

## Platform Support

Stria ensures consistent behavior across all platforms:

- **Cross-platform consistency**: Identical results on all supported systems
- **Numeric precision**: Exact bit-width preservation for all numeric types
- **Type emulation**: Software emulation for unsupported native types
- **Deterministic output**: Reproducible configuration generation

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Related Projects

- [Stria VS Code Extension](https://github.com/sharo-jef/stria-vscode) - Official VS Code support
- [Stria Language Server](https://github.com/sharo-jef/stria-lsp) - Language Server Protocol implementation

## Documentation

- [Language Specification](spec/docs/index.md) - Complete Stria language specification
- [API Documentation](https://docs.rs/stria-rust) - Rust API documentation
- [Examples](examples/) - Example configurations and use cases

## Support

- [GitHub Issues](https://github.com/sharo-jef/stria-rust/issues) - Bug reports and feature requests
- [Discussions](https://github.com/sharo-jef/stria-rust/discussions) - Community discussions

---

**Note**: This is an implementation of the Stria language specification. For the complete language reference, see [`spec/docs/index.md`](spec/docs/index.md).
