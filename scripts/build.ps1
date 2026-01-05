Param(
  [switch]$Clean
)

$ErrorActionPreference = 'Stop'

Set-Location (Join-Path $PSScriptRoot '..')

Write-Host ""
Write-Host "SLLV build (PowerShell)" -ForegroundColor Cyan
Write-Host "======================" -ForegroundColor Cyan
Write-Host ""

if ($Clean) {
  if (Test-Path dist) { Remove-Item dist -Recurse -Force }
}

New-Item -ItemType Directory -Force -Path dist | Out-Null

function Pause-Exit {
  param([int]$Code)
  Write-Host ""
  Read-Host -Prompt "Press Enter to close"
  exit $Code
}

try {
  if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Rust is not installed or cargo is not on PATH." -ForegroundColor Red
    Write-Host "Fix: Install Rust from https://www.rust-lang.org/tools/install and re-open PowerShell." -ForegroundColor Yellow
    Pause-Exit 1
  }

  Write-Host "Building... (first build can take a few minutes)" -ForegroundColor Gray
  cargo build -p sllv-cli --release

  $exe = "target/release/sllv.exe"
  if (-not (Test-Path $exe)) {
    Write-Host "ERROR: $exe not found after build." -ForegroundColor Red
    Pause-Exit 1
  }

  Copy-Item $exe dist/sllv.exe -Force
  Write-Host "OK: Built dist\sllv.exe" -ForegroundColor Green
  Write-Host "Next: dist\sllv.exe doctor" -ForegroundColor Gray
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
