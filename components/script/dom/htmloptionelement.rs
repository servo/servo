/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElementBinding::HTMLSelectElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::element::{AttributeMutation, Element};
use dom::htmlelement::HTMLElement;
use dom::htmlformelement::HTMLFormElement;
use dom::htmloptgroupelement::HTMLOptGroupElement;
use dom::htmlscriptelement::HTMLScriptElement;
use dom::htmlselectelement::HTMLSelectElement;
use dom::node::{Node, UnbindContext};
use dom::text::Text;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use std::cell::Cell;
use style::element_state::ElementState;
use style::str::{split_html_space_chars, str_join};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLOptionElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,

    /// <https://html.spec.whatwg.org/multipage/#attr-option-selected>
    selectedness: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#concept-option-dirtiness>
    dirtiness: Cell<bool>,
}

impl<TH: TypeHolderTrait> HTMLOptionElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLOptionElement<TH> {
        HTMLOptionElement {
            htmlelement:
                HTMLElement::new_inherited_with_state(ElementState::IN_ENABLED_STATE,
                                                      local_name, prefix, document),
            selectedness: Cell::new(false),
            dirtiness: Cell::new(false),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLOptionElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLOptionElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLOptionElementBinding::Wrap)
    }

    pub fn set_selectedness(&self, selected: bool) {
        self.selectedness.set(selected);
    }

    pub fn set_dirtiness(&self, dirtiness: bool) {
        self.dirtiness.set(dirtiness);
    }

    fn pick_if_selected_and_reset(&self) {
        if let Some(select) = self.upcast::<Node<TH>>().ancestors()
                .filter_map(DomRoot::downcast::<HTMLSelectElement<TH>>)
                .next() {
            if self.Selected() {
                select.pick_option(self);
            }
            select.ask_for_reset();
        }
    }
}

// FIXME(ajeffrey): Provide a way of buffering DOMStrings other than using Strings
fn collect_text<TH: TypeHolderTrait>(element: &Element<TH>, value: &mut String) {
    let svg_script = *element.namespace() == ns!(svg) && element.local_name() == &local_name!("script");
    let html_script = element.is::<HTMLScriptElement<TH>>();
    if svg_script || html_script {
        return;
    }

    for child in element.upcast::<Node<TH>>().children() {
        if child.is::<Text<TH>>() {
            let characterdata = child.downcast::<CharacterData<TH>>().unwrap();
            value.push_str(&characterdata.Data());
        } else if let Some(element_child) = child.downcast() {
            collect_text(element_child, value);
        }
    }
}

impl<TH: TypeHolderTrait> HTMLOptionElementMethods<TH> for HTMLOptionElement<TH> {
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
        self.upcast::<Node<TH>>().SetTextContent(Some(value))
    }

    // https://html.spec.whatwg.org/multipage/#dom-option-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement<TH>>> {
        let parent = self.upcast::<Node<TH>>().GetParentNode().and_then(|p|
            if p.is::<HTMLOptGroupElement<TH>>() {
                p.upcast::<Node<TH>>().GetParentNode()
            } else {
                Some(p)
            }
        );

        parent.and_then(|p| p.downcast::<HTMLSelectElement<TH>>().and_then(|s| s.GetForm()))
    }

    // https://html.spec.whatwg.org/multipage/#attr-option-value
    fn Value(&self) -> DOMString {
        let element = self.upcast::<Element<TH>>();
        let attr = &local_name!("value");
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
        let element = self.upcast::<Element<TH>>();
        let attr = &local_name!("label");
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

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLOptionElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn attribute_mutated(&self, attr: &Attr<TH>, mutation: AttributeMutation<TH>) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("disabled") => {
                let el = self.upcast::<Element<TH>>();
                match mutation {
                    AttributeMutation::Set(_) => {
                        el.set_disabled_state(true);
                        el.set_enabled_state(false);
                    },
                    AttributeMutation::Removed => {
                        el.set_disabled_state(false);
                        el.set_enabled_state(true);
                        el.check_parent_disabled_state_for_option();
                    },
                    AttributeMutation::_p(_) => return,
                }
            },
            &local_name!("selected") => {
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
                    AttributeMutation::_p(_) => return,
                }
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.upcast::<Element<TH>>().check_parent_disabled_state_for_option();

        self.pick_if_selected_and_reset();
    }

    fn unbind_from_tree(&self, context: &UnbindContext<TH>) {
        self.super_type().unwrap().unbind_from_tree(context);

        if let Some(select) = context.parent.inclusive_ancestors()
                .filter_map(DomRoot::downcast::<HTMLSelectElement<TH>>)
                .next() {
            select.ask_for_reset();
        }

        let node = self.upcast::<Node<TH>>();
        let el = self.upcast::<Element<TH>>();
        if node.GetParentNode().is_some() {
            el.check_parent_disabled_state_for_option();
        } else {
            el.check_disabled_attribute();
        }
    }
}
