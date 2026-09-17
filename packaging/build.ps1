# Builds the standalone app for Windows.
#
#   powershell -ExecutionPolicy Bypass -File packaging\build.ps1
#
# PyInstaller does not build across platforms — this script has to run on a
# Windows computer. Result, outside the repository, next to the folder `repo`:
#   <Projects>\OpenETV_App\OpenETV\OpenETV.exe  and  OpenETV-<version>-Windows.zip
# Another place: set $env:OPENETV_APP_DIR = "D:\path" before the call.
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $here

if (-not (Test-Path ".venv\Scripts\python.exe")) {
  py -3 -m venv .venv
  .venv\Scripts\python.exe -m pip install --upgrade pip
}
$target = if ($env:OPENETV_APP_DIR) { $env:OPENETV_APP_DIR } else { Join-Path (Split-Path -Parent (Split-Path -Parent $here)) "OpenETV_App" }
$work = Join-Path ([System.IO.Path]::GetTempPath()) ("openetv-build-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force $target | Out-Null
$version = (Select-String -Path "tools\etvlib\__init__.py" -Pattern '__version__ = "([^"]+)"').Matches[0].Groups[1].Value

.venv\Scripts\python.exe -m pip install --quiet -r requirements-dev.txt
.venv\Scripts\python.exe -m PyInstaller packaging\openetv.spec --noconfirm --clean `
  --distpath (Join-Path $work "dist") --workpath (Join-Path $work "build")

Remove-Item -Recurse -Force (Join-Path $target "OpenETV") -ErrorAction SilentlyContinue
Move-Item (Join-Path $work "dist\OpenETV") $target
$zip = Join-Path $target "OpenETV-$version-Windows.zip"
Remove-Item -Force $zip -ErrorAction SilentlyContinue
Compress-Archive -Path (Join-Path $target "OpenETV") -DestinationPath $zip
Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue

Write-Host "Done: $target\OpenETV\OpenETV.exe"
Write-Host "      $zip"
