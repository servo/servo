/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLHeadElementBinding;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use dom::bindings::codegen::InheritTypes::{HTMLElementCast, HTMLHeadElementDerived, NodeCast};
use dom::bindings::js::{JSRef, OptionalRootable, Temporary, RootedReference};
use dom::document::Document;
use dom::eventtarget::{EventTarget, EventTargetTypeId};
use dom::element::ElementTypeId;
use dom::element::AttributeHandlers;
use dom::htmlelement::{HTMLElement, HTMLElementTypeId};
use dom::node::{Node, NodeHelpers, NodeTypeId};
use dom::virtualmethods::VirtualMethods;
use util::str::DOMString;
use util::opts;
use util::resource_files::resources_dir_path;
use std::borrow::ToOwned;
use std::fs::read_dir;

#[dom_struct]
pub struct HTMLHeadElement {
    htmlelement: HTMLElement
}

impl HTMLHeadElementDerived for EventTarget {
    fn is_htmlheadelement(&self) -> bool {
        *self.type_id() == EventTargetTypeId::Node(NodeTypeId::Element(ElementTypeId::HTMLElement(HTMLElementTypeId::HTMLHeadElement)))
    }
}

impl HTMLHeadElement {
    fn new_inherited(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> HTMLHeadElement {
        HTMLHeadElement {
            htmlelement: HTMLElement::new_inherited(HTMLElementTypeId::HTMLHeadElement, localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: DOMString, prefix: Option<DOMString>, document: JSRef<Document>) -> Temporary<HTMLHeadElement> {
        let element = HTMLHeadElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLHeadElementBinding::Wrap)
    }
}

impl<'a> VirtualMethods for JSRef<'a, HTMLHeadElement> {
    fn super_type<'b>(&'b self) -> Option<&'b VirtualMethods> {
        let htmlelement: &JSRef<HTMLElement> = HTMLElementCast::from_borrowed_ref(self);
        Some(htmlelement as &VirtualMethods)
    }
    fn bind_to_tree(&self, _tree_in_doc: bool) {
        if !opts::get().userscripts {
            return;
        }

        let node: &JSRef<Node> = NodeCast::from_borrowed_ref(self);
        let first_child = node.GetFirstChild().root();
        let doc = node.owner_doc().root();
        let doc = doc.r();

        let mut path = resources_dir_path();
        path.push("user-agent-js");
        let mut files = match read_dir(&path) {
            Ok(d) => d.filter_map(|e| e.ok()).map(|e| e.path()).collect::<Vec<_>>(),
            Err(_) => return
        };

        files.sort();

        for file in files {
            let name = match file.into_os_string().into_string() {
                Ok(ref s) if s.ends_with(".js") => "file://".to_owned() + &s[..],
                _ => continue
            };
            let new_script = doc.CreateElement("script".to_owned()).unwrap().root();
            let new_script = new_script.r();
            new_script.set_string_attribute(&atom!("src"), name);
            let new_script_node: &JSRef<Node> = NodeCast::from_borrowed_ref(&new_script);
            node.InsertBefore(*new_script_node, first_child.r()).unwrap();
        }
    }
}
