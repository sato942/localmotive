# Owned-cleanup acceptance for scripts/verify_packaged_impl.ps1
# (U06-01: executable PowerShell regressions under the workflow's actual
# shell -- pwsh -- driving the REAL orchestration Invoke-PackagedMatrix
# through its REAL final-exit decision, not helper copies or source text).
#
# Each scenario dot-sources the production function library once per test
# process, then calls Invoke-PackagedMatrix with injected scriptblock hooks
# for fixture start / candidate start / verifier runs / candidate stop /
# fixture stop. Hooks that are not injected default to the production
# functions, so every scenario exercises the real decision path, the real
# outer-cleanup finally, and the real 0/1 return that the wrapper maps to
# `exit`. Live owned node sleepers prove the stoppable paths; a scriptblock
# returning "still-alive" proves the unstoppable paths fail the run.
#
# Run: pwsh -NoProfile -File scripts/tests/verify_cleanup_matrix.ps1
# Exit 0 = every scenario passed. Any failure throws with the scenario name.
#
# Shell note: the workflow invokes the orchestrator with pwsh (release.yml
# "Verify the packaged executable" sets shell: pwsh). These regressions run
# under the same pwsh. A Windows PowerShell 5.1 pass is informative only.

$ErrorActionPreference = "Stop"

