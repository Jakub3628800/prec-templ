# prec-templ

Automatically detects technologies in your repository and generates an appropriate `.pre-commit-config.yaml` file with relevant hooks.

`prec-templ` is opinionated to my preferred pre-commit hook stack.
It is written in Rust and distributed as a Python package so it fits Python workflow tooling conventions (`ruff`, `pre-commit`, `uv`).

Repository: https://github.com/Jakub3628800/prec-templ
Latest release: https://github.com/Jakub3628800/prec-templ/releases/latest

## Quick Start

```bash
uv tool run --isolated --from "git+https://github.com/Jakub3628800/prec-templ@master" prec-templ
```

## Usage

```bash
prec-templ                  # generate .pre-commit-config.yaml
prec-templ --install        # generate, install hooks, and run pre-commit on all files
prec-templ --generate-only  # compatibility alias for the default generation behavior
prec-templ -i               # interactively customize and print YAML to stdout
prec-templ --path /repo     # analyze a specific directory
```

`prec-templ` will:
1. Scan your repository for technologies (Python, JavaScript, Go, Docker, etc.)
2. Generate a `.pre-commit-config.yaml` file with appropriate hooks
3. Install and run pre-commit hooks only when `--install` is passed

Python projects get Ruff by default. Pyrefly is included only when a `pyrefly.toml`
file is present or when enabled interactively.
JavaScript projects get Prettier by default; ESLint is generated only when a
flat `eslint.config.*` file is present.

## License

MIT License
