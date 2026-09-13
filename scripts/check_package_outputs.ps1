# F9-04: the package job's missing-evidence report. Extracted from the workflow
# so the exact logic that runs on a package-stage failure can be exercised
# locally: a controlled failure must name every output the job did not produce
# and exit non-zero without touching the artifacts that do exist.
param(
  [Parameter(Mandatory = $true)][string]$Root,
  [Parameter(Mandatory = $true)][string]$Version
)
$ErrorActionPreference = "Stop"

$expected = @(
  "artifacts/Localmotive_${Version}_x64-portable.exe",
  "artifacts/Localmotive_${Version}_x64-setup.exe",
  "artifacts/Localmotive_${Version}_x64.msi",
  "artifacts/candidate-inventory-${Version}.json",
  "artifacts/SHA256SUMS-${Version}.txt",
  "artifacts/packaged-verification-${Version}.json"
)
$missing = @($expected | Where-Object { -not (Test-Path (Join-Path $Root $_)) })
if ($missing.Count -gt 0) {
  Write-Host "Packaging failed before producing:"
  $missing | ForEach-Object { Write-Host " - MISSING $_" }
  exit 1
}
Write-Host "Every expected package output exists; the failure is downstream of packaging."
exit 0
