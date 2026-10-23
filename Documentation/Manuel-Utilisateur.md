# HomeRPG ArchLord — Manuel d'utilisation

*English version: [User-Manual.md](User-Manual.md).*

HomeRPG ArchLord fait tourner ArchLord en solo sur votre PC : un serveur ArchCore local, une base MariaDB
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
- Le **client ArchRust** : `ArchRust.exe` et, dans le même dossier, les données du client ArchLord 64 bits
  (`character\`, `ini\`, `world\`, `texture\`, `object\`, `effect\`, `sound\`…). HomeRPG ne fournit aucune
  donnée du jeu. Le client d'origine n'est pas pris en charge.
- Facultatif : les **tables du serveur ArchLord d'origine** (`spawndatatable.txt`, `spawninstancetable.txt`,
  `itemdroptable.txt`, `groupdroprate.txt`, `droprankrate.txt`, `world\node_data`). Elles décrivent les
  apparitions de monstres et le butin ; elles ne sont pas dans le client. Sans elles, le monde a ses PNJ, ses
  boutiques et ses quêtes, mais aucun monstre.
- Aucune connexion Internet : tout fonctionne sur 127.0.0.1.

## 2. Installation

1. Décompressez l'archive dans un dossier de votre disque, par exemple `C:\Jeux\HomeRPG ArchLord\` (pas dans
   Program Files, pas dans un dossier synchronisé en nuage).
2. Copiez votre client ArchRust dans `Client\`, ou laissez-le où il est et indiquez son dossier dans
   **Paramètres** (le bouton « Rechercher les clients » le trouve s'il est près du dossier d'installation).
3. Facultatif : copiez les tables du serveur d'origine dans `Server\Data\server-ini\`, ou indiquez leur dossier
   dans **Paramètres**.
4. Ouvrez `HomeRPG ArchLord.exe`. Si Windows affiche « Windows a protégé votre ordinateur » : « Informations
   complémentaires » puis « Exécuter quand même ».

Si le dossier est en lecture seule (DVD, Program Files), le launcher propose de le copier sur le disque.

## 3. Premier démarrage

Cliquez sur **JOUER**. Le launcher enchaîne :

1. **MariaDB** : création de la base dans `UserData\Database` (mot de passe tiré au hasard, stocké dans
   `UserData\db-credentials.json`), démarrage sur 127.0.0.1:3316.
2. **Schéma** : `archcore db-init` crée les quatre bases (`archcore_auth`, `archcore_characters`,
   `archcore_world`, `archcore_logs`) et applique les migrations. Cette étape se refait à chaque démarrage :
   après une mise à jour d'ArchCore, le schéma suit tout seul.
3. **Monde** (une seule fois) : les tables du client sont déchiffrées dans `Server\Data\client-ini`, puis
   importées dans la base monde avec le relief du terrain et, si vous les avez fournies, les tables du serveur
   d'origine ; les titres de PNJ sont traduits. Compter une à quelques minutes.
4. **ArchCore** : connexion sur 127.0.0.1:11002, monde sur 127.0.0.1:11008. Le monde se charge en quelques
   secondes (36 000 monstres environ avec les tables du serveur).
5. **ArchRust** : `realmlist.wtf` pointe sur le serveur local, la langue du jeu est appliquée, puis le jeu
   s'ouvre.

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
**Réimporter la base monde** relit le monde depuis le client (après avoir ajouté les tables du serveur ou mis à
jour le client) ; comptes et personnages sont conservés.

## 7. Packs et AddOns

**Packs de ressources** : graphismes et sons qui remplacent ceux du jeu dans ArchRust. Le pack officiel
`free-assets` est toujours chargé ; ceux de la communauté sont dans le dossier `patches` du client (dossiers ou
archives `.ahp`, chacun avec son `patch.toml`). Vous pouvez les activer, les désactiver, en ajouter (archive ou
dossier) et les supprimer. Le launcher écrit le même fichier que la fenêtre Options du jeu
(`saved-variables\packs.toml`) ; les changements valent au prochain lancement du jeu.

**AddOns Lua** : installés depuis une archive zip (le ou les dossiers contenant un `.toc` sont reconnus),
désactivés par renommage `.off`, supprimés. Ils sont dans le dossier `addons` du client. L'AddOn
[ArchCoreAdmin](https://github.com/BlaMacfly/ArchCoreAdmin) ajoute un panneau MJ en jeu.

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
- **Dossier du client ArchRust**, **tables du serveur d'origine**, **ports** (11002 / 11008 par défaut :
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
| `Server\Database\mariadb` | MariaDB | non (programme) |
| `UserData` | base de données, réglages, identifiants SQL, sauvegardes | **oui** |
| `Logs` | journaux | au besoin |
| `Client` | votre client ArchRust (si vous l'y avez mis) | oui |

Mise à jour : remplacez tout sauf `UserData` (et `Client`, `Server\Data\server-ini`).

## 14. Dépannage

| Symptôme | Que faire |
|---|---|
| « client ArchRust introuvable ou incomplet » | Paramètres > dossier du client : il doit contenir `ArchRust.exe` et `character\Reference.Dat` |
| « le port 11002 est déjà utilisé » | un autre serveur ArchCore tourne : arrêtez-le, ou changez les ports dans Paramètres |
| Aucun monstre | tables du serveur d'origine absentes : ajoutez-les, puis Monde > Réimporter la base monde |
| ArchRust ne se connecte pas | vérifiez que le serveur est « En service » ; regardez Journaux > Client puis Serveur |
| Le jeu se ferme sur une erreur | Journaux > Client (niveau `debug` dans Paramètres pour plus de détails) |
| Le serveur s'arrête au démarrage | Journaux > Serveur ; essayez Réparer |
| Windows bloque un programme | autorisez-le dans Sécurité Windows, ou ajoutez le dossier aux exclusions de l'antivirus |

Pour demander de l'aide, joignez le diagnostic (Sauvegardes > Exporter un diagnostic). Discord :
[discord.gg/JCyrsTm7fR](https://discord.gg/JCyrsTm7fR).
