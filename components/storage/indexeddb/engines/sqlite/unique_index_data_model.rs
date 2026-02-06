/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use sea_query::Iden;

#[derive(Clone, Copy, Iden)]
#[expect(unused)]
pub enum Column {
    #[iden = "unique_index_data"]
    Table,
    IndexId,
    Value,
    ObjectDataKey,
    ObjectStoreId,
    ValueLocale,
}
