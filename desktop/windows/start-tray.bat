@echo off
rem PencariMovie Server - tray launcher (Windows).
rem
rem The tray is INSIDE the binary: `pencarimovie-server.exe --tray` re-launches
rem itself with no console at all (log: storage\server.log) and puts the icon in
rem the notification area. This window therefore closes immediately - no
rem PowerShell, no tray.ps1, no .ico beside the exe, no lingering prompt.
rem Double-click the icon = dashboard; right-click = menu (pages, copy add-on URL,
rem open log, run at login, stop server).
setlocal
cd /d "%~dp0"
if "%PORT%"=="" set PORT=8088
rem Bind all interfaces unless the user already chose one. The tray, `pms` and
rem the add-on URL the installer prints all assume a TV on this wifi can reach
rem the port, and a loopback-only listener is unreachable from every other
rem device. `--host` on the command line still wins over this.
if not defined PM_LISTEN_HOST if not defined HOST set "PM_LISTEN_HOST=0.0.0.0"
if not exist "%~dp0storage" mkdir "%~dp0storage" >nul 2>nul
rem `start` returns at once, so this window closes immediately; the server gets
rem its own console, which `--tray` hides itself. Do NOT call the exe directly
rem and do NOT use /wait: either one would keep this window open (and block the
rem caller) for the whole life of the server.
start "" /min "%~dp0pencarimovie-server.exe" --tray --port %PORT% <nul >nul 2>&1
