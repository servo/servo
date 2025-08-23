/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub(crate) fn create_tables(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        r#"create table database
(
    name    varchar          not null
        primary key,
    origin  varchar          not null,
    version bigint default 0 not null
) WITHOUT ROWID;"#,
        [],
    )?;

    conn.execute(
        r#"create table object_store
(
    id             integer               not null
        primary key autoincrement,
    name           varchar               not null
        unique,
    key_path       varbinary_blob,
    auto_increment boolean default FALSE not null
);"#,
        [],
    )?;

    conn.execute(
        r#"create table object_data
(
    object_store_id integer not null
        references object_store,
    key             blob    not null,
    data            blob    not null,
    constraint "pk-object_data"
        primary key (object_store_id, key)
) WITHOUT ROWID;"#,
        [],
    )?;

    conn.execute(
        r#"create table object_store_index
            (
                id                integer        not null
                primary key autoincrement,
                object_store_id   integer        not null
                references object_store,
                name              varchar        not null
                unique,
                key_path          varbinary_blob not null,
                unique_index      boolean        not null,
                multi_entry_index boolean        not null
            );"#,
        [],
    )?;

    conn.execute(
        r#"CREATE TABLE index_data
        (
            index_id INTEGER NOT NULL,
            value BLOB NOT NULL,
            object_data_key BLOB NOT NULL,
            object_store_id INTEGER NOT NULL,
            value_locale BLOB,
            PRIMARY KEY (index_id, value, object_data_key)
            FOREIGN KEY (index_id) REFERENCES object_store_index(id),
            FOREIGN KEY (object_store_id, object_data_key)
            REFERENCES object_data(object_store_id, key)
        ) WITHOUT ROWID;"#,
        [],
    )?;

    conn.execute(
        r#"CREATE TABLE unique_index_data
        (
            index_id INTEGER NOT NULL,
            value BLOB NOT NULL,
            object_store_id INTEGER NOT NULL,
            object_data_key BLOB NOT NULL,
            value_locale BLOB,
            PRIMARY KEY (index_id, value),
            FOREIGN KEY (index_id) REFERENCES object_store_index(id),
            FOREIGN KEY (object_store_id, object_data_key)
            REFERENCES object_data(object_store_id, key)
        ) WITHOUT ROWID;"#,
        [],
    )?;
    Ok(())
}
