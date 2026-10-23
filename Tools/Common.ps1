# Fonctions partagées par les scripts Tools/, Build/ et Tests/. À « dot-sourcer » : . "$PSScriptRoot\Common.ps1"
# Tous les chemins sont relatifs à la racine du projet : premier dossier parent contenant Server\ (Tools\ en
# développement, Tools\Scripts\ dans le package distribué).

$script:ProjectRoot = Split-Path -Parent $PSScriptRoot
while ($script:ProjectRoot -and -not (Test-Path (Join-Path $script:ProjectRoot 'Server'))) { $script:ProjectRoot = Split-Path -Parent $script:ProjectRoot }
if (-not $script:ProjectRoot) { throw "Racine HomeRPG introuvable (aucun dossier Server au-dessus de $PSScriptRoot)" }

function Get-ProjectPath {
    param([Parameter(Mandatory)][string]$Relative)
    return Join-Path $script:ProjectRoot $Relative
}

# Dossiers et ports documentés (voir Documentation/02-Construction.md). Les ports d'ArchCore sont ceux par défaut ;
# Paramètres du launcher peut les changer (UserData\settings.json : login_port, world_port).
$script:Layout = [ordered]@{
    ArchCore     = Get-ProjectPath 'Server\ArchCore'
    Config       = Get-ProjectPath 'Server\Config'
    MariaDbBin   = Get-ProjectPath 'Server\Database\mariadb\bin'
    MariaDbData  = Get-ProjectPath 'UserData\Database'        # données personnelles : hors Server/
    MariaDbIni   = Get-ProjectPath 'Server\Config\my.ini'
    Credentials  = Get-ProjectPath 'UserData\db-credentials.json'
    Settings     = Get-ProjectPath 'UserData\settings.json'
    Logs         = Get-ProjectPath 'Logs'
    MariaDbPort  = 3316
    LoginPort    = 11002
    WorldPort    = 11008
}
if (Test-Path $script:Layout.Settings) {
    $s = Get-Content $script:Layout.Settings -Raw -Encoding UTF8 | ConvertFrom-Json
    if ($s.login_port) { $script:Layout.LoginPort = [int]$s.login_port }
    if ($s.world_port) { $script:Layout.WorldPort = [int]$s.world_port }
}

function Write-Step { param([string]$Message) Write-Host "[$(Get-Date -Format HH:mm:ss)] $Message" }

function Test-Port {
    param([int]$Port)
    try { $c = New-Object Net.Sockets.TcpClient; $c.Connect('127.0.0.1', $Port); $c.Close(); return $true } catch { return $false }
}

function Get-DbCredentials {
    # Identifiants du compte SQL local (créés par le launcher au premier démarrage, stockés hors de Server/).
    $path = $script:Layout.Credentials
    if (-not (Test-Path $path)) { throw "Identifiants absents : $path (lancer d'abord le launcher une fois)" }
    return Get-Content $path -Raw -Encoding UTF8 | ConvertFrom-Json
}

function Test-MariaDbAlive {
    param([int]$TimeoutSeconds = 0)
    if (-not (Test-Path $script:Layout.Credentials)) { return (Test-Port $script:Layout.MariaDbPort) }
    $admin = Join-Path $script:Layout.MariaDbBin 'mariadb-admin.exe'
    $rootPwd = (Get-DbCredentials).rootPassword
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        # cmd /c pour absorber la sortie d'erreur : sous PowerShell 5.1, « 2>$null » sur un exe natif
        # transforme chaque ligne stderr en erreur (NativeCommandError) et casse le script avec ErrorActionPreference=Stop.
        cmd /c "`"$admin`" --host=127.0.0.1 --port=$($script:Layout.MariaDbPort) --protocol=TCP --user=root --password=$rootPwd ping >nul 2>&1"
        if ($LASTEXITCODE -eq 0) { return $true }
        if ($TimeoutSeconds -gt 0) { Start-Sleep -Milliseconds 500 }
    } while ((Get-Date) -lt $deadline)
    return $false
}

function Invoke-Sql {
    # Exécute une requête via le client mariadb.exe (TCP, loopback, root). Renvoie les lignes (tabulations).
    param([string]$Database = '', [Parameter(Mandatory)][string]$Query)
    $cred = Get-DbCredentials
    $client = Join-Path $script:Layout.MariaDbBin 'mariadb.exe'
    $args = @('--host=127.0.0.1', "--port=$($script:Layout.MariaDbPort)", '--protocol=TCP', '--user=root', "--password=$($cred.rootPassword)", '--default-character-set=utf8mb4', '--batch', '--skip-column-names')
    if ($Database) { $args += $Database }
    $args += @('--execute', $Query)
    & $client @args
    if ($LASTEXITCODE -ne 0) { throw "Échec SQL (code $LASTEXITCODE)" }
}

function Invoke-ArchCore {
    # Lance un outil d'ArchCore (db-init, db-check, sim…) avec la configuration générée par le launcher.
    param([Parameter(ValueFromRemainingArguments)][string[]]$Arguments)
    $exe = Join-Path $script:Layout.ArchCore 'archcore.exe'
    if (-not (Test-Path (Join-Path $script:Layout.Config 'world.conf'))) { throw "Configuration absente : démarrer d'abord le serveur une fois avec le launcher" }
    $env:ARCHCORE_ROOT = $script:Layout.ArchCore
    $env:ARCHCORE_CONF_DIR = $script:Layout.Config
    $env:MSYS_NO_PATHCONV = '1'
    Push-Location $script:Layout.ArchCore
    try { & $exe @Arguments | Out-Host } finally { Pop-Location }
    return $LASTEXITCODE
}
