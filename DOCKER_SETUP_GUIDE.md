# GitHub Docker Setup Guide

This guide explains how to configure GitHub to automatically build and publish Docker images to GitHub Container Registry (ghcr.io) on every push to `main` or when you create a version tag.

## 📋 Prerequisites

- GitHub repository created
- Docker image configuration ready (Dockerfile exists)
- GitHub Actions workflow configured (`.github/workflows/ci.yml`)

## 🔧 Step-by-Step Setup

### 1. Enable GitHub Container Registry (GHCR)

GitHub Container Registry is **automatically enabled** for all repositories. No manual setup required! 🎉

The workflow uses `GITHUB_TOKEN` which is automatically provided by GitHub Actions.

### 2. Configure Repository Permissions

#### Option A: Using GITHUB_TOKEN (Recommended - Automatic)

The workflow already uses `GITHUB_TOKEN` which is automatically available:

```yaml
permissions:
  contents: read
  packages: write  # Required for pushing to ghcr.io
```

**No additional setup needed!** The token is automatically scoped to your repository.

#### Option B: Using Personal Access Token (Optional - Advanced)

Only needed if you want to push to a different repository or organization.

1. **Create Personal Access Token (PAT)**:
   - Go to GitHub Settings → Developer settings → Personal access tokens → Tokens (classic)
   - Click "Generate new token (classic)"
   - Give it a name: `DOCKER_PUSH_TOKEN`
   - Select scopes:
     - ✅ `write:packages` (Upload packages to GitHub Package Registry)
     - ✅ `read:packages` (Download packages from GitHub Package Registry)
     - ✅ `delete:packages` (Delete packages from GitHub Package Registry - optional)
   - Click "Generate token"
   - **Copy the token immediately** (you won't see it again!)

2. **Add Token to Repository Secrets**:
   - Go to your repository → Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `DOCKER_TOKEN`
   - Value: Paste your PAT
   - Click "Add secret"

3. **Update Workflow** (only if using PAT):
   ```yaml
   - name: Log in to GitHub Container Registry
     uses: docker/login-action@v3
     with:
       registry: ghcr.io
       username: ${{ github.actor }}
       password: ${{ secrets.DOCKER_TOKEN }}  # Use PAT instead of GITHUB_TOKEN
   ```

### 3. Enable Workflow Permissions (Important!)

1. Go to your repository → **Settings** → **Actions** → **General**
2. Scroll down to **"Workflow permissions"**
3. Select **"Read and write permissions"** ✅
4. Check **"Allow GitHub Actions to create and approve pull requests"** ✅
5. Click **"Save"**

**This is critical!** Without this, the workflow cannot push Docker images.

### 4. Configure Package Visibility

After the first successful build, configure package visibility:

1. Go to your repository's main page
2. Click on **"Packages"** (right sidebar)
3. Click on your package name (e.g., `toonstore`)
4. Click **"Package settings"** (bottom of the page)
5. Under **"Danger Zone"** → **"Change visibility"**:
   - **Public**: Anyone can pull the image (recommended for open source)
   - **Private**: Only you and collaborators can pull

### 5. Link Package to Repository (Optional but Recommended)

1. On the package settings page
2. Under **"Package settings"** → **"Manage Actions access"**
3. Add your repository if it's not already linked
4. This makes the package appear in your repository's sidebar

## 🚀 Usage After Setup

### Automatic Builds

Once configured, Docker images are automatically built when:

1. **Push to main**:
   ```bash
   git push origin main
   ```
   Creates tags: `latest`, `main`, `main-<sha>`

2. **Create a version tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
   Creates tags: `v0.1.0`, `0.1`, `0`, `latest`

### Pull the Image

```bash
# Public package (no auth needed)
docker pull ghcr.io/OWNER/REPO:latest

# Example:
docker pull ghcr.io/yourusername/toonstore:latest

# Specific version
docker pull ghcr.io/yourusername/toonstore:v0.1.0
```

### Authenticate to Pull Private Images

```bash
# Create a PAT with read:packages scope
# Then login:
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin

# Or use the GitHub CLI:
gh auth token | docker login ghcr.io -u USERNAME --password-stdin
```

## 📦 Available Tags

The workflow automatically creates these tags:

| Tag Pattern | Example | Description |
|-------------|---------|-------------|
| `latest` | `latest` | Most recent build from main |
| `main` | `main` | Latest main branch build |
| `main-<sha>` | `main-a1b2c3d` | Specific commit on main |
| `v*.*.*` | `v0.1.0` | Semantic version (on git tag) |
| `v*.*` | `v0.1` | Major.minor version |
| `v*` | `v0` | Major version only |

## 🔍 Verify Setup

### Check Workflow Status

1. Go to repository → **Actions** tab
2. Look for "CI/CD Pipeline" workflow
3. Check if "Build & Push Docker Image" job succeeds
4. Green checkmark = Success! ✅

### Check Package Registry

1. Go to repository main page
2. Look for **"Packages"** in right sidebar
3. Click on package name
4. You should see all tags listed

### Test Pull

```bash
# Pull the image
docker pull ghcr.io/OWNER/REPO:latest

# Run it
docker run --rm ghcr.io/OWNER/REPO:latest --help
```

## 🐛 Troubleshooting

### Error: "denied: permission_denied"

**Solution**: Enable "Read and write permissions" in Settings → Actions → General

### Error: "failed to authorize: invalid token"

**Solution**: Check that GITHUB_TOKEN or PAT has `write:packages` scope

### Error: "package does not exist"

**Solution**: 
1. Check package visibility (make it public)
2. Verify you're using the correct repository name
3. Check if first build succeeded

### Docker image not appearing in Packages

**Solution**:
1. Wait for workflow to complete (check Actions tab)
2. Link package to repository in Package settings
3. Refresh the page

### Multi-arch build fails

**Solution**: QEMU and Buildx are required (already in workflow)
- Check logs for specific platform errors
- Try building single platform first (remove `linux/arm64`)

## 🔐 Security Best Practices

1. **Use GITHUB_TOKEN** (not PAT) when possible
2. **Never commit tokens** to the repository
3. **Set package visibility** appropriately:
   - Public: Open source projects
   - Private: Internal/commercial projects
4. **Rotate PATs** regularly (if using custom tokens)
5. **Use least privilege**: Only give required scopes

## 📝 Example: Complete Setup Checklist

- [ ] Repository created on GitHub
- [ ] Dockerfile exists and is working
- [ ] `.github/workflows/ci.yml` added to repo
- [ ] Workflow permissions set to "Read and write"
- [ ] First push to main or tag created
- [ ] Workflow succeeded (check Actions tab)
- [ ] Package appears in Packages sidebar
- [ ] Package visibility configured (public/private)
- [ ] Package linked to repository
- [ ] Tested pulling the image
- [ ] README updated with docker pull command

## 🎯 Quick Start Commands

```bash
# 1. Commit and push workflow
git add .github/workflows/ci.yml
git commit -m "Add CI/CD with Docker build"
git push origin main

# 2. Create first release
git tag v0.1.0
git push origin v0.1.0

# 3. Pull and test
docker pull ghcr.io/OWNER/REPO:latest
docker run --rm ghcr.io/OWNER/REPO:latest --version

# 4. Run the server
docker run -d \
  -p 6379:6379 \
  -v $(pwd)/data:/data \
  --name toonstore \
  ghcr.io/OWNER/REPO:latest
```

## 📚 Additional Resources

- [GitHub Packages Documentation](https://docs.github.com/en/packages)
- [GitHub Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
- [Docker Build Push Action](https://github.com/docker/build-push-action)
- [GitHub Actions Permissions](https://docs.github.com/en/actions/security-guides/automatic-token-authentication)

## ✅ Success Indicators

You know it's working when:

1. ✅ Workflow completes successfully (green checkmark)
2. ✅ Package appears in repository sidebar
3. ✅ `docker pull ghcr.io/OWNER/REPO:latest` works
4. ✅ Image runs without errors
5. ✅ New commits trigger automatic builds
6. ✅ Tags create versioned images

---

**That's it!** Your Docker images will now be automatically built and published on every push to main or when you create version tags. 🎉
