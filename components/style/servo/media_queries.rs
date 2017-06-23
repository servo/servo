/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's media-query device and expression representation.

use app_units::Au;
use context::QuirksMode;
use cssparser::{Parser, RGBA};
use euclid::{ScaleFactor, Size2D, TypedSize2D};
use font_metrics::ServoMetricsProvider;
use media_queries::MediaType;
use parser::ParserContext;
use properties::{ComputedValues, StyleBuilder};
use properties::longhands::font_size;
use selectors::parser::SelectorParseError;
use std::fmt;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use style_traits::{CSSPixel, DevicePixel, ToCss, ParseError};
use style_traits::viewport::ViewportConstraints;
use values::computed::{self, ToComputedValue};
use values::specified;

/// A device is a structure that represents the current media a given document
/// is displayed in.
///
/// This is the struct against which media queries are evaluated.
#[derive(HeapSizeOf)]
pub struct Device {
    /// The current media type used by de device.
    media_type: MediaType,
    /// The current viewport size, in CSS pixels.
    viewport_size: TypedSize2D<f32, CSSPixel>,
    /// The current device pixel ratio, from CSS pixels to device pixels.
    device_pixel_ratio: ScaleFactor<f32, CSSPixel, DevicePixel>,

    /// The font size of the root element
    /// This is set when computing the style of the root
    /// element, and used for rem units in other elements
    ///
    /// When computing the style of the root element, there can't be any
    /// other style being computed at the same time, given we need the style of
    /// the parent to compute everything else. So it is correct to just use
    /// a relaxed atomic here.
    #[ignore_heap_size_of = "Pure stack type"]
    root_font_size: AtomicIsize,
    /// Whether any styles computed in the document relied on the root font-size
    /// by using rem units.
    #[ignore_heap_size_of = "Pure stack type"]
    used_root_font_size: AtomicBool,
}

impl Device {
    /// Trivially construct a new `Device`.
    pub fn new(media_type: MediaType,
               viewport_size: TypedSize2D<f32, CSSPixel>,
               device_pixel_ratio: ScaleFactor<f32, CSSPixel, DevicePixel>)
               -> Device {
        Device {
            media_type: media_type,
            viewport_size: viewport_size,
            device_pixel_ratio: device_pixel_ratio,
            root_font_size: AtomicIsize::new(font_size::get_initial_value().0 as isize), // FIXME(bz): Seems dubious?
            used_root_font_size: AtomicBool::new(false),
        }
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
        self.root_font_size.store(size.0 as isize, Ordering::Relaxed)
    }

    /// Returns whether we ever looked up the root font size of the Device.
    pub fn used_root_font_size(&self) -> bool {
        self.used_root_font_size.load(Ordering::Relaxed)
    }

    /// Returns the viewport size of the current device in app units, needed,
    /// among other things, to resolve viewport units.
    #[inline]
    pub fn au_viewport_size(&self) -> Size2D<Au> {
        Size2D::new(Au::from_f32_px(self.viewport_size.width),
                    Au::from_f32_px(self.viewport_size.height))
    }

    /// Returns the viewport size in pixels.
    #[inline]
    pub fn px_viewport_size(&self) -> TypedSize2D<f32, CSSPixel> {
        self.viewport_size
    }

    /// Returns the device pixel ratio.
    pub fn device_pixel_ratio(&self) -> ScaleFactor<f32, CSSPixel, DevicePixel> {
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

/// A expression kind servo understands and parses.
///
/// Only `pub` for unit testing, please don't use it directly!
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum ExpressionKind {
    /// http://dev.w3.org/csswg/mediaqueries-3/#width
    Width(Range<specified::Length>),
}

/// A single expression a per:
///
/// http://dev.w3.org/csswg/mediaqueries-3/#media1
#[derive(PartialEq, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub struct Expression(ExpressionKind);

impl Expression {
    /// The kind of expression we're, just for unit testing.
    ///
    /// Eventually this will become servo-only.
    pub fn kind_for_testing(&self) -> &ExpressionKind {
        &self.0
    }

    /// Parse a media expression of the form:
    ///
    /// ```
    /// (media-feature: media-value)
    /// ```
    ///
    /// Only supports width and width ranges for now.
    pub fn parse<'i, 't>(context: &ParserContext, input: &mut Parser<'i, 't>)
                         -> Result<Self, ParseError<'i>> {
        input.expect_parenthesis_block()?;
        input.parse_nested_block(|input| {
            let name = input.expect_ident()?;
            input.expect_colon()?;
            // TODO: Handle other media features
            Ok(Expression(match_ignore_ascii_case! { &name,
                "min-width" => {
                    ExpressionKind::Width(Range::Min(specified::Length::parse_non_negative(context, input)?))
                },
                "max-width" => {
                    ExpressionKind::Width(Range::Max(specified::Length::parse_non_negative(context, input)?))
                },
                "width" => {
                    ExpressionKind::Width(Range::Eq(specified::Length::parse_non_negative(context, input)?))
                },
                _ => return Err(SelectorParseError::UnexpectedIdent(name.clone()).into())
            }))
        })
    }

    /// Evaluate this expression and return whether it matches the current
    /// device.
    pub fn matches(&self, device: &Device, quirks_mode: QuirksMode) -> bool {
        let viewport_size = device.au_viewport_size();
        let value = viewport_size.width;
        match self.0 {
            ExpressionKind::Width(ref range) => {
                match range.to_computed_range(device, quirks_mode) {
                    Range::Min(ref width) => { value >= *width },
                    Range::Max(ref width) => { value <= *width },
                    Range::Eq(ref width) => { value == *width },
                }
            }
        }
    }
}

impl ToCss for Expression {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        write!(dest, "(")?;
        let (mm, l) = match self.0 {
            ExpressionKind::Width(Range::Min(ref l)) => ("min-", l),
            ExpressionKind::Width(Range::Max(ref l)) => ("max-", l),
            ExpressionKind::Width(Range::Eq(ref l)) => ("", l),
        };
        write!(dest, "{}width: ", mm)?;
        l.to_css(dest)?;
        write!(dest, ")")
    }
}

/// An enumeration that represents a ranged value.
///
/// Only public for testing, implementation details of `Expression` may change
/// for Stylo.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
#[cfg_attr(feature = "servo", derive(HeapSizeOf))]
pub enum Range<T> {
    /// At least the inner value.
    Min(T),
    /// At most the inner value.
    Max(T),
    /// Exactly the inner value.
    Eq(T),
}

impl Range<specified::Length> {
    fn to_computed_range(&self, device: &Device, quirks_mode: QuirksMode) -> Range<Au> {
        let default_values = device.default_computed_values();
        // http://dev.w3.org/csswg/mediaqueries3/#units
        // em units are relative to the initial font-size.
        let context = computed::Context {
            is_root_element: false,
            builder: StyleBuilder::for_derived_style(device, default_values, None, None),
            // Servo doesn't support font metrics
            // A real provider will be needed here once we do; since
            // ch units can exist in media queries.
            font_metrics_provider: &ServoMetricsProvider,
            in_media_query: true,
            cached_system_font: None,
            quirks_mode: quirks_mode,
        };

        match *self {
            Range::Min(ref width) => Range::Min(width.to_computed_value(&context)),
            Range::Max(ref width) => Range::Max(width.to_computed_value(&context)),
            Range::Eq(ref width) => Range::Eq(width.to_computed_value(&context))
        }
    }
}
