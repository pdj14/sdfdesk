$ErrorActionPreference = "Stop"

# Define paths
$ProjectRoot = "c:\workspace\rustdesk\sdfdesk"
$JniLibsDir = "$ProjectRoot\java_apk\app\src\main\jniLibs\arm64-v8a"
$Target = "aarch64-linux-android"

# Ensure jniLibs directory exists
if (-not (Test-Path -Path $JniLibsDir)) {
    New-Item -ItemType Directory -Force -Path $JniLibsDir | Out-Null
    Write-Host "Created directory: $JniLibsDir"
}

# Build Rust library
Write-Host "Building Rust library for $Target..."
cargo ndk -t $Target -o "$ProjectRoot\java_apk\app\src\main\jniLibs" build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "Build successful."
    # Verify file existence
    $LibPath = "$JniLibsDir\librustdesk.so"
    if (Test-Path -Path $LibPath) {
        Write-Host "Library copied to: $LibPath"
    } else {
        Write-Error "Library not found at expected path: $LibPath"
    }
} else {
    Write-Error "Build failed."
}
