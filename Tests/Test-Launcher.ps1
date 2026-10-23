<#
.SYNOPSIS
    Test d'intégration du launcher : démarrage complet automatique (--start), fermeture après N secondes
    (--quit-after), puis contrôle du journal, des fichiers de journaux serveur, de la libération des ports et de
    l'absence de processus orphelins. Le premier démarrage (import du monde) peut prendre plusieurs minutes.
#>
[CmdletBinding()]
param([int]$QuitAfter = 60, [string]$Exe)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"

if (-not $Exe) { $Exe = Get-ProjectPath 'Launcher\target\release\homerpg-archlord-launcher.exe' }
if (-not (Test-Path $Exe)) { throw "Launcher non compilé : $Exe" }
if (Get-Process -Name 'homerpg-archlord-launcher', 'HomeRPG ArchLord' -ErrorAction SilentlyContinue) { throw "Un launcher HomeRPG est déjà ouvert : le fermer d'abord" }
foreach ($p in $Layout.MariaDbPort, $Layout.LoginPort, $Layout.WorldPort) { if (Test-Port $p) { throw "Port $p déjà occupé : arrêter le serveur qui l'utilise" } }

$log = Get-ProjectPath 'Logs\launcher.log'
$before = if (Test-Path $log) { (Get-Content $log | Measure-Object).Count } else { 0 }
Write-Step "Lancement du launcher : --start --lang fr --quit-after $QuitAfter (après la mise en service)"
$sw = [Diagnostics.Stopwatch]::StartNew()
$p = Start-Process -FilePath $Exe -ArgumentList @('--start', '--lang', 'fr', '--quit-after', '7200') -WorkingDirectory (Split-Path $Exe) -PassThru

# Les trois ports doivent s'ouvrir ; le délai --quit-after ne compte qu'à partir du serveur prêt (import compris).
$seen = @{}
while (-not $p.HasExited -and -not $seen[$Layout.WorldPort]) {
    foreach ($port in $Layout.MariaDbPort, $Layout.LoginPort, $Layout.WorldPort) { if (-not $seen[$port] -and (Test-Port $port)) { $seen[$port] = [int]$sw.Elapsed.TotalSeconds; Write-Step "port $port ouvert à t+$($seen[$port]) s" } }
    Start-Sleep -Milliseconds 500
    if ($sw.Elapsed.TotalMinutes -gt 30) { Write-Step "délai de démarrage dépassé"; break }
}
Start-Sleep -Seconds $QuitAfter
# Fermeture comme un clic sur la croix : la fenêtre demande l'arrêt des services.
if (-not $p.HasExited) { $null = $p.CloseMainWindow() }
if (-not $p.WaitForExit(180000)) { Write-Step "le launcher ne se ferme pas : arrêt forcé"; $p.Kill() }
Write-Step "launcher terminé (code $($p.ExitCode)) après $([int]$sw.Elapsed.TotalSeconds) s"
Start-Sleep -Seconds 2

$lines = @(Get-Content $log -Encoding UTF8 | Select-Object -Skip $before)
$text = $lines -join "`n"
$serverLog = Get-ChildItem (Get-ProjectPath 'Logs\server') -Filter 'archcore-*.log' -ErrorAction SilentlyContinue | Sort-Object Name | Select-Object -Last 1
$serverText = if ($serverLog) { Get-Content $serverLog.FullName -Raw -Encoding UTF8 } else { '' }
$checks = [ordered]@{
    'MariaDB démarrée'                 = ($text -match 'MariaDB opérationnelle')
    'ArchCore opérationnel'            = ($text -match 'ArchCore opérationnel')
    'ArchCore arrêté proprement'       = ($text -match 'ArchCore arrêté proprement')
    'MariaDB arrêtée'                  = ($text -match 'MariaDB arrêtée')
    'aucune ERREUR'                    = -not ($text -match 'ERREUR')
    'journal serveur écrit'            = ($serverText -match 'en écoute sur 127\.0\.0\.1')
    "port $($Layout.MariaDbPort) libéré" = -not (Test-Port $Layout.MariaDbPort)
    "port $($Layout.LoginPort) libéré"   = -not (Test-Port $Layout.LoginPort)
    "port $($Layout.WorldPort) libéré"   = -not (Test-Port $Layout.WorldPort)
    'aucun processus orphelin'         = -not (Get-Process archcore, mariadbd -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($ProjectRoot, [StringComparison]::OrdinalIgnoreCase) })
}
$ok = $true
foreach ($k in $checks.Keys) { Write-Step ("{0,-30} {1}" -f $k, $(if ($checks[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $checks[$k]) { $ok = $false } }
Write-Step "Journal du launcher ($($lines.Count) lignes) :"
$lines | Select-Object -Last 25 | ForEach-Object { "    $_" }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
