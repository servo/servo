/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

#![macro_use]

use std::ascii::AsciiExt;
use std::borrow::ToOwned;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;

use util::logical_geometry::{WritingMode, LogicalMargin};
use util::geometry::Au;
use url::Url;
use cssparser::{Parser, Color, RGBA, AtRuleParser, DeclarationParser,
                DeclarationListParser, parse_important, ToCss};
use geom::SideOffsets2D;

use values::specified::BorderStyle;
use values::computed;
use selector_matching::DeclarationBlock;
use parser::{ParserContext, log_css_error};
use stylesheets::Origin;
use computed_values;

use self::property_bit_field::PropertyBitField;


<%!

import re

def to_rust_ident(name):
    name = name.replace("-", "_")
    if name in ["static", "super", "box", "move"]:  # Rust keywords
        name += "_"
    return name

def to_camel_case(ident):
    return re.sub("_([a-z])", lambda m: m.group(1).upper(), ident.strip("_").capitalize())

class Longhand(object):
    def __init__(self, name, derived_from=None, experimental=False):
        self.name = name
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.style_struct = THIS_STYLE_STRUCT
        self.experimental = experimental
        if derived_from is None:
            self.derived_from = None
        else:
            self.derived_from = [ to_rust_ident(name) for name in derived_from ]

class Shorthand(object):
    def __init__(self, name, sub_properties):
        self.name = name
        self.ident = to_rust_ident(name)
        self.camel_case = to_camel_case(self.ident)
        self.derived_from = None
        self.sub_properties = [LONGHANDS_BY_NAME[s] for s in sub_properties]

class StyleStruct(object):
    def __init__(self, name, inherited):
        self.name = name
        self.ident = to_rust_ident(name.lower())
        self.longhands = []
        self.inherited = inherited

STYLE_STRUCTS = []
THIS_STYLE_STRUCT = None
LONGHANDS = []
LONGHANDS_BY_NAME = {}
DERIVED_LONGHANDS = {}
SHORTHANDS = []

def new_style_struct(name, is_inherited):
    global THIS_STYLE_STRUCT

    style_struct = StyleStruct(name, is_inherited)
    STYLE_STRUCTS.append(style_struct)
    THIS_STYLE_STRUCT = style_struct
    return ""

def switch_to_style_struct(name):
    global THIS_STYLE_STRUCT

    for style_struct in STYLE_STRUCTS:
        if style_struct.name == name:
            THIS_STYLE_STRUCT = style_struct
            return ""
    fail()
%>

pub mod longhands {
    use values::computed;

    pub fn computed_as_specified<T>(value: T, _context: &computed::Context) -> T {
        value
    }

    <%def name="raw_longhand(name, derived_from=None, experimental=False)">
    <%
        if derived_from is not None:
            derived_from = derived_from.split()

        property = Longhand(name, derived_from=derived_from, experimental=experimental)
        THIS_STYLE_STRUCT.longhands.append(property)
        LONGHANDS.append(property)
        LONGHANDS_BY_NAME[name] = property

        if derived_from is not None:
            for name in derived_from:
                DERIVED_LONGHANDS.setdefault(name, []).append(property)
    %>
        pub mod ${property.ident} {
            % if derived_from is None:
                use cssparser::Parser;
                use parser::ParserContext;
                use properties::{CSSWideKeyword, DeclaredValue};
            % endif
            #[allow(unused_imports)]
            use values::{computed, specified};
            ${caller.body()}
            % if derived_from is None:
                pub fn parse_declared(context: &ParserContext, input: &mut Parser)
                                   -> Result<DeclaredValue<SpecifiedValue>, ()> {
                    match input.try(CSSWideKeyword::parse) {
                        Ok(CSSWideKeyword::InheritKeyword) => Ok(DeclaredValue::Inherit),
                        Ok(CSSWideKeyword::InitialKeyword) => Ok(DeclaredValue::Initial),
                        Ok(CSSWideKeyword::UnsetKeyword) => Ok(DeclaredValue::${
                            "Inherit" if THIS_STYLE_STRUCT.inherited else "Initial"}),
                        Err(()) => parse_specified(context, input),
                    }
                }
            % endif
        }
    </%def>

    <%def name="longhand(name, derived_from=None, experimental=False)">
        <%self:raw_longhand name="${name}" derived_from="${derived_from}"
                            experimental="${experimental}">
            ${caller.body()}
            % if derived_from is None:
                pub fn parse_specified(context: &ParserContext, input: &mut Parser)
                                   -> Result<DeclaredValue<SpecifiedValue>, ()> {
                    parse(context, input).map(DeclaredValue::SpecifiedValue)
                }
            % endif
        </%self:raw_longhand>
    </%def>

