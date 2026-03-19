# Contributing to Ubertooth One Connector

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## Development Setup

### Prerequisites
- Rust 1.70 or later
- Cargo
- Git

### Getting Started
```bash
# Clone the repository
git clone https://github.com/jtomek-strike48/ubertooth-one-connector.git
cd ubertooth-one-connector

# Build the project
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Check for issues
cargo clippy
```

## Code Style

### Formatting
We use `rustfmt` to maintain consistent code style. Configuration is in `rustfmt.toml`.

**Before committing:**
```bash
# Format your code
cargo fmt

# Verify formatting
cargo fmt --check
```

### Linting
We use `clippy` for code quality checks:

```bash
# Check for issues
cargo clippy --all-targets --all-features

# Auto-fix where possible
cargo clippy --fix
```

### Code Quality Standards
- Maximum file size: 800 lines (enforce via code review)
- Maximum function size: 50 lines
- No deep nesting (>4 levels)
- Comprehensive error handling
- 80%+ test coverage for new code

## Testing

### Running Tests
```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Writing Tests
- Write unit tests in the same file as the code
- Write integration tests in `crates/integration-tests/`
- Follow TDD when possible (write tests first)
- Ensure 80%+ code coverage

## Commit Guidelines

### Commit Message Format
```
<type>: <description>

[optional body]

[optional footer]
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `refactor`: Code refactoring
- `docs`: Documentation changes
- `test`: Test changes
- `chore`: Build/tooling changes
- `perf`: Performance improvements
- `style`: Formatting changes

**Examples:**
```
feat: Add BLE advertising data parser

fix: Resolve USB timeout in bulk read

refactor: Split ui.rs into modular components

docs: Update API documentation for capture store
```

## Pull Request Process

1. **Create a feature branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes**
   - Write tests first (TDD)
   - Implement the feature
   - Ensure all tests pass
   - Format and lint your code

3. **Commit your changes**
   ```bash
   git add <files>
   git commit -m "feat: Your feature description"
   ```

4. **Push and create PR**
   ```bash
   git push -u origin feature/your-feature-name
   gh pr create --title "Your PR title" --body "Description"
   ```

5. **PR checklist**
   - [ ] All tests pass
   - [ ] Code is formatted (`cargo fmt --check`)
   - [ ] No clippy warnings
   - [ ] Documentation updated
   - [ ] No files exceed 800 lines
   - [ ] Test coverage ≥80%

## Project Structure

```
ubertooth-one-connector/
├── apps/           # Application binaries
│   ├── api/       # REST API server
│   ├── cli/       # CLI application
│   └── headless/  # Headless daemon
├── crates/        # Library crates
│   ├── core/      # Core types and traits
│   ├── usb/       # USB hardware layer
│   ├── platform/  # Business logic
│   ├── tools/     # CLI tool implementations
│   └── plugin/    # Plugin system
├── examples/      # Example programs
└── docs/          # Documentation
```

## Architecture Guidelines

### File Organization
- Keep files under 800 lines
- One responsibility per file
- Extract large functions to separate files
- Use subdirectories for related modules

### Naming Conventions
- Files: `snake_case.rs`
- Structs/Enums: `PascalCase`
- Functions/Variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- Modules: `snake_case`

### Error Handling
- Use `Result<T, E>` for fallible operations
- Provide user-friendly error messages
- Log detailed error context
- Never silently swallow errors

## Getting Help

- Check existing issues: https://github.com/jtomek-strike48/ubertooth-one-connector/issues
- Read the documentation in `docs/`
- Ask questions in issue discussions

## License

By contributing, you agree that your contributions will be licensed under the project's license.
