/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "database")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub name: String,
    pub origin: String,
    #[sea_orm(default_value = 0)]
    pub version: i32,
    // TODO: Hold timestamp for vacuuming
    // TODO: implement vacuuming
}

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
