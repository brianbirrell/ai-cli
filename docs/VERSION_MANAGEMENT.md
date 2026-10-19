# Version Management Guide

This project keeps `Cargo.toml`, `Cargo.lock`, and GitHub release tags in sync.

## Branch Protection and Review

`development` and `main` are protected branches. See `AGENTS.md` for the full
rules, which apply to every change including automated ones.

| Branch       | Required review | Required check     | Merge style | Direct push |
| ------------ | --------------- | ------------------ | ----------- | ----------- |
| `development` | 1 approval      | `Test Development` | Squash only | Not allowed |
| `main`       | 0 approvals     | `Test`             | Squash only | Not allowed |

`enforce_admins` is enabled on `development`, so administrator privileges do
not bypass the review requirement. A pull request into `development` is
therefore reviewed by a human before it lands.

## Primary Release Workflow

The release is prepared on `development`, promoted to `main` through a pull
request, and then published with a tag.

### 1. Prepare the release on `development`

- Bump `version` in `Cargo.toml` and refresh `Cargo.lock` (`cargo check`).
- Generate the changelog section from the commit history:

  ```sh
  cargo run --example generate_changelog -- 0.3.2
  ```

  The generator reads the commits between the previous release tag and
  `origin/development`, groups them by category, and rewrites the
  `## Unreleased` section as `## <version> (<date>)`. It refuses to write an
  empty section or to duplicate an existing version.

  Useful options: `--dry-run` to preview, `--from-tag` to override the
  previous release tag, `--date` to set the release date, and `--branch` to
  read history from a branch other than `origin/development`.

  The generator is a cargo example, so it needs nothing beyond the Rust
  toolchain and `git`. It is not installed by `cargo install` and is not part
  of the release artifacts.

  `CHANGELOG.md` is only touched here, when a release is being cut. It is not
  updated for ordinary changes.

- Update the README release notes and the current release line.

### 2. Promote to `main`

- Open a pull request from `development` to `main`.
- Wait for the required review, then merge with a **rebase** merge:

  ```sh
  gh pr merge <pr> --rebase
  ```

  **Never squash this merge.** `main` is merged back into `development` after
  every release by the `Sync Development Branch with Main` workflow. A squash
  merge gives `main` a new commit with the same content but a different SHA,
  so `development` would hold the same change twice, joined by a merge commit,
  and the duplication would compound with every release.

  A rebase merge keeps the history linear, which `main` requires, and when
  `main` is a direct ancestor of `development` it fast-forwards, so `main`
  holds the exact same commits with the same SHAs. The sync back to
  `development` is then a no-op.

  Squash merges remain correct for pull requests from a feature branch into
  `development`, where the squashed commit is carried across to `main` by the
  rebase.

### 3. Publish the release

```sh
gh release create vX.Y.Z --target main --title "vX.Y.Z" --generate-notes
```

Publishing the release triggers `.github/workflows/release.yml`. Its
`Build and Release` job verifies that the tag version matches `Cargo.toml`,
checks out a clean tree, builds the release binary, and uploads the `tar.gz`,
`zip`, and `deb` artifacts to the release.

### 4. Verify

```sh
gh release view vX.Y.Z --json assets
gh run list --workflow release.yml --limit 1
```

The release should show three assets, and the `Release` workflow run should
have completed successfully.

## Automated Release Workflow

`.github/workflows/automate-release.yml` can be run manually from the Actions
tab with a `workflow_dispatch` `version` input. It validates the version,
bumps `Cargo.toml` and `Cargo.lock`, commits and pushes to `main`, runs the
test suite, builds the artifacts, and publishes the release in one job.

**Limitation:** the workflow's commit step fails when `Cargo.toml` already
contains the target version, because there is nothing left to commit. Use it
only when `Cargo.toml` still holds the previous version, or use the primary
workflow above.

## Known Issues

- **`sync-version-and-tag` in `release.yml` cannot run.** The workflow's only
  trigger is `release: published`, so the `push` condition on that job is
  unreachable. It has never created a version bump or a tag.
- **`sync-development.yml` fails while branch protection is in place.** It
  merges `main` into `development` by pushing directly to the protected
  `development` branch, and GitHub rejects the push with `GH006`. If the
  branches have diverged, the maintainer must reconcile them.
- **Archive layouts differ.** The `tar.gz` contains a flat `ai-cli` binary
  while the `zip` contains `artifacts/ai-cli`, because the workflow tars the
  contents of `artifacts/` but zips the directory itself.

## Troubleshooting

### Version validation fails

Cause: Version format is not `MAJOR.MINOR.PATCH`.

Fix:

- Provide version in format `1.2.3` (not `v1.2.3`, not `1.2.3-beta`, etc.)

### Tag already exists

Cause: Release version was already created.

Fix:

- Use a new version number.
- Or delete the existing tag and release if created in error.

### Commit to main fails

Cause: main branch has branch protection preventing direct commits.

Fix:

- Allow GitHub Actions to push commits to main.
- Or disable branch protection for Actions pushes on this workflow.

### A pull request into `development` is blocked

Cause: `development` requires one approving review, and `enforce_admins` is
enabled.

Fix:

- Ask the maintainer to review and approve the pull request.
- Do not modify the branch protection to work around this.

### Release binary shows `-dirty`

Cause: Build metadata marked the tree as dirty at compile time.

Notes:

- `ai-cli --version` appends `-dirty` when tracked Git files differ from `HEAD` during `cargo build`.
- Untracked files should not affect release metadata.

Fix:

- Ensure release workflow uses a clean checkout before build.
- Add a pre-build cleanliness check:
  - `git update-index -q --refresh`
  - `git diff-index --quiet --ignore-submodules=dirty HEAD --`
- Re-run the release workflow to rebuild artifacts.

## Manual fallback

If automation is blocked by policy:

1. Manually update `Cargo.toml` and `Cargo.lock` version on `main`.
2. Manually commit and push to `main`.
3. Publish the release with `gh release create`; the `Release` workflow builds
   and uploads the artifacts.
4. If the workflow cannot run, build the artifacts locally with
   `cargo build --release` and `cargo deb --no-build`, then upload them with
   `gh release upload`.
