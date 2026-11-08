<#
.SYNOPSIS
    Compile ArchCore (serveur ArchLord en Rust) avec le module ArchBot (bots autonomes et compagnons) et l'installe
    dans Server\ArchCore : archcore.exe, sql\ (schéma et migrations), content\ (couche de contenu), data\i18n\ (titres
    de PNJ traduits), config\*.dist (modèles, dont archbot.conf.dist), archbot\ (patch appliqué), licences.

.DESCRIPTION
    ArchCore est compilé sur une COPIE propre, Build\tmp\ArchCore-src : clone de -Source (copie locale du dépôt,
    branche Core), sinon de https://github.com/BlaMacfly/ArchCore, au commit du manifeste. Le dépôt -Source n'est
    jamais modifié (ni fichier, ni branche, ni target\ : l'archcore.exe qui s'y trouve n'est jamais touché).
    ArchBot est placé dans modules\ArchBot de cette copie (arbre de travail de -ArchBot, sans target\ ni _ref\, ou
    clone du manifeste), puis son patch patches\archcore-archbot.patch est appliqué (git apply), comme le fait son
    script d'installation. Compilation dans Build\tmp\archcore-target (--target-dir), --locked (le patch porte
    Cargo.lock), avec la toolchain 1.98.1.

.PARAMETER Source
    Dossier du dépôt ArchCore (branche Core), lu seulement. Défaut : clone du dépôt du manifeste.
.PARAMETER ArchBot
    Dossier du dépôt ArchBot. Défaut : clone du dépôt du manifeste au commit indiqué.
.PARAMETER NoArchBot
    ArchCore seul, sans ArchBot (ni patch, ni archbot.conf.dist).
