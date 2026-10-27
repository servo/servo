# Native Windows Servo Browser - Automated Installer
param(
    [string]$TargetDir = "$env:LOCALAPPDATA\Programs\Servo Browser"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " Installing Servo Browser (Native Windows) " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path

# 1. Trust Self-Signed Certificate
$CertPath = Join-Path $ScriptDir "ServoBrowser.cer"
if (Test-Path $CertPath) {
    Write-Host "[1/4] Installing Self-Signed Certificate into Trusted Root..." -ForegroundColor Yellow
    try {
        Import-Certificate -FilePath $CertPath -CertStoreLocation "Cert:\CurrentUser\Root" | Out-Null
        Write-Host "      Certificate installed successfully." -ForegroundColor Green
    } catch {
        Write-Warning "Failed to install certificate automatically. You can install it manually from $CertPath"
    }
}

# 2. Copy Application Files
Write-Host "[2/4] Deploying application binaries to $TargetDir..." -ForegroundColor Yellow
if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir -Force | Out-Null
}

$Exclude = @("install.ps1", "Servo-Browser-Setup.zip")
Get-ChildItem -Path $ScriptDir | Where-Object { $_.Name -notin $Exclude } | ForEach-Object {
    Copy-Item -Path $_.FullName -Destination $TargetDir -Recurse -Force
}
Write-Host "      Binaries deployed." -ForegroundColor Green

# 3. Create Shortcuts
Write-Host "[3/4] Creating Start Menu and Desktop shortcuts..." -ForegroundColor Yellow
$ExePath = Join-Path $TargetDir "servo_browser.exe"
$WshShell = New-Object -ComObject WScript.Shell

# Desktop shortcut
$DesktopPath = [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::Desktop)
$DesktopShortcut = $WshShell.CreateShortcut((Join-Path $DesktopPath "Servo Browser.lnk"))
$DesktopShortcut.TargetPath = $ExePath
$DesktopShortcut.WorkingDirectory = $TargetDir
$DesktopShortcut.Description = "Servo Modern Web Browser"
$DesktopShortcut.Save()

# Start Menu shortcut
$StartMenuPath = [System.Environment]::GetFolderPath([System.Environment+SpecialFolder]::Programs)
$StartShortcut = $WshShell.CreateShortcut((Join-Path $StartMenuPath "Servo Browser.lnk"))
$StartShortcut.TargetPath = $ExePath
$StartShortcut.WorkingDirectory = $TargetDir
$StartShortcut.Description = "Servo Modern Web Browser"
$StartShortcut.Save()
Write-Host "      Shortcuts created." -ForegroundColor Green

# 4. Register in Windows Add/Remove Programs
Write-Host "[4/4] Registering in Windows Add/Remove Programs..." -ForegroundColor Yellow
$UninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\ServoBrowser"
if (-not (Test-Path $UninstallKey)) {
    New-Item -Path $UninstallKey -Force | Out-Null
}

Set-ItemProperty -Path $UninstallKey -Name "DisplayName" -Value "Servo Browser"
Set-ItemProperty -Path $UninstallKey -Name "DisplayVersion" -Value "1.0.0"
Set-ItemProperty -Path $UninstallKey -Name "Publisher" -Value "Servo Project"
Set-ItemProperty -Path $UninstallKey -Name "DisplayIcon" -Value "$ExePath,0"
Set-ItemProperty -Path $UninstallKey -Name "InstallLocation" -Value $TargetDir
Set-ItemProperty -Path $UninstallKey -Name "UninstallString" -Value "powershell.exe -ExecutionPolicy Bypass -File `"$TargetDir\uninstall.ps1`""
Write-Host "      Registered successfully." -ForegroundColor Green

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " Servo Browser installation complete!     " -ForegroundColor Green
Write-Host " Launching Servo Browser...               " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

Start-Process -FilePath $ExePath -WorkingDirectory $TargetDir
