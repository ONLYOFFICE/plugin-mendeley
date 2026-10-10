$ErrorActionPreference = 'Stop'
trap {
    [Console]::Error.WriteLine((ConvertTo-Json -Compress @{ timestamp = [DateTime]::UtcNow.ToString('o'); level = 'error'; event = 'windows_installer.failed'; data = @{ message = $_.Exception.Message } }))
    break
}
$root = Split-Path -Parent $PSScriptRoot
. (Join-Path $PSScriptRoot 'windows-installer-health.ps1')
$installer = Get-ChildItem (Join-Path $root 'dist/installers/Mendeley-ONLYOFFICE-Setup-*.exe') | Select-Object -First 1
$helper = Join-Path $root 'dist/windows/mendeley-loopback-server.exe'
$appDir = Join-Path $env:LOCALAPPDATA 'Programs/MendeleyOnlyOffice'
$uninstaller = Join-Path $appDir 'unins000.exe'
$pythonHelper = Join-Path $root 'scripts/mendeley-loopback-server.py'

if (-not $installer) { throw 'Windows installer was not produced' }
if (-not (Test-Path $helper -PathType Leaf)) { throw 'Rust helper executable was not staged' }
if (-not (Test-Path $pythonHelper -PathType Leaf)) { throw 'Existing Python helper source is missing' }

$connection = [Net.Sockets.TcpClient]::new()
try {
    $connection.Connect('127.0.0.1', 8080)
    throw 'Port 8080 is already occupied; refusing to stop an unrelated service'
} catch [Net.Sockets.SocketException] {
} finally {
    $connection.Dispose()
}
Write-Output (ConvertTo-Json -Compress @{ timestamp = [DateTime]::UtcNow.ToString('o'); level = 'debug'; event = 'windows_installer.port_available'; data = @{ port = 8080 } })

$install = Start-Process $installer.FullName -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait -PassThru
if ($install.ExitCode -ne 0) { throw "Installer failed with exit code $($install.ExitCode)" }
Write-Output (ConvertTo-Json -Compress @{ timestamp = [DateTime]::UtcNow.ToString('o'); level = 'info'; event = 'windows_installer.installed'; data = @{ exitCode = $install.ExitCode } })
try {
    $installedHelper = Join-Path $appDir 'bin/mendeley-loopback-server.exe'
    if (-not (Test-Path $installedHelper -PathType Leaf)) { throw 'Installer did not install Rust helper' }
    if (-not (Wait-ForHelperState $true)) { throw 'Installer did not start a healthy Rust helper' }
    Write-Output (ConvertTo-Json -Compress @{ timestamp = [DateTime]::UtcNow.ToString('o'); level = 'success'; event = 'windows_installer.helper_healthy'; data = @{ service = 'mendeley-loopback' } })
} finally {
    if (Test-Path $uninstaller -PathType Leaf) {
        $uninstall = Start-Process $uninstaller -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART' -Wait -PassThru
        if ($uninstall.ExitCode -ne 0) { throw "Uninstaller failed with exit code $($uninstall.ExitCode)" }
    }
}
Write-Output (ConvertTo-Json -Compress @{ timestamp = [DateTime]::UtcNow.ToString('o'); level = 'info'; event = 'windows_installer.uninstalled'; data = @{} })

if (-not (Wait-ForHelperState $false)) { throw 'Uninstaller did not stop the Rust helper' }
if (Test-Path $uninstaller) { throw 'Uninstaller did not remove its installation' }
Write-Output (ConvertTo-Json -Compress @{ timestamp = [DateTime]::UtcNow.ToString('o'); level = 'success'; event = 'windows_installer.helper_stopped'; data = @{} })
