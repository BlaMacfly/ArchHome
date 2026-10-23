//! Toutes les chaînes affichées par le launcher, en français et en anglais.
//!
//! Une chaîne fixe est une paire `T("français", "english")` ; `T::s()` rend celle de la langue courante. Les
//! messages composés passent par les fonctions en bas de fichier ou par la macro `tf!` (deux formats, mêmes
//! arguments). La langue est globale (changée à chaud depuis Paramètres) : l'interface et l'orchestrateur la
//! lisent au moment d'afficher ou de journaliser.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Lang {
    Fr,
    En,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::Fr, Lang::En];

    /// Code envoyé au serveur (World.Language) et à ArchRust (options.toml).
    pub fn code(self) -> &'static str {
        match self {
            Lang::Fr => "fr",
            Lang::En => "en",
        }
    }

    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Fr => "Français",
            Lang::En => "English",
        }
    }

    pub fn parse(s: &str) -> Option<Lang> {
        match s.to_ascii_lowercase().as_str() {
            "fr" | "fra" | "french" | "français" => Some(Lang::Fr),
            "en" | "eng" | "english" | "anglais" => Some(Lang::En),
            _ => None,
        }
    }
}

static LANG: AtomicU8 = AtomicU8::new(0);

pub fn set_lang(l: Lang) {
    LANG.store(if l == Lang::En { 1 } else { 0 }, Ordering::Relaxed);
}

pub fn lang() -> Lang {
    if LANG.load(Ordering::Relaxed) == 1 { Lang::En } else { Lang::Fr }
}

pub fn is_fr() -> bool {
    lang() == Lang::Fr
}

/// Langue de l'interface de Windows : français si la langue principale est le français, anglais sinon.
pub fn system_lang() -> Lang {
    let id = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
    // Identifiant principal : 10 bits de poids faible ; 0x0C = français (France, Belgique, Canada, Suisse…).
    if id & 0x3ff == 0x0c { Lang::Fr } else { Lang::En }
}

/// Chaîne fixe en deux langues.
#[derive(Clone, Copy, Debug)]
pub struct T(pub &'static str, pub &'static str);

impl T {
    pub fn s(&self) -> &'static str {
        if is_fr() { self.0 } else { self.1 }
    }
}

/// Message composé dans la langue courante : `tf!("fr {x}", "en {x}")` ou `tf!("fr {}", "en {}", x)`.
#[macro_export]
macro_rules! tf {
    ($fr:literal, $en:literal $(, $a:expr)* $(,)?) => {
        if $crate::i18n::is_fr() { format!($fr $(, $a)*) } else { format!($en $(, $a)*) }
    };
}

// ------------------------------------------------------------------ général
pub const SUBTITLE: T = T("ArchLord en solo, hors ligne, sur ArchCore et ArchRust", "Single-player, offline ArchLord on ArchCore and ArchRust");
pub const CONFIRM: T = T("Confirmer", "Confirm");
pub const CANCEL: T = T("Annuler", "Cancel");
pub const SAVE: T = T("Enregistrer", "Save");
pub const REFRESH: T = T("Actualiser", "Refresh");
pub const BROWSE: T = T("Parcourir…", "Browse…");
pub const OPEN_DIR: T = T("Ouvrir le dossier", "Open folder");
pub const ENABLE: T = T("Activer", "Enable");
pub const DISABLE: T = T("Désactiver", "Disable");
pub const REMOVE: T = T("Supprimer", "Remove");
pub const ENABLED: T = T("activé", "enabled");
pub const DISABLED: T = T("désactivé", "disabled");
pub const ENABLED_LABEL: T = T("Activé", "Enabled");
pub const DISABLED_LABEL: T = T("Désactivé", "Disabled");
pub const PRESENT: T = T("présent", "present");
pub const ABSENT: T = T("ABSENT", "MISSING");
pub const INCOMPLETE: T = T("INCOMPLET", "INCOMPLETE");
pub const RUNNING: T = T("En service", "Running");
pub const STOPPED: T = T("Arrêté", "Stopped");
pub const YES: T = T("oui", "yes");
pub const NO: T = T("non", "no");
pub const LOGIN: T = T("connexion", "login");
pub const WORLD: T = T("monde", "world");
pub const SPAWN_BLOCKED_APP_CONTROL: T = T(
    "Windows a bloqué ce programme (Contrôle intelligent des applications ou stratégie d'entreprise) : autorisez-le dans Sécurité Windows > Contrôle des applications et du navigateur",
    "Windows blocked this program (Smart App Control or a company policy): allow it in Windows Security > App & browser control",
);
pub const SPAWN_BLOCKED_ANTIVIRUS: T = T(
    "l'antivirus a bloqué ce programme : ajoutez le dossier HomeRPG ArchLord aux exclusions",
    "the antivirus blocked this program: add the HomeRPG ArchLord folder to its exclusions",
);

// ------------------------------------------------------------------ onglets
pub const TAB_HOME: T = T("Accueil", "Home");
pub const TAB_ACCOUNTS: T = T("Comptes", "Accounts");
pub const TAB_WORLD: T = T("Monde", "World");
pub const TAB_PACKS: T = T("Packs et AddOns", "Packs & AddOns");
pub const TAB_DATABASE: T = T("Base de données", "Database");
pub const TAB_BACKUPS: T = T("Sauvegardes", "Backups");
pub const TAB_SETTINGS: T = T("Paramètres", "Settings");
pub const TAB_LOGS: T = T("Journaux", "Logs");
pub const TAB_HELP: T = T("Aide MJ", "GM help");

