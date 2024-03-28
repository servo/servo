/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod dom_manipulation;
pub mod file_reading;
pub mod gamepad;
pub mod history_traversal;
pub mod media_element;
pub mod networking;
pub mod performance_timeline;
pub mod port_message;
pub mod remote_event;
pub mod rendering;
pub mod timer;
pub mod user_interaction;
pub mod websocket;

use std::result::Result;

use enum_iterator::IntoEnumIterator;

use crate::dom::globalscope::GlobalScope;
use crate::task::{TaskCanceller, TaskOnce};

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
    PortMessage,
    UserInteraction,
    RemoteEvent,
    /// <https://html.spec.whatwg.org/multipage/#rendering-task-source>
    Rendering,
    MediaElement,
    Websocket,
    Timer,
    /// <https://www.w3.org/TR/gamepad/#dfn-gamepad-task-source>
    Gamepad,
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
