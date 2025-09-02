# AGENTS Guidelines for This Repository

This repository contains a Rust CLI application for interacting with AI models. When
working on the project interactively with an agent (e.g. the Codex CLI) please follow
the guidelines below so that the development experience continues to work smoothly.

## 1. Use Cargo for Development, **not** manual compilation

* **Always use `cargo run` for testing the CLI application** while iterating on the
  code. This ensures proper dependency resolution and incremental compilation.
* **Use `cargo check` for quick syntax checking** without full compilation.
* **Do _not_ run `cargo build --release` inside the agent session** unless specifically
  needed for performance testing. The release build takes longer and may not be necessary
  for development iterations.
* **Always create a specific branch** for any changes.  Use the 'development' branch as the
  base.
* **Do not create a PR** for any changes unless requested.

## 2. Keep Dependencies in Sync

If you add or update dependencies remember to:

1. Update `Cargo.toml` with the new dependencies.
2. Run `cargo update` to update `Cargo.lock` with the latest compatible versions.
3. Test the changes with `cargo run` to ensure everything still works.

## 3. Coding Conventions

* Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types).
* Use `anyhow` for error handling and `Result<T>` return types.
* Prefer `async/await` for I/O operations.
* Use `clap` for command-line argument parsing.
* Add appropriate logging with the `log` crate.
* Write unit tests in the same file using `#[cfg(test)]` modules.

## 4. Testing Guidelines

* **Always run `cargo test` after making changes** to ensure existing functionality
  still works.
* **Use `cargo test --verbose` for detailed test output** when debugging test failures.
* **Write tests for new functionality** to maintain code quality.

## 5. Configuration Management

* Configuration files are stored in `~/.config/ai-cli/config.toml`.
* Use the `toml` crate for configuration file parsing.
* Provide sensible defaults in the `AppConfig::default()` implementation.
* Allow command-line arguments to override configuration file settings.

## 6. Useful Commands Recap

| Command                | Purpose                                            |
| ---------------------- | -------------------------------------------------- |
| `cargo run`            | Run the CLI application with default arguments.    |
| `cargo run -- --help`  | Show CLI help and available options.               |
| `cargo check`          | Quick syntax and type checking without compilation.|
| `cargo clippy`         | Syntex checking with lint.                         |
| `cargo fmt `           | Format code to maintain coding style.              |
| `cargo test`           | Run the test suite.                                |
| `cargo test --verbose` | Run tests with detailed output.                    |
| `cargo build`          | Build the application in debug mode.               |
| `cargo build --release`| **Release build – _use sparingly during development_** |

## 7. Development Workflow

1. **Make changes to the code**
2. **Run `cargo check`** to verify syntax
3. **Run `cargo test`** to ensure tests pass
4. **Test with `cargo run`** to verify functionality
5. **Iterate as needed**
6. **Lint with 'cargo clippy'** to enforce proper syntex
7. **Format with 'cargo fmt'** to maintain coding style

## 8. Logging and Debugging

* Use the `log` crate for structured logging.
* Set appropriate log levels: `trace`, `debug`, `info`, `warn`, `error`.
* Use `-v` for debug logging, `-vv` for trace logging.
* Log important operations and error conditions.

## 9. Error Handling

* Use `anyhow::Result<T>` for functions that can fail.
* Provide meaningful error messages with context.
* Use `anyhow::Context` for adding context to errors.
* Handle network timeouts and API errors gracefully.

---

Following these practices ensures that the agent-assisted development workflow stays
fast and dependable for this Rust CLI project. When in doubt, run `cargo check` and
`cargo test` to verify your changes.
