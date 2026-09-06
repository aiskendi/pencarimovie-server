# PencariMovie Server - Windows One-Line Installer & Launcher
$ErrorActionPreference = 'Stop'
[Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12

$homeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $HOME }
$installDir = Join-Path $homeDir "pencarimovie-server"

if (-not (Test-Path -LiteralPath $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$batFile = Join-Path $installDir "pencarimovie-windows.bat"
$url = "https://raw.githubusercontent.com/aiskendi/pencarimovie-server/main/pencarimovie-windows.bat"

Write-Host "PencariMovie Server: $installDir" -ForegroundColor Cyan
Invoke-RestMethod -Uri $url -OutFile $batFile

# Run installer inside the fixed user directory
Push-Location $installDir
try {
    & $batFile @args
}
finally {
    Pop-Location
}
