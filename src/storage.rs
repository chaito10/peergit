use crate::error::Result;
use crate::identity::Visibility;
use rusqlite::{params, Connection};
use std::path::Path;

const SCHEMA_VERSION: i64 = 2;

#[derive(Debug, Clone)]
pub struct IdentityRecord {
    pub public_key: String,
    pub did: String,
    pub alias: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone)]
pub struct RepositoryRecord {
    pub rid: String,
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub owner_key: String,
    pub visibility: Visibility,
    pub fossil_db_path: Option<String>,
    pub created_at: String,
    pub protocol_version: u16,
    pub creation_nonce: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RepositorySummary {
    pub rid: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone)]
pub struct PeerRecord {
    pub peer_id: String,
    pub public_key: String,
    pub alias: Option<String>,
    pub addresses: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone)]
pub struct AdvertisedRepoRecord {
    pub rid: String,
    pub peer_id: String,
    pub announced_at: String,
    pub advertisement_json: Option<String>,
}

pub struct Database {
    pub conn: Connection,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    pub fn migrate(&self) -> Result<()> {
        let current: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE((SELECT value FROM schema_version WHERE key='version'), 0)",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);

        if current < 1 {
            self.migration_001_initial()?;
        }
        if current < 2 {
            self.migration_002_advertisements()?;
        }

        self.conn.execute(
            "INSERT OR REPLACE INTO schema_version (key, value) VALUES ('version', ?1)",
            params![SCHEMA_VERSION],
        )?;
        Ok(())
    }

