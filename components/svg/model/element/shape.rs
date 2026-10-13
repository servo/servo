/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use crate::model::geometry::{PathData, Point};
use crate::model::units::Length;

// Geometric shape.
pub enum Shape {
    Rect(Rectangle),
    Circle(Circle),
    Ellipse(Ellipse),
    Line(Line),
    Polyline(Polyline),
    Polygon(Polygon),
    Path(Path),
}

// <rect> element
pub struct Rectangle {
    pub x: Length,
    pub y: Length,
    pub width: Length,
    pub height: Length,
    pub rx: Option<Length>,
    pub ry: Option<Length>,
    pub path_length: Option<f32>,
}

// <circle> element
pub struct Circle {
    pub cx: Length,
    pub cy: Length,
    pub r: Length,
    pub path_length: Option<f32>,
}

// <ellipse> element
pub struct Ellipse {
    pub cx: Length,
    pub cy: Length,
    pub rx: Option<Length>,
    pub ry: Option<Length>,
    pub path_length: Option<f32>,
}

// <Line> element
pub struct Line {
    pub x1: Length,
    pub y1: Length,
    pub x2: Length,
    pub y2: Length,
    pub path_length: Option<f32>,
}

// <polyline> element
pub struct Polyline {
    pub points: Vec<Point>,
    pub path_length: Option<f32>,
}

// <polygon> element
pub struct Polygon {
    pub points: Vec<Point>,
    pub path_length: Option<f32>,
}

// <path> element
pub struct Path {
    pub path: PathData,
    pub path_length: Option<f32>,
}
