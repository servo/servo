/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use rusqlite::Connection;

/// A schema change. The list a database is given has to be ordered by `version`.
pub(crate) struct Migration {
    pub(crate) version: u32,
    pub(crate) statements: &'static str,
}

pub(crate) fn apply(connection: &mut Connection, migrations: &[Migration]) -> rusqlite::Result<()> {
    let version: u32 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let pending = migrations
        .iter()
        .filter(|migration| migration.version > version);
    for migration in pending {
        let transaction = connection.transaction()?;
        transaction.execute_batch(migration.statements)?;
        transaction.pragma_update(None, "user_version", migration.version)?;
        transaction.commit()?;
    }
    Ok(())
}
