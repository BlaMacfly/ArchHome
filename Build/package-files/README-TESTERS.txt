HomeRPG ArchLord v0.2.0 — README for testers
============================================

Thank you for testing! HomeRPG ArchLord runs ArchLord single-player and offline on your PC:
a local ArchCore server, its own MariaDB database and the ArchRust client, all started by a
single PLAY button. Nothing leaves your PC (everything listens on 127.0.0.1).

This build: ArchCore 7403088 (Core branch, October 26, 2026) and ArchRust 2e41983
(client branch, October 26, 2026).


1. WHAT YOU NEED
----------------

* Windows 10 or 11, 64-bit, about 2 GB free (plus your game data).
* The 64-bit ArchLord game data (character, ini, world, texture, object, effect, sound…
  folders). It is NOT included: take it from your own ArchLord client.
* Nothing else: the original server tables (monster spawns, loot, monster AI, refining,
  map) are included in Server\Data\server-ini\. The world has its monsters and loot from
  the very first start. Do not delete that folder: without it, the launcher shows a red
  warning and the world has no monsters or loot.
* Ports 3316, 11002 and 11008 must be free (any other ArchLord server stopped).


2. INSTALLATION
---------------

1. Unzip the HomeRPG-ArchLord-v0.2.0 folder somewhere you can write (for example
   C:\Games\). Avoid "Program Files".
2. Game data, either:
   - copy it into Client\, next to ArchRust.exe;
   - or leave it where it is and set its folder in the launcher:
     Settings > "ArchLord game data".
3. Open "HomeRPG ArchLord.exe". The launcher is not signed: if Windows shows "Windows
   protected your PC", click "More info" then "Run anyway".
4. Language: the launcher opens in French if Windows is in French, otherwise in English.
   Settings > "Launcher language" changes it; the game follows the same language (server
   texts, NPC names, item and skill texts). A few ArchRust screens of its own (the login
   screen, some labels) and some AddOn messages are still in French for now.


3. FIRST START
--------------

Click PLAY. The launcher goes through these steps, showing each one:
  1. starts MariaDB (creates the database the first time);
  2. updates the database schema (archcore db-init);
  3. imports the world from your game data (archcore import): a few minutes, once (and
     again after an ArchCore update);
  4. starts the ArchCore server;
  5. opens ArchRust, already set to the local server and the chosen language.
Closing ArchRust also stops the server and the database, and backs up your characters.


4. TEST ACCOUNTS
----------------

No account exists at first. In the launcher's Accounts tab, create for example:
  * tester / test1234  GM level 3: every game master command, and the ArchCore Admin
    panel in game (/aca);
  * player / test1234  GM level 0: a regular player, to check what a player without
    rights can do.
Account name: 1 to 16 letters or digits; password: 4 to 16 characters, no spaces.
Log in to ArchRust with one of them and create a character. The GM commands are listed in
the launcher's "GM help" tab.


5. YOU HAD v0.1.0?
------------------

With the launcher closed, copy the UserData\ folder of the old version into the new one
(accounts, characters, settings, backups). Also copy your game data (Client\) if you had
put it there. The server tables are already in the new version.
On the first PLAY, the launcher applies the new database migrations and then reimports the
world (ArchCore has new tables); your accounts and characters are kept.


6. WHAT'S NEW SINCE v0.1.0 — WHAT TO TRY
----------------------------------------

Full guilds (G key, guild registrar at level 20)
  * guild sub-master, leadership handover, enlarging, guild list;
  * guild marks;
  * guild battles (challenge, countdown, result);
  * guild alliances and hostilities;
  * guild warehouse, at a bank keeper.
Portal player-versus-player zones
  * player-versus-player destinations ask for confirmation; portals are refused while in
    combat; "Return" takes you back to where you came from.
Automatic reconnection
  * if the connection drops, ArchRust reconnects on its own; the character stays in the
    world for 60 s meanwhile. To try it: /coupure in the chat.
Archonia skills and items
  * higher and heroic skills (masteries, heroic points, charisma), damage sharing, purges,
    ignored defence, monster skills;
  * item options, runes and unique items' linked options.
Refining
  * at the refining NPC; GM command /last_refine (last recipe).
Item conversion
  * conversion and elemental damage.
Automatic potions
  * potions on the shortcut bar, automatic use.
Archonia options (Esc key > Options)
  * GFX, Display and Function tabs, kept from one session to the next.
Language
  * French or English: launcher (Settings) and game (server and ArchRust texts).
AddOns
  * included in Client\addons: ArchAce (frames and options, /options), ArchCore Admin
    (GM panel, /aca), HelloArch (example). The launcher's "Packs & AddOns" tab enables,
    disables or adds them (.zip file).
Trade, party and guild refusals
  * Options > Function: tick a refusal, then ask another character for a trade, a party
    or a guild invitation: the request must be refused.
A world closer to Archonia
  * monster levels, bosses respawning in 30 to 45 minutes, movement speed.

The launcher changed too:
  * ArchRust, its AddOns and the original server tables are included (only the game data
    is yours to provide): monsters and loot from the first start;
  * fully in French or English, and the game follows the chosen language;
  * clear warning if the original server tables go missing (otherwise: no monsters), and
    automatic reimport as soon as they are back;
  * automatic world reimport after an ArchCore update;
  * the file check (Home tab > Check) names every altered or missing file, in the log as
    well;
  * MariaDB connections are closed cleanly (no more "Aborted connection" every second in
    Logs\mariadb-error.log);
  * the unneeded MariaDB account root@<PC name> is removed after initialisation.

Two characters at once (party, trade, guild): while the game runs, you can open
Client\ArchRust.exe a second time and log in with the other account.


7. THE LOGS TO SEND US
----------------------

Easiest: Backups tab > "Export a diagnostic". The launcher creates
UserData\diagnostic-<date>.zip (recent launcher, server, client and MariaDB logs,
configuration, passwords masked) and opens its folder. Send that file.

Otherwise, the logs are in Logs\:
  * Logs\launcher.log            the launcher (steps, errors);
  * Logs\server\archcore-*.log   the ArchCore server, one file per start;
  * Logs\server\outils-*.log     database setup and world import;
  * Logs\client\archrust-*.log   the ArchRust client, one file per game;
  * Logs\mariadb-error.log       the database.
They are also shown in the launcher's Logs tab.

Along with the logs, tell us: what you were doing, what you expected, what happened, the
approximate time and the character name. A screenshot helps a lot.


8. GOOD TO KNOW
---------------

* Everything stays in the folder: delete it to uninstall. Keep UserData\ if you want to
  keep your characters.
* Backups tab: back up and restore your characters.
* Startup trouble? Home tab > Check, then Repair.
* World tab > "Reimport the world database": runs the import again (accounts and
  characters kept).
