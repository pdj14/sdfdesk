#!/bin/bash
set -e

# Fix git safe directory
git config --global --add safe.directory /app
git config --global --add safe.directory /app/flutter

# Build Flutter
cd /app/flutter
flutter pub get
flutter build apk --release

echo "=== Build Complete ==="
echo "APK: /app/flutter/build/app/outputs/flutter-apk/app-release.apk"
