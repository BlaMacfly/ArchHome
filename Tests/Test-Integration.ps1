<#
.SYNOPSIS
    Test de bout en bout : le launcher démarre tout et crée un compte de test (--do acccreate), un client simulé
    d'ArchCore (archcore sim client) se connecte, crée un personnage et entre en jeu, puis le client ArchRust rejoue
    la connexion sans affichage (ArchRust.exe login) si un client est configuré. Ensuite, sauvegarde et suppression
    du compte de test par le launcher, serveur arrêté (base démarrée le temps de l'opération).
#>
[CmdletBinding()]
param([string]$Exe, [string]$User = 'hrtest', [string]$Password = 'hrtest42', [string]$Character = 'HrTesteur')
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\..\Tools\Common.ps1"

if (-not $Exe) { $Exe = Get-ProjectPath 'Launcher\target\release\homerpg-archlord-launcher.exe' }
if (-not (Test-Path $Exe)) { throw "Launcher non compilé : $Exe" }
foreach ($p in $Layout.MariaDbPort, $Layout.LoginPort, $Layout.WorldPort) { if (Test-Port $p) { throw "Port $p déjà occupé" } }
$results = [ordered]@{}

Write-Step "1. Démarrage complet et création du compte $User"
$p = Start-Process -FilePath $Exe -ArgumentList @('--start', '--lang', 'fr', '--do', 'acccreate', $User, $Password, '3', '--quit-after', '7200') -WorkingDirectory (Split-Path $Exe) -PassThru
$deadline = (Get-Date).AddMinutes(30)
while (-not (Test-Port $Layout.WorldPort) -and -not $p.HasExited -and (Get-Date) -lt $deadline) { Start-Sleep -Seconds 1 }
$results['serveur prêt'] = Test-Port $Layout.WorldPort
Start-Sleep -Seconds 4
$acc = Invoke-Sql -Query "SELECT username, gm_level FROM archcore_auth.account WHERE username='$User'"
$results['compte créé (niveau 3)'] = ("$acc" -match "$User\s+3")

Write-Step "2. Client simulé : connexion, personnage, entrée en jeu"
$env:ARCHCORE_ROOT = $Layout.ArchCore; $env:ARCHCORE_CONF_DIR = $Layout.Config
$archcore = Join-Path $Layout.ArchCore 'archcore.exe'
$simOut = cmd /c "`"$archcore`" sim client $User $Password $Character 2>&1"
$simOut | Select-Object -Last 6 | ForEach-Object { "    $_" }
$results['archcore sim client : flux complet OK'] = (($simOut -join "`n") -match 'flux complet OK')

$settings = if (Test-Path $Layout.Settings) { Get-Content $Layout.Settings -Raw -Encoding UTF8 | ConvertFrom-Json } else { $null }
$client = if ($settings -and $settings.client_dir) { $settings.client_dir -replace '/', '\' } else { Get-ProjectPath 'Client' }
if (Test-Path (Join-Path $client 'ArchRust.exe')) {
    Write-Step "3. ArchRust sans affichage : $client"
    $arOut = cmd /c "cd /d `"$client`" && `"$client\ArchRust.exe`" login --port $($Layout.LoginPort) $User $Password $Character 2>&1"
    $arOut | Select-Object -Last 6 | ForEach-Object { "    $_" }
    $results['ArchRust login : code 0'] = ($LASTEXITCODE -eq 0)
} else {
    Write-Step "3. ArchRust absent ($client) : étape ignorée"
}

Write-Step "4. Fermeture du launcher"
$null = $p.CloseMainWindow()
if (-not $p.WaitForExit(180000)) { $p.Kill() }
$results['ports libérés'] = -not ((Test-Port $Layout.LoginPort) -or (Test-Port $Layout.WorldPort) -or (Test-Port $Layout.MariaDbPort))

Write-Step "5. Sauvegarde puis suppression du compte, serveur arrêté"
$bkDir = Get-ProjectPath 'UserData\Sauvegardes'
$nBefore = @(Get-ChildItem $bkDir -Filter '*.zip' -ErrorAction SilentlyContinue).Count
$p2 = Start-Process -FilePath $Exe -ArgumentList @('--lang', 'fr', '--do', 'backup', '--quit-after', '40') -WorkingDirectory (Split-Path $Exe) -PassThru
$null = $p2.WaitForExit(240000)
$results['sauvegarde créée'] = (@(Get-ChildItem $bkDir -Filter '*.zip' -ErrorAction SilentlyContinue).Count -gt $nBefore)
$p3 = Start-Process -FilePath $Exe -ArgumentList @('--lang', 'fr', '--do', 'accdel', $User, '--quit-after', '25') -WorkingDirectory (Split-Path $Exe) -PassThru
$null = $p3.WaitForExit(240000)
$log = Get-Content (Get-ProjectPath 'Logs\launcher.log') -Encoding UTF8 -Tail 40
$results['compte supprimé'] = (($log -join "`n") -match "compte $User supprimé")
$results['base arrêtée après les opérations'] = -not (Test-Port $Layout.MariaDbPort)

$ok = $true
foreach ($k in $results.Keys) { Write-Step ("{0,-42} {1}" -f $k, $(if ($results[$k]) { 'OK' } else { 'ÉCHEC' })); if (-not $results[$k]) { $ok = $false } }
if ($ok) { Write-Step "TEST RÉUSSI"; exit 0 } else { Write-Step "TEST EN ÉCHEC"; exit 1 }
