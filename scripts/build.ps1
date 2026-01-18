Param(
  [ValidateSet('all','cli','gui')][string]$Target = 'all',
  [switch]$Clean,
  [switch]$SkipDoctor,
  [switch]$NoPause
)

$ErrorActionPreference = 'Stop'

Set-Location (Join-Path $PSScriptRoot '..')

Write-Host "" 
Write-Host "SLLV build (PowerShell)" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Cyan
Write-Host "" 

function Pause-Exit {
  param([int]$Code)
  if (-not $NoPause) {
    Write-Host "" 
    Read-Host -Prompt "Press Enter to close" | Out-Null
  }
  exit $Code
}

function Require-Command($name, $fix) {
  if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: '$name' not found." -ForegroundColor Red
    Write-Host $fix -ForegroundColor Yellow
    Pause-Exit 1
  }
}

function Copy-Cli {
  if (Test-Path "target/release/sllv-cli.exe") {
    Copy-Item "target/release/sllv-cli.exe" "dist/sllv.exe" -Force
    return
  }
  if (Test-Path "target/release/sllv.exe") {
    Copy-Item "target/release/sllv.exe" "dist/sllv.exe" -Force
    return
  }
  throw "Could not find CLI exe (expected target/release/sllv-cli.exe or target/release/sllv.exe)"
}

function Copy-Gui {
  if (-not (Test-Path "target/release/sllv-gui.exe")) {
    throw "Could not find GUI exe (expected target/release/sllv-gui.exe)"
  }
  Copy-Item "target/release/sllv-gui.exe" "dist/sllv-gui.exe" -Force
}

try {
  if ($Clean) {
    if (Test-Path dist) { Remove-Item dist -Recurse -Force }
  }
  New-Item -ItemType Directory -Force -Path dist | Out-Null

  Require-Command cargo "Fix: Install Rust from https://www.rust-lang.org/tools/install and re-open PowerShell."

  if ($Target -eq 'all' -or $Target -eq 'cli') {
    Write-Host "Building CLI..." -ForegroundColor Gray
    cargo build -p sllv-cli --release
    Copy-Cli
    Write-Host "OK: Built dist\\sllv.exe" -ForegroundColor Green
  }

  if ($Target -eq 'all' -or $Target -eq 'gui') {
    Write-Host "Building GUI..." -ForegroundColor Gray
    cargo build -p sllv-gui --release
    Copy-Gui
    Write-Host "OK: Built dist\\sllv-gui.exe" -ForegroundColor Green
  }

  if (-not $SkipDoctor -and (Test-Path "dist/sllv.exe")) {
    Write-Host "" 
    Write-Host "Running: dist\\sllv.exe doctor" -ForegroundColor Gray
    & .\dist\sllv.exe doctor
  }

  Pause-Exit 0
}
catch {
  Write-Host "" 
  Write-Host "ERROR: Build failed." -ForegroundColor Red
  Write-Host $_.Exception.Message -ForegroundColor Red
  Write-Host "" 
  Write-Host "If this mentions a linker or C++ toolchain, install Visual Studio Build Tools" -ForegroundColor Yellow
  Write-Host "and select 'Desktop development with C++'." -ForegroundColor Yellow
  Pause-Exit 1
}
