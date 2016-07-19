/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! Common [values][values] used in CSS.
//!
//! [values]: https://drafts.csswg.org/css-values/

pub use cssparser::RGBA;

use app_units::Au;
use cssparser::CssStringWriter;
use std::fmt::{self, Write};
use url::Url;


/// The real ToCss trait can't be implemented for types in crates that don't
/// depend on each other.
pub trait LocalToCss {
    /// Serialize `self` in CSS syntax, writing to `dest`.
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write;

    /// Serialize `self` in CSS syntax and return a string.
    ///
    /// (This is a convenience wrapper for `to_css` and probably should not be overridden.)
    #[inline]
    fn to_css_string(&self) -> String {
        let mut s = String::new();
        self.to_css(&mut s).unwrap();
        s
    }
}

impl LocalToCss for Au {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        write!(dest, "{}px", self.to_f64_px())
    }
}

impl LocalToCss for Url {
    fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
        try!(dest.write_str("url(\""));
        try!(write!(CssStringWriter::new(dest), "{}", self));
        try!(dest.write_str("\")"));
        Ok(())
    }
}

macro_rules! define_numbered_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+,) => {
        define_numbered_css_keyword_enum!($name: $( $css => $variant = $value ),+);
    };
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Copy, RustcEncodable, Debug)]
        #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
        pub enum $name {
            $( $variant = $value ),+
        }

        impl $name {
            pub fn parse(input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    $( $css => Ok($name::$variant), )+
                    _ => Err(())
                }
            }
        }

        impl ::cssparser::ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where W: ::std::fmt::Write {
                match *self {
                    $( $name::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}

pub type CSSFloat = f32;

pub const FONT_MEDIUM_PX: i32 = 16;

pub trait HasViewportPercentage {
    fn has_viewport_percentage(&self) -> bool;
}

pub trait NoViewportPercentage {}

impl<T> HasViewportPercentage for T where T: NoViewportPercentage {
    fn has_viewport_percentage(&self) -> bool {
        false
    }
}

pub mod specified {
    use app_units::Au;
    use cssparser::{self, Parser, ToCss, Token};
    use euclid::size::Size2D;
    use parser::ParserContext;
    use std::ascii::AsciiExt;
    use std::cmp;
    use std::f32::consts::PI;
    use std::fmt;
    use std::ops::Mul;
    use style_traits::values::specified::AllowedNumericType;
    use super::computed::{Context, ToComputedValue};
    use super::{CSSFloat, FONT_MEDIUM_PX, HasViewportPercentage, LocalToCss, NoViewportPercentage};
    use url::Url;

    impl NoViewportPercentage for i32 {}  // For PropertyDeclaration::Order

    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct CSSColor {
        pub parsed: cssparser::Color,
        pub authored: Option<String>,
    }
    impl CSSColor {
        pub fn parse(input: &mut Parser) -> Result<CSSColor, ()> {
            let start_position = input.position();
            let authored = match input.next() {
                Ok(Token::Ident(s)) => Some(s.into_owned()),
                _ => None,
            };
            input.reset(start_position);
            Ok(CSSColor {
                parsed: try!(cssparser::Color::parse(input)),
                authored: authored,
            })
        }
    }

    impl NoViewportPercentage for CSSColor {}

    impl ToCss for CSSColor {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.authored {
                Some(ref s) => dest.write_str(s),
                None => self.parsed.to_css(dest),
            }
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct CSSRGBA {
        pub parsed: cssparser::RGBA,
        pub authored: Option<String>,
    }

    impl NoViewportPercentage for CSSRGBA {}

    impl ToCss for CSSRGBA {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.authored {
                Some(ref s) => dest.write_str(s),
                None => self.parsed.to_css(dest),
            }
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum FontRelativeLength {
        Em(CSSFloat),
        Ex(CSSFloat),
        Ch(CSSFloat),
        Rem(CSSFloat)
    }

    impl ToCss for FontRelativeLength {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                FontRelativeLength::Em(length) => write!(dest, "{}em", length),
                FontRelativeLength::Ex(length) => write!(dest, "{}ex", length),
                FontRelativeLength::Ch(length) => write!(dest, "{}ch", length),
                FontRelativeLength::Rem(length) => write!(dest, "{}rem", length)
            }
        }
    }

    impl FontRelativeLength {
        pub fn to_computed_value(&self,
                                 reference_font_size: Au,
                                 root_font_size: Au)
                                 -> Au
        {
            match *self {
                FontRelativeLength::Em(length) => reference_font_size.scale_by(length),
                FontRelativeLength::Ex(length) | FontRelativeLength::Ch(length) => {
                    // https://github.com/servo/servo/issues/7462
                    let em_factor = 0.5;
                    reference_font_size.scale_by(length * em_factor)
                },
                FontRelativeLength::Rem(length) => root_font_size.scale_by(length)
            }
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum ViewportPercentageLength {
        Vw(CSSFloat),
        Vh(CSSFloat),
        Vmin(CSSFloat),
        Vmax(CSSFloat)
    }

    impl HasViewportPercentage for ViewportPercentageLength {
        fn has_viewport_percentage(&self) -> bool {
            true
        }
    }

    impl ToCss for ViewportPercentageLength {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                ViewportPercentageLength::Vw(length) => write!(dest, "{}vw", length),
                ViewportPercentageLength::Vh(length) => write!(dest, "{}vh", length),
                ViewportPercentageLength::Vmin(length) => write!(dest, "{}vmin", length),
                ViewportPercentageLength::Vmax(length) => write!(dest, "{}vmax", length)
            }
        }
    }

    impl ViewportPercentageLength {
        pub fn to_computed_value(&self, viewport_size: Size2D<Au>) -> Au {
            macro_rules! to_unit {
                ($viewport_dimension:expr) => {
                    $viewport_dimension.to_f32_px() / 100.0
                }
            }

            let value = match *self {
                ViewportPercentageLength::Vw(length) =>
                    length * to_unit!(viewport_size.width),
                ViewportPercentageLength::Vh(length) =>
                    length * to_unit!(viewport_size.height),
                ViewportPercentageLength::Vmin(length) =>
                    length * to_unit!(cmp::min(viewport_size.width, viewport_size.height)),
                ViewportPercentageLength::Vmax(length) =>
                    length * to_unit!(cmp::max(viewport_size.width, viewport_size.height)),
            };
            Au::from_f32_px(value)
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct CharacterWidth(pub i32);

    impl CharacterWidth {
        pub fn to_computed_value(&self, reference_font_size: Au) -> Au {
            // This applies the *converting a character width to pixels* algorithm as specified
            // in HTML5 § 14.5.4.
            //
            // TODO(pcwalton): Find these from the font.
            let average_advance = reference_font_size.scale_by(0.5);
            let max_advance = reference_font_size;
            average_advance.scale_by(self.0 as CSSFloat - 1.0) + max_advance
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum Length {
        Absolute(Au),  // application units
        FontRelative(FontRelativeLength),
        ViewportPercentage(ViewportPercentageLength),

        /// HTML5 "character width", as defined in HTML5 § 14.5.4.
        ///
        /// This cannot be specified by the user directly and is only generated by
        /// `Stylist::synthesize_rules_for_legacy_attributes()`.
        ServoCharacterWidth(CharacterWidth),

        Calc(CalcLengthOrPercentage),
    }

    impl HasViewportPercentage for Length {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                Length::ViewportPercentage(_) => true,
                _ => false
            }
        }
    }

    impl ToCss for Length {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                Length::Absolute(length) => write!(dest, "{}px", length.to_f32_px()),
                Length::FontRelative(length) => length.to_css(dest),
                Length::ViewportPercentage(length) => length.to_css(dest),
                Length::Calc(calc) => calc.to_css(dest),
                Length::ServoCharacterWidth(_)
                => panic!("internal CSS values should never be serialized"),
            }
        }
    }

    impl Mul<CSSFloat> for Length {
        type Output = Length;

        #[inline]
        fn mul(self, scalar: CSSFloat) -> Length {
            match self {
                Length::Absolute(Au(v)) => Length::Absolute(Au(((v as f32) * scalar) as i32)),
                Length::FontRelative(v) => Length::FontRelative(v * scalar),
                Length::ViewportPercentage(v) => Length::ViewportPercentage(v * scalar),
                Length::Calc(_) => panic!("Can't multiply Calc!"),
                Length::ServoCharacterWidth(_) => panic!("Can't multiply ServoCharacterWidth!"),
            }
        }
    }

    impl Mul<CSSFloat> for FontRelativeLength {
        type Output = FontRelativeLength;

        #[inline]
        fn mul(self, scalar: CSSFloat) -> FontRelativeLength {
            match self {
                FontRelativeLength::Em(v) => FontRelativeLength::Em(v * scalar),
                FontRelativeLength::Ex(v) => FontRelativeLength::Ex(v * scalar),
                FontRelativeLength::Ch(v) => FontRelativeLength::Ch(v * scalar),
                FontRelativeLength::Rem(v) => FontRelativeLength::Rem(v * scalar),
            }
        }
    }

    impl Mul<CSSFloat> for ViewportPercentageLength {
        type Output = ViewportPercentageLength;

        #[inline]
        fn mul(self, scalar: CSSFloat) -> ViewportPercentageLength {
            match self {
                ViewportPercentageLength::Vw(v) => ViewportPercentageLength::Vw(v * scalar),
                ViewportPercentageLength::Vh(v) => ViewportPercentageLength::Vh(v * scalar),
                ViewportPercentageLength::Vmin(v) => ViewportPercentageLength::Vmin(v * scalar),
                ViewportPercentageLength::Vmax(v) => ViewportPercentageLength::Vmax(v * scalar),
            }
        }
    }

    const AU_PER_PX: CSSFloat = 60.;
    const AU_PER_IN: CSSFloat = AU_PER_PX * 96.;
    const AU_PER_CM: CSSFloat = AU_PER_IN / 2.54;
    const AU_PER_MM: CSSFloat = AU_PER_IN / 25.4;
    const AU_PER_Q: CSSFloat = AU_PER_MM / 4.;
    const AU_PER_PT: CSSFloat = AU_PER_IN / 72.;
    const AU_PER_PC: CSSFloat = AU_PER_PT * 12.;
    impl Length {
        // https://drafts.csswg.org/css-fonts-3/#font-size-prop
        pub fn from_str(s: &str) -> Option<Length> {
            Some(match_ignore_ascii_case! { s,
                "xx-small" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 3 / 5),
                "x-small" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 3 / 4),
                "small" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 8 / 9),
                "medium" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX)),
                "large" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 6 / 5),
                "x-large" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 3 / 2),
                "xx-large" => Length::Absolute(Au::from_px(FONT_MEDIUM_PX) * 2),

                // https://github.com/servo/servo/issues/3423#issuecomment-56321664
                "smaller" => Length::FontRelative(FontRelativeLength::Em(0.85)),
                "larger" => Length::FontRelative(FontRelativeLength::Em(1.2)),
                _ => return None
            })
        }

        #[inline]
        fn parse_internal(input: &mut Parser, context: &AllowedNumericType) -> Result<Length, ()> {
            match try!(input.next()) {
                Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                    Length::parse_dimension(value.value, unit),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(Length::Absolute(Au(0))),
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") =>
                    input.parse_nested_block(CalcLengthOrPercentage::parse_length),
                _ => Err(())
            }
        }
        pub fn parse(input: &mut Parser) -> Result<Length, ()> {
            Length::parse_internal(input, &AllowedNumericType::All)
        }
        pub fn parse_non_negative(input: &mut Parser) -> Result<Length, ()> {
            Length::parse_internal(input, &AllowedNumericType::NonNegative)
        }
        pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Length, ()> {
            match_ignore_ascii_case! { unit,
                "px" => Ok(Length::from_px(value)),
                "in" => Ok(Length::Absolute(Au((value * AU_PER_IN) as i32))),
                "cm" => Ok(Length::Absolute(Au((value * AU_PER_CM) as i32))),
                "mm" => Ok(Length::Absolute(Au((value * AU_PER_MM) as i32))),
                "q" => Ok(Length::Absolute(Au((value * AU_PER_Q) as i32))),
                "pt" => Ok(Length::Absolute(Au((value * AU_PER_PT) as i32))),
                "pc" => Ok(Length::Absolute(Au((value * AU_PER_PC) as i32))),
                // font-relative
                "em" => Ok(Length::FontRelative(FontRelativeLength::Em(value))),
                "ex" => Ok(Length::FontRelative(FontRelativeLength::Ex(value))),
                "ch" => Ok(Length::FontRelative(FontRelativeLength::Ch(value))),
                "rem" => Ok(Length::FontRelative(FontRelativeLength::Rem(value))),
                // viewport percentages
                "vw" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vw(value))),
                "vh" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vh(value))),
                "vmin" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vmin(value))),
                "vmax" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vmax(value))),
                _ => Err(())
            }
        }
        #[inline]
        pub fn from_px(px_value: CSSFloat) -> Length {
            Length::Absolute(Au((px_value * AU_PER_PX) as i32))
        }
    }

    #[derive(Clone, Debug)]
    struct CalcSumNode {
        products: Vec<CalcProductNode>,
    }

    #[derive(Clone, Debug)]
    struct CalcProductNode {
        values: Vec<CalcValueNode>
    }

    #[derive(Clone, Debug)]
    enum CalcValueNode {
        Length(Length),
        Angle(Angle),
        Time(Time),
        Percentage(CSSFloat),
        Number(CSSFloat),
        Sum(Box<CalcSumNode>),
    }

    #[derive(Clone, Debug)]
    struct SimplifiedSumNode {
        values: Vec<SimplifiedValueNode>,
    }
    impl<'a> Mul<CSSFloat> for &'a SimplifiedSumNode {
        type Output = SimplifiedSumNode;

        #[inline]
        fn mul(self, scalar: CSSFloat) -> SimplifiedSumNode {
            SimplifiedSumNode {
                values: self.values.iter().map(|p| p * scalar).collect()
            }
        }
    }

    #[derive(Clone, Debug)]
    enum SimplifiedValueNode {
        Length(Length),
        Angle(Angle),
        Time(Time),
        Percentage(CSSFloat),
        Number(CSSFloat),
        Sum(Box<SimplifiedSumNode>),
    }
    impl<'a> Mul<CSSFloat> for &'a SimplifiedValueNode {
        type Output = SimplifiedValueNode;

        #[inline]
        fn mul(self, scalar: CSSFloat) -> SimplifiedValueNode {
            match *self {
                SimplifiedValueNode::Length(l) => SimplifiedValueNode::Length(l * scalar),
                SimplifiedValueNode::Percentage(p) => SimplifiedValueNode::Percentage(p * scalar),
                SimplifiedValueNode::Angle(Angle(a)) => SimplifiedValueNode::Angle(Angle(a * scalar)),
                SimplifiedValueNode::Time(Time(t)) => SimplifiedValueNode::Time(Time(t * scalar)),
                SimplifiedValueNode::Number(n) => SimplifiedValueNode::Number(n * scalar),
                SimplifiedValueNode::Sum(ref s) => {
                    let sum = &**s * scalar;
                    SimplifiedValueNode::Sum(Box::new(sum))
                }
            }
        }
    }

    pub fn parse_integer(input: &mut Parser) -> Result<i32, ()> {
        match try!(input.next()) {
            Token::Number(ref value) => value.int_value.ok_or(()),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let ast = try!(input.parse_nested_block(|i| CalcLengthOrPercentage::parse_sum(i, CalcUnit::Integer)));

                let mut result = None;

                for ref node in ast.products {
                    match try!(CalcLengthOrPercentage::simplify_product(node)) {
                        SimplifiedValueNode::Number(val) =>
                            result = Some(result.unwrap_or(0) + val as i32),
                        _ => unreachable!()
                    }
                }

                match result {
                    Some(result) => Ok(result),
                    _ => Err(())
                }
            }
            _ => Err(())
        }
    }

    pub fn parse_number(input: &mut Parser) -> Result<f32, ()> {
        match try!(input.next()) {
            Token::Number(ref value) => Ok(value.value),
            Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                let ast = try!(input.parse_nested_block(|i| CalcLengthOrPercentage::parse_sum(i, CalcUnit::Number)));

                let mut result = None;

                for ref node in ast.products {
                    match try!(CalcLengthOrPercentage::simplify_product(node)) {
                        SimplifiedValueNode::Number(val) =>
                            result = Some(result.unwrap_or(0.) + val),
                        _ => unreachable!()
                    }
                }

                match result {
                    Some(result) => Ok(result),
                    _ => Err(())
                }
            }
            _ => Err(())
        }
    }

    #[derive(Clone, Copy, PartialEq)]
    enum CalcUnit {
        Number,
        Integer,
        Length,
        LengthOrPercentage,
        Angle,
        Time,
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct CalcLengthOrPercentage {
        pub absolute: Option<Au>,
        pub vw: Option<ViewportPercentageLength>,
        pub vh: Option<ViewportPercentageLength>,
        pub vmin: Option<ViewportPercentageLength>,
        pub vmax: Option<ViewportPercentageLength>,
        pub em: Option<FontRelativeLength>,
        pub ex: Option<FontRelativeLength>,
        pub ch: Option<FontRelativeLength>,
        pub rem: Option<FontRelativeLength>,
        pub percentage: Option<Percentage>,
    }
    impl CalcLengthOrPercentage {
        fn parse_sum(input: &mut Parser, expected_unit: CalcUnit) -> Result<CalcSumNode, ()> {
            let mut products = Vec::new();
            products.push(try!(CalcLengthOrPercentage::parse_product(input, expected_unit)));

            while let Ok(token) = input.next() {
                match token {
                    Token::Delim('+') => {
                        products.push(try!(CalcLengthOrPercentage::parse_product(input, expected_unit)));
                    }
                    Token::Delim('-') => {
                        let mut right = try!(CalcLengthOrPercentage::parse_product(input, expected_unit));
                        right.values.push(CalcValueNode::Number(-1.));
                        products.push(right);
                    }
                    _ => return Err(())
                }
            }

            Ok(CalcSumNode { products: products })
        }

        fn parse_product(input: &mut Parser, expected_unit: CalcUnit) -> Result<CalcProductNode, ()> {
            let mut values = Vec::new();
            values.push(try!(CalcLengthOrPercentage::parse_value(input, expected_unit)));

            loop {
                let position = input.position();
                match input.next() {
                    Ok(Token::Delim('*')) => {
                        values.push(try!(CalcLengthOrPercentage::parse_value(input, expected_unit)));
                    }
                    Ok(Token::Delim('/')) if expected_unit != CalcUnit::Integer => {
                        if let Ok(Token::Number(ref value)) = input.next() {
                            if value.value == 0. {
                                return Err(());
                            }
                            values.push(CalcValueNode::Number(1. / value.value));
                        } else {
                            return Err(());
                        }
                    }
                    _ => {
                        input.reset(position);
                        break
                    }
                }
            }

            Ok(CalcProductNode { values: values })
        }

        fn parse_value(input: &mut Parser, expected_unit: CalcUnit) -> Result<CalcValueNode, ()> {
            match (try!(input.next()), expected_unit) {
                (Token::Number(ref value), _) => Ok(CalcValueNode::Number(value.value)),
                (Token::Dimension(ref value, ref unit), CalcUnit::Length) |
                (Token::Dimension(ref value, ref unit), CalcUnit::LengthOrPercentage) => {
                    Length::parse_dimension(value.value, unit).map(CalcValueNode::Length)
                }
                (Token::Dimension(ref value, ref unit), CalcUnit::Angle) => {
                    Angle::parse_dimension(value.value, unit).map(CalcValueNode::Angle)
                }
                (Token::Dimension(ref value, ref unit), CalcUnit::Time) => {
                    Time::parse_dimension(value.value, unit).map(CalcValueNode::Time)
                }
                (Token::Percentage(ref value), CalcUnit::LengthOrPercentage) =>
                    Ok(CalcValueNode::Percentage(value.unit_value)),
                (Token::ParenthesisBlock, _) => {
                    input.parse_nested_block(|i| CalcLengthOrPercentage::parse_sum(i, expected_unit))
                         .map(|result| CalcValueNode::Sum(Box::new(result)))
                },
                _ => Err(())
            }
        }

        fn simplify_value_to_number(node: &CalcValueNode) -> Option<CSSFloat> {
            match *node {
                CalcValueNode::Number(number) => Some(number),
                CalcValueNode::Sum(ref sum) => CalcLengthOrPercentage::simplify_sum_to_number(sum),
                _ => None
            }
        }

        fn simplify_sum_to_number(node: &CalcSumNode) -> Option<CSSFloat> {
            let mut sum = 0.;
            for ref product in &node.products {
                match CalcLengthOrPercentage::simplify_product_to_number(product) {
                    Some(number) => sum += number,
                    _ => return None
                }
            }
            Some(sum)
        }

        fn simplify_product_to_number(node: &CalcProductNode) -> Option<CSSFloat> {
            let mut product = 1.;
            for ref value in &node.values {
                match CalcLengthOrPercentage::simplify_value_to_number(value) {
                    Some(number) => product *= number,
                    _ => return None
                }
            }
            Some(product)
        }

        fn simplify_products_in_sum(node: &CalcSumNode) -> Result<SimplifiedValueNode, ()> {
            let mut simplified = Vec::new();
            for product in &node.products {
                match try!(CalcLengthOrPercentage::simplify_product(product)) {
                    SimplifiedValueNode::Sum(ref sum) => simplified.extend_from_slice(&sum.values),
                    val => simplified.push(val),
                }
            }

            if simplified.len() == 1 {
                Ok(simplified[0].clone())
            } else {
                Ok(SimplifiedValueNode::Sum(Box::new(SimplifiedSumNode { values: simplified })))
            }
        }

        fn simplify_product(node: &CalcProductNode) -> Result<SimplifiedValueNode, ()> {
            let mut multiplier = 1.;
            let mut node_with_unit = None;
            for node in &node.values {
                match CalcLengthOrPercentage::simplify_value_to_number(&node) {
                    Some(number) => multiplier *= number,
                    _ if node_with_unit.is_none() => {
                        node_with_unit = Some(match *node {
                            CalcValueNode::Sum(ref sum) =>
                                try!(CalcLengthOrPercentage::simplify_products_in_sum(sum)),
                            CalcValueNode::Length(l) => SimplifiedValueNode::Length(l),
                            CalcValueNode::Angle(a) => SimplifiedValueNode::Angle(a),
                            CalcValueNode::Time(t) => SimplifiedValueNode::Time(t),
                            CalcValueNode::Percentage(p) => SimplifiedValueNode::Percentage(p),
                            _ => unreachable!("Numbers should have been handled by simplify_value_to_nubmer")
                        })
                    },
                    _ => return Err(()),
                }
            }

            match node_with_unit {
                None => Ok(SimplifiedValueNode::Number(multiplier)),
                Some(ref value) => Ok(value * multiplier)
            }
        }

        fn parse_length(input: &mut Parser) -> Result<Length, ()> {
            CalcLengthOrPercentage::parse(input, CalcUnit::Length).map(Length::Calc)
        }

        fn parse_length_or_percentage(input: &mut Parser) -> Result<CalcLengthOrPercentage, ()> {
            CalcLengthOrPercentage::parse(input, CalcUnit::LengthOrPercentage)
        }

        fn parse(input: &mut Parser, expected_unit: CalcUnit) -> Result<CalcLengthOrPercentage, ()> {
            let ast = try!(CalcLengthOrPercentage::parse_sum(input, expected_unit));

            let mut simplified = Vec::new();
            for ref node in ast.products {
                match try!(CalcLengthOrPercentage::simplify_product(node)) {
                    SimplifiedValueNode::Sum(sum) => simplified.extend_from_slice(&sum.values),
                    value => simplified.push(value),
                }
            }

            let mut absolute = None;
            let mut vw = None;
            let mut vh = None;
            let mut vmax = None;
            let mut vmin = None;
            let mut em = None;
            let mut ex = None;
            let mut ch = None;
            let mut rem = None;
            let mut percentage = None;
            let mut number = None;

            for value in simplified {
                match value {
                    SimplifiedValueNode::Percentage(p) =>
                        percentage = Some(percentage.unwrap_or(0.) + p),
                    SimplifiedValueNode::Length(Length::Absolute(Au(au))) =>
                        absolute = Some(absolute.unwrap_or(0) + au),
                    SimplifiedValueNode::Length(Length::ViewportPercentage(v)) =>
                        match v {
                            ViewportPercentageLength::Vw(val) =>
                                vw = Some(vw.unwrap_or(0.) + val),
                            ViewportPercentageLength::Vh(val) =>
                                vh = Some(vh.unwrap_or(0.) + val),
                            ViewportPercentageLength::Vmin(val) =>
                                vmin = Some(vmin.unwrap_or(0.) + val),
                            ViewportPercentageLength::Vmax(val) =>
                                vmax = Some(vmax.unwrap_or(0.) + val),
                        },
                    SimplifiedValueNode::Length(Length::FontRelative(f)) =>
                        match f {
                            FontRelativeLength::Em(val) =>
                                em = Some(em.unwrap_or(0.) + val),
                            FontRelativeLength::Ex(val) =>
                                ex = Some(ex.unwrap_or(0.) + val),
                            FontRelativeLength::Ch(val) =>
                                ch = Some(ch.unwrap_or(0.) + val),
                            FontRelativeLength::Rem(val) =>
                                rem = Some(rem.unwrap_or(0.) + val),
                        },
                    SimplifiedValueNode::Number(val) => number = Some(number.unwrap_or(0.) + val),
                    _ => return Err(()),
                }
            }

            Ok(CalcLengthOrPercentage {
                absolute: absolute.map(Au),
                vw: vw.map(ViewportPercentageLength::Vw),
                vh: vh.map(ViewportPercentageLength::Vh),
                vmax: vmax.map(ViewportPercentageLength::Vmax),
                vmin: vmin.map(ViewportPercentageLength::Vmin),
                em: em.map(FontRelativeLength::Em),
                ex: ex.map(FontRelativeLength::Ex),
                ch: ch.map(FontRelativeLength::Ch),
                rem: rem.map(FontRelativeLength::Rem),
                percentage: percentage.map(Percentage),
            })
        }

        pub fn parse_time(input: &mut Parser) -> Result<Time, ()> {
            let ast = try!(CalcLengthOrPercentage::parse_sum(input, CalcUnit::Time));

            let mut simplified = Vec::new();
            for ref node in ast.products {
                match try!(CalcLengthOrPercentage::simplify_product(node)) {
                    SimplifiedValueNode::Sum(sum) => simplified.extend_from_slice(&sum.values),
                    value => simplified.push(value),
                }
            }

            let mut time = None;

            for value in simplified {
                match value {
                    SimplifiedValueNode::Time(Time(val)) =>
                        time = Some(time.unwrap_or(0.) + val),
                    _ => return Err(()),
                }
            }

            match time {
                Some(time) => Ok(Time(time)),
                _ => Err(())
            }
        }

        pub fn parse_angle(input: &mut Parser) -> Result<Angle, ()> {
            let ast = try!(CalcLengthOrPercentage::parse_sum(input, CalcUnit::Angle));

            let mut simplified = Vec::new();
            for ref node in ast.products {
                match try!(CalcLengthOrPercentage::simplify_product(node)) {
                    SimplifiedValueNode::Sum(sum) => simplified.extend_from_slice(&sum.values),
                    value => simplified.push(value),
                }
            }

            let mut angle = None;
            let mut number = None;

            for value in simplified {
                match value {
                    SimplifiedValueNode::Angle(Angle(val)) =>
                        angle = Some(angle.unwrap_or(0.) + val),
                    SimplifiedValueNode::Number(val) => number = Some(number.unwrap_or(0.) + val),
                    _ => unreachable!()
                }
            }

            match (angle, number) {
                (Some(angle), None) => Ok(Angle(angle)),
                (None, Some(value)) if value == 0. => Ok(Angle(0.)),
                _ => Err(())
            }
        }
    }

    impl ToCss for CalcLengthOrPercentage {
        #[allow(unused_assignments)]
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            macro_rules! count {
                ( $( $val:ident ),* ) => {
                    {
                        let mut count = 0;
                        $(
                            if let Some(_) = self.$val {
                                count += 1;
                            }
                        )*
                        count
                     }
                };
            }

            macro_rules! serialize {
                ( $( $val:ident ),* ) => {
                    {
                        let mut first_value = true;
                        $(
                            if let Some(val) = self.$val {
                                if !first_value {
                                    try!(write!(dest, " + "));
                                } else {
                                    first_value = false;
                                }
                                try!(val.to_css(dest));
                            }
                        )*
                     }
                };
            }

            let count = count!(ch, em, ex, absolute, rem, vh, vmax, vmin, vw, percentage);
            assert!(count > 0);

            if count > 1 {
               try!(write!(dest, "calc("));
            }

            serialize!(ch, em, ex, absolute, rem, vh, vmax, vmin, vw, percentage);

            if count > 1 {
               try!(write!(dest, ")"));
            }
            Ok(())
         }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct Percentage(pub CSSFloat); // [0 .. 100%] maps to [0.0 .. 1.0]

    impl ToCss for Percentage {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            write!(dest, "{}%", self.0 * 100.)
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrPercentage {
        Length(Length),
        Percentage(Percentage),
        Calc(CalcLengthOrPercentage),
    }

    impl HasViewportPercentage for LengthOrPercentage {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                LengthOrPercentage::Length(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    impl ToCss for LengthOrPercentage {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrPercentage::Length(length) => length.to_css(dest),
                LengthOrPercentage::Percentage(percentage) => percentage.to_css(dest),
                LengthOrPercentage::Calc(calc) => calc.to_css(dest),
            }
        }
    }
    impl LengthOrPercentage {
        pub fn zero() -> LengthOrPercentage {
            LengthOrPercentage::Length(Length::Absolute(Au(0)))
        }

        fn parse_internal(input: &mut Parser, context: &AllowedNumericType)
                          -> Result<LengthOrPercentage, ()>
        {
            match try!(input.next()) {
                Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                    Length::parse_dimension(value.value, unit).map(LengthOrPercentage::Length),
                Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                    Ok(LengthOrPercentage::Percentage(Percentage(value.unit_value))),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(LengthOrPercentage::Length(Length::Absolute(Au(0)))),
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                    let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                    Ok(LengthOrPercentage::Calc(calc))
                },
                _ => Err(())
            }
        }
        #[inline]
        pub fn parse(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
            LengthOrPercentage::parse_internal(input, &AllowedNumericType::All)
        }
        #[inline]
        pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
            LengthOrPercentage::parse_internal(input, &AllowedNumericType::NonNegative)
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrPercentageOrAuto {
        Length(Length),
        Percentage(Percentage),
        Auto,
        Calc(CalcLengthOrPercentage),
    }

    impl HasViewportPercentage for LengthOrPercentageOrAuto {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                LengthOrPercentageOrAuto::Length(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    impl ToCss for LengthOrPercentageOrAuto {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrPercentageOrAuto::Length(length) => length.to_css(dest),
                LengthOrPercentageOrAuto::Percentage(percentage) => percentage.to_css(dest),
                LengthOrPercentageOrAuto::Auto => dest.write_str("auto"),
                LengthOrPercentageOrAuto::Calc(calc) => calc.to_css(dest),
            }
        }
    }

    impl LengthOrPercentageOrAuto {
        fn parse_internal(input: &mut Parser, context: &AllowedNumericType)
                          -> Result<LengthOrPercentageOrAuto, ()>
        {
            match try!(input.next()) {
                Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                    Length::parse_dimension(value.value, unit).map(LengthOrPercentageOrAuto::Length),
                Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                    Ok(LengthOrPercentageOrAuto::Percentage(Percentage(value.unit_value))),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(LengthOrPercentageOrAuto::Length(Length::Absolute(Au(0)))),
                Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                    Ok(LengthOrPercentageOrAuto::Auto),
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                    let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                    Ok(LengthOrPercentageOrAuto::Calc(calc))
                },
                _ => Err(())
            }
        }
        #[inline]
        pub fn parse(input: &mut Parser) -> Result<LengthOrPercentageOrAuto, ()> {
            LengthOrPercentageOrAuto::parse_internal(input, &AllowedNumericType::All)
        }
        #[inline]
        pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentageOrAuto, ()> {
            LengthOrPercentageOrAuto::parse_internal(input, &AllowedNumericType::NonNegative)
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrPercentageOrNone {
        Length(Length),
        Percentage(Percentage),
        Calc(CalcLengthOrPercentage),
        None,
    }

    impl HasViewportPercentage for LengthOrPercentageOrNone {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                LengthOrPercentageOrNone::Length(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    impl ToCss for LengthOrPercentageOrNone {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrPercentageOrNone::Length(length) => length.to_css(dest),
                LengthOrPercentageOrNone::Percentage(percentage) => percentage.to_css(dest),
                LengthOrPercentageOrNone::Calc(calc) => calc.to_css(dest),
                LengthOrPercentageOrNone::None => dest.write_str("none"),
            }
        }
    }
    impl LengthOrPercentageOrNone {
        fn parse_internal(input: &mut Parser, context: &AllowedNumericType)
                          -> Result<LengthOrPercentageOrNone, ()>
        {
            match try!(input.next()) {
                Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                    Length::parse_dimension(value.value, unit).map(LengthOrPercentageOrNone::Length),
                Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                    Ok(LengthOrPercentageOrNone::Percentage(Percentage(value.unit_value))),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(LengthOrPercentageOrNone::Length(Length::Absolute(Au(0)))),
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                    let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                    Ok(LengthOrPercentageOrNone::Calc(calc))
                },
                Token::Ident(ref value) if value.eq_ignore_ascii_case("none") =>
                    Ok(LengthOrPercentageOrNone::None),
                _ => Err(())
            }
        }
        #[inline]
        pub fn parse(input: &mut Parser) -> Result<LengthOrPercentageOrNone, ()> {
            LengthOrPercentageOrNone::parse_internal(input, &AllowedNumericType::All)
        }
        #[inline]
        pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentageOrNone, ()> {
            LengthOrPercentageOrNone::parse_internal(input, &AllowedNumericType::NonNegative)
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrNone {
        Length(Length),
        None,
    }

    impl HasViewportPercentage for LengthOrNone {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                LengthOrNone::Length(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    impl ToCss for LengthOrNone {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrNone::Length(length) => length.to_css(dest),
                LengthOrNone::None => dest.write_str("none"),
            }
        }
    }
    impl LengthOrNone {
        fn parse_internal(input: &mut Parser, context: &AllowedNumericType)
                          -> Result<LengthOrNone, ()>
        {
            match try!(input.next()) {
                Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                    Length::parse_dimension(value.value, unit).map(LengthOrNone::Length),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(LengthOrNone::Length(Length::Absolute(Au(0)))),
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") =>
                    input.parse_nested_block(CalcLengthOrPercentage::parse_length).map(LengthOrNone::Length),
                Token::Ident(ref value) if value.eq_ignore_ascii_case("none") =>
                    Ok(LengthOrNone::None),
                _ => Err(())
            }
        }
        #[inline]
        pub fn parse(input: &mut Parser) -> Result<LengthOrNone, ()> {
            LengthOrNone::parse_internal(input, &AllowedNumericType::All)
        }
        #[inline]
        pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrNone, ()> {
            LengthOrNone::parse_internal(input, &AllowedNumericType::NonNegative)
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrPercentageOrAutoOrContent {
        Length(Length),
        Percentage(Percentage),
        Calc(CalcLengthOrPercentage),
        Auto,
        Content
    }

    impl HasViewportPercentage for LengthOrPercentageOrAutoOrContent {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                LengthOrPercentageOrAutoOrContent::Length(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    impl ToCss for LengthOrPercentageOrAutoOrContent {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrPercentageOrAutoOrContent::Length(len) => len.to_css(dest),
                LengthOrPercentageOrAutoOrContent::Percentage(perc) => perc.to_css(dest),
                LengthOrPercentageOrAutoOrContent::Auto => dest.write_str("auto"),
                LengthOrPercentageOrAutoOrContent::Content => dest.write_str("content"),
                LengthOrPercentageOrAutoOrContent::Calc(calc) => calc.to_css(dest),
            }
        }
    }

    impl LengthOrPercentageOrAutoOrContent {
        pub fn parse(input: &mut Parser) -> Result<LengthOrPercentageOrAutoOrContent, ()> {
            let context = AllowedNumericType::NonNegative;
            match try!(input.next()) {
                Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                    Length::parse_dimension(value.value, unit).map(LengthOrPercentageOrAutoOrContent::Length),
                Token::Percentage(ref value) if context.is_ok(value.unit_value) =>
                    Ok(LengthOrPercentageOrAutoOrContent::Percentage(Percentage(value.unit_value))),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(LengthOrPercentageOrAutoOrContent::Length(Length::Absolute(Au(0)))),
                Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                    Ok(LengthOrPercentageOrAutoOrContent::Auto),
                Token::Ident(ref value) if value.eq_ignore_ascii_case("content") =>
                    Ok(LengthOrPercentageOrAutoOrContent::Content),
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                    let calc = try!(input.parse_nested_block(CalcLengthOrPercentage::parse_length_or_percentage));
                    Ok(LengthOrPercentageOrAutoOrContent::Calc(calc))
                },
                _ => Err(())
            }
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct BorderRadiusSize(pub Size2D<LengthOrPercentage>);

    impl NoViewportPercentage for BorderRadiusSize {}

    impl BorderRadiusSize {
        pub fn zero() -> BorderRadiusSize {
            let zero = LengthOrPercentage::Length(Length::Absolute(Au(0)));
                BorderRadiusSize(Size2D::new(zero, zero))
        }

        pub fn new(width: LengthOrPercentage, height: LengthOrPercentage) -> BorderRadiusSize {
            BorderRadiusSize(Size2D::new(width, height))
        }

        pub fn circle(radius: LengthOrPercentage) -> BorderRadiusSize {
            BorderRadiusSize(Size2D::new(radius, radius))
        }

        #[inline]
        pub fn parse(input: &mut Parser) -> Result<BorderRadiusSize, ()> {
            let first = try!(LengthOrPercentage::parse_non_negative(input));
            let second = input.try(LengthOrPercentage::parse_non_negative).unwrap_or(first);
            Ok(BorderRadiusSize(Size2D::new(first, second)))
        }
    }

    impl ToCss for BorderRadiusSize {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.0.width.to_css(dest));
            try!(dest.write_str(" "));
            self.0.height.to_css(dest)
        }
    }

    // http://dev.w3.org/csswg/css2/colors.html#propdef-background-position
    #[derive(Clone, PartialEq, Copy)]
    pub enum PositionComponent {
        LengthOrPercentage(LengthOrPercentage),
        Center,
        Left,
        Right,
        Top,
        Bottom,
    }

    impl HasViewportPercentage for PositionComponent {
        fn has_viewport_percentage(&self) -> bool {
            match *self {
                PositionComponent::LengthOrPercentage(length) => length.has_viewport_percentage(),
                _ => false
            }
        }
    }

    impl PositionComponent {
        pub fn parse(input: &mut Parser) -> Result<PositionComponent, ()> {
            input.try(LengthOrPercentage::parse)
            .map(PositionComponent::LengthOrPercentage)
            .or_else(|()| {
                match try!(input.next()) {
                    Token::Ident(value) => {
                        match_ignore_ascii_case! { value,
                            "center" => Ok(PositionComponent::Center),
                            "left" => Ok(PositionComponent::Left),
                            "right" => Ok(PositionComponent::Right),
                            "top" => Ok(PositionComponent::Top),
                            "bottom" => Ok(PositionComponent::Bottom),
                            _ => Err(())
                        }
                    },
                    _ => Err(())
                }
            })
        }
        #[inline]
        pub fn to_length_or_percentage(self) -> LengthOrPercentage {
            match self {
                PositionComponent::LengthOrPercentage(value) => value,
                PositionComponent::Center => LengthOrPercentage::Percentage(Percentage(0.5)),
                PositionComponent::Left |
                PositionComponent::Top => LengthOrPercentage::Percentage(Percentage(0.0)),
                PositionComponent::Right |
                PositionComponent::Bottom => LengthOrPercentage::Percentage(Percentage(1.0)),
            }
        }
    }

    #[derive(Clone, PartialEq, PartialOrd, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf, Deserialize, Serialize))]
    /// An angle, normalized to radians.
    pub struct Angle(pub CSSFloat);

    impl ToCss for Angle {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            write!(dest, "{}rad", self.0)
        }
    }

    impl Angle {
        #[inline]
        pub fn radians(self) -> f32 {
            self.0
        }

        #[inline]
        pub fn from_radians(r: f32) -> Self {
            Angle(r)
        }
    }

    const RAD_PER_DEG: CSSFloat = PI / 180.0;
    const RAD_PER_GRAD: CSSFloat = PI / 200.0;
    const RAD_PER_TURN: CSSFloat = PI * 2.0;

    impl Angle {
        /// Parses an angle according to CSS-VALUES § 6.1.
        pub fn parse(input: &mut Parser) -> Result<Angle, ()> {
            match try!(input.next()) {
                Token::Dimension(ref value, ref unit) => Angle::parse_dimension(value.value, unit),
                Token::Number(ref value) if value.value == 0. => Ok(Angle(0.)),
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                    input.parse_nested_block(CalcLengthOrPercentage::parse_angle)
                },
                _ => Err(())
            }
        }

        pub fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Angle, ()> {
            match_ignore_ascii_case! { unit,
                "deg" => Ok(Angle(value * RAD_PER_DEG)),
                "grad" => Ok(Angle(value * RAD_PER_GRAD)),
                "turn" => Ok(Angle(value * RAD_PER_TURN)),
                "rad" => Ok(Angle(value)),
                 _ => Err(())
            }
        }
    }

    /// Specified values for an image according to CSS-IMAGES.
    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum Image {
        Url(Url),
        LinearGradient(LinearGradient),
    }

    impl ToCss for Image {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            use values::LocalToCss;
            match *self {
                Image::Url(ref url) => {
                    url.to_css(dest)
                }
                Image::LinearGradient(ref gradient) => gradient.to_css(dest)
            }
        }
    }

    impl Image {
        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Image, ()> {
            if let Ok(url) = input.try(|input| input.expect_url()) {
                Ok(Image::Url(context.parse_url(&url)))
            } else {
                match_ignore_ascii_case! { try!(input.expect_function()),
                    "linear-gradient" => {
                        Ok(Image::LinearGradient(try!(
                            input.parse_nested_block(LinearGradient::parse_function))))
                    },
                    _ => Err(())
                }
            }
        }
    }

    /// Specified values for a CSS linear gradient.
    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct LinearGradient {
        /// The angle or corner of the gradient.
        pub angle_or_corner: AngleOrCorner,

        /// The color stops.
        pub stops: Vec<ColorStop>,
    }

    impl ToCss for LinearGradient {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(dest.write_str("linear-gradient("));
            try!(self.angle_or_corner.to_css(dest));
            for stop in &self.stops {
                try!(dest.write_str(", "));
                try!(stop.to_css(dest));
            }
            try!(dest.write_str(")"));
            Ok(())
        }
    }

    /// Specified values for an angle or a corner in a linear gradient.
    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum AngleOrCorner {
        Angle(Angle),
        Corner(HorizontalDirection, VerticalDirection),
    }

    impl ToCss for AngleOrCorner {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                AngleOrCorner::Angle(angle) => angle.to_css(dest),
                AngleOrCorner::Corner(horizontal, vertical) => {
                    try!(dest.write_str("to "));
                    try!(horizontal.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(vertical.to_css(dest));
                    Ok(())
                }
            }
        }
    }

    /// Specified values for one color stop in a linear gradient.
    #[derive(Clone, PartialEq, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct ColorStop {
        /// The color of this stop.
        pub color: CSSColor,

        /// The position of this stop. If not specified, this stop is placed halfway between the
        /// point that precedes it and the point that follows it.
        pub position: Option<LengthOrPercentage>,
    }

    impl ToCss for ColorStop {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.color.to_css(dest));
            if let Some(position) = self.position {
                try!(dest.write_str(" "));
                try!(position.to_css(dest));
            }
            Ok(())
        }
    }

    define_css_keyword_enum!(HorizontalDirection: "left" => Left, "right" => Right);
    define_css_keyword_enum!(VerticalDirection: "top" => Top, "bottom" => Bottom);

    fn parse_one_color_stop(input: &mut Parser) -> Result<ColorStop, ()> {
        Ok(ColorStop {
            color: try!(CSSColor::parse(input)),
            position: input.try(LengthOrPercentage::parse).ok(),
        })
    }

    impl LinearGradient {
        /// Parses a linear gradient from the given arguments.
        pub fn parse_function(input: &mut Parser) -> Result<LinearGradient, ()> {
            let angle_or_corner = if input.try(|input| input.expect_ident_matching("to")).is_ok() {
                let (horizontal, vertical) =
                if let Ok(value) = input.try(HorizontalDirection::parse) {
                    (Some(value), input.try(VerticalDirection::parse).ok())
                } else {
                    let value = try!(VerticalDirection::parse(input));
                    (input.try(HorizontalDirection::parse).ok(), Some(value))
                };
                try!(input.expect_comma());
                match (horizontal, vertical) {
                    (None, Some(VerticalDirection::Top)) => {
                        AngleOrCorner::Angle(Angle(0.0))
                    },
                    (Some(HorizontalDirection::Right), None) => {
                        AngleOrCorner::Angle(Angle(PI * 0.5))
                    },
                    (None, Some(VerticalDirection::Bottom)) => {
                        AngleOrCorner::Angle(Angle(PI))
                    },
                    (Some(HorizontalDirection::Left), None) => {
                        AngleOrCorner::Angle(Angle(PI * 1.5))
                    },
                    (Some(horizontal), Some(vertical)) => {
                        AngleOrCorner::Corner(horizontal, vertical)
                    }
                    (None, None) => unreachable!(),
                }
            } else if let Ok(angle) = input.try(Angle::parse) {
                try!(input.expect_comma());
                AngleOrCorner::Angle(angle)
            } else {
                AngleOrCorner::Angle(Angle(PI))
            };
            // Parse the color stops.
            let stops = try!(input.parse_comma_separated(parse_one_color_stop));
            if stops.len() < 2 {
                return Err(())
            }
            Ok(LinearGradient {
                angle_or_corner: angle_or_corner,
                stops: stops,
            })
        }
    }

    pub fn parse_border_radius(input: &mut Parser) -> Result<BorderRadiusSize, ()> {
        input.try(BorderRadiusSize::parse).or_else(|()| {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    "thin" => Ok(BorderRadiusSize::circle(
                                     LengthOrPercentage::Length(Length::from_px(1.)))),
                    "medium" => Ok(BorderRadiusSize::circle(
                                       LengthOrPercentage::Length(Length::from_px(3.)))),
                    "thick" => Ok(BorderRadiusSize::circle(
                                      LengthOrPercentage::Length(Length::from_px(5.)))),
                    _ => Err(())
                }
            })
    }

    pub fn parse_border_width(input: &mut Parser) -> Result<Length, ()> {
        input.try(Length::parse_non_negative).or_else(|()| {
            match_ignore_ascii_case! { try!(input.expect_ident()),
                "thin" => Ok(Length::from_px(1.)),
                "medium" => Ok(Length::from_px(3.)),
                "thick" => Ok(Length::from_px(5.)),
                _ => Err(())
            }
        })
    }

    // The integer values here correspond to the border conflict resolution rules in CSS 2.1 §
    // 17.6.2.1. Higher values override lower values.
    define_numbered_css_keyword_enum! { BorderStyle:
        "none" => none = -1,
        "solid" => solid = 6,
        "double" => double = 7,
        "dotted" => dotted = 4,
        "dashed" => dashed = 5,
        "hidden" => hidden = -2,
        "groove" => groove = 1,
        "ridge" => ridge = 3,
        "inset" => inset = 0,
        "outset" => outset = 2,
    }

    impl NoViewportPercentage for BorderStyle {}

    impl BorderStyle {
        pub fn none_or_hidden(&self) -> bool {
            matches!(*self, BorderStyle::none | BorderStyle::hidden)
        }
    }

    /// A time in seconds according to CSS-VALUES § 6.2.
    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct Time(pub CSSFloat);

    impl Time {
        /// Returns the time in fractional seconds.
        pub fn seconds(self) -> f32 {
            let Time(seconds) = self;
            seconds
        }

        /// Parses a time according to CSS-VALUES § 6.2.
        fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Time, ()> {
            if unit.eq_ignore_ascii_case("s") {
                Ok(Time(value))
            } else if unit.eq_ignore_ascii_case("ms") {
                Ok(Time(value / 1000.0))
            } else {
                Err(())
            }
        }

        pub fn parse(input: &mut Parser) -> Result<Time, ()> {
            match input.next() {
                Ok(Token::Dimension(ref value, ref unit)) => {
                    Time::parse_dimension(value.value, &unit)
                }
                Ok(Token::Function(ref name)) if name.eq_ignore_ascii_case("calc") => {
                    input.parse_nested_block(CalcLengthOrPercentage::parse_time)
                }
                _ => Err(())
            }
        }
    }

    impl ToComputedValue for Time {
        type ComputedValue = Time;

        #[inline]
        fn to_computed_value(&self, _: &Context) -> Time {
            *self
        }
    }

    impl ToCss for Time {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            write!(dest, "{}s", self.0)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct Number(pub CSSFloat);

    impl NoViewportPercentage for Number {}

    impl Number {
        pub fn parse(input: &mut Parser) -> Result<Number, ()> {
            parse_number(input).map(Number)
        }

        fn parse_with_minimum(input: &mut Parser, min: CSSFloat) -> Result<Number, ()> {
            match parse_number(input) {
                Ok(value) if value < min => Err(()),
                value => value.map(Number),
            }
        }

        pub fn parse_non_negative(input: &mut Parser) -> Result<Number, ()> {
            Number::parse_with_minimum(input, 0.0)
        }

        pub fn parse_at_least_one(input: &mut Parser) -> Result<Number, ()> {
            Number::parse_with_minimum(input, 1.0)
        }
    }

    impl ToComputedValue for Number {
        type ComputedValue = CSSFloat;

        #[inline]
        fn to_computed_value(&self, _: &Context) -> CSSFloat { self.0 }
    }

    impl ToCss for Number {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct Opacity(pub CSSFloat);

    impl NoViewportPercentage for Opacity {}

    impl Opacity {
        pub fn parse(input: &mut Parser) -> Result<Opacity, ()> {
            parse_number(input).map(Opacity)
        }
    }

    impl ToComputedValue for Opacity {
        type ComputedValue = CSSFloat;

        #[inline]
        fn to_computed_value(&self, _: &Context) -> CSSFloat {
            if self.0 < 0.0 {
                0.0
            } else if self.0 > 1.0 {
                1.0
            } else {
                self.0
            }
        }
    }

    impl ToCss for Opacity {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            self.0.to_css(dest)
        }
    }
}

