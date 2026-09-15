# Execute the guest's real collection block against both released storage shapes.
$ErrorActionPreference = 'Stop'
$source = Get-Content (Join-Path $PSScriptRoot '../sandbox/run-lifecycle-in-sandbox.ps1') -Raw
$begin = $source.IndexOf('    $mirrorAfter = Join-Path $userData "catalog-mirror.sqlite"')
$end = $source.IndexOf('    Uninstall-Nsis', $begin)
if ($begin -lt 0 -or $end -le $begin) { throw 'Missing preservation collection block' }
$collect = [scriptblock]::Create($source.Substring($begin, $end - $begin))
function Fail([string]$Message) { throw $Message }
$root = Join-Path ([IO.Path]::GetTempPath()) ('lm-preservation-collection-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $root | Out-Null
try {
  foreach ($preservationFlavor in @('mirror', 'cache')) {
    $userData = Join-Path $root "$preservationFlavor-userdata"
    $Shared = Join-Path $root "$preservationFlavor-collected"
    New-Item -ItemType Directory -Path $userData, $Shared | Out-Null
    $canaryAfter = Join-Path $userData 'canary-userdata.txt'
    Copy-Item (Join-Path $PSScriptRoot '../sandbox/canary-userdata.txt') $canaryAfter
    if ($preservationFlavor -eq 'cache') {
      Copy-Item (Join-Path $PSScriptRoot '../sandbox/canary-catalog-cache.json') (Join-Path $userData 'catalog-cache.json')
      $sourceName = 'catalog-cache.json'; $collectedName = 'collected-catalog-cache.json'
    } else {
      $bytes = [Convert]::FromBase64String((Get-Content (Join-Path $PSScriptRoot '../sandbox/canary-mirror.sqlite.b64') -Raw).Trim())
      [IO.File]::WriteAllBytes((Join-Path $userData 'catalog-mirror.sqlite'), $bytes)
      $sourceName = 'catalog-mirror.sqlite'; $collectedName = 'collected-mirror.sqlite'
    }
    & $collect
    if ((Get-FileHash (Join-Path $Shared $collectedName)).Hash -ne (Get-FileHash (Join-Path $userData $sourceName)).Hash) { throw "$preservationFlavor collected bytes changed" }
    if (-not (Test-Path (Join-Path $Shared 'collected-userdata.txt'))) { throw "$preservationFlavor lost user data" }
    if ($preservationFlavor -eq 'cache' -and (Test-Path (Join-Path $Shared 'collected-mirror.sqlite'))) { throw 'Cache-only baseline unexpectedly supplied a mirror' }
    Write-Host "PASS: $preservationFlavor-only baseline collects its actual files"
    $locked = [IO.File]::Open((Join-Path $Shared $collectedName), [IO.FileMode]::Open, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
    try {
      $ErrorActionPreference = 'Continue'
      $refused = $false
      try { & $collect } catch { $refused = $true }
    } finally {
      $ErrorActionPreference = 'Stop'
      $locked.Dispose()
    }
    if (-not $refused) { throw "$preservationFlavor collection accepted a failed required copy" }
    Remove-Item (Join-Path $userData $sourceName)
    $refused = $false
    try { & $collect } catch { $refused = $true }
    if (-not $refused) { throw "$preservationFlavor collection accepted missing required data" }
  }
  Write-Host 'PASS: missing required preservation data is rejected'
} finally {
  Remove-Item -LiteralPath $root -Recurse -Force
}
