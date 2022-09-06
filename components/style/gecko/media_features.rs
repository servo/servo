/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Gecko's media feature list and evaluator.

use crate::gecko_bindings::bindings;
use crate::gecko_bindings::structs;
use crate::queries::feature::{AllowsRanges, Evaluator, FeatureFlags, QueryFeatureDescription};
use crate::queries::values::Orientation;
use crate::media_queries::{Device, MediaType};
use crate::values::computed::{Context, CSSPixelLength, Ratio, Resolution};
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
fn eval_width(context: &Context) -> CSSPixelLength {
    CSSPixelLength::new(context.device().au_viewport_size().width.to_f32_px())
}

/// https://drafts.csswg.org/mediaqueries-4/#device-width
fn eval_device_width(context: &Context) -> CSSPixelLength {
    CSSPixelLength::new(device_size(context.device()).width.to_f32_px())
}

/// https://drafts.csswg.org/mediaqueries-4/#height
fn eval_height(context: &Context) -> CSSPixelLength {
    CSSPixelLength::new(context.device().au_viewport_size().height.to_f32_px())
}

/// https://drafts.csswg.org/mediaqueries-4/#device-height
fn eval_device_height(context: &Context) -> CSSPixelLength {
    CSSPixelLength::new(device_size(context.device()).height.to_f32_px())
}

fn eval_aspect_ratio_for<F>(context: &Context, get_size: F) -> Ratio
where
    F: FnOnce(&Device) -> Size2D<Au>,
{
    let size = get_size(context.device());
    Ratio::new(size.width.0 as f32, size.height.0 as f32)
}

/// https://drafts.csswg.org/mediaqueries-4/#aspect-ratio
fn eval_aspect_ratio(context: &Context) -> Ratio {
    eval_aspect_ratio_for(context, Device::au_viewport_size)
}

/// https://drafts.csswg.org/mediaqueries-4/#device-aspect-ratio
fn eval_device_aspect_ratio(context: &Context) -> Ratio {
    eval_aspect_ratio_for(context, device_size)
}

/// https://compat.spec.whatwg.org/#css-media-queries-webkit-device-pixel-ratio
fn eval_device_pixel_ratio(context: &Context) -> f32 {
    eval_resolution(context).dppx()
}

/// https://drafts.csswg.org/mediaqueries-4/#orientation
fn eval_orientation(context: &Context, value: Option<Orientation>) -> bool {
    Orientation::eval(context.device().au_viewport_size(), value)
}

