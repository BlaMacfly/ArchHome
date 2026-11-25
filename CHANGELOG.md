# Journal des versions / Changelog

## 0.3.10 — 25 novembre 2026

Version pour les testeurs / Tester build. ArchCore 70fb81c (branche Core) avec ArchBot 7a11dbb, ArchRust 87b4a45,
ArchCore Admin 1.4.0. Remplace la 0.3.9, à ne pas diffuser / Replaces 0.3.9, not to be distributed.

- Corrige la 0.3.9 (ArchCore 7b94e5b) : les formules d'AR, de DR et de pénalité de défense y étaient fausses (AGI
  comptée deux fois à l'affichage). AR = (niveau + AGI) × 0,6 + 12, DR = (niveau + AGI) × 0,4 + 8, plus les bonus ;
  pénalité nulle à 1 niveau d'écart et à 20 exactement, pas sur les résistances héroïques (ArchCore 70fb81c).
- Fixes 0.3.9: its AR, DR and defense penalty formulas were wrong (AGI displayed twice).
- Non vérifié / Not verified : rien n'a été essayé dans le vrai client ; PV = CON × 8, PM = SAG × 5,38, la pénalité
  sur les critiques, l'écart de niveau des compétences et le refus des effets plus faibles ne sont pas confirmés par
  Archonia ; les échecs de `sim aggro` et `sim pk` dans la suite complète d'ArchCore sont inexpliqués.
- Aucune migration ni réglage en plus ; ArchBot 7a11dbb s'applique tel quel, ArchRust inchangé.

## 0.3.9 — 24 novembre 2026

Version pour les testeurs / Tester build. ArchCore 7b94e5b (branche Core) avec ArchBot 7a11dbb, ArchRust 87b4a45,
ArchCore Admin 1.4.0.

- Règles de l'épisode 8 (ArchCore 7b94e5b) : PV, PM, AR et DR tirés de CON, SAG et AGI ; précision et esquive en
  chances directes, esquive à distance ou au corps à corps ; pénalité de défense selon l'écart de niveau ; effets
  plus faibles refusés, une famille à la fois ; potions en pour cent, options fixes, niveau plafond des objets ;
  PV, PM, mort et rattachement enregistrés (migration `202611231000_points_et_mort`, appliquée au démarrage).
- ArchCore 3dd6fe1 : réglage World.CreatureSpeed (vitesse des animaux, 1 par défaut) ; plus de fausse annulation
  d'échange à la sortie du jeu.
- ArchBot 7a11dbb : patch adapté à ArchCore 7b94e5b (mêmes 12 fichiers).
- ArchRust inchangé (87b4a45 ; 184108b n'ajoute qu'un outil de mesure).

## 0.3.8 — 22 novembre 2026

Version pour les testeurs / Tester build. ArchCore a8610ef (branche Core) avec ArchBot 109071c, ArchRust 87b4a45,
ArchCore Admin 1.4.0.

- Arrêt propre du serveur (ArchCore 1d3c785) : au Ctrl+C envoyé par le launcher, plus aucune entrée en jeu, annonce
  « Arrêt du serveur. », chaque joueur quitte le monde avec sa position enregistrée, taxes et journaux enregistrés,
  bases fermées, code de sortie 0 ; sans joueur, arrêt immédiat.
- Liens d'objet (ArchCore a8610ef, ArchRust 87b4a45) : après l'op 32 d'un objet renforcé ou serti, le serveur envoie
  son AGPMITEMCONVERT op 0 ; l'infobulle du lien montre le renforcement, les sockets et les pierres.
- Ramassage (ArchRust 87b4a45) : ligne SYSTEM dans la discussion comme dans Archonia, « Gained nom N Item(s) » pour un
  objet qui s'empile, sinon « Gained <nom + N [a/b]> Item » en couleur avec lien d'objet ; l'or et les refus gardent
  les messages de l'[EventMap].
- ArchBot 109071c : patch adapté à l'arrêt propre d'ArchCore ; à l'arrêt, les bots sortent avant les joueurs et la
  population visée passe à 0 (aucun bot ne rentre pendant l'annonce).
- Aucune migration ; ArchCore change, donc un UserData d'une version précédente fait réimporter le monde (sans
  Webzen chez les testeurs) : garder le UserData fourni.

## 0.3.7 — 19 novembre 2026

