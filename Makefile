# ABOUTME: Makefile for common development commands and workflow automation
# ABOUTME: Provides shortcuts for testing, formatting, linting, and building

.PHONY: test test-snapshots test-integration test-debug fmt lint build build-images build-no-images release clean install install-images help run run-debug run-piped run-images run-no-images test-images test-no-images debug-deps check dev-setup dev all

# Default target
help:
	@echo "Available commands:"
	@echo ""
	@echo "🧪 Testing:"
	@echo "  make test           - Run unit tests"
	@echo "  make test-snapshots - Run tests with snapshot review"
	@echo "  make test-integration - Run integration tests (requires LINEAR_API_KEY)"
	@echo "  make test-debug     - Run tests with debug output"
	@echo ""
	@echo "🔧 Development:"
	@echo "  make fmt            - Format code"
	@echo "  make lint           - Run clippy linter"
	@echo "  make check          - Run fmt and lint checks (CI-style)"
	@echo "  make build          - Build debug version"
	@echo "  make build-images   - Build debug version with inline-images feature"
	@echo "  make build-no-images - Build debug version without inline-images feature"
	@echo "  make release        - Build release version"
	@echo "  make install        - Install binary locally"
	@echo "  make install-images - Install binary locally with inline-images support"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "🚀 Running:"
	@echo "  make run            - Run CLI with example issues command"
	@echo "  make run-debug      - Run CLI with debug logging"
	@echo "  make run-piped      - Test CLI output when piped (no colors/TTY)"
	@echo "  make run-images     - Run CLI with inline-images feature enabled"
	@echo "  make run-no-images  - Run CLI without inline-images feature"
	@echo ""
	@echo "🖼️ Image Testing:"
	@echo "  make test-images    - Test issue with images (requires LINEAR_API_KEY)"
	@echo "  make test-no-images - Test issue without image support"
	@echo ""
	@echo "🛠️ Setup & Debugging:"
	@echo "  make dev-setup      - Setup development environment"
	@echo "  make debug-deps     - Show dependency tree and check for issues"
	@echo ""
	@echo "📦 Workflows:"
	@echo "  make all            - Run fmt, lint, test, and build"
	@echo "  make dev            - Quick development check (fmt, lint, test)"

# Run unit tests
test:
	cargo test --workspace

# Run unit tests with snapshot review
test-snapshots:
	cargo insta test --review

# Run integration tests (requires LINEAR_API_KEY)
test-integration:
	cargo test --workspace --features integration-tests -- --ignored

# Run tests with debug output
test-debug:
	RUST_LOG=debug cargo test --workspace -- --nocapture

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

# Install binary locally
install:
	cargo install --path linear-cli

# Install binary locally with inline-images support
install-images:
	cargo install --path linear-cli --features inline-images

# Clean build artifacts
clean:
	cargo clean

# Run format and lint checks (for CI)
check:
	cargo fmt --all -- --check
	cargo clippy --workspace --all-targets -- -D warnings

# Run everything
all: fmt lint test build

# Quick development workflow
dev: fmt lint test

# Run the CLI with example commands
run:
	@echo "Running linear CLI..."
	cargo run -p linear-cli -- issues --limit 5

# Run CLI with debug logging
run-debug:
	@echo "Running linear CLI with debug logging..."
	RUST_LOG=debug cargo run -p linear-cli -- issues --limit 5

# Test CLI output when piped (no TTY detection)
run-piped:
	@echo "Testing linear CLI piped output (no colors/TTY)..."
	cargo run -p linear-cli -- issues --limit 5 | cat

# Setup development environment
dev-setup:
	@echo "Running comprehensive development setup..."
	./scripts/dev-setup.sh

# Debug dependency issues
debug-deps:
	@echo "Checking dependency tree..."
	cargo tree --workspace
	@echo ""
	@echo "Checking for outdated dependencies..."
	@if cargo outdated --version >/dev/null 2>&1; then \
		cargo outdated; \
	else \
		echo "⚠️  cargo-outdated not installed. Install with: cargo install cargo-outdated"; \
	fi
	@echo ""
	@echo "Checking for security advisories..."
	@cargo audit 2>/dev/null || echo "⚠️  cargo-audit not installed. Install with: cargo install cargo-audit"

# Build with inline-images feature enabled
build-images:
	@echo "Building linear CLI with inline-images feature..."
	cargo build --workspace --features inline-images

# Build without inline-images feature (explicit)
build-no-images:
	@echo "Building linear CLI without inline-images feature..."
	cargo build --workspace --no-default-features --features oauth

# Run CLI with inline-images feature enabled
run-images:
	@echo "Running linear CLI with inline-images feature..."
	cargo run -p linear-cli --features inline-images -- issues --limit 5

# Run CLI without inline-images feature
run-no-images:
	@echo "Running linear CLI without inline-images feature..."
	cargo run -p linear-cli --no-default-features --features oauth -- issues --limit 5

# Test loading an issue with image support enabled
test-images:
	@echo "Testing issue with inline-images support (use STR-2 which has images)..."
	@echo "Set LINEAR_CLI_VERBOSE=1 for detailed output"
	@if [ -f .env ]; then \
		echo "Loading environment from .env..."; \
		set -a && . ./.env && set +a && \
		cargo run -p linear-cli --features inline-images -- issue STR-2; \
	else \
		echo "⚠️  No .env file found. Set LINEAR_API_KEY environment variable."; \
		cargo run -p linear-cli --features inline-images -- issue STR-2; \
	fi

# Test loading an issue without image support
test-no-images:
	@echo "Testing issue without inline-images support..."
	@if [ -f .env ]; then \
		echo "Loading environment from .env..."; \
		set -a && . ./.env && set +a && \
		cargo run -p linear-cli --no-default-features --features oauth -- issue STR-2; \
	else \
		echo "⚠️  No .env file found. Set LINEAR_API_KEY environment variable."; \
		cargo run -p linear-cli --no-default-features --features oauth -- issue STR-2; \
	fi
