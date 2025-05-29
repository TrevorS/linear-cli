# Pre-commit Hooks Setup

This document explains how to set up pre-commit hooks for this Rust project to ensure code quality before commits.

## Option 1: Using pre-commit framework (Recommended)

### Install pre-commit

```bash
# Using pip
pip install pre-commit

# Using homebrew (macOS)
brew install pre-commit

# Using conda
conda install -c conda-forge pre-commit
```

### Install the hooks

```bash
pre-commit install
```

### Run hooks manually

```bash
# Run on all files
pre-commit run --all-files

# Run on staged files only
pre-commit run
```

## Option 2: Manual Git Hook

If you prefer not to use the pre-commit framework, you can install the provided shell script:

```bash
# Copy the script to git hooks directory
cp scripts/pre-commit-hook.sh .git/hooks/pre-commit

# Make it executable
chmod +x .git/hooks/pre-commit
```

## What the hooks check

1. **Code formatting** - Ensures code is formatted with `cargo fmt`
2. **Clippy lints** - Runs clippy with warnings treated as errors
3. **Build check** - Verifies the project compiles
4. **File formatting** - Checks TOML/YAML syntax and fixes trailing whitespace

## Bypassing hooks

If you need to bypass the hooks for a specific commit (not recommended):

```bash
git commit --no-verify -m "emergency commit"
```

## Troubleshooting

### Hook fails with "command not found"
Ensure you have Rust toolchain installed and `cargo` is in your PATH.

### Slow pre-commit times
The hooks run `cargo clippy` which compiles the entire project. This is intentional to catch all issues early.

### Updating hook versions
Update `.pre-commit-config.yaml` and run:
```bash
pre-commit autoupdate
```