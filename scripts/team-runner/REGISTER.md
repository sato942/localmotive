# Register the team runner, verify it, then cut over (P1-12, A2)

Do these steps on the NEW machine, after `provision-team-runner.ps1`.

## 1. Unpack the runner

```powershell
cd C:\actions-runner-team
Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::ExtractToDirectory("actions-runner-win-x64-2.329.0.zip", ".")
```

## 2. Register (human types the token, it is never stored)

Get a token: GitHub → `sato942/localmotive` → Settings → Actions →
Runners → New self-hosted runner → Windows. Then:

```powershell
cd C:\actions-runner-team
.\config.cmd --unattended --url https://github.com/sato942/localmotive --token <TOKEN-FROM-UI> --labels localmotive-release --runasservice
```

The label set is exactly `self-hosted,Windows,X64,localmotive-release`
(the last three come from `--labels`; `self-hosted` is automatic). No
hardware labels: this machine never runs `hardware-qualify.yml`.

## 3. Start and verify

```powershell
.\run.cmd
```

Check the repository Runners page: the new runner is Idle. Then dispatch
`release.yml` once and confirm the jobs land on the new runner (the run
page shows the runner name). Record the run id in TODO.md P1-12.

## 4. Cut over

On the OWNER runner host only: re-configure without the release label so
release jobs stop landing on the owner's PC:

```powershell
cd C:\actions-runner-localmotive
.\config.cmd remove --token <TOKEN-FROM-UI>
.\config.cmd --unattended --url https://github.com/sato942/localmotive --token <TOKEN-FROM-UI> --labels localmotive-hw
```

(`self-hosted,Windows,X64` reattach automatically; the hardware labels
must be retyped — copy them from the Runners page before removing.)

## 5. Rollback

If the team runner fails: re-add `localmotive-release` to the owner
runner with the same remove/re-configure cycle, then remove the team
runner from the Runners page. Release jobs return to the owner host.
Record the reason in TODO.md section 7.
