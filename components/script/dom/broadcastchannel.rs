/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use crate::dom::bindings::codegen::Bindings::BroadcastChannelBinding::BroadcastChannelMethods;
use crate::dom::bindings::error::{Error, ErrorResult};
use crate::dom::bindings::reflector::{reflect_dom_object, DomObject};
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::bindings::structuredclone;
use crate::dom::eventtarget::EventTarget;
use crate::dom::globalscope::GlobalScope;
use crate::script_runtime::JSContext as SafeJSContext;
use dom_struct::dom_struct;
use js::rust::HandleValue;
use script_traits::BroadcastMsg;
use std::cell::Cell;
use uuid::Uuid;

#[dom_struct]
pub struct BroadcastChannel {
    eventtarget: EventTarget,
    name: DOMString,
    closed: Cell<bool>,
    id: Uuid,
}

impl BroadcastChannel {
    /// <https://html.spec.whatwg.org/multipage/#broadcastchannel>
    #[allow(non_snake_case)]
    pub fn Constructor(global: &GlobalScope, name: DOMString) -> DomRoot<BroadcastChannel> {
        BroadcastChannel::new(global, name)
    }

    pub fn new(global: &GlobalScope, name: DOMString) -> DomRoot<BroadcastChannel> {
        let channel = reflect_dom_object(Box::new(BroadcastChannel::new_inherited(name)), global);
        global.track_broadcast_channel(&*channel);
        channel
    }

    pub fn new_inherited(name: DOMString) -> BroadcastChannel {
        BroadcastChannel {
            eventtarget: EventTarget::new_inherited(),
            name,
            closed: Default::default(),
            id: Uuid::new_v4(),
        }
    }

    /// The unique Id of this channel.
    /// Used for filtering out the sender from the local broadcast.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Is this channel closed?
    pub fn closed(&self) -> bool {
        self.closed.get()
    }
}

impl BroadcastChannelMethods for BroadcastChannel {
    /// <https://html.spec.whatwg.org/multipage/#dom-messageport-postmessage>
    fn PostMessage(&self, cx: SafeJSContext, message: HandleValue) -> ErrorResult {
        // Step 3, if closed.
        if self.closed.get() {
            return Err(Error::InvalidState);
        }

        // Step 6, StructuredSerialize(message).
        let data = structuredclone::write(cx, message, None)?;

        let global = self.global();

        let msg = BroadcastMsg {
            origin: global.origin().immutable().clone(),
            channel_name: self.Name().to_string(),
            data,
        };

        global.schedule_broadcast(msg, &self.id);
        Ok(())
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-name>
    fn Name(&self) -> DOMString {
        self.name.clone()
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-broadcastchannel-close>
    fn Close(&self) {
        self.closed.set(true);
    }

    // <https://html.spec.whatwg.org/multipage/#handler-broadcastchannel-onmessageerror>
    event_handler!(messageerror, GetOnmessageerror, SetOnmessageerror);

    // <https://html.spec.whatwg.org/multipage/#handler-broadcastchannel-onmessage>
    event_handler!(message, GetOnmessage, SetOnmessage);
}
