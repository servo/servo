/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#![allow(non_camel_case_types)]

pub use cssparser::RGBA;

macro_rules! define_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident ),+,) => {
        define_css_keyword_enum!($name: $( $css => $variant ),+);
    };
    ($name: ident: $( $css: expr => $variant: ident ),+) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, Copy, Hash, RustcEncodable, Debug, HeapSizeOf)]
        #[derive(Deserialize, Serialize)]
        pub enum $name {
            $( $variant ),+
        }

        impl $name {
            pub fn parse(input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    $( $css => Ok($name::$variant) ),+
                    _ => Err(())
                }
            }
        }

        impl ::cssparser::ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where W: ::std::fmt::Write {
                match self {
                    $( &$name::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}

macro_rules! define_numbered_css_keyword_enum {
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+,) => {
        define_numbered_css_keyword_enum!($name: $( $css => $variant = $value ),+);
    };
    ($name: ident: $( $css: expr => $variant: ident = $value: expr ),+) => {
        #[allow(non_camel_case_types)]
        #[derive(Clone, Eq, PartialEq, PartialOrd, Ord, Copy, RustcEncodable, Debug, HeapSizeOf)]
        #[derive(Deserialize, Serialize)]
        pub enum $name {
            $( $variant = $value ),+
        }

        impl $name {
            pub fn parse(input: &mut ::cssparser::Parser) -> Result<$name, ()> {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    $( $css => Ok($name::$variant) ),+
                    _ => Err(())
                }
            }
        }

        impl ::cssparser::ToCss for $name {
            fn to_css<W>(&self, dest: &mut W) -> ::std::fmt::Result
            where W: ::std::fmt::Write {
                match self {
                    $( &$name::$variant => dest.write_str($css) ),+
                }
            }
        }
    }
}

pub type CSSFloat = f32;


pub mod specified {
    use std::ascii::AsciiExt;
    use std::cmp;
    use std::f32::consts::PI;
    use std::fmt;
    use std::fmt::Write;
    use std::ops::Mul;
    use url::Url;
    use cssparser::{self, Token, Parser, ToCss, CssStringWriter};
    use euclid::size::Size2D;
    use parser::ParserContext;
    use util::geometry::Au;
    use super::CSSFloat;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum AllowedNumericType {
        All,
        NonNegative
    }

    impl AllowedNumericType {
        #[inline]
        pub fn is_ok(&self, value: f32) -> bool {
            match self {
                &AllowedNumericType::All => true,
                &AllowedNumericType::NonNegative => value >= 0.,
            }
        }
    }

    #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
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

    impl ToCss for CSSColor {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.authored {
                Some(ref s) => dest.write_str(s),
                None => self.parsed.to_css(dest),
            }
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct CSSRGBA {
        pub parsed: cssparser::RGBA,
        pub authored: Option<String>,
    }

    impl ToCss for CSSRGBA {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self.authored {
                Some(ref s) => dest.write_str(s),
                None => self.parsed.to_css(dest),
            }
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub enum FontRelativeLength {
        Em(CSSFloat),
        Ex(CSSFloat),
        Rem(CSSFloat)
    }

    impl ToCss for FontRelativeLength {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &FontRelativeLength::Em(length) => write!(dest, "{}em", length),
                &FontRelativeLength::Ex(length) => write!(dest, "{}ex", length),
                &FontRelativeLength::Rem(length) => write!(dest, "{}rem", length)
            }
        }
    }

