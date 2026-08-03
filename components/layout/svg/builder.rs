/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use html5ever::local_name;
use layout_api::{LayoutElement, LayoutNode};
use script::layout_dom::{ServoLayoutElement, ServoLayoutNode};
use svg_engine::render_tree::*;
use svg_engine::shapes::Shape;
use web_atoms::ns;

use crate::context::LayoutContext;

pub(crate) struct SvgRenderTreeBuilder<'dom, 'a> {
    root_node: ServoLayoutNode<'dom>,
    _context: &'a LayoutContext<'a>,
    // TODO: css_rules
}

impl<'dom, 'a> SvgRenderTreeBuilder<'dom, 'a> {
    pub(crate) fn new(node: ServoLayoutNode<'dom>, context: &'a LayoutContext<'a>) -> Self {
        SvgRenderTreeBuilder {
            root_node: node,
            _context: context,
        }
    }

    pub(crate) fn build(self) -> Option<Arc<SvgRenderTree>> {
        let root = self.build_render_node(self.root_node)?;
        // TODO: extract viewport info, gradients, clip_paths, patterns, masks, filters

        let tree = SvgRenderTree {
            root,
            // TODO: viewport, gradients, clip_paths, patterns, masks, filters
        };

        Some(Arc::new(tree))
    }

    fn build_render_node(&self, node: ServoLayoutNode<'dom>) -> Option<SvgRenderNode> {
        let element = node.as_element()?;
        let tag = build_tag(node)?;
        let id = extract_id(&element);
        // TODO: implement style from computed values + presentation attributes

        let children: Vec<SvgRenderNode> = node
            .dom_children()
            .filter_map(|child| self.build_render_node(child))
            .collect();

        Some(SvgRenderNode {
            id,
            tag,
            // TODO: style, transforms
            children,
        })
    }
}

// ======================= Tag Dispatch =======================

fn build_tag(node: ServoLayoutNode) -> Option<SvgTag> {
    let element = node.as_element()?;
    let tag = element.local_name().as_ref();
    match tag {
        "svg" => Some(SvgTag::Container(Container::Svg)),
        "g" => Some(SvgTag::Container(Container::Group)),
        // TODO: defs, use, symbol, image, text, tspan
        _ => build_shape(tag).map(SvgTag::Shape),
    }
}

fn build_shape(tag_name: &str) -> Option<Shape> {
    use svg_engine::shapes::*;
    match tag_name {
        // TODO: Parse Geometry for each shape type
        "rect" => Some(Shape::Rect(Rectangle {})),
        "circle" => Some(Shape::Circle(Circle {})),
        "ellipse" => Some(Shape::Ellipse(Ellipse {})),
        "line" => Some(Shape::Line(Line {})),
        "polyline" => Some(Shape::Polyline(Polyline {})),
        "polygon" => Some(Shape::Polygon(Polygon {})),
        "path" => Some(Shape::Path(Path {})),
        _ => None,
    }
}

// ======================= Helpers =======================

fn extract_id(element: &ServoLayoutElement) -> Option<String> {
    element
        .attribute_as_str(&ns!(), &local_name!("id"))
        .map(|s| s.to_string())
}
