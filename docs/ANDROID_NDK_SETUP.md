# Android NDK Setup Guide

This guide explains how to set up Android NDK for building sdfdesk.

## Required Version

- **NDK Version**: r26c (recommended)
- **Download URL**: https://developer.android.com/ndk/downloads

## Download Instructions

### Option 1: Direct Download

1. Download NDK r26c from [Android NDK Downloads](https://developer.android.com/ndk/downloads)
2. Extract to `sdfdesk/android-ndk-r26c/`

### Option 2: Using Command Line

**Windows (PowerShell):**
```powershell
cd sdfdesk
Invoke-WebRequest -Uri "https://dl.google.com/android/repository/android-ndk-r26c-windows.zip" -OutFile "android-ndk-r26c.zip"
Expand-Archive -Path "android-ndk-r26c.zip" -DestinationPath "."
Remove-Item "android-ndk-r26c.zip"
```

**Linux/WSL:**
```bash
cd sdfdesk
wget https://dl.google.com/android/repository/android-ndk-r26c-linux.zip
unzip android-ndk-r26c-linux.zip
rm android-ndk-r26c-linux.zip
```

**macOS:**
```bash
cd sdfdesk
curl -O https://dl.google.com/android/repository/android-ndk-r26c-darwin.zip
unzip android-ndk-r26c-darwin.zip
rm android-ndk-r26c-darwin.zip
```

## Environment Variables

Set the following environment variables:

```bash
export ANDROID_NDK_HOME=/path/to/sdfdesk/android-ndk-r26c
export ANDROID_NDK_ROOT=/path/to/sdfdesk/android-ndk-r26c
```

## Verification

Verify NDK installation:
```bash
$ANDROID_NDK_HOME/ndk-build --version
```

## Notes

- The NDK is excluded from git (`.gitignore`) due to its large size (~1GB+)
- Each developer must download and set up NDK locally
- Different OS requires different NDK binaries (Windows/Linux/macOS)
