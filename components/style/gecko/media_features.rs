/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Gecko's media feature list and evaluator.

use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs;
use crate::media_queries::media_feature::{AllowsRanges, ParsingRequirements};
use crate::media_queries::media_feature::{Evaluator, MediaFeatureDescription};
use crate::media_queries::media_feature_expression::{AspectRatio, RangeOrOperator};
use crate::media_queries::{Device, MediaType};
use crate::values::computed::CSSPixelLength;
use crate::values::computed::Resolution;
use crate::Atom;
use app_units::Au;
use euclid::Size2D;

fn viewport_size(device: &Device) -> Size2D<Au> {
    if let Some(pc) = device.pres_context() {
        if pc.mIsRootPaginatedDocument() != 0 {
            // We want the page size, including unprintable areas and margins.
            // FIXME(emilio, bug 1414600): Not quite!
            let area = &pc.mPageSize;
            return Size2D::new(Au(area.width), Au(area.height));
        }
    }
    device.au_viewport_size()
}

fn device_size(device: &Device) -> Size2D<Au> {
    let mut width = 0;
    let mut height = 0;
    unsafe {
        bindings::Gecko_MediaFeatures_GetDeviceSize(device.document(), &mut width, &mut height);
    }
    Size2D::new(Au(width), Au(height))
}

/// https://drafts.csswg.org/mediaqueries-4/#width
fn eval_width(
    device: &Device,
    value: Option<CSSPixelLength>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    RangeOrOperator::evaluate(
        range_or_operator,
        value.map(Au::from),
        viewport_size(device).width,
    )
}

/// https://drafts.csswg.org/mediaqueries-4/#device-width
fn eval_device_width(
    device: &Device,
    value: Option<CSSPixelLength>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    RangeOrOperator::evaluate(
        range_or_operator,
        value.map(Au::from),
        device_size(device).width,
    )
}

/// https://drafts.csswg.org/mediaqueries-4/#height
fn eval_height(
    device: &Device,
    value: Option<CSSPixelLength>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    RangeOrOperator::evaluate(
        range_or_operator,
        value.map(Au::from),
        viewport_size(device).height,
    )
}

/// https://drafts.csswg.org/mediaqueries-4/#device-height
fn eval_device_height(
    device: &Device,
    value: Option<CSSPixelLength>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    RangeOrOperator::evaluate(
        range_or_operator,
        value.map(Au::from),
        device_size(device).height,
    )
}

fn eval_aspect_ratio_for<F>(
    device: &Device,
    query_value: Option<AspectRatio>,
    range_or_operator: Option<RangeOrOperator>,
    get_size: F,
) -> bool
where
    F: FnOnce(&Device) -> Size2D<Au>,
{
    let query_value = match query_value {
        Some(v) => v,
        None => return true,
    };

    let size = get_size(device);
    let value = AspectRatio(size.width.0 as u32, size.height.0 as u32);
    RangeOrOperator::evaluate_with_query_value(range_or_operator, query_value, value)
}

/// https://drafts.csswg.org/mediaqueries-4/#aspect-ratio
fn eval_aspect_ratio(
    device: &Device,
    query_value: Option<AspectRatio>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    eval_aspect_ratio_for(device, query_value, range_or_operator, viewport_size)
}

/// https://drafts.csswg.org/mediaqueries-4/#device-aspect-ratio
fn eval_device_aspect_ratio(
    device: &Device,
    query_value: Option<AspectRatio>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    eval_aspect_ratio_for(device, query_value, range_or_operator, device_size)
}

/// https://compat.spec.whatwg.org/#css-media-queries-webkit-device-pixel-ratio
fn eval_device_pixel_ratio(
    device: &Device,
    query_value: Option<f32>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    eval_resolution(
        device,
        query_value.map(Resolution::from_dppx),
        range_or_operator,
    )
}

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum Orientation {
    Landscape,
    Portrait,
}

fn eval_orientation_for<F>(device: &Device, value: Option<Orientation>, get_size: F) -> bool
where
    F: FnOnce(&Device) -> Size2D<Au>,
{
    let query_orientation = match value {
        Some(v) => v,
        None => return true,
    };

    let size = get_size(device);

    // Per spec, square viewports should be 'portrait'
    let is_landscape = size.width > size.height;
    match query_orientation {
        Orientation::Landscape => is_landscape,
        Orientation::Portrait => !is_landscape,
    }
}

/// https://drafts.csswg.org/mediaqueries-4/#orientation
fn eval_orientation(device: &Device, value: Option<Orientation>) -> bool {
    eval_orientation_for(device, value, viewport_size)
}

