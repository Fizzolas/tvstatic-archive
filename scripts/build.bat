@echo off
setlocal

REM SLLV build wrapper (Windows)
REM Prefer PowerShell for reliable output and error handling.

cd /d "%~dp0\.."

where powershell >nul 2>nul
if errorlevel 1 (
  echo ERROR: PowerShell not found.
  echo Fix: Use Windows PowerShell or PowerShell 7.
  goto :fail
)

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0build.ps1" %*
set ERR=%ERRORLEVEL%
if not "%ERR%"=="0" goto :fail

REM If double-clicked (no args), keep the window open so the user can read output.
if "%~1"=="" (
  echo.
  echo Press any key to close.
  pause >nul
)

exit /b %ERR%

:fail
if "%~1"=="" (
  echo.
  echo Build failed. Press any key to close.
  pause >nul
)
exit /b 1
