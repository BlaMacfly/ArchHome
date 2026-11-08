<#
.SYNOPSIS
    Paquet des testeurs, prêt à jouer : copie du package (Build\package\HomeRPG-ArchLord-v<version>) dans
    <OutDir>\ArchHome-Testing-v<version>, avec les données du jeu d'un client complet dans Client\ et (avec
    -InitDatabase) la base déjà initialisée et le monde importé, puis archive .7z
    et SHA256SUMS.txt à côté. Diffusion privée aux testeurs, comme le paquet 0.2.0 ; jamais dans un dépôt.

.DESCRIPTION
    De -GameData sont copiés les dossiers et fichiers du client ArchLord (character, ini*, world, texture, object,
    effect, sound, etc, images…) et ses ressources (patches, free-assets), sans ce que le package ou le launcher
    fournissent ou écrivent : ArchRust.exe, addons, saved-variables, realmlist.wtf, server-data, assets, .bat,
    LISEZMOI.txt. Le dossier de sortie ne doit pas exister (un ancien paquet n'est jamais remplacé).
    Le SHA256SUMS.txt du package (dans le dossier) couvre les fichiers du package ; celui d'à côté couvre l'archive.

.PARAMETER GameData
    Dossier d'un client complet (données du jeu ArchLord 64 bits à côté d'ArchRust.exe).
.PARAMETER OutDir
    Dossier de sortie, par exemple « Bureau\HomeRPG ArchLord v0.3.0 ».
.PARAMETER NoArchive
    Dossier seulement, sans .7z.
.PARAMETER HardLinks
    Données du jeu en liens physiques NTFS vers -GameData au lieu d'une copie (même volume) : aucun espace disque
    en plus. Les fichiers sont alors partagés avec -GameData : n'en modifier aucun dans le paquet.
.PARAMETER InitDatabase
    Premier lancement fait dans le paquet avant l'archive (comme le paquet 0.2.0) : MariaDB initialisée dans
    UserData\Database, bases migrées, monde importé aux valeurs de l'épisode 8, Client\server-data écrit. Le
    launcher tourne le temps de l'opération sur les ports -InitPorts (pas ceux d'un serveur de développement),
    puis UserData\settings.json et Logs\ sont retirés : le testeur garde les réglages par défaut.
