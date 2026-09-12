# GH-06.V2 witness harness: drive `host-run-lifecycle.ps1` through its
# terminal-outcome paths and assert that each one leaves a bounded structured
# outcome with its stage and (where known) the candidate identity:
#
#   missing-assets   -> FAIL   at resolve-installers
#   timeout          -> TIMEOUT at sandbox-timeout
#   malformed-result -> FAIL   at sandbox-run
#   cancellation     -> the process is killed; no PASS artifact may exist
#   preservation-missing -> FAIL at preservation-verification (R05)
#   live-lock        -> a run refuses to start while a live owner exists
#   stale-lock       -> a dead owner's lock is taken over and released
#
# The fault paths are default-off in the real run; this script is their
# evidence producer. Usage (from the repository root):
#   powershell -NoProfile -ExecutionPolicy Bypass -File scripts/sandbox/test-fault-evidence.ps1
param(
  [string]$Version = "0.6.0",
  [string]$CandidateDir = ".hermes-0.6/final-candidates",
  [string]$PreviousTag = "v0.4.1"
)
$ErrorActionPreference = "Stop"

# The harness requires a token even for the witness legs (its published-asset
# fallback guard); the witness legs never reach a network call.
if (-not $env:GH_TOKEN) { $env:GH_TOKEN = "fault-simulation" }
# Do NOT export LOCALMOTIVE_SOURCE_REVISION here: the harness binds every
# evidence document to the candidate inventory's full source SHA, and an
# environment value that merely echoes the current HEAD is refused (and was
# the exact HEAD-substitution the binding exists to stop).

$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$attestations = Join-Path $root "release-evidence\$Version\attestations"
$candidateDir = Join-Path $root $CandidateDir
$script:expectedSetupSha = (Get-FileHash (Join-Path $candidateDir "Localmotive_${Version}_x64-setup.exe") -Algorithm SHA256).Hash.ToLowerInvariant()
$script:expectedMsiSha = (Get-FileHash (Join-Path $candidateDir "Localmotive_${Version}_x64.msi") -Algorithm SHA256).Hash.ToLowerInvariant()
New-Item -ItemType Directory -Force -Path $attestations | Out-Null

function Remove-Witness([string]$WitnessName) {
  # A deleted file cannot be satisfied by a previous candidate's document; the
  # leg must produce its own evidence (a stale FAIL once defeated this check).
  foreach ($suffix in @(".json", ".log")) {
    Remove-Item (Join-Path $attestations "$WitnessName$suffix") -Force -ErrorAction SilentlyContinue
  }
}

function Assert-Witness([string]$WitnessName, [string]$ExpectedStatus, [string]$ExpectedStage, [bool]$ExpectDigests, [string]$ExpectedPreservation = "") {
  $docPath = Join-Path $attestations "$WitnessName.json"
  if (-not (Test-Path $docPath)) { throw "witness $WitnessName produced no evidence document" }
  $doc = Get-Content $docPath -Raw | ConvertFrom-Json
  if ($doc.status -ne $ExpectedStatus) { throw "witness ${WitnessName}: status '$($doc.status)' != '$ExpectedStatus'" }
  if ($doc.stage -ne $ExpectedStage) { throw "witness ${WitnessName}: stage '$($doc.stage)' != '$ExpectedStage'" }
  if (-not ($doc.PSObject.Properties.Name -contains "sourceRevision")) { throw "witness ${WitnessName}: missing sourceRevision binding" }
  if (-not ($doc.PSObject.Properties.Name -contains "candidateDigests")) { throw "witness ${WitnessName}: missing candidateDigests binding" }
  if (-not ($doc.PSObject.Properties.Name -contains "candidateInventorySha256")) { throw "witness ${WitnessName}: missing candidateInventorySha256 binding" }
  if ($ExpectedPreservation) {
    if ($doc.preservation.status -ne $ExpectedPreservation) { throw "witness ${WitnessName}: preservation '$($doc.preservation.status)' != '$ExpectedPreservation'" }
  }
  if ($ExpectDigests) {
    if ($doc.candidateDigests.currentSetup -ne $script:expectedSetupSha) { throw "witness ${WitnessName}: setup digest '$($doc.candidateDigests.currentSetup)' does not match the staged candidate '$script:expectedSetupSha'" }
    if ($doc.candidateDigests.currentMsi -ne $script:expectedMsiSha) { throw "witness ${WitnessName}: msi digest does not match the staged candidate" }
  }
  Write-Host "WITNESS OK: $WitnessName -> $($doc.status) at stage $($doc.stage)"
}

