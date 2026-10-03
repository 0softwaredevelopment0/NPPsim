@echo off
setlocal
rem ============================================
rem  NPP-SIM — portable launcher (Rust build)
rem  Runs NPPsim.exe next to this file;
rem  builds it first with cargo if missing.
rem ============================================
cd /d "%~dp0"

if exist "%~dp0NPPsim.exe" (
  start "" "%~dp0NPPsim.exe"
  exit /b 0
)

where cargo >nul 2>nul
if %errorlevel%==0 (
  echo [NPP-SIM] NPPsim.exe not found — building with cargo...
  cargo build --release
  if exist "%~dp0target\release\NPPsim.exe" (
    copy /y "%~dp0target\release\NPPsim.exe" "%~dp0NPPsim.exe" >nul
    start "" "%~dp0NPPsim.exe"
    exit /b 0
  )
  echo [NPP-SIM] Build failed.
  pause
  exit /b 1
)

echo.
echo [NPP-SIM] NPPsim.exe not found and cargo (Rust) is not installed.
echo Install Rust: https://rustup.rs
echo.
pause
