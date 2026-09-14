# Packaged-verification orchestration for the Release verify `package` job
# (audit GH-05; runs 34819219725, 34815989537, 34813162097, 34810494640).
#
# This script owns the native exit-code contract the inline workflow block
# kept getting wrong: GitHub's PowerShell runner appends
#   if ((Test-Path -LiteralPath variable:\LASTEXITCODE)) { exit $LASTEXITCODE }
# so the LAST native command in the step decides the step outcome even after
# every verifier passed. A final `taskkill` of an already-dead fixture PID
# (prep() already stopped it) therefore failed green runs with exit 1 and no
# error text, ~282 ms after the merge PASS line.
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

function Write-Phase([string]$Message) {
  $stamp = (Get-Date).ToString("o")
  Write-Host "verify-orchestrator [$stamp] $Message"
}

# Run one native command, capture its exit code immediately, and classify it.
# Already-gone owned processes (taskkill 128) are successful cleanup; anything
# else nonzero is returned to the caller for an explicit decision. Nothing
# here throws, so cleanup can never mask the functional outcome by accident.
function Invoke-Native([string]$Description, [scriptblock]$Command) {
  & $Command | Out-Null
  $code = $LASTEXITCODE
  if ($null -eq $code) { $code = 0 }
  Write-Phase "$Description exit=$code"
  return $code
}

function Stop-OwnedProcess([int]$TargetPid, [string]$Why) {
  if (-not $TargetPid -or $TargetPid -eq 0) { return "skipped-no-pid" }
  $alive = Get-Process -Id $TargetPid -ErrorAction SilentlyContinue
  if (-not $alive) {
    Write-Phase "cleanup $Why pid=$TargetPid already-stopped"
    return "already-stopped"
  }
  $code = Invoke-Native "taskkill $Why pid=$TargetPid" { cmd /c "taskkill /F /PID $TargetPid" 2>$null }
  $deadline = (Get-Date).AddSeconds(15)
  while ((Get-Date) -lt $deadline) {
    if (-not (Get-Process -Id $TargetPid -ErrorAction SilentlyContinue)) {
      Write-Phase "cleanup $Why pid=$TargetPid stopped"
      return "stopped"
    }
    Start-Sleep -Milliseconds 500
  }
  Write-Phase "cleanup $Why pid=$TargetPid STILL-ALIVE"
  return "still-alive"
}

function Stop-VerifyLeftovers([int]$Port) {
  $pids = @()
  try {
    $conns = Get-NetTCPConnection -LocalPort $Port -ErrorAction SilentlyContinue
    if ($conns) { $pids += @($conns | Select-Object -ExpandProperty OwningProcess -Unique) }
  } catch { }
  $pids += @(Get-Process -Name "localmotive", "msedgewebview2" -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*$env:RUNNER_TEMP*" } |
    Select-Object -ExpandProperty Id)
  foreach ($procId in ($pids | Select-Object -Unique)) {
    if ($procId -and $procId -ne 0) {
      $code = Invoke-Native "taskkill leftover pid=$procId" { cmd /c "taskkill /F /T /PID $procId" 2>$null }
      if ($code -ne 0 -and $code -ne 128) {
        $script:cleanupNotes += "leftover pid=$procId exit=$code"
      }
    }
  }
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
      Stop-OwnedProcess -TargetPid $process.Id -Why "candidate-startup-timeout" | Out-Null
      throw "Candidate WebView did not expose a page CDP target in 90s (port $Port)"
    }
    Start-Sleep -Seconds 2
  }
  Write-Phase "candidate started pid=$($process.Id) port=$Port"
  return $process
}

function Stop-Candidate($process) {
  if ($process -and -not $process.HasExited) {
    $outcome = Stop-OwnedProcess -TargetPid $process.Id -Why "candidate"
    if ($outcome -eq "still-alive") {
      $script:cleanupNotes += "candidate pid=$($process.Id) still-alive"
    }
  }
}

