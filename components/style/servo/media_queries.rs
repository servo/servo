/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's media-query device and expression representation.

use app_units::Au;
use context::QuirksMode;
use cssparser::{Parser, RGBA};
use euclid::{Size2D, TypedScale, TypedSize2D};
use media_queries::MediaType;
use parser::ParserContext;
use properties::ComputedValues;
use selectors::parser::SelectorParseErrorKind;
use std::fmt::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use style_traits::{CSSPixel, CssWriter, DevicePixel, ParseError, ToCss};
use style_traits::viewport::ViewportConstraints;
use values::{specified, KeyframesName};
use values::computed::{self, ToComputedValue};
use values::computed::font::FontSize;

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

/// A expression kind servo understands and parses.
///
/// Only `pub` for unit testing, please don't use it directly!
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub enum ExpressionKind {
    /// <http://dev.w3.org/csswg/mediaqueries-3/#width>
    Width(Range<specified::Length>),
}

/// A single expression a per:
///
/// <http://dev.w3.org/csswg/mediaqueries-3/#media1>
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
pub struct MediaFeatureExpression(pub ExpressionKind);

impl MediaFeatureExpression {
    /// The kind of expression we're, just for unit testing.
    ///
    /// Eventually this will become servo-only.
    pub fn kind_for_testing(&self) -> &ExpressionKind {
        &self.0
    }

    /// Parse a media expression of the form:
    ///
    /// ```
    /// media-feature: media-value
    /// ```
    ///
    /// Only supports width ranges for now.
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_parenthesis_block()?;
        input.parse_nested_block(|input| {
            Self::parse_in_parenthesis_block(context, input)
        })
    }

    /// Parse a media range expression where we've already consumed the
    /// parenthesis.
    pub fn parse_in_parenthesis_block<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        let name = input.expect_ident_cloned()?;
        input.expect_colon()?;
        // TODO: Handle other media features
        Ok(MediaFeatureExpression(match_ignore_ascii_case! { &name,
            "min-width" => {
                ExpressionKind::Width(Range::Min(specified::Length::parse_non_negative(context, input)?))
            },
            "max-width" => {
                ExpressionKind::Width(Range::Max(specified::Length::parse_non_negative(context, input)?))
            },
            "width" => {
                ExpressionKind::Width(Range::Eq(specified::Length::parse_non_negative(context, input)?))
            },
            _ => return Err(input.new_custom_error(SelectorParseErrorKind::UnexpectedIdent(name)))
        }))
    }

    /// Evaluate this expression and return whether it matches the current
    /// device.
    pub fn matches(&self, device: &Device, quirks_mode: QuirksMode) -> bool {
        let viewport_size = device.au_viewport_size();
        let value = viewport_size.width;
        match self.0 {
            ExpressionKind::Width(ref range) => {
                match range.to_computed_range(device, quirks_mode) {
                    Range::Min(ref width) => value >= *width,
                    Range::Max(ref width) => value <= *width,
                    Range::Eq(ref width) => value == *width,
                }
            },
        }
    }
}

impl ToCss for MediaFeatureExpression {
    fn to_css<W>(&self, dest: &mut CssWriter<W>) -> fmt::Result
    where
        W: Write,
    {
        let (s, l) = match self.0 {
            ExpressionKind::Width(Range::Min(ref l)) => ("(min-width: ", l),
            ExpressionKind::Width(Range::Max(ref l)) => ("(max-width: ", l),
            ExpressionKind::Width(Range::Eq(ref l)) => ("(width: ", l),
        };
        dest.write_str(s)?;
        l.to_css(dest)?;
        dest.write_char(')')
    }
}

/// An enumeration that represents a ranged value.
///
/// Only public for testing, implementation details of `MediaFeatureExpression`
/// may change for Stylo.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "servo", derive(MallocSizeOf))]
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
        computed::Context::for_media_query_evaluation(device, quirks_mode, |context| match *self {
            Range::Min(ref width) => Range::Min(Au::from(width.to_computed_value(&context))),
            Range::Max(ref width) => Range::Max(Au::from(width.to_computed_value(&context))),
            Range::Eq(ref width) => Range::Eq(Au::from(width.to_computed_value(&context))),
        })
    }
}