pub mod computed {
    use app_units::Au;
    use euclid::size::Size2D;
    use properties::ComputedValues;
    use std::fmt;
    use super::LocalToCss;
    use super::specified::AngleOrCorner;
    use super::{CSSFloat, specified};
    use url::Url;
    pub use cssparser::Color as CSSColor;
    pub use super::specified::{Angle, BorderStyle, Time};

    pub struct Context<'a> {
        pub is_root_element: bool,
        pub viewport_size: Size2D<Au>,
        pub inherited_style: &'a ComputedValues,

        /// Values access through this need to be in the properties "computed early":
        /// color, text-decoration, font-size, display, position, float, border-*-style, outline-style
        pub style: ComputedValues,
    }

    impl<'a> Context<'a> {
        pub fn is_root_element(&self) -> bool { self.is_root_element }
        pub fn viewport_size(&self) -> Size2D<Au> { self.viewport_size }
        pub fn inherited_style(&self) -> &ComputedValues { &self.inherited_style }
        pub fn style(&self) -> &ComputedValues { &self.style }
        pub fn mutate_style(&mut self) -> &mut ComputedValues { &mut self.style }
    }

    pub trait ToComputedValue {
        type ComputedValue;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> Self::ComputedValue;
    }

    pub trait ComputedValueAsSpecified {}

    impl<T> ToComputedValue for T where T: ComputedValueAsSpecified + Clone {
        type ComputedValue = T;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> T {
            self.clone()
        }
    }

    impl ToComputedValue for specified::CSSColor {
        type ComputedValue = CSSColor;

        #[inline]
        fn to_computed_value(&self, _context: &Context) -> CSSColor {
            self.parsed
        }
    }

    impl ComputedValueAsSpecified for specified::BorderStyle {}

    impl ToComputedValue for specified::Length {
        type ComputedValue = Au;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> Au {
            match *self {
                specified::Length::Absolute(length) => length,
                specified::Length::Calc(calc) => calc.to_computed_value(context).length(),
                specified::Length::FontRelative(length) =>
                    length.to_computed_value(context.style().get_font().clone_font_size(),
                                             context.style().root_font_size()),
                specified::Length::ViewportPercentage(length) =>
                    length.to_computed_value(context.viewport_size()),
                specified::Length::ServoCharacterWidth(length) =>
                    length.to_computed_value(context.style().get_font().clone_font_size())
            }
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct CalcLengthOrPercentage {
        pub length: Option<Au>,
        pub percentage: Option<CSSFloat>,
    }

    impl CalcLengthOrPercentage {
        #[inline]
        pub fn length(&self) -> Au {
            self.length.unwrap_or(Au(0))
        }

        #[inline]
        pub fn percentage(&self) -> CSSFloat {
            self.percentage.unwrap_or(0.)
        }
    }

    impl From<LengthOrPercentage> for CalcLengthOrPercentage {
        fn from(len: LengthOrPercentage) -> CalcLengthOrPercentage {
            match len {
                LengthOrPercentage::Percentage(this) => {
                    CalcLengthOrPercentage {
                        length: None,
                        percentage: Some(this),
                    }
                }
                LengthOrPercentage::Length(this) => {
                    CalcLengthOrPercentage {
                        length: Some(this),
                        percentage: None,
                    }
                }
                LengthOrPercentage::Calc(this) => {
                    this
                }
            }
        }
    }

    impl From<LengthOrPercentageOrAuto> for Option<CalcLengthOrPercentage> {
        fn from(len: LengthOrPercentageOrAuto) -> Option<CalcLengthOrPercentage> {
            match len {
                LengthOrPercentageOrAuto::Percentage(this) => {
                    Some(CalcLengthOrPercentage {
                        length: None,
                        percentage: Some(this),
                    })
                }
                LengthOrPercentageOrAuto::Length(this) => {
                    Some(CalcLengthOrPercentage {
                        length: Some(this),
                        percentage: None,
                    })
                }
                LengthOrPercentageOrAuto::Calc(this) => {
                    Some(this)
                }
                LengthOrPercentageOrAuto::Auto => {
                    None
                }
            }
        }
    }

    impl ::cssparser::ToCss for CalcLengthOrPercentage {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match (self.length, self.percentage) {
                (None, Some(p)) => write!(dest, "{}%", p * 100.),
                (Some(l), None) => write!(dest, "{}px", Au::to_px(l)),
                (Some(l), Some(p)) => write!(dest, "calc({}px + {}%)", Au::to_px(l), p * 100.),
                _ => unreachable!()
            }
        }
    }

    impl ToComputedValue for specified::CalcLengthOrPercentage {
        type ComputedValue = CalcLengthOrPercentage;

        fn to_computed_value(&self, context: &Context) -> CalcLengthOrPercentage {
            let mut length = None;

            if let Some(absolute) = self.absolute {
                length = Some(length.unwrap_or(Au(0)) + absolute);
            }

            for val in &[self.vw, self.vh, self.vmin, self.vmax] {
                if let Some(val) = *val {
                    length = Some(length.unwrap_or(Au(0)) +
                        val.to_computed_value(context.viewport_size()));
                }
            }
            for val in &[self.ch, self.em, self.ex, self.rem] {
                if let Some(val) = *val {
                    length = Some(length.unwrap_or(Au(0)) + val.to_computed_value(
                        context.style().get_font().clone_font_size(), context.style().root_font_size()));
                }
            }

            CalcLengthOrPercentage { length: length, percentage: self.percentage.map(|p| p.0) }
        }
    }


    #[derive(Debug, PartialEq, Clone, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct BorderRadiusSize(pub Size2D<LengthOrPercentage>);

    impl BorderRadiusSize {
        pub fn zero() -> BorderRadiusSize {
            BorderRadiusSize(Size2D::new(LengthOrPercentage::Length(Au(0)), LengthOrPercentage::Length(Au(0))))
        }
    }

    impl ToComputedValue for specified::BorderRadiusSize {
        type ComputedValue = BorderRadiusSize;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> BorderRadiusSize {
            let w = self.0.width.to_computed_value(context);
            let h = self.0.height.to_computed_value(context);
            BorderRadiusSize(Size2D::new(w, h))
        }
    }

    impl ::cssparser::ToCss for BorderRadiusSize {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.0.width.to_css(dest));
            try!(dest.write_str("/"));
            self.0.height.to_css(dest)
        }
    }

    #[derive(PartialEq, Clone, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrPercentage {
        Length(Au),
        Percentage(CSSFloat),
        Calc(CalcLengthOrPercentage),
    }

    impl LengthOrPercentage {
        #[inline]
        pub fn zero() -> LengthOrPercentage {
            LengthOrPercentage::Length(Au(0))
        }

        /// Returns true if the computed value is absolute 0 or 0%.
        ///
        /// (Returns false for calc() values, even if ones that may resolve to zero.)
        #[inline]
        pub fn is_definitely_zero(&self) -> bool {
            use self::LengthOrPercentage::*;
            match *self {
                Length(Au(0)) | Percentage(0.0) => true,
                Length(_) | Percentage(_) | Calc(_) => false
            }
        }
    }

    impl fmt::Debug for LengthOrPercentage {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                LengthOrPercentage::Length(length) => write!(f, "{:?}", length),
                LengthOrPercentage::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
                LengthOrPercentage::Calc(calc) => write!(f, "{:?}", calc),
            }
        }
    }

    impl ToComputedValue for specified::LengthOrPercentage {
        type ComputedValue = LengthOrPercentage;

        fn to_computed_value(&self, context: &Context) -> LengthOrPercentage {
            match *self {
                specified::LengthOrPercentage::Length(value) => {
                    LengthOrPercentage::Length(value.to_computed_value(context))
                }
                specified::LengthOrPercentage::Percentage(value) => {
                    LengthOrPercentage::Percentage(value.0)
                }
                specified::LengthOrPercentage::Calc(calc) => {
                    LengthOrPercentage::Calc(calc.to_computed_value(context))
                }
            }
        }
    }

    impl ::cssparser::ToCss for LengthOrPercentage {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrPercentage::Length(length) => length.to_css(dest),
                LengthOrPercentage::Percentage(percentage)
                => write!(dest, "{}%", percentage * 100.),
                LengthOrPercentage::Calc(calc) => calc.to_css(dest),
            }
        }
    }

    #[derive(PartialEq, Clone, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrPercentageOrAuto {
        Length(Au),
        Percentage(CSSFloat),
        Auto,
        Calc(CalcLengthOrPercentage),
    }

    impl LengthOrPercentageOrAuto {
        /// Returns true if the computed value is absolute 0 or 0%.
        ///
        /// (Returns false for calc() values, even if ones that may resolve to zero.)
        #[inline]
        pub fn is_definitely_zero(&self) -> bool {
            use self::LengthOrPercentageOrAuto::*;
            match *self {
                Length(Au(0)) | Percentage(0.0) => true,
                Length(_) | Percentage(_) | Calc(_) | Auto => false
            }
        }
    }

    impl fmt::Debug for LengthOrPercentageOrAuto {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                LengthOrPercentageOrAuto::Length(length) => write!(f, "{:?}", length),
                LengthOrPercentageOrAuto::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
                LengthOrPercentageOrAuto::Auto => write!(f, "auto"),
                LengthOrPercentageOrAuto::Calc(calc) => write!(f, "{:?}", calc),
            }
        }
    }

    impl ToComputedValue for specified::LengthOrPercentageOrAuto {
        type ComputedValue = LengthOrPercentageOrAuto;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> LengthOrPercentageOrAuto {
            match *self {
                specified::LengthOrPercentageOrAuto::Length(value) => {
                    LengthOrPercentageOrAuto::Length(value.to_computed_value(context))
                }
                specified::LengthOrPercentageOrAuto::Percentage(value) => {
                    LengthOrPercentageOrAuto::Percentage(value.0)
                }
                specified::LengthOrPercentageOrAuto::Auto => {
                    LengthOrPercentageOrAuto::Auto
                }
                specified::LengthOrPercentageOrAuto::Calc(calc) => {
                    LengthOrPercentageOrAuto::Calc(calc.to_computed_value(context))
                }
            }
        }
    }

    impl ::cssparser::ToCss for LengthOrPercentageOrAuto {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrPercentageOrAuto::Length(length) => length.to_css(dest),
                LengthOrPercentageOrAuto::Percentage(percentage)
                => write!(dest, "{}%", percentage * 100.),
                LengthOrPercentageOrAuto::Auto => dest.write_str("auto"),
                LengthOrPercentageOrAuto::Calc(calc) => calc.to_css(dest),
            }
        }
    }

    #[derive(PartialEq, Clone, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrPercentageOrAutoOrContent {
        Length(Au),
        Percentage(CSSFloat),
        Calc(CalcLengthOrPercentage),
        Auto,
        Content
    }

    impl fmt::Debug for LengthOrPercentageOrAutoOrContent {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                LengthOrPercentageOrAutoOrContent::Length(length) => write!(f, "{:?}", length),
                LengthOrPercentageOrAutoOrContent::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
                LengthOrPercentageOrAutoOrContent::Calc(calc) => write!(f, "{:?}", calc),
                LengthOrPercentageOrAutoOrContent::Auto => write!(f, "auto"),
                LengthOrPercentageOrAutoOrContent::Content => write!(f, "content")
            }
        }
    }

    impl ToComputedValue for specified::LengthOrPercentageOrAutoOrContent {
        type ComputedValue = LengthOrPercentageOrAutoOrContent;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> LengthOrPercentageOrAutoOrContent {
            match *self {
                specified::LengthOrPercentageOrAutoOrContent::Length(value) => {
                    LengthOrPercentageOrAutoOrContent::Length(value.to_computed_value(context))
                },
                specified::LengthOrPercentageOrAutoOrContent::Percentage(value) => {
                    LengthOrPercentageOrAutoOrContent::Percentage(value.0)
                },
                specified::LengthOrPercentageOrAutoOrContent::Calc(calc) => {
                    LengthOrPercentageOrAutoOrContent::Calc(calc.to_computed_value(context))
                },
                specified::LengthOrPercentageOrAutoOrContent::Auto => {
                    LengthOrPercentageOrAutoOrContent::Auto
                },
                specified::LengthOrPercentageOrAutoOrContent::Content => {
                    LengthOrPercentageOrAutoOrContent::Content
                }
            }
        }
    }

    impl ::cssparser::ToCss for LengthOrPercentageOrAutoOrContent {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrPercentageOrAutoOrContent::Length(length) => length.to_css(dest),
                LengthOrPercentageOrAutoOrContent::Percentage(percentage)
                => write!(dest, "{}%", percentage * 100.),
                LengthOrPercentageOrAutoOrContent::Calc(calc) => calc.to_css(dest),
                LengthOrPercentageOrAutoOrContent::Auto => dest.write_str("auto"),
                LengthOrPercentageOrAutoOrContent::Content => dest.write_str("content")
            }
        }
    }

    #[derive(PartialEq, Clone, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrPercentageOrNone {
        Length(Au),
        Percentage(CSSFloat),
        Calc(CalcLengthOrPercentage),
        None,
    }

    impl fmt::Debug for LengthOrPercentageOrNone {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                LengthOrPercentageOrNone::Length(length) => write!(f, "{:?}", length),
                LengthOrPercentageOrNone::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
                LengthOrPercentageOrNone::Calc(calc) => write!(f, "{:?}", calc),
                LengthOrPercentageOrNone::None => write!(f, "none"),
            }
        }
    }

    impl ToComputedValue for specified::LengthOrPercentageOrNone {
        type ComputedValue = LengthOrPercentageOrNone;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> LengthOrPercentageOrNone {
            match *self {
                specified::LengthOrPercentageOrNone::Length(value) => {
                    LengthOrPercentageOrNone::Length(value.to_computed_value(context))
                }
                specified::LengthOrPercentageOrNone::Percentage(value) => {
                    LengthOrPercentageOrNone::Percentage(value.0)
                }
                specified::LengthOrPercentageOrNone::Calc(calc) => {
                    LengthOrPercentageOrNone::Calc(calc.to_computed_value(context))
                }
                specified::LengthOrPercentageOrNone::None => {
                    LengthOrPercentageOrNone::None
                }
            }
        }
    }

    impl ::cssparser::ToCss for LengthOrPercentageOrNone {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrPercentageOrNone::Length(length) => length.to_css(dest),
                LengthOrPercentageOrNone::Percentage(percentage) =>
                    write!(dest, "{}%", percentage * 100.),
                LengthOrPercentageOrNone::Calc(calc) => calc.to_css(dest),
                LengthOrPercentageOrNone::None => dest.write_str("none"),
            }
        }
    }

    #[derive(PartialEq, Clone, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum LengthOrNone {
        Length(Au),
        None,
    }

    impl fmt::Debug for LengthOrNone {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                LengthOrNone::Length(length) => write!(f, "{:?}", length),
                LengthOrNone::None => write!(f, "none"),
            }
        }
    }

    impl ToComputedValue for specified::LengthOrNone {
        type ComputedValue = LengthOrNone;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> LengthOrNone {
            match *self {
                specified::LengthOrNone::Length(specified::Length::Calc(calc)) => {
                    LengthOrNone::Length(calc.to_computed_value(context).length())
                }
                specified::LengthOrNone::Length(value) => {
                    LengthOrNone::Length(value.to_computed_value(context))
                }
                specified::LengthOrNone::None => {
                    LengthOrNone::None
                }
            }
        }
    }

    impl ::cssparser::ToCss for LengthOrNone {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match *self {
                LengthOrNone::Length(length) => length.to_css(dest),
                LengthOrNone::None => dest.write_str("none"),
            }
        }
    }

    impl ToComputedValue for specified::Image {
        type ComputedValue = Image;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> Image {
            match *self {
                specified::Image::Url(ref url) => Image::Url(url.clone()),
                specified::Image::LinearGradient(ref linear_gradient) => {
                    Image::LinearGradient(linear_gradient.to_computed_value(context))
                }
            }
        }
    }


    /// Computed values for an image according to CSS-IMAGES.
    #[derive(Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub enum Image {
        Url(Url),
        LinearGradient(LinearGradient),
    }

    impl fmt::Debug for Image {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match *self {
                Image::Url(ref url) => write!(f, "url(\"{}\")", url),
                Image::LinearGradient(ref grad) => write!(f, "linear-gradient({:?})", grad),
            }
        }
    }

    /// Computed values for a CSS linear gradient.
    #[derive(Clone, PartialEq)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct LinearGradient {
        /// The angle or corner of the gradient.
        pub angle_or_corner: AngleOrCorner,

        /// The color stops.
        pub stops: Vec<ColorStop>,
    }

    impl ::cssparser::ToCss for LinearGradient {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(dest.write_str("linear-gradient("));
            try!(self.angle_or_corner.to_css(dest));
            for stop in &self.stops {
                try!(dest.write_str(", "));
                try!(stop.to_css(dest));
            }
            try!(dest.write_str(")"));
            Ok(())
        }
    }

    impl fmt::Debug for LinearGradient {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let _ = write!(f, "{:?}", self.angle_or_corner);
            for stop in &self.stops {
                let _ = write!(f, ", {:?}", stop);
            }
            Ok(())
        }
    }

    /// Computed values for one color stop in a linear gradient.
    #[derive(Clone, PartialEq, Copy)]
    #[cfg_attr(feature = "servo", derive(HeapSizeOf))]
    pub struct ColorStop {
        /// The color of this stop.
        pub color: CSSColor,

        /// The position of this stop. If not specified, this stop is placed halfway between the
        /// point that precedes it and the point that follows it per CSS-IMAGES § 3.4.
        pub position: Option<LengthOrPercentage>,
    }

    impl ::cssparser::ToCss for ColorStop {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            try!(self.color.to_css(dest));
            if let Some(position) = self.position {
                try!(dest.write_str(" "));
                try!(position.to_css(dest));
            }
            Ok(())
        }
    }

    impl fmt::Debug for ColorStop {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            let _ = write!(f, "{:?}", self.color);
            self.position.map(|pos| {
                let _ = write!(f, " {:?}", pos);
            });
            Ok(())
        }
    }

    impl ToComputedValue for specified::LinearGradient {
        type ComputedValue = LinearGradient;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> LinearGradient {
            let specified::LinearGradient {
                angle_or_corner,
                ref stops
            } = *self;
            LinearGradient {
                angle_or_corner: angle_or_corner,
                stops: stops.iter().map(|stop| {
                    ColorStop {
                        color: stop.color.parsed,
                        position: match stop.position {
                            None => None,
                            Some(value) => Some(value.to_computed_value(context)),
                        },
                    }
                }).collect()
            }
        }
    }
    pub type Length = Au;
    pub type Number = CSSFloat;
    pub type Opacity = CSSFloat;
}
