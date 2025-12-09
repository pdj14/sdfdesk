#!/bin/bash
# Flutter Android Build Script for Docker

set -e

echo "=== Flutter Android Build in Docker ==="

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "Error: Docker is not running"
    exit 1
fi

cd /mnt/c/workspace/rustdesk/sdfdesk

# Build Docker image (first time only)
echo "Building Docker image..."
docker build -f Dockerfile.flutter -t flutter-rustdesk-builder .

# Run the build
echo "Building Flutter APK..."
docker run --rm \
    -v "$(pwd):/app" \
    -v "flutter-gradle-cache:/root/.gradle" \
    -v "flutter-pub-cache:/opt/flutter/.pub-cache" \
    -v "flutter-cargo-cache:/root/.cargo/registry" \
    -w /app \
    flutter-rustdesk-builder \
    bash -c "cd flutter && flutter pub get && flutter build apk --release"

echo "=== Build Complete ==="
echo "APK location: flutter/build/app/outputs/flutter-apk/app-release.apk"
