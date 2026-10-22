/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::sync::Arc;

use malloc_size_of_derive::MallocSizeOf;
use svgtypes::Color as SvgColor;

use crate::model::document::{GradientDef, PatternDef};
use crate::model::units::Id;

#[derive(Debug, MallocSizeOf)]
pub enum PaintServer {
    Solid(#[ignore_malloc_size_of = "External Type"] SvgColor),
    Gradient(#[conditional_malloc_size_of] Arc<GradientDef>),
    Pattern(#[conditional_malloc_size_of] Arc<PatternDef>),
    Ref {
        id: Id,
        #[ignore_malloc_size_of = "External Type"]
        fallback: Option<SvgColor>,
    },
    ContextFill,
    ContextStroke,
}
