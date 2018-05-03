/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLDetailsElementBinding;
use dom::bindings::codegen::Bindings::HTMLDetailsElementBinding::HTMLDetailsElementMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::refcounted::Trusted;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::element::AttributeMutation;
use dom::eventtarget::EventTarget;
use dom::htmlelement::HTMLElement;
use dom::node::{Node, window_from_node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use std::cell::Cell;
use task_source::TaskSource;
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLDetailsElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
    toggle_counter: Cell<u32>
}

impl<TH: TypeHolderTrait> HTMLDetailsElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLDetailsElement<TH> {
        HTMLDetailsElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document),
            toggle_counter: Cell::new(0)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLDetailsElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLDetailsElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLDetailsElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLDetailsElementMethods for HTMLDetailsElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_getter!(Open, "open");

    // https://html.spec.whatwg.org/multipage/#dom-details-open
    make_bool_setter!(SetOpen, "open");
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLDetailsElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn attribute_mutated(&self, attr: &Attr<TH>, mutation: AttributeMutation<TH>) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);

        if attr.local_name() == &local_name!("open") {
            let counter = self.toggle_counter.get() + 1;
            self.toggle_counter.set(counter);

            let window = window_from_node(self);
            let this = Trusted::new(self);
            // FIXME(nox): Why are errors silenced here?
            let _ = window.dom_manipulation_task_source().queue(
                task!(details_notification_task_steps: move || {
                    let this = this.root();
                    if counter == this.toggle_counter.get() {
                        this.upcast::<EventTarget<TH>>().fire_event(atom!("toggle"));
                    }
                }),
                window.upcast(),
            );
        }
    }
}
