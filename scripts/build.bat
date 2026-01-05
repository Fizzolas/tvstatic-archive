@echo off
setlocal enabledelayedexpansion

cd /d "%~dp0\.."

echo.
echo SLLV build (Windows)
echo ==================
echo.

where cargo >nul 2>nul
if errorlevel 1 (
  echo ERROR: Rust is not installed or cargo is not on PATH.
  echo Fix: Install Rust from https://www.rust-lang.org/tools/install and re-open this window.
  goto :fail
)

if not exist dist mkdir dist

echo Building CLI...
cargo build -p sllv-cli --release
if errorlevel 1 goto :fail

echo Building GUI...
cargo build -p sllv-gui --release
if errorlevel 1 goto :fail

if exist target\release\sllv-cli.exe (
  copy /y target\release\sllv-cli.exe dist\sllv.exe >nul
) else if exist target\release\sllv.exe (
  copy /y target\release\sllv.exe dist\sllv.exe >nul
) else (
  echo ERROR: Could not find CLI exe in target\release.
  goto :fail
)

if exist target\release\sllv-gui.exe (
  copy /y target\release\sllv-gui.exe dist\sllv-gui.exe >nul
) else (
  echo ERROR: Could not find GUI exe (target\release\sllv-gui.exe).
  goto :fail
)

echo.
echo OK: Built dist\sllv.exe and dist\sllv-gui.exe

echo.
echo Running: dist\sllv.exe doctor

dist\sllv.exe doctor

echo.
echo Press any key to close.
pause >nul
exit /b 0

:fail
echo.
echo ERROR: Build failed. The details are above.
echo Press any key to close.
pause >nul
exit /b 1
