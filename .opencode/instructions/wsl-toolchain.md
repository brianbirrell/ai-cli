# Working on ai-cli from this environment

## Where things live

- The repository is a **WSL** checkout at `/home/brian/git/ai-cli`.
- The OpenCode server runs on **Windows**, so project files are addressed by
  UNC path (`\\wsl.localhost\Ubuntu-Dev\home\brian\git\ai-cli\...`).
- The Rust toolchain lives in WSL at `~/.cargo/bin` (cargo, rustc, rustfmt,
  clippy). It is **not** on the default `PATH` of a non-login shell.

## Running Rust commands

Always shell out to WSL with a login shell so `~/.profile` sources
`~/.cargo/env` and puts the toolchain on `PATH`:

```sh
wsl -d Ubuntu-Dev -- bash -lc "cd /home/brian/git/ai-cli && cargo test"
```

Do not run `cargo`, `rustc`, or `rustfmt` directly from a Windows shell — they
are not installed there.

The formatter is bridged automatically: `opencode.json` routes `.rs` files
through `/home/brian/.opencode/bin/rustfmt-wsl`, which translates the UNC path
into a WSL path before calling `rustfmt`.

## Project conventions (see `AGENTS.md` for the full list)

| Task                | Command                                |
| ------------------- | -------------------------------------- |
| Quick type check    | `cargo check`                          |
| Lint                | `cargo clippy`                         |
| Format              | `cargo fmt`                            |
| Test                | `cargo test`                           |
| Run the CLI         | `cargo run -- -p "your prompt"`        |

- Branch from `development`; never work directly on `main`.
- Do **not** run `cargo build --release` unless a release build is actually needed.
- Update `README.md` and `docs/` alongside code changes.
- Leave `CHANGELOG.md` alone unless a release is being prepared, and then
  only by running the generator. See the changelog policy below.
- Do not create a pull request unless it is explicitly requested.

## Changelog policy

`CHANGELOG.md` is **generated, not written by hand**.

- **Only** update it when the user asks for a release. Do not touch it for
  ordinary changes, documentation updates, dependency bumps, or pull request
  work.
- When a release is requested, generate the section rather than writing it:

  ```sh
  cargo run --example generate_changelog -- <version>
  ```

  It reads the commits between the previous release tag and `origin/development`,
  groups them by category, and rewrites the `## Unreleased` section as
  `## <version> (<date>)`. Use `--dry-run` to preview first.

- Do not add entries by hand. The generator is the single source of truth, so
  a hand written entry will be lost on the next release.

## Review workflow (mandatory)

`development` and `main` are protected branches. `development` requires **one
approving review** from someone with write access, and `enforce_admins` is on,
so admin rights do not bypass it. Linear history is enforced, so squash merges
are the only accepted merge style.

**Never merge into `development` or `main` on your own initiative.** The
required flow is:

1. Commit to a feature branch off `development`.
2. Push the branch.
3. Open a pull request against `development`.
4. Post the pull request link to the maintainer.
5. **Stop and wait** for an explicit approval before merging.

Also:

- **Never modify branch protection** to get past a merge failure. If a pull
  request is blocked for want of a review, that is correct behaviour — report
  it and let the maintainer approve.
- **Never push directly to `development` or `main`**, and never force-push or
  delete them.
- **Never close someone else's pull request** unprompted.

The `gh` CLI is authenticated for this repository, so `gh pr create`,
`gh pr checks`, and `gh run watch` are all available for preparing and
monitoring a pull request.