// ------------------------------------------------------------------ états et messages
pub const STATE_NOT_INSTALLED: T = T("NON INSTALLÉ", "NOT INSTALLED");
pub const STATE_INSTALLED: T = T("ARRÊTÉ", "STOPPED");
pub const STATE_VERIFYING: T = T("VÉRIFICATION", "CHECKING");
pub const STATE_INITIALIZING: T = T("PRÉPARATION", "PREPARING");
pub const STATE_STARTING_DB: T = T("DÉMARRAGE DE LA BASE", "STARTING DATABASE");
pub const STATE_IMPORTING: T = T("IMPORT DU MONDE", "IMPORTING WORLD");
pub const STATE_STARTING_SERVER: T = T("DÉMARRAGE DU SERVEUR", "STARTING SERVER");
pub const STATE_READY: T = T("PRÊT", "READY");
pub const STATE_PLAYING: T = T("JEU EN COURS", "PLAYING");
pub const STATE_STOPPING: T = T("ARRÊT", "STOPPING");
pub const STATE_ERROR: T = T("ERREUR", "ERROR");
pub const STATE_REPAIRING: T = T("MAINTENANCE", "MAINTENANCE");

pub const MSG_IDLE: T = T("Services arrêtés. Cliquez sur JOUER.", "Services stopped. Click PLAY.");
pub const MSG_NOT_INSTALLED: T = T("Composants manquants : utilisez la version complète du package.", "Missing components: use the complete package.");
pub const MSG_VERIFY: T = T("Vérification des composants…", "Checking components…");
pub const MSG_VERIFYING_FILES: T = T("Vérification des fichiers", "Checking files");
pub const MSG_DB: T = T("Démarrage de MariaDB…", "Starting MariaDB…");
pub const MSG_SCHEMA: T = T("Mise à jour du schéma des bases (archcore db-init)…", "Updating the database schema (archcore db-init)…");
pub const MSG_DECRYPT: T = T("Premier démarrage : lecture des tables du client…", "First start: reading the client tables…");
pub const MSG_IMPORT: T = T("Premier démarrage : import du monde (quelques minutes, une seule fois)…", "First start: importing the world (a few minutes, only once)…");
pub const MSG_I18N: T = T("Premier démarrage : textes traduits…", "First start: translated texts…");
pub const MSG_IMPORT_EMPTY: T = T("l'import n'a rempli aucune table du monde : voir le journal du serveur", "the import filled no world table: see the server log");
pub const MSG_SERVER: T = T("Démarrage d'ArchCore et chargement du monde…", "Starting ArchCore and loading the world…");
pub const MSG_READY: T = T("Serveur prêt. Cliquez sur JOUER.", "Server ready. Click PLAY.");
pub const MSG_PLAYING: T = T("Bon jeu ! La sauvegarde automatique se fera à la fermeture du jeu.", "Have fun! The automatic backup runs when the game closes.");
pub const MSG_STOPPING: T = T("Arrêt des services…", "Stopping services…");
pub const MSG_STOPPED: T = T("Services arrêtés.", "Services stopped.");
pub const MSG_CLOSING: T = T("Fermeture : arrêt des services en cours…", "Closing: stopping services…");
pub const MSG_CLIENT_NEEDED: T = T(
    "client ArchRust introuvable ou incomplet : indiquez son dossier dans Paramètres (ArchRust.exe et les données du jeu)",
    "ArchRust client not found or incomplete: set its folder in Settings (ArchRust.exe and the game data)",
);
pub const MSG_BACKUP: T = T("Sauvegarde en cours", "Backing up");
pub const MSG_BACKUP_DONE: T = T("Sauvegarde terminée :", "Backup finished:");
pub const MSG_RESTORE: T = T("Restauration en cours", "Restoring");
pub const MSG_RESTORE_DONE: T = T("Restauration terminée. Cliquez sur JOUER.", "Restore finished. Click PLAY.");
pub const MSG_REPAIR: T = T("Réparation en cours", "Repairing");
pub const MSG_REPAIR_DONE: T = T("Réparation terminée.", "Repair finished.");
pub const MSG_WORLD_REIMPORT: T = T("Réimport de la base monde", "Reimporting the world database");
pub const MSG_WORLD_REIMPORT_DONE: T = T("Base monde réimportée. Cliquez sur JOUER.", "World database reimported. Click PLAY.");
pub const MSG_INSTALLING: T = T("Copie sur le disque", "Copying to disk");
pub const MSG_INSTALLED: T = T("Copie terminée : le launcher installé s'ouvre.", "Copy finished: the installed launcher opens.");
pub const MSG_DIAG_DONE: T = T("Diagnostic exporté :", "Diagnostic exported:");
pub const DB_TIMEOUT: T = T("la base de données ne répond pas dans le délai imparti", "the database does not respond in time");
pub const DB_STOP_TIMEOUT: T = T("la base de données répond toujours après le délai d'arrêt", "the database still responds after the stop timeout");
pub const DB_NOT_OURS: T = T(" (déjà en service, non gérée par ce launcher)", " (already running, not managed by this launcher)");
pub const WORLD_TIMEOUT: T = T("le serveur ne répond pas dans le délai imparti (10 minutes)", "the server does not respond in time (10 minutes)");
pub const GAME_CRASHED: T = T("ArchRust s'est fermé sur une erreur : voir Journaux > Client.", "ArchRust closed with an error: see Logs > Client.");
pub const CLIENT_FILES_PLAYING: T = T("Fermez le jeu avant de modifier ses packs ou ses AddOns.", "Close the game before changing its packs or AddOns.");

// Contextes d'erreur (« <contexte> : <erreur> »)
pub const CTX_START: T = T("démarrage", "start");
pub const CTX_VERIFY: T = T("vérification", "check");
pub const CTX_DB: T = T("démarrage de MariaDB", "MariaDB start");
pub const CTX_SCHEMA: T = T("schéma des bases", "database schema");
pub const CTX_WORLD: T = T("import du monde", "world import");
pub const CTX_SERVER: T = T("démarrage d'ArchCore", "ArchCore start");
pub const CTX_CLIENT: T = T("client", "client");
pub const CTX_SERVICE_DOWN: T = T("service interrompu", "service interrupted");
pub const CTX_REPAIR: T = T("réparation", "repair");
pub const CTX_RESTORE: T = T("restauration", "restore");
pub const CTX_BACKUP: T = T("Sauvegarde impossible", "Backup failed");
pub const CTX_REIMPORT: T = T("réimport de la base monde", "world database reimport");
pub const CTX_INSTALL: T = T("installation", "installation");

