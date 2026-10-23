//! Machine à états et pilotage des services, sur un thread dédié. L'interface n'y accède que par
//! des commandes (canal) et par un instantané partagé (`Snapshot`).

use crate::backup::{self, BackupInfo};
use crate::client::{self, ClientInfo};
use crate::credentials;
use crate::db::{AccountRow, MariaDb, QueryResult, WorldStatus};
use crate::i18n as t;
use crate::logs::{self, Line, Source};
use crate::paths::{Layout, ALL_DBS, MARIADB_PORT};
use crate::repair::{self, VerifyReport};
use crate::server::{self, ArchCore};
use crate::settings::Settings;
use crate::tf;
use crate::util;
use std::collections::VecDeque;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    NotInstalled,
    Installed,
    Verifying,
    Initializing,
    StartingDb,
    ImportingWorld,
    StartingServer,
    Ready,
    Playing,
    Stopping,
    Error,
    Repairing,
}

impl State {
    pub fn label(self) -> &'static str {
        match self {
            State::NotInstalled => t::STATE_NOT_INSTALLED.s(),
            State::Installed => t::STATE_INSTALLED.s(),
            State::Verifying => t::STATE_VERIFYING.s(),
            State::Initializing => t::STATE_INITIALIZING.s(),
            State::StartingDb => t::STATE_STARTING_DB.s(),
            State::ImportingWorld => t::STATE_IMPORTING.s(),
            State::StartingServer => t::STATE_STARTING_SERVER.s(),
            State::Ready => t::STATE_READY.s(),
            State::Playing => t::STATE_PLAYING.s(),
            State::Stopping => t::STATE_STOPPING.s(),
            State::Error => t::STATE_ERROR.s(),
            State::Repairing => t::STATE_REPAIRING.s(),
        }
    }
    pub fn is_busy(self) -> bool {
        matches!(self, State::Verifying | State::Initializing | State::StartingDb | State::ImportingWorld | State::StartingServer | State::Stopping | State::Repairing)
    }
    pub fn services_up(self) -> bool {
        matches!(self, State::Ready | State::Playing)
    }
    pub fn idle(self) -> bool {
        matches!(self, State::Installed | State::Error | State::Ready | State::NotInstalled)
    }
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub state: State,
    pub message: String,
    pub error: Option<String>,
    pub progress: f32,
    pub db_up: bool,
    pub login_up: bool,
    pub world_up: bool,
    pub client: ClientInfo,
    /// Lignes des trois journaux (launcher, serveur, client), les plus récentes à la fin
    pub log_launcher: VecDeque<String>,
    pub log_server: VecDeque<String>,
    pub log_client: VecDeque<String>,
    /// Fichiers en cours d'écriture (serveur, client)
    pub server_log_file: Option<PathBuf>,
    pub client_log_file: Option<PathBuf>,
    pub accounts: Vec<AccountRow>,
    pub databases: Vec<String>,
    pub tables: Vec<String>,
    pub query: Option<Result<QueryResult, String>>,
    pub notice: Option<String>,
    pub components: Vec<(String, bool)>,
    pub backups: Vec<BackupInfo>,
    pub report: Option<VerifyReport>,
    pub report_fresh: bool,
    pub readonly: bool,
    pub exit_requested: bool,
    pub packs: Vec<crate::packs::PackEntry>,
    pub addons: Vec<crate::addons::AddonEntry>,
    pub table_view: Option<TableView>,
    /// Une écriture a eu lieu : la page affichée doit être rechargée
    pub table_dirty: bool,
    /// settings.json remplacé par une restauration : l'interface doit relire ses réglages (sinon elle réécrirait les anciens)
    pub settings_reloaded: bool,
    pub world: Option<WorldStatus>,
    pub server_tables: bool,
    /// Tables du client déjà lues (Server/Data/client-ini) : le monde a été importé au moins une fois
    pub client_ini: bool,
    pub archcore_version: String,
}

#[derive(Clone, Debug)]
pub struct TableView {
    pub db: String,
    pub table: String,
    pub columns: Vec<crate::db::ColumnInfo>,
    pub page: QueryResult,
    pub total: u64,
    pub offset: u64,
}

pub enum Command {
    StartAll,
    Play,
    StopAll,
    Verify,
    Repair,
    Backup(String),
    RefreshBackups,
    Restore(PathBuf),
    DeleteBackup(PathBuf),
    ExportDiagnostic,
    RefreshAccounts,
    CreateAccount { user: String, pass: String, gm: u8 },
    SetPassword { user: String, pass: String },
    SetGm { user: String, gm: u8 },
    DeleteAccount(String),
    ListTables(String),
    Query { db: String, sql: String },
    /// Instruction d'écriture confirmée par l'utilisateur
    Exec { db: String, sql: String },
    /// Charge structure + page de données d'une table
    LoadTable { db: String, table: String, filter: String, order: Option<(String, bool)>, offset: u64 },
    OpenHeidi,
    SettingsChanged(Settings),
    InstallTo(PathBuf),
    KillGame,
    RefreshPacks,
    PackSetEnabled(String, bool),
    PackAdd(PathBuf),
    PackRemove(String),
    AddonAdd(PathBuf),
    AddonSetActive(String, bool),
    AddonRemove(String),
    /// Réimporte la base monde depuis le client ; personnages et comptes conservés
    ReimportWorld,
    ClearLog(Source),
    Shutdown,
}

pub struct Orchestrator {
    layout: Layout,
    settings: Settings,
    shared: Arc<Mutex<Snapshot>>,
    rx: Receiver<Command>,
    log_tx: Sender<Line>,
    log_rx: Receiver<Line>,
    db: Option<MariaDb>,
    /// Le processus mariadbd a été lancé par ce launcher (sinon : base d'une autre instance ou d'un script, jamais arrêtée ici)
    db_owned: bool,
    /// Base démarrée pour une opération annexe (sauvegarde, comptes…) : arrêtée dès la fin de l'opération si possédée
    db_annex: bool,
    server: Option<ArchCore>,
    game: Option<Child>,
    log_file: Option<std::fs::File>,
    /// Journal des outils d'ArchCore de cette session (créé au premier outil lancé)
    tool_log: Option<PathBuf>,
    /// Demi-secondes d'attente écoulées : le contrôle périodique (`tick`) a lieu chaque seconde.
    ticks: u64,
}

