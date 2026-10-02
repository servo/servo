/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::paint_servers::PaintServer;
use crate::model::document::{DefRef, MarkerDef};
use crate::model::units::{Length, Opacity};

// Fill properties.
pub struct FillParams {
    pub paint_server: Option<PaintServer>,
    pub opacity: Opacity,
    pub fill_rule: FillRule,
}

pub enum FillRule {
    NonZero,
    EvenOdd,
}

pub struct StrokeParams {
    pub paint_server: Option<PaintServer>,
    pub opacity: Opacity,
    pub width: Length,
    pub line_cap: LineCap,
    pub line_join: LineJoin,
    pub miter_limit: f32,
    pub dash_array: Option<Vec<f32>>,
    pub dash_offset: f32,
}

pub enum LineCap {
    Butt,
    Round,
    Square,
}

pub enum LineJoin {
    Miter,
    MiterClip,
    Round,
    Bevel,
    Arcs,
}

pub struct MarkerRefs {
    pub start: Option<DefRef<MarkerDef>>,
    pub mid: Option<DefRef<MarkerDef>>,
    pub end: Option<DefRef<MarkerDef>>,
}
