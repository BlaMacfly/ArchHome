# 01 — Architecture et choix

HomeRPG ArchLord est l'équivalent, pour ArchLord, de HomeCraft Edition Vanilla (World of Warcraft 1.12.1) : un
dossier autonome, un serveur local, un launcher à un seul bouton. Ce document fixe ce qui a été repris, ce qui
change et pourquoi.

## Composants

| Rôle | HomeCraft Vanilla | HomeRPG ArchLord |
|---|---|---|
| Serveur | VMaNGOS : realmd + mangosd, binaires officiels | ArchCore : un seul `archcore.exe` (`all` = connexion + monde), compilé depuis ses sources avec le module ArchBot |
| Base | MariaDB portable, 127.0.0.1:3315 | MariaDB portable 13.0.2, 127.0.0.1:3316 |
| Monde | dumps SQL de VMaNGOS | importé au premier démarrage (`archcore decrypt` puis `import --server`) : valeurs de l'épisode 8 depuis les tables du serveur d'origine, noms et visuels depuis le client ; réimporté après une mise à jour d'ArchCore |
| Client | WoW.exe 1.12.1 ou Benilla | exécutable ArchRust inclus ; données du jeu fournies par le joueur (pas de client d'origine) |
| Compagnons | PartyBots | ArchBot (module PlayerBots d'ArchCore) : bots autonomes et compagnons, réglés dans l'onglet ArchBot (`archbot.conf`) |
| Patchs client | archives MPQ | packs de ressources d'ArchRust (`patches/`, `packs.toml`) |
| AddOns | `Interface\AddOns` | `addons/` d'ArchRust (Lua) |
| Langue | français | interface **française et anglaise** (décision du 2026-10-23), langue du jeu réglable |
| Journaux | launcher + console mangosd | launcher, **serveur** (ArchCore, outils) et **client** (ArchRust), un fichier par session (décision du 2026-10-23) |

## Démarrage (orchestrator.rs)

`NotInstalled → Installed → Verifying → StartingDb → Initializing → ImportingWorld → StartingServer → Ready ⇄
Playing → Stopping → Installed`, `Error` depuis tout état de démarrage.

1. MariaDB : `my.ini` généré, datadir initialisé au premier lancement (mot de passe root aléatoire), compte SQL
   `archcore` avec tous les droits sur `archcore\_%`.
2. `auth.conf` / `world.conf` / `archbot.conf` générés dans `Server/Config` (`ARCHCORE_CONF_DIR`) depuis les
   modèles `.dist` d'ArchCore et d'ArchBot : écoute sur 127.0.0.1, ports, accès SQL, `Client.DataDir` (données
   du jeu), `Client.TablesDir` (`server-data` du client), langue, réglages de l'onglet Monde (dont la section
   Bots). ArchCore lit le reste (sql, content, data/i18n) dans `Server/ArchCore` (`ARCHCORE_ROOT`).
3. `archcore db-init` à chaque démarrage : bases créées, migrations appliquées (ArchCore refuse de démarrer
   sur une base en retard).
4. Monde vide ou importé par une autre version d'ArchCore (`UserData/monde-importe-par.txt`) : `decrypt
   <client>/ini Server/Data/client-ini`, `import Server/Data/client-ini --server <tables>` (les 31 tables
   exigées par l'import EP8 sont contrôlées avant : `repair::SERVER_TABLES`), `i18n db fr`. Le résultat est
   contrôlé (`character_template`, `item_template` non vides). Puis, une seule fois par installation
   (`UserData/migration-ep8.txt`) et s'il y a des personnages : sauvegarde « avant-ep8 » et `migrate-ep8
   --apply`.
4 bis. `export-client-tables <client>/server-data --game <données du jeu>` si `tables.toml` manque ou est plus
   ancien que le dernier import : ArchRust lit ces tables avant ses packs, le serveur annonce leur empreinte.
5. Ligne `archcore_auth.world` : nom « HomeRPG », adresse `127.0.0.1:<port monde>` (c'est cette table, pas
   world.conf, que lit le serveur de connexion).
6. `archcore all` ; prêt quand les deux ports répondent. ArchBot démarre avec le monde (journal `[archbot]`)
   et ses bots sortent du monde à l'arrêt (Ctrl+C). Ensuite seul le processus est surveillé (un sondage
   de port ouvrirait une session notée dans le journal du serveur).

## Arrêt d'ArchCore

ArchCore s'arrête sur Ctrl+C (`tokio::signal::ctrl_c`), après avoir fermé ses bases ; il n'a pas de console
de commandes. Il est lancé avec `CREATE_NO_WINDOW` : il a sa propre console, sans fenêtre. Le launcher s'y
attache (`AttachConsole`), ignore lui-même l'événement (`SetConsoleCtrlHandler(NULL, TRUE)`) et émet
`CTRL_C_EVENT` ; terminaison forcée seulement après 30 s. Validé par `Tests/Test-Launcher.ps1` (« ArchCore
arrêté proprement »).

## Comptes

Écriture directe dans `archcore_auth.account`, comme `archcore db-init --account` : empreinte SHA-256 du mot de
passe en Windows-1252 ; les mots de passe acceptés sont en ASCII imprimable, identique dans les deux
encodages. Fonctionne serveur arrêté (base démarrée le temps de l'opération). Suppression : personnages
(cascade sur objets, quêtes, compétences, membres de guilde), banque du compte, compte.

## Langues (i18n.rs)

Chaque chaîne est une paire `T("français", "english")` ; la langue est globale (atomique), changée à chaud
depuis Paramètres, et lue au moment d'afficher ou de journaliser. `--lang fr|en` la force (tests). Langue
par défaut : celle de l'interface de Windows. La langue du jeu est distincte : `World.Language` du serveur et
`language` d'ArchRust (`saved-variables/options.toml`).

## Journaux (logs.rs)

Les sorties standard et d'erreur de chaque processus suivi sont lues ligne à ligne : écrites dans un fichier
horodaté et envoyées à l'interface (trois files de 4 000 lignes). Niveaux reconnus pour la couleur et le
filtre : `WARN`/`ERROR` d'ArchCore et d'env_logger (ArchRust), `ERREUR`/`ERROR` du launcher. Rotation : 20
fichiers par sorte. ArchRust est lancé avec `RUST_LOG` réglable ; ArchCore avec `ARCHCORE_TRACE=1` sur option.

## Données jamais redistribuées

Les données du jeu dans `Client/` (dont `Client/server-data`, écrit depuis la base monde) et
`Server/Data/client-ini` (tables du client déchiffrées), `UserData/` et `Logs/`. Le package contient l'exécutable ArchRust et ses AddOns sous GPL, ArchCore, MariaDB, HeidiSQL, le
launcher, la documentation et les tables du serveur d'origine (`Server/Data/server-ini`, jamais dans le dépôt) ;
aucune donnée du client n'est fournie.