.PARAMETER WebzenDir
    Avec -InitDatabase : dossier ini du client officiel de Webzen, passé à l'import (--webzen) pour les quêtes, titres,
    maîtrises et la courbe d'expérience d'origine. Seule la base importée va dans le paquet : ni ce dossier, ni son
    chemin, ni le rapport Server\ArchCore\data\import-webzen.log.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$GameData,
    [Parameter(Mandatory=$true)][string]$OutDir,
    [switch]$NoArchive,
    [switch]$HardLinks,
    [switch]$InitDatabase,
    [int[]]$InitPorts = @(11102, 11108),
    [string]$WebzenDir
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$cargo = Get-Content (Join-Path $root 'Launcher\Cargo.toml') -Raw
$version = [regex]::Match($cargo, '(?m)^version\s*=\s*"([^"]+)"').Groups[1].Value
$package = Join-Path $PSScriptRoot "package\HomeRPG-ArchLord-v$version"
$name = "ArchHome-Testing-v$version"
$dest = Join-Path $OutDir $name
$archive = Join-Path $OutDir "$name.7z"
function Step { param($m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }

if (-not (Test-Path (Join-Path $package 'HomeRPG ArchLord.exe'))) { throw "Package absent : $package (Build\New-Package.ps1)" }
if (-not (Test-Path -LiteralPath (Join-Path $GameData 'character\Reference.Dat'))) { throw "Pas de données du jeu dans $GameData (character\Reference.Dat)" }
if (Test-Path -LiteralPath $dest) { throw "Existe déjà : $dest" }
if (-not $NoArchive -and (Test-Path -LiteralPath $archive)) { throw "Existe déjà : $archive" }
$sevenZip = 'C:\Program Files\7-Zip\7z.exe'
if (-not $NoArchive -and -not (Test-Path $sevenZip)) { throw "7-Zip absent : $sevenZip" }
New-Item -ItemType Directory -Force $OutDir | Out-Null

Step "Package $version -> $dest"
cmd /c "robocopy `"$package`" `"$dest`" /E /NFL /NDL /NJH /NJS /NP" | Out-Null
if ($LASTEXITCODE -ge 8) { throw "robocopy du package : échec (code $LASTEXITCODE)" }

Step "Données du jeu depuis $GameData$(if ($HardLinks) { ' (liens physiques)' })"
$exclDirs = @('addons', 'addons-*', 'saved-variables', 'server-data', 'assets', 'Screenshots')
$exclFiles = @('ArchRust.exe', '*.bat', 'LISEZMOI.txt', 'realmlist.wtf', '*.log')
if ($HardLinks) {
    $from = (Resolve-Path -LiteralPath $GameData).Path.TrimEnd('\')
    $to = Join-Path $dest 'Client'
    $n = 0
    foreach ($f in Get-ChildItem -LiteralPath $from -Recurse -File) {
        $rel = $f.FullName.Substring($from.Length + 1)
        $parts = $rel.Split('\')
        if ($parts.Count -gt 1 -and ($parts[0..($parts.Count - 2)] | Where-Object { $d = $_; $exclDirs | Where-Object { $d -like $_ } })) { continue }
        if ($exclFiles | Where-Object { $f.Name -like $_ }) { continue }
        $target = Join-Path $to $rel
        if (Test-Path -LiteralPath $target) { continue }
        New-Item -ItemType Directory -Force (Split-Path $target) | Out-Null
        New-Item -ItemType HardLink -Path $target -Target $f.FullName | Out-Null
        $n++
    }
    Step "$n liens physiques"
} else {
    $rc = @("`"$GameData`"", "`"$(Join-Path $dest 'Client')`"", '/E', '/MT:16', '/NFL', '/NDL', '/NJH', '/NJS', '/NP', '/XD') + $exclDirs + @('/XF') + $exclFiles
    cmd /c "robocopy $($rc -join ' ')" | Out-Null
    if ($LASTEXITCODE -ge 8) { throw "robocopy des données du jeu : échec (code $LASTEXITCODE)" }
}
if ($InitDatabase) {
    Step "Premier lancement dans le paquet (ports $($InitPorts -join ' / ') le temps de l'opération)"
    if (Get-Process -Name 'homerpg-archlord-launcher', 'HomeRPG ArchLord' -ErrorAction SilentlyContinue) { throw "Un launcher HomeRPG est ouvert : le fermer d'abord" }
    New-Item -ItemType Directory -Force (Join-Path $dest 'UserData') | Out-Null
    $settings = [ordered]@{ ui_language = 'fr'; game_language = 'fr'; login_port = $InitPorts[0]; world_port = $InitPorts[1] }
    if ($WebzenDir) { $settings.webzen_dir = (Resolve-Path -LiteralPath $WebzenDir).Path }
    [System.IO.File]::WriteAllText((Join-Path $dest 'UserData\settings.json'), ($settings | ConvertTo-Json), (New-Object System.Text.UTF8Encoding $false))
    $p = Start-Process -FilePath (Join-Path $dest 'HomeRPG ArchLord.exe') -ArgumentList @('--start', '--lang', 'fr', '--quit-after', '30') -WorkingDirectory $dest -PassThru
    if (-not $p.WaitForExit(1800000)) { $p.Kill(); throw "Premier lancement trop long (30 min)" }
    $log = Get-Content (Join-Path $dest 'Logs\launcher.log') -Raw -Encoding UTF8
    $wanted = @('monde importé', 'db-check : bases cohérentes', 'tables de l''épisode 8 écrites', 'ArchCore opérationnel', 'MariaDB arrêtée')
    if ($WebzenDir) { $wanted += 'import avec les données de Webzen' }
    foreach ($want in $wanted) {
        if ($log -notmatch [regex]::Escape($want)) { throw "Premier lancement incomplet (« $want » absent de Logs\launcher.log)" }
    }
    Remove-Item -Recurse -Force (Join-Path $dest 'Logs'), (Join-Path $dest 'UserData\settings.json')
    Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $dest 'Server\ArchCore\data\import-webzen.log')
    # Clés RSA de MariaDB (private_key.pem, public_key.pem) : propres à chaque installation, régénérées par MariaDB
    # à son démarrage suivant ; jamais dans une archive partagée.
    Get-ChildItem (Join-Path $dest 'UserData\Database') -Filter '*.pem' | Remove-Item -Force
    Step ("Base prête : {0:N0} Mo dans UserData" -f ((Get-ChildItem (Join-Path $dest 'UserData') -Recurse -File | Measure-Object Length -Sum).Sum / 1MB))
}
$size = (Get-ChildItem $dest -Recurse -File | Measure-Object Length -Sum).Sum
Step ("Dossier : {0:N1} Go" -f ($size / 1GB))

if (-not $NoArchive) {
    Step "Archive $archive (7-Zip, LZMA2 -mx=5, multi-cœur)"
    # Quatre fils : une compression sur tous les cœurs a déjà produit une archive corrompue sur ce PC. L'archive
    # est toujours testée (7z t) avant son empreinte.
    & $sevenZip a -t7z -mx=5 -mmt=4 -bso0 -bsp0 $archive $dest
    if ($LASTEXITCODE -ne 0) { throw "7-Zip : échec (code $LASTEXITCODE)" }
    Step "Test de l'archive (7z t)"
    & $sevenZip t $archive -bso0 -bsp0
    if ($LASTEXITCODE -ne 0) { throw "Archive corrompue (7z t, code $LASTEXITCODE) : la refaire" }
    $h = (Get-FileHash -Algorithm SHA256 $archive).Hash.ToLowerInvariant()
    [System.IO.File]::WriteAllText((Join-Path $OutDir 'SHA256SUMS.txt'), "$h  $name.7z`n", (New-Object System.Text.UTF8Encoding $false))
    Step ("Archive : {0:N1} Go, sha256 {1}" -f ((Get-Item $archive).Length / 1GB), $h)
}
Copy-Item (Join-Path $dest 'LISEZMOI-TESTEURS.txt'), (Join-Path $dest 'README-TESTERS.txt') $OutDir
Step "Terminé : $OutDir"
