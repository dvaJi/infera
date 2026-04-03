$ErrorActionPreference = 'Stop'

$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$binaryPath = Join-Path $toolsDir 'infs.exe'

if (-not (Test-Path $binaryPath)) {
    throw "Expected packaged binary at $binaryPath."
}

Install-BinFile -Name 'infs' -Path $binaryPath