// Support en lecture seule
pub const RO_TITLE: T = T("Le dossier actuel est en lecture seule", "The current folder is read-only");
pub const RO_TEXT: T = T(
    "HomeRPG ArchLord doit être copié sur un disque inscriptible (pas dans Program Files ni sur un DVD).",
    "HomeRPG ArchLord must be copied to a writable disk (not Program Files, not a DVD).",
);
pub const RO_INSTALL: T = T("Copier sur le disque…", "Copy to disk…");
pub const RO_PICK_TITLE: T = T("Dossier de destination", "Destination folder");

// ------------------------------------------------------------------ accueil
pub const COMP_CLIENT: T = T("Client ArchRust", "ArchRust client");
pub const COMP_DB: T = T("Base de données", "Database");
pub const COMP_SERVER: T = T("Serveur ArchCore", "ArchCore server");
pub const COMP_WORLD: T = T("Monde", "World");
pub const COMP_TABLES: T = T("Tables du serveur d'origine", "Original server tables");
pub const CLIENT_OK: T = T("Prêt", "Ready");
pub const CLIENT_REPAIR: T = T("Données du jeu incomplètes", "Game data incomplete");
pub const CLIENT_MISSING: T = T("ArchRust.exe introuvable", "ArchRust.exe not found");
pub const CLIENT_MISSING_DATA: T = T("Données manquantes :", "Missing data:");
pub const SERVER_RUNNING: T = T("En service (connexion et monde)", "Running (login and world)");
pub const SERVER_STARTING: T = T("Démarrage…", "Starting…");
pub const WORLD_EMPTY: T = T("Pas encore importé (au premier JOUER)", "Not imported yet (on the first PLAY)");
pub const WORLD_IMPORTED_STOPPED: T = T("Importé (le détail s'affiche quand la base tourne)", "Imported (details show while the database runs)");
pub const TABLES_OK: T = T("Présentes : monstres et butin importés", "Present: monsters and loot imported");
pub const TABLES_MISSING: T = T("Absentes : le monde n'aura ni monstres ni butin (voir Paramètres)", "Missing: the world will have no monsters or loot (see Settings)");
pub const BTN_PLAY: T = T("JOUER", "PLAY");
pub const BTN_START_SERVER: T = T("Démarrer le serveur seul", "Start the server only");
pub const BTN_STOP: T = T("Arrêter", "Stop");
pub const BTN_KILL_GAME: T = T("Fermer le jeu", "Close the game");
pub const BTN_KILL_GAME_HINT: T = T("Ferme ArchRust de force (s'il ne répond plus).", "Force ArchRust to close (if it no longer responds).");
pub const BTN_VERIFY: T = T("Vérifier", "Check");
pub const BTN_REPAIR: T = T("Réparer", "Repair");
pub const BTN_BACKUP: T = T("Sauvegarder", "Back up");
pub const BTN_RESTORE: T = T("Restaurer…", "Restore…");
pub const HOME_CLIENT_HINT: T = T(
    "Indiquez le dossier du client ArchRust (ArchRust.exe et les données d'ArchLord) dans Paramètres.",
    "Set the ArchRust client folder (ArchRust.exe and the ArchLord data) in Settings.",
);
pub const HOME_FIRST_START: T = T(
    "Premier démarrage : le monde est importé depuis votre client (quelques minutes, une seule fois).",
    "First start: the world is imported from your client (a few minutes, only once).",
);
pub const HOME_DIAG: T = T("Informations", "Information");
pub const HOME_INSTALL_DIR: T = T("Dossier d'installation", "Installation folder");
pub const HOME_VERSION: T = T("Version", "Version");
pub const HOME_PORTS: T = T("Ports (127.0.0.1)", "Ports (127.0.0.1)");
pub const LOG_SEE_SERVER: T = T("Voir le journal du serveur", "See the server log");

// ------------------------------------------------------------------ comptes
pub const ACC_TITLE: T = T("Comptes de jeu", "Game accounts");
pub const ACC_HINT: T = T(
    "Créez votre compte ici, puis connectez-vous avec dans ArchRust. Serveur arrêté, la base démarre le temps de l'opération.",
    "Create your account here, then log in with it in ArchRust. With the server stopped, the database starts for the operation.",
);
pub const ACC_USER: T = T("Identifiant", "Username");
pub const ACC_PASS: T = T("Mot de passe", "Password");
pub const ACC_GM: T = T("Niveau MJ", "GM level");
pub const ACC_GM_HINT: T = T("0 joueur, 1 à 3 maître du jeu (3 : toutes les commandes)", "0 player, 1 to 3 game master (3: every command)");
pub const ACC_LAST: T = T("Dernière connexion", "Last login");
pub const ACC_CREATE: T = T("Créer le compte", "Create account");
pub const ACC_SET_PASS: T = T("Changer le mot de passe", "Change password");
pub const ACC_SET_PASS_HINT: T = T("Du compte sélectionné dans la liste, avec le mot de passe saisi.", "Of the account selected in the list, with the password typed above.");
pub const ACC_SET_GM: T = T("Appliquer le niveau MJ", "Apply GM level");
pub const ACC_DELETE: T = T("Supprimer", "Delete");
pub const ACC_INVALID: T = T(
    "Identifiant : 1 à 16 lettres ou chiffres. Mot de passe : 4 à 16 caractères sans espace ni accent.",
    "Username: 1 to 16 letters or digits. Password: 4 to 16 characters, no spaces or accents.",
);
pub const ACC_CREATE_FAIL: T = T("Création du compte impossible", "Account creation failed");
pub const ACC_DELETE_FAIL: T = T("Suppression impossible", "Deletion failed");
pub const ACC_DELETE_PLAYING: T = T("Fermez le jeu avant de supprimer un compte.", "Close the game before deleting an account.");
pub const ACC_DELETE_CONFIRM_TITLE: T = T("Supprimer le compte ?", "Delete the account?");
pub const ACC_DELETE_CONFIRM: T = T(
    "Le compte et tous ses personnages (objets, quêtes, compétences, banque) seront supprimés définitivement. Faites une sauvegarde avant si vous avez un doute.",
    "The account and all its characters (items, quests, skills, bank) will be deleted for good. Make a backup first if in doubt.",
);
pub const ACC_DB_TEMP: T = T("La liste s'affiche quand la base est démarrée (Actualiser la démarre).", "The list appears when the database is running (Refresh starts it).");

