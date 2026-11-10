# Reports whether the WebView2 Runtime is installed and, only when it is not, installs it
# with Microsoft's Evergreen Bootstrapper. The log line is the evidence: it says which of the
# two happened on this runner. The detection and install commands are the ones documented at
# https://learn.microsoft.com/microsoft-edge/webview2/concepts/distribution
$ErrorActionPreference = 'Stop'

function Get-WebView2Version {
    $client = 'Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    foreach ($key in "HKLM:\SOFTWARE\WOW6432Node\$client", "HKLM:\SOFTWARE\$client", "HKCU:\Software\$client") {
        $version = (Get-ItemProperty -Path $key -Name pv -ErrorAction SilentlyContinue).pv
        if ($version -and $version -ne '0.0.0.0') { return $version }
    }
    return $null
}

$before = Get-WebView2Version
if ($before) {
    Write-Output "WebView2 Runtime present before provisioning: $before"
    exit 0
}
Write-Output 'WebView2 Runtime NOT detected before provisioning. Installing the Evergreen Runtime.'

$installer = Join-Path $env:RUNNER_TEMP 'MicrosoftEdgeWebview2Setup.exe'
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -UseBasicParsing -TimeoutSec 120 -Uri 'https://go.microsoft.com/fwlink/p/?LinkId=2124703' -OutFile $installer

$signature = Get-AuthenticodeSignature -FilePath $installer
if ($signature.Status -ne 'Valid' -or $signature.SignerCertificate.Subject -notmatch 'O=Microsoft Corporation') {
    throw "Refusing the bootstrapper: signature $($signature.Status), signer $($signature.SignerCertificate.Subject)"
}

$setup = Start-Process -FilePath $installer -ArgumentList '/silent', '/install' -PassThru
if (-not $setup.WaitForExit(300000)) {
    Stop-Process -Id $setup.Id -Force
    throw 'The WebView2 bootstrapper did not finish within 5 minutes.'
}
Write-Output "Bootstrapper exit code: $($setup.ExitCode)"

$after = Get-WebView2Version
if (-not $after) { throw 'The WebView2 Runtime is still not detected after provisioning.' }
Write-Output "WebView2 Runtime provisioned: $after"
