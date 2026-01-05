@echo off
setlocal enabledelayedexpansion

REM SLLV build helper (Windows)

cd /d "%~dp0\.."

echo.
echo SLLV build (Windows)
echo ==================
echo.

where cargo >nul 2>nul
if errorlevel 1 (
  echo ERROR: Rust is not installed or cargo is not on PATH.
  echo.
  echo Fix:
  echo  1^) Install Rust from https://www.rust-lang.org/tools/install
  echo  2^) Re-open Command Prompt after installing.
  echo.
  goto :fail
)

if not exist dist mkdir dist

echo Building... this can take a few minutes on first run.
cargo build -p sllv-cli --release
if errorlevel 1 (
  echo.
  echo ERROR: Build failed.
  echo.
  echo Tip: The error is usually above. Scroll up.
  echo.
  goto :fail
)

if exist target\release\sllv-cli.exe (
  copy /y target\release\sllv-cli.exe dist\sllv.exe >nul
) else if exist target\release\sllv.exe (
  copy /y target\release\sllv.exe dist\sllv.exe >nul
) else (
  echo.
  echo ERROR: Could not find compiled exe in target\release.
  goto :fail
)

echo.
echo OK: Built dist\sllv.exe

echo.
echo Running: dist\sllv.exe doctor

dist\sllv.exe doctor

echo.
echo Press any key to close.
pause >nul
exit /b 0

:fail
echo.
echo Press any key to close.
pause >nul
exit /b 1
