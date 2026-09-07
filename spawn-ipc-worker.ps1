param(
    [Parameter(Mandatory = $true)]
    [string]$PhpBin,
    [Parameter(Mandatory = $true)]
    [string]$Entry,
    [Parameter(Mandatory = $true)]
    [string]$SessionDir,
    [Parameter(Mandatory = $true)]
    [string]$StartupId
)

# Spawn a fully detached MadelineProto IPC worker that survives the calling
# web request. Start-Process returns immediately; do NOT use -PassThru or
# -RedirectStandardOutput/Error (those keep PowerShell alive until the child
# exits, which blocks the caller).
$argList = @(
    '-dhtml_errors=0',
    '-ddisplay_errors=0',
    '-dlog_errors=1',
    $Entry,
    'madeline-ipc',
    $SessionDir,
    $StartupId
)

Start-Process -FilePath $PhpBin -ArgumentList $argList -WindowStyle Hidden
