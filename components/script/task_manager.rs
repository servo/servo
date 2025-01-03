/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use base::id::PipelineId;

use crate::dom::bindings::cell::DomRefCell;
use crate::script_runtime::ScriptChan;
use crate::task::TaskCanceller;
use crate::task_source::{TaskSource, TaskSourceName};

macro_rules! task_source_functions {
    ($self:ident, $task_source:ident) => {
        pub(crate) fn $task_source(&$self) -> TaskSource {
            $self.$task_source.clone()
        }
    };
    ($self:ident, $with_canceller:ident, $task_source:ident) => {
        pub(crate) fn $with_canceller(&$self) -> (TaskSource, TaskCanceller) {
            ($self.$task_source.clone(), $self.task_canceller($self.$task_source.name))
        }

        pub(crate) fn $task_source(&$self) -> TaskSource {
            $self.$task_source.clone()
        }
    };
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct TaskManager {
    #[ignore_malloc_size_of = "task sources are hard"]
    pub task_cancellers: DomRefCell<HashMap<TaskSourceName, Arc<AtomicBool>>>,
    dom_manipulation_task_source: TaskSource,
    file_reading_task_source: TaskSource,
    gamepad_task_source: TaskSource,
    history_traversal_task_source: TaskSource,
    media_element_task_source: TaskSource,
    networking_task_source: TaskSource,
    performance_timeline_task_source: TaskSource,
    port_message_queue: TaskSource,
    user_interaction_task_source: TaskSource,
    remote_event_task_source: TaskSource,
    rendering_task_source: TaskSource,
    timer_task_source: TaskSource,
    websocket_task_source: TaskSource,
}

impl TaskManager {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(sender: Box<dyn ScriptChan + Send>, pipeline_id: PipelineId) -> Self {
        let task_source = |name| TaskSource {
            sender: sender.as_boxed(),
            pipeline_id,
            name,
        };

        TaskManager {
            dom_manipulation_task_source: task_source(TaskSourceName::DOMManipulation),
            file_reading_task_source: task_source(TaskSourceName::FileReading),
            gamepad_task_source: task_source(TaskSourceName::Gamepad),
            history_traversal_task_source: task_source(TaskSourceName::HistoryTraversal),
            media_element_task_source: task_source(TaskSourceName::MediaElement),
            networking_task_source: task_source(TaskSourceName::Networking),
            performance_timeline_task_source: task_source(TaskSourceName::PerformanceTimeline),
            port_message_queue: task_source(TaskSourceName::PortMessage),
            user_interaction_task_source: task_source(TaskSourceName::UserInteraction),
            remote_event_task_source: task_source(TaskSourceName::RemoteEvent),
            rendering_task_source: task_source(TaskSourceName::Rendering),
            timer_task_source: task_source(TaskSourceName::Timer),
            websocket_task_source: task_source(TaskSourceName::WebSocket),
            task_cancellers: Default::default(),
        }
    }

    task_source_functions!(
        self,
        dom_manipulation_task_source_with_canceller,
        dom_manipulation_task_source
    );
    task_source_functions!(self, gamepad_task_source);
    task_source_functions!(
        self,
        media_element_task_source_with_canceller,
        media_element_task_source
    );
    task_source_functions!(self, user_interaction_task_source);
    task_source_functions!(
        self,
        networking_task_source_with_canceller,
        networking_task_source
    );
    task_source_functions!(self, file_reading_task_source);
    task_source_functions!(self, performance_timeline_task_source);
    task_source_functions!(self, port_message_queue);
    task_source_functions!(self, remote_event_task_source);
    task_source_functions!(self, rendering_task_source);
    task_source_functions!(self, timer_task_source);
    task_source_functions!(self, websocket_task_source);

    pub fn task_canceller(&self, name: TaskSourceName) -> TaskCanceller {
        let mut flags = self.task_cancellers.borrow_mut();
        let cancel_flag = flags.entry(name).or_default();
        TaskCanceller {
            cancelled: cancel_flag.clone(),
        }
    }
}
