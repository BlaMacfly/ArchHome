//! HomeRPG ArchLord — launcher (Rust, egui). © 2026 BlaMacfly. Licence propriétaire : voir LICENCE.txt.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod addons;
mod backup;
mod client;
mod conf_catalog;
mod credentials;
mod db;
mod i18n;
mod logs;
mod orchestrator;
mod packs;
mod paths;
mod repair;
mod server;
mod settings;
mod ui;
mod util;

fn main() -> eframe::Result {
    let layout = paths::Layout::detect();
    let settings = settings::Settings::load(&layout.settings_file());
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    // Langue : --lang fr|en (tests), sinon le réglage, sinon celle de Windows.
    i18n::set_lang(arg("--lang").and_then(|l| i18n::Lang::parse(&l)).unwrap_or_else(|| settings.ui_lang()));
    let (shared, tx, orchestrator_thread) = orchestrator::spawn(layout.clone(), settings.clone());

    // Options de test (Tests/) : --start démarre les services immédiatement, --quit-after N ferme après N secondes.
    let auto_start = args.iter().any(|a| a == "--start");
    let quit_after = arg("--quit-after").and_then(|v| v.parse::<u64>().ok());
    let start_tab = arg("--tab");
    if auto_start {
        let _ = tx.send(orchestrator::Command::StartAll);
    }
    // --do <action> [argument] : actions de l'interface pour les tests d'intégration.
    if let Some(i) = args.iter().position(|a| a == "--do") {
        let cmd = match (args.get(i + 1).map(String::as_str), args.get(i + 2)) {
            (Some("play"), _) => Some(orchestrator::Command::Play),
            (Some("backup"), _) => Some(orchestrator::Command::Backup("test".into())),
            (Some("verify"), _) => Some(orchestrator::Command::Verify),
            (Some("repair"), _) => Some(orchestrator::Command::Repair),
            (Some("diagnostic"), _) => Some(orchestrator::Command::ExportDiagnostic),
            (Some("restore"), Some(p)) => Some(orchestrator::Command::Restore(std::path::PathBuf::from(p))),
            (Some("reimportworld"), _) => Some(orchestrator::Command::ReimportWorld),
            (Some("accdel"), Some(n)) => Some(orchestrator::Command::DeleteAccount(n.clone())),
            // --do acccreate <compte> <mot de passe> [niveau]
            (Some("acccreate"), Some(u)) => args.get(i + 3).map(|p| orchestrator::Command::CreateAccount { user: u.clone(), pass: p.clone(), gm: args.get(i + 4).and_then(|g| g.parse().ok()).unwrap_or(0) }),
            (Some("packon"), Some(n)) => Some(orchestrator::Command::PackSetEnabled(n.clone(), true)),
            (Some("packoff"), Some(n)) => Some(orchestrator::Command::PackSetEnabled(n.clone(), false)),
            (Some("addonadd"), Some(p)) => Some(orchestrator::Command::AddonAdd(std::path::PathBuf::from(p))),
            (Some("addonon"), Some(n)) => Some(orchestrator::Command::AddonSetActive(n.clone(), true)),
            (Some("addonoff"), Some(n)) => Some(orchestrator::Command::AddonSetActive(n.clone(), false)),
            (Some("addondel"), Some(n)) => Some(orchestrator::Command::AddonRemove(n.clone())),
            _ => None,
        };
        if let Some(c) = cmd {
            let _ = tx.send(c);
        }
    }
    // --install-to <dossier> : copie depuis un support en lecture seule sans passer par le sélecteur (tests).
    if let Some(p) = arg("--install-to") {
        let _ = tx.send(orchestrator::Command::InstallTo(std::path::PathBuf::from(p)));
    }
    let mut viewport = eframe::egui::ViewportBuilder::default().with_inner_size([1000.0, 720.0]).with_min_inner_size([780.0, 540.0]).with_title("HomeRPG ArchLord");
    if let Ok(img) = image::load_from_memory(include_bytes!("../assets/logo-256.png")) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        viewport = viewport.with_icon(std::sync::Arc::new(eframe::egui::IconData { rgba: rgba.into_raw(), width, height }));
    }
    let options = eframe::NativeOptions { viewport, ..Default::default() };
    let tx_exit = tx.clone();
    let result = eframe::run_native("HomeRPG ArchLord", options, Box::new(move |cc| Ok(Box::new(ui::App::new(cc, layout, settings, shared, tx, quit_after, start_tab)))));
    // Fermeture : l'orchestrateur arrête les services qu'il a démarrés (MariaDB comprise, arrêt SQL propre) ; le
    // processus attend qu'il ait fini, sans quoi la fin de main interromprait l'arrêt en cours.
    let _ = tx_exit.send(orchestrator::Command::Shutdown);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(120);
    while !orchestrator_thread.is_finished() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    result
}
