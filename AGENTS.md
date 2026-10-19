# AGENTS Guidelines for This Repository

This repository contains a Rust CLI application for interacting with AI models. When
working on the project interactively with an agent (e.g. the Codex CLI) please follow
the guidelines below so that the development experience continues to work smoothly.

## 1. Use Cargo for Development, **not** manual compilation

* **Always create a specific branch** for any changes.  Use the 'development' branch as the
  base.
* **Create or update unit tests** for any code changes and always verify they are passing.
* **Do _not_ create a PR** for any changes unless requested.
* **Never merge into `development` or `main` without an explicit approval.** See
  section 2, *Branch Protection and Review Requirements*.
* **Update documentation** including README.md and markdown files in the 'docs' directory.
* **Do _not_ update CHANGELOG.md** unless a release is being prepared, and then
  only by running `cargo run --example generate_changelog --`. See section 3,
  *Changelog*.
* **Always use `cargo run` for testing the CLI application** while iterating on the
  code. This ensures proper dependency resolution and incremental compilation.
* **Use `cargo check` for quick syntax checking** without full compilation.
* **Do _not_ run `cargo build --release` inside the agent session** unless specifically
  needed for performance testing. The release build takes longer and may not be necessary
  for development iterations.

## 2. Branch Protection and Review Requirements (Mandatory)

The `development` and `main` branches are protected. These rules apply to every
change, including changes made by an automated agent.

### Protected branch rules

| Branch       | Required review | Required check     | Merge style | Direct push |
| ------------ | --------------- | ------------------ | ----------- | ----------- |
| `development` | 1 approval      | `Test Development` | Squash only | Not allowed |
| `main`       | 0 approvals     | `Test`             | Squash only | Not allowed |

`enforce_admins` is enabled on `development`, so administrator privileges do
**not** bypass the review requirement. Force pushes and branch deletion are
disabled on both branches. Linear history is enforced on both, so merge
commits are rejected.

### What you must do

1. **Never merge into `development` or `main` on your own initiative.** Open a
   pull request, give the reviewer the link, and then stop and wait for an
   explicit approval before merging.
2. **Never modify branch protection to work around a merge failure.** If a pull
   request cannot be merged because a review is missing, that is the system
   working as intended. Report it and ask the maintainer to approve it, or to
   relax the rule themselves if that is what they want.
3. **Never delete, force-push, or otherwise rewrite a protected branch.**
4. **Use squash merges.** Linear history is enforced, so `--merge` is rejected
   by the API.
5. **Do not close other people's pull requests** without being asked to.

### Review hand-off

When you open a pull request, include in the pull request body:

* A summary of the change and the reason for it.
* The verification you performed (tests, lints, formatting).
* Any risk, trade-off, or follow-up work.

Then post the pull request link to the maintainer and wait. Do not merge, do
not push further commits to the branch, and do not start a new pull request
against the same target while the first one is awaiting review.

### Releasing

`development` is the base for release work. Prepare the release on
`development`, promote it to `main` through a pull request, and publish the
release with a tag.

The release preparation steps are:

1. Bump `version` in `Cargo.toml` and refresh `Cargo.lock` with `cargo check`.
2. Generate the changelog section with
   `cargo run --example generate_changelog -- <version>` (see section 3).
3. Update the README release notes and the current release line.
4. Commit on `development`, open a pull request to `main`, and wait for review.
5. Merge the pull request with a squash merge.
6. Publish the release with
   `gh release create v<version> --target main --title "v<version>" --generate-notes`.
   Publishing triggers the `Release` workflow, which builds the artifacts and
   uploads them.

The `Automate Release` workflow is currently unreliable because its commit
step has nothing to commit when the version has already been bumped, and the
`sync-version-and-tag` job in `release.yml` is unreachable because that
workflow only triggers on `release: published`. Confirm the intended flow with
the maintainer before cutting a release.

`Sync Development Branch with Main` merges `main` into `development` on every
push to `main`. It pushes directly to a protected branch, so it fails while
branch protection is in place. If the branches have diverged, ask the
maintainer to reconcile them rather than pushing to `development` yourself.

## 3. Changelog

`CHANGELOG.md` is **generated, not written by hand**.

* **When a release is being prepared**, update `CHANGELOG.md` by running the
  generator, which derives the release section from the commit history of the
  `development` branch:

  ```sh
  cargo run --example generate_changelog -- <version>
  ```

  The generator reads the commits between the previous release tag and
  `origin/development`, groups them by category, and rewrites the `## Unreleased`
  section as `## <version> (<date>)`. Use `--dry-run` to preview and
  `--from-tag` if the previous release tag needs overriding. It refuses to
  write an empty section or to duplicate an existing version.

  The generator lives in `examples/generate_changelog.rs`. Examples are not
  installed by `cargo install` and are not part of the release artifacts, so
  it adds no runtime dependency and no extra binary to ship. It does not
  require anything beyond the Rust toolchain and `git`.

* **At all other times, do not touch `CHANGELOG.md`.** It is not updated for
  ordinary changes, documentation updates, dependency bumps, or pull request
  work. It changes only when a release is cut.

Entries link to the pull request when the commit subject carries a `(#NNN)`
reference, and to the commit otherwise. The generator is the single source of
truth, so a hand written entry that it does not produce will be lost on the
next release.

## 4. Keep Dependencies in Sync

If you add or update dependencies remember to:

1. Update `Cargo.toml` with the new dependencies.
2. Run `cargo update` to update `Cargo.lock` with the latest compatible versions.
3. Test the changes with `cargo run` to ensure everything still works.

## 5. Coding Conventions

* Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types).
* Use `anyhow` for error handling and `Result<T>` return types.
* Prefer `async/await` for I/O operations.
* Use `clap` for command-line argument parsing.
* Add appropriate logging with the `log` crate.
* Write unit tests in the same file using `#[cfg(test)]` modules.

## 6. Testing Guidelines

* **Always run `cargo test` after making changes** to ensure existing functionality
  still works.
* **Use `cargo test --verbose` for detailed test output** when debugging test failures.
* **Write tests for new functionality** to maintain code quality.

## 7. Configuration Management

* Configuration files are stored in `~/.config/ai-cli/config.toml`.
* Use the `toml` crate for configuration file parsing.
* Provide sensible defaults in the `AppConfig::default()` implementation.
* Allow command-line arguments to override configuration file settings.

## 8. Useful Commands Recap

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

## 9. Development Workflow

1. **Make changes to the code**
2. **Run `cargo check`** to verify syntax
3. **Run `cargo test`** to ensure tests pass
4. **Test with `cargo run`** to verify functionality
5. **Iterate as needed**
6. **Lint with 'cargo clippy'** to enforce proper syntex
7. **Format with 'cargo fmt'** to maintain coding style
8. **Open a pull request** against `development` and wait for review (see section 2)

## 10. Logging and Debugging

* Use the `log` crate for structured logging.
* Set appropriate log levels: `trace`, `debug`, `info`, `warn`, `error`.
* Use `-v` for debug logging, `-vv` for trace logging.
* Log important operations and error conditions.

## 11. Error Handling

* Use `anyhow::Result<T>` for functions that can fail.
* Provide meaningful error messages with context.
* Use `anyhow::Context` for adding context to errors.
* Handle network timeouts and API errors gracefully.

---

Following these practices ensures that the agent-assisted development workflow stays
fast and dependable for this Rust CLI project. When in doubt, run `cargo check` and
`cargo test` to verify your changes.
