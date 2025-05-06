/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::rust::HandleObject;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLSourceElementBinding::HTMLSourceElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::Document;
use crate::dom::element::AttributeMutation;
use crate::dom::htmlelement::HTMLElement;
use crate::dom::htmlimageelement::HTMLImageElement;
use crate::dom::htmlmediaelement::HTMLMediaElement;
use crate::dom::node::{BindContext, Node, UnbindContext};
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct HTMLSourceElement {
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

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<HTMLSourceElement> {
        Node::reflect_node_with_proto(
            Box::new(HTMLSourceElement::new_inherited(
                local_name, prefix, document,
            )),
            document,
            proto,
            can_gc,
        )
    }

    fn iterate_next_html_image_element_siblings(
        next_siblings_iterator: impl Iterator<Item = Root<Dom<Node>>>,
        can_gc: CanGc,
    ) {
        for next_sibling in next_siblings_iterator {
            if let Some(html_image_element_sibling) = next_sibling.downcast::<HTMLImageElement>() {
                html_image_element_sibling.update_the_image_data(can_gc);
            }
        }
    }
}

impl VirtualMethods for HTMLSourceElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<HTMLElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);
        match attr.local_name() {
            &local_name!("srcset") |
            &local_name!("sizes") |
            &local_name!("media") |
            &local_name!("type") => {
                let next_sibling_iterator = self.upcast::<Node>().following_siblings();
                HTMLSourceElement::iterate_next_html_image_element_siblings(
                    next_sibling_iterator,
                    CanGc::note(),
                );
            },
            _ => {},
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-source-element:nodes-are-inserted>
    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        self.super_type().unwrap().bind_to_tree(context, can_gc);
        let parent = self.upcast::<Node>().GetParentNode().unwrap();
        if let Some(media) = parent.downcast::<HTMLMediaElement>() {
            media.handle_source_child_insertion(CanGc::note());
        }
        let next_sibling_iterator = self.upcast::<Node>().following_siblings();
        HTMLSourceElement::iterate_next_html_image_element_siblings(
            next_sibling_iterator,
            CanGc::note(),
        );
    }

    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);
        if let Some(next_sibling) = context.next_sibling {
            let next_sibling_iterator = next_sibling.inclusively_following_siblings();
            HTMLSourceElement::iterate_next_html_image_element_siblings(
                next_sibling_iterator,
                CanGc::note(),
            );
        }
    }
}

impl HTMLSourceElementMethods<crate::DomTypeHolder> for HTMLSourceElement {
    // https://html.spec.whatwg.org/multipage/#dom-source-src
    make_url_getter!(Src, "src");

    // https://html.spec.whatwg.org/multipage/#dom-source-src
    make_url_setter!(SetSrc, "src");

    // https://html.spec.whatwg.org/multipage/#dom-source-type
    make_getter!(Type, "type");

    // https://html.spec.whatwg.org/multipage/#dom-source-type
    make_setter!(SetType, "type");

    // https://html.spec.whatwg.org/multipage/#dom-source-srcset
    make_url_getter!(Srcset, "srcset");

    // https://html.spec.whatwg.org/multipage/#dom-source-srcset
    make_url_setter!(SetSrcset, "srcset");

    // https://html.spec.whatwg.org/multipage/#dom-source-sizes
    make_getter!(Sizes, "sizes");

    // https://html.spec.whatwg.org/multipage/#dom-source-sizes
    make_setter!(SetSizes, "sizes");

    // https://html.spec.whatwg.org/multipage/#dom-source-media
    make_getter!(Media, "media");

    // https://html.spec.whatwg.org/multipage/#dom-source-media
    make_setter!(SetMedia, "media");
}
