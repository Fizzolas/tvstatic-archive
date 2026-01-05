Param(
  [switch]$Clean
)

$ErrorActionPreference = 'Stop'

Set-Location (Join-Path $PSScriptRoot '..')

if ($Clean) {
  if (Test-Path dist) { Remove-Item dist -Recurse -Force }
}

New-Item -ItemType Directory -Force -Path dist | Out-Null

cargo build -p sllv-cli --release
Copy-Item target\release\sllv.exe dist\sllv.exe -Force

Write-Host "Built dist\sllv.exe"
