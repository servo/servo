/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLFrameSetElementBinding;
use crate::dom::bindings::codegen::Bindings::HTMLFrameSetElementBinding::HTMLFrameSetElementMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::AttributeMutation;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{document_from_node, window_from_node, Node};
use crate::dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

#[dom_struct]
pub struct HTMLFrameSetElement {
    htmlelement: HTMLElement,
}

impl HTMLFrameSetElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLFrameSetElement {
        HTMLFrameSetElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> DomRoot<HTMLFrameSetElement> {
        Node::reflect_node(
            Box::new(HTMLFrameSetElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLFrameSetElementBinding::Wrap,
        )
    }
}

impl HTMLFrameSetElementMethods for HTMLFrameSetElement {
    // https://html.spec.whatwg.org/multipage/#windoweventhandlers
    window_event_handlers!(ForwardToWindow);
}

impl VirtualMethods for HTMLFrameSetElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        use crate::dom::macros::BODY_FRAMESET_EVENT_HANDLERS;
        match (attr.local_name(), mutation) {
            (name, AttributeMutation::Set(_)) if name.starts_with("on") => {
                // https://html.spec.whatwg.org/multipage/
                // #event-handlers-on-elements,-document-objects,-and-window-objects:event-handlers-3
                if BODY_FRAMESET_EVENT_HANDLERS.contains(name) {
                    let window = window_from_node(self);
                    let evtarget = window.upcast::<EventTarget>(); // forwarded event
                    let source_line = 1; //TODO(#9604) obtain current JS execution line
                    evtarget.set_event_handler_uncompiled(
                        window.get_url(),
                        source_line,
                        &name[2..],
                        DOMString::from((**attr.value()).to_owned()),
                    );
                    // No need to call supertype mutation checks; we're done
                    return;
                }
            },
            _ => (),
        };
        self.super_type().unwrap().attribute_mutated(attr, mutation);
    }
}