    impl FontRelativeLength {
        pub fn to_computed_value(&self,
                                 reference_font_size: Au,
                                 root_font_size: Au)
                                 -> Au
        {
            match self {
                &FontRelativeLength::Em(length) => reference_font_size.scale_by(length),
                &FontRelativeLength::Ex(length) => {
                    let x_height = 0.5;  // TODO: find that from the font
                    reference_font_size.scale_by(length * x_height)
                },
                &FontRelativeLength::Rem(length) => root_font_size.scale_by(length)
            }
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub enum ViewportPercentageLength {
        Vw(CSSFloat),
        Vh(CSSFloat),
        Vmin(CSSFloat),
        Vmax(CSSFloat)
    }

    impl ToCss for ViewportPercentageLength {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &ViewportPercentageLength::Vw(length) => write!(dest, "{}vw", length),
                &ViewportPercentageLength::Vh(length) => write!(dest, "{}vh", length),
                &ViewportPercentageLength::Vmin(length) => write!(dest, "{}vmin", length),
                &ViewportPercentageLength::Vmax(length) => write!(dest, "{}vmax", length)
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

            let value = match self {
                &ViewportPercentageLength::Vw(length) =>
                    length * to_unit!(viewport_size.width),
                &ViewportPercentageLength::Vh(length) =>
                    length * to_unit!(viewport_size.height),
                &ViewportPercentageLength::Vmin(length) =>
                    length * to_unit!(cmp::min(viewport_size.width, viewport_size.height)),
                &ViewportPercentageLength::Vmax(length) =>
                    length * to_unit!(cmp::max(viewport_size.width, viewport_size.height)),
            };
            Au::from_f32_px(value)
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
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

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub enum Length {
        Absolute(Au),  // application units
        FontRelative(FontRelativeLength),
        ViewportPercentage(ViewportPercentageLength),

        /// HTML5 "character width", as defined in HTML5 § 14.5.4.
        ///
        /// This cannot be specified by the user directly and is only generated by
        /// `Stylist::synthesize_rules_for_legacy_attributes()`.
        ServoCharacterWidth(CharacterWidth),
    }

    impl ToCss for Length {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &Length::Absolute(length) => write!(dest, "{}px", length.to_f32_px()),
                &Length::FontRelative(length) => length.to_css(dest),
                &Length::ViewportPercentage(length) => length.to_css(dest),
                &Length::ServoCharacterWidth(_)
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
    const AU_PER_PT: CSSFloat = AU_PER_IN / 72.;
    const AU_PER_PC: CSSFloat = AU_PER_PT * 12.;
    impl Length {
        #[inline]
        fn parse_internal(input: &mut Parser, context: &AllowedNumericType) -> Result<Length, ()> {
            match try!(input.next()) {
                Token::Dimension(ref value, ref unit) if context.is_ok(value.value) =>
                    Length::parse_dimension(value.value, unit),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(Length::Absolute(Au(0))),
                _ => Err(())
            }
        }
        #[allow(dead_code)]
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
                "pt" => Ok(Length::Absolute(Au((value * AU_PER_PT) as i32))),
                "pc" => Ok(Length::Absolute(Au((value * AU_PER_PC) as i32))),
                // font-relative
                "em" => Ok(Length::FontRelative(FontRelativeLength::Em(value))),
                "ex" => Ok(Length::FontRelative(FontRelativeLength::Ex(value))),
                "rem" => Ok(Length::FontRelative(FontRelativeLength::Rem(value))),
                // viewport percentages
                "vw" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vw(value))),
                "vh" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vh(value))),
                "vmin" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vmin(value))),
                "vmax" => Ok(Length::ViewportPercentage(ViewportPercentageLength::Vmax(value)))
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
        Percentage(CSSFloat),
        Number(CSSFloat),
        Sum(Box<CalcSumNode>),
    }

    #[derive(Clone, Debug)]
    enum CalcAstNode {
        Add(CalcSumNode),
        Multiply(CalcProductNode),
        Value(CalcValueNode),
    }

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub struct Calc {
        pub absolute: Option<Au>,
        pub font_relative: Option<FontRelativeLength>,
        pub viewport_percentage: Option<ViewportPercentageLength>,
        pub percentage: Option<CSSFloat>,
    }
    impl Calc {
        fn parse_sum(input: &mut Parser) -> Result<CalcSumNode, ()> {
            let mut products = Vec::new();
            products.push(try!(Calc::parse_product(input)));

            loop {
                let next = input.next();
                match next {
                    Ok(Token::Delim('+')) => {
                        products.push(try!(Calc::parse_product(input)));
                    }
                    Ok(Token::Delim('-')) => {
                        let mut right = try!(Calc::parse_product(input));
                        right.values.push(CalcValueNode::Number(-1.));
                        products.push(right);
                    }
                    Ok(_) => return Err(()),
                    _ => break
                }
            }

            let sum = CalcSumNode { products: products };
            println!("Parsed sum {:?} ", sum);
            Ok(sum)
        }

        fn parse_product(input: &mut Parser) -> Result<CalcProductNode, ()> {
            let mut values = Vec::new();
            values.push(try!(Calc::parse_value(input)));

            loop {
                let position = input.position();
                let next = input.next();
                match next {
                    Ok(Token::Delim('*')) => {
                        values.push(try!(Calc::parse_value(input)));
                    }
                    Ok(Token::Delim('/')) => {
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

            let sum = CalcProductNode { values: values };
            println!("Parsed product {:?} ", sum);
            Ok(sum)
        }

        fn parse_value(input: &mut Parser) -> Result<CalcValueNode, ()> {
            let next = input.next();
            match next {
                Ok(Token::Number(ref value)) => Ok(CalcValueNode::Number(value.value)),
                Ok(Token::Dimension(ref value, ref unit)) =>
                    Length::parse_dimension(value.value, unit).map(CalcValueNode::Length),
                Ok(Token::Percentage(ref value)) =>
                    Ok(CalcValueNode::Percentage(value.unit_value)),
                Ok(Token::ParenthesisBlock) => {
                    let result = try!(input.parse_nested_block(Calc::parse_sum));
                    Ok(CalcValueNode::Sum(box result))
                },
                _ => Err(())
            }
        }

        fn simplify_ast(node: CalcSumNode) -> Result<CalcSumNode, ()> {
            let mut simplified = Vec::new();
            for node in node.products {
                let node = try!(Calc::simplify_product(node));
                match node {
                    CalcAstNode::Value(value) => {
                        let product = CalcProductNode { values: vec!(value) };
                        simplified.push(product);
                    }
                    _ => return Err(())
                }
            }

            Ok(CalcSumNode {products: simplified} )
        }

        fn simplify_product(node: CalcProductNode) -> Result<CalcAstNode, ()> {
            let mut multiplier = 1.;
            let mut node_with_unit: Option<CalcAstNode> = None;
            let mut node_hack: CalcAstNode;
            for node in node.values {
                node_hack = try!(Calc::simplify_value(node));
                if let CalcAstNode::Value(CalcValueNode::Number(n)) = node_hack {
                    multiplier *= n;
                } else if node_with_unit.is_none() {
                    node_with_unit = Some(node_hack);
                } else {
                    return Err(());
                }
            }

            match node_with_unit {
                None => Ok(CalcAstNode::Value(CalcValueNode::Number(multiplier))),
                Some(CalcAstNode::Add(sum)) =>
                    Ok(CalcAstNode::Add(CalcSumNode {
                        products: sum.products
                                     .iter()
                                     .map(|p| Calc::multiply_product(p, multiplier))
                                     .collect()
                    })),
                Some(CalcAstNode::Value(ref value)) =>
                    Ok(CalcAstNode::Value(Calc::multiply_value(value, multiplier))),
                _ => unreachable!()
            }
        }

        fn multiply_product(node: &CalcProductNode, multiplier: CSSFloat) -> CalcProductNode {
            CalcProductNode {
                values: node.values
                            .iter()
                            .map(|v| Calc::multiply_value(v, multiplier))
                            .collect()
            }
        }

        fn multiply_value(node: &CalcValueNode, multiplier: CSSFloat) -> CalcValueNode {
            match node {
                &CalcValueNode::Number(_) => unreachable!(),
                &CalcValueNode::Percentage(p) => CalcValueNode::Percentage(p * multiplier),
                &CalcValueNode::Sum(_) => unreachable!(),
                &CalcValueNode::Length(l) => CalcValueNode::Length(Calc::multiply_length(l, multiplier))
            }
        }

        fn multiply_length(length: Length, multiplier: CSSFloat) -> Length {
            match length {
                Length::Absolute(Au(au)) =>
                    Length::Absolute(Au((au as CSSFloat * multiplier) as i32)),
                _ => panic!()
            }
        }

        fn simplify_sum(node: CalcSumNode) -> Result<CalcAstNode, ()> {
            let mut simplified = Vec::new();
            let length = node.products.len();
            for node in node.products {
                let node = try!(Calc::simplify_product(node));
                match node {
                    CalcAstNode::Value(value) => {
                        let product = CalcProductNode { values: vec!(value) };
                        if length == 1 {
                            return Ok(CalcAstNode::Multiply(product));
                        }
                        simplified.push(product);
                    }
                    CalcAstNode::Add(sum) => simplified.push_all(&sum.products),
                    _ => return Err(())
                }
            }

            Ok(CalcAstNode::Add(CalcSumNode {products: simplified} ))
        }

        fn simplify_value(node: CalcValueNode) -> Result<CalcAstNode, ()> {
            match node {
                CalcValueNode::Sum(box sum) => Calc::simplify_sum(sum),
                node => Ok(CalcAstNode::Value(node))
            }
        }

        pub fn parse(input: &mut Parser) -> Result<Calc, ()> {

            let ast = try!(Calc::parse_sum(input));
            let ast = try!(Calc::simplify_sum(ast));
            let mut calc = Calc {
                absolute: None,
                font_relative: None,
                viewport_percentage: None,
                percentage: None,
            };

            if let CalcAstNode::Add(ast) = ast {
                for value in ast.products {
                    assert!(value.values.len() == 1);
                    match value.values[0] {
                        CalcValueNode::Percentage(p) =>
                            calc.percentage = Some(calc.percentage.unwrap_or(0.) + p),
                        CalcValueNode::Length(Length::Absolute(Au(au))) =>
                            calc.absolute = Some(calc.absolute.unwrap_or(Au(0)) + Au(au)),
                        //CalcValueNode::Length(Length::FontRelative(Au(au)))
                            //calc.absolute = Some(calc.absolute.unwrap_or(0.) + au),
                        _ => return Err(())
                    }
                }
            } else {
                unreachable!()
            }

            Ok(calc)
        }
    }

    impl ToCss for Calc {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            // XXX WRONG HACK
            try!(write!(dest, "calc("));
            if let Some(absolute) = self.absolute {
                try!(write!(dest, "{}px", Au::to_px(absolute)));
            }
            if let Some(FontRelativeLength::Em(font_relative)) = self.font_relative {
                try!(write!(dest, "{}em", font_relative));
            }

            write!(dest, ")")
         }
    }

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub enum LengthOrPercentage {
        Length(Length),
        Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        Calc(Calc),
    }

    impl ToCss for LengthOrPercentage {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &LengthOrPercentage::Length(length) => length.to_css(dest),
                &LengthOrPercentage::Percentage(percentage)
                => write!(dest, "{}%", percentage * 100.),
                &LengthOrPercentage::Calc(calc) => calc.to_css(dest),
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
                    Ok(LengthOrPercentage::Percentage(value.unit_value)),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(LengthOrPercentage::Length(Length::Absolute(Au(0)))),
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                    let calc = try!(input.parse_nested_block(Calc::parse));
                    Ok(LengthOrPercentage::Calc(calc))
                },
                _ => Err(())
            }
        }
        #[allow(dead_code)]
        #[inline]
        pub fn parse(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
            LengthOrPercentage::parse_internal(input, &AllowedNumericType::All)
        }
        #[inline]
        pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentage, ()> {
            LengthOrPercentage::parse_internal(input, &AllowedNumericType::NonNegative)
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub enum LengthOrPercentageOrAuto {
        Length(Length),
        Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        Auto,
        Calc(Calc),
    }

    impl ToCss for LengthOrPercentageOrAuto {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &LengthOrPercentageOrAuto::Length(length) => length.to_css(dest),
                &LengthOrPercentageOrAuto::Percentage(percentage)
                => write!(dest, "{}%", percentage * 100.),
                &LengthOrPercentageOrAuto::Auto => dest.write_str("auto"),
                &LengthOrPercentageOrAuto::Calc(calc) => calc.to_css(dest),
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
                    Ok(LengthOrPercentageOrAuto::Percentage(value.unit_value)),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(LengthOrPercentageOrAuto::Length(Length::Absolute(Au(0)))),
                Token::Ident(ref value) if value.eq_ignore_ascii_case("auto") =>
                    Ok(LengthOrPercentageOrAuto::Auto),
                Token::Function(ref name) if name.eq_ignore_ascii_case("calc") => {
                    let calc = try!(input.parse_nested_block(Calc::parse));
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

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub enum LengthOrPercentageOrNone {
        Length(Length),
        Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        None,
    }

    impl ToCss for LengthOrPercentageOrNone {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &LengthOrPercentageOrNone::Length(length) => length.to_css(dest),
                &LengthOrPercentageOrNone::Percentage(percentage) =>
                    write!(dest, "{}%", percentage * 100.),
                &LengthOrPercentageOrNone::None => dest.write_str("none"),
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
                    Ok(LengthOrPercentageOrNone::Percentage(value.unit_value)),
                Token::Number(ref value) if value.value == 0. =>
                    Ok(LengthOrPercentageOrNone::Length(Length::Absolute(Au(0)))),
                Token::Ident(ref value) if value.eq_ignore_ascii_case("none") =>
                    Ok(LengthOrPercentageOrNone::None),
                _ => Err(())
            }
        }
        #[allow(dead_code)]
        #[inline]
        pub fn parse(input: &mut Parser) -> Result<LengthOrPercentageOrNone, ()> {
            LengthOrPercentageOrNone::parse_internal(input, &AllowedNumericType::All)
        }
        #[inline]
        pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrPercentageOrNone, ()> {
            LengthOrPercentageOrNone::parse_internal(input, &AllowedNumericType::NonNegative)
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub enum LengthOrNone {
        Length(Length),
        None,
    }

    impl ToCss for LengthOrNone {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &LengthOrNone::Length(length) => length.to_css(dest),
                &LengthOrNone::None => dest.write_str("none"),
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
                Token::Ident(ref value) if value.eq_ignore_ascii_case("none") =>
                    Ok(LengthOrNone::None),
                _ => Err(())
            }
        }
        #[allow(dead_code)]
        #[inline]
        pub fn parse(input: &mut Parser) -> Result<LengthOrNone, ()> {
            LengthOrNone::parse_internal(input, &AllowedNumericType::All)
        }
        #[inline]
        pub fn parse_non_negative(input: &mut Parser) -> Result<LengthOrNone, ()> {
            LengthOrNone::parse_internal(input, &AllowedNumericType::NonNegative)
        }
    }

    // http://dev.w3.org/csswg/css2/colors.html#propdef-background-position
    #[derive(Clone, PartialEq, Copy)]
    pub enum PositionComponent {
        Length(Length),
        Percentage(CSSFloat),  // [0 .. 100%] maps to [0.0 .. 1.0]
        Center,
        Left,
        Right,
        Top,
        Bottom,
    }
    impl PositionComponent {
        pub fn parse(input: &mut Parser) -> Result<PositionComponent, ()> {
            match try!(input.next()) {
                Token::Dimension(ref value, ref unit) => {
                    Length::parse_dimension(value.value, unit)
                    .map(PositionComponent::Length)
                }
                Token::Percentage(ref value) => {
                    Ok(PositionComponent::Percentage(value.unit_value))
                }
                Token::Number(ref value) if value.value == 0. => {
                    Ok(PositionComponent::Length(Length::Absolute(Au(0))))
                }
                Token::Ident(value) => {
                    match_ignore_ascii_case! { value,
                        "center" => Ok(PositionComponent::Center),
                        "left" => Ok(PositionComponent::Left),
                        "right" => Ok(PositionComponent::Right),
                        "top" => Ok(PositionComponent::Top),
                        "bottom" => Ok(PositionComponent::Bottom)
                        _ => Err(())
                    }
                }
                _ => Err(())
            }
        }
        #[inline]
        pub fn to_length_or_percentage(self) -> LengthOrPercentage {
            match self {
                PositionComponent::Length(x) => LengthOrPercentage::Length(x),
                PositionComponent::Percentage(x) => LengthOrPercentage::Percentage(x),
                PositionComponent::Center => LengthOrPercentage::Percentage(0.5),
                PositionComponent::Left |
                PositionComponent::Top => LengthOrPercentage::Percentage(0.0),
                PositionComponent::Right |
                PositionComponent::Bottom => LengthOrPercentage::Percentage(1.0),
            }
        }
    }

    #[derive(Clone, PartialEq, PartialOrd, Copy, Debug, HeapSizeOf, Deserialize, Serialize)]
    pub struct Angle(pub CSSFloat);

    impl ToCss for Angle {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            let Angle(value) = *self;
            write!(dest, "{}rad", value)
        }
    }

    impl Angle {
        pub fn radians(self) -> f32 {
            let Angle(radians) = self;
            radians
        }
    }

    const RAD_PER_DEG: CSSFloat = PI / 180.0;
    const RAD_PER_GRAD: CSSFloat = PI / 200.0;
    const RAD_PER_TURN: CSSFloat = PI * 2.0;

    impl Angle {
        /// Parses an angle according to CSS-VALUES § 6.1.
        pub fn parse(input: &mut Parser) -> Result<Angle, ()> {
            match try!(input.next()) {
                Token::Dimension(value, unit) => {
                    match_ignore_ascii_case! { unit,
                        "deg" => Ok(Angle(value.value * RAD_PER_DEG)),
                        "grad" => Ok(Angle(value.value * RAD_PER_GRAD)),
                        "turn" => Ok(Angle(value.value * RAD_PER_TURN)),
                        "rad" => Ok(Angle(value.value))
                        _ => Err(())
                    }
                }
                Token::Number(ref value) if value.value == 0. => Ok(Angle(0.)),
                _ => Err(())
            }
        }
    }

    /// Specified values for an image according to CSS-IMAGES.
    #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
    pub enum Image {
        Url(Url),
        LinearGradient(LinearGradient),
    }

    impl ToCss for Image {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &Image::Url(ref url) => {
                    try!(dest.write_str("url(\""));
                    try!(write!(&mut CssStringWriter::new(dest), "{}", url));
                    try!(dest.write_str("\")"));
                    Ok(())
                }
                &Image::LinearGradient(ref gradient) => gradient.to_css(dest)
            }
        }
    }

    impl Image {
        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Image, ()> {
            match try!(input.next()) {
                Token::Url(url) => {
                    Ok(Image::Url(context.parse_url(&url)))
                }
                Token::Function(name) => {
                    match_ignore_ascii_case! { name,
                        "linear-gradient" => {
                            Ok(Image::LinearGradient(try!(
                                input.parse_nested_block(LinearGradient::parse_function))))
                        }
                        _ => Err(())
                    }
                }
                _ => Err(())
            }
        }
    }

    /// Specified values for a CSS linear gradient.
    #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
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
            for stop in self.stops.iter() {
                try!(dest.write_str(", "));
                try!(stop.to_css(dest));
            }
            try!(dest.write_str(")"));
            Ok(())
        }
    }

    /// Specified values for an angle or a corner in a linear gradient.
    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub enum AngleOrCorner {
        Angle(Angle),
        Corner(HorizontalDirection, VerticalDirection),
    }

    impl ToCss for AngleOrCorner {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &AngleOrCorner::Angle(angle) => angle.to_css(dest),
                &AngleOrCorner::Corner(horizontal, vertical) => {
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
    #[derive(Clone, PartialEq, Debug, HeapSizeOf)]
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


    pub fn parse_border_width(input: &mut Parser) -> Result<Length, ()> {
        input.try(Length::parse_non_negative).or_else(|()| {
            match_ignore_ascii_case! { try!(input.expect_ident()),
                "thin" => Ok(Length::from_px(1.)),
                "medium" => Ok(Length::from_px(3.)),
                "thick" => Ok(Length::from_px(5.))
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

    /// A time in seconds according to CSS-VALUES § 6.2.
    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, HeapSizeOf)]
    pub struct Time(pub CSSFloat);

    impl Time {
        /// Returns the time in fractional seconds.
        pub fn seconds(self) -> f32 {
            let Time(seconds) = self;
            seconds
        }

        /// Parses a time according to CSS-VALUES § 6.2.
        fn parse_dimension(value: CSSFloat, unit: &str) -> Result<Time,()> {
            if unit.eq_ignore_ascii_case("s") {
                Ok(Time(value))
            } else if unit.eq_ignore_ascii_case("ms") {
                Ok(Time(value / 1000.0))
            } else {
                Err(())
            }
        }

        pub fn parse(input: &mut Parser) -> Result<Time,()> {
            match input.next() {
                Ok(Token::Dimension(ref value, ref unit)) => {
                    Time::parse_dimension(value.value, &unit)
                }
                _ => Err(()),
            }
        }
    }

    impl super::computed::ToComputedValue for Time {
        type ComputedValue = Time;

        #[inline]
        fn to_computed_value(&self, _: &super::computed::Context) -> Time {
            *self
        }
    }

    impl ToCss for Time {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            write!(dest, "{}ms", self.0)
        }
    }
}

pub mod computed {
    pub use super::specified::{Angle, BorderStyle, Time};
    use super::specified::{AngleOrCorner};
    use super::{specified, CSSFloat};
    pub use cssparser::Color as CSSColor;
    use euclid::size::Size2D;
    use properties::longhands;
    use std::fmt;
    use url::Url;
    use util::geometry::Au;

    pub struct Context {
        pub inherited_font_weight: longhands::font_weight::computed_value::T,
        pub inherited_font_size: longhands::font_size::computed_value::T,
        pub inherited_text_decorations_in_effect:
            longhands::_servo_text_decorations_in_effect::computed_value::T,
        pub color: longhands::color::computed_value::T,
        pub text_decoration: longhands::text_decoration::computed_value::T,
        pub font_size: longhands::font_size::computed_value::T,
        pub root_font_size: longhands::font_size::computed_value::T,
        pub display: longhands::display::computed_value::T,
        pub overflow_x: longhands::overflow_x::computed_value::T,
        pub overflow_y: longhands::overflow_y::computed_value::T,
        pub positioned: bool,
        pub floated: bool,
        pub border_top_present: bool,
        pub border_right_present: bool,
        pub border_bottom_present: bool,
        pub border_left_present: bool,
        pub is_root_element: bool,
        pub viewport_size: Size2D<Au>,
        pub outline_style_present: bool,
        // TODO, as needed: viewport size, etc.
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
            match self {
                &specified::Length::Absolute(length) => length,
                &specified::Length::FontRelative(length) =>
                    length.to_computed_value(context.font_size, context.root_font_size),
                &specified::Length::ViewportPercentage(length) =>
                    length.to_computed_value(context.viewport_size),
                &specified::Length::ServoCharacterWidth(length) =>
                    length.to_computed_value(context.font_size)
            }
        }
    }

    #[derive(Clone, PartialEq, Copy, Debug, HeapSizeOf)]
    pub struct Calc {
        length: Option<Au>,
        percentage: Option<CSSFloat>,
    }

    impl Calc {
        pub fn length(&self) -> Au {
            self.length.unwrap_or(Au(0))
        }

        pub fn percentage(&self) -> CSSFloat {
            self.percentage.unwrap_or(0.)
        }
    }

    impl ::cssparser::ToCss for Calc {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match (self.length, self.percentage) {
                (None, Some(p)) => write!(dest, "{}%", p),
                (Some(l), None) => write!(dest, "{}px", Au::to_px(l)),
                (Some(l), Some(p)) => write!(dest, "calc({}px + {}%)", Au::to_px(l), p * 100.),
                _ => unreachable!()
            }
        }
    }

    impl ToComputedValue for specified::Calc {
        type ComputedValue = Calc;

        fn to_computed_value(&self, context: &Context) -> Calc {
            let length = self.absolute;
            let percentage = self.percentage;

            Calc { length: length, percentage: percentage }
        }
    }


    #[derive(PartialEq, Clone, Copy, HeapSizeOf)]
    pub enum LengthOrPercentage {
        Length(Au),
        Percentage(CSSFloat),
        Calc(Calc),
    }

    impl LengthOrPercentage {
        pub fn zero() -> LengthOrPercentage {
            LengthOrPercentage::Length(Au(0))
        }
    }

    impl fmt::Debug for LengthOrPercentage {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                &LengthOrPercentage::Length(length) => write!(f, "{:?}", length),
                &LengthOrPercentage::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
                // XXX HACK WRONG
                &LengthOrPercentage::Calc(calc) => write!(f, "{}%", 100.),
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
                    LengthOrPercentage::Percentage(value)
                }
                specified::LengthOrPercentage::Calc(calc) => {
                    LengthOrPercentage::Calc(calc.to_computed_value(context))
                }
            }
        }
    }

    impl ::cssparser::ToCss for LengthOrPercentage {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &LengthOrPercentage::Length(length) => length.to_css(dest),
                &LengthOrPercentage::Percentage(percentage)
                => write!(dest, "{}%", percentage * 100.),
                &LengthOrPercentage::Calc(calc) => calc.to_css(dest),
            }
        }
    }

    #[derive(PartialEq, Clone, Copy, HeapSizeOf)]
    pub enum LengthOrPercentageOrAuto {
        Length(Au),
        Percentage(CSSFloat),
        Auto,
        Calc(Calc),
    }
    impl fmt::Debug for LengthOrPercentageOrAuto {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                &LengthOrPercentageOrAuto::Length(length) => write!(f, "{:?}", length),
                &LengthOrPercentageOrAuto::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
                &LengthOrPercentageOrAuto::Auto => write!(f, "auto"),
                // XXX HACK WRONG
                &LengthOrPercentageOrAuto::Calc(calc) => write!(f, "{}%", 100.),
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
                    LengthOrPercentageOrAuto::Percentage(value)
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
            match self {
                &LengthOrPercentageOrAuto::Length(length) => length.to_css(dest),
                &LengthOrPercentageOrAuto::Percentage(percentage)
                => write!(dest, "{}%", percentage * 100.),
                &LengthOrPercentageOrAuto::Auto => dest.write_str("auto"),
                &LengthOrPercentageOrAuto::Calc(calc) => calc.to_css(dest),
            }
        }
    }

    #[derive(PartialEq, Clone, Copy, HeapSizeOf)]
    pub enum LengthOrPercentageOrNone {
        Length(Au),
        Percentage(CSSFloat),
        None,
    }
    impl fmt::Debug for LengthOrPercentageOrNone {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                &LengthOrPercentageOrNone::Length(length) => write!(f, "{:?}", length),
                &LengthOrPercentageOrNone::Percentage(percentage) => write!(f, "{}%", percentage * 100.),
                &LengthOrPercentageOrNone::None => write!(f, "none"),
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
                    LengthOrPercentageOrNone::Percentage(value)
                }
                specified::LengthOrPercentageOrNone::None => {
                    LengthOrPercentageOrNone::None
                }
            }
        }
    }

    impl ::cssparser::ToCss for LengthOrPercentageOrNone {
        fn to_css<W>(&self, dest: &mut W) -> fmt::Result where W: fmt::Write {
            match self {
                &LengthOrPercentageOrNone::Length(length) => length.to_css(dest),
                &LengthOrPercentageOrNone::Percentage(percentage) =>
                    write!(dest, "{}%", percentage * 100.),
                &LengthOrPercentageOrNone::None => dest.write_str("none"),
            }
        }
    }

    #[derive(PartialEq, Clone, Copy, HeapSizeOf)]
    pub enum LengthOrNone {
        Length(Au),
        None,
    }
    impl fmt::Debug for LengthOrNone {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                &LengthOrNone::Length(length) => write!(f, "{:?}", length),
                &LengthOrNone::None => write!(f, "none"),
            }
        }
    }

    impl ToComputedValue for specified::LengthOrNone {
        type ComputedValue = LengthOrNone;

        #[inline]
        fn to_computed_value(&self, context: &Context) -> LengthOrNone {
            match *self {
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
            match self {
                &LengthOrNone::Length(length) => length.to_css(dest),
                &LengthOrNone::None => dest.write_str("none"),
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
    #[derive(Clone, PartialEq, HeapSizeOf)]
    pub enum Image {
        Url(Url),
        LinearGradient(LinearGradient),
    }

    impl fmt::Debug for Image {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                &Image::Url(ref url) => write!(f, "url(\"{}\")", url),
                &Image::LinearGradient(ref grad) => write!(f, "linear-gradient({:?})", grad),
            }
        }
    }

    /// Computed values for a CSS linear gradient.
    #[derive(Clone, PartialEq, HeapSizeOf)]
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
            for stop in self.stops.iter() {
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
            for stop in self.stops.iter() {
                let _ = write!(f, ", {:?}", stop);
            }
            Ok(())
        }
    }

    /// Computed values for one color stop in a linear gradient.
    #[derive(Clone, PartialEq, Copy, HeapSizeOf)]
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
}
