/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko's media feature list and evaluator.

use Atom;
use app_units::Au;
use euclid::Size2D;
use gecko_bindings::bindings;
use values::computed::CSSPixelLength;
use values::computed::Resolution;

use media_queries::Device;
use media_queries::media_feature::{MediaFeatureDescription, Evaluator};
use media_queries::media_feature::{AllowsRanges, ParsingRequirements};
use media_queries::media_feature_expression::{AspectRatio, RangeOrOperator};

macro_rules! feature {
    ($name:expr, $allows_ranges:expr, $evaluator:expr, $reqs:expr,) => {
        MediaFeatureDescription {
            name: $name,
            allows_ranges: $allows_ranges,
            evaluator: $evaluator,
            requirements: $reqs,
        }
    }
}

fn viewport_size(device: &Device) -> Size2D<Au> {
    let pc = device.pres_context();
    if pc.mIsRootPaginatedDocument() != 0 {
        // We want the page size, including unprintable areas and margins.
        // FIXME(emilio, bug 1414600): Not quite!
        let area = &pc.mPageSize;
        return Size2D::new(Au(area.width), Au(area.height))
    }
    device.au_viewport_size()
}

fn device_size(device: &Device) -> Size2D<Au> {
    let mut width = 0;
    let mut height = 0;
    unsafe {
        bindings::Gecko_MediaFeatures_GetDeviceSize(
            device.document(),
            &mut width,
            &mut height,
        );
    }
    Size2D::new(Au(width), Au(height))
}

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
    RangeOrOperator::evaluate(
        range_or_operator,
        Some(size.height.0 as u64 * query_value.0 as u64),
        size.width.0 as u64 * query_value.1 as u64,
    )
}

fn eval_aspect_ratio(
    device: &Device,
    query_value: Option<AspectRatio>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    eval_aspect_ratio_for(device, query_value, range_or_operator, viewport_size)
}

fn eval_device_aspect_ratio(
    device: &Device,
    query_value: Option<AspectRatio>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    eval_aspect_ratio_for(device, query_value, range_or_operator, device_size)
}

fn eval_device_pixel_ratio(
    device: &Device,
    query_value: Option<f32>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    let ratio = unsafe {
        bindings::Gecko_MediaFeatures_GetDevicePixelRatio(device.document())
    };

    RangeOrOperator::evaluate(
        range_or_operator,
        query_value,
        ratio,
    )
}

#[derive(Debug, Copy, Clone, FromPrimitive, ToCss, Parse)]
#[repr(u8)]
enum Orientation {
    Landscape,
    Portrait,
}

fn eval_orientation_for<F>(
    device: &Device,
    value: Option<Orientation>,
    get_size: F,
) -> bool
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

fn eval_orientation(
    device: &Device,
    value: Option<Orientation>,
) -> bool {
    eval_orientation_for(device, value, viewport_size)
}

fn eval_device_orientation(
    device: &Device,
    value: Option<Orientation>,
) -> bool {
    eval_orientation_for(device, value, device_size)
}

/// Values for the display-mode media feature.
#[derive(Debug, Copy, Clone, FromPrimitive, ToCss, Parse)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum DisplayMode {
  Browser = 0,
  MinimalUi,
  Standalone,
  Fullscreen,
}

fn eval_display_mode(
    device: &Device,
    query_value: Option<DisplayMode>,
) -> bool {
    let query_value = match query_value {
        Some(v) => v,
        None => return true,
    };

    let gecko_display_mode = unsafe {
        bindings::Gecko_MediaFeatures_GetDisplayMode(device.document())
    };

    // NOTE: cbindgen guarantees the same representation.
    gecko_display_mode as u8 == query_value as u8
}

fn eval_grid(_: &Device, query_value: Option<bool>, _: Option<RangeOrOperator>) -> bool {
    // Gecko doesn't support grid devices (e.g., ttys), so the 'grid' feature
    // is always 0.
    let supports_grid = false;
    query_value.map_or(supports_grid, |v| v == supports_grid)
}

fn eval_transform_3d(
    _: &Device,
    query_value: Option<bool>,
    _: Option<RangeOrOperator>,
) -> bool {
    let supports_transforms = true;
    query_value.map_or(supports_transforms, |v| v == supports_transforms)
}

#[derive(Debug, Copy, Clone, FromPrimitive, ToCss, Parse)]
#[repr(u8)]
enum Scan {
    Progressive,
    Interlace,
}

fn eval_scan(_: &Device, _: Option<Scan>) -> bool {
    // Since Gecko doesn't support the 'tv' media type, the 'scan' feature never
    // matches.
    false
}

fn eval_color(
    device: &Device,
    query_value: Option<u32>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    let color_bits_per_channel =
        unsafe { bindings::Gecko_MediaFeatures_GetColorDepth(device.document()) };
    RangeOrOperator::evaluate(
        range_or_operator,
        query_value,
        color_bits_per_channel,
    )
}

fn eval_color_index(
    _: &Device,
    query_value: Option<u32>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    // We should return zero if the device does not use a color lookup
    // table.
    let index = 0;
    RangeOrOperator::evaluate(
        range_or_operator,
        query_value,
        index,
    )
}

