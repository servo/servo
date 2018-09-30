Start-Transcript -Path "C:\first_boot.txt"

Get-ChildItem Env: | Out-File "C:\install_env.txt"

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

# Open up firewall for livelog (both PUT and GET interfaces)
New-NetFirewallRule -DisplayName "Allow livelog PUT requests" `
    -Direction Inbound -LocalPort 60022 -Protocol TCP -Action Allow
New-NetFirewallRule -DisplayName "Allow livelog GET requests" `
    -Direction Inbound -LocalPort 60023 -Protocol TCP -Action Allow

# Install generic-worker and dependencies
md C:\generic-worker
$client.DownloadFile("https://github.com/taskcluster/generic-worker/releases/download" +
    "/v10.11.3/generic-worker-windows-amd64.exe", "C:\generic-worker\generic-worker.exe")
$client.DownloadFile("https://github.com/taskcluster/livelog/releases/download" +
    "/v1.1.0/livelog-windows-amd64.exe", "C:\generic-worker\livelog.exe")
Expand-ZIPFile -File "C:\nssm-2.24.zip" -Destination "C:\" `
    -Url "http://www.nssm.cc/release/nssm-2.24.zip"
Start-Process C:\generic-worker\generic-worker.exe -ArgumentList `
    "new-openpgp-keypair --file C:\generic-worker\generic-worker-gpg-signing-key.key" `
    -Wait -NoNewWindow -PassThru `
    -RedirectStandardOutput C:\generic-worker\generate-signing-key.log `
    -RedirectStandardError C:\generic-worker\generate-signing-key.err
Start-Process C:\generic-worker\generic-worker.exe -ArgumentList (
        "install service --nssm C:\nssm-2.24\win64\nssm.exe " +
        "--config C:\generic-worker\generic-worker.config"
    ) -Wait -NoNewWindow -PassThru `
    -RedirectStandardOutput C:\generic-worker\install.log `
    -RedirectStandardError C:\generic-worker\install.err

# Now shutdown, in preparation for creating an image
shutdown -s