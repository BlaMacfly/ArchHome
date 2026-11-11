ArchHome v0.3.4 — README for testers
====================================

Thank you for testing! ArchHome runs ArchLord single-player and offline on your PC:
a local ArchCore server, its own MariaDB database and the ArchRust client, all started by a
single PLAY button. Nothing leaves your PC (everything listens on 127.0.0.1).

This build: ArchCore 23c2ced (Core branch, November 10, 2026) with the ArchBot module
(bots), ArchRust 2f82448 (November 10, 2026), ArchCore Admin 1.4.0. The included world is
imported with the original quests, titles, masteries and experience curve, translated
into every language.

This is the first build for IN-GAME testing: all the code was checked with the simulator,
now it is your turn to really play it. Every report counts.


1. WHAT YOU NEED
----------------

* Windows 10 or 11, 64-bit, about 2 GB free (plus the game data).
* The 64-bit ArchLord game data (character, ini, world, texture, object, effect, sound…
  folders). If your test package does not already have it in Client\, take it from your
  own ArchLord client.
* Nothing else: the original ArchLord server tables (Episode 8 game values, monster
  spawns, loot, AI, refining, map) are included in Server\Data\server-ini\. Do not delete
  that folder: without it, the world cannot be imported.
* Ports 3316, 11002 and 11008 must be free (any other ArchLord server stopped).


2. INSTALLATION
---------------

1. Unzip the folder somewhere you can write (for example C:\Games\). Avoid
   "Program Files".
2. Game data, if it is not already in Client\, either:
   - copy it into Client\, next to ArchRust.exe;
   - or leave it where it is and set its folder in the launcher:
     Settings > "ArchLord game data".
3. Open "ArchHome.exe". The launcher is not signed: if Windows shows "Windows
   protected your PC", click "More info" then "Run anyway".
4. Language: the launcher opens in French if Windows is in French, otherwise in English.
   Settings > "Launcher language" changes it; the game follows the same language.


3. FIRST START
--------------

Click PLAY. The launcher goes through these steps, showing each one:
  1. starts MariaDB (creates the database the first time);
  2. updates the database schema (archcore db-init);
  3. imports the world: Episode 8 values from the original server tables, names and
     visuals from your game data (archcore import): about a minute, once (and again after
     an ArchCore update);
  4. writes the Episode 8 tables for ArchRust into Client\server-data (archcore
     export-client-tables): the tooltips, prices and requirements shown in game are the
     server's;
  5. starts the ArchCore server, with its bots;
  6. opens ArchRust, already set to the local server and the chosen language.
Closing ArchRust backs up your characters; closing the launcher stops the server and the
database.


4. TEST ACCOUNTS
----------------

No account exists at first. In the launcher's Accounts tab, create for example:
  * tester / test1234  GM level 3: every game master command, the ArchCore Admin panel in
    game (/aca) and the bots' GM commands;
  * player / test1234  GM level 0: a regular player.
Account name: 1 to 16 letters or digits; password: 4 to 16 characters, no spaces.
In ArchRust, log in on ArchRust's login panel (account and password; the local server is
already filled in). Tick "Enregistrer le compte" (save account) so you no longer have to
type them. The archbot_… accounts belong to the bots: the launcher hides them, leave them
alone.


5. YOU HAD AN EARLIER VERSION?
------------------------------

The program is now called ArchHome.exe (before: HomeRPG ArchLord.exe): recreate your
shortcuts. The UserData\ folder and your settings carry over as before.

From v0.3.3 or earlier: two choices.
  * For the original quests, titles, masteries and experience (recommended): keep the
    UserData\ shipped with v0.3.4; your earlier characters are not carried over.
  * To keep your characters: copy your earlier UserData\ into the new version; the world
    is then reimported WITHOUT the original Webzen data (the client's quests, titles,
    masteries and experience, as in v0.3.1, and no world object quests).

From v0.3.0: with the launcher closed, copy the UserData\ folder of the old version into
the new one. On the first PLAY, the launcher adds the Gacha table, reimports the world,
checks the databases (archcore db-check) and rewrites Client\server-data; accounts,
characters and bots are kept.

From v0.2.0: with the launcher closed, copy the UserData\ folder of the old version into the new one
(accounts, characters, settings, backups). Also copy your game data (Client\) if you had
put it there and the new package does not have it.
On the first PLAY, the launcher applies the database migrations, reimports the world with
the Episode 8 values, then brings your characters to the same values (archcore
migrate-ep8), once, after an "avant-ep8" backup (Backups tab): items, options and skills
that only Archonia had are set aside (nothing is destroyed, it can be undone) and a free
skill reset is offered, since skill costs changed.


6. WHAT'S NEW — WHAT TO TRY
---------------------------

