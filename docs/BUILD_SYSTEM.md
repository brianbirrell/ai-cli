# Build System Documentation

## Overview

The build system provides:
- **Pre-release builds** from the development branch
- **Release builds** from the main branch
- **Automatic versioning and tagging**
- **Multiple artifact formats** (tar.gz and zip)
- **Comprehensive testing** on all branches

## Workflow Files

### 1. `ci.yml` - Main CI/CD Pipeline
- **Triggers**: Push/PR to `main` or `development` branches
- **Jobs**:
  - `test`: Runs all tests and linting
  - `build-development`: Creates development artifacts when pushing to development branch
  - `build-release`: Creates release artifacts when pushing to main branch

### 2. `release.yml` - Release Management
- **Triggers**: Push to `main` branch or tag creation
- **Jobs**:
  - `build-and-release`: Builds and creates GitHub releases from tags
  - `auto-version`: Automatically bumps version and creates tags on main branch pushes

### 3. `development.yml` - Development Builds
- **Triggers**: Push/PR to `development` branch
- **Jobs**:
  - `test-development`: Runs tests on development branch
  - `build-pre-release`: Creates pre-release artifacts with commit info
  - `create-development-release`: Creates GitHub pre-releases for testing

### 4. `rust.yml` - Enhanced Rust CI
- **Triggers**: Push/PR to `main` or `development` branches
- **Jobs**:
  - `test`: Comprehensive testing including formatting and clippy checks

### 5. `build_release.yml` - Legacy Release Support
- **Triggers**: Manual release creation or workflow dispatch
- **Jobs**:
  - `build`: Legacy release build for manual releases

## Branch Strategy

### Development Branch
- Used for feature development and testing
- Creates pre-release builds with commit-based versioning
- Generates artifacts with format: `ai-cli-development-YYYY-MM-DD-commit.tar.gz`
- Creates GitHub pre-releases for easy testing

### Main Branch
- Used for stable releases
- Automatically bumps version numbers
- Creates official releases with semantic versioning
- Generates artifacts with format: `ai-cli-vX.Y.Z.tar.gz`

## Version Management

### Automatic Versioning
- Version is stored in `Cargo.toml`
- Auto-version job increments patch version on main branch pushes
- Creates tags in format: `vX.Y.Z`
- Automatically creates GitHub releases from tags

### Manual Versioning
- Can manually create tags: `git tag v1.2.3`
- Legacy workflow supports manual release creation
- Supports workflow dispatch for manual builds

## Artifact Formats

### Development Builds
- `ai-cli-development-YYYY-MM-DD-commit.tar.gz`
- `ai-cli-development-YYYY-MM-DD-commit.zip`
- Includes commit hash for traceability

### Release Builds
- `ai-cli-vX.Y.Z.tar.gz`
- `ai-cli-vX.Y.Z.zip`
- Uses semantic versioning

## Usage Examples

### Development Workflow
1. Create feature branch from development
2. Make changes and test locally
3. Push to development branch
4. GitHub Actions creates pre-release build
5. Test pre-release artifacts
6. Merge to main when ready

### Release Workflow
1. Push to main branch
2. Auto-version job bumps version
3. Creates tag automatically
4. Release job builds and creates GitHub release
5. Artifacts available for download

### Manual Release
1. Create tag manually: `git tag v1.2.3`
2. Push tag: `git push origin v1.2.3`
3. Legacy workflow creates release artifacts

## Configuration

### Required Secrets
- `GITHUB_TOKEN`: Automatically provided by GitHub Actions

### Optional Configuration
- Modify version bumping logic in `release.yml`
- Adjust artifact retention periods
- Customize release notes generation

## Monitoring

### Workflow Status
- Check Actions tab in GitHub repository
- Monitor artifact uploads and releases
- Review logs for any build failures

### Release Tracking
- Development builds: GitHub pre-releases
- Stable releases: GitHub releases with semantic versioning
- Artifacts: Available for download from releases

## Troubleshooting

### Common Issues
1. **Version conflicts**: Ensure Cargo.toml version is correct
2. **Build failures**: Check Rust toolchain and dependencies
3. **Release creation**: Verify GITHUB_TOKEN permissions
4. **Artifact upload**: Check file paths and permissions

### Debugging
- Enable verbose logging in workflows
- Check workflow logs for detailed error messages
- Verify branch protection rules and permissions

## Future Enhancements

Potential improvements to consider:
- Multi-platform builds (Windows, macOS, Linux)
- Docker container builds
- Automated changelog generation
- Release notes from commit messages
- Integration with package managers (cargo install, etc.) 