# 03 — Tests

Tous les tests se lancent launcher fermé, ports 3316, 11002 et 11008 libres (aucun autre serveur ArchCore en
marche).

| Script | Vérifie |
|---|---|
| `cargo test --release` (Launcher) | empreinte des mots de passe identique à ArchCore, règles des comptes, bascule des langues, niveaux des journaux, décodage console |
| `Tests\Test-Launcher.ps1` | démarrage complet (`--start`), import du monde au premier lancement, ArchCore prêt, fermeture par la fenêtre : ArchCore arrêté proprement (Ctrl+C), MariaDB arrêtée, journal serveur écrit, ports libérés, aucun processus orphelin, aucune erreur |
| `Tests\Test-Integration.ps1` | compte créé par le launcher (`--do acccreate`), `archcore sim client` (personnage créé, entrée en jeu, commande MJ), `ArchRust.exe login` sans affichage, sauvegarde et suppression du compte serveur arrêté |
| `Tests\Test-Offline.ps1` | écoute sur 127.0.0.1 uniquement, aucune connexion sortante |
| `archcore sim bots <compte MJ> <mdp> <perso>` (serveur en marche, `ARCHCORE_ROOT` et `ARCHCORE_CONF_DIR` de l'installation) | ArchBot de bout en bout : 16 vérifications (bots visibles et en mouvement, `/bot hire`, groupe, suivi, ordres, combat, métriques, départ) |

Une installation d'essai à part (copie du package, `UserData\settings.json` avec d'autres ports de connexion et de
monde, par exemple 11102 / 11108) laisse libres les ports 11002 / 11008 d'un serveur de développement ; MariaDB
garde le port 3316 de ArchHome (celle de développement, 3306, n'est jamais touchée).

## Résultats du 2026-11-22 (version 0.3.8, ArchCore a8610ef + ArchBot 109071c, ArchRust 87b4a45)

- Le patch d'ArchBot ne s'appliquait plus (`server.rs` : arrêt propre d'ArchCore 1d3c785) : adapté et publié dans
  ArchBot 109071c (mêmes 12 fichiers), `archbot::shutdown()` avant `world::shutdown::run`. ArchCore patché :
  384 tests ; ArchBot : 48 tests ; launcher : `cargo test --release` OK.
- Installation d'essai mise à jour : réimport avec Webzen et les traductions, `db-check` sans anomalie.
- Arrêt par le launcher, SimRetour en jeu et 15 bots : « ArchCore arrêté proprement » (`stop()` vrai) en 1 à 2 s ;
  journal du serveur : 15 bots sortis, annonce « Arrêt du serveur. » (reçue par le client, capture), joueur sorti avec
  sa position, « arrêt terminé » en dernière ligne ; ports libérés. Un premier essai montrait un bot rentré pendant la
  seconde d'annonce : corrigé dans ArchBot 109071c (population visée à 0 à l'arrêt), plus aucun au second essai.
- Constaté : après l'annonce, le client affiche aussi « The trade has been cancelled » sans échange en cours
  (message d'ArchCore à la sortie du joueur).
- Les ports 11102 / 11108 étant pris par un serveur de test d'une autre session, l'installation d'essai et le premier
  lancement du paquet (`New-TesterPackage.ps1 -InitPorts 11202,11208`) ont utilisé 11202 / 11208.

## Résultats du 2026-11-19 (version 0.3.7, ArchCore 5cbde83 + ArchBot d06afc8, ArchRust 256888e)

- Serveur inchangé (ArchCore ad280a6 ne touche que la documentation) : l'installation d'essai garde sa base, sans
  réimport ; `cargo test --release` du launcher : OK.
- Liens d'objet, en jeu à deux clients ArchRust (SimRetour, MJ, amène SimQuete à côté par `/move`) : un message de
  SimRetour contenant `<ITEM::298::Small Life Potion>` (envoyé par `--exec`) s'affiche chez SimQuete
  « [SimRetour] Regarde <Small Life Potion> ! », le lien sur fond bleu (capture). La saisie s'ouvre par Entrée.
- Non vérifié ici : le Ctrl + clic dans le sac et le clic sur le lien (infobulle). Des clics envoyés par messages à
  une fenêtre en arrière-plan n'atteignent pas l'interface (Windows signale aussitôt que la souris a quitté la
  fenêtre), et la vraie souris n'a pas été prise. L'échange réseau (op 31 genre 8 → op 32) passe avec l'exemple
  `item_link` d'ArchRust.

## Résultats du 2026-11-19 (version 0.3.6, ArchCore 5cbde83 + ArchBot d06afc8, ArchRust fd73c30, puis 1f23b66 et 19e1c30)

- Patch d'ArchBot appliqué tel quel ; aucune migration en plus ; `cashmall.txt` lu s'il est là (dans le package).
- Mise à jour de l'installation d'essai 0.3.5 : réimport avec Webzen et les traductions (`cashmall.txt` rangé dans
  `client_table`), `db-check` sans anomalie, empreinte `sha256:7eba6c30…` inchangée ; journal du serveur :
  « boutique Cash : 7 onglet(s), 486 produit(s) ».
- `migrate-ep8` : rien à migrer ; `sim client` : OK ; `sim bots` : 16/16 ; `sim cashmall` : « boutique Cash et
  recrutement : OK » ; `sim admin` (avec un second compte SimA) : « outil MJ, console de développement et
  KillGameServer : OK » ; `sim scroll` : OK ; client en jeu, empreinte conforme.
- Constaté avec ArchRust fd73c30 : `/boutique` était pris d'abord par la commande des PNJ et ouvrait la boutique d'un
  marchand. Corrigé dans ArchRust 1f23b66 (`/boutiquecash`, `/cashmall`) : la fenêtre « CHANTRA SHOP » s'ouvre en
  jeu (7 onglets, 12 produits par page, capture) ; la grille des produits débordait du cadre à droite et en bas, et le
  solde et la page chevauchaient les prix de la dernière rangée. Corrigé dans ArchRust 19e1c30 (décalage compté deux
  fois) : en jeu, `/boutiquecash` ouvre la fenêtre avec les 12 produits (4 × 3) dans le cadre, les pages et le solde
  sous la grille, sans chevauchement (capture). Le menu Chantra n'a pas été essayé à la souris.

## Résultats du 2026-11-18 (version 0.3.5, ArchCore 7fa20a0 + ArchBot d06afc8, ArchRust d2e6ee1)

- Patch d'ArchBot appliqué tel quel ; aucune migration ni table du serveur exigée en plus. Deux avertissements de
  compilation dans ArchCore lui-même (`item_info.rs` : constantes `CHAT_LINK` et `TEMPLATE` inutilisées).
- Mise à jour de l'installation d'essai 0.3.4 : réimport avec Webzen et les traductions (3 235 quêtes, 2 768
  titres), `db-check` sans anomalie, empreinte de `server-data` `sha256:7eba6c30…` (inchangée).
- `migrate-ep8` : rien à migrer ; `sim client` : OK ; `sim bots` : 16/16 ; `sim questobject`, `sim questgather` :
  OK ; `sim scroll` avec un personnage neuf : « parchemins : OK » (op 31 → op 32, aller-retour, abandon op 8 ;
  avec l'ancien personnage d'essai, arrêt à l'aller-retour) ; `ArchRust.exe play --connect` (d2e6ee1) : en jeu à
  Anchorville, empreinte conforme, 3 AddOns, pas de cube à damier.

## Résultats du 2026-11-10 (version 0.3.4, ArchCore 23c2ced + ArchBot d06afc8, ArchRust 2f82448)

- Patch d'ArchBot d06afc8 appliqué tel quel ; mêmes tables du serveur exigées.
- Mise à jour de l'installation d'essai 0.3.3 avec « Données de Webzen » et « Traductions d'ArchCore » réglés :
  migration `source_des_traductions`, réimport avec `--webzen` et `--traductions` (7 langues chargées ; textes de
  quêtes ajoutés en de, en, es, fr, pt-BR), mêmes chiffres de quêtes (3 235, 780 objets, 1 044 groupes) ; `db-check`
  sans anomalie ; empreinte de `server-data` `sha256:7eba6c30…`, la même que le client de développement.
- `migrate-ep8` : rien à migrer ; `sim client` : OK ; `sim bots` : 16/16 ; `sim questobject` et `sim questgather` :
  OK ; `ArchRust.exe play --connect` (2f82448) : en jeu, empreinte conforme, 3 AddOns. Non vérifiés à part : la
  sortie des bots en trop (`/bot population`), les marques de quête et l'animation du ramassage à l'écran.
- Nom ArchHome et nouveau fond : fenêtre « ArchHome » à 1016×759 et en plein écran (1936×1048) : image en
  recouvrement centré, sans déformation, texte et boutons lisibles (captures). Dossier Testing renommé
  « ArchHome v0.3.4 » (chemin avec espace) : démarrage sans réimport, monde inscrit sous le nom « ArchHome ».

## Résultats du 2026-11-09 (version 0.3.3, ArchCore 6b1bf96 + ArchBot e214b2c, ArchRust 5c96570)

- Patch d'ArchBot appliqué tel quel ; mêmes tables du serveur exigées.
- Mise à jour de l'installation d'essai 0.3.2 (Webzen réglé) : migrations `quetes_des_objets` et
  `ramassage_des_objets`, réimport avec `--webzen` : `quest_template` 3 235, `quest_object` 780, `quest_group` 1 044 ;
  `db-check` sans anomalie ; empreinte de `server-data` `sha256:e9e04d30…` (la même que le client de développement).
- `migrate-ep8` (rapport) : rien à migrer. `sim client` : OK ; `sim bots` : 16/16 PASS ; avec un compte MJ et un
  personnage neufs : `sim questobject` « quêtes des objets : OK », `sim questgather` « ramassage des quêtes : OK »
  (avec l'ancien personnage d'essai, inventaire plein : arrêt sur délai d'attente) ; `ArchRust.exe play --connect`
  (5c96570) : en jeu, empreinte conforme, 3 AddOns.

## Résultats du 2026-11-08, soir (version 0.3.2, ArchCore ea667d5 + ArchBot e214b2c, ArchRust 072ebee)

- Patch d'ArchBot appliqué tel quel sur ea667d5 ; mêmes tables du serveur exigées, aucune nouvelle migration.
- Mise à jour de l'installation d'essai 0.3.1 avec « Données de Webzen » réglé : réimport avec `--webzen` en 36 s :
  `level_exp` 121 niveaux, `skill_mastery` 287, `quest_template` 3 082 (188 écartées, détail dans
  `Server\ArchCore\data\import-webzen.log`), `title_template` 2 768 ; `db-check` sans anomalie ; empreinte de
  `server-data` `sha256:cdd4b453…`, la même que celle du client de développement.
- `migrate-ep8` (rapport) : 20 personnages, rien à migrer (quêtes et titres compris) : pas de `--apply`.
- `sim client` : OK ; `sim bots` : 16/16 PASS ; `ArchRust.exe play --connect` (072ebee) : en jeu, empreinte conforme,
  3 AddOns. `sim title` et `sim quest` non concluants sur cette installation (second compte de test absent,
  inventaire du personnage d'essai plein) : non comptés.

## Résultats du 2026-11-08 (version 0.3.1, ArchCore 6912f16 + ArchBot, ArchRust 7a6c7f3, ArchCore Admin 1.4.0)

- Compilation : patch d'ArchBot adapté à 6912f16 (contexte de `sim.rs` ; `start_cast` reçoit la position au sol de
  `request_cast`). Tests : ArchCore 343/343 avec le patch, ArchBot 45/45, launcher 7/7 (1 ignoré).
- Tables du serveur d'origine d'ArchCore (`data/server-ini` de 6912f16) identiques octet pour octet à celles de la
  0.3.0 (41 tables, `objects`, `node_data`) ; 31 exigées par l'import (Gacha, loteries, costumes en plus).
- Mise à jour de l'installation d'essai 0.3.0 (comptes, personnages, 19 bots) : migration `202611081200_gacha`,
  réimport (60 socles de Gacha, 5 tables de tirage), `db-check` : schéma à jour, aucune anomalie ; export des tables
  (empreinte `sha256:a00439fc…`, la même que celle du client de développement) ; serveur prêt en 69 s.
- `sim client` : flux complet OK ; `sim bots` : 16/16 PASS ; `sim gacha` : OK (tirages au pass et au charisme,
  inventaire plein refusé sans rien prendre).
- `ArchRust.exe play --connect` (7a6c7f3) : en jeu, empreinte conforme, 3 AddOns ; ArchCore Admin 1.4.0 détecte le
  niveau MJ (« Niveau MJ : 3 ou plus ») : le « Pas de réponse du serveur » de la 0.3.0 a disparu.

## Résultats du 2026-11-03 (version 0.3.0, ArchCore f2ffef3 + ArchBot 7436b3c adapté, ArchRust 70cd441, puis b25a2a2)

Installation d'essai à part (`Build\tmp\essai-0.3.0`, ports 11102 / 11108, données du jeu lues dans un autre
dossier par le réglage « Données du jeu ») :

- Compilation : ArchCore f2ffef3 + patch d'ArchBot (12 fichiers, adapté : seul le contexte de `main.rs` changeait)
  et adaptateur d'ArchBot corrigé pour f2ffef3 (`party::on_management` avec `WorldCtx`, `GroundItem::owner` en
  `Option`). Tests : ArchCore 343/343 avec le patch, ArchBot 45/45, launcher 7/7 (1 ignoré).
- Premier lancement (`--start --do acccreate`) : datadir créé, `root@<PC>` supprimé, import EP8 en 33 s (16 973
  objets dont 3 839 d'Archonia écartés, 622 modèles, 557 PNJ, 12 802 apparitions, 36 960 monstres),
  `export-client-tables` vers `Client\server-data` (ini 20 tables, 7 dossiers de langue), `migrate-ep8` inutile
  (aucun personnage), serveur prêt à t+64 s. Journal serveur : empreinte des tables annoncée, `[archbot] v0.1.0 :
  bots 40 au plus, population initiale 15, niveaux 1..10…`, 15 bots créés et en jeu ; aucun WARN ni ERROR.
- `archcore sim client gmtest … "/bot list"` : flux complet OK, 15 bots listés. `archcore sim bots` : 16/16 PASS.
- `ArchRust.exe login --port 11102` : extensions `client-tables/1` et `language/1`, en jeu à Anchorville (40
  entités). Fenêtre `play --connect` : en jeu, 3 AddOns chargés, « empreinte … conforme au serveur », capture.
- Fermeture par la fenêtre : les 16 bots sortent du monde, ArchCore arrêté proprement, MariaDB arrêtée, ports
  libérés, aucun orphelin.
- Second lancement avec `--do play` : ni réimport ni réexport, `realmlist.wtf` (127.0.0.1:11102) et `options.toml`
  écrits dans `Client\`, ArchRust lit `Client\server-data` ; jeu fermé : « jeu terminé (code 0) », sauvegarde
  automatique (20 personnages dont les bots).
- `ArchBot.Enable = 0` (réglage de l'onglet ArchBot) : « [archbot] module désactivé ».
- Mise à jour depuis le paquet testeurs 0.2.0 (copie de son `UserData`, monde importé par ArchCore 208487b) : 32
  migrations (1 + 18 + 11 + 2), réimport automatique, sauvegarde « avant-ep8 », `migrate-ep8` puis `--apply` (1 personnage actif,
  rien hors épisode 8), export des tables, serveur prêt.
- ArchRust b25a2a2 (écran de connexion d'Archonia retiré) : JOUER par le launcher ouvre le panneau de connexion
  d'ArchRust, serveur pré-rempli depuis `realmlist.wtf` (127.0.0.1:11102, capture) ; connexion avec « Enregistrer
  le compte » coché : compte authentifié, `saved-variables\compte.dat` écrit ; client relancé dans le même dossier :
  « compte enregistré : gmtest » ; `play --connect` : en jeu, empreinte conforme, 3 AddOns. Aucun changement du
  launcher nécessaire (il ne touche ni à `compte.dat` ni au reste d'`options.toml`).
- Onglet ArchBot vérifié à l'écran (capture) ; `ArchBot.Classes = "96,6"`, population 5 et rayon d'exploration
  5000 réglés par l'onglet arrivent dans `archbot.conf` et dans la ligne `[archbot] v0.1.0 …`, sans avertissement.
- Non vérifié : l'onglet Comptes (comptes de bots masqués) à l'écran ; la
  connexion à la main sur le panneau d'ArchRust ; une partie réelle avec les bots.
- Constaté : ArchCore Admin 1.3.0 affiche « Pas de réponse du serveur » à l'entrée en jeu alors que le serveur
  répond à sa sonde `/SetExp` (journal : `[chat] GmTest (type 0) : /SetExp`) — à voir dans ArchCore Admin ou
  ArchRust.

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

Le soir, package avec les tables du serveur d'origine (`-ServerTables`) :

- Installation neuve sans aucun réglage de tables : import avec `Server\Data\server-ini`, aucun avertissement,
  map_segment 45 312, 36 960 monstres chargés.
- Anglais (`ui_language` et `game_language` = en) : launcher entièrement en anglais (Accueil, Paramètres,
  journal), `options.toml` → `language = "en"`, `World.Language = "en"`, en jeu : noms et titres des PNJ,
  onglets de discussion en anglais. Restent en français les écrans propres à ArchRust (connexion, « Joueur »,
  message de bienvenue) et des messages d'AddOns : textes en dur dans ArchRust, hors de ce dépôt.
- Installation importée sans tables puis mise à jour avec elles : réimport automatique, 36 960 monstres.

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
