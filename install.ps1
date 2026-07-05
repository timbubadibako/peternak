$ErrorActionPreference = "Stop"

$App = "peternak-aiai"
$Version = if ($env:PETERNAK_VERSION) { $env:PETERNAK_VERSION } else { "latest" }
$BaseUrl = if ($env:PETERNAK_DOWNLOAD_BASE) {
    $env:PETERNAK_DOWNLOAD_BASE.TrimEnd("/")
} else {
    "https://github.com/jrilym/peternak-aiai/releases/$Version/download"
}
$InstallDir = if ($env:PETERNAK_INSTALL_DIR) {
    $env:PETERNAK_INSTALL_DIR
} else {
    Join-Path $env:LOCALAPPDATA "peternak-aiai\bin"
}

if (-not [Environment]::Is64BitOperatingSystem) {
    throw "Only 64-bit Windows is supported."
}

$Target = "x86_64-pc-windows-msvc"
$Archive = "$App-$Target.tar.gz"
$TempDir = Join-Path ([IO.Path]::GetTempPath()) ([IO.Path]::GetRandomFileName())
$Url = "$BaseUrl/$Archive"

New-Item -ItemType Directory -Force -Path $TempDir | Out-Null
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

try {
    Write-Host "Downloading $Url"
    Invoke-WebRequest -Uri $Url -OutFile (Join-Path $TempDir $Archive)
    tar -xzf (Join-Path $TempDir $Archive) -C $TempDir
    Copy-Item (Join-Path $TempDir "$App.exe") (Join-Path $InstallDir "$App.exe") -Force
    Write-Host "Installed $App to $InstallDir"
    Write-Host "Add this directory to PATH if needed."
} finally {
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
}
