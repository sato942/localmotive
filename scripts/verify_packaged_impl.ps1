# Packaged-verification function library for the Release verify `package` job.
#
# This file holds every function `scripts/verify_packaged_matrix.ps1` uses.
# It contains NO top-level executable state: no param() consumers, no
# Join-Path on $env:RUNNER_TEMP, no try/main body, no exit. That lets the
# cleanup acceptance matrix (scripts/tests/verify_cleanup_matrix.ps1)
# dot-source this file directly under pwsh and drive the REAL functions
# (owned handles, real owned node sleepers, real final-exit decisions)
# instead of copying predicates or grepping source text.
#
# The wrapper script sets the run identities ($AttemptId etc.), then calls
# Invoke-PackagedMatrix, then maps its return code to `exit`.
#
# Revision 3 (U06-01): candidate still-alive fails the run while preserving
# an earlier verifier error; fixture adoption refuses unverified identity
# (CIM failure or missing/foreign command line never adopts); every
# Invoke-Verifier call carries the attempt identity explicitly.

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
# The attempt identity is an explicit parameter so this function never
# depends on wrapper-level run state.
function Invoke-Verifier([string]$Description, [string[]]$Arguments, [string]$AttemptId) {
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

function Start-Candidate([int]$Port, [string]$Portable) {
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
      # A live owned candidate after its deadline is a cleanup failure that
      # fails the run. An earlier verifier error is preserved as the prefix
      # so the original failure is never replaced by the cleanup note.
      $script:functionalFailed = $true
      if (-not $script:functionalError) {
        $script:functionalError = "Candidate process could not be stopped within the deadline"
      } else {
        $script:functionalError = "$($script:functionalError); candidate still-alive after cleanup deadline"
      }
      $script:cleanupNotes += "candidate still-alive"
    }
  }
}

function Get-FixtureAdoption([int]$RecordedPid) {
  # Validate that a recorded fixture PID really is this run's fixture server
  # and return the adopted process. Refuses (throws) rather than adopting an
  # unrelated process: a reused PID, a foreign command line, an unreadable
  # command line, or a failed identity lookup all refuse. A CIM exception is
  # never proof the process exited and never a license to accept by name.
  $adopted = Get-Process -Id $RecordedPid -ErrorAction SilentlyContinue
  if (-not $adopted) { throw "The catalog fixture server exited before adoption (pid $RecordedPid)" }
  $adoptedCmd = ""
  $cimError = ""
  try {
    $adoptedCmd = (Get-CimInstance Win32_Process -Filter "ProcessId=$($adopted.Id)" -ErrorAction Stop).CommandLine
  } catch {
    $cimError = $_.Exception.Message
    Write-Phase "fixture adoption CIM lookup failed for pid $($adopted.Id): $cimError"
  }
  if ($cimError) {
    throw "Recorded fixture pid $($adopted.Id) identity is unverified (CIM lookup failed: $cimError); refusing to adopt"
  }
  if (-not $adoptedCmd) {
    throw "Recorded fixture pid $($adopted.Id) has no readable command line; refusing to adopt"
  }
  if ($adoptedCmd -notlike "*__serve*") {
    throw "Recorded fixture pid $($adopted.Id) does not belong to this attempt; refusing to adopt"
  }
  return $adopted
}

