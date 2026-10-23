//! Ressources Windows de l'exécutable : icône (affichée par l'Explorateur, la barre des tâches, les raccourcis)
//! et informations de version. L'icône est générée par Build\New-Icon.ps1 depuis assets\HomeRPG-original.png.

fn main() {
    println!("cargo:rerun-if-changed=assets/homerpg.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/homerpg.ico");
        res.set("ProductName", "HomeRPG ArchLord");
        res.set("FileDescription", "HomeRPG ArchLord");
        res.set("CompanyName", "BlaMacfly");
        res.set("LegalCopyright", "© 2026 BlaMacfly. Tous droits réservés / All rights reserved.");
        res.set("OriginalFilename", "HomeRPG ArchLord.exe");
        res.set_language(0x040c); // français (France)
        res.compile().expect("ressources Windows (winresource : rc.exe du SDK Windows requis)");
    }
}