/// FIXME: There's no spec for `-moz-device-orientation`.
fn eval_device_orientation(device: &Device, value: Option<Orientation>) -> bool {
    eval_orientation_for(device, value, device_size)
}

/// Values for the display-mode media feature.
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum DisplayMode {
    Browser = 0,
    MinimalUi,
    Standalone,
    Fullscreen,
}

/// https://w3c.github.io/manifest/#the-display-mode-media-feature
fn eval_display_mode(device: &Device, query_value: Option<DisplayMode>) -> bool {
    match query_value {
        Some(v) => v == unsafe { bindings::Gecko_MediaFeatures_GetDisplayMode(device.document()) },
        None => true,
    }
}

/// https://drafts.csswg.org/mediaqueries-4/#grid
fn eval_grid(_: &Device, query_value: Option<bool>, _: Option<RangeOrOperator>) -> bool {
    // Gecko doesn't support grid devices (e.g., ttys), so the 'grid' feature
    // is always 0.
    let supports_grid = false;
    query_value.map_or(supports_grid, |v| v == supports_grid)
}

/// https://compat.spec.whatwg.org/#css-media-queries-webkit-transform-3d
fn eval_transform_3d(_: &Device, query_value: Option<bool>, _: Option<RangeOrOperator>) -> bool {
    let supports_transforms = true;
    query_value.map_or(supports_transforms, |v| v == supports_transforms)
}

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum Scan {
    Progressive,
    Interlace,
}

/// https://drafts.csswg.org/mediaqueries-4/#scan
fn eval_scan(_: &Device, _: Option<Scan>) -> bool {
    // Since Gecko doesn't support the 'tv' media type, the 'scan' feature never
    // matches.
    false
}

/// https://drafts.csswg.org/mediaqueries-4/#color
fn eval_color(
    device: &Device,
    query_value: Option<u32>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    let color_bits_per_channel =
        unsafe { bindings::Gecko_MediaFeatures_GetColorDepth(device.document()) };
    RangeOrOperator::evaluate(range_or_operator, query_value, color_bits_per_channel)
}

/// https://drafts.csswg.org/mediaqueries-4/#color-index
fn eval_color_index(
    _: &Device,
    query_value: Option<u32>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    // We should return zero if the device does not use a color lookup table.
    let index = 0;
    RangeOrOperator::evaluate(range_or_operator, query_value, index)
}

/// https://drafts.csswg.org/mediaqueries-4/#monochrome
fn eval_monochrome(
    _: &Device,
    query_value: Option<u32>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    // For color devices we should return 0.
    // FIXME: On a monochrome device, return the actual color depth, not 0!
    let depth = 0;
    RangeOrOperator::evaluate(range_or_operator, query_value, depth)
}

/// https://drafts.csswg.org/mediaqueries-4/#resolution
fn eval_resolution(
    device: &Device,
    query_value: Option<Resolution>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    let resolution_dppx = unsafe { bindings::Gecko_MediaFeatures_GetResolution(device.document()) };
    RangeOrOperator::evaluate(
        range_or_operator,
        query_value.map(|r| r.dppx()),
        resolution_dppx,
    )
}

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum PrefersReducedMotion {
    NoPreference,
    Reduce,
}

/// Values for the prefers-color-scheme media feature.
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum PrefersColorScheme {
    Light,
    Dark,
    NoPreference,
}

