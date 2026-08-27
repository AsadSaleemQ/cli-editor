[CmdletBinding()]
param(
    [Parameter(Mandatory)] [string] $OutputPath
)

$ErrorActionPreference = 'Stop'
$root = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$extensionRoot = Join-Path $root 'vscode-extension'
$package = Get-Content -Raw -LiteralPath (Join-Path $extensionRoot 'package.json') | ConvertFrom-Json
if ($package.publisher -ne 'asadsaleemq' -or $package.name -ne 'cli-editor-vscode') {
    throw 'Unexpected VS Code extension identity'
}
$destination = [IO.Path]::GetFullPath($OutputPath)
if (Test-Path -LiteralPath $destination) { throw "Refusing to overwrite VSIX: $destination" }
[IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($destination)) | Out-Null
$manifest = @"
<?xml version="1.0" encoding="utf-8"?>
<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011">
  <Metadata>
    <Identity Language="en-US" Id="$($package.publisher).$($package.name)" Version="$($package.version)" Publisher="$($package.publisher)" />
    <DisplayName>$($package.displayName)</DisplayName>
    <Description xml:space="preserve">$($package.description)</Description>
    <Tags>terminal,codex,cli-editor</Tags>
    <Categories>Other</Categories>
    <GalleryFlags>Public</GalleryFlags>
    <Properties>
      <Property Id="Microsoft.VisualStudio.Code.Engine" Value="$($package.engines.vscode)" />
      <Property Id="Microsoft.VisualStudio.Services.Links.Source" Value="https://github.com/AsadSaleemQ/cli-editor" />
    </Properties>
  </Metadata>
  <Installation><InstallationTarget Id="Microsoft.VisualStudio.Code" /></Installation>
  <Dependencies />
  <Assets>
    <Asset Type="Microsoft.VisualStudio.Code.Manifest" Path="extension/package.json" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Content.Details" Path="extension/README.md" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Content.License" Path="extension/LICENSE" Addressable="true" />
  </Assets>
</PackageManifest>
"@
$readme = "# CLI Editor Terminal Bridge`n`nBundled companion for [CLI Editor](https://github.com/AsadSaleemQ/cli-editor). It preserves VS Code terminal defaults unless the active CLI Editor prompt owns input.`n"
$contentTypes = @"
<?xml version="1.0" encoding="utf-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="json" ContentType="application/json" />
  <Default Extension="js" ContentType="application/javascript" />
  <Default Extension="md" ContentType="text/markdown" />
  <Override PartName="/extension.vsixmanifest" ContentType="text/xml" />
  <Override PartName="/extension/LICENSE" ContentType="text/plain" />
</Types>
"@
Add-Type -AssemblyName System.IO.Compression
$stream = [IO.File]::Open($destination, [IO.FileMode]::CreateNew, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
try {
    $archive = New-Object IO.Compression.ZipArchive($stream, [IO.Compression.ZipArchiveMode]::Create, $true)
    try {
        $entries = [ordered]@{
            'extension/package.json' = [IO.File]::ReadAllText((Join-Path $extensionRoot 'package.json'))
            'extension/extension.js' = [IO.File]::ReadAllText((Join-Path $extensionRoot 'extension.js'))
            'extension/LICENSE' = [IO.File]::ReadAllText((Join-Path $root 'LICENSE'))
            'extension/README.md' = $readme
            'extension.vsixmanifest' = $manifest
            '[Content_Types].xml' = $contentTypes
        }
        foreach ($item in $entries.GetEnumerator()) {
            $entry = $archive.CreateEntry($item.Key, [IO.Compression.CompressionLevel]::Optimal)
            $entry.LastWriteTime = [DateTimeOffset]::new(1980, 1, 1, 0, 0, 0, [TimeSpan]::Zero)
            $writer = New-Object IO.StreamWriter($entry.Open(), (New-Object Text.UTF8Encoding($false)))
            try { $writer.Write(($item.Value -replace "`r`n", "`n")) } finally { $writer.Dispose() }
        }
    }
    finally { $archive.Dispose() }
}
finally { $stream.Dispose() }
Write-Output $destination