// ------------------------------------------------------------------ monde
pub const WORLD_TITLE: T = T("Réglages du monde", "World settings");
pub const WORLD_INTRO: T = T(
    "Réglages d'ArchCore (world.conf), avec leurs valeurs d'origine. Survolez un libellé pour son explication.",
    "ArchCore settings (world.conf), with their original values. Hover a label for its explanation.",
);
pub const OPT_MODIFIED: T = T("modifié", "modified");
pub const OPT_DEFAULT: T = T("défaut", "default");
pub const OPT_RESET_ALL: T = T("Tout remettre par défaut", "Reset all to default");
pub const WORLD_DATA_TITLE: T = T("Données du monde", "World data");
pub const WORLD_DATA_TEXT: T = T(
    "Le monde (PNJ, objets, quêtes, compétences, relief) est importé de votre client au premier démarrage ; monstres et butin viennent des tables du serveur d'origine si vous les avez fournies (Paramètres).",
    "The world (NPCs, items, quests, skills, terrain) is imported from your client on first start; monsters and loot come from the original server tables if you provided them (Settings).",
);
pub const BTN_WORLD_REIMPORT: T = T("Réimporter la base monde", "Reimport the world database");
pub const WORLD_REIMPORT_HINT: T = T(
    "Après avoir ajouté les tables du serveur d'origine ou mis à jour le client. Personnages et comptes conservés.",
    "After adding the original server tables or updating the client. Characters and accounts are kept.",
);
pub const WORLD_REIMPORT_PLAYING: T = T("Fermez le jeu avant de réimporter le monde.", "Close the game before reimporting the world.");
pub const WORLD_REIMPORT_CONFIRM_TITLE: T = T("Réimporter la base monde ?", "Reimport the world database?");
pub const WORLD_REIMPORT_CONFIRM: T = T(
    "Le serveur s'arrête, puis le monde est relu depuis le client (quelques minutes). Personnages et comptes ne sont pas touchés.",
    "The server stops, then the world is read again from the client (a few minutes). Characters and accounts are not touched.",
);

// ------------------------------------------------------------------ packs et AddOns
pub const PACK_TITLE: T = T("Packs de ressources", "Resource packs");
pub const PACK_INTRO: T = T(
    "Graphismes et sons qui remplacent ceux du jeu dans ArchRust : le pack officiel (free-assets) et ceux de la communauté (dossier patches, dossiers ou archives .ahp).",
    "Graphics and sounds that replace the game's in ArchRust: the official pack (free-assets) and community packs (patches folder, folders or .ahp archives).",
);
pub const PACK_ADD_FILE: T = T("Ajouter une archive .ahp…", "Add an .ahp archive…");
pub const PACK_ADD_DIR: T = T("Ajouter un dossier…", "Add a folder…");
pub const PACK_NONE: T = T("Aucun pack trouvé dans le dossier du client.", "No pack found in the client folder.");
pub const PACK_NAME: T = T("Pack", "Pack");
pub const PACK_VERSION: T = T("Version", "Version");
pub const PACK_AUTHOR: T = T("Auteur", "Author");
pub const PACK_SIZE: T = T("Taille", "Size");
pub const PACK_STATE: T = T("État", "State");
pub const PACK_PRIORITY: T = T("priorité", "priority");
pub const PACK_OFFICIAL_LABEL: T = T("Officiel (toujours chargé)", "Official (always loaded)");
pub const PACK_OFFICIAL: T = T("le pack officiel ne se supprime pas", "the official pack cannot be removed");
pub const PACK_NO_MANIFEST: T = T("pas de patch.toml : ce n'est pas un pack ArchRust", "no patch.toml: this is not an ArchRust pack");
pub const PACK_ADDED: T = T("Pack ajouté :", "Pack added:");
pub const PACK_ADD_FAIL: T = T("Ajout impossible", "Could not add");
pub const PACK_RESTART_NOTE: T = T(
    "Pris en compte au prochain lancement du jeu. Les mêmes réglages se trouvent dans le jeu : Options > Packs de ressources.",
    "Applied the next time the game starts. The same settings are in the game: Options > Resource packs.",
);
pub const PACK_REMOVE_CONFIRM_TITLE: T = T("Supprimer le pack ?", "Remove the pack?");
pub const PACK_REMOVE_CONFIRM: T = T("Le pack est effacé du dossier patches du client.", "The pack is deleted from the client's patches folder.");
pub const ADDON_TITLE: T = T("AddOns Lua", "Lua AddOns");
pub const ADDON_INTRO: T = T(
    "AddOns d'ArchRust (dossier addons du client) : installation depuis une archive zip, désactivation, suppression.",
    "ArchRust AddOns (client addons folder): install from a zip archive, disable, remove.",
);
pub const ADDON_ADD: T = T("Installer depuis un zip…", "Install from a zip…");
pub const ADDON_NONE: T = T("Aucun AddOn installé.", "No AddOn installed.");
pub const ADDON_NAME: T = T("Dossier", "Folder");
pub const ADDON_TITLE_COL: T = T("Titre", "Title");
pub const ADDON_NO_TOC: T = T("Pas de .toc", "No .toc");
pub const ADDON_NO_TOC_ZIP: T = T("aucun fichier .toc trouvé : ce n'est pas une archive d'AddOn", "no .toc file found: this is not an AddOn archive");
pub const ADDON_ADDED: T = T("AddOn installé :", "AddOn installed:");
pub const ADDON_ADD_FAIL: T = T("Installation impossible", "Installation failed");
pub const ADDON_REMOVE_CONFIRM_TITLE: T = T("Supprimer l'AddOn ?", "Remove the AddOn?");
pub const ADDON_REMOVE_CONFIRM: T = T("Le dossier de l'AddOn est effacé (ses variables sauvegardées restent).", "The AddOn folder is deleted (its saved variables stay).");

