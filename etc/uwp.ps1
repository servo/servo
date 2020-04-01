Write-Output "PAUL: CURRENT LOCATION -----"
Get-Location
Write-Output "PAUL: CERTS LIST FOR Allizom -----"
dir cert: -Recurse | Where-Object {$_.Issuer -eq "CN=Allizom"}
Write-Output "PAUL: LIST OF LOCALHOST-ALLOWED APPS BEFORE INSTALLATION -----"
CheckNetIsolation LoopbackExempt -s

Write-Output "PAUL: UNINSTALL PKG -----"
$(Get-AppxPackage MozillaFoundation.FirefoxReality)| Remove-AppxPackage

$url = "https://community-tc.services.mozilla.com/api/queue/v1/task/a5TVjKpZTk-Df-uZcQFhxw/runs/0/artifacts/public/ServoApp_1.0.0.0_Debug_Test.zip"

Write-Output "PAUL: DOWNLOADING AND UNZIPPING PKG -----"
Invoke-WebRequest -Uri $url -OutFile tc.zip
Expand-Archive tc.zip
Set-Location -Path tc\servo\

Write-Output "PAUL: READING PKG SIGNATURE -----"
Get-AuthenticodeSignature -FilePath ServoApp_1.0.0.0_x64_Debug.msixbundle | Select-Object *

Write-Output "PAUL: ADD APPX x2 -----"
Add-AppxPackage -Path Dependencies\x64\Microsoft.VCLibs.x64.Debug.14.00.appx
Add-AppxPackage -Path ServoApp_1.0.0.0_x64_Debug.msixbundle

$fam = Get-AppxPackage MozillaFoundation.FirefoxReality | select -expandproperty PackageFamilyName

Write-Output "PAUL: Pacakge installed" $fam

Write-Output "PAUL: LIST OF LOCALHOST-ALLOWED APPS BEFORE INSTALLATION -----"
CheckNetIsolation LoopbackExempt -a -n="$fam"
CheckNetIsolation LoopbackExempt -s

# $app_dir = Get-AppxPackage MozillaFoundation.FirefoxReality | select -expandproperty InstallLocation
# $dumpbin = 'C:\Program Files (x86)\Microsoft Visual Studio\2017\Community\VC\Tools\MSVC\14.16.27023\bin\Hostx64\x64\dumpbin.exe'
# # $dumpbin = 'C:\Program Files (x86)\Microsoft Visual Studio\2017\BuildTools\VC\Tools\MSVC\14.16.27023\bin\Hostx64\x64\dumpbin.exe'
# & $dumpbin /DEPENDENTS $app_dir\ServoApp.exe

Write-Output "PAUL: START PROCESS -----"
Start-Process -ArgumentList "http://localhost:56012" shell:AppsFolder\$fam!App
# start "fxr://http://example.com"

Write-Output "PAUL: SLEEP -----"
Start-Sleep -seconds 5

Write-Output "PAUL: System logs -----"
# Get-EventLog System
Get-WinEvent -FilterHashtable @{LogName="System"; StartTime=(get-date).AddHours(-3)}
# Write-Output "PAUL: Application logs -----"
# Get-EventLog Application


Write-Output "PAUL: FIND PROCESS -----"
Get-Process ServoApp  | Format-List *


Write-Output "PAUL: STOP & UNINSTALL PKG -----"
$(Get-AppxPackage MozillaFoundation.FirefoxReality)| Remove-AppxPackage

Set-Location -Path ..\..
