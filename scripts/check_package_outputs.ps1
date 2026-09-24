# F9-04: the package job's missing-evidence report. Extracted from the workflow
# so the exact logic that runs on a package-stage failure can be exercised
# locally: a controlled failure must name every output the job did not produce
# and exit non-zero without touching the artifacts that do exist.
# Paths are relative to -Root, which is the release stage directory (P0-5):
# the workflow passes the stage under $RUNNER_TEMP, never the checkout.
param(
  [Parameter(Mandatory = $true)][string]$Root,
  [Parameter(Mandatory = $true)][string]$Version
)
$ErrorActionPreference = "Stop"

$packaging = @(
  "Localmotive_${Version}_x64-portable.exe",
  "Localmotive_${Version}_x64-setup.exe",
  "Localmotive_${Version}_x64.msi",
  "candidate-inventory-${Version}.json",
  "SHA256SUMS-${Version}.txt"
)
# P0-6 (REL-09): verification evidence belongs to the packaged-verification
# step, not to packaging. A missing verification record must name that step
# instead of reporting "Packaging failed".
$verification = @(
  "packaged-verification-${Version}.json"
)
$missingPackaging = @($packaging | Where-Object { -not (Test-Path (Join-Path $Root $_)) })
if ($missingPackaging.Count -gt 0) {
  Write-Host "Packaging failed before producing:"
  $missingPackaging | ForEach-Object { Write-Host " - MISSING $_" }
  exit 1
}
$missingVerification = @($verification | Where-Object { -not (Test-Path (Join-Path $Root $_)) })
if ($missingVerification.Count -gt 0) {
  Write-Host "Packaged verification failed before producing:"
  $missingVerification | ForEach-Object { Write-Host " - MISSING $_" }
  exit 1
}
Write-Host "Every expected package output exists; the failure is downstream of packaging and packaged verification."
exit 0