Version pour les testeurs / Tester build. ArchCore 5cbde83 (branche Core) avec ArchBot d06afc8, ArchRust 256888e,
ArchCore Admin 1.4.0.

- Liens d'objet dans la discussion (ArchRust 256888e) : Ctrl + clic sur un objet du sac, saisie de discussion
  ouverte, ajoute la balise `<ITEM::IID::nom>` ; dans les lignes reçues, le lien s'affiche `<nom>` sur fond bleu et
  un clic montre l'infobulle de l'objet (AgpmItem op 31 genre 8, réponse op 32).
- ArchCore inchangé : ad280a6 ne change que la documentation, le serveur de 5cbde83 et la base du paquet 0.3.6
  restent valables ; un UserData de la 0.3.6 se reprend tel quel.

## 0.3.6 — 19 novembre 2026

Version pour les testeurs / Tester build. ArchCore 5cbde83 (branche Core) avec ArchBot d06afc8, ArchRust 19e1c30,
ArchCore Admin 1.4.0.

- Boutique Cash (ArchCore 5cbde83, ArchRust 19e1c30) : produits de `cashmall.txt` des tables du serveur d'origine,
  achat débité sur le Cash du personnage, sans aucun paiement réel ; fenêtres d'origine `UI_CashMall` et
  `UI_CashMallBuy`, ouvertes par le menu Chantra (`MenuBtnRoniMall`), `/boutiquecash` ou `/cashmall` ; `/boutique`
  reste la boutique d'un PNJ. Le Cash part de 0 : un MJ en
  donne avec `/cash <montant> "nom"`.
- Outil MJ (AgpmAdmin) et commandes `/search` et `/disconnect` ; derniers types de paquets de l'épisode 8 acceptés
  par le serveur (recrutement, Casper, scripts de la console de développement).
- Base du paquet réimportée avec Webzen et les traductions d'ArchCore (sans réimport, la boutique serait vide),
  `db-check` sans anomalie.

## 0.3.5 — 18 novembre 2026

Version pour les testeurs / Tester build. ArchCore 7fa20a0 (branche Core) avec ArchBot d06afc8, ArchRust d2e6ee1,
ArchCore Admin 1.4.0.

- ArchCore 7fa20a0 : renseignements sur un objet (infobulle d'un objet que le joueur ne détient pas : lien de la
  discussion, bouton d'objet d'une fenêtre) ; un aller-retour
  peut être abandonné, et le retour refusé en combat est expliqué.
- ArchRust d2e6ee1 : les packs d'exemple (glTF, texture, son) sont désactivés par défaut, ce qui supprime le cube
  à damier ; les fenêtres de dialogue et de banque d'un PNJ se ferment à 150 unités, la fenêtre des quêtes ne se
  ferme plus à distance ; les fenêtres d'Archonia sont pilotables par les AddOns.
- Base du paquet réimportée avec Webzen et les traductions d'ArchCore, `db-check` sans anomalie.

## 0.3.4 — 10 novembre 2026

Version pour les testeurs / Tester build. ArchCore 23c2ced (branche Core) avec ArchBot d06afc8, ArchRust 2f82448,
ArchCore Admin 1.4.0.

- Quêtes et titres traduits dans toutes les langues : la base du paquet est importée avec les données de Webzen et
  les traductions d'ArchCore (`archcore import --webzen … --traductions …`), qui complètent les textes de Webzen
  sans jamais les remplacer. Nouvelle migration de la base monde (`source_des_traductions`).
- ArchRust 2f82448 : marques de quête au-dessus des PNJ et des objets du décor, animation du ramassage.
- ArchCore 23c2ced : protocole vérifié contre Webzen, refus des options du joueur (échange, groupe, guilde…).
- ArchBot d06afc8 : les bots en trop sortent quand on baisse leur nombre ; expérience sur 64 bits.
- Launcher : réglage « Traductions d'ArchCore » (Paramètres, facultatif, avec les données de Webzen) passé à
  l'import (`--traductions`) ; son ajout déclenche un réimport. `New-TesterPackage.ps1 -TranslationsDir`.
- Le produit s'appelle **ArchHome** (HomeRPG est le projet plus large qui regroupe plusieurs MMO) : titre de la
  fenêtre, textes, `ArchHome.exe`, package `ArchHome-v<version>`, monde « ArchHome » dans la liste des serveurs,
  documentation, licence. Un dossier d'installation reste reconnu sous son ancien nom ; aucune clé de registre ni
  aucun chemin de données ne dépend du nom.
