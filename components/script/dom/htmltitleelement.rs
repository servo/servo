/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::bindings::codegen::Bindings::HTMLTitleElementBinding::HTMLTitleElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::DomRoot;
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::node::{BindContext, ChildrenMutation, Node};
use crate::dom::virtualmethods::VirtualMethods;

#[dom_struct]
pub struct HTMLTitleElement {
    htmlelement: HTMLElement,
}

impl HTMLTitleElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLTitleElement {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLTitleElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLTitleElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }
}

impl HTMLTitleElementMethods for HTMLTitleElement {
    // https://html.spec.whatwg.org/multipage/#dom-title-text
    fn Text(&self) -> DOMString {
        self.upcast::<Node>().child_text_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-title-text
    fn SetText(&self, value: DOMString) {
        self.upcast::<Node>().SetTextContent(Some(value))
    }
}

impl VirtualMethods for HTMLTitleElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn children_changed(&self, mutation: &ChildrenMutation) {
        if let Some(s) = self.super_type() {
            s.children_changed(mutation);
        }
        let node = self.upcast::<Node>();
        if node.is_in_doc() {
            node.owner_doc().title_changed();
        }
    }

    fn bind_to_tree(&self, context: &BindContext) {
        if let Some(s) = self.super_type() {
            s.bind_to_tree(context);
        }
        let node = self.upcast::<Node>();
        if context.tree_in_doc {
            node.owner_doc().title_changed();
        }
    }
}
