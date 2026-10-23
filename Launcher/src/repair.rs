//! Vérification et diagnostic : manifeste de fichiers gérés (Build/files-manifest.json, produit au packaging),
//! données dérivées du client, tables du serveur d'origine, état des bases, export de diagnostic (journaux du
//! launcher, du serveur et du client, configurations avec mots de passe masqués).

use crate::paths::Layout;
use crate::util;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FilesManifest {
    pub generated: String,
    pub files: Vec<FileEntry>,
}

#[derive(Clone, Debug, Default)]
pub struct VerifyReport {
    pub manifest_present: bool,
    pub checked: usize,
    pub missing: Vec<String>,
    pub altered: Vec<String>,
    /// (élément, présent)
    pub data: Vec<(String, bool)>,
    /// Bases complètes et monde importé (None : base injoignable)
    pub db_ok: Option<bool>,
    pub world: Option<crate::db::WorldStatus>,
    pub notes: Vec<String>,
}

impl VerifyReport {
    pub fn files_ok(&self) -> bool {
        self.missing.is_empty() && self.altered.is_empty()
    }
}

pub fn manifest_path(layout: &Layout) -> PathBuf {
    layout.root.join("Build").join("files-manifest.json")
}

pub fn load_manifest(layout: &Layout) -> Option<FilesManifest> {
    let txt = std::fs::read_to_string(manifest_path(layout)).ok()?;
    serde_json::from_str(txt.trim_start_matches('\u{feff}')).ok()
}

/// Compare les fichiers gérés au manifeste (taille puis SHA-256).
pub fn verify_files(layout: &Layout, manifest: &FilesManifest, progress: &dyn Fn(usize, usize)) -> (Vec<String>, Vec<String>) {
    let mut missing = Vec::new();
    let mut altered = Vec::new();
    let total = manifest.files.len();
    for (i, f) in manifest.files.iter().enumerate() {
        progress(i + 1, total);
        let p = layout.root.join(&f.path);
        match std::fs::metadata(&p) {
            Err(_) => missing.push(f.path.clone()),
            Ok(m) => {
                if m.len() != f.size || util::sha256_file(&p).map(|h| h != f.sha256).unwrap_or(true) {
                    altered.push(f.path.clone());
                }
            }
        }
    }
    (missing, altered)
}

/// Tables du serveur d'origine présentes dans `dir` (au minimum les apparitions de monstres).
pub fn server_tables_present(dir: &Path) -> bool {
    dir.join("spawndatatable.txt").is_file() && dir.join("spawninstancetable.txt").is_file()
}

/// Données dérivées : tables du client déchiffrées (le client 64 bits en compte plus de 200).
pub fn client_ini_present(layout: &Layout) -> bool {
    util::dir_file_count(&layout.client_ini()) >= 50
}

/// Diagnostic : zip dans UserData avec les journaux récents, les configurations (mots de passe masqués) et l'état.
pub fn export_diagnostic(layout: &Layout, settings: &crate::settings::Settings, state: &str) -> Result<PathBuf, String> {
    std::fs::create_dir_all(layout.user_data()).map_err(|e| e.to_string())?;
    let out = layout.user_data().join(format!("diagnostic-{}.zip", util::datetime_compact()));
    let f = std::fs::File::create(&out).map_err(|e| e.to_string())?;
    let mut z = zip::ZipWriter::new(f);
    let opts = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated).large_file(true);
    let mut add = |name: &str, data: &[u8]| -> Result<(), String> {
        z.start_file(name, opts).map_err(|e| e.to_string())?;
        z.write_all(data).map_err(|e| e.to_string())
    };
    // Journaux : launcher, MariaDB, trois derniers du serveur, des outils et du client (fin des fichiers seulement).
    let tail_file = |p: &Path| crate::logs::tail(p, 20_000).join("\n").into_bytes();
    add("journaux/launcher.log", &tail_file(&layout.launcher_log()))?;
    add("journaux/mariadb-error.log", &tail_file(&layout.logs().join("mariadb-error.log")))?;
    for (dir, prefix, folder) in [(layout.server_logs(), "archcore", "serveur"), (layout.server_logs(), "outils", "serveur"), (layout.logs().join("client"), "archrust", "client")] {
        for p in crate::logs::files(&dir, prefix).into_iter().take(3) {
            let name = p.file_name().unwrap_or_default().to_string_lossy().to_string();
            add(&format!("journaux/{folder}/{name}"), &tail_file(&p))?;
        }
    }
    // Configurations générées, mots de passe masqués.
    for name in ["auth.conf", "world.conf", "my.ini"] {
        if let Ok(t) = std::fs::read_to_string(layout.server_config().join(name)) {
            let masked: Vec<String> = t
                .lines()
                .map(|l| if l.trim_start().to_ascii_lowercase().starts_with("database.password") { "Database.Password = \"********\"".to_string() } else { l.to_string() })
                .collect();
            add(&format!("config/{name}"), masked.join("\n").as_bytes())?;
        }
    }
    if let Ok(t) = std::fs::read(layout.settings_file()) {
        add("config/settings.json", &t)?;
    }
    let info = crate::client::from_settings(layout, settings);
    let mut state_txt = format!(
        "HomeRPG ArchLord {}\nracine : {}\nétat : {state}\nclient : {} (ArchRust.exe {}, {}), données du jeu : {} (manquant : {})\n",
        Layout::version(),
        layout.root.display(),
        info.dir.display(),
        if info.exe_present { "présent" } else { "absent" },
        info.exe_info,
        info.game_dir.display(),
        info.missing.join(", ")
    );
    for (n, ok) in layout.components_present() {
        state_txt.push_str(&format!("composant {n} : {}\n", if ok { "présent" } else { "ABSENT" }));
    }
    if let Ok(v) = std::fs::read_to_string(layout.archcore_dir().join("VERSION.txt")) {
        state_txt.push_str(&format!("ArchCore : {}\n", v.trim()));
    }
    add("etat.txt", state_txt.as_bytes())?;
    z.finish().map_err(|e| e.to_string())?;
    Ok(out)
}
