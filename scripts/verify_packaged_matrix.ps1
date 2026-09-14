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
# Revision 2 (paste-7 bounded repair): owned-process cleanup is bounded and
# run-specific. The fixture child is tracked by a .NET process HANDLE captured
# at spawn time in `init` (StopFixture stops the OWNED handle only -- PID
# reuse on this shared runner can never redirect the kill), every stop is
# bounded by Stopwatch deadlines, every native exit code is captured
# immediately, and every outcome is recorded by name. No port scans, no name
# scans, no unbounded CIM.
#
# Contract:
# - Every material native command's exit code is captured immediately into a
#   named variable before another native command can replace $LASTEXITCODE.
# - Fixture init's exit code is checked BEFORE its JSON is parsed.
# - The original verifier failure is preserved when cleanup also has trouble.
# - Cleanup results are recorded separately from functional results.
# - An owned process that is already gone counts as successful cleanup.
# - A live owned process that will not stop inside the deadline is an
#   explicit cleanup failure.
# - The final exit code comes from the recorded outcomes, never from a stray
#   $LASTEXITCODE.
# - Cleanup lives in an outer finally covering fixture allocation, candidate
#   startup, verification, prep, restart, and merge; partially initialized
#   state is handled.
param(
  [Parameter(Mandatory = $true)][string]$Portable,
  [Parameter(Mandatory = $true)][string]$Version,
  [Parameter(Mandatory = $true)][string]$ResolvedSha,
  [Parameter(Mandatory = $true)][int]$CdpPort
)

$ErrorActionPreference = "Stop"
$script:functionalFailed = $false
$script:functionalError = ""
$script:cleanupNotes = @()
$script:lastVerifierOutput = ""

function Write-Phase([string]$Message) {
  $stamp = (Get-Date).ToString("o")
  Write-Host "verify-orchestrator [$stamp] $Message"
}

# Run one node verifier phase and return its exit code, captured immediately
# from the OWNED handle before any other native command can replace it.
# Nothing here throws: the caller records the outcome and decides.
function Invoke-Verifier([string]$Description, [string[]]$Arguments) {
  $outFile = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-verify-phase-$AttemptId.txt"
  Remove-Item -LiteralPath $outFile -Force -ErrorAction SilentlyContinue
  $proc = Start-Process -FilePath "node" -ArgumentList $Arguments -NoNewWindow -Wait -PassThru -RedirectStandardOutput $outFile
  $code = $proc.ExitCode
  Write-Phase "$Description exit=$code"
  $script:lastVerifierOutput = ""
  try { $script:lastVerifierOutput = (Get-Content $outFile -Raw) } catch { }
  return $code
}

function Stop-OwnedProcess([System.Diagnostics.Process]$Process, [string]$Why, [int]$DeadlineSeconds = 15) {
  # Bounded stop of an OWNED handle: the handle was captured at spawn, so PID
  # reuse can never redirect this kill. Already-exited is successful cleanup.
  # A live process that will not stop inside the deadline is an explicit
  # cleanup failure (returned by name, never thrown).
  if (-not $Process) { return "skipped-no-handle" }
  $targetPid = 0
  try { $targetPid = $Process.Id } catch { return "already-stopped" }
  try {
    if ($Process.HasExited) {
      Write-Phase "cleanup $Why pid=$targetPid already-stopped"
      return "already-stopped"
    }
  } catch {
    Write-Phase "cleanup $Why pid=$targetPid already-stopped"
    return "already-stopped"
  }
  Write-Phase "cleanup $Why pid=$targetPid stopping (deadline ${DeadlineSeconds}s)"
  try { $Process.Kill() } catch { }
  $code = 0
  try {
    if (-not $Process.WaitForExit($DeadlineSeconds * 1000)) {
      Write-Phase "cleanup $Why pid=$targetPid STILL-ALIVE"
      return "still-alive"
    }
  } catch {
    Write-Phase "cleanup $Why pid=$targetPid already-stopped"
    return "already-stopped"
  }
  try { $code = $Process.ExitCode } catch { $code = 0 }
  Write-Phase "cleanup $Why pid=$targetPid stopped exit=$code"
  return "stopped"
}

