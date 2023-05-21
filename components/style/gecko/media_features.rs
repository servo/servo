/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Gecko's media feature list and evaluator.

use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs;
use crate::media_queries::media_feature::{AllowsRanges, ParsingRequirements};
use crate::media_queries::media_feature::{Evaluator, MediaFeatureDescription};
use crate::media_queries::media_feature_expression::RangeOrOperator;
use crate::media_queries::{Device, MediaType};
use crate::values::computed::CSSPixelLength;
use crate::values::computed::Ratio;
use crate::values::computed::Resolution;
use crate::Atom;
use app_units::Au;
use euclid::default::Size2D;

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
        device.au_viewport_size().width,
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
        device.au_viewport_size().height,
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
    query_value: Option<Ratio>,
    range_or_operator: Option<RangeOrOperator>,
    get_size: F,
) -> bool
where
    F: FnOnce(&Device) -> Size2D<Au>,
{
    // A ratio of 0/0 behaves as the ratio 1/0, so we need to call used_value()
    // to convert it if necessary.
    // FIXME: we may need to update here once
    // https://github.com/w3c/csswg-drafts/issues/4954 got resolved.
    let query_value = match query_value {
        Some(v) => v.used_value(),
        None => return true,
    };

    let size = get_size(device);
    let value = Ratio::new(size.width.0 as f32, size.height.0 as f32);
    RangeOrOperator::evaluate_with_query_value(range_or_operator, query_value, value)
}

/// https://drafts.csswg.org/mediaqueries-4/#aspect-ratio
fn eval_aspect_ratio(
    device: &Device,
    query_value: Option<Ratio>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    eval_aspect_ratio_for(
        device,
        query_value,
        range_or_operator,
        Device::au_viewport_size,
    )
}

/// https://drafts.csswg.org/mediaqueries-4/#device-aspect-ratio
fn eval_device_aspect_ratio(
    device: &Device,
    query_value: Option<Ratio>,
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
    eval_orientation_for(device, value, Device::au_viewport_size)
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
    device: &Device,
    query_value: Option<u32>,
    range_or_operator: Option<RangeOrOperator>,
) -> bool {
    // For color devices we should return 0.
    let depth =
        unsafe { bindings::Gecko_MediaFeatures_GetMonochromeBitsPerPixel(device.document()) };
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

/// Possible values for prefers-contrast media query.
/// https://drafts.csswg.org/mediaqueries-5/#prefers-contrast
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
pub enum PrefersContrast {
    /// More contrast is preferred. Corresponds to an accessibility theme
    /// being enabled or Firefox forcing high contrast colors.
    More,
    /// Low contrast is preferred.
    Less,
    /// The default value if neither high or low contrast is enabled.
    NoPreference,
}

/// https://drafts.csswg.org/mediaqueries-5/#prefers-contrast
fn eval_prefers_contrast(device: &Device, query_value: Option<PrefersContrast>) -> bool {
    let prefers_contrast =
        unsafe { bindings::Gecko_MediaFeatures_PrefersContrast(device.document()) };
    match query_value {
        Some(v) => v == prefers_contrast,
        None => prefers_contrast != PrefersContrast::NoPreference,
    }
}

/// Possible values for the forced-colors media query.
/// https://drafts.csswg.org/mediaqueries-5/#forced-colors
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
pub enum ForcedColors {
    /// Page colors are not being forced.
    None,
    /// Page colors are being forced.
    Active,
}

/// https://drafts.csswg.org/mediaqueries-5/#forced-colors
fn eval_forced_colors(device: &Device, query_value: Option<ForcedColors>) -> bool {
    let forced = !device.use_document_colors();
    match query_value {
        Some(query_value) => forced == (query_value == ForcedColors::Active),
        None => forced,
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
        None => true,
    }
}

/// Values for the -moz-toolbar-prefers-color-scheme media feature.
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, ToCss)]
#[repr(u8)]
enum ToolbarPrefersColorScheme {
    Dark,
    Light,
    System,
}

