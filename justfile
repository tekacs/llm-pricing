# Default recipe to display help
default:
    @just --list

# Build the project
build:
    cargo build

# Build for release
build-release:
    cargo build --release

# Run the application
run *args:
    cargo run -- {{args}}

# Check code formatting
check:
    cargo check

# Format code
fmt:
    cargo fmt

# Run clippy for linting
clippy:
    cargo clippy

# Clean build artifacts
clean:
    cargo clean

# Install the binary locally
install:
    cargo install --path .

# Show help for the application
help:
    cargo run -- --help