/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::collections::HashMap;
use std::sync::Arc;

use html5ever::local_name;
use layout_api::{LayoutElement, LayoutNode};
use log::debug;
use script::layout_dom::{ServoLayoutElement, ServoLayoutNode};
use servo_arc::Arc as ServoArc;
use servo_svg::document::{SvgTree, ViewportInfo};
use servo_svg::element::{Container, SvgNode, SvgTag};
use servo_svg::style::NodeStyle;
use servo_svg::units::{Id, Length};
use web_atoms::ns;

use crate::context::LayoutContext;
use crate::svg::primitives::geometry::build_shape;

pub(crate) struct SvgTreeBuilder<'dom, 'a> {
    root_node: ServoLayoutNode<'dom>,
    context: &'a LayoutContext<'a>,
    root_width: f32,
    root_height: f32,
}

impl<'dom, 'a> SvgTreeBuilder<'dom, 'a> {
    pub(crate) fn new(
        node: ServoLayoutNode<'dom>,
        context: &'a LayoutContext<'a>,
        root_width: f32,
        root_height: f32,
    ) -> Self {
        SvgTreeBuilder {
            root_node: node,
            context,
            root_width,
            root_height,
        }
    }

    pub(crate) fn build(self) -> Option<Arc<SvgTree>> {
        let root = self.build_node(self.root_node)?;

        let root = Arc::try_unwrap(root).expect("the root `<svg>` is never cached");

        let tree = SvgTree {
            root,
            viewport: ViewportInfo {
                width: Length::new(self.root_width),
                height: Length::new(self.root_height),
                view_box: None,          // TODO
                overflow_visible: false, // TODO
                aspect_ratio: None,      // TODO
            },
            gradients: HashMap::new(),  // TODO
            clip_paths: HashMap::new(), // TODO
            patterns: HashMap::new(),   // TODO
            masks: HashMap::new(),      // TODO
            filters: HashMap::new(),    // TODO
            markers: HashMap::new(),    // TODO
        };
        Some(Arc::new(tree))
    }

    fn build_node(&self, node: ServoLayoutNode<'dom>) -> Option<Arc<SvgNode>> {
        let element = node.as_element()?;
        let is_root = node == self.root_node;

        let computed_style = self.computed_style(node);
        let version = node.inclusive_descendants_version();

        if !is_root {
            match self.context.svg_subtree_cache.read().lookup(
                node.opaque(),
                version,
                computed_style.as_ref(),
            ) {
                Ok(hit) => {
                    debug!(target: "svg", "build_node: reuse {}", describe(node));
                    return Some(hit);
                },
                Err(reason) => {
                    debug!(target: "svg", "build_node: build {} [{reason:?}]", describe(node))
                },
            }
        }

        let tag = build_tag(
            &element,
            computed_style.as_deref(),
            self.root_width,
            self.root_height,
        )?;
        let id = extract_id(&element);
        let children = node
            .dom_children()
            .filter_map(|child| self.build_node(child))
            .collect();

        let built = Arc::new(SvgNode {
            id,
            tag,
            style: NodeStyle::default(), // TODO
            transforms: Vec::new(),      // TODO
            viewport: None,              // TODO
            children,
        });

        if !is_root {
            self.context.svg_subtree_cache.write().store(
                node.opaque(),
                version,
                computed_style,
                Arc::clone(&built),
            );
        }
        Some(built)
    }

    fn computed_style(
        &self,
        node: ServoLayoutNode<'dom>,
    ) -> Option<ServoArc<style::properties::ComputedValues>> {
        let element = node.as_element()?;
        if element.style_data().is_none() {
            return None;
        }
        Some(node.style(&self.context.style_context))
    }
}

fn build_tag(
    element: &ServoLayoutElement,
    computed: Option<&style::properties::ComputedValues>,
    vw: f32,
    vh: f32,
) -> Option<SvgTag> {
    let tag = element.local_name().as_ref();
    match tag {
        "svg" => Some(SvgTag::Container(Container::Svg)),
        "g" => Some(SvgTag::Container(Container::Group)),
        "defs" => None,
        "use" => None,
        "symbol" => None,
        "switch" => None,
        "text" => None,
        "tspan" => None,
        "image" => None,
        _ => build_shape(element, tag, computed, vw, vh).map(SvgTag::Shape),
    }
}

fn describe<'dom>(node: ServoLayoutNode<'dom>) -> String {
    let Some(element) = node.as_element() else {
        return "<node>".to_owned();
    };
    let tag = element.local_name().to_string();
    match element.attribute_as_str(&ns!(), &local_name!("id")) {
        Some(id) => format!("<{tag}#{id}>"),
        None => format!("<{tag}>"),
    }
}

fn extract_id(element: &ServoLayoutElement) -> Option<Id> {
    element
        .attribute_as_str(&ns!(), &local_name!("id"))
        .map(Id::new)
}
