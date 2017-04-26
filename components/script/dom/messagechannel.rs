/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::MessageChannelBinding::{MessageChannelMethods, Wrap};
use dom::bindings::error::{Error, Fallible};
use dom::bindings::reflector::{Reflector, reflect_dom_object};
use dom::bindings::root::{Dom, DomRoot};
use dom::globalscope::GlobalScope;
use dom::messageport::MessagePort;
use dom_struct::dom_struct;

#[dom_struct]
pub struct MessageChannel {
    reflector_: Reflector,
    port1: Dom<MessagePort>,
    port2: Dom<MessagePort>,
}

impl MessageChannel {
    /// <https://html.spec.whatwg.org/multipage/#dom-messagechannel>
    pub fn Constructor(global: &GlobalScope) -> Fallible<DomRoot<MessageChannel>> {
        let incumbent = GlobalScope::incumbent().ok_or(Error::InvalidState)?;

        // Step 1
        let port1 = MessagePort::new(&incumbent);

        // Step 2
        let port2 = MessagePort::new(&incumbent);

        // Step 3
        port1.entangle(&port2);

        // Steps 4-6
        let channel = reflect_dom_object(Box::new(
            MessageChannel {
                reflector_: Reflector::new(),
                port1: Dom::from_ref(&port1),
                port2: Dom::from_ref(&port2),
            }),
            global,
            Wrap
        );

        // Step 7
        Ok(channel)
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
