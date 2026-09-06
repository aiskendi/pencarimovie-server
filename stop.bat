@echo off
setlocal
set HOST=0.0.0.0
set PORT=8088
for %%I in ("%~dp0.") do set "ROOT=%%~fI"

echo Stopping PencariMovie Server on %HOST%:%PORT%...
echo Stopping Addon service on 0.0.0.0:8089...

rem Stop Addon on port 8089 if running
for /f "tokens=5" %%P in ('netstat -ano 2^>nul ^| findstr "0.0.0.0:8089 127.0.0.1:8089 [::]:8089" ^| findstr "LISTENING"') do (
  echo Killing addon process PID %%P
  taskkill /PID %%P /F >nul 2>nul
)
powershell -NoProfile -Command "Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object { $_.Name -like '*addon*' -or ($_.CommandLine -and ($_.CommandLine -like '*addon.js*' -or $_.CommandLine -like '*addon.exe*')) } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }" >nul 2>nul

for /f "tokens=5" %%P in ('netstat -ano 2^>nul ^| findstr "0.0.0.0:%PORT% 127.0.0.1:%PORT% [::]:%PORT%" ^| findstr "LISTENING"') do (
  echo Killing process PID %%P
  taskkill /PID %%P /F >nul 2>nul
)

call :stop_tray
call :stop_tunnel

echo Stop command completed.
endlocal
goto :eof

:stop_tray
powershell -NoProfile -Command "try { $e = New-Object System.Threading.EventWaitHandle $false, ([System.Threading.EventResetMode]::AutoReset), 'Global\PencariMovieServerTrayStop'; $e.Set() | Out-Null; $e.Dispose() } catch {}" >nul 2>nul
timeout /t 1 /nobreak >nul
if exist "%ROOT%\storage\tray.pid" (
  for /f "usebackq delims=" %%P in ("%ROOT%\storage\tray.pid") do (
    if not "%%P"=="" taskkill /PID %%P /F >nul 2>nul
  )
  del /q "%ROOT%\storage\tray.pid" >nul 2>nul
)
powershell -NoProfile -Command "Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object { $_.CommandLine -and $_.CommandLine -match 'pencarimovie.+tray\.ps1' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }" >nul 2>nul
goto :eof

:stop_tunnel
if exist "%ROOT%\storage\tunnel\cloudflared.pid" (
  for /f "usebackq delims=" %%P in ("%ROOT%\storage\tunnel\cloudflared.pid") do (
    if not "%%P"=="" (
      echo Stopping Cloudflare tunnel PID %%P
      taskkill /PID %%P /T /F >nul 2>nul
    )
  )
  del /q "%ROOT%\storage\tunnel\cloudflared.pid" >nul 2>nul
)
powershell -NoProfile -Command "Get-CimInstance Win32_Process -ErrorAction SilentlyContinue | Where-Object { $_.Name -match 'cloudflared' -and $_.CommandLine -and $_.CommandLine -match 'storage[\\/]tunnel' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }" >nul 2>nul
if exist "%ROOT%\storage\tunnel\state.json" del /q "%ROOT%\storage\tunnel\state.json" >nul 2>nul
goto :eof
