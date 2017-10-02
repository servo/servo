/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Gecko's media-query device and expression representation.

use app_units::AU_PER_PX;
use app_units::Au;
use context::QuirksMode;
use cssparser::{CssStringWriter, Parser, RGBA, Token, BasicParseError};
use euclid::ScaleFactor;
use euclid::Size2D;
use font_metrics::get_metrics_provider_for_product;
use gecko::values::{convert_nscolor_to_rgba, convert_rgba_to_nscolor};
use gecko_bindings::bindings;
use gecko_bindings::structs;
use gecko_bindings::structs::{nsCSSKeyword, nsCSSProps_KTableEntry, nsCSSValue, nsCSSUnit};
use gecko_bindings::structs::{nsMediaExpression_Range, nsMediaFeature};
use gecko_bindings::structs::{nsMediaFeature_ValueType, nsMediaFeature_RangeType, nsMediaFeature_RequirementFlags};
use gecko_bindings::structs::{nsPresContext, RawGeckoPresContextOwned};
use media_queries::MediaType;
use parser::ParserContext;
use properties::{ComputedValues, StyleBuilder};
use properties::longhands::font_size;
use rule_cache::RuleCacheConditions;
use servo_arc::Arc;
use std::cell::RefCell;
use std::fmt::{self, Write};
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use str::starts_with_ignore_ascii_case;
use string_cache::Atom;
use style_traits::{CSSPixel, DevicePixel};
use style_traits::{ToCss, ParseError, StyleParseError};
use style_traits::viewport::ViewportConstraints;
use stylesheets::Origin;
use values::{CSSFloat, CustomIdent, serialize_dimension};
use values::computed::{self, ToComputedValue};
use values::specified::Length;

/// The `Device` in Gecko wraps a pres context, has a default values computed,
/// and contains all the viewport rule state.
pub struct Device {
    /// NB: The pres context lifetime is tied to the styleset, who owns the
    /// stylist, and thus the `Device`, so having a raw pres context pointer
    /// here is fine.
    pres_context: RawGeckoPresContextOwned,
    default_values: Arc<ComputedValues>,
    /// The font size of the root element
    /// This is set when computing the style of the root
    /// element, and used for rem units in other elements.
    ///
    /// When computing the style of the root element, there can't be any
    /// other style being computed at the same time, given we need the style of
    /// the parent to compute everything else. So it is correct to just use
    /// a relaxed atomic here.
    root_font_size: AtomicIsize,
    /// The body text color, stored as an `nscolor`, used for the "tables
    /// inherit from body" quirk.
    ///
    /// https://quirks.spec.whatwg.org/#the-tables-inherit-color-from-body-quirk
    body_text_color: AtomicUsize,
    /// Whether any styles computed in the document relied on the root font-size
    /// by using rem units.
    used_root_font_size: AtomicBool,
    /// Whether any styles computed in the document relied on the viewport size
    /// by using vw/vh/vmin/vmax units.
    used_viewport_size: AtomicBool,
}

unsafe impl Sync for Device {}
unsafe impl Send for Device {}

impl Device {
    /// Trivially constructs a new `Device`.
    pub fn new(pres_context: RawGeckoPresContextOwned) -> Self {
        assert!(!pres_context.is_null());
        Device {
            pres_context: pres_context,
            default_values: ComputedValues::default_values(unsafe { &*pres_context }),
            // FIXME(bz): Seems dubious?
            root_font_size: AtomicIsize::new(font_size::get_initial_value().size().0 as isize),
            body_text_color: AtomicUsize::new(unsafe { &*pres_context }.mDefaultColor as usize),
            used_root_font_size: AtomicBool::new(false),
            used_viewport_size: AtomicBool::new(false),
        }
    }

    /// Tells the device that a new viewport rule has been found, and stores the
    /// relevant viewport constraints.
    pub fn account_for_viewport_rule(
        &mut self,
        _constraints: &ViewportConstraints
    ) {
        unreachable!("Gecko doesn't support @viewport");
    }

