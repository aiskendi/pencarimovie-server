@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
for %%I in ("%ROOT%") do set "ROOT=%%~fI"
set "DIST=%ROOT%\dist"
set "BUILD=%DIST%\.build-win-%RANDOM%%RANDOM%"
set "ZIP=%DIST%\pencarimovie-downloader-windows-x86_64.zip"

echo Building Windows release package...

REM Kill any lingering server processes that might lock vendor or runtime files
taskkill /F /IM php.exe >nul 2>nul
taskkill /F /IM frankenphp.exe >nul 2>nul
taskkill /F /IM cloudflared.exe >nul 2>nul

REM Clean any stale build folders if possible
if exist "%DIST%\pencarimovie-downloader-windows-x86_64" (
  rmdir /s /q "%DIST%\pencarimovie-downloader-windows-x86_64" 2>nul
)
if exist "%DIST%\pencarimovie-old" (
  rmdir /s /q "%DIST%\pencarimovie-old" 2>nul
)
if exist "%ZIP%" del /f /q "%ZIP%" 2>nul
if not exist "%DIST%" mkdir "%DIST%"
if not exist "%BUILD%" mkdir "%BUILD%"

call :copy_file Caddyfile
call :copy_file backend.php
call :copy_file index.php
call :copy_file router.php
call :copy_file install.bat
call :copy_file start.bat
call :copy_file stop.bat
call :copy_file restart.bat
call :copy_file pencarimovie-windows.bat
call :copy_file update.ps1
call :copy_file auth-write.ps1
call :copy_file tray.ps1
call :copy_file tray.ico
call :copy_file tray.png
call :copy_file start-hidden.ps1
call :copy_file tunnel-spawn.ps1
call :copy_file package.json
call :copy_file README.md
call :copy_file LICENSE
call :copy_file SECURITY.md

xcopy "%ROOT%\public" "%BUILD%\public" /E /I /Y >nul
if exist "%ROOT%\patches" (
  xcopy "%ROOT%\patches" "%BUILD%\patches" /E /I /Y >nul
)
mkdir "%BUILD%\storage"
copy /Y "%ROOT%\storage\.gitkeep" "%BUILD%\storage\.gitkeep" >nul 2>nul
copy /Y "%ROOT%\storage\config.example.json" "%BUILD%\storage\config.example.json" >nul
REM NOTE: storage\catalog_settings.json is intentionally NOT shipped. It is
REM per-user state; shipping the developer's copy would override the built-in
REM defaults in fd_load_catalog_settings() (including the default AIOMetadata
REM upstream manifest) on every fresh install.

REM Extract bin/ from official FrankenPHP Windows release ZIP directly to BUILD/bin
REM Then overlay repo php.ini (official ZIP only has php.ini-development/production templates)
if exist "%ROOT%\frankenphp-windows-x86_64.zip" (
  echo Extracting Windows FrankenPHP runtime from frankenphp-windows-x86_64.zip...
  mkdir "%BUILD%\bin"
  powershell -NoProfile -Command "Expand-Archive -Force '%ROOT%\frankenphp-windows-x86_64.zip' -DestinationPath '%BUILD%\bin'"
  if exist "%ROOT%\bin\php.ini" (
    echo Overlaying repo php.ini for Windows runtime...
    copy /Y "%ROOT%\bin\php.ini" "%BUILD%\bin\php.ini" >nul
  )
) else (
  echo WARNING: frankenphp-windows-x86_64.zip not found. Using repo bin/ as fallback.
  xcopy "%ROOT%\bin" "%BUILD%\bin" /E /I /Y >nul
)

REM Prune unused PHP dev/test binaries that trigger antivirus false positives.
REM php_dl_test.dll is a test-only extension (never loaded in production) and is a
REM well-known generic Riskware/HackTool false-positive magnet. phpdbg/php-cgi/
REM php-win/apache DLLs are also unused by FrankenPHP and only add unsigned EXEs.
if exist "%BUILD%\bin\ext\php_dl_test.dll" del /f /q "%BUILD%\bin\ext\php_dl_test.dll" >nul 2>nul
if exist "%BUILD%\bin\ext\php_zend_test.dll" del /f /q "%BUILD%\bin\ext\php_zend_test.dll" >nul 2>nul
if exist "%BUILD%\bin\phpdbg.exe" del /f /q "%BUILD%\bin\phpdbg.exe" >nul 2>nul
if exist "%BUILD%\bin\php8phpdbg.dll" del /f /q "%BUILD%\bin\php8phpdbg.dll" >nul 2>nul
if exist "%BUILD%\bin\php-cgi.exe" del /f /q "%BUILD%\bin\php-cgi.exe" >nul 2>nul
if exist "%BUILD%\bin\php-win.exe" del /f /q "%BUILD%\bin\php-win.exe" >nul 2>nul
if exist "%BUILD%\bin\php8apache2_4.dll" del /f /q "%BUILD%\bin\php8apache2_4.dll" >nul 2>nul
if exist "%BUILD%\bin\deplister.exe" del /f /q "%BUILD%\bin\deplister.exe" >nul 2>nul
if exist "%BUILD%\bin\dev" rmdir /s /q "%BUILD%\bin\dev" >nul 2>nul

