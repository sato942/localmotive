# Host-side: download installers, launch Windows Sandbox, wait for result.json
param(
  [Parameter(Mandatory=$true)][string]$Tag,
  [Parameter(Mandatory=$true)][string]$Version,
  [string]$PreviousTag = "v0.4.0",
  [string]$Repo = "sato942/localmotive",
  [int]$TimeoutMinutes = 45
)
$ErrorActionPreference = "Stop"
$Root = Join-Path $env:TEMP ("localmotive-sandbox-" + $Version + "-" + (Get-Date -Format "yyyyMMddHHmmss"))
$Shared = Join-Path $Root "shared"
New-Item -ItemType Directory -Force -Path $Shared | Out-Null
Write-Host "Work dir: $Root"

function Download-Asset([string]$RelTag, [string]$Name, [string]$OutPath) {
  Write-Host "Downloading $RelTag / $Name"
  gh release download $RelTag -R $Repo -p $Name -D (Split-Path $OutPath -Parent) --clobber
  $downloaded = Join-Path (Split-Path $OutPath -Parent) $Name
  if (-not (Test-Path $downloaded)) { throw "Download failed: $Name from $RelTag" }
  if ($downloaded -ne $OutPath) { Move-Item -Force $downloaded $OutPath }
}

# Wait for current release assets (Release workflow may still be publishing)
$deadline = (Get-Date).AddMinutes(30)
$currentSetup = "Localmotive_${Version}_x64-setup.exe"
$currentMsi = "Localmotive_${Version}_x64.msi"
$prevVersion = $PreviousTag.TrimStart("v")
$previousSetup = "Localmotive_${prevVersion}_x64-setup.exe"

while ($true) {
  try {
    $assets = gh release view $Tag -R $Repo --json assets --jq ".assets[].name"
    if (($assets -match [regex]::Escape($currentSetup)) -and ($assets -match [regex]::Escape($currentMsi))) { break }
    Write-Host "Waiting for release assets on $Tag ..."
  } catch {
    Write-Host "Release $Tag not visible yet: $($_.Exception.Message)"
  }
  if ((Get-Date) -gt $deadline) { throw "Timed out waiting for $Tag assets ($currentSetup, $currentMsi)" }
  Start-Sleep -Seconds 30
}

Download-Asset $Tag $currentSetup (Join-Path $Shared $currentSetup)
Download-Asset $Tag $currentMsi (Join-Path $Shared $currentMsi)
Download-Asset $PreviousTag $previousSetup (Join-Path $Shared $previousSetup)

# Copy in-sandbox script from repo checkout if present, else expect it beside this script
$here = $PSScriptRoot
$inScriptSrc = Join-Path $here "run-lifecycle-in-sandbox.ps1"
if (-not (Test-Path $inScriptSrc)) { throw "Missing $inScriptSrc" }
Copy-Item $inScriptSrc (Join-Path $Shared "run-lifecycle.ps1") -Force

$meta = @{
  tag = $Tag
  version = $Version
  previousTag = $PreviousTag
  currentSetup = $currentSetup
  currentMsi = $currentMsi
  previousSetup = $previousSetup
} | ConvertTo-Json
Set-Content -Path (Join-Path $Shared "meta.json") -Value $meta -Encoding UTF8

$wsbPath = Join-Path $Root "localmotive-lifecycle.wsb"
$sharedXml = $Shared
$wsb = @"
<Configuration>
  <VGpu>Disable</VGpu>
  <Networking>Disable</Networking>
  <MappedFolders>
    <MappedFolder>
      <HostFolder>$sharedXml</HostFolder>
      <SandboxFolder>C:\Users\WDAGUtilityAccount\Desktop\shared</SandboxFolder>
      <ReadOnly>false</ReadOnly>
    </MappedFolder>
  </MappedFolders>
  <LogonCommand>
    <Command>powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\Users\WDAGUtilityAccount\Desktop\shared\run-lifecycle.ps1</Command>
  </LogonCommand>
</Configuration>
"@
Set-Content -Path $wsbPath -Value $wsb -Encoding UTF8

$resultPath = Join-Path $Shared "result.json"
if (Test-Path $resultPath) { Remove-Item $resultPath -Force }

Write-Host "Launching Windows Sandbox..."
$sandbox = Start-Process -FilePath "$env:WINDIR\System32\WindowsSandbox.exe" -ArgumentList "`"$wsbPath`"" -PassThru

$waitUntil = (Get-Date).AddMinutes($TimeoutMinutes)
while ((Get-Date) -lt $waitUntil) {
  if (Test-Path $resultPath) {
    Start-Sleep -Seconds 2
    $result = Get-Content $resultPath -Raw | ConvertFrom-Json
    Write-Host "Sandbox result: $($result.status)"
    Get-Process -Name "WindowsSandbox","WindowsSandboxClient","WindowsSandboxRemoteSession" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
    if ($result.status -ne "PASS") {
      if (Test-Path (Join-Path $Shared "lifecycle.log")) { Get-Content (Join-Path $Shared "lifecycle.log") | Write-Host }
      throw "Clean-account lifecycle FAILED: $($result.error)"
    }
    # expose for workflow
    $outDir = Join-Path $PWD "release-evidence\$Version\attestations"
    New-Item -ItemType Directory -Force -Path $outDir | Out-Null
    Copy-Item $resultPath (Join-Path $outDir "sandbox-clean-account-lifecycle.json") -Force
    if (Test-Path (Join-Path $Shared "lifecycle.log")) {
      Copy-Item (Join-Path $Shared "lifecycle.log") (Join-Path $outDir "sandbox-clean-account-lifecycle.log") -Force
    }
    Write-Host "PASS — evidence copied to $outDir"
    exit 0
  }
  Start-Sleep -Seconds 5
}

Get-Process -Name "WindowsSandbox","WindowsSandboxClient","WindowsSandboxRemoteSession" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
if (Test-Path (Join-Path $Shared "lifecycle.log")) { Get-Content (Join-Path $Shared "lifecycle.log") | Write-Host }
throw "Timed out waiting for Sandbox result.json after $TimeoutMinutes minutes"