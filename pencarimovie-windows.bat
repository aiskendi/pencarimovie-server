@echo off
setlocal enabledelayedexpansion
title PencariMovie Server
call :print_banner

set "REPO=aiskendi/pencarimovie-server"
set "FALLBACK_TAG=v1.0.0"
set "PORT=8088"
set "HAD_APP=0"
set "UPDATED=0"
set "IN_PLACE=0"
rem IN_PLACE = running from the developer git repo (has .git). Installed app
rem folders also contain backend.php/start.bat but must still auto-update, so
rem only treat a git checkout as "in place" (skip OTA).
if exist "%~dp0backend.php" if exist "%~dp0start.bat" if exist "%~dp0.git" set "IN_PLACE=1"

if "%IN_PLACE%"=="1" (
    set "APP_DIR=%~dp0."
) else (
    set "APP_DIR=%USERPROFILE%\pencarimovie-server"
)

if "%1"=="--stop" goto stop
if "%1"=="stop" goto stop
if "%1"=="--restart" goto restart
if "%1"=="restart" goto restart
if "%1"=="--start" goto start
if "%1"=="start" goto start
if "%1"=="--uninstall" goto uninstall
if "%1"=="uninstall" goto uninstall
if not "%1"=="" (
    echo Usage: %~nx0 [start^|stop^|restart^|uninstall]
    pause
    exit /b 1
)

:start
if exist "%USERPROFILE%\pencarimovie-downloader" (
    if exist "%USERPROFILE%\pencarimovie-downloader\storage" if not exist "%APP_DIR%\storage" (
        mkdir "%APP_DIR%" 2>nul
        robocopy "%USERPROFILE%\pencarimovie-downloader\storage" "%APP_DIR%\storage" /e /np /nfl /ndl /njh /njs >nul 2>&1
    )
    rmdir /s /q "%USERPROFILE%\pencarimovie-downloader" 2>nul
)
if exist "%APP_DIR%" set "HAD_APP=1"
call :install_or_update
call :register_cmd_path

curl -s -o nul http://127.0.0.1:%PORT% >nul 2>&1
if not errorlevel 1 goto port_busy
powershell -NoProfile -Command "try { $r=Invoke-WebRequest -Uri 'http://127.0.0.1:%PORT%' -Method HEAD -TimeoutSec 2; exit 0 } catch { exit 1 }" >nul 2>&1
if not errorlevel 1 goto port_busy
goto not_running

:port_busy
if "%HAD_APP%"=="1" if "%UPDATED%"=="0" goto already_running
echo Port %PORT% is already in use; stopping leftover process...
call :stop_quiet
ping 127.0.0.1 -n 2 >nul
goto not_running

:already_running
echo Server is already running on port %PORT%.
call :start_tray 1
call :print_urls
echo   CLI:      pms [start^|stop^|restart]
echo   Stop:     pms stop
echo   Restart:  pms restart
echo   Tray:     right-click the PencariMovie icon in the system tray
exit /b 0

:not_running
if not exist "%APP_DIR%" (
    echo App directory was not installed.
    pause
    exit /b 1
)
cd /d "%APP_DIR%"
echo Starting PencariMovie Server in the background...
if exist "%cd%\tray.ps1" (
    call :start_tray 1
) else (
    if exist "bin\frankenphp.exe" (
        if exist "%cd%\start-hidden.ps1" (
            powershell -NoProfile -ExecutionPolicy Bypass -File "%cd%\start-hidden.ps1" -FilePath "%cd%\bin\frankenphp.exe" -CommandLine "php-server --listen 0.0.0.0:%PORT% --root ""%cd%"""
        ) else (
            start "PencariMovie Server" /MIN "%cd%\bin\frankenphp.exe" php-server --listen 0.0.0.0:%PORT% --root "%cd%"
        )
    ) else (
        if exist "%cd%\start-hidden.ps1" (
            powershell -NoProfile -ExecutionPolicy Bypass -File "%cd%\start-hidden.ps1" -FilePath php -CommandLine "-S 0.0.0.0:%PORT% router.php"
        ) else (
            start "PencariMovie Server" /MIN php -S 0.0.0.0:%PORT% router.php
        )
    )
    call :start_tray
)

echo.
echo PencariMovie Server is running in the background.
call :print_urls
echo   CLI:      pms [start^|stop^|restart]
echo   Stop:     pms stop
echo   Restart:  pms restart
echo   Tray:     right-click the PencariMovie icon in the system tray
exit /b 0

:stop
echo Stopping PencariMovie Server on 0.0.0.0:%PORT%...
call :stop_quiet
echo Server stopped.
exit /b 0

:uninstall
echo Stopping PencariMovie Server...
call :stop_quiet
rem Remove CLI wrappers
del /q "%USERPROFILE%\pencarimovie-server\pm.cmd" 2>nul
del /q "%USERPROFILE%\pencarimovie-server\pms.cmd" 2>nul
del /q "%USERPROFILE%\pencarimovie-server\pencarimovie.cmd" 2>nul
rem Remove the app directory
if exist "%APP_DIR%" (
    echo Removing %APP_DIR% ...
    rmdir /s /q "%APP_DIR%" 2>nul
)
echo PencariMovie Server has been uninstalled.
exit /b 0