pub fn spawn(layout: Layout, settings: Settings) -> (Arc<Mutex<Snapshot>>, Sender<Command>, std::thread::JoinHandle<()>) {
    let shared = Arc::new(Mutex::new(Snapshot {
        state: State::Installed,
        message: t::MSG_IDLE.s().into(),
        error: None,
        progress: 0.0,
        db_up: false,
        login_up: false,
        world_up: false,
        client: ClientInfo::default(),
        log_launcher: VecDeque::new(),
        log_server: VecDeque::new(),
        log_client: VecDeque::new(),
        server_log_file: None,
        client_log_file: None,
        accounts: Vec::new(),
        databases: ALL_DBS.iter().map(|d| d.to_string()).collect(),
        tables: Vec::new(),
        query: None,
        notice: None,
        components: Vec::new(),
        backups: Vec::new(),
        report: None,
        report_fresh: false,
        readonly: false,
        exit_requested: false,
        packs: Vec::new(),
        addons: Vec::new(),
        table_view: None,
        table_dirty: false,
        settings_reloaded: false,
        world: None,
        server_tables: false,
        client_ini: false,
        archcore_version: String::new(),
    }));
    let (tx, rx) = mpsc::channel();
    let (log_tx, log_rx) = mpsc::channel();
    let shared2 = shared.clone();
    let handle = std::thread::Builder::new()
        .name("orchestrateur".into())
        .spawn(move || {
            let log_file = std::fs::create_dir_all(layout.logs()).ok().and_then(|_| std::fs::OpenOptions::new().create(true).append(true).open(layout.launcher_log()).ok());
            let mut o = Orchestrator { layout, settings, shared: shared2, rx, log_tx, log_rx, db: None, db_owned: false, db_annex: false, server: None, game: None, log_file, tool_log: None, ticks: 0 };
            o.run();
        })
        .expect("thread orchestrateur");
    (shared, tx, handle)
}

fn push_line(q: &mut VecDeque<String>, line: String) {
    q.push_back(line);
    while q.len() > logs::KEEP_LINES {
        q.pop_front();
    }
}

impl Orchestrator {
    fn log(&mut self, line: impl Into<String>) {
        let stamped = format!("{} {}", util::time_hms(), line.into());
        if let Some(f) = self.log_file.as_mut() {
            let _ = writeln!(f, "{} {stamped}", util::datetime_readable().split(' ').next().unwrap_or(""));
        }
        push_line(&mut self.shared.lock().unwrap().log_launcher, stamped);
    }

    fn set(&self, state: State, message: &str, progress: f32) {
        let mut s = self.shared.lock().unwrap();
        s.state = state;
        s.message = message.to_string();
        s.progress = progress;
        if state != State::Error {
            s.error = None;
        }
    }

    fn set_message(&self, message: String) {
        self.shared.lock().unwrap().message = message;
    }

    fn notice(&self, n: impl Into<String>) {
        self.shared.lock().unwrap().notice = Some(n.into());
    }

    fn state(&self) -> State {
        self.shared.lock().unwrap().state
    }

    fn fail(&mut self, context: &str, err: String) {
        let msg = format!("{context} : {err}");
        self.log(format!("{} {msg}", t::STATE_ERROR.s()));
        let mut s = self.shared.lock().unwrap();
        s.state = State::Error;
        s.error = Some(msg.clone());
        s.message = msg;
        s.progress = 0.0;
    }

    fn fail_ret(&mut self, context: &str, err: String) -> bool {
        self.fail(context, err);
        self.stop_services_quiet();
        false
    }

    fn client_dir(&self) -> PathBuf {
        server::client_dir(&self.layout, &self.settings)
    }

    fn server_tables_dir(&self) -> PathBuf {
        if self.settings.server_tables_dir.trim().is_empty() { self.layout.default_server_tables() } else { PathBuf::from(self.settings.server_tables_dir.trim()) }
    }

    /// Lignes des processus suivis (serveur, outils, client) vers les journaux de l'interface.
    fn drain_process_logs(&mut self) {
        let mut lines = Vec::new();
        while let Ok(l) = self.log_rx.try_recv() {
            lines.push(l);
        }
        if lines.is_empty() {
            return;
        }
        let mut s = self.shared.lock().unwrap();
        for l in lines {
            match l.source {
                Source::Server => push_line(&mut s.log_server, l.text),
                Source::Client => push_line(&mut s.log_client, l.text),
                Source::Launcher => push_line(&mut s.log_launcher, l.text),
            }
        }
    }

    fn run(&mut self) {
        self.log(tf!("HomeRPG ArchLord {} — {}", "HomeRPG ArchLord {} — {}", Layout::version(), self.layout.root.display()));
        if !self.layout.is_writable() {
            self.log(tf!("support en lecture seule : {}", "read-only medium: {}", self.layout.root.display()));
            let mut s = self.shared.lock().unwrap();
            s.readonly = true;
            s.state = State::NotInstalled;
            s.message = t::RO_TITLE.s().into();
        } else {
            self.verify(false);
            self.refresh_backups();
        }
        loop {
            self.drain_process_logs();
            match self.rx.recv_timeout(Duration::from_millis(500)) {
                Ok(Command::Shutdown) => {
                    self.stop_all();
                    break;
                }
                Ok(cmd) => {
                    self.handle(cmd);
                    self.drain_process_logs();
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    self.ticks += 1;
                    if self.ticks % 2 == 0 {
                        self.tick();
                    }
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.stop_all();
                    break;
                }
            }
        }
    }

