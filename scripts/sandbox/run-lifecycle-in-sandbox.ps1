# Runs inside Windows Sandbox. Writes result.json to the shared folder.
$ErrorActionPreference = "Continue"
$Shared = "C:\Users\WDAGUtilityAccount\Desktop\shared"
$Log = Join-Path $Shared "lifecycle.log"
$ResultPath = Join-Path $Shared "result.json"
function Log($m) {
  $line = "[{0:o}] {1}" -f (Get-Date).ToUniversalTime(), $m
  Add-Content -Path $Log -Value $line -Encoding UTF8
  Write-Host $line
}
function Fail($m) {
  Log "FAIL: $m"

  $doc = @{
    schema = "localmotive.sandbox-lifecycle.v0"
    status = "FAIL"
    error = $m
    finishedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    log = $Log
  }
  ($doc | ConvertTo-Json -Depth 6) | Set-Content -Path $ResultPath -Encoding UTF8
  exit 1
}
function Find-AppExe {
  $candidates = @(
    "$env:LOCALAPPDATA\Localmotive\Localmotive.exe",
    "$env:LOCALAPPDATA\Programs\Localmotive\Localmotive.exe",
    "${env:ProgramFiles}\Localmotive\Localmotive.exe",
    "${env:ProgramFiles(x86)}\Localmotive\Localmotive.exe"
  )
  foreach ($c in $candidates) { if (Test-Path $c) { return $c } }
  $hit = Get-ChildItem -Path "$env:LOCALAPPDATA","$env:ProgramFiles","${env:ProgramFiles(x86)}" -Filter "Localmotive.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
  if ($hit) { return $hit.FullName }
  return $null
}
function Launch-Smoke($exe) {
  Log "Launch smoke: $exe"
  $p = Start-Process -FilePath $exe -PassThru
  Start-Sleep -Seconds 8
  if ($p.HasExited) { throw "App exited during smoke launch code=$($p.ExitCode)" }
  Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
  Start-Sleep -Seconds 2
  Log "Startup smoke OK: the process survived 8 seconds; this is a startup smoke, not full functional verification"
}
function Install-Nsis($setup) {
  Log "NSIS install: $setup"
  if (-not (Test-Path $setup)) { throw "NSIS setup missing: $setup" }
  $p = Start-Process -FilePath $setup -ArgumentList "/S" -Wait -PassThru
  if ($p.ExitCode -ne 0) {
    $wv = $null
    try { $wv = (Get-ItemProperty "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" -ErrorAction SilentlyContinue).pv } catch {}
    throw "NSIS install exit $($p.ExitCode) (WebView2 pv=$wv; need Sandbox Networking=Enable or embedBootstrapper)"
  }
  Start-Sleep -Seconds 2
}
function Uninstall-Nsis {
  $unins = @(
    "$env:LOCALAPPDATA\Localmotive\uninstall.exe",
    "$env:LOCALAPPDATA\Programs\Localmotive\uninstall.exe",
    "${env:ProgramFiles}\Localmotive\uninstall.exe"
  ) | Where-Object { Test-Path $_ } | Select-Object -First 1
  if (-not $unins) {
    $unins = Get-ChildItem -Path "$env:LOCALAPPDATA","$env:ProgramFiles" -Filter "uninstall.exe" -Recurse -ErrorAction SilentlyContinue |
      Where-Object { $_.DirectoryName -match "Localmotive" } | Select-Object -First 1 -ExpandProperty FullName
  }
  if (-not $unins) { throw "NSIS uninstall.exe not found" }
  Log "NSIS uninstall: $unins"
  $p = Start-Process -FilePath $unins -ArgumentList "/S" -Wait -PassThru
  if ($p.ExitCode -ne 0) { throw "NSIS uninstall exit $($p.ExitCode)" }
  Start-Sleep -Seconds 2
}
function Install-Msi($msi) {
  Log "MSI install: $msi"
  $p = Start-Process -FilePath "msiexec.exe" -ArgumentList "/i `"$msi`" /qn /norestart" -Wait -PassThru
  if ($p.ExitCode -ne 0 -and $p.ExitCode -ne 3010) { throw "MSI install exit $($p.ExitCode)" }
  Start-Sleep -Seconds 2
}
function Test-MsiProductInstalled {
  # The MSI registers the product under an uninstall key; a leftover
  # executable search alone cannot prove removal (audit GH-04 I1).
  $roots = @(
    "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKLM:\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*",
    "HKCU:\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\*"
  )
  foreach ($root in $roots) {
    $hit = Get-ItemProperty $root -ErrorAction SilentlyContinue |
      Where-Object { $_.DisplayName -like "*Localmotive*" } |
      Select-Object -First 1
    if ($hit) { return $true }
  }
  return $false
}
function Assert-AppVersion($exe, $expected, $label) {
  # An upgrade verdict must prove the INSTALLED executable carries the new
  # version, not merely that the installer exited zero (audit GH-04 I2), and
  # every installed copy's digest is captured so the harness can require one
  # payload across NSIS, MSI and the update path, and replaced bytes across
  # the update (audit GH-04 I3: digest after upgrade). The digest is compared
  # across installer paths rather than to a host-built binary: Rust release
  # builds are not byte-reproducible between invocations.
  $version = (Get-Item $exe).VersionInfo.FileVersion
  # R09 (follow-up review db548c8): the comparison is exact. StartsWith
  # accepted near-miss identities ("0.6.00", "0.6.0-beta"), which is a
  # version-identity hole. The canonical 4-part form normalizes to the
  # documented value; everything else fails.
  $normalized = if ($version) { $version.Trim() } else { $null }
  if ($normalized -eq "$expected.0") { $normalized = $expected }
  if ($normalized -ne $expected) {
    Fail "$label expected executable version $expected exactly but found '$version' at $exe"
  }
  $installedSha = (Get-FileHash -Path $exe -Algorithm SHA256).Hash.ToLower()
  $script:InstalledDigests[$label] = $installedSha
  Log "$label executable version $version digest $installedSha at $exe"
  return $version
}
function Uninstall-Msi($msi) {
  Log "MSI uninstall via package: $msi"
  $p = Start-Process -FilePath "msiexec.exe" -ArgumentList "/x `"$msi`" /qn /norestart" -Wait -PassThru
  if ($p.ExitCode -ne 0 -and $p.ExitCode -ne 1605 -and $p.ExitCode -ne 3010) { throw "MSI uninstall exit $($p.ExitCode)" }
  Start-Sleep -Seconds 2
}