// ------------------------------------------------------------------ base de données
pub const DB_TITLE: T = T("Base de données", "Database");
pub const DB_INTRO: T = T(
    "Explorateur des bases d'ArchCore : comptes (auth), personnages (characters), monde importé (world), journaux (logs). Les écritures demandent une confirmation : faites une sauvegarde avant.",
    "Explorer for the ArchCore databases: accounts (auth), characters, imported world, logs. Writes ask for confirmation: make a backup first.",
);
pub const DB_OPEN_HEIDI: T = T("Ouvrir dans HeidiSQL", "Open in HeidiSQL");
pub const DB_HEIDI_MISSING: T = T("HeidiSQL n'est pas présent dans Tools\\HeidiSQL.", "HeidiSQL is not in Tools\\HeidiSQL.");
pub const DB_NEED_DB: T = T("La base de données n'est pas démarrée.", "The database is not running.");
pub const DB_READONLY_ONLY: T = T("seules les lectures passent par cette commande", "only reads go through this command");
pub const DB_TAB_DATA: T = T("Données", "Data");
pub const DB_TAB_STRUCTURE: T = T("Structure", "Structure");
pub const DB_FILTER: T = T("Filtre (WHERE) :", "Filter (WHERE):");
pub const DB_APPLY_FILTER: T = T("Appliquer", "Apply");
pub const DB_INSERT: T = T("Insérer une ligne", "Insert a row");
pub const DB_INSERT_HINT: T = T("Champs vides : valeur par défaut. NULL : valeur nulle.", "Empty fields: default value. NULL: null value.");
pub const DB_NO_PK: T = T("Table sans clé primaire : lecture seule.", "Table without a primary key: read-only.");
pub const DB_EDIT_HINT: T = T("Double-clic sur une cellule pour la modifier, Entrée pour valider.", "Double-click a cell to edit it, Enter to apply.");
pub const DB_DELETE_ROW: T = T("Supprimer la ligne ?", "Delete the row?");
pub const DB_DELETE_ROW_CONFIRM: T = T("Cette ligne sera supprimée de la base.", "This row will be deleted from the database.");
pub const DB_COL_NAME: T = T("Colonne", "Column");
pub const DB_COL_KEY: T = T("Clé", "Key");
pub const DB_COL_DEFAULT: T = T("Défaut", "Default");
pub const DB_RUN_READ: T = T("Exécuter (lecture)", "Run (read)");
pub const DB_RUN_WRITE: T = T("Exécuter (écriture)…", "Run (write)…");
pub const DB_WRITE_CONFIRM_TITLE: T = T("Exécuter cette écriture ?", "Run this write?");
pub const DB_WRITE_CONFIRM: T = T("L'instruction modifie la base. Une sauvegarde préalable est conseillée.", "The statement changes the database. A backup beforehand is advised.");
pub const DB_HISTORY: T = T("Historique", "History");
pub const DB_AFFECTED: T = T("ligne(s) affectée(s)", "row(s) affected");
pub const DB_ROWS: T = T("ligne(s)", "row(s)");

// ------------------------------------------------------------------ sauvegardes
pub const BK_TITLE: T = T("Sauvegardes", "Backups");
pub const BK_HINT: T = T(
    "Comptes, personnages et réglages, dans une archive vérifiée par empreinte (UserData\\Sauvegardes). Une sauvegarde automatique est faite à chaque fermeture du jeu.",
    "Accounts, characters and settings, in a checksum-verified archive (UserData\\Sauvegardes). An automatic backup is made each time the game closes.",
);
pub const BK_PLAYING_WARN: T = T("Jeu en cours : la sauvegarde contient l'état enregistré par le serveur (positions à la minute près).", "Game running: the backup holds the state saved by the server (positions to the minute).");
pub const BK_LABEL: T = T("Libellé", "Label");
pub const BK_NOW: T = T("Sauvegarder maintenant", "Back up now");
pub const BK_RESTORE_SEL: T = T("Restaurer la sélection…", "Restore selection…");
pub const BK_DELETE_SEL: T = T("Supprimer la sélection…", "Delete selection…");
pub const BK_DIAG: T = T("Exporter un diagnostic", "Export a diagnostic");
pub const BK_DIAG_HINT: T = T(
    "Archive zip avec les journaux du launcher, du serveur et du client, et les configurations (mots de passe masqués), à joindre à une demande d'aide.",
    "Zip archive with the launcher, server and client logs and the configuration (passwords masked), to attach to a help request.",
);
pub const BK_NONE: T = T("Aucune sauvegarde pour l'instant.", "No backup yet.");
pub const BK_DATE: T = T("Date", "Date");
pub const BK_CHARS: T = T("Personnages", "Characters");
pub const BK_ACCOUNTS: T = T("Comptes", "Accounts");
pub const BK_RESTORE_PLAYING: T = T("Fermez le jeu avant de restaurer.", "Close the game before restoring.");
pub const BK_RESTORE_CONFIRM_TITLE: T = T("Restaurer cette sauvegarde ?", "Restore this backup?");
pub const BK_RESTORE_CONFIRM: T = T(
    "Le serveur s'arrête, l'état actuel est d'abord sauvegardé, puis comptes, personnages et réglages sont remplacés par ceux de l'archive.",
    "The server stops, the current state is backed up first, then accounts, characters and settings are replaced by those of the archive.",
);
pub const BK_DELETE_CONFIRM_TITLE: T = T("Supprimer cette sauvegarde ?", "Delete this backup?");
pub const BK_DELETE_CONFIRM: T = T("L'archive sera effacée du disque.", "The archive will be erased from disk.");
pub const BK_COMPRESSING: T = T("compression", "compressing");
pub const BK_CHECKING: T = T("vérification de l'archive", "checking the archive");
pub const BK_SAFETY: T = T("sauvegarde de sécurité de l'état actuel", "safety backup of the current state");