fn eval_monochrome(
    _: &Device,
    query_value: Option<u32>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    // For color devices we should return 0.
    // FIXME: On a monochrome device, return the actual color depth, not 0!
    let depth = 0;
    RangeOrOperator::evaluate(
        range_or_operator,
        query_value,
        depth,
    )
}

fn eval_resolution(
    device: &Device,
    query_value: Option<Resolution>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    let resolution_dppx =
        unsafe { bindings::Gecko_MediaFeatures_GetResolution(device.document()) };
    RangeOrOperator::evaluate(
        range_or_operator,
        query_value.map(|r| r.dppx()),
        resolution_dppx,
    )
}

#[derive(Debug, Copy, Clone, FromPrimitive, ToCss, Parse)]
#[repr(u8)]
enum PrefersReducedMotion {
    NoPreference,
    Reduce,
}

fn eval_prefers_reduced_motion(
    device: &Device,
    query_value: Option<PrefersReducedMotion>,
) -> bool {
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

fn eval_moz_is_glyph(
    device: &Device,
    query_value: Option<bool>,
    _: Option<RangeOrOperator>,
) -> bool {
    let is_glyph = unsafe { (*device.document()).mIsSVGGlyphsDocument() };
    query_value.map_or(is_glyph, |v| v == is_glyph)
}

fn eval_moz_is_resource_document(
    device: &Device,
    query_value: Option<bool>,
    _: Option<RangeOrOperator>,
) -> bool {
    let is_resource_doc  = unsafe {
        bindings::Gecko_MediaFeatures_IsResourceDocument(device.document())
    };
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
        atom!("touch-enabled"),
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

    let os_version = unsafe {
        bindings::Gecko_MediaFeatures_GetOperatingSystemVersion(device.document())
    };

    query_value.as_ptr() == os_version
}

macro_rules! system_metric_feature {
    ($feature_name:expr, $metric_name:expr) => {
        {
            fn __eval(
                device: &Device,
                query_value: Option<bool>,
                _: Option<RangeOrOperator>,
            ) -> bool {
                eval_system_metric(
                    device,
                    query_value,
                    $metric_name,
                    /* accessible_from_content = */ false,
                )
            }

            feature!(
                $feature_name,
                AllowsRanges::No,
                Evaluator::BoolInteger(__eval),
                ParsingRequirements::CHROME_AND_UA_ONLY,
            )
        }
    }
}

lazy_static! {
    /// Adding new media features requires (1) adding the new feature to this
    /// array, with appropriate entries (and potentially any new code needed
    /// to support new types in these entries and (2) ensuring that either
    /// nsPresContext::MediaFeatureValuesChanged is called when the value that
    /// would be returned by the evaluator function could change.
    pub static ref MEDIA_FEATURES: [MediaFeatureDescription; 43] = [
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
        // FIXME(emilio): make system metrics store the -moz- atom, and remove
        // some duplication here.
        system_metric_feature!(atom!("-moz-scrollbar-start-backward"), atom!("scrollbar-start-backward")),
        system_metric_feature!(atom!("-moz-scrollbar-start-forward"), atom!("scrollbar-start-forward")),
        system_metric_feature!(atom!("-moz-scrollbar-end-backward"), atom!("scrollbar-end-backward")),
        system_metric_feature!(atom!("-moz-scrollbar-end-forward"), atom!("scrollbar-end-forward")),
        system_metric_feature!(atom!("-moz-scrollbar-thumb-proportional"), atom!("scrollbar-thumb-proportional")),
        system_metric_feature!(atom!("-moz-overlay-scrollbars"), atom!("overlay-scrollbars")),
        system_metric_feature!(atom!("-moz-windows-default-theme"), atom!("windows-default-theme")),
        system_metric_feature!(atom!("-moz-mac-graphite-theme"), atom!("mac-graphite-theme")),
        system_metric_feature!(atom!("-moz-mac-yosemite-theme"), atom!("mac-yosemite-theme")),
        system_metric_feature!(atom!("-moz-windows-accent-color-in-titlebar"), atom!("windows-accent-color-in-titlebar")),
        system_metric_feature!(atom!("-moz-windows-compositor"), atom!("windows-compositor")),
        system_metric_feature!(atom!("-moz-windows-classic"), atom!("windows-classic")),
        system_metric_feature!(atom!("-moz-windows-glass"), atom!("windows-glass")),
        system_metric_feature!(atom!("-moz-menubar-drag"), atom!("menubar-drag")),
        system_metric_feature!(atom!("-moz-swipe-animation-enabled"), atom!("swipe-animation-enabled")),
        system_metric_feature!(atom!("-moz-gtk-csd-available"), atom!("gtk-csd-available")),
        system_metric_feature!(atom!("-moz-gtk-csd-minimize-button"), atom!("gtk-csd-minimize-button")),
        system_metric_feature!(atom!("-moz-gtk-csd-maximize-button"), atom!("gtk-csd-maximize-button")),
        system_metric_feature!(atom!("-moz-gtk-csd-close-button"), atom!("gtk-csd-close-button")),
        system_metric_feature!(atom!("-moz-system-dark-theme"), atom!("system-dark-theme")),
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