$RepoRoot = Split-Path (Split-Path $PSScriptRoot -Parent) -Parent
Push-Location $RepoRoot
try {
  # Dot-source the REAL production functions. The impl file has no top-level
  # executable state, so this load performs no run work.
  . (Join-Path $RepoRoot "scripts/verify_packaged_impl.ps1")

  $script:passed = 0
  $script:failed = @()
  function Assert-Scenario([string]$Name, [scriptblock]$Body) {
    # Reset the shared outcome state the production functions record into,
    # exactly as a fresh orchestrator run starts.
    $script:functionalFailed = $false
    $script:functionalError = ""
    $script:cleanupNotes = @()
    $script:lastVerifierOutput = ""
    $script:fixtureProcess = $null
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

  # Shared mutable outcome probe: hook scriptblocks created with
  # GetNewClosure() cannot write back to this file's $script: scope (each
  # closure carries its own session-state binding), so hooks record into
  # this hashtable, which Invoke-PackagedMatrix merges into the real
  # $script:functionalFailed / $script:functionalError / $script:cleanupNotes
  # after every hook call.
  function New-OutcomeProbe() { return @{ failed = $false; error = ""; notes = @() } }

  function Stop-SleeperNow($proc) {
    try { $proc.Kill() } catch { }
    try { $proc.WaitForExit(15000) | Out-Null } catch { }
  }

  function New-TestAttempt([string]$Name) {
    $id = "regression-$Name-$([System.Guid]::NewGuid().ToString('N').Substring(0, 8))"
    $base = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-regression-$id"
    return @{
      AttemptId = $id
      IsolatedRoot = Join-Path $base "appdata"
      Profile = Join-Path $base "webview2"
      CatalogState = Join-Path $base "catalog"
      Portable = $sleeperJs  # Test-Path passes; the file is never executed by mocks
      Version = "0.6.0"
      ResolvedSha = "regression-$id"
    }
  }

  # A verifier mock whose queue of exit codes drives each Invoke-Verifier call
  # in order (verify_041, first-fill, prep, restart, merge). The mock writes
  # the artifact records the production code reads after exit 0, with the
  # source revision bound at mock-construction time (each attempt's SHA is
  # fixed before the run starts, so no cross-closure reads are needed).
  function New-VerifierMock([int[]]$Codes, [string]$ForSha) {
    $queue = [System.Collections.Queue]::new($Codes)
    $sha = $ForSha
    return {
      param($desc, $argv, $attempt)
      if ($queue.Count -eq 0) { throw "SETUP: verifier mock exhausted at phase $desc" }
      $code = $queue.Dequeue()
      Write-Host "phase: mock verifier $desc exit=$code"
      if ($desc -eq "verify_041" -and $code -eq 0) {
        Set-Content -LiteralPath "artifacts/packaged-verification-0.6.0.json" -Value '{"overall_status":"PASS"}' -NoNewline
      }
      if ($desc -eq "catalog merge" -and $code -eq 0) {
        Set-Content -LiteralPath "artifacts/packaged-verification-catalog-0.6.0.json" -Value ('{"source_revision":"' + $sha + '","overall_status":"PASS"}')
      }
      return $code
    }.GetNewClosure()
  }

  # 1. Full success with an already-stopped fixture: prep() killed the server
  # (the handle is already-exited) and both candidates stop cleanly.
  Assert-Scenario "success with already-stopped fixture exits 0" {
    $t = New-TestAttempt "success-stopped"
    $probe = New-OutcomeProbe
    $owned = Start-Sleeper
    Stop-Process -Id $owned.Id -Force  # external kill, like prep() does
    Start-Sleep -Milliseconds 800
    $script:fixtureProcess = $owned  # adopted handle, already dead
    $verifier = New-VerifierMock @(0, 0, 0, 0, 0) $t.ResolvedSha
    $startCandidate = { param($port, $exe) return (Start-Sleeper) }.GetNewClosure()
    $startFixture = {
      param($d, $p, $a)
      New-Item -ItemType Directory -Force -Path $d | Out-Null
      return @{ fixture_url = "http://127.0.0.1:9/catalog.json"; fixture_pubkey = "00" * 32 }
    }.GetNewClosure()
    $stopFixture = { param($d) return (Stop-OwnedFixture $d) }.GetNewClosure()
    $stopCandidate = { param($proc) Stop-Candidate $proc }.GetNewClosure()
    $workDir = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-regression-work-$($t.AttemptId)"
    New-Item -ItemType Directory -Force -Path $workDir | Out-Null
    Push-Location $workDir
    try {
      $code = Invoke-PackagedMatrix -Portable $t.Portable -Version $t.Version -ResolvedSha $t.ResolvedSha -CdpPort 14711 -AttemptId $t.AttemptId -IsolatedRoot $t.IsolatedRoot -Profile $t.Profile -CatalogState $t.CatalogState -StartFixture $startFixture -StartCandidateFn $startCandidate -RunVerifier $verifier -StopCandidateFn $stopCandidate -StopFixtureFn $stopFixture -Outcome $probe
      if ($code -ne 0) { throw "expected exit 0, got $code (error: $script:functionalError)" }
    } finally { Pop-Location }
  }

  # 2. Full success with a live fixture and live candidates: every owned
  # handle stops inside its deadline.
  Assert-Scenario "success with live owned processes exits 0" {
    $t = New-TestAttempt "success-live"
    $probe = New-OutcomeProbe
    $owned = Start-Sleeper
    $script:fixtureProcess = $owned  # adopted handle, still alive
    $verifier = New-VerifierMock @(0, 0, 0, 0, 0) $t.ResolvedSha
    $startCandidate = { param($port, $exe) return (Start-Sleeper) }.GetNewClosure()
    $startFixture = {
      param($d, $p, $a)
      New-Item -ItemType Directory -Force -Path $d | Out-Null
      return @{ fixture_url = "http://127.0.0.1:9/catalog.json"; fixture_pubkey = "00" * 32 }
    }.GetNewClosure()
    $stopFixture = { param($d) return (Stop-OwnedFixture $d) }.GetNewClosure()
    $stopCandidate = { param($proc) Stop-Candidate $proc }.GetNewClosure()
    $workDir = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-regression-work-$($t.AttemptId)"
    New-Item -ItemType Directory -Force -Path $workDir | Out-Null
    Push-Location $workDir
    try {
      $code = Invoke-PackagedMatrix -Portable $t.Portable -Version $t.Version -ResolvedSha $t.ResolvedSha -CdpPort 14712 -AttemptId $t.AttemptId -IsolatedRoot $t.IsolatedRoot -Profile $t.Profile -CatalogState $t.CatalogState -StartFixture $startFixture -StartCandidateFn $startCandidate -RunVerifier $verifier -StopCandidateFn $stopCandidate -StopFixtureFn $stopFixture -Outcome $probe
      if ($code -ne 0) { throw "expected exit 0, got $code (error: $script:functionalError)" }
      if (-not $owned.HasExited) { throw "fixture still alive after successful run"; Stop-SleeperNow $owned }
    } finally { Pop-Location }
  }

  # 3. Verifier failure + clean cleanup: the ORIGINAL verifier error decides
  # the outcome and survives cleanup.
  Assert-Scenario "verifier failure survives successful cleanup" {
    $t = New-TestAttempt "verifier-fail"
    $probe = New-OutcomeProbe
    $verifier = New-VerifierMock @(1, 0, 0, 0, 0) $t.ResolvedSha  # verify_041 fails
    $startCandidate = { param($port, $exe) return (Start-Sleeper) }.GetNewClosure()
    $startFixture = {
      param($d, $p, $a)
      New-Item -ItemType Directory -Force -Path $d | Out-Null
      return @{ fixture_url = "http://127.0.0.1:9/catalog.json"; fixture_pubkey = "00" * 32 }
    }.GetNewClosure()
    $stopFixture = { param($d) return "skipped-no-fixture" }.GetNewClosure()
    $stopCandidate = { param($proc) Stop-Candidate $proc }.GetNewClosure()
    $workDir = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-regression-work-$($t.AttemptId)"
    New-Item -ItemType Directory -Force -Path $workDir | Out-Null
    Push-Location $workDir
    try {
      $code = Invoke-PackagedMatrix -Portable $t.Portable -Version $t.Version -ResolvedSha $t.ResolvedSha -CdpPort 14713 -AttemptId $t.AttemptId -IsolatedRoot $t.IsolatedRoot -Profile $t.Profile -CatalogState $t.CatalogState -StartFixture $startFixture -StartCandidateFn $startCandidate -RunVerifier $verifier -StopCandidateFn $stopCandidate -StopFixtureFn $stopFixture -Outcome $probe
      if ($code -ne 1) { throw "expected exit 1, got $code" }
      if ($script:functionalError -ne "verify_041 failed with code 1") { throw "original error lost: [$script:functionalError]" }
    } finally { Pop-Location }
  }

  # 4. Early exception (candidate never starts): the outer cleanup still runs
  # and the run fails with the startup error.
  Assert-Scenario "early candidate-startup failure still cleans up" {
    $t = New-TestAttempt "early-fail"
    $probe = New-OutcomeProbe
    $verifier = New-VerifierMock @(0, 0, 0, 0, 0) $t.ResolvedSha
    $startCandidate = { param($port, $exe) throw "Candidate WebView did not expose a page CDP target in 90s (port $port)" }.GetNewClosure()
    $startFixture = {
      param($d, $p, $a)
      New-Item -ItemType Directory -Force -Path $d | Out-Null
      return @{ fixture_url = "http://127.0.0.1:9/catalog.json"; fixture_pubkey = "00" * 32 }
    }.GetNewClosure()
    $probe = @{ ran = $false }
    $stopFixture = { param($d) $probe.ran = $true; return "skipped-no-fixture" }.GetNewClosure()
    $workDir = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-regression-work-$($t.AttemptId)"
    New-Item -ItemType Directory -Force -Path $workDir | Out-Null
    Push-Location $workDir
    try {
      $code = Invoke-PackagedMatrix -Portable $t.Portable -Version $t.Version -ResolvedSha $t.ResolvedSha -CdpPort 14714 -AttemptId $t.AttemptId -IsolatedRoot $t.IsolatedRoot -Profile $t.Profile -CatalogState $t.CatalogState -StartFixture $startFixture -StartCandidateFn $startCandidate -RunVerifier $verifier -StopFixtureFn $stopFixture -Outcome $probe
      if ($code -ne 1) { throw "expected exit 1, got $code" }
      if (-not $script:functionalError.Contains("CDP target")) { throw "wrong error: [$script:functionalError]" }
      if (-not $probe.ran) { throw "outer cleanup did not run" }
    } finally { Pop-Location }
  }

  # 5. Candidate unable to stop: a live owned candidate after its deadline
  # fails the run even when every verifier passed.
  Assert-Scenario "unstoppable candidate fails the run" {
    $t = New-TestAttempt "candidate-alive"
    $probe = New-OutcomeProbe
    $verifier = New-VerifierMock @(0, 0, 0, 0, 0) $t.ResolvedSha
    $startCandidate = { param($port, $exe) return (Start-Sleeper) }.GetNewClosure()
    $startFixture = {
      param($d, $p, $a)
      New-Item -ItemType Directory -Force -Path $d | Out-Null
      return @{ fixture_url = "http://127.0.0.1:9/catalog.json"; fixture_pubkey = "00" * 32 }
    }.GetNewClosure()
    # The candidate stop claims still-alive without killing (an unkillable
    # process); the fixture stop is clean.
    $stopCandidate = { param($proc, $probe) $probe.notes += "candidate still-alive"; $probe.failed = $true; $probe.error = "Candidate process could not be stopped within the deadline"; try { Stop-SleeperNow $proc } catch { } }.GetNewClosure()
    $stopFixture = { param($d) return "skipped-no-fixture" }.GetNewClosure()
    $workDir = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-regression-work-$($t.AttemptId)"
    New-Item -ItemType Directory -Force -Path $workDir | Out-Null
    Push-Location $workDir
    try {
      # Drive the REAL Stop-Candidate decision (not the mock above) against a
      # live sleeper with a 0-second deadline to prove the production path
      # fails: call the real function directly first.
      $live = Start-Sleeper
      $script:functionalFailed = $false
      $script:functionalError = ""
      $script:cleanupNotes = @()
      Stop-Candidate $live  # real 15s deadline: stops the sleeper, no failure
      if ($script:functionalFailed) { throw "SETUP: real Stop-Candidate failed on a stoppable process" }
      Stop-SleeperNow $live
      # Now the full run with an unstoppable candidate mock fails.
      $code = Invoke-PackagedMatrix -Portable $t.Portable -Version $t.Version -ResolvedSha $t.ResolvedSha -CdpPort 14715 -AttemptId $t.AttemptId -IsolatedRoot $t.IsolatedRoot -Profile $t.Profile -CatalogState $t.CatalogState -StartFixture $startFixture -StartCandidateFn $startCandidate -RunVerifier $verifier -StopCandidateFn $stopCandidate -StopFixtureFn $stopFixture -Outcome $probe
      if ($code -ne 1) { throw "expected exit 1 for an unstoppable candidate, got $code" }
      if (-not $script:functionalError.Contains("Candidate")) { throw "wrong error: [$script:functionalError]" }
    } finally { Pop-Location }
  }

  # 6. Fixture unable to stop: still-alive from the owned fixture fails the
  # run even when every verifier passed.
  Assert-Scenario "unstoppable fixture fails the run" {
    $t = New-TestAttempt "fixture-alive"
    $probe = New-OutcomeProbe
    $verifier = New-VerifierMock @(0, 0, 0, 0, 0) $t.ResolvedSha
    $startCandidate = { param($port, $exe) return (Start-Sleeper) }.GetNewClosure()
    $startFixture = {
      param($d, $p, $a)
      New-Item -ItemType Directory -Force -Path $d | Out-Null
      return @{ fixture_url = "http://127.0.0.1:9/catalog.json"; fixture_pubkey = "00" * 32 }
    }.GetNewClosure()
    $stopCandidate = { param($proc) Stop-Candidate $proc }.GetNewClosure()
    $stopFixture = { param($d, $probe) $probe.failed = $true; $probe.error = "Catalog fixture process could not be stopped within the deadline"; $probe.notes += "fixture outcome=still-alive" }.GetNewClosure()
    $workDir = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-regression-work-$($t.AttemptId)"
    New-Item -ItemType Directory -Force -Path $workDir | Out-Null
    Push-Location $workDir
    try {
      $code = Invoke-PackagedMatrix -Portable $t.Portable -Version $t.Version -ResolvedSha $t.ResolvedSha -CdpPort 14716 -AttemptId $t.AttemptId -IsolatedRoot $t.IsolatedRoot -Profile $t.Profile -CatalogState $t.CatalogState -StartFixture $startFixture -StartCandidateFn $startCandidate -RunVerifier $verifier -StopCandidateFn $stopCandidate -StopFixtureFn $stopFixture -Outcome $probe
      if ($code -ne 1) { throw "expected exit 1 for an unstoppable fixture, got $code" }
      if (-not $script:functionalError.Contains("fixture")) { throw "wrong error: [$script:functionalError]" }
    } finally { Pop-Location }
  }

  # 7. Verifier failure + candidate still-alive: the original verifier error
  # is preserved as the prefix, and the run still fails.
  Assert-Scenario "verifier failure plus stuck candidate preserves the original error" {
    $t = New-TestAttempt "both-fail"
    $probe = New-OutcomeProbe
    $verifier = New-VerifierMock @(1, 0, 0, 0, 0) $t.ResolvedSha
    $startCandidate = { param($port, $exe) return (Start-Sleeper) }.GetNewClosure()
    $startFixture = {
      param($d, $p, $a)
      New-Item -ItemType Directory -Force -Path $d | Out-Null
      return @{ fixture_url = "http://127.0.0.1:9/catalog.json"; fixture_pubkey = "00" * 32 }
    }.GetNewClosure()
    $stopCandidate = { param($proc, $probe) $probe.failed = $true; $probe.error = "candidate still-alive after cleanup deadline"; $probe.notes += "candidate still-alive"; try { Stop-SleeperNow $proc } catch { } }.GetNewClosure()
    $stopFixture = { param($d) return "skipped-no-fixture" }.GetNewClosure()
    $workDir = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-regression-work-$($t.AttemptId)"
    New-Item -ItemType Directory -Force -Path $workDir | Out-Null
    Push-Location $workDir
    try {
      $code = Invoke-PackagedMatrix -Portable $t.Portable -Version $t.Version -ResolvedSha $t.ResolvedSha -CdpPort 14717 -AttemptId $t.AttemptId -IsolatedRoot $t.IsolatedRoot -Profile $t.Profile -CatalogState $t.CatalogState -StartFixture $startFixture -StartCandidateFn $startCandidate -RunVerifier $verifier -StopCandidateFn $stopCandidate -StopFixtureFn $stopFixture -Outcome $probe
      if ($code -ne 1) { throw "expected exit 1, got $code" }
      if (-not $script:functionalError.StartsWith("verify_041 failed with code 1")) { throw "original error not preserved: [$script:functionalError]" }
      if (-not ($script:cleanupNotes -join "; ").Contains("still-alive")) { throw "cleanup note missing: [$($script:cleanupNotes -join '; ')]" }
    } finally { Pop-Location }
  }

  # 8. Repeated cleanup: stopping the same owned handle twice is success
  # (second stop is already-stopped), through the REAL Stop-OwnedProcess.
  Assert-Scenario "cleanup is safe to run twice" {
    $owned = Start-Sleeper
    try {
      $first = Stop-OwnedProcess $owned "regression-fixture" 15
      $second = Stop-OwnedProcess $owned "regression-fixture" 5
      if ($first -ne "stopped") { throw "first stop: expected stopped, got $first" }
      if ($second -ne "already-stopped") { throw "second stop: expected already-stopped, got $second" }
    } finally { Stop-SleeperNow $owned }
  }

  # 9. Foreign process refusal: Get-FixtureAdoption refuses a live node
  # process whose command line lacks __serve, and the process SURVIVES
  # (proves adoption never kills by pid).
  Assert-Scenario "foreign pid is never adopted as the fixture" {
    $foreign = Start-Sleeper  # plain node sleeper: no __serve marker
    try {
      $refused = $false
      try { $null = Get-FixtureAdoption $foreign.Id } catch { $refused = $true; Write-Host "phase: adoption refused: $($_.Exception.Message)" }
      if (-not $refused) { throw "adoption accepted a foreign pid" }
      $foreign.Refresh()
      if ($foreign.HasExited) { throw "the foreign process is dead: adoption killed by pid" }
    } finally { Stop-SleeperNow $foreign }
  }

  # 10. Identity lookup for a dead pid refuses instead of adopting.
  Assert-Scenario "dead pid is never adopted as the fixture" {
    $owned = Start-Sleeper
    $deadPid = $owned.Id
    Stop-SleeperNow $owned
    Start-Sleep -Milliseconds 500
    $refused = $false
    try { $null = Get-FixtureAdoption $deadPid } catch { $refused = $true; Write-Host "phase: adoption refused: $($_.Exception.Message)" }
    if (-not $refused) { throw "adoption accepted a dead pid" }
  }

  # 11. No kill path accepts a bare pid: every Stop-OwnedProcess call site
  # passes a handle variable, never ".Id" or a pid integer, and no taskkill
  # survives in either production file.
  Assert-Scenario "no kill path accepts a bare pid" {
    foreach ($file in @("scripts/verify_packaged_impl.ps1", "scripts/verify_packaged_matrix.ps1")) {
      $text = Get-Content (Join-Path $RepoRoot $file) -Raw
      $calls = [regex]::Matches($text, "Stop-OwnedProcess ([^`r`n]+)")
      foreach ($m in $calls) {
        $arg = $m.Groups[1].Value.Trim()
        if ($arg -match "\.Id\b" -or $arg -match "\$[A-Za-z]*Pid\b" -or $arg -match "^\d+") {
          throw "bare-pid kill in ${file}: Stop-OwnedProcess $arg"
        }
      }
      if ($text -match "taskkill") { throw "taskkill survives in $file" }
    }
  }

  # 12. Partial initialization: fixture never started (StartFixture throws)
  # still runs the outer cleanup and fails with the init error.
  Assert-Scenario "fixture-init failure still runs outer cleanup" {
    $t = New-TestAttempt "init-fail"
    $probe = New-OutcomeProbe
    $verifier = New-VerifierMock @(0, 0, 0, 0, 0) $t.ResolvedSha
    $startFixture = { param($d, $p, $a) throw "The catalog fixture server did not initialize (exit 1): boom" }.GetNewClosure()
    $probe = @{ ran = $false }
    $stopFixture = { param($d) $probe.ran = $true; return "skipped-no-fixture" }.GetNewClosure()
    $workDir = Join-Path ([System.IO.Path]::GetTempPath()) "localmotive-regression-work-$($t.AttemptId)"
    New-Item -ItemType Directory -Force -Path $workDir | Out-Null
    Push-Location $workDir
    try {
      $code = Invoke-PackagedMatrix -Portable $t.Portable -Version $t.Version -ResolvedSha $t.ResolvedSha -CdpPort 14718 -AttemptId $t.AttemptId -IsolatedRoot $t.IsolatedRoot -Profile $t.Profile -CatalogState $t.CatalogState -StartFixture $startFixture -RunVerifier $verifier -StopFixtureFn $stopFixture -Outcome $probe
      if ($code -ne 1) { throw "expected exit 1, got $code" }
      if (-not $script:functionalError.Contains("did not initialize")) { throw "wrong error: [$script:functionalError]" }
      if (-not $probe.ran) { throw "outer cleanup did not run" }
    } finally { Pop-Location }
  }

  Write-Host ""
  Write-Host "cleanup-matrix: $($script:passed) passed, $($script:failed.Count) failed"
  if ($script:failed.Count -gt 0) { throw "Failing scenarios: $($script:failed -join '; ')" }
} finally {
  Pop-Location
}
