/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Servo's media-query device and expression representation.

use app_units::Au;
use context::QuirksMode;
use cssparser::Parser;
use euclid::{Size2D, TypedSize2D};
use font_metrics::ServoMetricsProvider;
use media_queries::MediaType;
use parser::ParserContext;
use properties::ComputedValues;
use std::fmt;
use style_traits::{CSSPixel, ToCss};
use style_traits::viewport::ViewportConstraints;
use values::computed::{self, ToComputedValue};
use values::specified;

/// A device is a structure that represents the current media a given document
/// is displayed in.
///
/// This is the struct against which media queries are evaluated.
#[derive(Debug, HeapSizeOf)]
pub struct Device {
    /// The current media type used by de device.
    media_type: MediaType,
    /// The current viewport size, in CSS pixels.
    viewport_size: TypedSize2D<f32, CSSPixel>,
}

impl Device {
    /// Trivially construct a new `Device`.
    pub fn new(media_type: MediaType,
               viewport_size: TypedSize2D<f32, CSSPixel>)
               -> Device {
        Device {
            media_type: media_type,
            viewport_size: viewport_size,
        }
    }

    /// Return the default computed values for this device.
    pub fn default_computed_values(&self) -> &ComputedValues {
        // FIXME(bz): This isn't really right, but it's no more wrong
        // than what we used to do.  See
        // https://github.com/servo/servo/issues/14773 for fixing it properly.
        ComputedValues::initial_values()
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

    /// Take into account a viewport rule taken from the stylesheets.
    pub fn account_for_viewport_rule(&mut self, constraints: &ViewportConstraints) {
        self.viewport_size = constraints.size;
    }

    /// Return the media type of the current device.
    pub fn media_type(&self) -> MediaType {
        self.media_type.clone()
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
    pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Self, ()> {
        try!(input.expect_parenthesis_block());
        input.parse_nested_block(|input| {
            let name = try!(input.expect_ident());
            try!(input.expect_colon());
            // TODO: Handle other media features
            Ok(Expression(match_ignore_ascii_case! { &name,
                "min-width" => {
                    ExpressionKind::Width(Range::Min(try!(specified::Length::parse_non_negative(context, input))))
                },
                "max-width" => {
                    ExpressionKind::Width(Range::Max(try!(specified::Length::parse_non_negative(context, input))))
                },
                "width" => {
                    ExpressionKind::Width(Range::Eq(try!(specified::Length::parse_non_negative(context, input))))
                },
                _ => return Err(())
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
        try!(write!(dest, "("));
        let (mm, l) = match self.0 {
            ExpressionKind::Width(Range::Min(ref l)) => ("min-", l),
            ExpressionKind::Width(Range::Max(ref l)) => ("max-", l),
            ExpressionKind::Width(Range::Eq(ref l)) => ("", l),
        };
        try!(write!(dest, "{}width: ", mm));
        try!(l.to_css(dest));
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
            device: device,
            inherited_style: default_values,
            layout_parent_style: default_values,
            // This cloning business is kind of dumb.... It's because Context
            // insists on having an actual ComputedValues inside itself.
            style: default_values.clone(),
            // Servo doesn't support font metrics
            // A real provider will be needed here once we do; since
            // ch units can exist in media queries.
            font_metrics_provider: &ServoMetricsProvider,
            in_media_query: true,
            quirks_mode: quirks_mode,
        };

        match *self {
            Range::Min(ref width) => Range::Min(width.to_computed_value(&context)),
            Range::Max(ref width) => Range::Max(width.to_computed_value(&context)),
            Range::Eq(ref width) => Range::Eq(width.to_computed_value(&context))
        }
    }
}
