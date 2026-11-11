<p align="center">
  <img src="Launcher/assets/logo-256.png" alt="ArchHome" width="180">
</p>

<h1 align="center">ArchHome</h1>

<p align="center">
  <b>ArchLord en solo, hors ligne — Single-player, offline ArchLord</b><br>
  Serveur ArchCore local avec ses bots · client ArchRust · launcher Rust à un seul bouton · interface français / English
</p>

<p align="center">
  <a href="#-français">🇫🇷 Français</a> ·
  <a href="#-english">🇬🇧 English</a> ·
  <a href="Documentation/Manuel-Utilisateur.md">📖 Manuel</a> ·
  <a href="Documentation/User-Manual.md">📖 Manual</a> ·
  <a href="https://discord.gg/JCyrsTm7fR">💬 Discord</a>
</p>

---

## 🇫🇷 Français

**ArchHome** transforme le client ArchLord 64 bits en jeu solo complet, sans serveur distant, sans compte
en ligne, sans rien installer dans Windows. Tout tient dans un dossier :

| Composant | Rôle |
|---|---|
| [ArchCore](https://github.com/BlaMacfly/ArchCore) | serveur ArchLord (connexion + monde) en Rust, valeurs d'origine de l'épisode 8, sur 127.0.0.1 uniquement |
| [ArchBot](https://github.com/BlaMacfly/ArchBot) | module PlayerBots compilé dans ArchCore : bots autonomes qui peuplent le monde, compagnons à engager |
| MariaDB portable | comptes, personnages et monde importé, sur 127.0.0.1:3316 |
| Launcher ArchHome (Rust, egui) | démarre tout dans le bon ordre, importe le monde au premier lancement, configure ArchRust, crée les comptes, sauvegarde, répare, et expose les réglages dans des onglets |
| [ArchRust](https://github.com/BlaMacfly/ArchRust) | le client ArchLord réécrit en Rust, inclus dans le package ; le joueur fournit les données du jeu |

Le package fournit l'exécutable ArchRust et ses AddOns ; le joueur fournit les données du jeu ArchLord. Le serveur en importe
le monde au premier démarrage. Le client d'origine n'est pas pris en charge : ArchHome joue avec ArchRust.

### Fonctionnalités

**Jouer**

- Un bouton **JOUER** : base de données → schéma (`archcore db-init`) → monde → tables pour ArchRust →
  ArchCore et ses bots → configuration d'ArchRust (`realmlist.wtf`, langue) → lancement du jeu.
- Premier démarrage automatique : monde importé aux **valeurs d'origine de l'épisode 8** (tables du serveur
  ArchLord d'origine, incluses dans le package) avec les noms et visuels du client, relief, monstres et butin,
  titres de PNJ traduits — environ une minute ; refait seul après une mise à jour d'ArchCore (personnages
  conservés, et passés une fois aux valeurs de l'épisode 8 après une sauvegarde en venant de la 0.2.0).
- Tables de l'épisode 8 écrites pour ArchRust dans `Client\server-data` (`archcore export-client-tables`) : les
  infobulles, prix et conditions du jeu sont ceux du serveur, empreinte vérifiée à la connexion.
- **Bots** (ArchBot) : une population de bots autonomes qui chassent, ramassent et progressent, et des
  compagnons à engager en jeu (`/bot hire`) qui vous suivent, vous défendent et obéissent à vos ordres.

**Régler**

- **Langues** : launcher en français ou en anglais (automatique d'après Windows, changeable à chaud) ; langue
  du jeu (textes du serveur et d'ArchRust).
- Onglet **Monde** : densité de monstres, taux (expérience, or, butin, charisme, prestige), horloge du ciel,
  niveau MJ requis par commande, seuils techniques. Valeurs d'origine rappelées, retour aux défauts en un clic.
- Onglet **ArchBot** : tous les réglages des bots (activation, population, classes, comportement, survie,
  performances, commandes MJ) et les commandes du jeu.
- **Packs et AddOns** : packs de ressources d'ArchRust (activer, désactiver, ajouter `.ahp` ou dossier,
  supprimer ; même fichier que la fenêtre Options du jeu) et AddOns Lua (installation depuis un zip,
  activation, suppression).
- **Ports** de connexion et de monde réglables (11002 / 11008 par défaut).

**Protéger**

- Sauvegardes vérifiées par empreinte (comptes, personnages, réglages), automatiques à chaque fermeture du
  jeu avec rotation, sauvegarde de sécurité avant toute restauration.
- Vérification des fichiers par manifeste SHA-256, réparation (bases et migrations, configurations, monde),
  réimport de la base monde sans perte de personnages.
- **Journaux** côté serveur (ArchCore, imports) et côté client (ArchRust, niveau réglable), un fichier par
  session dans `Logs\`, onglet Journaux avec filtre et mise en couleur, export de diagnostic.
- Comptes : création avec niveau MJ, mot de passe, suppression avec confirmation — serveur arrêté ou en marche.
- Base de données : explorateur intégré (grille éditable, structure, SQL avec confirmation d'écriture) et
  bouton vers HeidiSQL.
- Aide MJ : les commandes d'ArchCore et les raccourcis d'ArchRust dans un onglet.

### Prérequis

| Élément | Minimum |
|---|---|
| Système | Windows 10 ou 11, 64 bits |
| Mémoire | 8 Go |
| Espace disque | ≈ 1 Go pour ArchHome, en plus du client |
| Client | ArchRust (`Client\ArchRust.exe`, inclus) et données du jeu ArchLord 64 bits fournies par vous |
| Réseau | aucun : tout fonctionne hors ligne |

### Installation

1. Décompressez l'archive dans un dossier de votre disque (pas dans Program Files).
2. Placez les données du jeu ArchLord 64 bits dans `Client\` à côté d'`ArchRust.exe`, ou indiquez leur dossier dans **Paramètres**.
3. Rien d'autre : les tables du serveur d'origine (valeurs de l'épisode 8, monstres, butin) sont incluses dans
   `Server\Data\server-ini\`.
4. Ouvrez `ArchHome.exe`, créez votre compte dans **Comptes**, puis cliquez sur **JOUER**.

Le [manuel d'utilisation](Documentation/Manuel-Utilisateur.md) détaille chaque onglet et le dépannage.

### Ce dépôt et ce qu'il ne contient pas

Ce dépôt contient le code source du launcher, les scripts de construction, de test et d'exploitation, la
documentation. L'archive générée inclut aussi l'exécutable ArchRust sous GPL, issu du commit noté dans
`Build/manifest.json`, avec son lien vers le code source. Le dépôt et le package ne contiennent pas :

- les données du client ArchLord ni rien qui en dérive (`Client/` hors `ArchRust.exe` et ses AddOns,
  `Server/Data/client-ini`) : jamais redistribuées. Le package inclut les tables du serveur d'origine
  (`Server/Data/server-ini`), jamais versionnées dans ce dépôt ;
- les binaires des composants autres qu'ArchRust : ArchCore est compilé depuis ses sources au commit figé dans
  `Build/manifest.json`, avec ArchBot et son patch (`Build/Build-ArchCore.ps1`) ; MariaDB et HeidiSQL sont figés par URL et empreinte
  SHA-256 (`Build/Get-Components.ps1`, `Build/Install-Components.ps1`) ;
- les données personnelles (`UserData/`, `Logs/`).

### Construire depuis les sources

Prérequis : Windows, rustup (toolchain 1.98.1 MSVC : la stable plante sur le poste de développement), Visual
Studio Build Tools 2022, SDK Windows (`rc.exe`), git, PowerShell 5.1, 7-Zip (paquet des testeurs).

```powershell
# 1. ArchCore avec ArchBot (copie clonée au commit du manifeste ; le dépôt source n'est pas modifié)
powershell -ExecutionPolicy Bypass -File Build\Build-ArchCore.ps1 [-Source <dossier ArchCore>] [-ArchBot <dossier ArchBot>]
# 2. MariaDB et HeidiSQL (téléchargés, empreintes vérifiées), puis installés à leur place
powershell -ExecutionPolicy Bypass -File Build\Get-Components.ps1 -Pin
powershell -ExecutionPolicy Bypass -File Build\Install-Components.ps1
# 3. Launcher
cd Launcher; cargo build --release; cd ..
# 4. Package portable (dossier + zip), puis paquet des testeurs (données du jeu, .7z, SHA256SUMS)
& .\Build\New-Package.ps1 -Zip -ArchRustExe <ArchRust.exe> -Addons <dossiers d'AddOns> -ServerTables <tables du serveur>
& .\Build\New-TesterPackage.ps1 -GameData <client complet> -OutDir <dossier de sortie>
```

Détails : [Documentation/02-Construction.md](Documentation/02-Construction.md).

### Tests

| Script | Vérifie |
|---|---|
| `Tests\Test-Launcher.ps1` | démarrage complet par le launcher, import du monde, arrêt propre d'ArchCore, ports libérés, aucun orphelin, aucune erreur |
| `Tests\Test-Integration.ps1` | compte créé, client simulé qui crée un personnage et entre en jeu, connexion ArchRust sans affichage, sauvegarde, suppression du compte serveur arrêté |
| `Tests\Test-Offline.ps1` | services sur 127.0.0.1 seulement, aucune connexion sortante |

Ne lancez jamais un test pendant que votre launcher est ouvert.

### Architecture

```
ArchHome.exe            launcher Rust (eframe/egui 0.32, crate mysql pur Rust, aucune DLL)
├─ orchestrateur (thread)       machine à états, premier démarrage, surveillance, commandes de l'interface
├─ db.rs / server.rs            MariaDB portable, comptes ; auth.conf / world.conf / archbot.conf, outils ArchCore, Ctrl+C
├─ client.rs / packs.rs / addons.rs   ArchRust : realmlist.wtf, langue, lancement, packs, AddOns
├─ logs.rs                      journaux launcher / serveur / client, rotation
├─ backup.rs / repair.rs        sauvegardes vérifiées, manifeste de fichiers, diagnostic
└─ i18n.rs                      toutes les chaînes, français et anglais
Server/   ArchCore + ArchBot (archcore.exe, sql, content, i18n), MariaDB, configurations générées, tables
UserData/ base vivante, réglages, sauvegardes (à conserver)
Logs/     launcher.log, server/archcore-*.log, server/outils-*.log, client/archrust-*.log
Tools/    scripts PowerShell d'exploitation (Common.ps1 : chemins, ports, SQL)
Build/    manifeste des composants, compilation d'ArchCore, packaging
```

### Licence

- **ArchHome** (launcher, scripts, documentation, nom et logo) : © 2026 BlaMacfly, licence
  propriétaire [LICENCE.txt](LICENCE.txt) — usage personnel, consultation et fork sur GitHub autorisés ;
  redistribution, modification et usage commercial interdits sans accord écrit.
- Composants tiers, sous leurs licences propres : ArchCore, ArchBot, ArchRust (GPL-3.0-or-later), MariaDB (GPL-2.0), HeidiSQL
  (GPL-2.0), bibliothèques Rust (MIT/Apache-2.0 et assimilées).
- ArchLord est une marque de son titulaire. ArchHome est une création indépendante, sans lien avec lui. Les
  données du jeu ne sont pas fournies.

---

## 🇬🇧 English

**ArchHome** turns the 64-bit ArchLord client into a complete single-player game: no remote server, no
online account, nothing installed into Windows. Everything lives in one folder: a local
[ArchCore](https://github.com/BlaMacfly/ArchCore) server (login + world, Rust, original Episode 8 values) with
its [ArchBot](https://github.com/BlaMacfly/ArchBot) bots, bound to 127.0.0.1, a portable MariaDB, and a one-button
Rust launcher that starts everything in order, imports the world on first start, configures
[ArchRust](https://github.com/BlaMacfly/ArchRust), creates accounts, backs up and repairs. The package includes the ArchRust executable; you provide the ArchLord game data. The original client
is not supported.

**Highlights**

- **PLAY** button: database → schema → world import with the original Episode 8 values (about a minute, once)
  → Episode 8 tables written for ArchRust (`Client\server-data`) → ArchCore and its bots → ArchRust.
- **Bots** (ArchBot): autonomous bots that hunt, loot and level up, and companions you hire in game
  (`/bot hire`) who follow you, defend you and obey your orders.
- **Bilingual interface**: French or English (follows Windows, switchable live in Settings), plus a game
  language for server texts and ArchRust.
- **World** tab: monster density, rates, sky clock, GM command levels, technical thresholds.
- **ArchBot** tab: every bot setting (activation, population, classes, behaviour, survival, performance, GM
  commands) and the in-game commands.
- **Packs & AddOns**: ArchRust resource packs and Lua AddOns.
- **Backups** verified by checksum, automatic when the game closes; repair, world reimport, file check.
- **Logs** for the server (ArchCore, imports) and the client (ArchRust, adjustable level), one file per
  session in `Logs\`, a Logs tab with filter and colours, and a diagnostic export.
- Accounts, database explorer with HeidiSQL button, GM help tab.

The original server tables are included: Episode 8 values, monsters and loot from the first start. Requirements: Windows 10/11 64-bit, 8 GB,
your ArchRust client. Installation and every tab are described in the
[user manual](Documentation/User-Manual.md); building from source in
[02-Construction.md](Documentation/02-Construction.md) (French).

**Licence**: ArchHome is proprietary (personal use; viewing and forking on GitHub allowed; no
redistribution, modification or commercial use without written permission) — see [LICENCE.txt](LICENCE.txt).
Third-party components keep their own licences (ArchCore, ArchBot, ArchRust GPL-3.0-or-later, MariaDB and HeidiSQL GPL-2.0,
Rust crates MIT/Apache-2.0). No ArchLord game data is provided.
