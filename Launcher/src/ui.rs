//! Interface egui : écran principal, onglets Comptes / Monde / Packs / Base de données / Sauvegardes /
//! Paramètres / Journaux / Aide MJ, fenêtres de confirmation et rapport de vérification. Bilingue (français,
//! anglais) : la langue se change à chaud dans Paramètres.

use crate::conf_catalog;
use crate::i18n::{self as t, Lang};
use crate::logs::{self, Source};
use crate::orchestrator::{Command, Snapshot, State};
use crate::paths::Layout;
use crate::settings::Settings;
use crate::util;
use eframe::egui::{self, Color32, RichText};
use egui_extras::{Column, TableBuilder};
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Home,
    Accounts,
    World,
    Packs,
    Database,
    Backups,
    Settings,
    Logs,
    Help,
}

enum Pending {
    Restore(PathBuf, String),
    Delete(PathBuf, String),
    RemovePack(String),
    RemoveAddon(String),
    DeleteAccount(String, String),
    /// (base, sql, titre, texte)
    ExecSql(String, String, t::T, t::T),
    ReimportWorld,
}

pub struct App {
    layout: Layout,
    shared: Arc<Mutex<Snapshot>>,
    tx: Sender<Command>,
    tab: Tab,
    settings: Settings,
    settings_notice: Option<String>,
    closing: bool,
    quit_at: Option<std::time::Instant>,
    pending: Option<Pending>,
    show_report: bool,
    // Comptes
    acc_user: String,
    acc_pass: String,
    acc_gm: u8,
    acc_selected: Option<String>,
    // Base de données
    db_selected: String,
    table_selected: String,
    sql: String,
    db_tab: u8,
    db_filter: String,
    db_order: Option<(String, bool)>,
    editing: Option<(usize, usize, String)>,
    insert_form: Option<Vec<String>>,
    sql_history: Vec<String>,
    tables_requested_for: Option<String>,
    // Sauvegardes
    backup_selected: Option<PathBuf>,
    backup_label: String,
    // Journaux
    log_source: Source,
    log_filter: String,
    log_problems_only: bool,
    // Paramètres : client détecté dans le dossier saisi (recalculé quand le chemin change), clients trouvés à proximité
    client_probe: Option<(String, crate::client::ClientInfo)>,
    discovered: Option<Vec<crate::client::ClientInfo>>,
    /// Recherche des clients en cours (fil séparé)
    discover_rx: Option<std::sync::mpsc::Receiver<Vec<crate::client::ClientInfo>>>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, layout: Layout, settings: Settings, shared: Arc<Mutex<Snapshot>>, tx: Sender<Command>, quit_after: Option<u64>, start_tab: Option<String>) -> App {
        let tab = match start_tab.as_deref() {
            Some("accounts") => Tab::Accounts,
            Some("world") => Tab::World,
            Some("packs") => Tab::Packs,
            Some("database") => Tab::Database,
            Some("backups") => Tab::Backups,
            Some("settings") => Tab::Settings,
            Some("logs") => Tab::Logs,
            Some("help") => Tab::Help,
            _ => Tab::Home,
        };
        egui_extras::install_image_loaders(&cc.egui_ctx);
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::TRANSPARENT;
        visuals.window_fill = Color32::from_rgba_unmultiplied(22, 16, 10, 245);
        visuals.extreme_bg_color = Color32::from_black_alpha(160);
        visuals.faint_bg_color = Color32::from_white_alpha(10);
        visuals.selection.bg_fill = Color32::from_rgb(140, 96, 30);
        cc.egui_ctx.set_visuals(visuals);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(12.0, 6.0);
        cc.egui_ctx.set_style(style);
        App {
            layout,
            shared,
            tx,
            tab,
            settings,
            settings_notice: None,
            closing: false,
            quit_at: quit_after.map(|s| std::time::Instant::now() + std::time::Duration::from_secs(s)),
            pending: None,
            show_report: false,
            acc_user: String::new(),
            acc_pass: String::new(),
            acc_gm: 3,
            acc_selected: None,
            db_selected: crate::paths::DB_CHARACTERS.into(),
            table_selected: String::new(),
            sql: "SELECT id, name, level, money, pos_x, pos_z FROM characters LIMIT 200".into(),
            db_tab: 0,
            db_filter: String::new(),
            db_order: None,
            editing: None,
            insert_form: None,
            sql_history: Vec::new(),
            tables_requested_for: None,
            backup_selected: None,
            backup_label: String::new(),
            log_source: Source::Launcher,
            log_filter: String::new(),
            log_problems_only: false,
            client_probe: None,
            discovered: None,
            discover_rx: None,
        }
    }

    fn send(&self, c: Command) {
        let _ = self.tx.send(c);
    }
}

fn status_color(ok: bool, busy: bool) -> Color32 {
    if ok {
        Color32::from_rgb(110, 200, 90)
    } else if busy {
        Color32::from_rgb(230, 180, 60)
    } else {
        Color32::from_rgb(210, 95, 85)
    }
}

const GOLD: Color32 = Color32::from_rgb(232, 186, 90);
const WARN: Color32 = Color32::from_rgb(230, 180, 60);
const ERR: Color32 = Color32::from_rgb(230, 110, 110);

fn open_in_explorer(p: &std::path::Path) {
    let _ = std::process::Command::new("explorer.exe").arg(p).spawn();
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(std::time::Duration::from_millis(500));
        let snap = self.shared.lock().unwrap().clone();
        if snap.report_fresh {
            self.shared.lock().unwrap().report_fresh = false;
            self.show_report = true;
        }
        if snap.settings_reloaded {
            self.shared.lock().unwrap().settings_reloaded = false;
            self.settings = Settings::load(&self.layout.settings_file());
            t::set_lang(self.settings.ui_lang());
        }

        if let Some(at) = self.quit_at {
            if std::time::Instant::now() >= at && !self.closing {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        if snap.exit_requested && !self.closing {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        // Fermeture : on arrête d'abord les services, puis on laisse la fenêtre se fermer.
        if ctx.input(|i| i.viewport().close_requested()) && (snap.state.services_up() || snap.state.is_busy()) {
            if !self.closing {
                self.closing = true;
                self.send(Command::StopAll);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }
        if self.closing && !(snap.state.services_up() || snap.state.is_busy()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Fond : dégradé sombre et logo en filigrane, peints sur le calque d'arrière-plan avant les panneaux.
        {
            let screen = ctx.screen_rect();
            let painter = ctx.layer_painter(egui::LayerId::background());
            let mut mesh = egui::Mesh::default();
            let top = Color32::from_rgb(34, 24, 14);
            let bottom = Color32::from_rgb(8, 7, 12);
            mesh.colored_vertex(screen.left_top(), top);
            mesh.colored_vertex(screen.right_top(), top);
            mesh.colored_vertex(screen.right_bottom(), bottom);
            mesh.colored_vertex(screen.left_bottom(), bottom);
            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(0, 2, 3);
            painter.add(egui::Shape::mesh(mesh));
            let side = screen.height().min(screen.width()) * 0.8;
            let rect = egui::Rect::from_center_size(egui::pos2(screen.right() - side * 0.42, screen.center().y + 30.0), egui::vec2(side, side));
            let img = egui::Image::new(egui::include_image!("../assets/logo-512.png"));
            if let Ok(egui::load::TexturePoll::Ready { texture }) = img.load_for_size(ctx, rect.size()) {
                painter.image(texture.id, rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), Color32::from_white_alpha(18));
            }
        }
        let panel_frame = egui::Frame::new().inner_margin(8.0).fill(Color32::from_black_alpha(110));

        egui::TopBottomPanel::top("titre").frame(panel_frame).show(ctx, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.add(egui::Image::new(egui::include_image!("../assets/logo-512.png")).fit_to_exact_size(egui::vec2(56.0, 56.0)));
                ui.add_space(8.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new("HomeRPG ArchLord").size(26.0).strong().color(GOLD));
                    ui.label(RichText::new(t::SUBTITLE.s()).italics().color(Color32::from_gray(190)));
                });
            });
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Home, t::TAB_HOME.s());
                ui.selectable_value(&mut self.tab, Tab::Accounts, t::TAB_ACCOUNTS.s());
                ui.selectable_value(&mut self.tab, Tab::World, t::TAB_WORLD.s());
                ui.selectable_value(&mut self.tab, Tab::Packs, t::TAB_PACKS.s());
                ui.selectable_value(&mut self.tab, Tab::Database, t::TAB_DATABASE.s());
                ui.selectable_value(&mut self.tab, Tab::Backups, t::TAB_BACKUPS.s());
                ui.selectable_value(&mut self.tab, Tab::Settings, t::TAB_SETTINGS.s());
                ui.selectable_value(&mut self.tab, Tab::Logs, t::TAB_LOGS.s());
                ui.selectable_value(&mut self.tab, Tab::Help, t::TAB_HELP.s());
            });
            ui.add_space(4.0);
        });

        egui::TopBottomPanel::bottom("etat").frame(panel_frame).show(ctx, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(RichText::new(snap.state.label()).strong().color(status_color(snap.state.services_up(), snap.state.is_busy())));
                ui.separator();
                ui.label(if self.closing { t::MSG_CLOSING.s() } else { snap.message.as_str() });
            });
            if snap.state.is_busy() {
                ui.add(egui::ProgressBar::new(snap.progress).animate(true));
            }
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().frame(egui::Frame::new().inner_margin(14.0).fill(Color32::from_black_alpha(90))).show(ctx, |ui| match self.tab {
            Tab::Home => self.home(ui, &snap),
            Tab::Accounts => self.accounts(ui, &snap),
            Tab::World => self.world(ui, &snap),
            Tab::Packs => self.packs(ui, &snap),
            Tab::Database => self.database(ui, &snap),
            Tab::Backups => self.backups(ui, &snap),
            Tab::Settings => self.settings_tab(ui, &snap),
            Tab::Logs => self.logs(ui, &snap),
            Tab::Help => self.help(ui),
        });

        self.confirm_window(ctx);
        self.report_window(ctx, &snap);
    }
}

