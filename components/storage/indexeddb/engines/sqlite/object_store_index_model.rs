/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use rusqlite::Row;
use sea_query::Iden;

#[derive(Iden)]
#[expect(unused)]
pub enum Column {
    #[iden = "object_store_index"]
    Table,
    ObjectStoreId,
    Name,
    KeyPath,
    UniqueIndex,
    MultiEntryIndex,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub id: i32,
    pub object_store_id: i32,
    pub name: String,
    pub key_path: Vec<u8>,
    pub unique_index: bool,
    pub multi_entry_index: bool,
}

impl TryFrom<&Row<'_>> for Model {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.get(0)?,
            object_store_id: value.get(1)?,
            name: value.get(2)?,
            key_path: value.get(3)?,
            unique_index: value.get(4)?,
            multi_entry_index: value.get(5)?,
        })
    }
}
