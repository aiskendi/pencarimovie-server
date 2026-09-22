@echo off
setlocal enabledelayedexpansion

set "ROOT=%~dp0.."
for %%I in ("%ROOT%") do set "ROOT=%%~fI"

echo ======================================================================
echo  PencariMovie Downloader - Local Release Builder
echo ======================================================================
echo.

set "TARGET_FILTER=%~1"
set "TAG_NAME=%~2"

if "%TARGET_FILTER%"=="" (
  echo Select release target to build:
  echo   1. All targets (Windows, Linux x86_64/gnu/mimalloc, Linux arm64/gnu, Mac, Server standalone)
  echo   2. Windows x86_64 only
  echo   3. Linux aarch64 (musl + gnu) only
  echo   4. Linux x86_64 (musl + gnu + mimalloc) only
  echo   5. Mac (arm64 and x86_64) only
  echo   6. Standalone pencarimovie-server.tar.gz (No FrankenPHP, universal native PHP)
  echo.
  set /p "CHOICE=Enter choice [1-6, default=1]: "
  if "!CHOICE!"=="2" set "TARGET_FILTER=win"
  if "!CHOICE!"=="3" set "TARGET_FILTER=arm64"
  if "!CHOICE!"=="4" set "TARGET_FILTER=linux-x64"
  if "!CHOICE!"=="5" set "TARGET_FILTER=mac"
  if "!CHOICE!"=="6" set "TARGET_FILTER=server"
  if "!TARGET_FILTER!"=="" set "TARGET_FILTER=all"
)

if "%TAG_NAME%"=="" (
  set /p "TAG_INPUT=Enter release tag (e.g. v1.9.0 or v1.9.0-beta.1) [or press Enter to skip]: "
  if not "!TAG_INPUT!"=="" set "TAG_NAME=!TAG_INPUT!"
)

if not "%TAG_NAME%"=="" (
  echo Setting release stamp tag: %TAG_NAME%
  <nul set /p "=%TAG_NAME%"> "%ROOT%\.release-tag"
  call :update_version "%TAG_NAME%"
)

echo.
echo Target selected: %TARGET_FILTER%
echo.

REM ------------------------------------------------------------------
REM Step 1: Download latest FrankenPHP assets from GitHub (if needed)
REM ------------------------------------------------------------------
if not "%TARGET_FILTER%"=="server" (
  echo [1/2] Checking and fetching latest FrankenPHP binaries from GitHub...
  powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0download-frankenphp.ps1" -Root "%ROOT%"
  if errorlevel 1 (
    echo.
    echo ERROR: Failed to obtain FrankenPHP binaries from GitHub.
    exit /b 1
  )
) else (
  echo [1/2] Skipping FrankenPHP download for standalone server package.
)

echo.
REM ------------------------------------------------------------------
REM Step 2: Build release packages (Native PHP/cURL, no Bun required)
REM ------------------------------------------------------------------
echo [2/2] Packaging release archives...
echo.

REM Terminate any background servers or workers holding file handles
taskkill /F /IM php.exe >nul 2>nul
taskkill /F /IM frankenphp.exe >nul 2>nul
taskkill /F /IM cloudflared.exe >nul 2>nul

set "ROOT2=%ROOT:\=\\%"
if not exist "%ROOT%\dist" mkdir "%ROOT%\dist"

REM ------------------------------------------------------------------
REM Windows x86_64
REM ------------------------------------------------------------------
if "%TARGET_FILTER%"=="all" goto :build_win
if "%TARGET_FILTER%"=="win" goto :build_win
goto :skip_win

:build_win
echo Packaging Windows x86_64...
call "%~dp0package-windows.bat"
if errorlevel 1 (
  echo ERROR: Windows package build failed
  if "%TARGET_FILTER%"=="win" exit /b 1
)
:skip_win

REM ------------------------------------------------------------------
REM Unix packages
REM ------------------------------------------------------------------
REM Linux x86_64 variants (musl, gnu, mimalloc)
if "%TARGET_FILTER%"=="all" goto :build_linux_x64
if "%TARGET_FILTER%"=="linux-x64" goto :build_linux_x64
goto :skip_linux_x64

