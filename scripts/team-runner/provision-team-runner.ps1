<#nodoc>
.SYNOPSIS
  Provisions a team-controlled self-hosted runner machine (P1-12, A2).
.DESCRIPTION
  Idempotent. Run in an elevated PowerShell on the NEW machine, never on
  the owner's PC (it refuses DESKTOP-HPTF57N). Installs Git, Node.js LTS
  and Rustup via winget, creates C:\actions-runner-team with team-local
  CARGO_HOME/RUSTUP_HOME, and downloads the latest GitHub runner.
  Supports -WhatIf dry-run. Takes no token and stores no secret.
#>
[CmdletBinding(SupportsShouldProcess = $true)]
param(
  [string]$Root = "C:\actions-runner-team"
)

$ErrorActionPreference = "Stop"

if ($env:COMPUTERNAME -eq "DESKTOP-HPTF57N") {
  throw "Refusing to provision on the owner runner host. Run this on the new team machine."
}

function Ensure-WingetPackage([string]$id) {
  $found = winget list --id $id --exact 2>$null | Select-String $id
  if ($found) { Write-Output "present: $id"; return }
  if ($PSCmdlet.ShouldProcess($id, "winget install")) {
    winget install --exact --id $id --accept-source-agreements --accept-package-agreements
  }
}

Ensure-WingetPackage "Git.Git"
Ensure-WingetPackage "OpenJS.NodeJS.LTS"
Ensure-WingetPackage "Rustlang.Rustup"

if ($PSCmdlet.ShouldProcess($Root, "create runner root")) {
  New-Item -ItemType Directory -Force -Path $Root | Out-Null
  New-Item -ItemType Directory -Force -Path "$Root\.cargo" | Out-Null
  New-Item -ItemType Directory -Force -Path "$Root\.rustup" | Out-Null
}

if ($PSCmdlet.ShouldProcess("machine environment", "set team CARGO_HOME/RUSTUP_HOME/RUNNER_TEMP")) {
  [Environment]::SetEnvironmentVariable("CARGO_HOME", "$Root\.cargo", "Machine")
  [Environment]::SetEnvironmentVariable("RUSTUP_HOME", "$Root\.rustup", "Machine")
  [Environment]::SetEnvironmentVariable("RUNNER_TEMP", "$Root\_temp", "Machine")
  New-Item -ItemType Directory -Force -Path "$Root\_temp" | Out-Null
}

$runnerVersion = "2.329.0"
$runnerZip = "$Root\actions-runner-win-x64-$runnerVersion.zip"
if (-not (Test-Path $runnerZip)) {
  if ($PSCmdlet.ShouldProcess($runnerZip, "download runner $runnerVersion")) {
    Invoke-WebRequest -Uri "https://github.com/actions/runner/releases/download/v$runnerVersion/actions-runner-win-x64-$runnerVersion.zip" -OutFile $runnerZip
  }
} else {
  Write-Output "present: runner $runnerVersion zip"
}

Write-Output "Provision done. Next: REGISTER.md step 2 (config.cmd with your token)."
