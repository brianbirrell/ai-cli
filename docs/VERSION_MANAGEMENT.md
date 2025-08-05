# Version Management Guide

This guide explains how to keep the version in `Cargo.toml` and GitHub release tags synchronized.

## How Version Sync Works

### Automatic Version Management

The release workflow automatically handles version synchronization:

1. **Version Source**: `Cargo.toml` is the single source of truth
2. **Automatic Bumping**: Patch version is incremented on main branch pushes
3. **Tag Creation**: Git tags are created with the same version as `Cargo.toml`
4. **Verification**: Multiple checks ensure versions stay in sync

### Version Flow

```
Cargo.toml (1.0.0) → Push to main → Auto-bump (1.0.1) → Create tag (v1.0.1) → Release
```

## Workflow Steps

### 1. Version Sync Job (`sync-version-and-tag`)

**Triggers**: Push to main branch
**Purpose**: Updates `Cargo.toml` and creates synchronized tag

**Steps**:
1. **Read Current Version**: Extracts version from `Cargo.toml`
2. **Generate New Version**: Increments patch version
3. **Update Cargo.toml**: Modifies version in file
4. **Commit Changes**: Pushes version bump to main
5. **Create Tag**: Creates git tag with same version
6. **Verify Sync**: Ensures tag and `Cargo.toml` match

### 2. Release Job (`build-and-release`)

**Triggers**: Tag creation (`v*`)
**Purpose**: Builds and releases with version verification

**Steps**:
1. **Verify Version Sync**: Checks tag version matches `Cargo.toml`
2. **Build Release**: Compiles the binary
3. **Create Artifacts**: Packages with versioned names
4. **Create GitHub Release**: Publishes with proper versioning

## Version Synchronization Features

### ✅ Automatic Verification

The workflow includes multiple verification steps:

```bash
# Extract version from tag
TAG_VERSION=${GITHUB_REF#refs/tags/v}

# Get version from Cargo.toml
CARGO_VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)

# Verify they match
if [ "$TAG_VERSION" != "$CARGO_VERSION" ]; then
  echo "❌ Version mismatch!"
  exit 1
fi
```

### ✅ Fail-Safe Design

- **Pre-release check**: Verifies sync before building
- **Post-tag check**: Verifies sync after tag creation
- **Build-time check**: Verifies sync during release build
- **Automatic rollback**: Fails fast if versions don't match

### ✅ Clear Logging

Each step provides detailed logging:

```
Current version from Cargo.toml: 1.0.0
Current version: 1.0.0
New version: 1.0.1
New tag: v1.0.1
✅ Cargo.toml version updated successfully
✅ Version bump committed and pushed
✅ Tag v1.0.1 created and pushed
✅ Final verification passed: 1.0.1
```

## Manual Version Management

### Creating Manual Releases

If you need to create a release manually:

1. **Update Cargo.toml**:
   ```toml
   version = "1.2.3"
   ```

2. **Commit the change**:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 1.2.3"
   git push origin main
   ```

3. **Create tag**:
   ```bash
   git tag -a v1.2.3 -m "Release 1.2.3"
   git push origin v1.2.3
   ```

4. **Verify sync**:
   ```bash
   # Check Cargo.toml version
   grep '^version = ' Cargo.toml
   
   # Check latest tag
   git describe --tags --abbrev=0
   ```

### Version Bumping Strategies

#### Patch Version (Default)
```bash
# Automatic: 1.0.0 → 1.0.1
# For bug fixes and minor improvements
```

#### Minor Version
```bash
# Manual: 1.0.0 → 1.1.0
# For new features (backward compatible)
```

#### Major Version
```bash
# Manual: 1.0.0 → 2.0.0
# For breaking changes
```

## Troubleshooting Version Sync

### Common Issues

#### 1. Version Mismatch Error
```
❌ Version mismatch! Tag version (1.0.1) != Cargo.toml version (1.0.0)
```

**Solution**:
- Check that `Cargo.toml` version matches the tag
- Ensure the tag was created from the correct commit
- Manually sync if needed

#### 2. Tag Already Exists
```
fatal: tag 'v1.0.1' already exists
```

**Solution**:
- Delete the existing tag: `git tag -d v1.0.1`
- Push the deletion: `git push origin :refs/tags/v1.0.1`
- Re-run the workflow

#### 3. Workflow Fails on Version Bump
```
❌ Failed to update Cargo.toml version
```

**Solution**:
- Check `Cargo.toml` syntax
- Ensure the version line format is correct
- Verify file permissions

### Debugging Commands

```bash
# Check current version in Cargo.toml
grep '^version = ' Cargo.toml

# Check latest tag
git describe --tags --abbrev=0

# Check all tags
git tag --sort=-version:refname

# Verify tag points to correct commit
git show v1.0.1

# Check if versions match
TAG_VERSION=$(git describe --tags --abbrev=0 | sed 's/^v//')
CARGO_VERSION=$(grep '^version = ' Cargo.toml | cut -d'"' -f2)
echo "Tag: $TAG_VERSION, Cargo: $CARGO_VERSION"
```

## Best Practices

### ✅ Do's
- Let the workflow handle automatic versioning
- Use semantic versioning (MAJOR.MINOR.PATCH)
- Verify versions before manual releases
- Keep `Cargo.toml` as the single source of truth

### ❌ Don'ts
- Don't manually create tags without updating `Cargo.toml`
- Don't edit version in `Cargo.toml` without creating a tag
- Don't skip version verification steps
- Don't use non-semantic versioning

### Version Format Standards

```toml
# ✅ Correct format
version = "1.2.3"

# ❌ Incorrect formats
version = "1.2.3.0"
version = "v1.2.3"
version = "1.2.3-beta"
```

## Monitoring Version Sync

### GitHub Actions Logs

Check the workflow logs for version sync status:

1. Go to Actions tab
2. Click on the latest workflow run
3. Look for version sync messages:
   - ✅ "Version sync verified"
   - ✅ "Cargo.toml version updated successfully"
   - ✅ "Final verification passed"

### Automated Checks

The workflow includes automated checks:

- **Pre-build verification**: Ensures tag matches `Cargo.toml`
- **Post-tag verification**: Confirms sync after tag creation
- **Build-time verification**: Double-checks during release build

## Advanced Configuration

### Custom Version Bumping

To modify the version bumping logic, edit the workflow:

```yaml
# In .github/workflows/release.yml
- name: Generate new version
  run: |
    # Custom version bumping logic here
    # Example: Increment minor version instead of patch
    NEW_MINOR=$((MINOR + 1))
    NEW_VERSION="$MAJOR.$NEW_MINOR.0"
```

### Version Validation

Add custom validation rules:

```yaml
- name: Validate version format
  run: |
    VERSION=${{ steps.version.outputs.new_version }}
    if ! [[ $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
      echo "❌ Invalid version format: $VERSION"
      exit 1
    fi
```

This version synchronization system ensures your `Cargo.toml` and GitHub release tags always stay in sync! 🎯 