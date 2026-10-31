/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use html5ever::LocalName;
use layout_api::LayoutElement;
use script::layout_dom::ServoLayoutElement;
use svgtypes::Length as SvgLength;
use web_atoms::ns;

/// Read an SVG presentation attribute as a string.
pub(crate) fn get_attr(element: &ServoLayoutElement, attr: &str) -> Option<String> {
    element
        .attribute_as_str(&ns!(), &LocalName::from(attr))
        .map(|s| s.to_string())
}

/// Parse a presentation-attribute length to pixels.
pub(crate) fn parse_length(
    attr: &str,
    element: &ServoLayoutElement,
    font_size: f32,
) -> Result<f32, String> {
    let value = get_attr(element, attr).ok_or_else(|| format!("missing SVG attribute: {attr}"))?;
    let len: SvgLength = value.parse().map_err(|e| format!("{attr}: {e}"))?;
    Ok(to_px(len, font_size))
}

/// Convert a presentation-attribute length to pixels.
fn to_px(len: SvgLength, font_size: f32) -> f32 {
    let n = len.number as f32;
    match len.unit {
        // Absolute units: <https://www.w3.org/TR/css-values-3/#absolute-lengths>
        svgtypes::LengthUnit::None | svgtypes::LengthUnit::Px => n,
        svgtypes::LengthUnit::In => n * 96.0,
        svgtypes::LengthUnit::Cm => n * 96.0 / 2.54,
        svgtypes::LengthUnit::Mm => n * 96.0 / 25.4,
        svgtypes::LengthUnit::Pt => n * 96.0 / 72.0,
        svgtypes::LengthUnit::Pc => n * 96.0 / 6.0,
        // Font-relative units
        svgtypes::LengthUnit::Em => n * font_size,
        svgtypes::LengthUnit::Ex => n * font_size * 0.5,
        // Percentage
        svgtypes::LengthUnit::Percent => n,
    }
}

/// Parse a presentation-attribute length, resolving a percentage against `reference`.
pub(crate) fn parse_length_resolved(
    attr: &str,
    element: &ServoLayoutElement,
    font_size: f32,
    reference: f32,
) -> Result<f32, String> {
    let value = get_attr(element, attr).ok_or_else(|| format!("missing SVG attribute: {attr}"))?;
    let len: SvgLength = value.parse().map_err(|e| format!("{attr}: {e}"))?;
    Ok(to_px_resolved(len, font_size, reference))
}

/// Convert a presentation-attribute length to pixels, resolving a percentage against `reference`.
fn to_px_resolved(len: SvgLength, font_size: f32, reference: f32) -> f32 {
    if len.unit == svgtypes::LengthUnit::Percent {
        (len.number as f32) * reference / 100.0
    } else {
        to_px(len, font_size)
    }
}
