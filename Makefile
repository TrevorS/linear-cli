# ABOUTME: Makefile for common development commands and workflow automation
# ABOUTME: Provides shortcuts for testing, formatting, linting, and building

.PHONY: test test-snapshots test-integration fmt lint build release clean help run

# Default target
help:
	@echo "Available commands:"
	@echo "  make test           - Run unit tests"
	@echo "  make test-snapshots - Run tests with snapshot review"
	@echo "  make test-integration - Run integration tests (requires LINEAR_API_KEY)"
	@echo "  make fmt            - Format code"
	@echo "  make lint           - Run clippy linter"
	@echo "  make build          - Build debug version"
	@echo "  make release        - Build release version"
	@echo "  make clean          - Clean build artifacts"
	@echo "  make check          - Run fmt and lint checks"
	@echo "  make run            - Run the CLI with example commands"
	@echo "  make all            - Run fmt, lint, test, and build"

# Run unit tests
test:
	cargo test --workspace

# Run unit tests with snapshot review
test-snapshots:
	cargo insta test --review

# Run integration tests (requires LINEAR_API_KEY)
test-integration:
	cargo test --workspace --features integration-tests -- --ignored

# Format code
fmt:
	cargo fmt --all

# Run linter
lint:
	cargo clippy --workspace --all-targets -- -D warnings

# Build debug version
build:
	cargo build --workspace

# Build release version
release:
	cargo build --release --workspace

# Clean build artifacts
clean:
	cargo clean

# Run format and lint checks (for CI)
check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings

# Run everything
all: fmt lint test build

# Run the CLI with example commands
run:
	@echo "Running linear CLI..."
	cargo run -p linear-cli -- issues --limit 5
