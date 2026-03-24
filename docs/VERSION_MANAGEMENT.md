# Version Management Guide

This project uses an automated release workflow that keeps `Cargo.toml` and GitHub release tags in sync.

## Primary Release Workflow

Use `.github/workflows/automate-release.yml` with `workflow_dispatch`.

### What it automates

1. Validates the provided version (`MAJOR.MINOR.PATCH`)
2. Checks out main branch
3. Updates `Cargo.toml` and refreshes `Cargo.lock`
4. Commits and pushes version bump directly to main
5. Builds release artifacts (binary, tar.gz, zip, deb)
6. Creates a GitHub Release with tag `vX.X.X` and uploads artifacts
7. GitHub release publication optionally triggers production workflows

## Required Configuration

### Required repository settings

- GitHub Actions must have write access to contents.
- main branch should allow direct commits from GitHub Actions (no PR required).

## How to Run a Release

1. Open **Actions** in GitHub.
2. Run **Automate Release**.
3. Provide `version` (example: `0.3.0`).
4. Monitor the job until completion.

Expected result:

- main branch `Cargo.toml` contains `version = "0.3.0"`
- main branch is pushed with release commit
- Release tag `v0.3.0` is created
- GitHub release `v0.3.0` is published with artifacts (tar.gz, zip, deb)
- Any dependent workflows trigger from the release event

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

## Manual fallback

If automation is blocked by policy:

1. Manually update `Cargo.toml` version on main.
2. Manually commit and push to main.
3. Run `cargo build --release` and create artifacts.
4. Create a GitHub release with tag `vX.X.X` and upload artifacts manually.

