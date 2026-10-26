# 03 — Tests

Tous les tests se lancent launcher fermé, ports 3316, 11002 et 11008 libres (aucun autre serveur ArchCore en
marche).

| Script | Vérifie |
|---|---|
| `cargo test --release` (Launcher) | empreinte des mots de passe identique à ArchCore, règles des comptes, bascule des langues, niveaux des journaux, décodage console |
| `Tests\Test-Launcher.ps1` | démarrage complet (`--start`), import du monde au premier lancement, ArchCore prêt, fermeture par la fenêtre : ArchCore arrêté proprement (Ctrl+C), MariaDB arrêtée, journal serveur écrit, ports libérés, aucun processus orphelin, aucune erreur |
| `Tests\Test-Integration.ps1` | compte créé par le launcher (`--do acccreate`), `archcore sim client` (personnage créé, entrée en jeu, commande MJ), `ArchRust.exe login` sans affichage, sauvegarde et suppression du compte serveur arrêté |
| `Tests\Test-Offline.ps1` | écoute sur 127.0.0.1 uniquement, aucune connexion sortante |

## Résultats du 2026-10-26 (version 0.2.0, ArchCore 7403088, ArchRust 2e41983)

Essais du package comme un testeur (zip décompressé dans un dossier neuf, données du jeu dans un autre dossier
indiqué par le réglage « Données du jeu ») :

- Tests unitaires : 7/7 (1 ignoré).
- Premier lancement (`--start --do acccreate`) : base créée, `root@<nom du PC>` supprimé, 4 bases migrées (2 + 9
  + 7 + 1 migrations), monde importé avec les tables du serveur d'origine en 32 s (monster_ai 7 674,
  skill_mastery 264, guild_mark 166, tables d'options et de raffinage) ; ArchCore prêt à t+53 s.
- Entrée en jeu : `archcore sim client` (« flux complet OK »), `ArchRust.exe login --game` (Anchorville, 29
  entités, textes en français), puis la fenêtre d'ArchRust (`play --connect`) : en jeu, 3 AddOns chargés.
- Fermeture par la fenêtre : ArchCore arrêté proprement, MariaDB arrêtée (fichier pid), ports libérés.
- JOUER (`--do play`) au second lancement : pas de réimport, `realmlist.wtf` et `options.toml` écrits dans
  `Client\` du package ; journal MariaDB sans « Aborted connection » ni avertissement `root@…`.
- Vérifier avec un fichier modifié et un fichier retiré : les deux sont nommés dans le journal ; AddOns,
  `realmlist.wtf` et `saved-variables` hors contrôle.
- Sans tables du serveur : avertissement, monde sans monstres ; tables ajoutées : réimport automatique au
  démarrage suivant (12 802 apparitions).
- Mise à jour depuis la v0.1.0 (compte et personnage créés avec ArchCore d59181a, `UserData` copié) : 8 + 3
  migrations, `root@<nom du PC>` supprimé, réimport automatique, le personnage entre en jeu avec ArchRust.
- Correctif trouvé par ces essais : `my.ini` écrivait les chemins avec des `\` ; un dossier commençant par `s`,
  `t`, `n`, `r` ou `b` (`…\scratchpad\…`) empêchait MariaDB de démarrer. Chemins désormais en `/`.

Non testé : les nouveautés du jeu une à une (guildes, portails, raffinage…), laissées aux testeurs ; le package
sur un autre PC.

## Résultats du 2026-10-23 (version 0.1.0, ArchCore d59181a, MariaDB 13.0.2)

- Tests unitaires : 6/6.
- `Test-Launcher.ps1` : réussi. Premier démarrage : MariaDB prête en 7 s ; import du monde avec les tables du
  serveur d'origine en 30 s (622 modèles de personnages, 16 973 objets, 567 PNJ, 12 802 apparitions de
  monstres) ; ArchCore prêt à t+41 s (36 960 monstres chargés) ; arrêt propre.
- `Test-Integration.ps1` : réussi. `sim client` : « flux complet OK » ; ArchRust `login` : en jeu à
  Anchorville, 29 entités en vue, titres de PNJ en français ; sauvegarde (1 personnage, 1 compte) ; compte
  supprimé avec son personnage ; base arrêtée ensuite.
- JOUER (`--do play`) : ArchRust lancé, journal client écrit dans `Logs\client\` (rendu Vulkan, packs, AddOns,
  serveur 127.0.0.1:11002), jeu fermé avec les services.
- Interface vérifiée en capture, en français et en anglais.
- `Test-Offline.ps1` : réussi (écoute sur 127.0.0.1:3316, 11002, 11008 seulement, aucune connexion sortante).
- Restauration (`--do restore`) : sauvegarde de sécurité, puis compte et personnage restaurés.
- Composants officiels (`Get-Components.ps1 -Pin`, `Install-Components.ps1`) : MariaDB 13.0.2 (empreinte
  figée) et HeidiSQL 12.21 ; `Test-Launcher.ps1` repassé avec eux.
- Package `New-Package.ps1 -Zip` : 300 fichiers, 317 Mo (zip 101 Mo) ; Vérifier depuis le package : 300
  fichiers vérifiés, 0 manquant, 0 altéré.

Non testé par une personne en jeu : la partie complète dans la fenêtre d'ArchRust (connexion manuelle,
création de personnage à l'écran), la fermeture normale du jeu suivie de la sauvegarde automatique, la
restauration depuis l'interface, le package `New-Package.ps1` installé sur un autre PC.