    <%def name="single_keyword_computed(name, values, experimental=False)">
        <%self:longhand name="${name}" experimental="${experimental}">
            pub use self::computed_value::T as SpecifiedValue;
            ${caller.body()}
            pub mod computed_value {
                define_css_keyword_enum! { T:
                    % for value in values.split():
                        "${value}" => ${to_rust_ident(value)},
                    % endfor
                }
            }
            #[inline] pub fn get_initial_value() -> computed_value::T {
                computed_value::T::${to_rust_ident(values.split()[0])}
            }
            pub fn parse(_context: &ParserContext, input: &mut Parser)
                         -> Result<SpecifiedValue, ()> {
                computed_value::T::parse(input)
            }
        </%self:longhand>
    </%def>

    <%def name="single_keyword(name, values, experimental=False)">
        <%self:single_keyword_computed name="${name}"
                                       values="${values}"
                                       experimental="${experimental}">
            // The computed value is the same as the specified value.
            pub use super::computed_as_specified as to_computed_value;
        </%self:single_keyword_computed>
    </%def>

    <%def name="predefined_type(name, type, initial_value, parse_method='parse')">
        <%self:longhand name="${name}">
            #[allow(unused_imports)]
            use util::geometry::Au;
            pub use values::computed::compute_${type} as to_computed_value;
            pub type SpecifiedValue = specified::${type};
            pub mod computed_value {
                pub use values::computed::${type} as T;
            }
            #[inline] pub fn get_initial_value() -> computed_value::T { ${initial_value} }
            #[inline] pub fn parse(_context: &ParserContext, input: &mut Parser)
                                   -> Result<SpecifiedValue, ()> {
                specified::${type}::${parse_method}(input)
            }
        </%self:longhand>
    </%def>


    // CSS 2.1, Section 8 - Box model

    ${new_style_struct("Margin", is_inherited=False)}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("margin-" + side, "LengthOrPercentageOrAuto",
                          "computed::LengthOrPercentageOrAuto::Length(Au(0))")}
    % endfor

    ${new_style_struct("Padding", is_inherited=False)}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("padding-" + side, "LengthOrPercentage",
                          "computed::LengthOrPercentage::Length(Au(0))",
                          "parse_non_negative")}
    % endfor

    ${new_style_struct("Border", is_inherited=False)}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("border-%s-color" % side, "CSSColor", "::cssparser::Color::CurrentColor")}
    % endfor

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type("border-%s-style" % side, "BorderStyle", "computed::BorderStyle::none")}
    % endfor

    % for side in ["top", "right", "bottom", "left"]:
        <%self:longhand name="border-${side}-width">
            use util::geometry::Au;
            #[inline]
            pub fn parse(_context: &ParserContext, input: &mut Parser)
                                   -> Result<SpecifiedValue, ()> {
                specified::parse_border_width(input)
            }
            pub type SpecifiedValue = specified::Length;
            pub mod computed_value {
                use util::geometry::Au;
                pub type T = Au;
            }
            #[inline] pub fn get_initial_value() -> computed_value::T {
                Au::from_px(3)  // medium
            }
            #[inline]
            pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                  -> computed_value::T {
                if !context.border_${side}_present {
                    Au(0)
                } else {
                    computed::compute_Au(value, context)
                }
            }
        </%self:longhand>
    % endfor

    // FIXME(#4126): when gfx supports painting it, make this Size2D<LengthOrPercentage>
    % for corner in ["top-left", "top-right", "bottom-right", "bottom-left"]:
        ${predefined_type("border-" + corner + "-radius", "LengthOrPercentage",
                          "computed::LengthOrPercentage::Length(Au(0))",
                          "parse_non_negative")}
    % endfor

    ${new_style_struct("Outline", is_inherited=False)}

    // TODO(pcwalton): `invert`
    ${predefined_type("outline-color", "CSSColor", "::cssparser::Color::CurrentColor")}

    <%self:longhand name="outline-style">
        pub use values::specified::BorderStyle as SpecifiedValue;
        pub use super::computed_as_specified as to_computed_value;
        pub fn get_initial_value() -> SpecifiedValue { SpecifiedValue::none }
        pub mod computed_value {
            pub use values::specified::BorderStyle as T;
        }
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            match SpecifiedValue::parse(input) {
                Ok(SpecifiedValue::hidden) => Err(()),
                result => result
            }
        }
    </%self:longhand>

    <%self:longhand name="outline-width">
        pub use super::border_top_width::{get_initial_value, parse};
        pub use values::computed::compute_Au as to_computed_value;
        pub type SpecifiedValue = super::border_top_width::SpecifiedValue;
        pub mod computed_value {
            pub use util::geometry::Au as T;
        }
    </%self:longhand>

    ${predefined_type("outline-offset", "Length", "Au(0)")}

    ${new_style_struct("PositionOffsets", is_inherited=False)}

    % for side in ["top", "right", "bottom", "left"]:
        ${predefined_type(side, "LengthOrPercentageOrAuto",
                          "computed::LengthOrPercentageOrAuto::Auto")}
    % endfor

    // CSS 2.1, Section 9 - Visual formatting model

    ${new_style_struct("Box", is_inherited=False)}

    // TODO(SimonSapin): don't parse `inline-table`, since we don't support it
    <%self:single_keyword_computed name="display"
            values="inline block inline-block
            table inline-table table-row-group table-header-group table-footer-group
            table-row table-column-group table-column table-cell table-caption
            list-item
            none">
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            use self::computed_value::T;
//            if context.is_root_element && value == list_item {
//                return block
//            }
            if context.positioned || context.floated || context.is_root_element {
                match value {
                    T::inline_table => T::table,
                    T::inline | T::inline_block |
                    T::table_row_group | T::table_column |
                    T::table_column_group | T::table_header_group |
                    T::table_footer_group | T::table_row | T::table_cell |
                    T::table_caption
                    => T::block,
                    _ => value,
                }
            } else {
                value
            }
        }
    </%self:single_keyword_computed>

    ${single_keyword("position", "static absolute relative fixed")}
    ${single_keyword("float", "none left right")}
    ${single_keyword("clear", "none left right both")}

    <%self:longhand name="-servo-display-for-hypothetical-box" derived_from="display">
        pub use super::computed_as_specified as to_computed_value;
        pub use super::display::{SpecifiedValue, get_initial_value};
        pub use super::display::{parse};

        pub mod computed_value {
            pub type T = super::SpecifiedValue;
        }

        #[inline]
        pub fn derive_from_display(_: super::display::computed_value::T,
                                   context: &computed::Context)
                                   -> computed_value::T {
            context.display
        }

    </%self:longhand>

    <%self:longhand name="z-index">
        pub use super::computed_as_specified as to_computed_value;
        pub type SpecifiedValue = computed_value::T;
        pub mod computed_value {
            use cssparser::ToCss;
            use text_writer::{self, TextWriter};

            #[derive(PartialEq, Clone, Eq, Copy)]
            pub enum T {
                Auto,
                Number(i32),
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                    match self {
                        &T::Auto => dest.write_str("auto"),
                        &T::Number(number) => write!(dest, "{}", number),
                    }
                }
            }

            impl T {
                pub fn number_or_zero(self) -> i32 {
                    match self {
                        T::Auto => 0,
                        T::Number(value) => value,
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::Auto
        }
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                Ok(computed_value::T::Auto)
            } else {
                Ok(computed_value::T::Number(try!(input.expect_integer()) as i32))
            }
        }
    </%self:longhand>

    ${new_style_struct("InheritedBox", is_inherited=True)}

    ${single_keyword("direction", "ltr rtl", experimental=True)}

    // CSS 2.1, Section 10 - Visual formatting model details

    ${switch_to_style_struct("Box")}

    ${predefined_type("width", "LengthOrPercentageOrAuto",
                      "computed::LengthOrPercentageOrAuto::Auto",
                      "parse_non_negative")}
    <%self:longhand name="height">
        pub type SpecifiedValue = specified::LengthOrPercentageOrAuto;
        pub mod computed_value {
            pub use values::computed::LengthOrPercentageOrAuto as T;
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { computed::LengthOrPercentageOrAuto::Auto }
        #[inline]
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            specified::LengthOrPercentageOrAuto::parse_non_negative(input)
        }
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match (value, context.inherited_height) {
                (specified::LengthOrPercentageOrAuto::Percentage(_),
                 computed::LengthOrPercentageOrAuto::Auto)
                if !context.is_root_element && !context.positioned => {
                    computed::LengthOrPercentageOrAuto::Auto
                },
                _ => computed::compute_LengthOrPercentageOrAuto(value, context)
            }
        }
    </%self:longhand>

    ${predefined_type("min-width", "LengthOrPercentage",
                      "computed::LengthOrPercentage::Length(Au(0))",
                      "parse_non_negative")}
    ${predefined_type("max-width", "LengthOrPercentageOrNone",
                      "computed::LengthOrPercentageOrNone::None",
                      "parse_non_negative")}

    ${predefined_type("min-height", "LengthOrPercentage",
                      "computed::LengthOrPercentage::Length(Au(0))",
                      "parse_non_negative")}
    ${predefined_type("max-height", "LengthOrPercentageOrNone",
                      "computed::LengthOrPercentageOrNone::None",
                      "parse_non_negative")}

    ${switch_to_style_struct("InheritedBox")}

    <%self:longhand name="line-height">
        use cssparser::ToCss;
        use text_writer::{self, TextWriter};
        use values::CSSFloat;

        #[derive(Clone, PartialEq, Copy)]
        pub enum SpecifiedValue {
            Normal,
            Length(specified::Length),
            Number(CSSFloat),
            Percentage(CSSFloat),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                match self {
                    &SpecifiedValue::Normal => dest.write_str("normal"),
                    &SpecifiedValue::Length(length) => length.to_css(dest),
                    &SpecifiedValue::Number(number) => write!(dest, "{}", number),
                    &SpecifiedValue::Percentage(number) => write!(dest, "{}%", number * 100.),
                }
            }
        }
        /// normal | <number> | <length> | <percentage>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            use std::ascii::AsciiExt;
            use cssparser::Token;
            match try!(input.next()) {
                Token::Number(ref value) if value.value >= 0. => {
                    Ok(SpecifiedValue::Number(value.value))
                }
                Token::Percentage(ref value) if value.unit_value >= 0. => {
                    Ok(SpecifiedValue::Percentage(value.unit_value))
                }
                Token::Dimension(ref value, ref unit) if value.value >= 0. => {
                    specified::Length::parse_dimension(value.value, unit)
                    .map(SpecifiedValue::Length)
                }
                Token::Ident(ref value) if value.eq_ignore_ascii_case("normal") => {
                    Ok(SpecifiedValue::Normal)
                }
                _ => Err(()),
            }
        }
        pub mod computed_value {
            use values::CSSFloat;
            use util::geometry::Au;
            use std::fmt;
            #[derive(PartialEq, Copy, Clone)]
            pub enum T {
                Normal,
                Length(Au),
                Number(CSSFloat),
            }
            impl fmt::Debug for T {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        &T::Normal => write!(f, "normal"),
                        &T::Length(length) => write!(f, "{:?}%", length),
                        &T::Number(number) => write!(f, "{}", number),
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { computed_value::T::Normal }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match value {
                SpecifiedValue::Normal => computed_value::T::Normal,
                SpecifiedValue::Length(value) => {
                    computed_value::T::Length(computed::compute_Au(value, context))
                }
                SpecifiedValue::Number(value) => computed_value::T::Number(value),
                SpecifiedValue::Percentage(value) => {
                    computed_value::T::Length(computed::compute_Au(
                        specified::Length::Em(value), context))
                }
            }
        }
    </%self:longhand>

    ${switch_to_style_struct("Box")}

    <%self:longhand name="vertical-align">
        use cssparser::ToCss;
        use text_writer::{self, TextWriter};

        <% vertical_align_keywords = (
            "baseline sub super top text-top middle bottom text-bottom".split()) %>
        #[allow(non_camel_case_types)]
        #[derive(Clone, PartialEq, Copy)]
        pub enum SpecifiedValue {
            % for keyword in vertical_align_keywords:
                ${to_rust_ident(keyword)},
            % endfor
            LengthOrPercentage(specified::LengthOrPercentage),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                match self {
                    % for keyword in vertical_align_keywords:
                        &SpecifiedValue::${to_rust_ident(keyword)} => dest.write_str("${keyword}"),
                    % endfor
                    &SpecifiedValue::LengthOrPercentage(value) => value.to_css(dest),
                }
            }
        }
        /// baseline | sub | super | top | text-top | middle | bottom | text-bottom
        /// | <percentage> | <length>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.try(specified::LengthOrPercentage::parse_non_negative)
            .map(SpecifiedValue::LengthOrPercentage)
            .or_else(|()| {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    % for keyword in vertical_align_keywords[:-1]:
                        "${keyword}" => Ok(SpecifiedValue::${to_rust_ident(keyword)}),
                    % endfor

                    // Hack to work around quirks of macro_rules parsing in match_ignore_ascii_case!
                    % for keyword in vertical_align_keywords[-1:]:
                        "${keyword}" => Ok(SpecifiedValue::${to_rust_ident(keyword)})
                    % endfor
                    _ => Err(())
                }
            })
        }
        pub mod computed_value {
            use values::CSSFloat;
            use util::geometry::Au;
            use std::fmt;
            #[allow(non_camel_case_types)]
            #[derive(PartialEq, Copy, Clone)]
            pub enum T {
                % for keyword in vertical_align_keywords:
                    ${to_rust_ident(keyword)},
                % endfor
                Length(Au),
                Percentage(CSSFloat),
            }
            impl fmt::Debug for T {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        % for keyword in vertical_align_keywords:
                            &T::${to_rust_ident(keyword)} => write!(f, "${keyword}"),
                        % endfor
                        &T::Length(length) => write!(f, "{:?}", length),
                        &T::Percentage(number) => write!(f, "{}%", number),
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { computed_value::T::baseline }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match value {
                % for keyword in vertical_align_keywords:
                    SpecifiedValue::${to_rust_ident(keyword)} => {
                        computed_value::T::${to_rust_ident(keyword)}
                    }
                % endfor
                SpecifiedValue::LengthOrPercentage(value) => {
                    match computed::compute_LengthOrPercentage(value, context) {
                        computed::LengthOrPercentage::Length(value) => {
                            computed_value::T::Length(value)
                        }
                        computed::LengthOrPercentage::Percentage(value) => {
                            computed_value::T::Percentage(value)
                        }
                    }
                }
            }
        }
    </%self:longhand>


    // CSS 2.1, Section 11 - Visual effects
    // FIXME: Implement scrolling for `scroll` and `auto` (#2742).
    ${single_keyword("overflow", "visible hidden scroll auto")}

    ${switch_to_style_struct("InheritedBox")}

    // TODO: collapse. Well, do tables first.
    ${single_keyword("visibility", "visible hidden")}

    // CSS 2.1, Section 12 - Generated content, automatic numbering, and lists

    ${switch_to_style_struct("Box")}

    <%self:longhand name="content">
            pub use super::computed_as_specified as to_computed_value;
            pub use self::computed_value::T as SpecifiedValue;
            pub use self::computed_value::ContentItem;
            use cssparser::Token;

            pub mod computed_value {
                use std::borrow::IntoCow;
                use cssparser::{ToCss, Token};
                use text_writer::{self, TextWriter};

                #[derive(PartialEq, Eq, Clone)]
                pub enum ContentItem {
                    StringContent(String),
                }

                impl ToCss for ContentItem {
                    fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                        match self {
                            &ContentItem::StringContent(ref s) => {
                                Token::QuotedString((&**s).into_cow()).to_css(dest)
                            }
                        }
                    }
                }

                #[allow(non_camel_case_types)]
                #[derive(PartialEq, Eq, Clone)]
                pub enum T {
                    normal,
                    none,
                    Content(Vec<ContentItem>),
                }

                impl ToCss for T {
                    fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                        match self {
                            &T::normal => dest.write_str("normal"),
                            &T::none => dest.write_str("none"),
                            &T::Content(ref content) => {
                                let mut iter = content.iter();
                                try!(iter.next().unwrap().to_css(dest));
                                for c in iter {
                                    try!(c.to_css(dest));
                                }
                                Ok(())
                            }
                        }
                    }
                }
            }
            #[inline]
            pub fn get_initial_value() -> computed_value::T  {
                computed_value::T::normal
            }

            // normal | none | [ <string> ]+
            // TODO: <uri>, <counter>, attr(<identifier>), open-quote, close-quote, no-open-quote, no-close-quote
            pub fn parse(_context: &ParserContext, input: &mut Parser)
                         -> Result<SpecifiedValue, ()> {
                if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                    return Ok(SpecifiedValue::normal)
                }
                if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                    return Ok(SpecifiedValue::none)
                }
                let mut content = vec![];
                loop {
                    match input.next() {
                        Ok(Token::QuotedString(value)) => {
                            content.push(ContentItem::StringContent(value.into_owned()))
                        }
                        Err(()) if !content.is_empty() => {
                            return Ok(SpecifiedValue::Content(content))
                        }
                        _ => return Err(())
                    }
                }
            }
    </%self:longhand>

    ${new_style_struct("List", is_inherited=True)}

    ${single_keyword("list-style-position", "outside inside")}

    // TODO(pcwalton): Implement the full set of counter styles per CSS-COUNTER-STYLES [1] 6.1:
    //
    //     decimal, decimal-leading-zero, arabic-indic, armenian, upper-armenian, lower-armenian,
    //     bengali, cambodian, khmer, cjk-decimal, devanagiri, georgian, gujarati, gurmukhi,
    //     hebrew, kannada, lao, malayalam, mongolian, myanmar, oriya, persian, lower-roman,
    //     upper-roman, telugu, thai, tibetan
    //
    // [1]: http://dev.w3.org/csswg/css-counter-styles/
    ${single_keyword("list-style-type",
                     "disc none circle square disclosure-open disclosure-closed")}

    <%self:longhand name="list-style-image">
        use std::borrow::IntoCow;
        use url::Url;
        use cssparser::{ToCss, Token};
        use text_writer::{self, TextWriter};

        #[derive(Clone, PartialEq, Eq)]
        pub enum SpecifiedValue {
            None,
            Url(Url),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                match *self {
                    SpecifiedValue::None => dest.write_str("none"),
                    SpecifiedValue::Url(ref url) => {
                        Token::Url(url.to_string().into_cow()).to_css(dest)
                    }
                }
            }
        }

        pub mod computed_value {
            use url::Url;
            pub type T = Option<Url>;
        }

        pub fn to_computed_value(value: SpecifiedValue, _context: &computed::Context)
                                 -> computed_value::T {
            match value {
                SpecifiedValue::None => None,
                SpecifiedValue::Url(url) => Some(url),
            }
        }

        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(SpecifiedValue::None)
            } else {
                Ok(SpecifiedValue::Url(context.parse_url(&*try!(input.expect_url()))))
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            None
        }
    </%self:longhand>

    // CSS 2.1, Section 13 - Paged media

    // CSS 2.1, Section 14 - Colors and Backgrounds

    ${new_style_struct("Background", is_inherited=False)}
    ${predefined_type("background-color", "CSSColor",
                      "::cssparser::Color::RGBA(::cssparser::RGBA { red: 0., green: 0., blue: 0., alpha: 0. }) /* transparent */")}

    <%self:longhand name="background-image">
        use values::specified::{CSSImage, Image};
        pub mod computed_value {
            use values::computed;
            pub type T = Option<computed::Image>;
        }
        pub type SpecifiedValue = CSSImage;
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            None
        }
        pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(CSSImage(None))
            } else {
                Ok(CSSImage(Some(try!(Image::parse(context, input)))))
            }
        }
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            match value {
                CSSImage(None) => None,
                CSSImage(Some(image)) => Some(image.to_computed_value(context)),
            }
        }
    </%self:longhand>

    <%self:longhand name="background-position">
            use cssparser::ToCss;
            use text_writer::{self, TextWriter};

            pub mod computed_value {
                use values::computed::LengthOrPercentage;

                #[derive(PartialEq, Copy, Clone)]
                pub struct T {
                    pub horizontal: LengthOrPercentage,
                    pub vertical: LengthOrPercentage,
                }
            }

            #[derive(Clone, PartialEq, Copy)]
            pub struct SpecifiedValue {
                pub horizontal: specified::LengthOrPercentage,
                pub vertical: specified::LengthOrPercentage,
            }

            impl ToCss for SpecifiedValue {
                fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                    try!(self.horizontal.to_css(dest));
                    try!(dest.write_str(" "));
                    try!(self.vertical.to_css(dest));
                    Ok(())
                }
            }

            impl SpecifiedValue {
                fn new(first: specified::PositionComponent, second: specified::PositionComponent)
                        -> Result<SpecifiedValue, ()> {
                    let (horiz, vert) = match (category(first), category(second)) {
                        // Don't allow two vertical keywords or two horizontal keywords.
                        (PositionCategory::HorizontalKeyword, PositionCategory::HorizontalKeyword) |
                        (PositionCategory::VerticalKeyword, PositionCategory::VerticalKeyword) => return Err(()),

                        // Swap if both are keywords and vertical precedes horizontal.
                        (PositionCategory::VerticalKeyword, PositionCategory::HorizontalKeyword) |
                        (PositionCategory::VerticalKeyword, PositionCategory::OtherKeyword) |
                        (PositionCategory::OtherKeyword, PositionCategory::HorizontalKeyword) => (second, first),

                        // By default, horizontal is first.
                        _ => (first, second),
                    };
                    Ok(SpecifiedValue {
                        horizontal: horiz.to_length_or_percentage(),
                        vertical: vert.to_length_or_percentage(),
                    })
                }
            }

            // Collapse `Position` into a few categories to simplify the above `match` expression.
            enum PositionCategory {
                HorizontalKeyword,
                VerticalKeyword,
                OtherKeyword,
                LengthOrPercentage,
            }
            fn category(p: specified::PositionComponent) -> PositionCategory {
                match p {
                    specified::PositionComponent::Left |
                    specified::PositionComponent::Right =>
                        PositionCategory::HorizontalKeyword,
                    specified::PositionComponent::Top |
                    specified::PositionComponent::Bottom =>
                        PositionCategory::VerticalKeyword,
                    specified::PositionComponent::Center =>
                        PositionCategory::OtherKeyword,
                    specified::PositionComponent::Length(_) |
                    specified::PositionComponent::Percentage(_) =>
                        PositionCategory::LengthOrPercentage,
                }
            }

            #[inline]
            pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                     -> computed_value::T {
                computed_value::T {
                    horizontal: computed::compute_LengthOrPercentage(value.horizontal, context),
                    vertical: computed::compute_LengthOrPercentage(value.vertical, context),
                }
            }

            #[inline]
            pub fn get_initial_value() -> computed_value::T {
                computed_value::T {
                    horizontal: computed::LengthOrPercentage::Percentage(0.0),
                    vertical: computed::LengthOrPercentage::Percentage(0.0),
                }
            }

            pub fn parse(_context: &ParserContext, input: &mut Parser)
                         -> Result<SpecifiedValue, ()> {
                let first = try!(specified::PositionComponent::parse(input));
                let second = input.try(specified::PositionComponent::parse)
                    .unwrap_or(specified::PositionComponent::Center);
                SpecifiedValue::new(first, second)
            }
    </%self:longhand>

    ${single_keyword("background-repeat", "repeat repeat-x repeat-y no-repeat")}

    ${single_keyword("background-attachment", "scroll fixed")}

    ${new_style_struct("Color", is_inherited=True)}

    <%self:raw_longhand name="color">
        use cssparser::{Color, RGBA};
        use values::specified::{CSSColor, CSSRGBA};
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, _context: &computed::Context)
                                 -> computed_value::T {
            value.parsed
        }

        pub type SpecifiedValue = CSSRGBA;
        pub mod computed_value {
            use cssparser;
            pub type T = cssparser::RGBA;
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            RGBA { red: 0., green: 0., blue: 0., alpha: 1. }  /* black */
        }
        pub fn parse_specified(_context: &ParserContext, input: &mut Parser)
                               -> Result<DeclaredValue<SpecifiedValue>, ()> {
            let value = try!(CSSColor::parse(input));
            let rgba = match value.parsed {
                Color::RGBA(rgba) => rgba,
                Color::CurrentColor => return Ok(DeclaredValue::Inherit)
            };
            Ok(DeclaredValue::SpecifiedValue(CSSRGBA {
                parsed: rgba,
                authored: value.authored,
            }))
        }
    </%self:raw_longhand>

    // CSS 2.1, Section 15 - Fonts

    ${new_style_struct("Font", is_inherited=True)}

    <%self:longhand name="font-family">
        pub use super::computed_as_specified as to_computed_value;
        use std::borrow::ToOwned;
        use self::computed_value::FontFamily;
        pub mod computed_value {
            use cssparser::ToCss;
            use text_writer::{self, TextWriter};

            #[derive(PartialEq, Eq, Clone)]
            pub enum FontFamily {
                FamilyName(String),
                // Generic
//                Serif,
//                SansSerif,
//                Cursive,
//                Fantasy,
//                Monospace,
            }
            impl FontFamily {
                pub fn name(&self) -> &str {
                    match *self {
                        FontFamily::FamilyName(ref name) => name,
                    }
                }
            }
            impl ToCss for FontFamily {
                fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                    match self {
                        &FontFamily::FamilyName(ref name) => dest.write_str(&**name),
                    }
                }
            }
            impl ToCss for Vec<FontFamily> {
                fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                    let mut iter = self.iter();
                    try!(iter.next().unwrap().to_css(dest));
                    for family in iter {
                        try!(dest.write_str(", "));
                        try!(family.to_css(dest));
                    }
                    Ok(())
                }
            }
            pub type T = Vec<FontFamily>;
        }
        pub type SpecifiedValue = computed_value::T;

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            vec![FontFamily::FamilyName("serif".to_owned())]
        }
        /// <family-name>#
        /// <family-name> = <string> | [ <ident>+ ]
        /// TODO: <generic-family>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.parse_comma_separated(parse_one_family)
        }
        pub fn parse_one_family(input: &mut Parser) -> Result<FontFamily, ()> {
            if let Ok(value) = input.try(|input| input.expect_string()) {
                return Ok(FontFamily::FamilyName(value.into_owned()))
            }
            let first_ident = try!(input.expect_ident());
//            match_ignore_ascii_case! { first_ident,
//                "serif" => return Ok(Serif),
//                "sans-serif" => return Ok(SansSerif),
//                "cursive" => return Ok(Cursive),
//                "fantasy" => return Ok(Fantasy),
//                "monospace" => return Ok(Monospace)
//                _ => {}
//            }
            let mut value = first_ident.into_owned();
            while let Ok(ident) = input.try(|input| input.expect_ident()) {
                value.push_str(" ");
                value.push_str(&ident);
            }
            Ok(FontFamily::FamilyName(value))
        }
    </%self:longhand>


    ${single_keyword("font-style", "normal italic oblique")}
    ${single_keyword("font-variant", "normal small-caps")}

    <%self:longhand name="font-weight">
        use cssparser::ToCss;
        use text_writer::{self, TextWriter};

        #[derive(Clone, PartialEq, Eq, Copy)]
        pub enum SpecifiedValue {
            Bolder,
            Lighter,
            % for weight in range(100, 901, 100):
                Weight${weight},
            % endfor
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                match self {
                    &SpecifiedValue::Bolder => dest.write_str("bolder"),
                    &SpecifiedValue::Lighter => dest.write_str("lighter"),
                    % for weight in range(100, 901, 100):
                        &SpecifiedValue::Weight${weight} => dest.write_str("${weight}"),
                    % endfor
                }
            }
        }
        /// normal | bold | bolder | lighter | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.try(|input| {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    "bold" => Ok(SpecifiedValue::Weight700),
                    "normal" => Ok(SpecifiedValue::Weight400),
                    "bolder" => Ok(SpecifiedValue::Bolder),
                    "lighter" => Ok(SpecifiedValue::Lighter)
                    _ => Err(())
                }
            }).or_else(|()| {
                match try!(input.expect_integer()) {
                    100 => Ok(SpecifiedValue::Weight100),
                    200 => Ok(SpecifiedValue::Weight200),
                    300 => Ok(SpecifiedValue::Weight300),
                    400 => Ok(SpecifiedValue::Weight400),
                    500 => Ok(SpecifiedValue::Weight500),
                    600 => Ok(SpecifiedValue::Weight600),
                    700 => Ok(SpecifiedValue::Weight700),
                    800 => Ok(SpecifiedValue::Weight800),
                    900 => Ok(SpecifiedValue::Weight900),
                    _ => Err(())
                }
            })
        }
        pub mod computed_value {
            use std::fmt;
            #[derive(PartialEq, Eq, Copy, Clone)]
            pub enum T {
                % for weight in range(100, 901, 100):
                    Weight${weight},
                % endfor
            }
            impl fmt::Debug for T {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        % for weight in range(100, 901, 100):
                            &T::Weight${weight} => write!(f, "{}", ${weight}i),
                        % endfor
                    }
                }
            }
            impl T {
                pub fn is_bold(self) -> bool {
                    match self {
                        T::Weight900 | T::Weight800 |
                        T::Weight700 | T::Weight600 => true,
                        _ => false
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::Weight400  // normal
        }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match value {
                % for weight in range(100, 901, 100):
                    SpecifiedValue::Weight${weight} => computed_value::T::Weight${weight},
                % endfor
                SpecifiedValue::Bolder => match context.inherited_font_weight {
                    computed_value::T::Weight100 => computed_value::T::Weight400,
                    computed_value::T::Weight200 => computed_value::T::Weight400,
                    computed_value::T::Weight300 => computed_value::T::Weight400,
                    computed_value::T::Weight400 => computed_value::T::Weight700,
                    computed_value::T::Weight500 => computed_value::T::Weight700,
                    computed_value::T::Weight600 => computed_value::T::Weight900,
                    computed_value::T::Weight700 => computed_value::T::Weight900,
                    computed_value::T::Weight800 => computed_value::T::Weight900,
                    computed_value::T::Weight900 => computed_value::T::Weight900,
                },
                SpecifiedValue::Lighter => match context.inherited_font_weight {
                    computed_value::T::Weight100 => computed_value::T::Weight100,
                    computed_value::T::Weight200 => computed_value::T::Weight100,
                    computed_value::T::Weight300 => computed_value::T::Weight100,
                    computed_value::T::Weight400 => computed_value::T::Weight100,
                    computed_value::T::Weight500 => computed_value::T::Weight100,
                    computed_value::T::Weight600 => computed_value::T::Weight400,
                    computed_value::T::Weight700 => computed_value::T::Weight400,
                    computed_value::T::Weight800 => computed_value::T::Weight700,
                    computed_value::T::Weight900 => computed_value::T::Weight700,
                },
            }
        }
    </%self:longhand>

    <%self:longhand name="font-size">
        use util::geometry::Au;
        pub type SpecifiedValue = specified::Length;  // Percentages are the same as em.
        pub mod computed_value {
            use util::geometry::Au;
            pub type T = Au;
        }
        const MEDIUM_PX: int = 16;
        #[inline] pub fn get_initial_value() -> computed_value::T {
            Au::from_px(MEDIUM_PX)
        }
        #[inline]
        pub fn to_computed_value(_value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            // We already computed this element's font size; no need to compute it again.
            return context.font_size
        }
        /// <length> | <percentage> | <absolute-size> | <relative-size>
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.try(specified::LengthOrPercentage::parse_non_negative)
            .map(|value| match value {
                specified::LengthOrPercentage::Length(value) => value,
                specified::LengthOrPercentage::Percentage(value) => specified::Length::Em(value)
            })
            .or_else(|()| {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    "xx-small" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 3 / 5)),
                    "x-small" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 3 / 4)),
                    "small" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 8 / 9)),
                    "medium" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX))),
                    "large" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 6 / 5)),
                    "x-large" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 3 / 2)),
                    "xx-large" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 2)),

                    // https://github.com/servo/servo/issues/3423#issuecomment-56321664
                    "smaller" => Ok(specified::Length::Em(0.85)),
                    "larger" => Ok(specified::Length::Em(1.2))

                    _ => Err(())
                }
            })
        }
    </%self:longhand>

    ${single_keyword("font-stretch",
                     "normal ultra-condensed extra-condensed condensed semi-condensed semi-expanded expanded extra-expanded ultra-expanded")}

    // CSS 2.1, Section 16 - Text

    ${new_style_struct("InheritedText", is_inherited=True)}

    // TODO: initial value should be 'start' (CSS Text Level 3, direction-dependent.)
    ${single_keyword("text-align", "left right center justify")}

    <%self:longhand name="letter-spacing">
        use cssparser::ToCss;
        use text_writer::{self, TextWriter};

        #[derive(Clone, Copy, PartialEq)]
        pub enum SpecifiedValue {
            Normal,
            Specified(specified::Length),
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                match *self {
                    SpecifiedValue::Normal => dest.write_str("normal"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use util::geometry::Au;
            pub type T = Option<Au>;
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            None
        }

        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            match value {
                SpecifiedValue::Normal => None,
                SpecifiedValue::Specified(l) => Some(computed::compute_Au(l, context))
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                Ok(SpecifiedValue::Normal)
            } else {
                specified::Length::parse_non_negative(input).map(SpecifiedValue::Specified)
            }
        }
    </%self:longhand>

    <%self:longhand name="word-spacing">
        use cssparser::ToCss;
        use text_writer::{self, TextWriter};

        #[derive(Clone, Copy, PartialEq)]
        pub enum SpecifiedValue {
            Normal,
            Specified(specified::Length),  // FIXME(SimonSapin) support percentages
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                match *self {
                    SpecifiedValue::Normal => dest.write_str("normal"),
                    SpecifiedValue::Specified(l) => l.to_css(dest),
                }
            }
        }

        pub mod computed_value {
            use util::geometry::Au;
            pub type T = Option<Au>;
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            None
        }

        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            match value {
                SpecifiedValue::Normal => None,
                SpecifiedValue::Specified(l) => Some(computed::compute_Au(l, context))
            }
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                Ok(SpecifiedValue::Normal)
            } else {
                specified::Length::parse_non_negative(input).map(SpecifiedValue::Specified)
            }
        }
    </%self:longhand>

    ${predefined_type("text-indent", "LengthOrPercentage", "computed::LengthOrPercentage::Length(Au(0))")}

    // Also known as "word-wrap" (which is more popular because of IE), but this is the preferred
    // name per CSS-TEXT 6.2.
    ${single_keyword("overflow-wrap", "normal break-word")}

    // TODO(pcwalton): Support `word-break: keep-all` once we have better CJK support.
    ${single_keyword("word-break", "normal break-all")}

    ${single_keyword("text-overflow", "clip ellipsis")}

    // TODO(pcwalton): Support `text-justify: distribute`.
    ${single_keyword("text-justify", "auto none inter-word")}

    ${new_style_struct("Text", is_inherited=False)}

    <%self:longhand name="text-decoration">
        pub use super::computed_as_specified as to_computed_value;
        use cssparser::ToCss;
        use text_writer::{self, TextWriter};

        #[derive(PartialEq, Eq, Copy, Clone)]
        pub struct SpecifiedValue {
            pub underline: bool,
            pub overline: bool,
            pub line_through: bool,
            // 'blink' is accepted in the parser but ignored.
            // Just not blinking the text is a conforming implementation per CSS 2.1.
        }

        impl ToCss for SpecifiedValue {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                let mut space = false;
                if self.underline {
                    try!(dest.write_str("underline"));
                    space = true;
                }
                if self.overline {
                    if space {
                        try!(dest.write_str(" "));
                    }
                    try!(dest.write_str("overline"));
                    space = true;
                }
                if self.line_through {
                    if space {
                        try!(dest.write_str(" "));
                    }
                    try!(dest.write_str("line-through"));
                }
                Ok(())
            }
        }
        pub mod computed_value {
            pub type T = super::SpecifiedValue;
            #[allow(non_upper_case_globals)]
            pub const none: T = super::SpecifiedValue {
                underline: false, overline: false, line_through: false
            };
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            computed_value::none
        }
        /// none | [ underline || overline || line-through || blink ]
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            let mut result = SpecifiedValue {
                underline: false, overline: false, line_through: false,
            };
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(result)
            }
            let mut blink = false;
            let mut empty = true;
            loop {
                match_ignore_ascii_case! { try!(input.expect_ident()),
                    "underline" => if result.underline { return Err(()) }
                                  else { empty = false; result.underline = true },
                    "overline" => if result.overline { return Err(()) }
                                  else { empty = false; result.overline = true },
                    "line-through" => if result.line_through { return Err(()) }
                                      else { empty = false; result.line_through = true },
                    "blink" => if blink { return Err(()) }
                               else { empty = false; blink = true }
                    _ => break
                }
            }
            if !empty { Ok(result) } else { Err(()) }
        }
    </%self:longhand>

    ${switch_to_style_struct("InheritedText")}

    <%self:longhand name="-servo-text-decorations-in-effect"
                    derived_from="display text-decoration">
        use cssparser::RGBA;
        pub use super::computed_as_specified as to_computed_value;

        #[derive(Clone, PartialEq, Copy)]
        pub struct SpecifiedValue {
            pub underline: Option<RGBA>,
            pub overline: Option<RGBA>,
            pub line_through: Option<RGBA>,
        }

        pub mod computed_value {
            pub type T = super::SpecifiedValue;
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            SpecifiedValue {
                underline: None,
                overline: None,
                line_through: None,
            }
        }

        fn maybe(flag: bool, context: &computed::Context) -> Option<RGBA> {
            if flag {
                Some(context.color)
            } else {
                None
            }
        }

        fn derive(context: &computed::Context) -> computed_value::T {
            // Start with no declarations if this is a block; otherwise, start with the
            // declarations in effect and add in the text decorations that this inline specifies.
            let mut result = match context.display {
                super::display::computed_value::T::inline => {
                    context.inherited_text_decorations_in_effect
                }
                _ => {
                    SpecifiedValue {
                        underline: None,
                        overline: None,
                        line_through: None,
                    }
                }
            };

            if result.underline.is_none() {
                result.underline = maybe(context.text_decoration.underline, context)
            }
            if result.overline.is_none() {
                result.overline = maybe(context.text_decoration.overline, context)
            }
            if result.line_through.is_none() {
                result.line_through = maybe(context.text_decoration.line_through, context)
            }

            result
        }

        #[inline]
        pub fn derive_from_text_decoration(_: super::text_decoration::computed_value::T,
                                           context: &computed::Context)
                                           -> computed_value::T {
            derive(context)
        }

        #[inline]
        pub fn derive_from_display(_: super::display::computed_value::T,
                                   context: &computed::Context)
                                   -> computed_value::T {
            derive(context)
        }
    </%self:longhand>

    ${single_keyword("white-space", "normal pre nowrap")}

    // TODO(pcwalton): `full-width`
    ${single_keyword("text-transform", "none capitalize uppercase lowercase")}

    ${single_keyword("text-rendering", "auto optimizespeed optimizelegibility geometricprecision")}

    // CSS 2.1, Section 17 - Tables
    ${new_style_struct("Table", is_inherited=False)}

    ${single_keyword("table-layout", "auto fixed")}

    ${new_style_struct("InheritedTable", is_inherited=True)}

    ${single_keyword("empty-cells", "show hide")}

    ${single_keyword("caption-side", "top bottom")}

    // CSS 2.1, Section 18 - User interface


    // CSS Writing Modes Level 3
    // http://dev.w3.org/csswg/css-writing-modes/
    ${switch_to_style_struct("InheritedBox")}

    ${single_keyword("writing-mode", "horizontal-tb vertical-rl vertical-lr", experimental=True)}

    // FIXME(SimonSapin): Add 'mixed' and 'upright' (needs vertical text support)
    // FIXME(SimonSapin): initial (first) value should be 'mixed', when that's implemented
    ${single_keyword("text-orientation", "sideways sideways-left sideways-right", experimental=True)}

    // CSS Basic User Interface Module Level 3
    // http://dev.w3.org/csswg/css-ui/
    ${switch_to_style_struct("Box")}

    ${single_keyword("box-sizing", "content-box border-box")}

    ${new_style_struct("Pointing", is_inherited=True)}

    <%self:longhand name="cursor">
        use util::cursor as util_cursor;
        pub use super::computed_as_specified as to_computed_value;
        pub use self::computed_value::T as SpecifiedValue;

        pub mod computed_value {
            use cssparser::ToCss;
            use text_writer::{self, TextWriter};
            use util::cursor::Cursor;

            #[derive(Clone, PartialEq, Eq, Copy, Debug)]
            pub enum T {
                AutoCursor,
                SpecifiedCursor(Cursor),
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                    match *self {
                        T::AutoCursor => dest.write_str("auto"),
                        T::SpecifiedCursor(c) => c.to_css(dest),
                    }
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::AutoCursor
        }
        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            use std::ascii::AsciiExt;
            let ident = try!(input.expect_ident());
            if ident.eq_ignore_ascii_case("auto") {
                Ok(SpecifiedValue::AutoCursor)
            } else {
                util_cursor::Cursor::from_css_keyword(&ident)
                .map(SpecifiedValue::SpecifiedCursor)
            }
        }
    </%self:longhand>

    // NB: `pointer-events: auto` (and use of `pointer-events` in anything that isn't SVG, in fact)
    // is nonstandard, slated for CSS4-UI.
    // TODO(pcwalton): SVG-only values.
    ${single_keyword("pointer-events", "auto none")}

    // Box-shadow, etc.
    ${new_style_struct("Effects", is_inherited=False)}

    <%self:longhand name="opacity">
        use values::CSSFloat;
        pub type SpecifiedValue = CSSFloat;
        pub mod computed_value {
            use values::CSSFloat;
            pub type T = CSSFloat;
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            1.0
        }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, _: &computed::Context)
                                 -> computed_value::T {
            if value < 0.0 {
                0.0
            } else if value > 1.0 {
                1.0
            } else {
                value
            }
        }
        fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            input.expect_number()
        }
    </%self:longhand>

    <%self:longhand name="box-shadow">
        use cssparser::{self, ToCss};
        use text_writer::{self, TextWriter};

        pub type SpecifiedValue = Vec<SpecifiedBoxShadow>;

        #[derive(Clone, PartialEq)]
        pub struct SpecifiedBoxShadow {
            pub offset_x: specified::Length,
            pub offset_y: specified::Length,
            pub blur_radius: specified::Length,
            pub spread_radius: specified::Length,
            pub color: Option<specified::CSSColor>,
            pub inset: bool,
        }

        impl ToCss for Vec<SpecifiedBoxShadow> {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                let mut iter = self.iter();
                if let Some(shadow) = iter.next() {
                    try!(shadow.to_css(dest));
                } else {
                    try!(dest.write_str("none"));
                    return Ok(())
                }
                for shadow in iter {
                    try!(dest.write_str(", "));
                    try!(shadow.to_css(dest));
                }
                Ok(())
            }
        }

        impl ToCss for SpecifiedBoxShadow {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                if self.inset {
                    try!(dest.write_str("inset "));
                }
                try!(self.blur_radius.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.spread_radius.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_x.to_css(dest));
                try!(dest.write_str(" "));
                try!(self.offset_y.to_css(dest));

                if let Some(ref color) = self.color {
                    try!(dest.write_str(" "));
                    try!(color.to_css(dest));
                }
                Ok(())
            }
        }

        pub mod computed_value {
            use util::geometry::Au;
            use values::computed;
            use std::fmt;

            pub type T = Vec<BoxShadow>;

            #[derive(Clone, PartialEq, Copy)]
            pub struct BoxShadow {
                pub offset_x: Au,
                pub offset_y: Au,
                pub blur_radius: Au,
                pub spread_radius: Au,
                pub color: computed::CSSColor,
                pub inset: bool,
            }

            impl fmt::Debug for BoxShadow {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    if self.inset {
                        let _ = write!(f, "inset ");
                    }
                    let _ = write!(f, "{:?} {:?} {:?} {:?} {:?}", self.offset_x, self.offset_y,
                                   self.blur_radius, self.spread_radius, self.color);
                    Ok(())
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            Vec::new()
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                Ok(Vec::new())
            } else {
                input.parse_comma_separated(parse_one_box_shadow)
            }
        }

        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            value.into_iter().map(|value| compute_one_box_shadow(value, context)).collect()
        }

        pub fn compute_one_box_shadow(value: SpecifiedBoxShadow, context: &computed::Context)
                                      -> computed_value::BoxShadow {
            computed_value::BoxShadow {
                offset_x: computed::compute_Au(value.offset_x, context),
                offset_y: computed::compute_Au(value.offset_y, context),
                blur_radius: computed::compute_Au(value.blur_radius, context),
                spread_radius: computed::compute_Au(value.spread_radius, context),
                color: value.color
                            .map(|color| color.parsed)
                            .unwrap_or(cssparser::Color::CurrentColor),
                inset: value.inset,
            }
        }

        pub fn parse_one_box_shadow(input: &mut Parser) -> Result<SpecifiedBoxShadow, ()> {
            use util::geometry::Au;
            let mut lengths = [specified::Length::Au(Au(0)); 4];
            let mut lengths_parsed = false;
            let mut color = None;
            let mut inset = false;

            loop {
                if !inset {
                    if input.try(|input| input.expect_ident_matching("inset")).is_ok() {
                        inset = true;
                        continue
                    }
                }
                if !lengths_parsed {
                    if let Ok(value) = input.try(specified::Length::parse) {
                        lengths[0] = value;
                        let mut length_parsed_count = 1;
                        while length_parsed_count < 4 {
                            if let Ok(value) = input.try(specified::Length::parse) {
                                lengths[length_parsed_count] = value
                            } else {
                                break
                            }
                            length_parsed_count += 1;
                        }

                        // The first two lengths must be specified.
                        if length_parsed_count < 2 {
                            return Err(())
                        }

                        lengths_parsed = true;
                        continue
                    }
                }
                if color.is_none() {
                    if let Ok(value) = input.try(specified::CSSColor::parse) {
                        color = Some(value);
                        continue
                    }
                }
                break
            }

            // Lengths must be specified.
            if !lengths_parsed {
                return Err(())
            }

            Ok(SpecifiedBoxShadow {
                offset_x: lengths[0],
                offset_y: lengths[1],
                blur_radius: lengths[2],
                spread_radius: lengths[3],
                color: color,
                inset: inset,
            })
        }
    </%self:longhand>

    <%self:longhand name="clip">
        use cssparser::ToCss;
        use text_writer::{self, TextWriter};

        // NB: `top` and `left` are 0 if `auto` per CSS 2.1 11.1.2.

        pub mod computed_value {
            use util::geometry::Au;

            #[derive(Clone, PartialEq, Eq, Copy, Debug)]
            pub struct ClipRect {
                pub top: Au,
                pub right: Option<Au>,
                pub bottom: Option<Au>,
                pub left: Au,
            }

            pub type T = Option<ClipRect>;
        }

        #[derive(Clone, Debug, PartialEq, Copy)]
        pub struct SpecifiedClipRect {
            pub top: specified::Length,
            pub right: Option<specified::Length>,
            pub bottom: Option<specified::Length>,
            pub left: specified::Length,
        }

        pub type SpecifiedValue = Option<SpecifiedClipRect>;

        impl ToCss for SpecifiedClipRect {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                try!(dest.write_str("rect("));

                try!(self.top.to_css(dest));
                try!(dest.write_str(", "));

                if let Some(right) = self.right {
                    try!(right.to_css(dest));
                    try!(dest.write_str(", "));
                } else {
                    try!(dest.write_str("auto, "));
                }

                if let Some(bottom) = self.right {
                    try!(bottom.to_css(dest));
                    try!(dest.write_str(", "));
                } else {
                    try!(dest.write_str("auto, "));
                }

                try!(self.left.to_css(dest));

                try!(dest.write_str(")"));
                Ok(())
            }
        }

        impl ToCss for Option<SpecifiedClipRect> {
            fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                if let Some(ref rect) = *self {
                    rect.to_css(dest)
                } else {
                    dest.write_str("auto")
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            None
        }

        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            value.map(|value| computed_value::ClipRect {
                top: computed::compute_Au(value.top, context),
                right: value.right.map(|right| computed::compute_Au(right, context)),
                bottom: value.bottom.map(|bottom| computed::compute_Au(bottom, context)),
                left: computed::compute_Au(value.left, context),
            })
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            use std::ascii::AsciiExt;
            use util::geometry::Au;
            use values::specified::Length;

            if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                return Ok(None)
            }
            if !try!(input.expect_function()).eq_ignore_ascii_case("rect") {
                return Err(())
            }
            let sides = try!(input.parse_nested_block(|input| {
                input.parse_comma_separated(|input| {
                    if input.try(|input| input.expect_ident_matching("auto")).is_ok() {
                        Ok(None)
                    } else {
                        Length::parse(input).map(Some)
                    }
                })
            }));
            if sides.len() == 4 {
                Ok(Some(SpecifiedClipRect {
                    top: sides[0].unwrap_or(Length::Au(Au(0))),
                    right: sides[1],
                    bottom: sides[2],
                    left: sides[3].unwrap_or(Length::Au(Au(0))),
                }))
            } else {
                Err(())
            }
        }
    </%self:longhand>

    <%self:longhand name="filter">
        use values::specified::Angle;
        pub use super::computed_as_specified as to_computed_value;
        pub use self::computed_value::T as SpecifiedValue;
        pub use self::computed_value::Filter;

        pub mod computed_value {
            use values::specified::Angle;
            use values::CSSFloat;
            use cssparser::ToCss;
            use text_writer::{self, TextWriter};

            // TODO(pcwalton): `blur`, `drop-shadow`
            #[derive(Clone, PartialEq, Debug)]
            pub enum Filter {
                Brightness(CSSFloat),
                Contrast(CSSFloat),
                Grayscale(CSSFloat),
                HueRotate(Angle),
                Invert(CSSFloat),
                Opacity(CSSFloat),
                Saturate(CSSFloat),
                Sepia(CSSFloat),
            }

            impl ToCss for Filter {
                fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                    match *self {
                        Filter::Brightness(value) => try!(write!(dest, "brightness({})", value)),
                        Filter::Contrast(value) => try!(write!(dest, "contrast({})", value)),
                        Filter::Grayscale(value) => try!(write!(dest, "grayscale({})", value)),
                        Filter::HueRotate(value) => {
                            try!(dest.write_str("hue-rotate("));
                            try!(value.to_css(dest));
                            try!(dest.write_str(")"));
                        }
                        Filter::Invert(value) => try!(write!(dest, "invert({})", value)),
                        Filter::Opacity(value) => try!(write!(dest, "opacity({})", value)),
                        Filter::Saturate(value) => try!(write!(dest, "saturate({})", value)),
                        Filter::Sepia(value) => try!(write!(dest, "sepia({})", value)),
                    }
                    Ok(())
                }
            }

            #[derive(Clone, PartialEq, Debug)]
            pub struct T {
                pub filters: Vec<Filter>,
            }

            impl ToCss for T {
                fn to_css<W>(&self, dest: &mut W) -> text_writer::Result where W: TextWriter {
                    let mut iter = self.filters.iter();
                    if let Some(filter) = iter.next() {
                        try!(filter.to_css(dest));
                    } else {
                        try!(dest.write_str("none"));
                        return Ok(())
                    }
                    for filter in iter {
                        try!(dest.write_str(" "));
                        try!(filter.to_css(dest));
                    }
                    Ok(())
                }
            }

            impl T {
                /// Creates a new filter pipeline.
                #[inline]
                pub fn new(filters: Vec<Filter>) -> T {
                    T {
                        filters: filters,
                    }
                }

                /// Adds a new filter to the filter pipeline.
                #[inline]
                pub fn push(&mut self, filter: Filter) {
                    self.filters.push(filter)
                }

                /// Returns true if this filter pipeline is empty and false otherwise.
                #[inline]
                pub fn is_empty(&self) -> bool {
                    self.filters.is_empty()
                }

                /// Returns the resulting opacity of this filter pipeline.
                #[inline]
                pub fn opacity(&self) -> CSSFloat {
                    let mut opacity = 1.0;
                    for filter in self.filters.iter() {
                        if let Filter::Opacity(ref opacity_value) = *filter {
                            opacity *= *opacity_value
                        }
                    }
                    opacity
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::new(Vec::new())
        }

        pub fn parse(_context: &ParserContext, input: &mut Parser) -> Result<SpecifiedValue, ()> {
            let mut filters = Vec::new();
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                return Ok(SpecifiedValue::new(filters))
            }
            loop {
                if let Ok(function_name) = input.try(|input| input.expect_function()) {
                    filters.push(try!(input.parse_nested_block(|input| {
                        match_ignore_ascii_case! { function_name,
                            "brightness" => parse_factor(input).map(Filter::Brightness),
                            "contrast" => parse_factor(input).map(Filter::Contrast),
                            "grayscale" => parse_factor(input).map(Filter::Grayscale),
                            "hue-rotate" => Angle::parse(input).map(Filter::HueRotate),
                            "invert" => parse_factor(input).map(Filter::Invert),
                            "opacity" => parse_factor(input).map(Filter::Opacity),
                            "saturate" => parse_factor(input).map(Filter::Saturate),
                            "sepia" => parse_factor(input).map(Filter::Sepia)
                            _ => Err(())
                        }
                    })));
                } else if filters.is_empty() {
                    return Err(())
                } else {
                    return Ok(SpecifiedValue::new(filters))
                }
            }
        }

        fn parse_factor(input: &mut Parser) -> Result<::values::CSSFloat, ()> {
            use cssparser::Token;
            match input.next() {
                Ok(Token::Number(value)) => Ok(value.value),
                Ok(Token::Percentage(value)) => Ok(value.unit_value),
                _ => Err(())
            }
        }
    </%self:longhand>

    ${single_keyword("mix-blend-mode",
                     """normal multiply screen overlay darken lighten color-dodge
                        color-burn hard-light soft-light difference exclusion hue
                        saturation color luminosity""")}
}


