/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko's media-query device and expression representation.

use app_units::Au;
use cssparser::{Parser, Token};
use euclid::Size2D;
use gecko_bindings::bindings;
use gecko_bindings::structs::{nsMediaExpression_Range, nsMediaFeature};
use gecko_bindings::structs::{nsMediaFeature_ValueType, nsMediaFeature_RangeType, nsMediaFeature_RequirementFlags};
use gecko_bindings::structs::RawGeckoPresContextOwned;
use media_queries::MediaType;
use properties::ComputedValues;
use std::ascii::AsciiExt;
use std::fmt;
use std::sync::Arc;
use string_cache::Atom;
use style_traits::ToCss;
use style_traits::viewport::ViewportConstraints;
use values::{CSSFloat, specified};

/// The `Device` in Gecko wraps a pres context, has a default values computed,
/// and contains all the viewport rule state.
pub struct Device {
    /// NB: The pres context lifetime is tied to the styleset, who owns the
    /// stylist, and thus the `Device`, so having a raw pres context pointer
    /// here is fine.
    pres_context: RawGeckoPresContextOwned,
    default_values: Arc<ComputedValues>,
    viewport_override: Option<ViewportConstraints>,
}

impl Device {
    /// Trivially constructs a new `Device`.
    pub fn new(pres_context: RawGeckoPresContextOwned) -> Self {
        assert!(!pres_context.is_null());
        Device {
            pres_context: pres_context,
            default_values: ComputedValues::default_values(unsafe { &*pres_context }),
            viewport_override: None,
        }
    }

    /// Tells the device that a new viewport rule has been found, and stores the
    /// relevant viewport constraints.
    pub fn account_for_viewport_rule(&mut self,
                                     constraints: &ViewportConstraints) {
        self.viewport_override = Some(constraints.clone());
    }

    /// Returns the default computed values as a reference, in order to match
    /// Servo.
    pub fn default_values(&self) -> &ComputedValues {
        &*self.default_values
    }

    /// Returns the default computed values as an `Arc`, in order to avoid
    /// clones.
    pub fn default_values_arc(&self) -> &Arc<ComputedValues> {
        &self.default_values
    }

    /// Recreates all the temporary state that the `Device` stores.
    ///
    /// This includes the viewport override from `@viewport` rules, and also the
    /// default computed values.
    pub fn reset(&mut self) {
        // NB: A following stylesheet flush will populate this if appropriate.
        self.viewport_override = None;
        self.default_values = ComputedValues::default_values(unsafe { &*self.pres_context });
    }

    /// Returns the current media type of the device.
    pub fn media_type(&self) -> MediaType {
        // TODO
        MediaType::Screen
    }

    /// Returns the current viewport size in app units.
    pub fn au_viewport_size(&self) -> Size2D<Au> {
        self.viewport_override.as_ref().map(|v| {
            Size2D::new(Au::from_f32_px(v.size.width),
                        Au::from_f32_px(v.size.height))
        }).unwrap_or_else(|| {
            // TODO(emilio): Grab from pres context.
            Size2D::new(Au::from_f32_px(1024.0),
                        Au::from_f32_px(768.0))
        })
    }
}

unsafe impl Sync for Device {}
unsafe impl Send for Device {}

/// A expression for gecko contains a reference to the media feature, the value
/// the media query contained, and the range to evaluate.
#[derive(Debug, Clone)]
pub struct Expression {
    feature: &'static nsMediaFeature,
    value: MediaExpressionValue,
    range: nsMediaExpression_Range
}

impl ToCss for Expression {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        dest.write_str("(")?;
        match self.range {
            nsMediaExpression_Range::eMin => dest.write_str("min-")?,
            nsMediaExpression_Range::eMax => dest.write_str("max-")?,
            nsMediaExpression_Range::eEqual => {},
        }
        // NB: CSSStringWriter not needed, features are under control.
        write!(dest, "{}", Atom::from(unsafe { *self.feature.mName }))?;
        dest.write_str(": ")?;

        self.value.to_css(dest)?;
        dest.write_str(")")
    }
}

/// A resolution.
#[derive(Debug, Clone)]
pub enum Resolution {
    /// Dots per inch.
    Dpi(CSSFloat),
    /// Dots per pixel.
    Dppx(CSSFloat),
    /// Dots per centimeter.
    Dpcm(CSSFloat),
}

impl Resolution {
    fn parse(input: &mut Parser) -> Result<Self, ()> {
        let (value, unit) = match try!(input.next()) {
            Token::Dimension(value, unit) => {
                (value.value, unit)
            },
            _ => return Err(()),
        };

        Ok(match_ignore_ascii_case! { unit,
            "dpi" => Resolution::Dpi(value),
            "dppx" => Resolution::Dppx(value),
            "dpcm" => Resolution::Dpcm(value),
            _ => return Err(())
        })
    }
}

impl ToCss for Resolution {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            Resolution::Dpi(v) => write!(dest, "{}dpi", v),
            Resolution::Dppx(v) => write!(dest, "{}dppx", v),
            Resolution::Dpcm(v) => write!(dest, "{}dpcm", v),
        }
    }
}

/// A value found or expected in a media expression.
#[derive(Debug, Clone)]
pub enum MediaExpressionValue {
    /// A length.
    Length(specified::Length),
    /// A (non-negative) integer.
    Integer(u32),
    /// A floating point value.
    Float(CSSFloat),
    /// A boolean value, specified as an integer (i.e., either 0 or 1).
    BoolInteger(bool),
    /// Two integers separated by '/', with optional whitespace on either side
    /// of the '/'.
    IntRatio(u32, u32),
    /// A resolution.
    Resolution(Resolution),
    /// An enumerated index into the variant keyword table.
    Enumerated(u32),
    /// An identifier.
    Ident(Atom),
}

