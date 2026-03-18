# Testing Guide

## Overview

This project follows Test-Driven Development (TDD) principles with a goal of 80%+ code coverage.

## Test Structure

```
tests/
├── test_utils/
│   └── mod.rs          # Shared test utilities and fixtures
├── capture_tests.rs    # Capture storage system tests
└── tui_integration.rs  # TUI workflow and state machine tests
```

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suite
```bash
# Capture tests only
cargo test --test capture_tests

# TUI integration tests only
cargo test --test tui_integration
```

### Run Individual Test
```bash
cargo test test_save_and_load_metadata
```

### Run Tests with Output
```bash
cargo test -- --nocapture
```

### Run Tests in Parallel
```bash
cargo test -- --test-threads=4
```

## Test Coverage

### Generate Coverage Report
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage
cargo tarpaulin --out Html --output-dir coverage
```

### View Coverage
```bash
open coverage/index.html
```

## Test Categories

### Unit Tests
Located in individual crate modules using `#[cfg(test)]`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Test implementation
    }
}
```

### Integration Tests
Located in `tests/` directory - test public APIs and workflows.

### Fixtures and Utilities

#### CaptureStoreFixture
Provides temporary storage for testing:

```rust
use test_utils::CaptureStoreFixture;

#[test]
fn test_capture_workflow() {
    let fixture = CaptureStoreFixture::new();

    // Use fixture.store for testing
    let metadata = fixture.create_sample_metadata("ble-scan", 100);
    fixture.store.save_metadata(&metadata).unwrap();
}
```

#### MockUsbDevice
Simulates USB device without hardware:

```rust
use test_utils::MockUsbDevice;

#[test]
fn test_device_detection() {
    let device = MockUsbDevice::ubertooth_one();
    assert!(device.is_ubertooth());
}
```

#### create_test_pcap
Creates temporary PCAP files:

```rust
use test_utils::create_test_pcap;

#[test]
fn test_pcap_parsing() {
    let temp_dir = tempfile::tempdir().unwrap();
    let path = temp_dir.path().join("test.pcap");

    create_test_pcap(&path, 10).unwrap();
    // Parse PCAP...
}
```

## Testing Patterns

### Error Testing
```rust
use test_utils::assert_error_contains;

#[test]
fn test_error_handling() {
    let result = some_function_that_fails();
    assert_error_contains!(result, "expected error message");
}
```

### Async Testing
```rust
#[tokio::test]
async fn test_async_function() {
    let result = async_function().await;
    assert!(result.is_ok());
}
```

### Snapshot Testing
```rust
use insta::assert_json_snapshot;

#[test]
fn test_json_output() {
    let output = generate_json();
    assert_json_snapshot!(output);
}
```

## Writing Tests

### TDD Workflow

1. **Write test first** (RED)
   ```rust
   #[test]
   fn test_new_feature() {
       let result = new_feature();
       assert_eq!(result, expected);
   }
   ```

2. **Run test - it should FAIL**
   ```bash
   cargo test test_new_feature
   ```

3. **Implement minimal code** (GREEN)
   ```rust
   fn new_feature() -> Type {
       // Minimal implementation
   }
   ```

4. **Run test - it should PASS**
   ```bash
   cargo test test_new_feature
   ```

5. **Refactor** (IMPROVE)
   - Improve implementation
   - Verify tests still pass

### Test Naming Convention

- `test_<function>_<scenario>` - e.g., `test_save_metadata_success`
- `test_<feature>_<behavior>` - e.g., `test_capture_list_empty`
- `test_<error_case>` - e.g., `test_load_nonexistent_capture`

### Test Organization

Group related tests in modules:

```rust
#[cfg(test)]
mod app_state_tests {
    use super::*;

    #[test]
    fn test_state_transitions() { }

    #[test]
    fn test_back_navigation() { }
}

#[cfg(test)]
mod keyboard_tests {
    #[test]
    fn test_shortcuts() { }
}
```

## Continuous Integration

Tests run automatically on:
- Pull requests
- Main branch commits
- Release builds

### CI Test Command
```bash
cargo test --all-features --workspace
```

## Test Dependencies

Added to `Cargo.toml`:

```toml
[workspace.dev-dependencies]
insta = "1.34"          # Snapshot testing
mockito = "1.2"         # HTTP mocking
tempfile = "3"          # Temporary test directories
```

## Best Practices

1. **Isolation** - Tests should not depend on each other
2. **Speed** - Keep tests fast (< 1 second per test)
3. **Clarity** - Test one thing per test
4. **Coverage** - Aim for 80%+ code coverage
5. **Fixtures** - Use shared fixtures via `test_utils`
6. **Mocking** - Mock external dependencies (USB, filesystem)
7. **Error cases** - Test both success and failure paths
8. **Edge cases** - Test boundary conditions

## Troubleshooting

### Test Failures

1. Run with verbose output:
   ```bash
   cargo test -- --nocapture
   ```

2. Run single test to isolate:
   ```bash
   cargo test test_name -- --exact
   ```

3. Check for race conditions:
   ```bash
   cargo test -- --test-threads=1
   ```

### Coverage Issues

If coverage is below 80%:

1. Generate coverage report
2. Identify uncovered code
3. Add tests for uncovered paths
4. Re-run coverage

### Flaky Tests

- Check for timing issues
- Verify test isolation
- Look for shared state
- Use deterministic seeds for randomness

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Cargo Test Documentation](https://doc.rust-lang.org/cargo/commands/cargo-test.html)
- [Insta Snapshot Testing](https://insta.rs/)
- [Mockito HTTP Mocking](https://docs.rs/mockito/)

## Next Steps

- Add unit tests to individual crates
- Increase integration test coverage
- Set up CI/CD with coverage reporting
- Add benchmark tests for performance
