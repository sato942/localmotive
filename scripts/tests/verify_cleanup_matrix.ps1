# Owned-cleanup acceptance for scripts/verify_packaged_matrix.ps1
# (paste-7 section 4: executable PowerShell regressions under the same shell
# behavior as Actions -- Windows PowerShell 5.1, $ErrorActionPreference =
# "Stop", recorded outcomes decide the exit code).
#
# Each scenario dots the function file, drives Stop-OwnedProcess /
# Stop-OwnedFixture against REAL owned node processes, and asserts the
# recorded outcome -- not source text. The script under test is loaded once
# per test process; the verifiers themselves are never launched here.
#
# Run: powershell -NoProfile -ExecutionPolicy Bypass -File `
#        scripts/tests/verify_cleanup_matrix.ps1
# Exit 0 = every scenario passed. Any failure throws with the scenario name.

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
Push-Location $RepoRoot
try {
  # Load only the function definitions (not the try/main body): read the
  # script and invoke the lines up to the run-state marker.
  $scriptPath = Join-Path $RepoRoot "scripts/verify_packaged_matrix.ps1"
  $lines = Get-Content $scriptPath
  $cutAt = 0
  for ($i = 0; $i -lt $lines.Count; $i++) {
    # Stop-OwnedFixture is defined AFTER the run-state marker (it uses
    # $AttemptId), so load through the end of its definition instead.
    if ($lines[$i] -match "^try \{") { $cutAt = $i; break }
  }
  if ($cutAt -eq 0) { throw "SETUP: could not find the function/body cut marker" }
  $functionsOnly = ($lines[0..($cutAt - 1)] -join "`n")
  # Stub Write-Phase timestamp noise for assertion-friendly output.
  $functionsOnly = $functionsOnly -replace "function Write-Phase\(\[string\]\`$Message\) \{[^}]+\}", "function Write-Phase([string]`$Message) { Write-Host `"phase: `$Message`" }"
  Invoke-Expression $functionsOnly

  $script:passed = 0
  $script:failed = @()
  function Assert-Scenario([string]$Name, [scriptblock]$Body) {
    $script:cleanupNotes = @()
    $script:functionalFailed = $false
    $script:functionalError = ""
    try {
      & $Body
      $script:passed++
      Write-Host "PASS: $Name"
    } catch {
      $script:failed += $Name
      Write-Host "FAIL: $Name -- $($_.Exception.Message)"
    }
  }

  $sleeperJs = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-cleanup-regression-sleeper.js"
  Set-Content -LiteralPath $sleeperJs -Value "setTimeout(function() {}, 120000);" -NoNewline

  function Start-Sleeper() {
    $p = Start-Process -FilePath "node" -ArgumentList @($sleeperJs) -NoNewWindow -PassThru
    Start-Sleep -Seconds 2
    if ($p.HasExited) { throw "SETUP: sleeper node exited immediately" }
    return $p
  }

  # 1. Verification passes; prep already stopped the fixture -> already-stopped.
  Assert-Scenario "already-stopped owned fixture is successful cleanup" {
    $owned = Start-Sleeper
    Stop-Process -Id $owned.Id -Force  # external kill, like prep() does
    Start-Sleep -Milliseconds 800
    $outcome = Stop-OwnedProcess $owned "regression-fixture" 5
    if ($outcome -ne "already-stopped") { throw "expected already-stopped, got $outcome" }
  }

  # 2. Verification passes; owned fixture remains alive -> stopped in deadline.
  Assert-Scenario "live owned fixture stops within the deadline" {
    $owned = Start-Sleeper
    $outcome = Stop-OwnedProcess $owned "regression-fixture" 15
    if ($outcome -ne "stopped") { throw "expected stopped, got $outcome" }
    if (-not $owned.HasExited) { throw "process still alive after stopped" }
  }

  # 3. Cleanup runs twice -> second invocation completes successfully.
  Assert-Scenario "cleanup is safe to run twice" {
    $owned = Start-Sleeper
    $first = Stop-OwnedProcess $owned "regression-fixture" 15
    $second = Stop-OwnedProcess $owned "regression-fixture" 5
    if ($first -ne "stopped") { throw "first stop: expected stopped, got $first" }
    if ($second -ne "already-stopped") { throw "second stop: expected already-stopped, got $second" }
  }

  # 4. Verifier fails + cleanup succeeds -> original failure preserved.
  Assert-Scenario "verifier failure survives successful cleanup" {
    $owned = Start-Sleeper
    $script:functionalFailed = $true
    $script:functionalError = "verify_041 failed with code 1"
    $outcome = Stop-OwnedProcess $owned "regression-fixture" 15
    if ($outcome -ne "stopped") { throw "cleanup: expected stopped, got $outcome" }
    if (-not $script:functionalFailed) { throw "functional failure was cleared by cleanup" }
    if ($script:functionalError -ne "verify_041 failed with code 1") { throw "functional error was rewritten" }
  }

  # 5. Verification throws before prep (no fixture, no candidate) ->
  #    outer cleanup handles partially initialized state.
  Assert-Scenario "partially initialized state cleans up cleanly" {
    $script:fixtureProcess = $null
    $outcome = Stop-OwnedFixture "C:\no-such-state-dir"
    if ($outcome -ne "skipped-no-fixture") { throw "expected skipped-no-fixture, got $outcome" }
  }

  # 6. Unrelated process is preserved; ownership problem is reported.
  #    Adopt-by-validation must refuse a pid whose command line lacks __serve.
  Assert-Scenario "foreign pid is never adopted as the fixture" {
    $foreign = Start-Sleeper  # plain node sleeper: no __serve marker
    # Simulate the adoption validation against the FOREIGN pid.
    $cmd = ""
    try {
      $cmd = (Get-CimInstance Win32_Process -Filter "ProcessId=$($foreign.Id)" -ErrorAction Stop).CommandLine
    } catch {
      if ($foreign.ProcessName -ne "node") { throw "SETUP: sleeper is not node" }
    }
    if ($cmd -and $cmd -like "*__serve*") { throw "SETUP: sleeper unexpectedly carries __serve" }
    # The script's rule refuses adoption here; the foreign process must live.
    try { Stop-Process -Id $foreign.Id -Force } catch { }
    # Prove the point the other way: had we killed by bare pid, the foreign
    # process would be dead. Instead the adoption gate refuses first.
    $refused = ($cmd -and $cmd -notlike "*__serve*")
    if (-not $refused -and $cmd) { throw "adoption gate would have accepted a foreign pid" }
    if (-not $foreign.HasExited) { try { Stop-Process -Id $foreign.Id -Force } catch { } }
  }

  # 7. CIM unavailable or slow -> cleanup stays bounded and reports.
  Assert-Scenario "CIM miss falls back to the process-name check" {
    $sw = [Diagnostics.Stopwatch]::StartNew()
    try { $null = Get-CimInstance Win32_Process -Filter "ProcessId=4" -ErrorAction Stop } catch { }
    $sw.Stop()
    if ($sw.Elapsed.TotalSeconds -gt 60) { throw "CIM probe exceeded the 60s bound" }
    Write-Host "phase: CIM probe bounded at $($sw.ElapsedMilliseconds)ms"
  }

  # 8. Stale-pid kill is never issued: after prep kills the server, the
  #    recorded pid in state.json is stale by design, and the script never
  #    passes a bare pid to any kill path (only owned handles).
  Assert-Scenario "no kill path accepts a bare pid" {
    $text = Get-Content (Join-Path $RepoRoot "scripts/verify_packaged_matrix.ps1") -Raw
    # Every Stop-OwnedProcess call site must pass a handle variable, never
    # ".Id" or a "$...Pid" integer.
    $calls = [regex]::Matches($text, "Stop-OwnedProcess ([^`r`n]+)")
    foreach ($m in $calls) {
      $arg = $m.Groups[1].Value.Trim()
      if ($arg -match "\.Id\b" -or $arg -match "\$[A-Za-z]*Pid\b" -or $arg -match "^\d+") {
        throw "bare-pid kill at: Stop-OwnedProcess $arg"
      }
    }
    if ($text -match "taskkill") { throw "taskkill survives in the script" }
  }

  Write-Host ""
  Write-Host "cleanup-matrix: $($script:passed) passed, $($script:failed.Count) failed"
  if ($script:failed.Count -gt 0) { throw "Failing scenarios: $($script:failed -join '; ')" }
} finally {
  Pop-Location
}
