/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, namespace_url, ns};
use js::rust::HandleObject;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLDialogElementBinding::HTMLDialogElementMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{Node, NodeTraits};
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLDialogElement {
    htmlelement: HTMLElement,
    return_value: DomRefCell<DOMString>,
}

impl HTMLDialogElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLDialogElement {
        HTMLDialogElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
            return_value: DomRefCell::new(DOMString::new()),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLDialogElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLDialogElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }
}

impl HTMLDialogElementMethods<crate::DomTypeHolder> for HTMLDialogElement {
    // https://html.spec.whatwg.org/multipage/#dom-dialog-open
    make_bool_getter!(Open, "open");

    // https://html.spec.whatwg.org/multipage/#dom-dialog-open
    make_bool_setter!(SetOpen, "open");

    // https://html.spec.whatwg.org/multipage/#dom-dialog-returnvalue
    fn ReturnValue(&self) -> DOMString {
        let return_value = self.return_value.borrow();
        return_value.clone()
    }

    // https://html.spec.whatwg.org/multipage/#dom-dialog-returnvalue
    fn SetReturnValue(&self, return_value: DOMString) {
        *self.return_value.borrow_mut() = return_value;
    }

    /// <https://html.spec.whatwg.org/multipage/#dom-dialog-show>
    fn Show(&self, can_gc: CanGc) {
        let element = self.upcast::<Element>();

        // Step 1 TODO: Check is modal flag is false
        if element.has_attribute(&local_name!("open")) {
            return;
        }

        // TODO: Step 2 If this has an open attribute, then throw an "InvalidStateError" DOMException.

        // Step 3
        element.set_bool_attribute(&local_name!("open"), true, can_gc);

        // TODO: Step 4 Set this's previously focused element to the focused element.

        // TODO: Step 5 Let hideUntil be the result of running topmost popover ancestor given this, null, and false.

        // TODO: Step 6 If hideUntil is null, then set hideUntil to this's node document.

        // TODO: Step 7 Run hide all popovers until given hideUntil, false, and true.

        // TODO(Issue #32702): Step 8 Run the dialog focusing steps given this.
    }

    // https://html.spec.whatwg.org/multipage/#dom-dialog-close
    fn Close(&self, return_value: Option<DOMString>, can_gc: CanGc) {
        let element = self.upcast::<Element>();
        let target = self.upcast::<EventTarget>();

        // Step 1 & 2
        if element
            .remove_attribute(&ns!(), &local_name!("open"), can_gc)
            .is_none()
        {
            return;
        }

        // Step 3
        if let Some(new_value) = return_value {
            *self.return_value.borrow_mut() = new_value;
        }

        // TODO: Step 4 implement pending dialog stack removal

        // Step 5
        self.owner_global()
            .task_manager()
            .dom_manipulation_task_source()
            .queue_simple_event(target, atom!("close"));
    }
}