- Nouveau fond du launcher : image d'ArchHome (taverne) en recouvrement centré, voile sombre en haut et en bas
  pour la lisibilité.

## 0.3.3 — 9 novembre 2026

Version pour les testeurs / Tester build. ArchCore 6b1bf96 (branche Core) avec ArchBot e214b2c, ArchRust 5c96570,
ArchCore Admin 1.4.0.

- Quêtes des objets du monde : 153 quêtes de plus, données ou rendues par des objets du décor (3 235 quêtes,
  780 objets de quête, 1 044 groupes) ; base du paquet importée avec les données de Webzen, comme en 0.3.2.
- Ramassage : les objets de quête se ramassent sur le décor (barre de 5 s), pour 588 quêtes.
- Deux nouvelles migrations de la base monde (`quetes_des_objets`, `ramassage_des_objets`), appliquées seules au
  démarrage (`db-init`).

## 0.3.2 — 8 novembre 2026

Version pour les testeurs / Tester build. ArchCore ea667d5 (branche Core) avec ArchBot e214b2c, ArchRust 072ebee,
ArchCore Admin 1.4.0.

- Quêtes, titres, maîtrises et courbe d'expérience d'origine : le monde du paquet des testeurs est importé avec les
  données du client officiel de Webzen (`archcore import --webzen`) : 3 082 quêtes, 2 768 titres, 287 maîtrises,
  expérience des niveaux 1 à 121. **La montée en niveau est plus lente qu'en 0.3.1.** Seule la base importée est
  fournie, pas les données de Webzen.
- ArchCore d053af0 : une guilde ne se dissout plus trop tôt (code 46, `Guild.DestroyDelayHours = 24`).
- ArchRust 072ebee : titres et maîtrises d'origine, confirmations, minicarte et barre rapide d'Archonia.
- Launcher : réglage « Données de Webzen » (Paramètres, facultatif) passé à l'import (`--webzen`) ; un dossier
  indiqué après un import sans lui déclenche un réimport ; un réimport sans lui, sur un monde importé avec, est
  signalé (retour aux quêtes, titres, maîtrises et à l'expérience du client).
- `New-TesterPackage.ps1 -WebzenDir` : base du paquet importée avec Webzen, sans le dossier ni son chemin.

## 0.3.1 — 8 novembre 2026

Version pour les testeurs / Tester build. ArchCore 6912f16 (branche Core) avec ArchBot (patch adapté à 6912f16),
ArchRust 7a6c7f3, ArchCore Admin 1.4.0.

- ArchCore 6912f16 : Gacha de l'épisode 8 (socles, tables de tirage, coût en pass, or et charisme, refus), retour de
  bug n° 2 (reprise d'objet à l'échange, loterie, apparence, inventaire, compétences), règle d'écart de niveau du
  butin (`Rules.LootLevelGap = 11`), incantation sur place. Nouvelle migration de la base monde (`gacha_object`).
- ArchRust 7a6c7f3 : Gacha et retour de bug n° 2 côté client. ArchCore Admin 1.4.0 : refus expliqués, liste de
  monstres, fiche et filtre de classe.
- ArchBot : patch régénéré pour 6912f16 (`start_cast` reçoit la position au sol des compétences de zone).
- Import : les tables du serveur d'origine exigées passent de 23 à 31 (loteries, costumes, Gacha) ; le package les
  prend désormais dans ArchCore (`data/server-ini` du commit du manifeste), identiques à celles de la 0.3.0.
- Après chaque import, `archcore db-check` contrôle les bases : un problème est signalé (journal et accueil) sans
  bloquer le démarrage.
- Mise à jour d'une installation 0.3.0 : migration Gacha (`db-init`), réimport du monde, `db-check`, tables
  d'ArchRust réécrites ; comptes, personnages et bots conservés.

## 0.3.0 — 3 novembre 2026

Version pour les essais en jeu / In-game test build. ArchCore f2ffef3 (branche Core) avec ArchBot 7436b3c (+ adaptation
à f2ffef3), ArchRust b25a2a2 (dépôt ArchRust), ArchCore Admin 1.3.0.

- ArchCore f2ffef3 : valeurs d'origine de l'épisode 8. Les valeurs de jeu (compétences, objets, options, runes,
  monstres, raffinage, régions, boutiques, portails, expérience) viennent des tables du serveur ArchLord d'origine ;
  les ajouts d'Archonia sont écartés. Nouvelles migrations des bases, titres, consommables, échange privé,
  métamorphose, règles de combat et des régions, anti-triche côté serveur.
- **ArchBot intégré** au serveur : bots autonomes qui peuplent le monde et compagnons à engager (`/bot hire`,
  ordres par chuchotement ou `/p`). `Build-ArchCore.ps1` compile ArchCore sur une copie, avec ArchBot dans
  `modules/ArchBot` et son patch ; le dépôt source d'ArchCore n'est jamais modifié. Nouvel **onglet ArchBot** :
  les 35 réglages d'`archbot.conf` (activation, population, comportement, survie, performances, administration ;
  population de départ 15 et 40 bots au plus pour le solo, le reste aux valeurs d'ArchBot) et les commandes du
  jeu, écrits dans `Server/Config/archbot.conf`. Comptes `archbot_…` masqués dans l'onglet Comptes.
- Import du monde : les tables du serveur d'origine sont **exigées** (ArchCore refuse d'importer sans elles) ; le
  package en contient désormais le dossier complet (41 tables, `objects/`, `world/node_data`). Sans elles, le
  démarrage s'arrête avec une explication au lieu d'importer un monde vide.
