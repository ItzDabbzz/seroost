#Requires -Version 5.1
# Release build + zip for local Seroost builds on Windows
# Usage: .\scripts\release.ps1 [-Version "0.2.0"]
# Output: releases\seroost-<VERSION>-<target>.zip

param(
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $PSScriptRoot

if (-not $Version) {
    $cargoToml = Get-Content "$projectRoot\Cargo.toml" -Raw
    if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
        $Version = $Matches[1]
    } else {
        throw "Could not parse version from Cargo.toml"
    }
}

$target = & rustc -vV | Select-String "^host:" | ForEach-Object { $_ -replace "^host:\s*", "" }
$zipName = "seroost-${Version}-${target}"
$releaseDir = "$projectRoot\releases"
$buildDir = "$projectRoot\target\release"
$zipPath = "$releaseDir\${zipName}.zip"

Write-Host "Building Seroost v$Version for $target ..."
Push-Location $projectRoot
& cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build failed" }
Pop-Location

Write-Host "Packaging release ..."
New-Item -ItemType Directory -Force -Path $releaseDir | Out-Null
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }

$staging = Join-Path $env:TEMP ([System.Guid]::NewGuid().ToString())
New-Item -ItemType Directory -Force -Path $staging | Out-Null

$pkgDir = "$staging\$zipName"
New-Item -ItemType Directory -Force -Path $pkgDir | Out-Null

$exeSrc = "$buildDir\seroost.exe"
if (-not (Test-Path $exeSrc)) {
    # Fallback for non-Windows targets (unlikely on Windows, but safe)
    $exeSrc = "$buildDir\seroost"
}
Copy-Item $exeSrc -Destination $pkgDir
Copy-Item "$projectRoot\readme.md" -Destination $pkgDir
if (Test-Path "$projectRoot\CHANGELOG.md") {
    Copy-Item "$projectRoot\CHANGELOG.md" -Destination $pkgDir
}

# Compress using Compress-Archive (built into Windows)
Compress-Archive -Path "$pkgDir\*" -DestinationPath $zipPath -Force

# Cleanup staging
Remove-Item $staging -Recurse -Force

Write-Host ""
Write-Host "Release created: $zipPath"
Get-Item $zipPath | Select-Object Length, LastWriteTime, FullName | Format-List
