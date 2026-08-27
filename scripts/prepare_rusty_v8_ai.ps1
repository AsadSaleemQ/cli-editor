[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $UpstreamRoot,
    [Parameter(Mandatory)] [string] $DestinationRoot
)

$ErrorActionPreference = 'Stop'
$upstream = [IO.Path]::GetFullPath($UpstreamRoot)
$destination = [IO.Path]::GetFullPath($DestinationRoot)
$version = '150.4.0'
$target = 'x86_64-pc-windows-msvc'
$profile = 'ptrcomp_sandbox_release'
$releaseTag = "rusty-v8-v$version"
$baseUrl = "https://github.com/openai/codex/releases/download/$releaseTag"
$archiveName = "rusty_v8_${profile}_${target}.lib.gz"
$bindingName = "src_binding_${profile}_${target}.rs"
$archiveSha256 = '732ec5da4243aa166799780c8519a5eea6f32f6e47657a323342794dc3c239d6'
$bindingSha256 = 'dabf78ba1faac127660db9862b1d0354175c71b8db2d4fcb5bacbd9c93576b16'
$archiveBytes = 40897677L
$bindingBytes = 40354L

function Get-Sha256([string] $Path) {
    $stream = [IO.File]::OpenRead($Path)
    try {
        $algorithm = [Security.Cryptography.SHA256]::Create()
        try { return ([BitConverter]::ToString($algorithm.ComputeHash($stream))).Replace('-', '').ToLowerInvariant() }
        finally { $algorithm.Dispose() }
    }
    finally { $stream.Dispose() }
}

function Assert-Artifact([string] $Path, [string] $ExpectedSha256, [int64] $ExpectedBytes) {
    $item = Get-Item -LiteralPath $Path
    if ($item.Length -ne $ExpectedBytes) { throw "rusty_v8 artifact size mismatch: $Path" }
    if ((Get-Sha256 $Path) -ne $ExpectedSha256) { throw "rusty_v8 artifact SHA-256 mismatch: $Path" }
}

$lockPath = Join-Path $upstream 'codex-rs\Cargo.lock'
$modulePath = Join-Path $upstream 'MODULE.bazel'
if (-not (Test-Path -LiteralPath $lockPath -PathType Leaf) -or -not (Test-Path -LiteralPath $modulePath -PathType Leaf)) {
    throw 'pinned Codex upstream is missing Cargo.lock or MODULE.bazel'
}
$lockText = [IO.File]::ReadAllText($lockPath)
$v8VersionPattern = '(?ms)^name = "v8"\r?\nversion = "' + [regex]::Escape($version) + '"'
if ($lockText -notmatch $v8VersionPattern) {
    throw "pinned Codex no longer resolves v8 $version"
}
$moduleText = [IO.File]::ReadAllText($modulePath)
if (-not $moduleText.Contains($archiveSha256) -or -not $moduleText.Contains("$baseUrl/$archiveName")) {
    throw 'pinned Codex rusty_v8 MSVC provenance changed'
}

[IO.Directory]::CreateDirectory($destination) | Out-Null
$python = Get-Command python -ErrorAction SilentlyContinue
if ($null -eq $python) { $python = Get-Command python3 -ErrorAction SilentlyContinue }
if ($null -eq $python) { throw 'Python is required to download pinned rusty_v8 artifacts' }

$artifacts = @(
    [pscustomobject]@{ Name = $archiveName; Sha256 = $archiveSha256; Bytes = $archiveBytes },
    [pscustomobject]@{ Name = $bindingName; Sha256 = $bindingSha256; Bytes = $bindingBytes }
)
foreach ($artifact in $artifacts) {
    $path = Join-Path $destination $artifact.Name
    if (Test-Path -LiteralPath $path -PathType Leaf) {
        Assert-Artifact $path $artifact.Sha256 $artifact.Bytes
        continue
    }
    $temporary = "$path.download.$PID.$([Guid]::NewGuid().ToString('N'))"
    try {
        $url = "$baseUrl/$($artifact.Name)"
        & $python.Source -c 'import sys, urllib.request; urllib.request.urlretrieve(sys.argv[1], sys.argv[2])' $url $temporary
        if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $temporary -PathType Leaf)) {
            throw "rusty_v8 download failed: $url"
        }
        Assert-Artifact $temporary $artifact.Sha256 $artifact.Bytes
        [IO.File]::Move($temporary, $path)
    }
    finally {
        if (Test-Path -LiteralPath $temporary -PathType Leaf) { [IO.File]::Delete($temporary) }
    }
}

[pscustomobject]@{
    Version = $version
    ArchivePath = (Join-Path $destination $archiveName)
    BindingPath = (Join-Path $destination $bindingName)
}
