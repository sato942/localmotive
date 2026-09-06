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
  Log "Launch smoke OK"
}
function Install-Nsis($setup) {
  Log "NSIS install: $setup"
  $p = Start-Process -FilePath $setup -ArgumentList "/S" -Wait -PassThru
  if ($p.ExitCode -ne 0) { throw "NSIS install exit $($p.ExitCode)" }
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
  $steps += @{ name = "nsis-fresh-install-launch-uninstall"; status = "PASS" }
  Log "NSIS fresh path PASS"

  # 2) MSI fresh install + launch + uninstall
  Install-Msi $curMsi
  $exe = Find-AppExe
  if (-not $exe) { Fail "Localmotive.exe not found after MSI install" }
  Launch-Smoke $exe
  Uninstall-Msi $curMsi
  Start-Sleep -Seconds 2
  if (Find-AppExe) { Log "WARNING: exe still present after MSI uninstall; continuing" }
  $steps += @{ name = "msi-fresh-install-launch-uninstall"; status = "PASS" }
  Log "MSI fresh path PASS"

  # 3) Update: previous NSIS -> current NSIS
  Install-Nsis $oldSetup
  if (-not (Find-AppExe)) { Fail "Previous version missing after old NSIS install" }
  Install-Nsis $curSetup
  $exe = Find-AppExe
  if (-not $exe) { Fail "App missing after update install" }
  Launch-Smoke $exe
  Uninstall-Nsis
  if (Find-AppExe) { Fail "App still present after post-update uninstall" }
  $steps += @{ name = "nsis-update-from-previous-launch-uninstall"; status = "PASS" }
  Log "NSIS update path PASS"

  $doc = @{
    schema = "localmotive.sandbox-lifecycle.v0"
    status = "PASS"
    tag = $meta.tag
    version = $meta.version
    previousTag = $meta.previousTag
    finishedAtUtc = (Get-Date).ToUniversalTime().ToString("o")
    steps = $steps
  }
  ($doc | ConvertTo-Json -Depth 6) | Set-Content -Path $ResultPath -Encoding UTF8
  Log "ALL PASS"
  exit 0
} catch {
  Fail $_.Exception.Message
}