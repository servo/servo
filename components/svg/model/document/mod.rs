/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod defs;
mod viewport;

use std::collections::HashMap;
use std::sync::Arc;

pub use self::defs::{ClipPathDef, DefRef, FilterDef, GradientDef, MarkerDef, MaskDef, PatternDef};
pub use self::viewport::{
    AspectAlign, AspectRatio, MeetOrSlice, SvgViewport, ViewBox, ViewportInfo,
};
use crate::model::element::SvgNode;

pub struct SvgTree {
    pub root: SvgNode,
    pub viewport: ViewportInfo,
    pub gradients: HashMap<String, Arc<GradientDef>>,
    pub clip_paths: HashMap<String, Arc<ClipPathDef>>,
    pub patterns: HashMap<String, Arc<PatternDef>>,
    pub masks: HashMap<String, Arc<MaskDef>>,
    pub filters: HashMap<String, Arc<FilterDef>>,
    pub markers: HashMap<String, Arc<MarkerDef>>,
}
