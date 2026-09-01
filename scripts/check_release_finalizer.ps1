[CmdletBinding()]
param([Parameter(Mandatory)] [string] $SignerPath)

$ErrorActionPreference = 'Stop'
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$builderSource = [IO.File]::ReadAllText((Join-Path $repository 'scripts\build_release.ps1'))
$manifestWriteIndex = $builderSource.IndexOf('[IO.File]::WriteAllText($manifestPath', [StringComparison]::Ordinal)
$manifestHashIndex = $builderSource.IndexOf('$manifestDigest = Get-Sha256 $manifestPath', [StringComparison]::Ordinal)
if ($manifestWriteIndex -lt 0 -or $manifestHashIndex -lt 0 -or $manifestWriteIndex -gt $manifestHashIndex) {
    throw 'release builder must serialize compatibility-manifest.json before hashing it for the SBOM'
}
$caseName = "finalizer-$PID-$([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds())"
$caseRoot = Join-Path $repository ".work\$caseName"
$trashRoot = Join-Path $repository '.trash'
$version = '0.1.1'
$issued = 1800000000
$sequence = 1
$expires = $issued + 86400
$artifactsRoot = Join-Path $caseRoot '.artifacts'
$bundle = Join-Path $caseRoot ".artifacts\codex-cli-editor-$version-windows-x64"
$releaseTools = Join-Path $caseRoot '.artifacts\release-tools'
$secondName = $null
$secondRoot = $null
$duplicateName = $null
$duplicateRoot = $null

function Get-Sha256([string] $Path) {
    $stream = [IO.File]::OpenRead($Path)
    try {
        $algorithm = [Security.Cryptography.SHA256]::Create()
        try { return ([BitConverter]::ToString($algorithm.ComputeHash($stream))).Replace('-', '').ToLowerInvariant() }
        finally { $algorithm.Dispose() }
    }
    finally { $stream.Dispose() }
}

