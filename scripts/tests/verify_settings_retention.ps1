# Exercise the host's actual retention block with disposable observed-value fixtures.
$ErrorActionPreference = 'Stop'
$source = Get-Content (Join-Path $PSScriptRoot '../sandbox/host-run-lifecycle.ps1') -Raw
$begin = $source.IndexOf('        if (Test-Path $collectedMirror)')
$end = $source.IndexOf("`n", $source.IndexOf('        $preservationOutput | Set-Content', $begin))
if ($begin -lt 0 -or $end -le $begin) { throw 'Missing host retention block' }
$retain = [scriptblock]::Create($source.Substring($begin, $end - $begin))
$bindBegin = $source.IndexOf('      $preservation = [ordered]')
$bindEnd = $source.IndexOf('      # Bind the pass verdict', $bindBegin)
$bind = [scriptblock]::Create($source.Substring($bindBegin, $bindEnd - $bindBegin))
$root = Join-Path ([IO.Path]::GetTempPath()) ('lm-settings-retention-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
function Assert-LockOwnership { if (-not $script:allowWrite) { throw 'fixture ownership lost' } }
function Get-FileSha256([string]$Path) { (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant() }
try {
  $OutDir = Join-Path $root 'retained'
  New-Item -ItemType Directory -Path $OutDir | Out-Null
  $EvidenceName = 'settings-retention-fixture'
  $collectedMirror = Join-Path $root 'absent-mirror.sqlite'
  $collectedCache = Join-Path $root 'absent-cache.json'
  $collectedUserdata = Join-Path $root 'absent-userdata.txt'
  $collectedSettings = Join-Path $root 'collected-settings.json'
  Set-Content -LiteralPath $collectedSettings -Encoding UTF8 -Value '{"schema":"localmotive.settings-reads.v1","reads":{"fixture":"observed"}}'
  $preservationStatus = 'PASS'; $preservationOutput = 'fixture verification'
  $script:allowWrite = $true
  . $retain
  $retained = Join-Path $OutDir "$EvidenceName-collected-settings.json"
  if (-not (Test-Path -LiteralPath $retained)) { throw 'Host dropped the actual settings reads instead of retaining them' }
  if ((Get-FileSha256 $retained) -ne (Get-FileSha256 $collectedSettings)) { throw 'Retained settings bytes changed' }
  $doc = [pscustomobject]@{ status = 'PASS' }
  . $bind
  if ($doc.collectedSettings.file -ne "$EvidenceName-collected-settings.json" -or $doc.collectedSettings.sha256 -ne (Get-FileSha256 $retained)) { throw 'Lifecycle evidence did not bind the retained settings file and digest' }
  Write-Host 'PASS: settings reads are retained byte-for-byte and bound to the lifecycle record'
  Remove-Item -LiteralPath $retained
  $script:allowWrite = $false
  $refused = $false
  try { . $retain } catch { $refused = $_.Exception.Message -eq 'fixture ownership lost' }
  if (-not $refused -or (Test-Path -LiteralPath $retained)) { throw 'Lost ownership still allowed a settings evidence write' }
  Write-Host 'PASS: lost ownership refuses settings evidence writes'
} finally {
  Remove-Item -LiteralPath $root -Recurse -Force
}
