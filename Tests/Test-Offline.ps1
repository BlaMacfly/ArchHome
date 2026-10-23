<#
.SYNOPSIS
    Vérifie le fonctionnement local/hors ligne : pendant un démarrage complet, MariaDB et ArchCore n'écoutent que
    sur 127.0.0.1 et aucun processus du projet n'ouvre de connexion vers une adresse non locale.
#>
[CmdletBinding()]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"
if (-not $Exe) { $Exe = Get-ProjectPath 'Launcher\target\release\homerpg-archlord-launcher.exe' }
foreach ($port in $Layout.LoginPort, $Layout.WorldPort, $Layout.MariaDbPort) { if (Test-Port $port) { throw "Port $port déjà occupé : arrêter d'abord" } }

$p = Start-Process -FilePath $Exe -ArgumentList @('--start', '--lang', 'fr', '--quit-after', '7200') -WorkingDirectory (Split-Path $Exe) -PassThru
$d = (Get-Date).AddMinutes(30)
while (-not (Test-Port $Layout.WorldPort) -and (Get-Date) -lt $d) { Start-Sleep -Seconds 1 }
Start-Sleep -Seconds 8
# Processus du projet seulement : une MariaDB installée par ailleurs sur le PC n'est pas concernée.
$pids = @(Get-Process -Name 'homerpg-archlord-launcher', 'HomeRPG ArchLord', 'archcore', 'mariadbd' -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($ProjectRoot, [StringComparison]::OrdinalIgnoreCase) } | Select-Object -ExpandProperty Id)
$conns = Get-NetTCPConnection -ErrorAction SilentlyContinue | Where-Object { $pids -contains $_.OwningProcess }
$listen = $conns | Where-Object { $_.State -eq 'Listen' }
$established = $conns | Where-Object { $_.State -ne 'Listen' -and $_.State -ne 'Bound' }
$results = [ordered]@{}
Write-Step "Ports en écoute :"
$listen | ForEach-Object { Write-Step ("   {0}:{1} (pid {2})" -f $_.LocalAddress, $_.LocalPort, $_.OwningProcess) }
$results['écoute uniquement sur 127.0.0.1'] = ($listen.Count -gt 0) -and -not ($listen | Where-Object { $_.LocalAddress -notin '127.0.0.1', '::1' })
$expected = $Layout.MariaDbPort, $Layout.LoginPort, $Layout.WorldPort
$results["ports attendus $($expected -join ', ') présents"] = ($expected | ForEach-Object { $_ -in ($listen.LocalPort) }) -notcontains $false
$remote = $established | Where-Object { $_.RemoteAddress -notin '127.0.0.1', '::1', '0.0.0.0' }
Write-Step "Connexions établies non locales : $($remote.Count)"
$results['aucune connexion sortante non locale'] = ($remote.Count -eq 0)
$null = $p.CloseMainWindow()
if (-not $p.WaitForExit(180000)) { $p.Kill() }
$ok = $true
foreach ($k in $results.Keys) { Write-Step ("{0,-44} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