pub mod shorthands {
    use cssparser::Parser;
    use parser::ParserContext;
    use values::specified;

    <%def name="shorthand(name, sub_properties)">
    <%
        shorthand = Shorthand(name, sub_properties.split())
        SHORTHANDS.append(shorthand)
    %>
        pub mod ${shorthand.ident} {
            use cssparser::Parser;
            use parser::ParserContext;
            use properties::longhands;

            #[allow(missing_copy_implementations)]
            pub struct Longhands {
                % for sub_property in shorthand.sub_properties:
                    pub ${sub_property.ident}:
                        Option<longhands::${sub_property.ident}::SpecifiedValue>,
                % endfor
            }
            pub fn parse(context: &ParserContext, input: &mut Parser) -> Result<Longhands, ()> {
                ${caller.body()}
            }
        }
    </%def>

    <%def name="four_sides_shorthand(name, sub_property_pattern, parser_function)">
        <%self:shorthand name="${name}" sub_properties="${
                ' '.join(sub_property_pattern % side
                         for side in ['top', 'right', 'bottom', 'left'])}">
            use values::specified;
            let _unused = context;
            // zero or more than four values is invalid.
            // one value sets them all
            // two values set (top, bottom) and (left, right)
            // three values set top, (left, right) and bottom
            // four values set them in order
            let top = try!(${parser_function}(input));
            let right;
            let bottom;
            let left;
            match input.try(${parser_function}) {
                Err(()) => {
                    right = top.clone();
                    bottom = top.clone();
                    left = top.clone();
                }
                Ok(value) => {
                    right = value;
                    match input.try(${parser_function}) {
                        Err(()) => {
                            bottom = top.clone();
                            left = right.clone();
                        }
                        Ok(value) => {
                            bottom = value;
                            match input.try(${parser_function}) {
                                Err(()) => {
                                    left = right.clone();
                                }
                                Ok(value) => {
                                    left = value;
                                }
                            }

                        }
                    }

                }
            }
            Ok(Longhands {
                % for side in ["top", "right", "bottom", "left"]:
                    ${to_rust_ident(sub_property_pattern % side)}: Some(${side}),
                % endfor
            })
        </%self:shorthand>
    </%def>

    // TODO: other background-* properties
    <%self:shorthand name="background"
                     sub_properties="background-color background-position background-repeat background-attachment background-image">
        use properties::longhands::{background_color, background_position, background_repeat,
                                    background_attachment, background_image};

        let mut color = None;
        let mut image = None;
        let mut position = None;
        let mut repeat = None;
        let mut attachment = None;
        let mut any = false;

        loop {
            if position.is_none() {
                if let Ok(value) = input.try(|input| background_position::parse(context, input)) {
                    position = Some(value);
                    any = true;
                    continue
                }
            }
            if color.is_none() {
                if let Ok(value) = input.try(|input| background_color::parse(context, input)) {
                    color = Some(value);
                    any = true;
                    continue
                }
            }
            if image.is_none() {
                if let Ok(value) = input.try(|input| background_image::parse(context, input)) {
                    image = Some(value);
                    any = true;
                    continue
                }
            }
            if repeat.is_none() {
                if let Ok(value) = input.try(|input| background_repeat::parse(context, input)) {
                    repeat = Some(value);
                    any = true;
                    continue
                }
            }
            if attachment.is_none() {
                if let Ok(value) = input.try(|input| background_attachment::parse(context, input)) {
                    attachment = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }

        if any {
            Ok(Longhands {
                background_color: color,
                background_image: image,
                background_position: position,
                background_repeat: repeat,
                background_attachment: attachment,
            })
        } else {
            Err(())
        }
    </%self:shorthand>

    ${four_sides_shorthand("margin", "margin-%s", "specified::LengthOrPercentageOrAuto::parse")}
    ${four_sides_shorthand("padding", "padding-%s", "specified::LengthOrPercentage::parse")}

    ${four_sides_shorthand("border-color", "border-%s-color", "specified::CSSColor::parse")}
    ${four_sides_shorthand("border-style", "border-%s-style",
                           "specified::BorderStyle::parse")}
    ${four_sides_shorthand("border-width", "border-%s-width",
                           "specified::parse_border_width")}

    pub fn parse_border(context: &ParserContext, input: &mut Parser)
                     -> Result<(Option<specified::CSSColor>,
                                Option<specified::BorderStyle>,
                                Option<specified::Length>), ()> {
        use values::specified;
        let _unused = context;
        let mut color = None;
        let mut style = None;
        let mut width = None;
        let mut any = false;
        loop {
            if color.is_none() {
                if let Ok(value) = input.try(specified::CSSColor::parse) {
                    color = Some(value);
                    any = true;
                    continue
                }
            }
            if style.is_none() {
                if let Ok(value) = input.try(specified::BorderStyle::parse) {
                    style = Some(value);
                    any = true;
                    continue
                }
            }
            if width.is_none() {
                if let Ok(value) = input.try(specified::parse_border_width) {
                    width = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }
        if any { Ok((color, style, width)) } else { Err(()) }
    }


    % for side in ["top", "right", "bottom", "left"]:
        <%self:shorthand name="border-${side}" sub_properties="${' '.join(
            'border-%s-%s' % (side, prop)
            for prop in ['color', 'style', 'width']
        )}">
            let (color, style, width) = try!(super::parse_border(context, input));
            Ok(Longhands {
                % for prop in ["color", "style", "width"]:
                    ${"border_%s_%s: %s," % (side, prop, prop)}
                % endfor
            })
        </%self:shorthand>
    % endfor

    <%self:shorthand name="border" sub_properties="${' '.join(
        'border-%s-%s' % (side, prop)
        for side in ['top', 'right', 'bottom', 'left']
        for prop in ['color', 'style', 'width']
    )}">
        let (color, style, width) = try!(super::parse_border(context, input));
        Ok(Longhands {
            % for side in ["top", "right", "bottom", "left"]:
                % for prop in ["color", "style", "width"]:
                    ${"border_%s_%s: %s.clone()," % (side, prop, prop)}
                % endfor
            % endfor
        })
    </%self:shorthand>

    <%self:shorthand name="border-radius" sub_properties="${' '.join(
        'border-%s-radius' % (corner)
         for corner in ['top-left', 'top-right', 'bottom-right', 'bottom-left']
    )}">
        use util::geometry::Au;
        use values::specified::{Length, LengthOrPercentage};
        let _ignored = context;

        fn parse_one_set_of_border_radii(mut input: &mut Parser)
                                         -> Result<[LengthOrPercentage; 4], ()> {
            let mut count = 0;
            let mut values = [LengthOrPercentage::Length(Length::Au(Au(0))); 4];
            while count < 4 {
                if let Ok(value) = input.try(LengthOrPercentage::parse) {
                    values[count] = value;
                    count += 1;
                } else {
                    break
                }
            }

            match count {
                1 => Ok([values[0], values[0], values[0], values[0]]),
                2 => Ok([values[0], values[1], values[0], values[1]]),
                3 => Ok([values[0], values[1], values[2], values[1]]),
                4 => Ok([values[0], values[1], values[2], values[3]]),
                _ => Err(()),
            }
        }

        let radii = try!(parse_one_set_of_border_radii(input));
        // TODO(pcwalton): Elliptical borders.

        Ok(Longhands {
            border_top_left_radius: Some(radii[0]),
            border_top_right_radius: Some(radii[1]),
            border_bottom_right_radius: Some(radii[2]),
            border_bottom_left_radius: Some(radii[3]),
        })
    </%self:shorthand>

    <%self:shorthand name="outline" sub_properties="outline-color outline-style outline-width">
        use values::specified;

        let _unused = context;
        let mut color = None;
        let mut style = None;
        let mut width = None;
        let mut any = false;
        loop {
            if color.is_none() {
                if let Ok(value) = input.try(specified::CSSColor::parse) {
                    color = Some(value);
                    any = true;
                    continue
                }
            }
            if style.is_none() {
                if let Ok(value) = input.try(specified::BorderStyle::parse) {
                    style = Some(value);
                    any = true;
                    continue
                }
            }
            if width.is_none() {
                if let Ok(value) = input.try(specified::parse_border_width) {
                    width = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }
        if any {
            Ok(Longhands {
                outline_color: color,
                outline_style: style,
                outline_width: width,
            })
        } else {
            Err(())
        }
    </%self:shorthand>

    <%self:shorthand name="font" sub_properties="font-style font-variant font-weight
                                                 font-size line-height font-family">
        use properties::longhands::{font_style, font_variant, font_weight, font_size,
                                    line_height, font_family};
        let mut nb_normals = 0;
        let mut style = None;
        let mut variant = None;
        let mut weight = None;
        let size;
        loop {
            // Special-case 'normal' because it is valid in each of
            // font-style, font-weight and font-variant.
            // Leaves the values to None, 'normal' is the initial value for each of them.
            if input.try(|input| input.expect_ident_matching("normal")).is_ok() {
                nb_normals += 1;
                continue;
            }
            if style.is_none() {
                if let Ok(value) = input.try(|input| font_style::parse(context, input)) {
                    style = Some(value);
                    continue
                }
            }
            if weight.is_none() {
                if let Ok(value) = input.try(|input| font_weight::parse(context, input)) {
                    weight = Some(value);
                    continue
                }
            }
            if variant.is_none() {
                if let Ok(value) = input.try(|input| font_variant::parse(context, input)) {
                    variant = Some(value);
                    continue
                }
            }
            size = Some(try!(font_size::parse(context, input)));
            break
        }
        #[inline]
        fn count<T>(opt: &Option<T>) -> u8 {
            if opt.is_some() { 1 } else { 0 }
        }
        if size.is_none() || (count(&style) + count(&weight) + count(&variant) + nb_normals) > 3 {
            return Err(())
        }
        let line_height = if input.try(|input| input.expect_delim('/')).is_ok() {
            Some(try!(line_height::parse(context, input)))
        } else {
            None
        };
        let family = try!(input.parse_comma_separated(font_family::parse_one_family));
        Ok(Longhands {
            font_style: style,
            font_variant: variant,
            font_weight: weight,
            font_size: size,
            line_height: line_height,
            font_family: Some(family)
        })
    </%self:shorthand>

    // Per CSS-TEXT 6.2, "for legacy reasons, UAs must treat `word-wrap` as an alternate name for
    // the `overflow-wrap` property, as if it were a shorthand of `overflow-wrap`."
    <%self:shorthand name="word-wrap" sub_properties="overflow-wrap">
        use properties::longhands::overflow_wrap;
        Ok(Longhands {
            overflow_wrap: Some(try!(overflow_wrap::parse(context, input))),
        })
    </%self:shorthand>

    <%self:shorthand name="list-style"
                     sub_properties="list-style-image list-style-position list-style-type">
        use properties::longhands::{list_style_image, list_style_position, list_style_type};

        // `none` is ambiguous until we've finished parsing the shorthands, so we count the number
        // of times we see it.
        let mut nones = 0u8;
        let (mut image, mut position, mut list_style_type, mut any) = (None, None, None, false);
        loop {
            if input.try(|input| input.expect_ident_matching("none")).is_ok() {
                nones = nones + 1;
                if nones > 2 {
                    return Err(())
                }
                any = true;
                continue
            }

            if list_style_type.is_none() {
                if let Ok(value) = input.try(|input| list_style_type::parse(context, input)) {
                    list_style_type = Some(value);
                    any = true;
                    continue
                }
            }

            if image.is_none() {
                if let Ok(value) = input.try(|input| list_style_image::parse(context, input)) {
                    image = Some(value);
                    any = true;
                    continue
                }
            }

            if position.is_none() {
                if let Ok(value) = input.try(|input| list_style_position::parse(context, input)) {
                    position = Some(value);
                    any = true;
                    continue
                }
            }
            break
        }

        // If there are two `none`s, then we can't have a type or image; if there is one `none`,
        // then we can't have both a type *and* an image; if there is no `none` then we're fine as
        // long as we parsed something.
        match (any, nones, list_style_type, image) {
            (true, 2, None, None) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(list_style_image::SpecifiedValue::None),
                    list_style_type: Some(list_style_type::SpecifiedValue::none),
                })
            }
            (true, 1, None, Some(image)) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(image),
                    list_style_type: Some(list_style_type::SpecifiedValue::none),
                })
            }
            (true, 1, Some(list_style_type), None) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(list_style_image::SpecifiedValue::None),
                    list_style_type: Some(list_style_type),
                })
            }
            (true, 1, None, None) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(list_style_image::SpecifiedValue::None),
                    list_style_type: Some(list_style_type::SpecifiedValue::none),
                })
            }
            (true, 0, list_style_type, image) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: image,
                    list_style_type: list_style_type,
                })
            }
            _ => Err(()),
        }
    </%self:shorthand>
}


