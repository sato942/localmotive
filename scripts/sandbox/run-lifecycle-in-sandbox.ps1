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
function Get-ProbeReply([string]$text) {
  # CDP may batch several messages (events plus our reply) into one frame, so
  # locate OUR reply by scanning balanced JSON objects for result.result.value
  # instead of assuming one message per frame.
  $index = 0
  while ($true) {
    $start = $text.IndexOf('{"id":1', $index)
    if ($start -lt 0) { return $null }
    $depth = 0
    $i = $start
    $inString = $false
    $escaped = $false
    while ($i -lt $text.Length) {
      $ch = $text[$i]
      if ($inString) {
        if ($escaped) { $escaped = $false }
        elseif ($ch -eq "\") { $escaped = $true }
        elseif ($ch -eq '"') { $inString = $false }
      } elseif ($ch -eq '"') { $inString = $true }
      elseif ($ch -eq "{") { $depth++ }
      elseif ($ch -eq "}") {
        $depth--
        if ($depth -eq 0) { break }
      }
      $i++
    }
    if ($depth -eq 0 -and $i -lt $text.Length) {
      $candidate = $text.Substring($start, $i - $start + 1)
      try {
        $parsed = $candidate | ConvertFrom-Json
        if ($parsed.result -and $parsed.result.result -and $parsed.result.result.value) {
          return $parsed.result.result.value
        }
      } catch { }
    }
    $index = $start + 1
  }
  return $null
}
function Invoke-CdpEvaluate([int]$Port, [string]$Script, [int]$TimeoutSeconds = 60) {
  # Attach to the installed app's page target and evaluate one expression in
  # it; the launch itself is the caller's job. Returns the parsed value.
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  $page = $null
  while ((Get-Date) -lt $deadline -and -not $page) {
    try {
      $tabs = Invoke-RestMethod "http://127.0.0.1:$Port/json/list" -TimeoutSec 5
      $page = $tabs | Where-Object { $_.type -eq "page" -and $_.webSocketDebuggerUrl } | Select-Object -First 1
    } catch { }
    if (-not $page) { Start-Sleep -Milliseconds 500 }
  }
  if (-not $page) { throw "The installed app exposed no CDP page target within $TimeoutSeconds seconds" }
  $ws = New-Object System.Net.WebSockets.ClientWebSocket
  # [void]: the awaited task results must not leak into this function's output
  # (PowerShell would return them alongside the probe value).
  [void]$ws.ConnectAsync([Uri]$page.webSocketDebuggerUrl, [Threading.CancellationToken]::None).GetAwaiter().GetResult()
  try {
    $expression = $Script
    $request = @{ id = 1; method = "Runtime.evaluate"; params = @{ expression = $expression; returnByValue = $true } } | ConvertTo-Json -Compress -Depth 5
    $bytes = [Text.Encoding]::UTF8.GetBytes($request)
    [void]$ws.SendAsync(
      (New-Object System.ArraySegment[byte] -ArgumentList @(, $bytes)),
      [System.Net.WebSockets.WebSocketMessageType]::Text, $true, [Threading.CancellationToken]::None
    ).GetAwaiter().GetResult()
    $value = $null
    $readDeadline = (Get-Date).AddSeconds(15)
    while (-not $value -and (Get-Date) -lt $readDeadline) {
      $buffer = New-Object byte[] 262144
      $received = [System.Net.WebSockets.WebSocketReceiveResult]$ws.ReceiveAsync(
        (New-Object System.ArraySegment[byte] -ArgumentList @(, $buffer)),
        [Threading.CancellationToken]::None
      ).GetAwaiter().GetResult()
      if ($received.MessageType -eq [System.Net.WebSockets.WebSocketMessageType]::Close) { break }
      $value = Get-ProbeReply ([Text.Encoding]::UTF8.GetString($buffer, 0, $received.Count))
    }
    if (-not $value) { throw "CDP evaluate produced no value for the installed app" }
    return ($value | ConvertFrom-Json)
  } finally {
    try { $ws.Dispose() } catch { }
  }
}
function Invoke-CdpProbe([int]$Port, [int]$TimeoutSeconds = 60) {
  # Functional scope for the INSTALLED payload: the real installed executable is
  # already running with the WebView2 debugger; this evaluates the app's own
  # rendered DOM. Process survival alone is not functional evidence; the probe
  # proves the installed bytes boot WebView2, load the bundled assets and render
  # the application shell.
  $expression = '(function(){const buttons=document.querySelectorAll("nav button");const text=(document.body&&document.body.innerText)||"";return JSON.stringify({navButtons:buttons.length,hasLocalmotive:/Localmotive/i.test(text),hasManagedControls:/Stop server|Start profile|HF catalog|Benchmark/i.test(text),bodyChars:text.length});})()'
  return (Invoke-CdpEvaluate $Port $expression $TimeoutSeconds)
}

function Use-SettingsSession($exe, [string]$label, [scriptblock]$Body) {
  # F9-05: one bounded launch of an installed build with the WebView2 debugger,
  # used to seed or read the application's own persisted settings.
  Log "$label settings session: $exe"
  $port = 10093
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$port"
  try {
    $p = Start-Process -FilePath $exe -PassThru
    Start-Sleep -Seconds 8
    if ($p.HasExited) { throw "$label exited during the settings session" }
    & $Body $port
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
  } finally {
    Remove-Item Env:\WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
  }
}

function Seed-SettingsV041([string]$exe) {
  # The baseline (v0.4.1) build writes the fixture's keys through its OWN storage
  # path, so the upgraded build can only recover them by honoring that data.
  $fixturePath = Join-Path $Shared "canary-settings.json"
  if (-not (Test-Path $fixturePath)) { throw "settings fixture missing: $fixturePath" }
  $fixture = Get-Content $fixturePath -Raw | ConvertFrom-Json
  $pairs = @()
  foreach ($property in $fixture.keys.PSObject.Properties) {
    $pairs += @{ key = $property.Name; value = [string]$property.Value }
  }
  if ($pairs.Count -lt 4) { throw "settings fixture carries too few keys: $($pairs.Count)" }
  # Base64 keeps the values (quotes, slashes, braces) out of the expression's
  # quoting rules.
  $payload = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes(($pairs | ConvertTo-Json -Compress -Depth 5)))
  $expression = "(function(){const pairs=JSON.parse(decodeURIComponent(escape(atob('$payload'))));for(const p of pairs){localStorage.setItem(p.key,p.value);}return JSON.stringify({seeded:pairs.length});})()"
  Use-SettingsSession $exe "Preservation baseline" {
    param($port)
    $result = Invoke-CdpEvaluate $port $expression
    Log "Seeded $($result.seeded) v0.4.1 settings keys through the baseline build"
  }
}

