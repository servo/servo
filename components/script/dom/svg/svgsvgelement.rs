/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use base64::Engine as _;
use cssparser::{Parser, ParserInput};
use dom_struct::dom_struct;
use html5ever::{LocalName, Prefix, local_name, ns};
use js::rust::HandleObject;
use layout_api::SVGElementData;
use servo_url::ServoUrl;
use style::attr::AttrValue;
use style::parser::ParserContext;
use style::stylesheets::Origin;
use style::values::specified::Length;
use style_traits::ParsingMode;
use uuid::Uuid;
use xml5ever::serialize::TraversalScope;

use crate::dom::attr::Attr;
use crate::dom::bindings::cell::DomRefCell;
use crate::dom::bindings::codegen::Bindings::DocumentBinding::DocumentMethods;
use crate::dom::bindings::codegen::Bindings::NodeBinding::NodeMethods;
use crate::dom::bindings::inheritance::Castable;
use crate::dom::bindings::root::{DomRoot, LayoutDom};
use crate::dom::bindings::str::DOMString;
use crate::dom::document::Document;
use crate::dom::element::{AttributeMutation, Element, LayoutElementHelpers};
use crate::dom::node::{
    ChildrenMutation, CloneChildrenFlag, Node, NodeDamage, NodeTraits, ShadowIncluding
};
use crate::dom::svg::svggraphicselement::SVGGraphicsElement;
use crate::dom::virtualmethods::VirtualMethods;
use crate::script_runtime::CanGc;

#[dom_struct]
pub(crate) struct SVGSVGElement {
    svggraphicselement: SVGGraphicsElement,
    uuid: String,
    // The XML source of subtree rooted at this SVG element, serialized into
    // a base64 encoded `data:` url. This is cached to avoid recomputation
    // on each layout and must be invalidated when the subtree changes.
    #[no_trace]
    cached_serialized_data_url: DomRefCell<Option<Result<ServoUrl, ()>>>,
}

impl SVGSVGElement {
    fn new_inherited(
        local_name: LocalName,
        prefix: Option<Prefix>,
        document: &Document,
    ) -> SVGSVGElement {
        SVGSVGElement {
            svggraphicselement: SVGGraphicsElement::new_inherited(local_name, prefix, document),
            uuid: Uuid::new_v4().to_string(),
            cached_serialized_data_url: Default::default(),
        }
    }

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
        let can_gc = CanGc::note();
        let cloned_nodes = self.process_use_elements(can_gc);

        let serialize_result = self
            .upcast::<Node>()
            .xml_serialize(TraversalScope::IncludeNode);

        self.cleanup_cloned_nodes(&cloned_nodes, can_gc);

        let Ok(xml_source) = serialize_result else {
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

    fn process_use_elements(&self, can_gc: CanGc) -> Vec<DomRoot<Node>> {
        let mut cloned_nodes = Vec::new();
        let root_node = self.upcast::<Node>();

        for node in root_node.traverse_preorder(ShadowIncluding::No) {
            if let Some(element) = node.downcast::<Element>() {
                if element.local_name() == &local_name!("use") {
                    if let Some(cloned) = self.process_single_use_element(element, can_gc) {
                        cloned_nodes.push(cloned);
                    }
                }
            }
        }

        cloned_nodes
    }

    fn process_single_use_element(
        &self,
        use_element: &Element,
        can_gc: CanGc,
    ) -> Option<DomRoot<Node>> {
        let href = use_element.get_string_attribute(&local_name!("href"));
        let href_view = href.str();
        let id_str = href_view.strip_prefix("#")?;
        let id = DOMString::from(id_str);
        let document = self.upcast::<Node>().owner_doc();
        let referenced_element = document.GetElementById(id)?;
        let referenced_node = referenced_element.upcast::<Node>();
        let has_svg_ancestor = referenced_node
            .inclusive_ancestors(ShadowIncluding::No)
            .any(|ancestor| ancestor.is::<SVGSVGElement>());
        if !has_svg_ancestor {
            return None;
        }
        let cloned_node = Node::clone(
            referenced_node,
            None,
            CloneChildrenFlag::CloneChildren,
            None,
            can_gc,
        );
        let root_node = self.upcast::<Node>();
        let _ = root_node.AppendChild(&cloned_node, can_gc);

        Some(cloned_node)
    }

    fn cleanup_cloned_nodes(&self, cloned_nodes: &[DomRoot<Node>], can_gc: CanGc) {
        if cloned_nodes.is_empty() {
            return;
        }
        let root_node = self.upcast::<Node>();

        for cloned_node in cloned_nodes {
            let _ = root_node.RemoveChild(cloned_node, can_gc);
        }
    }

    fn invalidate_cached_serialized_subtree(&self) {
        *self.cached_serialized_data_url.borrow_mut() = None;
        self.upcast::<Node>().dirty(NodeDamage::Other);
    }
}

pub(crate) trait LayoutSVGSVGElementHelpers<'dom> {
    fn data(self) -> SVGElementData<'dom>;
}

impl<'dom> LayoutSVGSVGElementHelpers<'dom> for LayoutDom<'dom, SVGSVGElement> {
    #[expect(unsafe_code)]
    fn data(self) -> SVGElementData<'dom> {
        let svg_id = self.unsafe_get().uuid.clone();
        let element = self.upcast::<Element>();
        let width = element.get_attr_for_layout(&ns!(), &local_name!("width"));
        let height = element.get_attr_for_layout(&ns!(), &local_name!("height"));
        let view_box = element.get_attr_for_layout(&ns!(), &local_name!("viewBox"));
        SVGElementData {
            source: unsafe {
                self.unsafe_get()
                    .cached_serialized_data_url
                    .borrow_for_layout()
                    .clone()
            },
            width,
            height,
            view_box,
            svg_id,
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

    fn attribute_affects_presentational_hints(&self, attr: &Attr) -> bool {
        match attr.local_name() {
            &local_name!("width") | &local_name!("height") => true,
            _ => self
                .super_type()
                .unwrap()
                .attribute_affects_presentational_hints(attr),
        }
    }

    fn parse_plain_attribute(&self, name: &LocalName, value: DOMString) -> AttrValue {
        match *name {
            local_name!("width") | local_name!("height") => {
                let value = &value.str();
                let parser_input = &mut ParserInput::new(value);
                let parser = &mut Parser::new(parser_input);
                let doc = self.owner_document();
                let url = doc.url().into_url().into();
                let context = ParserContext::new(
                    Origin::Author,
                    &url,
                    None,
                    ParsingMode::ALLOW_UNITLESS_LENGTH,
                    doc.quirks_mode(),
                    /* namespaces = */ Default::default(),
                    None,
                    None,
                );
                let val = Length::parse_quirky(
                    &context,
                    parser,
                    style::values::specified::AllowQuirks::Always,
                );
                AttrValue::Length(value.to_string(), val.ok())
            },
            _ => self
                .super_type()
                .unwrap()
                .parse_plain_attribute(name, value),
        }
    }

    fn children_changed(&self, mutation: &ChildrenMutation, can_gc: CanGc) {
        if let Some(super_type) = self.super_type() {
            super_type.children_changed(mutation, can_gc);
        }

        self.invalidate_cached_serialized_subtree();
    }

    fn unbind_from_tree(&self, _context: &UnbindContext<'_>, _can_gc: CanGc) {
        self.owner_window()
            .image_cache()
            .evict_rasterized_image(&self.uuid);
    }
}