// ------------------------------------------------------------------ paramètres
pub const SET_LANG_TITLE: T = T("Langues", "Languages");
pub const SET_UI_LANG: T = T("Langue du launcher :", "Launcher language:");
pub const SET_LANG_AUTO: T = T("Automatique (langue de Windows)", "Automatic (Windows language)");
pub const SET_GAME_LANG: T = T("Langue du jeu :", "Game language:");
pub const SET_GAME_LANG_HINT: T = T(
    "Textes envoyés par le serveur (titres des PNJ) et langue d'ArchRust, appliqués au prochain démarrage.",
    "Texts sent by the server (NPC titles) and ArchRust's language, applied on the next start.",
);
pub const SET_CLIENT_DIR: T = T("Dossier du client ArchRust", "ArchRust client folder");
pub const SET_CLIENT_HINT: T = T(
    "Dossier contenant ArchRust.exe et les données du jeu ArchLord (character, ini, world…). Vide : dossier Client du launcher.",
    "Folder holding ArchRust.exe and the ArchLord game data (character, ini, world…). Empty: the launcher's Client folder.",
);
pub const SET_GAME_DIR: T = T("Données du jeu ArchLord (facultatif)", "ArchLord game data (optional)");
pub const SET_GAME_DIR_AUTO: T = T("automatique : à côté d'ArchRust.exe", "automatic: next to ArchRust.exe");
pub const SET_GAME_DIR_HINT: T = T(
    "Seulement si les données du jeu (character, ini, world…) sont dans un autre dossier qu'ArchRust.exe, par exemple le dossier Game d'une installation d'ArchLord. ArchRust est alors lancé avec --game.",
    "Only if the game data (character, ini, world…) is in another folder than ArchRust.exe, for example the Game folder of an ArchLord installation. ArchRust is then started with --game.",
);
pub const SET_GAME_DIR_USED: T = T("Données du jeu utilisées :", "Game data used:");
pub const CLIENT_NO_DATA_HINT: T = T(
    "ArchRust.exe est là, mais pas les données du jeu : indiquez leur dossier dans « Données du jeu », ou choisissez un client complet ci-dessous.",
    "ArchRust.exe is here, but not the game data: set their folder in \"Game data\", or pick a complete client below.",
);
pub const CLIENT_SOURCE_TREE: T = T(
    "Ce dossier est celui des sources d'ArchRust (Cargo.toml, crates, target), pas un client prêt à jouer : il ne contient pas les données du jeu. Choisissez le dossier de jeu (ArchRust.exe avec character, ini, world…), ou indiquez les données du jeu ci-dessus.",
    "This is ArchRust's source folder (Cargo.toml, crates, target), not a ready-to-play client: it holds no game data. Pick the game folder (ArchRust.exe with character, ini, world…), or set the game data above.",
);
pub const CLIENT_SOURCE_TREE_SHORT: T = T("dossier des sources d'ArchRust, sans données du jeu", "ArchRust source folder, no game data");
pub const SET_CLIENT_DETECTED: T = T("Client détecté :", "Detected client:");
pub const SET_CLIENT_SCAN: T = T("Rechercher les clients", "Find clients");
pub const SET_CLIENT_SCAN_HINT: T = T(
    "Cherche ArchRust.exe autour du dossier d'installation, dans le Bureau, les Documents, les Téléchargements et à la racine des disques.",
    "Looks for ArchRust.exe around the installation folder, in Desktop, Documents, Downloads and at the root of the drives.",
);
pub const SET_CLIENT_SEARCHING: T = T("Recherche des clients ArchRust…", "Searching for ArchRust clients…");
pub const SET_CLIENT_NONE_FOUND: T = T(
    "Aucun ArchRust.exe trouvé. Indiquez son dossier avec « Parcourir… » (ArchRust.exe et les données du jeu dans le même dossier).",
    "No ArchRust.exe found. Set its folder with \"Browse…\" (ArchRust.exe and the game data in the same folder).",
);
pub const SET_CLIENT_USE: T = T("Utiliser", "Use");
pub const SET_CLIENT_IN_USE: T = T("Utilisé", "In use");
pub const SET_TABLES_DIR: T = T("Tables du serveur d'origine (facultatif)", "Original server tables (optional)");
pub const SET_TABLES_HINT: T = T(
    "Apparitions des monstres, butin et carte du serveur ArchLord d'origine (spawndatatable.txt, spawninstancetable.txt, itemdroptable.txt…). Elles ne sont pas dans le client et HomeRPG ne les fournit pas. Après les avoir ajoutées : Monde > Réimporter la base monde.",
    "Monster spawns, loot and map of the original ArchLord server (spawndatatable.txt, spawninstancetable.txt, itemdroptable.txt…). They are not in the client and HomeRPG does not provide them. After adding them: World > Reimport the world database.",
);
pub const SET_PORTS: T = T("Ports du serveur", "Server ports");
pub const SET_PORTS_HINT: T = T(
    "11002 et 11008 par défaut, comme le jeu d'origine. Changez-les si un autre serveur ArchCore tourne déjà sur ce PC.",
    "11002 and 11008 by default, like the original game. Change them if another ArchCore server already runs on this PC.",
);
pub const SET_AUTO_BACKUP: T = T("Sauvegarde automatique à la fermeture du jeu", "Automatic backup when the game closes");
pub const SET_AUTO_BACKUP_KEEP: T = T("Sauvegardes automatiques conservées :", "Automatic backups kept:");
pub const SET_SERVER_TRACE: T = T("Journal détaillé du serveur (chaque paquet)", "Detailed server log (every packet)");
pub const SET_SERVER_TRACE_HINT: T = T("Pour un rapport de bug : le journal grossit vite. Au prochain démarrage du serveur.", "For a bug report: the log grows fast. On the next server start.");
pub const SET_CLIENT_LOG: T = T("Niveau du journal d'ArchRust :", "ArchRust log level:");
pub const SET_CLIENT_LOG_HINT: T = T("debug : le plus détaillé. Au prochain lancement du jeu.", "debug: most detailed. On the next game start.");
pub const SET_SAVED: T = T("Réglages enregistrés.", "Settings saved.");
pub const SET_RESTART_NEEDED: T = T("Serveur en marche : les changements s'appliquent au prochain démarrage (Arrêter puis JOUER).", "Server running: changes apply on the next start (Stop, then PLAY).");