function Collect-SettingsReads([string]$exe) {
  # Read the same keys back from the UPGRADED build. The host verifier compares
  # these reads against the fixture: a lost key reads as null, a corrupted value
  # reads differently, and both must fail the leg.
  $fixture = Get-Content (Join-Path $Shared "canary-settings.json") -Raw | ConvertFrom-Json
  $keys = @($fixture.keys.PSObject.Properties | ForEach-Object { $_.Name })
  $payload = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes(($keys | ConvertTo-Json -Compress)))
  $expression = "(function(){const keys=JSON.parse(decodeURIComponent(escape(atob('$payload'))));const reads={};for(const k of keys){reads[k]=localStorage.getItem(k);}return JSON.stringify({reads});})()"
  Use-SettingsSession $exe "Preservation upgrade" {
    param($port)
    $result = Invoke-CdpEvaluate $port $expression
    $reads = $result.reads
    $collected = [ordered]@{ schema = "localmotive.settings-reads.v1"; reads = $reads }
    $collected | ConvertTo-Json -Depth 8 | Set-Content (Join-Path $Shared "collected-settings.json") -Encoding UTF8
    $recovered = @($keys | Where-Object { $null -ne $reads.$_ }).Count
    Log "Collected settings reads from the upgraded build: $recovered/$($keys.Count) keys answered"
  }
}

