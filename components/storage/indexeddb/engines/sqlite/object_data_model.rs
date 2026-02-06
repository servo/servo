/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use rusqlite::Row;
use sea_query::Iden;

#[derive(Clone, Copy, Iden)]
pub enum Column {
    #[iden = "object_data"]
    Table,
    ObjectStoreId,
    Key,
    Data,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub object_store_id: i32,
    pub key: Vec<u8>,
    pub data: Vec<u8>,
}

impl TryFrom<&Row<'_>> for Model {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            object_store_id: value.get(0)?,
            key: value.get(1)?,
            data: value.get(2)?,
        })
    }
}
