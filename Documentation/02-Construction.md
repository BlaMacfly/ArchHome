# 02 — Construction

## Prérequis du poste

- rustup (toolchain 1.98.1 `x86_64-pc-windows-msvc`), Visual Studio Build Tools 2022 (« Développement Desktop
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
| `Server/ArchCore/` | ArchCore compilé avec ArchBot (`Build-ArchCore.ps1`) | non |
| `Server/Database/mariadb/`, `Tools/HeidiSQL/` | composants tiers (`Install-Components.ps1`) | non |
| `Server/Config/` | configurations générées | non (sauf README) |
| données de jeu sous `Server/Data/` et `Client/`, `UserData/`, `Logs/` | données du jeu, données personnelles | jamais (le package reçoit `Server/Data/server-ini` par `-ServerTables`) |

## Étapes

```powershell
# ArchCore + ArchBot : copie propre d'ArchCore (clone de -Source au commit du manifeste) dans Build\tmp\ArchCore-src,
# ArchBot dans modules\ArchBot (arbre de travail de -ArchBot, ou clone au commit du manifeste), patch d'ArchBot,
# cargo build --locked --target-dir Build\tmp\archcore-target, toolchain 1.98.1. Le dépôt -Source n'est jamais modifié.
powershell -ExecutionPolicy Bypass -File Build\Build-ArchCore.ps1 -Source "C:\…\ArchCore-core" -ArchBot "C:\…\ArchBot"

# ArchRust : compilé depuis son dépôt au commit du manifeste, dans un dossier cible à part.
git clone https://github.com/BlaMacfly/ArchRust Build\tmp\ArchRust-src; git -C Build\tmp\ArchRust-src checkout 7a6c7f3
cargo +1.98.1 build --release -p archrust --locked --manifest-path Build\tmp\ArchRust-src\Cargo.toml --target-dir Build\tmp\archrust-target

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
& .\Build\New-Package.ps1 -Zip -ArchRustExe Build\tmp\archrust-target\x86_64-pc-windows-msvc\release\archrust.exe `
    -Addons Build\tmp\ArchRust-src\addons\ArchAce,Build\tmp\ArchRust-src\addons\HelloArch,"C:\…\ArchCore Admin\ArchCoreAdmin" `
    -ServerTables Build\tmp\ArchCore-src\data\server-ini   # tables du serveur d'origine livrées par ArchCore (copie de Build-ArchCore)

# Paquet des testeurs prêt à jouer (dossier + .7z + SHA256SUMS, sur le Bureau, jamais dans un dépôt) : le package
# ci-dessus, les données du jeu d'un client complet dans Client\ et, avec -InitDatabase, la base déjà initialisée et
# importée aux valeurs de l'épisode 8 avec Client\server-data (diffusion privée aux testeurs, comme le paquet 0.2.0).
& .\Build\New-TesterPackage.ps1 -GameData "C:\…\Client ArchRust" -OutDir "C:\…\Bureau\HomeRPG ArchLord v0.3.4" -HardLinks -InitDatabase `
    -WebzenDir "C:\…\client-webzen-2017\app\ini" -TranslationsDir "C:\…\data\traductions"   # données d'origine et traductions ; seule la base va dans le paquet
# -HardLinks : données du jeu en liens physiques vers -GameData (même disque, aucun espace en plus ; n'y rien modifier).
```

`-Addons` prend une liste PowerShell : appeler le script avec `&` dans PowerShell (avec `powershell -File`, la liste
arrive en une seule chaîne).

`New-Package.ps1` produit `Build\package\HomeRPG-ArchLord-v<version>\` : `HomeRPG ArchLord.exe`, `Server\`
(ArchCore, MariaDB allégée, dossiers vides `Config`, `Data\server-ini` avec un LISEZMOI), `Client\` (`ArchRust.exe`,
les AddOns de `-Addons` dans `addons\`, hors du contrôle des fichiers, et un LISEZMOI, sans données du jeu),
`Data\server-ini` (les tables du serveur d'origine lues par `archcore import --server` : tout le dossier, les 31
tables exigées étant contrôlées, `objects\` et `world\node_data`), `Server\ArchCore\archbot` (patch d'ArchBot
appliqué et licence),
`LISEZMOI-TESTEURS.txt`, `README-TESTERS.txt`, `Documentation\CHANGELOG.md`,
`Tools\Scripts`, `Documentation`, `LICENCES` (dont `SOURCES.md` : versions, commits, licences, crates Rust),
`Sources` (launcher et scripts), `Build\files-manifest.json` (empreintes utilisées par Vérifier) et
`SHA256SUMS.txt`.

## Mise à jour d'ArchCore et d'ArchBot

1. Changer `archcore.commit` (et `archbot.commit`) dans `Build/manifest.json`. Si le patch d'ArchBot ne s'applique
   plus, `Build-ArchCore.ps1` s'arrête : l'adapter dans le dépôt ArchBot (`docs/ARCHCORE-CHANGES.md`), jamais
   dans ArchCore.
2. `Build-ArchCore.ps1`, puis `Tests/Test-Launcher.ps1` et `Tests/Test-Integration.ps1`.
3. Les migrations s'appliquent seules au démarrage suivant (`archcore db-init`), chez l'utilisateur aussi ; le
   monde est réimporté une fois, car `UserData\monde-importe-par.txt` ne correspond plus à
   `Server\ArchCore\VERSION.txt` (« Core <commit> (<date>) + ArchBot <commit> »), puis `Client\server-data` est
   réécrit.

`rustc` plante parfois sur ce PC (STATUS_ACCESS_VIOLATION) : la toolchain 1.98.1 est imposée par
`Build-ArchCore.ps1`, qui relance la compilation jusqu'à trois fois (`CARGO_INCREMENTAL=0`). La toolchain stable
plante : ne pas l'utiliser.

Pour ArchRust, le packageur copie l'exécutable release donné par `-ArchRustExe`. Mettre à jour le commit
`archrust.commit` du manifeste au commit source correspondant, ainsi que la liste `addons` (source et commit de
chaque AddOn) ; les données du jeu ne sont jamais copiées.

## Lancer le launcher en développement

`Launcher\target\release\homerpg-archlord-launcher.exe` trouve la racine en remontant jusqu'au dossier qui
contient `Server\` et `Tools\` (ou `HOMERPG_ROOT`). Options : `--start`, `--quit-after N`, `--lang fr|en`,
`--tab <onglet>`, `--do play|backup|verify|repair|diagnostic|restore <zip>|reimportworld|acccreate <compte>
<mdp> [niveau]|accdel <compte>|packon <id>|packoff <id>|addonadd <zip>|addonon <nom>|addonoff <nom>|addondel
<nom>`, `--install-to <dossier>`.
