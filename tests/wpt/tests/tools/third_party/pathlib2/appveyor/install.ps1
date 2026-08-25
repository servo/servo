# Sample script to install Python and pip under Windows
# Authors: Olivier Grisel and Kyle Kastner
# License: CC0 1.0 Universal: http://creativecommons.org/publicdomain/zero/1.0/

$GET_PIP_URL = "https://bootstrap.pypa.io/get-pip.py"
$GET_PIP_PATH = "C:\get-pip.py"


function InstallPip ($python_home) {
    $pip_path = $python_home + "/Scripts/pip.exe"
    $python_path = $python_home + "/python.exe"
    if (-not(Test-Path $pip_path)) {
        Write-Host "Installing pip..."
        $webclient = New-Object System.Net.WebClient
        $webclient.DownloadFile($GET_PIP_URL, $GET_PIP_PATH)
        Write-Host "Executing:" $python_path $GET_PIP_PATH
        Start-Process -FilePath "$python_path" -ArgumentList "$GET_PIP_PATH" -Wait -Passthru
    } else {
        Write-Host "Upgrading pip..."
        & $python_path -m pip install --upgrade pip
    }
    Write-Host "Upgrading setuptools..."
    & $python_path -m pip install --upgrade setuptools
}

function InstallPackage ($python_home, $pkg) {
    $pip_path = $python_home + "/Scripts/pip.exe"
    & $pip_path install $pkg
}

function InstallRequirements ($python_home, $reqs) {
    $pip_path = $python_home + "/Scripts/pip.exe"
    & $pip_path install -r $reqs
}

function main () {
    InstallPip $env:PYTHON
    InstallRequirements $env:PYTHON -r requirements.txt
    InstallPackage $env:PYTHON pytest-cov
    InstallPackage $env:PYTHON unittest2
    InstallPackage $env:PYTHON .
}

main
