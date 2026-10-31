/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod builder;
mod cache;
mod primitives;

use std::sync::Arc;

pub(crate) use cache::SvgSubtreeCache;
use script::layout_dom::ServoLayoutNode;
use servo_svg::document::SvgTree;

use crate::context::LayoutContext;

/// The entry point for building an SvgTree.
pub(crate) fn build_svg_tree<'dom>(
    node: ServoLayoutNode<'dom>,
    context: &LayoutContext,
    viewport_width: f32,
    viewport_height: f32,
) -> Option<Arc<SvgTree>> {
    builder::SvgTreeBuilder::new(node, context, viewport_width, viewport_height).build()
}
