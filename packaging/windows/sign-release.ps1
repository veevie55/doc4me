<#
.SYNOPSIS
  Package and sign an existing Windows build on a separate release runner.

.DESCRIPTION
  The checkout must match SourceCommit. ArtifactDir is the downloaded build artifact for Arch.
  Only the two PE files and the portable bundle's font notices are taken from the archive.
  The packaging scripts and installer definition come from this checkout.
#>
[CmdletBinding()]
param(
  [Parameter(Mandatory)] [ValidateSet('x64', 'x86', 'arm64')] [string] $Arch,
  [Parameter(Mandatory)] [string] $ArtifactDir,
  [Parameter(Mandatory)] [string] $Dist,
  [Parameter(Mandatory)] [ValidatePattern('^[0-9a-f]{40}\z')] [string] $SourceCommit,
  [Parameter(Mandatory)] [ValidateLength(1, 64)]
  [ValidatePattern('^(0|[1-9][0-9]{0,4})\.(0|[1-9][0-9]{0,4})\.(0|[1-9][0-9]{0,4})(-[0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*)?\z')]
  [string] $Version,
  [Parameter(Mandatory)] [ValidateSet('commit', 'version', 'version-commit', 'none')] [string] $FileName
)
$ErrorActionPreference = 'Stop'
$Root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$checkedOut = git -C $Root rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $checkedOut -cne $SourceCommit) {
  throw 'The signing checkout does not match the release source commit'
}

$ArtifactDir = (Resolve-Path -LiteralPath $ArtifactDir).Path
foreach ($record in @{ 'SOURCE_COMMIT.txt' = $SourceCommit; 'VERSION.txt' = $Version }.GetEnumerator()) {
  $recordPath = Join-Path $ArtifactDir $record.Key
  if ([IO.File]::ReadAllText($recordPath).Trim() -cne $record.Value) {
    throw "Build artifact $($record.Key) does not match the release request"
  }
}

$shortCommit = $SourceCommit.Substring(0, 8)
$label = switch ($FileName) {
  'commit' { "-$shortCommit" }
  'version' { "-$Version" }
  'version-commit' { "-$Version-$shortCommit" }
  'none' { '' }
}
$stem = "doc4me$label-windows-$Arch"
$zipName = "$stem-portable.zip"
$target = switch ($Arch) {
  'x64' { 'x86_64-pc-windows-msvc' }
  'x86' { 'i686-pc-windows-msvc' }
  'arm64' { 'aarch64-pc-windows-msvc' }
}
$temp = Join-Path ([IO.Path]::GetTempPath()) "doc4me-release-$([guid]::NewGuid())"
$bin = Join-Path $temp "target/$target/release"
$notices = Join-Path $temp 'notices'
$packages = Join-Path $temp 'packages'
$exes = 'doc4me.exe', 'doc4me-cli.exe'
try {
  New-Item -ItemType Directory -Path $bin, $notices, $packages -Force | Out-Null
  $archive = [IO.Compression.ZipFile]::OpenRead((Join-Path $ArtifactDir $zipName))
  try {
    $prefix = "$stem-portable/"
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($entry in $archive.Entries) {
      $path = $entry.FullName.Replace('\', '/')
      if (-not $path.StartsWith($prefix, [StringComparison]::Ordinal)) { continue }
      $name = $path.Substring($prefix.Length)
      if ($exes -ccontains $name) {
        $dest = Join-Path $bin $name
      } elseif ($name -cmatch '^OFL-[A-Za-z0-9][A-Za-z0-9_.-]*\.txt\z') {
        $dest = Join-Path $notices $name
      } else {
        continue
      }
      if (-not $seen.Add($name) -or $entry.Length -eq 0) { throw "Invalid archive entry: $name" }
      [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $dest)
    }
    foreach ($name in $exes) {
      if (-not $seen.Contains($name)) { throw "Portable build archive is missing $name" }
    }
  } finally {
    $archive.Dispose()
  }

  $env:CARGO_TARGET_DIR = Join-Path $temp 'target'
  $env:DIST = $packages
  $env:DOC4ME_BUILD_SHA = $SourceCommit
  $env:DOC4ME_VERSION = $Version
  $env:DOC4ME_FILE_NAME = $FileName
  & (Join-Path $PSScriptRoot 'package.ps1') -Arch $Arch -SkipBuild -RequireSignature -PortableFontNotices $notices

  $files = Get-Item -LiteralPath (Join-Path $packages "$stem.msi"), (Join-Path $packages $zipName)
  if ((Test-Path -LiteralPath $Dist) -and @(Get-ChildItem -LiteralPath $Dist -Force).Count -ne 0) {
    throw 'Signed artifact output directory must be empty'
  }
  New-Item -ItemType Directory -Path $Dist -Force | Out-Null
  foreach ($file in $files) { Move-Item -LiteralPath $file.FullName -Destination $Dist }
  Set-Content -LiteralPath (Join-Path $Dist 'SOURCE_COMMIT.txt') -Value $SourceCommit
  Set-Content -LiteralPath (Join-Path $Dist 'VERSION.txt') -Value $Version
} finally {
  Remove-Item -LiteralPath $temp -Recurse -Force -ErrorAction SilentlyContinue
}
