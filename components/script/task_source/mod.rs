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

// The names of all task sources, used to differentiate TaskCancellers.
// Note: When adding a task source, update this enum.
// Note: The HistoryTraversalTaskSource is not part of this,
// because it doesn't implement TaskSource.
#[derive(Eq, Hash, JSTraceable, PartialEq)]
pub enum TaskSourceName {
    DOMManipulation,
    FileReading,
    HistoryTraversal,
    Networking,
    PerformanceTimeline,
    UserInteraction
}

impl TaskSourceName {
    // Retuns a vec of variants of TaskSourceName.
    // Note: When adding a variant, update the vec.
    pub fn all() -> Vec<TaskSourceName> {
        vec![
            TaskSourceName::DOMManipulation,
            TaskSourceName::FileReading,
            TaskSourceName::HistoryTraversal,
            TaskSourceName::Networking,
            TaskSourceName::PerformanceTimeline,
            TaskSourceName::UserInteraction
        ]
    }
}

pub trait TaskSource {
    fn queue_with_canceller<T>(
        &self,
        task: T,
        canceller: &TaskCanceller,
    ) -> Result<(), ()>
    where
        T: TaskOnce + 'static;

    fn choose_canceller(&self, global: &GlobalScope) -> TaskCanceller;

    fn queue<T>(&self, task: T, global: &GlobalScope) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let canceller = self.choose_canceller(global);
        self.queue_with_canceller(task, &canceller)
    }
}
