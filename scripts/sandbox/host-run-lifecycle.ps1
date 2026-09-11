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
  [int]$TimeoutMinutes = 45,
  [string]$EvidenceName = "sandbox-clean-account-lifecycle",
  # Fault simulation for the GH-06.V2 witness legs (default: a real run).
  # none | timeout | malformed-result | missing-assets | stall
  # Each simulated failure must leave a bounded structured outcome with its
  # stage and, where known, the candidate identity.
  [ValidateSet("none", "timeout", "malformed-result", "missing-assets", "stall")]
  [string]$FaultSimulation = "none"
)
$ErrorActionPreference = "Stop"

if (-not $env:GH_TOKEN -and -not $env:GITHUB_TOKEN) {
  throw "GH_TOKEN (or GITHUB_TOKEN) is required so gh can download release assets in Actions. Set env.GH_TOKEN: `${{ github.token }} on the workflow step."
}
if (-not $env:GH_TOKEN -and $env:GITHUB_TOKEN) { $env:GH_TOKEN = $env:GITHUB_TOKEN }

$Root = Join-Path $env:TEMP ("localmotive-sandbox-" + $Version + "-" + (Get-Date -Format "yyyyMMddHHmmss"))
$Shared = Join-Path $Root "shared"
$OutDir = Join-Path $PWD "release-evidence\$Version\attestations"
$EvidencePath = Join-Path $OutDir "$EvidenceName.json"
$EvidenceLog = Join-Path $OutDir "$EvidenceName.log"
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
  # Windows PowerShell 5.1 does not allow an `if` statement as an expression
  # inside a hashtable literal; compute it first so the harness runs under the
  # default host shell as well as pwsh.
  $sourceRevision = if ($env:LOCALMOTIVE_SOURCE_REVISION) { $env:LOCALMOTIVE_SOURCE_REVISION } else { $null }
  $doc = [ordered]@{
    schema = "localmotive.sandbox-lifecycle.v0"
    status = $Status
    stage = $stage
    error = $Message
    tag = $Tag
    version = $Version
    previousTag = $PreviousTag
    sourceRevision = $sourceRevision
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
  # GH-06.V2 witness: an early installer-asset failure must leave a bounded
  # structured outcome at this stage, before any sandbox work.
  $runStarted = Get-Date
  $stage = "resolve-installers"
  if ($FaultSimulation -eq "missing-assets") {
    throw "installer assets are unavailable (fault simulation)"
  }
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

  if ($FaultSimulation -eq "none") {
    Download-Asset $PreviousTag $previousSetup (Join-Path $Shared $previousSetup)
  } else {
    # The witness legs exercise terminal-outcome handling; the published
    # baseline download is not part of those paths.
    Write-Host "Fault simulation '$FaultSimulation': skipping the baseline download"
  }

  $stage = "stage-canaries"
  $fixtureDir = Join-Path $PWD "scripts\sandbox"
  $b64 = (Get-Content (Join-Path $fixtureDir "canary-mirror.sqlite.b64") -Raw).Trim()
  [IO.File]::WriteAllBytes((Join-Path $Shared "canary-mirror.sqlite"), [Convert]::FromBase64String($b64))
  Copy-Item (Join-Path $fixtureDir "canary-userdata.txt") (Join-Path $Shared "canary-userdata.txt") -Force
  Set-Content -Path (Join-Path $Shared "preserve.json") -Value '{"version":1}' -Encoding UTF8

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
  if ($FaultSimulation -eq "malformed-result") {
    # GH-06.V2 witness: a malformed in-sandbox result must become a bounded
    # FAIL outcome with this stage instead of a hang or a false pass.
    Set-Content -Path $resultPath -Value "{ this is not json" -Encoding UTF8
  } elseif ($FaultSimulation -eq "timeout") {
    # GH-06.V2 witness: force the bounded wait to expire immediately.
    $TimeoutMinutes = 0
  } elseif ($FaultSimulation -eq "stall") {
    # GH-06.V2 witness: a cancellable long stage. The cancellation leg kills
    # this process and asserts that a killed run leaves no PASS artifact.
    Write-Host "Fault simulation 'stall': waiting for cancellation..."
    Start-Sleep -Seconds 600
  } else {
    Write-Host "Launching Windows Sandbox..."
    $sandbox = Start-Process -FilePath "$env:WINDIR\System32\WindowsSandbox.exe" -ArgumentList "`"$wsbPath`"" -PassThru
  }

  $waitUntil = (Get-Date).AddMinutes($TimeoutMinutes)
  while ((Get-Date) -lt $waitUntil) {
    if (Test-Path $resultPath) {
      Start-Sleep -Seconds 2
      $result = Get-Content $resultPath -Raw | ConvertFrom-Json
      Write-Host "Sandbox result: $($result.status)"
      Get-Process -Name "WindowsSandbox","WindowsSandboxClient","WindowsSandboxRemoteSession" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
      if ($result.status -ne "PASS") {
        if (Test-Path (Join-Path $Shared "lifecycle.log")) { Get-Content (Join-Path $Shared "lifecycle.log") | Write-Host }
        # Retain host-side identity binding on failure too (audit GH-06 I1):
        # the in-sandbox document cannot carry the source revision or the
        # candidate digests it never saw. Without this the FAIL evidence was
        # retrievable but not attributable.
        $failDoc = Get-Content $resultPath -Raw | ConvertFrom-Json
        $failDoc | Add-Member -NotePropertyName sourceRevision -NotePropertyValue $env:LOCALMOTIVE_SOURCE_REVISION -Force
        $failDoc | Add-Member -NotePropertyName candidateDigests -NotePropertyValue $candidateDigests -Force
        ($failDoc | ConvertTo-Json -Depth 8) | Set-Content -Path $EvidencePath -Encoding UTF8
        $logPath = Join-Path $Shared "lifecycle.log"
        if (Test-Path $logPath) { Copy-Item $logPath $EvidenceLog -Force }
        throw "Clean-account lifecycle FAILED: $($result.error)"
      }
      # Host-side preservation verification: the collected mirror must still
      # carry its canary marker and the user-data canary must match
      # (G-06.I2). Failure flips the run verdict; evidence is written first.
      $collectedMirror = Join-Path $Shared "collected-mirror.sqlite"
      $collectedUserdata = Join-Path $Shared "collected-userdata.txt"
      $preservationStatus = "missing-files"
      $preservationOutput = "collected files absent; the preservation step did not run"
      if ((Test-Path $collectedMirror) -and (Test-Path $collectedUserdata)) {
        $py = Get-Command python -ErrorAction SilentlyContinue
        if ($py) {
          $verifyOut = & $py.Source (Join-Path $PWD "scripts\sandbox\verify_preservation.py") $collectedMirror $collectedUserdata 2>&1
          $verifyCode = $LASTEXITCODE
          $preservationOutput = ($verifyOut | Out-String).Trim()
          if ($verifyCode -eq 0) { $preservationStatus = "PASS" } else { $preservationStatus = "FAIL" }
        } else {
          $preservationStatus = "UNVERIFIED"
          $preservationOutput = "python unavailable on this host; verifier not run"
        }
        Copy-Item $collectedMirror (Join-Path $OutDir "$EvidenceName-collected-mirror.sqlite") -Force
        Copy-Item $collectedUserdata (Join-Path $OutDir "$EvidenceName-collected-userdata.txt") -Force
        $preservationOutput | Set-Content (Join-Path $OutDir "$EvidenceName-verify.log") -Encoding UTF8
      }
      $doc = Get-Content $resultPath -Raw | ConvertFrom-Json
      $preservation = [ordered]@{ status = $preservationStatus; output = $preservationOutput }
      $doc | Add-Member -NotePropertyName preservation -NotePropertyValue $preservation -Force
      # Bind the pass verdict to the immutable source revision and candidate
      # digests (GH-03/GH-06): the in-sandbox document alone cannot carry them.
      $doc | Add-Member -NotePropertyName sourceRevision -NotePropertyValue $env:LOCALMOTIVE_SOURCE_REVISION -Force
      $doc | Add-Member -NotePropertyName candidateDigests -NotePropertyValue $candidateDigests -Force
      ($doc | ConvertTo-Json -Depth 8) | Set-Content -Path $EvidencePath -Encoding UTF8
      if (Test-Path (Join-Path $Shared "lifecycle.log")) {
        Copy-Item (Join-Path $Shared "lifecycle.log") $EvidenceLog -Force
      }
      if ($preservationStatus -eq "FAIL") {
        throw "Host preservation verification FAILED: $preservationOutput"
      }
      Write-Host "PASS - evidence copied to $OutDir (preservation: $preservationStatus)"
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
  # original failure (audit GH-06 I1/I2). A file from THIS run (written after
  # the try began) is authoritative; anything older is stale identity and is
  # rewritten, so a previous candidate's FAIL can never masquerade as this
  # run's outcome.
  $existingEvidence = Test-Path $EvidencePath
  $staleEvidence = -not $existingEvidence -or (Get-Item $EvidencePath).LastWriteTime -lt $runStarted
  $passEvidence = $existingEvidence -and -not $staleEvidence -and (Get-Content $EvidencePath -Raw | ConvertFrom-Json).status -eq "PASS"
  if ($staleEvidence -or $passEvidence) {
    Write-FailureEvidence "FAIL" $_.Exception.Message
  }
  throw
}
