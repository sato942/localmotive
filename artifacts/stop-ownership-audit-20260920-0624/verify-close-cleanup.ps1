$ErrorActionPreference = 'Stop'
$apps = @(Get-Process | Where-Object { $_.ProcessName -eq 'localmotive' } | Select-Object Id, ProcessName)
$tests = @(Get-Process | Where-Object { $_.ProcessName -like 'localmotive_lib*' } | Select-Object Id, ProcessName)
$listeners = @(Get-NetTCPConnection -State Listen | Where-Object { $_.LocalPort -eq 10193 } | Select-Object LocalPort, OwningProcess)
$query = "SELECT ProcessId,ParentProcessId,CreationDate FROM Win32_Process WHERE Name='powershell.exe' AND CommandLine LIKE '%Start-Sleep -Seconds 600%'"
$standIns = @(Get-CimInstance -Query $query | Select-Object ProcessId, ParentProcessId, CreationDate)
$report = [ordered]@{
  checkedAt = (Get-Date -Format o)
  appProcesses = $apps
  testProcesses = $tests
  debuggerListeners = $listeners
  standInCandidates = $standIns
  status = 'PASS'
}
if ($apps.Count -or $tests.Count -or $listeners.Count -or $standIns.Count) {
  $report.status = 'NEEDS_INSPECTION'
}
$report | ConvertTo-Json -Depth 4 | Set-Content (Join-Path $PSScriptRoot 'close-cleanup.json')
$report | ConvertTo-Json -Depth 4
if ($report.status -ne 'PASS') { exit 1 }
