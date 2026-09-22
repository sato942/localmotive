# Execute the real session helper with real files and owned child processes.
# CDP discovery is stubbed here; the native lifecycle supplies application proof.
$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot '../sandbox/run-lifecycle-in-sandbox.ps1'
$tokens = $null
$errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($source, [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw ($errors | Out-String) }
foreach ($name in @('Log', 'Use-SettingsSession', 'Launch-Smoke')) {
  $definition = $ast.Find({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq $name }, $true)
  if (-not $definition) { throw "Missing function $name" }
  . ([scriptblock]::Create($definition.Extent.Text))
}

$root = Join-Path ([IO.Path]::GetTempPath()) ('lm-settings-regression-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
$savedTemp = $env:TEMP
$savedFolder = $env:WEBVIEW2_USER_DATA_FOLDER
$savedArguments = $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$Log = Join-Path $root 'session.log'
$script:children = @()
# Use a real sleeper, not an application or another user's process.
function Start-Process {
  param($FilePath, [switch]$PassThru)
  $child = Microsoft.PowerShell.Management\Start-Process -FilePath $FilePath -ArgumentList @('-NoProfile', '-Command', 'Start-Sleep -Seconds 60') -WindowStyle Hidden -PassThru
  $script:children += Get-Process -Id $child.Id
  if ($script:failCleanup) {
    # Inject the documented process-cleanup failure without changing OS permissions.
    Add-Member -InputObject $child -MemberType ScriptMethod -Name CloseMainWindow -Value { throw 'intentional cleanup failure' } -Force
  }
  return $child
}
function Invoke-RestMethod { return @{ type = 'page'; webSocketDebuggerUrl = 'fixture-only' } }
# Skip only the smoke dwell; these checks exercise ownership, not startup timing.
function Start-Sleep { }
function Invoke-CdpProbe {
  if ($script:failProbe) { throw 'intentional probe failure' }
  return @{ navButtons = 3; hasLocalmotive = $true; bodyChars = 100 }
}
function Stop-Process {
  [CmdletBinding()]param([int]$Id, [switch]$Force)
  Write-Error 'intentional Stop-Process failure'
}
try {
  $env:TEMP = $root
  $script:SettingsDataRoot = Join-Path $root 'lm-settings-session'
  New-Item -ItemType Directory -Path (Join-Path $script:SettingsDataRoot 'webview-profile') | Out-Null
  $exe = (Get-Command pwsh).Source
  Use-SettingsSession $exe 'baseline' {
    Set-Content -LiteralPath (Join-Path $env:WEBVIEW2_USER_DATA_FOLDER 'known-settings.txt') -Value 'baseline-value' -NoNewline
  }
  Use-SettingsSession $exe 'upgrade' {
    $path = Join-Path $env:WEBVIEW2_USER_DATA_FOLDER 'known-settings.txt'
    if (-not (Test-Path $path)) { throw 'Baseline settings were deleted before the upgraded session' }
    if ((Get-Content $path -Raw) -ne 'baseline-value') { throw 'Baseline settings changed before the upgraded session' }
  }
  Write-Host 'PASS: baseline storage survives the upgraded session'

  $failed = $false
  try { Use-SettingsSession $exe 'failed body' { throw 'intentional verifier failure' } }
  catch {
    if ($_.Exception.Message -ne 'intentional verifier failure') { throw }
    $failed = $true
  }
  if (-not $failed) { throw 'Session suppressed the verifier failure' }
  foreach ($child in $script:children) {
    $child.Refresh()
    if (-not $child.HasExited) { throw "Session left owned child $($child.Id) running" }
  }
  if ($env:WEBVIEW2_USER_DATA_FOLDER -ne $savedFolder -or $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ne $savedArguments) {
    throw 'Session did not restore the prior environment'
  }
  Write-Host 'PASS: success and failure stop owned children and restore the environment'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = 'fixture-saved-arguments'
  try {
    foreach ($script:failProbe in @($false, $true)) {
      Launch-Smoke $exe 'owned smoke'
      $child = $script:children[-1]
      $child.Refresh()
      if (-not $child.HasExited) { throw 'Launch smoke left its owned process running' }
      if ($env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ne 'fixture-saved-arguments') { throw 'Launch smoke did not restore its environment' }
    }
  } finally { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $savedArguments }
  Write-Host 'PASS: launch smoke joins its owned process and restores the environment'
  $script:failCleanup = $true
  $failed = $false
  try { Use-SettingsSession $exe 'failed body and cleanup' { throw 'intentional verifier failure' } }
  catch {
    if ($_.Exception.Message -notmatch 'intentional verifier failure' -or $_.Exception.Message -notmatch 'intentional cleanup failure') {
      throw "Session hid one failure: $($_.Exception.Message)"
    }
    $failed = $true
  } finally { $script:failCleanup = $false }
  if (-not $failed) { throw 'Session suppressed both failures' }
  # A failed graceful close must not leave the real child for this test's
  # outer safety cleanup to hide. The session itself owns termination.
  foreach ($child in $script:children) {
    $child.Refresh()
    if (-not $child.HasExited) { throw "Failed graceful close orphaned owned child $($child.Id)" }
  }
  if ($env:WEBVIEW2_USER_DATA_FOLDER -ne $savedFolder -or $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ne $savedArguments) { throw 'Failed cleanup did not restore the environment' }
  Write-Host 'PASS: verifier and cleanup failures remain visible with the environment restored'
} finally {
  $cleanupErrors = @()
  try {
    foreach ($child in $script:children) {
      try {
        $child.Refresh()
        if (-not $child.HasExited) {
          $child.Kill()
          if (-not $child.WaitForExit(15000)) { throw "Owned fixture child $($child.Id) did not exit" }
        }
      } catch { $cleanupErrors += $_.Exception.Message }
      finally { $child.Dispose() }
    }
  } finally {
    $env:TEMP = $savedTemp
    $env:WEBVIEW2_USER_DATA_FOLDER = $savedFolder
    $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = $savedArguments
    if (-not $cleanupErrors.Count) { Remove-Item -LiteralPath $root -Recurse -Force }
  }
  if ($cleanupErrors.Count) { throw "Fixture cleanup failed; retained $root`: $($cleanupErrors -join '; ')" }
}