# --- Run-specific isolated state (safe to clean twice) -----------------------
$isolatedRoot = Join-Path $env:RUNNER_TEMP "localmotive-verify-appdata"
$profile = Join-Path $env:RUNNER_TEMP "localmotive-verify-webview2"
$catalogState = Join-Path $env:RUNNER_TEMP "localmotive-verify-catalog"
$catalogFixture = $null
$candidate = $null

try {
  if (-not (Test-Path $Portable)) { throw "Portable executable was not built" }
  New-Item -ItemType Directory -Force -Path "artifacts" | Out-Null

  Stop-VerifyLeftovers -Port $CdpPort
  Stop-VerifyLeftovers -Port 10041
  Remove-Item -LiteralPath $isolatedRoot, $profile -Recurse -Force -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $isolatedRoot | Out-Null
  New-Item -ItemType Directory -Force -Path $profile | Out-Null
  $env:LOCALAPPDATA = $isolatedRoot
  $env:LOCALMOTIVE_VERIFY_ISOLATED_ROOT = $isolatedRoot
  $env:WEBVIEW2_USER_DATA_FOLDER = $profile
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$CdpPort --user-data-dir=$profile"

  Remove-Item -LiteralPath $catalogState -Recurse -Force -ErrorAction SilentlyContinue
  New-Item -ItemType Directory -Force -Path $catalogState | Out-Null

  # Fixture init: check the NATIVE exit code before trusting its JSON.
  Write-Phase "fixture init starting"
  $initRaw = node scripts/verify_060_catalog.mjs init $catalogState "artifacts/catalog-fixture-init.json"
  $initCode = $LASTEXITCODE
  if ($null -eq $initCode) { $initCode = 0 }
  Write-Phase "fixture init exit=$initCode"
  if ($initCode -ne 0) { throw "The catalog fixture server did not initialize (exit $initCode)" }
  $catalogFixture = ($initRaw | ConvertFrom-Json)
  if ($catalogFixture.status -ne "PASS") { throw "The catalog fixture server did not initialize" }
  Write-Phase "fixture init pid=$($catalogFixture.serverPid) url=$($catalogFixture.fixture_url)"
  $env:LOCALMOTIVE_CATALOG_URL = $catalogFixture.fixture_url
  $env:LOCALMOTIVE_CATALOG_PUBKEY = $catalogFixture.fixture_pubkey

  $catalogRoot = Join-Path $isolatedRoot "catalog-cache"
  New-Item -ItemType Directory -Force -Path $catalogRoot | Out-Null
  $env:LOCALMOTIVE_CATALOG_ROOT = $catalogRoot

  # --- Launch #1: packaged verification + first fill -------------------------
  $candidate = Start-Candidate $CdpPort
  try {
    Write-Phase "verify_041 starting"
    node scripts/verify_041.mjs $CdpPort $Portable "artifacts/packaged-verification-$Version.json"
    $verifyCode = $LASTEXITCODE
    if ($null -eq $verifyCode) { $verifyCode = 0 }
    Write-Phase "verify_041 exit=$verifyCode"
    if ($verifyCode -ne 0) { throw "verify_041 failed with code $verifyCode" }
    $record = Get-Content "artifacts/packaged-verification-$Version.json" -Raw | ConvertFrom-Json
    if ($record.overall_status -ne "PASS") { throw "Packaged verification record is not PASS" }

    Write-Phase "catalog first-fill starting"
    node scripts/verify_060_catalog.mjs first-fill $CdpPort $catalogState "artifacts/catalog-matrix-first-fill.json"
    $fillCode = $LASTEXITCODE
    if ($null -eq $fillCode) { $fillCode = 0 }
    Write-Phase "catalog first-fill exit=$fillCode"
    if ($fillCode -ne 0) { throw "Catalog first-fill phase failed with code $fillCode" }
  } finally {
    Write-Phase "candidate shutdown (launch 1)"
    Stop-Candidate $candidate
    $candidate = $null
  }

  Write-Phase "catalog prep starting"
  node scripts/verify_060_catalog.mjs prep $catalogState
  $prepCode = $LASTEXITCODE
  if ($null -eq $prepCode) { $prepCode = 0 }
  Write-Phase "catalog prep exit=$prepCode"
  if ($prepCode -ne 0) { throw "Catalog corruption phase failed with code $prepCode" }

  # --- Launch #2: restart matrix ----------------------------------------------
  $candidate = Start-Candidate $CdpPort
  try {
    Write-Phase "catalog restart starting"
    node scripts/verify_060_catalog.mjs restart $CdpPort $catalogState "artifacts/catalog-matrix-restart.json"
    $restartCode = $LASTEXITCODE
    if ($null -eq $restartCode) { $restartCode = 0 }
    Write-Phase "catalog restart exit=$restartCode"
    if ($restartCode -ne 0) { throw "Catalog restart phase failed with code $restartCode" }
  } finally {
    Write-Phase "candidate shutdown (launch 2)"
    Stop-Candidate $candidate
    $candidate = $null
  }

  # --- Merge: bind the record to this run's revision ---------------------------
  $env:LOCALMOTIVE_SOURCE_REVISION = $ResolvedSha
  Write-Phase "catalog merge starting"
  node scripts/verify_060_catalog.mjs merge $Portable "artifacts/catalog-matrix-first-fill.json" "artifacts/catalog-matrix-restart.json" "artifacts/packaged-verification-catalog-$Version.json"
  $mergeCode = $LASTEXITCODE
  if ($null -eq $mergeCode) { $mergeCode = 0 }
  Write-Phase "catalog merge exit=$mergeCode"
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
  # checked, and Stop-OwnedProcess treats already-gone as success.
  Write-Phase "outer cleanup starting"
  if ($candidate) {
    Stop-Candidate $candidate
    $candidate = $null
  }
  if ($catalogFixture) {
    $fixturePid = 0
    try {
      # Re-read the recorded pid: prep() stops the server but leaves the pid
      # in the state file, so the live value may be stale on purpose.
      $fixturePid = (Get-Content (Join-Path $catalogState 'state.json') -Raw | ConvertFrom-Json).serverPid
    } catch { }
    # Ownership check: only stop a node process whose command line carries
    # this run's fixture marker. A bare pid may have been reused by an
    # unrelated process on this shared runner; never kill by pid alone.
    $owned = $false
    if ($fixturePid) {
      $proc = Get-Process -Id $fixturePid -ErrorAction SilentlyContinue
      if ($proc) {
        try {
          $cmd = (Get-CimInstance Win32_Process -Filter "ProcessId=$fixturePid" -ErrorAction Stop).CommandLine
          if ($cmd -like "*verify_060_catalog.mjs*__serve*") { $owned = $true }
          else { $script:cleanupNotes += "fixture pid=$fixturePid command mismatch; not killed" }
        } catch {
          # CIM unavailable: fall back to process-name check, still bounded.
          if ($proc.ProcessName -eq "node") { $owned = $true }
          else { $script:cleanupNotes += "fixture pid=$fixturePid name=$($proc.ProcessName); not killed" }
        }
      } else {
        $script:cleanupNotes += "fixture pid=$fixturePid already-stopped"
      }
    }
    if ($owned) {
      $outcome = Stop-OwnedProcess -TargetPid $fixturePid -Why "catalog-fixture"
      $script:cleanupNotes += "fixture pid=$fixturePid outcome=$outcome"
      if ($outcome -eq "still-alive") {
        $script:functionalFailed = $true
        if (-not $script:functionalError) {
          $script:functionalError = "Catalog fixture process $fixturePid could not be stopped within the deadline"
        }
      }
    }
  }
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
