Param(
  [switch]$AutoInstall,
  [switch]$Clean
)

$ErrorActionPreference = 'Stop'

Set-Location (Join-Path $PSScriptRoot '..')

Write-Host "" 
Write-Host "SLLV setup (Windows)" -ForegroundColor Cyan
Write-Host "====================" -ForegroundColor Cyan
Write-Host "" 

function Pause-Exit {
  param([int]$Code)
  Write-Host "" 
  Read-Host -Prompt "Press Enter to close"
  exit $Code
}

function Has-Command($name) {
  return [bool](Get-Command $name -ErrorAction SilentlyContinue)
}

function Ensure-Winget {
  if (-not (Has-Command winget)) {
    Write-Host "ERROR: winget is not available on this system." -ForegroundColor Red
    Write-Host "This auto-installer needs Windows Package Manager (winget)." -ForegroundColor Yellow
    Write-Host "Fix: Update to Windows 10/11 with App Installer, or install Rust manually." -ForegroundColor Yellow
    Pause-Exit 1
  }
}

function Ensure-Rust {
  if (Has-Command cargo) { return }

  if (-not $AutoInstall) {
    Write-Host "ERROR: Rust isn't installed (cargo not found)." -ForegroundColor Red
    Write-Host "Run this script with -AutoInstall to install Rust automatically," -ForegroundColor Yellow
    Write-Host "or install it manually from: https://www.rust-lang.org/tools/install" -ForegroundColor Yellow
    Pause-Exit 1
  }

  Ensure-Winget
  Write-Host "Installing Rust (rustup) via winget..." -ForegroundColor Gray
  winget install -e --id Rustlang.Rustup

  Write-Host "" 
  Write-Host "Rust installed. Please close and re-open PowerShell, then re-run this script." -ForegroundColor Yellow
  Pause-Exit 0
}

function Ensure-BuildTools {
  # If cl.exe exists, assume MSVC tools are present.
  if (Has-Command cl) { return }

  if (-not $AutoInstall) {
    Write-Host "NOTE: Visual Studio C++ Build Tools not detected (cl.exe not found)." -ForegroundColor Yellow
    Write-Host "If the build fails with linker errors, re-run with -AutoInstall to install Build Tools." -ForegroundColor Yellow
    return
  }

  Ensure-Winget
  Write-Host "Installing Visual Studio 2022 Build Tools (C++ workload)..." -ForegroundColor Gray
  # Include C++ toolchain workload; this commonly resolves linker errors for Rust crates on Windows.
  winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools;includeRecommended"

  Write-Host "Build Tools install requested. If prompted, accept the UAC/admin prompt." -ForegroundColor Yellow
}

try {
  if ($Clean) {
    if (Test-Path dist) { Remove-Item dist -Recurse -Force }
  }

  New-Item -ItemType Directory -Force -Path dist | Out-Null

  Ensure-Rust
  Ensure-BuildTools

  Write-Host "" 
  Write-Host "Running preflight (doctor)..." -ForegroundColor Gray

  # Build first if the binary isn't present.
  if (-not (Test-Path "dist/sllv.exe")) {
    Write-Host "Building SLLV... (first build can take a few minutes)" -ForegroundColor Gray
    cargo build -p sllv-cli --release
    if (-not (Test-Path "target/release/sllv.exe")) {
      Write-Host "ERROR: target/release/sllv.exe not found after build." -ForegroundColor Red
      Pause-Exit 1
    }
    Copy-Item target/release/sllv.exe dist/sllv.exe -Force
  }

  & .\dist\sllv.exe doctor

  Write-Host "" 
  Write-Host "OK: SLLV is ready." -ForegroundColor Green
  Write-Host "Next example:" -ForegroundColor Gray
  Write-Host "  .\dist\sllv.exe encode --profile scan --input .\my_folder --out-frames .\frames_scan --fps 12" -ForegroundColor Gray
  Pause-Exit 0
}
catch {
  Write-Host "" 
  Write-Host "ERROR: setup failed." -ForegroundColor Red
  Write-Host $_.Exception.Message -ForegroundColor Red
  Pause-Exit 1
}
