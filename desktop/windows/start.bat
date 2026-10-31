@echo off
rem PencariMovie Server - console launcher (Windows).
rem
rem Use this when you want to SEE the log. For tray-only operation use
rem start-tray.bat, which runs the server hidden behind a tray icon.
setlocal
cd /d "%~dp0"
if "%PORT%"=="" set PORT=8088
rem Same default as start-tray.bat: a TV on this wifi has to reach the port.
rem An explicit --host in %* still wins.
if not defined PM_LISTEN_HOST if not defined HOST set "PM_LISTEN_HOST=0.0.0.0"
set "EXE=pencarimovie-server.exe"
if not exist "%EXE%" (
  echo pencarimovie-server.exe not found next to this file.
  pause
  exit /b 1
)
echo.
echo   PencariMovie Server
echo   dashboard : http://127.0.0.1:%PORT%/
echo   add-on    : http://127.0.0.1:%PORT%/manifest.json
echo.
echo   Close this window or press Ctrl+C to stop the server.
echo.
"%EXE%" --port %PORT% %*
