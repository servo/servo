/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLTitleElementBinding;
use dom::bindings::codegen::Bindings::HTMLTitleElementBinding::HTMLTitleElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::node::{ChildrenMutation, Node};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLTitleElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
}

impl<TH: TypeHolderTrait> HTMLTitleElement<TH> {
    fn new_inherited(local_name: LocalName, prefix: Option<Prefix>, document: &Document<TH>) -> HTMLTitleElement<TH> {
        HTMLTitleElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLTitleElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLTitleElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLTitleElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> HTMLTitleElementMethods for HTMLTitleElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-title-text
    fn Text(&self) -> DOMString {
        self.upcast::<Node<TH>>().child_text_content()
    }

    // https://html.spec.whatwg.org/multipage/#dom-title-text
    fn SetText(&self, value: DOMString) {
        self.upcast::<Node<TH>>().SetTextContent(Some(value))
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLTitleElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn children_changed(&self, mutation: &ChildrenMutation<TH>) {
        if let Some(ref s) = self.super_type() {
            s.children_changed(mutation);
        }
        let node = self.upcast::<Node<TH>>();
        if node.is_in_doc() {
            node.owner_doc().title_changed();
        }
    }

    fn bind_to_tree(&self, tree_in_doc: bool) {
        if let Some(ref s) = self.super_type() {
            s.bind_to_tree(tree_in_doc);
        }
        let node = self.upcast::<Node<TH>>();
        if tree_in_doc {
            node.owner_doc().title_changed();
        }
    }
}
