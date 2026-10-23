//! Journaux : launcher, serveur (ArchCore et ses outils d'import) et client (ArchRust).
//!
//! Chaque processus suivi écrit dans son propre fichier horodaté (`Logs/server/archcore-<date>.log`,
//! `Logs/server/outils-<date>.log`, `Logs/client/archrust-<date>.log`) et chaque ligne est aussi envoyée à
//! l'interface (onglet Journaux) par le canal de l'orchestrateur. Les fichiers les plus anciens sont supprimés
//! au-delà de `KEEP_FILES` par catégorie.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

/// Fichiers conservés par catégorie (archcore, outils, archrust).
pub const KEEP_FILES: usize = 20;
/// Lignes gardées en mémoire par source pour l'onglet Journaux.
pub const KEEP_LINES: usize = 4000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Launcher,
    Server,
    Client,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub source: Source,
    pub text: String,
}

/// Niveau d'une ligne, pour la couleur et le filtre « avertissements et erreurs ».
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Info,
    Warn,
    Error,
}

pub fn level(text: &str) -> Level {
    // ArchCore : « 2026-10-23 10:00:00 WARN  … » ; ArchRust (env_logger) : « [… WARN  archrust…] » ; launcher : « ERREUR … ».
    let head: String = text.chars().take(64).collect::<String>().to_ascii_uppercase();
    if head.contains(" ERROR") || head.contains("[ERROR") || head.contains("ERREUR") || head.contains("PANICKED") {
        Level::Error
    } else if head.contains(" WARN") || head.contains("[WARN") {
        Level::Warn
    } else {
        Level::Info
    }
}

/// Nouveau fichier `<dir>/<prefix>-<date>.log`, après rotation des anciens.
pub fn new_file(dir: &Path, prefix: &str) -> Option<(PathBuf, std::fs::File)> {
    std::fs::create_dir_all(dir).ok()?;
    rotate(dir, prefix, KEEP_FILES.saturating_sub(1));
    let path = dir.join(format!("{prefix}-{}.log", crate::util::datetime_compact()));
    let f = std::fs::OpenOptions::new().create(true).append(true).open(&path).ok()?;
    Some((path, f))
}

/// Fichiers `<prefix>-*.log` du dossier, du plus récent au plus ancien.
pub fn files(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.flatten()
                .map(|e| e.path())
                .filter(|p| p.file_name().map(|n| n.to_string_lossy().starts_with(&format!("{prefix}-")) && n.to_string_lossy().ends_with(".log")).unwrap_or(false))
                .collect()
        })
        .unwrap_or_default();
    // Le nom porte la date (AAAAMMJJ-HHMMSS) : l'ordre alphabétique est l'ordre chronologique.
    v.sort();
    v.reverse();
    v
}

fn rotate(dir: &Path, prefix: &str, keep: usize) {
    for old in files(dir, prefix).into_iter().skip(keep) {
        let _ = std::fs::remove_file(old);
    }
}

/// Lit les sorties d'un processus ligne à ligne : chaque ligne va dans `file` (s'il y en a un) et dans `sink`.
/// Renvoie les fils de lecture (ils se terminent avec le processus).
pub fn pipe(child: &mut std::process::Child, source: Source, file: Option<std::fs::File>, sink: Sender<Line>) -> Vec<std::thread::JoinHandle<()>> {
    let mut readers: Vec<(Box<dyn std::io::Read + Send>, bool)> = Vec::new();
    if let Some(out) = child.stdout.take() {
        readers.push((Box::new(out), false));
    }
    if let Some(err) = child.stderr.take() {
        readers.push((Box::new(err), true));
    }
    let mut handles = Vec::new();
    for (reader, _is_err) in readers {
        let sink = sink.clone();
        let mut file = file.as_ref().and_then(|f| f.try_clone().ok());
        handles.push(std::thread::spawn(move || {
            let mut reader = BufReader::new(reader);
            let mut buf = Vec::new();
            loop {
                buf.clear();
                match reader.read_until(b'\n', &mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        let line = crate::util::decode_console(&buf).trim_end().to_string();
                        if line.is_empty() {
                            continue;
                        }
                        if let Some(f) = file.as_mut() {
                            let _ = writeln!(f, "{line}");
                        }
                        let _ = sink.send(Line { source, text: line });
                    }
                }
            }
        }));
    }
    handles
}

/// Dernières lignes d'un fichier texte (lecture de la fin seulement).
pub fn tail(path: &Path, max_lines: usize) -> Vec<String> {
    use std::io::{Read, Seek, SeekFrom};
    let Ok(mut f) = std::fs::File::open(path) else { return Vec::new() };
    let len = f.metadata().map(|m| m.len()).unwrap_or(0);
    let start = len.saturating_sub(512 * 1024);
    let _ = f.seek(SeekFrom::Start(start));
    let mut buf = Vec::new();
    let _ = f.read_to_end(&mut buf);
    let text = crate::util::decode_console(&buf);
    let lines: Vec<String> = text.lines().map(str::to_string).collect();
    lines[lines.len().saturating_sub(max_lines)..].to_vec()
}

#[cfg(test)]
mod tests {
    use super::{level, Level};

    #[test]
    fn levels_are_recognised() {
        assert_eq!(level("2026-10-23 10:00:00 WARN  [world] requête lente"), Level::Warn);
        assert_eq!(level("2026-10-23 10:00:00 ERROR base injoignable"), Level::Error);
        assert_eq!(level("[2026-10-23T08:00:00Z WARN  archrust_engine] pack illisible"), Level::Warn);
        assert_eq!(level("10:00:00 ERREUR démarrage : port occupé"), Level::Error);
        assert_eq!(level("2026-10-23 10:00:00 INFO  [login] en écoute sur 127.0.0.1:11002"), Level::Info);
    }
}
