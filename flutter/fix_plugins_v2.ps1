$pubCacheHosted = "$env:LOCALAPPDATA\Pub\Cache\hosted\pub.dev"
$pubCacheGit = "$env:LOCALAPPDATA\Pub\Cache\git"
$plugins = Get-ChildItem -Path $pubCacheHosted -Directory
$plugins += Get-ChildItem -Path $pubCacheGit -Directory -Recurse -Depth 1 | Where-Object { $_.FullName -match "uni_links" -or $_.FullName -match "desktop_drop" }

Write-Host "Starting Plugin Fix V2..."

foreach ($plugin in $plugins) {
    $buildGradlePath = Join-Path $plugin.FullName "android\build.gradle"
    $manifestPath = Join-Path $plugin.FullName "android\src\main\AndroidManifest.xml"

    if (Test-Path $buildGradlePath) {
        $content = Get-Content $buildGradlePath -Raw
        $originalContent = $content
        
        # 1. Check if it is a Kotlin project
        # Look for 'apply plugin: ...kotlin-android' or 'id ...kotlin-android'
        $isKotlin = $content -match "kotlin-android"
        
        # 2. Get Package Name (Namespace)
        $package = $null
        if ($content -match "namespace\s+['`"]([^'`"]+)['`"]") {
            $package = $matches[1]
        } elseif (Test-Path $manifestPath) {
            $manifest = Get-Content $manifestPath -Raw
            if ($manifest -match 'package="([^"]+)"') {
                $package = $matches[1]
            }
        }

        if (-not $package) {
            Write-Warning "Skipping $($plugin.Name): Could not determine namespace."
            continue
        }

        # 3. Construct the correct configuration block
        $compileOptionsBlock = "
    compileSdkVersion 36
    compileOptions {
        sourceCompatibility JavaVersion.VERSION_17
        targetCompatibility JavaVersion.VERSION_17
    }"
        
        $kotlinOptionsBlock = ""
        if ($isKotlin) {
            $kotlinOptionsBlock = "
    kotlinOptions {
        jvmTarget = '17'
    }"
        }

        # 4. Clean up existing blocks to avoid duplication
        # Remove existing compileOptions
        $content = $content -replace "(?s)compileOptions\s*\{.*?\}", ""
        # Remove existing kotlinOptions
        $content = $content -replace "(?s)kotlinOptions\s*\{.*?\}", ""
        
        # 5. Inject new blocks
        # We inject them at the end of the 'android {' block
        # Find the last closing brace of 'android {' block is hard with regex alone reliably without parsing.
        # Instead, we will inject after 'defaultConfig { ... }' or 'namespace ...'
        
        # Strategy: Re-inject namespace (if missing in android block) and options
        
        if ($content -notmatch "namespace\s+['`"]") {
             # Inject namespace and options into android { ... }
             $content = $content -replace "android\s*\{", "android {`n    namespace `"$package`"$compileOptionsBlock$kotlinOptionsBlock"
        } else {
             # Namespace exists, inject options after it
             $content = $content -replace "(namespace\s+['`"][^'`"]+['`"])", "`$1$compileOptionsBlock$kotlinOptionsBlock"
        }

        # 6. Fix legacy JavaVersion references
        $content = $content -replace "JavaVersion.VERSION_1_8", "JavaVersion.VERSION_17"
        
        if ($content -ne $originalContent) {
            Write-Host "Updated $($plugin.Name) (Kotlin: $isKotlin)"
            Set-Content -Path $buildGradlePath -Value $content
        }
    }
}
Write-Host "Plugin Fix V2 Complete."
