[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $UpstreamRoot,
    [Parameter(Mandatory)] [string] $WorkRoot,
    [string] $Toolchain = '1.95.0-x86_64-pc-windows-msvc'
)

$ErrorActionPreference = 'Stop'
$manifest = Join-Path $UpstreamRoot 'codex-rs\Cargo.toml'
$lockfile = Join-Path $UpstreamRoot 'codex-rs\Cargo.lock'
if (-not (Test-Path -LiteralPath $manifest -PathType Leaf)) { throw "Missing upstream manifest: $manifest" }
if (-not (Test-Path -LiteralPath $lockfile -PathType Leaf)) { throw "Missing upstream lockfile: $lockfile" }

$metadataJson = & cargo "+$Toolchain" metadata --locked --manifest-path $manifest --format-version 1
if ($LASTEXITCODE -ne 0) { throw 'Unable to locate the pinned i18n-embed-fl source.' }
$metadata = $metadataJson | ConvertFrom-Json
$package = @($metadata.packages | Where-Object {
    $_.name -eq 'i18n-embed-fl' -and
    $_.version -eq '0.9.4' -and
    $_.source -like 'registry+*'
})
if ($package.Count -ne 1) { throw "Expected one registry i18n-embed-fl 0.9.4 package, found $($package.Count)." }

$sourceRoot = Split-Path -Parent ([string]$package[0].manifest_path)
$sourceFile = Join-Path $sourceRoot 'src\lib.rs'
if (-not (Test-Path -LiteralPath $sourceFile -PathType Leaf)) { throw "Missing pinned macro source: $sourceFile" }
$sha256 = [Security.Cryptography.SHA256]::Create()
try {
    $sourceHash = ([BitConverter]::ToString($sha256.ComputeHash([IO.File]::ReadAllBytes($sourceFile)))).Replace('-', '').ToLowerInvariant()
} finally {
    $sha256.Dispose()
}
if ($sourceHash -ne 'cfc7f5ef3efae615f3f8dc3cd04d5ea0c551a1f005d93e55d7370ac9cb7c2cea') {
    throw "Pinned i18n-embed-fl source hash changed: $sourceHash"
}

$vendorRoot = Join-Path $WorkRoot 'vendor\i18n-embed-fl-0.9.4'
if (Test-Path -LiteralPath $vendorRoot) { Remove-Item -LiteralPath $vendorRoot -Recurse -Force }
[IO.Directory]::CreateDirectory((Split-Path -Parent $vendorRoot)) | Out-Null
Copy-Item -LiteralPath $sourceRoot -Destination $vendorRoot -Recurse

$vendoredSource = Join-Path $vendorRoot 'src\lib.rs'
$text = [IO.File]::ReadAllText($vendoredSource).Replace("`r`n", "`n")
$needle = "            let mut arg_assignments = proc_macro2::TokenStream::default();`n            for (key, value) in &specified_args {"
$replacement = "            let mut arg_assignments = proc_macro2::TokenStream::default();`n            let mut sorted_specified_args: Vec<_> = specified_args.iter().collect();`n            sorted_specified_args.sort_by_key(|(key, _)| key.value());`n            for (key, value) in sorted_specified_args {"
if (($text.Split(@($needle), [StringSplitOptions]::None).Count - 1) -ne 1) {
    throw 'Expected exactly one non-deterministic i18n-embed-fl expansion loop.'
}
$text = $text.Replace($needle, $replacement)
[IO.File]::WriteAllText($vendoredSource, $text, [Text.UTF8Encoding]::new($false))

$manifestText = [IO.File]::ReadAllText($manifest)
$patchHeader = '(?m)^\[patch\.crates-io\](?=\r?$)'
if ([regex]::Matches($manifestText, $patchHeader).Count -ne 1) { throw 'Expected one upstream [patch.crates-io] section.' }
if ($manifestText -match '(?m)^i18n-embed-fl\s*=') { throw 'Upstream manifest already patches i18n-embed-fl.' }
$vendorTomlPath = $vendorRoot.Replace('\', '/')
$patchEntry = "[patch.crates-io]`ni18n-embed-fl = { path = `"$vendorTomlPath`" }"
$manifestText = [regex]::Replace($manifestText, $patchHeader, $patchEntry, 1)
[IO.File]::WriteAllText($manifest, $manifestText, [Text.UTF8Encoding]::new($false))

$beforeLock = [IO.File]::ReadAllText($lockfile)
$beforePattern = '(?ms)(\[\[package\]\]\r?\nname = "i18n-embed-fl"\r?\nversion = "0\.9\.4"\r?\n)source = "registry\+https://github\.com/rust-lang/crates\.io-index"\r?\nchecksum = "04b2969d0b3fc6143776c535184c19722032b43e6a642d710fa3f88faec53c2d"\r?\n'
if ([regex]::Matches($beforeLock, $beforePattern).Count -ne 1) { throw 'Pinned i18n-embed-fl lock entry changed.' }

$null = & cargo "+$Toolchain" metadata --manifest-path $manifest --format-version 1
if ($LASTEXITCODE -ne 0) { throw 'Unable to activate deterministic i18n-embed-fl source patch.' }
$afterLock = [IO.File]::ReadAllText($lockfile)
$expectedLock = [regex]::Replace($beforeLock, $beforePattern, '$1', 1)
if ($afterLock -ne $expectedLock) { throw 'Activating the deterministic macro patch changed more than the pinned package source.' }

Write-Output 'Activated pinned deterministic i18n-embed-fl 0.9.4 macro expansion patch.'