try {
  New-Item -ItemType Directory -Force -Path $Shared | Out-Null
  if (Test-Path $ResultPath) { Remove-Item $ResultPath -Force }
  "" | Set-Content $Log -Encoding UTF8
  Log "Sandbox lifecycle starting"

  $metaPath = Join-Path $Shared "meta.json"
  if (-not (Test-Path $metaPath)) { Fail "meta.json missing" }
  $meta = Get-Content $metaPath -Raw | ConvertFrom-Json
  $script:InstalledDigests = @{}
  $curSetup = Join-Path $Shared $meta.currentSetup
  $curMsi = Join-Path $Shared $meta.currentMsi
  $oldSetup = Join-Path $Shared $meta.previousSetup
  foreach ($f in @($curSetup,$curMsi,$oldSetup)) {
    if (-not (Test-Path $f)) { Fail "Missing installer: $f" }
  }

  $steps = @()

  # 1) NSIS fresh install + launch + uninstall
  Install-Nsis $curSetup
  $exe = Find-AppExe
  if (-not $exe) { Fail "Localmotive.exe not found after NSIS install" }
  $nsisVersion = Assert-AppVersion $exe $meta.version "NSIS fresh install"
  Launch-Smoke $exe
  Uninstall-Nsis
  if (Find-AppExe) { Fail "Localmotive.exe still present after NSIS uninstall" }
  $steps += @{ name = "nsis-fresh-install-launch-uninstall"; status = "PASS"; detail = "installed version $nsisVersion; uninstall removed the executable" }
  Log "NSIS fresh path PASS"

  # 2) MSI fresh install + launch + uninstall
  Install-Msi $curMsi
  $exe = Find-AppExe
  if (-not $exe) { Fail "Localmotive.exe not found after MSI install" }
  $msiVersion = Assert-AppVersion $exe $meta.version "MSI fresh install"
  Launch-Smoke $exe
  Uninstall-Msi $curMsi
  Start-Sleep -Seconds 2
  # Audit GH-04: a leftover expected application or product registration is
  # a FAILED uninstall verdict, never a warning to continue past.
  $leftoverExe = Find-AppExe
  if ($leftoverExe) { Fail "Localmotive.exe still present after MSI uninstall: $leftoverExe" }
  if (Test-MsiProductInstalled) { Fail "MSI product registration still present after uninstall" }
  $steps += @{ name = "msi-fresh-install-launch-uninstall"; status = "PASS"; detail = "installed version $msiVersion; uninstall removed the executable and the product registration" }
  Log "MSI fresh path PASS"

  # 3) Update: previous NSIS -> current NSIS, with version evidence on both
  #    sides of the upgrade (audit GH-04 I2).
  $previousVersion = $meta.previousTag.TrimStart("v")
  Install-Nsis $oldSetup
  $oldExe = Find-AppExe
  if (-not $oldExe) { Fail "Previous version missing after old NSIS install" }
  $installedOldVersion = Assert-AppVersion $oldExe $previousVersion "Pre-update install"
  Install-Nsis $curSetup
  $exe = Find-AppExe
  if (-not $exe) { Fail "App missing after update install" }
  $installedNewVersion = Assert-AppVersion $exe $meta.version "Post-update install"
  Launch-Smoke $exe
  Uninstall-Nsis
  if (Find-AppExe) { Fail "App still present after post-update uninstall" }
  $steps += @{ name = "nsis-update-from-previous-launch-uninstall"; status = "PASS"; detail = "executable version $installedOldVersion -> $installedNewVersion" }
  Log "NSIS update path PASS"

  # 4) Preservation: previous -> current with planted canaries (G-06.I2).
  #    A valid catalog mirror with a marker table plus a user-data canary file
  #    are planted after the previous install; the upgrade and a launch smoke
  #    must leave both intact, and the mirror is collected for host-side
  #    marker verification.
  $preservePath = Join-Path $Shared "preserve.json"
  if (Test-Path $preservePath) {
    Install-Nsis $oldSetup
    $oldExe = Find-AppExe
    if (-not $oldExe) { Fail "Previous version missing before preservation install" }
    Launch-Smoke $oldExe
    $userData = Join-Path $env:LOCALAPPDATA "io.github.localmotive.app"
    if (-not (Test-Path $userData)) { Fail "User-data dir missing after previous install: $userData" }
    Copy-Item (Join-Path $Shared "canary-userdata.txt") (Join-Path $userData "canary-userdata.txt") -Force
    # R07: plant the REAL baseline persistence for this leg's flavor. The
    # mirror flavor stages a v0.5.0-schema catalog mirror with a user override
    # row; the cache flavor stages the v0.4.1-era cache record at the path the
    # 0.6 application still reads.
    $preservationFlavor = if ($meta.PSObject.Properties.Name -contains "preservation") { [string]$meta.preservation } else { "mirror" }
    if ($preservationFlavor -eq "cache") {
      Copy-Item (Join-Path $Shared "canary-catalog-cache.json") (Join-Path $userData "catalog-cache.json") -Force
    } else {
      Copy-Item (Join-Path $Shared "canary-mirror.sqlite") (Join-Path $userData "catalog-mirror.sqlite") -Force
    }
    Install-Nsis $curSetup
    $exe = Find-AppExe
    if (-not $exe) { Fail "App missing after preservation upgrade" }
    $preservedVersion = Assert-AppVersion $exe $meta.version "Preservation upgrade"
    Launch-Smoke $exe
    $canaryAfter = Join-Path $userData "canary-userdata.txt"
    if (-not (Test-Path $canaryAfter)) { Fail "User-data canary missing after upgrade" }
    $mirrorAfter = Join-Path $userData "catalog-mirror.sqlite"
    if ($preservationFlavor -eq "cache") {
      $cacheAfter = Join-Path $userData "catalog-cache.json"
      if (-not (Test-Path $cacheAfter)) { Fail "Catalog cache record missing after upgrade: preserved data was deleted" }
    } elseif (-not (Test-Path $mirrorAfter)) {
      Fail "Catalog mirror missing after upgrade: preserved data was deleted"
    }
    Copy-Item $mirrorAfter (Join-Path $Shared "collected-mirror.sqlite") -Force
    Copy-Item $canaryAfter (Join-Path $Shared "collected-userdata.txt") -Force
    $cacheCollect = Join-Path $userData "catalog-cache.json"
    if (Test-Path $cacheCollect) {
      Copy-Item $cacheCollect (Join-Path $Shared "collected-catalog-cache.json") -Force
    }
    Uninstall-Nsis
    if (Find-AppExe) { Fail "App still present after preservation uninstall" }
    $steps += @{ name = "nsis-preservation-from-$($meta.previousTag)"; status = "PASS"; detail = "user-data canary and catalog mirror survived the upgrade; files collected for host verification; executable $preservedVersion" }
    Log "Preservation path PASS"
  }

  # Cross-path payload consistency (GH-04 I3): the same candidate executable
  # must be installed by NSIS, by MSI and by the update path, and the update
  # must replace the previous version's bytes.
  $nsisLabels = @("NSIS fresh install", "Post-update install", "Preservation upgrade")
  $nsisShas = @()
  foreach ($label in $nsisLabels) {
    if ($script:InstalledDigests.ContainsKey($label)) { $nsisShas += $script:InstalledDigests[$label] }
  }
  if ($nsisShas.Count -lt 2) { Fail "Installed digest capture incomplete: $($script:InstalledDigests | ConvertTo-Json -Compress)" }
  $nsisUnique = @($nsisShas | Select-Object -Unique)
  if ($nsisUnique.Count -ne 1) { Fail "NSIS-family installed executable digests differ: $($nsisShas -join ', ')" }
  if ($script:InstalledDigests.ContainsKey("Pre-update install") -and $script:InstalledDigests["Pre-update install"] -eq $nsisUnique[0]) {
    Fail "Post-update executable digest equals the pre-update digest; the update did not replace the executable"
  }
  Log "NSIS-family installed payload digest consistent: $($nsisUnique[0])"
  if ($script:InstalledDigests.ContainsKey("MSI fresh install")) {
    $msiSha = $script:InstalledDigests["MSI fresh install"]
    if ($msiSha -ne $nsisUnique[0]) {
      # Observed, recorded, and NOT silently accepted: the MSI payload's bytes
      # differ from the NSIS payload's bytes for the same version. Both carry
      # version 0.6.0 and each is self-consistent; cross-bundler byte identity
      # is not established (open observation in the tracker).
      Log "OBSERVED: MSI payload digest $msiSha differs from the NSIS payload digest $($nsisUnique[0])"
    } else {
      Log "MSI payload digest matches the NSIS payload digest: $msiSha"
    }
  }

  $msiPayloadDigest = if ($script:InstalledDigests.ContainsKey("MSI fresh install")) { $script:InstalledDigests["MSI fresh install"] } else { $null }
  $doc = @{
    schema = "localmotive.sandbox-lifecycle.v0"
    status = "PASS"
    installedDigests = $script:InstalledDigests
    nsisPayloadDigest = $nsisUnique[0]
    msiPayloadDigest = $msiPayloadDigest
    installedPayloadNote = "Established payload expectations (R09): every NSIS path (fresh, update, preservation) must install one byte-identical executable, and the update must differ from the previous version - both enforced above. The MSI-installed executable is measured and version-verified on its own path; cross-bundler byte identity between the MSI image and the NSIS image is NOT claimed (release builds are not byte-reproducible between bundlers). No claim is made about sidecar files beyond the executable; the executable digest, its version string, and process survival are the measured identity."
    tag = $meta.tag
    version = $meta.version
    previousTag = $meta.previousTag
    finishedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    coverageNote = "NSIS and MSI fresh install/launch/uninstall and the NSIS update path with executable version and cross-path digest evidence. Eight-second process survival is a startup smoke, not full functional verification. The preservation step (staged via preserve.json) plants, per flavor, the released baseline's REAL persistence after the previous install - the v0.5.0-schema catalog mirror with a user override row, or the v0.4.1-era catalog cache record - and requires it to survive the upgrade and a launch smoke. The collected files are verified on the host (R07): the user override rows must survive with their ownership flags and sentinel values (mirror flavor), or the cache record must remain present and parseable (cache flavor)."""
    steps = $steps
  }
  ($doc | ConvertTo-Json -Depth 6) | Set-Content -Path $ResultPath -Encoding UTF8
  Log "ALL PASS"
  exit 0
} catch {
  Fail $_.Exception.Message
}