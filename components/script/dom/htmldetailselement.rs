/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLDetailsElementBinding;
use dom::bindings::codegen::Bindings::HTMLDetailsElementBinding::HTMLDetailsElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::AttributeMutation;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::LocalName;
use script_thread::Runnable;
use std::cell::Cell;
use task_source::TaskSource;

#[dom_struct]
pub struct HTMLDetailsElement {
    htmlelement: HTMLElement,
    toggle_counter: Cell<u32>
}

impl HTMLDetailsElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLDetailsElement {
        HTMLDetailsElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            toggle_counter: Cell::new(0)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLDetailsElement> {
        Node::reflect_node(box HTMLDetailsElement::new_inherited(local_name, prefix, document),
                           document,
                           HTMLDetailsElementBinding::Wrap)
    }

    pub fn check_toggle_count(&self, number: u32) -> bool {
        number == self.toggle_counter.get()
    }
}

impl HTMLDetailsElementMethods for HTMLDetailsElement {
    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_getter!(Open, "open");

    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_setter!(SetOpen, "open");
}

impl VirtualMethods for HTMLDetailsElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        if attr.local_name() == &local_name!("open") {
            let counter = self.toggle_counter.get() + 1;
            self.toggle_counter.set(counter);

            let window = window_from_node(self);
            let task_source = window.dom_manipulation_task_source();
            let details = Trusted::new(self);
            let runnable = box DetailsNotificationRunnable {
                element: details,
                toggle_number: counter
            };
            let _ = task_source.queue(runnable, window.upcast());
        }
    }
}

pub struct DetailsNotificationRunnable {
    element: Trusted<HTMLDetailsElement>,
    toggle_number: u32
}

impl Runnable for DetailsNotificationRunnable {
    fn name(&self) -> &'static str { "DetailsNotificationRunnable" }

    fn handler(self: Box<DetailsNotificationRunnable>) {
        let target = self.element.root();
        if target.check_toggle_count(self.toggle_number) {
            target.upcast::<EventTarget>().fire_event(atom!("toggle"));
        }
    }
}
