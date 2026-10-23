//! Client ArchRust : détection, configuration de la connexion (realmlist.wtf) et de la langue, lancement et suivi.
//!
//! Le dossier du client contient `ArchRust.exe` ; les données du jeu ArchLord (`character/`, `ini/`, `world/`…),
//! fournies par le joueur, sont à côté de lui ou dans un dossier à part (réglage « Données du jeu », passé à
//! ArchRust par `--game`). Rien de ces dossiers n'est jamais redistribué.

use crate::logs::{self, Line, Source};
use crate::paths::Layout;
use crate::settings::Settings;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;

/// Dossiers de données attendus dans le dossier Game d'ArchLord.
pub const DATA_DIRS: [&str; 7] = ["character", "ini", "world", "texture", "object", "effect", "sound"];

#[derive(Clone, Debug, Default)]
pub struct ClientInfo {
    /// Dossier d'ArchRust.exe (realmlist.wtf, addons, patches, saved-variables)
    pub dir: PathBuf,
    /// Dossier des données du jeu (Game d'ArchLord) : `dir` lui-même le plus souvent
    pub game_dir: PathBuf,
    pub exe_present: bool,
    /// Données du jeu manquantes dans `game_dir` (dossiers ou fichier de référence)
    pub missing: Vec<String>,
    /// Taille d'ArchRust.exe et date de modification, pour le diagnostic
    pub exe_info: String,
    /// Dossier des sources d'ArchRust (Cargo.toml, crates/) : pas un client prêt à jouer
    pub source_tree: bool,
}

impl ClientInfo {
    pub fn is_ok(&self) -> bool {
        self.exe_present && self.missing.is_empty()
    }
    /// Données du jeu ailleurs qu'à côté d'ArchRust.exe (lancement avec `--game`).
    pub fn separate_data(&self) -> bool {
        !same_dir(&self.dir, &self.game_dir)
    }
}

/// Données du jeu absentes de `game` (vide : dossier Game complet).
pub fn data_missing(game: &Path) -> Vec<String> {
    let mut missing: Vec<String> = DATA_DIRS.iter().filter(|d| !game.join(d).is_dir()).map(|d| format!("{d}/")).collect();
    if missing.is_empty() && !game.join("character").join("Reference.Dat").is_file() {
        missing.push("character/Reference.Dat".into());
    }
    missing
}

/// Client du dossier `dir`. Données du jeu : `game` s'il est donné, sinon à côté d'ArchRust.exe, sinon aux
/// emplacements qu'ArchRust essaie lui-même (`assets-archonia/Game`, `assets-archonia`, `../client/ArchLord/Game`).
pub fn inspect(dir: &Path, game: Option<&Path>) -> ClientInfo {
    let exe = dir.join("ArchRust.exe");
    let exe_info = std::fs::metadata(&exe)
        .map(|m| {
            let date = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs()).unwrap_or(0);
            format!("{} ({} UTC)", crate::util::human_size(m.len()), crate::util::date_utc(date))
        })
        .unwrap_or_default();
    let game_dir = match game {
        Some(g) => g.to_path_buf(),
        None => {
            let mut candidates = vec![dir.to_path_buf(), dir.join("assets-archonia").join("Game"), dir.join("assets-archonia")];
            if let Some(parent) = dir.parent() {
                candidates.push(parent.join("client").join("ArchLord").join("Game"));
            }
            candidates.into_iter().find(|c| data_missing(c).is_empty()).unwrap_or_else(|| dir.to_path_buf())
        }
    };
    let missing = data_missing(&game_dir);
    let source_tree = dir.join("Cargo.toml").is_file() && dir.join("crates").is_dir();
    ClientInfo { dir: dir.to_path_buf(), game_dir, exe_present: exe.is_file(), missing, exe_info, source_tree }
}

/// Dossier d'ArchRust.exe selon les réglages (vide = Client/ à la racine).
pub fn exe_dir(layout: &Layout, settings: &Settings) -> PathBuf {
    if settings.client_dir.trim().is_empty() { layout.default_client_dir() } else { PathBuf::from(settings.client_dir.trim()) }
}

/// Client selon les réglages : dossier d'ArchRust.exe et, s'il est indiqué, dossier des données du jeu.
pub fn from_settings(layout: &Layout, settings: &Settings) -> ClientInfo {
    let game = settings.game_dir.trim();
    inspect(&exe_dir(layout, settings), (!game.is_empty()).then(|| Path::new(game)))
}

/// Même dossier, quelle que soit l'écriture (« C:/Jeux/x », « c:\jeux\x\ »).
pub fn same_dir(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| p.to_string_lossy().replace('/', "\\").trim_end_matches('\\').to_lowercase();
    norm(a) == norm(b)
}

/// Dossiers non parcourus : système, données du jeu (des milliers de fichiers), dépôts et compilations.
const SKIP_DIRS: [&str; 14] = ["windows", "$recycle.bin", "system volume information", "programdata", "appdata", "recovery", "perflogs", "node_modules", "target", "userdata", "winsxs", "msocache", "$windows.~bt", "$windows.~ws"];