// TODO(SimonSapin): Convert this to a syntax extension rather than a Mako template.
// Maybe submit for inclusion in libstd?
mod property_bit_field {
    use std::uint;
    use std::mem;

    pub struct PropertyBitField {
        storage: [uint; (${len(LONGHANDS)} - 1 + uint::BITS) / uint::BITS]
    }

    impl PropertyBitField {
        #[inline]
        pub fn new() -> PropertyBitField {
            PropertyBitField { storage: unsafe { mem::zeroed() } }
        }

        #[inline]
        fn get(&self, bit: uint) -> bool {
            (self.storage[bit / uint::BITS] & (1 << (bit % uint::BITS))) != 0
        }
        #[inline]
        fn set(&mut self, bit: uint) {
            self.storage[bit / uint::BITS] |= 1 << (bit % uint::BITS)
        }
        % for i, property in enumerate(LONGHANDS):
            % if property.derived_from is None:
                #[allow(non_snake_case)]
                #[inline]
                pub fn get_${property.ident}(&self) -> bool {
                    self.get(${i})
                }
                #[allow(non_snake_case)]
                #[inline]
                pub fn set_${property.ident}(&mut self) {
                    self.set(${i})
                }
            % endif
        % endfor
    }
}


/// Declarations are stored in reverse order.
/// Overridden declarations are skipped.
#[derive(Debug, PartialEq)]
pub struct PropertyDeclarationBlock {
    pub important: Arc<Vec<PropertyDeclaration>>,
    pub normal: Arc<Vec<PropertyDeclaration>>,
}


