/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name};
use js::rust::HandleObject;
use style::attr::AttrValue;

use crate::dom::attr::Attr;
use crate::dom::bindings::codegen::Bindings::HTMLSourceElementBinding::HTMLSourceElementMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::Node_Binding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{Dom, DomRoot, Root};
use crate::dom::bindings::str::{DOMString, USVString};
use crate::dom::document::Document;
use crate::dom::element::AttributeMutation;
use crate::dom::html::htmlelement::HTMLElement;
use crate::dom::html::htmlimageelement::HTMLImageElement;
use crate::dom::html::htmlmediaelement::HTMLMediaElement;
use crate::dom::html::htmlpictureelement::HTMLPictureElement;
use crate::dom::node::{BindContext, Node, NodeDamage, UnbindContext};
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
        callback: impl Fn(&HTMLImageElement),
    ) {
        for next_sibling in next_siblings_iterator {
            if let Some(html_image_element_sibling) = next_sibling.downcast::<HTMLImageElement>() {
                callback(html_image_element_sibling);
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
                // <https://html.spec.whatwg.org/multipage/#reacting-to-dom-mutations>
                // The element's parent is a picture element and a source element that is a previous
                // sibling has its srcset, sizes, media, type attributes set, changed, or removed.
                if let Some(parent) = self.upcast::<Node>().GetParentElement() {
                    if parent.is::<HTMLPictureElement>() {
                        let next_sibling_iterator = self.upcast::<Node>().following_siblings();
                        HTMLSourceElement::iterate_next_html_image_element_siblings(
                            next_sibling_iterator,
                            |image| image.update_the_image_data(can_gc),
                        );
                    }
                }
            },
            &local_name!("width") | &local_name!("height") => {
                // Note: Despite being explicitly stated in the specification that any width or
                // height attributes changes (set, changed, removed) of the source element should be
                // counted as relevant mutation for the sibling image element, these attributes
                // affect only the style presentational hints of the image element.
                if let Some(parent) = self.upcast::<Node>().GetParentElement() {
                    if parent.is::<HTMLPictureElement>() {
                        let next_sibling_iterator = self.upcast::<Node>().following_siblings();
                        HTMLSourceElement::iterate_next_html_image_element_siblings(
                            next_sibling_iterator,
                            |image| image.upcast::<Node>().dirty(NodeDamage::Other),
                        );
                    }
                }
            },
            _ => {},
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match name {
            &local_name!("width") | &local_name!("height") => {
                AttrValue::from_dimension(value.into())
            },
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-source-element:html-element-insertion-steps>
    fn bind_to_tree(&self, context: &BindContext, can_gc: CanGc) {
        self.super_type().unwrap().bind_to_tree(context, can_gc);

        // Step 1. Let parent be insertedNode's parent.
        let parent = self.upcast::<Node>().GetParentNode().unwrap();

        // Step 2. If parent is a media element that has no src attribute and whose networkState has
        // the value NETWORK_EMPTY, then invoke that media element's resource selection algorithm.
        if parent.is::<HTMLMediaElement>() && std::ptr::eq(&*parent, context.parent) {
            parent
                .downcast::<HTMLMediaElement>()
                .unwrap()
                .handle_source_child_insertion(self, can_gc);
        }

        // Step 3. If parent is a picture element, then for each child of parent's children, if
        // child is an img element, then count this as a relevant mutation for child.
        if parent.is::<HTMLPictureElement>() && std::ptr::eq(&*parent, context.parent) {
            let next_sibling_iterator = self.upcast::<Node>().following_siblings();
            HTMLSourceElement::iterate_next_html_image_element_siblings(
                next_sibling_iterator,
                |image| image.update_the_image_data(can_gc),
            );
        }
    }

    /// <https://html.spec.whatwg.org/multipage/#the-source-element:html-element-removing-steps>
    fn unbind_from_tree(&self, context: &UnbindContext, can_gc: CanGc) {
        self.super_type().unwrap().unbind_from_tree(context, can_gc);

        // Step 1. If oldParent is a picture element, then for each child of oldParent's children,
        // if child is an img element, then count this as a relevant mutation for child.
        if context.parent.is::<HTMLPictureElement>() && !self.upcast::<Node>().has_parent() {
            if let Some(next_sibling) = context.next_sibling {
                let next_sibling_iterator = next_sibling.inclusively_following_siblings();
                HTMLSourceElement::iterate_next_html_image_element_siblings(
                    next_sibling_iterator,
                    |image| image.update_the_image_data(can_gc),
                );
            }
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

    // <https://html.spec.whatwg.org/multipage/#dom-source-width>
    make_dimension_uint_getter!(Width, "width");

    // <https://html.spec.whatwg.org/multipage/#dom-source-width>
    make_dimension_uint_setter!(SetWidth, "width");

    // <https://html.spec.whatwg.org/multipage/#dom-source-height>
    make_dimension_uint_getter!(Height, "height");

    // <https://html.spec.whatwg.org/multipage/#dom-source-height>
    make_dimension_uint_setter!(SetHeight, "height");
}
