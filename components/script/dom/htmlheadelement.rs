/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLHeadElementBinding;
use dom::bindings::inheritance::Castable;
use dom::bindings::js::Root;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::Node;
use dom::userscripts::load_script;
use dom::virtualmethods::VirtualMethods;
use string_cache::Atom;
use util::str::DOMString;

#[dom_struct]
pub struct HTMLHeadElement {
    htmlelement: HTMLElement
}

impl HTMLHeadElement {
    fn new_inherited(localName: Atom,
                     prefix: Option<DOMString>,
                     document: &Document) -> HTMLHeadElement {
        HTMLHeadElement {
            htmlelement: HTMLElement::new_inherited(localName, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(localName: Atom,
               prefix: Option<DOMString>,
               document: &Document) -> Root<HTMLHeadElement> {
        let element = HTMLHeadElement::new_inherited(localName, prefix, document);
        Node::reflect_node(box element, document, HTMLHeadElementBinding::Wrap)
    }
}

impl VirtualMethods for HTMLHeadElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }
    fn bind_to_tree(&self, _tree_in_doc: bool) {
        load_script(self);
    }
}
