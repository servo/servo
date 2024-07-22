/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::Cell;
use std::convert::TryInto;

use dom_struct::dom_struct;
use html5ever::{local_name, namespace_url, ns, LocalName, Prefix, QualName};
use js::rust::HandleObject;
use style::str::{split_html_space_chars, str_join};
use style_dom::ElementState;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use crate::dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use crate::dom::bindings::codegen::Bindings::HTMLSelectElementBinding::HTMLSelectElement_Binding::HTMLSelectElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::codegen::Bindings::WindowBinding::WindowMethods;
use crate::dom::bindings::error::Fallible;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::characterdata::CharacterData;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, CustomElementCreationMode, Element, ElementCreator};
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlformelement::HTMLFormElement;
use crate::dom::htmloptgroupelement::HTMLOptGroupElement;
use crate::dom::htmlscriptelement::HTMLScriptElement;
use crate::dom::htmlselectelement::HTMLSelectElement;
use crate::dom::node::{BindContext, Node, ShadowIncluding, UnbindContext};
use crate::dom::text::Text;
use crate::dom::validation::Validatable;
use crate::dom::validitystate::ValidationFlags;
use crate::dom::virtualmethods::VirtualMethods;
use crate::dom::window::Window;

#[dom_struct]
pub struct HTMLOptionElement {
    htmlelement: HTMLElement,

    /// <https://html.spec.whatwg.org/multipage/#attr-option-selected>
    selectedness: Cell<bool>,

    /// <https://html.spec.whatwg.org/multipage/#concept-option-dirtiness>
    dirtiness: Cell<bool>,
}

impl HTMLOptionElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLOptionElement {
        HTMLOptionElement {
            htmlelement: HTMLElement::new_inherited_with_state(
                ElementState::ENABLED,
                local_name,
                prefix,
                document,
            ),
            selectedness: Cell::new(false),
            dirtiness: Cell::new(false),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLOptionElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLOptionElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    // https://html.spec.whatwg.org/multipage/#dom-option
    #[allow(non_snake_case)]
    pub fn Option(
        window: &Window,
        proto: Option<HandleObject>,
        text: DOMString,
        value: Option<DOMString>,
        default_selected: bool,
        selected: bool,
    ) -> Fallible<DomRoot<HTMLOptionElement>> {
        let element = Element::create(
            QualName::new(None, ns!(html), local_name!("option")),
            None,
            &window.Document(),
            ElementCreator::ScriptCreated,
            CustomElementCreationMode::Synchronous,
            proto,
        );

        let option = DomRoot::downcast::<HTMLOptionElement>(element).unwrap();

        if !text.is_empty() {
            option.upcast::<Node>().SetTextContent(Some(text))
        }

        if let Some(val) = value {
            option.SetValue(val)
        }

        option.SetDefaultSelected(default_selected);
        option.set_selectedness(selected);
        option.update_select_validity();
        Ok(option)
    }

    pub fn set_selectedness(&self, selected: bool) {
        self.selectedness.set(selected);
    }

    pub fn set_dirtiness(&self, dirtiness: bool) {
        self.dirtiness.set(dirtiness);
    }

    fn pick_if_selected_and_reset(&self) {
        if let Some(select) = self
            .upcast::<Node>()
            .ancestors()
            .filter_map(DomRoot::downcast::<HTMLSelectElement>)
            .next()
        {
            if self.Selected() {
                select.pick_option(self);
            }
            select.ask_for_reset();
        }
    }

    // https://html.spec.whatwg.org/multipage/#concept-option-index
    fn index(&self) -> i32 {
        if let Some(parent) = self.upcast::<Node>().GetParentNode() {
            if let Some(select_parent) = parent.downcast::<HTMLSelectElement>() {
                // return index in parent select's list of options
                return self.index_in_select(select_parent);
            } else if parent.is::<HTMLOptGroupElement>() {
                if let Some(grandparent) = parent.GetParentNode() {
                    if let Some(select_grandparent) = grandparent.downcast::<HTMLSelectElement>() {
                        // return index in grandparent select's list of options
                        return self.index_in_select(select_grandparent);
                    }
                }
            }
        }
        // "If the option element is not in a list of options,
        // then the option element's index is zero."
        // self is neither a child of a select, nor a grandchild of a select
        // via an optgroup, so it is not in a list of options
        0
    }

    fn index_in_select(&self, select: &HTMLSelectElement) -> i32 {
        match select.list_of_options().position(|n| &*n == self) {
            Some(index) => index.try_into().unwrap_or(0),
            None => {
                // shouldn't happen but not worth a browser panic
                warn!(
                    "HTMLOptionElement called index_in_select at a select that did not contain it"
                );
                0
            },
        }
    }

    fn update_select_validity(&self) {
        if let Some(select) = self
            .upcast::<Node>()
            .ancestors()
            .filter_map(DomRoot::downcast::<HTMLSelectElement>)
            .next()
        {
            select
                .validity_state()
                .perform_validation_and_update(ValidationFlags::all());
        }
    }
}

// FIXME(ajeffrey): Provide a way of buffering DOMStrings other than using Strings
fn collect_text(element: &Element, value: &mut String) {
    let svg_script =
        *element.namespace() == ns!(svg) && element.local_name() == &local_name!("script");
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

    // https://html.spec.whatwg.org/multipage/#dom-option-form
    fn GetForm(&self) -> Option<DomRoot<HTMLFormElement>> {
        let parent = self.upcast::<Node>().GetParentNode().and_then(|p| {
            if p.is::<HTMLOptGroupElement>() {
                p.upcast::<Node>().GetParentNode()
            } else {
                Some(p)
            }
        });

        parent.and_then(|p| p.downcast::<HTMLSelectElement>().and_then(|s| s.GetForm()))
    }

    // https://html.spec.whatwg.org/multipage/#attr-option-value
    fn Value(&self) -> DOMString {
        let element = self.upcast::<Element>();
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
        let element = self.upcast::<Element>();
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
        self.update_select_validity();
    }

    // https://html.spec.whatwg.org/multipage/#dom-option-index
    fn Index(&self) -> i32 {
        self.index()
    }
}

impl VirtualMethods for HTMLOptionElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match *attr.local_name() {
            local_name!("disabled") => {
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
                    },
                }
                self.update_select_validity();
            },
            local_name!("selected") => {
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
                self.update_select_validity();
            },
            _ => {},
        }
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }

        self.upcast::<Element>()
            .check_parent_disabled_state_for_option();

        self.pick_if_selected_and_reset();
        self.update_select_validity();
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);

        if let Some(select) = context
            .parent
            .inclusive_ancestors(ShadowIncluding::No)
            .filter_map(DomRoot::downcast::<HTMLSelectElement>)
            .next()
        {
            select
                .validity_state()
                .perform_validation_and_update(ValidationFlags::all());
            select.ask_for_reset();
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
