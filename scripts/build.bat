@echo off
setlocal enabledelayedexpansion

REM SLLV build helper (Windows)
REM This script is intended for non-developers.
REM It will keep the window open and show clear errors.

cd /d "%~dp0\.."

echo.
echo SLLV build (Windows)
echo ==================
echo.

REM Basic checks
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
  echo If you see linker/Visual Studio build tools errors, install:
  echo   "Visual Studio Build Tools" with "Desktop development with C++".
  echo.
  goto :fail
)

if exist target\release\sllv.exe (
  copy /y target\release\sllv.exe dist\sllv.exe >nul
  echo.
  echo OK: Built dist\sllv.exe
  echo.
  echo Next:
  echo   dist\sllv.exe doctor
  echo.
  goto :ok
) else (
  echo.
  echo ERROR: target\release\sllv.exe not found after build.
  goto :fail
)

:ok
echo Press any key to close.
pause >nul
exit /b 0

:fail
echo.
echo Press any key to close.
pause >nul
exit /b 1
