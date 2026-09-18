# Writes storage/auth.json for the dashboard password.
# Called by pencarimovie-windows.bat (pms password / reset-password / token rotate).
# Uses the bundled PHP so the hash matches password_verify() on the server.
param(
    [Parameter(Mandatory = $true)][string]$AppDir,
    [string]$Password = ''
)

$ErrorActionPreference = 'Stop'

$php = Join-Path $AppDir 'bin\php.exe'
if (-not (Test-Path -LiteralPath $php)) { $php = 'php' }

$authJson = Join-Path $AppDir 'storage\auth.json'
$tmpPhp = Join-Path $env:TEMP 'pm_auth_write.php'

$phpSource = @'
<?php
$f = $argv[1];
$pw = $argv[2];
$d = is_file($f) ? (json_decode((string)@file_get_contents($f), true) ?: []) : [];
if ($pw !== '') {
    $d['password_hash'] = password_hash($pw, PASSWORD_DEFAULT);
}
$d['token'] = bin2hex(random_bytes(16));
$d['enabled'] = true;
@mkdir(dirname($f), 0777, true);
file_put_contents($f, json_encode($d, JSON_UNESCAPED_SLASHES), LOCK_EX);
echo $d['token'], PHP_EOL;
'@

Set-Content -LiteralPath $tmpPhp -Value $phpSource -Encoding ASCII

try {
    # Always pass a string (never $null) so PHP does not warn on the rotate path.
    & $php $tmpPhp $authJson ([string]$Password)
}
finally {
    Remove-Item -LiteralPath $tmpPhp -Force -ErrorAction SilentlyContinue
}