pub fn parse_style_attribute(input: &str, base_url: &Url) -> PropertyDeclarationBlock {
    let context = ParserContext::new(Origin::Author, base_url);
    parse_property_declaration_list(&context, &mut Parser::new(input))
}


struct PropertyDeclarationParser<'a, 'b: 'a> {
    context: &'a ParserContext<'b>,
}


/// Default methods reject all at rules.
impl<'a, 'b> AtRuleParser for PropertyDeclarationParser<'a, 'b> {
    type Prelude = ();
    type AtRule = (Vec<PropertyDeclaration>, bool);
}


impl<'a, 'b> DeclarationParser for PropertyDeclarationParser<'a, 'b> {
    type Declaration = (Vec<PropertyDeclaration>, bool);

    fn parse_value(&self, name: &str, input: &mut Parser) -> Result<(Vec<PropertyDeclaration>, bool), ()> {
        let mut results = vec![];
        match PropertyDeclaration::parse(name, self.context, input, &mut results) {
            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => {}
            _ => return Err(())
        }
        let important = input.try(parse_important).is_ok();
        Ok((results, important))
    }
}


pub fn parse_property_declaration_list(context: &ParserContext, input: &mut Parser)
                                       -> PropertyDeclarationBlock {
    let mut important_declarations = Vec::new();
    let mut normal_declarations = Vec::new();
    let parser = PropertyDeclarationParser {
        context: context,
    };
    let mut iter = DeclarationListParser::new(input, parser);
    while let Some(declaration) = iter.next() {
        match declaration {
            Ok((results, important)) => {
                if important {
                    important_declarations.push_all(&results);
                } else {
                    normal_declarations.push_all(&results);
                }
            }
            Err(range) => {
                let pos = range.start;
                let message = format!("Unsupported property declaration: '{}'",
                                      iter.input.slice(range));
                log_css_error(iter.input, pos, &*message);
            }
        }
    }
    PropertyDeclarationBlock {
        important: Arc::new(deduplicate_property_declarations(important_declarations)),
        normal: Arc::new(deduplicate_property_declarations(normal_declarations)),
    }
}


