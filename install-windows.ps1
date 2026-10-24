[CmdletBinding()]
param(
    [string]$BinaryPath = (Join-Path $PSScriptRoot 'target\release\prism-discord-rpc.exe'),
    [switch]$StartAtLogon
)
$ErrorActionPreference = 'Stop'
$rpcSource = (Resolve-Path -LiteralPath $BinaryPath).Path
$rpcInstall = Join-Path $env:LOCALAPPDATA 'Programs\PrismDiscordRPC'
$rpcExe = Join-Path $rpcInstall 'prism-discord-rpc.exe'
New-Item -ItemType Directory -Path $rpcInstall -Force | Out-Null
Get-Process prism-discord-rpc -ErrorAction SilentlyContinue | Where-Object Path -EQ $rpcExe | Stop-Process
if ($rpcSource -ne $rpcExe) { Copy-Item -LiteralPath $rpcSource -Destination $rpcExe -Force }
@'
$ErrorActionPreference = 'Stop'
$rpcExe = Join-Path $PSScriptRoot 'prism-discord-rpc.exe'
if (Get-Process prism-discord-rpc -ErrorAction SilentlyContinue | Where-Object Path -EQ $rpcExe) { exit }
$env:RUST_LOG = 'info'
Start-Process -FilePath $rpcExe -WorkingDirectory $PSScriptRoot -WindowStyle Hidden -RedirectStandardOutput (Join-Path $PSScriptRoot 'stdout.log') -RedirectStandardError (Join-Path $PSScriptRoot 'rpc.log')
'@ | Set-Content -LiteralPath (Join-Path $rpcInstall 'Start.ps1')
if ($StartAtLogon) {
    $rpcShell = New-Object -ComObject WScript.Shell
    $rpcShortcut = $rpcShell.CreateShortcut((Join-Path ([Environment]::GetFolderPath('Startup')) 'Prism Discord RPC.lnk'))
    $rpcShortcut.TargetPath = "$env:SystemRoot\System32\WindowsPowerShell\v1.0\powershell.exe"
    $rpcShortcut.Arguments = '-NoProfile -ExecutionPolicy Bypass -WindowStyle Hidden -File "' + $rpcInstall + '\Start.ps1"'
    $rpcShortcut.WorkingDirectory = $rpcInstall
    $rpcShortcut.WindowStyle = 7
    $rpcShortcut.Description = 'Display the running Prism Launcher Minecraft instance on Discord'
    $rpcShortcut.Save()
}
& (Join-Path $rpcInstall 'Start.ps1')
Write-Output "Installed and started in $rpcInstall. Logs: $rpcInstall\rpc.log"
