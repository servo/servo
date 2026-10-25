# Native Windows Servo Browser - Automated Uninstaller
$ErrorActionPreference = "SilentlyContinue"

Write-Host "Uninstalling Servo Browser..." -ForegroundColor Cyan

# 1. Remove Shortcuts
$DesktopPath = [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::Desktop)
$StartMenuPath = [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::Programs)

Remove-Item (Join-Path $DesktopPath "Servo Browser.lnk") -Force
Remove-Item (Join-Path $StartMenuPath "Servo Browser.lnk") -Force

# 2. Remove Registry Entry
Remove-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ServoBrowser" -Recurse -Force

# 3. Schedule Application Directory Removal
$InstallDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Start-Process powershell.exe -ArgumentList "-NoProfile -Command `"Start-Sleep -Seconds 2; Remove-Item -Path '$InstallDir' -Recurse -Force`"" -WindowStyle Hidden

Write-Host "Servo Browser has been uninstalled." -ForegroundColor Green
