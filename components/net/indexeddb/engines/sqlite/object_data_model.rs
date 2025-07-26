/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "object_data")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub object_store_id: i32,
    #[sea_orm(primary_key, column_type = "Blob")]
    pub key: Vec<u8>,
    #[sea_orm(column_type = "Blob")]
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, DeriveRelation, EnumIter)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::object_store_model::Entity",
        from = "Column::ObjectStoreId",
        to = "super::object_store_model::Column::Id"
    )]
    Store,
}

impl Related<super::object_store_model::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Store.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
