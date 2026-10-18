<#
.SYNOPSIS
  Builds a signed Reveal release.

.DESCRIPTION
  Runs `tauri build` with bundle.windows.signCommand pointed at scripts/sign-file.ps1, so Tauri
  signs reveal.exe and the NSIS installer with Azure Artifact Signing before it creates the
  updater .nsis.zip and signs that with the Tauri updater key. Then verifies every signature,
  including the installer inside the updater archive.

  One-time setup on the build machine:
    - Azure CLI, logged in with `az login` as a user with the "Artifact Signing
      Certificate Profile Signer" role on the signing account.
    - Artifact Signing Client Tools (winget: Microsoft.Azure.ArtifactSigningClientTools).
    - Windows SDK signtool 10.0.22621 or newer.
    - Tauri updater key at %USERPROFILE%\.reveal\updater.key and its password in the
      REVEAL_UPDATER_KEY_PASSWORD environment variable. Never commit either.

  Tauri splits signCommand on spaces, so the repository path must not contain spaces.
#>
[CmdletBinding()]
param(
    [string]$Endpoint = "https://eus.codesigning.azure.net/",
    [string]$AccountName = "steele",
    [string]$CertificateProfile = "reveal",
    [string]$UpdaterKeyPath = (Join-Path $env:USERPROFILE ".reveal\updater.key")
)

$ErrorActionPreference = "Stop"
$root = Split-Path $PSScriptRoot -Parent
$tauriDir = Join-Path $root "src-tauri"
$version = (Get-Content (Join-Path $tauriDir "tauri.conf.json") -Raw | ConvertFrom-Json).package.version
$exe = Join-Path $tauriDir "target\release\reveal.exe"
$installer = Join-Path $tauriDir "target\release\bundle\nsis\Reveal_${version}_x64-setup.exe"
$updaterZip = $installer.Replace("-setup.exe", "-setup.nsis.zip")

function Step($message) { Write-Host "`n==> $message" -ForegroundColor Cyan }

# Windows PowerShell's "-Encoding utf8" adds a BOM, which the signing client and Tauri reject.
function Write-Utf8NoBom($path, $text) {
    [IO.File]::WriteAllText($path, $text, (New-Object Text.UTF8Encoding $false))
}

function Invoke-Checked {
    param([string]$FilePath, [string[]]$Arguments)
    # Windows PowerShell turns any stderr line from a native tool into an error under "Stop",
    # and pnpm/cargo/signtool print progress there. Judge them by exit code instead.
    $previous = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    try { & $FilePath @Arguments 2>&1 | ForEach-Object { "$_" } }
    finally { $ErrorActionPreference = $previous }
    if ($LASTEXITCODE -ne 0) { throw "$FilePath exited with code $LASTEXITCODE" }
}

function Assert-Signed($path) {
    $signature = Get-AuthenticodeSignature $path
    if ($signature.Status -ne "Valid") { throw "$(Split-Path $path -Leaf) signature is $($signature.Status)" }
    Write-Host ("  {0}: signed by {1}" -f (Split-Path $path -Leaf), $signature.SignerCertificate.Subject)
}

# --- Preflight -----------------------------------------------------------------------------
Step "Checking signing setup for Reveal v$version"

if ($root -match ' ') { throw "The repository path contains a space, which Tauri's signCommand can't handle: $root" }
if (-not (Test-Path (Join-Path $env:LOCALAPPDATA "Microsoft\MicrosoftArtifactSigningClientTools\Azure.CodeSigning.Dlib.dll"))) {
    throw "Artifact Signing Client Tools not found; run: winget install Microsoft.Azure.ArtifactSigningClientTools"
}
if (-not (Get-Command az -ErrorAction SilentlyContinue)) { throw "Azure CLI not found; run: winget install Microsoft.AzureCLI" }
try { $null = Invoke-Checked "az" @("account", "show") } catch { throw "Not logged in to Azure; run: az login" }

$runningBuild = Get-Process reveal -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq $exe }
if ($runningBuild) { throw "Close the running test build ($exe) first; it locks the file." }

if (-not (Test-Path $UpdaterKeyPath)) { throw "Updater key not found at $UpdaterKeyPath" }
if (-not $env:REVEAL_UPDATER_KEY_PASSWORD) { throw "Set REVEAL_UPDATER_KEY_PASSWORD to the updater key's password." }

# --- Signing configuration -----------------------------------------------------------------
# Only the Azure CLI login is used to authenticate; skip the slower credential probes.
$metadata = Join-Path $env:TEMP "reveal-signing-metadata.json"
Write-Utf8NoBom $metadata (@{
    Endpoint               = $Endpoint
    CodeSigningAccountName = $AccountName
    CertificateProfileName = $CertificateProfile
    ExcludeCredentials     = @(
        "ManagedIdentityCredential", "EnvironmentCredential", "WorkloadIdentityCredential",
        "SharedTokenCacheCredential", "VisualStudioCredential", "VisualStudioCodeCredential",
        "AzurePowerShellCredential", "AzureDeveloperCliCredential", "InteractiveBrowserCredential"
    )
} | ConvertTo-Json)

$signFile = Join-Path $PSScriptRoot "sign-file.ps1"
$signConfig = Join-Path $env:TEMP "reveal-sign-config.json"
Write-Utf8NoBom $signConfig (@{
    tauri = @{ bundle = @{ windows = @{
        signCommand = "powershell -NoProfile -ExecutionPolicy Bypass -File $signFile %1"
    } } }
} | ConvertTo-Json -Depth 5)

# --- Build, sign and bundle ----------------------------------------------------------------
Step "Building and signing"
$env:REVEAL_SIGNING_METADATA = $metadata
$env:TAURI_PRIVATE_KEY = (Get-Content $UpdaterKeyPath -Raw).Trim()
$env:TAURI_KEY_PASSWORD = $env:REVEAL_UPDATER_KEY_PASSWORD
Remove-Item $installer, $updaterZip, "$updaterZip.sig" -ErrorAction SilentlyContinue
Push-Location $root
try { Invoke-Checked "pnpm" @("tauri", "build", "--config", $signConfig) }
finally {
    Pop-Location
    Remove-Item Env:TAURI_PRIVATE_KEY, Env:TAURI_KEY_PASSWORD, Env:REVEAL_SIGNING_METADATA -ErrorAction SilentlyContinue
}

# --- Verify ----------------------------------------------------------------------------------
Step "Verifying signatures"
Assert-Signed $exe
Assert-Signed $installer

# The updater archive must carry the signed installer, or auto-updates would ship it unsigned.
$extracted = Join-Path $env:TEMP "reveal-updater-check"
Remove-Item $extracted -Recurse -Force -ErrorAction SilentlyContinue
Expand-Archive -Path $updaterZip -DestinationPath $extracted
Assert-Signed (Join-Path $extracted (Split-Path $installer -Leaf))
Remove-Item $extracted -Recurse -Force
if (-not (Test-Path "$updaterZip.sig")) { throw "Updater signature was not created." }

Step "Done: Reveal v$version"
Get-Item $installer, $updaterZip, "$updaterZip.sig" | ForEach-Object { "  {0}  {1:N0} bytes" -f $_.FullName, $_.Length }