    /// Returns the default computed values as a reference, in order to match
    /// Servo.
    pub fn default_computed_values(&self) -> &ComputedValues {
        &self.default_values
    }

    /// Returns the default computed values as an `Arc`.
    pub fn default_computed_values_arc(&self) -> &Arc<ComputedValues> {
        &self.default_values
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

    /// Sets the body text color for the "inherit color from body" quirk.
    ///
    /// https://quirks.spec.whatwg.org/#the-tables-inherit-color-from-body-quirk
    pub fn set_body_text_color(&self, color: RGBA) {
        self.body_text_color.store(convert_rgba_to_nscolor(&color) as usize, Ordering::Relaxed)
    }

    /// Returns the body text color.
    pub fn body_text_color(&self) -> RGBA {
        convert_nscolor_to_rgba(self.body_text_color.load(Ordering::Relaxed) as u32)
    }

    /// Gets the pres context associated with this document.
    pub fn pres_context(&self) -> &nsPresContext {
        unsafe { &*self.pres_context }
    }

    /// Recreates the default computed values.
    pub fn reset_computed_values(&mut self) {
        self.default_values = ComputedValues::default_values(self.pres_context());
    }

    /// Rebuild all the cached data.
    pub fn rebuild_cached_data(&mut self) {
        self.reset_computed_values();
        self.used_root_font_size.store(false, Ordering::Relaxed);
        self.used_viewport_size.store(false, Ordering::Relaxed);
    }

    /// Returns whether we ever looked up the root font size of the Device.
    pub fn used_root_font_size(&self) -> bool {
        self.used_root_font_size.load(Ordering::Relaxed)
    }

    /// Recreates all the temporary state that the `Device` stores.
    ///
    /// This includes the viewport override from `@viewport` rules, and also the
    /// default computed values.
    pub fn reset(&mut self) {
        self.reset_computed_values();
    }

    /// Returns the current media type of the device.
    pub fn media_type(&self) -> MediaType {
        unsafe {
            // Gecko allows emulating random media with mIsEmulatingMedia and
            // mMediaEmulated.
            let context = self.pres_context();
            let medium_to_use = if context.mIsEmulatingMedia() != 0 {
                context.mMediaEmulated.mRawPtr
            } else {
                context.mMedium
            };

            MediaType(CustomIdent(Atom::from(medium_to_use)))
        }
    }

    /// Returns the current viewport size in app units.
    pub fn au_viewport_size(&self) -> Size2D<Au> {
        let area = &self.pres_context().mVisibleArea;
        Size2D::new(Au(area.width), Au(area.height))
    }

    /// Returns the current viewport size in app units, recording that it's been
    /// used for viewport unit resolution.
    pub fn au_viewport_size_for_viewport_unit_resolution(&self) -> Size2D<Au> {
        self.used_viewport_size.store(true, Ordering::Relaxed);
        self.au_viewport_size()
    }

    /// Returns whether we ever looked up the viewport size of the Device.
    pub fn used_viewport_size(&self) -> bool {
        self.used_viewport_size.load(Ordering::Relaxed)
    }

    /// Returns the device pixel ratio.
    pub fn device_pixel_ratio(&self) -> ScaleFactor<f32, CSSPixel, DevicePixel> {
        let override_dppx = self.pres_context().mOverrideDPPX;
        if override_dppx > 0.0 { return ScaleFactor::new(override_dppx); }
        let au_per_dpx = self.pres_context().mCurAppUnitsPerDevPixel as f32;
        let au_per_px = AU_PER_PX as f32;
        ScaleFactor::new(au_per_px / au_per_dpx)
    }

    /// Returns whether document colors are enabled.
    pub fn use_document_colors(&self) -> bool {
        self.pres_context().mUseDocumentColors() != 0
    }

    /// Returns the default background color.
    pub fn default_background_color(&self) -> RGBA {
        convert_nscolor_to_rgba(self.pres_context().mBackgroundColor)
    }

    /// Applies text zoom to a font-size or line-height value (see nsStyleFont::ZoomText).
    pub fn zoom_text(&self, size: Au) -> Au {
        size.scale_by(self.pres_context().mEffectiveTextZoom)
    }
    /// Un-apply text zoom (see nsStyleFont::UnzoomText).
    pub fn unzoom_text(&self, size: Au) -> Au {
        size.scale_by(1. / self.pres_context().mEffectiveTextZoom)
    }
}

/// A expression for gecko contains a reference to the media feature, the value
/// the media query contained, and the range to evaluate.
#[derive(Clone, Debug)]
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

