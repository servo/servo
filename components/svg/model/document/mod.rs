/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

mod defs;
mod viewport;

use std::collections::HashMap;
use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;

pub use self::defs::{ClipPathDef, DefRef, FilterDef, GradientDef, MarkerDef, MaskDef, PatternDef};
pub use self::viewport::{
    AspectAlign, AspectRatio, MeetOrSlice, SvgViewport, ViewBox, ViewportInfo,
};
use crate::model::element::SvgNode;

#[derive(Debug, MallocSizeOf)]
pub struct SvgTree {
    pub root: SvgNode,
    pub viewport: ViewportInfo,
    #[conditional_malloc_size_of]
    pub gradients: HashMap<String, Arc<GradientDef>>,
    #[conditional_malloc_size_of]
    pub clip_paths: HashMap<String, Arc<ClipPathDef>>,
    #[conditional_malloc_size_of]
    pub patterns: HashMap<String, Arc<PatternDef>>,
    #[conditional_malloc_size_of]
    pub masks: HashMap<String, Arc<MaskDef>>,
    #[conditional_malloc_size_of]
    pub filters: HashMap<String, Arc<FilterDef>>,
    #[conditional_malloc_size_of]
    pub markers: HashMap<String, Arc<MarkerDef>>,
}
