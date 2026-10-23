<#
.SYNOPSIS
    Lance un outil d'ArchCore sur les bases de HomeRPG (MariaDB démarrée, configuration générée par le launcher).

.EXAMPLE
    Tools\Invoke-ArchCoreTool.ps1 db-check
    Tools\Invoke-ArchCoreTool.ps1 sim client moncompte monmotdepasse MonPerso
#>
param([Parameter(ValueFromRemainingArguments)][string[]]$Arguments)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\Common.ps1"
if (-not (Test-MariaDbAlive)) { throw "MariaDB n'est pas en service (Tools\Start-MariaDB.ps1, ou le launcher)" }
$code = Invoke-ArchCore @Arguments
exit $code