    fn handle(&mut self, cmd: Command) {
        match cmd {
            Command::StartAll => {
                if self.start_all() {
                    self.set(State::Ready, t::MSG_READY.s(), 1.0);
                }
            }
            Command::Play => {
                if !self.state().services_up() && !self.start_all() {
                    return;
                }
                self.play();
            }
            Command::StopAll => self.stop_all(),
            Command::Verify => self.verify(true),
            Command::Repair => self.repair(),
            Command::ReimportWorld => self.reimport_world(),
            Command::Backup(label) => self.backup(&label),
            Command::RefreshBackups => self.refresh_backups(),
            Command::Restore(p) => self.restore(&p),
            Command::DeleteBackup(p) => {
                // Seulement une archive du dossier des sauvegardes.
                if p.parent() == Some(self.layout.backups().as_path()) {
                    match std::fs::remove_file(&p) {
                        Ok(()) => self.log(tf!("sauvegarde supprimée : {}", "backup deleted: {}", p.display())),
                        Err(e) => self.log(format!("{} {} : {e}", t::STATE_ERROR.s(), p.display())),
                    }
                }
                self.refresh_backups();
            }
            Command::ExportDiagnostic => {
                let state = self.state().label().to_string();
                match repair::export_diagnostic(&self.layout, &self.settings, &state) {
                    Ok(p) => {
                        self.log(tf!("diagnostic exporté : {}", "diagnostic exported: {}", p.display()));
                        self.notice(format!("{} {}", t::MSG_DIAG_DONE.s(), p.display()));
                        let _ = std::process::Command::new("explorer.exe").arg("/select,").arg(&p).spawn();
                    }
                    Err(e) => self.log(format!("{} diagnostic : {e}", t::STATE_ERROR.s())),
                }
            }
            Command::RefreshAccounts => self.refresh_accounts(),
            Command::CreateAccount { user, pass, gm } => self.with_annex_db(|o, db| match db.account_exists(&user) {
                Ok(true) => o.notice(t::acc_exists(&user)),
                Ok(false) => match db.create_account(&user, &pass, gm) {
                    Ok(()) => {
                        o.log(tf!("compte {user} créé (niveau MJ {gm})", "account {user} created (GM level {gm})"));
                        o.notice(t::acc_created(&user, gm));
                    }
                    Err(e) => o.notice(format!("{} : {e}", t::ACC_CREATE_FAIL.s())),
                },
                Err(e) => o.notice(format!("{} : {e}", t::ACC_CREATE_FAIL.s())),
            }),
            Command::SetPassword { user, pass } => self.with_annex_db(|o, db| match db.set_password(&user, &pass) {
                Ok(_) => {
                    o.log(tf!("compte {user} : mot de passe changé", "account {user}: password changed"));
                    o.notice(tf!("Mot de passe de {user} changé.", "Password of {user} changed."));
                }
                Err(e) => o.notice(e),
            }),
            Command::SetGm { user, gm } => self.with_annex_db(|o, db| match db.set_gm(&user, gm) {
                Ok(_) => {
                    o.log(tf!("compte {user} : niveau MJ {gm}", "account {user}: GM level {gm}"));
                    o.notice(t::acc_gm_set(&user, gm));
                }
                Err(e) => o.notice(e),
            }),
            Command::DeleteAccount(user) => {
                if self.state() == State::Playing {
                    self.notice(t::ACC_DELETE_PLAYING.s());
                    return;
                }
                self.with_annex_db(|o, db| match db.delete_account(&user) {
                    Ok(n) => {
                        o.log(tf!("compte {user} supprimé ({n} personnage(s))", "account {user} deleted ({n} character(s))"));
                        o.notice(t::acc_deleted(&user, n));
                    }
                    Err(e) => o.notice(format!("{} : {e}", t::ACC_DELETE_FAIL.s())),
                })
            }
            Command::ListTables(dbname) => {
                let r = self.db.as_ref().ok_or(t::DB_NEED_DB.s().to_string()).and_then(|d| d.tables(&dbname));
                let mut s = self.shared.lock().unwrap();
                match r {
                    Ok(t) => s.tables = t,
                    Err(e) => s.query = Some(Err(e)),
                }
            }
            Command::Query { db: dbname, sql } => {
                let r = self.db.as_ref().ok_or(t::DB_NEED_DB.s().to_string()).and_then(|d| d.query(Some(&dbname), &sql, 500));
                self.shared.lock().unwrap().query = Some(r);
            }
            Command::Exec { db: dbname, sql } => {
                let r = self.db.as_ref().ok_or(t::DB_NEED_DB.s().to_string()).and_then(|d| d.exec(Some(&dbname), &sql));
                match &r {
                    Ok(n) => self.log(format!("SQL ({dbname}) : {n} — {}", sql.chars().take(160).collect::<String>())),
                    Err(e) => self.log(format!("{} SQL ({dbname}) : {e}", t::STATE_ERROR.s())),
                }
                let mut s = self.shared.lock().unwrap();
                s.query = Some(r.map(|n| QueryResult { affected: Some(n), ..Default::default() }));
                s.table_dirty = true;
            }
            Command::LoadTable { db: dbname, table, filter, order, offset } => {
                let r = self.db.as_ref().ok_or(t::DB_NEED_DB.s().to_string()).and_then(|d| {
                    let cols = d.table_columns(&dbname, &table)?;
                    let (page, total) = d.table_rows(&dbname, &table, &filter, order.as_ref().map(|(c, a)| (c.as_str(), *a)), offset, 200)?;
                    Ok(TableView { db: dbname.clone(), table: table.clone(), columns: cols, page, total, offset })
                });
                let mut s = self.shared.lock().unwrap();
                match r {
                    Ok(v) => {
                        s.table_view = Some(v);
                        s.query = None;
                    }
                    Err(e) => s.query = Some(Err(e)),
                }
                s.table_dirty = false;
            }
            Command::OpenHeidi => self.open_heidi(),
            Command::SettingsChanged(s) => {
                let restart_needed = self.server.is_some()
                    && (s.login_port != self.settings.login_port
                        || s.world_port != self.settings.world_port
                        || s.conf != self.settings.conf
                        || s.game_language != self.settings.game_language
                        || s.server_trace != self.settings.server_trace);
                self.settings = s;
                if restart_needed {
                    self.notice(t::SET_RESTART_NEEDED.s());
                }
                self.verify(false);
            }
            Command::InstallTo(dest) => self.install_to(&dest),
            Command::KillGame => {
                if let Some(mut g) = self.game.take() {
                    let _ = g.kill();
                    let _ = g.wait();
                    self.log(tf!("jeu fermé de force à la demande de l'utilisateur", "game closed by force at the user's request"));
                }
                if self.state() == State::Playing {
                    self.set(State::Ready, t::MSG_READY.s(), 1.0);
                }
            }
            Command::RefreshPacks => self.refresh_packs(),
            Command::PackSetEnabled(id, on) => {
                if self.refuse_while_playing() {
                    return;
                }
                match crate::packs::set_enabled(&self.client_dir(), &id, on) {
                    Ok(()) => self.log(tf!("pack {id} {}", "pack {id} {}", if on { t::ENABLED.s() } else { t::DISABLED.s() })),
                    Err(e) => self.notice(e),
                }
                self.refresh_packs();
            }
            Command::PackAdd(src) => {
                if self.refuse_while_playing() {
                    return;
                }
                match crate::packs::add(&src, &self.client_dir()) {
                    Ok(name) => {
                        self.log(tf!("pack ajouté : {name} (depuis {})", "pack added: {name} (from {})", src.display()));
                        self.notice(format!("{} {name}", t::PACK_ADDED.s()));
                    }
                    Err(e) => self.notice(format!("{} : {e}", t::PACK_ADD_FAIL.s())),
                }
                self.refresh_packs();
            }
            Command::PackRemove(id) => {
                if self.refuse_while_playing() {
                    return;
                }
                let client = self.client_dir();
                if let Some(e) = crate::packs::list(&client).into_iter().find(|p| p.id == id) {
                    match crate::packs::remove(&client, &e) {
                        Ok(()) => self.log(tf!("pack supprimé : {id}", "pack removed: {id}")),
                        Err(err) => self.notice(err),
                    }
                }
                self.refresh_packs();
            }
            Command::AddonAdd(src) => {
                if self.refuse_while_playing() {
                    return;
                }
                let prev = self.shared.lock().unwrap().message.clone();
                let shared = self.shared.clone();
                match crate::addons::add_from_zip(&src, &self.client_dir(), &|m| shared.lock().unwrap().message = m.to_string()) {
                    Ok(names) => {
                        self.log(tf!("AddOns installés : {} (depuis {})", "AddOns installed: {} (from {})", names.join(", "), src.display()));
                        self.notice(format!("{} {}", t::ADDON_ADDED.s(), names.join(", ")));
                    }
                    Err(e) => self.notice(format!("{} : {e}", t::ADDON_ADD_FAIL.s())),
                }
                self.set_message(prev);
                self.refresh_packs();
            }
            Command::AddonSetActive(name, active) => {
                if self.refuse_while_playing() {
                    return;
                }
                if let Some(e) = crate::addons::list(&self.client_dir()).iter().find(|a| a.name.eq_ignore_ascii_case(&name)) {
                    match crate::addons::set_active(e, active) {
                        Ok(()) => self.log(format!("AddOn {} {}", e.name, if active { t::ENABLED.s() } else { t::DISABLED.s() })),
                        Err(err) => self.notice(err),
                    }
                }
                self.refresh_packs();
            }
            Command::AddonRemove(name) => {
                if self.refuse_while_playing() {
                    return;
                }
                if let Some(e) = crate::addons::list(&self.client_dir()).iter().find(|a| a.name.eq_ignore_ascii_case(&name)) {
                    match crate::addons::remove(e) {
                        Ok(()) => self.log(tf!("AddOn supprimé : {}", "AddOn removed: {}", e.name)),
                        Err(err) => self.notice(err),
                    }
                }
                self.refresh_packs();
            }
            Command::ClearLog(src) => {
                let mut s = self.shared.lock().unwrap();
                match src {
                    Source::Launcher => s.log_launcher.clear(),
                    Source::Server => s.log_server.clear(),
                    Source::Client => s.log_client.clear(),
                }
            }
            Command::Shutdown => {}
        }
    }

