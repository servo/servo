/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use dom::attr::Attr;
use dom::bindings::codegen::Bindings::HTMLSourceElementBinding;
use dom::bindings::codegen::Bindings::HTMLSourceElementBinding::HTMLSourceElementMethods;
use dom::bindings::codegen::Bindings::NodeBinding::NodeBinding::NodeMethods;
use dom::bindings::inheritance::Castable;
use dom::bindings::root::{Dom, Root};
use dom::bindings::root::DomRoot;
use dom::bindings::str::DOMString;
use dom::document::Document;
use dom::element::AttributeMutation;
use dom::htmlelement::HTMLElement;
use dom::htmlimageelement::HTMLImageElement;
use dom::htmlmediaelement::HTMLMediaElement;
use dom::node::{Node, UnbindContext};
use dom::virtualmethods::VirtualMethods;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix};
use typeholder::TypeHolderTrait;

#[dom_struct]
pub struct HTMLSourceElement<TH: TypeHolderTrait> {
    htmlelement: HTMLElement<TH>,
}

impl<TH: TypeHolderTrait> HTMLSourceElement<TH> {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document<TH>,
    ) -> HTMLSourceElement<TH> {
        HTMLSourceElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document<TH>,
    ) -> DomRoot<HTMLSourceElement<TH>> {
        Node::<TH>::reflect_node(
            Box::new(HTMLSourceElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            HTMLSourceElementBinding::Wrap,
        )
    }

    fn iterate_next_html_image_element_siblings(next_siblings_iterator: impl Iterator<Item=Root<Dom<Node<TH>>>>) {
        for next_sibling in next_siblings_iterator {
            if let Some(html_image_element_sibling) = next_sibling.downcast::<HTMLImageElement<TH>>() {
                html_image_element_sibling.update_the_image_data();
            }
        }
    }
}

impl<TH: TypeHolderTrait> VirtualMethods<TH> for HTMLSourceElement<TH> {
    fn super_type(&self) -> Option<&VirtualMethods<TH>> {
        Some(self.upcast::<HTMLElement<TH>>() as &VirtualMethods<TH>)
    }

    fn attribute_mutated(&self, attr: &Attr<TH>, mutation: AttributeMutation<TH>) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("srcset") | &local_name!("sizes")  |
            &local_name!("media") | &local_name!("type") => {
                let next_sibling_iterator = self.upcast::<Node<TH>>().following_siblings();
                HTMLSourceElement::iterate_next_html_image_element_siblings(next_sibling_iterator);
            },
            _ => {},
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-source-element:nodes-are-inserted>
    fn bind_to_tree(&self, tree_in_doc: bool) {
        self.super_type().unwrap().bind_to_tree(tree_in_doc);
        let parent = self.upcast::<Node<TH>>().GetParentNode().unwrap();
        if let Some(media) = parent.downcast::<HTMLMediaElement<TH>>() {
            media.handle_source_child_insertion();
        }
        let next_sibling_iterator = self.upcast::<Node<TH>>().following_siblings();
        HTMLSourceElement::iterate_next_html_image_element_siblings(next_sibling_iterator);
    }

    fn unbind_from_tree(&self, context: &UnbindContext<TH>) {
        self.super_type().unwrap().unbind_from_tree(context);
        if let Some(next_sibling) = context.next_sibling {
            let next_sibling_iterator = next_sibling.inclusively_following_siblings();
            HTMLSourceElement::iterate_next_html_image_element_siblings(next_sibling_iterator);
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
