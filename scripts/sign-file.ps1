<#
.SYNOPSIS
  Signs one file with Azure Artifact Signing. Called by Tauri's bundler via bundle.windows.signCommand
  (set by scripts/release.ps1), once for each binary and installer it produces.
#>
param([Parameter(Mandatory)][string]$Path)

$ErrorActionPreference = "Stop"

$metadata = $env:REVEAL_SIGNING_METADATA
if (-not $metadata -or -not (Test-Path $metadata)) {
    throw "REVEAL_SIGNING_METADATA is not set; run scripts/release.ps1 instead of signing directly."
}

$signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin" -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match '\\x64\\' } |
    Where-Object { [version]($_.Directory.Parent.Name) -ge [version]"10.0.22621.0" } |
    Sort-Object { [version]($_.Directory.Parent.Name) } -Descending |
    Select-Object -First 1 -ExpandProperty FullName
if (-not $signtool) { throw "signtool 10.0.22621+ not found; install the Windows SDK." }

$dlib = Join-Path $env:LOCALAPPDATA "Microsoft\MicrosoftArtifactSigningClientTools\Azure.CodeSigning.Dlib.dll"
if (-not (Test-Path $dlib)) { throw "Artifact Signing Client Tools not found." }

# signtool reports progress on stderr; judge it by exit code only.
$ErrorActionPreference = "Continue"
& $signtool sign /fd SHA256 /tr "http://timestamp.acs.microsoft.com" /td SHA256 /dlib $dlib /dmdf $metadata $Path
exit $LASTEXITCODE
