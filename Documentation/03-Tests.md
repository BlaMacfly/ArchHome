# 03 — Tests

Tous les tests se lancent launcher fermé, ports 3316, 11002 et 11008 libres (aucun autre serveur ArchCore en
marche).

| Script | Vérifie |
|---|---|
| `cargo test --release` (Launcher) | empreinte des mots de passe identique à ArchCore, règles des comptes, bascule des langues, niveaux des journaux, décodage console |
| `Tests\Test-Launcher.ps1` | démarrage complet (`--start`), import du monde au premier lancement, ArchCore prêt, fermeture par la fenêtre : ArchCore arrêté proprement (Ctrl+C), MariaDB arrêtée, journal serveur écrit, ports libérés, aucun processus orphelin, aucune erreur |
| `Tests\Test-Integration.ps1` | compte créé par le launcher (`--do acccreate`), `archcore sim client` (personnage créé, entrée en jeu, commande MJ), `ArchRust.exe login` sans affichage, sauvegarde et suppression du compte serveur arrêté |
| `Tests\Test-Offline.ps1` | écoute sur 127.0.0.1 uniquement, aucune connexion sortante |

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

Non testé par une personne en jeu : la partie complète dans la fenêtre d'ArchRust (connexion manuelle,
création de personnage à l'écran), la fermeture normale du jeu suivie de la sauvegarde automatique, la
restauration depuis l'interface, le package `New-Package.ps1` installé sur un autre PC.
