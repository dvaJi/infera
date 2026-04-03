$ErrorActionPreference = 'Stop'

$toolsDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$binaryPath = Join-Path $toolsDir 'infs.exe'

Uninstall-BinFile -Name 'infs' -Path $binaryPath