New in v0.3.4
  * the launcher is now called ArchHome (ArchHome.exe) and has a new background; the
    world is called "ArchHome" in the server list;
  * quests and titles translated into every game language (missing original texts are
    completed by ArchCore's translations);
  * quest markers above NPCs and scenery objects;
  * gathering animation;
  * the player's options (trade, party, guild refusals…) are respected;
  * bots: when you lower their number (/bot population), the surplus bots leave the
    world.

New in v0.3.3
  * 153 more quests, given or completed by scenery objects (3,235 quests in all): walk up
    to the object and click it to open its quest window, as with an NPC;
  * gathering: quest items are gathered from the scenery (5-second bar), for 588 quests.

New in v0.3.2
  * original quests, titles, masteries and experience curve (official Webzen client):
    3,082 quests, 2,768 titles, 287 masteries, experience up to level 121;
  * levelling is SLOWER than in v0.3.1: this is intended (original curve);
  * a guild no longer disbands too early (24-hour wait);
  * original titles and masteries in ArchRust too, Archonia's confirmations, minimap and
    quick bar.

New in v0.3.1
  * Episode 8 Gacha: draw sockets, cost in passes, gold and charisma, explained
    refusals (full Cash inventory, level…);
  * bug report #2 fixed: taking back an item offered in a trade, lottery, appearance,
    inventory, skills of classes without evolutions;
  * loot: a monster too far from your level (11 levels or more, both ways) no longer
    drops loot;
  * while casting, the caster stays in place; every timed spell is announced;
  * ArchCore Admin 1.4.0 (/aca): explained refusals, monster list, item details and
    class filter.

Since v0.2.0

Original Episode 8 values
  * skills, items, options, runes, monsters, refining, regions, shops, portals and
    experience use the original game's values; Archonia's additions are left out. Level
    cap 120.
  * tell us anything that looks wrong: a price, damage, a required level, a shop, a
    monster too strong or too weak.
Bots (ArchBot)
  * autonomous bots populate the world (15 at start): they hunt, loot, equip, rest and
    come back after death;
  * hire a companion: "/bot hire" near a bot (or "/bot hire Name"), accept its party
    invitation; it follows and defends you. Orders: /bot follow, stay, come, attack,
    passive, defensive, aggressive, stop, status; also by whispering to the bot or with /p
    to the whole party. "/bot dismiss" releases it; "/bot list";
  * GM (level 2): /bot stats, /bot spawn n, /bot despawn n|all|name, /bot population n;
  * settings: launcher's ArchBot tab (on or off, autonomous, companions, numbers,
    levels, classes, behaviour, survival, performance), applied on the next start.
New systems
  * titles (674 titles, Title Manager), Dragon Scion metamorphosis and evolution,
    consumables (stews, talismans, resurrection orbs, Return Orb, Secret Safe…), Cash
    inventory and pet bag, party recall, social actions, quick bar kept by the server,
    return to character selection, private trade (Ex grid), Shrine shop, arrows and bolts
    for bows and crossbows, timed items.
ArchRust
  * ArchRust login panel (saved account), original camera, water, grass, animated
    scenery, shadows, 3D sound, mastery windows.
ArchCore Admin 1.3.0 (/aca)
  * visual item picker, loot simulator.

The launcher changed too:
  * it writes the server's tables for ArchRust (Client\server-data) and rewrites them
    after each import;
  * the import requires the original server tables: if some are missing, the start stops
    and says so (instead of an empty world);
  * new ArchBot tab: every bot setting, and the in-game commands;
  * automatic update of a 0.2.0 installation, characters included.

Two characters at once (party, trade, guild): while the game runs, you can open
Client\ArchRust.exe a second time and log in with the other account. Bots can also be
party partners.


7. KNOWN ISSUES
---------------

* Do not use World > "Reimport the world database": without the Webzen data (not
  included), the world would go back to the client's quests, titles, masteries and
  experience; the launcher reports it if it happens.
* The bots' model names show in Korean in the server log: only in the log.
* Bots do not do yet: shopping, quests, player versus player, mounts, guilds, healing
  allies.


8. THE LOGS TO SEND US
----------------------

Easiest: Backups tab > "Export a diagnostic". The launcher creates
UserData\diagnostic-<date>.zip (recent launcher, server, client and MariaDB logs,
configuration, passwords masked) and opens its folder. Send that file.

Otherwise, the logs are in Logs\:
  * Logs\launcher.log            the launcher (steps, errors);
  * Logs\server\archcore-*.log   the ArchCore server, one file per start ([archbot] lines
                                 for the bots);
  * Logs\server\outils-*.log     database setup, world import, table export;
  * Logs\client\archrust-*.log   the ArchRust client, one file per game;
  * Logs\mariadb-error.log       the database.
They are also shown in the launcher's Logs tab.

Along with the logs, tell us: what you were doing, what you expected, what happened, the
approximate time and the character name. A screenshot helps a lot.


9. GOOD TO KNOW
---------------

* Everything stays in the folder: delete it to uninstall. Keep UserData\ if you want to
  keep your characters.
* Backups tab: back up and restore your characters.
* Startup trouble? Home tab > Check, then Repair.
* World tab > "Reimport the world database": runs the import again (accounts and
  characters kept); avoid it in this build, see KNOWN ISSUES.
* Do not edit Client\server-data or Client\realmlist.wtf: the launcher writes them.