impl App {
    fn home(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let busy = snap.state.is_busy();
        if snap.readonly {
            ui.colored_label(WARN, RichText::new(t::RO_TITLE.s()).strong());
            ui.label(t::RO_TEXT.s());
            ui.add_space(6.0);
            if ui.add_enabled(!busy, egui::Button::new(RichText::new(t::RO_INSTALL.s()).size(18.0).strong()).min_size(egui::vec2(220.0, 40.0))).clicked() {
                if let Some(dir) = rfd::FileDialog::new().set_title(t::RO_PICK_TITLE.s()).pick_folder() {
                    let dest = if dir.file_name().map(|n| n.to_string_lossy().contains("HomeRPG")).unwrap_or(false) { dir } else { dir.join("HomeRPG ArchLord") };
                    self.send(Command::InstallTo(dest));
                }
            }
            if let Some(err) = &snap.error {
                ui.colored_label(ERR, err);
            }
            return;
        }
        egui::Grid::new("composants").num_columns(2).spacing([24.0, 8.0]).show(ui, |ui| {
            let client_txt = if snap.client.is_ok() { t::CLIENT_OK.s() } else if snap.client.exe_present { t::CLIENT_REPAIR.s() } else { t::CLIENT_MISSING.s() };
            ui.label(RichText::new(t::COMP_CLIENT.s()).strong());
            ui.label(RichText::new(client_txt).color(status_color(snap.client.is_ok(), false)));
            ui.end_row();
            ui.label(RichText::new(t::COMP_DB.s()).strong());
            ui.label(RichText::new(if snap.db_up { t::RUNNING.s() } else { t::STOPPED.s() }).color(status_color(snap.db_up, busy)));
            ui.end_row();
            let srv = if snap.world_up { t::SERVER_RUNNING.s() } else if busy { t::SERVER_STARTING.s() } else { t::STOPPED.s() };
            ui.label(RichText::new(t::COMP_SERVER.s()).strong());
            ui.label(RichText::new(srv).color(status_color(snap.world_up, busy)));
            ui.end_row();
            ui.label(RichText::new(t::COMP_WORLD.s()).strong());
            match &snap.world {
                Some(w) if w.imported() => {
                    let txt = t::world_summary(w.npcs, w.monsters, w.items);
                    ui.label(RichText::new(txt).color(status_color(w.monsters > 0, true)));
                }
                Some(_) => {
                    ui.label(RichText::new(t::WORLD_EMPTY.s()).color(WARN));
                }
                None if snap.client_ini => {
                    ui.label(RichText::new(t::WORLD_IMPORTED_STOPPED.s()).color(Color32::from_gray(190)));
                }
                None => {
                    ui.label(RichText::new(t::WORLD_EMPTY.s()).color(WARN));
                }
            }
            ui.end_row();
            ui.label(RichText::new(t::COMP_TABLES.s()).strong());
            ui.label(RichText::new(if snap.server_tables { t::TABLES_OK.s() } else { t::TABLES_MISSING.s() }).color(status_color(snap.server_tables, true)));
            ui.end_row();
        });
        ui.add_space(12.0);

        let can_play = snap.state == State::Ready && snap.client.is_ok();
        let can_start = matches!(snap.state, State::Installed | State::Error);
        let idle = snap.state.idle();
        ui.horizontal_wrapped(|ui| {
            let play = ui.add_enabled(can_play || (can_start && snap.client.is_ok()), egui::Button::new(RichText::new(t::BTN_PLAY.s()).size(20.0).strong().color(GOLD)).min_size(egui::vec2(180.0, 44.0)));
            if play.clicked() {
                self.send(Command::Play);
            }
            if ui.add_enabled(can_start, egui::Button::new(t::BTN_START_SERVER.s()).min_size(egui::vec2(0.0, 44.0))).clicked() {
                self.send(Command::StartAll);
            }
            if ui.add_enabled(snap.state.services_up(), egui::Button::new(t::BTN_STOP.s()).min_size(egui::vec2(0.0, 44.0))).clicked() {
                self.send(Command::StopAll);
            }
            if snap.state == State::Playing && ui.add(egui::Button::new(t::BTN_KILL_GAME.s()).min_size(egui::vec2(0.0, 44.0))).on_hover_text(t::BTN_KILL_GAME_HINT.s()).clicked() {
                self.send(Command::KillGame);
            }
        });
        if !snap.client.is_ok() {
            ui.colored_label(WARN, t::HOME_CLIENT_HINT.s());
        } else if snap.world.as_ref().map(|w| !w.imported()).unwrap_or(!snap.client_ini) && snap.state == State::Installed {
            ui.small(t::HOME_FIRST_START.s());
        }
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(!busy, egui::Button::new(t::BTN_VERIFY.s())).clicked() {
                self.send(Command::Verify);
            }
            if ui.add_enabled(idle, egui::Button::new(t::BTN_REPAIR.s())).on_hover_text(t::RPT_REPAIR_HINT.s()).clicked() {
                self.send(Command::Repair);
            }
            if ui.add_enabled(!busy, egui::Button::new(t::BTN_BACKUP.s())).clicked() {
                self.send(Command::Backup("manuelle".into()));
                self.tab = Tab::Backups;
            }
            if ui.add_enabled(idle, egui::Button::new(t::BTN_RESTORE.s())).clicked() {
                self.tab = Tab::Backups;
            }
            if ui.button(t::TAB_SETTINGS.s()).clicked() {
                self.tab = Tab::Settings;
            }
            if ui.button(t::TAB_LOGS.s()).clicked() {
                self.tab = Tab::Logs;
            }
        });
        if let Some(err) = &snap.error {
            ui.add_space(8.0);
            ui.colored_label(ERR, err);
            if ui.small_button(t::LOG_SEE_SERVER.s()).clicked() {
                self.tab = Tab::Logs;
                self.log_source = Source::Server;
            }
        }
        if let Some(n) = &snap.notice {
            ui.colored_label(WARN, n);
        }
        ui.add_space(12.0);
        ui.separator();
        ui.label(RichText::new(t::HOME_DIAG.s()).strong());
        egui::Grid::new("diag").num_columns(2).spacing([24.0, 4.0]).show(ui, |ui| {
            ui.label(t::HOME_INSTALL_DIR.s());
            ui.label(self.layout.root.display().to_string());
            ui.end_row();
            ui.label(t::HOME_VERSION.s());
            ui.label(format!("{}{}", Layout::version(), if snap.archcore_version.is_empty() { String::new() } else { format!(" — ArchCore {}", snap.archcore_version) }));
            ui.end_row();
            ui.label(t::COMP_CLIENT.s());
            ui.label(format!("{} — ArchRust.exe {}", snap.client.dir.display(), snap.client.exe_info));
            ui.end_row();
            if !snap.client.missing.is_empty() {
                ui.label(t::CLIENT_MISSING_DATA.s());
                ui.label(snap.client.missing.join(", "));
                ui.end_row();
            }
            for (name, ok) in &snap.components {
                ui.label(name);
                ui.label(RichText::new(if *ok { t::PRESENT.s() } else { t::ABSENT.s() }).color(status_color(*ok, false)));
                ui.end_row();
            }
            ui.label(t::HOME_PORTS.s());
            ui.label(format!("MariaDB 127.0.0.1:{}, {} {}, {} {}", crate::paths::MARIADB_PORT, t::LOGIN.s(), self.settings.login_port, t::WORLD.s(), self.settings.world_port));
            ui.end_row();
        });
    }

    fn accounts(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(t::ACC_TITLE.s());
        ui.label(t::ACC_HINT.s());
        ui.add_space(8.0);
        let busy = snap.state.is_busy();
        ui.group(|ui| {
            egui::Grid::new("acc_form").num_columns(2).show(ui, |ui| {
                ui.label(t::ACC_USER.s());
                ui.text_edit_singleline(&mut self.acc_user);
                ui.end_row();
                ui.label(t::ACC_PASS.s());
                ui.add(egui::TextEdit::singleline(&mut self.acc_pass).password(true));
                ui.end_row();
                ui.label(t::ACC_GM.s());
                ui.horizontal(|ui| {
                    ui.add(egui::Slider::new(&mut self.acc_gm, 0..=3));
                    ui.small(t::ACC_GM_HINT.s());
                });
                ui.end_row();
            });
            let user = self.acc_user.trim().to_string();
            let user_ok = crate::db::valid_username(&user);
            let pass_ok = crate::db::valid_password(&self.acc_pass);
            if (!user.is_empty() && !user_ok) || (!self.acc_pass.is_empty() && !pass_ok) {
                ui.colored_label(WARN, t::ACC_INVALID.s());
            }
            ui.horizontal_wrapped(|ui| {
                if ui.add_enabled(!busy && user_ok && pass_ok, egui::Button::new(t::ACC_CREATE.s())).clicked() {
                    self.send(Command::CreateAccount { user: user.clone(), pass: self.acc_pass.clone(), gm: self.acc_gm });
                    self.acc_pass.clear();
                }
                let sel = self.acc_selected.clone();
                if ui.add_enabled(!busy && sel.is_some() && pass_ok, egui::Button::new(t::ACC_SET_PASS.s())).on_hover_text(t::ACC_SET_PASS_HINT.s()).clicked() {
                    self.send(Command::SetPassword { user: sel.clone().unwrap(), pass: self.acc_pass.clone() });
                    self.acc_pass.clear();
                }
                if ui.add_enabled(!busy && sel.is_some(), egui::Button::new(t::ACC_SET_GM.s())).clicked() {
                    self.send(Command::SetGm { user: sel.unwrap(), gm: self.acc_gm });
                }
                if ui.add_enabled(!busy, egui::Button::new(t::REFRESH.s())).clicked() {
                    self.send(Command::RefreshAccounts);
                }
            });
        });
        if let Some(n) = &snap.notice {
            ui.colored_label(WARN, n);
        }
        if !snap.db_up && snap.accounts.is_empty() {
            ui.small(t::ACC_DB_TEMP.s());
        }
        ui.add_space(8.0);
        let playing = snap.state == State::Playing;
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().at_least(140.0))
            .column(Column::auto().at_least(80.0))
            .column(Column::auto().at_least(140.0))
            .column(Column::auto().at_least(100.0))
            .column(Column::remainder())
            .header(22.0, |mut h| {
                h.col(|ui| { ui.strong(t::ACC_USER.s()); });
                h.col(|ui| { ui.strong(t::ACC_GM.s()); });
                h.col(|ui| { ui.strong(t::ACC_LAST.s()); });
                h.col(|ui| { ui.strong(t::BK_CHARS.s()); });
                h.col(|_| {});
            })
            .body(|body| {
                body.rows(22.0, snap.accounts.len(), |mut row| {
                    let a = &snap.accounts[row.index()];
                    let selected = self.acc_selected.as_deref() == Some(a.username.as_str());
                    row.set_selected(selected);
                    row.col(|ui| { if ui.selectable_label(selected, &a.username).clicked() { self.acc_selected = Some(a.username.clone()); self.acc_gm = a.gmlevel.min(3); } });
                    row.col(|ui| { ui.label(a.gmlevel.to_string()); });
                    row.col(|ui| { ui.label(&a.last_login); });
                    row.col(|ui| { ui.label(a.characters.to_string()); });
                    row.col(|ui| {
                        if ui.add_enabled(!playing && !busy, egui::Button::new(RichText::new(t::ACC_DELETE.s()).color(ERR))).clicked() {
                            let detail = format!("{} — {} {}", a.username, a.characters, t::BK_CHARS.s().to_lowercase());
                            self.pending = Some(Pending::DeleteAccount(a.username.clone(), detail));
                        }
                    });
                });
            });
    }

    /// Rendu générique d'un catalogue d'options (voir conf_catalog) sur les surcharges `conf`.
    fn render_options(ui: &mut egui::Ui, catalog: &[conf_catalog::Opt], conf: &mut std::collections::BTreeMap<String, String>, id: &str) {
        let mut groups: Vec<&str> = Vec::new();
        for o in catalog {
            if !groups.contains(&o.group.s()) {
                groups.push(o.group.s());
            }
        }
        for g in groups {
            ui.add_space(6.0);
            ui.label(RichText::new(g).strong().color(GOLD));
            egui::Grid::new(format!("{id}-{g}")).num_columns(3).spacing([14.0, 6.0]).striped(true).show(ui, |ui| {
                for o in catalog.iter().filter(|o| o.group.s() == g) {
                    let current = conf.get(o.key).cloned().unwrap_or_else(|| o.default.to_string());
                    let mut new_value: Option<String> = None;
                    let r = ui.label(o.label.s());
                    if !o.help.s().is_empty() {
                        r.on_hover_text(o.help.s());
                    }
                    match o.kind {
                        conf_catalog::Kind::Bool => {
                            let mut b = current == "1";
                            if ui.checkbox(&mut b, "").changed() {
                                new_value = Some(if b { "1".into() } else { "0".into() });
                            }
                        }
                        conf_catalog::Kind::Int { min, max } => {
                            let mut v: i64 = current.parse().unwrap_or(0);
                            if ui.add(egui::Slider::new(&mut v, min..=max)).changed() {
                                new_value = Some(v.to_string());
                            }
                        }
                        conf_catalog::Kind::Float { min, max } => {
                            let mut v: f64 = current.parse().unwrap_or(1.0);
                            if ui.add(egui::Slider::new(&mut v, min..=max).step_by(0.1).fixed_decimals(1)).changed() {
                                new_value = Some(format!("{v:.1}").trim_end_matches(".0").to_string());
                            }
                        }
                        conf_catalog::Kind::Choice(items) => {
                            let label = items.iter().find(|(v, _)| *v == current).map(|(_, l)| l.s()).unwrap_or(current.as_str());
                            egui::ComboBox::from_id_salt(format!("{id}-{}", o.key)).selected_text(label).show_ui(ui, |ui| {
                                for (v, l) in items {
                                    if ui.selectable_label(*v == current, l.s()).clicked() {
                                        new_value = Some(v.to_string());
                                    }
                                }
                            });
                        }
                    }
                    let modified = conf.get(o.key).map(String::as_str).map(|v| v != o.default).unwrap_or(false);
                    ui.horizontal(|ui| {
                        if modified {
                            ui.small(RichText::new(t::OPT_MODIFIED.s()).color(WARN));
                            if ui.small_button(t::OPT_DEFAULT.s()).clicked() {
                                conf.remove(o.key);
                            }
                        }
                    });
                    ui.end_row();
                    if let Some(v) = new_value {
                        if v == o.default {
                            conf.remove(o.key);
                        } else {
                            conf.insert(o.key.to_string(), v);
                        }
                    }
                }
            });
        }
    }

    fn save_settings(&mut self) {
        match self.settings.save(&self.layout.settings_file()) {
            Ok(()) => {
                self.settings_notice = Some(t::SET_SAVED.s().into());
                self.send(Command::SettingsChanged(self.settings.clone()));
            }
            Err(e) => self.settings_notice = Some(e),
        }
    }

    fn world(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(t::WORLD_TITLE.s());
        ui.label(t::WORLD_INTRO.s());
        if snap.state.services_up() {
            ui.colored_label(WARN, t::SET_RESTART_NEEDED.s());
        }
        ui.horizontal(|ui| {
            if ui.button(t::SAVE.s()).clicked() {
                self.save_settings();
            }
            if ui.button(t::OPT_RESET_ALL.s()).clicked() {
                for o in conf_catalog::WORLD_OPTIONS {
                    self.settings.conf.remove(o.key);
                }
            }
            if let Some(n) = &self.settings_notice {
                ui.label(n);
            }
        });
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            Self::render_options(ui, conf_catalog::WORLD_OPTIONS, &mut self.settings.conf, "world");
            ui.add_space(12.0);
            ui.separator();
            ui.label(RichText::new(t::WORLD_DATA_TITLE.s()).strong().color(GOLD));
            ui.label(t::WORLD_DATA_TEXT.s());
            let idle = snap.state.idle();
            if ui.add_enabled(idle, egui::Button::new(t::BTN_WORLD_REIMPORT.s())).on_hover_text(t::WORLD_REIMPORT_HINT.s()).clicked() {
                self.pending = Some(Pending::ReimportWorld);
            }
            ui.small(t::WORLD_REIMPORT_HINT.s());
        });
    }

    fn packs(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(t::PACK_TITLE.s());
        ui.label(t::PACK_INTRO.s());
        let playing = snap.state == State::Playing;
        if playing {
            ui.colored_label(ERR, t::CLIENT_FILES_PLAYING.s());
        }
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            if ui.add_enabled(!playing, egui::Button::new(t::PACK_ADD_FILE.s())).clicked() {
                if let Some(f) = rfd::FileDialog::new().set_title(t::PACK_ADD_FILE.s()).add_filter("ArchRust (.ahp)", &["ahp"]).pick_file() {
                    self.send(Command::PackAdd(f));
                }
            }
            if ui.add_enabled(!playing, egui::Button::new(t::PACK_ADD_DIR.s())).clicked() {
                if let Some(d) = rfd::FileDialog::new().set_title(t::PACK_ADD_DIR.s()).pick_folder() {
                    self.send(Command::PackAdd(d));
                }
            }
            if ui.button(t::OPEN_DIR.s()).clicked() {
                let dir = crate::packs::patches_dir(&snap.client.dir);
                let _ = std::fs::create_dir_all(&dir);
                open_in_explorer(&dir);
            }
            if ui.button(t::REFRESH.s()).clicked() {
                self.send(Command::RefreshPacks);
            }
        });
        if let Some(n) = &snap.notice {
            ui.colored_label(WARN, n);
        }
        ui.add_space(6.0);
        let mut action: Option<Command> = None;
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            if snap.packs.is_empty() {
                ui.label(t::PACK_NONE.s());
            }
            egui::Grid::new("packs").num_columns(6).striped(true).spacing([12.0, 4.0]).show(ui, |ui| {
                for h in [t::PACK_NAME.s(), t::PACK_VERSION.s(), t::PACK_AUTHOR.s(), t::PACK_SIZE.s(), t::PACK_STATE.s(), ""] {
                    ui.strong(h);
                }
                ui.end_row();
                for p in &snap.packs {
                    let r = ui.label(&p.name);
                    let mut hover = format!("id : {}\n{}: {}", p.id, t::PACK_PRIORITY.s(), p.priority);
                    if !p.description.is_empty() {
                        hover = format!("{}\n{hover}", p.description);
                    }
                    r.on_hover_text(hover);
                    ui.label(&p.version);
                    ui.label(&p.author);
                    ui.label(util::human_size(p.size));
                    if let Some(e) = &p.error {
                        ui.colored_label(ERR, e);
                    } else if p.official {
                        ui.label(t::PACK_OFFICIAL_LABEL.s());
                    } else if p.enabled {
                        ui.colored_label(status_color(true, false), t::ENABLED_LABEL.s());
                    } else {
                        ui.colored_label(ERR, t::DISABLED_LABEL.s());
                    }
                    ui.horizontal(|ui| {
                        if !p.official {
                            if ui.add_enabled(!playing, egui::Button::new(if p.enabled { t::DISABLE.s() } else { t::ENABLE.s() })).clicked() {
                                action = Some(Command::PackSetEnabled(p.id.clone(), !p.enabled));
                            }
                            if ui.add_enabled(!playing, egui::Button::new(t::REMOVE.s())).clicked() {
                                self.pending = Some(Pending::RemovePack(p.id.clone()));
                            }
                        }
                    });
                    ui.end_row();
                }
            });
            ui.small(t::PACK_RESTART_NOTE.s());
            ui.add_space(12.0);
            self.addons_section(ui, snap, &mut action);
        });
        if let Some(c) = action {
            self.send(c);
        }
    }

    fn addons_section(&mut self, ui: &mut egui::Ui, snap: &Snapshot, action: &mut Option<Command>) {
        ui.separator();
        ui.label(RichText::new(t::ADDON_TITLE.s()).strong().color(GOLD));
        ui.small(t::ADDON_INTRO.s());
        let playing = snap.state == State::Playing;
        ui.horizontal(|ui| {
            if ui.add_enabled(!playing, egui::Button::new(t::ADDON_ADD.s())).clicked() {
                if let Some(f) = rfd::FileDialog::new().set_title(t::ADDON_ADD.s()).add_filter("zip", &["zip"]).pick_file() {
                    self.send(Command::AddonAdd(f));
                }
            }
            if ui.button(t::OPEN_DIR.s()).clicked() {
                let dir = crate::addons::addons_dir(&snap.client.dir);
                let _ = std::fs::create_dir_all(&dir);
                open_in_explorer(&dir);
            }
        });
        if snap.addons.is_empty() {
            ui.label(t::ADDON_NONE.s());
            return;
        }
        egui::Grid::new("addons").num_columns(5).striped(true).spacing([12.0, 4.0]).show(ui, |ui| {
            for h in [t::ADDON_NAME.s(), t::ADDON_TITLE_COL.s(), t::PACK_VERSION.s(), t::PACK_STATE.s(), ""] {
                ui.strong(h);
            }
            ui.end_row();
            for a in &snap.addons {
                ui.label(&a.name);
                let r = ui.label(&a.title);
                if !a.interface.is_empty() {
                    r.on_hover_text(format!("Interface : {}", a.interface));
                }
                ui.label(&a.version);
                if !a.has_toc {
                    ui.colored_label(WARN, t::ADDON_NO_TOC.s());
                } else if a.active {
                    ui.colored_label(status_color(true, false), t::ENABLED_LABEL.s());
                } else {
                    ui.colored_label(ERR, t::DISABLED_LABEL.s());
                }
                ui.horizontal(|ui| {
                    if ui.add_enabled(!playing, egui::Button::new(if a.active { t::DISABLE.s() } else { t::ENABLE.s() })).clicked() {
                        *action = Some(Command::AddonSetActive(a.name.clone(), !a.active));
                    }
                    if ui.add_enabled(!playing, egui::Button::new(t::REMOVE.s())).clicked() {
                        self.pending = Some(Pending::RemoveAddon(a.name.clone()));
                    }
                });
                ui.end_row();
            }
        });
    }

    fn load_table(&self, offset: u64) {
        if self.table_selected.is_empty() {
            return;
        }
        self.send(Command::LoadTable { db: self.db_selected.clone(), table: self.table_selected.clone(), filter: self.db_filter.clone(), order: self.db_order.clone(), offset });
    }

    /// Clause WHERE identifiant une ligne par sa clé primaire (None si la table n'en a pas).
    fn pk_where(view: &crate::orchestrator::TableView, row: &[String]) -> Option<String> {
        let parts: Vec<String> = view
            .columns
            .iter()
            .enumerate()
            .filter(|(_, c)| c.primary)
            .map(|(i, c)| format!("`{}`={}", c.name, crate::db::sql_literal(row.get(i).map(String::as_str).unwrap_or("NULL"))))
            .collect();
        if parts.is_empty() { None } else { Some(parts.join(" AND ")) }
    }

    fn database(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(t::DB_TITLE.s());
        ui.small(t::DB_INTRO.s());
        ui.horizontal(|ui| {
            if ui.add_enabled(snap.db_up, egui::Button::new(t::DB_OPEN_HEIDI.s())).clicked() {
                self.send(Command::OpenHeidi);
            }
            if !snap.db_up {
                ui.colored_label(WARN, t::DB_NEED_DB.s());
            }
        });
        if snap.table_dirty && snap.db_up {
            let off = snap.table_view.as_ref().map(|v| v.offset).unwrap_or(0);
            self.load_table(off);
        }
        // Liste des tables de la base courante chargée dès que la base répond (une seule demande par base).
        if snap.db_up && snap.tables.is_empty() && self.tables_requested_for.as_deref() != Some(self.db_selected.as_str()) {
            self.tables_requested_for = Some(self.db_selected.clone());
            self.send(Command::ListTables(self.db_selected.clone()));
        }
        ui.add_space(4.0);
        egui::SidePanel::left("db_tree").resizable(true).default_width(230.0).show_inside(ui, |ui| {
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                for d in snap.databases.clone() {
                    let open = self.db_selected == d;
                    let header = egui::CollapsingHeader::new(RichText::new(&d).strong()).default_open(open).show(ui, |ui| {
                        if open {
                            if snap.tables.is_empty() && snap.db_up {
                                ui.small("…");
                            }
                            for tb in &snap.tables {
                                let sel = self.table_selected == *tb;
                                if ui.selectable_label(sel, tb).clicked() {
                                    self.table_selected = tb.clone();
                                    self.db_filter.clear();
                                    self.db_order = None;
                                    self.editing = None;
                                    self.load_table(0);
                                }
                            }
                        }
                    });
                    if header.header_response.clicked() && !open && snap.db_up {
                        self.db_selected = d.clone();
                        self.table_selected.clear();
                        self.tables_requested_for = Some(d.clone());
                        self.send(Command::ListTables(d.clone()));
                    }
                }
            });
        });
        egui::CentralPanel::default().frame(egui::Frame::NONE).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.db_tab, 0, t::DB_TAB_DATA.s());
                ui.selectable_value(&mut self.db_tab, 1, t::DB_TAB_STRUCTURE.s());
                ui.selectable_value(&mut self.db_tab, 2, "SQL");
                if !self.table_selected.is_empty() {
                    ui.separator();
                    ui.label(RichText::new(format!("{}.{}", self.db_selected, self.table_selected)).strong());
                }
            });
            ui.separator();
            match self.db_tab {
                0 => self.db_data_tab(ui, snap),
                1 => self.db_structure_tab(ui, snap),
                _ => self.db_sql_tab(ui, snap),
            }
        });
    }

    fn db_data_tab(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(view) = snap.table_view.as_ref() else {
            if let Some(Err(e)) = &snap.query {
                ui.colored_label(ERR, e);
            }
            return;
        };
        let has_pk = view.columns.iter().any(|c| c.primary);
        ui.horizontal(|ui| {
            ui.label(t::DB_FILTER.s());
            let r = ui.add(egui::TextEdit::singleline(&mut self.db_filter).desired_width(320.0).hint_text("level > 10 AND account_id = 1"));
            if ui.button(t::DB_APPLY_FILTER.s()).clicked() || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                self.load_table(0);
            }
            ui.separator();
            if ui.add_enabled(view.offset > 0, egui::Button::new("◀")).clicked() {
                self.load_table(view.offset.saturating_sub(200));
            }
            ui.label(format!("{}–{} / {}", view.offset + 1, (view.offset + view.page.rows.len() as u64).min(view.total), view.total));
            if ui.add_enabled(view.offset + 200 < view.total, egui::Button::new("▶")).clicked() {
                self.load_table(view.offset + 200);
            }
            ui.separator();
            if ui.add_enabled(has_pk, egui::Button::new(t::DB_INSERT.s())).clicked() {
                self.insert_form = Some(view.columns.iter().map(|_| String::new()).collect());
            }
        });
        if !has_pk {
            ui.colored_label(WARN, t::DB_NO_PK.s());
        } else {
            ui.small(t::DB_EDIT_HINT.s());
        }
        if let Some(Err(e)) = &snap.query {
            ui.colored_label(ERR, e);
        }
        // Formulaire d'insertion
        if let Some(form) = self.insert_form.as_mut() {
            let mut submit = false;
            let mut cancel = false;
            egui::Window::new(t::DB_INSERT.s()).collapsible(false).show(ui.ctx(), |ui| {
                ui.small(t::DB_INSERT_HINT.s());
                egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                    egui::Grid::new("insert_form").num_columns(2).show(ui, |ui| {
                        for (i, c) in view.columns.iter().enumerate() {
                            ui.label(format!("{} ({})", c.name, c.col_type));
                            ui.add(egui::TextEdit::singleline(&mut form[i]).desired_width(260.0).hint_text(c.default.clone().unwrap_or_default()));
                            ui.end_row();
                        }
                    });
                });
                ui.horizontal(|ui| {
                    if ui.button(t::CONFIRM.s()).clicked() {
                        submit = true;
                    }
                    if ui.button(t::CANCEL.s()).clicked() {
                        cancel = true;
                    }
                });
            });
            if submit {
                let cols: Vec<String> = view.columns.iter().zip(form.iter()).filter(|(_, v)| !v.is_empty()).map(|(c, _)| format!("`{}`", c.name)).collect();
                let vals: Vec<String> = form.iter().filter(|v| !v.is_empty()).map(|v| crate::db::sql_literal(v)).collect();
                let sql = if cols.is_empty() { format!("INSERT INTO `{}` () VALUES ()", view.table) } else { format!("INSERT INTO `{}` ({}) VALUES ({})", view.table, cols.join(","), vals.join(",")) };
                self.send(Command::Exec { db: view.db.clone(), sql });
                self.insert_form = None;
            } else if cancel {
                self.insert_form = None;
            }
        }
        // Grille
        let ncols = view.columns.len().max(1);
        let mut pending_sql: Option<String> = None;
        let mut delete_row: Option<usize> = None;
        let mut sort_click: Option<usize> = None;
        egui::ScrollArea::horizontal().show(ui, |ui| {
            let mut tb = TableBuilder::new(ui).striped(true).resizable(true).vscroll(true).max_scroll_height(f32::INFINITY).column(Column::auto().at_least(28.0));
            for _ in 0..ncols {
                tb = tb.column(Column::auto().at_least(70.0).clip(true));
            }
            tb.header(22.0, |mut h| {
                h.col(|_| {});
                for (ci, c) in view.columns.iter().enumerate() {
                    h.col(|ui| {
                        let mark = match &self.db_order {
                            Some((col, asc)) if *col == c.name => {
                                if *asc { " ▲" } else { " ▼" }
                            }
                            _ => "",
                        };
                        if ui.add(egui::Label::new(RichText::new(format!("{}{}{}", if c.primary { "🔑 " } else { "" }, c.name, mark)).strong()).sense(egui::Sense::click())).clicked() {
                            sort_click = Some(ci);
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(20.0, view.page.rows.len(), |mut row| {
                    let ri = row.index();
                    let data = &view.page.rows[ri];
                    row.col(|ui| {
                        if has_pk && ui.add(egui::Button::new("✖").small()).on_hover_text(t::DB_DELETE_ROW.s()).clicked() {
                            delete_row = Some(ri);
                        }
                    });
                    for (ci, v) in data.iter().enumerate() {
                        row.col(|ui| {
                            let editing_here = matches!(self.editing, Some((r, c, _)) if r == ri && c == ci);
                            if editing_here {
                                let (_, _, text) = self.editing.as_mut().unwrap();
                                let resp = ui.add(egui::TextEdit::singleline(text).desired_width(f32::INFINITY));
                                resp.request_focus();
                                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                                    if let Some(w) = Self::pk_where(view, data) {
                                        let new_val = text.clone();
                                        if new_val != *v {
                                            pending_sql = Some(format!("UPDATE `{}` SET `{}`={} WHERE {} LIMIT 1", view.table, view.columns[ci].name, crate::db::sql_literal(&new_val), w));
                                        }
                                    }
                                    self.editing = None;
                                } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                    self.editing = None;
                                }
                            } else {
                                let r = ui.add(egui::Label::new(v).sense(egui::Sense::click()));
                                if r.double_clicked() && has_pk {
                                    self.editing = Some((ri, ci, v.clone()));
                                }
                            }
                        });
                    }
                });
            });
        });
        if let Some(ci) = sort_click {
            let col = view.columns[ci].name.clone();
            self.db_order = match &self.db_order {
                Some((c, true)) if *c == col => Some((col, false)),
                Some((c, false)) if *c == col => None,
                _ => Some((col, true)),
            };
            self.load_table(0);
        }
        if let Some(sql) = pending_sql {
            self.send(Command::Exec { db: view.db.clone(), sql });
        }
        if let Some(ri) = delete_row {
            if let Some(w) = Self::pk_where(view, &view.page.rows[ri]) {
                self.pending = Some(Pending::ExecSql(view.db.clone(), format!("DELETE FROM `{}` WHERE {} LIMIT 1", view.table, w), t::DB_DELETE_ROW, t::DB_DELETE_ROW_CONFIRM));
            }
        }
    }

    fn db_structure_tab(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        let Some(view) = snap.table_view.as_ref() else { return };
        egui::Grid::new("structure").num_columns(6).striped(true).spacing([14.0, 4.0]).show(ui, |ui| {
            for h in [t::DB_COL_NAME.s(), "Type", "NULL", t::DB_COL_KEY.s(), t::DB_COL_DEFAULT.s(), "Extra"] {
                ui.strong(h);
            }
            ui.end_row();
            for c in &view.columns {
                ui.label(&c.name);
                ui.label(&c.col_type);
                ui.label(if c.nullable { t::YES.s() } else { t::NO.s() });
                ui.label(if c.primary { "PRIMARY" } else { "" });
                ui.label(c.default.clone().unwrap_or_else(|| "NULL".into()));
                ui.label(&c.extra);
                ui.end_row();
            }
        });
    }

    fn db_sql_tab(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.add(egui::TextEdit::multiline(&mut self.sql).desired_rows(5).desired_width(f32::INFINITY).code_editor());
        ui.horizontal(|ui| {
            let is_read = crate::db::is_read_statement(&self.sql);
            if ui.add_enabled(snap.db_up && is_read, egui::Button::new(t::DB_RUN_READ.s())).clicked() {
                self.sql_history.push(self.sql.clone());
                self.send(Command::Query { db: self.db_selected.clone(), sql: self.sql.clone() });
            }
            if ui.add_enabled(snap.db_up && !is_read && !self.sql.trim().is_empty(), egui::Button::new(RichText::new(t::DB_RUN_WRITE.s()).color(WARN))).clicked() {
                self.sql_history.push(self.sql.clone());
                self.pending = Some(Pending::ExecSql(self.db_selected.clone(), self.sql.clone(), t::DB_WRITE_CONFIRM_TITLE, t::DB_WRITE_CONFIRM));
            }
            if !self.sql_history.is_empty() {
                egui::ComboBox::from_id_salt("sql_hist").selected_text(t::DB_HISTORY.s()).show_ui(ui, |ui| {
                    for h in self.sql_history.iter().rev().take(20) {
                        if ui.selectable_label(false, h.lines().next().unwrap_or("").chars().take(80).collect::<String>()).clicked() {
                            self.sql = h.clone();
                        }
                    }
                });
            }
        });
        match &snap.query {
            None => {}
            Some(Err(e)) => {
                ui.colored_label(ERR, e);
            }
            Some(Ok(r)) if r.affected.is_some() => {
                ui.colored_label(status_color(true, false), format!("{} {}", r.affected.unwrap(), t::DB_AFFECTED.s()));
            }
            Some(Ok(r)) => {
                ui.label(format!("{} {}{}", r.rows.len(), t::DB_ROWS.s(), if r.truncated { " (500 max)" } else { "" }));
                let ncols = r.columns.len().max(1);
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    let mut tb = TableBuilder::new(ui).striped(true).resizable(true).vscroll(true).max_scroll_height(f32::INFINITY);
                    for _ in 0..ncols {
                        tb = tb.column(Column::auto().at_least(60.0).clip(true));
                    }
                    tb.header(22.0, |mut h| {
                        for c in &r.columns {
                            h.col(|ui| {
                                ui.strong(c);
                            });
                        }
                    })
                    .body(|body| {
                        body.rows(18.0, r.rows.len(), |mut row| {
                            let data = &r.rows[row.index()];
                            for v in data {
                                row.col(|ui| {
                                    ui.label(v);
                                });
                            }
                        });
                    });
                });
            }
        }
    }

    fn backups(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(t::BK_TITLE.s());
        ui.label(t::BK_HINT.s());
        if snap.state == State::Playing {
            ui.colored_label(WARN, t::BK_PLAYING_WARN.s());
        }
        if let Some(n) = &snap.notice {
            ui.colored_label(WARN, n);
        }
        ui.add_space(6.0);
        let busy = snap.state.is_busy();
        let idle = snap.state.idle();
        ui.horizontal_wrapped(|ui| {
            ui.label(t::BK_LABEL.s());
            ui.add(egui::TextEdit::singleline(&mut self.backup_label).desired_width(200.0));
            if ui.add_enabled(!busy, egui::Button::new(t::BK_NOW.s())).clicked() {
                let label = if self.backup_label.trim().is_empty() { "manuelle".to_string() } else { self.backup_label.trim().to_string() };
                self.send(Command::Backup(label));
                self.backup_label.clear();
            }
            let sel = self.backup_selected.clone();
            let sel_info = sel.as_ref().and_then(|p| snap.backups.iter().find(|b| &b.path == p));
            if ui.add_enabled(idle && sel_info.is_some(), egui::Button::new(t::BK_RESTORE_SEL.s())).clicked() {
                let b = sel_info.unwrap();
                self.pending = Some(Pending::Restore(b.path.clone(), format!("{} — {} {}, {} {}", b.manifest.created, b.manifest.characters, t::BK_CHARS.s().to_lowercase(), b.manifest.accounts, t::BK_ACCOUNTS.s().to_lowercase())));
            }
            if ui.add_enabled(!busy && sel_info.is_some(), egui::Button::new(t::BK_DELETE_SEL.s())).clicked() {
                let b = sel_info.unwrap();
                self.pending = Some(Pending::Delete(b.path.clone(), b.manifest.created.clone()));
            }
            if ui.button(t::OPEN_DIR.s()).clicked() {
                let dir = self.layout.backups();
                let _ = std::fs::create_dir_all(&dir);
                open_in_explorer(&dir);
            }
            if ui.add_enabled(!busy, egui::Button::new(t::BK_DIAG.s())).on_hover_text(t::BK_DIAG_HINT.s()).clicked() {
                self.send(Command::ExportDiagnostic);
            }
            if ui.button(t::REFRESH.s()).clicked() {
                self.send(Command::RefreshBackups);
            }
        });
        ui.add_space(8.0);
        if snap.backups.is_empty() {
            ui.label(t::BK_NONE.s());
            return;
        }
        TableBuilder::new(ui)
            .striped(true)
            .column(Column::auto().at_least(150.0))
            .column(Column::auto().at_least(180.0))
            .column(Column::auto().at_least(90.0))
            .column(Column::auto().at_least(70.0))
            .column(Column::remainder())
            .header(22.0, |mut h| {
                h.col(|ui| { ui.strong(t::BK_DATE.s()); });
                h.col(|ui| { ui.strong(t::BK_LABEL.s()); });
                h.col(|ui| { ui.strong(t::BK_CHARS.s()); });
                h.col(|ui| { ui.strong(t::BK_ACCOUNTS.s()); });
                h.col(|ui| { ui.strong(t::PACK_SIZE.s()); });
            })
            .body(|body| {
                body.rows(20.0, snap.backups.len(), |mut row| {
                    let b = &snap.backups[row.index()];
                    let selected = self.backup_selected.as_ref() == Some(&b.path);
                    row.set_selected(selected);
                    row.col(|ui| { if ui.selectable_label(selected, &b.manifest.created).clicked() { self.backup_selected = Some(b.path.clone()); } });
                    row.col(|ui| { ui.label(&b.manifest.label); });
                    row.col(|ui| { ui.label(b.manifest.characters.to_string()); });
                    row.col(|ui| { ui.label(b.manifest.accounts.to_string()); });
                    row.col(|ui| { ui.label(util::human_size(b.size)); });
                });
            });
    }

    fn settings_tab(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(t::TAB_SETTINGS.s());
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            // ---- Langues
            ui.label(RichText::new(t::SET_LANG_TITLE.s()).strong().color(GOLD));
            ui.horizontal(|ui| {
                ui.label(t::SET_UI_LANG.s());
                let current = match self.settings.ui_language {
                    None => t::SET_LANG_AUTO.s().to_string(),
                    Some(l) => l.native_name().to_string(),
                };
                let mut chosen: Option<Option<Lang>> = None;
                egui::ComboBox::from_id_salt("ui_lang").selected_text(current).show_ui(ui, |ui| {
                    if ui.selectable_label(self.settings.ui_language.is_none(), t::SET_LANG_AUTO.s()).clicked() {
                        chosen = Some(None);
                    }
                    for l in Lang::ALL {
                        if ui.selectable_label(self.settings.ui_language == Some(l), l.native_name()).clicked() {
                            chosen = Some(Some(l));
                        }
                    }
                });
                if let Some(c) = chosen {
                    // Interface traduite tout de suite, réglage enregistré.
                    self.settings.ui_language = c;
                    t::set_lang(self.settings.ui_lang());
                    self.save_settings();
                }
            });
            ui.horizontal(|ui| {
                ui.label(t::SET_GAME_LANG.s());
                for l in Lang::ALL {
                    ui.radio_value(&mut self.settings.game_language, l, l.native_name());
                }
            });
            ui.small(t::SET_GAME_LANG_HINT.s());

            // ---- Client
            ui.add_space(10.0);
            ui.label(RichText::new(t::SET_CLIENT_DIR.s()).strong().color(GOLD));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.settings.client_dir).desired_width(480.0).hint_text(self.layout.default_client_dir().display().to_string()));
                if ui.button(t::BROWSE.s()).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().set_title(t::SET_CLIENT_DIR.s()).pick_folder() {
                        self.settings.client_dir = dir.to_string_lossy().into_owned();
                    }
                }
            });
            ui.small(t::SET_CLIENT_HINT.s());
            ui.label(t::SET_GAME_DIR.s());
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.settings.game_dir).desired_width(480.0).hint_text(t::SET_GAME_DIR_AUTO.s()));
                if ui.button(t::BROWSE.s()).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().set_title(t::SET_GAME_DIR.s()).pick_folder() {
                        self.settings.game_dir = dir.to_string_lossy().into_owned();
                    }
                }
            });
            ui.small(t::SET_GAME_DIR_HINT.s());
            // Client détecté dans les dossiers saisis (recalculé quand l'un des deux change).
            let typed = format!("{}|{}", self.settings.client_dir.trim(), self.settings.game_dir.trim());
            if self.client_probe.as_ref().map(|(p, _)| p != &typed).unwrap_or(true) {
                self.client_probe = Some((typed.clone(), crate::client::from_settings(&self.layout, &self.settings)));
            }
            let probe = self.client_probe.as_ref().map(|(_, i)| i.clone()).unwrap_or_default();
            ui.horizontal_wrapped(|ui| {
                ui.label(t::SET_CLIENT_DETECTED.s());
                let txt = if probe.is_ok() {
                    format!("ArchRust — {}", probe.exe_info)
                } else if !probe.exe_present {
                    t::CLIENT_MISSING.s().to_string()
                } else {
                    format!("{} {}", t::CLIENT_MISSING_DATA.s(), probe.missing.join(", "))
                };
                ui.label(RichText::new(txt).color(status_color(probe.is_ok(), false)).strong());
            });
            if probe.is_ok() && probe.separate_data() {
                ui.small(format!("{} {}", t::SET_GAME_DIR_USED.s(), probe.game_dir.display()));
            }
            if !probe.is_ok() && (probe.exe_present || probe.source_tree) {
                ui.colored_label(WARN, if probe.source_tree { t::CLIENT_SOURCE_TREE.s() } else { t::CLIENT_NO_DATA_HINT.s() });
            }
            // Recherche hors du fil de l'interface ; parcourt le launcher et vérifie aussi le chemin configuré.
            if let Some(rx) = &self.discover_rx {
                if let Ok(found) = rx.try_recv() {
                    self.discovered = Some(found);
                    self.discover_rx = None;
                }
            }
            let searching = self.discover_rx.is_some();
            ui.horizontal(|ui| {
                let clicked = ui.add_enabled(!searching, egui::Button::new(t::SET_CLIENT_SCAN.s())).on_hover_text(t::SET_CLIENT_SCAN_HINT.s()).clicked();
                if (clicked || self.discovered.is_none()) && !searching {
                    let (tx, rx) = std::sync::mpsc::channel();
                    let root = self.layout.root.clone();
                    let configured = crate::client::exe_dir(&self.layout, &self.settings);
                    let ctx = ui.ctx().clone();
                    std::thread::spawn(move || {
                        let _ = tx.send(crate::client::discover_with_configured(&root, &configured));
                        ctx.request_repaint();
                    });
                    self.discover_rx = Some(rx);
                    self.discovered = Some(Vec::new());
                }
                if searching {
                    ui.spinner();
                    ui.label(t::SET_CLIENT_SEARCHING.s());
                }
            });
            if let Some(found) = self.discovered.clone() {
                if found.is_empty() && !searching {
                    ui.colored_label(WARN, t::SET_CLIENT_NONE_FOUND.s());
                }
                let mut chosen: Option<PathBuf> = None;
                for c in found.iter() {
                    let path = c.dir.display().to_string();
                    let current = crate::client::same_dir(&probe.dir, &c.dir);
                    ui.horizontal(|ui| {
                        let label = if current { t::SET_CLIENT_IN_USE.s() } else { t::SET_CLIENT_USE.s() };
                        if ui.add_enabled(!current && c.is_ok(), egui::Button::new(label)).clicked() {
                            chosen = Some(c.dir.clone());
                        }
                        if c.is_ok() {
                            ui.label(&path);
                        } else {
                            ui.label(RichText::new(&path).color(Color32::from_gray(150)));
                            let why = if c.source_tree { t::CLIENT_SOURCE_TREE_SHORT.s().to_string() } else { format!("{} {}", t::CLIENT_MISSING_DATA.s(), c.missing.join(", ")) };
                            ui.small(RichText::new(why).color(WARN));
                        }
                    });
                }
                // Choix enregistré tout de suite : le client sert dès le prochain JOUER.
                if let Some(dir) = chosen {
                    self.settings.client_dir = if crate::client::same_dir(&dir, &self.layout.default_client_dir()) { String::new() } else { dir.display().to_string() };
                    // Données du jeu retrouvées automatiquement pour ce client.
                    self.settings.game_dir.clear();
                    self.save_settings();
                }
            }

            // ---- Tables du serveur d'origine
            ui.add_space(10.0);
            ui.label(RichText::new(t::SET_TABLES_DIR.s()).strong().color(GOLD));
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.settings.server_tables_dir).desired_width(480.0).hint_text(self.layout.default_server_tables().display().to_string()));
                if ui.button(t::BROWSE.s()).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().set_title(t::SET_TABLES_DIR.s()).pick_folder() {
                        self.settings.server_tables_dir = dir.to_string_lossy().into_owned();
                    }
                }
            });
            let tables_dir = if self.settings.server_tables_dir.trim().is_empty() { self.layout.default_server_tables() } else { PathBuf::from(self.settings.server_tables_dir.trim()) };
            let present = crate::repair::server_tables_present(&tables_dir);
            ui.colored_label(status_color(present, true), if present { t::TABLES_OK.s() } else { t::TABLES_MISSING.s() });
            ui.small(t::SET_TABLES_HINT.s());

            // ---- Ports
            ui.add_space(10.0);
            ui.label(RichText::new(t::SET_PORTS.s()).strong().color(GOLD));
            ui.horizontal(|ui| {
                ui.label(t::LOGIN.s());
                ui.add(egui::DragValue::new(&mut self.settings.login_port).range(1024..=65535));
                ui.label(t::WORLD.s());
                ui.add(egui::DragValue::new(&mut self.settings.world_port).range(1024..=65535));
                if ui.small_button(t::OPT_DEFAULT.s()).clicked() {
                    self.settings.login_port = crate::paths::DEFAULT_LOGIN_PORT;
                    self.settings.world_port = crate::paths::DEFAULT_WORLD_PORT;
                }
            });
            ui.small(t::SET_PORTS_HINT.s());

            // ---- Sauvegardes
            ui.add_space(10.0);
            ui.label(RichText::new(t::TAB_BACKUPS.s()).strong().color(GOLD));
            ui.checkbox(&mut self.settings.auto_backup, t::SET_AUTO_BACKUP.s());
            ui.horizontal(|ui| {
                ui.label(t::SET_AUTO_BACKUP_KEEP.s());
                ui.add(egui::DragValue::new(&mut self.settings.auto_backup_keep).range(1..=100));
            });

            // ---- Journaux
            ui.add_space(10.0);
            ui.label(RichText::new(t::TAB_LOGS.s()).strong().color(GOLD));
            ui.checkbox(&mut self.settings.server_trace, t::SET_SERVER_TRACE.s());
            ui.small(t::SET_SERVER_TRACE_HINT.s());
            ui.horizontal(|ui| {
                ui.label(t::SET_CLIENT_LOG.s());
                egui::ComboBox::from_id_salt("client_log").selected_text(self.settings.client_log_level.clone()).show_ui(ui, |ui| {
                    for l in ["error", "warn", "info", "debug"] {
                        ui.selectable_value(&mut self.settings.client_log_level, l.to_string(), l);
                    }
                });
            });
            ui.small(t::SET_CLIENT_LOG_HINT.s());

            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new(t::SAVE.s()).strong()).clicked() {
                    self.save_settings();
                }
                if let Some(n) = &self.settings_notice {
                    ui.label(n);
                }
            });
            if snap.state.services_up() {
                ui.colored_label(WARN, t::SET_RESTART_NEEDED.s());
            }
        });
    }

    fn logs(&mut self, ui: &mut egui::Ui, snap: &Snapshot) {
        ui.heading(t::LOG_TITLE.s());
        ui.horizontal_wrapped(|ui| {
            ui.selectable_value(&mut self.log_source, Source::Launcher, t::LOG_LAUNCHER.s());
            ui.selectable_value(&mut self.log_source, Source::Server, t::LOG_SERVER.s());
            ui.selectable_value(&mut self.log_source, Source::Client, t::LOG_CLIENT.s());
            ui.separator();
            ui.label(t::LOG_FILTER.s());
            ui.add(egui::TextEdit::singleline(&mut self.log_filter).desired_width(180.0));
            ui.checkbox(&mut self.log_problems_only, t::LOG_PROBLEMS.s());
        });
        let (lines, file, dir) = match self.log_source {
            Source::Launcher => (&snap.log_launcher, Some(self.layout.launcher_log()), self.layout.logs()),
            Source::Server => (&snap.log_server, snap.server_log_file.clone(), self.layout.server_logs()),
            Source::Client => (&snap.log_client, snap.client_log_file.clone(), self.layout.logs().join("client")),
        };
        ui.horizontal_wrapped(|ui| {
            if ui.button(t::OPEN_DIR.s()).clicked() {
                let _ = std::fs::create_dir_all(&dir);
                open_in_explorer(&dir);
            }
            if let Some(f) = file.as_ref().filter(|f| f.is_file()) {
                if ui.button(t::LOG_OPEN_FILE.s()).clicked() {
                    let _ = std::process::Command::new("notepad.exe").arg(f).spawn();
                }
                ui.small(f.display().to_string());
            }
            if ui.button(t::LOG_CLEAR.s()).on_hover_text(t::LOG_CLEAR_HINT.s()).clicked() {
                self.send(Command::ClearLog(self.log_source));
            }
        });
        if self.log_source == Source::Client && lines.is_empty() {
            ui.small(t::LOG_CLIENT_EMPTY.s());
        }
        if self.log_source == Source::Server && lines.is_empty() {
            ui.small(t::LOG_SERVER_EMPTY.s());
        }
        let filter = self.log_filter.to_lowercase();
        egui::ScrollArea::vertical().stick_to_bottom(true).auto_shrink([false, false]).show(ui, |ui| {
            for l in lines.iter() {
                let level = logs::level(l);
                if self.log_problems_only && level == logs::Level::Info {
                    continue;
                }
                if !filter.is_empty() && !l.to_lowercase().contains(&filter) {
                    continue;
                }
                let color = match level {
                    logs::Level::Error => ERR,
                    logs::Level::Warn => WARN,
                    logs::Level::Info => Color32::from_gray(205),
                };
                ui.label(RichText::new(l).monospace().color(color));
            }
        });
    }

    fn help(&mut self, ui: &mut egui::Ui) {
        ui.heading(t::HELP_TITLE.s());
        ui.label(t::HELP_INTRO.s());
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.add_space(6.0);
            ui.label(RichText::new(t::HELP_GM_TITLE.s()).strong().color(GOLD));
            egui::Grid::new("gm_cmds").num_columns(3).spacing([16.0, 4.0]).striped(true).show(ui, |ui| {
                ui.strong(t::HELP_CMD.s());
                ui.strong(t::ACC_GM.s());
                ui.strong(t::HELP_EFFECT.s());
                ui.end_row();
                for (cmd, lvl, desc) in t::gm_commands() {
                    ui.monospace(cmd);
                    ui.label(lvl);
                    ui.label(desc);
                    ui.end_row();
                }
            });
            ui.add_space(10.0);
            ui.label(RichText::new(t::HELP_KEYS_TITLE.s()).strong().color(GOLD));
            egui::Grid::new("keys").num_columns(2).spacing([16.0, 4.0]).striped(true).show(ui, |ui| {
                for (k, d) in t::game_keys() {
                    ui.monospace(k);
                    ui.label(d);
                    ui.end_row();
                }
            });
            ui.add_space(10.0);
            ui.small(t::HELP_ADMIN_ADDON.s());
        });
    }

    fn confirm_window(&mut self, ctx: &egui::Context) {
        let Some(p) = self.pending.as_ref() else { return };
        let (title, text, detail) = match p {
            Pending::Restore(_, d) => (t::BK_RESTORE_CONFIRM_TITLE.s(), t::BK_RESTORE_CONFIRM.s(), d.clone()),
            Pending::Delete(_, d) => (t::BK_DELETE_CONFIRM_TITLE.s(), t::BK_DELETE_CONFIRM.s(), d.clone()),
            Pending::RemovePack(n) => (t::PACK_REMOVE_CONFIRM_TITLE.s(), t::PACK_REMOVE_CONFIRM.s(), n.clone()),
            Pending::RemoveAddon(n) => (t::ADDON_REMOVE_CONFIRM_TITLE.s(), t::ADDON_REMOVE_CONFIRM.s(), n.clone()),
            Pending::DeleteAccount(_, d) => (t::ACC_DELETE_CONFIRM_TITLE.s(), t::ACC_DELETE_CONFIRM.s(), d.clone()),
            Pending::ExecSql(_, sql, tt, x) => (tt.s(), x.s(), sql.chars().take(300).collect()),
            Pending::ReimportWorld => (t::WORLD_REIMPORT_CONFIRM_TITLE.s(), t::WORLD_REIMPORT_CONFIRM.s(), String::new()),
        };
        let mut decision: Option<bool> = None;
        egui::Window::new(title).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.label(text);
            ui.label(RichText::new(detail).strong());
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button(RichText::new(t::CONFIRM.s()).strong()).clicked() {
                    decision = Some(true);
                }
                if ui.button(t::CANCEL.s()).clicked() {
                    decision = Some(false);
                }
            });
        });
        if let Some(ok) = decision {
            if let Some(p) = self.pending.take() {
                if ok {
                    match p {
                        Pending::Restore(path, _) => self.send(Command::Restore(path)),
                        Pending::Delete(path, _) => {
                            self.backup_selected = None;
                            self.send(Command::DeleteBackup(path));
                        }
                        Pending::RemovePack(n) => self.send(Command::PackRemove(n)),
                        Pending::RemoveAddon(n) => self.send(Command::AddonRemove(n)),
                        Pending::DeleteAccount(n, _) => {
                            self.acc_selected = None;
                            self.send(Command::DeleteAccount(n));
                        }
                        Pending::ExecSql(db, sql, _, _) => self.send(Command::Exec { db, sql }),
                        Pending::ReimportWorld => self.send(Command::ReimportWorld),
                    }
                }
            }
        }
    }

    fn report_window(&mut self, ctx: &egui::Context, snap: &Snapshot) {
        if !self.show_report {
            return;
        }
        let Some(r) = snap.report.as_ref() else { return };
        let mut open = true;
        egui::Window::new(t::RPT_TITLE.s()).open(&mut open).collapsible(false).default_width(560.0).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            egui::ScrollArea::vertical().max_height(420.0).show(ui, |ui| {
                if !r.manifest_present {
                    ui.colored_label(WARN, t::RPT_MANIFEST_MISSING.s());
                } else if r.files_ok() {
                    ui.colored_label(status_color(true, false), format!("{} ({})", t::RPT_FILES_OK.s(), r.checked));
                } else {
                    if !r.missing.is_empty() {
                        ui.colored_label(ERR, format!("{} ({})", t::RPT_MISSING.s(), r.missing.len()));
                        for m in r.missing.iter().take(50) {
                            ui.monospace(m);
                        }
                    }
                    if !r.altered.is_empty() {
                        ui.colored_label(ERR, format!("{} ({})", t::RPT_ALTERED.s(), r.altered.len()));
                        for m in r.altered.iter().take(50) {
                            ui.monospace(m);
                        }
                    }
                }
                ui.add_space(6.0);
                ui.strong(t::RPT_DATA.s());
                for (n, ok) in &r.data {
                    ui.horizontal(|ui| {
                        ui.label(n);
                        ui.label(RichText::new(if *ok { "OK" } else { t::ABSENT.s() }).color(status_color(*ok, false)));
                    });
                }
                ui.add_space(6.0);
                ui.strong(t::RPT_DB.s());
                ui.label(match r.db_ok {
                    Some(true) => t::RPT_DB_OK.s().to_string(),
                    Some(false) => t::RPT_DB_INCOMPLETE.s().to_string(),
                    None => t::RPT_DB_UNKNOWN.s().to_string(),
                });
                if let Some(w) = &r.world {
                    ui.label(t::world_summary(w.npcs, w.monsters, w.items));
                }
                for n in &r.notes {
                    ui.add_space(4.0);
                    ui.label(n);
                }
            });
            ui.add_space(6.0);
            ui.label(t::RPT_REPAIR_HINT.s());
        });
        if !open {
            self.show_report = false;
        }
    }
}