$savedSeed = $env:CODEX_CLI_EDITOR_SIGNING_PRIVATE_KEY_HEX
try {
    [IO.Directory]::CreateDirectory($bundle) | Out-Null
    [IO.Directory]::CreateDirectory($releaseTools) | Out-Null
    [IO.Directory]::CreateDirectory((Join-Path $caseRoot 'compatibility')) | Out-Null
    [IO.File]::Copy((Resolve-Path $SignerPath), (Join-Path $releaseTools 'sign_release.exe'), $false)
    [IO.File]::Copy((Join-Path $repository 'compatibility\manifest.schema.json'), (Join-Path $caseRoot 'compatibility\manifest.schema.json'), $false)
    [IO.File]::WriteAllText((Join-Path $caseRoot 'compatibility\public-key.hex'), "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a`n", (New-Object Text.UTF8Encoding($false)))
    foreach ($name in @('codex-cli-editor.exe', 'codex-enhanced.exe', 'codex-code-mode-host.exe', 'codex-cli-editor.vsix')) {
        [IO.File]::WriteAllText((Join-Path $bundle $name), "fixture-$name", (New-Object Text.UTF8Encoding($false)))
    }
    foreach ($name in @('LICENSE', 'NOTICE', 'THIRD_PARTY_NOTICES.md', 'THIRD_PARTY_LICENSES_CODEX_CLI_EDITOR.html', 'THIRD_PARTY_LICENSES_CODEX.html')) {
        [IO.File]::WriteAllText((Join-Path $bundle $name), "fixture-$name", (New-Object Text.UTF8Encoding($false)))
    }
    [IO.File]::WriteAllText((Join-Path $artifactsRoot "codex-cli-editor-$version.sbom.json"), '{}', (New-Object Text.UTF8Encoding($false)))
    [IO.File]::WriteAllText((Join-Path $artifactsRoot "codex-cli-editor-$version-source.zip"), 'fixture-source', (New-Object Text.UTF8Encoding($false)))
    $artifacts = foreach ($name in @('codex-cli-editor.exe', 'codex-enhanced.exe', 'codex-code-mode-host.exe', 'codex-cli-editor.vsix')) {
        $file = Get-Item -LiteralPath (Join-Path $bundle $name)
        [ordered]@{ name = $name; url = "https://example.invalid/$name"; sha256 = Get-Sha256 $file.FullName; size = [uint64]$file.Length }
    }
    $manifest = [ordered]@{
        schema_version = 1
        sequence = $sequence
        issued_unix = $issued
        expires_unix = $expires
        minimum_dispatcher_version = $version
        compatibility = @([ordered]@{ codex = '0.148.0'; vscode = @('fixture') })
        artifacts = @($artifacts)
    }
    [IO.File]::WriteAllText((Join-Path $bundle 'compatibility-manifest.json'), ($manifest | ConvertTo-Json -Depth 8) + "`n", (New-Object Text.UTF8Encoding($false)))
    $secondName = "$caseName-second"
    $secondRoot = Join-Path $repository ".work\$secondName"
    Copy-Item -LiteralPath $caseRoot -Destination $secondRoot -Recurse
    $env:CODEX_CLI_EDITOR_SIGNING_PRIVATE_KEY_HEX = '9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60'
    $duplicateName = "$caseName-duplicate-artifact"
    $duplicateRoot = Join-Path $repository ".work\$duplicateName"
    Copy-Item -LiteralPath $caseRoot -Destination $duplicateRoot -Recurse
    $duplicateManifestPath = Join-Path $duplicateRoot ".artifacts\codex-cli-editor-$version-windows-x64\compatibility-manifest.json"
    $duplicateManifest = [IO.File]::ReadAllText($duplicateManifestPath) | ConvertFrom-Json
    $duplicateManifest.artifacts[2] = $duplicateManifest.artifacts[1]
    [IO.File]::WriteAllText($duplicateManifestPath, ($duplicateManifest | ConvertTo-Json -Depth 8) + "`n", (New-Object Text.UTF8Encoding($false)))
    $duplicateRejected = $false
    try {
        & (Join-Path $repository 'scripts\finalize_release.ps1') -Version $version -IssuedUnix $issued -ManifestSequence $sequence -ExpiresUnix $expires -RepositoryRoot $duplicateRoot
    } catch {
        if ($_.Exception.Message -eq 'manifest artifact names must match the exact unique release inventory') { $duplicateRejected = $true }
        else { throw }
    }
    if (-not $duplicateRejected) { throw 'release finalizer accepted duplicate artifact names' }
    & (Join-Path $repository 'scripts\finalize_release.ps1') -Version $version -IssuedUnix $issued -ManifestSequence $sequence -ExpiresUnix $expires -RepositoryRoot $caseRoot
    & (Join-Path $repository 'scripts\finalize_release.ps1') -Version $version -IssuedUnix $issued -ManifestSequence $sequence -ExpiresUnix $expires -RepositoryRoot $secondRoot
    $firstArchive = Join-Path $caseRoot ".artifacts\codex-cli-editor-$version-windows-x64.zip"
    $secondArchive = Join-Path $secondRoot ".artifacts\codex-cli-editor-$version-windows-x64.zip"
    if ((Get-Sha256 $firstArchive) -ne (Get-Sha256 $secondArchive)) { throw 'independent finalizer ZIPs are not deterministic' }
    foreach ($path in @(
        (Join-Path $bundle 'compatibility-manifest.sig'),
        (Join-Path $caseRoot ".artifacts\codex-cli-editor-$version-windows-x64.zip"),
        (Join-Path $caseRoot ".artifacts\codex-cli-editor-$version-windows-x64.zip.sha256")
    )) {
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "finalizer output is missing: $path" }
    }
    Write-Output 'Release finalizer fixture passed'
} finally {
    if ($null -eq $savedSeed) { Remove-Item Env:CODEX_CLI_EDITOR_SIGNING_PRIVATE_KEY_HEX -ErrorAction SilentlyContinue }
    else { $env:CODEX_CLI_EDITOR_SIGNING_PRIVATE_KEY_HEX = $savedSeed }
    foreach ($entry in @(@{ Root = $caseRoot; Name = $caseName }, @{ Root = $secondRoot; Name = $secondName }, @{ Root = $duplicateRoot; Name = $duplicateName })) {
        if ($entry.Root -and (Test-Path -LiteralPath $entry.Root)) {
            [IO.Directory]::CreateDirectory($trashRoot) | Out-Null
            Move-Item -LiteralPath $entry.Root -Destination (Join-Path $trashRoot $entry.Name)
        }
    }
}
