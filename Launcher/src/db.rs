//! MariaDB portabl : configuration, initialisation du datadir, démarrage, santé, arrêt propre, requêtes,
//! comptes ArchCore (table `archcore_auth.account`).

use crate::credentials::DbCredentials;
use crate::paths::{Layout, ALL_DBS, DB_AUTH, DB_CHARACTERS, DB_WORLD, MARIADB_PORT};
use mysql::prelude::*;
use mysql::{Conn, OptsBuilder, Row, Value};
use sha2::{Digest, Sha256};
use std::net::{SocketAddr, TcpStream};
use std::os::windows::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone)]
pub struct MariaDb {
    layout: Layout,
    creds: DbCredentials,
}

#[derive(Clone, Debug, Default)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub truncated: bool,
    /// Pour une instruction d'écriture : lignes affectées
    pub affected: Option<u64>,
}

#[derive(Clone, Debug, Default)]
pub struct ColumnInfo {
    pub name: String,
    pub col_type: String,
    pub nullable: bool,
    pub primary: bool,
    pub default: Option<String>,
    pub extra: String,
}

#[derive(Clone, Debug, Default)]
pub struct AccountRow {
    pub username: String,
    pub gmlevel: u8,
    pub last_login: String,
    pub characters: u64,
}

/// État du monde importé dans `archcore_world`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldStatus {
    pub templates: u64,
    pub items: u64,
    pub npcs: u64,
    pub monsters: u64,
}

impl WorldStatus {
    pub fn imported(&self) -> bool {
        self.templates > 0 && self.items > 0
    }
}

