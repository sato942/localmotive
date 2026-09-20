param([string]$OutputDir = 'artifacts/ideas-audit-20260919-2202')
$ErrorActionPreference = 'Stop'
$paths = @(
  'src-tauri/target/release/localmotive.exe',
  'src-tauri/target/release/bundle/msi/Localmotive_0.6.0_x64_en-US.msi',
  'src-tauri/target/release/bundle/nsis/Localmotive_0.6.0_x64-setup.exe'
)
$records = foreach ($path in $paths) {
  $file = Get-Item -LiteralPath $path
  $signature = Get-AuthenticodeSignature -LiteralPath $path
  $version = $file.VersionInfo.ProductVersion
  if ($file.Extension -eq '.msi') {
    $installer = New-Object -ComObject WindowsInstaller.Installer
    $database = $installer.OpenDatabase($file.FullName, 0)
    $view = $database.OpenView("SELECT ``Value`` FROM ``Property`` WHERE ``Property``='ProductVersion'")
    [void]$view.Execute()
    $row = $view.Fetch()
    $version = $row.StringData(1)
    [void]$view.Close()
    foreach ($item in @($row, $view, $database, $installer)) {
      [void][Runtime.InteropServices.Marshal]::FinalReleaseComObject($item)
    }
  }
  [pscustomobject]@{
    path = $file.FullName
    bytes = $file.Length
    sha256 = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    signature = $signature.Status.ToString()
    version = $version
  }
}
$records | ConvertTo-Json | Set-Content (Join-Path $OutputDir 'built-artifacts.json')
$records | ConvertTo-Json
Get-Date -Format o
