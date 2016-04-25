/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLOutputElementBinding;
use dom::bindings::codegen::Bindings::HTMLOutputElementBinding::HTMLOutputElementMethods;
use dom::bindings::codegen::Bindings::ValidityStateBinding::ValidityStateMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::Element;
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::node::{Node, window_from_node};
use dom::nodelist::NodeList;
use dom::validitystate::ValidityState;
use string_cache::Atom;
use util::str::DOMString;


#[dom_struct]
pub struct HTMLOutputElement {
    htmlelement: HTMLElement
}

impl HTMLOutputElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLOutputElement {
        HTMLOutputElement {
            htmlelement:
                HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLOutputElement> {
        let element = HTMLOutputElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLOutputElementBinding::Wrap)
    }
}

impl HTMLOutputElementMethods for HTMLOutputElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(window.r(), self.upcast())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> Root<NodeList> {
        self.upcast::<HTMLElement>().labels()
    }
}

impl FormControl for HTMLOutputElement {
    fn candidate_for_validation(&self, element: &Element) -> bool {
       if element.as_maybe_validatable().is_some() {
            return true
        }
        else {
           return false
        }
    }

    fn satisfies_constraints(&self, element: &Element) -> bool {
        let vs = ValidityState::new(window_from_node(self).r(), element);
        return vs.Valid()
    }
    fn ValueMissing(&self) -> bool {
            return false;
    }
    fn TypeMismatch(&self) -> bool {
            return false;
    }
    fn PatternMismatch(&self) -> bool {
            return false;
    }
    fn TooLong(&self) -> bool {
            return false;
    }
    fn TooShort(&self) -> bool {
            return false;
    }
    fn RangeUnderflow(&self) -> bool {
            return false;
    }
    fn RangeOverflow(&self) -> bool {
            return false;
    }
    fn StepMismatch(&self) -> bool {
            return false;
    }
    fn BadInput(&self) -> bool {
            return false;
    }
    fn CustomError(&self) -> bool {
            return false;
    }
}