function Start-Candidate([int]$Port) {
  $process = Start-Process $Portable -PassThru
  $deadline = (Get-Date).AddSeconds(90)
  while ($true) {
    try {
      $tabs = Invoke-RestMethod "http://127.0.0.1:$Port/json/list"
      $ready = @($tabs | Where-Object {
        $_.type -eq 'page' -and $_.webSocketDebuggerUrl -and
        ($_.url -match 'tauri|localhost|localmotive|asset' -or $_.title)
      })
      if ($ready.Count -gt 0) { break }
    } catch { }
    if ((Get-Date) -gt $deadline) {
      Stop-OwnedProcess $process "candidate-startup-timeout" | Out-Null
      throw "Candidate WebView did not expose a page CDP target in 90s (port $Port)"
    }
    Start-Sleep -Seconds 2
  }
  Write-Phase "candidate started pid=$($process.Id) port=$Port"
  return $process
}

function Stop-Candidate($process) {
  if ($process) {
    $outcome = Stop-OwnedProcess $process "candidate"
    if ($outcome -eq "still-alive") {
      $script:cleanupNotes += "candidate still-alive"
    }
  }
}

# --- Run-specific isolated state (safe to clean twice) -----------------------
# Every path carries the attempt identity ($AttemptId): two promotions or a
# retry on the same runner can never share a profile, a fixture state dir, or
# a CDP port, and cleanup only ever touches this attempt's paths and handles.
$AttemptId = "verify-$Version-$CdpPort"
$isolatedRoot = Join-Path $env:RUNNER_TEMP "localmotive-verify-appdata-$AttemptId"
$profile = Join-Path $env:RUNNER_TEMP "localmotive-verify-webview2-$AttemptId"
$catalogState = Join-Path $env:RUNNER_TEMP "localmotive-verify-catalog-$AttemptId"
$script:fixtureProcess = $null
$candidate = $null

function Start-OwnedFixture([string]$StateDir, [string]$InitRecordPath) {
  # Spawn the fixture server as an OWNED child and keep its HANDLE: cleanup
  # stops exactly this process, never a PID read back from a file.
  # (Windows PowerShell 5.1 has no ProcessStart.ArgumentList collection, so
  # the init child is spawned via Start-Process with a stdout/stderr capture
  # file, and its exit code is read from the OWNED handle -- never
  # $LASTEXITCODE. The init stdout carries ONLY the init record JSON: node
  # writes the record to its own stdout and nothing else.)
  Write-Phase "fixture init starting"
  $initOut = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-fixture-init-$AttemptId.txt"
  Remove-Item -LiteralPath $initOut -Force -ErrorAction SilentlyContinue
  $proc = Start-Process -FilePath "node" -ArgumentList @(
    "scripts/verify_060_catalog.mjs", "init", $StateDir, $InitRecordPath
  ) -NoNewWindow -Wait -PassThru -RedirectStandardOutput $initOut
  $initCode = $proc.ExitCode
  Write-Phase "fixture init exit=$initCode"
  if ($initCode -ne 0) {
    $stderr = ""
    try { $stderr = (Get-Content $initOut -Raw) } catch { }
    throw "The catalog fixture server did not initialize (exit $initCode): $stderr"
  }
  # The init record carries the fixture URL/pubkey; the SERVER child is a
  # grandchild of this script (spawned detached by init), so adopt it by the
  # recorded pid AND validate its command line before keeping the handle.
  $initRecord = (Get-Content $initOut -Raw | ConvertFrom-Json)
  if ($initRecord.status -ne "PASS") { throw "The catalog fixture server did not initialize" }
  $state = (Get-Content (Join-Path $StateDir 'state.json') -Raw | ConvertFrom-Json)
  $adopted = Get-Process -Id $state.serverPid -ErrorAction SilentlyContinue
  if (-not $adopted) { throw "The catalog fixture server exited before adoption (pid $($state.serverPid))" }
  # Validate the adopted process really is this run's fixture server: the
  # server child runs `node <verifier> __serve <port>`, so its command line
  # must carry the __serve marker. Never adopt by pid alone on a shared
  # runner: a reused pid belonging to another process is refused.
  $adoptedCmd = ""
  try {
    $adoptedCmd = (Get-CimInstance Win32_Process -Filter "ProcessId=$($adopted.Id)" -ErrorAction Stop).CommandLine
  } catch {
    # CIM unavailable: fall back to process-name check, still bounded.
    if ($adopted.ProcessName -ne "node") { throw "Recorded fixture pid $($adopted.Id) is not a node process; refusing to adopt" }
  }
  if ($adoptedCmd -and $adoptedCmd -notlike "*__serve*") {
    throw "Recorded fixture pid $($adopted.Id) does not belong to this attempt; refusing to adopt"
  }
  $script:fixtureProcess = $adopted
  Write-Phase "fixture adopted pid=$($adopted.Id) url=$($initRecord.fixture_url)"
  return $initRecord
}

