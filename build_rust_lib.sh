#!/bin/bash
set -e

# Define paths
PROJECT_ROOT=$(pwd)
JNI_LIBS_DIR="$PROJECT_ROOT/java_apk/app/src/main/jniLibs/arm64-v8a"
TARGET="aarch64-linux-android"

# Ensure jniLibs directory exists
mkdir -p "$JNI_LIBS_DIR"
echo "Created directory: $JNI_LIBS_DIR"

# Build Rust library
echo "Building Rust library for $TARGET..."
cargo ndk -t $TARGET -o "$PROJECT_ROOT/java_apk/app/src/main/jniLibs" build --release

if [ $? -eq 0 ]; then
    echo "Build successful."
    # Verify file existence
    LIB_PATH="$JNI_LIBS_DIR/librustdesk.so"
    if [ -f "$LIB_PATH" ]; then
        echo "Library copied to: $LIB_PATH"
    else
        echo "Error: Library not found at expected path: $LIB_PATH"
        exit 1
    fi
else
    echo "Build failed."
    exit 1
fi
