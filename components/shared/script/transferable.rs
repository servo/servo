/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains implementations in script that are transferable.
//! The implementations are here instead of in script
//! so that the other modules involved in the transfer don't have
//! to depend on script.

use std::collections::VecDeque;

use base::id::MessagePortId;
use malloc_size_of_derive::MallocSizeOf;
use serde::{Deserialize, Serialize};

use crate::PortMessageTask;

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
enum MessagePortState {
    /// <https://html.spec.whatwg.org/multipage/#detached>
    Detached,
    /// <https://html.spec.whatwg.org/multipage/#port-message-queue>
    /// The message-queue of this port is enabled,
    /// the boolean represents awaiting completion of a transfer.
    Enabled(bool),
    /// <https://html.spec.whatwg.org/multipage/#port-message-queue>
    /// The message-queue of this port is disabled,
    /// the boolean represents awaiting completion of a transfer.
    Disabled(bool),
}

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
/// The data and logic backing the DOM managed MessagePort.
pub struct MessagePortImpl {
    /// The current state of the port.
    state: MessagePortState,

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    entangled_port: Option<MessagePortId>,

    /// <https://html.spec.whatwg.org/multipage/#port-message-queue>
    message_buffer: Option<VecDeque<PortMessageTask>>,

    /// The UUID of this port.
    message_port_id: MessagePortId,
}

impl MessagePortImpl {
    /// Create a new messageport impl.
    pub fn new(port_id: MessagePortId) -> MessagePortImpl {
        MessagePortImpl {
            state: MessagePortState::Disabled(false),
            entangled_port: None,
            message_buffer: None,
            message_port_id: port_id,
        }
    }

    /// Get the Id.
    pub fn message_port_id(&self) -> &MessagePortId {
        &self.message_port_id
    }

    /// Maybe get the Id of the entangled port.
    pub fn entangled_port_id(&self) -> Option<MessagePortId> {
        self.entangled_port
    }

    /// Entanged this port with another.
    pub fn entangle(&mut self, other_id: MessagePortId) {
        self.entangled_port = Some(other_id);
    }

    /// Is this port enabled?
    pub fn enabled(&self) -> bool {
        matches!(self.state, MessagePortState::Enabled(_))
    }

    /// Mark this port as having been shipped.
    /// <https://html.spec.whatwg.org/multipage/#has-been-shipped>
    pub fn set_has_been_shipped(&mut self) {
        match self.state {
            MessagePortState::Detached => {
                panic!("Messageport set_has_been_shipped called in detached state")
            },
            MessagePortState::Enabled(_) => self.state = MessagePortState::Enabled(true),
            MessagePortState::Disabled(_) => self.state = MessagePortState::Disabled(true),
        }
    }

    /// Handle the completion of the transfer,
    /// this is data received from the constellation.
    pub fn complete_transfer(&mut self, mut tasks: VecDeque<PortMessageTask>) {
        match self.state {
            MessagePortState::Detached => return,
            MessagePortState::Enabled(_) => self.state = MessagePortState::Enabled(false),
            MessagePortState::Disabled(_) => self.state = MessagePortState::Disabled(false),
        }

        // Note: these are the tasks that were buffered while the transfer was ongoing,
        // hence they need to execute first.
        // The global will call `start` if we are enabled,
        // which will add tasks on the event-loop to dispatch incoming messages.
        match self.message_buffer {
            Some(ref mut incoming_buffer) => {
                while let Some(task) = tasks.pop_back() {
                    incoming_buffer.push_front(task);
                }
            },
            None => self.message_buffer = Some(tasks),
        }
    }

    /// A message was received from our entangled port,
    /// returns an optional task to be dispatched.
    pub fn handle_incoming(&mut self, task: PortMessageTask) -> Option<PortMessageTask> {
        let should_dispatch = match self.state {
            MessagePortState::Detached => return None,
            MessagePortState::Enabled(in_transfer) => !in_transfer,
            MessagePortState::Disabled(_) => false,
        };

        if should_dispatch {
            Some(task)
        } else {
            match self.message_buffer {
                Some(ref mut buffer) => {
                    buffer.push_back(task);
                },
                None => {
                    let mut queue = VecDeque::new();
                    queue.push_back(task);
                    self.message_buffer = Some(queue);
                },
            }
            None
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    /// returns an optional queue of tasks that were buffered while the port was disabled.
    pub fn start(&mut self) -> Option<VecDeque<PortMessageTask>> {
        match self.state {
            MessagePortState::Detached => return None,
            MessagePortState::Enabled(_) => {},
            MessagePortState::Disabled(in_transfer) => {
                self.state = MessagePortState::Enabled(in_transfer);
            },
        }
        if let MessagePortState::Enabled(true) = self.state {
            return None;
        }
        self.message_buffer.take()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    pub fn close(&mut self) {
        // Step 1
        self.state = MessagePortState::Detached;
    }
}
