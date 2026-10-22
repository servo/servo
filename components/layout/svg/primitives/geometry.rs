/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use script::layout_dom::ServoLayoutElement;
use servo_svg::element::{Rectangle, Shape};
use servo_svg::units::Length;
use style::values::computed::{Length as CssLength, LengthPercentage};
use style::values::generics::length::GenericLengthPercentageOrAuto;

use crate::svg::primitives::attrs::{get_attr, parse_length, parse_length_resolved};

const SVG_DEFAULT_FONT_SIZE: f32 = 16.0;

/// Build a Shape from a DOM element.
pub(crate) fn build_shape(
    element: &ServoLayoutElement,
    tag_name: &str,
    computed: Option<&style::properties::ComputedValues>,
    vw: f32,
    vh: f32,
) -> Option<Shape> {
    let fs = SVG_DEFAULT_FONT_SIZE;

    match tag_name {
        "rect" => parse_rect(element, fs, computed, vw, vh),
        // TODO: parse circle, ellipse, line, polyline, polygon and path
        _ => None,
    }
}

// ========================= Shape Parsers =========================
/// Parse a `<rect>` element.
fn parse_rect(
    element: &ServoLayoutElement,
    fs: f32,
    computed: Option<&style::properties::ComputedValues>,
    vw: f32,
    vh: f32,
) -> Option<Shape> {
    let w = parse_length("width", element, fs).unwrap_or(0.0);
    let h = parse_length("height", element, fs).unwrap_or(0.0);
    if w < 0.0 || h < 0.0 {
        return None;
    }

    // x/y/rx/ry: computed CSS when present, else presentation attributes.
    let (x, y, rx, ry) = match computed {
        Some(cv) => {
            let svg = cv.get_svg();
            (
                resolve_length_percentage(&svg.clone_x(), vw),
                resolve_length_percentage(&svg.clone_y(), vh),
                match svg.clone_rx() {
                    GenericLengthPercentageOrAuto::LengthPercentage(nn_lp) => {
                        resolve_radius(&nn_lp.0, w)
                    },
                    _ => None,
                },
                match svg.clone_ry() {
                    GenericLengthPercentageOrAuto::LengthPercentage(nn_lp) => {
                        resolve_radius(&nn_lp.0, h)
                    },
                    _ => None,
                },
            )
        },
        None => (
            parse_length_resolved("x", element, fs, vw).unwrap_or(0.0),
            parse_length_resolved("y", element, fs, vh).unwrap_or(0.0),
            parse_radius("rx", element, fs, w),
            parse_radius("ry", element, fs, h),
        ),
    };
    Some(Shape::Rect(Rectangle {
        x: Length::new(x),
        y: Length::new(y),
        width: Length::new(w),
        height: Length::new(h),
        rx: rx.map(Length::new),
        ry: ry.map(Length::new),
        path_length: parse_path_length(element),
    }))
}
// ========================= Helpers =========================

/// Resolve a computed CSS `LengthPercentage` to pixels.
fn resolve_length_percentage(lp: &LengthPercentage, reference: f32) -> f32 {
    lp.resolve(CssLength::new(reference)).px()
}

/// Resolve a computed CSS radius, mapping a negative value to `None`.
fn resolve_radius(lp: &LengthPercentage, reference: f32) -> Option<f32> {
    let v = resolve_length_percentage(lp, reference);
    (v >= 0.0).then_some(v)
}

/// Parse the `rx`/`ry` presentation attributes, resolving percentages against `reference`.
fn parse_radius(name: &str, element: &ServoLayoutElement, fs: f32, reference: f32) -> Option<f32> {
    parse_length_resolved(name, element, fs, reference)
        .ok()
        .and_then(|v| (v >= 0.0).then_some(v))
}

/// Parse the `pathLength` presentation attribute.
fn parse_path_length(element: &ServoLayoutElement) -> Option<f32> {
    get_attr(element, "pathLength")
        .and_then(|v| v.trim().parse::<f32>().ok())
        .filter(|&n| n.is_finite() && n > 0.0)
}
