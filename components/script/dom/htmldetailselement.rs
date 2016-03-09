/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLDetailsElementBinding;
use dom::bindings::codegen::Bindings::HTMLDetailsElementBinding::HTMLDetailsElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::bindings::refcounted::Trusted;
use dom::document::Document;
use dom::element::AttributeMutation;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use script_thread::{MainThreadScriptChan, Runnable, ScriptChan};
use std::cell::Cell;
use string_cache::Atom;
use task_source::dom_manipulation::DOMManipulationTask;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLDetailsElement {
    htmlelement: HTMLElement,
    toggle_counter: Cell<u32>
}

impl HTMLDetailsElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLDetailsElement {
        HTMLDetailsElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document),
            toggle_counter: Cell::new(0)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLDetailsElement> {
        let element = HTMLDetailsElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLDetailsElementBinding::Wrap)
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

        if attr.local_name() == &atom!("open") {
            let counter = self.toggle_counter.get() + 1;
            self.toggle_counter.set(counter);

            let window = window_from_node(self);
            let window = window.r();
            let task_source = window.dom_manipulation_task_source();
            let chan = MainThreadScriptChan(window.main_thread_script_chan().clone()).clone();
            let details = Trusted::new(self, chan.clone());
            let runnable = box ToggleEventRunnable {
                element: details,
                toggle_number: counter
            };
            let _ = task_source.queue(DOMManipulationTask::FireToggleEvent(runnable));
        }
    }
}

pub struct ToggleEventRunnable {
    element: Trusted<HTMLDetailsElement>,
    toggle_number: u32
}

impl Runnable for ToggleEventRunnable {
    fn handler(self: Box<ToggleEventRunnable>) {
        let target = self.element.root();
        if target.check_toggle_count(self.toggle_number) {
            target.upcast::<EventTarget>().fire_simple_event("toggle");
        }
    }
}
