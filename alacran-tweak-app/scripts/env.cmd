@echo off
rem Shared environment setup: VS 2022 Build Tools (vcvars64), cargo on PATH, short CARGO_TARGET_DIR.
set "VSROOT="
set "VSWHERE=%ProgramFiles(x86)%\Microsoft Visual Studio\Installer\vswhere.exe"
if exist "%VSWHERE%" (
  for /f "usebackq delims=" %%i in (`"%VSWHERE%" -products Microsoft.VisualStudio.Product.BuildTools -version [17.0^,18.0^) -property installationPath`) do set "VSROOT=%%i"
)
if not defined VSROOT set "VSROOT=%ProgramFiles(x86)%\Microsoft Visual Studio\2022\BuildTools"
if not exist "%VSROOT%\VC\Auxiliary\Build\vcvars64.bat" (
  echo vcvars64.bat not found under "%VSROOT%" 1>&2
  exit /b 1
)
call "%VSROOT%\VC\Auxiliary\Build\vcvars64.bat" >nul
if errorlevel 1 exit /b 1
set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
rem The repo path exceeds MAX_PATH, so build into a short directory.
if not defined CARGO_TARGET_DIR set "CARGO_TARGET_DIR=C:\at-target"
cd /d "%~dp0.."
