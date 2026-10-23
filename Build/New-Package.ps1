<#
.SYNOPSIS
    Construit la version portable distribuable dans Build/package/HomeRPG-ArchLord-v<version>/ : launcher, ArchCore,
    MariaDB allégée, HeidiSQL (s'il est installé), documentation, licences et sources, manifeste de fichiers
    (empreintes) et SHA256SUMS.txt. Avec -Zip, produit aussi l'archive .zip.
    Jamais inclus : Client/, Server/Data/ (données dérivées du client), UserData/, Logs/, Build/downloads/.
#>
[CmdletBinding()]
param([switch]$Zip)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$cargo = Get-Content (Join-Path $root 'Launcher\Cargo.toml') -Raw
$version = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
$name = "HomeRPG-ArchLord-v$version"
$out = Join-Path $PSScriptRoot "package\$name"
$launcher = Join-Path $root 'Launcher\target\release\homerpg-archlord-launcher.exe'
if (-not (Test-Path $launcher)) { throw "Launcher non compilé : $launcher" }
if (-not (Test-Path (Join-Path $root 'Server\ArchCore\archcore.exe'))) { throw "ArchCore absent : Build\Build-ArchCore.ps1" }
if (-not (Test-Path (Join-Path $root 'Server\Database\mariadb\bin\mariadbd.exe'))) { throw "MariaDB absente : Build\Get-Components.ps1 puis Build\Install-Components.ps1" }
$sig = Get-AuthenticodeSignature $launcher
if (-not $sig.SignerCertificate) { Write-Warning "Launcher non signé (Build\Sign-Launcher.ps1) : Windows affichera « éditeur inconnu »." }

function Step { param($m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }
if (Test-Path $out) { Remove-Item -Recurse -Force $out }
New-Item -ItemType Directory -Force $out | Out-Null
function CopyTree { param($src, $dst, [string[]]$Exclude = @())
    New-Item -ItemType Directory -Force $dst | Out-Null
    Get-ChildItem $src -Recurse -File | ForEach-Object {
        $rel = $_.FullName.Substring($src.Length + 1)
        if ($Exclude | Where-Object { $rel -like $_ }) { return }
        $to = Join-Path $dst $rel
        New-Item -ItemType Directory -Force (Split-Path $to) | Out-Null
        Copy-Item $_.FullName $to
    }
}

Step "Launcher"
Copy-Item $launcher (Join-Path $out 'HomeRPG ArchLord.exe')

Step "ArchCore (programme, schéma, couche de contenu, traductions, modèles de configuration)"
CopyTree (Join-Path $root 'Server\ArchCore') (Join-Path $out 'Server\ArchCore')
New-Item -ItemType Directory -Force (Join-Path $out 'Server\Config'), (Join-Path $out 'Server\Data\server-ini'), (Join-Path $out 'Client') | Out-Null
Copy-Item (Join-Path $PSScriptRoot 'package-files\Client-LISEZMOI.txt') (Join-Path $out 'Client\LISEZMOI.txt')
Copy-Item (Join-Path $PSScriptRoot 'package-files\server-ini-LISEZMOI.txt') (Join-Path $out 'Server\Data\server-ini\LISEZMOI.txt')

Step "MariaDB allégée (sans symboles de débogage, en-têtes, bibliothèques de liaison ni tests)"
CopyTree (Join-Path $root 'Server\Database\mariadb') (Join-Path $out 'Server\Database\mariadb') -Exclude @('*.pdb', 'include\*', 'lib\*.lib', 'lib\*.pdb', 'mysql-test\*', 'sql-bench\*', 'bin\mariadb-test*.exe', 'bin\*.pdb', 'data\*')

Step "HeidiSQL, scripts d'exploitation, documentation"
if (Test-Path (Join-Path $root 'Tools\HeidiSQL')) { CopyTree (Join-Path $root 'Tools\HeidiSQL') (Join-Path $out 'Tools\HeidiSQL') } else { Write-Warning "HeidiSQL absent : le bouton « Ouvrir dans HeidiSQL » sera inactif." }
New-Item -ItemType Directory -Force (Join-Path $out 'Tools\Scripts') | Out-Null
Get-ChildItem (Join-Path $root 'Tools') -Filter '*.ps1' | ForEach-Object { Copy-Item $_.FullName (Join-Path $out "Tools\Scripts\$($_.Name)") }
CopyTree (Join-Path $root 'Documentation') (Join-Path $out 'Documentation')
Copy-Item (Join-Path $root 'README.md') (Join-Path $out 'Documentation\00-README-projet.md')
Copy-Item (Join-Path $PSScriptRoot 'package-files\LISEZMOI.txt') (Join-Path $out 'LISEZMOI.txt')
Copy-Item (Join-Path $PSScriptRoot 'package-files\README.txt') (Join-Path $out 'README.txt')
New-Item -ItemType Directory -Force (Join-Path $out 'Build') | Out-Null
Copy-Item (Join-Path $PSScriptRoot 'manifest.json') (Join-Path $out 'Build\manifest.json')

Step "Licences et sources"
$lic = Join-Path $out 'LICENCES'
New-Item -ItemType Directory -Force $lic | Out-Null
Copy-Item (Join-Path $root 'LICENCE.txt') (Join-Path $out 'LICENCE.txt')
Copy-Item (Join-Path $root 'LICENCE.txt') (Join-Path $lic 'HomeRPG-Licence.txt')
Copy-Item (Join-Path $root 'Server\ArchCore\LICENSE') (Join-Path $lic 'ArchCore-GPL-3.0.txt')
$mdbLic = Get-ChildItem (Join-Path $root 'Server\Database\mariadb') -File | Where-Object { $_.Name -match '^(COPYING|LICENSE|THIRDPARTY|README)' }
foreach ($f in $mdbLic) { Copy-Item $f.FullName (Join-Path $lic "MariaDB-$($f.Name)") }
foreach ($f in 'gpl.txt', 'license.txt') { $p = Join-Path $root "Tools\HeidiSQL\$f"; if (Test-Path $p) { Copy-Item $p (Join-Path $lic "HeidiSQL-$f") } }
# Licences des crates Rust (cargo metadata)
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$metaFile = Join-Path $env:TEMP 'homerpg-cargo-metadata.json'
Push-Location (Join-Path $root 'Launcher')
cmd /c "cargo metadata --format-version 1 --locked > `"$metaFile`" 2>nul"
Pop-Location
$meta = Get-Content $metaFile -Raw | ConvertFrom-Json
$crates = $meta.packages | Where-Object { $_.name -ne 'homerpg-archlord-launcher' } | Sort-Object name | ForEach-Object { "| $($_.name) | $($_.version) | $($_.license) |" }
$m = Get-Content (Join-Path $PSScriptRoot 'manifest.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$archcoreVersion = (Get-Content (Join-Path $root 'Server\ArchCore\VERSION.txt') -Raw).Trim()
$sources = @("# Sources et licences", "", "Package HomeRPG ArchLord v$version généré le $(Get-Date -Format 'yyyy-MM-dd HH:mm').", "",
    "## Composants", "", "| Composant | Version / commit | Source | Licence |", "|---|---|---|---|",
    "| ArchCore (Server\ArchCore) | $archcoreVersion | $($m.archcore.source) (branche $($m.archcore.branch)) | $($m.archcore.license) |")
foreach ($c in $m.components) { $sources += "| $($c.id) | $($c.commit) | $($c.source) | $($c.license) |" }
$sources += @("| HomeRPG ArchLord (launcher, scripts, documentation, identité visuelle) | v$version | Sources\Launcher, Sources\Build, Tools\Scripts (ce package) | Propriétaire : LICENCE.txt |", "",
    "Les licences des composants tiers ci-dessus s'appliquent uniquement à ces composants ; la licence HomeRPG ne les restreint pas. ArchCore est un programme distinct, lancé par le launcher comme processus séparé, et livré sans modification depuis le commit indiqué.", "",
    "Le code source complet d'ArchCore, MariaDB et HeidiSQL est disponible aux adresses ci-dessus pour les versions exactes indiquées. Sur demande, l'éditeur de ce package en fournit une copie.", "",
    "## Bibliothèques Rust du launcher (licences déclarées par les auteurs)", "", "| Crate | Version | Licence |", "|---|---|---|") + $crates
[System.IO.File]::WriteAllText((Join-Path $lic 'SOURCES.md'), ($sources -join "`r`n"), (New-Object System.Text.UTF8Encoding $false))
$src = Join-Path $out 'Sources\Launcher'
CopyTree (Join-Path $root 'Launcher\src') (Join-Path $src 'src')
CopyTree (Join-Path $root 'Launcher\assets') (Join-Path $src 'assets')
foreach ($f in 'Cargo.toml', 'Cargo.lock', 'README.md', 'build.rs') { Copy-Item (Join-Path $root "Launcher\$f") (Join-Path $src $f) }
CopyTree $PSScriptRoot (Join-Path $out 'Sources\Build') -Exclude @('package\*', 'downloads\*', 'tmp\*', 'files-manifest.json')

Step "Manifeste de fichiers et SHA256SUMS"
$files = New-Object System.Collections.Generic.List[object]
$sums = New-Object System.Collections.Generic.List[string]
Get-ChildItem $out -Recurse -File | Sort-Object FullName | ForEach-Object {
    $rel = $_.FullName.Substring($out.Length + 1)
    if ($rel -in 'Build\files-manifest.json', 'SHA256SUMS.txt') { return }
    $h = (Get-FileHash -Algorithm SHA256 $_.FullName).Hash.ToLowerInvariant()
    $files.Add([pscustomobject]@{ path = $rel; size = $_.Length; sha256 = $h })
    $sums.Add("$h  $($rel -replace '\\','/')")
}
$manifest = [pscustomobject]@{ generated = (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'); version = $version; files = $files }
[System.IO.File]::WriteAllText((Join-Path $out 'Build\files-manifest.json'), ($manifest | ConvertTo-Json -Depth 4), (New-Object System.Text.UTF8Encoding $false))
[System.IO.File]::WriteAllText((Join-Path $out 'SHA256SUMS.txt'), (($sums -join "`n") + "`n"), (New-Object System.Text.UTF8Encoding $false))

$total = (Get-ChildItem $out -Recurse -File | Measure-Object Length -Sum).Sum
Step ("Package : {0} fichiers, {1:N0} Mo -> {2}" -f $files.Count, ($total / 1MB), $out)
if ($Zip) {
    $zipPath = Join-Path $PSScriptRoot "package\$name.zip"
    if (Test-Path $zipPath) { Remove-Item $zipPath }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory($out, $zipPath, [System.IO.Compression.CompressionLevel]::Optimal, $true)
    Step ("Archive : {0:N0} Mo -> {1}" -f ((Get-Item $zipPath).Length / 1MB), $zipPath)
}
