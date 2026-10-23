//! Packs de ressources d'ArchRust (graphismes, sons) : le pack officiel `free-assets/` et ceux de la communauté
//! dans `patches/` (dossiers ou archives `.ahp`, chacun avec son `patch.toml`). Activation enregistrée dans
//! `saved-variables/packs.toml`, le fichier même qu'utilise la fenêtre Options du jeu : un changement fait ici se
//! voit dans le jeu et inversement. Pris en compte au prochain lancement d'ArchRust.

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct PackEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub priority: i64,
    pub path: PathBuf,
    /// Pack officiel `free-assets/` : toujours chargé, ni désactivable ni supprimable ici
    pub official: bool,
    pub archive: bool,
    pub enabled: bool,
    pub size: u64,
    /// Manifeste illisible ou absent
    pub error: Option<String>,
}

fn yes() -> bool {
    true
}

/// Même format que les préférences d'ArchRust (`packs.toml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PackChoice {
    id: String,
    #[serde(default = "yes")]
    enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    priority: Option<i32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PackSettings {
    #[serde(default, rename = "pack")]
    packs: Vec<PackChoice>,
}

pub fn patches_dir(client: &Path) -> PathBuf {
    client.join("patches")
}

fn prefs_path(client: &Path) -> PathBuf {
    client.join("saved-variables").join("packs.toml")
}

fn load_prefs(client: &Path) -> PackSettings {
    std::fs::read_to_string(prefs_path(client)).ok().and_then(|t| toml::from_str(&t).ok()).unwrap_or_default()
}

fn save_prefs(client: &Path, prefs: &PackSettings) -> Result<(), String> {
    let p = prefs_path(client);
    if let Some(dir) = p.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let text = toml::to_string_pretty(prefs).map_err(|e| e.to_string())?;
    std::fs::write(&p, format!("# Packs de ressources d'ArchRust : activés ou non, priorité (supprimer ce fichier rétablit la configuration par défaut).\n{text}")).map_err(|e| format!("{} : {e}", p.display()))
}

/// Lit le `[patch]` d'un manifeste.
fn parse_manifest(text: &str, e: &mut PackEntry) {
    match toml::from_str::<toml::Value>(text) {
        Ok(v) => {
            let p = v.get("patch");
            let s = |k: &str| p.and_then(|p| p.get(k)).and_then(|x| x.as_str()).unwrap_or("").to_string();
            e.id = s("id");
            e.name = s("name");
            e.version = s("version");
            e.author = s("author");
            e.description = s("description");
            e.priority = p.and_then(|p| p.get("priority")).and_then(|x| x.as_integer()).unwrap_or(0);
            if e.id.is_empty() {
                e.error = Some("patch.toml : [patch] id ?".into());
            }
        }
        Err(err) => e.error = Some(format!("patch.toml : {err}")),
    }
}

