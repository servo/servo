# Sign binaries with self-signed certificate and prepare installable package
param(
    [string]$DistDir = "dist/servo-windows-browser"
)

$ErrorActionPreference = "Stop"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " Code-Signing and Packaging Servo Browser " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

if (-not (Test-Path $DistDir)) {
    throw "Distribution directory not found: $DistDir"
}

# 1. Generate Self-Signed Code-Signing Certificate
Write-Host "[1/3] Generating Self-Signed Code-Signing Certificate..." -ForegroundColor Yellow
$Cert = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject "CN=Servo Browser, O=Servo Project" `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -NotAfter (Get-Date).AddYears(5)

$CertPath = Join-Path $DistDir "ServoBrowser.cer"
Export-Certificate -Cert $Cert -FilePath $CertPath -Force | Out-Null
Write-Host "      Exported public certificate to $CertPath" -ForegroundColor Green

# 2. Sign Binaries (Authenticode)
Write-Host "[2/3] Signing application executables and libraries..." -ForegroundColor Yellow
$Binaries = @(
    (Join-Path $DistDir "servo_browser.exe"),
    (Join-Path $DistDir "servo_capi.dll")
)

foreach ($Bin in $Binaries) {
    if (Test-Path $Bin) {
        $sig = Set-AuthenticodeSignature -FilePath $Bin -Certificate $Cert -HashAlgorithm SHA256
        Write-Host "      Signed: $Bin (Status: $($sig.Status))" -ForegroundColor Green
    }
}

# 3. Copy Installer Scripts
Write-Host "[3/3] Including installer scripts and generating zip package..." -ForegroundColor Yellow
$PackagingDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Copy-Item (Join-Path $PackagingDir "install.ps1") -Destination $DistDir/ -Force
Copy-Item (Join-Path $PackagingDir "uninstall.ps1") -Destination $DistDir/ -Force

# Create Installable Zip Bundle
$ZipPath = "dist/servo-windows-browser-setup.zip"
$ZipDir = Split-Path -Parent $ZipPath
if (-not (Test-Path $ZipDir)) {
    New-Item -ItemType Directory -Force -Path $ZipDir | Out-Null
}
if (Test-Path $ZipPath) {
    Remove-Item $ZipPath -Force
}
Compress-Archive -Path "$DistDir/*" -DestinationPath $ZipPath -CompressionLevel Optimal

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host " Self-signed, installable bundle ready:   " -ForegroundColor Green
Write-Host " $ZipPath" -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Cyan
