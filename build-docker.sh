#!/bin/bash
# Build and push ToonStore Docker image
set -e

# Configuration
IMAGE_NAME="toonstore/toonstoredb"
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
LATEST_TAG="${IMAGE_NAME}:latest"
VERSION_TAG="${IMAGE_NAME}:${VERSION}"

echo "═══════════════════════════════════════════════════════════"
echo "Building ToonStore Docker Image"
echo "═══════════════════════════════════════════════════════════"
echo "Image: ${IMAGE_NAME}"
echo "Version: ${VERSION}"
echo "Tags: latest, ${VERSION}"
echo "═══════════════════════════════════════════════════════════"

# Build the image
echo ""
echo "📦 Building Docker image..."
docker build -t ${LATEST_TAG} -t ${VERSION_TAG} .

if [ $? -ne 0 ]; then
    echo "❌ Build failed!"
    exit 1
fi

echo "✅ Build successful!"
echo ""
echo "🏷️  Tagged as:"
echo "   - ${LATEST_TAG}"
echo "   - ${VERSION_TAG}"

# Ask if user wants to push
echo ""
read -p "Do you want to push to Docker Hub? (y/N) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "📤 Pushing to Docker Hub..."
    
    # Push both tags
    docker push ${LATEST_TAG}
    docker push ${VERSION_TAG}
    
    if [ $? -ne 0 ]; then
        echo "❌ Push failed!"
        echo "Make sure you're logged in: docker login"
        exit 1
    fi
    
    echo "✅ Push successful!"
    echo ""
    echo "🎉 Image published:"
    echo "   docker pull ${LATEST_TAG}"
    echo "   docker pull ${VERSION_TAG}"
else
    echo ""
    echo "ℹ️  Skipping push. Image available locally:"
    echo "   docker run -p 6379:6379 ${LATEST_TAG}"
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo "✅ Done!"
echo "═══════════════════════════════════════════════════════════"
