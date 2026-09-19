@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
for %%I in ("%ROOT%") do set "ROOT=%%~fI"

set "TAG=%~1"
if "%TAG%"=="" set "TAG=apk-latest"

set "ARM64_APK=%ROOT%\android\termux-app-fork\app\build\outputs\apk\release\pencarimovie_arm64-v8a.apk"
set "X86_64_APK=%ROOT%\android\termux-app-fork-x86_64\app\build\outputs\apk\release\pencarimovie_x86_64.apk"

echo ======================================================================
echo  PencariMovie Downloader - Release APKs to GitHub
echo ======================================================================
echo Target tag: %TAG%
echo.

if not exist "%ARM64_APK%" (
    echo [ERROR] ARM64 APK not found at:
    echo   %ARM64_APK%
    exit /b 1
)

if not exist "%X86_64_APK%" (
    echo [ERROR] x86_64 APK not found at:
    echo   %X86_64_APK%
    exit /b 1
)

echo [1/2] Checking if release "%TAG%" exists on GitHub...
gh release view "%TAG%" >nul 2>nul
if errorlevel 1 (
    echo [2/2] Creating release "%TAG%" and uploading APKs...
    gh release create "%TAG%" "%ARM64_APK%" "%X86_64_APK%" --title "PencariMovie Downloader - Android APK" --notes "Standalone Android APK releases for PencariMovie Downloader"
) else (
    echo [2/2] Uploading APKs to existing release "%TAG%" (--clobber)...
    gh release upload "%TAG%" "%ARM64_APK%" "%X86_64_APK%" --clobber
)

if errorlevel 1 (
    echo [ERROR] Failed to release APKs to GitHub.
    exit /b 1
)

echo.
echo [OK] APKs released successfully to:
echo   https://github.com/aiskendi/pencarimovie-server/releases/tag/%TAG%
echo.
