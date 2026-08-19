param(
  [ValidateSet("all", "portable")]
  [string]$Mode = "all"
)

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")

$pkgJson = Get-Content (Join-Path $root "package.json") -Raw | ConvertFrom-Json
$tauriConf = Get-Content (Join-Path $root "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json

$version = [string]$pkgJson.version
$confVersion = [string]$tauriConf.version
if ($version -ne $confVersion) {
  throw "La version en package.json ($version) no coincide con tauri.conf.json ($confVersion). Sincronizalas antes de empaquetar."
}

$productName = [string]$tauriConf.productName
$mainBinary = if ([string]::IsNullOrWhiteSpace([string]$tauriConf.mainBinaryName)) { $productName } else { [string]$tauriConf.mainBinaryName }

$base = "Prometeo-$version"
$dist = Join-Path $root "dist"
$exeSource = Join-Path $root "src-tauri\target\release\$mainBinary.exe"
$nsisDir = Join-Path $root "src-tauri\target\release\bundle\nsis"

if (-not (Test-Path -LiteralPath $exeSource)) {
  throw "No se encontro el ejecutable release: $exeSource. Ejecuta primero 'npm run build:windows' o 'npm run build:portable' (compilan en modo release)."
}

New-Item -ItemType Directory -Force -Path $dist | Out-Null

$setupDest = Join-Path $dist "$base-Setup.exe"
$exeDest = Join-Path $dist "$base.exe"
$zipDest = Join-Path $dist "$base-Portable.zip"

$artifacts = New-Object System.Collections.Generic.List[string]

if ($Mode -eq "all") {
  $setupSource = Get-ChildItem -LiteralPath $nsisDir -Filter "*-setup.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
  if (-not $setupSource) {
    throw "No se encontro el instalador NSIS en $nsisDir. Ejecuta 'npm run build:windows' para generarlo."
  }
  Copy-Item -LiteralPath $setupSource.FullName -Destination $setupDest -Force
  $artifacts.Add($setupDest)
}

Copy-Item -LiteralPath $exeSource -Destination $exeDest -Force
$artifacts.Add($exeDest)

$stage = Join-Path $dist ".tmp"
$portableDirName = "$base-Portable"
$portableStage = Join-Path $stage $portableDirName
if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
New-Item -ItemType Directory -Force -Path $portableStage | Out-Null

Copy-Item -LiteralPath $exeSource -Destination (Join-Path $portableStage "$mainBinary.exe") -Force

if ($tauriConf.bundle.resources) {
  foreach ($res in @($tauriConf.bundle.resources)) {
    $resSource = Join-Path (Join-Path $root "src-tauri") $res
    if (Test-Path -LiteralPath $resSource) {
      Copy-Item -LiteralPath $resSource -Destination $portableStage -Recurse -Force
    } else {
      Write-Warning "Recurso declarado no encontrado (se omite): $res"
    }
  }
}

Compress-Archive -LiteralPath $portableStage -DestinationPath $zipDest -Force
Remove-Item -LiteralPath $stage -Recurse -Force
$artifacts.Add($zipDest)

$checksumPaths = @()
foreach ($artifact in $artifacts) {
  $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $artifact).Hash
  $name = Split-Path $artifact -Leaf
  $shaFile = "$artifact.sha256"
  Set-Content -LiteralPath $shaFile -Value "$hash  $name" -Encoding ascii
  $checksumPaths += $shaFile
}

Write-Output ""
Write-Output "BUILD SUCCESSFUL"
Write-Output ""
Write-Output "Setup:"
if ($Mode -eq "all") { Write-Output "  $setupDest" } else { Write-Output "  (no generado en modo portable)" }
Write-Output ""
Write-Output "Executable:"
Write-Output "  $exeDest"
Write-Output ""
Write-Output "Portable:"
Write-Output "  $zipDest"
Write-Output ""
Write-Output "Checksums:"
foreach ($c in $checksumPaths) { Write-Output "  $c" }
Write-Output ""
Write-Output "Architecture:"
Write-Output "  x86_64-pc-windows-msvc"
Write-Output ""
Write-Output "Version:"
Write-Output "  $version"
