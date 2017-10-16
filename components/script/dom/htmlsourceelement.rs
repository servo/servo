/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLSourceElementBinding;
use dom::bindings::codegen::Bindings::NodeBinding::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::htmlmediaelement::HTMLMediaElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};

#[dom_struct]
pub struct HTMLSourceElement {
    htmlelement: HTMLElement
}

impl HTMLSourceElement {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document) -> HTMLSourceElement {
        HTMLSourceElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document) -> DomRoot<HTMLSourceElement> {
        Node::reflect_node(Box::new(HTMLSourceElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLSourceElementBinding::Wrap)
    }
}

impl VirtualMethods for HTMLSourceElement {
    fn super_type(&self) -> Option<&VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &VirtualMethods)
    }

    /// https://html.spec.whatwg.org/multipage/#the-source-element:nodes-are-inserted
    fn bind_to_tree(&self, tree_in_doc: bool) {
        self.super_type().unwrap().bind_to_tree(tree_in_doc);
        let parent = self.upcast::<Node>().GetParentNode().unwrap();
        if let Some(media) = parent.downcast::<HTMLMediaElement>() {
            media.handle_source_child_insertion();
        }
    }
}
