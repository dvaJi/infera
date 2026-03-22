#Requires -Version 5.1
param(
    [string]$Version = "",
    [string]$InstallDir = "",
    [switch]$Help
)

$ErrorActionPreference = "Stop"
$Repo = "dvaji/infera"
$BinaryName = "infs.exe"

if ($Help) {
    Write-Host "Usage: iex \"& { `$(irm https://raw.githubusercontent.com/dvaji/infera/main/install.ps1) }\""
    Write-Host "   or: iex \"& { `$(irm https://raw.githubusercontent.com/dvaji/infera/main/install.ps1) } -Version v1.0.0\""
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  -Version    - Version to install (default: latest)"
    Write-Host "  -InstallDir - Directory to install infs (default: `$env:USERPROFILE\.local\bin)"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  iex \"& { `$(irm https://raw.githubusercontent.com/dvaji/infera/main/install.ps1) }\""
    Write-Host "  iex \"& { `$(irm https://raw.githubusercontent.com/dvaji/infera/main/install.ps1) } -InstallDir 'C:\Tools'\""
    exit 0
}

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-Err {
    param([string]$Message)
    Write-Host "[ERROR] " -ForegroundColor Red -NoNewline
    Write-Host $Message
    exit 1
}

function Get-LatestVersion {
    $releaseUrl = "https://api.github.com/repos/$Repo/releases/latest"
    try {
        $response = Invoke-RestMethod -Uri $releaseUrl -UseBasicParsing
        return $response.tag_name
    } catch {
        Write-Err "Failed to fetch latest version: $_"
    }
}

function Get-Platform {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
    $archString = switch ($arch) {
        "X64" { "x86_64" }
        "Arm64" { Write-Err "Windows ARM64 is not supported. Only x86_64 builds are available." }
        default { Write-Err "Unsupported architecture: $arch" }
    }
    return "windows-$archString"
}

if (-not $InstallDir) {
    $InstallDir = Join-Path $env:USERPROFILE ".local\bin"
}

if (-not $Version) {
    $Version = Get-LatestVersion
}

$Platform = Get-Platform
$AssetName = "infs-$Platform.exe"
$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$AssetName"

Write-Info "Installing infs $Version for $Platform..."

$installPath = Join-Path $InstallDir $BinaryName

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    Write-Info "Created directory: $InstallDir"
}

Write-Info "Downloading from $DownloadUrl..."

try {
    $tempFile = Join-Path $env:TEMP "infs-download.exe"
    Invoke-WebRequest -Uri $DownloadUrl -OutFile $tempFile -UseBasicParsing
    
    if (Test-Path $installPath) {
        $oldPath = "$installPath.old"
        if (Test-Path $oldPath) {
            Remove-Item $oldPath -Force
        }
        Move-Item $installPath $oldPath -Force
    }
    
    Move-Item $tempFile $installPath -Force
} catch {
    Write-Err "Failed to download or install infs: $_"
}

Write-Info "Installed infs to $installPath"

$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($null -eq $userPath -or $userPath -notlike "*$InstallDir*") {
    Write-Info "Adding $InstallDir to your PATH..."
    $currentPath = if ($null -eq $userPath -or $userPath -eq "") { "" } else { $userPath }
    $newPath = if ($currentPath -eq "" -or $currentPath.EndsWith(";")) { $currentPath + $InstallDir } else { $currentPath + ";" + $InstallDir }
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    $env:PATH = if ($env:PATH.EndsWith(";")) { $env:PATH + $InstallDir } else { $env:PATH + ";" + $InstallDir }
    Write-Info "Added to PATH. You may need to restart your terminal for the change to take effect."
}

& $installPath --version
Write-Info "Installation complete!"
