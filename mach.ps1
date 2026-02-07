$workdir = $PSScriptRoot
$arguments = $args
$expanded = $false
if ($arguments.Count -gt 0) {
    $expandedArgs = @()
    foreach ($arg in $arguments) {
        if ($arg -match '[\*\?\[\]]') {
            $expandedItems = Get-Item -Path $arg -ErrorAction SilentlyContinue
            if ($expandedItems) {
                $expandedArgs += $expandedItems | Select-Object -ExpandProperty FullName
                $expanded = $true
            } else {
                # Expansion fails. Use original arg.
                $expandedArgs += $arg
            }
        } else {
            $expandedArgs += $arg
        }
    }
    $arguments = $expandedArgs
    if ($expanded){
        Write-Host "Expanded arguments: $arguments"
    }
}

# UV defaults to x86_64 Python on Arm64, so we need to override that.
# https://github.com/astral-sh/uv/issues/12906
if ([System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture -eq 'Arm64') {
    $env:UV_PYTHON='arm64'
}

uv run --frozen python (Join-Path $workdir "mach") @arguments