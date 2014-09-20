/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding;
use dom::bindings::codegen::Bindings::HTMLFieldSetElementBinding::HTMLFieldSetElementMethods;
use dom::bindings::codegen::InheritTypes::{ElementCast, HTMLFieldSetElementDerived, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLLegendElementDerived};
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::document::Document;
use dom::element::{AttributeHandlers, Element, HTMLFieldSetElementTypeId, HTMLButtonElementTypeId};
use dom::element::{HTMLInputElementTypeId, HTMLSelectElementTypeId, HTMLTextAreaElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlcollection::{HTMLCollection, CollectionFilter};
use dom::htmlelement::HTMLElement;
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, ElementNodeTypeId, window_from_node};
use dom::validitystate::ValidityState;
use dom::virtualmethods::VirtualMethods;

use servo_util::atom::Atom;
use servo_util::str::{DOMString, StaticStringVec};

#[deriving(Encodable)]
#[must_root]
pub struct HTMLFieldSetElement {
    pub htmlelement: HTMLElement
}

impl HTMLFieldSetElementDerived for EventTarget {
    fn is_htmlfieldsetelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLFieldSetElementTypeId))
    }
}

impl HTMLFieldSetElement {
    pub fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLFieldSetElement {
        HTMLFieldSetElement {
            htmlelement: HTMLElement::new_inherited(HTMLFieldSetElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLFieldSetElement> {
        let element = HTMLFieldSetElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLFieldSetElementBinding::Wrap)
    }
}

impl<'a> HTMLFieldSetElementMethods for JSRef<'a, HTMLFieldSetElement> {
    // http://www.whatwg.org/html/#dom-fieldset-elements
    fn Elements(self) -> Temporary<HTMLCollection> {
        struct ElementsFilter;
        impl CollectionFilter for ElementsFilter {
            fn filter(&self, elem: JSRef<Element>, root: JSRef<Node>) -> bool {
                static tag_names: StaticStringVec = &["button", "fieldset", "input",
                    "keygen", "object", "output", "select", "textarea"];
                let root: JSRef<Element> = ElementCast::to_ref(root).unwrap();
                elem != root && tag_names.iter().any(|&tag_name| tag_name == elem.deref().local_name.as_slice())
            }
        }
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let filter = box ElementsFilter;
        let window = window_from_node(node).root();
        HTMLCollection::create(*window, node, filter)
    }

    fn Validity(self) -> Temporary<ValidityState> {
        let window = window_from_node(self).root();
        ValidityState::new(*window)
    }

    // http://www.whatwg.org/html/#dom-fieldset-disabled
    make_bool_getter!(Disabled)

    // http://www.whatwg.org/html/#dom-fieldset-disabled
    fn SetDisabled(self, disabled: bool) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_bool_attribute("disabled", disabled)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLFieldSetElement> {
    fn super_type<'a>(&'a self) -> Option<&'a VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(name, value.clone()),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match name.as_slice() {
            "disabled" => {
                node.set_disabled_state(true);
                node.set_enabled_state(false);
                let maybe_legend = node.children().find(|node| node.is_htmllegendelement());
                let filtered: Vec<JSRef<Node>> = node.children().filter(|child| {
                    maybe_legend.map_or(true, |legend| legend != *child)
                }).collect();
                for descendant in filtered.iter().flat_map(|child| child.traverse_preorder()) {
                    match descendant.type_id() {
                        ElementNodeTypeId(HTMLButtonElementTypeId) |
                        ElementNodeTypeId(HTMLInputElementTypeId) |
                        ElementNodeTypeId(HTMLSelectElementTypeId) |
                        ElementNodeTypeId(HTMLTextAreaElementTypeId) => {
                            descendant.set_disabled_state(true);
                            descendant.set_enabled_state(false);
                        },
                        _ => ()
                    }
                }
            },
            _ => ()
        }
    }

    fn before_remove_attr(&self, name: &Atom, value: DOMString) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(name, value),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        match name.as_slice() {
            "disabled" => {
                node.set_disabled_state(false);
                node.set_enabled_state(true);
                let maybe_legend = node.children().find(|node| node.is_htmllegendelement());
                let filtered: Vec<JSRef<Node>> = node.children().filter(|child| {
                    maybe_legend.map_or(true, |legend| legend != *child)
                }).collect();
                for descendant in filtered.iter().flat_map(|child| child.traverse_preorder()) {
                    match descendant.type_id() {
                        ElementNodeTypeId(HTMLButtonElementTypeId) |
                        ElementNodeTypeId(HTMLInputElementTypeId) |
                        ElementNodeTypeId(HTMLSelectElementTypeId) |
                        ElementNodeTypeId(HTMLTextAreaElementTypeId) => {
                            descendant.check_disabled_attribute();
                            descendant.check_ancestors_disabled_state_for_form_control();
                        },
                        _ => ()
                    }
                }
            },
            _ => ()
        }
    }
}

impl Reflectable for HTMLFieldSetElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
