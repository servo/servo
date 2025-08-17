/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */
use sea_query::Iden;

#[derive(Iden)]
#[expect(unused)]
pub enum Column {
    #[iden = "database"]
    Table,
    Name,
    Origin,
    Version,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Model {
    pub name: String,
    pub origin: String,
    pub version: i64,
    // TODO: Hold timestamp for vacuuming
    // TODO: implement vacuuming
}
