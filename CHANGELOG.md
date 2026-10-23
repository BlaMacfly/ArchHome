# Journal des versions / Changelog

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