        if (self.feature.mReqFlags & nsMediaFeature_RequirementFlags::eHasWebkitPrefix as u8) != 0 {
            dest.write_str("-webkit-")?;
        }
        match self.range {
            nsMediaExpression_Range::eMin => dest.write_str("min-")?,
            nsMediaExpression_Range::eMax => dest.write_str("max-")?,
            nsMediaExpression_Range::eEqual => {},
        }

        // NB: CssStringWriter not needed, feature names are under control.
        write!(dest, "{}", Atom::from(unsafe { *self.feature.mName }))?;

        if let Some(ref val) = self.value {
            dest.write_str(": ")?;
            val.to_css(dest, self)?;
        }

        dest.write_str(")")
    }
}

impl PartialEq for Expression {
    fn eq(&self, other: &Expression) -> bool {
        self.feature.mName == other.feature.mName &&
            self.value == other.value && self.range == other.range
    }
}

/// A resolution.
#[derive(Clone, Debug, PartialEq)]
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

    fn parse<'i, 't>(input: &mut Parser<'i, 't>) -> Result<Self, ParseError<'i>> {
        let (value, unit) = match *input.next()? {
            Token::Dimension { value, ref unit, .. } => {
                (value, unit)
            },
            ref t => return Err(BasicParseError::UnexpectedToken(t.clone()).into()),
        };

        if value <= 0. {
            return Err(StyleParseError::UnspecifiedError.into())
        }

        (match_ignore_ascii_case! { &unit,
            "dpi" => Ok(Resolution::Dpi(value)),
            "dppx" => Ok(Resolution::Dppx(value)),
            "dpcm" => Ok(Resolution::Dpcm(value)),
            _ => Err(())
        }).map_err(|()| StyleParseError::UnexpectedDimension(unit.clone()).into())
    }
}

impl ToCss for Resolution {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            Resolution::Dpi(v) => serialize_dimension(v, "dpi", dest),
            Resolution::Dppx(v) => serialize_dimension(v, "dppx", dest),
            Resolution::Dpcm(v) => serialize_dimension(v, "dpcm", dest),
        }
    }
}

/// A value found or expected in a media expression.
#[derive(Clone, Debug, PartialEq)]
pub enum MediaExpressionValue {
    /// A length.
    Length(Length),
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
    Enumerated(i16),
    /// An identifier.
    ///
    /// TODO(emilio): Maybe atomize?
    Ident(String),
}

