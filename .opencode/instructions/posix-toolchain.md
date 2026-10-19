## Where things live

- The repository is a POSIX checkout at `/home/brian/git/ai-cli`.
- **Development is supported on POSIX platforms only: Linux and WSL.** A
  Windows shell is not a supported development environment.
- The Rust toolchain lives in `~/.cargo/bin` (cargo, rustc, rustfmt, clippy).
  It is **not** on the default `PATH` of a non-login shell, so use a login
  shell, or source `~/.cargo/env`.

## Running Rust commands

```sh
cd /home/brian/git/ai-cli
cargo test
```

`cargo`, `rustc`, `rustfmt` and `cargo-clippy` are on the `PATH`. If they
are not in a particular shell, use a login shell so `~/.profile` sources
`~/.cargo/env`, or source it directly.

Formatting is handled by the built-in rustfmt formatter, which `opencode.json`
enables. `rustfmt.toml` sets the edition, because rustfmt does not read it
from `Cargo.toml` when it is run outside cargo. `cargo fmt` is unaffected and
takes the edition from `Cargo.toml`.

`gh` is authenticated for this repository, so `gh pr create`,
`gh pr checks`, `gh run watch` and `gh issue create` are all available for
preparing and monitoring a pull request.

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

## Deferred fixes and issue proposals

When you find something that ought to be fixed but is not part of the current
task, do not fix it silently and do not file an issue on your own initiative.

Present it in this shape and ask whether to file it, then wait:

> **Proposed issue**
>
> **Title:** `[BUG] one line summary of the problem`
> **Labels:** `bug`
> **Where:** `path/to/file:line`, or the workflow file
>
> **Problem:** what is wrong, in one or two sentences.
>
> **Impact:** who is affected and how.
>
> **Suggested fix:** the concrete change, with the code or the commands.
>
> **Verification:** how to confirm the fix works.

Once the maintainer approves, file it with:

```sh
gh issue create --title "<title>" --label <label> --body-file <body>
```

- **Never file an issue without explicit approval.** A recommendation is not
  an instruction.
- **Never fix a deferred problem as a side effect of another task.**
- One issue per problem. Do not bundle unrelated findings.
- Title prefixes `[BUG]`, `[FEATURE]`, `[DOCS]`, `[PERF]`, with the matching
  labels `bug`, `enhancement`, `documentation`, `performance`.
- Check what is already tracked first: `gh issue list --state all` and the
  "Known Issues" section of `docs/VERSION_MANAGEMENT.md`. If a problem is
  already filed, refer to that issue instead of proposing a new one.

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

`development` and `main` are protected branches. `development` is protected by
a **ruleset** with no bypass actors, so it applies to administrators too.
Linear history is enforced on both branches, so merge commits are rejected and
the merge style is what differs between them:

- feature branch into `development` — squash (`gh pr merge <pr> --squash`)
- `development` into `main` — rebase (`gh pr merge <pr> --rebase`)

**Never squash a `development` into `main` merge.** `main` is merged back
into `development` after every release by the `Sync Development Branch with
Main` workflow. A squash merge gives `main` a new commit with the same content
as the commits on `development` but a different SHA, so `development` ends up
holding the same change twice, joined by a merge commit, and the duplication
compounds with every release.

A rebase merge keeps `main` linear and, when `main` is a direct ancestor of
`development`, fast-forwards so that both branches hold the exact same commits
with the same SHAs. The sync back to `development` is then a no-op.

**Never merge into `development` or `main` on your own initiative.** The
required flow is:

1. Commit to a feature branch off `development`.
2. Push the branch.
3. Open a pull request against `development`.
4. Post the pull request link to the maintainer.
5. **Stop and wait** for the maintainer to add the `approved` label.
6. Merge only once the `Approved` check is green and the maintainer has asked
   you to merge.

**Never add or remove the `approved` label.** GitHub does not allow the
author of a pull request to approve it, so the maintainer signals approval by
adding that label, and the `Approve Pull Request` workflow turns it into the
`Approved` status check the rulesets require. Removing the label revokes the
approval. Adding the label is the maintainer's action alone.

Also:

- **Never modify branch protection or rulesets** to get past a merge failure.
  If a pull request is blocked for want of a review or an approval, that is
  correct behaviour — report it and let the maintainer approve.
- **Never push directly to `development` or `main`**, and never force-push or
  delete them.
- **Never close someone else's pull request** unprompted.