:restart
call :stop_quiet
ping 127.0.0.1 -n 3 >nul
goto start

:stop_quiet
rem Kill port 8089 (Addon)
powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort 8089 -State Listen -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess | ForEach-Object { Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue }" >nul 2>nul

rem Kill port %PORT% (Server)
powershell -NoProfile -Command "Get-NetTCPConnection -LocalPort %PORT% -State Listen -ErrorAction SilentlyContinue | Select-Object -ExpandProperty OwningProcess | ForEach-Object { Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue }" >nul 2>nul

powershell -NoProfile -Command "Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object { $_.Name -like '*addon*' -or ($_.CommandLine -and ($_.CommandLine -like '*addon.js*' -or $_.CommandLine -like '*addon.exe*')) } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }" >nul 2>nul

rem Kill any leftover FrankenPHP or PHP CLI IPC processes that may be holding DLL locks (e.g. php_curl.dll)
powershell -NoProfile -Command "Get-Process -Name frankenphp,php -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path -like '*pencarimovie*' } | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }" >nul 2>nul
call :stop_tray
goto :eof

:start_tray
if not exist "%APP_DIR%\tray.ps1" goto :eof
if not exist "%APP_DIR%\start-hidden.ps1" (
    start "PencariMovie Tray" /MIN powershell.exe -NoProfile -STA -WindowStyle Hidden -ExecutionPolicy Bypass -File "%APP_DIR%\tray.ps1" -Port %PORT% -OpenUrl http://127.0.0.1:%PORT% -StopBat "%APP_DIR%\stop.bat" -StartServer
    goto :eof
)
powershell -NoProfile -ExecutionPolicy Bypass -File "%APP_DIR%\start-hidden.ps1" -FilePath powershell.exe -CommandLine "-NoProfile -STA -WindowStyle Hidden -ExecutionPolicy Bypass -File ""%APP_DIR%\tray.ps1"" -Port %PORT% -OpenUrl http://127.0.0.1:%PORT% -StopBat ""%APP_DIR%\stop.bat"" -StartServer"
goto :eof

:stop_tray
powershell -NoProfile -Command "try { $e = New-Object System.Threading.EventWaitHandle $false, ([System.Threading.EventResetMode]::AutoReset), 'Global\PencariMovieServerTrayStop'; $e.Set() | Out-Null; $e.Dispose() } catch {}" >nul 2>nul
ping 127.0.0.1 -n 2 >nul
if exist "%APP_DIR%\storage\tray.pid" (
    for /f "usebackq delims=" %%P in ("%APP_DIR%\storage\tray.pid") do (
        if not "%%P"=="" powershell -NoProfile -Command "Stop-Process -Id %%P -Force -ErrorAction SilentlyContinue" >nul 2>nul
    )
    del /q "%APP_DIR%\storage\tray.pid" >nul 2>nul
)
powershell -NoProfile -Command "Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -and $_.CommandLine -match 'pencarimovie.+tray\.ps1' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }" >nul 2>nul
goto :eof

:print_urls
echo   Local:    http://127.0.0.1:%PORT%
powershell -NoProfile -Command "& { $ip = (Get-NetIPAddress -AddressFamily IPv4 -InterfaceAlias 'Wi-Fi*','Ethernet*','vEthernet*' -ErrorAction SilentlyContinue | Where-Object { $_.IPAddress -notlike '127.*' -and $_.IPAddress -notlike '169.254.*' } | Select-Object -First 1 -ExpandProperty IPAddress); if ($ip) { Write-Host ('  Network:  http://' + $ip + ':%PORT%') } }" 2>nul
goto :eof

:install_or_update
if "%IN_PLACE%"=="1" (
    echo Starting from this folder; skipping GitHub extract.
    goto :eof
)
set "APP_PATH=!APP_DIR!"
set "CURRENT="
if exist "!APP_PATH!\.release-tag" (
    for /f "usebackq delims=" %%A in ("!APP_PATH!\.release-tag") do set "CURRENT=%%A"
)
if exist "!APP_PATH!" if not defined CURRENT (
    set "CURRENT=%FALLBACK_TAG%"
    >"!APP_PATH!\.release-tag" echo %FALLBACK_TAG%
)

set "LATEST="
for /f "usebackq delims=" %%i in (`powershell -NoProfile -Command "$ErrorActionPreference='Stop'; [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; try { $r = Invoke-RestMethod -Uri 'https://api.github.com/repos/%REPO%/releases/latest' -Headers @{'User-Agent'='pencarimovie-server'}; if ($r.tag_name -match '^v[0-9]') { $r.tag_name; exit 0 } } catch {}; $req = [System.Net.HttpWebRequest]::Create('https://github.com/%REPO%/releases/latest'); $req.AllowAutoRedirect = $true; $req.Method = 'GET'; $req.UserAgent = 'pencarimovie-server'; try { $resp = $req.GetResponse(); $loc = [string]$resp.ResponseUri; $resp.Close(); $tag = ($loc.TrimEnd('/') -split '/')[-1]; if ($tag -match '^v[0-9]') { $tag; exit 0 } } catch {}; exit 1"`) do set "LATEST=%%i"