/// FIXME: There's no spec for `-moz-device-orientation`.
fn eval_device_orientation(context: &Context, value: Option<Orientation>) -> bool {
    Orientation::eval(device_size(context.device()), value)
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
fn eval_display_mode(context: &Context, query_value: Option<DisplayMode>) -> bool {
    match query_value {
        Some(v) => v == unsafe { bindings::Gecko_MediaFeatures_GetDisplayMode(context.device().document()) },
        None => true,
    }
}

/// https://drafts.csswg.org/mediaqueries-4/#grid
fn eval_grid(_: &Context) -> bool {
    // Gecko doesn't support grid devices (e.g., ttys), so the 'grid' feature
    // is always 0.
    false
}

/// https://compat.spec.whatwg.org/#css-media-queries-webkit-transform-3d
fn eval_transform_3d(_: &Context) -> bool {
    true
}

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum Scan {
    Progressive,
    Interlace,
}

/// https://drafts.csswg.org/mediaqueries-4/#scan
fn eval_scan(_: &Context, _: Option<Scan>) -> bool {
    // Since Gecko doesn't support the 'tv' media type, the 'scan' feature never
    // matches.
    false
}

/// https://drafts.csswg.org/mediaqueries-4/#color
fn eval_color(context: &Context) -> u32 {
    unsafe { bindings::Gecko_MediaFeatures_GetColorDepth(context.device().document()) }
}

/// https://drafts.csswg.org/mediaqueries-4/#color-index
fn eval_color_index(_: &Context) -> u32 {
    // We should return zero if the device does not use a color lookup table.
    0
}

/// https://drafts.csswg.org/mediaqueries-4/#monochrome
fn eval_monochrome(context: &Context) -> u32 {
    // For color devices we should return 0.
    unsafe { bindings::Gecko_MediaFeatures_GetMonochromeBitsPerPixel(context.device().document()) }
}

/// https://drafts.csswg.org/mediaqueries-4/#resolution
fn eval_resolution(context: &Context) -> Resolution {
    let resolution_dppx = unsafe { bindings::Gecko_MediaFeatures_GetResolution(context.device().document()) };
    Resolution::from_dppx(resolution_dppx)
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

/// Values for the dynamic-range and video-dynamic-range media features.
/// https://drafts.csswg.org/mediaqueries-5/#dynamic-range
/// This implements PartialOrd so that lower values will correctly match
/// higher capabilities.
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, PartialEq, PartialOrd, ToCss)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum DynamicRange {
    Standard,
    High,
}

/// https://drafts.csswg.org/mediaqueries-5/#prefers-reduced-motion
fn eval_prefers_reduced_motion(context: &Context, query_value: Option<PrefersReducedMotion>) -> bool {
    let prefers_reduced =
        unsafe { bindings::Gecko_MediaFeatures_PrefersReducedMotion(context.device().document()) };
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
    /// More contrast is preferred.
    More,
    /// Low contrast is preferred.
    Less,
    /// Custom (not more, not less).
    Custom,
    /// The default value if neither high or low contrast is enabled.
    NoPreference,
}

/// https://drafts.csswg.org/mediaqueries-5/#prefers-contrast
fn eval_prefers_contrast(context: &Context, query_value: Option<PrefersContrast>) -> bool {
    let prefers_contrast =
        unsafe { bindings::Gecko_MediaFeatures_PrefersContrast(context.device().document()) };
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
fn eval_forced_colors(context: &Context, query_value: Option<ForcedColors>) -> bool {
    let forced = !context.device().use_document_colors();
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
fn eval_overflow_block(context: &Context, query_value: Option<OverflowBlock>) -> bool {
    // For the time being, assume that printing (including previews)
    // is the only time when we paginate, and we are otherwise always
    // scrolling. This is true at the moment in Firefox, but may need
    // updating in the future (e.g., ebook readers built with Stylo, a
    // billboard mode that doesn't support overflow at all).
    //
    // If this ever changes, don't forget to change eval_overflow_inline too.
    let scrolling = context.device().media_type() != MediaType::print();
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
fn eval_overflow_inline(context: &Context, query_value: Option<OverflowInline>) -> bool {
    // See the note in eval_overflow_block.
    let scrolling = context.device().media_type() != MediaType::print();
    let query_value = match query_value {
        Some(v) => v,
        None => return scrolling,
    };

    match query_value {
        OverflowInline::None => !scrolling,
        OverflowInline::Scroll => scrolling,
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum Update {
    None,
    Slow,
    Fast,
}

/// https://drafts.csswg.org/mediaqueries-4/#update
fn eval_update(context: &Context, query_value: Option<Update>) -> bool {
    // This has similar caveats to those described in eval_overflow_block.
    // For now, we report that print (incl. print media simulation,
    // which can in fact update but is limited to the developer tools)
    // is `update: none` and that all other contexts are `update: fast`,
    // which may not be true for future platforms, like e-ink devices.
    let can_update = context.device().media_type() != MediaType::print();
    let query_value = match query_value {
        Some(v) => v,
        None => return can_update,
    };

    match query_value {
        Update::None => !can_update,
        Update::Slow => false,
        Update::Fast => can_update,
    }
}

fn do_eval_prefers_color_scheme(
    context: &Context,
    use_content: bool,
    query_value: Option<PrefersColorScheme>,
) -> bool {
    let prefers_color_scheme =
        unsafe { bindings::Gecko_MediaFeatures_PrefersColorScheme(context.device().document(), use_content) };
    match query_value {
        Some(v) => prefers_color_scheme == v,
        None => true,
    }
}

/// https://drafts.csswg.org/mediaqueries-5/#prefers-color-scheme
fn eval_prefers_color_scheme(context: &Context, query_value: Option<PrefersColorScheme>) -> bool {
    do_eval_prefers_color_scheme(context, /* use_content = */ false, query_value)
}

fn eval_content_prefers_color_scheme(
    context: &Context,
    query_value: Option<PrefersColorScheme>,
) -> bool {
    do_eval_prefers_color_scheme(context, /* use_content = */ true, query_value)
}

/// https://drafts.csswg.org/mediaqueries-5/#dynamic-range
fn eval_dynamic_range(context: &Context, query_value: Option<DynamicRange>) -> bool {
    let dynamic_range =
        unsafe { bindings::Gecko_MediaFeatures_DynamicRange(context.device().document()) };
    match query_value {
        Some(v) => dynamic_range >= v,
        None => false,
    }
}
/// https://drafts.csswg.org/mediaqueries-5/#video-dynamic-range
fn eval_video_dynamic_range(context: &Context, query_value: Option<DynamicRange>) -> bool {
    let dynamic_range =
        unsafe { bindings::Gecko_MediaFeatures_VideoDynamicRange(context.device().document()) };
    match query_value {
        Some(v) => dynamic_range >= v,
        None => false,
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

fn primary_pointer_capabilities(context: &Context) -> PointerCapabilities {
    PointerCapabilities::from_bits_truncate(unsafe {
        bindings::Gecko_MediaFeatures_PrimaryPointerCapabilities(context.device().document())
    })
}

fn all_pointer_capabilities(context: &Context) -> PointerCapabilities {
    PointerCapabilities::from_bits_truncate(unsafe {
        bindings::Gecko_MediaFeatures_AllPointerCapabilities(context.device().document())
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
fn eval_pointer(context: &Context, query_value: Option<Pointer>) -> bool {
    eval_pointer_capabilities(query_value, primary_pointer_capabilities(context))
}

/// https://drafts.csswg.org/mediaqueries-4/#descdef-media-any-pointer
fn eval_any_pointer(context: &Context, query_value: Option<Pointer>) -> bool {
    eval_pointer_capabilities(query_value, all_pointer_capabilities(context))
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
fn eval_hover(context: &Context, query_value: Option<Hover>) -> bool {
    eval_hover_capabilities(query_value, primary_pointer_capabilities(context))
}

/// https://drafts.csswg.org/mediaqueries-4/#descdef-media-any-hover
fn eval_any_hover(context: &Context, query_value: Option<Hover>) -> bool {
    eval_hover_capabilities(query_value, all_pointer_capabilities(context))
}

fn eval_moz_is_glyph(context: &Context) -> bool {
    context.device().document().mIsSVGGlyphsDocument()
}

fn eval_moz_print_preview(context: &Context) -> bool {
    let is_print_preview = context.device().is_print_preview();
    if is_print_preview {
        debug_assert_eq!(context.device().media_type(), MediaType::print());
    }
    is_print_preview
}

fn eval_moz_non_native_content_theme(context: &Context) -> bool {
    unsafe { bindings::Gecko_MediaFeatures_ShouldAvoidNativeTheme(context.device().document()) }
}

fn eval_moz_is_resource_document(context: &Context) -> bool {
    unsafe { bindings::Gecko_MediaFeatures_IsResourceDocument(context.device().document()) }
}

/// Allows front-end CSS to discern platform via media queries.
#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
pub enum Platform {
    /// Matches any Android version.
    Android,
    /// For our purposes here, "linux" is just "gtk" (so unix-but-not-mac).
    /// There's no need for our front-end code to differentiate between those
    /// platforms and they already use the "linux" string elsewhere (e.g.,
    /// toolkit/themes/linux).
    Linux,
    /// Matches any macOS version.
    Macos,
    /// Matches any Windows version.
    Windows,
    /// Matches only Windows 7.
    WindowsWin7,
    /// Matches only Windows 8.
    WindowsWin8,
    /// Matches windows 10 and actually matches windows 11 too, as of right now.
    WindowsWin10,
}

fn eval_moz_platform(_: &Context, query_value: Option<Platform>) -> bool {
    let query_value = match query_value {
        Some(v) => v,
        None => return false,
    };

    unsafe { bindings::Gecko_MediaFeatures_MatchesPlatform(query_value) }
}

fn eval_moz_windows_non_native_menus(_: &Context) -> bool {
    unsafe { bindings::Gecko_MediaFeatures_WindowsNonNativeMenus() }
}

fn eval_moz_overlay_scrollbars(context: &Context) -> bool {
    unsafe { bindings::Gecko_MediaFeatures_UseOverlayScrollbars(context.device().document()) }
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
        fn __eval(_: &Context) -> bool {
            $get_value(bindings::LookAndFeel_IntID::$int_id as i32)
        }

        feature!(
            $feature_name,
            AllowsRanges::No,
            Evaluator::BoolInteger(__eval),
            FeatureFlags::CHROME_AND_UA_ONLY,
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
#[allow(unused)]
macro_rules! bool_pref_feature {
    ($feature_name:expr, $pref:tt) => {{
        fn __eval(_: &Context) -> bool {
            static_prefs::pref!($pref)
        }

        feature!(
            $feature_name,
            AllowsRanges::No,
            Evaluator::BoolInteger(__eval),
            FeatureFlags::CHROME_AND_UA_ONLY,
        )
    }};
}

/// Adding new media features requires (1) adding the new feature to this
/// array, with appropriate entries (and potentially any new code needed
/// to support new types in these entries and (2) ensuring that either
/// nsPresContext::MediaFeatureValuesChanged is called when the value that
/// would be returned by the evaluator function could change.
pub static MEDIA_FEATURES: [QueryFeatureDescription; 61] = [
    feature!(
        atom!("width"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_width),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("height"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_height),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("aspect-ratio"),
        AllowsRanges::Yes,
        Evaluator::NumberRatio(eval_aspect_ratio),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("orientation"),
        AllowsRanges::No,
        keyword_evaluator!(eval_orientation, Orientation),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("device-width"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_device_width),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("device-height"),
        AllowsRanges::Yes,
        Evaluator::Length(eval_device_height),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("device-aspect-ratio"),
        AllowsRanges::Yes,
        Evaluator::NumberRatio(eval_device_aspect_ratio),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("-moz-device-orientation"),
        AllowsRanges::No,
        keyword_evaluator!(eval_device_orientation, Orientation),
        FeatureFlags::empty(),
    ),
    // Webkit extensions that we support for de-facto web compatibility.
    // -webkit-{min|max}-device-pixel-ratio (controlled with its own pref):
    feature!(
        atom!("device-pixel-ratio"),
        AllowsRanges::Yes,
        Evaluator::Float(eval_device_pixel_ratio),
        FeatureFlags::WEBKIT_PREFIX,
    ),
    // -webkit-transform-3d.
    feature!(
        atom!("transform-3d"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_transform_3d),
        FeatureFlags::WEBKIT_PREFIX,
    ),
    feature!(
        atom!("-moz-device-pixel-ratio"),
        AllowsRanges::Yes,
        Evaluator::Float(eval_device_pixel_ratio),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("resolution"),
        AllowsRanges::Yes,
        Evaluator::Resolution(eval_resolution),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("display-mode"),
        AllowsRanges::No,
        keyword_evaluator!(eval_display_mode, DisplayMode),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("grid"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_grid),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("scan"),
        AllowsRanges::No,
        keyword_evaluator!(eval_scan, Scan),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("color"),
        AllowsRanges::Yes,
        Evaluator::Integer(eval_color),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("color-index"),
        AllowsRanges::Yes,
        Evaluator::Integer(eval_color_index),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("monochrome"),
        AllowsRanges::Yes,
        Evaluator::Integer(eval_monochrome),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("prefers-reduced-motion"),
        AllowsRanges::No,
        keyword_evaluator!(eval_prefers_reduced_motion, PrefersReducedMotion),
        FeatureFlags::empty(),
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
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("forced-colors"),
        AllowsRanges::No,
        keyword_evaluator!(eval_forced_colors, ForcedColors),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("overflow-block"),
        AllowsRanges::No,
        keyword_evaluator!(eval_overflow_block, OverflowBlock),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("overflow-inline"),
        AllowsRanges::No,
        keyword_evaluator!(eval_overflow_inline, OverflowInline),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("update"),
        AllowsRanges::No,
        keyword_evaluator!(eval_update, Update),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("prefers-color-scheme"),
        AllowsRanges::No,
        keyword_evaluator!(eval_prefers_color_scheme, PrefersColorScheme),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("dynamic-range"),
        AllowsRanges::No,
        keyword_evaluator!(eval_dynamic_range, DynamicRange),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("video-dynamic-range"),
        AllowsRanges::No,
        keyword_evaluator!(eval_video_dynamic_range, DynamicRange),
        FeatureFlags::empty(),
    ),
    // Evaluates to the preferred color scheme for content. Only useful in
    // chrome context, where the chrome color-scheme and the content
    // color-scheme might differ.
    feature!(
        atom!("-moz-content-prefers-color-scheme"),
        AllowsRanges::No,
        keyword_evaluator!(eval_content_prefers_color_scheme, PrefersColorScheme),
        FeatureFlags::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("pointer"),
        AllowsRanges::No,
        keyword_evaluator!(eval_pointer, Pointer),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("any-pointer"),
        AllowsRanges::No,
        keyword_evaluator!(eval_any_pointer, Pointer),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("hover"),
        AllowsRanges::No,
        keyword_evaluator!(eval_hover, Hover),
        FeatureFlags::empty(),
    ),
    feature!(
        atom!("any-hover"),
        AllowsRanges::No,
        keyword_evaluator!(eval_any_hover, Hover),
        FeatureFlags::empty(),
    ),
    // Internal -moz-is-glyph media feature: applies only inside SVG glyphs.
    // Internal because it is really only useful in the user agent anyway
    // and therefore not worth standardizing.
    feature!(
        atom!("-moz-is-glyph"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_moz_is_glyph),
        FeatureFlags::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("-moz-is-resource-document"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_moz_is_resource_document),
        FeatureFlags::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("-moz-platform"),
        AllowsRanges::No,
        keyword_evaluator!(eval_moz_platform, Platform),
        FeatureFlags::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("-moz-print-preview"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_moz_print_preview),
        FeatureFlags::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("-moz-non-native-content-theme"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_moz_non_native_content_theme),
        FeatureFlags::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("-moz-windows-non-native-menus"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_moz_windows_non_native_menus),
        FeatureFlags::CHROME_AND_UA_ONLY,
    ),
    feature!(
        atom!("-moz-overlay-scrollbars"),
        AllowsRanges::No,
        Evaluator::BoolInteger(eval_moz_overlay_scrollbars),
        FeatureFlags::CHROME_AND_UA_ONLY,
    ),
    lnf_int_feature!(
        atom!("-moz-scrollbar-start-backward"),
        ScrollArrowStyle,
        get_scrollbar_start_backward
    ),
    lnf_int_feature!(
        atom!("-moz-scrollbar-start-forward"),
        ScrollArrowStyle,
        get_scrollbar_start_forward
    ),
    lnf_int_feature!(
        atom!("-moz-scrollbar-end-backward"),
        ScrollArrowStyle,
        get_scrollbar_end_backward
    ),
    lnf_int_feature!(
        atom!("-moz-scrollbar-end-forward"),
        ScrollArrowStyle,
        get_scrollbar_end_forward
    ),
    lnf_int_feature!(atom!("-moz-menubar-drag"), MenuBarDrag),
    lnf_int_feature!(atom!("-moz-windows-default-theme"), WindowsDefaultTheme),
    lnf_int_feature!(atom!("-moz-mac-graphite-theme"), MacGraphiteTheme),
    lnf_int_feature!(atom!("-moz-mac-big-sur-theme"), MacBigSurTheme),
    lnf_int_feature!(atom!("-moz-mac-rtl"), MacRTL),
    lnf_int_feature!(
        atom!("-moz-windows-accent-color-in-titlebar"),
        WindowsAccentColorInTitlebar
    ),
    lnf_int_feature!(atom!("-moz-windows-compositor"), DWMCompositor),
    lnf_int_feature!(atom!("-moz-windows-classic"), WindowsClassic),
    lnf_int_feature!(atom!("-moz-windows-glass"), WindowsGlass),
    lnf_int_feature!(atom!("-moz-swipe-animation-enabled"), SwipeAnimationEnabled),
    lnf_int_feature!(atom!("-moz-gtk-csd-available"), GTKCSDAvailable),
    lnf_int_feature!(atom!("-moz-gtk-csd-minimize-button"), GTKCSDMinimizeButton),
    lnf_int_feature!(atom!("-moz-gtk-csd-maximize-button"), GTKCSDMaximizeButton),
    lnf_int_feature!(atom!("-moz-gtk-csd-close-button"), GTKCSDCloseButton),
    lnf_int_feature!(
        atom!("-moz-gtk-csd-reversed-placement"),
        GTKCSDReversedPlacement
    ),
    lnf_int_feature!(atom!("-moz-system-dark-theme"), SystemUsesDarkTheme),
    bool_pref_feature!(atom!("-moz-box-flexbox-emulation"), "layout.css.moz-box-flexbox-emulation.enabled"),
    // media query for MathML Core's implementation of maction/semantics
    bool_pref_feature!(atom!("-moz-mathml-core-maction-and-semantics"), "mathml.legacy_maction_and_semantics_implementations.disabled"),
];
