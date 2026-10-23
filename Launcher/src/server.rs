//! ArchCore : génération de auth.conf / world.conf, outils (db-init, decrypt, import, i18n), serveur
//! (connexion + monde en un seul processus `archcore all`), arrêt propre par Ctrl+C.

use crate::credentials::DbCredentials;
use crate::db::{port_open, CREATE_NO_WINDOW};
use crate::logs::{self, Line, Source};
use crate::paths::{Layout, MARIADB_PORT, WORLD_NAME};
use crate::settings::Settings;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

/// Remplace la valeur de `key` (ligne « Clé = valeur » non commentée) ; l'ajoute à la fin si le modèle ne l'a pas.
fn set_conf_value(lines: &mut Vec<String>, key: &str, value: &str) {
    for l in lines.iter_mut() {
        let trimmed = l.trim_start();
        if trimmed.starts_with(key) && trimmed[key.len()..].trim_start().starts_with('=') {
            *l = format!("{key} = {value}");
            return;
        }
    }
    lines.push(format!("{key} = {value}"));
}

fn quoted(v: &str) -> String {
    format!("\"{}\"", v.replace('"', "'"))
}

fn database_keys(lines: &mut Vec<String>, creds: &DbCredentials) {
    set_conf_value(lines, "Database.Host", "\"127.0.0.1\"");
    set_conf_value(lines, "Database.Port", &MARIADB_PORT.to_string());
    set_conf_value(lines, "Database.User", &quoted(&creds.user));
    set_conf_value(lines, "Database.Password", &quoted(&creds.password));
}

/// Dossier des données du jeu lu par les outils d'import (tables `ini/`, relief `world/`).
pub fn game_dir(layout: &Layout, settings: &Settings) -> std::path::PathBuf {
    crate::client::from_settings(layout, settings).game_dir
}

/// Les .conf sont régénérés à chaque démarrage depuis les modèles d'ArchCore : les clés gérées (écoute sur
/// 127.0.0.1, ports, base, chemins) d'abord, puis les réglages de l'onglet Monde.
pub fn generate_configs(layout: &Layout, creds: &DbCredentials, settings: &Settings) -> Result<(), String> {
    let dist = layout.archcore_dir().join("config");
    let out = layout.server_config();
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let read = |name: &str| std::fs::read_to_string(dist.join(name)).map(|t| t.lines().map(str::to_string).collect::<Vec<_>>()).map_err(|e| format!("{name} : {e}"));

    // auth.conf
    let mut a = read("auth.conf.dist")?;
    set_conf_value(&mut a, "LoginServer.Bind", "\"127.0.0.1\"");
    set_conf_value(&mut a, "LoginServer.Port", &settings.login_port.to_string());
    database_keys(&mut a, creds);
    std::fs::write(out.join("auth.conf"), a.join("\n") + "\n").map_err(|e| e.to_string())?;

    // world.conf
    let mut w = read("world.conf.dist")?;
    set_conf_value(&mut w, "WorldServer.Bind", "\"127.0.0.1\"");
    set_conf_value(&mut w, "WorldServer.Port", &settings.world_port.to_string());
    set_conf_value(&mut w, "WorldServer.Name", &quoted(WORLD_NAME));
    set_conf_value(&mut w, "WorldServer.Group", &quoted(WORLD_NAME));
    set_conf_value(&mut w, "WorldServer.PublicAddress", "\"127.0.0.1\"");
    database_keys(&mut w, creds);
    set_conf_value(&mut w, "Client.DataDir", &Layout::conf_path(&game_dir(layout, settings)));
    set_conf_value(&mut w, "World.Language", &quoted(settings.game_language.code()));
    for opt in crate::conf_catalog::WORLD_OPTIONS {
        let v = crate::conf_catalog::value(&settings.conf, opt);
        let v = match opt.kind {
            crate::conf_catalog::Kind::Choice(_) => quoted(v),
            _ => v.to_string(),
        };
        set_conf_value(&mut w, opt.key, &v);
    }
    std::fs::write(out.join("world.conf"), w.join("\n") + "\n").map_err(|e| e.to_string())?;
    Ok(())
}

fn archcore_command(layout: &Layout, settings: &Settings) -> Command {
    let mut c = Command::new(layout.archcore_exe());
    c.current_dir(layout.archcore_dir())
        .env("ARCHCORE_ROOT", layout.archcore_dir())
        .env("ARCHCORE_CONF_DIR", layout.server_config())
        .creation_flags(CREATE_NO_WINDOW)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if settings.server_trace {
        c.env("ARCHCORE_TRACE", "1");
    } else {
        c.env_remove("ARCHCORE_TRACE");
    }
    c
}

