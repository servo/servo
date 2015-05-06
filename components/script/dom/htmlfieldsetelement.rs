/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding::HTMLFieldSetElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLFieldSetElementDerived, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLLegendElementDerived};
use dom::bindings::js::{JS, MutNullableHeap, Root, RootedReference};
use dom::document::Document;
use dom::element::{AttributeHandlers, Element, ElementHelpers};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::htmlformelement::{HTMLFormElement, FormControl};
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, NodeTypeId, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;

use string_cache::Atom;
use util::str::{DOMString, StaticStringVec};

use std::default::Default;

#[dom_struct]
pub struct HTMLFieldSetElement {
    htmlelement: HTMLElement,
    form_owner: MutNullableHeap<JS<HTMLFormElement>>,
}

impl HTMLFieldSetElementDerived for EventTarget {
    fn is_htmlfieldsetelement(&self) -> bool {
        *self.type_id() ==
            EventTargetTypeId::Node(
                NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLFieldSetElement)))
    }
}

impl HTMLFieldSetElement {
    fn new_inherited(localName: DOMString,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement:
                HTMLElement::new_inherited(HTMLElementTypeId::HTMLFieldSetElement, localName, prefix, document),
            form_owner: Default::default(),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLFieldSetElement> {
        let element = HTMLFieldSetElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLFieldSetElementBinding::Wrap)
    }
}

impl<'a> HTMLFieldSetElementMethods for &'a HTMLFieldSetElement {
    // https://www.whatwg.org/html/#dom-fieldset-elements
    fn Elements(self) -> Root<HTMLCollection> {
        #[derive(JSTraceable)]
        struct ElementsFilter;
        impl CollectionFilter for ElementsFilter {
            fn filter<'a>(&self, elem: &'a Element, _root: &'a Node) -> bool {
                static TAG_NAMES: StaticStringVec = &["button", "fieldset", "input",
                    "keygen", "object", "output", "select", "textarea"];
                TAG_NAMES.iter().any(|&tag_name| tag_name == &**elem.local_name())
            }
        }
        let node = NodeCast::from_ref(self);
        let filter = box ElementsFilter;
        let window = window_from_node(node);
        HTMLCollection::create(window.r(), node, filter)
    }

    fn Validity(self) -> Root<ValidityState> {
        let window = window_from_node(self);
        ValidityState::new(window.r())
    }

    // https://html.spec.whatwg.org/multipage/#dom-fae-form
    fn GetForm(self) -> Option<Root<HTMLFormElement>> {
        self.form_owner()
    }

    // https://www.whatwg.org/html/#dom-fieldset-disabled
    make_bool_getter!(Disabled);

    // https://www.whatwg.org/html/#dom-fieldset-disabled
    make_bool_setter!(SetDisabled, "disabled");
}

impl<'a> VirtualMethods for &'a HTMLFieldSetElement {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &&HTMLElement = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.after_set_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node = NodeCast::from_ref(*self);
                node.set_disabled_state(true);
                node.set_enabled_state(false);
                let maybe_legend = node.children()
                                       .find(|node| node.r().is_htmllegendelement());

                for child in node.children() {
                    if Some(child.r()) == maybe_legend.r() {
                        continue;
                    }

                    for descendant in child.r().traverse_preorder() {
                        match descendant.r().type_id() {
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                                descendant.r().set_disabled_state(true);
                                descendant.r().set_enabled_state(false);
                            },
                            _ => ()
                        }
                    }
                }
            },
            &atom!("form") => {
                self.after_set_form_attr();
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: &Attr) {
        if let Some(ref s) = self.super_type() {
            s.before_remove_attr(attr);
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node = NodeCast::from_ref(*self);
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                let maybe_legend = node.children()
                                       .find(|node| node.r().is_htmllegendelement());

                for child in node.children() {
                    if Some(child.r()) == maybe_legend.r() {
                        continue;
                    }

                    for descendant in child.r().traverse_preorder() {
                        match descendant.r().type_id() {
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLButtonElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLInputElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLSelectElement)) |
                            NodeTypeId::Element(
                                    ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLTextAreaElement)) => {
                                descendant.r().check_disabled_attribute();
                                descendant.r().check_ancestors_disabled_state_for_form_control();
                            },
                            _ => ()
                        }
                    }
                }
            },
            &atom!("form") => {
                self.before_remove_form_attr();
            },
            _ => ()
        }
    }

    fn after_remove_attr(&self, attr: &Atom) {
        if let Some(ref s) = self.super_type() {
            s.after_remove_attr(attr);
        }

        match attr {
            &atom!("form") => {
                self.after_remove_form_attr();
            }
            _ => ()
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }

        self.bind_form_control_to_tree();
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.unbind_from_tree(tree_in_doc);
        }

        self.unbind_form_control_from_tree();
    }
}

impl<'a> FormControl for &'a HTMLFieldSetElement {
    fn form_owner(&self) -> Option<Root<HTMLFormElement>> {
        self.form_owner.get().map(Root::from_rooted)
    }

    fn set_form_owner(&self, form: Option<&HTMLFormElement>) {
        self.form_owner.set(form.map(JS::from_ref));
    }

    fn to_element<'b>(&'b self) -> &'b Element {
        ElementCast::from_ref(*self)
    }
}
