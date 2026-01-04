# run

A lightweight task runner for defining and executing shell commands with a clean, readable syntax. Define functions in a `Runfile` (or `~/.runfile`) and call them from the command line to streamline your development workflow.

[![Crates.io](https://img.shields.io/crates/v/devrun.svg)](https://crates.io/crates/devrun)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

#### Why use `run`?

It hits a common sweet spot — lightweight, readable, and shell-native for quick CLI automation without the overhead of heavier task systems.

- **Familiar Syntax:** Low onboarding cost for anyone comfortable with a shell.
- **Block Functions (`{}`):** Clean multi-statement definitions without messy shell escaping.
- **Nested Names:** Organize commands logically (e.g., `db:up`, `db:down`).
- **REPL:** An interactive mode for rapid iterative development.
- **Dual Scope:** Global (`~/.runfile`) and project-specific (`./Runfile`) logic.

---

## Installation

### macOS / Linux
```sh
brew tap nihilok/tap
brew install devrun
# OR
cargo install devrun
```

### Windows

```powershell
# Via Scoop
scoop bucket add nihilok https://github.com/nihilok/scoop-bucket
scoop install devrun

# OR via Cargo
cargo install devrun
```

### Tab Completions (macOS / Linux / WSL)

```sh
run --install-completion  # Auto-detects bash/zsh/fish
```

---

## Migration Guide

If you are coming from other tools, here is how `run` compares and how to translate your existing workflows.

| Feature | `make` | `just` | `run` |
| --- | --- | --- | --- |
| **Primary Goal** | Build system / Dependencies | Command runner | Shell-native task orchestrator |
| **Syntax** | Tab-indented Makefile | Custom DSL | Bash-like functions |
| **Namespacing** | None (flat) | Limited | Native (using `:` to space) |
| **Interactive** | No | No | **Yes (REPL mode)** |

### From `make` to `run`

In `make`, you use `target: dependencies \n \t command`. In `run`, you focus on the action: `build() cargo build`. No more tab-indentation errors or `.PHONY` declarations.

### From `just` to `run`

In `just`, recipes are defined like `recipe: \n    command`. In `run`, you get native shell blocks: `task() { cmd1; cmd2; }`. Additionally, `run` handles nested namespaces more gracefully: `run docker build` maps directly to `docker:build()`.

---

## Quick Start

Create a `Runfile` in your project root:

```runfile
# Simple one-liners
build() cargo build --release
test() cargo test

# Multi-statement blocks
ci() {
    echo "Running CI pipeline..."
    cargo fmt -- --check
    cargo clippy
    cargo test
    echo "All checks passed!"
}

# Namespaced commands with arguments
docker:shell() docker compose exec $1 bash
git:commit() git add . && git commit -m "$1" && echo "${2:-Done}"
```

**Execute commands:**

```sh
run build
run docker shell web
run git commit "Initial commit"
```

---

## Tips & Tricks

* **The "Colon" Shortcut:** If you define `web:deploy()`, you can run it as `run web:deploy` OR `run web deploy`. The latter makes your CLI feel like a first-class tool.
* **Default Arguments:** Use `${1:-default_value}` to make arguments optional.
* **Global Quality of Life:** Put your most-used utility commands in `~/.runfile`. They will be available in every directory.
* **The REPL for Debugging:** If you are building a complex chain of commands, just type `run` to enter the REPL. Test your functions without restarting the process.
* **Shell Overrides:** Use the `RUN_SHELL` environment variable to switch engines (e.g., `RUN_SHELL=zsh run task`).

---

## Contributing & Roadmap

We welcome contributions! Here is what is currently on the horizon for `run`:

1. **Task Dependencies:** Internal function calling (e.g., `deploy()` automatically triggers `build()`).
2. **`.env` Support:** Automatic loading of environment variables from a local `.env` file.
3. **Watch Mode:** A built-in `--watch` flag to trigger functions on file system changes.
4. **Private Functions:** Support for "hidden" tasks (e.g., `_setup()`) that don't appear in the `--list` view.

### How to Contribute

1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/AmazingFeature`).
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`).
4. Push to the branch (`git push origin feature/AmazingFeature`).
5. Open a Pull Request.

---

## License

MIT