REM No FFmpeg is bundled. ALAC -> FLAC is handled by alac-decoder.php (pure PHP),
REM and AAC/MP3/FLAC are decoded natively by the client.

if exist "%ROOT%\vendor\autoload.php" (
  powershell -NoProfile -ExecutionPolicy Bypass -Command "Copy-Item -LiteralPath '%ROOT%\vendor' -Destination '%BUILD%\vendor' -Recurse -Force"
)

REM Apply the MadelineProto IPC patch to the COPIED vendor tree so the release
REM always ships the patched getSlow()/ProcessRunner/ClientAbstract, regardless
REM of whether the dev tree's vendor/ was patched by composer-patches. Without
REM this a release built from an unpatched vendor/ ships the old
REM `getSlow()` (Magic::$isIpcWorker || PHP_OS_FAMILY === 'Windows'), which
REM forces a full-mode boot on Windows and produces the
REM "It seems like the session is busy." / "Could not connect to MadelineProto"
REM storm on a fresh install.
if exist "%BUILD%\vendor\danog\madelineproto" (
  if exist "%ROOT%\patches\madelineproto-windows-ipc.patch" (
    pushd "%BUILD%\vendor\danog\madelineproto"
    git apply --recount --whitespace=nowarn "%ROOT%\patches\madelineproto-windows-ipc.patch" 2>nul
    if errorlevel 1 (
      echo NOTE: git apply reported issues; verifying getSlow() directly...
    )
    popd
  )
)

REM Hard guarantee: if the patch could not be applied (no git, or already
REM applied), rewrite getSlow() in the copied vendor tree so the release can
REM never ship the Windows full-boot form.
REM
REM Write with a BOM-less UTF8 encoding: Set-Content -Encoding UTF8 prepends a
REM BOM, which makes `declare(strict_types=1)` fail with
REM "strict_types declaration must be the very first statement".
powershell -NoProfile -ExecutionPolicy Bypass -Command "$p = '%BUILD%\vendor\danog\madelineproto\src\Settings\Ipc.php'; if (Test-Path $p) { $c = [System.IO.File]::ReadAllText($p); $old = 'return Magic::$isIpcWorker || \PHP_OS_FAMILY === ''Windows'';'; $new = 'return Magic::$isIpcWorker || !empty($GLOBALS[''FD_FORCE_FULL_BOOT'']);'; if ($c.Contains($old)) { $c = $c.Replace($old, $new); [System.IO.File]::WriteAllText($p, $c, (New-Object System.Text.UTF8Encoding $false)); Write-Host 'getSlow() patched in release vendor tree' } elseif ($c.Contains($new)) { Write-Host 'getSlow() already patched' } else { Write-Host 'WARNING: getSlow() form not recognised' } }"

if exist "%ROOT%\.release-tag" (
  copy /Y "%ROOT%\.release-tag" "%BUILD%\.release-tag" >nul
)

echo Creating %ZIP%...
tar -acf "%ZIP%" -C "%BUILD%" .
if errorlevel 1 (
  powershell -NoProfile -Command "Compress-Archive -Force -Path '%BUILD%\*' -DestinationPath '%ZIP%'"
)
if errorlevel 1 (
  echo ERROR: Failed to create %ZIP%
  exit /b 1
)

REM Publish a SHA-256 sidecar so update.ps1 can verify the download before
REM extracting. This also gives antivirus heuristics a legitimate integrity
REM check to weigh against the download-and-extract pattern.
powershell -NoProfile -Command "$h = (Get-FileHash -LiteralPath '%ZIP%' -Algorithm SHA256).Hash; Set-Content -LiteralPath '%ZIP%.sha256' -Value $h -Encoding ASCII"
echo SHA-256 sidecar written to %ZIP%.sha256

rmdir /s /q "%BUILD%"
echo Windows package created successfully at %ZIP%
exit /b 0

:copy_file
if exist "%ROOT%\%~1" (
  copy /Y "%ROOT%\%~1" "%BUILD%\%~1" >nul
)
goto :eof
