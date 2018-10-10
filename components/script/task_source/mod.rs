/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod dom_manipulation;
pub mod file_reading;
pub mod history_traversal;
pub mod media_element;
pub mod networking;
pub mod performance_timeline;
pub mod remote_event;
pub mod user_interaction;
pub mod websocket;

use dom::globalscope::GlobalScope;
use enum_iterator::IntoEnumIterator;
use std::result::Result;
use task::{TaskCanceller, TaskOnce};

// The names of all task sources, used to differentiate TaskCancellers.
// Note: When adding a task source, update this enum.
// Note: The HistoryTraversalTaskSource is not part of this,
// because it doesn't implement TaskSource.
#[derive(Clone, Eq, Hash, IntoEnumIterator, JSTraceable, PartialEq)]
pub enum TaskSourceName {
    DOMManipulation,
    FileReading,
    HistoryTraversal,
    Networking,
    PerformanceTimeline,
    UserInteraction,
    RemoteEvent,
    MediaElement,
    Websocket,
}

impl TaskSourceName {
    pub fn all() -> Vec<TaskSourceName> {
        TaskSourceName::into_enum_iter().collect()
    }
}

pub trait TaskSource {
    const NAME: TaskSourceName;

    fn queue_with_canceller<T>(&self, task: T, canceller: &TaskCanceller) -> Result<(), ()>
    where
        T: TaskOnce + 'static;

    fn queue<T>(&self, task: T, global: &GlobalScope) -> Result<(), ()>
    where
        T: TaskOnce + 'static,
    {
        let canceller = global.task_canceller(Self::NAME);
        self.queue_with_canceller(task, &canceller)
    }
}