/// https://drafts.csswg.org/mediaqueries-5/#prefers-reduced-motion
fn eval_prefers_reduced_motion(device: &Device, query_value: Option<PrefersReducedMotion>) -> bool {
    let prefers_reduced =
        unsafe { bindings::Gecko_MediaFeatures_PrefersReducedMotion(device.document()) };
    let query_value = match query_value {
        Some(v) => v,
        None => return prefers_reduced,
    };

    match query_value {
        PrefersReducedMotion::NoPreference => !prefers_reduced,
        PrefersReducedMotion::Reduce => prefers_reduced,
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum OverflowBlock {
    None,
    Scroll,
    OptionalPaged,
    Paged,
}

/// https://drafts.csswg.org/mediaqueries-4/#mf-overflow-block
fn eval_overflow_block(device: &Device, query_value: Option<OverflowBlock>) -> bool {
    // For the time being, assume that printing (including previews)
    // is the only time when we paginate, and we are otherwise always
    // scrolling. This is true at the moment in Firefox, but may need
    // updating in the future (e.g., ebook readers built with Stylo, a
    // billboard mode that doesn't support overflow at all).
    //
    // If this ever changes, don't forget to change eval_overflow_inline too.
    let scrolling = device.media_type() != MediaType::print();
    let query_value = match query_value {
        Some(v) => v,
        None => return true,
    };

    match query_value {
        OverflowBlock::None | OverflowBlock::OptionalPaged => false,
        OverflowBlock::Scroll => scrolling,
        OverflowBlock::Paged => !scrolling,
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum OverflowInline {
    None,
    Scroll,
}

/// https://drafts.csswg.org/mediaqueries-4/#mf-overflow-inline
fn eval_overflow_inline(device: &Device, query_value: Option<OverflowInline>) -> bool {
    // See the note in eval_overflow_block.
    let scrolling = device.media_type() != MediaType::print();
    let query_value = match query_value {
        Some(v) => v,
        None => return scrolling,
    };

    match query_value {
        OverflowInline::None => !scrolling,
        OverflowInline::Scroll => scrolling,
    }
}

/// https://drafts.csswg.org/mediaqueries-5/#prefers-color-scheme
fn eval_prefers_color_scheme(device: &Device, query_value: Option<PrefersColorScheme>) -> bool {
    let prefers_color_scheme =
        unsafe { bindings::Gecko_MediaFeatures_PrefersColorScheme(device.document()) };
    match query_value {
        Some(v) => prefers_color_scheme == v,
        None => prefers_color_scheme != PrefersColorScheme::NoPreference,
    }
}

bitflags! {
    /// https://drafts.csswg.org/mediaqueries-4/#mf-interaction
    struct PointerCapabilities: u8 {
        const COARSE = structs::PointerCapabilities_Coarse;
        const FINE = structs::PointerCapabilities_Fine;
        const HOVER = structs::PointerCapabilities_Hover;
    }
}

fn primary_pointer_capabilities(device: &Device) -> PointerCapabilities {
    PointerCapabilities::from_bits_truncate(unsafe {
        bindings::Gecko_MediaFeatures_PrimaryPointerCapabilities(device.document())
    })
}

fn all_pointer_capabilities(device: &Device) -> PointerCapabilities {
    PointerCapabilities::from_bits_truncate(unsafe {
        bindings::Gecko_MediaFeatures_AllPointerCapabilities(device.document())
    })
}

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum Pointer {
    None,
    Coarse,
    Fine,
}

fn eval_pointer_capabilities(
    query_value: Option<Pointer>,
    pointer_capabilities: PointerCapabilities,
) -> bool {
    let query_value = match query_value {
        Some(v) => v,
        None => return !pointer_capabilities.is_empty(),
    };

    match query_value {
        Pointer::None => pointer_capabilities.is_empty(),
        Pointer::Coarse => pointer_capabilities.intersects(PointerCapabilities::COARSE),
        Pointer::Fine => pointer_capabilities.intersects(PointerCapabilities::FINE),
    }
}

/// https://drafts.csswg.org/mediaqueries-4/#pointer
fn eval_pointer(device: &Device, query_value: Option<Pointer>) -> bool {
    eval_pointer_capabilities(query_value, primary_pointer_capabilities(device))
}

/// https://drafts.csswg.org/mediaqueries-4/#descdef-media-any-pointer
fn eval_any_pointer(device: &Device, query_value: Option<Pointer>) -> bool {
    eval_pointer_capabilities(query_value, all_pointer_capabilities(device))
}

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum Hover {
    None,
    Hover,
}

fn eval_hover_capabilities(
    query_value: Option<Hover>,
    pointer_capabilities: PointerCapabilities,
) -> bool {
    let can_hover = pointer_capabilities.intersects(PointerCapabilities::HOVER);
    let query_value = match query_value {
        Some(v) => v,
        None => return can_hover,
    };

    match query_value {
        Hover::None => !can_hover,
        Hover::Hover => can_hover,
    }
}

/// https://drafts.csswg.org/mediaqueries-4/#hover
fn eval_hover(device: &Device, query_value: Option<Hover>) -> bool {
    eval_hover_capabilities(query_value, primary_pointer_capabilities(device))
}

/// https://drafts.csswg.org/mediaqueries-4/#descdef-media-any-hover
fn eval_any_hover(device: &Device, query_value: Option<Hover>) -> bool {
    eval_hover_capabilities(query_value, all_pointer_capabilities(device))
}

fn eval_moz_is_glyph(
    device: &Device,
    query_value: Option<bool>,
    _: Option<RangeOrOperator>,
) -> bool {
    let is_glyph = device.document().mIsSVGGlyphsDocument();
    query_value.map_or(is_glyph, |v| v == is_glyph)
}

fn eval_moz_is_resource_document(
    device: &Device,
    query_value: Option<bool>,
    _: Option<RangeOrOperator>,
) -> bool {
    let is_resource_doc =
        unsafe { bindings::Gecko_MediaFeatures_IsResourceDocument(device.document()) };
    query_value.map_or(is_resource_doc, |v| v == is_resource_doc)
}

fn eval_system_metric(
    device: &Device,
    query_value: Option<bool>,
    metric: Atom,
    accessible_from_content: bool,
) -> bool {
    let supports_metric = unsafe {
        bindings::Gecko_MediaFeatures_HasSystemMetric(
            device.document(),
            metric.as_ptr(),
            accessible_from_content,
        )
    };
    query_value.map_or(supports_metric, |v| v == supports_metric)
}

fn eval_moz_touch_enabled(
    device: &Device,
    query_value: Option<bool>,
    _: Option<RangeOrOperator>,
) -> bool {
    eval_system_metric(
        device,
        query_value,
        atom!("-moz-touch-enabled"),
        /* accessible_from_content = */ true,
    )
}

fn eval_moz_os_version(
    device: &Device,
    query_value: Option<Atom>,
    _: Option<RangeOrOperator>,
) -> bool {
    let query_value = match query_value {
        Some(v) => v,
        None => return false,
    };

    let os_version =
        unsafe { bindings::Gecko_MediaFeatures_GetOperatingSystemVersion(device.document()) };

    query_value.as_ptr() == os_version
}

macro_rules! system_metric_feature {
    ($feature_name:expr) => {{
        fn __eval(device: &Device, query_value: Option<bool>, _: Option<RangeOrOperator>) -> bool {
            eval_system_metric(
                device,
                query_value,
                $feature_name,
                /* accessible_from_content = */ false,
            )
        }

        feature!(
            $feature_name,
            AllowsRanges::No,
            Evaluator::BoolInteger(__eval),
            ParsingRequirements::CHROME_AND_UA_ONLY,
        )
    }};
}

lazy_static! {
    /// Adding new media features requires (1) adding the new feature to this
    /// array, with appropriate entries (and potentially any new code needed
    /// to support new types in these entries and (2) ensuring that either
    /// nsPresContext::MediaFeatureValuesChanged is called when the value that
    /// would be returned by the evaluator function could change.
    pub static ref MEDIA_FEATURES: [MediaFeatureDescription; 53] = [
        feature!(
            atom!("width"),
            AllowsRanges::Yes,
            Evaluator::Length(eval_width),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("height"),
            AllowsRanges::Yes,
            Evaluator::Length(eval_height),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("aspect-ratio"),
            AllowsRanges::Yes,
            Evaluator::IntRatio(eval_aspect_ratio),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("orientation"),
            AllowsRanges::No,
            keyword_evaluator!(eval_orientation, Orientation),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("device-width"),
            AllowsRanges::Yes,
            Evaluator::Length(eval_device_width),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("device-height"),
            AllowsRanges::Yes,
            Evaluator::Length(eval_device_height),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("device-aspect-ratio"),
            AllowsRanges::Yes,
            Evaluator::IntRatio(eval_device_aspect_ratio),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("-moz-device-orientation"),
            AllowsRanges::No,
            keyword_evaluator!(eval_device_orientation, Orientation),
            ParsingRequirements::empty(),
        ),
        // Webkit extensions that we support for de-facto web compatibility.
        // -webkit-{min|max}-device-pixel-ratio (controlled with its own pref):
        feature!(
            atom!("device-pixel-ratio"),
            AllowsRanges::Yes,
            Evaluator::Float(eval_device_pixel_ratio),
            ParsingRequirements::WEBKIT_PREFIX |
                ParsingRequirements::WEBKIT_DEVICE_PIXEL_RATIO_PREF_ENABLED,
        ),
        // -webkit-transform-3d.
        feature!(
            atom!("transform-3d"),
            AllowsRanges::No,
            Evaluator::BoolInteger(eval_transform_3d),
            ParsingRequirements::WEBKIT_PREFIX,
        ),
        feature!(
            atom!("-moz-device-pixel-ratio"),
            AllowsRanges::Yes,
            Evaluator::Float(eval_device_pixel_ratio),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("resolution"),
            AllowsRanges::Yes,
            Evaluator::Resolution(eval_resolution),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("display-mode"),
            AllowsRanges::No,
            keyword_evaluator!(eval_display_mode, DisplayMode),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("grid"),
            AllowsRanges::No,
            Evaluator::BoolInteger(eval_grid),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("scan"),
            AllowsRanges::No,
            keyword_evaluator!(eval_scan, Scan),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("color"),
            AllowsRanges::Yes,
            Evaluator::Integer(eval_color),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("color-index"),
            AllowsRanges::Yes,
            Evaluator::Integer(eval_color_index),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("monochrome"),
            AllowsRanges::Yes,
            Evaluator::Integer(eval_monochrome),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("prefers-reduced-motion"),
            AllowsRanges::No,
            keyword_evaluator!(eval_prefers_reduced_motion, PrefersReducedMotion),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("overflow-block"),
            AllowsRanges::No,
            keyword_evaluator!(eval_overflow_block, OverflowBlock),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("overflow-inline"),
            AllowsRanges::No,
            keyword_evaluator!(eval_overflow_inline, OverflowInline),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("prefers-color-scheme"),
            AllowsRanges::No,
            keyword_evaluator!(eval_prefers_color_scheme, PrefersColorScheme),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("pointer"),
            AllowsRanges::No,
            keyword_evaluator!(eval_pointer, Pointer),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("any-pointer"),
            AllowsRanges::No,
            keyword_evaluator!(eval_any_pointer, Pointer),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("hover"),
            AllowsRanges::No,
            keyword_evaluator!(eval_hover, Hover),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("any-hover"),
            AllowsRanges::No,
            keyword_evaluator!(eval_any_hover, Hover),
            ParsingRequirements::empty(),
        ),

        // Internal -moz-is-glyph media feature: applies only inside SVG glyphs.
        // Internal because it is really only useful in the user agent anyway
        // and therefore not worth standardizing.
        feature!(
            atom!("-moz-is-glyph"),
            AllowsRanges::No,
            Evaluator::BoolInteger(eval_moz_is_glyph),
            ParsingRequirements::CHROME_AND_UA_ONLY,
        ),
        feature!(
            atom!("-moz-is-resource-document"),
            AllowsRanges::No,
            Evaluator::BoolInteger(eval_moz_is_resource_document),
            ParsingRequirements::CHROME_AND_UA_ONLY,
        ),
        feature!(
            atom!("-moz-os-version"),
            AllowsRanges::No,
            Evaluator::Ident(eval_moz_os_version),
            ParsingRequirements::CHROME_AND_UA_ONLY,
        ),
        system_metric_feature!(atom!("-moz-scrollbar-start-backward")),
        system_metric_feature!(atom!("-moz-scrollbar-start-forward")),
        system_metric_feature!(atom!("-moz-scrollbar-end-backward")),
        system_metric_feature!(atom!("-moz-scrollbar-end-forward")),
        system_metric_feature!(atom!("-moz-scrollbar-thumb-proportional")),
        system_metric_feature!(atom!("-moz-overlay-scrollbars")),
        system_metric_feature!(atom!("-moz-windows-default-theme")),
        system_metric_feature!(atom!("-moz-mac-graphite-theme")),
        system_metric_feature!(atom!("-moz-mac-yosemite-theme")),
        system_metric_feature!(atom!("-moz-windows-accent-color-in-titlebar")),
        system_metric_feature!(atom!("-moz-windows-compositor")),
        system_metric_feature!(atom!("-moz-windows-classic")),
        system_metric_feature!(atom!("-moz-windows-glass")),
        system_metric_feature!(atom!("-moz-menubar-drag")),
        system_metric_feature!(atom!("-moz-swipe-animation-enabled")),
        system_metric_feature!(atom!("-moz-gtk-csd-available")),
        system_metric_feature!(atom!("-moz-gtk-csd-hide-titlebar-by-default")),
        system_metric_feature!(atom!("-moz-gtk-csd-transparent-background")),
        system_metric_feature!(atom!("-moz-gtk-csd-minimize-button")),
        system_metric_feature!(atom!("-moz-gtk-csd-maximize-button")),
        system_metric_feature!(atom!("-moz-gtk-csd-close-button")),
        system_metric_feature!(atom!("-moz-gtk-csd-reversed-placement")),
        system_metric_feature!(atom!("-moz-system-dark-theme")),
        // This is the only system-metric media feature that's accessible to
        // content as of today.
        // FIXME(emilio): Restrict (or remove?) when bug 1035774 lands.
        feature!(
            atom!("-moz-touch-enabled"),
            AllowsRanges::No,
            Evaluator::BoolInteger(eval_moz_touch_enabled),
            ParsingRequirements::empty(),
        ),
    ];
}
