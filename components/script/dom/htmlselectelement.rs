/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::{Attr, AttrValue};
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use dom::bindings::codegen::Bindings::HTMLSelectElementBinding;
use dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElementMethods;
use dom::bindings::codegen::UnionTypes::HTMLElementOrLong;
use dom::bindings::codegen::UnionTypes::HTMLOptionElementOrHTMLOptGroupElement;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmlfieldsetelement::HTMLFieldSetElement;
use dom::htmlformelement::{FormControl, HTMLFormElement};
use dom::htmloptionelement::HTMLOptionElement;
use dom::node::{Node, window_from_node};
use dom::nodelist::NodeList;
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;
use selectors::states::*;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLSelectElement {
    htmlelement: HTMLElement
}

static DEFAULT_SELECT_SIZE: u32 = 0;

impl HTMLSelectElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLSelectElement {
        HTMLSelectElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(IN_ENABLED_STATE,
                                                      localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLSelectElement> {
        let element = HTMLSelectElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLSelectElementBinding::Wrap)
    }

    // https://html.spec.whatwg.org/multipage/#ask-for-a-reset
    pub fn ask_for_reset(&self) {
        if self.Multiple() {
            return;
        }

        let mut first_enabled: Option<Root<HTMLOptionElement>> = None;
        let mut last_selected: Option<Root<HTMLOptionElement>> = None;

        let node = self.upcast::<Node>();
        for opt in node.traverse_preorder().filter_map(Root::downcast::<HTMLOptionElement>) {
            if opt.Selected() {
                opt.set_selectedness(false);
                last_selected = Some(Root::from_ref(opt.r()));
            }
            let element = opt.upcast::<Element>();
            if first_enabled.is_none() && !element.get_disabled_state() {
                first_enabled = Some(Root::from_ref(opt.r()));
            }
        }

        if let Some(last_selected) = last_selected {
            last_selected.set_selectedness(true);
        } else {
            if self.display_size() == 1 {
                if let Some(first_enabled) = first_enabled {
                    first_enabled.set_selectedness(true);
                }
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-select-pick
    pub fn pick_option(&self, picked: &HTMLOptionElement) {
        if !self.Multiple() {
            let node = self.upcast::<Node>();
            let picked = picked.upcast();
            for opt in node.traverse_preorder().filter_map(Root::downcast::<HTMLOptionElement>) {
                if opt.upcast::<HTMLElement>() != picked {
                    opt.set_selectedness(false);
                }
            }
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-select-size
    fn display_size(&self) -> u32 {
         if self.Size() == 0 {
             if self.Multiple() {
                 4
             } else {
                 1
             }
         } else {
             self.Size()
         }
     }
}

impl HTMLSelectElementMethods for HTMLSelectElement {
    // https://html.spec.whatwg.org/multipage/#dom-cva-validity
    fn Validity(&self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(window.r())
    }

    // Note: this function currently only exists for union.html.
    // https://html.spec.whatwg.org/multipage/#dom-select-add
    fn Add(&self, _element: HTMLOptionElementOrHTMLOptGroupElement, _before: Option<HTMLElementOrLong>) {
    }

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_getter!(Disabled);

    // https://html.spec.whatwg.org/multipage/#dom-fe-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://html.spec.whatwg.org/multipage/#dom-select-multiple
    make_bool_getter!(Multiple);

    // https://html.spec.whatwg.org/multipage/#dom-select-multiple
    make_bool_setter!(SetMultiple, "multiple");

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_getter!(Name);

    // https://html.spec.whatwg.org/multipage/#dom-fe-name
    make_setter!(SetName, "name");

    // https://html.spec.whatwg.org/multipage/#dom-select-size
    make_uint_getter!(Size, "size", DEFAULT_SELECT_SIZE);

    // https://html.spec.whatwg.org/multipage/#dom-select-size
    make_uint_setter!(SetSize, "size", DEFAULT_SELECT_SIZE);

    // https://html.spec.whatwg.org/multipage/#dom-select-type
    fn Type(&self) -> DOMString {
        DOMString::from(if self.Multiple() {
            "select-multiple"
        } else {
            "select-one"
        })
    }

    // https://html.spec.whatwg.org/multipage/#dom-lfe-labels
    fn Labels(&self) -> Root<NodeList> {
        self.upcast::<HTMLElement>().labels()
    }
}

impl VirtualMethods for HTMLSelectElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        if attr.local_name() == &atom!("disabled") {
            let el = self.upcast::<Element>();
            match mutation {
                AttributeMutation::Set(_) => {
                    el.set_disabled_state(true);
                    el.set_enabled_state(false);
                },
                AttributeMutation::Removed => {
                    el.set_disabled_state(false);
                    el.set_enabled_state(true);
                    el.check_ancestors_disabled_state_for_form_control();
                }
            }
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.upcast::<Element>().check_ancestors_disabled_state_for_form_control();
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        let node = self.upcast::<Node>();
        let el = self.upcast::<Element>();
        if node.ancestors().any(|ancestor| ancestor.is::<HTMLFieldSetElement>()) {
            el.check_ancestors_disabled_state_for_form_control();
        } else {
            el.check_disabled_attribute();
        }
    }

    fn parse_plain_attribute(&self, local_name: &Atom, value: DOMString) -> AttrValue {
        match *local_name {
            atom!("size") => AttrValue::from_u32(value, DEFAULT_SELECT_SIZE),
            _ => self.super_type().unwrap().parse_plain_attribute(local_name, value),
        }
    }
}

impl FormControl for HTMLSelectElement {}
