# Build on Windows. The native executable embeds the manual, fonts and sample data.
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $here
$version = (Select-String -Path "rust\Cargo.toml" -Pattern '^version = "([^"]+)"' | Select-Object -First 1).Matches[0].Groups[1].Value
if (-not $version) { throw "Missing Cargo package version" }
$target = if ($env:OPENETV_APP_DIR) { $env:OPENETV_APP_DIR } else { Join-Path (Split-Path -Parent (Split-Path -Parent $here)) "OpenETV_App" }
$buildDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $here "rust\target" }
if (-not [System.IO.Path]::IsPathRooted($buildDir)) { $buildDir = Join-Path $here $buildDir }
cargo build --manifest-path rust/Cargo.toml --release --locked -j3 --target-dir $buildDir
if ($LASTEXITCODE -ne 0) { throw "Cargo build failed" }
$binary = Join-Path $buildDir "release\openetv.exe"
if (-not (Test-Path $binary)) { throw "Native binary missing" }
$work = Join-Path ([System.IO.Path]::GetTempPath()) ("openetv-build-" + [guid]::NewGuid())
try {
    $app = Join-Path $work "OpenETV"
    New-Item -ItemType Directory -Force $app, $target | Out-Null
    Copy-Item $binary (Join-Path $app "OpenETV.exe")
    Copy-Item "rust\MANUAL.md", "LICENSE" $app
    Copy-Item -Recurse "rust\assets\look" (Join-Path $app "third-party-look")
    $destination = Join-Path $target "OpenETV"
    if (Test-Path $destination) { Remove-Item -Recurse -Force $destination }
    Move-Item $app $destination
    $zip = Join-Path $target "OpenETV-$version-Windows.zip"
    Compress-Archive -Force -Path $destination -DestinationPath $zip
    Write-Host "Built: $destination\OpenETV.exe"
    Write-Host "Built: $zip"
} finally {
    if (Test-Path $work) { Remove-Item -Recurse -Force $work }
}
