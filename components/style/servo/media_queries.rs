/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Servo's media-query device and expression representation.

use app_units::Au;
use crate::custom_properties::CssEnvironment;
use crate::media_queries::media_feature::{AllowsRanges, ParsingRequirements};
use crate::media_queries::media_feature::{Evaluator, MediaFeatureDescription};
use crate::media_queries::media_feature_expression::RangeOrOperator;
use crate::media_queries::MediaType;
use crate::properties::ComputedValues;
use crate::values::computed::font::FontSize;
use crate::values::computed::CSSPixelLength;
use crate::values::KeyframesName;
use cssparser::RGBA;
use euclid::{Size2D, TypedScale, TypedSize2D};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use style_traits::viewport::ViewportConstraints;
use style_traits::{CSSPixel, DevicePixel};

/// A device is a structure that represents the current media a given document
/// is displayed in.
///
/// This is the struct against which media queries are evaluated.
#[derive(Debug, MallocSizeOf)]
pub struct Device {
    /// The current media type used by de device.
    media_type: MediaType,
    /// The current viewport size, in CSS pixels.
    viewport_size: TypedSize2D<f32, CSSPixel>,
    /// The current device pixel ratio, from CSS pixels to device pixels.
    device_pixel_ratio: TypedScale<f32, CSSPixel, DevicePixel>,

    /// The font size of the root element
    /// This is set when computing the style of the root
    /// element, and used for rem units in other elements
    ///
    /// When computing the style of the root element, there can't be any
    /// other style being computed at the same time, given we need the style of
    /// the parent to compute everything else. So it is correct to just use
    /// a relaxed atomic here.
    #[ignore_malloc_size_of = "Pure stack type"]
    root_font_size: AtomicIsize,
    /// Whether any styles computed in the document relied on the root font-size
    /// by using rem units.
    #[ignore_malloc_size_of = "Pure stack type"]
    used_root_font_size: AtomicBool,
    /// Whether any styles computed in the document relied on the viewport size.
    #[ignore_malloc_size_of = "Pure stack type"]
    used_viewport_units: AtomicBool,
    /// The CssEnvironment object responsible of getting CSS environment
    /// variables.
    environment: CssEnvironment,
}

impl Device {
    /// Trivially construct a new `Device`.
    pub fn new(
        media_type: MediaType,
        viewport_size: TypedSize2D<f32, CSSPixel>,
        device_pixel_ratio: TypedScale<f32, CSSPixel, DevicePixel>,
    ) -> Device {
        Device {
            media_type,
            viewport_size,
            device_pixel_ratio,
            // FIXME(bz): Seems dubious?
            root_font_size: AtomicIsize::new(FontSize::medium().size().0 as isize),
            used_root_font_size: AtomicBool::new(false),
            used_viewport_units: AtomicBool::new(false),
            environment: CssEnvironment,
        }
    }

    /// Get the relevant environment to resolve `env()` functions.
    #[inline]
    pub fn environment(&self) -> &CssEnvironment {
        &self.environment
    }

    /// Return the default computed values for this device.
    pub fn default_computed_values(&self) -> &ComputedValues {
        // FIXME(bz): This isn't really right, but it's no more wrong
        // than what we used to do.  See
        // https://github.com/servo/servo/issues/14773 for fixing it properly.
        ComputedValues::initial_values()
    }

    /// Get the font size of the root element (for rem)
    pub fn root_font_size(&self) -> Au {
        self.used_root_font_size.store(true, Ordering::Relaxed);
        Au::new(self.root_font_size.load(Ordering::Relaxed) as i32)
    }

    /// Set the font size of the root element (for rem)
    pub fn set_root_font_size(&self, size: Au) {
        self.root_font_size
            .store(size.0 as isize, Ordering::Relaxed)
    }

    /// Sets the body text color for the "inherit color from body" quirk.
    ///
    /// <https://quirks.spec.whatwg.org/#the-tables-inherit-color-from-body-quirk>
    pub fn set_body_text_color(&self, _color: RGBA) {
        // Servo doesn't implement this quirk (yet)
    }

    /// Whether a given animation name may be referenced from style.
    pub fn animation_name_may_be_referenced(&self, _: &KeyframesName) -> bool {
        // Assume it is, since we don't have any good way to prove it's not.
        true
    }

    /// Returns whether we ever looked up the root font size of the Device.
    pub fn used_root_font_size(&self) -> bool {
        self.used_root_font_size.load(Ordering::Relaxed)
    }

    /// Returns the viewport size of the current device in app units, needed,
    /// among other things, to resolve viewport units.
    #[inline]
    pub fn au_viewport_size(&self) -> Size2D<Au> {
        Size2D::new(
            Au::from_f32_px(self.viewport_size.width),
            Au::from_f32_px(self.viewport_size.height),
        )
    }

    /// Like the above, but records that we've used viewport units.
    pub fn au_viewport_size_for_viewport_unit_resolution(&self) -> Size2D<Au> {
        self.used_viewport_units.store(true, Ordering::Relaxed);
        self.au_viewport_size()
    }

    /// Whether viewport units were used since the last device change.
    pub fn used_viewport_units(&self) -> bool {
        self.used_viewport_units.load(Ordering::Relaxed)
    }

    /// Returns the device pixel ratio.
    pub fn device_pixel_ratio(&self) -> TypedScale<f32, CSSPixel, DevicePixel> {
        self.device_pixel_ratio
    }

    /// Take into account a viewport rule taken from the stylesheets.
    pub fn account_for_viewport_rule(&mut self, constraints: &ViewportConstraints) {
        self.viewport_size = constraints.size;
    }

    /// Return the media type of the current device.
    pub fn media_type(&self) -> MediaType {
        self.media_type.clone()
    }

    /// Returns whether document colors are enabled.
    pub fn use_document_colors(&self) -> bool {
        true
    }

    /// Returns the default background color.
    pub fn default_background_color(&self) -> RGBA {
        RGBA::new(255, 255, 255, 255)
    }
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

#[derive(Clone, Copy, Debug, FromPrimitive, Parse, ToCss)]
#[repr(u8)]
enum Scan {
    Progressive,
    Interlace,
}

/// https://drafts.csswg.org/mediaqueries-4/#scan
fn eval_scan(_: &Device, _: Option<Scan>) -> bool {
    // Since we doesn't support the 'tv' media type, the 'scan' feature never
    // matches.
    false
}

lazy_static! {
    /// A list with all the media features that Servo supports.
    pub static ref MEDIA_FEATURES: [MediaFeatureDescription; 2] = [
        feature!(
            atom!("width"),
            AllowsRanges::Yes,
            Evaluator::Length(eval_width),
            ParsingRequirements::empty(),
        ),
        feature!(
            atom!("scan"),
            AllowsRanges::No,
            keyword_evaluator!(eval_scan, Scan),
            ParsingRequirements::empty(),
        ),
    ];
}
