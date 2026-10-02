/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

pub mod image;
pub mod shape;
pub mod text;

pub use self::image::SvgImage;
pub use self::shape::{Circle, Ellipse, Line, Path, Polygon, Polyline, Rectangle, Shape};
pub use self::text::TextSpan;
use crate::model::document::SvgViewport;
use crate::model::style::NodeStyle;
use crate::model::transform::TransformOp;
use crate::model::units::Id;

/// A single node in SvgTree.
pub struct SvgNode {
    pub id: Option<Id>,
    pub tag: SvgTag,
    pub style: NodeStyle,
    pub transforms: Vec<TransformOp>,
    /// For nested '<svg>'
    pub viewport: Option<SvgViewport>,
    pub children: Vec<SvgNode>,
}

pub enum SvgTag {
    Shape(Shape),
    Text(TextSpan),
    Image(SvgImage),
    Container(Container),
}

pub enum Container {
    Group,
    Svg,
    Defs,
    Use,
    Switch,
    Symbol,
    Text,
}
