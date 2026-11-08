//! Catalogue des réglages de world.conf (onglet Monde) et de archbot.conf (onglet ArchBot). Chaque entrée : clé exacte d'ArchCore,
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
    /// Texte libre, écrit entre guillemets
    Text,
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

const G_BOT_ON: T = T("Activation", "Activation");
const G_BOT_POP: T = T("Population", "Population");
const G_BOT_BEHAVIOUR: T = T("Comportement", "Behaviour");
const G_BOT_SURVIVAL: T = T("Survie", "Survival");
const G_BOT_PERF: T = T("Performances", "Performance");
const G_BOT_ADMIN: T = T("Administration", "Administration");

const PERCENT: Kind = Kind::Int { min: 0, max: 100 };

const fn bot(key: &'static str, group: T, label: T, help: T, kind: Kind, default: &'static str) -> Opt {
    Opt { key, group, label, help, kind, default }
}

/// Réglages de archbot.conf (module ArchBot compilé dans archcore.exe), onglet ArchBot : toutes les clés du modèle
/// archbot.conf.dist sauf ArchBot.AccountPrefix et ArchBot.NamePrefix (le launcher reconnaît les comptes des bots à
/// « archbot_ »). Défauts du modèle, sauf la population, choisie pour un jeu solo sur un seul PC (modèle : 100 bots
/// au plus, 20 au départ).
pub const ARCHBOT_OPTIONS: &[Opt] = &[
    // ---- Activation
    bot(
        "ArchBot.Enable",
        G_BOT_ON,
        T("Bots activés", "Bots enabled"),
        T(
            "Personnages pilotés par le serveur : de vrais personnages (compte, classe, niveau, équipement) qui peuplent le monde ou vous accompagnent. Décoché : le module ne démarre pas, aucun bot n'entre en jeu.",
            "Server-driven characters: real characters (account, class, level, equipment) who populate the world or travel with you. Unchecked: the module does not start, no bot enters the game.",
        ),
        Kind::Bool,
        "1",
    ),
    bot(
        "ArchBot.AutonomousBots",
        G_BOT_ON,
        T("Bots autonomes", "Autonomous bots"),
        T(
            "Une population entretenue : ils quittent leur village, chassent les monstres de leur niveau, ramassent le butin, s'équipent, apprennent leurs compétences, se reposent et reviennent après leur mort.",
            "A maintained population: they leave their village, hunt monsters of their level, loot, equip upgrades, learn their skills, rest and come back after death.",
        ),
        Kind::Bool,
        "1",
    ),
    bot(
        "ArchBot.CompanionBots",
        G_BOT_ON,
        T("Compagnons", "Companions"),
        T(
            "« /bot hire » près d'un bot (ou chuchotez-lui « hire ») : il rejoint votre groupe, vous suit, vous défend et obéit à follow, stay, come, attack, passive, defensive, aggressive, stop, status.",
            "\"/bot hire\" near a bot (or whisper \"hire\" to it): it joins your party, follows you, defends you and obeys follow, stay, come, attack, passive, defensive, aggressive, stop, status.",
        ),
        Kind::Bool,
        "1",
    ),
    // ---- Population
    bot(
        "ArchBot.InitialPopulation",
        G_BOT_POP,
        T("Bots autonomes au démarrage", "Autonomous bots at start"),
        T("Population visée au démarrage (ramenée au maximum). /bot population n la change en jeu (MJ).", "Population aimed at on start (capped by the maximum). /bot population n changes it in game (GM)."),
        Kind::Int { min: 0, max: 200 },
        "15",
    ),
    bot(
        "ArchBot.MaxBots",
        G_BOT_POP,
        T("Bots en jeu au plus", "Maximum bots in game"),
        T("Autonomes et compagnons compris. Chaque bot coûte un peu de processeur au serveur.", "Autonomous bots and companions together. Each bot costs the server a little CPU."),
        Kind::Int { min: 0, max: 500 },
        "40",
    ),
    bot(
        "ArchBot.SpawnIntervalMs",
        G_BOT_POP,
        T("Délai entre deux entrées de bots (ms)", "Delay between two bot entries (ms)"),
        T("La population monte progressivement, sans à-coup sur le serveur.", "The population grows step by step, without straining the server."),
        Kind::Int { min: 100, max: 60000 },
        "1000",
    ),
    bot("ArchBot.MinLevel", G_BOT_POP, T("Niveau des nouveaux bots : minimum", "New bots' level: minimum"), T("0 : niveau du modèle.", "0: the template's level."), Kind::Int { min: 0, max: 120 }, "1"),
    bot(
        "ArchBot.MaxLevel",
        G_BOT_POP,
        T("Niveau des nouveaux bots : maximum", "New bots' level: maximum"),
        T("Tiré à la création ; un bot garde ensuite sa progression réelle.", "Drawn at creation; a bot then keeps its real progress."),
        Kind::Int { min: 0, max: 120 },
        "10",
    ),
    bot(
        "ArchBot.Classes",
        G_BOT_POP,
        T("Classes permises (TID)", "Allowed classes (TID)"),
        T(
            "TID des modèles jouables, séparés par des virgules : 96 Knight, 1 Archer, 6 Mage, 4 Berserker, 8 Hunter, 3 Sorcerer, 377 Swashbuckler, 460 Ranger, 9 Elementalist, 1724 Scion. Vide : toutes les classes de l'écran de création.",
            "Playable template TIDs, comma-separated: 96 Knight, 1 Archer, 6 Mage, 4 Berserker, 8 Hunter, 3 Sorcerer, 377 Swashbuckler, 460 Ranger, 9 Elementalist, 1724 Scion. Empty: every class of the creation screen.",
        ),
        Kind::Text,
        "",
    ),
    bot(
        "ArchBot.PersistCharacters",
        G_BOT_POP,
        T("Garder les personnages des bots", "Keep the bots' characters"),
        T("Coché : niveau, équipement et position repris au démarrage suivant. Décoché : un bot qui quitte le monde est supprimé.", "Ticked: level, equipment and position are kept for the next start. Unticked: a bot leaving the world is deleted."),
        Kind::Bool,
        "1",
    ),
    bot(
        "ArchBot.RotationMinutes",
        G_BOT_POP,
        T("Durée de session d'un bot autonome (min)", "Autonomous bot session length (min)"),
        T("Entre 50 % et 150 % de cette valeur, puis il cède sa place à un autre personnage de la réserve. 0 : les bots restent en jeu.", "Between 50% and 150% of this value, then it hands over to another character of the pool. 0: bots stay in game."),
        Kind::Int { min: 0, max: 1440 },
        "0",
    ),
    // ---- Comportement
    bot("ArchBot.AllowCombat", G_BOT_BEHAVIOUR, T("Combat contre les monstres", "Fighting monsters"), NO_HELP, Kind::Bool, "1"),
    bot("ArchBot.AllowLoot", G_BOT_BEHAVIOUR, T("Ramassage du butin", "Looting"), NO_HELP, Kind::Bool, "1"),
    bot(
        "ArchBot.AllowProgression",
        G_BOT_BEHAVIOUR,
        T("Progression (équipement, compétences)", "Progression (equipment, skills)"),
        T("Les bots portent leurs meilleurs objets et apprennent leurs compétences.", "Bots wear their best items and learn their skills."),
        Kind::Bool,
        "1",
    ),
    bot(
        "ArchBot.AllowParty",
        G_BOT_BEHAVIOUR,
        T("Groupes", "Parties"),
        T("Un bot accepte vos invitations de groupe ; un compagnon invite son maître.", "A bot accepts your party invitations; a companion invites its master."),
        Kind::Bool,
        "1",
    ),
    bot(
        "ArchBot.ScanRadius",
        G_BOT_BEHAVIOUR,
        T("Rayon de perception", "Perception radius"),
        T("Monstres, joueurs et objets au sol vus par un bot, en unités du jeu.", "Monsters, players and ground items a bot sees, in game units."),
        Kind::Int { min: 500, max: 20000 },
        "4000",
    ),
    bot(
        "ArchBot.ExploreRadius",
        G_BOT_BEHAVIOUR,
        T("Rayon d'exploration", "Exploration radius"),
        T("Autour du lieu d'apparition d'un bot autonome. 0 : il ne se promène pas.", "Around an autonomous bot's spawn point. 0: it does not wander."),
        Kind::Int { min: 0, max: 20000 },
        "3000",
    ),
    bot("ArchBot.PickupRadius", G_BOT_BEHAVIOUR, T("Distance de ramassage", "Pickup distance"), T("Le serveur refuse au-delà de 2000.", "The server refuses beyond 2000."), Kind::Int { min: 100, max: 2000 }, "1500"),
    bot(
        "ArchBot.TargetLevelBelow",
        G_BOT_BEHAVIOUR,
        T("Monstres choisis : niveaux en dessous", "Chosen monsters: levels below"),
        T("Un bot autonome choisit des monstres de son niveau moins cette valeur à son niveau plus la suivante. Il riposte toujours à qui le frappe.", "An autonomous bot picks monsters from its level minus this value to its level plus the next one. It always strikes back."),
        Kind::Int { min: 0, max: 50 },
        "6",
    ),
    bot("ArchBot.TargetLevelAbove", G_BOT_BEHAVIOUR, T("Monstres choisis : niveaux au-dessus", "Chosen monsters: levels above"), NO_HELP, Kind::Int { min: 0, max: 50 }, "2"),
    bot(
        "ArchBot.FollowDistance",
        G_BOT_BEHAVIOUR,
        T("Distance de suivi d'un compagnon", "Companion follow distance"),
        T("« follow 500 » la change pour un compagnon.", "\"follow 500\" changes it for one companion."),
        Kind::Int { min: 100, max: 3000 },
        "300",
    ),
    bot("ArchBot.Chatter", G_BOT_BEHAVIOUR, T("Les bots répondent", "Bots reply"), T("Réponses aux ordres et aux chuchotements.", "Replies to orders and whispers."), Kind::Bool, "1"),
    // ---- Survie
    bot(
        "ArchBot.FleeHpPercent",
        G_BOT_SURVIVAL,
        T("Fuite sous (% de PV)", "Flee below (% HP)"),
        T("S'il n'a plus de potion, s'il a plusieurs agresseurs ou si l'un est bien plus haut que lui. 0 : jamais.", "If it has no potion left, several attackers or one far above it. 0: never."),
        PERCENT,
        "20",
    ),
    bot("ArchBot.RestHpPercent", G_BOT_SURVIVAL, T("Repos sous (% de PV)", "Rest below (% HP)"), T("Hors combat.", "Out of combat."), PERCENT, "45"),
    bot("ArchBot.RestUntilPercent", G_BOT_SURVIVAL, T("Repos jusqu'à (% de PV)", "Rest until (% HP)"), NO_HELP, PERCENT, "85"),
    bot(
        "ArchBot.PotionHpPercent",
        G_BOT_SURVIVAL,
        T("Potion de vie sous (% de PV)", "Health potion below (% HP)"),
        T("Potions de son inventaire seulement : rien n'est créé.", "Only potions from its inventory: nothing is created."),
        PERCENT,
        "50",
    ),
    bot("ArchBot.PotionMpPercent", G_BOT_SURVIVAL, T("Potion de mana sous (% de PM)", "Mana potion below (% MP)"), NO_HELP, PERCENT, "20"),
    bot("ArchBot.EngageHpPercent", G_BOT_SURVIVAL, T("Nouveau combat au-dessus de (% de PV)", "New fight above (% HP)"), NO_HELP, PERCENT, "60"),
    bot("ArchBot.ResurrectDelaySeconds", G_BOT_SURVIVAL, T("Attente avant résurrection (s)", "Wait before resurrection (s)"), T("Au village.", "In the village."), Kind::Int { min: 0, max: 3600 }, "8"),
    // ---- Performances
    bot("ArchBot.AIUpdateIntervalMs", G_BOT_PERF, T("Cadence de la boucle d'IA (ms)", "AI loop interval (ms)"), NO_HELP, Kind::Int { min: 100, max: 10000 }, "500"),
    bot(
        "ArchBot.BotsPerTick",
        G_BOT_PERF,
        T("Bots qui pensent par tour", "Bots thinking per tick"),
        T("Au-delà, les bots sont répartis sur plusieurs tours.", "Beyond that, bots are spread over several ticks."),
        Kind::Int { min: 1, max: 5000 },
        "50",
    ),
    bot(
        "ArchBot.FarFromPlayersUnits",
        G_BOT_PERF,
        T("Distance « loin des joueurs »", "\"Far from players\" distance"),
        T("Un bot sans vrai joueur plus près ne pense qu'un tour sur le diviseur suivant.", "A bot with no real player closer only thinks one tick out of the next divisor."),
        Kind::Int { min: 0, max: 200000 },
        "30000",
    ),
    bot("ArchBot.FarTickDivisor", G_BOT_PERF, T("Diviseur loin des joueurs", "Divisor far from players"), T("1 : toujours.", "1: always."), Kind::Int { min: 1, max: 100 }, "4"),
    // ---- Administration
    bot(
        "ArchBot.CommandLevel",
        G_BOT_ADMIN,
        T("/bot stats, spawn, despawn, population : niveau MJ", "/bot stats, spawn, despawn, population: GM level"),
        T("0 : commandes désactivées. /bot hire et les ordres aux compagnons sont ouverts à tous.", "0: commands disabled. /bot hire and companion orders are open to everyone."),
        GM_LEVEL,
        "2",
    ),
    bot(
        "ArchBot.Debug",
        G_BOT_ADMIN,
        T("Journal détaillé des bots", "Detailed bot log"),
        T("Chaque action des bots dans le journal du serveur, et un rapport de métriques tous les 20 tours. Volumineux.", "Every bot action in the server log, and a metrics report every 20 ticks. Large."),
        Kind::Bool,
        "0",
    ),
];

/// Valeur effective (surcharge ou défaut).
pub fn value<'a>(overrides: &'a std::collections::BTreeMap<String, String>, opt: &'a Opt) -> &'a str {
    overrides.get(opt.key).map(String::as_str).unwrap_or(opt.default)
}

/// Valeur telle qu'elle s'écrit dans le .conf (choix entre guillemets).
pub fn conf_value(overrides: &std::collections::BTreeMap<String, String>, opt: &Opt) -> String {
    let v = value(overrides, opt);
    match opt.kind {
        Kind::Choice(_) | Kind::Text => format!("\"{}\"", v.replace('"', "'")),
        _ => v.to_string(),
    }
}
