# Version Management Guide

This project uses an automated release workflow that keeps `Cargo.toml` and GitHub release tags in sync.

## Primary Release Workflow

Use `.github/workflows/automate-release.yml` with `workflow_dispatch`.

### What it automates

1. Validates the provided version (`MAJOR.MINOR.PATCH`)
2. Creates a release branch from `development`
3. Updates `Cargo.toml` (and refreshes `Cargo.lock`)
4. Opens a PR to `main`
5. Approves the PR (if token permissions allow)
6. Merges the PR to `main`
7. Creates a GitHub Release with tag `vX.X.X`

Publishing that release triggers `.github/workflows/release.yml` to produce production artifacts.

## Required Configuration

### Required repository settings

- GitHub Actions must have write access to contents and pull requests.
- Branch protection rules on `main` must allow the configured automation path (approval + merge).

## How to Run a Release

1. Open **Actions** in GitHub.
2. Run **Automate Release**.
3. Provide `version` (example: `0.3.0`).
4. Monitor the job until completion.

Expected result:

- PR is created and merged into `main`
- `main` contains `version = "0.3.0"` in `Cargo.toml`
- GitHub release `v0.3.0` is created
- Production build workflow runs from the release event

## Troubleshooting

### Approval fails

Cause: The default `GITHUB_TOKEN` cannot satisfy review requirements.

Fix:

- Allow the GitHub Actions bot review to satisfy branch protection, or complete approval manually.
- Confirm branch protection rules allow an automation-driven merge path.

### Merge fails

Cause: Required checks or required review policy not satisfied.

Fix:

- Verify required status checks passed.
- Verify approval requirements are met.

### Version or tag already exists

Cause: Requested version already released or tag already present.

Fix:

- Run with a new version.
- Or clean up the existing tag/release if it was created in error.

## Manual fallback

If automation is blocked by policy, manual flow remains:

1. Create/merge PR to `main` with version update.
2. Create release tag `vX.X.X` in GitHub Releases.
3. Let `.github/workflows/release.yml` build artifacts.