    fn refuse_while_playing(&mut self) -> bool {
        if self.state() == State::Playing {
            self.notice(t::CLIENT_FILES_PLAYING.s());
            return true;
        }
        false
    }

    /// Packs de ressources et AddOns du client.
    fn refresh_packs(&mut self) {
        let dir = self.client_dir();
        let packs = crate::packs::list(&dir);
        let addons = crate::addons::list(&dir);
        let mut s = self.shared.lock().unwrap();
        s.packs = packs;
        s.addons = addons;
    }

    /// Exécute une opération de base (comptes) : base démarrée temporairement si besoin, schéma créé à la première
    /// installation, base temporaire arrêtée ensuite.
    fn with_annex_db(&mut self, op: impl FnOnce(&mut Self, &MariaDb)) {
        if self.state().is_busy() {
            return;
        }
        let was_temp = self.db.is_none();
        match self.ensure_db(true) {
            Ok(db) => op(self, &db),
            Err(e) => self.notice(e),
        }
        self.refresh_accounts();
        if was_temp {
            self.release_temp_db();
        }
    }

    /// Installation depuis un support en lecture seule : copie intégrale vers `dest`, puis lancement du launcher copié.
    fn install_to(&mut self, dest: &Path) {
        self.set(State::Repairing, t::MSG_INSTALLING.s(), 0.0);
        let shared = self.shared.clone();
        let r = std::fs::create_dir_all(dest).map_err(|e| e.to_string()).and_then(|_| {
            util::copy_tree(&self.layout.root, dest, &["UserData", "Logs", "target"], &|i, n| {
                let mut s = shared.lock().unwrap();
                s.progress = i as f32 / n.max(1) as f32;
                s.message = format!("{} {i}/{n}", t::MSG_INSTALLING.s());
            })
        });
        match r {
            Ok(()) => {
                self.log(tf!("copié vers {}", "copied to {}", dest.display()));
                let exe_name = std::env::current_exe().ok().and_then(|p| p.file_name().map(|n| n.to_os_string())).unwrap_or_else(|| "HomeRPG ArchLord.exe".into());
                let launched = std::process::Command::new(dest.join(&exe_name)).current_dir(dest).spawn().is_ok();
                let mut s = self.shared.lock().unwrap();
                s.state = State::NotInstalled;
                s.message = t::MSG_INSTALLED.s().into();
                s.progress = 0.0;
                s.exit_requested = launched;
            }
            Err(e) => self.fail(t::CTX_INSTALL.s(), e),
        }
    }

