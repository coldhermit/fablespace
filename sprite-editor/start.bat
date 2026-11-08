@echo off
cd /d "%~dp0"
if not exist node_modules (
  where node >nul 2>&1
  if errorlevel 1 (
    echo Node.js is required. Install it from https://nodejs.org and run this again.
    pause
    exit /b 1
  )
  call npm install
  if errorlevel 1 (
    echo npm install failed.
    pause
    exit /b 1
  )
)
call npm start
