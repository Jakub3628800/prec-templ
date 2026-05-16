This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Setup and Installation
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build
```

### Testing
```bash
cargo test              # Run all tests
cargo test <test_name>  # Run specific test
```

### Linting and Formatting
```bash
cargo clippy            # Run Rust linter
cargo fmt               # Format code
cargo fmt --check       # Check formatting without modifying
```

### Building for Distribution
```bash
maturin build --release     # Build Python wheel
maturin develop             # Build and install in current virtualenv
```

### Running Locally
```bash
cargo run                   # Generate .pre-commit-config.yaml
cargo run -- --install      # Generate, install hooks, and run pre-commit
cargo run -- -i             # Run in interactive mode
cargo run -- --path /some/dir  # Analyze specific directory
```

## Architecture Overview

This is a Rust CLI tool that auto-generates `.pre-commit-config.yaml` files by detecting technologies in repositories. It is packaged as a Python wheel using maturin for easy installation via pip/uv.

### Source Files (src/)
- **main.rs**: Entry point, orchestrates CLI flow
- **cli.rs**: Command-line argument parsing using clap
- **config.rs**: Configuration struct (serde serialization)
- **discover.rs**: Technology detection via filesystem scanning
- **render.rs**: Template rendering using MiniJinja
- **ui.rs**: Terminal UI (console, dialoguer, indicatif)

### Templates (templates/)
MiniJinja templates for generating YAML configs:
- `base.j2` - Base pre-commit hooks
- `python.j2` - Python-specific hooks (Ruff, optional Pyrefly, optional uv-lock)
- `js.j2` - JavaScript/TypeScript hooks
- `go.j2` - Go hooks
- `docker.j2` - Docker hooks
- `github_actions.j2` - GitHub Actions hooks
- `meta.j2` - Wrapper template with header

## Key Dependencies
- **clap**: CLI argument parsing
- **serde**: Serialization/deserialization
- **minijinja**: Jinja2-compatible template engine
- **console + dialoguer + indicatif**: Terminal UI (Rich equivalent)
- **ignore**: .gitignore-aware file walking

## Building Wheels
```bash
# For development
maturin develop

# For release
maturin build --release

# Install locally
uv tool install dist/prec_templ-*.whl
```

## Entry Point
The binary `prec-templ` is installed as a CLI command when the wheel is installed.
