# Launcher — HomeRPG ArchLord (Rust)

Licence propriétaire HomeRPG (voir `../LICENCE.txt`) ; les bibliothèques Rust utilisées restent sous leurs
licences MIT/Apache-2.0 et assimilées. Un seul exécutable, 100 % Rust, CRT liée statiquement (aucun
« Visual C++ Redistributable » requis).

## Construction

```bash
cargo build --release      # target/release/homerpg-archlord-launcher.exe
cargo test --release
```

Prérequis : rustup (cible `x86_64-pc-windows-msvc`), Build Tools 2022, SDK Windows (`rc.exe`, pour l'icône
`assets/homerpg.ico` générée par `../Build/New-Icon.ps1`).

L'exécutable détecte la racine d'installation en remontant jusqu'au dossier contenant `Server/` et `Tools/`
(ou `HOMERPG_ROOT`) : lancé depuis `Launcher/target/release`, il utilise le dossier du dépôt.

## Modules

| Fichier | Rôle |
|---|---|
| `paths.rs` | disposition des dossiers, ports (MariaDB 3316, ArchCore 11002 / 11008 par défaut), bases `archcore_*` |
| `credentials.rs` | compte SQL local (`UserData/db-credentials.json`) |
| `settings.rs` | réglages utilisateur (`UserData/settings.json`) : langues, client, tables du serveur, ports, sauvegardes, journaux |
| `db.rs` | MariaDB portable : `my.ini`, initialisation, démarrage, santé, `SHUTDOWN`, compte SQL, comptes ArchCore, explorateur |
| `server.rs` | `auth.conf` / `world.conf` depuis les modèles d'ArchCore, outils (`db-init`, `decrypt`, `import`, `i18n`), serveur `archcore all`, arrêt par Ctrl+C |
| `client.rs` | ArchRust : détection, `realmlist.wtf`, langue (`options.toml`), lancement avec `RUST_LOG` |
| `packs.rs`, `addons.rs` | packs de ressources (`patches/`, `packs.toml`) et AddOns Lua (`addons/`) d'ArchRust |
| `logs.rs` | journaux serveur et client : lecture des sorties, fichiers horodatés, rotation, niveaux |
| `orchestrator.rs` | machine à états, premier démarrage (import du monde), surveillance, comptes, sauvegardes, réparation |
| `backup.rs`, `repair.rs` | sauvegardes vérifiées et restauration ; manifeste de fichiers, diagnostic |
| `conf_catalog.rs` | réglages de world.conf exposés dans l'onglet Monde |
| `ui.rs` | interface egui (9 onglets) |
| `i18n.rs` | toutes les chaînes, français et anglais ; macro `tf!` pour les messages composés |
| `util.rs` | horodatage local, SHA-256, copie d'arborescence |

## Machine à états

`NotInstalled → Installed → Verifying → StartingDb → Initializing → ImportingWorld → StartingServer → Ready ⇄
Playing → Stopping → Installed`, `Error` depuis tout état de démarrage, `Repairing` pour la maintenance.
JOUER n'est actif qu'avec un client valide ; la fermeture de la fenêtre arrête d'abord les services.
