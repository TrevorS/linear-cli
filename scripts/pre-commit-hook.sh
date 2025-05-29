#!/bin/sh
# Pre-commit hook script for Rust projects
# Run this with: cp scripts/pre-commit-hook.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit

set -eu

echo "Running pre-commit checks..."

# Check code formatting
echo "Checking code formatting with cargo fmt..."
if ! cargo fmt --all -- --check
then
    echo "❌ Code formatting issues found. Run 'cargo fmt --all' to fix."
    exit 1
fi
echo "✅ Code formatting OK"

# Run clippy lints
echo "Running clippy lints..."
if ! cargo clippy --workspace --all-targets -- -D warnings
then
    echo "❌ Clippy found issues. Please fix them before committing."
    exit 1
fi
echo "✅ Clippy checks passed"

# Build check
echo "Running cargo check..."
if ! cargo check --workspace
then
    echo "❌ Build check failed."
    exit 1
fi
echo "✅ Build check passed"

# Optional: Run tests (commented out as they can be slow)
# echo "Running tests..."
# if ! cargo test --workspace
# then
#     echo "❌ Tests failed."
#     exit 1
# fi
# echo "✅ Tests passed"

echo "✅ All pre-commit checks passed!"
exit 0