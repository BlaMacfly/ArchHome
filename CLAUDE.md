# HomeRPG ArchLord — instructions de session

Projet distinct de Sylvania et du développement d'ArchCore/ArchRust (voir `../CLAUDE.md` et
`../HomeRPG ArchLord/PROMPT-CODEX-RELAIS.md`). Répondre en français.

C'est l'équivalent, pour ArchLord, de HomeCraft Edition Vanilla (https://github.com/BlaMacfly/HomeCraft-Vanilla) :
serveur ArchCore local + MariaDB portable + launcher Rust à un seul bouton.

## Lire d'abord
1. `README.md`, puis `Documentation/01-Architecture.md` (choix, démarrage, arrêt par Ctrl+C, comptes, langues, journaux).
2. `Documentation/02-Construction.md` (Build-ArchCore, composants, package) et `Documentation/03-Tests.md`.

## Décisions de l'utilisateur (2026-10-23)
- **ArchRust seulement** : pas de client d'origine (Archonia.exe).
- **Interface bilingue français / anglais**, avec gestion des langues (Paramètres, `i18n.rs`).
- **Journaux côté serveur et côté client** (`Logs/server`, `Logs/client`, onglet Journaux).

## Règles
- La recherche des clients parcourt le dossier du launcher et ses sous-dossiers, et inspecte aussi le chemin
  client explicitement configuré, même s'il est ailleurs. Accepter un chemin vers le dossier ou directement
  vers `ArchRust.exe` ; ne pas lancer de recherche générale sur les disques.
- Toute chaîne affichée passe par `Launcher/src/i18n.rs` (`T("fr", "en")` ou `tf!`), dans les deux langues.
- Ne jamais redistribuer ni committer les données de jeu de `Client/`, `Server/Data/` (tables du client
  déchiffrées, tables du serveur d'origine), `UserData/` ou `Logs/`. Le package peut contenir uniquement
  `Client/ArchRust.exe` sous GPL, avec le commit source indiqué dans `Build/manifest.json` ; aucune donnée du
  client ArchLord dans le dépôt ou le package.
- Les émulateurs de `../HomeRPG ArchLord/references/` ne sont jamais nommés ni copiés (même règle qu'ArchCore).
  Les tables du serveur d'origine se décrivent comme « tables du serveur ArchLord d'origine ».
- ArchCore n'est pas modifié ici : il est compilé tel quel par `Build/Build-ArchCore.ps1` (target-dir séparé,
  `--locked`) au commit de `Build/manifest.json`. Ne jamais remplacer `ArchCore-core/target/release/archcore.exe`.
- Le serveur principal de l'utilisateur (MariaDB 3306, bases `archcore_*` de son instance, ports 11002/11008) n'est
  pas celui de HomeRPG : HomeRPG a sa propre MariaDB sur 3316 (`UserData/Database`). Avant un test, vérifier que
  11002/11008 sont libres ; ne jamais arrêter le serveur de l'utilisateur.
- Un test qui lance JOUER modifie `realmlist.wtf` et `saved-variables/options.toml` du client choisi : sauvegarder
  puis remettre ces deux fichiers si c'est le client de l'utilisateur.
- Scripts PowerShell en UTF-8 avec BOM, jamais `2>$null` sur un exe natif (passer par `cmd /c … 2>&1`).
- Ne jamais lancer un test pendant que le launcher de l'utilisateur est ouvert.
- Commit ou push seulement à la demande de l'utilisateur.