// ------------------------------------------------------------------ journaux
pub const LOG_TITLE: T = T("Journaux", "Logs");
pub const LOG_LAUNCHER: T = T("Launcher", "Launcher");
pub const LOG_SERVER: T = T("Serveur", "Server");
pub const LOG_CLIENT: T = T("Client", "Client");
pub const LOG_FILTER: T = T("Filtre :", "Filter:");
pub const LOG_PROBLEMS: T = T("Avertissements et erreurs seulement", "Warnings and errors only");
pub const LOG_OPEN_FILE: T = T("Ouvrir le fichier", "Open the file");
pub const LOG_CLEAR: T = T("Vider l'affichage", "Clear the view");
pub const LOG_CLEAR_HINT: T = T("Les fichiers ne sont pas touchés.", "Files are not touched.");
pub const LOG_CLIENT_EMPTY: T = T("Le journal d'ArchRust s'affiche ici pendant le jeu (Logs\\client).", "ArchRust's log shows here while playing (Logs\\client).");
pub const LOG_SERVER_EMPTY: T = T("Le journal d'ArchCore et des imports s'affiche ici (Logs\\server).", "The ArchCore and import log shows here (Logs\\server).");

// ------------------------------------------------------------------ rapport
pub const RPT_TITLE: T = T("Rapport de vérification", "Check report");
pub const RPT_MANIFEST_MISSING: T = T("Manifeste de fichiers absent (version de développement) : fichiers non vérifiés.", "File manifest missing (development build): files not checked.");
pub const RPT_FILES_OK: T = T("Fichiers du programme intacts", "Program files intact");
pub const RPT_MISSING: T = T("Fichiers manquants", "Missing files");
pub const RPT_ALTERED: T = T("Fichiers altérés", "Altered files");
pub const RPT_UNREPAIRABLE: T = T("Des fichiers du programme manquent ou sont altérés : réinstallez le package (UserData est conservé).", "Program files are missing or altered: reinstall the package (UserData is kept).");
pub const RPT_DATA: T = T("Données", "Data");
pub const RPT_CLIENT: T = T("Client ArchRust", "ArchRust client");
pub const RPT_CLIENT_INI: T = T("Tables du client lues (Server\\Data\\client-ini)", "Client tables read (Server\\Data\\client-ini)");
pub const RPT_SERVER_TABLES: T = T("Tables du serveur d'origine (facultatives)", "Original server tables (optional)");
pub const RPT_DB: T = T("Bases", "Databases");
pub const RPT_DB_OK: T = T("Bases complètes, monde importé.", "Databases complete, world imported.");
pub const RPT_DB_INCOMPLETE: T = T("Bases ou monde incomplets : utilisez Réparer.", "Databases or world incomplete: use Repair.");
pub const RPT_DB_UNKNOWN: T = T("Base arrêtée : non vérifiée (Réparer la démarre).", "Database stopped: not checked (Repair starts it).");
pub const RPT_REPAIR_HINT: T = T(
    "Réparer : recrée les bases manquantes et applique les migrations (sans toucher aux personnages), régénère les configurations, réimporte le monde s'il est vide.",
    "Repair: recreates missing databases and applies migrations (characters untouched), regenerates the configuration, reimports the world if it is empty.",
);
pub const RPT_NOTE_SCHEMA: T = T("schéma des bases à jour", "database schema up to date");
pub const RPT_NOTE_CONFIGS: T = T("configurations régénérées", "configuration regenerated");
pub const RPT_NOTE_WORLD: T = T("monde réimporté", "world reimported");

// ------------------------------------------------------------------ aide MJ
pub const HELP_TITLE: T = T("Aide du maître du jeu", "Game master help");
pub const HELP_INTRO: T = T(
    "Commandes à taper dans la discussion du jeu, avec un compte de niveau MJ suffisant (onglet Comptes). Les noms de personnages se mettent entre guillemets.",
    "Commands to type in the game chat, with an account of high enough GM level (Accounts tab). Character names go in double quotes.",
);
pub const HELP_GM_TITLE: T = T("Commandes MJ d'ArchCore", "ArchCore GM commands");
pub const HELP_CMD: T = T("Commande", "Command");
pub const HELP_EFFECT: T = T("Effet", "Effect");
pub const HELP_KEYS_TITLE: T = T("Raccourcis d'ArchRust", "ArchRust shortcuts");
pub const HELP_ADMIN_ADDON: T = T(
    "L'AddOn ArchCoreAdmin (github.com/BlaMacfly/ArchCoreAdmin) propose un bouton pour chaque commande : installez-le dans Packs et AddOns.",
    "The ArchCoreAdmin AddOn (github.com/BlaMacfly/ArchCoreAdmin) offers a button for every command: install it in Packs & AddOns.",
);

