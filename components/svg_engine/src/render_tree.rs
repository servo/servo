/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use malloc_size_of_derive::MallocSizeOf;

use crate::shapes::Shape;

#[derive(Debug, MallocSizeOf)]
pub struct SvgRenderTree {
    pub root: SvgRenderNode,
    // pub viewport: ViewportInfo,
    // pub gradients: HashMap<String, GradientDef>,
    // pub clip_paths: HashMap<String, ClipPathDef>,
    // pub patterns: HashMap<String, PatternDef>,
    // pub masks: HashMap<String, MaskDef>,
    // pub filters: HashMap<String, FilterDef>,
}

#[derive(Debug, MallocSizeOf)]
pub struct SvgRenderNode {
    pub id: Option<String>,
    pub tag: SvgTag,
    // pub style: NodeStyle,
    // pub transforms: Vec<TransformOp>,
    pub children: Vec<SvgRenderNode>,
}

#[derive(Debug, MallocSizeOf)]
pub enum SvgTag {
    Shape(Shape),
    Container(Container),
    // TODO: Text, Image
}

#[derive(Debug, MallocSizeOf)]
pub enum Container {
    Group,
    Svg,
    // TODO: Defs, Symbol, Use
}
