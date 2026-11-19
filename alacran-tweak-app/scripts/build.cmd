@echo off
setlocal
call "%~dp0env.cmd" || exit /b 1
call npm run tauri build -- --no-bundle || exit /b 1
if not exist "%CARGO_TARGET_DIR%\release\AlacranTweak.exe" (
  echo Build finished but AlacranTweak.exe is missing 1>&2
  exit /b 1
)
echo Built: %CARGO_TARGET_DIR%\release\AlacranTweak.exe
