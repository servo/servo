/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod dom_manipulation;
pub mod file_reading;
pub mod history_traversal;
pub mod networking;
pub mod performance_timeline;
pub mod user_interaction;

use dom::globalscope::GlobalScope;
use std::result::Result;
use task::{Task, TaskCanceller};

pub trait TaskSource {
    fn queue_with_canceller<T>(
        &self,
        msg: Box<T>,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
    where
        T: Send + Task + 'static;

    fn queue<T: Task + Send + 'static>(&self, msg: Box<T>, global: &GlobalScope) -> Result<(), ()> {
        self.queue_with_canceller(msg, &global.task_canceller())
    }
}
