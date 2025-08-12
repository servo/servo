// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cell::RefCell;

use base64::Engine as _;
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use layout_api::SVGElementData;
use servo_url::ServoUrl;
use style::attr::{AttrValue, parse_integer, parse_unsigned_integer};
use style::str::char_is_whitespace;
use xml5ever::serialize::TraversalScope;

use crate::dom::attr::Attr;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, LayoutElementHelpers};
use crate::dom::node::{Node, NodeDamage};
use crate::dom::svggraphicselement::SVGGraphicsElement;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct SVGSVGElement {
    svggraphicselement: SVGGraphicsElement,
    // The XML source of subtree rooted at this SVG element, serialized into
    // a base64 encoded `data:` url. This is cached to avoid recomputation
    // on each layout and must be invalidated when the subtree changes.
    #[no_trace]
    cached_serialized_data_url: RefCell<Option<Result<ServoUrl, ()>>>,
}

impl SVGSVGElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGSVGElement {
        SVGSVGElement {
            svggraphicselement: SVGGraphicsElement::new_inherited(local_name, prefix, document),
            cached_serialized_data_url: Default::default(),
        }
    }

    #[cfg_attr(crown, allow(crown::unrooted_must_root))]
    pub(crate) fn new(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
        proto: Option<HandleObject>,
        can_gc: CanGc,
    ) -> DomRoot<SVGSVGElement> {
        Node::reflect_node_with_proto(
            Box::new(SVGSVGElement::new_inherited(local_name, prefix, document)),
            document,
            proto,
            can_gc,
        )
    }

    pub(crate) fn serialize_and_cache_subtree(&self) {
        let Ok(xml_source) = self
            .upcast::<Node>()
            .xml_serialize(TraversalScope::IncludeNode)
        else {
            *self.cached_serialized_data_url.borrow_mut() = Some(Err(()));
            return;
        };

        let xml_source: String = xml_source.into();
        let base64_encoded_source = base64::engine::general_purpose::STANDARD.encode(xml_source);
        let data_url = format!("data:image/svg+xml;base64,{}", base64_encoded_source);
        match ServoUrl::parse(&data_url) {
            Ok(url) => *self.cached_serialized_data_url.borrow_mut() = Some(Ok(url)),
            Err(error) => error!("Unable to parse serialized SVG data url: {error}"),
        };
    }

    fn invalidate_cached_serialized_subtree(&self) {
        *self.cached_serialized_data_url.borrow_mut() = None;
        self.upcast::<Node>().dirty(NodeDamage::Other);
    }
}

pub(crate) trait LayoutSVGSVGElementHelpers {
    fn data(self) -> SVGElementData;
}

fn ratio_from_view_box(view_box: &AttrValue) -> Option<f32> {
    let mut iter = view_box.chars();
    let _min_x = parse_integer(&mut iter).ok()?;
    let _min_y = parse_integer(&mut iter).ok()?;
    let width = parse_unsigned_integer(&mut iter).ok()?;
    if width == 0 {
        return None;
    }
    let height = parse_unsigned_integer(&mut iter).ok()?;
    if height == 0 {
        return None;
    }
    let mut iter = iter.skip_while(|c| char_is_whitespace(*c));
    iter.next().is_none().then(|| width as f32 / height as f32)
}

impl LayoutSVGSVGElementHelpers for LayoutDom<'_, SVGSVGElement> {
    fn data(self) -> SVGElementData {
        let element = self.upcast::<Element>();
        let get_size = |attr| {
            element
                .get_attr_for_layout(&ns!(), &attr)
                .map(|val| val.as_int())
                .filter(|val| *val >= 0)
        };
        let width = get_size(local_name!("width"));
        let height = get_size(local_name!("height"));
        let ratio = match (width, height) {
            (Some(width), Some(height)) if width != 0 && height != 0 => {
                Some(width as f32 / height as f32)
            },
            _ => element
                .get_attr_for_layout(&ns!(), &local_name!("viewBox"))
                .and_then(ratio_from_view_box),
        };
        SVGElementData {
            source: self
                .unsafe_get()
                .cached_serialized_data_url
                .borrow()
                .clone(),
            width,
            height,
            ratio,
        }
    }
}

impl VirtualMethods for SVGSVGElement {
    fn super_type(&self) -> Option<&dyn VirtualMethods> {
        Some(self.upcast::<SVGGraphicsElement>() as &dyn VirtualMethods)
    }

    fn attribute_mutated(&self, attr: &Attr, mutation: AttributeMutation, can_gc: CanGc) {
        self.super_type()
            .unwrap()
            .attribute_mutated(attr, mutation, can_gc);

        self.invalidate_cached_serialized_subtree();
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            // TODO: This should accept lengths in arbitrary units instead of assuming px.
            local_name!("width") | local_name!("height") => AttrValue::from_i32(value.into(), -1),
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn children_changed(&self, mutation: &super::node::ChildrenMutation) {
        if let Some(super_type) = self.super_type() {
            super_type.children_changed(mutation);
        }

        self.invalidate_cached_serialized_subtree();
    }
}
