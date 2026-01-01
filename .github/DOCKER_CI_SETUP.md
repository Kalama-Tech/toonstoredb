# GitHub Actions CI/CD Setup Guide

This document explains how to set up automatic Docker image building and publishing using GitHub Actions.

---

## 🚀 What It Does

When you push to `main` or merge a pull request:
1. ✅ Automatically builds Docker image
2. ✅ Tags with version from `Cargo.toml` (e.g., `0.1.0`)
3. ✅ Tags as `latest`
4. ✅ Pushes to Docker Hub (`samso9th/toonstore`)
5. ✅ Updates Docker Hub README from your GitHub README
6. ✅ Builds for multiple architectures (amd64 and arm64)

---

## 📋 Setup Instructions

### Step 1: Add Docker Hub Secrets to GitHub

You need to add your Docker Hub credentials as GitHub secrets:

1. **Go to your GitHub repository**
   ```
   https://github.com/yourusername/toonstoredb
   ```

2. **Navigate to Settings → Secrets and variables → Actions**
   ```
   Settings → Secrets and variables → Actions → New repository secret
   ```

3. **Add DOCKER_USERNAME**
   - Name: `DOCKER_USERNAME`
   - Value: `samso9th` (your Docker Hub username)
   - Click "Add secret"

4. **Add DOCKER_PASSWORD**
   - Name: `DOCKER_PASSWORD`
   - Value: Your Docker Hub password or access token (recommended)
   - Click "Add secret"

   **Recommended:** Use an access token instead of password:
   - Go to https://hub.docker.com/settings/security
   - Click "New Access Token"
   - Description: "GitHub Actions"
   - Permissions: Read, Write, Delete
   - Copy the token and use it as `DOCKER_PASSWORD`

### Step 2: Verify Secrets

After adding secrets, verify they're set:
- Go to: `Settings → Secrets and variables → Actions`
- You should see:
  - ✅ `DOCKER_USERNAME`
  - ✅ `DOCKER_PASSWORD`

---

## 🔄 How It Works

### Workflow File: `.github/workflows/docker.yml`

**Triggers:**
- Push to `main` branch
- Pull request merged to `main`

**Process:**
1. Checkout code
2. Extract version from `Cargo.toml`
3. Set up Docker Buildx (multi-platform builds)
4. Login to Docker Hub using secrets
5. Build Docker image for amd64 and arm64
6. Tag image:
   - `samso9th/toonstore:latest`
   - `samso9th/toonstore:0.1.0` (version from Cargo.toml)
   - `samso9th/toonstore:0.1` (major.minor)
   - `samso9th/toonstore:main-<git-sha>` (commit hash)
7. Push all tags to Docker Hub
8. Update Docker Hub description with README.md

---

## 🧪 Testing the Workflow

### Test Push to Main

```bash
# Make a change
git add .
git commit -m "Test Docker workflow"
git push origin main

# Watch the workflow
# Go to: https://github.com/yourusername/toonstoredb/actions
```

### Test PR Merge

```bash
# Create a branch
git checkout -b test-docker-build

# Make a change
echo "# Test" >> README.md
git add README.md
git commit -m "Test Docker build on PR merge"
git push origin test-docker-build

# Create PR on GitHub
# Merge the PR
# Watch the workflow run
```

---

## 📊 Workflow Status

You can check the workflow status:

**GitHub Actions:**
```
https://github.com/yourusername/toonstoredb/actions
```

**Docker Hub:**
```
https://hub.docker.com/r/samso9th/toonstore
```

---

## 🏷️ Image Tags

After successful build, your images will be tagged as:

| Tag | Description | Example |
|-----|-------------|---------|
| `latest` | Always points to latest main build | `samso9th/toonstore:latest` |
| `<version>` | Semantic version from Cargo.toml | `samso9th/toonstore:0.1.0` |
| `<major>.<minor>` | Major and minor version | `samso9th/toonstore:0.1` |
| `main-<sha>` | Commit-specific tag | `samso9th/toonstore:main-abc1234` |

---

## 🌍 Multi-Architecture Support

The workflow builds for:
- ✅ **linux/amd64** (Intel/AMD x86-64)
- ✅ **linux/arm64** (Apple Silicon, ARM servers)

Users can pull and run on any architecture:
```bash
# Works on x86-64 (Intel/AMD)
docker pull samso9th/toonstore:latest

# Also works on ARM64 (Apple Silicon, Raspberry Pi, AWS Graviton)
docker pull samso9th/toonstore:latest
```

