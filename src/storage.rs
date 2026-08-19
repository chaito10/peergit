use crate::error::Result;
use rusqlite::{params, Connection};
use std::path::Path;

pub struct Database {
    pub conn: Connection,
}

pub type RepoRecord = (String, Option<String>, String, String, String, Option<String>);
pub type RepoSummary = (String, String, Option<String>, String);
pub type PeerRecord = (String, String, Option<String>, Option<String>, String);

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_tables()?;
        Ok(db)
    }

    fn init_tables(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS identity (
                public_key TEXT PRIMARY KEY,
                secret_key_encrypted BLOB,
                did TEXT NOT NULL,
                alias TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS repositories (
                rid TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                path TEXT NOT NULL,
                owner_key TEXT NOT NULL,
                visibility TEXT NOT NULL DEFAULT 'public',
                fossil_db_path TEXT,
                created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS known_peers (
                peer_id TEXT PRIMARY KEY,
                public_key TEXT NOT NULL,
                alias TEXT,
                addresses TEXT,
                first_seen TEXT NOT NULL,
                last_seen TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS advertised_repos (
                rid TEXT NOT NULL,
                peer_id TEXT NOT NULL,
                announced_at TEXT NOT NULL,
                PRIMARY KEY (rid, peer_id)
            );",
        )?;
        Ok(())
    }

    pub fn store_identity(
        &self,
        pk: &str,
        did: &str,
        alias: Option<&str>,
    ) -> Result<()> {
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        self.conn.execute(
            "INSERT OR REPLACE INTO identity (public_key, did, alias, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![pk, did, alias, now],
        )?;
        Ok(())
    }

    pub fn load_identity(
        &self,
        pk: &str,
    ) -> Result<Option<(String, Option<String>)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT did, alias FROM identity WHERE public_key = ?1")?;
        let mut rows = stmt.query_map(params![pk], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
            ))
        })?;
        Ok(rows.next().transpose()?)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn store_repository(
        &self,
        rid: &str,
        name: &str,
        description: Option<&str>,
        path: &str,
        owner_key: &str,
        visibility: &str,
        fossil_db_path: Option<&str>,
    ) -> Result<()> {
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        self.conn.execute(
            "INSERT OR REPLACE INTO repositories (rid, name, description, path, owner_key, visibility, fossil_db_path, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![rid, name, description, path, owner_key, visibility, fossil_db_path, now],
        )?;
        Ok(())
    }

    pub fn load_repository(
        &self,
        rid: &str,
    ) -> Result<Option<RepoRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, description, path, owner_key, visibility, fossil_db_path FROM repositories WHERE rid = ?1",
        )?;
        let mut rows = stmt.query_map(params![rid], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, Option<String>>(5)?,
            ))
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn list_repositories(
        &self,
    ) -> Result<Vec<RepoSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT rid, name, description, visibility FROM repositories ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn store_peer(
        &self,
        peer_id: &str,
        public_key: &str,
        alias: Option<&str>,
        addresses: Option<&str>,
    ) -> Result<()> {
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        self.conn.execute(
            "INSERT OR REPLACE INTO known_peers (peer_id, public_key, alias, addresses, first_seen, last_seen) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
            params![peer_id, public_key, alias, addresses, now],
        )?;
        Ok(())
    }

    pub fn update_peer_seen(&self, peer_id: &str) -> Result<()> {
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        self.conn.execute(
            "UPDATE known_peers SET last_seen = ?1 WHERE peer_id = ?2",
            params![now, peer_id],
        )?;
        Ok(())
    }

    pub fn list_peers(
        &self,
    ) -> Result<Vec<PeerRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT peer_id, public_key, alias, addresses, last_seen FROM known_peers ORDER BY last_seen DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
            ))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn advertise_repo(
        &self,
        rid: &str,
        peer_id: &str,
    ) -> Result<()> {
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        self.conn.execute(
            "INSERT OR REPLACE INTO advertised_repos (rid, peer_id, announced_at) VALUES (?1, ?2, ?3)",
            params![rid, peer_id, now],
        )?;
        Ok(())
    }

    pub fn list_advertised_repos(
        &self,
        peer_id: &str,
    ) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT rid FROM advertised_repos WHERE peer_id = ?1")?;
        let rows = stmt.query_map(params![peer_id], |row| row.get::<_, String>(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}