/// (commande, niveau, effet)
pub fn gm_commands() -> Vec<(&'static str, &'static str, &'static str)> {
    let fr = is_fr();
    let p = |a: &'static str, b: &'static str| if fr { a } else { b };
    vec![
        ("/additem <TID> [n]", "3", p("Ajoute un objet (TID de la base monde, table item_template) à votre inventaire", "Adds an item (TID from the world database, item_template table) to your inventory")),
        ("/create \"nom\" <TID> <n> 0", "3", p("Donne un objet à un personnage, même hors ligne", "Gives an item to a character, even offline")),
        ("/level <niveau> [\"nom\"]", "3", p("Fixe le niveau (1 à 120)", "Sets the level (1 to 120)")),
        ("/money <montant> [\"nom\"]", "3", p("Fixe l'or porté", "Sets the gold carried")),
        ("/speed <1-10>", "2", p("Multiplie votre vitesse de déplacement ; 1 la rétablit", "Multiplies your movement speed; 1 restores it")),
        ("/invincible [0|1]", "2", p("Les monstres ne vous blessent plus (1) ou de nouveau (0)", "Monsters no longer hurt you (1) or do again (0)")),
        ("/move x,z", "1", p("Téléportation aux coordonnées", "Teleport to coordinates")),
        ("/kill [\"nom\"]", "2", p("Met les PV à 0 (sans nom : sur soi)", "Sets HP to 0 (no name: yourself)")),
        ("/recast", "3", p("Remet à zéro les délais de relance de vos compétences", "Resets your skill cooldowns")),
        ("/questreset <TID>", "3", p("Efface une quête de votre journal", "Removes a quest from your journal")),
        ("/rename_char \"ancien\" \"nouveau\"", "2", p("Renomme un personnage", "Renames a character")),
        ("/SetExp <x>  /GetExp", "3 / 1", p("Taux d'expérience en mémoire (aussi SetGold, SetDrop…)", "In-memory experience rate (also SetGold, SetDrop…)")),
        ("/chatban \"nom\" <minutes>", "1", p("Interdit la discussion ; 0 la rétablit", "Bans chat; 0 lifts it")),
        ("/announce_restart <min> <s>", "3", p("Annonce un redémarrage ; 0 0 annule", "Announces a restart; 0 0 cancels")),
    ]
}

/// (touche, effet)
pub fn game_keys() -> Vec<(&'static str, &'static str)> {
    let fr = is_fr();
    let p = |a: &'static str, b: &'static str| if fr { a } else { b };
    vec![
        (p("Clic gauche au sol", "Left click on the ground"), p("Marcher", "Walk")),
        (p("Clic droit maintenu, molette", "Hold right click, wheel"), p("Tourner la caméra, zoom", "Rotate the camera, zoom")),
        (p("Clic droit sur un monstre, F", "Right click on a monster, F"), p("Attaquer (Tab : cibler)", "Attack (Tab: target)")),
        (p("Espace", "Space"), p("Ramasser le butin le plus proche", "Pick up the nearest loot")),
        ("I / C / K / L / G", p("Inventaire, personnage, compétences, quêtes, guilde", "Inventory, character, skills, quests, guild")),
        ("1 … =", p("Barre de raccourcis", "Action bar")),
        (p("Échap, /reglages", "Escape, /reglages"), p("Options du client (son, langue, packs)", "Client options (sound, language, packs)")),
        ("/groupe, /inviter nom", p("Groupe", "Party")),
        ("/P, /G, /N texte", p("Discussion de groupe, de guilde, normale", "Party, guild, normal chat")),
    ]
}

// ------------------------------------------------------------------ messages composés

pub fn port_busy(port: u16) -> String {
    crate::tf!("le port {port} est déjà utilisé par un autre programme (un autre serveur ArchCore ?)", "port {port} is already used by another program (another ArchCore server?)")
}

pub fn acc_exists(user: &str) -> String {
    crate::tf!("Le compte {user} existe déjà.", "Account {user} already exists.")
}

pub fn acc_created(user: &str, gm: u8) -> String {
    if gm > 0 {
        crate::tf!("Compte {user} créé (niveau MJ {gm}). Connectez-vous avec dans ArchRust.", "Account {user} created (GM level {gm}). Log in with it in ArchRust.")
    } else {
        crate::tf!("Compte {user} créé. Connectez-vous avec dans ArchRust.", "Account {user} created. Log in with it in ArchRust.")
    }
}

pub fn acc_gm_set(user: &str, gm: u8) -> String {
    crate::tf!("Niveau MJ de {user} : {gm} (pris en compte à sa prochaine connexion).", "GM level of {user}: {gm} (applies at its next login).")
}

pub fn acc_deleted(user: &str, n: u64) -> String {
    crate::tf!("Compte {user} supprimé ({n} personnage(s)).", "Account {user} deleted ({n} character(s)).")
}

pub fn world_summary(npcs: u64, monsters: u64, items: u64) -> String {
    if monsters == 0 {
        crate::tf!("Importé : {npcs} PNJ, {items} objets, aucun monstre", "Imported: {npcs} NPCs, {items} items, no monsters")
    } else {
        crate::tf!("Importé : {npcs} PNJ, {monsters} apparitions de monstres, {items} objets", "Imported: {npcs} NPCs, {monsters} monster spawns, {items} items")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_switch_with_the_language() {
        set_lang(Lang::En);
        assert_eq!(BTN_PLAY.s(), "PLAY");
        assert_eq!(acc_deleted("bob", 2), "Account bob deleted (2 character(s)).");
        set_lang(Lang::Fr);
        assert_eq!(BTN_PLAY.s(), "JOUER");
        let n = 3;
        assert_eq!(crate::tf!("{n} lignes", "{n} lines"), "3 lignes");
        assert_eq!(Lang::parse("EN"), Some(Lang::En));
    }
}
