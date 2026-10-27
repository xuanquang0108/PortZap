# Build bản release và đóng gói zip portable.
# Cách dùng:  powershell -ExecutionPolicy Bypass -File scripts\package.ps1
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

$cargo = Get-Content -LiteralPath (Join-Path $root 'Cargo.toml') -Raw
if ($cargo -notmatch 'version\s*=\s*"([^"]+)"') { throw 'Không đọc được version trong Cargo.toml' }
$version = $Matches[1]

Write-Host "==> cargo build --release (v$version)" -ForegroundColor Cyan
Push-Location $root
try { cargo build --release } finally { Pop-Location }

$exe = Join-Path $root 'target\release\portzap.exe'
if (-not (Test-Path $exe)) { throw "Không tìm thấy $exe" }

$assets = Join-Path $root 'assets'
if (-not (Test-Path $assets)) { New-Item -ItemType Directory -Path $assets | Out-Null }

$stage = Join-Path $assets 'stage'
if (Test-Path $stage) { Remove-Item -Recurse -Force $stage }
New-Item -ItemType Directory -Path $stage | Out-Null

Copy-Item $exe $stage
Copy-Item (Join-Path $root 'README.md') $stage
Copy-Item (Join-Path $root 'LICENSE') $stage

$zip = Join-Path $assets "PortZap-v$version-win-x64.zip"
if (Test-Path $zip) { Remove-Item -Force $zip }
Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $zip -Force
Remove-Item -Recurse -Force $stage

Write-Host "==> $zip ($([math]::Round((Get-Item $zip).Length / 1MB, 2)) MB)" -ForegroundColor Green
