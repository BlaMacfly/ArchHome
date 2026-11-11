# ArchHome — Manuel d'utilisation

*English version: [User-Manual.md](User-Manual.md).*

ArchHome fait tourner ArchLord en solo sur votre PC : un serveur ArchCore local, une base MariaDB
portable et un launcher qui s'occupe de tout. Vous jouez avec le client ArchRust.

## Sommaire

1. [Prérequis](#1-prérequis)
2. [Installation](#2-installation)
3. [Premier démarrage](#3-premier-démarrage)
4. [Accueil](#4-accueil)
5. [Comptes](#5-comptes)
6. [Monde](#6-monde)
7. [Packs et AddOns](#7-packs-et-addons)
8. [Base de données](#8-base-de-données)
9. [Sauvegardes](#9-sauvegardes)
10. [Paramètres et langues](#10-paramètres-et-langues)
11. [Journaux](#11-journaux)
12. [Aide MJ](#12-aide-mj)
13. [Dossiers](#13-dossiers)
14. [Dépannage](#14-dépannage)

## 1. Prérequis

- Windows 10 ou 11, 64 bits, 8 Go de mémoire.
- L'exécutable **ArchRust** (`Client\ArchRust.exe`, fourni dans le package) et les données du client ArchLord
  64 bits (`character\`, `ini\`, `world\`, `texture\`, `object\`, `effect\`, `sound\`…), que vous fournissez.
  ArchHome ne distribue aucune donnée du jeu. Le client d'origine n'est pas pris en charge.
- Les **tables du serveur ArchLord d'origine** (valeurs du jeu de l'épisode 8 : compétences, objets, options,
  monstres, régions, boutiques, portails, expérience ; apparitions des monstres, butin, IA, raffinage,
  `world\node_data`) sont incluses dans `Server\Data\server-ini\`. L'import du monde les exige : ne supprimez
  pas ce dossier.
- Aucune connexion Internet : tout fonctionne sur 127.0.0.1.

## 2. Installation

1. Décompressez l'archive dans un dossier de votre disque, par exemple `C:\Jeux\ArchHome\` (pas dans
   Program Files, pas dans un dossier synchronisé en nuage).
2. Placez vos données du jeu dans `Client\` à côté d'`ArchRust.exe`, ou indiquez leur dossier dans
   **Paramètres** > « Données du jeu ». Le launcher détecte l'exécutable fourni dans `Client\`.
   Si les données sont dans un autre dossier (par exemple le dossier `Game` d'une installation d'ArchLord),
   ArchRust est lancé avec `--game`.
   Attention : le dossier des **sources** d'ArchRust (`Cargo.toml`, `crates`, `target`) n'est pas un client
   prêt à jouer.
3. Les tables du serveur d'origine sont déjà dans `Server\Data\server-ini\` ; **Paramètres** permet d'en indiquer
   d'autres.
4. Ouvrez `ArchHome.exe`. Si Windows affiche « Windows a protégé votre ordinateur » : « Informations
   complémentaires » puis « Exécuter quand même ».

Si le dossier est en lecture seule (DVD, Program Files), le launcher propose de le copier sur le disque.

## 3. Premier démarrage

Cliquez sur **JOUER**. Le launcher enchaîne :

1. **MariaDB** : création de la base dans `UserData\Database` (mot de passe tiré au hasard, stocké dans
   `UserData\db-credentials.json`), démarrage sur 127.0.0.1:3316.
2. **Schéma** : `archcore db-init` crée les quatre bases (`archcore_auth`, `archcore_characters`,
   `archcore_world`, `archcore_logs`) et applique les migrations. Cette étape se refait à chaque démarrage :
   après une mise à jour d'ArchCore, le schéma suit tout seul.
3. **Monde** (au premier démarrage) : les tables du client sont déchiffrées dans `Server\Data\client-ini`, puis
   importées dans la base monde avec le relief du terrain : les **valeurs de l'épisode 8** viennent des tables
   du serveur d'origine, les noms et visuels du client ; les titres de PNJ sont traduits. Compter environ une
   minute. Sans les tables du serveur d'origine, le démarrage s'arrête et le dit. L'import se refait seul après
   une mise à jour d'ArchCore ; comptes et personnages sont conservés. Une installation mise à jour depuis une
   version d'avant l'épisode 8 (0.2.0 et avant) passe ensuite ses personnages aux mêmes valeurs
   (`archcore migrate-ep8`), une seule fois, après une sauvegarde « avant-ep8 » : objets, options et compétences
   propres à Archonia sont mis en quarantaine (réversible), l'oubli des compétences est offert.
4. **Tables pour ArchRust** : `archcore export-client-tables` écrit les tables de l'épisode 8 au format du client
   dans `Client\server-data` (à côté de `realmlist.wtf`), après chaque import et si elles manquent. Le serveur
   en annonce l'empreinte (`Client.TablesDir`) ; ArchRust vérifie qu'elle est « conforme au serveur ».
5. **ArchCore** : connexion sur 127.0.0.1:11002, monde sur 127.0.0.1:11008. Le monde se charge en quelques
   secondes (36 000 monstres environ), puis les **bots** d'ArchBot entrent en jeu un par un (voir Monde).
6. **ArchRust** : `realmlist.wtf` pointe sur le serveur local, la langue du jeu est appliquée, puis le jeu
   s'ouvre sur le panneau de connexion d'ArchRust : identifiant et mot de passe, serveur local déjà rempli,
   case « Enregistrer le compte » pour ne plus les saisir (le premier monde, « ArchHome », est pris d'office).

Avant de vous connecter, créez votre compte dans l'onglet **Comptes** (le serveur n'a pas besoin de tourner).

## 4. Accueil

- **État** : client, base de données, serveur, monde (PNJ, monstres, objets importés), tables du serveur.
- **JOUER** : démarre ce qui manque puis lance ArchRust. **Démarrer le serveur seul** : utile pour
  l'explorateur de base ou un autre client. **Arrêter** : arrête ArchCore (Ctrl+C, il ferme proprement ses
  bases) puis MariaDB ; le jeu, s'il est ouvert, est fermé. **Fermer le jeu** : force la fermeture
  d'ArchRust.
- **Vérifier** : contrôle les fichiers du programme (empreintes SHA-256 du package), les données et les bases.
- **Réparer** : recrée les bases manquantes, applique les migrations, régénère les configurations, réimporte
  le monde s'il est vide. Les personnages ne sont jamais touchés.
- Fermer la fenêtre arrête d'abord les services.

## 5. Comptes

- **Créer le compte** : identifiant de 1 à 16 lettres ou chiffres, mot de passe de 4 à 16 caractères sans
  espace ni accent, niveau MJ de 0 (joueur) à 3 (toutes les commandes). Le mot de passe est enregistré comme
  le fait ArchCore (empreinte SHA-256).
- Les comptes `archbot_…` sont ceux des bots, créés par le serveur : ils sont masqués (leur nombre est indiqué).
- Sélectionnez un compte dans la liste pour **changer son mot de passe** (celui saisi au-dessus) ou
  **appliquer le niveau MJ** choisi ; un joueur connecté le voit à sa reconnexion.
- **Supprimer** : le compte, ses personnages (objets, quêtes, compétences, appartenance aux guildes) et sa
  banque, après confirmation. Refusé pendant le jeu.
- Serveur arrêté, la base démarre le temps de l'opération puis s'arrête.

## 6. Monde

Réglages d'ArchCore (`world.conf`) avec leur valeur d'origine ; survolez un libellé pour son explication.

| Groupe | Réglages |
|---|---|
| Monde | densité de monstres (1 à 5), portée de la discussion normale, horloge du ciel (heure du PC ou temps universel) |
| Taux | expérience, or, butin, butin unique, charisme, prestige, bonus de discussion |
| Commandes MJ | niveau requis pour `/move`, `/speed`, `/invincible`, `/kill`, `/additem`, `/level`, `/money` |
| Technique | seuils des « paquets lents » et « requêtes lentes » notés dans le journal du serveur |

« Enregistrer » les garde dans `UserData\settings.json` ; ils s'appliquent au prochain démarrage du serveur.

### Onglet ArchBot

Tous les réglages des bots (`Server\Config\archbot.conf`), par groupe, avec leur explication au survol :

| Groupe | Réglages |
|---|---|
| Activation | bots activés, bots autonomes, compagnons |
| Population | bots autonomes au démarrage (15), bots en jeu au plus (40), délai entre deux entrées, niveaux des nouveaux bots (1 à 10), classes permises (TID), personnages gardés d'une partie à l'autre, durée de session |
| Comportement | combat, butin, progression, groupes, rayons de perception, d'exploration et de ramassage, niveaux des monstres choisis, distance de suivi, réponses des bots |
| Survie | seuils de fuite, de repos, de potions, d'engagement, attente avant résurrection |
| Performances | cadence de l'IA, bots par tour, ralentissement loin des joueurs |
| Administration | niveau MJ des commandes `/bot` de gestion (2), journal détaillé |

Les valeurs sont celles d'ArchBot, sauf la population, réduite pour un jeu solo (modèle : 20 au départ, 100 au
plus). L'onglet rappelle aussi les commandes du jeu. Les changements valent au prochain démarrage du serveur.

**Bots.** Le serveur contient [ArchBot](https://github.com/BlaMacfly/ArchBot) : des personnages pilotés par le
serveur, avec un compte, une classe, un niveau et un équipement, qui suivent les mêmes règles que vous. Les
**bots autonomes** quittent leur village, chassent les monstres de leur niveau, ramassent, s'équipent, se
reposent et reviennent après leur mort ; ils sont repris d'une partie à l'autre. Un **compagnon** s'engage en
jeu : `/bot hire` (le bot libre le plus proche) ou `/bot hire Nom`, puis acceptez son invitation de groupe ; il
vous suit et vous défend. Ordres : `/bot follow [distance]`, `stay`, `come`, `attack`, `passive`, `defensive`,
`aggressive`, `stop`, `status`, aussi en chuchotant au bot ou à tout le groupe par `/p` ; `/bot dismiss` le
libère, `/bot list` liste les bots. Pour les MJ : `/bot stats`, `/bot spawn n`, `/bot despawn n|all|nom`,
`/bot population n`. Journal du serveur : lignes `[archbot]`. Décochez « Bots activés » (onglet ArchBot) pour jouer sans eux.
**Réimporter la base monde** relit le monde depuis le client et les tables du serveur d'origine (après une mise
à jour du client, par exemple) ; comptes et personnages sont conservés, les tables d'ArchRust sont réécrites au
démarrage suivant.

## 7. Packs et AddOns

**Packs de ressources** : graphismes et sons qui remplacent ceux du jeu dans ArchRust. Le pack officiel
`free-assets` est toujours chargé ; ceux de la communauté sont dans le dossier `patches` du client (dossiers ou
archives `.ahp`, chacun avec son `patch.toml`). Vous pouvez les activer, les désactiver, en ajouter (archive ou
dossier) et les supprimer. Le launcher écrit le même fichier que la fenêtre Options du jeu
(`saved-variables\packs.toml`) ; les changements valent au prochain lancement du jeu.

**AddOns Lua** : installés depuis une archive zip (le ou les dossiers contenant un `.toc` sont reconnus),
désactivés par renommage `.off`, supprimés. Ils sont dans le dossier `addons` du client. Le package fournit
ArchAce (cadres d'unité et options, `/options`), HelloArch (exemple) et
[ArchCore Admin](https://github.com/BlaMacfly/ArchCoreAdmin) 1.3.0, qui ajoute un panneau MJ en jeu (`/aca`).

Fermez le jeu avant de changer ses packs ou ses AddOns.

## 8. Base de données

Explorateur des quatre bases : liste des tables, données (filtre `WHERE`, tri, pages de 200 lignes, édition
par double-clic, insertion, suppression de ligne), structure, et requêtes SQL. Les écritures demandent une
confirmation : faites une sauvegarde avant. **Ouvrir dans HeidiSQL** lance HeidiSQL (s'il est fourni) déjà
connecté.

## 9. Sauvegardes

- **Sauvegarder maintenant** : comptes (`archcore_auth`), personnages (`archcore_characters`) et réglages dans
  une archive zip de `UserData\Sauvegardes`, avec l'empreinte de chaque fichier.
- Une **sauvegarde automatique** est faite à chaque fermeture du jeu ; les plus anciennes sont supprimées
  au-delà du nombre choisi dans Paramètres (10 par défaut).
- **Restaurer** : le serveur s'arrête, les empreintes de l'archive sont vérifiées, l'état actuel est
  sauvegardé (« avant-restauration »), puis comptes, personnages et réglages sont remplacés.
- **Exporter un diagnostic** : archive avec les journaux du launcher, du serveur et du client, les
  configurations (mots de passe masqués) et l'état de l'installation, à joindre à une demande d'aide.

## 10. Paramètres et langues

- **Langue du launcher** : automatique (langue de Windows), français ou anglais. Le changement est immédiat.
- **Langue du jeu** : textes envoyés par le serveur (titres des PNJ) et langue d'ArchRust, au prochain
  démarrage.
- **Dossier du client ArchRust**, **données du jeu** (facultatif, si elles ne sont pas à côté d'ArchRust.exe),
  **tables du serveur d'origine**, **données de Webzen** (facultatives : dossier `ini` du client officiel de
  Webzen ; l'import en tire les quêtes, titres, maîtrises et la courbe d'expérience d'origine ; le paquet des
  testeurs a déjà un monde importé avec elles, un réimport sans elles revient à celles du client),
  **traductions d'ArchCore** (facultatives, avec les données de Webzen : dossier des `<langue>.tsv` qui
  complètent les textes de Webzen dans les langues qui ne les ont pas),
  **ports** (11002 / 11008 par défaut :
  changez-les si un autre serveur ArchCore tourne sur ce PC).
- **Sauvegardes automatiques** : activées ou non, nombre conservé.
- **Journaux** : journal détaillé du serveur (chaque paquet, pour un rapport de bug), niveau du journal
  d'ArchRust (`error`, `warn`, `info`, `debug`).

## 11. Journaux

L'onglet **Journaux** a trois vues, avec un filtre texte et un affichage des seuls avertissements et erreurs
(en couleur) :

| Vue | Contenu | Fichiers |
|---|---|---|
| Launcher | chaque étape (démarrages, imports, comptes, sauvegardes, erreurs) | `Logs\launcher.log` |
| Serveur | sortie d'ArchCore (chargement du monde, connexions, avertissements) et des outils d'import | `Logs\server\archcore-<date>.log` (un par démarrage), `Logs\server\outils-<date>.log` |
| Client | sortie d'ArchRust (rendu, packs, AddOns, réseau) | `Logs\client\archrust-<date>.log` (un par partie) |

Les 20 fichiers les plus récents de chaque sorte sont conservés. `Logs\mariadb-error.log` est le journal de la
base. « Ouvrir le fichier » ouvre le fichier en cours dans le Bloc-notes.

## 12. Aide MJ

Les commandes MJ d'ArchCore (à taper dans la discussion du jeu avec un compte de niveau suffisant) et les
raccourcis d'ArchRust. Exemples : `/additem <TID> [n]`, `/level 50`, `/money 1000000`, `/speed 5`,
`/invincible 1`, `/move x,z`.

## 13. Dossiers

| Dossier | Contenu | À conserver |
|---|---|---|
| `Server\ArchCore` | serveur ArchCore, schéma, couche de contenu, traductions | non (programme) |
| `Server\Config` | configurations générées | non |
| `Server\Data` | tables du client déchiffrées, tables du serveur d'origine | régénérées |
| `Client\server-data` | tables de l'épisode 8 écrites pour ArchRust par le serveur | régénérées |
| `Server\Database\mariadb` | MariaDB | non (programme) |
| `UserData` | base de données, réglages, identifiants SQL, sauvegardes | **oui** |
| `Logs` | journaux | au besoin |
| `Client` | exécutable ArchRust fourni ; données du jeu à ajouter par vous | oui |

Mise à jour : décompressez la nouvelle version dans un nouveau dossier, puis, launcher fermé, copiez-y
`UserData` ainsi que vos données du jeu (si elles étaient dans `Client`). Gardez l'`ArchRust.exe`, les AddOns et
les tables `Server\Data\server-ini` de la nouvelle version (celles de la 0.2.0 ne suffisent plus). Au premier
JOUER, les bases sont mises à jour, le monde est réimporté, les tables d'ArchRust sont écrites et, en venant d'une
version d'avant l'épisode 8, les personnages passent aux valeurs de l'épisode 8 (voir Premier démarrage).

## 14. Dépannage

| Symptôme | Que faire |
|---|---|
| « client ArchRust introuvable ou incomplet » | le package doit contenir `Client\ArchRust.exe` ; indiquez vos données du jeu dans Paramètres, avec `character\Reference.Dat` |
| « le port 11002 est déjà utilisé » | un autre serveur ArchCore tourne : arrêtez-le, ou changez les ports dans Paramètres |
| « tables du serveur ArchLord d'origine incomplètes » | des tables manquent dans `Server\Data\server-ini` (le journal du launcher les nomme) : remettez-les depuis le package |
| Infobulles ou prix différents du serveur | le journal du client signale une empreinte de `server-data` différente : relancez le jeu par le launcher (il réécrit les tables), ou Monde > Réimporter |
| Pas de bots | onglet ArchBot : « Bots activés » ; journal du serveur : lignes `[archbot]` |
| ArchRust ne se connecte pas | vérifiez que le serveur est « En service » ; regardez Journaux > Client puis Serveur |
| Le jeu se ferme sur une erreur | Journaux > Client (niveau `debug` dans Paramètres pour plus de détails) |
| Le serveur s'arrête au démarrage | Journaux > Serveur ; essayez Réparer |
| Windows bloque un programme | autorisez-le dans Sécurité Windows, ou ajoutez le dossier aux exclusions de l'antivirus |

Pour demander de l'aide, joignez le diagnostic (Sauvegardes > Exporter un diagnostic). Discord :
[discord.gg/JCyrsTm7fR](https://discord.gg/JCyrsTm7fR).
