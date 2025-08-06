# Manual Workflow Triggers

This guide explains how to manually trigger the development and release workflows when needed.

## Available Workflows

### 1. Development Build Workflow
- **File**: `.github/workflows/development.yml`
- **Purpose**: Creates pre-release builds from development branch
- **Trigger**: Manual via GitHub Actions UI

### 2. Release Workflow
- **File**: `.github/workflows/release.yml`
- **Purpose**: Creates official releases with versioning
- **Trigger**: Manual via GitHub Actions UI

### 3. CI/CD Pipeline
- **File**: `.github/workflows/ci.yml`
- **Purpose**: Comprehensive testing and artifact creation
- **Trigger**: Manual via GitHub Actions UI

## How to Manually Trigger Workflows

### Method 1: GitHub Actions UI (Recommended)

1. **Navigate to Actions Tab**
   - Go to your repository on GitHub
   - Click on the "Actions" tab

2. **Select Workflow**
   - Choose the workflow you want to run:
     - "Development Build" for pre-release builds
     - "Release" for official releases
     - "CI/CD Pipeline" for comprehensive testing

3. **Trigger Manual Run**
   - Click the "Run workflow" button (blue button on the right)
   - Select the branch you want to run from (main, development, etc.)
   - Click "Run workflow"

### Method 2: GitHub CLI

```bash
# Trigger development build workflow
gh workflow run "Development Build" --ref development

# Trigger release workflow
gh workflow run "Release" --ref main

# Trigger CI/CD pipeline
gh workflow run "CI/CD Pipeline" --ref main
```

### Method 3: REST API

```bash
# Get workflow ID first
gh api repos/:owner/:repo/actions/workflows

# Trigger workflow
gh api repos/:owner/:repo/actions/workflows/:workflow_id/dispatches \
  -f ref=main
```

## When to Use Manual Triggers

### Development Build Workflow
- ✅ **Testing pre-release features** before merging to main
- ✅ **Creating development artifacts** for testing
- ✅ **Debugging build issues** on development branch
- ✅ **Generating pre-release builds** for stakeholders

### Release Workflow
- ✅ **Creating releases** without pushing to main
- ✅ **Testing release process** before production
- ✅ **Creating hotfix releases** manually
- ✅ **Debugging release issues**

### CI/CD Pipeline
- ✅ **Testing changes** on any branch
- ✅ **Running comprehensive tests** manually
- ✅ **Creating artifacts** for any branch
- ✅ **Debugging CI/CD issues**

## Workflow-Specific Instructions

### Development Build Workflow

**What it does:**
- Runs tests on development branch
- Creates pre-release artifacts with commit info
- Creates GitHub pre-releases for testing
- Generates both `.tar.gz` and `.zip` formats

**Manual trigger:**
```bash
gh workflow run "Development Build" --ref development
```

**Expected output:**
- Pre-release artifacts: `ai-cli-development-YYYY-MM-DD-commit.tar.gz`
- GitHub pre-release with commit information
- 30-day artifact retention

### Release Workflow

**What it does:**
- Builds release artifacts from main branch
- Creates GitHub releases with versioning
- Generates both `.tar.gz` and `.zip` formats
- Supports automatic version bumping

**Manual trigger:**
```bash
gh workflow run "Release" --ref main
```

**Expected output:**
- Release artifacts: `ai-cli-vX.Y.Z.tar.gz` and `ai-cli-vX.Y.Z.zip`
- GitHub release with semantic versioning
- Automatic release notes

### CI/CD Pipeline

**What it does:**
- Runs comprehensive tests
- Creates development artifacts on development branch
- Creates release artifacts on main branch
- Includes dependency caching

**Manual trigger:**
```bash
gh workflow run "CI/CD Pipeline" --ref main
# or
gh workflow run "CI/CD Pipeline" --ref development
```

## Troubleshooting Manual Triggers

### Common Issues

1. **Workflow not appearing in UI**
   - Ensure the workflow file is in `.github/workflows/`
   - Check that `workflow_dispatch` is in the `on:` section
   - Verify the workflow file has valid YAML syntax

2. **Permission denied**
   - Ensure you have write access to the repository
   - Check repository settings for workflow permissions

3. **Branch not available**
   - Ensure the branch exists in the repository
   - Check that the workflow is configured for the branch

4. **Workflow fails**
   - Check the Actions tab for detailed error logs
   - Verify all required secrets are configured
   - Ensure the repository has proper permissions

### Debugging Steps

1. **Check workflow syntax**
   ```bash
   # Validate YAML syntax
   yamllint .github/workflows/*.yml
   ```

2. **Verify triggers**
   ```bash
   # Check workflow configuration
   cat .github/workflows/development.yml | grep -A 5 "on:"
   ```

3. **Test locally**
   ```bash
   # Test workflow locally (if using act)
   act workflow_dispatch -W .github/workflows/development.yml
   ```

## Best Practices

### ✅ Do's
- Use manual triggers for testing and debugging
- Run workflows on appropriate branches
- Monitor workflow logs for issues
- Keep workflow files up to date

### ❌ Don'ts
- Don't trigger workflows unnecessarily
- Don't run production workflows on development branches
- Don't ignore workflow failures
- Don't modify workflow files without testing

## Monitoring Manual Triggers

### Check Workflow Status
```bash
# List recent workflow runs
gh run list --workflow="Development Build"

# Get detailed status
gh run view <run-id>

# Download artifacts
gh run download <run-id>
```

### View Logs
- Go to Actions tab in GitHub
- Click on the workflow run
- View detailed logs for each step
- Download artifacts if needed

## Examples

### Example 1: Test Development Build
```bash
# Trigger development build manually
gh workflow run "Development Build" --ref development

# Check status
gh run list --workflow="Development Build" --limit 1
```

### Example 2: Create Manual Release
```bash
# Trigger release workflow
gh workflow run "Release" --ref main

# Monitor the release
gh release list
```

### Example 3: Test CI/CD Pipeline
```bash
# Run comprehensive tests
gh workflow run "CI/CD Pipeline" --ref main

# Check results
gh run list --workflow="CI/CD Pipeline" --limit 1
```

This manual trigger capability gives you full control over when and how to run your build workflows! 🚀 