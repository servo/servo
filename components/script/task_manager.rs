/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DomRefCell;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use task::TaskCanceller;
use task_source::TaskSourceName;
use task_source::dom_manipulation::DOMManipulationTaskSource;
use task_source::file_reading::FileReadingTaskSource;
use task_source::history_traversal::HistoryTraversalTaskSource;
use task_source::networking::NetworkingTaskSource;
use task_source::performance_timeline::PerformanceTimelineTaskSource;
use task_source::remote_event::RemoteEventTaskSource;
use task_source::user_interaction::UserInteractionTaskSource;
use task_source::websocket::WebsocketTaskSource;

#[derive(JSTraceable)]
pub struct TaskManager {
    task_cancellers: DomRefCell<HashMap<TaskSourceName, Arc<AtomicBool>>>,
    dom_manipulation_task_source: DOMManipulationTaskSource,
    file_reading_task_source: FileReadingTaskSource,
    history_traversal_task_source: HistoryTraversalTaskSource,
    networking_task_source: NetworkingTaskSource,
    performance_timeline_task_source: PerformanceTimelineTaskSource,
    user_interaction_task_source: UserInteractionTaskSource,
    remote_event_task_source: RemoteEventTaskSource,
    websocket_task_source: WebsocketTaskSource,
}

impl TaskManager {
    pub fn new(dom_manipulation_task_source: DOMManipulationTaskSource,
           file_reading_task_source: FileReadingTaskSource,
           history_traversal_task_source: HistoryTraversalTaskSource,
           networking_task_source: NetworkingTaskSource,
           performance_timeline_task_source: PerformanceTimelineTaskSource,
           user_interaction_task_source: UserInteractionTaskSource,
           remote_event_task_source: RemoteEventTaskSource,
           websocket_task_source: WebsocketTaskSource) -> Self {
        TaskManager {
            dom_manipulation_task_source,
            file_reading_task_source,
            history_traversal_task_source,
            networking_task_source,
            performance_timeline_task_source,
            user_interaction_task_source,
            remote_event_task_source,
            websocket_task_source,
            task_cancellers: Default::default(),
        }
    }

    pub fn dom_manipulation_task_source_with_canceller(&self) -> (DOMManipulationTaskSource, TaskCanceller) {
        (self.dom_manipulation_task_source.clone(), self.task_canceller(TaskSourceName::DOMManipulation))
    }

    pub fn user_interaction_task_source_with_canceller(&self) -> (UserInteractionTaskSource, TaskCanceller) {
        (self.user_interaction_task_source.clone(), self.task_canceller(TaskSourceName::UserInteraction))
    }

    pub fn networking_task_source_with_canceller(&self) -> (NetworkingTaskSource, TaskCanceller) {
        (self.networking_task_source.clone(), self.task_canceller(TaskSourceName::Networking))
    }

    pub fn file_reading_task_source_with_canceller(&self) -> (FileReadingTaskSource, TaskCanceller) {
        (self.file_reading_task_source.clone(), self.task_canceller(TaskSourceName::FileReading))
    }

    pub fn history_traversal_task_source_with_canceller(&self) -> (HistoryTraversalTaskSource, TaskCanceller) {
        (self.history_traversal_task_source.clone(), self.task_canceller(TaskSourceName::HistoryTraversal))
    }

    pub fn performance_timeline_task_source_with_canceller(&self) -> (PerformanceTimelineTaskSource, TaskCanceller) {
        (self.performance_timeline_task_source.clone(), self.task_canceller(TaskSourceName::PerformanceTimeline))
    }

    pub fn remote_event_task_source_with_canceller(&self) -> (RemoteEventTaskSource, TaskCanceller) {
        (self.remote_event_task_source.clone(), self.task_canceller(TaskSourceName::RemoteEvent))
    }

    pub fn websocket_task_source_with_canceller(&self) -> (WebsocketTaskSource, TaskCanceller) {
        (self.websocket_task_source.clone(), self.task_canceller(TaskSourceName::Websocket))
    }

    pub fn dom_manipulation_task_source(&self) -> DOMManipulationTaskSource {
        self.dom_manipulation_task_source.clone()
    }

    pub fn user_interaction_task_source(&self) -> UserInteractionTaskSource {
        self.user_interaction_task_source.clone()
    }

    pub fn networking_task_source(&self) -> NetworkingTaskSource {
        self.networking_task_source.clone()
    }

    pub fn file_reading_task_source(&self) -> FileReadingTaskSource {
        self.file_reading_task_source.clone()
    }

    pub fn history_traversal_task_source(&self) -> HistoryTraversalTaskSource {
        self.history_traversal_task_source.clone()
    }

    pub fn performance_timeline_task_source(&self) -> PerformanceTimelineTaskSource {
        self.performance_timeline_task_source.clone()
    }

    pub fn remote_event_task_source(&self) -> RemoteEventTaskSource {
        self.remote_event_task_source.clone()
    }

    pub fn websocket_task_source(&self) -> WebsocketTaskSource {
        self.websocket_task_source.clone()
    }

    pub fn task_canceller(&self, name: TaskSourceName) -> TaskCanceller {
        let mut flags = self.task_cancellers.borrow_mut();
        let cancel_flag = flags.entry(name).or_insert(Default::default());
        TaskCanceller {
            cancelled: Some(cancel_flag.clone()),
        }
    }
}
