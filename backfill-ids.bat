@echo off
REM ─────────────────────────────────────────────────────────────────────────────
REM Local-only catalog ID backfill runner (Windows).
REM
REM Walks the catalog in batches and populates external IDs so future users can
REM resolve tmdb:/kitsu:/mal:/anilist:/tvdb: WITHOUT upstream Stremio addons.
REM
REM Usage:
REM   backfill-ids.bat              run continuously until the catalog is done
REM   backfill-ids.bat 50           batch size (default 25)
REM   backfill-ids.bat 50 500       batch size + sleep ms between scrapes
REM
REM Press Ctrl+C to stop. Progress is saved, so re-running resumes.
REM ─────────────────────────────────────────────────────────────────────────────

setlocal

set BATCH=%1
if "%BATCH%"=="" set BATCH=25

set SLEEP=%2
if "%SLEEP%"=="" set SLEEP=800

echo ============================================================
echo  PencariMovie catalog ID backfill
echo  Batch size : %BATCH%
echo  Sleep      : %SLEEP% ms
echo  Press Ctrl+C to stop (progress is saved).
echo ============================================================
echo.

:loop
php "%~dp0backfill-ids.php" --limit=%BATCH% --sleep=%SLEEP%
if errorlevel 1 (
    echo.
    echo Backfill stopped with an error.
    goto :end
)

REM Stop when the script reports no more posts.
php "%~dp0backfill-ids.php" --status | findstr /C:"\"processed\"" >nul
echo.
echo --- batch complete, continuing ---
echo.

REM Detect completion: if the last run processed 0 posts, stop.
for /f "tokens=*" %%L in ('php "%~dp0backfill-ids.php" --status') do set STATUS=%%L
echo %STATUS% | findstr /C:"\"processed\": 0" >nul
if not errorlevel 1 (
    echo Catalog backfill complete.
    goto :end
)

goto :loop

:end
echo.
echo Final status:
php "%~dp0backfill-ids.php" --status
endlocal