/// Only keep the last declaration for any given property.
/// The input is in source order, output in reverse source order.
fn deduplicate_property_declarations(declarations: Vec<PropertyDeclaration>)
                                     -> Vec<PropertyDeclaration> {
    let mut deduplicated = vec![];
    let mut seen = PropertyBitField::new();
    for declaration in declarations.into_iter().rev() {
        match declaration {
            % for property in LONGHANDS:
                PropertyDeclaration::${property.camel_case}(..) => {
                    % if property.derived_from is None:
                        if seen.get_${property.ident}() {
                            continue
                        }
                        seen.set_${property.ident}()
                    % else:
                        unreachable!();
                    % endif
                },
            % endfor
        }
        deduplicated.push(declaration)
    }
    deduplicated
}


#[derive(Copy, PartialEq, Eq, Debug)]
pub enum CSSWideKeyword {
    InitialKeyword,
    InheritKeyword,
    UnsetKeyword,
}

impl CSSWideKeyword {
    pub fn parse(input: &mut Parser) -> Result<CSSWideKeyword, ()> {
        match_ignore_ascii_case! { try!(input.expect_ident()),
            "initial" => Ok(CSSWideKeyword::InitialKeyword),
            "inherit" => Ok(CSSWideKeyword::InheritKeyword),
            "unset" => Ok(CSSWideKeyword::UnsetKeyword)
            _ => Err(())
        }
    }
}


