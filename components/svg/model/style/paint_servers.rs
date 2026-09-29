/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use svgtypes::Color as SvgColor;

use crate::model::document::{GradientDef, PatternDef};
use crate::model::units::Id;

pub enum PaintServer {
    Solid(SvgColor),
    Gradient(Arc<GradientDef>),
    Pattern(Arc<PatternDef>),
    Ref { id: Id, fallback: Option<SvgColor> },
    ContextFill,
    ContextStroke,
}
