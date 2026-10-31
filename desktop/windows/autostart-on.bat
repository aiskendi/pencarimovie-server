@echo off
rem Start PencariMovie Server (tray + server) when this user logs in.
rem Registers a per-user HKCU Run entry - no administrator rights needed.
rem The entry points at the BINARY with --tray; the tray is part of the exe, so
rem there is no script to keep in place for the login start.
setlocal
cd /d "%~dp0"
if "%PORT%"=="" set PORT=8088
set "RUNKEY=HKCU\Software\Microsoft\Windows\CurrentVersion\Run"
set "NAME=PencariMovie Server"
rem The login entry is the one launch path with no environment to inherit, so the
rem bind address has to be written into the command itself. Without it the server
rem came back on loopback after every reboot and the TV's add-on URL went dead.
set "HOSTARG=--host 0.0.0.0"
if defined PM_LISTEN_HOST set "HOSTARG=--host %PM_LISTEN_HOST%"
if not defined PM_LISTEN_HOST if defined HOST set "HOSTARG=--host %HOST%"
set "CMD="%~dp0pencarimovie-server.exe" --tray --port %PORT% %HOSTARG%"
reg add "%RUNKEY%" /v "%NAME%" /t REG_SZ /d "%CMD%" /f >nul
if errorlevel 1 (
  echo Could not register the login entry.
  exit /b 1
)
echo Registered: "%NAME%" will start at login.
echo   command: %CMD%