#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum DeclaredValue<T> {
    SpecifiedValue(T),
    Initial,
    Inherit,
    // There is no Unset variant here.
    // The 'unset' keyword is represented as either Initial or Inherit,
    // depending on whether the property is inherited.
}

impl<T: ToCss> DeclaredValue<T> {
    pub fn specified_value(&self) -> String {
        match self {
            &DeclaredValue::SpecifiedValue(ref inner) => inner.to_css_string(),
            &DeclaredValue::Initial => "initial".to_owned(),
            &DeclaredValue::Inherit => "inherit".to_owned(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum PropertyDeclaration {
    % for property in LONGHANDS:
        ${property.camel_case}(DeclaredValue<longhands::${property.ident}::SpecifiedValue>),
    % endfor
}


#[derive(Eq, PartialEq, Copy)]
pub enum PropertyDeclarationParseResult {
    UnknownProperty,
    ExperimentalProperty,
    InvalidValue,
    ValidOrIgnoredDeclaration,
}

impl PropertyDeclaration {
    pub fn name(&self) -> &'static str {
        match self {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    &PropertyDeclaration::${property.camel_case}(..) => "${property.name}",
                % endif
            % endfor
            _ => "",
        }
    }

    pub fn value(&self) -> String {
        match self {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    &PropertyDeclaration::${property.camel_case}(ref value) =>
                        value.specified_value(),
                % endif
            % endfor
            decl => panic!("unsupported property declaration: {:?}", decl.name()),
        }
    }

    pub fn matches(&self, name: &str) -> bool {
        match *self {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    PropertyDeclaration::${property.camel_case}(..) => {
                        name.eq_ignore_ascii_case("${property.name}")
                    }
                % endif
            % endfor
            _ => false,
        }
    }

    pub fn parse(name: &str, context: &ParserContext, input: &mut Parser,
                 result_list: &mut Vec<PropertyDeclaration>) -> PropertyDeclarationParseResult {
        match_ignore_ascii_case! { name,
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    "${property.name}" => {
                        % if property.experimental:
                            if !::util::opts::experimental_enabled() {
                                return PropertyDeclarationParseResult::ExperimentalProperty
                            }
                        % endif
                        match longhands::${property.ident}::parse_declared(context, input) {
                            Ok(value) => {
                                result_list.push(PropertyDeclaration::${property.camel_case}(value));
                                PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                            },
                            Err(()) => PropertyDeclarationParseResult::InvalidValue,
                        }
                    },
                % else:
                    "${property.name}" => PropertyDeclarationParseResult::UnknownProperty,
                % endif
            % endfor
            % for shorthand in SHORTHANDS:
                "${shorthand.name}" => {
                    match input.try(CSSWideKeyword::parse) {
                        Ok(CSSWideKeyword::InheritKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(
                                    PropertyDeclaration::${sub_property.camel_case}(
                                        DeclaredValue::Inherit));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Ok(CSSWideKeyword::InitialKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(
                                    PropertyDeclaration::${sub_property.camel_case}(
                                        DeclaredValue::Initial));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Ok(CSSWideKeyword::UnsetKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                result_list.push(PropertyDeclaration::${sub_property.camel_case}(
                                    DeclaredValue::${"Inherit" if sub_property.style_struct.inherited else "Initial"}
                                ));
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Err(()) => match shorthands::${shorthand.ident}::parse(context, input) {
                            Ok(result) => {
                                % for sub_property in shorthand.sub_properties:
                                    result_list.push(PropertyDeclaration::${sub_property.camel_case}(
                                        match result.${sub_property.ident} {
                                            Some(value) => DeclaredValue::SpecifiedValue(value),
                                            None => DeclaredValue::Initial,
                                        }
                                    ));
                                % endfor
                                PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                            },
                            Err(()) => PropertyDeclarationParseResult::InvalidValue,
                        }
                    }
                },
            % endfor

            // Hack to work around quirks of macro_rules parsing in match_ignore_ascii_case!
            "_nonexistent" => PropertyDeclarationParseResult::UnknownProperty

            _ => PropertyDeclarationParseResult::UnknownProperty
        }
    }
}

impl Debug for PropertyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name(), self.value())
    }
}


pub mod style_structs {
    use super::longhands;

    % for style_struct in STYLE_STRUCTS:
        #[allow(missing_copy_implementations)]
        #[derive(PartialEq, Clone)]
        pub struct ${style_struct.name} {
            % for longhand in style_struct.longhands:
                pub ${longhand.ident}: longhands::${longhand.ident}::computed_value::T,
            % endfor
        }
    % endfor
}

#[derive(Clone)]
pub struct ComputedValues {
    % for style_struct in STYLE_STRUCTS:
        ${style_struct.ident}: Arc<style_structs::${style_struct.name}>,
    % endfor
    shareable: bool,
    pub writing_mode: WritingMode,
    pub root_font_size: Au,
}

impl ComputedValues {
    /// Resolves the currentColor keyword.
    /// Any color value form computed values (except for the 'color' property itself)
    /// should go through this method.
    ///
    /// Usage example:
    /// let top_color = style.resolve_color(style.Border.border_top_color);
    #[inline]
    pub fn resolve_color(&self, color: Color) -> RGBA {
        match color {
            Color::RGBA(rgba) => rgba,
            Color::CurrentColor => self.get_color().color,
        }
    }

    #[inline]
    pub fn content_inline_size(&self) -> computed::LengthOrPercentageOrAuto {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.height } else { box_style.width }
    }

    #[inline]
    pub fn content_block_size(&self) -> computed::LengthOrPercentageOrAuto {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.width } else { box_style.height }
    }

    #[inline]
    pub fn min_inline_size(&self) -> computed::LengthOrPercentage {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.min_height } else { box_style.min_width }
    }

    #[inline]
    pub fn min_block_size(&self) -> computed::LengthOrPercentage {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.min_width } else { box_style.min_height }
    }

    #[inline]
    pub fn max_inline_size(&self) -> computed::LengthOrPercentageOrNone {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.max_height } else { box_style.max_width }
    }

    #[inline]
    pub fn max_block_size(&self) -> computed::LengthOrPercentageOrNone {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.max_width } else { box_style.max_height }
    }

    #[inline]
    pub fn logical_padding(&self) -> LogicalMargin<computed::LengthOrPercentage> {
        let padding_style = self.get_padding();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            padding_style.padding_top,
            padding_style.padding_right,
            padding_style.padding_bottom,
            padding_style.padding_left,
        ))
    }

    #[inline]
    pub fn logical_border_width(&self) -> LogicalMargin<Au> {
        let border_style = self.get_border();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            border_style.border_top_width,
            border_style.border_right_width,
            border_style.border_bottom_width,
            border_style.border_left_width,
        ))
    }

    #[inline]
    pub fn logical_margin(&self) -> LogicalMargin<computed::LengthOrPercentageOrAuto> {
        let margin_style = self.get_margin();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            margin_style.margin_top,
            margin_style.margin_right,
            margin_style.margin_bottom,
            margin_style.margin_left,
        ))
    }

    #[inline]
    pub fn logical_position(&self) -> LogicalMargin<computed::LengthOrPercentageOrAuto> {
        // FIXME(SimonSapin): should be the writing mode of the containing block, maybe?
        let position_style = self.get_positionoffsets();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            position_style.top,
            position_style.right,
            position_style.bottom,
            position_style.left,
        ))
    }

    #[inline]
    pub fn get_font_arc(&self) -> Arc<style_structs::Font> {
        self.font.clone()
    }

    % for style_struct in STYLE_STRUCTS:
        #[inline]
        pub fn get_${style_struct.name.lower()}
                <'a>(&'a self) -> &'a style_structs::${style_struct.name} {
            &*self.${style_struct.ident}
        }
    % endfor
}


/// Return a WritingMode bitflags from the relevant CSS properties.
fn get_writing_mode(inheritedbox_style: &style_structs::InheritedBox) -> WritingMode {
    use util::logical_geometry;
    let mut flags = WritingMode::empty();
    match inheritedbox_style.direction {
        computed_values::direction::T::ltr => {},
        computed_values::direction::T::rtl => {
            flags.insert(logical_geometry::FLAG_RTL);
        },
    }
    match inheritedbox_style.writing_mode {
        computed_values::writing_mode::T::horizontal_tb => {},
        computed_values::writing_mode::T::vertical_rl => {
            flags.insert(logical_geometry::FLAG_VERTICAL);
        },
        computed_values::writing_mode::T::vertical_lr => {
            flags.insert(logical_geometry::FLAG_VERTICAL);
            flags.insert(logical_geometry::FLAG_VERTICAL_LR);
        },
    }
    match inheritedbox_style.text_orientation {
        computed_values::text_orientation::T::sideways_right => {},
        computed_values::text_orientation::T::sideways_left => {
            flags.insert(logical_geometry::FLAG_VERTICAL_LR);
        },
        computed_values::text_orientation::T::sideways => {
            if flags.intersects(logical_geometry::FLAG_VERTICAL_LR) {
                flags.insert(logical_geometry::FLAG_SIDEWAYS_LEFT);
            }
        },
    }
    flags
}


/// The initial values for all style structs as defined by the specification.
lazy_static! {
    static ref INITIAL_VALUES: ComputedValues = ComputedValues {
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.ident}: Arc::new(style_structs::${style_struct.name} {
                % for longhand in style_struct.longhands:
                    ${longhand.ident}: longhands::${longhand.ident}::get_initial_value(),
                % endfor
            }),
        % endfor
        shareable: true,
        writing_mode: WritingMode::empty(),
        root_font_size: longhands::font_size::get_initial_value(),
    };
}


#[test]
fn initial_writing_mode_is_empty() {
    assert_eq!(get_writing_mode(INITIAL_VALUES.get_inheritedbox()), WritingMode::empty())
}

