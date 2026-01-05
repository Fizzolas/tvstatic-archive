@echo off
setlocal enabledelayedexpansion

REM Build a standalone CLI binary and put it in dist/.
REM Requires Rust toolchain installed.

cd /d "%~dp0\.."

if not exist dist mkdir dist

cargo build -p sllv-cli --release
if errorlevel 1 exit /b 1

copy /y target\release\sllv.exe dist\sllv.exe >nul

echo Built dist\sllv.exe
