//! Client ArchRust : détection, configuration de la connexion (realmlist.wtf) et de la langue, lancement et suivi.
//!
//! Le dossier du client contient `ArchRust.exe` et, à côté, les données du jeu ArchLord (`character/`, `ini/`,
//! `world/`…) que le joueur fournit. Rien de ce dossier n'est jamais redistribué.

use crate::logs::{self, Line, Source};
use crate::settings::Settings;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;

/// Dossiers de données attendus à côté d'ArchRust.exe.
pub const DATA_DIRS: [&str; 7] = ["character", "ini", "world", "texture", "object", "effect", "sound"];

#[derive(Clone, Debug, Default)]
pub struct ClientInfo {
    pub dir: PathBuf,
    pub exe_present: bool,
    /// Données du jeu manquantes (dossiers ou fichier de référence)
    pub missing: Vec<String>,
    /// Taille d'ArchRust.exe et date de modification, pour le diagnostic
    pub exe_info: String,
}

impl ClientInfo {
    pub fn is_ok(&self) -> bool {
        self.exe_present && self.missing.is_empty()
    }
}

pub fn inspect(dir: &Path) -> ClientInfo {
    let exe = dir.join("ArchRust.exe");
    let exe_info = std::fs::metadata(&exe)
        .map(|m| {
            let date = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            format!("{} ({} UTC)", crate::util::human_size(m.len()), crate::util::date_utc(date))
        })
        .unwrap_or_default();
    let mut missing: Vec<String> = DATA_DIRS.iter().filter(|d| !dir.join(d).is_dir()).map(|d| format!("{d}/")).collect();
    if missing.is_empty() && !dir.join("character").join("Reference.Dat").is_file() {
        missing.push("character/Reference.Dat".into());
    }
    ClientInfo { dir: dir.to_path_buf(), exe_present: exe.is_file(), missing, exe_info }
}

/// Dossiers ArchRust trouvés autour de l'installation : `Client/`, ses voisins et leurs sous-dossiers
/// (`…/Client ArchRust`, `…/client/ArchLord/Game`), sur trois niveaux au plus.
pub fn discover(root: &Path) -> Vec<ClientInfo> {
    let mut out: Vec<ClientInfo> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let mut visit = |d: &Path| {
        if !d.join("ArchRust.exe").is_file() {
            return;
        }
        let i = inspect(d);
        if i.is_ok() && seen.insert(d.to_string_lossy().to_lowercase()) {
            out.push(i);
        }
    };
    fn walk(d: &Path, depth: usize, visit: &mut dyn FnMut(&Path)) {
        visit(d);
        if depth == 0 {
            return;
        }
        if let Ok(rd) = std::fs::read_dir(d) {
            for e in rd.flatten() {
                let p = e.path();
                let name = e.file_name().to_string_lossy().to_lowercase();
                // Dossiers de données du jeu : inutile d'y descendre (des milliers de fichiers).
                if p.is_dir() && !DATA_DIRS.contains(&name.as_str()) && !name.starts_with('.') && name != "target" && name != "userdata" {
                    walk(&p, depth - 1, visit);
                }
            }
        }
    }
    walk(&root.join("Client"), 2, &mut visit);
    if let Some(parent) = root.parent() {
        walk(parent, 3, &mut visit);
    }
    out
}

/// Prépare ArchRust : adresse du serveur (realmlist.wtf, lu à côté de l'exécutable) et langue
/// (clé `language` de saved-variables/options.toml, le reste du fichier est conservé).
pub fn configure(info: &ClientInfo, settings: &Settings) -> Result<(), String> {
    let realmlist = info.dir.join("realmlist.wtf");
    let text = format!(
        "# Serveur de connexion ArchRust (ecrit par HomeRPG ArchLord).\n# Forme : set realmlist <hote>[:<port>]   (port 11002 par defaut)\nset realmlist 127.0.0.1:{}\n",
        settings.login_port
    );
    std::fs::write(&realmlist, text).map_err(|e| format!("{} : {e}", realmlist.display()))?;
    let opts = info.dir.join("saved-variables").join("options.toml");
    let code = settings.game_language.code();
    let current = std::fs::read_to_string(&opts).unwrap_or_default();
    let mut lines: Vec<String> = current.lines().map(str::to_string).collect();
    match lines.iter().position(|l| l.trim_start().starts_with("language") && l.contains('=')) {
        Some(i) => lines[i] = format!("language = \"{code}\""),
        None => {
            // Clé de premier niveau : avant la première table.
            let at = lines.iter().position(|l| l.trim_start().starts_with('[')).unwrap_or(lines.len());
            lines.insert(at, format!("language = \"{code}\""));
        }
    }
    if let Some(parent) = opts.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&opts, lines.join("\n") + "\n").map_err(|e| format!("{} : {e}", opts.display()))
}

/// Lance ArchRust : sorties dans `Logs/client/archrust-<date>.log` et dans l'onglet Journaux (Client).
/// Le niveau de détail vient du réglage (RUST_LOG).
pub fn launch(info: &ClientInfo, settings: &Settings, client_logs: &Path, sink: Sender<Line>) -> Result<(Child, Option<PathBuf>), String> {
    let level = match settings.client_log_level.as_str() {
        l @ ("error" | "warn" | "info" | "debug" | "trace") => l.to_string(),
        _ => "info".to_string(),
    };
    let mut child = Command::new(info.dir.join("ArchRust.exe"))
        .arg("play")
        .current_dir(&info.dir)
        .env("RUST_LOG", level)
        .creation_flags(crate::db::CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| crate::util::spawn_error("ArchRust.exe", &e))?;
    let (path, file) = match logs::new_file(client_logs, "archrust") {
        Some((p, f)) => (Some(p), Some(f)),
        None => (None, None),
    };
    logs::pipe(&mut child, Source::Client, file, sink);
    Ok((child, path))
}
