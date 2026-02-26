# Ubertooth One Connector - Build Commands

# Check all crates
check:
    cargo check --workspace

# Lint with clippy
lint:
    cargo clippy --workspace -- -D warnings

# Format code
fmt:
    cargo fmt --all

# Check formatting
fmt-check:
    cargo fmt --all -- --check

# Run tests
test:
    cargo test --workspace

# Build debug
build:
    cargo build --workspace

# Build release
build-release:
    cargo build --release --workspace

# Run headless agent
run:
    cargo run --package ubertooth-agent

# Run headless agent (release)
run-release:
    cargo run --release --package ubertooth-agent

# Run CLI
run-cli:
    cargo run --package ubertooth-cli

# Run all validation checks (for local CI)
ci: check fmt-check lint test

# Clean build artifacts
clean:
    cargo clean

# Install udev rules for Ubertooth One
install-udev:
    @echo "Installing udev rules for Ubertooth One..."
    @echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="1d50", ATTR{idProduct}=="6002", MODE="0666", GROUP="plugdev"' | sudo tee /etc/udev/rules.d/52-ubertooth.rules
    @sudo udevadm control --reload-rules
    @sudo udevadm trigger
    @echo "Done! Please replug your Ubertooth One device."
    @echo "You may need to add your user to the plugdev group: sudo usermod -aG plugdev $USER"
