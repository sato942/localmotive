# Start the team runner interactively (template — copy next to the runner root)
$RunnerRoot = "C:\actions-runner-team"
Start-Process -FilePath "$RunnerRoot\run.cmd" -WorkingDirectory $RunnerRoot -WindowStyle Hidden
