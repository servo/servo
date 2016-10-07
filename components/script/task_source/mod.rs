/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod dom_manipulation;
pub mod file_reading;
pub mod history_traversal;
pub mod networking;
pub mod user_interaction;

use dom::globalscope::GlobalScope;
use script_thread::{Runnable, RunnableWrapper};
use std::result::Result;

pub trait TaskSource {
    fn queue_with_wrapper<T>(&self,
                             msg: Box<T>,
                             wrapper: &RunnableWrapper)
                             -> Result<(), ()>
                             where T: Runnable + Send + 'static;
    fn queue<T: Runnable + Send + 'static>(&self, msg: Box<T>, global: &GlobalScope) -> Result<(), ()> {
        self.queue_with_wrapper(msg, &global.get_runnable_wrapper())
    }
}