function Launch-Smoke($exe, [string]$label) {
  # One bounded launch that checks BOTH survival and function: the process
  # must live 8 seconds AND the rendered app must answer a CDP probe.
  Log "$label launch probe: $exe"
  $port = 10093
  $script:lastProbe = $null
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = "--remote-debugging-port=$port"
  try {
    $p = Start-Process -FilePath $exe -PassThru
    Start-Sleep -Seconds 8
    if ($p.HasExited) { throw "$label exited during the launch probe code=$($p.ExitCode)" }
    # The CDP functional probe is best-effort INSIDE the sandbox: sandboxed
    # WebView2 may not expose a debugger target at all. A missing target is
    # recorded as a diagnostic and does NOT by itself fail the leg; the leg's
    # hard gates stay the installed version identity, the installed digest,
    # install/uninstall behavior and the preservation verdict. The functional
    # shell of these exact payload bytes is proven separately on the host
    # (verify_installer_payloads.mjs --functional, bound by the payload digest
    # this leg records).
    try {
      $script:lastProbe = Invoke-CdpProbe $port
    } catch {
      Log "CDP functional probe unavailable in this sandbox: $($_.Exception.Message)"
      $script:lastProbe = [ordered]@{ available = $false; diagnostic = $_.Exception.Message }
    }
    Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2
  } finally {
    Remove-Item Env:\WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS -ErrorAction SilentlyContinue
  }
  if ($script:lastProbe.available -ne $false) {
    if ($script:lastProbe.navButtons -lt 3 -or -not $script:lastProbe.hasLocalmotive -or $script:lastProbe.bodyChars -lt 40) {
      throw "$label rendered no usable application shell: $($script:lastProbe | ConvertTo-Json -Compress)"
    }
    Log "$label launch probe OK: process survived 8 s; rendered shell navButtons=$($script:lastProbe.navButtons) bodyChars=$($script:lastProbe.bodyChars) managedControls=$($script:lastProbe.hasManagedControls)"
  } else {
    Log "$label launch probe: process survived 8 s; CDP target unavailable in this sandbox - $($script:lastProbe.diagnostic)"
  }
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
  Launch-Smoke $exe "NSIS fresh install"
  Uninstall-Nsis
  if (Find-AppExe) { Fail "Localmotive.exe still present after NSIS uninstall" }
  $steps += @{ name = "nsis-fresh-install-launch-uninstall"; status = "PASS"; detail = "installed version $nsisVersion; uninstall removed the executable" }
  Log "NSIS fresh path PASS"

  # 2) MSI fresh install + launch + uninstall
  Install-Msi $curMsi
  $exe = Find-AppExe
  if (-not $exe) { Fail "Localmotive.exe not found after MSI install" }
  $msiVersion = Assert-AppVersion $exe $meta.version "MSI fresh install"
  Launch-Smoke $exe "MSI fresh install"
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
  Launch-Smoke $exe "Post-update install"
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
    # R07/F9-05: plant the REAL baseline persistence for this leg's flavor. The
    # mirror flavor stages a v0.5.0-schema catalog mirror with a user override
    # row; the cache flavor stages the v0.4.1-era cache record at the path the
    # 0.6 application still reads AND seeds the v0.4.1 settings/profile keys
    # through the baseline build's own storage, so the upgrade must recover
    # those values rather than a marker.
    $preservationFlavor = if ($meta.PSObject.Properties.Name -contains "preservation") { [string]$meta.preservation } else { "mirror" }
    if ($preservationFlavor -eq "cache") {
      Seed-SettingsV041 $oldExe
    } else {
      Launch-Smoke $oldExe "Preservation baseline"
    }
    $userData = Join-Path $env:LOCALAPPDATA "io.github.localmotive.app"
    if (-not (Test-Path $userData)) { Fail "User-data dir missing after previous install: $userData" }
    Copy-Item (Join-Path $Shared "canary-userdata.txt") (Join-Path $userData "canary-userdata.txt") -Force
    if ($preservationFlavor -eq "cache") {
      Copy-Item (Join-Path $Shared "canary-catalog-cache.json") (Join-Path $userData "catalog-cache.json") -Force
    } else {
      Copy-Item (Join-Path $Shared "canary-mirror.sqlite") (Join-Path $userData "catalog-mirror.sqlite") -Force
    }
    Install-Nsis $curSetup
    $exe = Find-AppExe
    if (-not $exe) { Fail "App missing after preservation upgrade" }
    $preservedVersion = Assert-AppVersion $exe $meta.version "Preservation upgrade"
    Launch-Smoke $exe "Preservation upgrade"
    $canaryAfter = Join-Path $userData "canary-userdata.txt"
    if (-not (Test-Path $canaryAfter)) { Fail "User-data canary missing after upgrade" }
    if ($preservationFlavor -eq "cache") {
      # F9-05: the upgraded build must report the seeded v0.4.1 values back.
      Collect-SettingsReads $exe
      $collectedSettings = Join-Path $Shared "collected-settings.json"
      if (-not (Test-Path $collectedSettings)) { Fail "Settings reads were not collected after the upgrade" }
    }
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
    $stepDetail = if ($preservationFlavor -eq "cache") {
      "user-data canary survived and the seeded v0.4.1 settings/profile keys were re-read from the upgraded build (collected for host value verification); catalog cache record staged and collected; executable $preservedVersion"
    } else {
      "user-data canary and catalog mirror survived the upgrade; files collected for host verification; executable $preservedVersion"
    }
    $steps += @{ name = "nsis-preservation-from-$($meta.previousTag)"; status = "PASS"; detail = $stepDetail }
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
    installedPayloadNote = "Established payload expectations (R09): every NSIS path (fresh, update, preservation) must install one byte-identical executable, and the update must differ from the previous version - both enforced above. The MSI-installed executable is measured and version-verified on its own path; cross-bundler byte identity between the MSI image and the NSIS image is NOT claimed. No claim is made about sidecar files beyond the executable. Functional scope: the installed executable digest recorded here equals the payload digest that scripts/verify_installer_payloads.mjs --functional launches from the extracted installer payloads and evaluates over CDP (navigation-button count, managed-control text, body length); this leg's own in-sandbox CDP probe is best-effort and may report unavailable, which never upgrades survival into functional qualification."
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