//! Disposition des dossiers et ports du projet. Tout est relatif à la racine d'installation,
//! détectée depuis l'emplacement de l'exécutable (ou forcée par la variable HOMERPG_ROOT).

use std::path::{Path, PathBuf};

/// MariaDB portable du projet : port distinct du 3306 d'une MariaDB/MySQL déjà installée sur le PC.
pub const MARIADB_PORT: u16 = 3316;
/// Ports d'ArchCore par défaut (ceux du client ArchLord) ; modifiables dans Paramètres.
pub const DEFAULT_LOGIN_PORT: u16 = 11002;
pub const DEFAULT_WORLD_PORT: u16 = 11008;
pub const WORLD_NAME: &str = "HomeRPG";

/// Bases d'ArchCore, dans l'ordre de création.
pub const DB_AUTH: &str = "archcore_auth";
pub const DB_CHARACTERS: &str = "archcore_characters";
pub const DB_WORLD: &str = "archcore_world";
pub const DB_LOGS: &str = "archcore_logs";
pub const ALL_DBS: [&str; 4] = [DB_AUTH, DB_CHARACTERS, DB_WORLD, DB_LOGS];

#[derive(Clone, Debug)]
pub struct Layout {
    pub root: PathBuf,
}

impl Layout {
    pub fn detect() -> Layout {
        if let Ok(r) = std::env::var("HOMERPG_ROOT") {
            return Layout { root: PathBuf::from(r) };
        }
        let exe = std::env::current_exe().ok();
        let mut dir = exe.as_deref().and_then(Path::parent).map(Path::to_path_buf);
        while let Some(d) = dir {
            if d.join("Server").is_dir() && d.join("Tools").is_dir() {
                return Layout { root: d };
            }
            dir = d.parent().map(Path::to_path_buf);
        }
        Layout { root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")) }
    }

    // Programme (remplaçable à la mise à jour)
    /// Racine d'ArchCore (ARCHCORE_ROOT) : archcore.exe, sql/, content/, data/i18n/, config/*.dist
    pub fn archcore_dir(&self) -> PathBuf { self.root.join("Server").join("ArchCore") }
    pub fn archcore_exe(&self) -> PathBuf { self.archcore_dir().join("archcore.exe") }
    /// Configurations générées à chaque démarrage (ARCHCORE_CONF_DIR) et my.ini
    pub fn server_config(&self) -> PathBuf { self.root.join("Server").join("Config") }
    /// Données dérivées du client (tables déchiffrées) : jamais redistribuées
    pub fn server_data(&self) -> PathBuf { self.root.join("Server").join("Data") }
    pub fn client_ini(&self) -> PathBuf { self.server_data().join("client-ini") }
    /// Emplacement par défaut des tables du serveur d'origine (apparitions de monstres, butin, carte)
    pub fn default_server_tables(&self) -> PathBuf { self.server_data().join("server-ini") }
    pub fn mariadb_dir(&self) -> PathBuf { self.root.join("Server").join("Database").join("mariadb") }
    pub fn mariadb_bin(&self) -> PathBuf { self.mariadb_dir().join("bin") }
    pub fn heidisql_exe(&self) -> PathBuf { self.root.join("Tools").join("HeidiSQL").join("heidisql.exe") }
    pub fn default_client_dir(&self) -> PathBuf { self.root.join("Client") }

    // Données personnelles (jamais écrasées par une mise à jour)
    pub fn user_data(&self) -> PathBuf { self.root.join("UserData") }
    pub fn db_data(&self) -> PathBuf { self.user_data().join("Database") }
    pub fn credentials_file(&self) -> PathBuf { self.user_data().join("db-credentials.json") }
    pub fn settings_file(&self) -> PathBuf { self.user_data().join("settings.json") }
    pub fn backups(&self) -> PathBuf { self.user_data().join("Sauvegardes") }

    // Journaux
    pub fn logs(&self) -> PathBuf { self.root.join("Logs") }
    pub fn server_logs(&self) -> PathBuf { self.logs().join("server") }
    pub fn launcher_log(&self) -> PathBuf { self.logs().join("launcher.log") }

    /// Les composants gérés sont-ils présents ? (état NON INSTALLÉ sinon)
    pub fn components_present(&self) -> Vec<(String, bool)> {
        let a = self.archcore_dir();
        let checks = [
            ("archcore.exe", self.archcore_exe()),
            ("auth.conf.dist", a.join("config").join("auth.conf.dist")),
            ("world.conf.dist", a.join("config").join("world.conf.dist")),
            ("sql/001_auth.sql", a.join("sql").join("001_auth.sql")),
            ("data/i18n", a.join("data").join("i18n").join("npc-titles-fr.json")),
            ("mariadbd.exe", self.mariadb_bin().join("mariadbd.exe")),
        ];
        checks.iter().map(|(n, p)| (n.to_string(), p.is_file())).collect()
    }

    /// Le support est-il inscriptible ? (faux sur un DVD ou un dossier protégé : il faut copier sur le disque)
    pub fn is_writable(&self) -> bool {
        let probe_dir = self.user_data();
        if std::fs::create_dir_all(&probe_dir).is_err() {
            return false;
        }
        let probe = probe_dir.join(".ecriture-test");
        let ok = std::fs::write(&probe, b"ok").is_ok();
        let _ = std::fs::remove_file(&probe);
        ok
    }

    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Chemin écrit dans les .conf d'ArchCore (séparateurs « / », entre guillemets).
    pub fn conf_path(p: &Path) -> String {
        format!("\"{}\"", p.to_string_lossy().replace('\\', "/"))
    }
}
