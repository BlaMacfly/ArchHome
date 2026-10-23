<#
.SYNOPSIS
    Compile ArchCore (serveur ArchLord en Rust) et l'installe dans Server\ArchCore : archcore.exe, sql\ (schéma et
    migrations), content\ (couche de contenu), data\i18n\ (titres de PNJ traduits), config\*.dist (modèles), LICENSE.

.DESCRIPTION
    Sources : -Source (copie locale du dépôt, branche Core), sinon clone de https://github.com/BlaMacfly/ArchCore au
    commit du manifeste dans Build\tmp\ArchCore-src. La compilation se fait dans Build\tmp\archcore-target
    (--target-dir) : le dossier target\ des sources et l'archcore.exe qui s'y trouve ne sont jamais touchés, et
    --locked interdit toute modification de Cargo.lock.

.PARAMETER Source
    Dossier du dépôt ArchCore (branche Core). Défaut : celui du manifeste, cloné si absent.
#>
[CmdletBinding()]
param([string]$Source)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$manifest = Get-Content (Join-Path $PSScriptRoot 'manifest.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$tmp = Join-Path $PSScriptRoot 'tmp'
New-Item -ItemType Directory -Force $tmp | Out-Null
function Step { param($m) Write-Host "[$(Get-Date -Format HH:mm:ss)] $m" }
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"

if (-not $Source) {
    $Source = Join-Path $tmp 'ArchCore-src'
    if (-not (Test-Path (Join-Path $Source 'Cargo.toml'))) {
        Step "Clone de $($manifest.archcore.source) (branche $($manifest.archcore.branch))"
        git clone --quiet --branch $manifest.archcore.branch $manifest.archcore.source $Source
        if ($LASTEXITCODE -ne 0) { throw "git clone a échoué" }
    }
    git -C $Source fetch --quiet origin $manifest.archcore.branch
    git -C $Source checkout --quiet $manifest.archcore.commit
    if ($LASTEXITCODE -ne 0) { throw "commit $($manifest.archcore.commit) introuvable" }
}
if (-not (Test-Path (Join-Path $Source 'crates\archcore'))) { throw "Ce n'est pas un dépôt ArchCore (branche Core) : $Source" }
$commit = (git -C $Source rev-parse --short HEAD).Trim()
$date = (git -C $Source log -1 --format=%cs).Trim()
$dirty = @(git -C $Source status --porcelain --untracked-files=no).Count -gt 0
Step "ArchCore $commit ($date)$(if ($dirty) { ', avec des modifications locales non commitées' })"
if ($commit -ne $manifest.archcore.commit) { Write-Warning "Commit $commit différent du manifeste ($($manifest.archcore.commit)) : mettre à jour Build\manifest.json si c'est voulu." }

Step "Compilation (cargo build --release -p archcore --locked)"
$target = Join-Path $tmp 'archcore-target'
cargo build --release -p archcore --locked --manifest-path (Join-Path $Source 'Cargo.toml') --target-dir $target
if ($LASTEXITCODE -ne 0) { throw "cargo build a échoué (code $LASTEXITCODE)" }

Step "Installation dans Server\ArchCore"
$dest = Join-Path $root 'Server\ArchCore'
if (Test-Path $dest) { Remove-Item -Recurse -Force $dest }
New-Item -ItemType Directory -Force $dest, (Join-Path $dest 'config'), (Join-Path $dest 'data\i18n') | Out-Null
Copy-Item (Join-Path $target 'release\archcore.exe') $dest
Copy-Item -Recurse (Join-Path $Source 'sql') (Join-Path $dest 'sql')
Copy-Item -Recurse (Join-Path $Source 'content') (Join-Path $dest 'content')
# Traductions utilisées par le serveur ; pending-en.json (liste de travail des traducteurs, 3 Mo) n'en fait pas partie.
Get-ChildItem (Join-Path $Source 'data\i18n') -Filter '*.json' | Where-Object { $_.Name -ne 'pending-en.json' } | Copy-Item -Destination (Join-Path $dest 'data\i18n')
Copy-Item (Join-Path $Source 'config\auth.conf.dist'), (Join-Path $Source 'config\world.conf.dist') (Join-Path $dest 'config')
Copy-Item (Join-Path $Source 'LICENSE') (Join-Path $dest 'LICENSE')
Set-Content -Path (Join-Path $dest 'VERSION.txt') -Encoding ASCII -Value "Core $commit ($date)$(if ($dirty) { ' +local' })"
$size = (Get-ChildItem $dest -Recurse -File | Measure-Object Length -Sum).Sum
Step ("Server\ArchCore : {0:N1} Mo, ArchCore {1}" -f ($size / 1MB), $commit)
