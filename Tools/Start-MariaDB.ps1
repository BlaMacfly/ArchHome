<#
.SYNOPSIS
    Démarre la MariaDB portable du projet sur 127.0.0.1:3316, sans le launcher (maintenance, HeidiSQL).
    Le datadir et le my.ini doivent exister : lancer d'abord le launcher une fois.
#>
[CmdletBinding()]
param([int]$TimeoutSeconds = 60)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"

$mariadbd = Join-Path $Layout.MariaDbBin 'mariadbd.exe'
if (-not (Test-Path $mariadbd)) { throw "MariaDB absente : $mariadbd (Build\Install-Components.ps1)" }
if (-not (Test-Path $Layout.MariaDbIni) -or -not (Test-Path (Join-Path $Layout.MariaDbData 'mysql'))) { throw "Base pas encore initialisée : ouvrir le launcher et cliquer sur JOUER une première fois" }
if (Test-MariaDbAlive) { Write-Step "MariaDB déjà en service sur 127.0.0.1:$($Layout.MariaDbPort)"; exit 0 }

Write-Step "Démarrage de mariadbd"
$p = Start-Process -FilePath $mariadbd -ArgumentList "--defaults-file=`"$($Layout.MariaDbIni)`"" -WindowStyle Hidden -PassThru
if (-not (Test-MariaDbAlive -TimeoutSeconds $TimeoutSeconds)) {
    if ($p.HasExited) { throw "mariadbd s'est arrêté (code $($p.ExitCode)). Voir $($Layout.Logs)\mariadb-error.log" }
    throw "MariaDB ne répond pas après $TimeoutSeconds s"
}
Write-Step "MariaDB opérationnelle (PID $($p.Id))"
