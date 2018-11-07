# Use this script is to get a build environment
# when booting a Windows EC2 instance outside of Taskcluster.


[Environment]::SetEnvironmentVariable("Path", $env:Path +
    ";C:\git\cmd;C:\python2;C:\python2\Scripts;C:\Users\Administrator\.cargo\bin",
    [EnvironmentVariableTarget]::Machine)
[Environment]::SetEnvironmentVariable("Lib", $env:Lib +
    ";C:\gstreamer\1.0\x86_64\lib",
    [EnvironmentVariableTarget]::Machine)


# use TLS 1.2 (see bug 1443595)
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# For making http requests
$client = New-Object system.net.WebClient
$shell = new-object -com shell.application

# Download a zip file and extract it
function Expand-ZIPFile($file, $destination, $url)
{
    $client.DownloadFile($url, $file)
    $zip = $shell.NameSpace($file)
    foreach($item in $zip.items())
    {
        $shell.Namespace($destination).copyhere($item)
    }
}

# Optional
$client.DownloadFile(
    "https://download.tuxfamily.org/dvorak/windows/bepo.exe",
    "C:\bepo.exe"
)

md C:\git
Expand-ZIPFile -File "C:\git.zip" -Destination "C:\git" -Url `
    "https://github.com/git-for-windows/git/releases/download/v2.19.0.windows.1/MinGit-2.19.0-64-bit.zip"

$client.DownloadFile(
    "https://static.rust-lang.org/rustup/archive/1.13.0/i686-pc-windows-gnu/rustup-init.exe",
    "C:\rustup-init.exe"
)

Start-Process C:\rustup-init.exe -Wait -NoNewWindow -ArgumentList `
    "--default-toolchain none -y"

md C:\python2
Expand-ZIPFile -File "C:\python2.zip" -Destination "C:\python2" -Url `
    "https://queue.taskcluster.net/v1/task/RIuts6jOQtCSjMbuaOU6yw/runs/0/artifacts/public/repacked.zip"

Expand-ZIPFile -File "C:\gst.zip" -Destination "C:\" -Url `
    "https://queue.taskcluster.net/v1/task/KAzPF1ZYSFmg2BQKLt0LwA/runs/0/artifacts/public/repacked.zip"