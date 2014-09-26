/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLOptionElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLScriptElementDerived};
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::js::{JSRef, Temporary};
use dom::bindings::utils::{Reflectable, Reflector};
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::element::{AttributeHandlers, Element, HTMLOptionElementTypeId};
use dom::eventtarget::{EventTarget, NodeTargetTypeId};
use dom::htmlelement::HTMLElement;
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, ElementNodeTypeId};
use dom::virtualmethods::VirtualMethods;

use servo_util::atom::Atom;
use servo_util::namespace;
use servo_util::str::{DOMString, split_html_space_chars};

#[jstraceable]
#[must_root]
pub struct HTMLOptionElement {
    pub htmlelement: HTMLElement
}

impl HTMLOptionElementDerived for EventTarget {
    fn is_htmloptionelement(&self) -> bool {
        self.type_id == NodeTargetTypeId(ElementNodeTypeId(HTMLOptionElementTypeId))
    }
}

impl HTMLOptionElement {
    fn new_inherited(localName: DOMString, document: JSRef<Document>) -> HTMLOptionElement {
        HTMLOptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLOptionElementTypeId, localName, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, document: JSRef<Document>) -> Temporary<HTMLOptionElement> {
        let element = HTMLOptionElement::new_inherited(localName, document);
        Node::reflect_node(box element, document, HTMLOptionElementBinding::Wrap)
    }
}

fn collect_text(node: &JSRef<Node>, value: &mut DOMString) {
    let elem: JSRef<Element> = ElementCast::to_ref(*node).unwrap();
    let svg_script = elem.namespace == namespace::SVG && elem.local_name.as_slice() == "script";
    let html_script = node.is_htmlscriptelement();
    if svg_script || html_script {
        return;
    } else {
        for child in node.children() {
            if child.is_text() {
                let characterdata: JSRef<CharacterData> = CharacterDataCast::to_ref(child).unwrap();
                value.push_str(characterdata.Data().as_slice());
            } else {
                collect_text(&child, value);
            }
        }
    }
}

impl<'a> HTMLOptionElementMethods for JSRef<'a, HTMLOptionElement> {
    // http://www.whatwg.org/html/#dom-option-disabled
    make_bool_getter!(Disabled)

    // http://www.whatwg.org/html/#dom-option-disabled
    fn SetDisabled(self, disabled: bool) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_bool_attribute("disabled", disabled)
    }

    // http://www.whatwg.org/html/#dom-option-text
    fn Text(self) -> DOMString {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        let mut content = String::new();
        collect_text(&node, &mut content);
        let v: Vec<&str> = split_html_space_chars(content.as_slice()).collect();
        v.connect(" ")
    }

    // http://www.whatwg.org/html/#dom-option-text
    fn SetText(self, value: DOMString) {
        let node: JSRef<Node> = NodeCast::from_ref(self);
        node.SetTextContent(Some(value))
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLOptionElement> {
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
                node.check_parent_disabled_state_for_option();
            },
            _ => ()
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.bind_to_tree(tree_in_doc),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        node.check_parent_disabled_state_for_option();
    }

    fn unbind_from_tree(&self, tree_in_doc: bool) {
        match self.super_type() {
            Some(ref s) => s.unbind_from_tree(tree_in_doc),
            _ => (),
        }

        let node: JSRef<Node> = NodeCast::from_ref(*self);
        if node.parent_node().is_some() {
            node.check_parent_disabled_state_for_option();
        } else {
            node.check_disabled_attribute();
        }
    }
}

impl Reflectable for HTMLOptionElement {
    fn reflector<'a>(&'a self) -> &'a Reflector {
        self.htmlelement.reflector()
    }
}
