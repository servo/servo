/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! This module contains implementations in script that are transferable.
//! The implementations are here instead of in script
//! so that the other modules involved in the transfer don't have
//! to depend on script.

use crate::PortMessageTask;
use msg::constellation_msg::MessagePortId;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

#[derive(Debug, Deserialize, MallocSizeOf, Serialize)]
/// The data and logic backing the DOM managed MessagePort.
pub struct MessagePortImpl {
    /// <https://html.spec.whatwg.org/multipage/#detached>
    detached: Cell<bool>,

    /// Is the message-queue of this port enabled?
    enabled: Cell<bool>,

    /// Is this port awaiting completiong of its transfer by the constellation?
    awaiting_transfer: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#entangle>
    entangled_port: RefCell<Option<MessagePortId>>,

    /// <https://html.spec.whatwg.org/multipage/#port-message-queue>
    message_buffer: RefCell<VecDeque<PortMessageTask>>,

    /// <https://html.spec.whatwg.org/multipage/#has-been-shipped>
    has_been_shipped: Cell<bool>,

    /// The UUID of this port.
    message_port_id: MessagePortId,
}

impl MessagePortImpl {
    /// Create a new messageport impl.
    pub fn new(port_id: MessagePortId) -> MessagePortImpl {
        MessagePortImpl {
            detached: Cell::new(false),
            enabled: Cell::new(false),
            awaiting_transfer: Cell::new(false),
            entangled_port: RefCell::new(None),
            message_buffer: RefCell::new(VecDeque::new()),
            has_been_shipped: Cell::new(false),
            message_port_id: port_id,
        }
    }

    /// Get the Id.
    pub fn message_port_id(&self) -> &MessagePortId {
        &self.message_port_id
    }

    /// Maybe get the Id of the entangled port.
    pub fn entangled_port_id(&self) -> Option<MessagePortId> {
        self.entangled_port.borrow().clone()
    }

    /// Entanged this port with another.
    pub fn entangle(&self, other_id: MessagePortId) {
        *self.entangled_port.borrow_mut() = Some(other_id);
    }

    /// Is this port enabled?
    pub fn enabled(&self) -> bool {
        self.enabled.get()
    }

    /// Mark this port as having been shipped.
    /// <https://html.spec.whatwg.org/multipage/#has-been-shipped>
    pub fn set_has_been_shipped(&self) {
        self.has_been_shipped.set(true);
        self.awaiting_transfer.set(true);
    }

    /// Handle the completion of the transfer,
    /// this is data received from the constellation.
    pub fn complete_transfer(&self, tasks: Option<VecDeque<PortMessageTask>>) {
        if self.detached.get() {
            return;
        }
        self.awaiting_transfer.set(false);

        if let Some(mut tasks) = tasks {
            // Note: these are the tasks that were buffered while the transfer was ongoing,
            // hence they need to execute first.
            // The global will call `start` if we are enabled,
            // which will add tasks on the event-loop to dispatch incoming messages.
            let mut incoming_buffer = self.message_buffer.borrow_mut();
            while let Some(task) = tasks.pop_back() {
                incoming_buffer.push_front(task);
            }
        }
    }

    /// A message was received from our entangled port.
    pub fn handle_incoming(&self, task: PortMessageTask) -> Option<PortMessageTask> {
        println!("Handling incoming task for {:?}", self.message_port_id);
        if self.detached.get() {
            return None;
        }

        if self.enabled.get() && !self.awaiting_transfer.get() {
            println!("Dispatgchin incoming task for {:?}", self.message_port_id);
            Some(task)
        } else {
            println!("Buffering incoming task for {:?}", self.message_port_id);
            self.message_buffer.borrow_mut().push_back(task);
            None
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-start>
    pub fn start(&self) -> Option<VecDeque<PortMessageTask>> {
        self.enabled.set(true);
        if self.awaiting_transfer.get() {
            return None;
        }
        Some(self.message_buffer.borrow_mut().drain(0..).collect())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-close>
    pub fn close(&self) {
        // Step 1
        self.detached.set(true);
    }
}
