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
  # version, not merely that the installer exited zero (audit GH-04 I2).
  $version = (Get-Item $exe).VersionInfo.FileVersion
  if (-not $version -or -not $version.StartsWith($expected)) {
    Fail "$label expected executable version $expected but found '$version' at $exe"
  }
  Log "$label executable version $version at $exe"
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
  Launch-Smoke $exe
  Uninstall-Nsis
  if (Find-AppExe) { Fail "Localmotive.exe still present after NSIS uninstall" }
  $steps += @{ name = "nsis-fresh-install-launch-uninstall"; status = "PASS"; detail = "uninstall removed the executable" }
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

  $doc = @{
    schema = "localmotive.sandbox-lifecycle.v0"
    status = "PASS"
    tag = $meta.tag
    version = $meta.version
    previousTag = $meta.previousTag
    finishedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    coverageNote = "NSIS and MSI fresh install/launch/uninstall and the NSIS update path with executable version evidence. Eight-second process survival is a startup smoke, not full functional verification. Persisted-profile migration scenarios are not exercised in this harness."
    steps = $steps
  }
  ($doc | ConvertTo-Json -Depth 6) | Set-Content -Path $ResultPath -Encoding UTF8
  Log "ALL PASS"
  exit 0
} catch {
  Fail $_.Exception.Message
}