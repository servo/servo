/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix};
use js::rust::HandleObject;
use style::attr::AttrValue;

use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::HTMLDialogElementBinding::HTMLDialogElementMethods;
use crate::dom::bindings::import::module::{Error, Fallible};
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::Element;
use crate::dom::eventtarget::EventTarget;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{window_from_node, Node};

#[dom_struct]
pub struct HTMLDialogElement {
    htmlelement: HTMLElement,
    return_value: DomRefCell<DOMString>,
    close_watcher: DomRefCell<Option<()>>,
    is_modal: DomRefCell<bool>,
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
            // https://html.spec.whatwg.org/multipage/#dialog-close-watcher,
            close_watcher: DomRefCell::new(None),
            // https://html.spec.whatwg.org/multipage/#is-modal
            is_modal: DomRefCell::new(false),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLDialogElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLDialogElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl HTMLDialogElementMethods for HTMLDialogElement {
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

    // https://html.spec.whatwg.org/multipage/#dom-dialog-showmodal
    fn ShowModal(&self) -> Fallible<()> {
        let element = self.upcast::<Element>();
        let node = element.upcast::<Node>();
        let _doc = node.downcast::<Document>().unwrap();
        let has_open = element.has_attribute(&local_name!("open"));

        // 1. If this has an open attribute and the is modal flag of this is true, then return.
        if has_open && *self.is_modal.borrow() {
            return Ok(());
        }

        // 2. If this has an open attribute, then throw an "InvalidStateError" DOMException.
        // 3. If this is not connected, then throw an "InvalidStateError" DOMException.
        // TODO: 4. If this is in the popover showing state, then throw an "InvalidStateError" DOMException.
        if has_open || !element.is_connected() {
            return Err(Error::InvalidState);
        }

        // 5. Add an open attribute to this, whose value is the empty string.
        element.set_attribute(&local_name!("open"), AttrValue::String("".to_string()));

        // 6. Set the is modal flag of this to true.
        *self.is_modal.borrow_mut() = true;

        // TODO: 7. Let this's node document be blocked by the modal dialog this.

        // TODO: 8. If this's node document's top layer does not already contain this, then add an element to the top layer given this.

        // TODO: 9. Set this's close watcher to the result of establishing a close watcher given this's relevant global object, with:
        //       cancelAction given canPreventClose being to return the result of firing an event named cancel at this, with the cancelable attribute initialized to canPreventClose.
        //       closeAction being to close the dialog given this and null.
        *self.close_watcher.borrow_mut() = Some(());

        // TODO: 10. Set this's previously focused element to the focused element.

        // TODO: 11. Let hideUntil be the result of running topmost popover ancestor given this, null, and false.

        // TODO: 12. If hideUntil is null, then set hideUntil to this's node document.

        // TODO: 13. Run hide all popovers until given hideUntil, false, and true.

        // TODO: 14. Run the dialog focusing steps given this.

        return Ok(());
    }

    // https://html.spec.whatwg.org/multipage/#dom-dialog-close
    fn Close(&self, return_value: Option<DOMString>) {
        let element = self.upcast::<Element>();
        let target = self.upcast::<EventTarget>();
        let win = window_from_node(self);

        // Step 1 & 2
        if element
            .remove_attribute(&ns!(), &local_name!("open"))
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
        win.task_manager()
            .dom_manipulation_task_source()
            .queue_simple_event(target, atom!("close"), &win);
    }
}
