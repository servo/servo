/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::bindings::codegen::Bindings::HTMLSourceElementBinding;
use dom::bindings::codegen::Bindings::HTMLSourceElementBinding::HTMLSourceElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::htmlelement::HTMLElement;
use dom::htmlmediaelement::HTMLMediaElement;
use dom::node::Node;
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLSourceElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>
}

impl<TH: TypeHolderTrait> HTMLSourceElement<TH> {
    fn new_inherited(local_name: LocalName,
                     prefix: Option<Prefix>,
                     document: &Document<TH>) -> HTMLSourceElement<TH> {
        HTMLSourceElement {
            htmlelement:
                HTMLElement::new_inherited(local_name, prefix, document)
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(local_name: LocalName,
               prefix: Option<Prefix>,
               document: &Document<TH>) -> DomRoot<HTMLSourceElement<TH>> {
        Node::<TH>::reflect_node(Box::new(HTMLSourceElement::new_inherited(local_name, prefix, document)),
                           document,
                           HTMLSourceElementBinding::Wrap)
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLSourceElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    /// <https://html.spec.whatwg.org/multipage/#the-source-element:nodes-are-inserted>
    fn bind_to_tree(&self, tree_in_doc: bool) {
        self.super_type().unwrap().bind_to_tree(tree_in_doc);
        let parent = self.upcast::<Node<TH>>().GetParentNode().unwrap();
        if let Some(media) = parent.downcast::<HTMLMediaElement<TH>>() {
            media.handle_source_child_insertion();
        }
    }
}

impl<TH: TypeHolderTrait> HTMLSourceElementMethods for HTMLSourceElement<TH> {
    // https://html.spec.whatwg.org/multipage/#dom-source-src
    make_getter!(Src, "src");

    // https://html.spec.whatwg.org/multipage/#dom-source-src
    make_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-source-type
    make_getter!(Type, "type");

    // https://html.spec.whatwg.org/multipage/#dom-source-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-source-srcset
    make_getter!(Srcset, "srcset");

    // https://html.spec.whatwg.org/multipage/#dom-source-srcset
    make_setter!(SetSrcset, "srcset");

    // https://html.spec.whatwg.org/multipage/#dom-source-sizes
    make_getter!(Sizes, "sizes");

    // https://html.spec.whatwg.org/multipage/#dom-source-sizes
    make_setter!(SetSizes, "sizes");

    // https://html.spec.whatwg.org/multipage/#dom-source-media
    make_getter!(Media, "media");

    // https://html.spec.whatwg.org/multipage/#dom-source-media
    make_setter!(SetMedia, "media");

}
