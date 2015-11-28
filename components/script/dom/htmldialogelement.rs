/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::cell::DOMRefCell;
use dom::bindings::codegen::Bindings::HTMLDialogElementBinding;
use dom::bindings::codegen::Bindings::HTMLDialogElementBinding::HTMLDialogElementMethods;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLDialogElement {
    htmlelement: HTMLElement,
    return_value: DOMRefCell<DOMString>,
}

impl HTMLDialogElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLDialogElement {
        HTMLDialogElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document),
            return_value: DOMRefCell::new(DOMString::new()),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLDialogElement> {
        let element = HTMLDialogElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLDialogElementBinding::Wrap)
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
}