---

## 🔧 Customization

### Change Docker Hub Username

Edit `.github/workflows/docker.yml`:
```yaml
env:
  DOCKER_IMAGE: your-username/toonstore  # Change this
```

### Add More Platforms

Edit `.github/workflows/docker.yml`:
```yaml
env:
  DOCKER_PLATFORMS: linux/amd64,linux/arm64,linux/arm/v7  # Add more
```

### Change Trigger Branches

Edit `.github/workflows/docker.yml`:
```yaml
on:
  push:
    branches:
      - main
      - develop  # Add more branches
```

---

## 🐛 Troubleshooting

### Build Fails: "Error: buildx failed"

**Solution:** Check Dockerfile syntax
```bash
# Test locally
docker build -t test .
```

### Push Fails: "unauthorized"

**Solution:** Check secrets
- Verify `DOCKER_USERNAME` is correct
- Verify `DOCKER_PASSWORD` is valid
- Try generating new access token

### Workflow Not Triggering

**Solution:** Check branch name
- Workflow only runs on `main` branch
- Check your default branch name: `git branch`

### Can't See Workflow

**Solution:** Check GitHub Actions permissions
- Go to: `Settings → Actions → General`
- Enable: "Allow all actions and reusable workflows"

---

## 📈 Build Times

Expected build times:
- **First build:** ~5-10 minutes (compiling Rust)
- **Subsequent builds:** ~2-5 minutes (using cache)
- **Multi-arch builds:** ~8-15 minutes (building for 2 platforms)

---

## 💰 GitHub Actions Costs

- **Free tier:** 2,000 minutes/month for public repos
- **Free tier:** 500 MB storage for artifacts/cache
- **This workflow:** ~10 minutes per build

**Example:**
- 20 pushes/month × 10 minutes = 200 minutes
- Well within free tier! ✅

---

## 🎯 Best Practices

### 1. Use Access Tokens (Not Passwords)
```
✅ Use: Docker Hub Access Token
❌ Don't use: Your Docker Hub password
```

### 2. Version Your Releases
```bash
# Update version in Cargo.toml
version = "0.2.0"

# Commit and push
git add Cargo.toml
git commit -m "Bump version to 0.2.0"
git push origin main

# Workflow automatically tags: samso9th/toonstore:0.2.0
```

### 3. Test Locally First
```bash
# Build locally before pushing
docker build -t samso9th/toonstore:test .

# Test locally
docker run -p 6379:6379 samso9th/toonstore:test

# Then push to GitHub
git push origin main
```

---

## 🔗 Quick Links

- **Workflow File:** `.github/workflows/docker.yml`
- **Docker Hub:** https://hub.docker.com/r/samso9th/toonstore
- **GitHub Actions:** https://github.com/yourusername/toonstoredb/actions
- **Secrets Setup:** https://github.com/yourusername/toonstoredb/settings/secrets/actions

---

## ✅ Setup Checklist

Before first push:
- [ ] Created Docker Hub repository: `samso9th/toonstore`
- [ ] Added `DOCKER_USERNAME` secret to GitHub
- [ ] Added `DOCKER_PASSWORD` secret to GitHub
- [ ] Workflow file exists: `.github/workflows/docker.yml`
- [ ] Tested locally: `docker build -t test .`

After first push:
- [ ] Workflow ran successfully
- [ ] Image pushed to Docker Hub
- [ ] Can pull: `docker pull samso9th/toonstore:latest`
- [ ] Can run: `docker run -p 6379:6379 samso9th/toonstore:latest`

---

## 📚 Additional Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Docker Build Push Action](https://github.com/marketplace/actions/build-and-push-docker-images)
- [Docker Hub Access Tokens](https://docs.docker.com/docker-hub/access-tokens/)

---

## 🎉 Summary

**What you get:**
- ✅ Automatic Docker builds on every push to main
- ✅ Multi-architecture support (amd64 + arm64)
- ✅ Automatic version tagging from Cargo.toml
- ✅ Docker Hub README auto-sync
- ✅ Free on GitHub Actions (public repos)

**Users can:**
```bash
docker pull samso9th/toonstore:latest
docker run -p 6379:6379 samso9th/toonstore:latest
```

**You just:**
```bash
git push origin main  # That's it! Everything else is automatic.
```