/// Lance un outil d'ArchCore (`db-init`, `decrypt`, `import`, `i18n db`…) et attend sa fin. Sa sortie va dans
/// le journal des outils (`Logs/server/outils-<date>.log`, un fichier par session du launcher) et dans l'onglet
/// Journaux (Serveur).
pub fn run_tool(layout: &Layout, settings: &Settings, args: &[String], tool_log: &Path, sink: &Sender<Line>) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(tool_log).ok();
    let shown = format!("> archcore {}", args.join(" "));
    if let Some(f) = file.as_mut() {
        use std::io::Write;
        let _ = writeln!(f, "{} {shown}", crate::util::datetime_readable());
    }
    let _ = sink.send(Line { source: Source::Server, text: shown });
    let mut child = archcore_command(layout, settings).args(args).spawn().map_err(|e| crate::util::spawn_error("archcore", &e))?;
    let readers = logs::pipe(&mut child, Source::Server, file, sink.clone());
    let status = child.wait().map_err(|e| e.to_string())?;
    for r in readers {
        let _ = r.join();
    }
    if status.success() {
        Ok(())
    } else {
        Err(crate::tf!("archcore {} : échec (code {:?}), voir le journal du serveur", "archcore {}: failed (code {:?}), see the server log", args.first().cloned().unwrap_or_default(), status.code()))
    }
}

/// Serveur ArchCore en marche (connexion et monde dans le même processus).
pub struct ArchCore {
    child: Child,
    pub log_path: Option<std::path::PathBuf>,
}

impl ArchCore {
    pub fn start(layout: &Layout, settings: &Settings, sink: Sender<Line>) -> Result<ArchCore, String> {
        for port in [settings.login_port, settings.world_port] {
            if port_open(port) {
                return Err(crate::i18n::port_busy(port));
            }
        }
        let mut child = archcore_command(layout, settings).arg("all").spawn().map_err(|e| crate::util::spawn_error("archcore", &e))?;
        let (log_path, file) = match logs::new_file(&layout.server_logs(), "archcore") {
            Some((p, f)) => (Some(p), Some(f)),
            None => (None, None),
        };
        logs::pipe(&mut child, Source::Server, file, sink);
        Ok(ArchCore { child, log_path })
    }

    /// Prêt = les deux ports ouverts (le monde n'écoute qu'une fois chargé).
    pub fn wait_ready(&mut self, login_port: u16, world_port: u16, timeout: Duration) -> Result<(), String> {
        let start = Instant::now();
        while start.elapsed() < timeout {
            if port_open(login_port) && port_open(world_port) {
                return Ok(());
            }
            if let Ok(Some(st)) = self.child.try_wait() {
                return Err(crate::tf!("archcore s'est arrêté pendant le chargement (code {:?}), voir le journal du serveur", "archcore stopped while loading (code {:?}), see the server log", st.code()));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        Err(crate::i18n::WORLD_TIMEOUT.s().into())
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    /// Arrêt propre : Ctrl+C envoyé à la console (invisible) d'archcore, qui ferme ses bases et quitte. Terminaison
    /// forcée seulement après le délai. Renvoie `true` si l'arrêt a été propre.
    pub fn stop(&mut self, timeout: Duration) -> bool {
        if !self.is_running() {
            return true;
        }
        let sent = send_ctrl_c(self.child.id());
        let start = Instant::now();
        let mut clean = false;
        if sent {
            while start.elapsed() < timeout {
                if let Ok(Some(_)) = self.child.try_wait() {
                    clean = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        }
        release_console();
        if !clean {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        clean
    }
}

/// Envoie Ctrl+C à un processus console lancé avec CREATE_NO_WINDOW (il a sa propre console, sans fenêtre) :
/// le launcher s'attache à cette console, ignore lui-même l'événement, puis l'émet pour tous ses processus.
/// `release_console` doit suivre, une fois le processus arrêté.
fn send_ctrl_c(pid: u32) -> bool {
    use windows_sys::Win32::System::Console::{AttachConsole, FreeConsole, GenerateConsoleCtrlEvent, SetConsoleCtrlHandler, CTRL_C_EVENT};
    unsafe {
        FreeConsole();
        if AttachConsole(pid) == 0 {
            return false;
        }
        SetConsoleCtrlHandler(None, 1);
        GenerateConsoleCtrlEvent(CTRL_C_EVENT, 0) != 0
    }
}

fn release_console() {
    use windows_sys::Win32::System::Console::{FreeConsole, SetConsoleCtrlHandler};
    unsafe {
        FreeConsole();
        // L'événement est traité de façon asynchrone : laisser le temps de le livrer avant de le réécouter.
        std::thread::sleep(Duration::from_millis(100));
        SetConsoleCtrlHandler(None, 0);
    }
}