fn dir_size(p: &Path) -> u64 {
    let mut total = 0;
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let path = e.path();
            if path.is_dir() {
                total += dir_size(&path);
            } else {
                total += e.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    total
}

/// `patch.toml` d'une archive `.ahp` (à la racine).
fn archive_manifest(path: &Path) -> Result<String, String> {
    let f = std::fs::File::open(path).map_err(|e| e.to_string())?;
    let mut z = zip::ZipArchive::new(f).map_err(|e| e.to_string())?;
    let mut entry = z.by_name("patch.toml").map_err(|_| "patch.toml ?".to_string())?;
    let mut s = String::new();
    entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
    Ok(s)
}

fn read_entry(path: &Path, official: bool) -> Option<PackEntry> {
    let mut e = PackEntry { path: path.to_path_buf(), official, enabled: true, ..Default::default() };
    let file_name = path.file_name()?.to_string_lossy().to_string();
    if path.is_dir() {
        let toml = path.join("patch.toml");
        if !toml.is_file() {
            return None;
        }
        match std::fs::read_to_string(&toml) {
            Ok(t) => parse_manifest(&t, &mut e),
            Err(err) => e.error = Some(err.to_string()),
        }
        e.size = dir_size(path);
    } else if file_name.to_ascii_lowercase().ends_with(".ahp") {
        e.archive = true;
        match archive_manifest(path) {
            Ok(t) => parse_manifest(&t, &mut e),
            Err(err) => e.error = Some(err),
        }
        e.size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    } else {
        return None;
    }
    if e.id.is_empty() {
        e.id = file_name.clone();
    }
    if e.name.is_empty() {
        e.name = file_name;
    }
    Some(e)
}

/// Pack officiel puis packs de la communauté, du plus prioritaire au moins prioritaire.
pub fn list(client: &Path) -> Vec<PackEntry> {
    let prefs = load_prefs(client);
    let mut community: Vec<PackEntry> = std::fs::read_dir(patches_dir(client))
        .map(|rd| rd.flatten().filter_map(|e| read_entry(&e.path(), false)).collect())
        .unwrap_or_default();
    for e in community.iter_mut() {
        if let Some(c) = prefs.packs.iter().find(|c| c.id.eq_ignore_ascii_case(&e.id)) {
            e.enabled = c.enabled;
            if let Some(p) = c.priority {
                e.priority = p as i64;
            }
        }
    }
    community.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.id.cmp(&b.id)));
    let mut out = Vec::new();
    if let Some(official) = read_entry(&client.join("free-assets"), true) {
        out.push(official);
    }
    out.extend(community);
    out
}

pub fn set_enabled(client: &Path, id: &str, enabled: bool) -> Result<(), String> {
    let mut prefs = load_prefs(client);
    match prefs.packs.iter_mut().find(|c| c.id.eq_ignore_ascii_case(id)) {
        Some(c) => c.enabled = enabled,
        None => prefs.packs.push(PackChoice { id: id.to_string(), enabled, priority: None }),
    }
    save_prefs(client, &prefs)
}

/// Ajoute un pack : archive `.ahp` copiée dans `patches/`, ou dossier contenant `patch.toml` copié tel quel.
pub fn add(src: &Path, client: &Path) -> Result<String, String> {
    let dir = patches_dir(client);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = src.file_name().ok_or("?")?.to_string_lossy().to_string();
    let dest = dir.join(&name);
    if dest.exists() {
        return Err(crate::tf!("{name} existe déjà dans patches/", "{name} already exists in patches/"));
    }
    if src.is_dir() {
        if !src.join("patch.toml").is_file() {
            return Err(crate::i18n::PACK_NO_MANIFEST.s().into());
        }
        crate::util::copy_tree(src, &dest, &[], &|_, _| {})?;
    } else {
        archive_manifest(src).map_err(|_| crate::i18n::PACK_NO_MANIFEST.s().to_string())?;
        // Copie sous un nom provisoire puis renommage : jamais d'archive tronquée dans patches/.
        let part = dir.join(format!("{name}.part"));
        std::fs::copy(src, &part).map_err(|e| e.to_string())?;
        std::fs::rename(&part, &dest).map_err(|e| e.to_string())?;
    }
    Ok(name)
}

pub fn remove(client: &Path, entry: &PackEntry) -> Result<(), String> {
    if entry.official {
        return Err(crate::i18n::PACK_OFFICIAL.s().into());
    }
    // Seulement ce qui est dans patches/ : un chemin venu d'ailleurs n'est jamais supprimé.
    if entry.path.parent() != Some(patches_dir(client).as_path()) {
        return Err(format!("{} ?", entry.path.display()));
    }
    if entry.path.is_dir() {
        std::fs::remove_dir_all(&entry.path).map_err(|e| e.to_string())?;
    } else {
        std::fs::remove_file(&entry.path).map_err(|e| e.to_string())?;
    }
    let mut prefs = load_prefs(client);
    let before = prefs.packs.len();
    prefs.packs.retain(|c| !c.id.eq_ignore_ascii_case(&entry.id));
    if prefs.packs.len() != before {
        save_prefs(client, &prefs)?;
    }
    Ok(())
}