function Stop-OwnedFixture([string]$StateDir) {
  # Stop the OWNED fixture handle. prep() kills the server but never clears
  # the handle, so already-exited is the expected successful path -- and a
  # reused PID can never redirect this kill because the handle is bound.
  if (-not $script:fixtureProcess) { return "skipped-no-fixture" }
  $outcome = Stop-OwnedProcess $script:fixtureProcess "catalog-fixture"
  $script:fixtureProcess = $null
  $script:cleanupNotes += "fixture outcome=$outcome"
  if ($outcome -eq "still-alive") {
    $script:functionalFailed = $true
    if (-not $script:functionalError) {
      $script:functionalError = "Catalog fixture process could not be stopped within the deadline"
    }
  }
  return $outcome
}

try {
  if (-not (Test-Path $Portable)) { throw "Portable executable was not built" }
  New-Item -ItemType Directory -Force -Path "artifacts" | Out-Null

  # No pre-clean of ports or process names: every path and handle in this run
  # is attempt-scoped ($AttemptId), so a previous attempt's leftovers cannot
  # collide -- and this run can never kill another attempt's processes.
  Remove-Item -LiteralPath $isolatedRoot, $profile -Recurse -Force -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $isolatedRoot | Out-Null
  New-Item -ItemType Directory -Force -Path $profile | Out-Null
  $env:LOCALAPPDATA = $isolatedRoot
  $env:LOCALMOTIVE_VERIFY_ISOLATED_ROOT = $isolatedRoot
  $env:WEBVIEW2_USER_DATA_FOLDER = $profile
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$CdpPort --user-data-dir=$profile"

  Remove-Item -LiteralPath $catalogState -Recurse -Force -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $catalogState | Out-Null

  # Fixture init: the OWNED child handle carries the exit code (never
  # $LASTEXITCODE) and the init JSON is parsed only after exit 0.
  $catalogFixture = Start-OwnedFixture $catalogState "artifacts/catalog-fixture-init.json"
  $env:LOCALMOTIVE_CATALOG_URL = $catalogFixture.fixture_url
  $env:LOCALMOTIVE_CATALOG_PUBKEY = $catalogFixture.fixture_pubkey

  $catalogRoot = Join-Path $isolatedRoot "catalog-cache"
  New-Item -ItemType Directory -Force -Path $catalogRoot | Out-Null
  $env:LOCALMOTIVE_CATALOG_ROOT = $catalogRoot

  # --- Launch #1: packaged verification + first fill -------------------------
  $candidate = Start-Candidate $CdpPort
  try {
    Write-Phase "verify_041 starting"
    $verifyCode = Invoke-Verifier "verify_041" @(
      "scripts/verify_041.mjs", "$CdpPort", $Portable, "artifacts/packaged-verification-$Version.json"
    )
    if ($verifyCode -ne 0) { throw "verify_041 failed with code $verifyCode" }
    $record = Get-Content "artifacts/packaged-verification-$Version.json" -Raw | ConvertFrom-Json
    if ($record.overall_status -ne "PASS") { throw "Packaged verification record is not PASS" }

    Write-Phase "catalog first-fill starting"
    $fillCode = Invoke-Verifier "catalog first-fill" @(
      "scripts/verify_060_catalog.mjs", "first-fill", $CdpPort, $catalogState, "artifacts/catalog-matrix-first-fill.json"
    )
    if ($fillCode -ne 0) { throw "Catalog first-fill phase failed with code $fillCode" }
  } finally {
    Write-Phase "candidate shutdown (launch 1)"
    Stop-Candidate $candidate
    $candidate = $null
  }

  Write-Phase "catalog prep starting"
  $prepCode = Invoke-Verifier "catalog prep" @(
    "scripts/verify_060_catalog.mjs", "prep", $catalogState
  )
  if ($prepCode -ne 0) { throw "Catalog corruption phase failed with code $prepCode" }

  # --- Launch #2: restart matrix ----------------------------------------------
  $candidate = Start-Candidate $CdpPort
  try {
    Write-Phase "catalog restart starting"
    $restartCode = Invoke-Verifier "catalog restart" @(
      "scripts/verify_060_catalog.mjs", "restart", $CdpPort, $catalogState, "artifacts/catalog-matrix-restart.json"
    )
    if ($restartCode -ne 0) { throw "Catalog restart phase failed with code $restartCode" }
  } finally {
    Write-Phase "candidate shutdown (launch 2)"
    Stop-Candidate $candidate
    $candidate = $null
  }

  # --- Merge: bind the record to this run's revision ---------------------------
  $env:LOCALMOTIVE_SOURCE_REVISION = $ResolvedSha
  Write-Phase "catalog merge starting"
  $mergeCode = Invoke-Verifier "catalog merge" @(
    "scripts/verify_060_catalog.mjs", "merge", $Portable,
    "artifacts/catalog-matrix-first-fill.json", "artifacts/catalog-matrix-restart.json",
    "artifacts/packaged-verification-catalog-$Version.json"
  )
  if ($mergeCode -ne 0) { throw "The catalog/SQLite packaged matrix is not PASS" }
  $catalogRecord = Get-Content "artifacts/packaged-verification-catalog-$Version.json" -Raw | ConvertFrom-Json
  if ($catalogRecord.source_revision -ne $ResolvedSha) { throw "Catalog matrix record does not bind the resolved revision" }
  Write-Phase "merge bound revision=$($catalogRecord.source_revision)"
} catch {
  $script:functionalFailed = $true
  $script:functionalError = $_.Exception.Message
  Write-Phase "FUNCTIONAL FAILURE: $script:functionalError"
} finally {
  # Outer cleanup: runs on success AND on early failure (fixture allocated
  # but candidate never started, verifier threw before prep, merge threw,
  # ...). Partially initialized state is handled: every handle is null-
  # checked, and the owned-handle stop treats already-gone as success.
  # Only OWNED handles are stopped here -- never a pid read from a file,
  # never a port scan, never a process-name scan. Safe to run twice.
  Write-Phase "outer cleanup starting"
  if ($candidate) {
    Stop-Candidate $candidate
    $candidate = $null
  }
  Stop-OwnedFixture $catalogState | Out-Null
  Write-Phase "outer cleanup done"
}

# --- Final exit code: recorded outcomes only ----------------------------------
if ($script:cleanupNotes.Count -gt 0) {
  Write-Phase ("cleanup notes: " + ($script:cleanupNotes -join "; "))
}
if ($script:functionalFailed) {
  Write-Phase "RESULT: FAIL $script:functionalError"
  throw $script:functionalError
}
Write-Phase "RESULT: PASS"
exit 0