:build_linux_x64
echo Packaging Linux x86_64 variants (musl, gnu, mimalloc)...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $root='%ROOT2%'; $dist=$root+'\dist'; foreach($target in @('frankenphp-linux-x86_64','frankenphp-linux-x86_64-gnu','frankenphp-linux-x86_64-mimalloc')){$source=$root+'\'+$target; if(-not (Test-Path $source)){Write-Warning('Source not found: '+$source);continue}; $pkg=$target.Substring(11); $build=$dist+'\.build-tmp\pencarimovie-downloader-'+$pkg; Remove-Item -Recurse -Force $build -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force -Path ($build+'\bin')|Out-Null; foreach($f in @('Caddyfile','backend.php','index.php','router.php','install.sh','start.sh','restart.sh','stop.sh','pencarimovie-linux.sh','package.json','README.md','LICENSE','SECURITY.md','patches')){$src=$root+'\'+$f;if(Test-Path $src){Copy-Item -Recurse -Force $src $build}}; Copy-Item -Recurse -Force ($root+'\public') ($build+'\public'); New-Item -ItemType Directory -Force -Path ($build+'\storage')|Out-Null; Copy-Item -Force ($root+'\storage\.gitkeep') ($build+'\storage\.gitkeep') -ErrorAction SilentlyContinue; if(Test-Path ($root+'\storage\config.example.json')){Copy-Item -Force ($root+'\storage\config.example.json') ($build+'\storage\config.example.json')}; if(Test-Path ($root+'\storage\catalog_settings.json')){Copy-Item -Force ($root+'\storage\catalog_settings.json') ($build+'\storage\catalog_settings.json')}; Copy-Item -Force ($root+'\bin\php.ini.unix') ($build+'\bin\php.ini'); Copy-Item -Force ($root+'\bin\php') ($build+'\bin\php'); Copy-Item -Force $source ($build+'\bin\frankenphp'); if(Test-Path ($root+'\vendor\autoload.php')){Copy-Item -Recurse -Force ($root+'\vendor') ($build+'\vendor')}; if(Test-Path ($root+'\.release-tag')){Copy-Item -Force ($root+'\.release-tag') ($build+'\.release-tag')}; Get-ChildItem -Path $build -Filter *.sh -Recurse | ForEach-Object { $c = [System.IO.File]::ReadAllText($_.FullName) -replace \"`r\", ''; [System.IO.File]::WriteAllText($_.FullName, $c, (New-Object System.Text.UTF8Encoding $false)) }; $bp = Join-Path $build 'bin\php'; if (Test-Path $bp) { $c = [System.IO.File]::ReadAllText($bp) -replace \"`r\", ''; [System.IO.File]::WriteAllText($bp, $c, (New-Object System.Text.UTF8Encoding $false)) }; $tar=$dist+'\pencarimovie-downloader-'+$pkg+'.tar.gz'; tar -czf $tar -C ($dist+'\.build-tmp') ('pencarimovie-downloader-'+$pkg); Remove-Item -Recurse -Force $build}"
if errorlevel 1 echo ERROR: Linux x86_64 packaging failed
:skip_linux_x64

REM Linux aarch64 variants (musl/Termux, gnu)
if "%TARGET_FILTER%"=="all" goto :build_arm64
if "%TARGET_FILTER%"=="arm64" goto :build_arm64
goto :skip_arm64

:build_arm64
echo Packaging Linux aarch64 variants (musl/Termux, gnu)...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $root='%ROOT2%'; $dist=$root+'\dist'; foreach($target in @('frankenphp-linux-aarch64','frankenphp-linux-aarch64-gnu')){$source=$root+'\'+$target; if(-not (Test-Path $source)){Write-Warning('Source not found: '+$source);continue}; $pkg=$target.Substring(11); $build=$dist+'\.build-tmp\pencarimovie-downloader-'+$pkg; Remove-Item -Recurse -Force $build -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force -Path ($build+'\bin')|Out-Null; foreach($f in @('Caddyfile','backend.php','index.php','router.php','install.sh','install-termux.sh','start.sh','start-termux.sh','restart.sh','restart-termux.sh','stop.sh','pencarimovie-linux.sh','pencarimovie-termux.sh','package.json','README.md','LICENSE','SECURITY.md','patches')){$src=$root+'\'+$f;if(Test-Path $src){Copy-Item -Recurse -Force $src $build}}; Copy-Item -Recurse -Force ($root+'\public') ($build+'\public'); New-Item -ItemType Directory -Force -Path ($build+'\storage')|Out-Null; Copy-Item -Force ($root+'\storage\.gitkeep') ($build+'\storage\.gitkeep') -ErrorAction SilentlyContinue; if(Test-Path ($root+'\storage\config.example.json')){Copy-Item -Force ($root+'\storage\config.example.json') ($build+'\storage\config.example.json')}; if(Test-Path ($root+'\storage\catalog_settings.json')){Copy-Item -Force ($root+'\storage\catalog_settings.json') ($build+'\storage\catalog_settings.json')}; Copy-Item -Force ($root+'\bin\php.ini.unix') ($build+'\bin\php.ini'); Copy-Item -Force ($root+'\bin\php') ($build+'\bin\php'); Copy-Item -Force $source ($build+'\bin\frankenphp'); if(Test-Path ($root+'\vendor\autoload.php')){Copy-Item -Recurse -Force ($root+'\vendor') ($build+'\vendor')}; if(Test-Path ($root+'\.release-tag')){Copy-Item -Force ($root+'\.release-tag') ($build+'\.release-tag')}; Get-ChildItem -Path $build -Filter *.sh -Recurse | ForEach-Object { $c = [System.IO.File]::ReadAllText($_.FullName) -replace \"`r\", ''; [System.IO.File]::WriteAllText($_.FullName, $c, (New-Object System.Text.UTF8Encoding $false)) }; $bp = Join-Path $build 'bin\php'; if (Test-Path $bp) { $c = [System.IO.File]::ReadAllText($bp) -replace \"`r\", ''; [System.IO.File]::WriteAllText($bp, $c, (New-Object System.Text.UTF8Encoding $false)) }; $tar=$dist+'\pencarimovie-downloader-'+$pkg+'.tar.gz'; tar -czf $tar -C ($dist+'\.build-tmp') ('pencarimovie-downloader-'+$pkg); Remove-Item -Recurse -Force $build}"
if errorlevel 1 echo ERROR: Linux aarch64 packaging failed
:skip_arm64

REM Mac
if "%TARGET_FILTER%"=="all" goto :build_mac
if "%TARGET_FILTER%"=="mac" goto :build_mac
goto :skip_mac

:build_mac
echo Packaging Mac arm64 and x86_64...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $root='%ROOT2%'; $dist=$root+'\dist'; foreach($target in @('frankenphp-mac-arm64','frankenphp-mac-x86_64')){$source=$root+'\'+$target; if(-not (Test-Path $source)){Write-Warning('Source not found: '+$source);continue}; $pkg=$target.Substring(11); $build=$dist+'\.build-tmp\pencarimovie-downloader-'+$pkg; Remove-Item -Recurse -Force $build -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force -Path ($build+'\bin')|Out-Null; foreach($f in @('Caddyfile','backend.php','index.php','router.php','install.sh','start.sh','restart.sh','stop.sh','pencarimovie-linux.sh','package.json','README.md','LICENSE','SECURITY.md','patches')){$src=$root+'\'+$f;if(Test-Path $src){Copy-Item -Recurse -Force $src $build}}; Copy-Item -Recurse -Force ($root+'\public') ($build+'\public'); New-Item -ItemType Directory -Force -Path ($build+'\storage')|Out-Null; Copy-Item -Force ($root+'\storage\.gitkeep') ($build+'\storage\.gitkeep') -ErrorAction SilentlyContinue; if(Test-Path ($root+'\storage\config.example.json')){Copy-Item -Force ($root+'\storage\config.example.json') ($build+'\storage\config.example.json')}; if(Test-Path ($root+'\storage\catalog_settings.json')){Copy-Item -Force ($root+'\storage\catalog_settings.json') ($build+'\storage\catalog_settings.json')}; Copy-Item -Force ($root+'\bin\php.ini.unix') ($build+'\bin\php.ini'); Copy-Item -Force ($root+'\bin\php') ($build+'\bin\php'); Copy-Item -Force $source ($build+'\bin\frankenphp'); if(Test-Path ($root+'\vendor\autoload.php')){Copy-Item -Recurse -Force ($root+'\vendor') ($build+'\vendor')}; if(Test-Path ($root+'\.release-tag')){Copy-Item -Force ($root+'\.release-tag') ($build+'\.release-tag')}; Get-ChildItem -Path $build -Filter *.sh -Recurse | ForEach-Object { $c = [System.IO.File]::ReadAllText($_.FullName) -replace \"`r\", ''; [System.IO.File]::WriteAllText($_.FullName, $c, (New-Object System.Text.UTF8Encoding $false)) }; $bp = Join-Path $build 'bin\php'; if (Test-Path $bp) { $c = [System.IO.File]::ReadAllText($bp) -replace \"`r\", ''; [System.IO.File]::WriteAllText($bp, $c, (New-Object System.Text.UTF8Encoding $false)) }; $tar=$dist+'\pencarimovie-downloader-'+$pkg+'.tar.gz'; tar -czf $tar -C ($dist+'\.build-tmp') ('pencarimovie-downloader-'+$pkg); Remove-Item -Recurse -Force $build}"
:skip_mac

REM ------------------------------------------------------------------
REM Standalone Universal Server (No FrankenPHP, for native Termux/Linux PHP)
REM ------------------------------------------------------------------
if "%TARGET_FILTER%"=="all" goto :build_server
if "%TARGET_FILTER%"=="server" goto :build_server
goto :skip_server

:build_server
echo Packaging Universal Standalone Server (pencarimovie-server.tar.gz without FrankenPHP)...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ErrorActionPreference='Stop'; $root='%ROOT2%'; $dist=$root+'\dist'; $build=$dist+'\.build-tmp\pencarimovie-server'; Remove-Item -Recurse -Force $build -ErrorAction SilentlyContinue; New-Item -ItemType Directory -Force -Path ($build+'\bin')|Out-Null; foreach($f in @('Caddyfile','backend.php','index.php','router.php','install.sh','install-termux.sh','start.sh','start-termux.sh','restart.sh','restart-termux.sh','stop.sh','pencarimovie-linux.sh','pencarimovie-termux.sh','package.json','README.md','LICENSE','SECURITY.md','patches')){$src=$root+'\'+$f;if(Test-Path $src){Copy-Item -Recurse -Force $src $build}}; Copy-Item -Recurse -Force ($root+'\public') ($build+'\public'); New-Item -ItemType Directory -Force -Path ($build+'\storage')|Out-Null; Copy-Item -Force ($root+'\storage\.gitkeep') ($build+'\storage\.gitkeep') -ErrorAction SilentlyContinue; if(Test-Path ($root+'\storage\config.example.json')){Copy-Item -Force ($root+'\storage\config.example.json') ($build+'\storage\config.example.json')}; if(Test-Path ($root+'\storage\catalog_settings.json')){Copy-Item -Force ($root+'\storage\catalog_settings.json') ($build+'\storage\catalog_settings.json')}; Copy-Item -Force ($root+'\bin\php.ini.unix') ($build+'\bin\php.ini') -ErrorAction SilentlyContinue; Copy-Item -Force ($root+'\bin\php') ($build+'\bin\php') -ErrorAction SilentlyContinue; if(Test-Path ($root+'\vendor\autoload.php')){Copy-Item -Recurse -Force ($root+'\vendor') ($build+'\vendor')}; if(Test-Path ($root+'\.release-tag')){Copy-Item -Force ($root+'\.release-tag') ($build+'\.release-tag')}; Get-ChildItem -Path $build -Filter *.sh -Recurse | ForEach-Object { $c = [System.IO.File]::ReadAllText($_.FullName) -replace \"`r\", ''; [System.IO.File]::WriteAllText($_.FullName, $c, (New-Object System.Text.UTF8Encoding $false)) }; $bp = Join-Path $build 'bin\php'; if (Test-Path $bp) { $c = [System.IO.File]::ReadAllText($bp) -replace \"`r\", ''; [System.IO.File]::WriteAllText($bp, $c, (New-Object System.Text.UTF8Encoding $false)) }; $tar=$dist+'\pencarimovie-server.tar.gz'; tar -czf $tar -C ($dist+'\.build-tmp') 'pencarimovie-server'; Remove-Item -Recurse -Force $build"
if errorlevel 1 echo ERROR: Standalone server packaging failed
:skip_server

REM Clean build tmp
if exist "%ROOT%\dist\.build-tmp" rmdir /s /q "%ROOT%\dist\.build-tmp"

echo.
echo Done. Built packages are in "%ROOT%\dist":
dir /b "%ROOT%\dist"
exit /b 0

:update_version
set "RAW_TAG=%~1"
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ver = '%RAW_TAG%'.Trim() -replace '^[vV]', ''; if ($ver) { $utf8 = [System.Text.UTF8Encoding]::new($false); $bp = '%ROOT%\backend.php'; if (Test-Path $bp) { $c = [System.IO.File]::ReadAllText($bp); $u = [regex]::Replace($c, 'define\(''FD_APP_VERSION'',\s*''[^'']+''\);', ('define(''FD_APP_VERSION'', ''' + $ver + ''');')); [System.IO.File]::WriteAllText($bp, $u, $utf8); Write-Host ('[+] Updated backend.php FD_APP_VERSION to ' + $ver) }; $pj = '%ROOT%\package.json'; if (Test-Path $pj) { $c = [System.IO.File]::ReadAllText($pj); $u = [regex]::Replace($c, '\"version\":\s*\"[^\"]+\"', ('\"version\": \"' + $ver + '\"')); [System.IO.File]::WriteAllText($pj, $u, $utf8); Write-Host ('[+] Updated package.json version to ' + $ver) }; $aj = '%ROOT%\public\app.js'; if (Test-Path $aj) { $c = [System.IO.File]::ReadAllText($aj); $u = [regex]::Replace($c, 'this\.version\s*=\s*''[^'']+''', ('this.version = ''' + $ver + '''')); [System.IO.File]::WriteAllText($aj, $u, $utf8); Write-Host ('[+] Updated public/app.js version to ' + $ver) } }"
goto :eof
