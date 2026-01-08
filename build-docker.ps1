# Build and push ToonStore Docker image (Windows)

# Configuration
$IMAGE_NAME = "toonstore/toonstoredb"
$VERSION = (Select-String -Path "Cargo.toml" -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value
$LATEST_TAG = "${IMAGE_NAME}:latest"
$VERSION_TAG = "${IMAGE_NAME}:${VERSION}"

Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Building ToonStore Docker Image" -ForegroundColor Cyan
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "Image: ${IMAGE_NAME}"
Write-Host "Version: ${VERSION}"
Write-Host "Tags: latest, ${VERSION}"
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host ""

# Build the image
Write-Host "📦 Building Docker image..." -ForegroundColor Yellow
docker build -t $LATEST_TAG -t $VERSION_TAG .

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Build successful!" -ForegroundColor Green
Write-Host ""
Write-Host "🏷️  Tagged as:"
Write-Host "   - ${LATEST_TAG}"
Write-Host "   - ${VERSION_TAG}"

# Ask if user wants to push
Write-Host ""
$push = Read-Host "Do you want to push to Docker Hub? (y/N)"

if ($push -eq "y" -or $push -eq "Y") {
    Write-Host ""
    Write-Host "📤 Pushing to Docker Hub..." -ForegroundColor Yellow
    
    # Push both tags
    docker push $LATEST_TAG
    docker push $VERSION_TAG
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Push failed!" -ForegroundColor Red
        Write-Host "Make sure you're logged in: docker login" -ForegroundColor Yellow
        exit 1
    }
    
    Write-Host "✅ Push successful!" -ForegroundColor Green
    Write-Host ""
    Write-Host "🎉 Image published:" -ForegroundColor Green
    Write-Host "   docker pull ${LATEST_TAG}"
    Write-Host "   docker pull ${VERSION_TAG}"
} else {
    Write-Host ""
    Write-Host "ℹ️  Skipping push. Image available locally:" -ForegroundColor Cyan
    Write-Host "   docker run -p 6379:6379 ${LATEST_TAG}"
}

Write-Host ""
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "✅ Done!" -ForegroundColor Green
Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Cyan