    /// Contrôle périodique (chaque seconde) : santé réelle des services, fin du jeu, plantages.
    fn tick(&mut self) {
        let db_up = self.db.as_ref().map(|d| d.ping()).unwrap_or(false);
        // Serveur prêt = ses deux ports ont répondu au démarrage (wait_ready) ; ensuite seul le processus est
        // surveillé : chaque sondage de port ouvrirait une session notée dans le journal du serveur.
        let server_alive = self.server.as_mut().map(|s| s.is_running()).unwrap_or(false);
        let login_up = server_alive;
        let world_up = server_alive;
        let state = self.state();
        {
            let mut s = self.shared.lock().unwrap();
            s.db_up = db_up;
            s.login_up = login_up;
            s.world_up = world_up;
        }
        if state.services_up() && !(db_up && login_up && world_up) {
            let which = if !db_up { t::COMP_DB.s() } else { t::COMP_SERVER.s() };
            self.fail(t::CTX_SERVICE_DOWN.s(), tf!("{which} ne répond plus", "{which} no longer responds"));
            self.stop_services_quiet();
            return;
        }
        if state == State::Playing {
            if let Some(g) = self.game.as_mut() {
                if let Ok(Some(st)) = g.try_wait() {
                    self.game = None;
                    self.drain_process_logs();
                    self.log(tf!("jeu terminé (code {:?})", "game ended (code {:?})", st.code()));
                    if st.code().unwrap_or(0) != 0 {
                        self.notice(t::GAME_CRASHED.s());
                    }
                    self.set(State::Ready, t::MSG_READY.s(), 1.0);
                    if self.settings.auto_backup {
                        self.backup(backup::AUTO_LABEL);
                        self.rotate_auto_backups();
                        self.set_message(t::MSG_READY.s().into());
                    }
                }
            }
        }
    }

    /// Ne conserve que les `auto_backup_keep` sauvegardes automatiques les plus récentes.
    fn rotate_auto_backups(&mut self) {
        let keep = self.settings.auto_backup_keep.max(1) as usize;
        let auto: Vec<_> = backup::list(&self.layout).into_iter().filter(|b| b.manifest.label == backup::AUTO_LABEL).collect();
        let mut removed = 0;
        for b in auto.iter().skip(keep) {
            if std::fs::remove_file(&b.path).is_ok() {
                removed += 1;
            }
        }
        if removed > 0 {
            self.log(tf!("rotation des sauvegardes automatiques : {removed} supprimée(s), {keep} conservée(s)", "automatic backup rotation: {removed} deleted, {keep} kept"));
            self.refresh_backups();
        }
    }

    // ------------------------------------------------------------------ outils d'ArchCore

    fn tool(&mut self, args: &[&str]) -> Result<(), String> {
        let path = match &self.tool_log {
            Some(p) => p.clone(),
            None => {
                let p = logs::new_file(&self.layout.server_logs(), "outils").map(|(p, _)| p).unwrap_or_else(|| self.layout.server_logs().join("outils.log"));
                self.tool_log = Some(p.clone());
                p
            }
        };
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let r = server::run_tool(&self.layout, &self.settings, &args, &path, &self.log_tx);
        self.drain_process_logs();
        r
    }

    /// Schéma des bases : `archcore db-init` crée les bases manquantes et applique les migrations (sans risque :
    /// il n'ajoute que ce qui manque). Le serveur refuse de démarrer sur une base en retard.
    fn schema(&mut self, db: &MariaDb) -> Result<(), String> {
        db.ensure_user()?;
        server::generate_configs(&self.layout, db.creds(), &self.settings)?;
        self.tool(&["db-init"])
    }

    /// Monde : tables du client déchiffrées, importées dans `archcore_world` (avec les tables du serveur
    /// d'origine si elles sont fournies), titres de PNJ traduits. Seulement si le monde est vide, ou si `force`.
    fn prepare_world(&mut self, db: &MariaDb, force: bool) -> Result<(), String> {
        let status = db.world_status();
        if status.imported() && !force {
            self.shared.lock().unwrap().world = Some(status);
            return Ok(());
        }
        let client = client::inspect(&self.client_dir());
        if !client.is_ok() {
            return Err(t::MSG_CLIENT_NEEDED.s().into());
        }
        let ini = client.dir.join("ini");
        let out = self.layout.client_ini();
        std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
        self.set(State::ImportingWorld, t::MSG_DECRYPT.s(), 0.4);
        self.tool(&["decrypt", &ini.to_string_lossy(), &out.to_string_lossy()])?;
        self.set(State::ImportingWorld, t::MSG_IMPORT.s(), 0.5);
        let tables = self.server_tables_dir();
        let out_s = out.to_string_lossy().to_string();
        let tables_s = tables.to_string_lossy().to_string();
        if repair::server_tables_present(&tables) {
            self.log(tf!("import avec les tables du serveur d'origine : {}", "import with the original server tables: {}", tables.display()));
            self.tool(&["import", &out_s, "--server", &tables_s])?;
        } else {
            self.log(tf!(
                "tables du serveur d'origine absentes ({}) : monde importé sans monstres ni butin",
                "original server tables missing ({}): world imported without monsters or loot",
                tables.display()
            ));
            self.tool(&["import", &out_s])?;
        }
        self.set(State::ImportingWorld, t::MSG_I18N.s(), 0.7);
        if let Err(e) = self.tool(&["i18n", "db", "fr"]) {
            // Facultatif : les titres de PNJ traduits servent de repli.
            self.log(format!("i18n : {e}"));
        }
        let status = db.world_status();
        self.log(tf!(
            "monde importé : {} modèles de personnages, {} objets, {} PNJ, {} apparitions de monstres",
            "world imported: {} character templates, {} items, {} NPCs, {} monster spawns",
            status.templates,
            status.items,
            status.npcs,
            status.monsters
        ));
        self.shared.lock().unwrap().world = Some(status.clone());
        if !status.imported() {
            return Err(t::MSG_IMPORT_EMPTY.s().into());
        }
        Ok(())
    }

    fn reimport_world(&mut self) {
        let st = self.state();
        if st.is_busy() {
            return;
        }
        if st == State::Playing {
            self.notice(t::WORLD_REIMPORT_PLAYING.s());
            return;
        }
        if self.server.is_some() {
            self.set(State::Stopping, t::MSG_STOPPING.s(), 0.3);
            self.stop_server_only();
        }
        self.set(State::Repairing, t::MSG_WORLD_REIMPORT.s(), 0.2);
        let r = self.ensure_db(true).and_then(|db| self.prepare_world(&db, true));
        self.release_temp_db();
        match r {
            Ok(()) => {
                self.log(tf!("base monde réimportée (personnages et comptes conservés)", "world database reimported (characters and accounts kept)"));
                self.set(State::Installed, t::MSG_WORLD_REIMPORT_DONE.s(), 0.0);
            }
            Err(e) => self.fail(t::CTX_REIMPORT.s(), e),
        }
    }

    // ------------------------------------------------------------------ vérification / réparation

