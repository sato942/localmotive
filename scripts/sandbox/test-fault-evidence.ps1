# GH-06.V2 witness harness: drive `host-run-lifecycle.ps1` through its
# terminal-outcome paths and assert that each one leaves a bounded structured
# outcome with its stage and (where known) the candidate identity:
#
#   missing-assets   -> FAIL   at resolve-installers
#   timeout          -> TIMEOUT at sandbox-timeout
#   malformed-result -> FAIL   at sandbox-run
#   cancellation     -> the process is killed; no PASS artifact may exist
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
if (-not $env:LOCALMOTIVE_SOURCE_REVISION) {
  $env:LOCALMOTIVE_SOURCE_REVISION = (& git rev-parse HEAD).Trim()
}

$root = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
$attestations = Join-Path $root "release-evidence\$Version\attestations"
New-Item -ItemType Directory -Force -Path $attestations | Out-Null

function Assert-Witness([string]$WitnessName, [string]$ExpectedStatus, [string]$ExpectedStage, [bool]$ExpectDigests) {
  $docPath = Join-Path $attestations "$WitnessName.json"
  if (-not (Test-Path $docPath)) { throw "witness $WitnessName produced no evidence document" }
  $doc = Get-Content $docPath -Raw | ConvertFrom-Json
  if ($doc.status -ne $ExpectedStatus) { throw "witness ${WitnessName}: status '$($doc.status)' != '$ExpectedStatus'" }
  if ($doc.stage -ne $ExpectedStage) { throw "witness ${WitnessName}: stage '$($doc.stage)' != '$ExpectedStage'" }
  if (-not ($doc.PSObject.Properties.Name -contains "sourceRevision")) { throw "witness ${WitnessName}: missing sourceRevision binding" }
  if (-not ($doc.PSObject.Properties.Name -contains "candidateDigests")) { throw "witness ${WitnessName}: missing candidateDigests binding" }
  if ($ExpectDigests -and -not $doc.candidateDigests.currentSetup) { throw "witness ${WitnessName}: candidate digest not bound" }
  Write-Host "WITNESS OK: $WitnessName -> $($doc.status) at stage $($doc.stage)"
}

Push-Location $root
try {
  # Leg 1: an early installer-asset failure is bounded at resolve-installers.
  try {
    & "$PSScriptRoot\host-run-lifecycle.ps1" -Tag "v$Version" -Version $Version `
      -CandidateDir (Join-Path $root "missing-assets-fixture") `
      -EvidenceName "witness-missing-assets" -FaultSimulation "missing-assets"
  } catch { }
  Assert-Witness "witness-missing-assets" "FAIL" "resolve-installers" $false

  # Leg 2: the bounded wait expiring produces TIMEOUT at sandbox-timeout with
  # the candidate identity bound.
  try {
    & "$PSScriptRoot\host-run-lifecycle.ps1" -Tag "v$Version" -Version $Version `
      -CandidateDir $CandidateDir -PreviousTag $PreviousTag `
      -EvidenceName "witness-timeout" -FaultSimulation "timeout"
  } catch { }
  Assert-Witness "witness-timeout" "TIMEOUT" "sandbox-timeout" $true

  # Leg 3: a malformed in-sandbox result produces FAIL at sandbox-run with the
  # candidate identity bound.
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

  Write-Host "ALL WITNESS LEGS PASS"
} finally {
  Pop-Location
}
