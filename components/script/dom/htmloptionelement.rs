/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::node::Node;
use dom::text::Text;
use dom::virtualmethods::VirtualMethods;
use selectors::states::*;
use std::cell::Cell;
use util::str::{DOMString, split_html_space_chars, str_join};

#[dom_struct]
pub struct HTMLOptionElement {
    htmlelement: HTMLElement,

    /// https://html.spec.whatwg.org/multipage/#attr-option-selected
    selectedness: Cell<bool>,

    /// https://html.spec.whatwg.org/multipage/#concept-option-dirtiness
    dirtiness: Cell<bool>,
}

impl HTMLOptionElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLOptionElement {
        HTMLOptionElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(IN_ENABLED_STATE,
                                                      localName, prefix, document),
            selectedness: Cell::new(false),
            dirtiness: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLOptionElement> {
        let element = HTMLOptionElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLOptionElementBinding::Wrap)
    }

    pub fn set_selectedness(&self, selected: bool) {
        self.selectedness.set(selected);
    }

    fn pick_if_selected_and_reset(&self) {
        if let Some(select) = self.upcast::<Node>().ancestors()
                .filter_map(Root::downcast::<HTMLSelectElement>)
                .next() {
            if self.Selected() {
                select.pick_option(self);
            }
            select.ask_for_reset();
        }
    }
}

// FIXME(ajeffrey): Provide a way of buffering DOMStrings other than using Strings
fn collect_text(element: &Element, value: &mut String) {
    let svg_script = *element.namespace() == ns!(svg) && element.local_name() == &atom!("script");
    let html_script = element.is::<HTMLScriptElement>();
    if svg_script || html_script {
        return;
    }

    for child in element.upcast::<Node>().children() {
        if child.is::<Text>() {
            let characterdata = child.downcast::<CharacterData>().unwrap();
            value.push_str(&characterdata.Data());
        } else if let Some(element_child) = child.downcast() {
            collect_text(element_child, value);
        }
    }
}

impl HTMLOptionElementMethods for HTMLOptionElement {
    // https://html.spec.whatwg.org/multipage/#dom-option-disabled
    make_bool_getter!(Disabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-option-disabled
    make_bool_setter!(SetDisabled, "disabled");

    // https://html.spec.whatwg.org/multipage/#dom-option-text
    fn Text(&self) -> DOMString {
        let mut content = String::new();
        collect_text(self.upcast(), &mut content);
        DOMString::from(str_join(split_html_space_chars(&content), " "))
    }

    // https://html.spec.whatwg.org/multipage/#dom-option-text
    fn SetText(&self, value: DOMString) {
        self.upcast::<Node>().SetTextContent(Some(value))
    }

    // https://html.spec.whatwg.org/multipage/#attr-option-value
    fn Value(&self) -> DOMString {
        let element = self.upcast::<Element>();
        let attr = &atom!("value");
        if element.has_attribute(attr) {
            element.get_string_attribute(attr)
        } else {
            self.Text()
        }
    }

    // https://html.spec.whatwg.org/multipage/#attr-option-value
    make_setter!(SetValue, "value");

    // https://html.spec.whatwg.org/multipage/#attr-option-label
    fn Label(&self) -> DOMString {
        let element = self.upcast::<Element>();
        let attr = &atom!("label");
        if element.has_attribute(attr) {
            element.get_string_attribute(attr)
        } else {
            self.Text()
        }
    }

    // https://html.spec.whatwg.org/multipage/#attr-option-label
    make_setter!(SetLabel, "label");

    // https://html.spec.whatwg.org/multipage/#dom-option-defaultselected
    make_bool_getter!(DefaultSelected, "selected");

    // https://html.spec.whatwg.org/multipage/#dom-option-defaultselected
    make_bool_setter!(SetDefaultSelected, "selected");

    // https://html.spec.whatwg.org/multipage/#dom-option-selected
    fn Selected(&self) -> bool {
        self.selectedness.get()
    }

    // https://html.spec.whatwg.org/multipage/#dom-option-selected
    fn SetSelected(&self, selected: bool) {
        self.dirtiness.set(true);
        self.selectedness.set(selected);
        self.pick_if_selected_and_reset();
    }
}

impl VirtualMethods for HTMLOptionElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &atom!("disabled") => {
                let el = self.upcast::<Element>();
                match mutation {
                    AttributeMutation::Set(_) => {
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_disabled_state(false);
                        el.set_enabled_state(true);
                        el.check_parent_disabled_state_for_option();
                    }
                }
            },
            &atom!("selected") => {
                match mutation {
                    AttributeMutation::Set(_) => {
                        // https://html.spec.whatwg.org/multipage/#concept-option-selectedness
                        if !self.dirtiness.get() {
                            self.selectedness.set(true);
                        }
                    },
                    AttributeMutation::Removed => {
                        // https://html.spec.whatwg.org/multipage/#concept-option-selectedness
                        if !self.dirtiness.get() {
                            self.selectedness.set(false);
                        }
                    },
                }
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.upcast::<Element>().check_parent_disabled_state_for_option();

        self.pick_if_selected_and_reset();
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        let node = self.upcast::<Node>();
        let el = self.upcast::<Element>();
        if node.GetParentNode().is_some() {
            el.check_parent_disabled_state_for_option();
        } else {
            el.check_disabled_attribute();
        }
    }
}