    /// Vérification : composants, client, et (si `full`) fichiers gérés par empreinte + données + bases.
    fn verify(&mut self, full: bool) {
        let prev = self.state();
        let components = self.layout.components_present();
        let info = client::inspect(&self.client_dir());
        let all = components.iter().all(|(_, ok)| *ok);
        let tables = repair::server_tables_present(&self.server_tables_dir());
        let version = std::fs::read_to_string(self.layout.archcore_dir().join("VERSION.txt")).map(|v| v.trim().to_string()).unwrap_or_default();
        self.log(tf!(
            "vérification : composants {}, client {} ({}){}",
            "check: components {}, client {} ({}){}",
            if all { "OK" } else { t::INCOMPLETE.s() },
            if info.is_ok() { "OK" } else { t::INCOMPLETE.s() },
            info.dir.display(),
            if info.missing.is_empty() { String::new() } else { format!(" — {}", info.missing.join(", ")) }
        ));
        {
            let mut s = self.shared.lock().unwrap();
            s.components = components;
            s.client = info;
            s.server_tables = tables;
            s.client_ini = repair::client_ini_present(&self.layout);
            s.archcore_version = version;
        }
        self.refresh_packs();
        // Base déjà en service (lancée par un script ou une autre instance) : on s'y attache pour l'explorateur
        // et les comptes, sans en prendre la propriété.
        if self.db.is_none() {
            if let Ok(creds) = credentials::load_or_create(&self.layout.credentials_file()) {
                let d = MariaDb::new(self.layout.clone(), creds);
                if d.ping() {
                    self.db = Some(d);
                    self.db_owned = false;
                    self.db_annex = false;
                    self.shared.lock().unwrap().db_up = true;
                    self.log(tf!("MariaDB déjà en service : utilisée telle quelle (non gérée par ce launcher)", "MariaDB already running: used as is (not managed by this launcher)"));
                    self.refresh_accounts();
                }
            }
        }
        if full && !prev.is_busy() {
            self.set(State::Verifying, t::MSG_VERIFYING_FILES.s(), 0.1);
            let report = self.build_report();
            self.log(tf!(
                "fichiers gérés : {} vérifiés, {} manquants, {} altérés",
                "managed files: {} checked, {} missing, {} altered",
                report.checked,
                report.missing.len(),
                report.altered.len()
            ));
            let mut s = self.shared.lock().unwrap();
            s.report = Some(report);
            s.report_fresh = true;
            s.state = prev;
            s.message = if prev.services_up() { t::MSG_READY.s().into() } else { t::MSG_IDLE.s().into() };
            s.progress = 0.0;
            return;
        }
        if !prev.services_up() && !prev.is_busy() {
            let mut s = self.shared.lock().unwrap();
            if s.state != State::Error || all {
                s.state = if all { State::Installed } else { State::NotInstalled };
                s.message = if all { t::MSG_IDLE.s().into() } else { t::MSG_NOT_INSTALLED.s().into() };
            }
        }
    }

    fn build_report(&mut self) -> VerifyReport {
        let mut report = VerifyReport::default();
        match repair::load_manifest(&self.layout) {
            Some(m) => {
                report.manifest_present = true;
                let shared = self.shared.clone();
                let (missing, altered) = repair::verify_files(&self.layout, &m, &|i, n| {
                    let mut s = shared.lock().unwrap();
                    s.progress = i as f32 / n.max(1) as f32;
                    s.message = format!("{} {i}/{n}", t::MSG_VERIFYING_FILES.s());
                });
                report.checked = m.files.len();
                report.missing = missing;
                report.altered = altered;
            }
            None => report.notes.push(t::RPT_MANIFEST_MISSING.s().into()),
        }
        let client = client::inspect(&self.client_dir());
        report.data = vec![
            (t::RPT_CLIENT.s().to_string(), client.is_ok()),
            (t::RPT_CLIENT_INI.s().to_string(), repair::client_ini_present(&self.layout)),
            (t::RPT_SERVER_TABLES.s().to_string(), repair::server_tables_present(&self.server_tables_dir())),
        ];
        if let Some(d) = self.db.as_ref().filter(|d| d.ping()) {
            let world = d.world_status();
            report.db_ok = Some(d.databases_missing().map(|m| m.is_empty()).unwrap_or(false) && world.imported());
            report.world = Some(world);
        }
        if !report.files_ok() {
            report.notes.push(t::RPT_UNREPAIRABLE.s().into());
        }
        report
    }

    /// Base disponible pour une opération hors service (sauvegarde, comptes, réparation) : démarrée temporairement
    /// si besoin ; avec `schema`, bases créées et migrations appliquées si elles manquent.
    fn ensure_db(&mut self, schema: bool) -> Result<MariaDb, String> {
        let db = match self.db.as_ref().filter(|d| d.ping()) {
            Some(d) => d.clone(),
            None => {
                let creds = credentials::load_or_create(&self.layout.credentials_file())?;
                let d = MariaDb::new(self.layout.clone(), creds);
                if d.ping() {
                    self.log(tf!("MariaDB déjà en service : utilisée telle quelle", "MariaDB already running: used as is"));
                    self.db_owned = false;
                    self.db_annex = false;
                } else {
                    d.write_my_ini()?;
                    if !d.is_initialized() {
                        self.log(tf!("première initialisation du datadir MariaDB", "first initialisation of the MariaDB datadir"));
                        d.initialize()?;
                    }
                    let started = d.start(Duration::from_secs(90))?;
                    self.log(tf!("MariaDB démarrée", "MariaDB started"));
                    self.db_owned = started;
                    self.db_annex = started;
                }
                self.db = Some(d.clone());
                self.shared.lock().unwrap().db_up = true;
                d
            }
        };
        if schema && !db.databases_missing()?.is_empty() {
            self.schema(&db)?;
        }
        Ok(db)
    }

    fn release_temp_db(&mut self) {
        if self.db_annex {
            if let Some(d) = self.db.take() {
                if self.db_owned {
                    match d.shutdown(Duration::from_secs(60)) {
                        Ok(()) => self.log(tf!("MariaDB temporaire arrêtée", "temporary MariaDB stopped")),
                        Err(e) => self.log(format!("MariaDB : {e}")),
                    }
                }
            }
            self.db_annex = false;
            self.db_owned = false;
            self.shared.lock().unwrap().db_up = false;
        }
    }

