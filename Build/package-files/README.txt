HomeRPG ArchLord — single-player, offline ArchLord
===================================================

(Français : voir LISEZMOI.txt — testers: read README-TESTERS.txt)

1. ArchRust.exe and its AddOns are included in Client\. Put your 64-bit ArchLord game data
   (character\, ini\, world\, texture\…) next to it, or set the data folder in Settings.
   The game data is not included.

2. The original ArchLord server tables (Episode 8 game values, monster spawns, loot) are
   included in Server\Data\server-ini\: the world import requires them.

3. Open "HomeRPG ArchLord.exe" and click PLAY.
   The first start imports the world (about a minute, once, and again after an ArchCore
   update) and writes the server's tables for ArchRust into Client\server-data. The server
   brings its own bots (ArchBot tab); in game, "/bot hire" hires a companion.

4. Create your account in the Accounts tab (GM level 3 for every command),
   then log in with it in ArchRust.

Everything runs on this PC (127.0.0.1), with no Internet connection.
Your accounts, characters, settings and backups are in UserData\: keep this folder
(to move to a new version, copy it into the new folder).
Logs: Logs\ (launcher, server, client), and the launcher's Logs tab.

Full manual: Documentation\User-Manual.md ; changes: Documentation\CHANGELOG.md
Licence: LICENCE.txt ; third-party components: LICENCES\
