/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;

use dom_struct::dom_struct;
use html5ever::{local_name, LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLDetailsElementBinding::HTMLDetailsElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::refcounted::Trusted;
use crate::dom::bindings::root::DomRoot;
use crate::dom::document::Document;
use crate::dom::element::AttributeMutation;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{window_from_node, Node, NodeDamage};
use crate::dom::virtualmethods::VirtualMethods;
use crate::task_source::TaskSource;

#[dom_struct]
pub struct HTMLDetailsElement {
    htmlelement: HTMLElement,
    toggle_counter: Cell<u32>,
}

impl HTMLDetailsElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLDetailsElement {
        HTMLDetailsElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            toggle_counter: Cell::new(0),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLDetailsElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLDetailsElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    pub fn toggle(&self) {
        self.SetOpen(!self.Open());
    }
}

impl HTMLDetailsElementMethods for HTMLDetailsElement {
    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_getter!(Open, "open");

    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_setter!(SetOpen, "open");
}

impl VirtualMethods for HTMLDetailsElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        if attr.local_name() == &local_name!("open") {
            let counter = self.toggle_counter.get() + 1;
            self.toggle_counter.set(counter);

            let window = window_from_node(self);
            let this = Trusted::new(self);
            // FIXME(nox): Why are errors silenced here?
            let _ = window.task_manager().dom_manipulation_task_source().queue(
                task!(details_notification_task_steps: move || {
                    let this = this.root();
                    if counter == this.toggle_counter.get() {
                        this.upcast::<EventTarget>().fire_event(atom!("toggle"));
                    }
                }),
                window.upcast(),
            );
            self.upcast::<Node>().dirty(NodeDamage::OtherNodeDamage)
        }
    }
}
