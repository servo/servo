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
use task::{TaskCanceller, TaskOnce};

pub trait TaskSource {
    fn queue_with_canceller<T>(
        &self,
        task: T,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
    where
        T: TaskOnce + 'static;

    fn queue<T>(&self, task: T, global: &GlobalScope) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        self.queue_with_canceller(task, &global.task_canceller())
    }
}