/// Dossiers parcourus au plus par une recherche (garde-fou sur les disques très remplis).
const MAX_VISITED: usize = 60_000;

/// Cherche les dossiers contenant ArchRust.exe : autour de l'installation (dossier `Client`, dossiers parents
/// et voisins), dans le Bureau, les Documents, les Téléchargements et le dossier de l'utilisateur, puis à la
/// racine des disques fixes (`C:\Jeux\…`, `D:\ArchLord\…`, Program Files). Les clients aux données incomplètes
/// sont rendus aussi, pour l'expliquer à l'utilisateur. Peut prendre quelques secondes : à lancer hors du fil
/// de l'interface.
pub fn discover(root: &Path) -> Vec<ClientInfo> {
    let mut starts: Vec<(PathBuf, usize)> = vec![(root.join("Client"), 2)];
    let mut anc = root.parent();
    for depth in [3, 3] {
        if let Some(a) = anc {
            starts.push((a.to_path_buf(), depth));
            anc = a.parent();
        }
    }
    if let Some(home) = std::env::var_os("USERPROFILE").map(PathBuf::from) {
        for (sub, depth) in [("Desktop", 4), ("Bureau", 4), ("OneDrive\\Desktop", 4), ("OneDrive\\Bureau", 4), ("Documents", 3), ("Downloads", 3), ("", 2)] {
            starts.push((if sub.is_empty() { home.clone() } else { home.join(sub) }, depth));
        }
    }
    for drive in fixed_drives() {
        starts.push((drive, 3));
    }

    let mut out: Vec<ClientInfo> = Vec::new();
    let mut visited: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    fn walk(d: &Path, depth: usize, visited: &mut std::collections::HashMap<String, usize>, out: &mut Vec<ClientInfo>) {
        // Dossier déjà parcouru au moins aussi profondément : rien de plus à y trouver.
        let key = d.to_string_lossy().replace('/', "\\").trim_end_matches('\\').to_lowercase();
        let first = match visited.get(&key) {
            Some(&done) if done >= depth => return,
            Some(_) => false,
            None => true,
        };
        if first && visited.len() >= MAX_VISITED {
            return;
        }
        visited.insert(key, depth);
        if first && d.join("ArchRust.exe").is_file() {
            out.push(inspect(d, None));
        }
        if depth == 0 {
            return;
        }
        let Ok(rd) = std::fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let Ok(ft) = e.file_type() else { continue };
            // Pas de lien symbolique ni de jonction (boucles, « Mes documents » → Documents…).
            if !ft.is_dir() || ft.is_symlink() {
                continue;
            }
            let name = e.file_name().to_string_lossy().to_lowercase();
            if DATA_DIRS.contains(&name.as_str()) || SKIP_DIRS.contains(&name.as_str()) || name.starts_with('.') {
                continue;
            }
            walk(&e.path(), depth - 1, visited, out);
        }
    }
    for (dir, depth) in starts {
        if dir.is_dir() {
            walk(&dir, depth, &mut visited, &mut out);
        }
    }
    // Clients complets d'abord.
    out.sort_by_key(|c| !c.is_ok());
    out
}

/// Racines des disques fixes (C:\, D:\…), sans les lecteurs réseau, amovibles ou optiques.
fn fixed_drives() -> Vec<PathBuf> {
    use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};
    const DRIVE_FIXED: u32 = 3;
    let mask = unsafe { GetLogicalDrives() };
    (0..26u8)
        .filter(|i| mask & (1 << i) != 0)
        .map(|i| format!("{}:\\", (b'A' + i) as char))
        .filter(|root| {
            let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
            unsafe { GetDriveTypeW(wide.as_ptr()) == DRIVE_FIXED }
        })
        .map(PathBuf::from)
        .collect()
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
    let mut cmd = Command::new(info.dir.join("ArchRust.exe"));
    cmd.arg("play");
    // Données du jeu dans un autre dossier : indiquées explicitement (sinon ArchRust les cherche à côté de lui).
    if info.separate_data() {
        cmd.arg("--game").arg(&info.game_dir);
    }
    let mut child = cmd
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

#[cfg(test)]
mod tests {
    /// Recherche réelle sur ce PC : `HOMERPG_TEST_ROOT=<dossier d'installation> cargo test -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn discover_from_install_root() {
        let root = std::env::var("HOMERPG_TEST_ROOT").map(std::path::PathBuf::from).unwrap_or_else(|_| std::env::current_dir().unwrap());
        let t = std::time::Instant::now();
        let found = super::discover(&root);
        println!("{} client(s) en {:?} depuis {}", found.len(), t.elapsed(), root.display());
        for c in &found {
            println!("  {} {} {:?}", if c.is_ok() { "OK " } else { "-- " }, c.dir.display(), c.missing);
        }
    }

    #[test]
    fn same_dir_ignores_separators_and_case() {
        assert!(super::same_dir(std::path::Path::new("C:/Jeux/ArchRust"), std::path::Path::new(r"c:\jeux\archrust\")));
        assert!(!super::same_dir(std::path::Path::new("C:/Jeux/A"), std::path::Path::new("C:/Jeux/B")));
    }
}
