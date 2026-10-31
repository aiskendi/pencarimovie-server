# PencariMovie Server - compatibility shim. NOT the tray any more.
#
# The tray (icon, menu, actions, run-at-login) is compiled into
# `pencarimovie-server.exe` and runs inside the server process itself:
#
#     pencarimovie-server.exe --tray --port 8088
#
# This file exists for one reason: an HKCU Run entry written by an OLDER version
# points at `powershell -File tray.ps1 -StartServer`. Those entries must keep
# working after an in-place update, so this forwards to the binary and exits.
#
# Why the tray moved into the exe: Windows stores the per-app tray settings in
# `HKCU\Control Panel\NotifyIconSettings` keyed by the HOSTING executable, so an
# icon created by `powershell.exe` shared that executable's single entry with
# every other PowerShell-hosted tray (9Router on the machine that reported this)
# and could only ever live in the "hidden icons" flyout.
#
# To migrate an old login entry, untick/tick "Run at login" in the tray menu (or
# run autostart-on.bat): both now write the binary's own command line.

param(
    [switch]$StartServer,
    [int]$Port = 0,
    [string]$OpenUrl = '',
    [string]$StopBat = ''
)

$ErrorActionPreference = 'Stop'
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
$exe = Join-Path $here 'pencarimovie-server.exe'

if (-not (Test-Path $exe)) {
    # No WinForms here on purpose: this shim must not depend on the very thing
    # that made the old tray unreliable.
    Write-Error "pencarimovie-server.exe was not found next to tray.ps1."
    exit 1
}
if (-not $Port) {
    if ($env:PORT) { $Port = [int]$env:PORT } else { $Port = 8088 }
}

# Bind all interfaces unless the user chose one: the tray's "copy add-on URL" and
# the installer both hand out a LAN address a TV has to reach, so a loopback-only
# listener would make that address dead.
if (-not $env:PM_LISTEN_HOST -and -not $env:HOST) { $env:PM_LISTEN_HOST = '0.0.0.0' }

Start-Process -FilePath $exe -ArgumentList @('--tray', '--port', "$Port") `
    -WorkingDirectory $here -WindowStyle Hidden
