# Journal des versions / Changelog

## 0.1.0 — 23 octobre 2026

Correctif avant diffusion / Fix before release :

- Recherche des clients : elle ne regardait que le dossier `Client` et le dossier parent de l'installation ;
  elle parcourt maintenant aussi les dossiers parents, le Bureau, les Documents, les Téléchargements et la racine
  des disques fixes, en arrière-plan (l'interface ne se fige plus), signale les dossiers sans données du jeu
  (dont le dossier des sources d'ArchRust) et enregistre aussitôt le client choisi.
- Nouveau réglage « Données du jeu » : données d'ArchLord dans un autre dossier qu'ArchRust.exe (lancement avec
  `--game`), avec recherche automatique aux emplacements d'ArchRust (`assets-archonia`, `../client/ArchLord/Game`).

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
