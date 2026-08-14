# Firma Authenticode de un binario mediante SignTool.
# Invocado por Tauri (bundle.windows.signCommand) para firmar el ejecutable y el
# instalador NSIS en cada build. Este script NO contiene secretos: toda la
# configuracion se lee de variables de entorno (detalle en el README).
#
#   VA_SIGN_THUMBPRINT      Certificado ya instalado en el almacen de Windows
#                           (CurrentUser\My o LocalMachine\My). Sin contrasena.
#   VA_SIGN_PFX             Ruta a un archivo .pfx (usar junto a VA_SIGN_PASSWORD).
#   VA_SIGN_PASSWORD        Contrasena del .pfx.
#   VA_SIGN_TIMESTAMP_URL   (opcional) Servidor de timestamp RFC3161.
#                           Por defecto: http://timestamp.digicert.com
#   VA_SIGN_SIGNTOOL        (opcional) Ruta explicita a signtool.exe.
#
# Si no se define VA_SIGN_THUMBPRINT ni VA_SIGN_PFX, el build continua sin
# firmar (salida 0) y se registra un aviso.

param(
  [Parameter(Mandatory = $true)]
  [string]$Binary
)

$ErrorActionPreference = "Stop"

function Find-SignTool {
  if ($env:VA_SIGN_SIGNTOOL -and (Test-Path -LiteralPath $env:VA_SIGN_SIGNTOOL)) {
    return $env:VA_SIGN_SIGNTOOL
  }
  $fromPath = (Get-Command signtool.exe -ErrorAction SilentlyContinue).Source
  if ($fromPath) { return $fromPath }
  $kitsRoots = @(
    "${env:ProgramFiles(x86)}\Windows Kits\10\bin",
    "$env:ProgramFiles\Windows Kits\10\bin"
  )
  foreach ($kits in $kitsRoots) {
    if (Test-Path -LiteralPath $kits) {
      $found = Get-ChildItem -LiteralPath $kits -Directory -ErrorAction SilentlyContinue |
        Sort-Object Name -Descending |
        ForEach-Object { Join-Path $_.FullName "x64\signtool.exe" } |
        Where-Object { Test-Path -LiteralPath $_ } |
        Select-Object -First 1
      if ($found) { return $found }
    }
  }
  throw "No se encontro signtool.exe. Instala el Windows SDK o define VA_SIGN_SIGNTOOL."
}

function Find-StoreCert {
  param([string]$Thumbprint)
  $target = ($Thumbprint -replace "[^0-9a-fA-F]", "").ToLower()
  foreach ($store in @("Cert:\CurrentUser\My", "Cert:\LocalMachine\My")) {
    if (Test-Path -LiteralPath $store) {
      foreach ($c in Get-ChildItem -LiteralPath $store -ErrorAction SilentlyContinue) {
        if ((($c.Thumbprint -replace "[^0-9a-fA-F]", "").ToLower()) -eq $target) {
          return $c
        }
      }
    }
  }
  return $null
}

$tsa = if ($env:VA_SIGN_TIMESTAMP_URL) { $env:VA_SIGN_TIMESTAMP_URL } else { "http://timestamp.digicert.com" }

$credArg = $null
$credValue = $null

if ($env:VA_SIGN_THUMBPRINT) {
  $cert = Find-StoreCert $env:VA_SIGN_THUMBPRINT
  if (-not $cert) {
    throw "No se encontro el certificado con thumbprint $($env:VA_SIGN_THUMBPRINT) en el almacen. Instalalo o corrige VA_SIGN_THUMBPRINT."
  }
  $credArg = "/sha1"
  $credValue = $cert.Thumbprint
} elseif ($env:VA_SIGN_PFX) {
  if (-not (Test-Path -LiteralPath $env:VA_SIGN_PFX)) {
    throw "No existe el .pfx indicado en VA_SIGN_PFX: $($env:VA_SIGN_PFX)"
  }
  $credArg = "/f"
  $credValue = $env:VA_SIGN_PFX
} else {
  Write-Host "[sign] Sin credenciales configuradas: define VA_SIGN_THUMBPRINT o VA_SIGN_PFX/VA_SIGN_PASSWORD (ver README). Se omite la firma de $Binary"
  exit 0
}

$signTool = Find-SignTool
Write-Host "[sign] Firmando: $Binary"

$sigArgs = @("sign", "/fd", "sha256", "/tr", $tsa, "/td", "sha256", $credArg, $credValue)
if ($env:VA_SIGN_PFX -and ($null -ne $env:VA_SIGN_PASSWORD)) {
  $sigArgs += @("/p", $env:VA_SIGN_PASSWORD)
}
$sigArgs += $Binary

& $signTool @sigArgs
if ($LASTEXITCODE -ne 0) {
  throw "signtool fallo con codigo $LASTEXITCODE al firmar $Binary"
}

$status = (Get-AuthenticodeSignature -LiteralPath $Binary).Status
Write-Host "[sign] Estado tras firmar ${Binary}: $status"