impl ToCss for MediaExpressionValue {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            MediaExpressionValue::Length(ref l) => l.to_css(dest),
            MediaExpressionValue::Integer(v) => write!(dest, "{}", v),
            MediaExpressionValue::Float(v) => write!(dest, "{}", v),
            MediaExpressionValue::BoolInteger(v) => {
                dest.write_str(if v { "1" } else { "0" })
            },
            MediaExpressionValue::IntRatio(a, b) => {
                write!(dest, "{}/{}", a, b)
            },
            MediaExpressionValue::Resolution(ref r) => r.to_css(dest),
            MediaExpressionValue::Enumerated(..) |
            MediaExpressionValue::Ident(..) => {
                // TODO(emilio)
                unimplemented!()
            }
        }
    }
}

fn starts_with_ignore_ascii_case(string: &str, prefix: &str) -> bool {
    string.len() > prefix.len() &&
      string[0..prefix.len()].eq_ignore_ascii_case(prefix)
}

#[allow(warnings)]
fn find_feature<F>(mut f: F) -> Option<&'static nsMediaFeature>
    where F: FnMut(&'static nsMediaFeature) -> bool,
{
    // FIXME(emilio): With build-time bindgen, we would be able to use
    // structs::nsMediaFeatures_features. That would unfortunately break MSVC
    // builds, or require one bindings file per platform.
    //
    // I'm not into any of those, so meanwhile let's use a FFI function.
    unsafe {
        let mut features = bindings::Gecko_GetMediaFeatures();
        while !(*features).mName.is_null() {
            if f(&*features) {
                return Some(&*features);
            }
            features = features.offset(1);
        }
    }

    None
}

impl Expression {
    /// Trivially construct a new expression.
    fn new(feature: &'static nsMediaFeature,
           value: MediaExpressionValue,
           range: nsMediaExpression_Range) -> Self {
        Expression {
            feature: feature,
            value: value,
            range: range,
        }
    }

    /// Parse a media expression of the form:
    ///
    /// ```
    /// (media-feature: media-value)
    /// ```
    #[allow(warnings)]
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        try!(input.expect_parenthesis_block());
        input.parse_nested_block(|input| {
            let ident = try!(input.expect_ident());
            try!(input.expect_colon());

            let mut flags = 0;
            let mut feature_name = &*ident;

            // TODO(emilio): this is under a pref in Gecko.
            if starts_with_ignore_ascii_case(feature_name, "-webkit-") {
                feature_name = &feature_name[8..];
                flags |= nsMediaFeature_RequirementFlags::eHasWebkitPrefix as u8;
            }

            let range = if starts_with_ignore_ascii_case(feature_name, "min-") {
                feature_name = &feature_name[4..];
                nsMediaExpression_Range::eMin
            } else if starts_with_ignore_ascii_case(feature_name, "max-") {
                feature_name = &feature_name[4..];
                nsMediaExpression_Range::eMax
            } else {
                nsMediaExpression_Range::eEqual
            };

            let atom = Atom::from(feature_name);
            let feature =
                match find_feature(|f| atom.as_ptr() == unsafe { *f.mName }) {
                    Some(f) => f,
                    None => return Err(()),
                };

            if (feature.mReqFlags & !flags) != 0 {
                return Err(());
            }

            if range != nsMediaExpression_Range::eEqual &&
                feature.mRangeType != nsMediaFeature_RangeType::eMinMaxAllowed {
                return Err(());
            }

            let value = match feature.mValueType {
                nsMediaFeature_ValueType::eLength => {
                    MediaExpressionValue::Length(
                        specified::Length::parse_non_negative(input)?)
                },
                nsMediaFeature_ValueType::eInteger => {
                    let i = input.expect_integer()?;
                    if i < 0 {
                        return Err(())
                    }
                    MediaExpressionValue::Integer(i as u32)
                }
                nsMediaFeature_ValueType::eBoolInteger => {
                    let i = input.expect_integer()?;
                    if i < 0 || i > 1 {
                        return Err(())
                    }
                    MediaExpressionValue::BoolInteger(i == 1)
                }
                nsMediaFeature_ValueType::eFloat => {
                    MediaExpressionValue::Float(input.expect_number()?)
                }
                nsMediaFeature_ValueType::eIntRatio => {
                    let a = input.expect_integer()?;
                    if a <= 0 {
                        return Err(())
                    }

                    input.expect_delim('/')?;

                    let b = input.expect_integer()?;
                    if b <= 0 {
                        return Err(())
                    }
                    MediaExpressionValue::IntRatio(a as u32, b as u32)
                }
                nsMediaFeature_ValueType::eResolution => {
                    MediaExpressionValue::Resolution(Resolution::parse(input)?)
                }
                nsMediaFeature_ValueType::eEnumerated => {
                    let index = unsafe {
                        let _table = feature.mData.mKeywordTable.as_ref();
                        0
                    };
                    MediaExpressionValue::Enumerated(index)
                }
                nsMediaFeature_ValueType::eIdent => {
                    MediaExpressionValue::Ident(input.expect_ident()?.into())
                }
            };

            Ok(Expression::new(feature, value, range))
        })
    }

    /// Returns whether this media query evaluates to true for the given
    /// device.
    pub fn matches(&self, _device: &Device) -> bool {
        // TODO
        false
    }
}
