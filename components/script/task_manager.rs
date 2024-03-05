/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::dom::bindings::cell::DomRefCell;
use crate::task::TaskCanceller;
use crate::task_source::dom_manipulation::DOMManipulationTaskSource;
use crate::task_source::file_reading::FileReadingTaskSource;
use crate::task_source::gamepad::GamepadTaskSource;
use crate::task_source::history_traversal::HistoryTraversalTaskSource;
use crate::task_source::media_element::MediaElementTaskSource;
use crate::task_source::networking::NetworkingTaskSource;
use crate::task_source::performance_timeline::PerformanceTimelineTaskSource;
use crate::task_source::port_message::PortMessageQueue;
use crate::task_source::remote_event::RemoteEventTaskSource;
use crate::task_source::rendering::RenderingTaskSource;
use crate::task_source::timer::TimerTaskSource;
use crate::task_source::user_interaction::UserInteractionTaskSource;
use crate::task_source::websocket::WebsocketTaskSource;
use crate::task_source::TaskSourceName;

macro_rules! task_source_functions {
    ($self:ident,$with_canceller:ident,$task_source:ident,$task_source_type:ident,$task_source_name:ident) => {
        pub fn $with_canceller(&$self) -> ($task_source_type, TaskCanceller) {
            ($self.$task_source.clone(), $self.task_canceller(TaskSourceName::$task_source_name))
        }

        pub fn $task_source(&$self) -> $task_source_type {
            $self.$task_source.clone()
        }
    }
}

#[derive(JSTraceable, MallocSizeOf)]
pub struct TaskManager {
    #[ignore_malloc_size_of = "task sources are hard"]
    pub task_cancellers: DomRefCell<HashMap<TaskSourceName, Arc<AtomicBool>>>,
    #[ignore_malloc_size_of = "task sources are hard"]
    dom_manipulation_task_source: DOMManipulationTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    file_reading_task_source: FileReadingTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    gamepad_task_source: GamepadTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    history_traversal_task_source: HistoryTraversalTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    media_element_task_source: MediaElementTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    networking_task_source: NetworkingTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    performance_timeline_task_source: PerformanceTimelineTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    port_message_queue: PortMessageQueue,
    #[ignore_malloc_size_of = "task sources are hard"]
    user_interaction_task_source: UserInteractionTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    remote_event_task_source: RemoteEventTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    rendering_task_source: RenderingTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    timer_task_source: TimerTaskSource,
    #[ignore_malloc_size_of = "task sources are hard"]
    websocket_task_source: WebsocketTaskSource,
}

impl TaskManager {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dom_manipulation_task_source: DOMManipulationTaskSource,
        file_reading_task_source: FileReadingTaskSource,
        gamepad_task_source: GamepadTaskSource,
        history_traversal_task_source: HistoryTraversalTaskSource,
        media_element_task_source: MediaElementTaskSource,
        networking_task_source: NetworkingTaskSource,
        performance_timeline_task_source: PerformanceTimelineTaskSource,
        port_message_queue: PortMessageQueue,
        user_interaction_task_source: UserInteractionTaskSource,
        remote_event_task_source: RemoteEventTaskSource,
        rendering_task_source: RenderingTaskSource,
        timer_task_source: TimerTaskSource,
        websocket_task_source: WebsocketTaskSource,
    ) -> Self {
        TaskManager {
            dom_manipulation_task_source,
            file_reading_task_source,
            gamepad_task_source,
            history_traversal_task_source,
            media_element_task_source,
            networking_task_source,
            performance_timeline_task_source,
            port_message_queue,
            user_interaction_task_source,
            remote_event_task_source,
            rendering_task_source,
            timer_task_source,
            websocket_task_source,
            task_cancellers: Default::default(),
        }
    }

    task_source_functions!(
        self,
        dom_manipulation_task_source_with_canceller,
        dom_manipulation_task_source,
        DOMManipulationTaskSource,
        DOMManipulation
    );

    task_source_functions!(
        self,
        gamepad_task_source_with_canceller,
        gamepad_task_source,
        GamepadTaskSource,
        Gamepad
    );

    task_source_functions!(
        self,
        media_element_task_source_with_canceller,
        media_element_task_source,
        MediaElementTaskSource,
        MediaElement
    );

    task_source_functions!(
        self,
        user_interaction_task_source_with_canceller,
        user_interaction_task_source,
        UserInteractionTaskSource,
        UserInteraction
    );

    task_source_functions!(
        self,
        networking_task_source_with_canceller,
        networking_task_source,
        NetworkingTaskSource,
        Networking
    );

    task_source_functions!(
        self,
        file_reading_task_source_with_canceller,
        file_reading_task_source,
        FileReadingTaskSource,
        FileReading
    );

    task_source_functions!(
        self,
        history_traversal_task_source_with_canceller,
        history_traversal_task_source,
        HistoryTraversalTaskSource,
        HistoryTraversal
    );

    task_source_functions!(
        self,
        performance_timeline_task_source_with_canceller,
        performance_timeline_task_source,
        PerformanceTimelineTaskSource,
        PerformanceTimeline
    );

    task_source_functions!(
        self,
        port_message_queue_with_canceller,
        port_message_queue,
        PortMessageQueue,
        PortMessage
    );

    task_source_functions!(
        self,
        remote_event_task_source_with_canceller,
        remote_event_task_source,
        RemoteEventTaskSource,
        RemoteEvent
    );

    task_source_functions!(
        self,
        rendering_task_source_with_canceller,
        rendering_task_source,
        RenderingTaskSource,
        Rendering
    );

    task_source_functions!(
        self,
        timer_task_source_with_canceller,
        timer_task_source,
        TimerTaskSource,
        Timer
    );

    task_source_functions!(
        self,
        websocket_task_source_with_canceller,
        websocket_task_source,
        WebsocketTaskSource,
        Websocket
    );

    pub fn task_canceller(&self, name: TaskSourceName) -> TaskCanceller {
        let mut flags = self.task_cancellers.borrow_mut();
        let cancel_flag = flags.entry(name).or_default();
        TaskCanceller {
            cancelled: cancel_flag.clone(),
        }
    }
}