/// Littéral SQL à partir d'une saisie : `NULL` (insensible à la casse) → NULL, sinon chaîne échappée.
pub fn sql_literal(v: &str) -> String {
    if v.eq_ignore_ascii_case("null") {
        "NULL".into()
    } else {
        format!("'{}'", v.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

pub fn is_read_statement(sql: &str) -> bool {
    let head = sql.trim_start().to_ascii_uppercase();
    head.starts_with("SELECT") || head.starts_with("SHOW") || head.starts_with("DESCRIBE") || head.starts_with("EXPLAIN") || head.starts_with("DESC ")
}

pub fn port_open(port: u16) -> bool {
    let addr: SocketAddr = ([127, 0, 0, 1], port).into();
    TcpStream::connect_timeout(&addr, Duration::from_millis(400)).is_ok()
}

/// Empreinte du mot de passe telle qu'ArchCore la calcule : SHA-256 hexadécimal du texte en Windows-1252.
/// Les mots de passe acceptés par le launcher sont en ASCII imprimable, identique en Windows-1252.
pub fn password_hash(pass: &str) -> String {
    Sha256::digest(pass.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

/// Identifiant accepté : 1 à 16 lettres ou chiffres ASCII.
pub fn valid_username(u: &str) -> bool {
    (1..=16).contains(&u.len()) && u.chars().all(|c| c.is_ascii_alphanumeric())
}

/// Mot de passe accepté : 4 à 16 caractères ASCII imprimables, sans espace.
pub fn valid_password(p: &str) -> bool {
    (4..=16).contains(&p.len()) && p.chars().all(|c| c.is_ascii_graphic())
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::NULL => "NULL".to_string(),
        Value::Bytes(b) => String::from_utf8_lossy(b).into_owned(),
        Value::Int(i) => i.to_string(),
        Value::UInt(u) => u.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Double(d) => d.to_string(),
        other => other.as_sql(false),
    }
}

impl MariaDb {
    pub fn new(layout: Layout, creds: DbCredentials) -> MariaDb {
        MariaDb { layout, creds }
    }

    pub fn creds(&self) -> &DbCredentials {
        &self.creds
    }

    fn ini_path(&self) -> std::path::PathBuf {
        self.layout.server_config().join("my.ini")
    }

    /// my.ini géré : régénéré à chaque démarrage, aucune donnée personnelle dedans. Chemins avec des « / » : dans
    /// un fichier d'options, MariaDB lit `\s`, `\t`, `\n`, `\r`, `\b` comme des échappements (un dossier
    /// « …\test\… » ou « …\scratch\… » empêcherait le démarrage).
    pub fn write_my_ini(&self) -> Result<(), String> {
        let l = &self.layout;
        let p = |p: std::path::PathBuf| p.display().to_string().replace('\\', "/");
        std::fs::create_dir_all(l.db_data()).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(l.logs()).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(l.server_config()).map_err(|e| e.to_string())?;
        let ini = format!(
            "[mysqld]\nbasedir={base}\ndatadir={data}\nport={port}\nbind-address=127.0.0.1\nskip-name-resolve\n\
max_allowed_packet=256M\ncharacter-set-server=utf8mb4\ncollation-server=utf8mb4_general_ci\n\
innodb_buffer_pool_size=256M\ninnodb_flush_log_at_trx_commit=2\nlog_error={log}\npid-file={pid}\n\n[client]\nport={port}\ndefault-character-set=utf8mb4\n",
            base = p(l.mariadb_dir()),
            data = p(l.db_data()),
            port = MARIADB_PORT,
            log = p(l.logs().join("mariadb-error.log")),
            pid = p(l.db_data().join("mariadb.pid")),
        );
        std::fs::write(self.ini_path(), ini).map_err(|e| e.to_string())
    }

    pub fn is_initialized(&self) -> bool {
        self.layout.db_data().join("mysql").is_dir()
    }

    /// Premier lancement : création du datadir avec le mot de passe root généré.
    pub fn initialize(&self) -> Result<(), String> {
        let exe = self.layout.mariadb_bin().join("mariadb-install-db.exe");
        let status = Command::new(&exe)
            .arg(format!("--datadir={}", self.layout.db_data().display()))
            .arg(format!("--password={}", self.creds.root_password))
            .arg(format!("--port={MARIADB_PORT}"))
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| crate::util::spawn_error("mariadb-install-db", &e))?;
        if !status.success() {
            return Err(format!("mariadb-install-db : code {:?}", status.code()));
        }
        Ok(())
    }

    fn opts(&self, root: bool, db: Option<&str>) -> OptsBuilder {
        let (u, p) = if root { ("root", self.creds.root_password.as_str()) } else { (self.creds.user.as_str(), self.creds.password.as_str()) };
        OptsBuilder::new()
            .ip_or_hostname(Some("127.0.0.1"))
            .tcp_port(MARIADB_PORT)
            .prefer_socket(false)
            .user(Some(u))
            .pass(Some(p))
            .db_name(db)
            .tcp_connect_timeout(Some(Duration::from_secs(3)))
    }

    pub fn root_conn(&self) -> Result<Conn, String> {
        Conn::new(self.opts(true, None)).map_err(|e| format!("SQL (root) : {e}"))
    }

    fn root_conn_db(&self, db: Option<&str>) -> Result<Conn, String> {
        Conn::new(self.opts(true, db)).map_err(|e| format!("SQL : {e}"))
    }

    /// Santé réelle : connexion authentifiée ET requête aboutie. Pas de sondage du port : une connexion TCP fermée
    /// avant l'authentification est notée « Aborted connection » dans le journal de MariaDB (une par seconde en jeu).
    /// La connexion est refermée par COM_QUIT (fin de `Conn`).
    pub fn ping(&self) -> bool {
        match self.root_conn() {
            Ok(mut c) => c.query_first::<u64, _>("SELECT 1").map(|v| v == Some(1)).unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Démarre mariadbd (détaché, sans fenêtre) et attend la santé réelle.
    /// Renvoie `true` si le processus a été lancé ici, `false` s'il était déjà en service (il ne nous appartient pas).
    pub fn start(&self, timeout: Duration) -> Result<bool, String> {
        if self.ping() {
            return Ok(false);
        }
        if port_open(MARIADB_PORT) {
            return Err(crate::i18n::port_busy(MARIADB_PORT));
        }
        let exe = self.layout.mariadb_bin().join("mariadbd.exe");
        let mut child = Command::new(&exe)
            .arg(format!("--defaults-file={}", self.ini_path().display()))
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| crate::util::spawn_error("mariadbd", &e))?;
        let start = Instant::now();
        while start.elapsed() < timeout {
            if self.ping() {
                return Ok(true);
            }
            if let Ok(Some(st)) = child.try_wait() {
                return Err(crate::tf!("mariadbd s'est arrêté (code {:?}), voir Logs/mariadb-error.log", "mariadbd stopped (code {:?}), see Logs/mariadb-error.log", st.code()));
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        Err(crate::i18n::DB_TIMEOUT.s().into())
    }

    /// Arrêt propre par la commande SQL SHUTDOWN (jamais de kill). La fin est guettée par la disparition du fichier
    /// pid (MariaDB l'efface en dernier), pas en sondant le port.
    pub fn shutdown(&self, timeout: Duration) -> Result<(), String> {
        let mut c = match self.root_conn() {
            Ok(c) => c,
            Err(_) if !port_open(MARIADB_PORT) => return Ok(()),
            Err(e) => return Err(e),
        };
        let pid = self.layout.db_data().join("mariadb.pid");
        let watch_pid = pid.exists();
        let _ = c.query_drop("SHUTDOWN");
        drop(c);
        let start = Instant::now();
        while start.elapsed() < timeout {
            let stopped = if watch_pid { !pid.exists() } else { self.root_conn().is_err() };
            if stopped {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(300));
        }
        Err(crate::i18n::DB_STOP_TIMEOUT.s().into())
    }

    /// Compte SQL dédié à ArchCore : tous les droits sur les bases `archcore_*` (db-init les crée lui-même).
    pub fn ensure_user(&self) -> Result<(), String> {
        let user = &self.creds.user;
        let pass = &self.creds.password;
        let mut c = self.root_conn()?;
        for host in ["127.0.0.1", "localhost"] {
            c.query_drop(format!("CREATE USER IF NOT EXISTS '{user}'@'{host}' IDENTIFIED BY '{pass}'")).map_err(|e| e.to_string())?;
            c.query_drop(format!("ALTER USER '{user}'@'{host}' IDENTIFIED BY '{pass}'")).map_err(|e| e.to_string())?;
            c.query_drop(format!("GRANT ALL PRIVILEGES ON `archcore\\_%`.* TO '{user}'@'{host}'")).map_err(|e| e.to_string())?;
        }
        c.query_drop("FLUSH PRIVILEGES").map_err(|e| e.to_string())?;
        Ok(())
    }

    /// Comptes créés par mariadb-install-db pour le nom du PC (`root@<nom du PC>`, et l'anonyme s'il existe) :
    /// inutiles sur une base qui n'écoute que 127.0.0.1, et signalés à chaque démarrage (skip-name-resolve).
    /// Rend les comptes supprimés.
    pub fn drop_host_accounts(&self) -> Result<Vec<String>, String> {
        let mut c = self.root_conn()?;
        let rows: Vec<(String, String)> = c
            .query("SELECT User, Host FROM mysql.user WHERE (User = 'root' OR User = '') AND Host NOT IN ('localhost', '127.0.0.1', '::1')")
            .map_err(|e| e.to_string())?;
        let mut dropped = Vec::new();
        for (user, host) in rows {
            let esc = |s: &str| s.replace('\\', "\\\\").replace('\'', "\\'");
            c.query_drop(format!("DROP USER IF EXISTS '{}'@'{}'", esc(&user), esc(&host))).map_err(|e| e.to_string())?;
            c.query_drop(format!("DELETE FROM mysql.proxies_priv WHERE User = '{}' AND Host = '{}'", esc(&user), esc(&host))).map_err(|e| e.to_string())?;
            dropped.push(format!("{user}@{host}"));
        }
        if !dropped.is_empty() {
            c.query_drop("FLUSH PRIVILEGES").map_err(|e| e.to_string())?;
        }
        Ok(dropped)
    }

    /// Bases absentes parmi les quatre d'ArchCore.
    pub fn databases_missing(&self) -> Result<Vec<&'static str>, String> {
        let mut c = self.root_conn()?;
        let present: Vec<String> = c.query("SELECT schema_name FROM information_schema.schemata WHERE schema_name LIKE 'archcore\\_%'").map_err(|e| e.to_string())?;
        Ok(ALL_DBS.iter().copied().filter(|d| !present.iter().any(|p| p == d)).collect())
    }

    fn count(&self, sql: &str) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        c.query_first::<u64, _>(sql).map(|v| v.unwrap_or(0)).map_err(|e| e.to_string())
    }

    pub fn characters_count(&self) -> Result<u64, String> {
        self.count(&format!("SELECT COUNT(*) FROM {DB_CHARACTERS}.characters WHERE deleted_at IS NULL"))
    }

    pub fn accounts_count(&self) -> Result<u64, String> {
        self.count(&format!("SELECT COUNT(*) FROM {DB_AUTH}.account"))
    }

    /// Contenu de la base monde (0 partout si elle n'existe pas encore).
    pub fn world_status(&self) -> WorldStatus {
        let n = |t: &str| self.count(&format!("SELECT COUNT(*) FROM {DB_WORLD}.`{t}`")).unwrap_or(0);
        WorldStatus { templates: n("character_template"), items: n("item_template"), npcs: n("npc_spawn"), monsters: n("monster_spawn") }
    }

    /// Nom et adresse du monde dans la liste de l'écran de connexion (table `world` : c'est elle que lit le
    /// serveur de connexion, pas world.conf).
    pub fn ensure_world_row(&self, world_port: u16) -> Result<(), String> {
        let mut c = self.root_conn()?;
        let name = crate::paths::WORLD_NAME;
        c.query_drop(format!("INSERT INTO {DB_AUTH}.server_group (id, name) VALUES (1, '{name}') ON DUPLICATE KEY UPDATE name='{name}'")).map_err(|e| e.to_string())?;
        c.query_drop(format!(
            "INSERT INTO {DB_AUTH}.world (id, group_id, name, address, status, flag) VALUES (1, 1, '{name}', '127.0.0.1:{world_port}', 4, 4) \
             ON DUPLICATE KEY UPDATE group_id=1, name='{name}', address='127.0.0.1:{world_port}'"
        ))
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn client_args(&self) -> Vec<String> {
        vec![
            "--host=127.0.0.1".into(),
            format!("--port={MARIADB_PORT}"),
            "--protocol=TCP".into(),
            "--user=root".into(),
            format!("--password={}", self.creds.root_password),
            "--default-character-set=utf8mb4".into(),
        ]
    }

    /// Import d'un fichier SQL dans une base existante (client mariadb.exe, entrée standard = fichier).
    pub fn import_file(&self, db: &str, file: &std::path::Path) -> Result<(), String> {
        let f = std::fs::File::open(file).map_err(|e| format!("{} : {e}", file.display()))?;
        let mut c = self.root_conn()?;
        c.query_drop(format!("CREATE DATABASE IF NOT EXISTS `{db}` DEFAULT CHARSET utf8mb4 COLLATE utf8mb4_general_ci")).map_err(|e| e.to_string())?;
        drop(c);
        let status = Command::new(self.layout.mariadb_bin().join("mariadb.exe"))
            .args(self.client_args())
            .args(["--max_allowed_packet=268435456", db])
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::from(f))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| crate::util::spawn_error("mariadb.exe", &e))?;
        if !status.success() {
            return Err(format!("import {} : code {:?}", file.display(), status.code()));
        }
        Ok(())
    }

    /// Dump cohérent (InnoDB : instantané transactionnel, sans bloquer le serveur).
    pub fn dump_database(&self, db: &str, out: &std::path::Path) -> Result<(), String> {
        let status = Command::new(self.layout.mariadb_bin().join("mariadb-dump.exe"))
            .args(self.client_args())
            .args(["--single-transaction", "--add-drop-table", "--routines", "--triggers", "--skip-dump-date"])
            .arg(format!("--result-file={}", out.display()))
            .arg(db)
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| crate::util::spawn_error("mariadb-dump", &e))?;
        if !status.success() || !out.is_file() {
            return Err(format!("mariadb-dump {db} : code {:?}", status.code()));
        }
        Ok(())
    }

    // ------------------------------------------------------------------ comptes

    pub fn accounts(&self) -> Result<Vec<AccountRow>, String> {
        let mut c = self.root_conn()?;
        c.query_map(
            format!(
                "SELECT a.username, a.gm_level, COALESCE(DATE_FORMAT(a.last_login,'%Y-%m-%d %H:%i'),''), \
                 (SELECT COUNT(*) FROM {DB_CHARACTERS}.characters ch WHERE ch.account_id=a.id AND ch.deleted_at IS NULL) \
                 FROM {DB_AUTH}.account a ORDER BY a.username"
            ),
            |(username, gm, ll, nb): (String, u8, String, u64)| AccountRow { username, gmlevel: gm, last_login: ll, characters: nb },
        )
        .map_err(|e| e.to_string())
    }

    pub fn account_exists(&self, user: &str) -> Result<bool, String> {
        if !valid_username(user) {
            return Ok(false);
        }
        Ok(self.count(&format!("SELECT COUNT(*) FROM {DB_AUTH}.account WHERE username='{user}'"))? > 0)
    }

    /// Création directe en base, comme `archcore db-init --account` (même empreinte de mot de passe).
    pub fn create_account(&self, user: &str, pass: &str, gm: u8) -> Result<(), String> {
        if !valid_username(user) || !valid_password(pass) {
            return Err(crate::i18n::ACC_INVALID.s().into());
        }
        let mut c = self.root_conn()?;
        c.exec_drop(format!("INSERT INTO {DB_AUTH}.account (username, password_hash, gm_level) VALUES (?,?,?)"), (user, password_hash(pass), gm.min(3)))
            .map_err(|e| e.to_string())
    }

    pub fn set_password(&self, user: &str, pass: &str) -> Result<u64, String> {
        if !valid_username(user) || !valid_password(pass) {
            return Err(crate::i18n::ACC_INVALID.s().into());
        }
        let mut c = self.root_conn()?;
        c.exec_drop(format!("UPDATE {DB_AUTH}.account SET password_hash=? WHERE username=?"), (password_hash(pass), user)).map_err(|e| e.to_string())?;
        Ok(c.affected_rows())
    }

    pub fn set_gm(&self, user: &str, gm: u8) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        c.exec_drop(format!("UPDATE {DB_AUTH}.account SET gm_level=? WHERE username=?"), (gm.min(3), user)).map_err(|e| e.to_string())?;
        Ok(c.affected_rows())
    }

    /// Supprime un compte, ses personnages (objets, quêtes, compétences, appartenance aux guildes : clés
    /// étrangères ON DELETE CASCADE) et sa banque. Renvoie le nombre de personnages supprimés.
    pub fn delete_account(&self, user: &str) -> Result<u64, String> {
        let mut c = self.root_conn()?;
        let err = |e: mysql::Error| e.to_string();
        let Some(id) = c.exec_first::<u32, _, _>(format!("SELECT id FROM {DB_AUTH}.account WHERE username=?"), (user,)).map_err(err)? else {
            return Err(crate::tf!("compte {user} introuvable", "account {user} not found"));
        };
        let n: u64 = c.query_first(format!("SELECT COUNT(*) FROM {DB_CHARACTERS}.characters WHERE account_id={id}")).map_err(err)?.unwrap_or(0);
        for sql in [
            format!("DELETE FROM {DB_CHARACTERS}.characters WHERE account_id={id}"),
            format!("DELETE FROM {DB_CHARACTERS}.account_bank WHERE account_id={id}"),
            format!("DELETE FROM {DB_AUTH}.account WHERE id={id}"),
        ] {
            c.query_drop(&sql).map_err(|e| format!("{e} ({sql})"))?;
        }
        Ok(n)
    }

    // ------------------------------------------------------------------ explorateur

    /// Requête de lecture (explorateur intégré). Les écritures passent par `exec`.
    pub fn query(&self, db: Option<&str>, sql: &str, max_rows: usize) -> Result<QueryResult, String> {
        if !is_read_statement(sql) {
            return Err(crate::i18n::DB_READONLY_ONLY.s().to_string());
        }
        let mut c = self.root_conn_db(db)?;
        let mut result = c.query_iter(sql).map_err(|e| e.to_string())?;
        let mut out = QueryResult::default();
        if let Some(set) = result.iter() {
            out.columns = set.columns().as_ref().iter().map(|c| c.name_str().into_owned()).collect();
            for row in set {
                let row: Row = row.map_err(|e| e.to_string())?;
                if out.rows.len() >= max_rows {
                    out.truncated = true;
                    break;
                }
                out.rows.push((0..row.len()).map(|i| row.as_ref(i).map(value_to_string).unwrap_or_default()).collect());
            }
        }
        Ok(out)
    }

    /// Exécute une instruction d'écriture (UPDATE/INSERT/DELETE/… ) et renvoie le nombre de lignes affectées.
    pub fn exec(&self, db: Option<&str>, sql: &str) -> Result<u64, String> {
        let mut c = self.root_conn_db(db)?;
        c.query_drop(sql).map_err(|e| e.to_string())?;
        Ok(c.affected_rows())
    }

    /// Colonnes d'une table : (nom, type, nullable, clé, défaut).
    pub fn table_columns(&self, db: &str, table: &str) -> Result<Vec<ColumnInfo>, String> {
        let mut c = self.root_conn()?;
        let rows: Vec<(String, String, String, String, Option<String>, String)> = c
            .exec(
                "SELECT column_name, column_type, is_nullable, column_key, column_default, extra FROM information_schema.columns \
                 WHERE table_schema=? AND table_name=? ORDER BY ordinal_position",
                (db, table),
            )
            .map_err(|e| e.to_string())?;
        Ok(rows.into_iter().map(|(name, col_type, nullable, key, default, extra)| ColumnInfo { name, col_type, nullable: nullable == "YES", primary: key == "PRI", default, extra }).collect())
    }

    /// Lignes d'une table avec filtre et pagination ; renvoie aussi le total filtré.
    pub fn table_rows(&self, db: &str, table: &str, filter: &str, order: Option<(&str, bool)>, offset: u64, limit: u64) -> Result<(QueryResult, u64), String> {
        let where_clause = if filter.trim().is_empty() { String::new() } else { format!(" WHERE {}", filter.trim()) };
        let order_clause = match order {
            Some((col, asc)) => format!(" ORDER BY `{col}` {}", if asc { "ASC" } else { "DESC" }),
            None => String::new(),
        };
        let mut c = self.root_conn_db(Some(db))?;
        let total: Option<u64> = c.query_first(format!("SELECT COUNT(*) FROM `{table}`{where_clause}")).map_err(|e| e.to_string())?;
        let res = self.query(Some(db), &format!("SELECT * FROM `{table}`{where_clause}{order_clause} LIMIT {limit} OFFSET {offset}"), limit as usize)?;
        Ok((res, total.unwrap_or(0)))
    }

    pub fn tables(&self, db: &str) -> Result<Vec<String>, String> {
        let mut c = self.root_conn()?;
        c.exec("SELECT table_name FROM information_schema.tables WHERE table_schema=? ORDER BY table_name", (db,)).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn password_hash_matches_archcore() {
        // SHA-256 de « test » : compte de test d'ArchCore (db-init --account test test).
        assert_eq!(super::password_hash("test"), "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08");
    }

    #[test]
    fn account_rules() {
        assert!(super::valid_username("Bob42"));
        assert!(!super::valid_username("bob smith"));
        assert!(!super::valid_username(""));
        assert!(super::valid_password("abc!"));
        assert!(!super::valid_password("a b c d"));
        assert!(!super::valid_password("é123"));
    }
}
