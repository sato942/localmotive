param(
  [Parameter(Mandatory = $true)][string]$FilePath
)
$ErrorActionPreference = "Stop"
$thumbprint = $env:LOCALMOTIVE_SIGNING_THUMBPRINT
$timestampUrl = $env:LOCALMOTIVE_TIMESTAMP_URL
if ($thumbprint -notmatch '^[0-9A-Fa-f]{40}$') {
  throw "LOCALMOTIVE_SIGNING_THUMBPRINT must identify one CurrentUser\\My certificate"
}
if ($timestampUrl -notmatch '^https://') {
  throw "LOCALMOTIVE_TIMESTAMP_URL must be an HTTPS timestamp endpoint"
}
$certificate = Get-Item -LiteralPath "Cert:\CurrentUser\My\$thumbprint" -ErrorAction Stop
if (-not $certificate.HasPrivateKey) {
  throw "The selected code-signing certificate has no accessible private key"
}
if ($certificate.NotAfter -le [DateTime]::UtcNow) {
  throw "The selected code-signing certificate is expired"
}
$codeSigningOid = "1.3.6.1.5.5.7.3.3"
$hasCodeSigningUsage = $certificate.EnhancedKeyUsageList.ObjectId.Value -contains $codeSigningOid
if (-not $hasCodeSigningUsage) {
  throw "The selected certificate is not valid for code signing"
}
$resolvedFile = (Resolve-Path -LiteralPath $FilePath).Path
$signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
  Sort-Object FullName |
  Select-Object -Last 1
if (-not $signtool) {
  throw "signtool.exe is unavailable"
}
& $signtool.FullName sign /sha1 $thumbprint /fd SHA256 /tr $timestampUrl /td SHA256 /d "Localmotive" $resolvedFile
if ($LASTEXITCODE -ne 0) {
  throw "Authenticode signing failed"
}
& $signtool.FullName verify /pa /all /v $resolvedFile
if ($LASTEXITCODE -ne 0) {
  throw "Authenticode verification failed after signing"
}
$signature = Get-AuthenticodeSignature -LiteralPath $resolvedFile
if ($signature.Status -ne [System.Management.Automation.SignatureStatus]::Valid) {
  throw "PowerShell did not report a valid Authenticode signature"
}
if ($signature.SignerCertificate.Thumbprint -ne $thumbprint) {
  throw "The signed file does not use the approved certificate thumbprint"
}
if ($null -eq $signature.TimeStamperCertificate) {
  throw "The signed file has no trusted timestamp certificate"
}
