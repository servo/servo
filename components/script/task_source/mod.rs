/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod dom_manipulation;
pub mod file_reading;
pub mod history_traversal;
pub mod networking;
pub mod user_interaction;

use std::result::Result;

pub trait TaskSource<T> {
    fn queue(&self, msg: T) -> Result<(), ()>;
    fn clone(&self) -> Box<TaskSource<T> + Send>;
}
