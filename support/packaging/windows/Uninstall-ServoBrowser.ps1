<#
.SYNOPSIS
    Uninstalls Servo Browser for Windows, cleaning up shortcuts and registry keys.
#>
[CmdletBinding()]
param(
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\ServoBrowser"
)

$ErrorActionPreference = "SilentlyContinue"

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "  Uninstalling Servo Browser" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

# Remove shortcuts
$DesktopPath = [Environment]::GetFolderPath([Environment+SpecialFolder]::Desktop)
$DesktopShortcut = Join-Path $DesktopPath "Servo Browser.lnk"
if (Test-Path $DesktopShortcut) {
    Remove-Item $DesktopShortcut -Force
    Write-Host "[+] Removed Desktop shortcut" -ForegroundColor Green
}

$StartMenuPath = [Environment]::GetFolderPath([Environment+SpecialFolder]::Programs)
$StartMenuFolder = Join-Path $StartMenuPath "Servo Browser"
if (Test-Path $StartMenuFolder) {
    Remove-Item $StartMenuFolder -Recurse -Force
    Write-Host "[+] Removed Start Menu folder" -ForegroundColor Green
}

# Remove Registry entries
Remove-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\App Paths\servobrowser.exe" -Recurse -Force
Remove-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ServoBrowser" -Recurse -Force
Remove-Item -Path "HKCU:\Software\Clients\StartMenuInternet\ServoBrowser" -Recurse -Force
Remove-ItemProperty -Path "HKCU:\Software\RegisteredApplications" -Name "ServoBrowser" -Force
Write-Host "[+] Removed Registry registrations" -ForegroundColor Green

# Remove install directory
if (Test-Path $InstallDir) {
    # If uninstaller was run from inside $InstallDir, schedule cleanup
    Start-Process -FilePath "cmd.exe" -ArgumentList "/c timeout /t 1 /nobreak & rmdir /s /q `"$InstallDir`"" -WindowStyle Hidden
    Write-Host "[+] Removed installation directory" -ForegroundColor Green
}

Write-Host "Servo Browser uninstallation complete." -ForegroundColor Green
