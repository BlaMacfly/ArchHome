# Journal des versions / Changelog

## 0.2.0 — 26 octobre 2026

Version pour les testeurs / Tester build. ArchCore 7403088 (branche Core), ArchRust 2e41983 (branche client).

- ArchCore 7403088 : guildes complètes (second, passation, agrandissement, liste, emblèmes, combats de guildes,
  alliances et hostilités, entrepôt de guilde), zones de combat des portails, reconnexion automatique, compétences
  et objets d'Archonia, raffinage, conversion des objets, potions automatiques, refus d'échange, de groupe et de
  guilde, niveaux des monstres et réapparition des boss d'Archonia. Nouvelles migrations des quatre bases.
- Package : inclut ArchRust.exe (commit indiqué dans `Build/manifest.json`) et ses AddOns dans `Client/addons`
  (ArchAce, HelloArch, ArchCore Admin 1.2.0), hors du contrôle des fichiers ; les données du jeu restent fournies
  par le joueur. `LISEZMOI-TESTEURS.txt` (installation, comptes de test, nouveautés à essayer, journaux).
- Mise à jour d'une installation : `archcore db-init` à chaque démarrage (migrations), puis réimport automatique
  du monde quand la version d'ArchCore diffère de celle qui l'a importé (`UserData/monde-importe-par.txt`) ou
  quand les tables du serveur d'origine apparaissent après un import sans monstres. Personnages et comptes
  conservés ; sans client, le réimport est reporté et le serveur démarre sur le monde existant.
- Import sans les tables du serveur d'origine : avertissement visible sur l'accueil et dans le journal (le monde
  n'aura ni monstres ni butin) ; avertissement aussi si l'import ne place aucun monstre malgré elles.
- Contrôle des fichiers : chaque fichier manquant ou altéré est nommé dans le journal du launcher.
- MariaDB ne démarrait pas si le dossier d'installation contenait un dossier commençant par `s`, `t`, `n`, `r` ou
  `b` (`my.ini` lisait `\s`, `\t`… comme des échappements) : chemins écrits avec des `/`.
- MariaDB : la santé est vérifiée par une connexion authentifiée (fermée par COM_QUIT) au lieu d'un sondage du
  port, qui laissait un « Aborted connection » par seconde dans `mariadb-error.log` ; l'arrêt est suivi par le
  fichier pid. Le compte `root@<nom du PC>` créé par mariadb-install-db est supprimé à l'initialisation (et sur
  les installations existantes).
- Recherche des clients : inspecte aussi le chemin configuré, dossier ou `ArchRust.exe`, même hors du launcher.
- Build : `New-Package.ps1 -ArchRustExe <exe> -Addons <dossiers>`.

## 0.1.0 — 23 octobre 2026

Correctif avant diffusion / Fix before release :

- Recherche des clients : elle ne sonde que le dossier du launcher et ses sous-dossiers (décision de
  l'utilisateur : rien au-dessus, ni Bureau, ni disques ; un client ailleurs s'indique avec « Parcourir… »).
  Elle tourne en arrière-plan (l'interface ne se fige plus), dit quand elle ne trouve rien, signale les dossiers
  sans données du jeu (dont le dossier des sources d'ArchRust) et enregistre aussitôt le client choisi.
- Nouveau réglage « Données du jeu » : données d'ArchLord dans un autre dossier qu'ArchRust.exe (lancement avec
  `--game`), avec recherche automatique dans le dossier du client (`assets-archonia`).

Première version : l'équivalent de HomeCraft Edition Vanilla pour ArchLord.
First version: the ArchLord counterpart of HomeCraft Edition Vanilla.

- Launcher Rust (egui) à un seul bouton : MariaDB portable (127.0.0.1:3316), schéma (`archcore db-init`),
  import du monde depuis le client au premier démarrage (`decrypt`, `import` avec les tables du serveur
  d'origine si elles sont fournies, `i18n db`), ArchCore (`archcore all`, 127.0.0.1:11002 / 11008), ArchRust.
- Arrêt propre d'ArchCore par Ctrl+C envoyé à sa console invisible.
- Interface bilingue français / anglais, langue de Windows par défaut, changeable à chaud ; langue du jeu
  (serveur et ArchRust) réglable.
- Onglets Accueil, Comptes (création, mot de passe, niveau MJ, suppression, serveur arrêté ou non), Monde
  (réglages de world.conf), Packs et AddOns (packs de ressources et AddOns Lua d'ArchRust), Base de données
  (explorateur, HeidiSQL), Sauvegardes (vérifiées, automatiques, restauration, diagnostic), Paramètres,
  Journaux, Aide MJ.
- Journaux côté serveur (`Logs/server/archcore-*.log`, `outils-*.log`) et côté client
  (`Logs/client/archrust-*.log`, niveau réglable), onglet Journaux à trois vues avec filtres.
- Scripts : compilation d'ArchCore au commit figé, composants tiers par manifeste et empreinte, package
  portable, tests d'intégration (`Test-Launcher`, `Test-Integration`, `Test-Offline`).