function Start-OwnedFixture([string]$StateDir, [string]$InitRecordPath, [string]$AttemptId) {
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
  Remove-Item -LiteralPath $InitRecordPath -Force -ErrorAction SilentlyContinue
  # Spawn the init child fire-and-forget: it writes the init record file
  # itself (verify_060_catalog.mjs init() writes outPath on success) and
  # spawns the detached grandchild server, then exits. Never wait on the
  # child handle: Start-Process -Wait, Process.WaitForExit, and
  # Start-Job/Wait-Job all hang in this environment even after init
  # completes its work (observed 2026-09-14). Poll the record file with
  # a deadline instead; adopt the server by pid plus verified command
  # line, exactly as before.
  Start-Process -FilePath "node" -ArgumentList @(
    "scripts/verify_060_catalog.mjs", "init", $StateDir, $InitRecordPath
  ) -NoNewWindow | Out-Null
  $deadline = (Get-Date).AddSeconds(60)
  $initRecord = $null
  while ((Get-Date) -lt $deadline) {
    if (Test-Path -LiteralPath $InitRecordPath) {
      try { $initRecord = (Get-Content $InitRecordPath -Raw | ConvertFrom-Json) } catch { $initRecord = $null }
      if ($initRecord -and $initRecord.status -eq "PASS" -and $initRecord.fixture_url) { break }
      $initRecord = $null
    }
    Start-Sleep -Milliseconds 500
  }
  if (-not $initRecord) {
    throw "The catalog fixture server did not initialize (no PASS init record at $InitRecordPath within 60s)"
  }
  $initCode = 0
  Write-Phase "fixture init exit=$initCode"
  # Mirror the record to the legacy stdout-capture path so downstream
  # readers of $initOut keep working.
  try { Copy-Item -LiteralPath $InitRecordPath -Destination $initOut -Force } catch { }
  # The init record carries the fixture URL/pubkey; the SERVER child is a
  # grandchild of this script (spawned detached by init), so adopt it by the
  # recorded pid AND a verified command line before keeping the handle.
  $state = (Get-Content (Join-Path $StateDir 'state.json') -Raw | ConvertFrom-Json)
  $script:fixtureProcess = Get-FixtureAdoption $state.serverPid
  Write-Phase "fixture adopted pid=$($script:fixtureProcess.Id) url=$($initRecord.fixture_url)"
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

function Invoke-PackagedMatrix(
  [string]$Portable,
  [string]$Version,
  [string]$ResolvedSha,
  [int]$CdpPort,
  [string]$AttemptId,
  [string]$IsolatedRoot,
  [string]$Profile,
  [string]$CatalogState,
  [scriptblock]$StartFixture = $null,
  [scriptblock]$StartCandidateFn = $null,
  [scriptblock]$RunVerifier = $null,
  [scriptblock]$StopCandidateFn = $null,
  [scriptblock]$StopFixtureFn = $null,
  [hashtable]$Outcome = $null
) {
  # The full orchestration: fixture init, two candidate launches with their
  # verifier phases, prep, merge, and the outer cleanup that covers partial
  # initialization. Returns 0 on PASS, 1 on FAIL. The five hook parameters
  # default to the production functions above; the acceptance matrix injects
  # scriptblocks to drive failure paths through this REAL decision path.
  # Hook scriptblocks created with GetNewClosure() cannot write back to the
  # caller's $script: scope, so a hook may take the shared $Outcome probe as
  # its last parameter; after every hook call the probe merges into the real
  # outcome state (production hooks ignore the extra argument).
  # Merge-Probe copies a hook-recorded outcome into the real outcome state.
  function Merge-Probe([hashtable]$Probe) {
    if (-not $Probe) { return }
    if ($Probe.notes -and $Probe.notes.Count -gt 0) {
      foreach ($note in $Probe.notes) { $script:cleanupNotes += $note }
      $Probe.notes = @()
    }
    if ($Probe.failed) {
      $script:functionalFailed = $true
      if ($Probe.error -and -not $script:functionalError) {
        $script:functionalError = $Probe.error
      } elseif ($Probe.error -and $script:functionalError -notlike "*$($Probe.error)*") {
        $script:functionalError = "$($script:functionalError); $($Probe.error)"
      }
      $Probe.failed = $false
      $Probe.error = ""
    }
  }
  $doStartFixture = if ($StartFixture) { $StartFixture } else { { param($d, $p, $a) Start-OwnedFixture $d $p $a } }
  $doStartCandidate = if ($StartCandidateFn) { $StartCandidateFn } else { { param($port, $exe) Start-Candidate $port $exe } }
  $doRunVerifier = if ($RunVerifier) { $RunVerifier } else { { param($desc, $argv, $a) Invoke-Verifier $desc $argv $a } }
  $doStopCandidate = if ($StopCandidateFn) { $StopCandidateFn } else { { param($proc) Stop-Candidate $proc } }
  $doStopFixture = if ($StopFixtureFn) { $StopFixtureFn } else { { param($dir) Stop-OwnedFixture $dir } }
  $candidate = $null
  try {
    if (-not (Test-Path $Portable)) { throw "Portable executable was not built" }
    New-Item -ItemType Directory -Force -Path "artifacts" | Out-Null

    # No pre-clean of ports or process names: every path and handle in this run
    # is attempt-scoped ($AttemptId), so a previous attempt's leftovers cannot
    # collide -- and this run can never kill another attempt's processes.
    Remove-Item -LiteralPath $IsolatedRoot, $Profile -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $IsolatedRoot | Out-Null
    New-Item -ItemType Directory -Force -Path $Profile | Out-Null
    $env:LOCALAPPDATA = $IsolatedRoot
    $env:LOCALMOTIVE_VERIFY_ISOLATED_ROOT = $IsolatedRoot
    $env:WEBVIEW2_USER_DATA_FOLDER = $Profile
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$CdpPort --user-data-dir=$Profile"

    Remove-Item -LiteralPath $CatalogState -Recurse -Force -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path $CatalogState | Out-Null

    # Fixture init: the OWNED child handle carries the exit code (never
    # $LASTEXITCODE) and the init JSON is parsed only after exit 0.
    $catalogFixture = & $doStartFixture $CatalogState "artifacts/catalog-fixture-init.json" $AttemptId
    $env:LOCALMOTIVE_CATALOG_URL = $catalogFixture.fixture_url
    $env:LOCALMOTIVE_CATALOG_PUBKEY = $catalogFixture.fixture_pubkey

    $catalogRoot = Join-Path $IsolatedRoot "catalog-cache"
    New-Item -ItemType Directory -Force -Path $catalogRoot | Out-Null
    $env:LOCALMOTIVE_CATALOG_ROOT = $catalogRoot

    # --- Launch #1: packaged verification + first fill -------------------------
    $candidate = & $doStartCandidate $CdpPort $Portable
    try {
      Write-Phase "verify_041 starting"
      $verifyCode = & $doRunVerifier "verify_041" @(
        "scripts/verify_041.mjs", "$CdpPort", $Portable, "artifacts/packaged-verification-$Version.json"
      ) $AttemptId
      if ($verifyCode -ne 0) { throw "verify_041 failed with code $verifyCode" }
      $record = Get-Content "artifacts/packaged-verification-$Version.json" -Raw | ConvertFrom-Json
      if ($record.overall_status -ne "PASS") { throw "Packaged verification record is not PASS" }

      Write-Phase "catalog first-fill starting"
      $fillCode = & $doRunVerifier "catalog first-fill" @(
        "scripts/verify_060_catalog.mjs", "first-fill", $CdpPort, $CatalogState, "artifacts/catalog-matrix-first-fill.json"
      ) $AttemptId
      if ($fillCode -ne 0) { throw "Catalog first-fill phase failed with code $fillCode" }
    } finally {
      Write-Phase "candidate shutdown (launch 1)"
      & $doStopCandidate $candidate $Outcome
      Merge-Probe $Outcome
      $candidate = $null
    }

    Write-Phase "catalog prep starting"
    $prepCode = & $doRunVerifier "catalog prep" @(
      "scripts/verify_060_catalog.mjs", "prep", $CatalogState
    ) $AttemptId
    if ($prepCode -ne 0) { throw "Catalog corruption phase failed with code $prepCode" }

    # --- Launch #2: restart matrix ----------------------------------------------
    $candidate = & $doStartCandidate $CdpPort $Portable
    try {
      Write-Phase "catalog restart starting"
      $restartCode = & $doRunVerifier "catalog restart" @(
        "scripts/verify_060_catalog.mjs", "restart", $CdpPort, $CatalogState, "artifacts/catalog-matrix-restart.json"
      ) $AttemptId
      if ($restartCode -ne 0) { throw "Catalog restart phase failed with code $restartCode" }
    } finally {
      Write-Phase "candidate shutdown (launch 2)"
      & $doStopCandidate $candidate $Outcome
      Merge-Probe $Outcome
      $candidate = $null
    }

    # --- Merge: bind the record to this run's revision ---------------------------
    $env:LOCALMOTIVE_SOURCE_REVISION = $ResolvedSha
    Write-Phase "catalog merge starting"
    $mergeCode = & $doRunVerifier "catalog merge" @(
      "scripts/verify_060_catalog.mjs", "merge", $Portable,
      "artifacts/catalog-matrix-first-fill.json", "artifacts/catalog-matrix-restart.json",
      "artifacts/packaged-verification-catalog-$Version.json"
    ) $AttemptId
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
      & $doStopCandidate $candidate $Outcome
      Merge-Probe $Outcome
      $candidate = $null
    }
    & $doStopFixture $CatalogState $Outcome | Out-Null
    Merge-Probe $Outcome
    Write-Phase "outer cleanup done"
  }

  # --- Final exit code: recorded outcomes only ----------------------------------
  if ($script:cleanupNotes.Count -gt 0) {
    Write-Phase ("cleanup notes: " + ($script:cleanupNotes -join "; "))
  }
  if ($script:functionalFailed) {
    Write-Phase "RESULT: FAIL $script:functionalError"
    return 1
  }
  Write-Phase "RESULT: PASS"
  return 0
}