/// The color-scheme of the toolbar in the current Firefox theme. This is based
/// on a pref managed by the front-end.
fn eval_toolbar_prefers_color_scheme(d: &Device, query_value: Option<ToolbarPrefersColorScheme>) -> bool {
    let toolbar_value = match static_prefs::pref!("browser.theme.toolbar-theme") {
        0 => ToolbarPrefersColorScheme::Dark,
        1 => ToolbarPrefersColorScheme::Light,
        _ => ToolbarPrefersColorScheme::System,
    };

    let query_value = match query_value {
        Some(v) => v,
        None => return true,
    };

    if query_value == toolbar_value {
        return true;
    }

    if toolbar_value != ToolbarPrefersColorScheme::System {
        return false;
    }

    // System might match light and dark as well.
    match query_value {
        ToolbarPrefersColorScheme::Dark => eval_prefers_color_scheme(d, Some(PrefersColorScheme::Dark)),
        ToolbarPrefersColorScheme::Light => eval_prefers_color_scheme(d, Some(PrefersColorScheme::Light)),
        ToolbarPrefersColorScheme::System => true,
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

fn eval_moz_print_preview(
    device: &Device,
    query_value: Option<bool>,
    _: Option<RangeOrOperator>,
) -> bool {
    let is_print_preview = device.is_print_preview();
    if is_print_preview {
        debug_assert_eq!(device.media_type(), MediaType::print());
    }
    query_value.map_or(is_print_preview, |v| v == is_print_preview)
}

fn eval_moz_non_native_content_theme(
    device: &Device,
    query_value: Option<bool>,
    _: Option<RangeOrOperator>,
) -> bool {
    let non_native_theme =
        unsafe { bindings::Gecko_MediaFeatures_ShouldAvoidNativeTheme(device.document()) };
    query_value.map_or(non_native_theme, |v| v == non_native_theme)
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

fn eval_moz_windows_non_native_menus(
    device: &Device,
    query_value: Option<bool>,
    _: Option<RangeOrOperator>,
) -> bool {
    let use_non_native_menus = match static_prefs::pref!("browser.display.windows.non_native_menus") {
        0 => false,
        1 => true,
        _ => {
            eval_moz_os_version(device, Some(atom!("windows-win10")), None) &&
                get_lnf_int_as_bool(bindings::LookAndFeel_IntID::WindowsDefaultTheme as i32)
        },
    };

    query_value.map_or(use_non_native_menus, |v| v == use_non_native_menus)
}

fn get_lnf_int(int_id: i32) -> i32 {
    unsafe { bindings::Gecko_GetLookAndFeelInt(int_id) }
}

fn get_lnf_int_as_bool(int_id: i32) -> bool {
    get_lnf_int(int_id) != 0
}

fn get_scrollbar_start_backward(int_id: i32) -> bool {
    (get_lnf_int(int_id) & bindings::LookAndFeel_eScrollArrow_StartBackward as i32) != 0
}

fn get_scrollbar_start_forward(int_id: i32) -> bool {
    (get_lnf_int(int_id) & bindings::LookAndFeel_eScrollArrow_StartForward as i32) != 0
}

fn get_scrollbar_end_backward(int_id: i32) -> bool {
    (get_lnf_int(int_id) & bindings::LookAndFeel_eScrollArrow_EndBackward as i32) != 0
}

fn get_scrollbar_end_forward(int_id: i32) -> bool {
    (get_lnf_int(int_id) & bindings::LookAndFeel_eScrollArrow_EndForward as i32) != 0
}

macro_rules! lnf_int_feature {
    ($feature_name:expr, $int_id:ident, $get_value:ident) => {{
        fn __eval(_: &Device, query_value: Option<bool>, _: Option<RangeOrOperator>) -> bool {
            let value = $get_value(bindings::LookAndFeel_IntID::$int_id as i32);
            query_value.map_or(value, |v| v == value)
        }

        feature!(
            $feature_name,
            AllowsRanges::No,
            Evaluator::BoolInteger(__eval),
            ParsingRequirements::CHROME_AND_UA_ONLY,
        )
    }};
    ($feature_name:expr, $int_id:ident) => {{
        lnf_int_feature!($feature_name, $int_id, get_lnf_int_as_bool)
    }};
}

/// bool pref-based features are an slightly less convenient to start using
/// version of @supports -moz-bool-pref, but with some benefits, mainly that
/// they can support dynamic changes, and don't require a pref lookup every time
/// they're used.
///
/// In order to use them you need to make sure that the pref defined as a static
/// pref, with `rust: true`. The feature name needs to be defined in
/// `StaticAtoms.py` just like the others. In order to support dynamic changes,
/// you also need to add them to kMediaQueryPrefs in nsXPLookAndFeel.cpp
macro_rules! bool_pref_feature {
    ($feature_name:expr, $pref:tt) => {{
        fn __eval(_: &Device, query_value: Option<bool>, _: Option<RangeOrOperator>) -> bool {
            let value = static_prefs::pref!($pref);
            query_value.map_or(value, |v| v == value)
        }

        feature!(
            $feature_name,
            AllowsRanges::No,
            Evaluator::BoolInteger(__eval),
            ParsingRequirements::CHROME_AND_UA_ONLY,
        )
    }};
}

/// Adding new media features requires (1) adding the new feature to this
/// array, with appropriate entries (and potentially any new code needed
/// to support new types in these entries and (2) ensuring that either
/// nsPresContext::MediaFeatureValuesChanged is called when the value that
/// would be returned by the evaluator function could change.
pub static MEDIA_FEATURES: [MediaFeatureDescription; 60] = [
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
        Evaluator::NumberRatio(eval_aspect_ratio),
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
        Evaluator::NumberRatio(eval_device_aspect_ratio),
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
        ParsingRequirements::WEBKIT_PREFIX,
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
        atom!("prefers-contrast"),
        AllowsRanges::No,
        keyword_evaluator!(eval_prefers_contrast, PrefersContrast),
        // Note: by default this is only enabled in browser chrome and
        // ua. It can be enabled on the web via the
        // layout.css.prefers-contrast.enabled preference. See
        // disabed_by_pref in media_feature_expression.rs for how that
        // is done.
        ParsingRequirements::empty(),
    ),
    feature!(
        atom!("forced-colors"),
        AllowsRanges::No,
        keyword_evaluator!(eval_forced_colors, ForcedColors),
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
    feature!(
        atom!("-moz-print-preview"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_moz_print_preview),
        ParsingRequirements::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("-moz-non-native-content-theme"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_moz_non_native_content_theme),
        ParsingRequirements::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("-moz-toolbar-prefers-color-scheme"),
        AllowsRanges::No,
        keyword_evaluator!(eval_toolbar_prefers_color_scheme, ToolbarPrefersColorScheme),
        ParsingRequirements::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("-moz-windows-non-native-menus"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_moz_windows_non_native_menus),
        ParsingRequirements::CHROME_AND_UA_ONLY,
    ),

    lnf_int_feature!(atom!("-moz-scrollbar-start-backward"), ScrollArrowStyle, get_scrollbar_start_backward),
    lnf_int_feature!(atom!("-moz-scrollbar-start-forward"), ScrollArrowStyle, get_scrollbar_start_forward),
    lnf_int_feature!(atom!("-moz-scrollbar-end-backward"), ScrollArrowStyle, get_scrollbar_end_backward),
    lnf_int_feature!(atom!("-moz-scrollbar-end-forward"), ScrollArrowStyle, get_scrollbar_end_forward),
    lnf_int_feature!(atom!("-moz-scrollbar-thumb-proportional"), ScrollSliderStyle),
    lnf_int_feature!(atom!("-moz-overlay-scrollbars"), UseOverlayScrollbars),
    lnf_int_feature!(atom!("-moz-menubar-drag"), MenuBarDrag),
    lnf_int_feature!(atom!("-moz-windows-default-theme"), WindowsDefaultTheme),
    lnf_int_feature!(atom!("-moz-mac-graphite-theme"), MacGraphiteTheme),
    lnf_int_feature!(atom!("-moz-mac-big-sur-theme"), MacBigSurTheme),
    lnf_int_feature!(atom!("-moz-windows-accent-color-in-titlebar"), WindowsAccentColorInTitlebar),
    lnf_int_feature!(atom!("-moz-windows-compositor"), DWMCompositor),
    lnf_int_feature!(atom!("-moz-windows-classic"), WindowsClassic),
    lnf_int_feature!(atom!("-moz-windows-glass"), WindowsGlass),
    lnf_int_feature!(atom!("-moz-swipe-animation-enabled"), SwipeAnimationEnabled),
    lnf_int_feature!(atom!("-moz-gtk-csd-available"), GTKCSDAvailable),
    lnf_int_feature!(atom!("-moz-gtk-csd-hide-titlebar-by-default"), GTKCSDHideTitlebarByDefault),
    lnf_int_feature!(atom!("-moz-gtk-csd-transparent-background"), GTKCSDTransparentBackground),
    lnf_int_feature!(atom!("-moz-gtk-csd-minimize-button"), GTKCSDMinimizeButton),
    lnf_int_feature!(atom!("-moz-gtk-csd-maximize-button"), GTKCSDMaximizeButton),
    lnf_int_feature!(atom!("-moz-gtk-csd-close-button"), GTKCSDCloseButton),
    lnf_int_feature!(atom!("-moz-gtk-csd-reversed-placement"), GTKCSDReversedPlacement),
    lnf_int_feature!(atom!("-moz-system-dark-theme"), SystemUsesDarkTheme),
    bool_pref_feature!(atom!("-moz-proton"), "browser.proton.enabled"),
    bool_pref_feature!(atom!("-moz-proton-places-tooltip"), "browser.proton.places-tooltip.enabled"),
];
