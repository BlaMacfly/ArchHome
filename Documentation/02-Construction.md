# 02 — Construction

## Prérequis du poste

- rustup (toolchain stable `x86_64-pc-windows-msvc`), Visual Studio Build Tools 2022 (« Développement Desktop
  C++ »), SDK Windows (`rc.exe` pour l'icône, `signtool.exe` pour la signature).
- git, PowerShell 5.1. Les scripts sont en UTF-8 avec BOM.

## Disposition

| Dossier | Contenu | Versionné |
|---|---|---|
| `Launcher/` | launcher Rust (egui) | oui |
| `Build/` | manifeste, scripts de construction et de package, textes du package | oui |
| `Tools/` | scripts d'exploitation (`Common.ps1`, MariaDB, outils ArchCore) | oui |
| `Tests/` | tests d'intégration | oui |
| `Documentation/` | manuels, architecture, construction, tests | oui |
| `Server/ArchCore/` | ArchCore compilé (`Build-ArchCore.ps1`) | non |
| `Server/Database/mariadb/`, `Tools/HeidiSQL/` | composants tiers (`Install-Components.ps1`) | non |
| `Server/Config/` | configurations générées | non (sauf README) |
| `Server/Data/`, `Client/`, `UserData/`, `Logs/` | données du jeu, données personnelles | jamais |

## Étapes

```powershell
# ArchCore : compilé depuis une copie locale (branche Core) ou cloné au commit du manifeste.
# --target-dir Build\tmp\archcore-target et --locked : le dépôt source n'est pas modifié.
powershell -ExecutionPolicy Bypass -File Build\Build-ArchCore.ps1 -Source "C:\…\ArchCore-core"

# MariaDB 13.0.2 et HeidiSQL 12.21 : téléchargés dans Build\downloads, empreintes figées (-Pin la première fois).
powershell -ExecutionPolicy Bypass -File Build\Get-Components.ps1 -Pin
powershell -ExecutionPolicy Bypass -File Build\Install-Components.ps1
# Poste de développement : MariaDB déjà installée, même version.
powershell -ExecutionPolicy Bypass -File Build\Install-Components.ps1 -MariaDbFrom "C:\Program Files\MariaDB 13.0"

# Icône et logos (après un changement de Launcher\assets\HomeRPG-original.png)
powershell -ExecutionPolicy Bypass -File Build\New-Icon.ps1

# Launcher
cd Launcher; cargo build --release; cargo test --release; cd ..

# Signature (facultative, certificat de signature de code) et package
powershell -ExecutionPolicy Bypass -File Build\Sign-Launcher.ps1 -Thumbprint <empreinte>
powershell -ExecutionPolicy Bypass -File Build\New-Package.ps1 -Zip
```

`New-Package.ps1` produit `Build\package\HomeRPG-ArchLord-v<version>\` : `HomeRPG ArchLord.exe`, `Server\`
(ArchCore, MariaDB allégée, dossiers vides `Config`, `Data\server-ini` avec un LISEZMOI), `Client\` (LISEZMOI),
`Tools\Scripts`, `Documentation`, `LICENCES` (dont `SOURCES.md` : versions, commits, licences, crates Rust),
`Sources` (launcher et scripts), `Build\files-manifest.json` (empreintes utilisées par Vérifier) et
`SHA256SUMS.txt`.

## Mise à jour d'ArchCore

1. Changer `archcore.commit` dans `Build/manifest.json`.
2. `Build-ArchCore.ps1`, puis `Tests/Test-Launcher.ps1` et `Tests/Test-Integration.ps1`.
3. Les migrations s'appliquent seules au démarrage suivant (`archcore db-init`), chez l'utilisateur aussi.

## Lancer le launcher en développement

`Launcher\target\release\homerpg-archlord-launcher.exe` trouve la racine en remontant jusqu'au dossier qui
contient `Server\` et `Tools\` (ou `HOMERPG_ROOT`). Options : `--start`, `--quit-after N`, `--lang fr|en`,
`--tab <onglet>`, `--do play|backup|verify|repair|diagnostic|restore <zip>|reimportworld|acccreate <compte>
<mdp> [niveau]|accdel <compte>|packon <id>|packoff <id>|addonadd <zip>|addonon <nom>|addonoff <nom>|addondel
<nom>`, `--install-to <dossier>`.
