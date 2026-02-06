/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use rusqlite::Row;
use sea_query::Iden;

#[derive(Clone, Copy, Iden)]
pub enum Column {
    #[iden = "index_data"]
    Table,
    IndexId,
    Value,
    ObjectDataKey,
    ObjectStoreId,
    ValueLocale,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub index_id: i64,
    pub value: Vec<u8>,
    pub object_data_key: Vec<u8>,
    pub object_store_id: i64,
    pub value_locale: Vec<u8>,
}

impl TryFrom<&Row<'_>> for Model {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            index_id: value.get(0)?,
            value: value.get(1)?,
            object_data_key: value.get(2)?,
            object_store_id: value.get(3)?,
            value_locale: value.get(4)?,
        })
    }
}