if not defined LATEST (
    if exist "!APP_PATH!" (
        echo Could not check GitHub for updates; using installed copy.
        goto :eof
    )
    set "LATEST=%FALLBACK_TAG%"
)

if exist "!APP_PATH!" if /I "!CURRENT!"=="!LATEST!" goto :eof

if not exist "!APP_PATH!" (
    echo Downloading PencariMovie Server !LATEST!...
) else (
    echo Updating PencariMovie Server !CURRENT! -^> !LATEST!...
    call :stop_quiet
    ping 127.0.0.1 -n 2 >nul
)

set "OTA_URL=https://github.com/%REPO%/releases/download/!LATEST!/pencarimovie-downloader-windows-x86_64.zip"
powershell -NoProfile -ExecutionPolicy Bypass -Command "& { param($appDir, $tag, $url) $ErrorActionPreference='Stop'; [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12; Get-Process -Name frankenphp,php,cloudflared,addon -ErrorAction SilentlyContinue | Where-Object { try { $_.Path -and $_.Path -like ('*' + $appDir + '*') } catch { $false } } | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue }; Start-Sleep -Milliseconds 800; $tmp = Join-Path $env:TEMP ('pencarimovie-ota-' + [guid]::NewGuid().ToString()); New-Item -ItemType Directory -Path (Join-Path $tmp 'extract') -Force | Out-Null; Write-Host ('Downloading ' + $url); Invoke-WebRequest -Uri $url -OutFile (Join-Path $tmp 'pencarimovie.zip') -UseBasicParsing; Expand-Archive -Path (Join-Path $tmp 'pencarimovie.zip') -DestinationPath (Join-Path $tmp 'extract') -Force; $found = Get-ChildItem -Path (Join-Path $tmp 'extract') -Recurse -Filter 'backend.php' | Select-Object -First 1; if ($found) { $src = $found.DirectoryName } else { $src = Join-Path $tmp 'extract' }; if (-not (Test-Path -LiteralPath $appDir)) { New-Item -ItemType Directory -Path $appDir -Force | Out-Null }; Get-ChildItem -LiteralPath $src | Where-Object { $_.Name -ne 'storage' } | ForEach-Object { $dest = Join-Path $appDir $_.Name; for ($retry=0; $retry -lt 8; $retry++) { try { if (Test-Path $dest) { Remove-Item $dest -Recurse -Force -ErrorAction Stop }; Copy-Item $_.FullName $dest -Recurse -Force -ErrorAction Stop; break } catch { if ($retry -eq 7) { throw }; Start-Sleep -Milliseconds 500 } } }; [System.IO.File]::WriteAllText((Join-Path $appDir '.release-tag'), $tag + [char]10, [System.Text.Encoding]::ASCII); Remove-Item $tmp -Recurse -Force }" -appDir "%APP_DIR%" -tag "!LATEST!" -url "!OTA_URL!"
set "PS_ERR=!ERRORLEVEL!"
if not "!PS_ERR!"=="0" (
    echo Update download/extract failed.
    pause
    exit /b 1
)
set "UPDATED=1"
call :register_cmd_path
goto :eof

:register_cmd_path
if not exist "%USERPROFILE%\pencarimovie-server" mkdir "%USERPROFILE%\pencarimovie-server" 2>nul
(
    echo @echo off
    echo "%USERPROFILE%\pencarimovie-server\pencarimovie-windows.bat" %%*
) > "%USERPROFILE%\pencarimovie-server\pm.cmd" 2>nul
(
    echo @echo off
    echo "%USERPROFILE%\pencarimovie-server\pencarimovie-windows.bat" %%*
) > "%USERPROFILE%\pencarimovie-server\pms.cmd" 2>nul
(
    echo @echo off
    echo "%USERPROFILE%\pencarimovie-server\pencarimovie-windows.bat" %%*
) > "%USERPROFILE%\pencarimovie-server\pencarimovie.cmd" 2>nul

powershell -NoProfile -ExecutionPolicy Bypass -Command "& { $dir = Join-Path $env:USERPROFILE 'pencarimovie-server'; $curr = [Environment]::GetEnvironmentVariable('Path', 'User'); if ($curr -notlike ('*' + $dir + '*')) { [Environment]::SetEnvironmentVariable('Path', ($curr.TrimEnd(';') + ';' + $dir), 'User'); $env:Path += ';' + $dir } }" >nul 2>&1
goto :eof

:print_banner
echo(
echo  ========================================
echo           PencariMovie Server
echo  ========================================
echo(
goto :eof

endlocal
