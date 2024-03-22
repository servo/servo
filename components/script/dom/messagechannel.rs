/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::MessageChannelBinding::MessageChannelMethods;
use crate::dom::bindings::reflector::{reflect_dom_object_with_proto, Reflector};
use crate::dom::bindings::root::{Dom, DomRoot};
use crate::dom::globalscope::GlobalScope;
use crate::dom::messageport::MessagePort;

#[dom_struct]
pub struct MessageChannel {
    reflector_: Reflector,
    port1: Dom<MessagePort>,
    port2: Dom<MessagePort>,
}

impl MessageChannel {
    /// <https://html.spec.whatwg.org/multipage/#dom-messagechannel>
    #[allow(non_snake_case)]
    pub fn Constructor(
        global: &GlobalScope,
        proto: Option<HandleObject>,
    ) -> DomRoot<MessageChannel> {
        MessageChannel::new(global, proto)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messagechannel>
    fn new(incumbent: &GlobalScope, proto: Option<HandleObject>) -> DomRoot<MessageChannel> {
        // Step 1
        let port1 = MessagePort::new(incumbent);

        // Step 2
        let port2 = MessagePort::new(incumbent);

        incumbent.track_message_port(&*port1, None);
        incumbent.track_message_port(&*port2, None);

        // Step 3
        incumbent.entangle_ports(*port1.message_port_id(), *port2.message_port_id());

        // Steps 4-6
        reflect_dom_object_with_proto(
            Box::new(MessageChannel::new_inherited(&*port1, &*port2)),
            incumbent,
            proto,
        )
    }

    pub fn new_inherited(port1: &MessagePort, port2: &MessagePort) -> MessageChannel {
        MessageChannel {
            reflector_: Reflector::new(),
            port1: Dom::from_ref(port1),
            port2: Dom::from_ref(port2),
        }
    }
}

impl MessageChannelMethods for MessageChannel {
    /// <https://html.spec.whatwg.org/multipage/#dom-messagechannel-port1>
    fn Port1(&self) -> DomRoot<MessagePort> {
        DomRoot::from_ref(&*self.port1)
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-messagechannel-port2>
    fn Port2(&self) -> DomRoot<MessagePort> {
        DomRoot::from_ref(&*self.port2)
    }
}
