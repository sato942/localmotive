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
  [ValidateSet("none", "timeout", "malformed-result", "missing-assets", "stall", "preservation-missing")]
  [string]$FaultSimulation = "none",
  # R07 (follow-up review db548c8): which REAL baseline persistence the
  # preservation step plants and verifies. `mirror` = the v0.5.0+ SQLite
  # catalog mirror with a user override row; `cache` = the v0.4.1-era catalog
  # cache record (that version predates the mirror).
  [ValidateSet("mirror", "cache")]
  [string]$PreservationFlavor = "mirror"
)
$ErrorActionPreference = "Stop"

if (-not $env:GH_TOKEN -and -not $env:GITHUB_TOKEN) {
  throw "GH_TOKEN (or GITHUB_TOKEN) is required so gh can download release assets in Actions. Set env.GH_TOKEN: `${{ github.token }} on the workflow step."
}
if (-not $env:GH_TOKEN -and $env:GITHUB_TOKEN) { $env:GH_TOKEN = $env:GITHUB_TOKEN }

$Root = Join-Path $env:TEMP ("localmotive-sandbox-" + $Version + "-" + (Get-Date -Format "yyyyMMddHHmmss"))
$Shared = Join-Path $Root "shared"
$OutDir = Join-Path $PWD "release-evidence\$Version\attestations"

$harnessRevision = "unknown"
try {
  $harnessRevision = (git -C $PSScriptRoot rev-parse HEAD 2>$null).Trim()
  if (-not $harnessRevision) { $harnessRevision = "unknown" }
} catch { $harnessRevision = "unknown" }
$EvidencePath = Join-Path $OutDir "$EvidenceName.json"
$EvidenceLog = Join-Path $OutDir "$EvidenceName.log"
New-Item -ItemType Directory -Force -Path $Shared | Out-Null
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null
Write-Host "Work dir: $Root"

$startedAt = (Get-Date).ToUniversalTime()
$stage = "initialization"
$candidateDigests = @{}
$candidateInventorySha256 = $null

