/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko's media-query device and expression representation.

use app_units::Au;
use cssparser::{CssStringWriter, Parser, Token};
use euclid::Size2D;
use gecko_bindings::bindings;
use gecko_bindings::structs::{nsCSSValue, nsCSSUnit, nsStringBuffer};
use gecko_bindings::structs::{nsMediaExpression_Range, nsMediaFeature};
use gecko_bindings::structs::{nsMediaFeature_ValueType, nsMediaFeature_RangeType, nsMediaFeature_RequirementFlags};
use gecko_bindings::structs::RawGeckoPresContextOwned;
use media_queries::MediaType;
use properties::ComputedValues;
use std::ascii::AsciiExt;
use std::fmt::{self, Write};
use std::sync::Arc;
use string_cache::Atom;
use style_traits::ToCss;
use style_traits::viewport::ViewportConstraints;
use values::{CSSFloat, specified};
use values::computed::{self, ToComputedValue};

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
        unsafe {
            // FIXME(emilio): Gecko allows emulating random media with
            // mIsEmulatingMedia / mMediaEmulated . Refactor both sides so that
            // is supported (probably just making MediaType an Atom).
            if (*self.pres_context).mMedium == atom!("screen").as_ptr() {
                MediaType::Screen
            } else {
                debug_assert!((*self.pres_context).mMedium == atom!("print").as_ptr());
                MediaType::Print
            }
        }
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
    value: Option<MediaExpressionValue>,
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

        // NB: CssStringWriter not needed, feature names are under control.
        write!(dest, "{}", Atom::from(unsafe { *self.feature.mName }))?;

        if let Some(ref val) = self.value {
            dest.write_str(": ")?;
            val.to_css(dest)?;
        }

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
    fn to_dpi(&self) -> CSSFloat {
        match *self {
            Resolution::Dpi(f) => f,
            Resolution::Dppx(f) => f * 96.0,
            Resolution::Dpcm(f) => f * 2.54,
        }
    }

    fn parse(input: &mut Parser) -> Result<Self, ()> {
        let (value, unit) = match try!(input.next()) {
            Token::Dimension(value, unit) => {
                (value.value, unit)
            },
            _ => return Err(()),
        };

        Ok(match_ignore_ascii_case! { &unit,
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

unsafe fn string_from_ns_string_buffer(buffer: *const nsStringBuffer) -> String {
    use std::slice;
    debug_assert!(!buffer.is_null());
    let data = buffer.offset(1) as *const u16;
    let mut length = 0;
    let mut iter = data;
    while *iter != 0 {
        length += 1;
        iter = iter.offset(1);
    }
    String::from_utf16_lossy(slice::from_raw_parts(data, length))
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
    /// An enumerated value, defined by the variant keyword table in the
    /// feature's `mData` member.
    Enumerated(u32),
    /// An identifier.
    ///
    /// TODO(emilio): Maybe atomize?
    Ident(String),
}

impl MediaExpressionValue {
    fn from_css_value(for_expr: &Expression, css_value: &nsCSSValue) -> Option<Self> {
        // NB: If there's a null value, that means that we don't support the
        // feature.
        if css_value.mUnit == nsCSSUnit::eCSSUnit_Null {
            return None;
        }

        match for_expr.feature.mValueType {
            nsMediaFeature_ValueType::eLength => {
                debug_assert!(css_value.mUnit == nsCSSUnit::eCSSUnit_Pixel);
                let pixels = css_value.float_unchecked();
                Some(MediaExpressionValue::Length(specified::Length::from_px(pixels)))
            }
            nsMediaFeature_ValueType::eInteger => {
                let i = css_value.integer_unchecked();
                debug_assert!(i >= 0);
                Some(MediaExpressionValue::Integer(i as u32))
            }
            nsMediaFeature_ValueType::eFloat => {
                debug_assert!(css_value.mUnit == nsCSSUnit::eCSSUnit_Number);
                Some(MediaExpressionValue::Float(css_value.float_unchecked()))
            }
            nsMediaFeature_ValueType::eBoolInteger => {
                debug_assert!(css_value.mUnit == nsCSSUnit::eCSSUnit_Integer);
                let i = css_value.integer_unchecked();
                debug_assert!(i == 0 || i == 1);
                Some(MediaExpressionValue::BoolInteger(i == 1))
            }
            nsMediaFeature_ValueType::eResolution => {
                debug_assert!(css_value.mUnit == nsCSSUnit::eCSSUnit_Inch);
                Some(MediaExpressionValue::Resolution(Resolution::Dpi(css_value.float_unchecked())))
            }
            nsMediaFeature_ValueType::eEnumerated => {
                debug_assert!(css_value.mUnit == nsCSSUnit::eCSSUnit_Enumerated);
                let value = css_value.integer_unchecked();
                debug_assert!(value >= 0);
                Some(MediaExpressionValue::Enumerated(value as u32))
            }
            nsMediaFeature_ValueType::eIdent => {
                debug_assert!(css_value.mUnit == nsCSSUnit::eCSSUnit_Ident);
                let string = unsafe {
                    string_from_ns_string_buffer(*css_value.mValue.mString.as_ref())
                };
                Some(MediaExpressionValue::Ident(string))
            }
            nsMediaFeature_ValueType::eIntRatio => {
                let array = unsafe { css_value.array_unchecked() };
                debug_assert_eq!(array.len(), 2);
                let first = array[0].integer_unchecked();
                let second = array[1].integer_unchecked();

                debug_assert!(first >= 0 && second >= 0);
                Some(MediaExpressionValue::IntRatio(first as u32, second as u32))
            }
        }
    }
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
            MediaExpressionValue::Ident(ref ident) => {
                CssStringWriter::new(dest).write_str(ident)
            }
            MediaExpressionValue::Enumerated(..) => {
                // TODO(emilio): Use the CSS keyword table.
                unimplemented!()
            }
        }
    }
}

fn starts_with_ignore_ascii_case(string: &str, prefix: &str) -> bool {
    string.len() > prefix.len() &&
      string[0..prefix.len()].eq_ignore_ascii_case(prefix)
}

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
           value: Option<MediaExpressionValue>,
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
    pub fn parse(input: &mut Parser) -> Result<Self, ()> {
        try!(input.expect_parenthesis_block());
        input.parse_nested_block(|input| {
            let ident = try!(input.expect_ident());

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

            // If there's no colon, this is a media query of the form
            // '(<feature>)', that is, there's no value specified.
            //
            // FIXME(emilio): We need to check for range operators too here when
            // we support them, see:
            //
            // https://drafts.csswg.org/mediaqueries/#mq-ranges
            if input.try(|i| i.expect_colon()).is_err() {
                return Ok(Expression::new(feature, None, range));
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
                    // TODO(emilio): Use Gecko's CSS keyword table to parse
                    // this.
                    return Err(())
                }
                nsMediaFeature_ValueType::eIdent => {
                    MediaExpressionValue::Ident(input.expect_ident()?.into_owned())
                }
            };

            Ok(Expression::new(feature, Some(value), range))
        })
    }

    /// Returns whether this media query evaluates to true for the given device.
    pub fn matches(&self, device: &Device) -> bool {
        let mut css_value = nsCSSValue::null();
        unsafe {
            (self.feature.mGetter.unwrap())(device.pres_context,
                                            self.feature,
                                            &mut css_value)
        };

        let value = match MediaExpressionValue::from_css_value(self, &css_value) {
            Some(v) => v,
            None => return false,
        };

        self.evaluate_against(device, &value)
    }

    fn evaluate_against(&self,
                        device: &Device,
                        actual_value: &MediaExpressionValue)
                        -> bool {
        use self::MediaExpressionValue::*;
        use std::cmp::Ordering;

        debug_assert!(self.range == nsMediaExpression_Range::eEqual ||
                      self.feature.mRangeType == nsMediaFeature_RangeType::eMinMaxAllowed,
                      "Whoops, wrong range");

        let default_values = device.default_values();

        // http://dev.w3.org/csswg/mediaqueries3/#units
        // em units are relative to the initial font-size.
        let context = computed::Context {
            is_root_element: false,
            viewport_size: device.au_viewport_size(),
            inherited_style: default_values,
            layout_parent_style: default_values,
            // This cloning business is kind of dumb.... It's because Context
            // insists on having an actual ComputedValues inside itself.
            style: default_values.clone(),
            font_metrics_provider: None
        };

        let required_value = match self.value {
            Some(ref v) => v,
            None => {
                // If there's no value, always match unless it's a zero length
                // or a zero integer or boolean.
                return match *actual_value {
                    BoolInteger(v) => v,
                    Integer(v) => v != 0,
                    Length(ref l) => l.to_computed_value(&context) != Au(0),
                    _ => true,
                }
            }
        };

        // FIXME(emilio): Handle the possible floating point errors?
        let cmp = match (required_value, actual_value) {
            (&Length(ref one), &Length(ref other)) => {
                one.to_computed_value(&context)
                    .cmp(&other.to_computed_value(&context))
            }
            (&Integer(one), &Integer(ref other)) => one.cmp(other),
            (&BoolInteger(one), &BoolInteger(ref other)) => one.cmp(other),
            (&Float(one), &Float(ref other)) => one.partial_cmp(other).unwrap(),
            (&IntRatio(one_num, one_den), &IntRatio(other_num, other_den)) => {
                (one_num * other_den).partial_cmp(&(other_num * one_den)).unwrap()
            }
            (&Resolution(ref one), &Resolution(ref other)) => {
                let actual_dpi = unsafe {
                    if (*device.pres_context).mOverrideDPPX > 0.0 {
                        self::Resolution::Dppx((*device.pres_context).mOverrideDPPX)
                            .to_dpi()
                    } else {
                        other.to_dpi()
                    }
                };

                one.to_dpi().partial_cmp(&actual_dpi).unwrap()
            }
            (&Ident(ref one), &Ident(ref other)) => {
                debug_assert!(self.feature.mRangeType != nsMediaFeature_RangeType::eMinMaxAllowed);
                return one == other;
            }
            (&Enumerated(..), &Enumerated(..)) => {
                // TODO(emilio)
                unimplemented!();
            }
            _ => unreachable!(),
        };

        cmp == Ordering::Equal || match self.range {
            nsMediaExpression_Range::eMin => cmp == Ordering::Less,
            nsMediaExpression_Range::eEqual => false,
            nsMediaExpression_Range::eMax => cmp == Ordering::Greater,
        }
    }
}