Push-Location $root
try {
  # Leg 1: an early installer-asset failure is bounded at resolve-installers.
  Remove-Witness "witness-missing-assets"
  try {
    & "$PSScriptRoot\host-run-lifecycle.ps1" -Tag "v$Version" -Version $Version `
      -CandidateDir (Join-Path $root "missing-assets-fixture") `
      -EvidenceName "witness-missing-assets" -FaultSimulation "missing-assets"
  } catch { }
  Assert-Witness "witness-missing-assets" "FAIL" "resolve-installers" $false

  # Leg 2: the bounded wait expiring produces TIMEOUT at sandbox-timeout with
  # the candidate identity bound.
  Remove-Witness "witness-timeout"
  try {
    & "$PSScriptRoot\host-run-lifecycle.ps1" -Tag "v$Version" -Version $Version `
      -CandidateDir $CandidateDir -PreviousTag $PreviousTag `
      -EvidenceName "witness-timeout" -FaultSimulation "timeout"
  } catch { }
  Assert-Witness "witness-timeout" "TIMEOUT" "sandbox-timeout" $true

  # Leg 3: a malformed in-sandbox result produces FAIL at sandbox-run with the
  # candidate identity bound.
  Remove-Witness "witness-malformed-result"
  try {
    & "$PSScriptRoot\host-run-lifecycle.ps1" -Tag "v$Version" -Version $Version `
      -CandidateDir $CandidateDir -PreviousTag $PreviousTag `
      -EvidenceName "witness-malformed-result" -FaultSimulation "malformed-result"
  } catch { }
  Assert-Witness "witness-malformed-result" "FAIL" "sandbox-run" $true

  # Leg 4: cancellation (process kill) must leave no PASS artifact at all.
  $cancellationEvidence = Join-Path $attestations "witness-cancellation.json"
  if (Test-Path $cancellationEvidence) { Remove-Item $cancellationEvidence -Force }
  $stalled = Start-Process -FilePath "powershell" -PassThru -ArgumentList @(
    "-NoProfile", "-ExecutionPolicy", "Bypass",
    "-File", "$PSScriptRoot\host-run-lifecycle.ps1",
    "-Tag", "v$Version", "-Version", $Version,
    "-CandidateDir", $CandidateDir, "-PreviousTag", $PreviousTag,
    "-EvidenceName", "witness-cancellation", "-FaultSimulation", "stall"
  )
  Start-Sleep -Seconds 8
  if ($stalled.HasExited) { throw "the stall leg exited before it could be cancelled (exit $($stalled.ExitCode))" }
  Stop-Process -Id $stalled.Id -Force
  $stalled.WaitForExit()
  Write-Host "cancellation leg: killed PID $($stalled.Id), exit code $($stalled.ExitCode)"
  if (Test-Path $cancellationEvidence) {
    $doc = Get-Content $cancellationEvidence -Raw | ConvertFrom-Json
    if ($doc.status -eq "PASS") { throw "a killed run must not leave PASS evidence" }
    throw "a killed run left an evidence document; the workflow-level cancellation outcome is the 'MISSING' summary, not a file"
  }
  Write-Host "WITNESS OK: cancellation -> killed with no PASS artifact"

  # Leg 5: a PASS result whose preservation files never arrived must fail at
  # preservation-verification with the missing-files status retained (R05).
  Remove-Witness "witness-preservation-missing"
  $legFailed = $false
  try {
    & "$PSScriptRoot\host-run-lifecycle.ps1" -Tag "v$Version" -Version $Version `
      -CandidateDir $CandidateDir -PreviousTag $PreviousTag `
      -EvidenceName "witness-preservation-missing" -FaultSimulation "preservation-missing"
  } catch { $legFailed = $true }
  if (-not $legFailed) {
    throw "the run must exit nonzero when preservation is not PASS; a FAIL evidence document alone is a false green"
  }
  Assert-Witness "witness-preservation-missing" "FAIL" "preservation-verification" $true "missing-files"

  # Leg 6: concurrent contention AND owner loss, exercised with a REAL second
  # process (not a fabricated lock file). A live owner holds the OS-enforced
  # exclusive handle; a second attempt for the same evidence name must refuse
  # before writing anything. Killing the owner releases the handle; the next
  # attempt then takes over, publishes its own evidence, and releases the lock.
  # The killed owner must have written nothing.
  $lockDir = Join-Path $env:TEMP "localmotive-lifecycle-locks"
  New-Item -ItemType Directory -Force -Path $lockDir | Out-Null
  $liveLock = Join-Path $lockDir "witness-live-lock.lock"
  Remove-Witness "witness-live-lock"
  Remove-Item $liveLock -Force -ErrorAction SilentlyContinue
  $owner = Start-Process -FilePath "powershell" -PassThru -ArgumentList @(
    "-NoProfile", "-ExecutionPolicy", "Bypass",
    "-File", "$PSScriptRoot\host-run-lifecycle.ps1",
    "-Tag", "v$Version", "-Version", $Version,
    "-CandidateDir", $CandidateDir, "-PreviousTag", $PreviousTag,
    "-EvidenceName", "witness-live-lock", "-FaultSimulation", "stall",
    "-StallSeconds", "60"
  )
  $ownerHolds = $false
  for ($i = 0; $i -lt 80 -and -not $ownerHolds; $i++) {
    Start-Sleep -Milliseconds 500
    if (Test-Path $liveLock) {
      try {
        $holder = Get-Content $liveLock -Raw | ConvertFrom-Json
        $ownerHolds = [bool]($holder -and [int]$holder.pid -eq $owner.Id)
      } catch { $ownerHolds = $false }
    }
    if ($owner.HasExited) { break }
  }
  if (-not $ownerHolds) {
    if (-not $owner.HasExited) { Stop-Process -Id $owner.Id -Force }
    throw "the live owner (PID $($owner.Id)) never recorded its lock ownership"
  }
  $refusalMessage = ""
  try {
    & "$PSScriptRoot\host-run-lifecycle.ps1" -Tag "v$Version" -Version $Version `
      -CandidateDir (Join-Path $root "missing-assets-fixture") `
      -EvidenceName "witness-live-lock" -FaultSimulation "missing-assets" 2>&1 | Out-String | ForEach-Object { $refusalMessage = $_ }
  } catch { $refusalMessage = $_.Exception.Message }
  if ($refusalMessage -notmatch "Refusing to race its evidence") {
    if (-not $owner.HasExited) { Stop-Process -Id $owner.Id -Force }
    throw "a live lock must refuse the concurrent run; got: $refusalMessage"
  }
  if ($refusalMessage -notmatch "PID $($owner.Id)") {
    if (-not $owner.HasExited) { Stop-Process -Id $owner.Id -Force }
    throw "the refusal must name the live owner PID $($owner.Id); got: $refusalMessage"
  }
  if (Test-Path (Join-Path $attestations "witness-live-lock.json")) {
    if (-not $owner.HasExited) { Stop-Process -Id $owner.Id -Force }
    throw "a refused run must not write evidence over the live owner's records"
  }
  Write-Host "WITNESS OK: witness-live-lock -> concurrent attempt refused while the owner lives (PID $($owner.Id))"

  # Owner loss: kill the live owner; the OS releases its handle. The killed
  # owner wrote no evidence; the next attempt takes over and publishes.
  Stop-Process -Id $owner.Id -Force
  $owner.WaitForExit()
  Start-Sleep -Seconds 1
  if (Test-Path (Join-Path $attestations "witness-live-lock.json")) {
    throw "a killed owner must leave no evidence document"
  }
  $takeoverDone = $false
  for ($attempt = 0; $attempt -lt 5 -and -not $takeoverDone; $attempt++) {
    try {
      & "$PSScriptRoot\host-run-lifecycle.ps1" -Tag "v$Version" -Version $Version `
        -CandidateDir (Join-Path $root "missing-assets-fixture") `
        -EvidenceName "witness-live-lock" -FaultSimulation "missing-assets"
      $takeoverDone = $true
    } catch {
      if ($_.Exception.Message -match "Refusing to race its evidence") { Start-Sleep -Seconds 1 }
      else { $takeoverDone = $true }
    }
  }
  Assert-Witness "witness-live-lock" "FAIL" "resolve-installers" $false
  if (Test-Path $liveLock) {
    throw "the takeover run must release its lock at exit"
  }
  Write-Host "WITNESS OK: witness-live-lock -> owner killed, takeover published its own evidence, lock released"

  # Leg 7: a dead owner's lock is stale; the run takes it over, proceeds to
  # its normal bounded failure, and releases the lock on the way out.
  Remove-Witness "witness-stale-lock"
  $deadProc = Start-Process -FilePath "cmd" -ArgumentList "/c", "exit" -PassThru
  $deadProc.WaitForExit()
  $staleLock = Join-Path $lockDir "witness-stale-lock.lock"
  ([ordered]@{ pid = $deadProc.Id; evidenceName = "witness-stale-lock"; startedAtUtc = (Get-Date).ToUniversalTime().ToString("o"); host = "witness" } | ConvertTo-Json) | Set-Content -Path $staleLock -Encoding UTF8
  try {
    & "$PSScriptRoot\host-run-lifecycle.ps1" -Tag "v$Version" -Version $Version `
      -CandidateDir (Join-Path $root "missing-assets-fixture") `
      -EvidenceName "witness-stale-lock" -FaultSimulation "missing-assets"
  } catch { }
  Assert-Witness "witness-stale-lock" "FAIL" "resolve-installers" $false
  if (Test-Path $staleLock) {
    throw "the stale-lock leg must release its lock at exit"
  }
  Write-Host "WITNESS OK: witness-stale-lock -> stale owner taken over, run proceeded, lock released"

  Write-Host "ALL WITNESS LEGS PASS"
} finally {
  Pop-Location
}
