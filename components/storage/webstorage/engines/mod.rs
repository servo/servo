/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::webstorage::OriginEntry;

pub mod sqlite;

pub trait WebStorageEngine {
    type Error;
    fn load(&self) -> Result<OriginEntry, Self::Error>;
    fn clear(&mut self) -> Result<(), Self::Error>;
    fn delete(&mut self, key: &str) -> Result<(), Self::Error>;
    fn set(&mut self, key: &str, value: &str) -> Result<(), Self::Error>;
    fn save(&mut self, data: &OriginEntry);
}
