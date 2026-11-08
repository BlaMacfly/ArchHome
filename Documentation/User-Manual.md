# HomeRPG ArchLord — User manual

*Version française : [Manuel-Utilisateur.md](Manuel-Utilisateur.md).*

HomeRPG ArchLord runs ArchLord single-player on your PC: a local ArchCore server, a portable MariaDB database
and a launcher that handles everything. You play with the ArchRust client.

## 1. Requirements

- Windows 10 or 11, 64-bit, 8 GB of memory.
- The **ArchRust executable** (`Client\ArchRust.exe`, included in the package) and the 64-bit ArchLord game
  data (`character\`, `ini\`, `world\`, `texture\`, `object\`, `effect\`, `sound\`…), which you provide.
  HomeRPG distributes no game data. The original client is not supported.
- The **original ArchLord server tables** (Episode 8 game values: skills, items, options, monsters, regions,
  shops, portals, experience; monster spawns, loot, AI, refining, `world\node_data`) are included in
  `Server\Data\server-ini\`. The world import requires them: do not delete that folder.
- No Internet connection: everything runs on 127.0.0.1.

## 2. Installation

1. Unzip the archive into a folder on your disk, e.g. `C:\Games\HomeRPG ArchLord\` (not Program Files, not a
   cloud-synced folder).
2. Put your game data in `Client\` next to `ArchRust.exe`, or set its folder in **Settings** > "Game data".
   The launcher detects the included executable in `Client\`. If the game data is elsewhere (for example the
   `Game` folder of an ArchLord installation), ArchRust is started with `--game`.
3. The original server tables are already in `Server\Data\server-ini\`; **Settings** lets you use other ones.
4. Open `HomeRPG ArchLord.exe`. If Windows shows "Windows protected your PC": "More info", then "Run anyway".

## 3. First start

Click **PLAY**. The launcher goes through:

1. **MariaDB**: database created in `UserData\Database` (random password stored in
   `UserData\db-credentials.json`), started on 127.0.0.1:3316.
2. **Schema**: `archcore db-init` creates the four databases and applies migrations (on every start, so the
   schema follows ArchCore updates).
3. **World** (on the first start, again after an ArchCore update; accounts and characters are kept): the client
   tables are decrypted into `Server\Data\client-ini`, then imported into the world database with the terrain
   height: the **Episode 8 values** come from the original server tables, names and visuals from the client;
   NPC titles are translated. About a minute. Without the original server tables, the start stops and says so.
   An installation updated from a pre-Episode 8 version (0.2.0 and earlier) then brings its characters to the
   same values (`archcore migrate-ep8`), once, after an "avant-ep8" backup: items, options and skills that only
   Archonia had are quarantined (reversible) and a free skill reset is offered.
4. **Tables for ArchRust**: `archcore export-client-tables` writes the Episode 8 tables in the client's format
   into `Client\server-data` (next to `realmlist.wtf`), after each import and whenever they are missing. The
   server announces their fingerprint (`Client.TablesDir`); ArchRust checks it "matches the server".
5. **ArchCore**: login on 127.0.0.1:11002, world on 127.0.0.1:11008; then ArchBot's **bots** enter the game one
   by one (see Bots below).
6. **ArchRust**: `realmlist.wtf` points to the local server, the game language is applied, the game opens on
   ArchRust's login panel: account and password, local server already filled in, "Enregistrer le compte" box (save account) so you
   no longer type them (the first world, "HomeRPG", is picked automatically).

Before logging in, create your account in the **Accounts** tab (the server does not need to run).

## 4. Tabs

| Tab | What it does |
|---|---|
| Home | Status of the client, database, server, world and server tables; PLAY, start the server only, Stop, Close the game, Check, Repair, Back up |
| Accounts | Create an account (1–16 letters/digits, 4–16 character password without spaces or accents, GM level 0–3), change password, apply GM level, delete (account, characters, bank); the bots' `archbot_…` accounts are hidden |
| World | ArchCore settings (`world.conf`): monster density, chat range, sky clock, rates, GM command levels, slow packet/query thresholds; reimport the world database |
| ArchBot | Every bot setting (`archbot.conf`), by group: activation (bots, autonomous bots, companions), population (bots at start 15, maximum 40, entry delay, new bots' levels 1–10, allowed classes, kept characters, session length), behaviour, survival, performance, administration (GM level of the `/bot` management commands, detailed log); in-game commands. ArchBot's values except the population, reduced for single-player |
| Packs & AddOns | ArchRust resource packs (official `free-assets`, community packs in `patches`: enable, disable, add `.ahp` or folder, remove — same `packs.toml` as the in-game Options) and Lua AddOns (install from zip, disable, remove) |
| Database | Explorer for the four databases (data with filter, sort, edit, insert, delete; structure; SQL with write confirmation); Open in HeidiSQL |
| Backups | Back up now, restore (with a safety backup first), delete, export a diagnostic; automatic backup when the game closes |
| Settings | Launcher language (automatic, French, English — applied immediately), game language, client folder, game data folder (optional), server tables folder, ports, automatic backups, log options |
| Logs | Launcher, Server and Client views, text filter, warnings-and-errors filter, open the folder or current file |
| GM help | ArchCore GM commands and ArchRust shortcuts |

## 5. Bots

The server includes [ArchBot](https://github.com/BlaMacfly/ArchBot): server-driven characters with an account,
a class, a level and equipment, following the same rules as you. **Autonomous bots** leave their village, hunt
monsters of their level, loot, equip, rest and come back after death; they are kept from one session to the
next. Hire a **companion** in game: `/bot hire` (nearest free bot) or `/bot hire Name`, then accept its party
invitation; it follows and defends you. Orders: `/bot follow [distance]`, `stay`, `come`, `attack`, `passive`,
`defensive`, `aggressive`, `stop`, `status`, also by whispering to the bot or to the whole party with `/p`;
`/bot dismiss` releases it, `/bot list` lists the bots. For GMs: `/bot stats`, `/bot spawn n`,
`/bot despawn n|all|name`, `/bot population n`. Server log: `[archbot]` lines. Untick "Bots enabled" in the
ArchBot tab to play without them.

## 6. Logs

| View | Content | Files |
|---|---|---|
| Launcher | every step (starts, imports, accounts, backups, errors) | `Logs\launcher.log` |
| Server | ArchCore output (world loading, connections, warnings) and import tools | `Logs\server\archcore-<date>.log` (one per start), `Logs\server\outils-<date>.log` |
| Client | ArchRust output (rendering, packs, AddOns, network) | `Logs\client\archrust-<date>.log` (one per game) |

The 20 most recent files of each kind are kept. Settings can enable a detailed server log (every packet) and
set ArchRust's log level (`error`, `warn`, `info`, `debug`).

## 7. Folders

| Folder | Content | Keep |
|---|---|---|
| `Server\ArchCore` | ArchCore server, schema, content layer, translations | no (program) |
| `Server\Data` | decrypted client tables, original server tables | regenerated |
| `Client\server-data` | Episode 8 tables written for ArchRust by the server | regenerated |
| `UserData` | database, settings, SQL credentials, backups | **yes** |
| `Logs` | logs | if needed |
| `Client` | ArchRust.exe and its AddOns (included); your game data if you put it there | yes |

To update, unzip the new version into a new folder, then, with the launcher closed, copy `UserData` into it, along
with your game data (if it was in `Client`). Keep the new version's `ArchRust.exe`, AddOns and
`Server\Data\server-ini` tables (those of 0.2.0 are no longer enough). On the first PLAY, the databases are
updated, the world is reimported, ArchRust's tables are written and, coming from a pre-Episode 8 version, the
characters are brought to the Episode 8 values (see First start).

## 8. Troubleshooting

| Symptom | What to do |
|---|---|
| "ArchRust client not found or incomplete" | Settings > client folder: it must contain `ArchRust.exe` and `character\Reference.Dat` |
| "port 11002 is already used" | another ArchCore server runs: stop it, or change the ports in Settings |
| "original ArchLord server tables incomplete" | some tables are missing from `Server\Data\server-ini` (the launcher log names them): restore them from the package |
| Tooltips or prices differ from the server | the client log reports a different `server-data` fingerprint: start the game from the launcher (it rewrites the tables), or World > Reimport |
| No bots | ArchBot tab: "Bots enabled"; server log: `[archbot]` lines |
| ArchRust cannot connect | check the server is "Running"; look at Logs > Client then Server |
| The game closes with an error | Logs > Client (`debug` level in Settings for more detail) |
| The server stops while starting | Logs > Server; try Repair |

To ask for help, attach the diagnostic (Backups > Export a diagnostic). Discord:
[discord.gg/JCyrsTm7fR](https://discord.gg/JCyrsTm7fR).