    fn repair(&mut self) {
        let st = self.state();
        if st.is_busy() || st == State::Playing {
            return;
        }
        let server_was_up = self.server.is_some();
        if server_was_up {
            self.set(State::Stopping, t::MSG_STOPPING.s(), 0.1);
            self.stop_server_only();
        }
        self.set(State::Repairing, t::MSG_REPAIR.s(), 0.05);
        let mut notes: Vec<String> = Vec::new();
        let result: Result<(), String> = (|| {
            let db = self.ensure_db(false)?;
            // Schéma : bases manquantes recréées, migrations appliquées (les personnages ne sont jamais touchés).
            self.shared.lock().unwrap().progress = 0.2;
            self.schema(&db)?;
            notes.push(t::RPT_NOTE_SCHEMA.s().into());
            db.ensure_world_row(self.settings.world_port)?;
            notes.push(t::RPT_NOTE_CONFIGS.s().into());
            // Monde vide : réimport depuis le client.
            if !db.world_status().imported() {
                self.prepare_world(&db, false)?;
                notes.push(t::RPT_NOTE_WORLD.s().into());
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.fail(t::CTX_REPAIR.s(), e);
            self.release_temp_db();
            return;
        }
        self.shared.lock().unwrap().progress = 0.9;
        let mut report = self.build_report();
        report.notes.extend(notes);
        self.release_temp_db();
        self.log(tf!("réparation terminée : {}", "repair finished: {}", report.notes.join(" ; ")));
        {
            let mut s = self.shared.lock().unwrap();
            s.report = Some(report);
            s.report_fresh = true;
        }
        if server_was_up && self.start_all() {
            self.set(State::Ready, t::MSG_REPAIR_DONE.s(), 1.0);
        } else if !server_was_up {
            self.set(State::Installed, t::MSG_REPAIR_DONE.s(), 0.0);
        }
    }

    // ------------------------------------------------------------------ sauvegardes

    fn refresh_backups(&mut self) {
        let list = backup::list(&self.layout);
        self.shared.lock().unwrap().backups = list;
    }

    fn backup(&mut self, label: &str) {
        if self.state().is_busy() {
            return;
        }
        let prev_msg = self.shared.lock().unwrap().message.clone();
        self.set_message(t::MSG_BACKUP.s().into());
        let r = self.ensure_db(true).and_then(|db| {
            let shared = self.shared.clone();
            backup::create(&self.layout, &db, label, &|step| shared.lock().unwrap().message = format!("{} ({step})", t::MSG_BACKUP.s()))
        });
        self.release_temp_db();
        match r {
            Ok(info) => {
                self.log(tf!(
                    "sauvegarde créée : {} ({} personnages, {} comptes, {})",
                    "backup created: {} ({} characters, {} accounts, {})",
                    info.path.display(),
                    info.manifest.characters,
                    info.manifest.accounts,
                    util::human_size(info.size)
                ));
                self.set_message(format!("{} {}", t::MSG_BACKUP_DONE.s(), info.manifest.created));
            }
            Err(e) => {
                self.log(format!("{} {} : {e}", t::STATE_ERROR.s(), t::CTX_BACKUP.s()));
                self.notice(format!("{} : {e}", t::CTX_BACKUP.s()));
                self.set_message(prev_msg);
            }
        }
        self.refresh_backups();
    }

    fn restore(&mut self, path: &Path) {
        let st = self.state();
        if st.is_busy() {
            return;
        }
        if st == State::Playing {
            self.notice(t::BK_RESTORE_PLAYING.s());
            return;
        }
        if self.server.is_some() {
            self.set(State::Stopping, t::MSG_STOPPING.s(), 0.3);
            self.stop_server_only();
        }
        self.set(State::Repairing, t::MSG_RESTORE.s(), 0.2);
        let r = self.ensure_db(true).and_then(|db| {
            let shared = self.shared.clone();
            backup::restore(&self.layout, &db, path, &|step| shared.lock().unwrap().message = format!("{} ({step})", t::MSG_RESTORE.s()))
        });
        self.release_temp_db();
        self.refresh_backups();
        match r {
            Ok(m) => {
                self.log(tf!(
                    "restauration terminée depuis {} ({} personnages, {} comptes)",
                    "restore finished from {} ({} characters, {} accounts)",
                    path.display(),
                    m.characters,
                    m.accounts
                ));
                self.settings = Settings::load(&self.layout.settings_file());
                self.shared.lock().unwrap().settings_reloaded = true;
                self.set(State::Installed, t::MSG_RESTORE_DONE.s(), 0.0);
            }
            Err(e) => self.fail(t::CTX_RESTORE.s(), e),
        }
    }

    // ------------------------------------------------------------------ démarrage / arrêt

    /// Enchaînement complet : vérification -> base -> schéma -> monde (premier démarrage) -> serveur.
    fn start_all(&mut self) -> bool {
        let st = self.state();
        if st.is_busy() || st.services_up() {
            return st.services_up();
        }
        if self.shared.lock().unwrap().readonly {
            self.fail(t::CTX_START.s(), t::RO_TITLE.s().into());
            return false;
        }
        self.set(State::Verifying, t::MSG_VERIFY.s(), 0.05);
        self.verify(false);
        if self.state() == State::NotInstalled {
            self.fail(t::CTX_VERIFY.s(), t::MSG_NOT_INSTALLED.s().into());
            return false;
        }

        self.set(State::StartingDb, t::MSG_DB.s(), 0.1);
        // Base déjà lancée par ce launcher (laissée en service après une opération) : elle reste la nôtre.
        let already_ours = self.db_owned && self.db.as_ref().map(|d| d.ping()).unwrap_or(false);
        self.db_annex = false;
        let db = match self.ensure_db(false) {
            Ok(d) => d,
            Err(e) => return self.fail_ret(t::CTX_DB.s(), e),
        };
        self.db_owned = self.db_owned || already_ours;
        self.db_annex = false;
        self.log(tf!(
            "MariaDB opérationnelle sur 127.0.0.1:{MARIADB_PORT}{}",
            "MariaDB running on 127.0.0.1:{MARIADB_PORT}{}",
            if self.db_owned { "" } else { t::DB_NOT_OURS.s() }
        ));

        self.set(State::Initializing, t::MSG_SCHEMA.s(), 0.2);
        if let Err(e) = self.schema(&db) {
            return self.fail_ret(t::CTX_SCHEMA.s(), e);
        }
        if let Err(e) = self.prepare_world(&db, false) {
            return self.fail_ret(t::CTX_WORLD.s(), e);
        }
        if let Err(e) = db.ensure_world_row(self.settings.world_port) {
            return self.fail_ret(t::CTX_SCHEMA.s(), e);
        }

        self.set(State::StartingServer, t::MSG_SERVER.s(), 0.8);
        let mut srv = match ArchCore::start(&self.layout, &self.settings, self.log_tx.clone()) {
            Ok(s) => s,
            Err(e) => return self.fail_ret(t::CTX_SERVER.s(), e),
        };
        self.shared.lock().unwrap().server_log_file = srv.log_path.clone();
        let deadline = std::time::Instant::now() + Duration::from_secs(600);
        loop {
            self.drain_process_logs();
            match srv.wait_ready(self.settings.login_port, self.settings.world_port, Duration::from_millis(500)) {
                Ok(()) => break,
                Err(e) if std::time::Instant::now() > deadline || !srv.is_running() => {
                    // Toujours arrêté ici : il n'est pas encore dans `self.server`, l'arrêt général ne le verrait pas.
                    srv.stop(Duration::from_secs(20));
                    self.drain_process_logs();
                    return self.fail_ret(t::CTX_SERVER.s(), e);
                }
                Err(_) => {}
            }
        }
        self.server = Some(srv);
        self.log(tf!(
            "ArchCore opérationnel : connexion 127.0.0.1:{}, monde 127.0.0.1:{}",
            "ArchCore running: login 127.0.0.1:{}, world 127.0.0.1:{}",
            self.settings.login_port,
            self.settings.world_port
        ));
        {
            let mut s = self.shared.lock().unwrap();
            s.db_up = true;
            s.login_up = true;
            s.world_up = true;
        }
        self.refresh_accounts();
        true
    }

    fn play(&mut self) {
        let info = client::inspect(&self.client_dir());
        if !info.is_ok() {
            self.fail(t::CTX_CLIENT.s(), tf!("{} ({})", "{} ({})", t::MSG_CLIENT_NEEDED.s(), info.dir.display()));
            return;
        }
        if let Err(e) = client::configure(&info, &self.settings) {
            self.fail(t::CTX_CLIENT.s(), e);
            return;
        }
        match client::launch(&info, &self.settings, &self.layout.logs().join("client"), self.log_tx.clone()) {
            Ok((child, log_path)) => {
                self.log(tf!(
                    "jeu lancé : {} (journal : {})",
                    "game started: {} (log: {})",
                    info.dir.join("ArchRust.exe").display(),
                    log_path.as_ref().map(|p| p.display().to_string()).unwrap_or_default()
                ));
                self.shared.lock().unwrap().client_log_file = log_path;
                self.game = Some(child);
                self.set(State::Playing, t::MSG_PLAYING.s(), 1.0);
            }
            Err(e) => self.fail(t::CTX_CLIENT.s(), e),
        }
    }

    /// Arrête ArchCore, conserve MariaDB (restauration, réimport…).
    fn stop_server_only(&mut self) {
        if let Some(mut s) = self.server.take() {
            let clean = s.stop(Duration::from_secs(30));
            self.drain_process_logs();
            self.log(if clean { tf!("ArchCore arrêté proprement", "ArchCore stopped cleanly") } else { tf!("ArchCore arrêté de force après délai", "ArchCore stopped by force after the timeout") });
        }
        let mut s = self.shared.lock().unwrap();
        s.login_up = false;
        s.world_up = false;
    }

    fn stop_services_quiet(&mut self) {
        self.stop_server_only();
        if let Some(d) = self.db.take() {
            if self.db_owned {
                match d.shutdown(Duration::from_secs(60)) {
                    Ok(()) => self.log(tf!("MariaDB arrêtée", "MariaDB stopped")),
                    Err(e) => self.log(format!("MariaDB : {e}")),
                }
            } else {
                self.log(tf!("MariaDB laissée en service (non démarrée par ce launcher)", "MariaDB left running (not started by this launcher)"));
            }
        }
        self.db_owned = false;
        self.db_annex = false;
        self.shared.lock().unwrap().db_up = false;
    }

    fn stop_all(&mut self) {
        // Rien à arrêter (fermeture après un arrêt déjà fait) : pas de nouvelle vérification.
        if self.state() == State::Stopping || (self.server.is_none() && self.db.is_none() && self.game.is_none()) {
            return;
        }
        self.set(State::Stopping, t::MSG_STOPPING.s(), 0.5);
        // Sans serveur, le jeu ne peut que perdre sa connexion : il est fermé avec les services.
        if let Some(mut g) = self.game.take() {
            if matches!(g.try_wait(), Ok(None)) {
                let _ = g.kill();
                let _ = g.wait();
                self.log(tf!("jeu fermé : les services s'arrêtent", "game closed: the services are stopping"));
            }
        }
        self.stop_services_quiet();
        self.set(State::Installed, t::MSG_STOPPED.s(), 0.0);
        self.verify(false);
    }

    // ------------------------------------------------------------------ comptes, outils

    fn refresh_accounts(&mut self) {
        let Some(db) = self.db.as_ref() else { return };
        if db.databases_missing().map(|m| !m.is_empty()).unwrap_or(true) {
            return;
        }
        let world = db.world_status();
        match db.accounts() {
            Ok(list) => {
                let mut s = self.shared.lock().unwrap();
                s.accounts = list;
                s.world = Some(world);
            }
            Err(e) => self.log(format!("comptes / accounts : {e}")),
        }
    }

    fn open_heidi(&mut self) {
        let exe = self.layout.heidisql_exe();
        if !exe.is_file() {
            self.notice(t::DB_HEIDI_MISSING.s());
            return;
        }
        let Some(db) = self.db.as_ref() else {
            self.notice(t::DB_NEED_DB.s());
            return;
        };
        let c = db.creds();
        // Session non enregistrée : pas de -d (sinon HeidiSQL cherche une session sauvegardée de ce nom), adresse
        // sans guillemets (std::process les échapperait et HeidiSQL ne les comprendrait pas).
        let r = std::process::Command::new(&exe)
            .args(["-n=0".to_string(), "-h=127.0.0.1".to_string(), format!("-P={MARIADB_PORT}"), format!("-u={}", c.user), format!("-p={}", c.password), format!("-db={}", ALL_DBS.join(";"))])
            .current_dir(exe.parent().unwrap())
            .spawn();
        match r {
            Ok(_) => self.log("HeidiSQL"),
            Err(e) => self.log(format!("HeidiSQL : {e}")),
        }
    }
}
