# Host-side: consume candidate installers (or a published release), run the
# clean-account lifecycle in Windows Sandbox, and retain structured evidence
# on EVERY terminal outcome (audit GH-03 / GH-06).
param(
  [Parameter(Mandatory=$true)][string]$Tag,
  [Parameter(Mandatory=$true)][string]$Version,
  [string]$PreviousTag = "v0.4.0",
  [string]$Repo = "sato942/localmotive",
  [string]$CandidateDir = "",
  [int]$ReleaseWaitMinutes = 5,
  [int]$TimeoutMinutes = 45
)
$ErrorActionPreference = "Stop"

if (-not $env:GH_TOKEN -and -not $env:GITHUB_TOKEN) {
  throw "GH_TOKEN (or GITHUB_TOKEN) is required so gh can download release assets in Actions. Set env.GH_TOKEN: `${{ github.token }} on the workflow step."
}
if (-not $env:GH_TOKEN -and $env:GITHUB_TOKEN) { $env:GH_TOKEN = $env:GITHUB_TOKEN }

$Root = Join-Path $env:TEMP ("localmotive-sandbox-" + $Version + "-" + (Get-Date -Format "yyyyMMddHHmmss"))
$Shared = Join-Path $Root "shared"
$OutDir = Join-Path $PWD "release-evidence\$Version\attestations"
$EvidencePath = Join-Path $OutDir "sandbox-clean-account-lifecycle.json"
$EvidenceLog = Join-Path $OutDir "sandbox-clean-account-lifecycle.log"
New-Item -ItemType Directory -Force -Path $Shared | Out-Null
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Write-Host "Work dir: $Root"

$startedAt = (Get-Date).ToUniversalTime()
$stage = "initialization"
$candidateDigests = @{}

