/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::attr::AttrHelpers;
use dom::bindings::codegen::Bindings::CharacterDataBinding::CharacterDataMethods;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding;
use dom::bindings::codegen::Bindings::HTMLOptionElementBinding::HTMLOptionElementMethods;
use dom::bindings::codegen::InheritTypes::{CharacterDataCast, ElementCast, HTMLElementCast, NodeCast};
use dom::bindings::codegen::InheritTypes::{HTMLOptionElementDerived};
use dom::bindings::codegen::InheritTypes::{HTMLScriptElementDerived};
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::js::{JSRef, Temporary};
use dom::characterdata::CharacterData;
use dom::document::Document;
use dom::element::{AttributeHandlers, Element, ElementHelpers};
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{DisabledStateHelpers, Node, NodeHelpers, NodeTypeId};
use dom::virtualmethods::VirtualMethods;

use util::str::{DOMString, split_html_space_chars};
use string_cache::Atom;

#[dom_struct]
pub struct HTMLOptionElement {
    htmlelement: HTMLElement
}

impl HTMLOptionElementDerived for EventTarget {
    fn is_htmloptionelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLOptionElement)))
    }
}

impl HTMLOptionElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLOptionElement {
        HTMLOptionElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLOptionElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLOptionElement> {
        let element = HTMLOptionElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLOptionElementBinding::Wrap)
    }
}

fn collect_text(node: &JSRef<Node>, value: &mut DOMString) {
    let elem: JSRef<Element> = ElementCast::to_ref(*node).unwrap();
    let svg_script = *elem.namespace() == ns!(SVG) && elem.local_name().as_slice() == "script";
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
    make_bool_getter!(Disabled);

    // http://www.whatwg.org/html/#dom-option-disabled
    fn SetDisabled(self, disabled: bool) {
        let elem: JSRef<Element> = ElementCast::from_ref(self);
        elem.set_bool_attribute(&atom!("disabled"), disabled)
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

    // https://html.spec.whatwg.org/multipage/forms.html#attr-option-value
    fn Value(self) -> DOMString {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        let attr = &atom!("value");
        if element.has_attribute(attr) {
            element.get_string_attribute(attr)
        } else {
            self.Text()
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#attr-option-value
    make_setter!(SetValue, "value");

    // https://html.spec.whatwg.org/multipage/forms.html#attr-option-label
    fn Label(self) -> DOMString {
        let element: JSRef<Element> = ElementCast::from_ref(self);
        let attr = &atom!("label");
        if element.has_attribute(attr) {
            element.get_string_attribute(attr)
        } else {
            self.Text()
        }
    }

    // https://html.spec.whatwg.org/multipage/forms.html#attr-option-label
    make_setter!(SetLabel, "label");

}

impl<'a> VirtualMethods for JSRef<'a, HTMLOptionElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }

    fn after_set_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.after_set_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
                node.set_disabled_state(true);
                node.set_enabled_state(false);
            }
            _ => ()
        }
    }

    fn before_remove_attr(&self, attr: JSRef<Attr>) {
        match self.super_type() {
            Some(ref s) => s.before_remove_attr(attr),
            _ => ()
        }

        match attr.local_name() {
            &atom!("disabled") => {
                let node: JSRef<Node> = NodeCast::from_ref(*self);
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

