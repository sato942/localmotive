# Exercise the real cleanup path without terminating a user's Sandbox.
$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot '../sandbox/host-run-lifecycle.ps1'
$ast = [Management.Automation.Language.Parser]::ParseFile($source, [ref]$null, [ref]$null)
$definition = $ast.Find({ param($node) $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -eq 'Stop-LifecycleSandbox' }, $true)
if ($definition) {
  . ([scriptblock]::Create($definition.Extent.Text))
  $cleanup = { Stop-LifecycleSandbox }
} else {
  # RED: execute the old destructive pipeline, replacing only its OS boundary.
  $legacy = $ast.Find({ param($node) $node -is [Management.Automation.Language.PipelineAst] -and $node.Extent.Text.StartsWith('Get-Process -Name "WindowsSandbox"') }, $true)
  if (-not $legacy) { throw 'No Sandbox cleanup path found' }
  $cleanup = [scriptblock]::Create($legacy.Extent.Text)
}
$script:stoppedProcesses = @()
function Get-Process { @([pscustomobject]@{ Id = 101 }, [pscustomobject]@{ Id = 202 }) }
function Stop-Process {
  param([Parameter(ValueFromPipeline=$true)]$InputObject, [switch]$Force)
  process { $script:stoppedProcesses += $InputObject.Id }
}
$script:calls = @()
$script:stopCode = 0
function wsb.exe {
  $script:calls += ,@($args)
  $global:LASTEXITCODE = $script:stopCode
}
$owned = '00000000-0000-4000-8000-000000000101'
$script:SandboxId = $owned
& $cleanup
if ($script:stoppedProcesses.Count) { throw "Cleanup terminated unscoped processes: $($script:stoppedProcesses -join ',')" }
if ($script:calls.Count -ne 1 -or ($script:calls[0] -join ' ') -ne "stop --id $owned --raw") { throw 'Cleanup did not stop exactly the owned Sandbox ID' }
if ($script:SandboxId) { throw 'Successful cleanup retained a live Sandbox identity' }
Write-Host 'PASS: cleanup targets only the owned Sandbox ID'

$script:SandboxId = $owned
$script:stopCode = 1
$refused = $false
try { & $cleanup } catch { $refused = $true }
if (-not $refused -or $script:SandboxId -ne $owned) { throw 'Failed cleanup was suppressed or lost its Sandbox identity' }
Write-Host 'PASS: cleanup failure is reported and retains the owned identity'