function Get-FileSha256([string]$Path) {
  if (-not (Test-Path $Path)) { return $null }
  return (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLower()
}
function Write-FailureEvidence([string]$Status, [string]$Message) {
  # Every terminal outcome leaves retrievable structured evidence that
  # identifies the stage, source revision, candidate digests and timing
  # (audit GH-06). The original error still propagates afterwards.
  $doc = [ordered]@{
    schema = "localmotive.sandbox-lifecycle.v0"
    status = $Status
    stage = $stage
    error = $Message
    tag = $Tag
    version = $Version
    previousTag = $PreviousTag
    sourceRevision = (if ($env:LOCALMOTIVE_SOURCE_REVISION) { $env:LOCALMOTIVE_SOURCE_REVISION } else { $null })
    startedAtUtc = $startedAt.ToString("o")
    finishedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    candidateDigests = $candidateDigests
  }
  ($doc | ConvertTo-Json -Depth 6) | Set-Content -Path $EvidencePath -Encoding UTF8
  $lifecycleLog = Join-Path $Shared "lifecycle.log"
  $lines = @()
  if (Test-Path $lifecycleLog) { $lines += Get-Content $lifecycleLog }
  $lines += "[host] $Status during stage '$stage': $Message"
  $lines | Set-Content -Path $EvidenceLog -Encoding UTF8
}
function Download-Asset([string]$RelTag, [string]$Name, [string]$OutPath) {
  Write-Host "Downloading $RelTag / $Name"
  gh release download $RelTag -R $Repo -p $Name -D (Split-Path $OutPath -Parent) --clobber
  $downloaded = Join-Path (Split-Path $OutPath -Parent) $Name
  if (-not (Test-Path $downloaded)) { throw "Download failed: $Name from $RelTag" }
  if ($downloaded -ne $OutPath) { Move-Item -Force $downloaded $OutPath }
}

try {
  $stage = "resolve-installers"
  $currentSetup = "Localmotive_${Version}_x64-setup.exe"
  $currentMsi = "Localmotive_${Version}_x64.msi"
  $prevVersion = $PreviousTag.TrimStart("v")
  $previousSetup = "Localmotive_${prevVersion}_x64-setup.exe"

  $haveLocal = $false
  if ($CandidateDir -and (Test-Path $CandidateDir)) {
    $localSetup = Join-Path $CandidateDir $currentSetup
    $localMsi = Join-Path $CandidateDir $currentMsi
    if ((Test-Path $localSetup) -and (Test-Path $localMsi)) {
      Write-Host "Using local candidates from $CandidateDir"
      Copy-Item $localSetup (Join-Path $Shared $currentSetup) -Force
      Copy-Item $localMsi (Join-Path $Shared $currentMsi) -Force
      $haveLocal = $true
    }
  }

  if (-not $haveLocal) {
    # Fallback only: consume an already-published release. The release path
    # always passes -CandidateDir, so the producer is never blocked waiting
    # for assets it has not published yet (audit GH-03 I2).
    $deadline = (Get-Date).AddMinutes($ReleaseWaitMinutes)
    while ($true) {
      try {
        $assets = gh release view $Tag -R $Repo --json assets --jq ".assets[].name"
        if (($assets -match [regex]::Escape($currentSetup)) -and ($assets -match [regex]::Escape($currentMsi))) { break }
        Write-Host "Waiting for release assets on $Tag ..."
      } catch {
        Write-Host "Release $Tag not visible yet: $($_.Exception.Message)"
      }
      if ((Get-Date) -gt $deadline) {
        throw @"
Timed out after $ReleaseWaitMinutes minute(s) waiting for GitHub Release $Tag assets ($currentSetup, $currentMsi).
The preferred path passes -CandidateDir with freshly built installers; this wait only serves already-published releases.
"@
      }
      Start-Sleep -Seconds 20
    }
    Download-Asset $Tag $currentSetup (Join-Path $Shared $currentSetup)
    Download-Asset $Tag $currentMsi (Join-Path $Shared $currentMsi)
  }

  $candidateDigests = @{
    currentSetup = (Get-FileSha256 (Join-Path $Shared $currentSetup))
    currentMsi = (Get-FileSha256 (Join-Path $Shared $currentMsi))
  }

  Download-Asset $PreviousTag $previousSetup (Join-Path $Shared $previousSetup)

  $stage = "prepare-sandbox"
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
  <Networking>Enable</Networking>
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

  # Networking=Enable so WebView2 bootstrapper can download in Sandbox (Disable caused NSIS exit 2).
  $stage = "sandbox-run"
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
        Copy-Item $resultPath $EvidencePath -Force
        $logPath = Join-Path $Shared "lifecycle.log"
        if (Test-Path $logPath) { Copy-Item $logPath $EvidenceLog -Force }
        throw "Clean-account lifecycle FAILED: $($result.error)"
      }
      Copy-Item $resultPath $EvidencePath -Force
      if (Test-Path (Join-Path $Shared "lifecycle.log")) {
        Copy-Item (Join-Path $Shared "lifecycle.log") $EvidenceLog -Force
      }
      Write-Host "PASS - evidence copied to $OutDir"
      exit 0
    }
    Start-Sleep -Seconds 5
  }

  Get-Process -Name "WindowsSandbox","WindowsSandboxClient","WindowsSandboxRemoteSession" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
  if (Test-Path (Join-Path $Shared "lifecycle.log")) { Get-Content (Join-Path $Shared "lifecycle.log") | Write-Host }
  $stage = "sandbox-timeout"
  Write-FailureEvidence "TIMEOUT" "Sandbox produced no result.json within $TimeoutMinutes minutes"
  throw "Timed out waiting for Sandbox result.json after $TimeoutMinutes minutes"
} catch {
  # Retain structured evidence on any terminal outcome while preserving the
  # original failure (audit GH-06 I1/I2).
  if (-not (Test-Path $EvidencePath) -or (Get-Content $EvidencePath -Raw | ConvertFrom-Json).status -eq "PASS") {
    Write-FailureEvidence "FAIL" $_.Exception.Message
  }
  throw
}