function Get-FileSha256([string]$Path) {
  if (-not (Test-Path $Path)) { return $null }
  return (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLower()
}
# Concurrency guard (incident 2026-09-12): a killed parent left an orphaned
# attempt running, and its late TIMEOUT evidence overwrote a newer clean PASS
# record for the same scenario. One live run per evidence name: a second
# invocation refuses while a live owner exists, dead owners are taken over,
# and every evidence write is re-authorized against the lock so a superseded
# writer can never land output after losing ownership.
$LockDir = Join-Path $env:TEMP "localmotive-lifecycle-locks"
New-Item -ItemType Directory -Force -Path $LockDir | Out-Null
$script:LockPath = Join-Path $LockDir ("$EvidenceName.lock")
if (Test-Path $script:LockPath) {
  $owner = $null
  try { $owner = Get-Content $script:LockPath -Raw | ConvertFrom-Json } catch { $owner = $null }
  $ownerPid = if ($owner -and $owner.pid) { [int]$owner.pid } else { 0 }
  $alive = $ownerPid -gt 0 -and (Get-Process -Id $ownerPid -ErrorAction SilentlyContinue)
  if ($alive) {
    throw "Another lifecycle run for '$EvidenceName' is active (PID $ownerPid, started $($owner.startedAtUtc)). Refusing to race its evidence; stop that run or wait for it to finish."
  }
  Write-Host "Taking over a stale lifecycle lock for '$EvidenceName' (dead PID $ownerPid)."
}
([ordered]@{
  pid = $PID
  evidenceName = $EvidenceName
  startedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
  host = [System.Net.Dns]::GetHostName()
} | ConvertTo-Json) | Set-Content -Path $script:LockPath -Encoding UTF8
function Assert-LockOwnership {
  if (-not (Test-Path $script:LockPath)) {
    throw "The lifecycle lock for '$EvidenceName' disappeared mid-run; refusing to write evidence."
  }
  $current = $null
  try { $current = Get-Content $script:LockPath -Raw | ConvertFrom-Json } catch { $current = $null }
  if (-not $current -or [int]$current.pid -ne $PID) {
    throw "The lifecycle lock for '$EvidenceName' is owned by another process; this run no longer owns the evidence and refuses to write it."
  }
}

function Write-FailureEvidence([string]$Status, [string]$Message) {
  # Every terminal outcome leaves retrievable structured evidence that
  # identifies the stage, source revision, candidate digests and timing
  # (audit GH-06). The original error still propagates afterwards.
  # Windows PowerShell 5.1 does not allow an `if` statement as an expression
  # inside a hashtable literal; compute it first so the harness runs under the
  # default host shell as well as pwsh.
  Assert-LockOwnership
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
    harnessRevision = $harnessRevision
    startedAtUtc = $startedAt.ToString("o")
    finishedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    candidateDigests = $candidateDigests
    candidateInventorySha256 = $candidateInventorySha256
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

  # --- Candidate-inventory binding (2026-09-12 review; R13: inside the try
  # so a preflight refusal still produces structured failure evidence) ------
  # --- Candidate-inventory binding (2026-09-12 review) ------------------------
  # Lifecycle evidence binds to the ORIGINAL candidate inventory's full source
  # SHA. The environment may not substitute the current HEAD: a mismatch is a
  # hard refusal. The harness revision is recorded separately so the evidence
  # identifies both the candidate source and the tooling that produced it.
  if ($CandidateDir) {
    $inventoryPath = Join-Path $CandidateDir "candidate-inventory-$Version.json"
    if (-not (Test-Path $inventoryPath)) {
      if ($FaultSimulation -ne "none") {
        # Fault legs may run without staged candidates (the missing-assets
        # fixture is an absent directory by design); their evidence records a
        # null candidate binding instead of a substituted revision.
        $candidateSourceRevision = $null
      } else {
        throw "The candidate inventory is missing: $inventoryPath. Lifecycle evidence binds to the inventory's full source SHA; refusing to run without it."
      }
    } else {
      $inventory = Get-Content $inventoryPath -Raw | ConvertFrom-Json
      $candidateInventorySha256 = (Get-FileHash -Path $inventoryPath -Algorithm SHA256).Hash.ToLower()
      $candidateSourceRevision = [string]$inventory.sourceRevision
      if ($candidateSourceRevision -notmatch '^[0-9a-f]{40}$') {
        throw "The candidate inventory records an invalid source revision '$candidateSourceRevision'."
      }
      if ($env:LOCALMOTIVE_SOURCE_REVISION -and $env:LOCALMOTIVE_SOURCE_REVISION -ne $candidateSourceRevision) {
        throw "LOCALMOTIVE_SOURCE_REVISION '$($env:LOCALMOTIVE_SOURCE_REVISION)' does not match the candidate inventory's '$candidateSourceRevision'. Never substitute HEAD for the candidate source."
      }
      $env:LOCALMOTIVE_SOURCE_REVISION = $candidateSourceRevision
    }
  } else {
    $candidateSourceRevision = if ($env:LOCALMOTIVE_SOURCE_REVISION) { $env:LOCALMOTIVE_SOURCE_REVISION } else { $null }
  }
  $stage = "resolve-installers"
  if ($FaultSimulation -eq "missing-assets") {
    throw "installer assets are unavailable (fault simulation)"
  }
  $currentSetup = "Localmotive_${Version}_x64-setup.exe"
  $currentMsi = "Localmotive_${Version}_x64.msi"
  $prevVersion = $PreviousTag.TrimStart("v")
  $previousSetup = "Localmotive_${prevVersion}_x64-setup.exe"

  $haveLocal = $false
  if ($CandidateDir) {
    # R06 (follow-up review db548c8): a run that was told which candidates to
    # evaluate must evaluate exactly those bytes. A missing directory or file
    # is a hard refusal - never a silent fallback to a published release -
    # and every artifact this stage uses must match the run's inventory.
    if (-not (Test-Path $CandidateDir)) {
      throw "The explicitly supplied candidate directory does not exist: $CandidateDir"
    }
    $localSetup = Join-Path $CandidateDir $currentSetup
    $localMsi = Join-Path $CandidateDir $currentMsi
    if ((-not (Test-Path $localSetup)) -or (-not (Test-Path $localMsi))) {
      throw "The explicitly supplied candidate directory is missing installer artifacts ($currentSetup, $currentMsi); refusing to fall back to a published release."
    }
    if ($inventory) {
      foreach ($name in @($currentSetup, $currentMsi)) {
        $artifact = $inventory.artifacts | Where-Object { $_.name -eq $name }
        if (-not $artifact) {
          throw "The candidate inventory has no entry for $name; the qualification cannot bind these bytes."
        }
        $path = Join-Path $CandidateDir $name
        $sha = Get-FileSha256 $path
        $size = (Get-Item $path).Length
        if ($sha -ne ([string]$artifact.sha256).ToLower()) {
          throw "Candidate $name digest $sha does not match the inventory's $($artifact.sha256); refusing to qualify unverified bytes."
        }
        if ($size -ne [long]$artifact.sizeBytes) {
          throw "Candidate $name size $size does not match the inventory's $($artifact.sizeBytes)."
        }
      }
      Write-Host "Candidate installers verified against the inventory ($($inventory.sourceRevision))."
    }
    Write-Host "Using local candidates from $CandidateDir"
    Copy-Item $localSetup (Join-Path $Shared $currentSetup) -Force
    Copy-Item $localMsi (Join-Path $Shared $currentMsi) -Force
    $haveLocal = $true
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
  Copy-Item (Join-Path $fixtureDir "canary-catalog-cache.json") (Join-Path $Shared "canary-catalog-cache.json") -Force
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
    preservation = $PreservationFlavor
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
  if ($FaultSimulation -eq "preservation-missing") {
    # R05 witness: a PASS result whose collected preservation files never
    # arrive must fail the qualification at preservation-verification.
    Set-Content -Path $resultPath -Value '{"status":"PASS","notes":["fault simulation: preservation files omitted"]}' -Encoding UTF8
  } elseif ($FaultSimulation -eq "malformed-result") {
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
      $failDoc | Add-Member -NotePropertyName harnessRevision -NotePropertyValue $harnessRevision -Force
        $failDoc | Add-Member -NotePropertyName candidateDigests -NotePropertyValue $candidateDigests -Force
        Assert-LockOwnership
        ($failDoc | ConvertTo-Json -Depth 8) | Set-Content -Path $EvidencePath -Encoding UTF8
        $logPath = Join-Path $Shared "lifecycle.log"
        if (Test-Path $logPath) { Copy-Item $logPath $EvidenceLog -Force }
        throw "Clean-account lifecycle FAILED: $($result.error)"
      }
      # Host-side preservation verification: the collected mirror must still
      # carry its canary marker and the user-data canary must match
      # (G-06.I2). Failure flips the run verdict; evidence is written first.
      $stage = "preservation-verification"
      $collectedMirror = Join-Path $Shared "collected-mirror.sqlite"
      $collectedUserdata = Join-Path $Shared "collected-userdata.txt"
      $collectedCache = Join-Path $Shared "collected-catalog-cache.json"
      $preservationStatus = "missing-files"
      $preservationOutput = "collected files absent; the preservation step did not run"
      $preservationPresent = if ($PreservationFlavor -eq "mirror") {
        (Test-Path $collectedMirror) -and (Test-Path $collectedUserdata)
      } else {
        (Test-Path $collectedCache) -and (Test-Path $collectedUserdata)
      }
      if ($preservationPresent) {
        $py = Get-Command python -ErrorAction SilentlyContinue
        if ($py) {
          $verifyOut = & $py.Source (Join-Path $PWD "scripts\sandbox\verify_preservation.py") $collectedMirror $collectedUserdata $collectedCache "--flavor" $PreservationFlavor 2>&1
          $verifyCode = $LASTEXITCODE
          $preservationOutput = ($verifyOut | Out-String).Trim()
          if ($verifyCode -eq 0) { $preservationStatus = "PASS" } else { $preservationStatus = "FAIL" }
        } else {
          $preservationStatus = "UNVERIFIED"
          $preservationOutput = "python unavailable on this host; verifier not run"
        }
        if (Test-Path $collectedMirror) {
          Assert-LockOwnership
          Copy-Item $collectedMirror (Join-Path $OutDir "$EvidenceName-collected-mirror.sqlite") -Force
        }
        if (Test-Path $collectedCache) {
          Copy-Item $collectedCache (Join-Path $OutDir "$EvidenceName-collected-catalog-cache.json") -Force
        }
        if (Test-Path $collectedUserdata) {
          Copy-Item $collectedUserdata (Join-Path $OutDir "$EvidenceName-collected-userdata.txt") -Force
        }
        $preservationOutput | Set-Content (Join-Path $OutDir "$EvidenceName-verify.log") -Encoding UTF8
      }
      $doc = Get-Content $resultPath -Raw | ConvertFrom-Json
      if (-not ($doc.PSObject.Properties.Name -contains "stage")) {
        $doc | Add-Member -NotePropertyName stage -NotePropertyValue $stage -Force
      }
      $preservation = [ordered]@{ status = $preservationStatus; output = $preservationOutput }
      $doc | Add-Member -NotePropertyName preservation -NotePropertyValue $preservation -Force
      # Bind the pass verdict to the immutable source revision, candidate
      # digests, and candidate inventory (GH-03/GH-06/R06): the in-sandbox
      # document alone cannot carry them.
      $doc | Add-Member -NotePropertyName sourceRevision -NotePropertyValue $env:LOCALMOTIVE_SOURCE_REVISION -Force
      $doc | Add-Member -NotePropertyName harnessRevision -NotePropertyValue $harnessRevision -Force
      $doc | Add-Member -NotePropertyName candidateDigests -NotePropertyValue $candidateDigests -Force
      $doc | Add-Member -NotePropertyName candidateInventorySha256 -NotePropertyValue $candidateInventorySha256 -Force
      if ($preservationStatus -eq "PASS") {
        Assert-LockOwnership
      ($doc | ConvertTo-Json -Depth 8) | Set-Content -Path $EvidencePath -Encoding UTF8
        if (Test-Path (Join-Path $Shared "lifecycle.log")) {
          Copy-Item (Join-Path $Shared "lifecycle.log") $EvidenceLog -Force
        }
        Write-Host "PASS - evidence copied to $OutDir (preservation: PASS)"
        exit 0
      }
      # R05 (follow-up review db548c8): a lifecycle run whose preservation
      # step is missing or unverified is a FAILED qualification, not a pass
      # with a note. The evidence keeps the diagnostics and flips to FAIL
      # before the job exits nonzero.
      $doc.status = "FAIL"
      $doc.stage = "preservation-verification"
      Assert-LockOwnership
      ($doc | ConvertTo-Json -Depth 8) | Set-Content -Path $EvidencePath -Encoding UTF8
      if (Test-Path (Join-Path $Shared "lifecycle.log")) {
        Copy-Item (Join-Path $Shared "lifecycle.log") $EvidenceLog -Force
      }
      throw "Host preservation verification is not PASS ($preservationStatus): $preservationOutput"
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
} finally {
  # Release the lock only when this run still owns it; a run that lost
  # ownership (stale takeover) must not delete the new owner's lock.
  if (Test-Path $script:LockPath) {
    $holder = $null
    try { $holder = Get-Content $script:LockPath -Raw | ConvertFrom-Json } catch { $holder = $null }
    if ($holder -and [int]$holder.pid -eq $PID) { Remove-Item $script:LockPath -Force -ErrorAction SilentlyContinue }
  }
}