/// Fast path for the function below. Only computes new inherited styles.
#[allow(unused_mut)]
fn cascade_with_cached_declarations(applicable_declarations: &[DeclarationBlock],
                                    shareable: bool,
                                    parent_style: &ComputedValues,
                                    cached_style: &ComputedValues,
                                    context: &computed::Context)
                                    -> ComputedValues {
    % for style_struct in STYLE_STRUCTS:
        % if style_struct.inherited:
            let mut style_${style_struct.ident} = parent_style.${style_struct.ident}.clone();
        % else:
            let mut style_${style_struct.ident} = cached_style.${style_struct.ident}.clone();
        % endif
    % endfor

    let mut seen = PropertyBitField::new();
    // Declaration blocks are stored in increasing precedence order,
    // we want them in decreasing order here.
    for sub_list in applicable_declarations.iter().rev() {
        // Declarations are already stored in reverse order.
        for declaration in sub_list.declarations.iter() {
            match *declaration {
                % for style_struct in STYLE_STRUCTS:
                    % for property in style_struct.longhands:
                        % if property.derived_from is None:
                            PropertyDeclaration::${property.camel_case}(ref ${'_' if not style_struct.inherited else ''}declared_value) => {
                                % if style_struct.inherited:
                                    if seen.get_${property.ident}() {
                                        continue
                                    }
                                    seen.set_${property.ident}();
                                    let computed_value = match *declared_value {
                                        DeclaredValue::SpecifiedValue(ref specified_value)
                                        => longhands::${property.ident}::to_computed_value(
                                            (*specified_value).clone(),
                                            context
                                        ),
                                        DeclaredValue::Initial
                                        => longhands::${property.ident}::get_initial_value(),
                                        DeclaredValue::Inherit => {
                                            // This is a bit slow, but this is rare so it shouldn't
                                            // matter.
                                            //
                                            // FIXME: is it still?
                                            parent_style.${style_struct.ident}
                                                        .${property.ident}
                                                        .clone()
                                        }
                                    };
                                    style_${style_struct.ident}.make_unique()
                                        .${property.ident} = computed_value;
                                % endif

                                % if property.name in DERIVED_LONGHANDS:
                                    % if not style_struct.inherited:
                                        // Use the cached value.
                                        let computed_value = style_${style_struct.ident}
                                            .${property.ident}.clone();
                                    % endif
                                    % for derived in DERIVED_LONGHANDS[property.name]:
                                        style_${derived.style_struct.ident}
                                            .make_unique()
                                            .${derived.ident} =
                                            longhands::${derived.ident}
                                                     ::derive_from_${property.ident}(
                                                         computed_value,
                                                         context);
                                    % endfor
                                % endif
                            }
                        % else:
                            PropertyDeclaration::${property.camel_case}(_) => {
                                // Do not allow stylesheets to set derived properties.
                            }
                        % endif
                    % endfor
                % endfor
            }
        }
    }

    ComputedValues {
        writing_mode: get_writing_mode(&*style_inheritedbox),
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.ident}: style_${style_struct.ident},
        % endfor
        shareable: shareable,
        root_font_size: parent_style.root_font_size,
    }
}

/// Performs the CSS cascade, computing new styles for an element from its parent style and
/// optionally a cached related style. The arguments are:
///
///   * `applicable_declarations`: The list of CSS rules that matched.
///
///   * `shareable`: Whether the `ComputedValues` structure to be constructed should be considered
///     shareable.
///
///   * `parent_style`: The parent style, if applicable; if `None`, this is the root node.
///
///   * `cached_style`: If present, cascading is short-circuited for everything but inherited
///     values and these values are used instead. Obviously, you must be careful when supplying
///     this that it is safe to only provide inherited declarations. If `parent_style` is `None`,
///     this is ignored.
///
/// Returns the computed values and a boolean indicating whether the result is cacheable.
pub fn cascade(applicable_declarations: &[DeclarationBlock],
               shareable: bool,
               parent_style: Option< &ComputedValues >,
               cached_style: Option< &ComputedValues >)
               -> (ComputedValues, bool) {
    let initial_values = &*INITIAL_VALUES;
    let (is_root_element, inherited_style) = match parent_style {
        Some(parent_style) => (false, parent_style),
        None => (true, initial_values),
    };

    let mut context = {
        let inherited_font_style = inherited_style.get_font();
        computed::Context {
            is_root_element: is_root_element,
            inherited_font_weight: inherited_font_style.font_weight,
            inherited_font_size: inherited_font_style.font_size,
            inherited_height: inherited_style.get_box().height,
            inherited_text_decorations_in_effect:
                inherited_style.get_inheritedtext()._servo_text_decorations_in_effect,
            // To be overridden by applicable declarations:
            font_size: inherited_font_style.font_size,
            root_font_size: inherited_style.root_font_size,
            display: longhands::display::get_initial_value(),
            color: inherited_style.get_color().color,
            text_decoration: longhands::text_decoration::get_initial_value(),
            positioned: false,
            floated: false,
            border_top_present: false,
            border_right_present: false,
            border_bottom_present: false,
            border_left_present: false,
        }
    };

    // This assumes that the computed and specified values have the same Rust type.
    macro_rules! get_specified(
        ($style_struct_getter: ident, $property: ident, $declared_value: expr) => {
            match *$declared_value {
                DeclaredValue::SpecifiedValue(specified_value) => specified_value,
                DeclaredValue::Initial => longhands::$property::get_initial_value(),
                DeclaredValue::Inherit => inherited_style.$style_struct_getter().$property.clone(),
            }
        };
    );

    // Initialize `context`
    // Declarations blocks are already stored in increasing precedence order.
    for sub_list in applicable_declarations.iter() {
        // Declarations are stored in reverse source order, we want them in forward order here.
        for declaration in sub_list.declarations.iter().rev() {
            match *declaration {
                PropertyDeclaration::FontSize(ref value) => {
                    context.font_size = match *value {
                        DeclaredValue::SpecifiedValue(specified_value) => computed::compute_Au_with_font_size(
                            specified_value, context.inherited_font_size, context.root_font_size),
                        DeclaredValue::Initial => longhands::font_size::get_initial_value(),
                        DeclaredValue::Inherit => context.inherited_font_size,
                    }
                }
                PropertyDeclaration::Color(ref value) => {
                    context.color = match *value {
                        DeclaredValue::SpecifiedValue(ref specified_value) => specified_value.parsed,
                        DeclaredValue::Initial => longhands::color::get_initial_value(),
                        DeclaredValue::Inherit => inherited_style.get_color().color.clone(),
                    };
                }
                PropertyDeclaration::Display(ref value) => {
                    context.display = get_specified!(get_box, display, value);
                }
                PropertyDeclaration::Position(ref value) => {
                    context.positioned = match get_specified!(get_box, position, value) {
                        longhands::position::SpecifiedValue::absolute |
                        longhands::position::SpecifiedValue::fixed => true,
                        _ => false,
                    }
                }
                PropertyDeclaration::Float(ref value) => {
                    context.floated = get_specified!(get_box, float, value)
                                      != longhands::float::SpecifiedValue::none;
                }
                PropertyDeclaration::TextDecoration(ref value) => {
                    context.text_decoration = get_specified!(get_text, text_decoration, value);
                }
                % for side in ["top", "right", "bottom", "left"]:
                    PropertyDeclaration::Border${side.capitalize()}Style(ref value) => {
                        context.border_${side}_present =
                        match get_specified!(get_border, border_${side}_style, value) {
                            BorderStyle::none | BorderStyle::hidden => false,
                            _ => true,
                        };
                    }
                % endfor
                _ => {}
            }
        }
    }

    match (cached_style, parent_style) {
        (Some(cached_style), Some(parent_style)) => {
            return (cascade_with_cached_declarations(applicable_declarations,
                                                     shareable,
                                                     parent_style,
                                                     cached_style,
                                                     &context), false)
        }
        (_, _) => {}
    }

    // Set computed values, overwriting earlier declarations for the same property.
    % for style_struct in STYLE_STRUCTS:
        let mut style_${style_struct.ident} =
            % if style_struct.inherited:
                inherited_style
            % else:
                initial_values
            % endif
            .${style_struct.ident}.clone();
    % endfor
    let mut cacheable = true;
    let mut seen = PropertyBitField::new();
    // Declaration blocks are stored in increasing precedence order,
    // we want them in decreasing order here.
    for sub_list in applicable_declarations.iter().rev() {
        // Declarations are already stored in reverse order.
        for declaration in sub_list.declarations.iter() {
            match *declaration {
                % for style_struct in STYLE_STRUCTS:
                    % for property in style_struct.longhands:
                        % if property.derived_from is None:
                            PropertyDeclaration::${property.camel_case}(ref declared_value) => {
                                if seen.get_${property.ident}() {
                                    continue
                                }
                                seen.set_${property.ident}();
                                let computed_value = match *declared_value {
                                    DeclaredValue::SpecifiedValue(ref specified_value)
                                    => longhands::${property.ident}::to_computed_value(
                                        (*specified_value).clone(),
                                        &context
                                    ),
                                    DeclaredValue::Initial
                                    => longhands::${property.ident}::get_initial_value(),
                                    DeclaredValue::Inherit => {
                                        // This is a bit slow, but this is rare so it shouldn't
                                        // matter.
                                        //
                                        // FIXME: is it still?
                                        cacheable = false;
                                        inherited_style.${style_struct.ident}
                                                       .${property.ident}
                                                       .clone()
                                    }
                                };
                                style_${style_struct.ident}.make_unique()
                                    .${property.ident} = computed_value;

                                % if property.name in DERIVED_LONGHANDS:
                                    % for derived in DERIVED_LONGHANDS[property.name]:
                                        style_${derived.style_struct.ident}
                                            .make_unique()
                                            .${derived.ident} =
                                            longhands::${derived.ident}
                                                     ::derive_from_${property.ident}(
                                                         computed_value,
                                                         &context);
                                    % endfor
                                % endif
                            }
                        % else:
                            PropertyDeclaration::${property.camel_case}(_) => {
                                // Do not allow stylesheets to set derived properties.
                            }
                        % endif
                    % endfor
                % endfor
            }
        }
    }

    // The initial value of border-*-width may be changed at computed value time.
    {
        let border = style_border.make_unique();
        % for side in ["top", "right", "bottom", "left"]:
            // Like calling to_computed_value, which wouldn't type check.
            if !context.border_${side}_present {
                border.border_${side}_width = Au(0);
            }
        % endfor
    }

    // The initial value of display may be changed at computed value time.
    if !seen.get_display() {
        let box_ = style_box_.make_unique();
        box_.display = longhands::display::to_computed_value(box_.display, &context);
    }

    if is_root_element {
        context.root_font_size = context.font_size;
    }

    (ComputedValues {
        writing_mode: get_writing_mode(&*style_inheritedbox),
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.ident}: style_${style_struct.ident},
        % endfor
        shareable: shareable,
        root_font_size: context.root_font_size,
    }, cacheable)
}


/// Equivalent to `cascade()` with an empty `applicable_declarations`
/// Performs the CSS cascade for an anonymous box.
///
///   * `parent_style`: Computed style of the element this anonymous box inherits from.
pub fn cascade_anonymous(parent_style: &ComputedValues) -> ComputedValues {
    let initial_values = &*INITIAL_VALUES;
    let mut result = ComputedValues {
        % for style_struct in STYLE_STRUCTS:
            ${style_struct.ident}:
                % if style_struct.inherited:
                    parent_style
                % else:
                    initial_values
                % endif
                .${style_struct.ident}.clone(),
        % endfor
        shareable: false,
        writing_mode: parent_style.writing_mode,
        root_font_size: parent_style.root_font_size,
    };
    {
        let border = result.border.make_unique();
        % for side in ["top", "right", "bottom", "left"]:
            // Like calling to_computed_value, which wouldn't type check.
            border.border_${side}_width = Au(0);
        % endfor
    }
    // None of the teaks on 'display' apply here.
    result
}

/// Sets `display` to `inline` and `position` to `static`.
#[inline]
pub fn make_inline(style: &ComputedValues) -> ComputedValues {
    let mut style = (*style).clone();
    style.box_.make_unique().display = longhands::display::computed_value::T::inline;
    style.box_.make_unique().position = longhands::position::computed_value::T::static_;
    style
}

pub fn is_supported_property(property: &str) -> bool {
    match property {
        % for property in SHORTHANDS + LONGHANDS:
            "${property.name}" => true,
        % endfor
        _ => false,
    }
}

#[macro_export]
macro_rules! css_properties_accessors {
    ($macro_name: ident) => {
        $macro_name! {
            % for property in SHORTHANDS + LONGHANDS:
                ## Servo internal CSS properties are not accessible.
                ## FIXME: Add BinaryName WebIDL annotation (#4435).
                % if property.derived_from is None and property.name != "float":
                    % if property != LONGHANDS[-1]:
                        [${property.camel_case}, Set${property.camel_case}, "${property.name}"],
                    % else:
                        [${property.camel_case}, Set${property.camel_case}, "${property.name}"]
                    % endif
                % endif
            % endfor
        }
    }
}


macro_rules! longhand_properties_idents {
    ($macro_name: ident) => {
        $macro_name! {
            % for property in LONGHANDS:
                ${property.ident}
            % endfor
        }
    }
}

pub fn longhands_from_shorthand(shorthand: &str) -> Option<Vec<String>> {
    match shorthand {
        % for property in SHORTHANDS:
            "${property.name}" => Some(vec!(
            % for sub in property.sub_properties:
                "${sub.name}".to_owned(),
            % endfor
            )),
        % endfor
        _ => None,
    }
}
