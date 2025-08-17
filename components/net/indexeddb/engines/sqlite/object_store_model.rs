/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use rusqlite::Row;
use sea_query::Iden;

#[derive(Iden)]
#[expect(unused)]
pub enum Column {
    #[iden = "object_store"]
    Table,
    Id,
    Name,
    KeyPath,
    AutoIncrement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub id: i32,
    pub name: String,
    pub key_path: Option<Vec<u8>>,
    pub auto_increment: bool,
}

impl TryFrom<&Row<'_>> for Model {
    type Error = rusqlite::Error;

    fn try_from(value: &Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: value.get(0)?,
            name: value.get(1)?,
            key_path: value.get(2)?,
            auto_increment: value.get(3)?,
        })
    }
}
