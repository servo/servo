/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

// Adapted from:
// https://github.com/mozilla-firefox/firefox/blob/ee102e926521b3e460293b0aea6b54b1a03f6f74/dom/indexedDB/DBSchema.cpp#L78

pub(crate) fn create_tables(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    const DATABASE: &str = r#"
create table database (
    name    varchar          not null
        primary key,
    origin  varchar          not null,
    version bigint default 0 not null
) WITHOUT ROWID;"#;
    conn.execute(DATABASE, [])?;

    const OBJECT_STORE: &str = r#"
create table object_store (
    id             integer               not null
        primary key autoincrement,
    name           varchar               not null
        unique,
    key_path       varbinary_blob,
    auto_increment integer default FALSE not null
);"#;
    conn.execute(OBJECT_STORE, [])?;

    const OBJECT_DATA: &str = r#"
create table object_data (
    object_store_id integer not null
        references object_store,
    key             blob    not null,
    data            blob    not null,
    constraint "pk-object_data"
        primary key (object_store_id, key)
) WITHOUT ROWID;"#;
    conn.execute(OBJECT_DATA, [])?;

    const OBJECT_STORE_INDEX: &str = r#"
create table object_store_index (
    id                integer        not null
    primary key autoincrement,
    object_store_id   integer        not null
    references object_store,
    name              varchar        not null,
    key_path          varbinary_blob not null,
    unique_index      boolean        not null,
    multi_entry_index boolean        not null,
    constraint "uq-object_store_index-name"
        unique (object_store_id, name)
);"#;
    conn.execute(OBJECT_STORE_INDEX, [])?;

    const INDEX_DATA: &str = r#"
create table index_data (
    index_id        integer not null
        references object_store_index,
    index_key       blob    not null,
    object_key      blob    not null,
    constraint "pk-index_data"
        primary key (index_id, index_key, object_key)
) WITHOUT ROWID;"#;
    conn.execute(INDEX_DATA, [])?;

    Ok(())
}
