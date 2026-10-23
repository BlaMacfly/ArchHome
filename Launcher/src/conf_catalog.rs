//! Catalgue des réglages de world.conf exposés dans l'onglet Monde. Chaque entrée : clé exacte d'ArchCore,
//! libellé et aide (français, anglais), type, valeur par défaut (celle du modèle world.conf.dist). Les valeurs
//! modifiées sont stockées dans UserData/settings.json (`conf`) et appliquées à la génération de world.conf.
//! Seules les clés de ce catalogue peuvent être surchargées : les clés gérées (ports, bases, chemins) ne le sont jamais.

use crate::i18n::T;

#[derive(Clone, Copy)]
pub enum Kind {
    Bool,
    Int { min: i64, max: i64 },
    Float { min: f64, max: f64 },
    Choice(&'static [(&'static str, T)]), // (valeur, libellé)
}

#[derive(Clone, Copy)]
pub struct Opt {
    pub key: &'static str,
    pub group: T,
    pub label: T,
    pub help: T,
    pub kind: Kind,
    pub default: &'static str,
}

const G_WORLD: T = T("Monde", "World");
const G_RATES: T = T("Taux (1 = normal, 2 = double)", "Rates (1 = normal, 2 = double)");
const G_GM: T = T("Commandes MJ : niveau requis", "GM commands: required level");
const G_TECH: T = T("Technique", "Technical");
const NO_HELP: T = T("", "");

const RATE: Kind = Kind::Float { min: 0.0, max: 20.0 };
const GM_LEVEL: Kind = Kind::Int { min: 0, max: 3 };

pub const WORLD_OPTIONS: &[Opt] = &[
    // ---- Monde
    Opt {
        key: "World.MonsterDensity",
        group: G_WORLD,
        label: T("Densité de monstres", "Monster density"),
        help: T(
            "Multiplie le nombre de monstres de chaque point d'apparition. 1 : tables telles quelles, peu peuplées par endroits ; 2 ou 3 rapprochent de la densité du jeu d'origine.",
            "Multiplies the number of monsters at each spawn point. 1: tables as they are, sparse in places; 2 or 3 come closer to the original game's density.",
        ),
        kind: Kind::Int { min: 1, max: 5 },
        default: "1",
    },
    Opt {
        key: "World.ViewRadius",
        group: G_WORLD,
        label: T("Portée de la discussion normale", "Normal chat range"),
        help: T(
            "En unités du jeu. Le reste (PNJ, monstres, joueurs) se voit par secteurs de 6400 unités.",
            "In game units. Everything else (NPCs, monsters, players) is seen by 6400-unit sectors.",
        ),
        kind: Kind::Int { min: 2000, max: 32000 },
        default: "8000",
    },
    Opt {
        key: "World.TimeZone",
        group: G_WORLD,
        label: T("Horloge du ciel", "Sky clock"),
        help: T(
            "Heure envoyée au client, qui en tire le jour et la nuit.",
            "Time sent to the client, which derives day and night from it.",
        ),
        kind: Kind::Choice(&[("local", T("Heure de ce PC", "This PC's time")), ("utc", T("Temps universel", "Universal time"))]),
        default: "local",
    },
    // ---- Taux
    Opt { key: "Rate.Exp", group: G_RATES, label: T("Expérience", "Experience"), help: NO_HELP, kind: RATE, default: "1" },
    Opt { key: "Rate.Gold", group: G_RATES, label: T("Or ramassé", "Gold looted"), help: NO_HELP, kind: RATE, default: "1" },
    Opt { key: "Rate.Drop", group: G_RATES, label: T("Chance de butin", "Drop chance"), help: NO_HELP, kind: RATE, default: "1" },
    Opt { key: "Rate.UniqueDrop", group: G_RATES, label: T("Chance de butin unique", "Unique drop chance"), help: NO_HELP, kind: RATE, default: "1" },
    Opt { key: "Rate.Charisma", group: G_RATES, label: T("Charisme", "Charisma"), help: NO_HELP, kind: RATE, default: "1" },
    Opt { key: "Rate.Prestige", group: G_RATES, label: T("Prestige", "Prestige"), help: NO_HELP, kind: RATE, default: "1" },
    Opt {
        key: "Rate.ChatBuff",
        group: G_RATES,
        label: T("Bonus de discussion", "Chat buff"),
        help: T("Activé ou désactivé.", "On or off."),
        kind: Kind::Bool,
        default: "1",
    },
    // ---- Commandes MJ
    Opt {
        key: "GM.CommandLevel.move",
        group: G_GM,
        label: T("/move x,z (téléportation)", "/move x,z (teleport)"),
        help: T("0 : tous les joueurs ; 3 : maître du jeu complet.", "0: every player; 3: full game master."),
        kind: GM_LEVEL,
        default: "1",
    },
    Opt { key: "GM.CommandLevel.speed", group: G_GM, label: T("/speed 1-10 (vitesse)", "/speed 1-10 (speed)"), help: NO_HELP, kind: GM_LEVEL, default: "2" },
    Opt { key: "GM.CommandLevel.invincible", group: G_GM, label: T("/invincible", "/invincible"), help: NO_HELP, kind: GM_LEVEL, default: "2" },
    Opt { key: "GM.CommandLevel.kill", group: G_GM, label: T("/kill", "/kill"), help: NO_HELP, kind: GM_LEVEL, default: "2" },
    Opt { key: "GM.CommandLevel.additem", group: G_GM, label: T("/additem TID [n] (objet)", "/additem TID [n] (item)"), help: NO_HELP, kind: GM_LEVEL, default: "3" },
    Opt { key: "GM.CommandLevel.level", group: G_GM, label: T("/level (niveau du personnage)", "/level (character level)"), help: NO_HELP, kind: GM_LEVEL, default: "3" },
    Opt { key: "GM.CommandLevel.money", group: G_GM, label: T("/money (or du personnage)", "/money (character gold)"), help: NO_HELP, kind: GM_LEVEL, default: "3" },
    // ---- Technique
    Opt {
        key: "World.SlowPacketMs",
        group: G_TECH,
        label: T("Seuil des « paquets lents » (ms)", "\"Slow packet\" threshold (ms)"),
        help: T("Un paquet plus long est noté dans le journal du serveur. 0 : jamais.", "A longer packet is written to the server log. 0: never."),
        kind: Kind::Int { min: 0, max: 5000 },
        default: "100",
    },
    Opt {
        key: "Database.SlowQueryMs",
        group: G_TECH,
        label: T("Seuil des « requêtes lentes » (ms)", "\"Slow query\" threshold (ms)"),
        help: T("Une requête plus longue est notée dans le journal du serveur. 0 : jamais.", "A longer query is written to the server log. 0: never."),
        kind: Kind::Int { min: 0, max: 5000 },
        default: "200",
    },
];

/// Valeur effective (surcharge ou défaut).
pub fn value<'a>(overrides: &'a std::collections::BTreeMap<String, String>, opt: &'a Opt) -> &'a str {
    overrides.get(opt.key).map(String::as_str).unwrap_or(opt.default)
}
