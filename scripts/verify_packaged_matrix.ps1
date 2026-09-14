# Packaged-verification orchestration for the Release verify `package` job
# (audit GH-05; runs 34819219725, 34815989537, 34813162097, 34810494640).
#
# This script owns the native exit-code contract the inline workflow block
# kept getting wrong: GitHub's PowerShell runner appends
#   if ((Test-Path -LiteralPath variable:\LASTEXITCODE)) { exit $LASTEXITCODE }
# so the LAST native command in the step decides the step outcome even after
# every verifier passed. A final native kill of an already-dead fixture PID
# (prep() already stopped it) therefore failed green runs with exit 1 and no
# error text, ~282 ms after the merge PASS line.
#
# Revision 3 (U06-01): all logic lives in scripts/verify_packaged_impl.ps1
# so the cleanup acceptance matrix can dot-source the REAL functions and
# drive the REAL orchestration (Invoke-PackagedMatrix) under pwsh. This
# wrapper only sets the run identities, calls the matrix, and maps its
# return code to `exit`. The workflow invocation is unchanged.
#
# Contract:
# - Every material native command's exit code is captured immediately into a
#   named variable before another native command can replace $LASTEXITCODE.
# - Fixture init's exit code is checked BEFORE its JSON is parsed.
# - The original verifier failure is preserved when cleanup also has trouble.
# - Cleanup results are recorded separately from functional results.
# - An owned process that is already gone counts as successful cleanup.
# - A live owned process that will not stop inside the deadline is an
#   explicit cleanup failure that fails the run.
# - The final exit code comes from the recorded outcomes, never from a stray
#   $LASTEXITCODE.
# - Cleanup lives in an outer finally covering fixture allocation, candidate
#   startup, verification, prep, restart, and merge; partially initialized
#   state is handled.
# - Fixture adoption refuses unverified identity: a failed/slow CIM lookup,
#   a missing command line, or a foreign command line never adopts.
param(
  [Parameter(Mandatory = $true)][string]$Portable,
  [Parameter(Mandatory = $true)][string]$Version,
  [Parameter(Mandatory = $true)][string]$ResolvedSha,
  [Parameter(Mandatory = $true)][int]$CdpPort
)

. (Join-Path $PSScriptRoot "verify_packaged_impl.ps1")

# --- Run-specific isolated state (safe to clean twice) -----------------------
# Every path carries the attempt identity ($AttemptId): two promotions or a
# retry on the same runner can never share a profile, a fixture state dir, or
# a CDP port, and cleanup only ever touches this attempt's paths and handles.
$AttemptId = "verify-$Version-$CdpPort"
$isolatedRoot = Join-Path $env:RUNNER_TEMP "localmotive-verify-appdata-$AttemptId"
$profile = Join-Path $env:RUNNER_TEMP "localmotive-verify-webview2-$AttemptId"
$catalogState = Join-Path $env:RUNNER_TEMP "localmotive-verify-catalog-$AttemptId"

$code = Invoke-PackagedMatrix `
  -Portable $Portable `
  -Version $Version `
  -ResolvedSha $ResolvedSha `
  -CdpPort $CdpPort `
  -AttemptId $AttemptId `
  -IsolatedRoot $isolatedRoot `
  -Profile $profile `
  -CatalogState $catalogState

# The final exit comes from the recorded outcome only. A stray $LASTEXITCODE
# from any native command above must never decide the step.
exit $code
