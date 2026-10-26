<p align="center">
  <img src="Launcher/assets/logo-256.png" alt="HomeRPG ArchLord" width="180">
</p>

<h1 align="center">HomeRPG ArchLord</h1>

<p align="center">
  <b>ArchLord en solo, hors ligne — Single-player, offline ArchLord</b><br>
  Serveur ArchCore local · client ArchRust · launcher Rust à un seul bouton · interface français / English
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

**HomeRPG ArchLord** transforme le client ArchLord 64 bits en jeu solo complet, sans serveur distant, sans compte
en ligne, sans rien installer dans Windows. Tout tient dans un dossier :

| Composant | Rôle |
|---|---|
| [ArchCore](https://github.com/BlaMacfly/ArchCore) | serveur ArchLord (connexion + monde) en Rust, sur 127.0.0.1 uniquement |
| MariaDB portable | comptes, personnages et monde importé, sur 127.0.0.1:3316 |
| Launcher HomeRPG (Rust, egui) | démarre tout dans le bon ordre, importe le monde au premier lancement, configure ArchRust, crée les comptes, sauvegarde, répare, et expose les réglages dans des onglets |
| [ArchRust](https://github.com/BlaMacfly/ArchCore/tree/client) | le client ArchLord réécrit en Rust, inclus dans le package ; le joueur fournit les données du jeu |

Le package fournit l'exécutable ArchRust et ses AddOns ; le joueur fournit les données du jeu ArchLord. Le serveur en importe
le monde au premier démarrage. Le client d'origine n'est pas pris en charge : HomeRPG joue avec ArchRust.

### Fonctionnalités

**Jouer**

- Un bouton **JOUER** : base de données → schéma (`archcore db-init`) → monde → ArchCore → configuration
  d'ArchRust (`realmlist.wtf`, langue) → lancement du jeu.
- Premier démarrage automatique : tables du client déchiffrées, monde importé (PNJ, objets, quêtes,
  compétences, relief), titres de PNJ traduits — environ une minute ; refait seul après une mise à jour
  d'ArchCore (personnages conservés).
- Monstres et butin si vous fournissez les tables du serveur d'origine (non incluses) ; sans elles, un
  avertissement le dit, et le monde est réimporté dès qu'elles sont ajoutées.

**Régler**

- **Langues** : launcher en français ou en anglais (automatique d'après Windows, changeable à chaud) ; langue
  du jeu (textes du serveur et d'ArchRust).
- Onglet **Monde** : densité de monstres, taux (expérience, or, butin, charisme, prestige), horloge du ciel,
  niveau MJ requis par commande, seuils techniques. Valeurs d'origine rappelées, retour aux défauts en un clic.
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
| Espace disque | ≈ 1 Go pour HomeRPG, en plus du client |
| Client | ArchRust (`Client\ArchRust.exe`, inclus) et données du jeu ArchLord 64 bits fournies par vous |
| Réseau | aucun : tout fonctionne hors ligne |

### Installation

1. Décompressez l'archive dans un dossier de votre disque (pas dans Program Files).
2. Placez les données du jeu ArchLord 64 bits dans `Client\` à côté d'`ArchRust.exe`, ou indiquez leur dossier dans **Paramètres**.
3. Facultatif : tables du serveur d'origine dans `Server\Data\server-ini\`.
4. Ouvrez `HomeRPG ArchLord.exe`, créez votre compte dans **Comptes**, puis cliquez sur **JOUER**.

Le [manuel d'utilisation](Documentation/Manuel-Utilisateur.md) détaille chaque onglet et le dépannage.

### Ce dépôt et ce qu'il ne contient pas

Ce dépôt contient le code source du launcher, les scripts de construction, de test et d'exploitation, la
documentation. L'archive générée inclut aussi l'exécutable ArchRust sous GPL, issu du commit noté dans
`Build/manifest.json`, avec son lien vers le code source. Le dépôt et le package ne contiennent pas :

- les données du jeu ArchLord ni rien qui en dérive (`Client/` hors `ArchRust.exe`, `Server/Data/`), ni les
  tables du serveur d'origine : jamais redistribuées ;
- les binaires des composants autres qu'ArchRust : ArchCore est compilé depuis ses sources au commit figé dans
  `Build/manifest.json` (`Build/Build-ArchCore.ps1`) ; MariaDB et HeidiSQL sont figés par URL et empreinte
  SHA-256 (`Build/Get-Components.ps1`, `Build/Install-Components.ps1`) ;
- les données personnelles (`UserData/`, `Logs/`).

### Construire depuis les sources

Prérequis : Windows, rustup (toolchain stable MSVC), Visual Studio Build Tools 2022, SDK Windows (`rc.exe`),
git, PowerShell 5.1.

```powershell
# 1. ArchCore (compilé depuis une copie locale du dépôt, ou cloné au commit du manifeste)
powershell -ExecutionPolicy Bypass -File Build\Build-ArchCore.ps1 [-Source <dossier ArchCore, branche Core>]
# 2. MariaDB et HeidiSQL (téléchargés, empreintes vérifiées), puis installés à leur place
powershell -ExecutionPolicy Bypass -File Build\Get-Components.ps1 -Pin
powershell -ExecutionPolicy Bypass -File Build\Install-Components.ps1
# 3. Launcher
cd Launcher; cargo build --release; cd ..
# 4. Package portable (dossier + zip)
powershell -ExecutionPolicy Bypass -File Build\New-Package.ps1 -Zip -ArchRustExe <ArchRust.exe> -Addons <dossiers d'AddOns>
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
HomeRPG ArchLord.exe            launcher Rust (eframe/egui 0.32, crate mysql pur Rust, aucune DLL)
├─ orchestrateur (thread)       machine à états, premier démarrage, surveillance, commandes de l'interface
├─ db.rs / server.rs            MariaDB portable, comptes ; auth.conf / world.conf, outils ArchCore, Ctrl+C
├─ client.rs / packs.rs / addons.rs   ArchRust : realmlist.wtf, langue, lancement, packs, AddOns
├─ logs.rs                      journaux launcher / serveur / client, rotation
├─ backup.rs / repair.rs        sauvegardes vérifiées, manifeste de fichiers, diagnostic
└─ i18n.rs                      toutes les chaînes, français et anglais
Server/   ArchCore (archcore.exe, sql, content, i18n), MariaDB, configurations générées, données dérivées
UserData/ base vivante, réglages, sauvegardes (à conserver)
Logs/     launcher.log, server/archcore-*.log, server/outils-*.log, client/archrust-*.log
Tools/    scripts PowerShell d'exploitation (Common.ps1 : chemins, ports, SQL)
Build/    manifeste des composants, compilation d'ArchCore, packaging
```

### Licence

- **HomeRPG ArchLord** (launcher, scripts, documentation, nom et logo) : © 2026 BlaMacfly, licence
  propriétaire [LICENCE.txt](LICENCE.txt) — usage personnel, consultation et fork sur GitHub autorisés ;
  redistribution, modification et usage commercial interdits sans accord écrit.
- Composants tiers, sous leurs licences propres : ArchCore (GPL-3.0-or-later), MariaDB (GPL-2.0), HeidiSQL
  (GPL-2.0), bibliothèques Rust (MIT/Apache-2.0 et assimilées).
- ArchLord est une marque de son titulaire. HomeRPG est une création indépendante, sans lien avec lui. Les
  données du jeu ne sont pas fournies.

---

## 🇬🇧 English

**HomeRPG ArchLord** turns the 64-bit ArchLord client into a complete single-player game: no remote server, no
online account, nothing installed into Windows. Everything lives in one folder: a local
[ArchCore](https://github.com/BlaMacfly/ArchCore) server (login + world, Rust) bound to 127.0.0.1, a portable
MariaDB, and a one-button Rust launcher that starts everything in order, imports the world on first start,
configures [ArchRust](https://github.com/BlaMacfly/ArchCore/tree/client), creates accounts, backs up and
repairs. The package includes the ArchRust executable; you provide the ArchLord game data. The original client
is not supported.

**Highlights**

- **PLAY** button: database → schema → world import (about a minute, once) → ArchCore → ArchRust.
- **Bilingual interface**: French or English (follows Windows, switchable live in Settings), plus a game
  language for server texts and ArchRust.
- **World** tab: monster density, rates, sky clock, GM command levels, technical thresholds.
- **Packs & AddOns**: ArchRust resource packs and Lua AddOns.
- **Backups** verified by checksum, automatic when the game closes; repair, world reimport, file check.
- **Logs** for the server (ArchCore, imports) and the client (ArchRust, adjustable level), one file per
  session in `Logs\`, a Logs tab with filter and colours, and a diagnostic export.
- Accounts, database explorer with HeidiSQL button, GM help tab.

Monsters and loot need the original server tables (not included). Requirements: Windows 10/11 64-bit, 8 GB,
your ArchRust client. Installation and every tab are described in the
[user manual](Documentation/User-Manual.md); building from source in
[02-Construction.md](Documentation/02-Construction.md) (French).

**Licence**: HomeRPG ArchLord is proprietary (personal use; viewing and forking on GitHub allowed; no
redistribution, modification or commercial use without written permission) — see [LICENCE.txt](LICENCE.txt).
Third-party components keep their own licences (ArchCore GPL-3.0-or-later, MariaDB and HeidiSQL GPL-2.0,
Rust crates MIT/Apache-2.0). No ArchLord game data is provided.