#>
[CmdletBinding()]
param([string]$Source, [string]$ArchBot, [switch]$NoArchBot)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$manifest = Get-Content (Join-Path $PSScriptRoot 'manifest.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$tmp = Join-Path $PSScriptRoot 'tmp'
New-Item -ItemType Directory -Force $tmp | Out-Null
function Step { param($m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }
function Invoke-Git { & git.exe @args; if ($LASTEXITCODE -ne 0) { throw "git $($args -join ' ') : échec (code $LASTEXITCODE)" } }
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:RUSTUP_TOOLCHAIN = '1.98.1'
$env:CARGO_INCREMENTAL = '0'

# ---- Copie d'ArchCore au commit du manifeste
$origin = if ($Source) { (Resolve-Path -LiteralPath $Source).Path } else { $manifest.archcore.source }
if ($Source -and -not (Test-Path (Join-Path $origin 'crates\archcore'))) { throw "Ce n'est pas un dépôt ArchCore (branche Core) : $Source" }
$src = Join-Path $tmp 'ArchCore-src'
if (Test-Path $src) { Remove-Item -Recurse -Force $src }
Step "Copie d'ArchCore ($($manifest.archcore.commit)) depuis $origin"
Invoke-Git clone --quiet --no-checkout $origin $src
Invoke-Git -C $src checkout --quiet $manifest.archcore.commit
$commit = (git -C $src rev-parse --short HEAD).Trim()
$date = (git -C $src log -1 --format=%cs).Trim()
Step "ArchCore $commit ($date)"

# ---- ArchBot dans modules\ArchBot, patch d'ArchCore
$botLabel = ''
if (-not $NoArchBot) {
    $mod = Join-Path $src 'modules\ArchBot'
    if ($ArchBot) {
        $from = (Resolve-Path -LiteralPath $ArchBot).Path
        if (-not (Test-Path (Join-Path $from 'patches\archcore-archbot.patch'))) { throw "Ce n'est pas un dépôt ArchBot : $ArchBot" }
        Step "ArchBot depuis $from (arbre de travail, sans target ni _ref)"
        New-Item -ItemType Directory -Force $mod | Out-Null
        cmd /c "robocopy `"$from`" `"$mod`" /E /XD target _ref .git .claude /XF *.log /NFL /NDL /NJH /NJS /NP 2>&1" | Out-Null
        if ($LASTEXITCODE -ge 8) { throw "robocopy ArchBot : échec (code $LASTEXITCODE)" }
        $botCommit = (git -C $from rev-parse --short HEAD).Trim()
        $botDirty = @(git -C $from status --porcelain --untracked-files=no).Count -gt 0
    } else {
        Step "Clone d'ArchBot ($($manifest.archbot.commit)) depuis $($manifest.archbot.source)"
        Invoke-Git clone --quiet --no-checkout $manifest.archbot.source $mod
        Invoke-Git -C $mod checkout --quiet $manifest.archbot.commit
        $botCommit = (git -C $mod rev-parse --short HEAD).Trim()
        $botDirty = $false
    }
    $botLabel = " + ArchBot $botCommit$(if ($botDirty) { ' +local' })"
    Step "ArchBot $botCommit$(if ($botDirty) { ', avec des modifications locales non commitées' })"
    if ($botCommit -ne $manifest.archbot.commit) { Write-Warning "ArchBot $botCommit différent du manifeste ($($manifest.archbot.commit)) : mettre à jour Build\manifest.json si c'est voulu." }
    $patch = Join-Path $mod 'patches\archcore-archbot.patch'
    Push-Location $src
    try {
        cmd /c "git apply --check `"$patch`" 2>&1" | Out-Host
        if ($LASTEXITCODE -ne 0) { throw "Le patch d'ArchBot ne s'applique pas sur ArchCore $commit (voir modules\ArchBot\docs\ARCHCORE-CHANGES.md)" }
        Invoke-Git apply $patch
    } finally { Pop-Location }
    $changed = @(git -C $src diff --name-only).Count
    Step "Patch d'ArchBot appliqué ($changed fichiers d'ArchCore)"
}

# ---- Compilation
Step "Compilation (cargo build --release -p archcore --locked, toolchain $env:RUSTUP_TOOLCHAIN)"
$target = Join-Path $tmp 'archcore-target'
$built = $false
# rustc plante parfois sur ce PC (STATUS_ACCESS_VIOLATION) : jusqu'à trois essais.
for ($i = 1; $i -le 3 -and -not $built; $i++) {
    cargo build --release -p archcore --locked --manifest-path (Join-Path $src 'Cargo.toml') --target-dir $target
    $built = $LASTEXITCODE -eq 0
    if (-not $built -and $i -lt 3) { Write-Warning "cargo build : échec (code $LASTEXITCODE), nouvel essai" }
}
if (-not $built) { throw "cargo build a échoué (code $LASTEXITCODE)" }

# ---- Installation
Step "Installation dans Server\ArchCore"
$dest = Join-Path $root 'Server\ArchCore'
if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
New-Item -ItemType Directory -Force $dest, (Join-Path $dest 'config'), (Join-Path $dest 'data\i18n') | Out-Null
Copy-Item (Join-Path $target 'release\archcore.exe') $dest
Copy-Item -Recurse (Join-Path $src 'sql') (Join-Path $dest 'sql')
Copy-Item -Recurse (Join-Path $src 'content') (Join-Path $dest 'content')
# Traductions utilisées par le serveur ; pending-en.json (liste de travail des traducteurs, 3 Mo) n'en fait pas partie.
Get-ChildItem (Join-Path $src 'data\i18n') -Filter '*.json' | Where-Object { $_.Name -ne 'pending-en.json' } | Copy-Item -Destination (Join-Path $dest 'data\i18n')
Copy-Item (Join-Path $src 'config\auth.conf.dist'), (Join-Path $src 'config\world.conf.dist') (Join-Path $dest 'config')
Copy-Item (Join-Path $src 'LICENSE') (Join-Path $dest 'LICENSE')
if (-not $NoArchBot) {
    # Modèle de archbot.conf (le launcher en tire Server\Config\archbot.conf), patch appliqué et licence du module :
    # le code source correspondant de l'exécutable est ArchCore au commit indiqué + ArchBot + ce patch.
    Copy-Item (Join-Path $mod 'config\archbot.conf.dist') (Join-Path $dest 'config')
    New-Item -ItemType Directory -Force (Join-Path $dest 'archbot') | Out-Null
    Copy-Item $patch (Join-Path $dest 'archbot\archcore-archbot.patch')
    Copy-Item (Join-Path $mod 'LICENSE') (Join-Path $dest 'archbot\LICENSE')
}
Set-Content -Path (Join-Path $dest 'VERSION.txt') -Encoding ASCII -Value "Core $commit ($date)$botLabel"
$size = (Get-ChildItem $dest -Recurse -File | Measure-Object Length -Sum).Sum
Step ("Server\ArchCore : {0:N1} Mo, ArchCore {1}{2}" -f ($size / 1MB), $commit, $botLabel)
