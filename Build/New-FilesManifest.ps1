<#
.SYNOPSIS
    Génère Build/files-manifest.json dans le dossier de développement : liste (chemin relatif, taille, SHA-256) des
    fichiers gérés par le launcher (ArchCore, ArchRust, MariaDB, HeidiSQL). Le launcher s'en sert pour VÉRIFIER.
    New-Package.ps1 produit le sien pour le package. Jamais : données de jeu sous Client/ et Server/Data/, UserData/, Logs/.
#>
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$roots = @('Server\ArchCore', 'Client\ArchRust.exe', 'Server\Database\mariadb', 'Tools\HeidiSQL', 'Build\manifest.json')
$files = New-Object System.Collections.Generic.List[object]
foreach ($r in $roots) {
    $p = Join-Path $root $r
    if (-not (Test-Path $p)) { Write-Warning "absent : $r"; continue }
    $items = if ((Get-Item $p).PSIsContainer) { Get-ChildItem $p -Recurse -File | Where-Object { $_.FullName -notlike '*\mariadb\data\*' } } else { @(Get-Item $p) }
    foreach ($f in $items) {
        $rel = $f.FullName.Substring($root.Length + 1)
        $files.Add([pscustomobject]@{ path = $rel; size = $f.Length; sha256 = (Get-FileHash -Algorithm SHA256 $f.FullName).Hash.ToLowerInvariant() })
    }
}
$manifest = [pscustomobject]@{ generated = (Get-Date -Format 'yyyy-MM-dd HH:mm:ss'); files = $files }
$out = Join-Path $PSScriptRoot 'files-manifest.json'
[System.IO.File]::WriteAllText($out, ($manifest | ConvertTo-Json -Depth 4), (New-Object System.Text.UTF8Encoding $false))
"$($files.Count) fichiers, $([math]::Round(($files | Measure-Object size -Sum).Sum/1MB)) Mo -> $out"