- Tables pour ArchRust : après chaque import, `archcore export-client-tables` écrit les tables de l'épisode 8 au
  format du client dans `Client/server-data` (à côté de `realmlist.wtf`) ; `Client.TablesDir` les annonce au
  client, qui vérifie l'empreinte (journal du client : « conforme au serveur »). Réécrites si elles manquent ou
  datent d'avant le dernier import.
- Mise à jour d'une installation 0.2.0 : migrations (`db-init`), réimport du monde, puis, s'il y a des personnages,
  sauvegarde « avant-ep8 » et `archcore migrate-ep8 --apply` une seule fois (objets, options et compétences hors
  épisode 8 mis en quarantaine réversible, oubli des compétences offert ; `UserData/migration-ep8.txt`).
- ArchRust b25a2a2 (nouveau dépôt BlaMacfly/ArchRust) : valeurs EP8 du serveur lues dans `server-data/`, panneau
  de connexion d'ArchRust (case « Enregistrer le compte », premier monde annoncé pris d'office ; l'écran de
  connexion d'Archonia est retiré), `realmlist.wtf` toujours écrit par le launcher. ArchCore Admin 1.3.0 (choix visuel des objets, simulateur de
  butin).
- Build : `Build-ArchCore.ps1 -Source <ArchCore> -ArchBot <ArchBot>` (toolchain 1.98.1, trois essais si rustc
  plante) ; `New-Package.ps1 -ServerTables` copie tout le dossier des tables.

## 0.2.0 — 26 octobre 2026

Version pour les testeurs / Tester build. ArchCore 7403088 (branche Core), ArchRust 2e41983 (branche client).

- ArchCore 7403088 : guildes complètes (second, passation, agrandissement, liste, emblèmes, combats de guildes,
  alliances et hostilités, entrepôt de guilde), zones de combat des portails, reconnexion automatique, compétences
  et objets d'Archonia, raffinage, conversion des objets, potions automatiques, refus d'échange, de groupe et de
  guilde, niveaux des monstres et réapparition des boss d'Archonia. Nouvelles migrations des quatre bases.
- Package : inclut ArchRust.exe (commit indiqué dans `Build/manifest.json`) et ses AddOns dans `Client/addons`
  (ArchAce, HelloArch, ArchCore Admin 1.2.0), hors du contrôle des fichiers ; les données du jeu restent fournies
  par le joueur. `LISEZMOI-TESTEURS.txt` et `README-TESTERS.txt` (installation, comptes de test, nouveautés à
  essayer, journaux).
- Package : inclut les tables du serveur d'origine lues par `archcore import --server` dans
  `Server/Data/server-ini` (`New-Package.ps1 -ServerTables`, jamais dans le dépôt) : monstres et butin dès le
  premier lancement, sans rien à fournir. L'avertissement ne s'affiche plus que si elles manquent.
- Anglais : changer la langue du launcher règle aussi celle du jeu (serveur et `options.toml` d'ArchRust) ;
  anglais par défaut si Windows n'est pas en français.
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
