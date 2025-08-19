/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use sea_query::Iden;

#[derive(Iden)]
#[expect(unused)]
pub enum Column {
    Table,
    ObjectStoreId,
    Name,
    KeyPath,
    UniqueIndex,
    MultiEntryIndex,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub id: i32,
    pub object_store_id: i32,
    pub name: String,
    pub key_path: Vec<u8>,
    pub unique_index: bool,
    pub multi_entry_index: bool,
}
