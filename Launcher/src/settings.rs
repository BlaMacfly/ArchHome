//! Réglages de l'utilisateur (UserData/settings.json).

use crate::i18n::Lang;
use crate::paths::{DEFAULT_LOGIN_PORT, DEFAULT_WORLD_PORT};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    /// Langue de l'interface du launcher (None = langue de Windows)
    pub ui_language: Option<Lang>,
    /// Langue du jeu : textes du serveur (World.Language) et langue d'ArchRust (options.toml)
    pub game_language: Lang,
    /// Dossier du client ArchRust avec les données du jeu (vide = Client/ à la racine)
    pub client_dir: String,
    /// Dossier des données du jeu ArchLord (Game), s'il n'est pas à côté d'ArchRust.exe (vide = recherche
    /// automatique : à côté de l'exécutable, puis aux emplacements par défaut d'ArchRust)
    pub game_dir: String,
    /// Dossier des tables du serveur d'origine (vide = Server/Data/server-ini)
    pub server_tables_dir: String,
    pub login_port: u16,
    pub world_port: u16,
    /// Surcharges de world.conf (clé -> valeur), limitées au catalogue `conf_catalog` (onglet Monde).
    pub conf: std::collections::BTreeMap<String, String>,
    /// Sauvegarde automatique des personnages à chaque fermeture du jeu
    pub auto_backup: bool,
    /// Nombre de sauvegardes automatiques conservées (les plus anciennes sont supprimées)
    pub auto_backup_keep: u32,
    /// Journal détaillé du serveur : chaque paquet reçu et envoyé (ARCHCORE_TRACE=1). Volumineux.
    pub server_trace: bool,
    /// Niveau du journal d'ArchRust (RUST_LOG) : error, warn, info, debug
    pub client_log_level: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            ui_language: None,
            game_language: crate::i18n::system_lang(),
            client_dir: String::new(),
            game_dir: String::new(),
            server_tables_dir: String::new(),
            login_port: DEFAULT_LOGIN_PORT,
            world_port: DEFAULT_WORLD_PORT,
            conf: std::collections::BTreeMap::new(),
            auto_backup: true,
            auto_backup_keep: 10,
            server_trace: false,
            client_log_level: "info".into(),
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Settings {
        std::fs::read_to_string(path).ok().and_then(|t| serde_json::from_str(t.trim_start_matches('\u{feff}')).ok()).unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self).unwrap()).map_err(|e| e.to_string())
    }

    /// Langue effective de l'interface.
    pub fn ui_lang(&self) -> Lang {
        self.ui_language.unwrap_or_else(crate::i18n::system_lang)
    }
}
