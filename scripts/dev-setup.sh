#!/bin/bash
# ABOUTME: Complete development environment setup script
# ABOUTME: Sets up everything needed for Linear CLI development

set -e

echo "🚀 Setting up Linear CLI development environment..."

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "linear-cli" ]; then
    echo "❌ Please run this script from the linear-cli project root"
    exit 1
fi

# Check for required tools
echo "🔍 Checking for required tools..."

if ! command -v cargo &> /dev/null; then
    echo "❌ Rust/Cargo not found. Please install from https://rustup.rs/"
    exit 1
fi
echo "✅ Rust/Cargo found"

if ! command -v uv &> /dev/null; then
    echo "⚠️  uv not found. Installing..."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    source ~/.cargo/env
fi
echo "✅ uv found"

# Install pre-commit
echo "📋 Setting up pre-commit hooks..."
uv tool install pre-commit
pre-commit install
echo "✅ Pre-commit hooks installed"

# Setup environment file if it doesn't exist
if [ ! -f ".env" ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please edit .env and add your Linear API key"
else
    echo "✅ .env file already exists"
fi

# Install additional cargo tools
echo "🔧 Installing useful cargo tools..."
echo "📦 Installing cargo-insta for snapshot testing..."
cargo install cargo-insta 2>/dev/null || echo "⚠️  cargo-insta installation failed"
echo "📦 Installing cargo-outdated for dependency updates..."
cargo install cargo-outdated 2>/dev/null || echo "⚠️  cargo-outdated installation failed"
echo "📦 Installing cargo-audit for security advisories..."
cargo install cargo-audit 2>/dev/null || echo "⚠️  cargo-audit installation failed"
echo "✅ Cargo tools installation complete"

# Build the project
echo "🔨 Building project..."
cargo build --workspace
echo "✅ Project built successfully"

# Run tests to verify setup
echo "🧪 Running tests to verify setup..."
cargo test --workspace
echo "✅ Tests pass"

echo ""
echo "🎉 Development environment setup complete!"
echo ""
echo "Next steps:"
echo "  1. Edit .env and add your Linear API key from https://linear.app/settings/api"
echo "  2. Run 'make help' to see available commands"
echo "  3. Run 'make dev' to run the full development workflow"
echo "  4. Run 'make run' to test the CLI"
echo ""
echo "Happy coding! 🦀"
