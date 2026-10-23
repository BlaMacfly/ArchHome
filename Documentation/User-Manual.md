# HomeRPG ArchLord — User manual

*Version française : [Manuel-Utilisateur.md](Manuel-Utilisateur.md).*

HomeRPG ArchLord runs ArchLord single-player on your PC: a local ArchCore server, a portable MariaDB database
and a launcher that handles everything. You play with the ArchRust client.

## 1. Requirements

- Windows 10 or 11, 64-bit, 8 GB of memory.
- The **ArchRust client**: `ArchRust.exe` and, in the same folder, the 64-bit ArchLord client data
  (`character\`, `ini\`, `world\`, `texture\`, `object\`, `effect\`, `sound\`…). HomeRPG provides no game data.
  The original client is not supported.
- Optional: the **original ArchLord server tables** (`spawndatatable.txt`, `spawninstancetable.txt`,
  `itemdroptable.txt`, `groupdroprate.txt`, `droprankrate.txt`, `world\node_data`). They describe monster
  spawns and loot and are not in the client. Without them the world has its NPCs, shops and quests, but no
  monsters.
- No Internet connection: everything runs on 127.0.0.1.

## 2. Installation

1. Unzip the archive into a folder on your disk, e.g. `C:\Games\HomeRPG ArchLord\` (not Program Files, not a
   cloud-synced folder).
2. Copy your ArchRust client into `Client\`, or leave it where it is and set its folder in **Settings** ("Find
   clients" finds it if it is near the installation folder).
3. Optional: copy the original server tables into `Server\Data\server-ini\`, or set their folder in
   **Settings**.
4. Open `HomeRPG ArchLord.exe`. If Windows shows "Windows protected your PC": "More info", then "Run anyway".

## 3. First start

Click **PLAY**. The launcher goes through:

1. **MariaDB**: database created in `UserData\Database` (random password stored in
   `UserData\db-credentials.json`), started on 127.0.0.1:3316.
2. **Schema**: `archcore db-init` creates the four databases and applies migrations (on every start, so the
   schema follows ArchCore updates).
3. **World** (once): the client tables are decrypted into `Server\Data\client-ini`, then imported into the
   world database with the terrain height and, if provided, the original server tables; NPC titles are
   translated. One to a few minutes.
4. **ArchCore**: login on 127.0.0.1:11002, world on 127.0.0.1:11008.
5. **ArchRust**: `realmlist.wtf` points to the local server, the game language is applied, the game opens.

Before logging in, create your account in the **Accounts** tab (the server does not need to run).

## 4. Tabs

| Tab | What it does |
|---|---|
| Home | Status of the client, database, server, world and server tables; PLAY, start the server only, Stop, Close the game, Check, Repair, Back up |
| Accounts | Create an account (1–16 letters/digits, 4–16 character password without spaces or accents, GM level 0–3), change password, apply GM level, delete (account, characters, bank) |
| World | ArchCore settings (`world.conf`): monster density, chat range, sky clock, rates, GM command levels, slow packet/query thresholds; reimport the world database |
| Packs & AddOns | ArchRust resource packs (official `free-assets`, community packs in `patches`: enable, disable, add `.ahp` or folder, remove — same `packs.toml` as the in-game Options) and Lua AddOns (install from zip, disable, remove) |
| Database | Explorer for the four databases (data with filter, sort, edit, insert, delete; structure; SQL with write confirmation); Open in HeidiSQL |
| Backups | Back up now, restore (with a safety backup first), delete, export a diagnostic; automatic backup when the game closes |
| Settings | Launcher language (automatic, French, English — applied immediately), game language, client folder, server tables folder, ports, automatic backups, log options |
| Logs | Launcher, Server and Client views, text filter, warnings-and-errors filter, open the folder or current file |
| GM help | ArchCore GM commands and ArchRust shortcuts |

## 5. Logs

| View | Content | Files |
|---|---|---|
| Launcher | every step (starts, imports, accounts, backups, errors) | `Logs\launcher.log` |
| Server | ArchCore output (world loading, connections, warnings) and import tools | `Logs\server\archcore-<date>.log` (one per start), `Logs\server\outils-<date>.log` |
| Client | ArchRust output (rendering, packs, AddOns, network) | `Logs\client\archrust-<date>.log` (one per game) |

The 20 most recent files of each kind are kept. Settings can enable a detailed server log (every packet) and
set ArchRust's log level (`error`, `warn`, `info`, `debug`).

## 6. Folders

| Folder | Content | Keep |
|---|---|---|
| `Server\ArchCore` | ArchCore server, schema, content layer, translations | no (program) |
| `Server\Data` | decrypted client tables, original server tables | regenerated |
| `UserData` | database, settings, SQL credentials, backups | **yes** |
| `Logs` | logs | if needed |
| `Client` | your ArchRust client (if you put it there) | yes |

To update, replace everything except `UserData` (and `Client`, `Server\Data\server-ini`).

## 7. Troubleshooting

| Symptom | What to do |
|---|---|
| "ArchRust client not found or incomplete" | Settings > client folder: it must contain `ArchRust.exe` and `character\Reference.Dat` |
| "port 11002 is already used" | another ArchCore server runs: stop it, or change the ports in Settings |
| No monsters | original server tables missing: add them, then World > Reimport the world database |
| ArchRust cannot connect | check the server is "Running"; look at Logs > Client then Server |
| The game closes with an error | Logs > Client (`debug` level in Settings for more detail) |
| The server stops while starting | Logs > Server; try Repair |

To ask for help, attach the diagnostic (Backups > Export a diagnostic). Discord:
[discord.gg/JCyrsTm7fR](https://discord.gg/JCyrsTm7fR).
