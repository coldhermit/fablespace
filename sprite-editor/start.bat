@echo off
cd /d "%~dp0"
if exist "dist\win-unpacked\SpriteEditor.exe" (
  start "" "dist\win-unpacked\SpriteEditor.exe"
  exit /b 0
)
where node >nul 2>&1
if errorlevel 1 (
  echo SpriteEditor.exe is not built yet, and Node.js is not installed.
  echo Download the Windows zip from the latest release, unzip it, and run SpriteEditor.exe.
  echo https://github.com/coldhermit/fablespace/releases
  pause
  exit /b 1
)
if not exist node_modules call npm install
if errorlevel 1 (
  echo npm install failed.
  pause
  exit /b 1
)
call npm run pack
if errorlevel 1 (
  echo The Windows app failed to build.
  pause
  exit /b 1
)
start "" "dist\win-unpacked\SpriteEditor.exe"
exit /b 0
