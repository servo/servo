/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "object_store")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,
    #[sea_orm(unique)]
    pub name: String,
    #[sea_orm(nullable)]
    pub key_path: Option<Vec<u8>>,
    #[sea_orm(default_value = false)]
    pub auto_increment: bool,
}

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(has_many = "super::object_data_model::Entity")]
    Data,
    #[sea_orm(has_many = "super::object_store_index_model::Entity")]
    Index,
}

impl Related<super::object_data_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Data.def()
    }
}

impl Related<super::object_store_index_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Index.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
