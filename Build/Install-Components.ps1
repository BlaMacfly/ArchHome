<#
.SYNOPSIS
    Installe les composants tiers vérifiés par Get-Components.ps1 (Build\downloads) à leur place : MariaDB dans
    Server\Database\mariadb, HeidiSQL dans Tools\HeidiSQL.

.PARAMETER MariaDbFrom
    Poste de développement : copie une MariaDB déjà installée (ex. « C:\Program Files\MariaDB 13.0 ») au lieu de
    l'archive téléchargée. Son dossier data\ n'est jamais copié. La version doit être celle du manifeste.
#>
[CmdletBinding()]
param([string]$MariaDbFrom)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$manifest = Get-Content (Join-Path $PSScriptRoot 'manifest.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$downloads = Join-Path $PSScriptRoot 'downloads'
Add-Type -AssemblyName System.IO.Compression.FileSystem
function Step { param($m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }

# Extrait une archive zip dans $dest ; si elle contient un seul dossier racine, son contenu est remonté.
function Expand-Flat { param($zip, $dest)
    $tmp = Join-Path $PSScriptRoot "tmp\extract-$([IO.Path]::GetFileNameWithoutExtension($zip))"
    if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }
    [System.IO.Compression.ZipFile]::ExtractToDirectory($zip, $tmp)
    $top = @(Get-ChildItem $tmp)
    $src = if ($top.Count -eq 1 -and $top[0].PSIsContainer) { $top[0].FullName } else { $tmp }
    if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
    New-Item -ItemType Directory -Force (Split-Path $dest) | Out-Null
    Move-Item $src $dest
    if (Test-Path $tmp) { Remove-Item -Recurse -Force $tmp }
}

foreach ($c in $manifest.components) {
    $dest = Join-Path $root $c.install_to
    if ($c.id -eq 'mariadb-server-windows' -and $MariaDbFrom) {
        $ver = & (Join-Path $MariaDbFrom 'bin\mariadbd.exe') --version
        Step "MariaDB copiée depuis $MariaDbFrom ($ver)"
        if ($ver -notmatch [regex]::Escape($c.commit)) { Write-Warning "Version différente du manifeste ($($c.commit))" }
        if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
        New-Item -ItemType Directory -Force $dest | Out-Null
        Get-ChildItem $MariaDbFrom | Where-Object { $_.Name -ne 'data' } | Copy-Item -Destination $dest -Recurse
        continue
    }
    $zip = Join-Path $downloads $c.file
    if (-not (Test-Path $zip)) { Write-Warning "$($c.id) : $($c.file) absent de Build\downloads (lancer Get-Components.ps1)"; continue }
    Step "$($c.id) -> $($c.install_to)"
    Expand-Flat $zip $dest
}
# HeidiSQL en mode portable : ses réglages restent dans son dossier.
$heidi = Join-Path $root 'Tools\HeidiSQL'
if ((Test-Path $heidi) -and -not (Test-Path (Join-Path $heidi 'portable_settings.txt'))) { Set-Content (Join-Path $heidi 'portable_settings.txt') '' -Encoding ASCII }
Step "Composants installés."
