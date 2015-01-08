/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

// This file is a Mako template: http://www.makotemplates.org/

pub use std::ascii::AsciiExt;
use std::fmt;
use std::fmt::Show;
use std::sync::Arc;

use servo_util::logical_geometry::{WritingMode, LogicalMargin};
pub use url::Url;

pub use cssparser::*;
pub use cssparser::ast::*;
pub use cssparser::ast::ComponentValue::*;
pub use geom::SideOffsets2D;
pub use self::common_types::specified::{Angle, AngleOrCorner};
pub use self::common_types::specified::{HorizontalDirection, VerticalDirection};

use errors::{ErrorLoggerIterator, log_css_error};
pub use parsing_utils::*;
pub use self::common_types::*;
use selector_matching::DeclarationBlock;


pub use self::property_bit_field::PropertyBitField;
pub mod common_types;


<%!

import re

def to_rust_ident(name):
    name = name.replace("-", "_")
    if name in ["static", "super", "box", "move"]:  # Rust keywords
        name += "_"
    return name

class Longhand(object):
    def __init__(self, name, derived_from=None, experimental=False):
        self.name = name
        self.ident = to_rust_ident(name)
        self.camel_case, _ = re.subn(
            "_([a-z])",
            lambda m: m.group(1).upper(),
            self.ident.strip("_").capitalize())
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
    pub use super::*;
    pub use std;

    pub fn computed_as_specified<T>(value: T, _context: &computed::Context) -> T {
        value
    }

    <%def name="raw_longhand(name, no_super=False, derived_from=None, experimental=False)">
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
            % if not no_super:
                use super::*;
            % endif
            pub use self::computed_value::*;
            ${caller.body()}
            % if derived_from is None:
                pub fn parse_declared(input: &[ComponentValue], base_url: &Url)
                                   -> Result<DeclaredValue<SpecifiedValue>, ()> {
                    match CSSWideKeyword::parse(input) {
                        Ok(CSSWideKeyword::InheritKeyword) => Ok(DeclaredValue::Inherit),
                        Ok(CSSWideKeyword::InitialKeyword) => Ok(DeclaredValue::Initial),
                        Ok(CSSWideKeyword::UnsetKeyword) => Ok(DeclaredValue::${
                            "Inherit" if THIS_STYLE_STRUCT.inherited else "Initial"}),
                        Err(()) => parse_specified(input, base_url),
                    }
                }
            % endif
        }
    </%def>

    <%def name="longhand(name, no_super=False, derived_from=None, experimental=False)">
        <%self:raw_longhand name="${name}" derived_from="${derived_from}"
                            experimental="${experimental}" no_super="${no_super}">
            ${caller.body()}
            % if derived_from is None:
                pub fn parse_specified(_input: &[ComponentValue], _base_url: &Url)
                                   -> Result<DeclaredValue<SpecifiedValue>, ()> {
                    parse(_input, _base_url).map(super::DeclaredValue::SpecifiedValue)
                }
            % endif
        </%self:raw_longhand>
    </%def>

    <%def name="single_component_value(name, derived_from=None, experimental=False)">
        <%self:longhand name="${name}" derived_from="${derived_from}"
                        experimental="${experimental}">
            ${caller.body()}
            pub fn parse(input: &[ComponentValue], base_url: &Url) -> Result<SpecifiedValue, ()> {
                one_component_value(input).and_then(|c| from_component_value(c, base_url))
            }
        </%self:longhand>
    </%def>

    <%def name="single_keyword_computed(name, values, experimental=False)">
        <%self:single_component_value name="${name}" experimental="${experimental}">
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
                T::${to_rust_ident(values.split()[0])}
            }
            pub fn from_component_value(v: &ComponentValue, _base_url: &Url)
                                        -> Result<SpecifiedValue, ()> {
                computed_value::T::parse(v)
            }
        </%self:single_component_value>
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
        <%self:single_component_value name="${name}">
            pub use super::super::common_types::computed::compute_${type} as to_computed_value;
            pub type SpecifiedValue = specified::${type};
            pub mod computed_value {
                pub type T = super::super::computed::${type};
            }
            #[inline] pub fn get_initial_value() -> computed_value::T { ${initial_value} }
            #[inline] pub fn from_component_value(v: &ComponentValue, _base_url: &Url)
                                                  -> Result<SpecifiedValue, ()> {
                specified::${type}::${parse_method}(v)
            }
        </%self:single_component_value>
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
        ${predefined_type("border-%s-color" % side, "CSSColor", "super::super::computed::CSSColor::CurrentColor")}
    % endfor

    ${single_keyword("border-top-style", values="none solid double dotted dashed hidden groove ridge inset outset")}

    % for side in ["right", "bottom", "left"]:
        <%self:longhand name="border-${side}-style">
            pub use super::border_top_style::{get_initial_value, parse, to_computed_value};
            pub type SpecifiedValue = super::border_top_style::SpecifiedValue;
            pub mod computed_value {
                pub type T = super::super::border_top_style::computed_value::T;
            }
        </%self:longhand>
    % endfor

    pub fn parse_border_width(component_value: &ComponentValue, _base_url: &Url)
                              -> Result<specified::Length, ()> {
        match component_value {
            &Ident(ref value) => {
                match value.as_slice().to_ascii_lower().as_slice() {
                    "thin" => Ok(specified::Length::from_px(1.)),
                    "medium" => Ok(specified::Length::from_px(3.)),
                    "thick" => Ok(specified::Length::from_px(5.)),
                    _ => Err(())
                }
            },
            _ => specified::Length::parse_non_negative(component_value)
        }
    }
    % for side in ["top", "right", "bottom", "left"]:
        <%self:longhand name="border-${side}-width">
            pub type SpecifiedValue = specified::Length;
            pub mod computed_value {
                use super::super::Au;
                pub type T = Au;
            }
            #[inline] pub fn get_initial_value() -> computed_value::T {
                Au::from_px(3)  // medium
            }
            pub fn parse(input: &[ComponentValue], base_url: &Url) -> Result<SpecifiedValue, ()> {
                one_component_value(input).and_then(|c| parse_border_width(c, base_url))
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

    <%self:longhand name="border-top-left-radius">
        #[deriving(Clone, Show, PartialEq, Copy)]
        pub struct SpecifiedValue {
            pub radius: specified::LengthOrPercentage,
        }

        pub mod computed_value {
            use super::super::computed;

            #[deriving(Clone, PartialEq, Copy, Show)]
            pub struct T {
                pub radius: computed::LengthOrPercentage,
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T {
                radius: computed::LengthOrPercentage::Length(Au(0)),
            }
        }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            computed_value::T {
                radius: computed::compute_LengthOrPercentage(value.radius, context),
            }
        }

        pub fn parse(input: &[ComponentValue], _: &Url) -> Result<SpecifiedValue,()> {
            let mut iter = input.skip_whitespace();

            let radius = match iter.next() {
                None     => return Err(()),
                Some(cv) => cv,
            };

            let radius = try!(specified::LengthOrPercentage::parse(radius));

            if iter.next().is_some() { return Err(()); }

            Ok(SpecifiedValue {
                radius: radius,
            })
        }
    </%self:longhand>

    % for corner in ["top-right", "bottom-right", "bottom-left"]:
        <%self:longhand name="border-${corner}-radius">
            pub type SpecifiedValue = super::border_top_left_radius::SpecifiedValue;

            pub mod computed_value {
                pub type T = super::super::border_top_left_radius::computed_value::T;
            }

            #[inline]
            pub fn get_initial_value() -> computed_value::T {
                super::border_top_left_radius::get_initial_value()
            }
            #[inline]
            pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                     -> computed_value::T {
                super::border_top_left_radius::to_computed_value(value, context)
            }

            pub fn parse(input: &[ComponentValue], u: &Url) -> Result<SpecifiedValue,()> {
                super::border_top_left_radius::parse(input, u)
            }
        </%self:longhand>
    % endfor

    ${new_style_struct("Outline", is_inherited=False)}

    // TODO(pcwalton): `invert`
    ${predefined_type("outline-color", "CSSColor", "super::super::computed::CSSColor::CurrentColor")}

    <%self:single_component_value name="outline-style">
        pub use super::border_top_style::{get_initial_value, to_computed_value};
        pub type SpecifiedValue = super::border_top_style::SpecifiedValue;
        pub mod computed_value {
            pub type T = super::super::border_top_style::computed_value::T;
        }
        pub fn from_component_value(value: &ComponentValue, base_url: &Url)
                                    -> Result<SpecifiedValue,()> {
            match value {
                &Ident(ref ident) if ident.eq_ignore_ascii_case("hidden") => {
                    // `hidden` is not a valid value.
                    Err(())
                }
                _ => super::border_top_style::from_component_value(value, base_url)
            }
        }
    </%self:single_component_value>

    <%self:longhand name="outline-width">
        pub use super::border_top_width::{get_initial_value, parse};
        pub use computed::compute_Au as to_computed_value;
        pub type SpecifiedValue = super::border_top_width::SpecifiedValue;
        pub mod computed_value {
            pub type T = super::super::border_top_width::computed_value::T;
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

    <%self:longhand name="-servo-display-for-hypothetical-box" derived_from="display" no_super="True">
        pub use super::computed_as_specified as to_computed_value;
        pub use super::display::{SpecifiedValue, get_initial_value};
        pub use super::display::{parse};
        use super::computed;
        use super::display;

        pub mod computed_value {
            pub type T = super::SpecifiedValue;
        }

        #[inline]
        pub fn derive_from_display(_: display::computed_value::T, context: &computed::Context)
                                   -> computed_value::T {
            context.display
        }

    </%self:longhand>

    <%self:single_component_value name="z-index">
        pub use super::computed_as_specified as to_computed_value;
        pub type SpecifiedValue = computed_value::T;
        pub mod computed_value {
            use std::fmt;

            #[deriving(PartialEq, Clone, Eq, Copy)]
            pub enum T {
                Auto,
                Number(i32),
            }
            impl fmt::Show for T {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        &T::Auto => write!(f, "auto"),
                        &T::Number(number) => write!(f, "{}", number),
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
            T::Auto
        }
        fn from_component_value(input: &ComponentValue, _: &Url) -> Result<SpecifiedValue,()> {
            match *input {
                Ident(ref keyword) if keyword.as_slice().eq_ignore_ascii_case("auto") => Ok(T::Auto),
                Number(NumericValue {
                    int_value: Some(value),
                    ..
                }) => Ok(T::Number(value as i32)),
                _ => Err(())
            }
        }
    </%self:single_component_value>

    ${new_style_struct("InheritedBox", is_inherited=True)}

    ${single_keyword("direction", "ltr rtl", experimental=True)}

    // CSS 2.1, Section 10 - Visual formatting model details

    ${switch_to_style_struct("Box")}

    ${predefined_type("width", "LengthOrPercentageOrAuto",
                      "computed::LengthOrPercentageOrAuto::Auto",
                      "parse_non_negative")}
    <%self:single_component_value name="height">
        pub type SpecifiedValue = specified::LengthOrPercentageOrAuto;
        pub mod computed_value {
            pub type T = super::super::computed::LengthOrPercentageOrAuto;
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { computed::LengthOrPercentageOrAuto::Auto }
        #[inline]
        pub fn from_component_value(v: &ComponentValue, _base_url: &Url)
                                              -> Result<SpecifiedValue, ()> {
            specified::LengthOrPercentageOrAuto::parse_non_negative(v)
        }
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match (value, context.inherited_height) {
                (specified::LengthOrPercentageOrAuto::Percentage(_), computed::LengthOrPercentageOrAuto::Auto)
                if !context.is_root_element && !context.positioned => {
                    computed::LengthOrPercentageOrAuto::Auto
                },
                _ => computed::compute_LengthOrPercentageOrAuto(value, context)
            }
        }
    </%self:single_component_value>

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

    <%self:single_component_value name="line-height">
        use std::fmt;
        #[deriving(Clone, PartialEq, Copy)]
        pub enum SpecifiedValue {
            Normal,
            Length(specified::Length),
            Number(CSSFloat),
            Percentage(CSSFloat),
        }
        impl fmt::Show for SpecifiedValue {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    &SpecifiedValue::Normal => write!(f, "normal"),
                    &SpecifiedValue::Length(length) => write!(f, "{}", length),
                    &SpecifiedValue::Number(number) => write!(f, "{}", number),
                    &SpecifiedValue::Percentage(number) => write!(f, "{}%", number * 100.),
                }
            }
        }
        /// normal | <number> | <length> | <percentage>
        pub fn from_component_value(input: &ComponentValue, _base_url: &Url)
                                    -> Result<SpecifiedValue, ()> {
            match input {
                &Number(ref value) if value.value >= 0. =>
                    Ok(SpecifiedValue::Number(value.value)),
                &Percentage(ref value) if value.value >= 0. =>
                    Ok(SpecifiedValue::Percentage(value.value / 100.)),
                &Dimension(ref value, ref unit) if value.value >= 0. =>
                    specified::Length::parse_dimension(value.value, unit.as_slice())
                        .map(SpecifiedValue::Length),
                &Ident(ref value) if value.as_slice().eq_ignore_ascii_case("normal") =>
                    Ok(SpecifiedValue::Normal),
                _ => Err(()),
            }
        }
        pub mod computed_value {
            use super::super::{Au, CSSFloat};
            use std::fmt;
            #[deriving(PartialEq, Copy, Clone)]
            pub enum T {
                Normal,
                Length(Au),
                Number(CSSFloat),
            }
            impl fmt::Show for T {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        &T::Normal => write!(f, "normal"),
                        &T::Length(length) => write!(f, "{}%", length),
                        &T::Number(number) => write!(f, "{}", number),
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { T::Normal }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match value {
                SpecifiedValue::Normal => T::Normal,
                SpecifiedValue::Length(value) => T::Length(computed::compute_Au(value, context)),
                SpecifiedValue::Number(value) => T::Number(value),
                SpecifiedValue::Percentage(value) => T::Length(computed::compute_Au(specified::Length::Em(value), context)),
            }
        }
    </%self:single_component_value>

    ${switch_to_style_struct("Box")}

    <%self:single_component_value name="vertical-align">
        use std::fmt;
        <% vertical_align_keywords = (
            "baseline sub super top text-top middle bottom text-bottom".split()) %>
        #[allow(non_camel_case_types)]
        #[deriving(Clone, PartialEq, Copy)]
        pub enum SpecifiedValue {
            % for keyword in vertical_align_keywords:
                ${to_rust_ident(keyword)},
            % endfor
            LengthOrPercentage(specified::LengthOrPercentage),
        }
        impl fmt::Show for SpecifiedValue {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    % for keyword in vertical_align_keywords:
                        &SpecifiedValue::${to_rust_ident(keyword)} => write!(f, "${keyword}"),
                    % endfor
                    &SpecifiedValue::LengthOrPercentage(lop) => write!(f, "{}", lop),
                }
            }
        }
        /// baseline | sub | super | top | text-top | middle | bottom | text-bottom
        /// | <percentage> | <length>
        pub fn from_component_value(input: &ComponentValue, _base_url: &Url)
                                    -> Result<SpecifiedValue, ()> {
            match input {
                &Ident(ref value) => {
                    match value.as_slice().to_ascii_lower().as_slice() {
                        % for keyword in vertical_align_keywords:
                        "${keyword}" => Ok(SpecifiedValue::${to_rust_ident(keyword)}),
                        % endfor
                        _ => Err(()),
                    }
                },
                _ => specified::LengthOrPercentage::parse_non_negative(input)
                     .map(SpecifiedValue::LengthOrPercentage)
            }
        }
        pub mod computed_value {
            use super::super::{Au, CSSFloat};
            use std::fmt;
            #[allow(non_camel_case_types)]
            #[deriving(PartialEq, Copy, Clone)]
            pub enum T {
                % for keyword in vertical_align_keywords:
                    ${to_rust_ident(keyword)},
                % endfor
                Length(Au),
                Percentage(CSSFloat),
            }
            impl fmt::Show for T {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        % for keyword in vertical_align_keywords:
                            &T::${to_rust_ident(keyword)} => write!(f, "${keyword}"),
                        % endfor
                        &T::Length(length) => write!(f, "{}", length),
                        &T::Percentage(number) => write!(f, "{}%", number),
                    }
                }
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T { T::baseline }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                              -> computed_value::T {
            match value {
                % for keyword in vertical_align_keywords:
                    SpecifiedValue::${to_rust_ident(keyword)} => computed_value::T::${to_rust_ident(keyword)},
                % endfor
                SpecifiedValue::LengthOrPercentage(value)
                => match computed::compute_LengthOrPercentage(value, context) {
                    computed::LengthOrPercentage::Length(value) => T::Length(value),
                    computed::LengthOrPercentage::Percentage(value) => T::Percentage(value)
                }
            }
        }
    </%self:single_component_value>


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
            pub mod computed_value {
            use std::fmt;
                #[deriving(PartialEq, Eq, Clone)]
                pub enum ContentItem {
                    StringContent(String),
                }
                impl fmt::Show for ContentItem {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        match self {
                            &ContentItem::StringContent(ref s) => write!(f, "\"{}\"", s),
                        }
                    }
                }
                #[allow(non_camel_case_types)]
                #[deriving(PartialEq, Eq, Clone)]
                pub enum T {
                    normal,
                    none,
                    Content(Vec<ContentItem>),
                }
                impl fmt::Show for T {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        match self {
                            &T::normal => write!(f, "normal"),
                            &T::none => write!(f, "none"),
                            &T::Content(ref content) => {
                                for c in content.iter() {
                                    let _ = write!(f, "{}", c);
                                }
                                Ok(())
                            }
                        }
                    }
                }
            }
            pub type SpecifiedValue = computed_value::T;
            #[inline] pub fn get_initial_value() -> computed_value::T  { T::normal }

            // normal | none | [ <string> ]+
            // TODO: <uri>, <counter>, attr(<identifier>), open-quote, close-quote, no-open-quote, no-close-quote
            pub fn parse(input: &[ComponentValue], _base_url: &Url) -> Result<SpecifiedValue, ()> {
                match one_component_value(input) {
                    Ok(&Ident(ref keyword)) => {
                        match keyword.as_slice().to_ascii_lower().as_slice() {
                            "normal" => return Ok(T::normal),
                            "none" => return Ok(T::none),
                            _ => ()
                        }
                    },
                    _ => ()
                }
                let mut content = vec!();
                for component_value in input.skip_whitespace() {
                    match component_value {
                        &QuotedString(ref value)
                        => content.push(ContentItem::StringContent(value.clone())),
                        _ => return Err(())  // invalid/unsupported value
                    }
                }
                Ok(T::Content(content))
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

    <%self:single_component_value name="list-style-image">
        pub use super::computed_as_specified as to_computed_value;
        pub type SpecifiedValue = Option<Url>;
        pub mod computed_value {
            use url::Url;
            pub type T = Option<Url>;
        }
        pub fn from_component_value(input: &ComponentValue, base_url: &Url)
                                    -> Result<SpecifiedValue,()> {
            match *input {
                URL(ref url) => Ok(Some(super::parse_url(url.as_slice(), base_url))),
                Ident(ref value) if value.as_slice().eq_ignore_ascii_case("none") => Ok(None),
                _ => Err(()),
            }
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            None
        }
    </%self:single_component_value>

    // CSS 2.1, Section 13 - Paged media

    // CSS 2.1, Section 14 - Colors and Backgrounds

    ${new_style_struct("Background", is_inherited=False)}
    ${predefined_type("background-color", "CSSColor",
                      "Color::RGBA(RGBA { red: 0., green: 0., blue: 0., alpha: 0. }) /* transparent */")}

    <%self:single_component_value name="background-image">
        use super::common_types::specified as common_specified;
        use super::super::common_types::specified::CSSImage as CSSImage;
        pub mod computed_value {
            use super::super::super::common_types::computed;
            pub type T = Option<computed::Image>;
        }
        pub type SpecifiedValue = common_specified::CSSImage;
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            None
        }
        pub fn from_component_value(component_value: &ComponentValue, base_url: &Url)
                                    -> Result<SpecifiedValue, ()> {
            match component_value {
                &Ident(ref value)
                if value.as_slice().eq_ignore_ascii_case("none") => {
                    Ok(CSSImage(None))
                }
                _ => {
                    match common_specified::Image::from_component_value(component_value,
                                                                        base_url) {
                        Err(err) => Err(err),
                        Ok(result) => Ok(CSSImage(Some(result))),
                    }
                }
            }
        }
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            match value {
                CSSImage(None) => None,
                CSSImage(Some(image)) => Some(image.to_computed_value(context)),
            }
        }
    </%self:single_component_value>

    <%self:longhand name="background-position">
            use std::fmt;

            pub mod computed_value {
                use super::super::super::common_types::computed::LengthOrPercentage;
                use std::fmt;

                #[deriving(PartialEq, Copy, Clone)]
                pub struct T {
                    pub horizontal: LengthOrPercentage,
                    pub vertical: LengthOrPercentage,
                }
                impl fmt::Show for T {
                    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "{} {}", self.horizontal, self.vertical)
                    }
                }
            }

            #[deriving(Clone, PartialEq, Copy)]
            pub struct SpecifiedValue {
                pub horizontal: specified::LengthOrPercentage,
                pub vertical: specified::LengthOrPercentage,
            }
            impl fmt::Show for SpecifiedValue {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write!(f, "{} {}", self.horizontal, self.vertical)
                }
            }

            impl SpecifiedValue {
                fn new(first: specified::PositionComponent, second: specified::PositionComponent)
                        -> Result<SpecifiedValue,()> {
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

            pub fn parse_one(first: &ComponentValue) -> Result<SpecifiedValue, ()> {
                let first = try!(specified::PositionComponent::parse(first));
                // If only one value is provided, use `center` for the second.
                SpecifiedValue::new(first, specified::PositionComponent::Center)
            }

            pub fn parse_two(first: &ComponentValue, second: &ComponentValue)
                    -> Result<SpecifiedValue, ()> {
                let first = try!(specified::PositionComponent::parse(first));
                let second = try!(specified::PositionComponent::parse(second));
                SpecifiedValue::new(first, second)
            }

            pub fn parse(input: &[ComponentValue], _: &Url) -> Result<SpecifiedValue, ()> {
                let mut input_iter = input.skip_whitespace();
                let first = input_iter.next();
                let second = input_iter.next();
                if input_iter.next().is_some() {
                    return Err(())
                }
                match (first, second) {
                    (Some(first), Some(second)) => {
                        parse_two(first, second)
                    }
                    (Some(first), None) => {
                        parse_one(first)
                    }
                    _ => Err(())
                }
            }
    </%self:longhand>

    ${single_keyword("background-repeat", "repeat repeat-x repeat-y no-repeat")}

    ${single_keyword("background-attachment", "scroll fixed")}

    ${new_style_struct("Color", is_inherited=True)}

    <%self:raw_longhand name="color">
        use super::super::common_types::specified::{CSSColor, CSSRGBA};
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
        pub fn parse_specified(input: &[ComponentValue], _base_url: &Url)
                               -> Result<DeclaredValue<SpecifiedValue>, ()> {
            match one_component_value(input).and_then(CSSColor::parse) {
                Ok(CSSColor { parsed: Color::RGBA(rgba), authored }) => {
                    let rgba = CSSRGBA {
                        parsed: rgba,
                        authored: authored,
                    };
                    Ok(DeclaredValue::SpecifiedValue(rgba))
                }
                Ok(CSSColor { parsed: Color::CurrentColor, .. }) => Ok(DeclaredValue::Inherit),
                Err(()) => Err(()),
            }
        }
    </%self:raw_longhand>

    // CSS 2.1, Section 15 - Fonts

    ${new_style_struct("Font", is_inherited=True)}

    <%self:longhand name="font-family">
        pub use super::computed_as_specified as to_computed_value;
        pub mod computed_value {
            use std::fmt;
            #[deriving(PartialEq, Eq, Clone)]
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
                        FontFamily::FamilyName(ref name) => name.as_slice(),
                    }
                }
            }
            impl fmt::Show for FontFamily {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    match self {
                        &FontFamily::FamilyName(ref name) => write!(f, "{}", name),
                    }
                }
            }
            pub type T = Vec<FontFamily>;
            /*impl fmt::Show for T {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    for font in self.iter() {
                        write!(f, "{}", font);
                    }
                    Ok(())
                }
            }*/
        }
        pub type SpecifiedValue = computed_value::T;

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            vec![FontFamily::FamilyName("serif".into_string())]
        }
        /// <familiy-name>#
        /// <familiy-name> = <string> | [ <ident>+ ]
        /// TODO: <generic-familiy>
        pub fn parse(input: &[ComponentValue], _base_url: &Url) -> Result<SpecifiedValue, ()> {
            parse_slice_comma_separated(input, parse_one_family)
        }
        pub fn parse_one_family<'a>(iter: ParserIter) -> Result<FontFamily, ()> {
            // TODO: avoid copying strings?
            let mut idents = match iter.next() {
                Some(&QuotedString(ref value)) => return Ok(FontFamily::FamilyName(value.clone())),
                Some(&Ident(ref value)) => {
//                    match value.as_slice().to_ascii_lower().as_slice() {
//                        "serif" => return Ok(Serif),
//                        "sans-serif" => return Ok(SansSerif),
//                        "cursive" => return Ok(Cursive),
//                        "fantasy" => return Ok(Fantasy),
//                        "monospace" => return Ok(Monospace),
//                        _ => {
                            vec![value.as_slice()]
//                        }
//                    }
                }
                _ => return Err(())
            };
            loop {
                match iter.next() {
                    Some(&Ident(ref value)) => {
                        idents.push(value.as_slice());
                        iter.next();
                    }
                    Some(component_value) => {
                        iter.push_back(component_value);
                        break
                    }
                    None => break,
                }
            }
            Ok(FontFamily::FamilyName(idents.connect(" ")))
        }
    </%self:longhand>


    ${single_keyword("font-style", "normal italic oblique")}
    ${single_keyword("font-variant", "normal small-caps")}

    <%self:single_component_value name="font-weight">
        use std::fmt;
        #[deriving(Clone, PartialEq, Eq, Copy)]
        pub enum SpecifiedValue {
            Bolder,
            Lighter,
            % for weight in range(100, 901, 100):
                SpecifiedWeight${weight},
            % endfor
        }
        impl fmt::Show for SpecifiedValue {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                match self {
                    &SpecifiedValue::Bolder => write!(f, "bolder"),
                    &SpecifiedValue::Lighter => write!(f, "lighter"),
                    % for weight in range(100, 901, 100):
                        &SpecifiedValue::SpecifiedWeight${weight} => write!(f, "{}", ${weight}i),
                    % endfor
                }
            }
        }
        /// normal | bold | bolder | lighter | 100 | 200 | 300 | 400 | 500 | 600 | 700 | 800 | 900
        pub fn from_component_value(input: &ComponentValue, _base_url: &Url)
                                    -> Result<SpecifiedValue, ()> {
            match input {
                &Ident(ref value) => {
                    match value.as_slice().to_ascii_lower().as_slice() {
                        "bold" => Ok(SpecifiedValue::SpecifiedWeight700),
                        "normal" => Ok(SpecifiedValue::SpecifiedWeight400),
                        "bolder" => Ok(SpecifiedValue::Bolder),
                        "lighter" => Ok(SpecifiedValue::Lighter),
                        _ => Err(()),
                    }
                },
                &Number(ref value) => match value.int_value {
                    Some(100) => Ok(SpecifiedValue::SpecifiedWeight100),
                    Some(200) => Ok(SpecifiedValue::SpecifiedWeight200),
                    Some(300) => Ok(SpecifiedValue::SpecifiedWeight300),
                    Some(400) => Ok(SpecifiedValue::SpecifiedWeight400),
                    Some(500) => Ok(SpecifiedValue::SpecifiedWeight500),
                    Some(600) => Ok(SpecifiedValue::SpecifiedWeight600),
                    Some(700) => Ok(SpecifiedValue::SpecifiedWeight700),
                    Some(800) => Ok(SpecifiedValue::SpecifiedWeight800),
                    Some(900) => Ok(SpecifiedValue::SpecifiedWeight900),
                    _ => Err(()),
                },
                _ => Err(())
            }
        }
        pub mod computed_value {
            use std::fmt;
            #[deriving(PartialEq, Eq, Copy, Clone)]
            pub enum T {
                % for weight in range(100, 901, 100):
                    Weight${weight},
                % endfor
            }
            impl fmt::Show for T {
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
                    SpecifiedValue::SpecifiedWeight${weight} => computed_value::T::Weight${weight},
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
    </%self:single_component_value>

    <%self:single_component_value name="font-size">
        pub type SpecifiedValue = specified::Length;  // Percentages are the same as em.
        pub mod computed_value {
            use super::super::Au;
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
        pub fn from_component_value(input: &ComponentValue, _base_url: &Url)
                                    -> Result<SpecifiedValue, ()> {
            match specified::LengthOrPercentage::parse_non_negative(input) {
                Ok(specified::LengthOrPercentage::Length(value)) => return Ok(value),
                Ok(specified::LengthOrPercentage::Percentage(value)) => return Ok(specified::Length::Em(value)),
                Err(()) => (),
            }
            match try!(get_ident_lower(input)).as_slice() {
                "xx-small" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 3 / 5)),
                "x-small" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 3 / 4)),
                "small" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 8 / 9)),
                "medium" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX))),
                "large" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 6 / 5)),
                "x-large" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 3 / 2)),
                "xx-large" => Ok(specified::Length::Au(Au::from_px(MEDIUM_PX) * 2)),

                // https://github.com/servo/servo/issues/3423#issuecomment-56321664
                "smaller" => Ok(specified::Length::Em(0.85)),
                "larger" => Ok(specified::Length::Em(1.2)),

                _ => return Err(())
            }
        }
    </%self:single_component_value>

    // CSS 2.1, Section 16 - Text

    ${new_style_struct("InheritedText", is_inherited=True)}

    // TODO: initial value should be 'start' (CSS Text Level 3, direction-dependent.)
    ${single_keyword("text-align", "left right center justify")}

    <%self:single_component_value name="letter-spacing">
        pub type SpecifiedValue = Option<specified::Length>;
        pub mod computed_value {
            use super::super::Au;
            pub type T = Option<Au>;
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            None
        }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            value.map(|length| computed::compute_Au(length, context))
        }
        pub fn from_component_value(input: &ComponentValue, _: &Url) -> Result<SpecifiedValue,()> {
            match input {
                &Ident(ref value) if value.eq_ignore_ascii_case("normal") => Ok(None),
                _ => specified::Length::parse_non_negative(input).map(|length| Some(length)),
            }
        }
    </%self:single_component_value>

    <%self:single_component_value name="word-spacing">
        pub type SpecifiedValue = Option<specified::Length>;
        pub mod computed_value {
            use super::super::Au;
            pub type T = Option<Au>;
        }
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            None
        }
        #[inline]
        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            value.map(|length| computed::compute_Au(length, context))
        }
        pub fn from_component_value(input: &ComponentValue, _: &Url) -> Result<SpecifiedValue,()> {
            match input {
                &Ident(ref value) if value.eq_ignore_ascii_case("normal") => Ok(None),
                _ => specified::Length::parse_non_negative(input).map(|length| Some(length)),
            }
        }
    </%self:single_component_value>

    ${predefined_type("text-indent", "LengthOrPercentage", "computed::LengthOrPercentage::Length(Au(0))")}

    // Also known as "word-wrap" (which is more popular because of IE), but this is the preferred
    // name per CSS-TEXT 6.2.
    ${single_keyword("overflow-wrap", "normal break-word")}

    // TODO(pcwalton): Support `word-break: keep-all` once we have better CJK support.
    ${single_keyword("word-break", "normal break-all")}

    ${new_style_struct("Text", is_inherited=False)}

    <%self:longhand name="text-decoration">
        pub use super::computed_as_specified as to_computed_value;
        use std::fmt;
        #[deriving(PartialEq, Eq, Copy, Clone)]
        pub struct SpecifiedValue {
            pub underline: bool,
            pub overline: bool,
            pub line_through: bool,
            // 'blink' is accepted in the parser but ignored.
            // Just not blinking the text is a conforming implementation per CSS 2.1.
        }
        impl fmt::Show for SpecifiedValue {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let mut space = false;
                if self.underline {
                    let _ = write!(f, "underline");
                    space = true;
                }
                if self.overline {
                    if space {
                        let _ = write!(f, " ");
                    }
                    let _ = write!(f, "overline");
                    space = true;
                }
                if self.line_through {
                    if space {
                        let _ = write!(f, " ");
                    }
                    let _ = write!(f, "line-through");
                }
                Ok(())
            }
        }
        pub mod computed_value {
            pub type T = super::SpecifiedValue;
            #[allow(non_upper_case_globals)]
            pub const none: T = super::SpecifiedValue { underline: false, overline: false, line_through: false };
        }
        #[inline] pub fn get_initial_value() -> computed_value::T {
            none
        }
        /// none | [ underline || overline || line-through || blink ]
        pub fn parse(input: &[ComponentValue], _base_url: &Url) -> Result<SpecifiedValue, ()> {
            let mut result = SpecifiedValue {
                underline: false, overline: false, line_through: false,
            };
            match one_component_value(input) {
                Ok(&Ident(ref value))
                if value.as_slice().eq_ignore_ascii_case("none") => return Ok(result),
                _ => {}
            }
            let mut blink = false;
            let mut empty = true;
            for component_value in input.skip_whitespace() {
                match get_ident_lower(component_value) {
                    Err(()) => return Err(()),
                    Ok(keyword) => match keyword.as_slice() {
                        "underline" => if result.underline { return Err(()) }
                                      else { empty = false; result.underline = true },
                        "overline" => if result.overline { return Err(()) }
                                      else { empty = false; result.overline = true },
                        "line-through" => if result.line_through { return Err(()) }
                                          else { empty = false; result.line_through = true },
                        "blink" => if blink { return Err(()) }
                                   else { empty = false; blink = true },
                        _ => return Err(()),
                    }
                }
            }
            if !empty { Ok(result) } else { Err(()) }
        }
    </%self:longhand>

    ${switch_to_style_struct("InheritedText")}

    <%self:longhand name="-servo-text-decorations-in-effect"
                    derived_from="display text-decoration">
        pub use super::computed_as_specified as to_computed_value;

        #[deriving(Clone, PartialEq, Copy)]
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
                display::computed_value::T::inline => context.inherited_text_decorations_in_effect,
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
        pub fn derive_from_text_decoration(_: text_decoration::computed_value::T,
                                           context: &computed::Context)
                                           -> computed_value::T {
            derive(context)
        }

        #[inline]
        pub fn derive_from_display(_: display::computed_value::T, context: &computed::Context)
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

    <%self:single_component_value name="cursor">
        use servo_util::cursor as util_cursor;
        pub use super::computed_as_specified as to_computed_value;

        pub mod computed_value {
            use servo_util::cursor::Cursor;
            #[deriving(Clone, PartialEq, Eq, Copy, Show)]
            pub enum T {
                AutoCursor,
                SpecifiedCursor(Cursor),
            }
        }
        pub type SpecifiedValue = computed_value::T;
        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            computed_value::T::AutoCursor
        }
        pub fn from_component_value(value: &ComponentValue, _: &Url)
                                    -> Result<SpecifiedValue,()> {
            match value {
                &Ident(ref ident) => {
                    if ident.eq_ignore_ascii_case("auto") {
                        Ok(T::AutoCursor)
                    } else {
                        util_cursor::Cursor::from_css_keyword(ident.as_slice())
                        .map(T::SpecifiedCursor)
                    }
                }
                _ => Err(())
            }
        }
    </%self:single_component_value>

    // NB: `pointer-events: auto` (and use of `pointer-events` in anything that isn't SVG, in fact)
    // is nonstandard, slated for CSS4-UI.
    // TODO(pcwalton): SVG-only values.
    ${single_keyword("pointer-events", "auto none")}

    // Box-shadow, etc.
    ${new_style_struct("Effects", is_inherited=False)}

    <%self:single_component_value name="opacity">
        pub type SpecifiedValue = CSSFloat;
        pub mod computed_value {
            use super::super::CSSFloat;
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
        fn from_component_value(input: &ComponentValue, _: &Url) -> Result<SpecifiedValue,()> {
            match *input {
                Number(ref value) => Ok(value.value),
                _ => Err(())
            }
        }
    </%self:single_component_value>

    <%self:longhand name="box-shadow">
        use cssparser;
        use std::fmt;

        pub type SpecifiedValue = Vec<SpecifiedBoxShadow>;

        #[deriving(Clone, PartialEq)]
        pub struct SpecifiedBoxShadow {
            pub offset_x: specified::Length,
            pub offset_y: specified::Length,
            pub blur_radius: specified::Length,
            pub spread_radius: specified::Length,
            pub color: Option<specified::CSSColor>,
            pub inset: bool,
        }

        impl fmt::Show for SpecifiedBoxShadow {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                if self.inset {
                    let _ = write!(f, "inset ");
                }
                let _ = write!(f, "{} {} {} {}", self.offset_x, self.offset_y,
                               self.blur_radius, self.spread_radius);
                if let Some(ref color) = self.color {
                    let _ = write!(f, "{}", color);
                }
                Ok(())
            }
        }

        pub mod computed_value {
            use super::super::Au;
            use super::super::super::computed;
            use std::fmt;

            pub type T = Vec<BoxShadow>;

            #[deriving(Clone, PartialEq, Copy)]
            pub struct BoxShadow {
                pub offset_x: Au,
                pub offset_y: Au,
                pub blur_radius: Au,
                pub spread_radius: Au,
                pub color: computed::CSSColor,
                pub inset: bool,
            }

            impl fmt::Show for BoxShadow {
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    if self.inset {
                        let _ = write!(f, "inset ");
                    }
                    let _ = write!(f, "{} {} {} {} {}", self.offset_x, self.offset_y,
                                   self.blur_radius, self.spread_radius, self.color);
                    Ok(())
                }
            }
        }

        #[inline]
        pub fn get_initial_value() -> computed_value::T {
            Vec::new()
        }

        pub fn parse(input: &[ComponentValue], _: &Url) -> Result<SpecifiedValue,()> {
            match one_component_value(input) {
                Ok(&Ident(ref value)) if value.as_slice().eq_ignore_ascii_case("none") => {
                    return Ok(Vec::new())
                }
                _ => {}
            }
            parse_slice_comma_separated(input, parse_one_box_shadow)
        }

        pub fn to_computed_value(value: SpecifiedValue, context: &computed::Context)
                                 -> computed_value::T {
            value.into_iter().map(|value| {
                computed_value::BoxShadow {
                    offset_x: computed::compute_Au(value.offset_x, context),
                    offset_y: computed::compute_Au(value.offset_y, context),
                    blur_radius: computed::compute_Au(value.blur_radius, context),
                    spread_radius: computed::compute_Au(value.spread_radius, context),
                    color: value.color.map(|color| color.parsed).unwrap_or(cssparser::Color::CurrentColor),
                    inset: value.inset,
                }
            }).collect()
        }

        fn parse_one_box_shadow(iter: ParserIter) -> Result<SpecifiedBoxShadow,()> {
            let mut lengths = [specified::Length::Au(Au(0)), ..4];
            let mut lengths_parsed = false;
            let mut color = None;
            let mut inset = false;

            loop {
                match iter.next() {
                    Some(&Ident(ref value)) if value.eq_ignore_ascii_case("inset") && !inset => {
                        inset = true;
                        continue
                    }
                    Some(value) => {
                        // Try to parse a length.
                        match specified::Length::parse(value) {
                            Ok(the_length) if !lengths_parsed => {
                                lengths[0] = the_length;
                                let mut length_parsed_count = 1;
                                while length_parsed_count < 4 {
                                    match iter.next() {
                                        Some(value) => {
                                            match specified::Length::parse(value) {
                                                Ok(the_length) => {
                                                    lengths[length_parsed_count] = the_length;
                                                }
                                                Err(_) => {
                                                    iter.push_back(value);
                                                    break
                                                }
                                            }
                                        }
                                        None => break,
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
                            Ok(_) => return Err(()),
                            Err(()) => {}
                        }

                        // Try to parse a color.
                        match specified::CSSColor::parse(value) {
                            Ok(ref the_color) if color.is_none() => {
                                color = Some(the_color.clone());
                                continue
                            }
                            Ok(_) => return Err(()),
                            Err(()) => {}
                        }

                        iter.push_back(value);
                        break
                    }
                    None => break,
                }
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

    <%self:single_component_value name="clip">
        // NB: `top` and `left` are 0 if `auto` per CSS 2.1 11.1.2.

        pub mod computed_value {
            use super::super::Au;

            #[deriving(Clone, PartialEq, Eq, Copy, Show)]
            pub struct ClipRect {
                pub top: Au,
                pub right: Option<Au>,
                pub bottom: Option<Au>,
                pub left: Au,
            }

            pub type T = Option<ClipRect>;
        }

        #[deriving(Clone, Show, PartialEq, Copy)]
        pub struct SpecifiedClipRect {
            pub top: specified::Length,
            pub right: Option<specified::Length>,
            pub bottom: Option<specified::Length>,
            pub left: specified::Length,
        }

        pub type SpecifiedValue = Option<SpecifiedClipRect>;

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

        pub fn from_component_value(input: &ComponentValue, _: &Url) -> Result<SpecifiedValue,()> {
            match *input {
                Function(ref name, ref args) if name.as_slice().eq_ignore_ascii_case("rect") => {
                    let sides = try!(parse_slice_comma_separated(args.as_slice(), |parser| {
                        match parser.next() {
                            Some(&Ident(ref ident)) if ident.eq_ignore_ascii_case("auto") => {
                                Ok(None)
                            }
                            Some(arg) => {
                                match specified::Length::parse(arg) {
                                    Err(_) => {
                                        parser.push_back(arg);
                                        Err(())
                                    }
                                    Ok(value) => Ok(Some(value)),
                                }
                            }
                            None => Err(()),
                        }
                    }));
                    if sides.len() != 4 {
                        return Err(())
                    }
                    Ok(Some(SpecifiedClipRect {
                        top: sides[0].unwrap_or(specified::Length::Au(Au(0))),
                        right: sides[1],
                        bottom: sides[2],
                        left: sides[3].unwrap_or(specified::Length::Au(Au(0))),
                    }))
                }
                Ident(ref ident) if ident.as_slice().eq_ignore_ascii_case("auto") => Ok(None),
                _ => Err(())
            }
        }
    </%self:single_component_value>
}


pub mod shorthands {
    pub use super::*;
    pub use super::longhands::*;

    <%def name="shorthand(name, sub_properties)">
    <%
        shorthand = Shorthand(name, sub_properties.split())
        SHORTHANDS.append(shorthand)
    %>
        pub mod ${shorthand.ident} {
            use super::*;
            pub struct Longhands {
                % for sub_property in shorthand.sub_properties:
                    pub ${sub_property.ident}: Option<${sub_property.ident}::SpecifiedValue>,
                % endfor
            }
            pub fn parse(input: &[ComponentValue], base_url: &Url) -> Result<Longhands, ()> {
                ${caller.body()}
            }
        }
    </%def>

    <%def name="four_sides_shorthand(name, sub_property_pattern, parser_function)">
        <%self:shorthand name="${name}" sub_properties="${
                ' '.join(sub_property_pattern % side
                         for side in ['top', 'right', 'bottom', 'left'])}">
            let mut iter = input.skip_whitespace().map(|c| ${parser_function}(c, base_url).ok());
            // zero or more than four values is invalid.
            // one value sets them all
            // two values set (top, bottom) and (left, right)
            // three values set top, (left, right) and bottom
            // four values set them in order
            let top = iter.next().unwrap_or(None);
            let right = iter.next().unwrap_or(top.clone());
            let bottom = iter.next().unwrap_or(top.clone());
            let left = iter.next().unwrap_or(right.clone());
            if top.is_some() && right.is_some() && bottom.is_some() && left.is_some()
            && iter.next().is_none() {
                Ok(Longhands {
                    % for side in ["top", "right", "bottom", "left"]:
                        ${to_rust_ident(sub_property_pattern % side)}: ${side},
                    % endfor
                })
            } else {
                Err(())
            }
        </%self:shorthand>
    </%def>

    // TODO: other background-* properties
    <%self:shorthand name="background"
                     sub_properties="background-color background-position background-repeat background-attachment background-image">
                use std::mem;

                let (mut color, mut image, mut position, mut repeat, mut attachment) =
                    (None, None, None, None, None);
                let mut unused_component_value = None;
                let mut any = false;

                for component_value in input.skip_whitespace() {
                    // Try `background-position` first because it might not use the value.
                    if position.is_none() {
                        match mem::replace(&mut unused_component_value, None) {
                            Some(saved_component_value) => {
                                // First try parsing a pair of values, then a single value.
                                match background_position::parse_two(saved_component_value,
                                                                     component_value) {
                                    Ok(v) => {
                                        position = Some(v);
                                        any = true;
                                        continue
                                    },
                                    Err(()) => {
                                        match background_position::parse_one(saved_component_value) {
                                            Ok(v) => {
                                                position = Some(v);
                                                any = true;
                                                // We haven't used the current `component_value`;
                                                // keep attempting to parse it below.
                                            },
                                            // If we get here, parsing failed.
                                            Err(()) => return Err(())
                                        }
                                    }
                                }
                            }
                            None => () // Wait until we have a pair of potential values.
                        }
                    }

                    if color.is_none() {
                        match background_color::from_component_value(component_value, base_url) {
                            Ok(v) => {
                                color = Some(v);
                                any = true;
                                continue
                            },
                            Err(()) => ()
                        }
                    }

                    if image.is_none() {
                        match background_image::from_component_value(component_value, base_url) {
                            Ok(v) => {
                                image = Some(v);
                                any = true;
                                continue
                            },
                            Err(()) => (),
                        }
                    }

                    if repeat.is_none() {
                        match background_repeat::from_component_value(component_value, base_url) {
                            Ok(v) => {
                                repeat = Some(v);
                                any = true;
                                continue
                            },
                            Err(()) => ()
                        }
                    }

                    if attachment.is_none() {
                        match background_attachment::from_component_value(component_value,
                                                                          base_url) {
                            Ok(v) => {
                                attachment = Some(v);
                                any = true;
                                continue
                            },
                            Err(()) => ()
                        }
                    }

                    // Save the component value.  It may the first of a background-position pair.
                    unused_component_value = Some(component_value);
                }

                if position.is_none() {
                    // Check for a lone trailing background-position value.
                    match mem::replace(&mut unused_component_value, None) {
                        Some(saved_component_value) => {
                            match background_position::parse_one(saved_component_value) {
                                Ok(v) => {
                                    position = Some(v);
                                    any = true;
                                },
                                Err(()) => return Err(())
                            }
                        }
                        None => ()
                    }
                }

                if any && unused_component_value.is_none() {
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

    ${four_sides_shorthand("margin", "margin-%s", "margin_top::from_component_value")}
    ${four_sides_shorthand("padding", "padding-%s", "padding_top::from_component_value")}

    pub fn parse_color(value: &ComponentValue, _base_url: &Url) -> Result<specified::CSSColor, ()> {
        specified::CSSColor::parse(value)
    }
    ${four_sides_shorthand("border-color", "border-%s-color", "parse_color")}
    ${four_sides_shorthand("border-style", "border-%s-style",
                           "border_top_style::from_component_value")}
    ${four_sides_shorthand("border-width", "border-%s-width", "parse_border_width")}

    pub fn parse_border(input: &[ComponentValue], base_url: &Url)
                     -> Result<(Option<specified::CSSColor>,
                                Option<border_top_style::SpecifiedValue>,
                                Option<specified::Length>), ()> {
        let mut color = None;
        let mut style = None;
        let mut width = None;
        let mut any = false;
        for component_value in input.skip_whitespace() {
            if color.is_none() {
                match specified::CSSColor::parse(component_value) {
                    Ok(c) => { color = Some(c); any = true; continue },
                    Err(()) => ()
                }
            }
            if style.is_none() {
                match border_top_style::from_component_value(component_value, base_url) {
                    Ok(s) => { style = Some(s); any = true; continue },
                    Err(()) => ()
                }
            }
            if width.is_none() {
                match parse_border_width(component_value, base_url) {
                    Ok(w) => { width = Some(w); any = true; continue },
                    Err(()) => ()
                }
            }
            return Err(())
        }
        if any { Ok((color, style, width)) } else { Err(()) }
    }


    % for side in ["top", "right", "bottom", "left"]:
        <%self:shorthand name="border-${side}" sub_properties="${' '.join(
            'border-%s-%s' % (side, prop)
            for prop in ['color', 'style', 'width']
        )}">
            parse_border(input, base_url).map(|(color, style, width)| {
                Longhands {
                    % for prop in ["color", "style", "width"]:
                        ${"border_%s_%s: %s," % (side, prop, prop)}
                    % endfor
                }
            })
        </%self:shorthand>
    % endfor

    <%self:shorthand name="border" sub_properties="${' '.join(
        'border-%s-%s' % (side, prop)
        for side in ['top', 'right', 'bottom', 'left']
        for prop in ['color', 'style', 'width']
    )}">
        parse_border(input, base_url).map(|(color, style, width)| {
            Longhands {
                % for side in ["top", "right", "bottom", "left"]:
                    % for prop in ["color", "style", "width"]:
                        ${"border_%s_%s: %s.clone()," % (side, prop, prop)}
                    % endfor
                % endfor
            }
        })
    </%self:shorthand>

    <%self:shorthand name="border-radius" sub_properties="${' '.join(
        'border-%s-radius' % (corner)
         for corner in ['top-left', 'top-right', 'bottom-right', 'bottom-left']
    )}">

        use std::iter::Peekable;

        let _ignored = base_url;

        fn parse_one_set_of_border_radii<'a,I>(mut input: Peekable< &'a ComponentValue,I >)
                                         -> Result<[specified::LengthOrPercentage, ..4],()>
                                         where I: Iterator< &'a ComponentValue > {
            let (mut count, mut values) = (0u, [specified::LengthOrPercentage::Length(specified::Length::Au(Au(0))), ..4]);
            while count < 4 {
                let token = match input.peek() {
                    None => break,
                    Some(token) => *token,
                };
                let value = match specified::LengthOrPercentage::parse(token) {
                    Err(_) => break,
                    Ok(value) => value,
                };
                drop(input.next());
                values[count] = value;
                count += 1
            }

            match count {
                1 => Ok([values[0], values[0], values[0], values[0]]),
                2 => Ok([values[0], values[1], values[0], values[1]]),
                3 => Ok([values[0], values[1], values[2], values[1]]),
                4 => Ok([values[0], values[1], values[2], values[3]]),
                _ => Err(()),
            }
        }

        let input = input.skip_whitespace().peekable();
        let radii = try!(parse_one_set_of_border_radii(input));
        // TODO(pcwalton): Elliptical borders.

        Ok(Longhands {
            border_top_left_radius: Some(border_top_left_radius::SpecifiedValue {
                radius: radii[0],
            }),
            border_top_right_radius: Some(border_top_left_radius::SpecifiedValue {
                radius: radii[1],
            }),
            border_bottom_right_radius: Some(border_top_left_radius::SpecifiedValue {
                radius: radii[2],
            }),
            border_bottom_left_radius: Some(border_top_left_radius::SpecifiedValue {
                radius: radii[3],
            }),
        })
    </%self:shorthand>

    <%self:shorthand name="outline" sub_properties="outline-color outline-style outline-width">
        let (mut color, mut style, mut width, mut any) = (None, None, None, false);
        for component_value in input.skip_whitespace() {
            if color.is_none() {
                match specified::CSSColor::parse(component_value) {
                    Ok(c) => {
                        color = Some(c);
                        any = true;
                        continue
                    }
                    Err(()) => {}
                }
            }
            if style.is_none() {
                match border_top_style::from_component_value(component_value, base_url) {
                    Ok(s) => {
                        style = Some(s);
                        any = true;
                        continue
                    }
                    Err(()) => {}
                }
            }
            if width.is_none() {
                match parse_border_width(component_value, base_url) {
                    Ok(w) => {
                        width = Some(w);
                        any = true;
                        continue
                    }
                    Err(()) => {}
                }
            }
            return Err(())
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
        let mut iter = input.skip_whitespace();
        let mut nb_normals = 0u;
        let mut style = None;
        let mut variant = None;
        let mut weight = None;
        let mut size = None;
        let mut line_height = None;
        for component_value in iter {
            // Special-case 'normal' because it is valid in each of
            // font-style, font-weight and font-variant.
            // Leaves the values to None, 'normal' is the initial value for each of them.
            match get_ident_lower(component_value) {
                Ok(ref ident) if ident.as_slice().eq_ignore_ascii_case("normal") => {
                    nb_normals += 1;
                    continue;
                }
                _ => {}
            }
            if style.is_none() {
                match font_style::from_component_value(component_value, base_url) {
                    Ok(s) => { style = Some(s); continue },
                    Err(()) => ()
                }
            }
            if weight.is_none() {
                match font_weight::from_component_value(component_value, base_url) {
                    Ok(w) => { weight = Some(w); continue },
                    Err(()) => ()
                }
            }
            if variant.is_none() {
                match font_variant::from_component_value(component_value, base_url) {
                    Ok(v) => { variant = Some(v); continue },
                    Err(()) => ()
                }
            }
            match font_size::from_component_value(component_value, base_url) {
                Ok(s) => { size = Some(s); break },
                Err(()) => return Err(())
            }
        }
        #[inline]
        fn count<T>(opt: &Option<T>) -> uint {
            match opt {
                &Some(_) => 1,
                &None => 0,
            }
        }
        if size.is_none() || (count(&style) + count(&weight) + count(&variant) + nb_normals) > 3 {
            return Err(())
        }
        let mut copied_iter = iter.clone();
        match copied_iter.next() {
            Some(&Delim('/')) => {
                iter = copied_iter;
                line_height = match iter.next() {
                    Some(v) => line_height::from_component_value(v, base_url).ok(),
                    _ => return Err(()),
                };
                if line_height.is_none() { return Err(()) }
            }
            _ => ()
        }
        let family = try!(parse_comma_separated(
            &mut BufferedIter::new(iter), font_family::parse_one_family));
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
        overflow_wrap::parse(input, base_url).map(|specified_value| {
            Longhands {
                overflow_wrap: Some(specified_value),
            }
        })
    </%self:shorthand>

    <%self:shorthand name="list-style"
                     sub_properties="list-style-image list-style-position list-style-type">
        // `none` is ambiguous until we've finished parsing the shorthands, so we count the number
        // of times we see it.
        let mut nones = 0u8;
        let (mut image, mut position, mut list_style_type, mut any) = (None, None, None, false);
        for component_value in input.skip_whitespace() {
            match component_value {
                &Ident(ref value) if value.eq_ignore_ascii_case("none") => {
                    nones = nones + 1;
                    if nones > 2 {
                        return Err(())
                    }
                    any = true;
                    continue
                }
                _ => {}
            }

            if list_style_type.is_none() {
                match list_style_type::from_component_value(component_value, base_url) {
                    Ok(v) => {
                        list_style_type = Some(v);
                        any = true;
                        continue
                    },
                    Err(()) => ()
                }
            }

            if image.is_none() {
                match list_style_image::from_component_value(component_value, base_url) {
                    Ok(v) => {
                        image = Some(v);
                        any = true;
                        continue
                    },
                    Err(()) => (),
                }
            }

            if position.is_none() {
                match list_style_position::from_component_value(component_value, base_url) {
                    Ok(v) => {
                        position = Some(v);
                        any = true;
                        continue
                    },
                    Err(()) => ()
                }
            }
        }

        // If there are two `none`s, then we can't have a type or image; if there is one `none`,
        // then we can't have both a type *and* an image; if there is no `none` then we're fine as
        // long as we parsed something.
        match (any, nones, list_style_type, image) {
            (true, 2, None, None) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(None),
                    list_style_type: Some(list_style_type::T::none),
                })
            }
            (true, 1, None, Some(image)) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(image),
                    list_style_type: Some(list_style_type::T::none),
                })
            }
            (true, 1, Some(list_style_type), None) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(None),
                    list_style_type: Some(list_style_type),
                })
            }
            (true, 1, None, None) => {
                Ok(Longhands {
                    list_style_position: position,
                    list_style_image: Some(None),
                    list_style_type: Some(list_style_type::T::none),
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
        storage: [uint, ..(${len(LONGHANDS)} - 1 + uint::BITS) / uint::BITS]
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
pub struct PropertyDeclarationBlock {
    pub important: Arc<Vec<PropertyDeclaration>>,
    pub normal: Arc<Vec<PropertyDeclaration>>,
}


pub fn parse_style_attribute(input: &str, base_url: &Url) -> PropertyDeclarationBlock {
    parse_property_declaration_list(tokenize(input), base_url)
}


pub fn parse_property_declaration_list<I: Iterator<Node>>(input: I, base_url: &Url) -> PropertyDeclarationBlock {
    let mut important_declarations = vec!();
    let mut normal_declarations = vec!();
    let mut important_seen = PropertyBitField::new();
    let mut normal_seen = PropertyBitField::new();
    let items: Vec<DeclarationListItem> =
        ErrorLoggerIterator(parse_declaration_list(input)).collect();
    for item in items.into_iter().rev() {
        match item {
            DeclarationListItem::AtRule(rule) => log_css_error(
                rule.location, format!("Unsupported at-rule in declaration list: @{}", rule.name).as_slice()),
            DeclarationListItem::Declaration(Declaration{ location: l, name: n, value: v, important: i}) => {
                // TODO: only keep the last valid declaration for a given name.
                let (list, seen) = if i {
                    (&mut important_declarations, &mut important_seen)
                } else {
                    (&mut normal_declarations, &mut normal_seen)
                };
                match PropertyDeclaration::parse(n.as_slice(), v.as_slice(), list, base_url, seen) {
                    PropertyDeclarationParseResult::UnknownProperty => log_css_error(l, format!(
                        "Unsupported property: {}:{}", n, v.to_css_string()).as_slice()),
                    PropertyDeclarationParseResult::ExperimentalProperty => log_css_error(l, format!(
                        "Experimental property, use `servo --enable_experimental` \
                         or `servo -e` to enable: {}:{}",
                        n, v.to_css_string()).as_slice()),
                    PropertyDeclarationParseResult::InvalidValue => log_css_error(l, format!(
                        "Invalid value: {}:{}", n, v.to_css_string()).as_slice()),
                    PropertyDeclarationParseResult::ValidOrIgnoredDeclaration => (),
                }
            }
        }
    }
    PropertyDeclarationBlock {
        important: Arc::new(important_declarations),
        normal: Arc::new(normal_declarations),
    }
}


pub enum CSSWideKeyword {
    InitialKeyword,
    InheritKeyword,
    UnsetKeyword,
}

impl CSSWideKeyword {
    pub fn parse(input: &[ComponentValue]) -> Result<CSSWideKeyword, ()> {
        one_component_value(input).and_then(get_ident_lower).and_then(|keyword| {
            match keyword.as_slice() {
                "initial" => Ok(CSSWideKeyword::InitialKeyword),
                "inherit" => Ok(CSSWideKeyword::InheritKeyword),
                "unset" => Ok(CSSWideKeyword::UnsetKeyword),
                _ => Err(())
            }
        })
    }
}


#[deriving(Clone, PartialEq, Eq, Copy)]
pub enum DeclaredValue<T> {
    SpecifiedValue(T),
    Initial,
    Inherit,
    // There is no Unset variant here.
    // The 'unset' keyword is represented as either Initial or Inherit,
    // depending on whether the property is inherited.
}

impl<T: Show> DeclaredValue<T> {
    pub fn specified_value(&self) -> Option<String> {
        match self {
            &DeclaredValue::SpecifiedValue(ref inner) => Some(format!("{}", inner)),
            &DeclaredValue::Initial => None,
            &DeclaredValue::Inherit => Some("inherit".into_string()),
        }
    }
}

#[deriving(Clone)]
pub enum PropertyDeclaration {
    % for property in LONGHANDS:
        ${property.camel_case}Declaration(DeclaredValue<longhands::${property.ident}::SpecifiedValue>),
    % endfor
}


#[deriving(Eq, PartialEq, Copy)]
pub enum PropertyDeclarationParseResult {
    UnknownProperty,
    ExperimentalProperty,
    InvalidValue,
    ValidOrIgnoredDeclaration,
}

impl PropertyDeclaration {
    pub fn name(&self) -> String {
        match self {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    &PropertyDeclaration::${property.camel_case}Declaration(..) => "${property.name}".into_string(),
                % endif
            % endfor
            _ => "".into_string(),
        }
    }

    pub fn value(&self) -> String {
        match self {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    &PropertyDeclaration::${property.camel_case}Declaration(ref value) =>
                        value.specified_value()
                             .unwrap_or_else(|| format!("{}", longhands::${property.ident}::get_initial_value())),
                % endif
            % endfor
            decl => panic!("unsupported property declaration: {}", decl.name()),
        }
    }

    pub fn matches(&self, name: &str) -> bool {
        let name_lower = name.as_slice().to_ascii_lower();
        match (self, name_lower.as_slice()) {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    (&PropertyDeclaration::${property.camel_case}Declaration(..), "${property.name}") => true,
                % endif
            % endfor
            _ => false,
        }
    }

    pub fn parse(name: &str, value: &[ComponentValue],
                 result_list: &mut Vec<PropertyDeclaration>,
                 base_url: &Url,
                 seen: &mut PropertyBitField) -> PropertyDeclarationParseResult {
        match name.to_ascii_lower().as_slice() {
            % for property in LONGHANDS:
                % if property.derived_from is None:
                    "${property.name}" => {
                        % if property.experimental:
                            if !::servo_util::opts::experimental_enabled() {
                                return PropertyDeclarationParseResult::ExperimentalProperty
                            }
                        % endif
                        if seen.get_${property.ident}() {
                            return PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        }
                        match longhands::${property.ident}::parse_declared(value, base_url) {
                            Ok(value) => {
                                seen.set_${property.ident}();
                                result_list.push(PropertyDeclaration::${property.camel_case}Declaration(value));
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
                    if ${" && ".join("seen.get_%s()" % sub_property.ident
                                     for sub_property in shorthand.sub_properties)} {
                        return PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                    }
                    match CSSWideKeyword::parse(value) {
                        Ok(CSSWideKeyword::InheritKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                if !seen.get_${sub_property.ident}() {
                                    seen.set_${sub_property.ident}();
                                    result_list.push(
                                        PropertyDeclaration::${sub_property.camel_case}Declaration(
                                            DeclaredValue::Inherit));
                                }
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Ok(CSSWideKeyword::InitialKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                if !seen.get_${sub_property.ident}() {
                                    seen.set_${sub_property.ident}();
                                    result_list.push(
                                        PropertyDeclaration::${sub_property.camel_case}Declaration(
                                            DeclaredValue::Initial));
                                }
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Ok(CSSWideKeyword::UnsetKeyword) => {
                            % for sub_property in shorthand.sub_properties:
                                if !seen.get_${sub_property.ident}() {
                                    seen.set_${sub_property.ident}();
                                    result_list.push(PropertyDeclaration::${sub_property.camel_case}Declaration(
                                        DeclaredValue::${"Inherit" if sub_property.style_struct.inherited else "Initial"}
                                    ));
                                }
                            % endfor
                            PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                        },
                        Err(()) => match shorthands::${shorthand.ident}::parse(value, base_url) {
                            Ok(result) => {
                                % for sub_property in shorthand.sub_properties:
                                    if !seen.get_${sub_property.ident}() {
                                        seen.set_${sub_property.ident}();
                                        result_list.push(PropertyDeclaration::${sub_property.camel_case}Declaration(
                                            match result.${sub_property.ident} {
                                                Some(value) => DeclaredValue::SpecifiedValue(value),
                                                None => DeclaredValue::Initial,
                                            }
                                        ));
                                    }
                                % endfor
                                PropertyDeclarationParseResult::ValidOrIgnoredDeclaration
                            },
                            Err(()) => PropertyDeclarationParseResult::InvalidValue,
                        }
                    }
                },
            % endfor
            _ => PropertyDeclarationParseResult::UnknownProperty,
        }
    }
}

impl Show for PropertyDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.name(), self.value())
    }
}


pub mod style_structs {
    use super::longhands;

    % for style_struct in STYLE_STRUCTS:
        #[allow(missing_copy_implementations)]
        #[deriving(PartialEq, Clone)]
        pub struct ${style_struct.name} {
            % for longhand in style_struct.longhands:
                pub ${longhand.ident}: longhands::${longhand.ident}::computed_value::T,
            % endfor
        }
    % endfor
}

#[deriving(Clone)]
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
    pub fn resolve_color(&self, color: computed::CSSColor) -> RGBA {
        match color {
            Color::RGBA(rgba) => rgba,
            Color::CurrentColor => self.get_color().color,
        }
    }

    #[inline]
    pub fn content_inline_size(&self) -> computed_values::LengthOrPercentageOrAuto {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.height } else { box_style.width }
    }

    #[inline]
    pub fn content_block_size(&self) -> computed_values::LengthOrPercentageOrAuto {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.width } else { box_style.height }
    }

    #[inline]
    pub fn min_inline_size(&self) -> computed_values::LengthOrPercentage {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.min_height } else { box_style.min_width }
    }

    #[inline]
    pub fn min_block_size(&self) -> computed_values::LengthOrPercentage {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.min_width } else { box_style.min_height }
    }

    #[inline]
    pub fn max_inline_size(&self) -> computed_values::LengthOrPercentageOrNone {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.max_height } else { box_style.max_width }
    }

    #[inline]
    pub fn max_block_size(&self) -> computed_values::LengthOrPercentageOrNone {
        let box_style = self.get_box();
        if self.writing_mode.is_vertical() { box_style.max_width } else { box_style.max_height }
    }

    #[inline]
    pub fn logical_padding(&self) -> LogicalMargin<computed_values::LengthOrPercentage> {
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
    pub fn logical_margin(&self) -> LogicalMargin<computed_values::LengthOrPercentageOrAuto> {
        let margin_style = self.get_margin();
        LogicalMargin::from_physical(self.writing_mode, SideOffsets2D::new(
            margin_style.margin_top,
            margin_style.margin_right,
            margin_style.margin_bottom,
            margin_style.margin_left,
        ))
    }

    #[inline]
    pub fn logical_position(&self) -> LogicalMargin<computed_values::LengthOrPercentageOrAuto> {
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
    use servo_util::logical_geometry;
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
                            PropertyDeclaration::${property.camel_case}Declaration(ref ${'_' if not style_struct.inherited else ''}declared_value) => {
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
                            PropertyDeclaration::${property.camel_case}Declaration(_) => {
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
    )

    // Initialize `context`
    // Declarations blocks are already stored in increasing precedence order.
    for sub_list in applicable_declarations.iter() {
        // Declarations are stored in reverse source order, we want them in forward order here.
        for declaration in sub_list.declarations.iter().rev() {
            match *declaration {
                PropertyDeclaration::FontSizeDeclaration(ref value) => {
                    context.font_size = match *value {
                        DeclaredValue::SpecifiedValue(specified_value) => computed::compute_Au_with_font_size(
                            specified_value, context.inherited_font_size, context.root_font_size),
                        DeclaredValue::Initial => longhands::font_size::get_initial_value(),
                        DeclaredValue::Inherit => context.inherited_font_size,
                    }
                }
                PropertyDeclaration::ColorDeclaration(ref value) => {
                    context.color = match *value {
                        DeclaredValue::SpecifiedValue(ref specified_value) => specified_value.parsed,
                        DeclaredValue::Initial => longhands::color::get_initial_value(),
                        DeclaredValue::Inherit => inherited_style.get_color().color.clone(),
                    };
                }
                PropertyDeclaration::DisplayDeclaration(ref value) => {
                    context.display = get_specified!(get_box, display, value);
                }
                PropertyDeclaration::PositionDeclaration(ref value) => {
                    context.positioned = match get_specified!(get_box, position, value) {
                        longhands::position::T::absolute | longhands::position::T::fixed => true,
                        _ => false,
                    }
                }
                PropertyDeclaration::FloatDeclaration(ref value) => {
                    context.floated = get_specified!(get_box, float, value)
                                      != longhands::float::T::none;
                }
                PropertyDeclaration::TextDecorationDeclaration(ref value) => {
                    context.text_decoration = get_specified!(get_text, text_decoration, value);
                }
                % for side in ["top", "right", "bottom", "left"]:
                    PropertyDeclaration::Border${side.capitalize()}StyleDeclaration(ref value) => {
                        context.border_${side}_present =
                        match get_specified!(get_border, border_${side}_style, value) {
                            longhands::border_top_style::T::none |
                            longhands::border_top_style::T::hidden => false,
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
                            PropertyDeclaration::${property.camel_case}Declaration(ref declared_value) => {
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
                            PropertyDeclaration::${property.camel_case}Declaration(_) => {
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
        % for property in SHORTHANDS:
            "${property.name}" => true,
        % endfor
        % for property in LONGHANDS:
            "${property.name}" => true,
        % endfor
        _ => false,
    }
}

pub fn longhands_from_shorthand(shorthand: &str) -> Option<Vec<String>> {
    match shorthand {
        % for property in SHORTHANDS:
            "${property.name}" => Some(vec!(
            % for sub in property.sub_properties:
                "${sub.name}".into_string(),
            % endfor
            )),
        % endfor
        _ => None,
    }
}

// Only re-export the types for computed values.
pub mod computed_values {
    % for property in LONGHANDS:
        pub use super::longhands::${property.ident}::computed_value as ${property.ident};
    % endfor
    // Don't use a side-specific name needlessly:
    pub use super::longhands::border_top_style::computed_value as border_style;

    pub use cssparser::RGBA;
    pub use super::common_types::computed::{
        LengthOrPercentage,
        LengthOrPercentageOrAuto,
        LengthOrPercentageOrNone};
}
