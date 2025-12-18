/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use storage_traits::indexeddb::BackendError;

use crate::dom::bindings::error::Error;

pub(crate) mod idbcursor;
pub(crate) mod idbcursorwithvalue;
pub(crate) mod idbdatabase;
pub(crate) mod idbfactory;
pub(crate) mod idbindex;
pub(crate) mod idbkeyrange;
pub(crate) mod idbobjectstore;
pub(crate) mod idbopendbrequest;
pub(crate) mod idbrequest;
pub(crate) mod idbtransaction;
pub(crate) mod idbversionchangeevent;
