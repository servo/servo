/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{local_name, LocalName, Prefix};
use js::rust::HandleObject;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLSourceElementBinding::HTMLSourceElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::AttributeMutation;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::htmlmediaelement::HTMLMediaElement;
use crate::dom::node::{BindContext, Node, UnbindContext};
use crate::dom::virtualmethods::VirtualMethods;

#[dom_struct]
pub struct HTMLSourceElement {
    htmlelement: HTMLElement,
}

impl HTMLSourceElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> HTMLSourceElement {
        HTMLSourceElement {
            htmlelement: HTMLElement::new_inherited(local_name, prefix, document),
        }
    }

    #[allow(crown::unrooted_must_root)]
    pub fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
    ) -> DomRoot<HTMLSourceElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLSourceElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
        )
    }

    fn iterate_next_html_image_element_siblings(
        next_siblings_iterator: impl Iterator<Item = Root<Dom<Node>>>,
    ) {
        for next_sibling in next_siblings_iterator {
            if let Some(html_image_element_sibling) = next_sibling.downcast::<HTMLImageElement>() {
                html_image_element_sibling.update_the_image_data();
            }
        }
    }
}

impl VirtualMethods for HTMLSourceElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation) {
        self.super_type().unwrap().attribute_mutated(attr, mutation);
        match attr.local_name() {
            &local_name!("srcset") |
            &local_name!("sizes") |
            &local_name!("media") |
            &local_name!("type") => {
                let next_sibling_iterator = self.upcast::<Node>().following_siblings();
                HTMLSourceElement::iterate_next_html_image_element_siblings(next_sibling_iterator);
            },
            _ => {},
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-source-element:nodes-are-inserted>
    fn bind_to_tree(&self, context: &BindContext) {
        self.super_type().unwrap().bind_to_tree(context);
        let parent = self.upcast::<Node>().GetParentNode().unwrap();
        if let Some(media) = parent.downcast::<HTMLMediaElement>() {
            media.handle_source_child_insertion();
        }
        let next_sibling_iterator = self.upcast::<Node>().following_siblings();
        HTMLSourceElement::iterate_next_html_image_element_siblings(next_sibling_iterator);
    }

    fn unbind_from_tree(&self, context: &UnbindContext) {
        self.super_type().unwrap().unbind_from_tree(context);
        if let Some(next_sibling) = context.next_sibling {
            let next_sibling_iterator = next_sibling.inclusively_following_siblings();
            HTMLSourceElement::iterate_next_html_image_element_siblings(next_sibling_iterator);
        }
    }
}

impl HTMLSourceElementMethods for HTMLSourceElement {
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
