/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use rusqlite::Row;
use sea_query::Iden;

#[derive(Copy, Clone, Iden)]
pub enum Column {
    #[iden = "index_data"]
    Table,
    IndexId,   // References object_store_index.id
    IndexKey,  // The value extracted from the object (serialized)
    ObjectKey, // The primary key of the object in object_data
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub index_id: i32,
    pub index_key: Vec<u8>,
    pub object_key: Vec<u8>,
}

impl TryFrom<&Row<'_>> for Model {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            index_id: value.get(0)?,
            index_key: value.get(1)?,
            object_key: value.get(2)?,
        })
    }
}
