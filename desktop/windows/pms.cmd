@echo off
rem ── PencariMovie Server launcher (native) ───────────────────────────────────
rem   pms                     start with the tray icon
rem   pms stop                stop the server (and the tray)
rem   pms restart             stop, then start with the tray
rem   pms uninstall           stop, remove autostart, delete the files
rem   pms autostart on|off    start with Windows / remove that
rem   pms tunnel              open the dashboard's tunnel settings
rem   pms <flags>             anything else goes to pencarimovie-server.exe
rem                           (--port 8088, --workers 2, --version, ...)
rem
rem The verbs are handled HERE on purpose: the binary treats "uninstall" as an
rem unknown flag and simply starts the server.
setlocal EnableExtensions EnableDelayedExpansion

set "PM_DIR="
if exist "%~dp0pencarimovie-server.exe" set "PM_DIR=%~dp0."
if not defined PM_DIR if exist "%USERPROFILE%\pencarimovie-server\pencarimovie-server.exe" set "PM_DIR=%USERPROFILE%\pencarimovie-server"
if not defined PM_DIR if exist "%~dp0port\rust\target\release\pencarimovie-server.exe" set "PM_DIR=%~dp0port\rust\target\release"

if not defined PM_DIR (
    echo PencariMovie Server is not installed.
    echo   Install:   irm telegra.my/win ^| iex
    echo   Releases:  https://github.com/aiskendi/pencarimovie-server/releases/latest
    exit /b 1
)

rem Bind all interfaces unless the user chose one, so the LAN/TV add-on URL works
rem on a fresh start. `--host 127.0.0.1` still wins (flags beat the environment).
if not defined PM_LISTEN_HOST if not defined HOST set "PM_LISTEN_HOST=0.0.0.0"

if /i "%~1"=="stop"       goto do_stop
if /i "%~1"=="--stop"     goto do_stop
if /i "%~1"=="restart"    goto do_restart
if /i "%~1"=="--restart"  goto do_restart
if /i "%~1"=="uninstall"  goto do_uninstall
if /i "%~1"=="--uninstall" goto do_uninstall
if /i "%~1"=="autostart"  goto do_autostart
if /i "%~1"=="tunnel"     goto do_tunnel
if /i "%~1"=="start"      goto do_start
if /i "%~1"=="--start"    goto do_start
if "%~1"==""              goto do_start
goto do_run

:do_start
if exist "%PM_DIR%\start-tray.bat" (
    pushd "%PM_DIR%"
    call start-tray.bat
    set "RC=%ERRORLEVEL%"
    popd
    exit /b %RC%
)
goto do_run

:do_run
pushd "%PM_DIR%"
if "%~1"=="" (
    "%PM_DIR%\pencarimovie-server.exe" --open
) else (
    "%PM_DIR%\pencarimovie-server.exe" %*
)
set "RC=%ERRORLEVEL%"
popd
exit /b %RC%

:do_stop
if exist "%PM_DIR%\stop.bat" (
    pushd "%PM_DIR%"
    call stop.bat
    set "RC=%ERRORLEVEL%"
    popd
    exit /b %RC%
)
echo Stopping PencariMovie Server...
taskkill /F /IM pencarimovie-server.exe >nul 2>&1
exit /b 0

:do_restart
call "%~f0" stop
timeout /t 1 /nobreak >nul
goto do_start

:do_autostart
if /i "%~2"=="off" (
    if exist "%PM_DIR%\autostart-off.bat" (
        pushd "%PM_DIR%"
        call autostart-off.bat
        set "RC=%ERRORLEVEL%"
        popd
        exit /b %RC%
    )
) else (
    if exist "%PM_DIR%\autostart-on.bat" (
        pushd "%PM_DIR%"
        call autostart-on.bat
        set "RC=%ERRORLEVEL%"
        popd
        exit /b %RC%
    )
)
echo Usage: pms autostart on^|off
exit /b 1

:do_tunnel
echo Opening the dashboard - tunnel settings are under Addon.
start "" "http://127.0.0.1:8088/#addon"
exit /b 0

:do_uninstall
rem ── SAFETY: only the per-user INSTALL may be deleted ───────────────────────
rem `pms` can resolve to a source checkout or to port\rust\target\release when it
rem is run from this repository. Uninstalling there would wipe the project, so
rem anything that is not exactly %USERPROFILE%\pencarimovie-server is refused.
set "PM_USER_DIR=%USERPROFILE%\pencarimovie-server"
for %%I in ("%PM_DIR%") do set "PM_ABS=%%~fI"
for %%I in ("%PM_USER_DIR%") do set "PM_USER_ABS=%%~fI"
if /i not "%PM_ABS%"=="%PM_USER_ABS%" (
    echo Refusing to delete: %PM_ABS%
    echo Only the installed copy can be uninstalled: %PM_USER_ABS%
    echo Stopping the server instead.
    call "%~f0" stop >nul 2>&1
    exit /b 1
)
if exist "%PM_DIR%\.git" (
    echo Refusing to delete a git checkout: %PM_DIR%
    exit /b 1
)
echo This stops the server, removes the Windows startup entry and deletes:
echo   %PM_ABS%
echo.
if /i not "%~2"=="--yes" (
    set /p "CONFIRM=Continue? [y/N] "
    if /i not "!CONFIRM!"=="y" (
        echo Cancelled.
        exit /b 1
    )
)
call "%~f0" stop >nul 2>&1
del "%APPDATA%\Microsoft\Windows\Start Menu\Programs\Startup\PencariMovie.vbs" >nul 2>&1
powershell -NoProfile -Command "$p=[Environment]::GetEnvironmentVariable('Path','User'); if($p){$n=(@($p -split ';') | Where-Object { $_ -ne '' -and $_ -ne '%PM_ABS%' }) -join ';'; [Environment]::SetEnvironmentVariable('Path', $n, 'User')}" >nul 2>&1
cd /d "%TEMP%"
echo Uninstalled. %PM_ABS% is gone.
(goto) 2>nul & rd /s /q "%PM_ABS%"