impl MediaExpressionValue {
    fn from_css_value(for_expr: &Expression, css_value: &nsCSSValue) -> Option<Self> {
        use gecko::conversions::string_from_chars_pointer;

        // NB: If there's a null value, that means that we don't support the
        // feature.
        if css_value.mUnit == nsCSSUnit::eCSSUnit_Null {
            return None;
        }

        match for_expr.feature.mValueType {
            nsMediaFeature_ValueType::eLength => {
                debug_assert!(css_value.mUnit == nsCSSUnit::eCSSUnit_Pixel);
                let pixels = css_value.float_unchecked();
                Some(MediaExpressionValue::Length(Length::from_px(pixels)))
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
                // This is temporarily more complicated to allow Gecko Bug
                // 1376931 to land. Parts of that bug will supply pixel values
                // and expect them to be passed through without conversion.
                // After all parts of that bug have landed, Bug 1404097 will
                // return this function to once again only allow one type of
                // value to be accepted: this time, only pixel values.
                let res = match css_value.mUnit {
                    nsCSSUnit::eCSSUnit_Pixel => Resolution::Dppx(css_value.float_unchecked()),
                    nsCSSUnit::eCSSUnit_Inch  => Resolution::Dpi(css_value.float_unchecked()),
                    _                         => unreachable!(),
                };
                Some(MediaExpressionValue::Resolution(res))
            }
            nsMediaFeature_ValueType::eEnumerated => {
                let value = css_value.integer_unchecked() as i16;
                Some(MediaExpressionValue::Enumerated(value))
            }
            nsMediaFeature_ValueType::eIdent => {
                debug_assert!(css_value.mUnit == nsCSSUnit::eCSSUnit_Ident);
                let string = unsafe {
                    let buffer = *css_value.mValue.mString.as_ref();
                    debug_assert!(!buffer.is_null());
                    string_from_chars_pointer(buffer.offset(1) as *const u16)
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

impl MediaExpressionValue {
    fn to_css<W>(&self, dest: &mut W, for_expr: &Expression) -> fmt::Result
        where W: fmt::Write,
    {
        match *self {
            MediaExpressionValue::Length(ref l) => l.to_css(dest),
            MediaExpressionValue::Integer(v) => v.to_css(dest),
            MediaExpressionValue::Float(v) => v.to_css(dest),
            MediaExpressionValue::BoolInteger(v) => {
                dest.write_str(if v { "1" } else { "0" })
            },
            MediaExpressionValue::IntRatio(a, b) => {
                a.to_css(dest)?;
                dest.write_char('/')?;
                b.to_css(dest)
            },
            MediaExpressionValue::Resolution(ref r) => r.to_css(dest),
            MediaExpressionValue::Ident(ref ident) => {
                CssStringWriter::new(dest).write_str(ident)
            }
            MediaExpressionValue::Enumerated(value) => unsafe {
                use std::{slice, str};
                use std::os::raw::c_char;

                // NB: All the keywords on nsMediaFeatures are static,
                // well-formed utf-8.
                let mut length = 0;

                let (keyword, _value) =
                    find_in_table(*for_expr.feature.mData.mKeywordTable.as_ref(),
                                  |_kw, val| val == value)
                        .expect("Value not found in the keyword table?");

                let buffer: *const c_char =
                    bindings::Gecko_CSSKeywordString(keyword, &mut length);
                let buffer =
                    slice::from_raw_parts(buffer as *const u8, length as usize);

                let string = str::from_utf8_unchecked(buffer);

                dest.write_str(string)
            }
        }
    }
}

fn find_feature<F>(mut f: F) -> Option<&'static nsMediaFeature>
    where F: FnMut(&'static nsMediaFeature) -> bool,
{
    unsafe {
        let mut features = structs::nsMediaFeatures_features.as_ptr();
        while !(*features).mName.is_null() {
            if f(&*features) {
                return Some(&*features);
            }
            features = features.offset(1);
        }
    }
    None
}

unsafe fn find_in_table<F>(mut current_entry: *const nsCSSProps_KTableEntry,
                           mut f: F)
                           -> Option<(nsCSSKeyword, i16)>
    where F: FnMut(nsCSSKeyword, i16) -> bool
{
    loop {
        let value = (*current_entry).mValue;
        let keyword = (*current_entry).mKeyword;

        if value == -1 {
            return None; // End of the table.
        }

        if f(keyword, value) {
            return Some((keyword, value));
        }

        current_entry = current_entry.offset(1);
    }
}

fn parse_feature_value<'i, 't>(feature: &nsMediaFeature,
                               feature_value_type: nsMediaFeature_ValueType,
                               context: &ParserContext,
                               input: &mut Parser<'i, 't>)
                               -> Result<MediaExpressionValue, ParseError<'i>> {
    let value = match feature_value_type {
        nsMediaFeature_ValueType::eLength => {
           let length = Length::parse_non_negative(context, input)?;
           // FIXME(canaltinova): See bug 1396057. Gecko doesn't support calc
           // inside media queries. This check is for temporarily remove it
           // for parity with gecko. We should remove this check when we want
           // to support it.
           if let Length::Calc(_) = length {
               return Err(StyleParseError::UnspecifiedError.into())
           }
           MediaExpressionValue::Length(length)
        },
        nsMediaFeature_ValueType::eInteger => {
           // FIXME(emilio): We should use `Integer::parse` to handle `calc`
           // properly in integer expressions. Note that calc is still not
           // supported in media queries per FIXME above.
           let i = input.expect_integer()?;
           if i < 0 {
               return Err(StyleParseError::UnspecifiedError.into())
           }
           MediaExpressionValue::Integer(i as u32)
        }
        nsMediaFeature_ValueType::eBoolInteger => {
           let i = input.expect_integer()?;
           if i < 0 || i > 1 {
               return Err(StyleParseError::UnspecifiedError.into())
           }
           MediaExpressionValue::BoolInteger(i == 1)
        }
        nsMediaFeature_ValueType::eFloat => {
           MediaExpressionValue::Float(input.expect_number()?)
        }
        nsMediaFeature_ValueType::eIntRatio => {
           let a = input.expect_integer()?;
           if a <= 0 {
               return Err(StyleParseError::UnspecifiedError.into())
           }

           input.expect_delim('/')?;

           let b = input.expect_integer()?;
           if b <= 0 {
               return Err(StyleParseError::UnspecifiedError.into())
           }
           MediaExpressionValue::IntRatio(a as u32, b as u32)
        }
        nsMediaFeature_ValueType::eResolution => {
           MediaExpressionValue::Resolution(Resolution::parse(input)?)
        }
        nsMediaFeature_ValueType::eEnumerated => {
           let keyword = input.expect_ident()?;
           let keyword = unsafe {
               bindings::Gecko_LookupCSSKeyword(keyword.as_bytes().as_ptr(),
               keyword.len() as u32)
           };

           let first_table_entry: *const nsCSSProps_KTableEntry = unsafe {
               *feature.mData.mKeywordTable.as_ref()
           };

           let value =
               match unsafe { find_in_table(first_table_entry, |kw, _| kw == keyword) } {
                   Some((_kw, value)) => {
                       value
                   }
                   None => return Err(StyleParseError::UnspecifiedError.into()),
               };

           MediaExpressionValue::Enumerated(value)
        }
        nsMediaFeature_ValueType::eIdent => {
           MediaExpressionValue::Ident(input.expect_ident()?.as_ref().to_owned())
        }
    };

    Ok(value)
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
    pub fn parse<'i, 't>(
        context: &ParserContext,
        input: &mut Parser<'i, 't>,
    ) -> Result<Self, ParseError<'i>> {
        input.expect_parenthesis_block().map_err(|err|
            match err {
                BasicParseError::UnexpectedToken(t) => StyleParseError::ExpectedIdentifier(t),
                _ => StyleParseError::UnspecifiedError,
            }
        )?;

        input.parse_nested_block(|input| {
            // FIXME: remove extra indented block when lifetimes are non-lexical
            let feature;
            let range;
            {
                let ident = input.expect_ident().map_err(|err|
                    match err {
                        BasicParseError::UnexpectedToken(t) => StyleParseError::ExpectedIdentifier(t),
                        _ => StyleParseError::UnspecifiedError,
                    }
                )?;

                let mut flags = 0;

                if context.in_chrome_stylesheet() ||
                    context.stylesheet_origin == Origin::UserAgent {
                    flags |= nsMediaFeature_RequirementFlags::eUserAgentAndChromeOnly as u8;
                }

                let result = {
                    let mut feature_name = &**ident;

                    if unsafe { structs::StylePrefs_sWebkitPrefixedAliasesEnabled } &&
                       starts_with_ignore_ascii_case(feature_name, "-webkit-") {
                        feature_name = &feature_name[8..];
                        flags |= nsMediaFeature_RequirementFlags::eHasWebkitPrefix as u8;
                        if unsafe { structs::StylePrefs_sWebkitDevicePixelRatioEnabled } {
                            flags |= nsMediaFeature_RequirementFlags::eWebkitDevicePixelRatioPrefEnabled as u8;
                        }
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
                    match find_feature(|f| atom.as_ptr() == unsafe { *f.mName }) {
                        Some(f) => Ok((f, range)),
                        None => Err(()),
                    }
                };

                match result {
                    Ok((f, r)) => {
                        feature = f;
                        range = r;
                    },
                    Err(()) => {
                        return Err(StyleParseError::MediaQueryExpectedFeatureName(ident.clone()).into())
                    },
                }

                if (feature.mReqFlags & !flags) != 0 {
                    return Err(StyleParseError::MediaQueryExpectedFeatureName(ident.clone()).into());
                }

                if range != nsMediaExpression_Range::eEqual &&
                   feature.mRangeType != nsMediaFeature_RangeType::eMinMaxAllowed {
                    return Err(StyleParseError::MediaQueryExpectedFeatureName(ident.clone()).into());
                }
            }

            // If there's no colon, this is a media query of the form
            // '(<feature>)', that is, there's no value specified.
            //
            // Gecko doesn't allow ranged expressions without a value, so just
            // reject them here too.
            if input.try(|i| i.expect_colon()).is_err() {
                if range != nsMediaExpression_Range::eEqual {
                    return Err(StyleParseError::RangedExpressionWithNoValue.into())
                }
                return Ok(Expression::new(feature, None, range));
            }

            let value = parse_feature_value(feature,
                                            feature.mValueType,
                                            context, input).map_err(|_|
                StyleParseError::MediaQueryExpectedFeatureValue
            )?;

            Ok(Expression::new(feature, Some(value), range))
        })
    }

    /// Returns whether this media query evaluates to true for the given device.
    pub fn matches(&self, device: &Device, quirks_mode: QuirksMode) -> bool {
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

        self.evaluate_against(device, &value, quirks_mode)
    }

    fn evaluate_against(
        &self,
        device: &Device,
        actual_value: &MediaExpressionValue,
        quirks_mode: QuirksMode,
    ) -> bool {
        use self::MediaExpressionValue::*;
        use std::cmp::Ordering;

        debug_assert!(self.range == nsMediaExpression_Range::eEqual ||
                      self.feature.mRangeType == nsMediaFeature_RangeType::eMinMaxAllowed,
                      "Whoops, wrong range");

        let default_values = device.default_computed_values();


        let provider = get_metrics_provider_for_product();

        // http://dev.w3.org/csswg/mediaqueries3/#units
        // em units are relative to the initial font-size.
        let mut conditions = RuleCacheConditions::default();
        let context = computed::Context {
            is_root_element: false,
            builder: StyleBuilder::for_derived_style(device, default_values, None, None),
            font_metrics_provider: &provider,
            cached_system_font: None,
            in_media_query: true,
            quirks_mode,
            for_smil_animation: false,
            for_non_inherited_property: None,
            rule_cache_conditions: RefCell::new(&mut conditions),
        };

        let required_value = match self.value {
            Some(ref v) => v,
            None => {
                // If there's no value, always match unless it's a zero length
                // or a zero integer or boolean.
                return match *actual_value {
                    BoolInteger(v) => v,
                    Integer(v) => v != 0,
                    Length(ref l) => l.to_computed_value(&context).px() != 0.,
                    _ => true,
                }
            }
        };

        // FIXME(emilio): Handle the possible floating point errors?
        let cmp = match (required_value, actual_value) {
            (&Length(ref one), &Length(ref other)) => {
                one.to_computed_value(&context).to_i32_au()
                    .cmp(&other.to_computed_value(&context).to_i32_au())
            }
            (&Integer(one), &Integer(ref other)) => one.cmp(other),
            (&BoolInteger(one), &BoolInteger(ref other)) => one.cmp(other),
            (&Float(one), &Float(ref other)) => one.partial_cmp(other).unwrap(),
            (&IntRatio(one_num, one_den), &IntRatio(other_num, other_den)) => {
                // Extend to avoid overflow.
                (one_num as u64 * other_den as u64).cmp(
                    &(other_num as u64 * one_den as u64))
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
            (&Enumerated(one), &Enumerated(other)) => {
                debug_assert!(self.feature.mRangeType != nsMediaFeature_RangeType::eMinMaxAllowed);
                return one == other;
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