    fn migration_001_initial(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                key TEXT PRIMARY KEY,
                value INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS identity (
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
            );
            INSERT OR REPLACE INTO schema_version (key, value) VALUES ('version', 1);",
        )?;
        Ok(())
    }

    fn migration_002_advertisements(&self) -> Result<()> {
        let has_column = |table: &str, column: &str| -> Result<bool> {
            let mut stmt = self
                .conn
                .prepare(&format!("PRAGMA table_info({table})"))?;
            let mut rows = stmt.query([])?;
            while let Some(row) = rows.next()? {
                let name: String = row.get(1)?;
                if name == column {
                    return Ok(true);
                }
            }
            Ok(false)
        };

        if !has_column("repositories", "protocol_version")? {
            self.conn
                .execute_batch("ALTER TABLE repositories ADD COLUMN protocol_version INTEGER NOT NULL DEFAULT 1;")?;
        }
        if !has_column("repositories", "creation_nonce")? {
            self.conn
                .execute_batch("ALTER TABLE repositories ADD COLUMN creation_nonce TEXT;")?;
        }
        if !has_column("advertised_repos", "advertisement_json")? {
            self.conn
                .execute_batch("ALTER TABLE advertised_repos ADD COLUMN advertisement_json TEXT;")?;
        }
        Ok(())
    }

    pub fn schema_version(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row(
                "SELECT COALESCE((SELECT value FROM schema_version WHERE key='version'), 0)",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0))
    }

    pub fn store_identity(&self, pk: &str, did: &str, alias: Option<&str>) -> Result<()> {
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        self.conn.execute(
            "INSERT INTO identity (public_key, did, alias, created_at) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(public_key) DO UPDATE SET did = excluded.did, alias = excluded.alias",
            params![pk, did, alias, now],
        )?;
        Ok(())
    }

    pub fn load_identity(&self, pk: &str) -> Result<Option<IdentityRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT public_key, did, alias, created_at FROM identity WHERE public_key = ?1")?;
        let mut rows = stmt.query_map(params![pk], |row| {
            Ok(IdentityRecord {
                public_key: row.get(0)?,
                did: row.get(1)?,
                alias: row.get(2)?,
                created_at: row.get(3)?,
            })
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
        visibility: &Visibility,
        fossil_db_path: Option<&str>,
        protocol_version: u16,
    ) -> Result<()> {
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        self.conn.execute(
            "INSERT INTO repositories (rid, name, description, path, owner_key, visibility, fossil_db_path, created_at, protocol_version) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(rid) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                path = excluded.path,
                visibility = excluded.visibility,
                fossil_db_path = excluded.fossil_db_path",
            params![
                rid,
                name,
                description,
                path,
                owner_key,
                visibility.as_db_str(),
                fossil_db_path,
                now,
                protocol_version
            ],
        )?;
        Ok(())
    }

    pub fn store_repository_identity(
        &self,
        rid: &str,
        creation_nonce: &str,
        protocol_version: u16,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE repositories SET creation_nonce = ?1, protocol_version = ?2 WHERE rid = ?3",
            params![creation_nonce, protocol_version, rid],
        )?;
        Ok(())
    }

    pub fn load_repository(&self, rid: &str) -> Result<Option<RepositoryRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT rid, name, description, path, owner_key, visibility, fossil_db_path, created_at, protocol_version, creation_nonce FROM repositories WHERE rid = ?1",
        )?;
        let mut rows = stmt.query_map(params![rid], |row| {
            let vis_str: String = row.get(5)?;
            Ok(RepositoryRecord {
                rid: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                path: row.get(3)?,
                owner_key: row.get(4)?,
                visibility: Visibility::from_db_str(&vis_str),
                fossil_db_path: row.get(6)?,
                created_at: row.get(7)?,
                protocol_version: row.get::<_, i64>(8)? as u16,
                creation_nonce: row.get(9)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn update_repository_visibility(&self, rid: &str, visibility: &Visibility) -> Result<()> {
        self.conn.execute(
            "UPDATE repositories SET visibility = ?1 WHERE rid = ?2",
            params![visibility.as_db_str(), rid],
        )?;
        Ok(())
    }

    pub fn list_repositories(&self) -> Result<Vec<RepositorySummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT rid, name, description, visibility FROM repositories ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            let vis_str: String = row.get(3)?;
            Ok(RepositorySummary {
                rid: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                visibility: Visibility::from_db_str(&vis_str),
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn delete_repository(&self, rid: &str) -> Result<()> {
        self.conn.execute("DELETE FROM repositories WHERE rid = ?1", params![rid])?;
        Ok(())
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
            "INSERT INTO known_peers (peer_id, public_key, alias, addresses, first_seen, last_seen) VALUES (?1, ?2, ?3, ?4, ?5, ?5)
             ON CONFLICT(peer_id) DO UPDATE SET
                alias = COALESCE(excluded.alias, known_peers.alias),
                addresses = excluded.addresses,
                last_seen = excluded.last_seen",
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

    pub fn update_peer_addresses(&self, peer_id: &str, addresses: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE known_peers SET addresses = ?1 WHERE peer_id = ?2",
            params![addresses, peer_id],
        )?;
        Ok(())
    }

    pub fn list_peers(&self) -> Result<Vec<PeerRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT peer_id, public_key, alias, addresses, first_seen, last_seen FROM known_peers ORDER BY last_seen DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(PeerRecord {
                peer_id: row.get(0)?,
                public_key: row.get(1)?,
                alias: row.get(2)?,
                addresses: row.get(3)?,
                first_seen: row.get(4)?,
                last_seen: row.get(5)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_peer(&self, peer_id: &str) -> Result<Option<PeerRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT peer_id, public_key, alias, addresses, first_seen, last_seen FROM known_peers WHERE peer_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![peer_id], |row| {
            Ok(PeerRecord {
                peer_id: row.get(0)?,
                public_key: row.get(1)?,
                alias: row.get(2)?,
                addresses: row.get(3)?,
                first_seen: row.get(4)?,
                last_seen: row.get(5)?,
            })
        })?;
        Ok(rows.next().transpose()?)
    }

    pub fn advertise_repo(&self, rid: &str, peer_id: &str, ad_json: Option<&str>) -> Result<()> {
        let now = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default();
        self.conn.execute(
            "INSERT INTO advertised_repos (rid, peer_id, announced_at, advertisement_json) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(rid, peer_id) DO UPDATE SET
                announced_at = excluded.announced_at,
                advertisement_json = excluded.advertisement_json",
            params![rid, peer_id, now, ad_json],
        )?;
        Ok(())
    }

    pub fn list_advertised_repos(&self, peer_id: &str) -> Result<Vec<AdvertisedRepoRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT rid, peer_id, announced_at, advertisement_json FROM advertised_repos WHERE peer_id = ?1 ORDER BY announced_at DESC",
        )?;
        let rows = stmt.query_map(params![peer_id], |row| {
            Ok(AdvertisedRepoRecord {
                rid: row.get(0)?,
                peer_id: row.get(1)?,
                announced_at: row.get(2)?,
                advertisement_json: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn find_repo_providers(&self, rid: &str) -> Result<Vec<AdvertisedRepoRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT rid, peer_id, announced_at, advertisement_json FROM advertised_repos WHERE rid = ?1 ORDER BY announced_at DESC",
        )?;
        let rows = stmt.query_map(params![rid], |row| {
            Ok(AdvertisedRepoRecord {
                rid: row.get(0)?,
                peer_id: row.get(1)?,
                announced_at: row.get(2)?,
                advertisement_json: row.get(3)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn clear_advertised_repos(&self, peer_id: &str) -> Result<()> {
        self.conn.execute(
            "DELETE FROM advertised_repos WHERE peer_id = ?1",
            params![peer_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::Visibility;

    fn temp_db() -> (tempfile::TempDir, Database) {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(&dir.path().join("test.db")).unwrap();
        (dir, db)
    }

    #[test]
    fn schema_version() {
        let (_dir, db) = temp_db();
        assert!(db.schema_version().unwrap() >= 1);
    }

    #[test]
    fn repository_store_load_list() {
        let (_dir, db) = temp_db();
        db.store_repository(
            "rid1",
            "test-repo",
            Some("desc"),
            "/tmp/test",
            "pk123",
            &Visibility::Public,
            None,
            1,
        )
        .unwrap();
        let rec = db.load_repository("rid1").unwrap().unwrap();
        assert_eq!(rec.name, "test-repo");
        assert_eq!(rec.visibility, Visibility::Public);

        let list = db.list_repositories().unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn repository_store_preserves_created_at_on_update() {
        let (_dir, db) = temp_db();
        db.store_repository(
            "rid1",
            "test-repo",
            None,
            "/tmp/a",
            "pk",
            &Visibility::Public,
            None,
            1,
        )
        .unwrap();
        let first = db.load_repository("rid1").unwrap().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        db.store_repository(
            "rid1",
            "test-repo-renamed",
            None,
            "/tmp/b",
            "pk",
            &Visibility::Private { allow: vec![] },
            None,
            1,
        )
        .unwrap();
        let second = db.load_repository("rid1").unwrap().unwrap();
        assert_eq!(second.name, "test-repo-renamed");
        assert_eq!(first.created_at, second.created_at);
        assert_eq!(second.visibility, Visibility::Private { allow: vec![] });
    }

    #[test]
    fn peer_store_preserves_first_seen() {
        let (_dir, db) = temp_db();
        db.store_peer("peer1", "pk1", Some("alice"), Some("/ip4/1.2.3.4/tcp/1")).unwrap();
        let first = db.get_peer("peer1").unwrap().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        db.store_peer("peer1", "pk1", Some("alice-2"), Some("/ip4/5.6.7.8/tcp/2")).unwrap();
        let second = db.get_peer("peer1").unwrap().unwrap();
        assert_eq!(first.first_seen, second.first_seen);
        assert_eq!(second.alias.as_deref(), Some("alice-2"));
        assert_eq!(second.addresses.as_deref(), Some("/ip4/5.6.7.8/tcp/2"));
    }

    #[test]
    fn advertisements_persist() {
        let (_dir, db) = temp_db();
        db.advertise_repo("rid1", "peer1", Some("{\"json\":true}")).unwrap();
        db.advertise_repo("rid1", "peer2", None).unwrap();
        let providers = db.find_repo_providers("rid1").unwrap();
        assert_eq!(providers.len(), 2);
        let peer1 = providers.iter().find(|p| p.peer_id == "peer1").unwrap();
        assert_eq!(peer1.advertisement_json.as_deref(), Some("{\"json\":true}"));
        let peer2 = providers.iter().find(|p| p.peer_id == "peer2").unwrap();
        assert!(peer2.advertisement_json.is_none());
    }

    #[test]
    fn visibility_update() {
        let (_dir, db) = temp_db();
        db.store_repository(
            "rid1",
            "test",
            None,
            "/tmp",
            "pk",
            &Visibility::Public,
            None,
            1,
        )
        .unwrap();
        db.update_repository_visibility("rid1", &Visibility::Protected).unwrap();
        let rec = db.load_repository("rid1").unwrap().unwrap();
        assert_eq!(rec.visibility, Visibility::Protected);
    }
}

// use tempfile in tests only
#[cfg(test)]
extern crate tempfile;