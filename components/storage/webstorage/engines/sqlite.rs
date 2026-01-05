/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::path::PathBuf;
use std::sync::Arc;

use base::threadpool::ThreadPool;
use log::error;
use rusqlite::Connection;

use crate::shared::{DB_IN_MEMORY_INIT_PRAGMAS, DB_IN_MEMORY_PRAGMAS, DB_INIT_PRAGMAS, DB_PRAGMAS};
use crate::webstorage::OriginEntry;
use crate::webstorage::engines::WebStorageEngine;

pub struct SqliteEngine {
    connection: Connection,
}

impl SqliteEngine {
    pub fn new(db_dir: &Option<PathBuf>, _pool: Arc<ThreadPool>) -> rusqlite::Result<Self> {
        let connection = match db_dir {
            Some(path) => {
                let path = path.join("webstorage.sqlite");
                Self::init_db(Some(&path))?
            },
            None => Self::init_db(None)?,
        };
        Ok(SqliteEngine { connection })
    }

    pub fn init_db(db_path: Option<&PathBuf>) -> rusqlite::Result<Connection> {
        let connection = if let Some(path) = db_path {
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let conn = Connection::open(path)?;
            for pragma in DB_INIT_PRAGMAS.iter() {
                let _ = conn.execute(pragma, []);
            }
            for pragma in DB_PRAGMAS.iter() {
                let _ = conn.execute(pragma, []);
            }
            conn
        } else {
            // TODO We probably don't need an in memory implementation at all.
            // WebStorageEnvironment already keeps all key value pairs in memory via its data field.
            // A future refactoring could avoid creating a WebStorageEngine entirely when config_dir is None.
            let conn = Connection::open_in_memory()?;
            for pragma in DB_IN_MEMORY_INIT_PRAGMAS.iter() {
                let _ = conn.execute(pragma, []);
            }
            for pragma in DB_IN_MEMORY_PRAGMAS.iter() {
                let _ = conn.execute(pragma, []);
            }
            conn
        };
        connection.execute("CREATE TABLE IF NOT EXISTS data (id INTEGER PRIMARY KEY AUTOINCREMENT, key TEXT, value TEXT);", [])?;
        Ok(connection)
    }
}

impl WebStorageEngine for SqliteEngine {
    type Error = rusqlite::Error;

    fn load(&self) -> Result<OriginEntry, Self::Error> {
        let mut stmt = self.connection.prepare("SELECT key, value FROM data;")?;
        let rows = stmt.query_map([], |row| {
            let key: String = row.get(0)?;
            let value: String = row.get(1)?;
            Ok((key, value))
        })?;

        let mut map = OriginEntry::default();
        for row in rows {
            let (key, value) = row?;
            map.insert(key, value);
        }
        Ok(map)
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        self.connection.execute("DELETE FROM data;", [])?;
        Ok(())
    }

    fn delete(&mut self, key: &str) -> Result<(), Self::Error> {
        self.connection
            .execute("DELETE FROM data WHERE key = ?;", [key])?;
        Ok(())
    }

    fn set(&mut self, key: &str, value: &str) -> Result<(), Self::Error> {
        // update or insert
        //
        // TODO: Replace this with an UPSERT once the schema guarantees a
        // UNIQUE/PRIMARY KEY constraint on `key`.
        let tx = self.connection.transaction()?;
        let rows = tx.execute("UPDATE data SET value = ? WHERE key = ?", [value, key])?;
        if rows == 0 {
            tx.execute("INSERT INTO data (key, value) VALUES (?, ?)", [key, value])?;
        }
        tx.commit()?;
        Ok(())
    }

    fn save(&mut self, data: &OriginEntry) {
        fn save_inner(conn: &mut Connection, data: &OriginEntry) -> rusqlite::Result<()> {
            let tx = conn.transaction()?;
            tx.execute("DELETE FROM data;", [])?;
            let mut stmt = tx.prepare("INSERT INTO data (key, value) VALUES (?, ?);")?;
            for (key, value) in data.inner() {
                stmt.execute(rusqlite::params![key, value])?;
            }
            drop(stmt);
            tx.commit()?;
            Ok(())
        }
        if let Err(e) = save_inner(&mut self.connection, data) {
            error!("localstorage save error: {:?}", e);
        }
    }
}
